//! Durable project channel navigation above provider execution sessions.

use super::models::{
    ChatActivityId, ChatChannelId, ChatChannelRead, ChatChannelSessionRead, ChatChannelTargetRead,
    ChatChannelTimelinePageRead, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ModelId,
    ModelOptionSelection, ProjectWorkingFolderId, ProviderInstanceId, UtcTimestamp, VersionedJson,
};
use super::repository::reads;
use crate::db_path;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::{BTreeMap, BTreeSet};

const MAX_CHANNEL_NAME_CHARS: usize = 80;
const MAX_CHANNEL_TOPIC_CHARS: usize = 250;
const MAX_CHANNEL_SEARCH_CHARS: usize = 240;
const MAX_CHANNEL_RESULTS: u32 = 500;
const MAX_TIMELINE_PAGE_SIZE: u32 = 200;
const CHANNEL_SEQUENCE_STRIDE: u64 = 1_000_000_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChatChannelTimelineCursor {
    session_ordinal: u64,
    sequence: u64,
    row_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelTargetInput {
    pub working_folder_id: Option<ProjectWorkingFolderId>,
    pub provider_instance_id: Option<ProviderInstanceId>,
    #[serde(default)]
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    #[serde(default)]
    pub model_options: Vec<ModelOptionSelection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatChannelCommand {
    pub id: ChatChannelId,
    pub project_id: String,
    pub name: String,
    #[serde(default)]
    pub topic: String,
    pub target: ChatChannelTargetInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatChannelDetailsCommand {
    pub channel_id: ChatChannelId,
    pub name: String,
    #[serde(default)]
    pub topic: String,
    pub expected_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatChannelTargetCommand {
    pub channel_id: ChatChannelId,
    pub target: ChatChannelTargetInput,
    pub expected_revision: u64,
}

#[tauri::command]
pub async fn chat_list_channels(
    app: tauri::AppHandle,
    db_url: String,
    project_id: String,
    archived: bool,
) -> ChatResult<Vec<ChatChannelRead>> {
    validate_project_id(&project_id)?;
    read_channels(&chat_pool(app, db_url).await?, &project_id, archived, None).await
}

#[tauri::command]
pub async fn chat_list_navigation_channels(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<Vec<ChatChannelRead>> {
    read_navigation_channels(&chat_pool(app, db_url).await?).await
}

#[tauri::command]
pub async fn chat_search_channels(
    app: tauri::AppHandle,
    db_url: String,
    project_id: String,
    query: String,
    archived: bool,
    limit: u32,
) -> ChatResult<Vec<ChatChannelRead>> {
    validate_project_id(&project_id)?;
    let query = query.trim().to_lowercase();
    if query.is_empty() || query.chars().count() > MAX_CHANNEL_SEARCH_CHARS {
        return Err(ChatError::validation(
            "query",
            "Channel search query is invalid",
        ));
    }
    read_channels(
        &chat_pool(app, db_url).await?,
        &project_id,
        archived,
        Some((&query, limit.clamp(1, MAX_CHANNEL_RESULTS))),
    )
    .await
}

#[tauri::command]
pub async fn chat_read_channel(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
) -> ChatResult<ChatChannelRead> {
    read_channel(&chat_pool(app, db_url).await?, &channel_id).await
}

#[tauri::command]
pub async fn chat_create_channel(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatChannelCommand,
) -> ChatResult<ChatChannelRead> {
    validate_project_id(&request.project_id)?;
    let name = normalized_channel_name(&request.name)?;
    let topic = normalized_topic(&request.topic)?;
    let pool = chat_pool(app, db_url).await?;
    validate_target(&pool, &request.project_id, &request.target).await?;
    require_unique_name(&pool, &request.project_id, &name, None).await?;
    let now = now_timestamp()?;
    let model = stored_model(&request.target)?;
    sqlx::query(
        "INSERT INTO chat_channels (
            id, project_id, name, topic, is_default, working_folder_id,
            provider_instance_id, model_selection_schema_version,
            model_selection_data, created_at, updated_at
         ) VALUES (?, ?, ?, ?, 0, ?, ?, 1, ?, ?, ?)",
    )
    .bind(request.id.as_str())
    .bind(&request.project_id)
    .bind(name)
    .bind(topic)
    .bind(
        request
            .target
            .working_folder_id
            .as_ref()
            .map(ProjectWorkingFolderId::as_str),
    )
    .bind(
        request
            .target
            .provider_instance_id
            .as_ref()
            .map(ProviderInstanceId::as_str),
    )
    .bind(model)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    read_channel(&pool, &request.id).await
}

#[tauri::command]
pub async fn chat_update_channel_details(
    app: tauri::AppHandle,
    db_url: String,
    request: UpdateChatChannelDetailsCommand,
) -> ChatResult<ChatChannelRead> {
    let pool = chat_pool(app, db_url).await?;
    let current = read_channel(&pool, &request.channel_id).await?;
    let name = normalized_channel_name(&request.name)?;
    if current.is_default && name != "general" {
        return Err(ChatError::validation(
            "name",
            "The general channel name is fixed",
        ));
    }
    require_unique_name(&pool, &current.project_id, &name, Some(&request.channel_id)).await?;
    let topic = normalized_topic(&request.topic)?;
    let updated = sqlx::query(
        "UPDATE chat_channels
         SET name = ?, topic = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(name)
    .bind(topic)
    .bind(now_timestamp()?.as_str())
    .bind(request.channel_id.as_str())
    .bind(i64_value(request.expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    require_updated(&pool, &request.channel_id, updated.rows_affected()).await?;
    read_channel(&pool, &request.channel_id).await
}

#[tauri::command]
pub async fn chat_update_channel_target(
    app: tauri::AppHandle,
    db_url: String,
    request: UpdateChatChannelTargetCommand,
) -> ChatResult<ChatChannelRead> {
    let pool = chat_pool(app, db_url).await?;
    let current = read_channel(&pool, &request.channel_id).await?;
    if current.current_thread.as_ref().is_some_and(thread_is_busy) {
        return Err(ChatError::new(
            ChatErrorCode::Busy,
            "Finish or stop the current agent run before changing its target",
            true,
        ));
    }
    validate_target(&pool, &current.project_id, &request.target).await?;
    let updated = sqlx::query(
        "UPDATE chat_channels
         SET working_folder_id = ?, provider_instance_id = ?,
             model_selection_schema_version = 1, model_selection_data = ?,
             revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(
        request
            .target
            .working_folder_id
            .as_ref()
            .map(ProjectWorkingFolderId::as_str),
    )
    .bind(
        request
            .target
            .provider_instance_id
            .as_ref()
            .map(ProviderInstanceId::as_str),
    )
    .bind(stored_model(&request.target)?)
    .bind(now_timestamp()?.as_str())
    .bind(request.channel_id.as_str())
    .bind(i64_value(request.expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    require_updated(&pool, &request.channel_id, updated.rows_affected()).await?;
    read_channel(&pool, &request.channel_id).await
}

#[tauri::command]
pub async fn chat_archive_channel(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    expected_revision: u64,
) -> ChatResult<ChatChannelRead> {
    set_channel_archived(app, db_url, channel_id, expected_revision, true).await
}

#[tauri::command]
pub async fn chat_restore_channel(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    expected_revision: u64,
) -> ChatResult<ChatChannelRead> {
    set_channel_archived(app, db_url, channel_id, expected_revision, false).await
}

#[tauri::command]
pub async fn chat_set_channel_read(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    read: bool,
) -> ChatResult<ChatChannelRead> {
    let pool = chat_pool(app, db_url).await?;
    read_channel(&pool, &channel_id).await?;
    let now = now_timestamp()?;
    if read {
        sqlx::query(
            "UPDATE chat_threads
             SET read_revision = revision + 1, unread_at = NULL,
                 revision = revision + 1, updated_at = ?
             WHERE id IN (
                 SELECT thread_id FROM chat_channel_sessions WHERE channel_id = ?
             )",
        )
        .bind(now.as_str())
        .bind(channel_id.as_str())
        .execute(&pool)
        .await
        .map_err(persistence_error)?;
    } else {
        sqlx::query(
            "UPDATE chat_threads SET unread_at = ?, revision = revision + 1, updated_at = ?
             WHERE id = (
                 SELECT thread_id FROM chat_channel_sessions
                 WHERE channel_id = ? AND is_current = 1
             )",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(channel_id.as_str())
        .execute(&pool)
        .await
        .map_err(persistence_error)?;
    }
    read_channel(&pool, &channel_id).await
}

#[tauri::command]
pub async fn chat_list_channel_sessions(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
) -> ChatResult<Vec<ChatChannelSessionRead>> {
    let pool = chat_pool(app, db_url).await?;
    read_channel(&pool, &channel_id).await?;
    let rows = sqlx::query(
        "SELECT thread_id, ordinal, is_current, created_at
         FROM chat_channel_sessions WHERE channel_id = ?
         ORDER BY ordinal, thread_id",
    )
    .bind(channel_id.as_str())
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let thread_ids = rows
        .iter()
        .map(|row| {
            row.try_get::<String, _>("thread_id")
                .map_err(persistence_error)
        })
        .collect::<ChatResult<BTreeSet<_>>>()?;
    let threads = reads::read_thread_shells_by_ids(&pool, &thread_ids)
        .await?
        .into_iter()
        .map(|thread| (thread.id.as_str().to_string(), thread))
        .collect::<BTreeMap<_, _>>();
    let mut sessions = Vec::with_capacity(rows.len());
    for row in rows {
        let thread_id: String = row.try_get("thread_id").map_err(persistence_error)?;
        let thread = threads.get(&thread_id).cloned().ok_or_else(corrupt_data)?;
        sessions.push(ChatChannelSessionRead {
            channel_id: channel_id.clone(),
            ordinal: unsigned(row.try_get("ordinal").map_err(persistence_error)?)?,
            is_current: row
                .try_get::<i64, _>("is_current")
                .map_err(persistence_error)?
                == 1,
            created_at: required_timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
            thread,
        });
    }
    Ok(sessions)
}

#[tauri::command]
pub async fn chat_read_channel_timeline_page(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    cursor: Option<String>,
    limit: u32,
) -> ChatResult<ChatChannelTimelinePageRead> {
    let cursor = cursor
        .as_deref()
        .map(parse_channel_timeline_cursor)
        .transpose()?;
    read_channel_timeline_page(
        &chat_pool(app, db_url).await?,
        &channel_id,
        cursor.as_ref(),
        limit,
    )
    .await
}

pub(crate) async fn read_channel_timeline_page(
    pool: &SqlitePool,
    channel_id: &ChatChannelId,
    cursor: Option<&ChatChannelTimelineCursor>,
    limit: u32,
) -> ChatResult<ChatChannelTimelinePageRead> {
    let revision: i64 = sqlx::query_scalar(
        "SELECT c.revision + COALESCE((
             SELECT SUM(t.revision)
             FROM chat_channel_sessions s
             JOIN chat_threads t ON t.id = s.thread_id
             WHERE s.channel_id = c.id
         ), 0)
         FROM chat_channels c WHERE c.id = ?",
    )
    .bind(channel_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(channel_not_found)?;
    let page_limit = limit.clamp(1, MAX_TIMELINE_PAGE_SIZE);
    let initial_page = cursor.is_none();
    let anchor_ordinal = cursor
        .map(|value| i64_value(value.session_ordinal))
        .transpose()?
        .unwrap_or(i64::MAX);
    let anchor_sequence = cursor
        .map(|value| i64_value(value.sequence))
        .transpose()?
        .unwrap_or(i64::MAX);
    let anchor_row_id = cursor.map(|value| value.row_id.as_str()).unwrap_or("");
    let mut rows = sqlx::query(
        "SELECT source_thread_id, session_ordinal, session_is_current,
                session_created_at, row_id, turn_id, sequence_anchor,
                item_kind, schema_version, item_data
         FROM (
           SELECT s.thread_id AS source_thread_id, s.ordinal AS session_ordinal,
                  s.is_current AS session_is_current, s.created_at AS session_created_at,
                  m.id AS row_id, m.turn_id, m.sequence_anchor,
                  'message' AS item_kind, m.content_metadata_schema_version AS schema_version,
                  json_object('role', m.role, 'markdown', m.normalized_markdown,
                              'streamingState', m.streaming_state,
                              'providerItemId', m.provider_item_id,
                              'metadata', json(m.content_metadata_data),
                              'createdAt', m.created_at, 'updatedAt', m.updated_at) AS item_data
           FROM chat_channel_sessions s
           JOIN chat_messages m ON m.thread_id = s.thread_id
           WHERE s.channel_id = ? AND (m.turn_id IS NULL OR EXISTS (
             SELECT 1 FROM chat_turns t WHERE t.id = m.turn_id AND t.invalidated_at IS NULL
           ))
           UNION ALL
           SELECT s.thread_id, s.ordinal, s.is_current, s.created_at,
                  a.id, a.turn_id, a.sequence_anchor, 'activity',
                  a.safe_metadata_schema_version,
                  json_object('activityKind', a.item_kind, 'status', a.status,
                              'title', a.title, 'detail', a.detail,
                              'providerItemId', a.provider_item_id,
                              'metadata', json(a.safe_metadata_data),
                              'createdAt', a.created_at, 'updatedAt', a.updated_at)
           FROM chat_channel_sessions s
           JOIN chat_activities a ON a.thread_id = s.thread_id
           WHERE s.channel_id = ? AND (a.turn_id IS NULL OR EXISTS (
             SELECT 1 FROM chat_turns t WHERE t.id = a.turn_id AND t.invalidated_at IS NULL
           ))
           UNION ALL
           SELECT s.thread_id, s.ordinal, s.is_current, s.created_at,
                  p.id, p.origin_turn_id, p.sequence_anchor, 'plan', p.steps_schema_version,
                  json_object('markdown', p.markdown, 'steps', json(p.steps_data),
                              'state', p.state, 'createdAt', p.created_at,
                              'updatedAt', p.updated_at)
           FROM chat_channel_sessions s
           JOIN chat_plans p ON p.thread_id = s.thread_id
           WHERE s.channel_id = ? AND (p.origin_turn_id IS NULL OR EXISTS (
             SELECT 1 FROM chat_turns t
             WHERE t.id = p.origin_turn_id AND t.invalidated_at IS NULL
           ))
           UNION ALL
           SELECT s.thread_id, s.ordinal, s.is_current, s.created_at,
                  'channel-session:' || s.thread_id, NULL, 0, 'activity', 1,
                  json_object('activityKind', 'channel_session_boundary',
                              'status', 'completed', 'title', CAST(s.ordinal AS TEXT),
                              'detail', NULL, 'providerItemId', NULL,
                              'metadata', json_object(
                                'providerInstanceId', t.provider_instance_id,
                                'modelId', json_extract(t.model_selection_data, '$.modelId'),
                                'workingFolderId', t.working_folder_id
                              ),
                              'createdAt', s.created_at, 'updatedAt', s.created_at)
           FROM chat_channel_sessions s
           JOIN chat_threads t ON t.id = s.thread_id
           WHERE s.channel_id = ?
         )
         WHERE ? = 1
            OR session_ordinal < ?
            OR (session_ordinal = ? AND sequence_anchor < ?)
            OR (session_ordinal = ? AND sequence_anchor = ? AND row_id < ?)
         ORDER BY session_ordinal DESC, sequence_anchor DESC, row_id DESC
         LIMIT COALESCE(?, -1)",
    )
    .bind(channel_id.as_str())
    .bind(channel_id.as_str())
    .bind(channel_id.as_str())
    .bind(channel_id.as_str())
    .bind(initial_page)
    .bind(anchor_ordinal)
    .bind(anchor_ordinal)
    .bind(anchor_sequence)
    .bind(anchor_ordinal)
    .bind(anchor_sequence)
    .bind(anchor_row_id)
    .bind(i64::from(page_limit) + 1)
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let has_older = rows.len() > page_limit as usize;
    rows.truncate(page_limit as usize);
    let previous_cursor = if has_older {
        rows.last()
            .map(|row| {
                Ok(ChatChannelTimelineCursor {
                    session_ordinal: unsigned(
                        row.try_get("session_ordinal").map_err(persistence_error)?,
                    )?,
                    sequence: unsigned(row.try_get("sequence_anchor").map_err(persistence_error)?)?,
                    row_id: row.try_get("row_id").map_err(persistence_error)?,
                })
            })
            .transpose()?
            .map(|value| serde_json::to_string(&value).map_err(serialization_error))
            .transpose()?
    } else {
        None
    };
    let mut session_metadata = BTreeMap::<String, (u64, bool, UtcTimestamp)>::new();
    let mut turn_ids = BTreeSet::new();
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let source_thread_id: String =
            row.try_get("source_thread_id").map_err(persistence_error)?;
        let session_ordinal = unsigned(row.try_get("session_ordinal").map_err(persistence_error)?)?;
        session_metadata.entry(source_thread_id.clone()).or_insert((
            session_ordinal,
            row.try_get::<i64, _>("session_is_current")
                .map_err(persistence_error)?
                == 1,
            required_timestamp(
                row.try_get("session_created_at")
                    .map_err(persistence_error)?,
            )?,
        ));
        let row_id: String = row.try_get("row_id").map_err(persistence_error)?;
        let turn_id = row
            .try_get::<Option<String>, _>("turn_id")
            .map_err(persistence_error)?
            .map(super::models::ChatTurnId::new)
            .transpose()
            .map_err(|_| corrupt_data())?;
        if let Some(turn_id) = turn_id.as_ref() {
            turn_ids.insert(turn_id.as_str().to_string());
        }
        let local_sequence = unsigned(row.try_get("sequence_anchor").map_err(persistence_error)?)?;
        let sequence_anchor = session_ordinal
            .checked_mul(CHANNEL_SEQUENCE_STRIDE)
            .and_then(|offset| offset.checked_add(local_sequence))
            .filter(|value| *value <= 9_007_199_254_740_991)
            .ok_or_else(corrupt_data)?;
        let data: String = row.try_get("item_data").map_err(persistence_error)?;
        items.push(super::models::ChatTimelineItemRead {
            activity_id: ChatActivityId::new(row_id).map_err(|_| corrupt_data())?,
            turn_id,
            sequence_anchor,
            kind: row.try_get("item_kind").map_err(persistence_error)?,
            data: VersionedJson {
                schema_version: u32::try_from(
                    row.try_get::<i64, _>("schema_version")
                        .map_err(persistence_error)?,
                )
                .map_err(|_| corrupt_data())?,
                value: serde_json::from_str(&data).map_err(serialization_error)?,
            },
            source_thread_id: Some(
                ChatThreadId::new(source_thread_id).map_err(|_| corrupt_data())?,
            ),
        });
    }
    items.reverse();
    let source_thread_ids = session_metadata.keys().cloned().collect::<BTreeSet<_>>();
    let threads = reads::read_thread_shells_by_ids(pool, &source_thread_ids)
        .await?
        .into_iter()
        .map(|thread| (thread.id.as_str().to_string(), thread))
        .collect::<BTreeMap<_, _>>();
    let mut sessions = session_metadata
        .into_iter()
        .map(|(thread_id, (ordinal, is_current, created_at))| {
            Ok(ChatChannelSessionRead {
                channel_id: channel_id.clone(),
                ordinal,
                is_current,
                created_at,
                thread: threads.get(&thread_id).cloned().ok_or_else(corrupt_data)?,
            })
        })
        .collect::<ChatResult<Vec<_>>>()?;
    sessions.sort_by(|left, right| {
        left.ordinal
            .cmp(&right.ordinal)
            .then_with(|| left.thread.id.as_str().cmp(right.thread.id.as_str()))
    });
    Ok(ChatChannelTimelinePageRead {
        channel_id: channel_id.clone(),
        sessions,
        items,
        turns: reads::read_timeline_turns_by_ids(pool, &turn_ids).await?,
        previous_cursor,
        revision: unsigned(revision)?,
    })
}

pub async fn link_channel_thread(
    pool: &SqlitePool,
    channel_id: &ChatChannelId,
    thread_id: &ChatThreadId,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let channel = read_channel(pool, channel_id).await?;
    let thread = reads::read_thread_shell(pool, thread_id).await?;
    if channel.project_id != thread.project_id {
        return Err(ChatError::validation(
            "threadId",
            "The execution session belongs to another project",
        ));
    }
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    link_channel_thread_in_transaction(
        &mut transaction,
        channel_id,
        thread_id,
        &thread.project_id,
        channel.revision,
        now,
    )
    .await?;
    transaction.commit().await.map_err(persistence_error)?;
    Ok(())
}

pub async fn link_channel_thread_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    channel_id: &ChatChannelId,
    thread_id: &ChatThreadId,
    project_id: &str,
    expected_channel_revision: u64,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let stored: Option<(String, i64, Option<String>)> =
        sqlx::query_as("SELECT project_id, revision, archived_at FROM chat_channels WHERE id = ?")
            .bind(channel_id.as_str())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(persistence_error)?;
    let Some((stored_project_id, stored_revision, archived_at)) = stored else {
        return Err(channel_not_found());
    };
    if stored_project_id != project_id {
        return Err(ChatError::validation(
            "channelId",
            "The channel belongs to another project",
        ));
    }
    if archived_at.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Restore the channel before sending a message",
            true,
        ));
    }
    if unsigned(stored_revision)? != expected_channel_revision {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The channel changed before send",
            true,
        ));
    }
    let existing: Option<String> =
        sqlx::query_scalar("SELECT channel_id FROM chat_channel_sessions WHERE thread_id = ?")
            .bind(thread_id.as_str())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(persistence_error)?;
    if let Some(existing) = existing {
        if existing == channel_id.as_str() {
            return Ok(());
        }
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The execution session is already linked to another channel",
            true,
        ));
    }
    let next_ordinal: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(ordinal), 0) + 1
         FROM chat_channel_sessions WHERE channel_id = ?",
    )
    .bind(channel_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query("UPDATE chat_channel_sessions SET is_current = 0 WHERE channel_id = ?")
        .bind(channel_id.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_channel_sessions
            (channel_id, thread_id, ordinal, is_current, created_at)
         VALUES (?, ?, ?, 1, ?)",
    )
    .bind(channel_id.as_str())
    .bind(thread_id.as_str())
    .bind(next_ordinal)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let updated = sqlx::query(
        "UPDATE chat_channels SET revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(now.as_str())
    .bind(channel_id.as_str())
    .bind(i64_value(expected_channel_revision)?)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The channel changed before send",
            true,
        ));
    }
    Ok(())
}

