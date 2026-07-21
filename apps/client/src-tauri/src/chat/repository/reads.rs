use crate::chat::models::{
    ChatActivityId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatThreadShellRead,
    ChatThreadState, ChatTimelineItemRead, ChatTimelinePageRead, ChatTurnId, ChatTurnState,
    ChatWorkspaceId, InteractionMode, ModelId, ModelOptionSelection, ProviderFamilyId,
    ProviderInstanceId, ProviderThreadId, SafetyMode, TurnModeSnapshot, UtcTimestamp,
    VersionedJson,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

const MAX_PAGE_SIZE: u32 = 200;
const MAX_SEARCH_LENGTH: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatProjectShellRead {
    pub project_id: Option<String>,
    pub workspace_id: ChatWorkspaceId,
    pub workspace_name: String,
    pub workspace_archived_at: Option<UtcTimestamp>,
    pub active_thread_count: u64,
    pub archived_thread_count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredModelSelection {
    #[serde(default)]
    model_id: Option<ModelId>,
    #[serde(default)]
    #[serde(alias = "options")]
    model_options: Vec<ModelOptionSelection>,
}

pub async fn read_project_shells(pool: &SqlitePool) -> ChatResult<Vec<ChatProjectShellRead>> {
    let rows = sqlx::query(
        "SELECT w.project_id, w.id, w.display_name, w.archived_at,
                SUM(CASE WHEN t.id IS NOT NULL AND t.archived_at IS NULL AND t.state != 'closed' THEN 1 ELSE 0 END) AS active_count,
                SUM(CASE WHEN t.archived_at IS NOT NULL THEN 1 ELSE 0 END) AS archived_count
         FROM chat_workspaces w
         LEFT JOIN chat_threads t ON t.workspace_id = w.id
         GROUP BY w.id
         ORDER BY w.project_id IS NULL, w.project_id, w.display_name COLLATE NOCASE, w.id",
    )
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(ChatProjectShellRead {
                project_id: row.try_get("project_id").map_err(persistence_error)?,
                workspace_id: id(row.try_get("id").map_err(persistence_error)?)?,
                workspace_name: row.try_get("display_name").map_err(persistence_error)?,
                workspace_archived_at: timestamp(
                    row.try_get("archived_at").map_err(persistence_error)?,
                )?,
                active_thread_count: unsigned(
                    row.try_get("active_count").map_err(persistence_error)?,
                )?,
                archived_thread_count: unsigned(
                    row.try_get("archived_count").map_err(persistence_error)?,
                )?,
            })
        })
        .collect()
}

pub async fn read_thread_shells(
    pool: &SqlitePool,
    workspace_id: Option<&ChatWorkspaceId>,
    archived: bool,
) -> ChatResult<Vec<ChatThreadShellRead>> {
    let rows = sqlx::query(
        "SELECT id, workspace_id, project_id, title, provider_family_id,
                provider_instance_id, provider_thread_id, model_selection_data,
                safety_mode, interaction_mode, state, latest_turn_state,
                latest_preview, message_count, revision, last_event_sequence,
                last_activity_at, unread_at, archived_at
         FROM chat_threads
         WHERE (? IS NULL OR workspace_id = ?)
           AND ((? = 1 AND archived_at IS NOT NULL) OR (? = 0 AND archived_at IS NULL AND state != 'closed'))
         ORDER BY CASE WHEN archived_at IS NULL THEN last_activity_at ELSE archived_at END DESC, id",
    )
    .bind(workspace_id.map(ChatWorkspaceId::as_str))
    .bind(workspace_id.map(ChatWorkspaceId::as_str))
    .bind(archived)
    .bind(archived)
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter().map(row_to_thread_shell).collect()
}