pub async fn read_channel(
    pool: &SqlitePool,
    channel_id: &ChatChannelId,
) -> ChatResult<ChatChannelRead> {
    for archived in [false, true] {
        let channels = rows_to_channels(
            pool,
            channel_query(pool, Some(channel_id.as_str()), None, archived, false, None).await?,
        )
        .await?;
        if let Some(channel) = channels.into_iter().next() {
            return Ok(channel);
        }
    }
    Err(channel_not_found())
}

async fn read_channels(
    pool: &SqlitePool,
    project_id: &str,
    archived: bool,
    search: Option<(&str, u32)>,
) -> ChatResult<Vec<ChatChannelRead>> {
    rows_to_channels(
        pool,
        channel_query(pool, None, Some(project_id), archived, false, search).await?,
    )
    .await
}

async fn read_navigation_channels(pool: &SqlitePool) -> ChatResult<Vec<ChatChannelRead>> {
    rows_to_channels(
        pool,
        channel_query(pool, None, None, false, true, None).await?,
    )
    .await
}

async fn channel_query(
    pool: &SqlitePool,
    channel_id: Option<&str>,
    project_id: Option<&str>,
    archived: bool,
    active_projects_only: bool,
    search: Option<(&str, u32)>,
) -> ChatResult<Vec<sqlx::sqlite::SqliteRow>> {
    let search_pattern = search.map(|(value, _)| format!("%{}%", escape_like(value)));
    sqlx::query(
        "SELECT c.id, c.project_id, c.name, c.topic, c.is_default,
                c.working_folder_id, c.provider_instance_id, c.model_selection_data,
                c.revision, c.archived_at, c.created_at, c.updated_at,
                current_session.thread_id AS current_thread_id,
                COUNT(all_sessions.thread_id) AS session_count,
                COALESCE(SUM(all_threads.message_count), 0) AS message_count,
                MAX(all_threads.unread_at) AS unread_at,
                COALESCE(MAX(all_threads.last_activity_at), c.updated_at) AS last_activity_at,
                current_thread.latest_preview AS latest_preview
         FROM chat_channels c
         JOIN projects project ON project.id = c.project_id
         LEFT JOIN chat_channel_sessions current_session
           ON current_session.channel_id = c.id AND current_session.is_current = 1
         LEFT JOIN chat_threads current_thread ON current_thread.id = current_session.thread_id
         LEFT JOIN chat_channel_sessions all_sessions ON all_sessions.channel_id = c.id
         LEFT JOIN chat_threads all_threads ON all_threads.id = all_sessions.thread_id
         WHERE (? IS NULL OR c.id = ?)
           AND (? IS NULL OR c.project_id = ?)
           AND (? = 0 OR project.status != 'archived')
           AND ((? = 1 AND c.archived_at IS NOT NULL) OR (? = 0 AND c.archived_at IS NULL))
           AND (? IS NULL OR lower(c.name) LIKE ? ESCAPE '\\' OR lower(c.topic) LIKE ? ESCAPE '\\')
         GROUP BY c.id
         ORDER BY c.is_default DESC, c.name, c.id
         LIMIT COALESCE(?, -1)",
    )
    .bind(channel_id)
    .bind(channel_id)
    .bind(project_id)
    .bind(project_id)
    .bind(active_projects_only)
    .bind(archived)
    .bind(archived)
    .bind(search_pattern.as_deref())
    .bind(search_pattern.as_deref())
    .bind(search_pattern.as_deref())
    .bind(search.map(|(_, limit)| i64::from(limit)))
    .fetch_all(pool)
    .await
    .map_err(persistence_error)
}