pub async fn read_thread_shell(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<ChatThreadShellRead> {
    sqlx::query(
        "SELECT id, workspace_id, project_id, title, provider_family_id,
                provider_instance_id, provider_thread_id, model_selection_data,
                safety_mode, interaction_mode, state, latest_turn_state,
                latest_preview, message_count, revision, last_event_sequence,
                last_activity_at, unread_at, archived_at
         FROM chat_threads WHERE id = ?",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .map(row_to_thread_shell)
    .transpose()?
    .ok_or_else(not_found)
}

pub async fn search_thread_titles(
    pool: &SqlitePool,
    query: &str,
    archived: Option<bool>,
    limit: u32,
) -> ChatResult<Vec<ChatThreadShellRead>> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() || normalized.len() > MAX_SEARCH_LENGTH {
        return Err(ChatError::validation(
            "query",
            "Chat title search query is invalid",
        ));
    }
    let pattern = format!("%{}%", escape_like(&normalized));
    let rows = sqlx::query(
        "SELECT id, workspace_id, project_id, title, provider_family_id,
                provider_instance_id, provider_thread_id, model_selection_data,
                safety_mode, interaction_mode, state, latest_turn_state,
                latest_preview, message_count, revision, last_event_sequence,
                last_activity_at, unread_at, archived_at
         FROM chat_threads
         WHERE title_search LIKE ? ESCAPE '\\'
           AND (? IS NULL OR (? = 1 AND archived_at IS NOT NULL) OR (? = 0 AND archived_at IS NULL AND state != 'closed'))
         ORDER BY CASE WHEN title_search = ? THEN 0 WHEN title_search LIKE ? ESCAPE '\\' THEN 1 ELSE 2 END,
                  last_activity_at DESC, id
         LIMIT ?",
    )
    .bind(pattern)
    .bind(archived)
    .bind(archived)
    .bind(archived)
    .bind(&normalized)
    .bind(format!("{}%", escape_like(&normalized)))
    .bind(i64::from(limit.clamp(1, MAX_PAGE_SIZE)))
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter().map(row_to_thread_shell).collect()
}

pub async fn read_timeline_page(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    anchor_sequence: Option<u64>,
    limit: u32,
) -> ChatResult<ChatTimelinePageRead> {
    let thread = sqlx::query("SELECT revision, last_event_sequence FROM chat_threads WHERE id = ?")
        .bind(thread_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .ok_or_else(not_found)?;
    let revision = unsigned(thread.try_get("revision").map_err(persistence_error)?)?;
    let last_sequence = unsigned(
        thread
            .try_get("last_event_sequence")
            .map_err(persistence_error)?,
    )?;
    let anchor = anchor_sequence.unwrap_or(last_sequence).min(last_sequence);
    let rows = sqlx::query(
        "SELECT row_id, turn_id, sequence_anchor, item_kind, schema_version, item_data
         FROM (
           SELECT id AS row_id, turn_id, sequence_anchor, 'message' AS item_kind,
                  content_metadata_schema_version AS schema_version,
                  json_object('role', role, 'markdown', normalized_markdown,
                              'streamingState', streaming_state,
                              'providerItemId', provider_item_id,
                              'metadata', json(content_metadata_data)) AS item_data
           FROM chat_messages WHERE thread_id = ? AND sequence_anchor <= ?
           UNION ALL
           SELECT id, turn_id, sequence_anchor, 'activity', safe_metadata_schema_version,
                  json_object('activityKind', item_kind, 'status', status, 'title', title,
                              'detail', detail, 'providerItemId', provider_item_id,
                              'metadata', json(safe_metadata_data))
           FROM chat_activities WHERE thread_id = ? AND sequence_anchor <= ?
           UNION ALL
           SELECT id, origin_turn_id, sequence_anchor, 'plan', steps_schema_version,
                  json_object('markdown', markdown, 'steps', json(steps_data), 'state', state)
           FROM chat_plans WHERE thread_id = ? AND sequence_anchor <= ?
         )
         ORDER BY sequence_anchor DESC, row_id DESC
         LIMIT ?",
    )
    .bind(thread_id.as_str())
    .bind(i64_value(anchor)?)
    .bind(thread_id.as_str())
    .bind(i64_value(anchor)?)
    .bind(thread_id.as_str())
    .bind(i64_value(anchor)?)
    .bind(i64::from(limit.clamp(1, MAX_PAGE_SIZE)))
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let mut items = rows
        .into_iter()
        .map(|row| {
            let row_id: String = row.try_get("row_id").map_err(persistence_error)?;
            let data: String = row.try_get("item_data").map_err(persistence_error)?;
            Ok(ChatTimelineItemRead {
                activity_id: ChatActivityId::new(row_id).map_err(|_| corrupt_data())?,
                turn_id: row
                    .try_get::<Option<String>, _>("turn_id")
                    .map_err(persistence_error)?
                    .map(ChatTurnId::new)
                    .transpose()
                    .map_err(|_| corrupt_data())?,
                sequence_anchor: unsigned(
                    row.try_get("sequence_anchor").map_err(persistence_error)?,
                )?,
                kind: row.try_get("item_kind").map_err(persistence_error)?,
                data: VersionedJson {
                    schema_version: u32::try_from(
                        row.try_get::<i64, _>("schema_version")
                            .map_err(persistence_error)?,
                    )
                    .map_err(|_| corrupt_data())?,
                    value: serde_json::from_str(&data).map_err(serialization_error)?,
                },
            })
        })
        .collect::<ChatResult<Vec<_>>>()?;
    items.reverse();
    let first = items.first().map(|item| item.sequence_anchor);
    let last = items.last().map(|item| item.sequence_anchor);
    Ok(ChatTimelinePageRead {
        thread_id: thread_id.clone(),
        previous_cursor: first
            .filter(|value| *value > 1)
            .map(|value| (value - 1).to_string()),
        next_cursor: last
            .filter(|value| *value < last_sequence)
            .map(|value| (value + 1).to_string()),
        items,
        thread_revision: revision,
    })
}

fn row_to_thread_shell(row: sqlx::sqlite::SqliteRow) -> ChatResult<ChatThreadShellRead> {
    let model_data: String = row
        .try_get("model_selection_data")
        .map_err(persistence_error)?;
    let model: StoredModelSelection =
        serde_json::from_str(&model_data).map_err(serialization_error)?;
    Ok(ChatThreadShellRead {
        id: ChatThreadId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
            .map_err(|_| corrupt_data())?,
        workspace_id: id(row
            .try_get::<String, _>("workspace_id")
            .map_err(persistence_error)?)?,
        project_id: row.try_get("project_id").map_err(persistence_error)?,
        title: row.try_get("title").map_err(persistence_error)?,
        provider_family_id: ProviderFamilyId::new(
            row.try_get::<String, _>("provider_family_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        provider_instance_id: ProviderInstanceId::new(
            row.try_get::<String, _>("provider_instance_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        provider_thread_id: row
            .try_get::<Option<String>, _>("provider_thread_id")
            .map_err(persistence_error)?
            .map(ProviderThreadId::new)
            .transpose()
            .map_err(|_| corrupt_data())?,
        model_id: model.model_id,
        model_options: model.model_options,
        modes: TurnModeSnapshot {
            safety_mode: parse_safety(
                &row.try_get::<String, _>("safety_mode")
                    .map_err(persistence_error)?,
            )?,
            interaction_mode: parse_interaction(
                &row.try_get::<String, _>("interaction_mode")
                    .map_err(persistence_error)?,
            )?,
        },
        state: parse_thread_state(
            &row.try_get::<String, _>("state")
                .map_err(persistence_error)?,
        )?,
        latest_turn_state: row
            .try_get::<Option<String>, _>("latest_turn_state")
            .map_err(persistence_error)?
            .map(|value| parse_turn_state(&value))
            .transpose()?,
        latest_preview: row.try_get("latest_preview").map_err(persistence_error)?,
        message_count: unsigned(row.try_get("message_count").map_err(persistence_error)?)?,
        revision: unsigned(row.try_get("revision").map_err(persistence_error)?)?,
        last_event_sequence: unsigned(
            row.try_get("last_event_sequence")
                .map_err(persistence_error)?,
        )?,
        last_activity_at: UtcTimestamp::new(
            row.try_get::<String, _>("last_activity_at")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        unread_at: timestamp(row.try_get("unread_at").map_err(persistence_error)?)?,
        archived_at: timestamp(row.try_get("archived_at").map_err(persistence_error)?)?,
    })
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
fn id(value: String) -> ChatResult<ChatWorkspaceId> {
    ChatWorkspaceId::new(value).map_err(|_| corrupt_data())
}
fn timestamp(value: Option<String>) -> ChatResult<Option<UtcTimestamp>> {
    value
        .map(UtcTimestamp::new)
        .transpose()
        .map_err(|_| corrupt_data())
}
fn unsigned(value: i64) -> ChatResult<u64> {
    u64::try_from(value).map_err(|_| corrupt_data())
}
fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value)
        .map_err(|_| ChatError::validation("sequence", "Chat sequence is too large"))
}
fn parse_safety(value: &str) -> ChatResult<SafetyMode> {
    match value {
        "supervised" => Ok(SafetyMode::Supervised),
        "auto_accept_edits" => Ok(SafetyMode::AutoAcceptEdits),
        "full_access" => Ok(SafetyMode::FullAccess),
        _ => Err(corrupt_data()),
    }
}
fn parse_interaction(value: &str) -> ChatResult<InteractionMode> {
    match value {
        "build" => Ok(InteractionMode::Build),
        "plan" => Ok(InteractionMode::Plan),
        _ => Err(corrupt_data()),
    }
}
fn parse_thread_state(value: &str) -> ChatResult<ChatThreadState> {
    match value {
        "draft" => Ok(ChatThreadState::Draft),
        "active" => Ok(ChatThreadState::Active),
        "waiting" => Ok(ChatThreadState::Waiting),
        "idle" => Ok(ChatThreadState::Idle),
        "error" => Ok(ChatThreadState::Error),
        "archived" => Ok(ChatThreadState::Archived),
        "closed" => Ok(ChatThreadState::Closed),
        _ => Err(corrupt_data()),
    }
}
fn parse_turn_state(value: &str) -> ChatResult<ChatTurnState> {
    match value {
        "pending" => Ok(ChatTurnState::Pending),
        "dispatching" => Ok(ChatTurnState::Dispatching),
        "active" => Ok(ChatTurnState::Active),
        "waiting_for_approval" => Ok(ChatTurnState::WaitingForApproval),
        "waiting_for_user_input" => Ok(ChatTurnState::WaitingForUserInput),
        "completed" => Ok(ChatTurnState::Completed),
        "interrupted" => Ok(ChatTurnState::Interrupted),
        "failed" => Ok(ChatTurnState::Failed),
        _ => Err(corrupt_data()),
    }
}
fn not_found() -> ChatError {
    ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true)
}
fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat read persistence failed",
        true,
    )
}
fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat JSON is invalid",
        false,
    )
}
fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat record is invalid",
        false,
    )
}