async fn rows_to_channels(
    pool: &SqlitePool,
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> ChatResult<Vec<ChatChannelRead>> {
    let current_thread_ids = rows
        .iter()
        .map(|row| {
            row.try_get::<Option<String>, _>("current_thread_id")
                .map_err(persistence_error)
        })
        .collect::<ChatResult<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<BTreeSet<_>>();
    let current_threads = reads::read_thread_shells_by_ids(pool, &current_thread_ids)
        .await?
        .into_iter()
        .map(|thread| (thread.id.as_str().to_string(), thread))
        .collect::<BTreeMap<_, _>>();
    rows.into_iter()
        .map(|row| row_to_channel(row, &current_threads))
        .collect()
}

fn row_to_channel(
    row: sqlx::sqlite::SqliteRow,
    current_threads: &BTreeMap<String, super::models::ChatThreadShellRead>,
) -> ChatResult<ChatChannelRead> {
    let model: StoredChannelModel = serde_json::from_str(
        &row.try_get::<String, _>("model_selection_data")
            .map_err(persistence_error)?,
    )
    .map_err(|_| corrupt_data())?;
    let current_thread_id = row
        .try_get::<Option<String>, _>("current_thread_id")
        .map_err(persistence_error)?;
    let current_thread = current_thread_id
        .map(|thread_id| {
            current_threads
                .get(&thread_id)
                .cloned()
                .ok_or_else(corrupt_data)
        })
        .transpose()?;
    Ok(ChatChannelRead {
        id: ChatChannelId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
            .map_err(|_| corrupt_data())?,
        project_id: row.try_get("project_id").map_err(persistence_error)?,
        name: row.try_get("name").map_err(persistence_error)?,
        topic: row.try_get("topic").map_err(persistence_error)?,
        is_default: row
            .try_get::<i64, _>("is_default")
            .map_err(persistence_error)?
            == 1,
        target: ChatChannelTargetRead {
            working_folder_id: row
                .try_get::<Option<String>, _>("working_folder_id")
                .map_err(persistence_error)?
                .map(ProjectWorkingFolderId::new)
                .transpose()
                .map_err(|_| corrupt_data())?,
            provider_instance_id: row
                .try_get::<Option<String>, _>("provider_instance_id")
                .map_err(persistence_error)?
                .map(ProviderInstanceId::new)
                .transpose()
                .map_err(|_| corrupt_data())?,
            provider_managed_model: model.provider_managed_model,
            model_id: model.model_id,
            model_options: model.model_options,
        },
        current_thread,
        session_count: unsigned(row.try_get("session_count").map_err(persistence_error)?)?,
        message_count: unsigned(row.try_get("message_count").map_err(persistence_error)?)?,
        latest_preview: row.try_get("latest_preview").map_err(persistence_error)?,
        last_activity_at: required_timestamp(
            row.try_get("last_activity_at").map_err(persistence_error)?,
        )?,
        unread_at: optional_timestamp(row.try_get("unread_at").map_err(persistence_error)?)?,
        revision: unsigned(row.try_get("revision").map_err(persistence_error)?)?,
        archived_at: optional_timestamp(row.try_get("archived_at").map_err(persistence_error)?)?,
        created_at: required_timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
        updated_at: required_timestamp(row.try_get("updated_at").map_err(persistence_error)?)?,
    })
}

async fn set_channel_archived(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    expected_revision: u64,
    archived: bool,
) -> ChatResult<ChatChannelRead> {
    let pool = chat_pool(app, db_url).await?;
    let channel = read_channel(&pool, &channel_id).await?;
    if archived && channel.is_default {
        return Err(ChatError::validation(
            "channelId",
            "The general channel cannot be archived",
        ));
    }
    if archived && channel.current_thread.as_ref().is_some_and(thread_is_busy) {
        return Err(ChatError::new(
            ChatErrorCode::Busy,
            "Finish or stop the current agent run before archiving its channel",
            true,
        ));
    }
    let now = now_timestamp()?;
    let updated = sqlx::query(
        "UPDATE chat_channels
         SET archived_at = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(archived.then_some(now.as_str()))
    .bind(now.as_str())
    .bind(channel_id.as_str())
    .bind(i64_value(expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    require_updated(&pool, &channel_id, updated.rows_affected()).await?;
    read_channel(&pool, &channel_id).await
}

async fn validate_target(
    pool: &SqlitePool,
    project_id: &str,
    target: &ChatChannelTargetInput,
) -> ChatResult<()> {
    if let Some(working_folder_id) = target.working_folder_id.as_ref() {
        let owner: Option<String> =
            sqlx::query_scalar("SELECT project_id FROM project_working_folders WHERE id = ?")
                .bind(working_folder_id.as_str())
                .fetch_optional(pool)
                .await
                .map_err(persistence_error)?;
        if owner.as_deref() != Some(project_id) {
            return Err(ChatError::validation(
                "workingFolderId",
                "The working folder belongs to another project",
            ));
        }
    }
    if target.provider_managed_model && target.model_id.is_some() {
        return Err(ChatError::validation(
            "modelId",
            "A provider-managed model cannot also have an explicit model ID",
        ));
    }
    if target.model_options.len() > 128 {
        return Err(ChatError::validation(
            "modelOptions",
            "The channel has too many model options",
        ));
    }
    Ok(())
}

fn normalized_channel_name(value: &str) -> ChatResult<String> {
    let value = value.trim().trim_start_matches('#');
    let mut normalized = String::new();
    let mut pending_separator = false;
    for character in value.chars() {
        if character.is_alphanumeric() {
            if pending_separator && !normalized.is_empty() {
                normalized.push('-');
            }
            for lowercase in character.to_lowercase() {
                normalized.push(lowercase);
            }
            pending_separator = false;
        } else if character == '-' || character.is_whitespace() {
            pending_separator = true;
        } else {
            return Err(ChatError::validation(
                "name",
                "Channel names use letters, numbers, and hyphens",
            ));
        }
    }
    if normalized.is_empty() || normalized.chars().count() > MAX_CHANNEL_NAME_CHARS {
        return Err(ChatError::validation(
            "name",
            "Channel name must contain between 1 and 80 characters",
        ));
    }
    Ok(normalized)
}

fn normalized_topic(value: &str) -> ChatResult<String> {
    let value = value.trim();
    if value.chars().count() > MAX_CHANNEL_TOPIC_CHARS || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "topic",
            "Channel topic must contain at most 250 characters",
        ));
    }
    Ok(value.to_string())
}

async fn require_unique_name(
    pool: &SqlitePool,
    project_id: &str,
    name: &str,
    except: Option<&ChatChannelId>,
) -> ChatResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM chat_channels
            WHERE project_id = ? AND name = ? AND (? IS NULL OR id != ?)
         )",
    )
    .bind(project_id)
    .bind(name)
    .bind(except.map(ChatChannelId::as_str))
    .bind(except.map(ChatChannelId::as_str))
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if exists == 1 {
        return Err(ChatError::validation(
            "name",
            "A channel with this name already exists in the project",
        ));
    }
    Ok(())
}

fn thread_is_busy(thread: &super::models::ChatThreadShellRead) -> bool {
    matches!(
        thread.latest_turn_state,
        Some(
            super::models::ChatTurnState::Pending
                | super::models::ChatTurnState::Dispatching
                | super::models::ChatTurnState::Active
                | super::models::ChatTurnState::WaitingForApproval
                | super::models::ChatTurnState::WaitingForUserInput
        )
    )
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredChannelModel {
    provider_managed_model: bool,
    model_id: Option<ModelId>,
    #[serde(default)]
    model_options: Vec<ModelOptionSelection>,
}

fn stored_model(target: &ChatChannelTargetInput) -> ChatResult<String> {
    serde_json::to_string(&StoredChannelModel {
        provider_managed_model: target.provider_managed_model,
        model_id: target.model_id.clone(),
        model_options: target.model_options.clone(),
    })
    .map_err(|_| corrupt_data())
}

fn validate_project_id(value: &str) -> ChatResult<()> {
    if value.trim().is_empty() || value.len() > 1_024 || value.chars().any(char::is_control) {
        return Err(ChatError::validation("projectId", "Project ID is invalid"));
    }
    Ok(())
}

async fn require_updated(
    pool: &SqlitePool,
    channel_id: &ChatChannelId,
    rows_affected: u64,
) -> ChatResult<()> {
    if rows_affected == 1 {
        return Ok(());
    }
    let exists: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chat_channels WHERE id = ?)")
        .bind(channel_id.as_str())
        .fetch_one(pool)
        .await
        .map_err(persistence_error)?;
    if exists == 0 {
        return Err(channel_not_found());
    }
    Err(ChatError::new(
        ChatErrorCode::StaleRevision,
        "The channel changed before this operation completed",
        true,
    ))
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn parse_channel_timeline_cursor(value: &str) -> ChatResult<ChatChannelTimelineCursor> {
    if value.is_empty() || value.len() > 1_200 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "cursor",
            "Chat channel timeline cursor is invalid",
        ));
    }
    let cursor: ChatChannelTimelineCursor = serde_json::from_str(value)
        .map_err(|_| ChatError::validation("cursor", "Chat channel timeline cursor is invalid"))?;
    if cursor.session_ordinal == 0 || cursor.row_id.is_empty() || cursor.row_id.len() > 1_024 {
        return Err(ChatError::validation(
            "cursor",
            "Chat channel timeline cursor is invalid",
        ));
    }
    Ok(cursor)
}

fn unsigned(value: i64) -> ChatResult<u64> {
    u64::try_from(value).map_err(|_| corrupt_data())
}

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value)
        .map_err(|_| ChatError::validation("revision", "Channel revision is too large"))
}

fn required_timestamp(value: String) -> ChatResult<UtcTimestamp> {
    UtcTimestamp::new(value).map_err(|_| corrupt_data())
}

fn optional_timestamp(value: Option<String>) -> ChatResult<Option<UtcTimestamp>> {
    value
        .map(UtcTimestamp::new)
        .transpose()
        .map_err(|_| corrupt_data())
}

fn channel_not_found() -> ChatError {
    ChatError::new(ChatErrorCode::NotFound, "Chat channel was not found", true)
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat channel persistence failed",
        true,
    )
}

fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat channel data could not be decoded",
        false,
    )
}

fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat channel data is invalid",
        false,
    )
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::tests::repository::pool_with_thread;

    const TEST_NOW: &str = "2026-07-31T12:00:00Z";

    #[test]
    fn normalizes_channel_names_for_durable_navigation() {
        assert_eq!(
            normalized_channel_name(" #Release Planning ").unwrap(),
            "release-planning"
        );
        assert_eq!(
            normalized_channel_name("RÉSUMÉ 2026").unwrap(),
            "résumé-2026"
        );
        assert!(normalized_channel_name("release/planning").is_err());
        assert!(normalized_channel_name("---").is_err());
    }

    #[test]
    fn managed_project_gets_a_protected_general_channel() {
        tauri::async_runtime::block_on(async {
            let pool = pool_with_thread().await;
            let channel_id = ChatChannelId::new(
                sqlx::query_scalar::<_, String>(
                    "SELECT id FROM chat_channels WHERE project_id = 'project-chat'",
                )
                .fetch_one(&pool)
                .await
                .unwrap(),
            )
            .unwrap();
            let channel = read_channel(&pool, &channel_id).await.unwrap();
            assert_eq!(channel.name, "general");
            assert!(channel.is_default);
            assert_eq!(
                channel.target.working_folder_id.unwrap().as_str(),
                "workspace-1"
            );
            assert_eq!(channel.session_count, 0);

            let archive = sqlx::query("UPDATE chat_channels SET archived_at = ? WHERE id = ?")
                .bind(TEST_NOW)
                .bind(channel_id.as_str())
                .execute(&pool)
                .await;
            assert!(archive.is_err());
        });
    }

    #[test]
    fn links_ordered_hidden_sessions_and_promotes_only_the_latest() {
        tauri::async_runtime::block_on(async {
            let pool = pool_with_thread().await;
            let channel_id = ChatChannelId::new(
                sqlx::query_scalar::<_, String>(
                    "SELECT id FROM chat_channels WHERE is_default = 1",
                )
                .fetch_one(&pool)
                .await
                .unwrap(),
            )
            .unwrap();
            let first_thread = ChatThreadId::new("thread-1").unwrap();
            let now = UtcTimestamp::new(TEST_NOW).unwrap();
            link_channel_thread(&pool, &channel_id, &first_thread, &now)
                .await
                .unwrap();
            link_channel_thread(&pool, &channel_id, &first_thread, &now)
                .await
                .unwrap();

            sqlx::query(
                "INSERT INTO chat_threads
                    (id, project_id, working_folder_id, title, provider_family_id,
                     provider_instance_id, continuation_group_id, safety_mode,
                     interaction_mode, state, last_activity_at, created_at, updated_at)
                 VALUES ('thread-2', 'project-chat', 'workspace-1', 'Second session',
                         'claude', 'claude-local', 'continuation-2',
                         'ask_for_approval', 'build', 'idle', ?, ?, ?)",
            )
            .bind(TEST_NOW)
            .bind(TEST_NOW)
            .bind(TEST_NOW)
            .execute(&pool)
            .await
            .unwrap();
            let second_thread = ChatThreadId::new("thread-2").unwrap();
            link_channel_thread(&pool, &channel_id, &second_thread, &now)
                .await
                .unwrap();

            let sessions = sqlx::query_as::<_, (String, i64, i64)>(
                "SELECT thread_id, ordinal, is_current
                 FROM chat_channel_sessions WHERE channel_id = ? ORDER BY ordinal",
            )
            .bind(channel_id.as_str())
            .fetch_all(&pool)
            .await
            .unwrap();
            assert_eq!(
                sessions,
                vec![("thread-1".into(), 1, 0), ("thread-2".into(), 2, 1)]
            );

            let channel = read_channel(&pool, &channel_id).await.unwrap();
            assert_eq!(channel.session_count, 2);
            assert_eq!(channel.current_thread.unwrap().id.as_str(), "thread-2");
            assert_eq!(channel.revision, 3);
        });
    }

    #[test]
    fn pages_one_chronological_timeline_across_hidden_sessions() {
        tauri::async_runtime::block_on(async {
            let pool = pool_with_thread().await;
            let channel_id = ChatChannelId::new(
                sqlx::query_scalar::<_, String>(
                    "SELECT id FROM chat_channels WHERE is_default = 1",
                )
                .fetch_one(&pool)
                .await
                .unwrap(),
            )
            .unwrap();
            let now = UtcTimestamp::new(TEST_NOW).unwrap();
            link_channel_thread(
                &pool,
                &channel_id,
                &ChatThreadId::new("thread-1").unwrap(),
                &now,
            )
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO chat_threads
                    (id, project_id, working_folder_id, title, provider_family_id,
                     provider_instance_id, continuation_group_id, safety_mode,
                     interaction_mode, state, last_activity_at, created_at, updated_at)
                 VALUES ('thread-2', 'project-chat', 'workspace-1', 'Second session',
                         'claude', 'claude-local', 'continuation-2',
                         'ask_for_approval', 'build', 'idle', ?, ?, ?)",
            )
            .bind(TEST_NOW)
            .bind(TEST_NOW)
            .bind(TEST_NOW)
            .execute(&pool)
            .await
            .unwrap();
            link_channel_thread(
                &pool,
                &channel_id,
                &ChatThreadId::new("thread-2").unwrap(),
                &now,
            )
            .await
            .unwrap();
            for (thread_id, message_id, markdown) in [
                ("thread-1", "message-1", "First session"),
                ("thread-2", "message-2", "Second session"),
            ] {
                sqlx::query(
                    "INSERT INTO chat_messages
                        (id, thread_id, sequence_anchor, role, normalized_markdown,
                         streaming_state, created_at, updated_at)
                     VALUES (?, ?, 1, 'assistant', ?, 'complete', ?, ?)",
                )
                .bind(message_id)
                .bind(thread_id)
                .bind(markdown)
                .bind(TEST_NOW)
                .bind(TEST_NOW)
                .execute(&pool)
                .await
                .unwrap();
                sqlx::query(
                    "UPDATE chat_threads
                     SET message_count = 1, last_event_sequence = 1, revision = revision + 1
                     WHERE id = ?",
                )
                .bind(thread_id)
                .execute(&pool)
                .await
                .unwrap();
            }

            let newest = read_channel_timeline_page(&pool, &channel_id, None, 2)
                .await
                .unwrap();
            assert_eq!(
                newest
                    .items
                    .iter()
                    .map(|item| item.activity_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["channel-session:thread-2", "message-2"]
            );
            assert_eq!(newest.sessions.len(), 1);
            assert_eq!(newest.sessions[0].thread.id.as_str(), "thread-2");
            let cursor = parse_channel_timeline_cursor(
                newest.previous_cursor.as_deref().expect("older cursor"),
            )
            .unwrap();

            let older = read_channel_timeline_page(&pool, &channel_id, Some(&cursor), 2)
                .await
                .unwrap();
            assert_eq!(
                older
                    .items
                    .iter()
                    .map(|item| item.activity_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["channel-session:thread-1", "message-1"]
            );
            assert!(older.previous_cursor.is_none());
            assert!(older.items[0].sequence_anchor < newest.items[0].sequence_anchor);
            assert_eq!(older.sessions[0].thread.id.as_str(), "thread-1");
        });
    }
}
