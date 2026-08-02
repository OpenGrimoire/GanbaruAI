//! Durable organizational messages, reply threads, teammates, and work assignments.

use super::channel_commands::{
    chat_pool, i64_value, identifier_error, now_timestamp, optional_timestamp, persistence_error,
    timestamp, u64_value,
};
use super::models::*;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const LOCAL_PARTICIPANT_ID: &str = "participant:local-owner";
const MAX_MESSAGE_BYTES: usize = 128 * 1024;
const MAX_CHANNEL_CONTEXT_MESSAGES: usize = 20;
const MAX_THREAD_CONTEXT_REPLIES: usize = 50;
const MAX_PAGE_SIZE: u32 = 100;
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_SEARCH_RESULTS: u32 = 100;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTeammatePolicyInput {
    pub provider_instance_id: ProviderInstanceId,
    #[serde(default)]
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    #[serde(default)]
    pub model_options: Vec<ModelOptionSelection>,
    pub effort: Option<String>,
    pub speed: Option<String>,
    pub provider_options: VersionedJson,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTeammateMembershipInput {
    pub channel_id: ChatChannelId,
    #[serde(default = "default_addressable")]
    pub addressable: bool,
    pub approval_policy: ChatApprovalPolicy,
    pub working_folder_ids: Vec<ProjectWorkingFolderId>,
    pub default_working_folder_id: ProjectWorkingFolderId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatTeammateCommand {
    pub teammate_id: ChatParticipantId,
    pub display_name: String,
    pub handle: String,
    pub avatar: VersionedJson,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub instructions: String,
    pub policy: ChatTeammatePolicyInput,
    #[serde(default)]
    pub memberships: Vec<ChatTeammateMembershipInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChatTeammateProfileCommand {
    pub teammate_id: ChatParticipantId,
    pub display_name: String,
    pub handle: String,
    pub avatar: VersionedJson,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub instructions: String,
    pub expected_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishChatTeammatePolicyCommand {
    pub teammate_id: ChatParticipantId,
    pub policy: ChatTeammatePolicyInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertChatTeammateMembershipCommand {
    pub teammate_id: ChatParticipantId,
    pub membership: ChatTeammateMembershipInput,
    pub expected_revision: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatParticipantMentionInput {
    pub participant_id: ChatParticipantId,
    pub participant_kind: ChatParticipantKind,
    pub handle_snapshot: Option<String>,
    pub label_snapshot: String,
    pub start_offset: u64,
    pub end_offset: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResourceReferenceInput {
    pub working_folder_id: ProjectWorkingFolderId,
    pub kind: String,
    pub relative_path: String,
    pub display_label: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostChatMessageCommand {
    pub client_command_id: ChatCommandId,
    pub channel_id: ChatChannelId,
    pub reply_thread_id: Option<ChatReplyThreadId>,
    pub normalized_markdown: String,
    pub rich_content: VersionedJson,
    #[serde(default)]
    pub attachment_ids: Vec<ChatAttachmentId>,
    #[serde(default)]
    pub participant_mentions: Vec<ChatParticipantMentionInput>,
    #[serde(default)]
    pub resource_references: Vec<ChatResourceReferenceInput>,
    #[serde(default)]
    pub post_without_invoking: bool,
    #[serde(default)]
    pub also_send_to_channel: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostChatMessageResult {
    pub message: ChatMessageRead,
    pub reply_thread_id: ChatReplyThreadId,
    pub assignment: Option<ChatWorkAssignmentRead>,
    pub assignment_input_queued: bool,
}

#[tauri::command]
pub async fn chat_list_teammates(
    app: tauri::AppHandle,
    db_url: String,
    archived: bool,
) -> ChatResult<Vec<ChatAiTeammateRead>> {
    let pool = chat_pool(app, db_url).await?;
    let rows = sqlx::query(
        "SELECT participant_id
         FROM chat_ai_teammates teammate
         JOIN chat_participants participant ON participant.id = teammate.participant_id
         WHERE (participant.archived_at IS NOT NULL) = ?
         ORDER BY participant.display_name COLLATE NOCASE, participant.id",
    )
    .bind(i64::from(archived))
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let mut teammates = Vec::with_capacity(rows.len());
    for row in rows {
        let id = ChatParticipantId::new(
            row.try_get::<String, _>("participant_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        teammates.push(read_teammate(&pool, &id).await?);
    }
    Ok(teammates)
}

#[tauri::command]
pub async fn chat_read_teammate(
    app: tauri::AppHandle,
    db_url: String,
    teammate_id: ChatParticipantId,
) -> ChatResult<ChatAiTeammateRead> {
    read_teammate(&chat_pool(app, db_url).await?, &teammate_id).await
}

#[tauri::command]
pub async fn chat_create_teammate(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatTeammateCommand,
) -> ChatResult<ChatAiTeammateRead> {
    let display_name = validate_display_name(&request.display_name)?;
    let handle = validate_handle(&request.handle)?;
    validate_profile_text(&request.purpose, 1_000, "purpose")?;
    validate_profile_text(&request.instructions, 65_536, "instructions")?;
    validate_policy(&app, &request.policy)?;
    let avatar_data = json_object(&request.avatar, "avatar")?;
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_participants
            (id, participant_kind, display_name, normalized_handle,
             avatar_schema_version, avatar_data, created_at, updated_at)
         VALUES (?, 'ai_teammate', ?, ?, ?, ?, ?, ?)",
    )
    .bind(request.teammate_id.as_str())
    .bind(display_name)
    .bind(handle)
    .bind(i64::from(request.avatar.schema_version))
    .bind(avatar_data)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(map_teammate_write_error)?;
    sqlx::query(
        "INSERT INTO chat_ai_teammates
            (participant_id, purpose, instructions, configuration_state, created_at, updated_at)
         VALUES (?, ?, ?, 'healthy', ?, ?)",
    )
    .bind(request.teammate_id.as_str())
    .bind(request.purpose.trim())
    .bind(request.instructions.trim())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    insert_policy_revision(
        &mut transaction,
        &request.teammate_id,
        1,
        &request.policy,
        &now,
    )
    .await?;
    for membership in &request.memberships {
        upsert_membership_in_transaction(
            &mut transaction,
            &request.teammate_id,
            membership,
            None,
            &now,
        )
        .await?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    read_teammate(&pool, &request.teammate_id).await
}

#[tauri::command]
pub async fn chat_update_teammate_profile(
    app: tauri::AppHandle,
    db_url: String,
    request: UpdateChatTeammateProfileCommand,
) -> ChatResult<ChatAiTeammateRead> {
    let display_name = validate_display_name(&request.display_name)?;
    let handle = validate_handle(&request.handle)?;
    validate_profile_text(&request.purpose, 1_000, "purpose")?;
    validate_profile_text(&request.instructions, 65_536, "instructions")?;
    let avatar_data = json_object(&request.avatar, "avatar")?;
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let updated = sqlx::query(
        "UPDATE chat_participants
         SET display_name = ?, normalized_handle = ?, avatar_schema_version = ?,
             avatar_data = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND participant_kind = 'ai_teammate' AND revision = ?",
    )
    .bind(display_name)
    .bind(handle)
    .bind(i64::from(request.avatar.schema_version))
    .bind(avatar_data)
    .bind(now.as_str())
    .bind(request.teammate_id.as_str())
    .bind(i64_value(request.expected_revision)?)
    .execute(&mut *transaction)
    .await
    .map_err(map_teammate_write_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The teammate changed before the update",
            true,
        ));
    }
    sqlx::query(
        "UPDATE chat_ai_teammates SET purpose = ?, instructions = ?, updated_at = ?
         WHERE participant_id = ?",
    )
    .bind(request.purpose.trim())
    .bind(request.instructions.trim())
    .bind(now.as_str())
    .bind(request.teammate_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)?;
    read_teammate(&pool, &request.teammate_id).await
}

#[tauri::command]
pub async fn chat_archive_teammate(
    app: tauri::AppHandle,
    db_url: String,
    teammate_id: ChatParticipantId,
    expected_revision: u64,
    archived: bool,
) -> ChatResult<ChatAiTeammateRead> {
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let archived_at = archived.then(|| now.as_str());
    let updated = sqlx::query(
        "UPDATE chat_participants
         SET archived_at = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND participant_kind = 'ai_teammate' AND revision = ?",
    )
    .bind(archived_at)
    .bind(now.as_str())
    .bind(teammate_id.as_str())
    .bind(i64_value(expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The teammate changed before the update",
            true,
        ));
    }
    read_teammate(&pool, &teammate_id).await
}

#[tauri::command]
pub async fn chat_publish_teammate_policy(
    app: tauri::AppHandle,
    db_url: String,
    request: PublishChatTeammatePolicyCommand,
) -> ChatResult<ChatTeammatePolicyRead> {
    validate_policy(&app, &request.policy)?;
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let revision: i64 = sqlx::query_scalar(
        "SELECT latest_policy_revision + 1 FROM chat_ai_teammates WHERE participant_id = ?",
    )
    .bind(request.teammate_id.as_str())
    .fetch_optional(&pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "AI teammate was not found", true))?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    insert_policy_revision(
        &mut transaction,
        &request.teammate_id,
        u64_value(revision)?,
        &request.policy,
        &now,
    )
    .await?;
    transaction.commit().await.map_err(persistence_error)?;
    read_policy(&pool, &request.teammate_id, u64_value(revision)?).await
}

#[tauri::command]
pub async fn chat_list_channel_memberships(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
) -> ChatResult<Vec<ChatConversationMembershipRead>> {
    let pool = chat_pool(app, db_url).await?;
    let channel = super::channel_commands::read_channel(&pool, &channel_id).await?;
    read_memberships_for_conversation(&pool, &channel.conversation_id, false).await
}

#[tauri::command]
pub async fn chat_read_project_primary_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    project_id: String,
) -> ChatResult<ChatProjectPrimaryWorkingFolderRead> {
    let row = sqlx::query(
        "SELECT project_id, working_folder_id, revision
         FROM chat_project_primary_working_folders WHERE project_id = ?",
    )
    .bind(&project_id)
    .fetch_optional(&chat_pool(app, db_url).await?)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Primary working folder was not found",
            true,
        )
    })?;
    Ok(ChatProjectPrimaryWorkingFolderRead {
        project_id: row.try_get("project_id").map_err(persistence_error)?,
        working_folder_id: ProjectWorkingFolderId::new(
            row.try_get::<String, _>("working_folder_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
    })
}

#[tauri::command]
pub async fn chat_set_project_primary_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    project_id: String,
    working_folder_id: ProjectWorkingFolderId,
    expected_revision: u64,
) -> ChatResult<ChatProjectPrimaryWorkingFolderRead> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let now = now_timestamp()?;
    let updated = sqlx::query(
        "UPDATE chat_project_primary_working_folders
         SET working_folder_id = ?, revision = revision + 1, updated_at = ?
         WHERE project_id = ? AND revision = ?",
    )
    .bind(working_folder_id.as_str())
    .bind(now.as_str())
    .bind(&project_id)
    .bind(i64_value(expected_revision)?)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The primary working folder changed before the update",
            true,
        ));
    }
    chat_read_project_primary_working_folder(app, db_url, project_id).await
}

#[tauri::command]
pub async fn chat_upsert_teammate_membership(
    app: tauri::AppHandle,
    db_url: String,
    request: UpsertChatTeammateMembershipCommand,
) -> ChatResult<ChatConversationMembershipRead> {
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let conversation_id = upsert_membership_in_transaction(
        &mut transaction,
        &request.teammate_id,
        &request.membership,
        request.expected_revision,
        &now,
    )
    .await?;
    transaction.commit().await.map_err(persistence_error)?;
    read_membership(&pool, &conversation_id, &request.teammate_id).await
}

#[tauri::command]
pub async fn chat_remove_teammate_membership(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    teammate_id: ChatParticipantId,
    expected_revision: u64,
    stop_active_work: bool,
) -> ChatResult<ChatConversationMembershipRead> {
    let pool = chat_pool(app, db_url).await?;
    let channel = super::channel_commands::read_channel(&pool, &channel_id).await?;
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*)
         FROM chat_work_assignments assignment
         JOIN chat_reply_threads thread ON thread.id = assignment.reply_thread_id
         WHERE thread.conversation_id = ? AND assignment.teammate_id = ?
           AND assignment.state IN (
             'queued', 'working', 'waiting_for_answer', 'waiting_for_approval', 'ready_for_review'
           )",
    )
    .bind(channel.conversation_id.as_str())
    .bind(teammate_id.as_str())
    .fetch_one(&pool)
    .await
    .map_err(persistence_error)?;
    if active > 0 && !stop_active_work {
        return Err(ChatError::new(
            ChatErrorCode::Busy,
            "Wait for active work or choose Remove and stop work",
            true,
        ));
    }
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    if active > 0 {
        sqlx::query(
            "UPDATE chat_work_assignments
             SET state = 'cancelled', state_reason = 'Channel membership removed',
                 settled_at = ?, revision = revision + 1, updated_at = ?
             WHERE teammate_id = ?
               AND reply_thread_id IN (
                 SELECT id FROM chat_reply_threads WHERE conversation_id = ?
               )
               AND state IN (
                 'queued', 'working', 'waiting_for_answer', 'waiting_for_approval', 'ready_for_review'
               )",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(teammate_id.as_str())
        .bind(channel.conversation_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_assignment_dispatch_jobs
             SET state = 'cancelled', updated_at = ?
             WHERE assignment_id IN (
               SELECT assignment.id
               FROM chat_work_assignments assignment
               JOIN chat_reply_threads thread ON thread.id = assignment.reply_thread_id
               WHERE thread.conversation_id = ? AND assignment.teammate_id = ?
             ) AND state IN ('queued', 'claimed')",
        )
        .bind(now.as_str())
        .bind(channel.conversation_id.as_str())
        .bind(teammate_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    let updated = sqlx::query(
        "UPDATE chat_conversation_memberships
         SET removed_at = ?, addressable = 0, revision = revision + 1, updated_at = ?
         WHERE conversation_id = ? AND participant_id = ? AND revision = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(channel.conversation_id.as_str())
    .bind(teammate_id.as_str())
    .bind(i64_value(expected_revision)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The channel membership changed before removal",
            true,
        ));
    }
    sqlx::query(
        "UPDATE chat_teammate_working_folder_grants
         SET revoked_at = ?, is_default = 0
         WHERE conversation_id = ? AND teammate_id = ? AND revoked_at IS NULL",
    )
    .bind(now.as_str())
    .bind(channel.conversation_id.as_str())
    .bind(teammate_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)?;
    read_membership(&pool, &channel.conversation_id, &teammate_id).await
}

#[tauri::command]
pub async fn chat_post_message(
    app: tauri::AppHandle,
    db_url: String,
    request: PostChatMessageCommand,
) -> ChatResult<PostChatMessageResult> {
    validate_message_request(&request)?;
    let dispatch_app = app.clone();
    let dispatch_db_url = db_url.clone();
    let pool = chat_pool(app, db_url).await?;
    if let Some(result) = read_post_receipt(&pool, &request.client_command_id).await? {
        return Ok(result);
    }
    let channel = super::channel_commands::read_channel(&pool, &request.channel_id).await?;
    if channel.archived_at.is_some() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Restore the channel before posting a message",
            true,
        ));
    }
    if let Some(reply_thread_id) = request.reply_thread_id.as_ref() {
        require_reply_thread(&pool, reply_thread_id, &channel.conversation_id).await?;
    }
    let resolved_invocation = resolve_invoked_teammate(
        &pool,
        &channel.conversation_id,
        request.reply_thread_id.as_ref(),
        &request.participant_mentions,
        request.post_without_invoking,
    )
    .await?;
    let now = now_timestamp()?;
    let item_id = conversation_item_id()?;
    let revision_id = message_revision_id()?;
    let new_reply_thread_id = request
        .reply_thread_id
        .clone()
        .unwrap_or(reply_thread_id()?);
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_organizational_command_receipts
            (client_command_id, command_kind, state, created_at, updated_at)
         VALUES (?, 'post_message', 'accepted', ?, ?)",
    )
    .bind(request.client_command_id.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(map_command_receipt_error)?;
    let ordinal = next_item_ordinal(
        &mut transaction,
        &channel.conversation_id,
        request.reply_thread_id.as_ref(),
    )
    .await?;
    insert_communication_message(
        &mut transaction,
        CommunicationMessageWrite {
            item_id: &item_id,
            revision_id: &revision_id,
            conversation_id: &channel.conversation_id,
            reply_thread_id: request.reply_thread_id.as_ref(),
            ordinal,
            request: &request,
            now: &now,
        },
    )
    .await?;
    if request.reply_thread_id.is_none() {
        sqlx::query(
            "INSERT INTO chat_reply_threads
                (id, conversation_id, root_item_id, last_activity_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_reply_thread_id.as_str())
        .bind(channel.conversation_id.as_str())
        .bind(item_id.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    } else {
        sqlx::query(
            "UPDATE chat_reply_threads
             SET reply_count = reply_count + 1, last_activity_at = ?,
                 revision = revision + 1, updated_at = ? WHERE id = ?",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(new_reply_thread_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    sqlx::query(
        "UPDATE chat_conversations
         SET last_activity_at = ?, revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(channel.conversation_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    let assignment_write = persist_assignment_routing(
        &mut transaction,
        &new_reply_thread_id,
        &item_id,
        resolved_invocation.as_ref(),
        &now,
    )
    .await?;
    if request.also_send_to_channel && request.reply_thread_id.is_some() {
        insert_channel_copy(&mut transaction, &channel.conversation_id, &request, &now).await?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    let message = read_message(&pool, &item_id).await?;
    let assignment = match assignment_write.assignment_id.as_ref() {
        Some(assignment_id) => Some(read_assignment(&pool, assignment_id).await?),
        None => read_active_or_latest_assignment(&pool, &new_reply_thread_id).await?,
    };
    let result = PostChatMessageResult {
        message,
        reply_thread_id: new_reply_thread_id,
        assignment,
        assignment_input_queued: assignment_write.input_queued,
    };
    let serialized = serde_json::to_string(&result).map_err(serialization_error)?;
    sqlx::query(
        "UPDATE chat_organizational_command_receipts
         SET state = 'completed', result_schema_version = 1, result_data = ?, updated_at = ?
         WHERE client_command_id = ?",
    )
    .bind(serialized)
    .bind(now_timestamp()?.as_str())
    .bind(request.client_command_id.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if let Some(assignment_id) = assignment_write.assignment_id.clone() {
        if assignment_write.input_queued {
            let input_message_id = item_id.clone();
            tauri::async_runtime::spawn(async move {
                let _ = deliver_assignment_input(
                    dispatch_app,
                    dispatch_db_url,
                    assignment_id,
                    input_message_id,
                )
                .await;
            });
        } else {
            tauri::async_runtime::spawn(async move {
                let _ = dispatch_assignment_job(dispatch_app, dispatch_db_url, assignment_id).await;
            });
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn chat_recover_assignment_dispatch_jobs(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<u32> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let now = now_timestamp()?;
    sqlx::query(
        "UPDATE chat_assignment_dispatch_jobs
         SET state = 'queued', claimed_at = NULL, claim_token = NULL, updated_at = ?
         WHERE state = 'claimed'
           AND claimed_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-5 minutes')",
    )
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    let assignment_ids = sqlx::query_scalar::<_, String>(
        "SELECT assignment_id FROM chat_assignment_dispatch_jobs
         WHERE state = 'queued' AND available_at <= ?
         ORDER BY created_at, id LIMIT 32",
    )
    .bind(now.as_str())
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let count = u32::try_from(assignment_ids.len()).unwrap_or(32);
    for assignment_id in assignment_ids {
        let assignment_id = ChatWorkAssignmentId::new(assignment_id).map_err(identifier_error)?;
        let worker_app = app.clone();
        let worker_db_url = db_url.clone();
        tauri::async_runtime::spawn(async move {
            let _ = dispatch_assignment_job(worker_app, worker_db_url, assignment_id).await;
        });
    }
    Ok(count)
}

#[tauri::command]
pub async fn chat_read_channel_page(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    cursor: Option<String>,
    limit: Option<u32>,
) -> ChatResult<ChatChannelPageRead> {
    let pool = chat_pool(app, db_url).await?;
    let channel = super::channel_commands::read_channel(&pool, &channel_id).await?;
    let before = parse_cursor(cursor.as_deref())?;
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let rows = sqlx::query(
        "SELECT id, ordinal
         FROM chat_conversation_items
         WHERE conversation_id = ? AND reply_thread_id IS NULL AND item_kind = 'message'
           AND (? IS NULL OR ordinal < ?)
         ORDER BY ordinal DESC LIMIT ?",
    )
    .bind(channel.conversation_id.as_str())
    .bind(before)
    .bind(before)
    .bind(i64::from(limit))
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let previous_cursor = rows
        .last()
        .map(|row| row.try_get::<i64, _>("ordinal"))
        .transpose()
        .map_err(persistence_error)?
        .filter(|_| rows.len() == limit as usize)
        .map(|ordinal| ordinal.to_string());
    let mut messages = Vec::with_capacity(rows.len());
    for row in rows.into_iter().rev() {
        let item_id =
            ChatConversationItemId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
                .map_err(identifier_error)?;
        messages.push(read_message(&pool, &item_id).await?);
    }
    Ok(ChatChannelPageRead {
        channel_id,
        messages,
        previous_cursor,
        revision: channel.revision,
    })
}

#[tauri::command]
pub async fn chat_read_reply_thread_page(
    app: tauri::AppHandle,
    db_url: String,
    reply_thread_id: ChatReplyThreadId,
    cursor: Option<String>,
    limit: Option<u32>,
) -> ChatResult<ChatReplyThreadPageRead> {
    let pool = chat_pool(app, db_url).await?;
    read_reply_thread_page(
        &pool,
        &reply_thread_id,
        parse_cursor(cursor.as_deref())?,
        limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE),
    )
    .await
}

#[tauri::command]
pub async fn chat_search_messages(
    app: tauri::AppHandle,
    db_url: String,
    query: String,
    project_id: Option<String>,
    limit: Option<u32>,
) -> ChatResult<Vec<ChatMessageSearchResultRead>> {
    let fts_query = normalized_fts_query(&query)?;
    let pool = chat_pool(app, db_url).await?;
    let rows = sqlx::query(
        "SELECT
            channel.project_id,
            channel.id AS channel_id,
            channel.name AS channel_name,
            search.conversation_id,
            search.reply_thread_id,
            search.message_item_id,
            item.ordinal,
            search.author_display_name,
            snippet(chat_communication_search_fts, 4, '<mark>', '</mark>', '…', 24) AS excerpt,
            item.created_at
         FROM chat_communication_search_fts search
         JOIN chat_conversation_items item ON item.id = search.message_item_id
         JOIN chat_channels channel ON channel.conversation_id = search.conversation_id
         WHERE chat_communication_search_fts MATCH ?
           AND (? IS NULL OR channel.project_id = ?)
         ORDER BY rank, item.created_at DESC
         LIMIT ?",
    )
    .bind(fts_query)
    .bind(project_id.as_deref())
    .bind(project_id.as_deref())
    .bind(i64::from(limit.unwrap_or(50).clamp(1, MAX_SEARCH_RESULTS)))
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(ChatMessageSearchResultRead {
                project_id: row.try_get("project_id").map_err(persistence_error)?,
                channel_id: ChatChannelId::new(
                    row.try_get::<String, _>("channel_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                channel_name: row.try_get("channel_name").map_err(persistence_error)?,
                conversation_id: ChatConversationId::new(
                    row.try_get::<String, _>("conversation_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                reply_thread_id: row
                    .try_get::<Option<String>, _>("reply_thread_id")
                    .map_err(persistence_error)?
                    .map(ChatReplyThreadId::new)
                    .transpose()
                    .map_err(identifier_error)?,
                message_item_id: ChatConversationItemId::new(
                    row.try_get::<String, _>("message_item_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                ordinal: u64_value(row.try_get("ordinal").map_err(persistence_error)?)?,
                author_display_name: row
                    .try_get("author_display_name")
                    .map_err(persistence_error)?,
                excerpt: row.try_get("excerpt").map_err(persistence_error)?,
                created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
            })
        })
        .collect()
}

#[tauri::command]
pub async fn chat_cancel_assignment(
    app: tauri::AppHandle,
    db_url: String,
    assignment_id: ChatWorkAssignmentId,
    expected_revision: u64,
) -> ChatResult<ChatWorkAssignmentRead> {
    let stop_app = app.clone();
    let stop_db_url = db_url.clone();
    let pool = chat_pool(app, db_url).await?;
    let cancelled = set_assignment_state(
        &pool,
        &assignment_id,
        expected_revision,
        ChatWorkAssignmentState::Cancelled,
        Some("Cancelled by the user"),
    )
    .await?;
    let provider_thread_id: Option<String> = sqlx::query_scalar(
        "SELECT provider_thread_id FROM chat_agent_runs
         WHERE assignment_id = ? AND provider_thread_id IS NOT NULL
         ORDER BY run_ordinal DESC LIMIT 1",
    )
    .bind(assignment_id.as_str())
    .fetch_optional(&pool)
    .await
    .map_err(persistence_error)?
    .flatten();
    if let Some(provider_thread_id) = provider_thread_id {
        let thread_id = ChatThreadId::new(provider_thread_id).map_err(identifier_error)?;
        let client_command_id =
            ChatCommandId::new(format!("assignment-cancel:{}", assignment_id.as_str()))
                .map_err(identifier_error)?;
        tauri::async_runtime::spawn(async move {
            let _ = super::interaction_commands::chat_stop_session(
                stop_app,
                stop_db_url,
                thread_id,
                false,
                client_command_id,
            )
            .await;
        });
    }
    Ok(cancelled)
}

#[tauri::command]
pub async fn chat_retry_assignment(
    app: tauri::AppHandle,
    db_url: String,
    assignment_id: ChatWorkAssignmentId,
    expected_revision: u64,
) -> ChatResult<ChatWorkAssignmentRead> {
    let dispatch_app = app.clone();
    let dispatch_db_url = db_url.clone();
    let pool = chat_pool(app, db_url).await?;
    let current = read_assignment(&pool, &assignment_id).await?;
    if !matches!(
        current.state,
        ChatWorkAssignmentState::Failed | ChatWorkAssignmentState::Cancelled
    ) {
        return Err(ChatError::new(
            ChatErrorCode::InvalidStateTransition,
            "Only failed or cancelled work can be retried",
            true,
        ));
    }
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let updated = sqlx::query(
        "UPDATE chat_work_assignments
         SET state = 'queued', state_reason = NULL, settled_at = NULL,
             revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .bind(i64_value(expected_revision)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The assignment changed before retry",
            true,
        ));
    }
    sqlx::query(
        "INSERT INTO chat_assignment_dispatch_jobs
            (id, assignment_id, state, available_at, created_at, updated_at)
         VALUES (?, ?, 'queued', ?, ?, ?)
         ON CONFLICT(assignment_id) DO UPDATE SET
            state = 'queued', available_at = excluded.available_at,
            claimed_at = NULL, claim_token = NULL, last_error = NULL,
            updated_at = excluded.updated_at",
    )
    .bind(new_id("dispatch"))
    .bind(assignment_id.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)?;
    let retried = read_assignment(&pool, &assignment_id).await?;
    tauri::async_runtime::spawn(async move {
        let _ = dispatch_assignment_job(dispatch_app, dispatch_db_url, assignment_id).await;
    });
    Ok(retried)
}

pub(crate) async fn read_memberships_for_conversation(
    pool: &SqlitePool,
    conversation_id: &ChatConversationId,
    include_removed: bool,
) -> ChatResult<Vec<ChatConversationMembershipRead>> {
    let rows = sqlx::query(
        "SELECT participant_id
         FROM chat_conversation_memberships
         WHERE conversation_id = ? AND (? OR removed_at IS NULL)
         ORDER BY membership_role = 'owner' DESC, created_at, participant_id",
    )
    .bind(conversation_id.as_str())
    .bind(include_removed)
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let mut memberships = Vec::with_capacity(rows.len());
    for row in rows {
        let participant_id = ChatParticipantId::new(
            row.try_get::<String, _>("participant_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        memberships.push(read_membership(pool, conversation_id, &participant_id).await?);
    }
    Ok(memberships)
}

async fn read_membership(
    pool: &SqlitePool,
    conversation_id: &ChatConversationId,
    participant_id: &ChatParticipantId,
) -> ChatResult<ChatConversationMembershipRead> {
    let row = sqlx::query(
        "SELECT addressable, approval_policy, revision, removed_at
         FROM chat_conversation_memberships
         WHERE conversation_id = ? AND participant_id = ?",
    )
    .bind(conversation_id.as_str())
    .bind(participant_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Channel membership was not found",
            true,
        )
    })?;
    let grants = sqlx::query(
        "SELECT grant_row.working_folder_id, folder.display_name, grant_row.is_default
         FROM chat_teammate_working_folder_grants grant_row
         JOIN project_working_folders folder ON folder.id = grant_row.working_folder_id
         WHERE grant_row.conversation_id = ? AND grant_row.teammate_id = ?
           AND grant_row.revoked_at IS NULL
         ORDER BY grant_row.is_default DESC, folder.sort_order, folder.display_name, folder.id",
    )
    .bind(conversation_id.as_str())
    .bind(participant_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?
    .into_iter()
    .map(|grant| {
        Ok(ChatWorkingFolderGrantRead {
            working_folder_id: ProjectWorkingFolderId::new(
                grant
                    .try_get::<String, _>("working_folder_id")
                    .map_err(persistence_error)?,
            )
            .map_err(identifier_error)?,
            display_name: grant.try_get("display_name").map_err(persistence_error)?,
            is_default: grant
                .try_get::<i64, _>("is_default")
                .map_err(persistence_error)?
                != 0,
        })
    })
    .collect::<ChatResult<Vec<_>>>()?;
    Ok(ChatConversationMembershipRead {
        conversation_id: conversation_id.clone(),
        participant: read_participant(pool, participant_id).await?,
        addressable: row
            .try_get::<i64, _>("addressable")
            .map_err(persistence_error)?
            != 0,
        approval_policy: parse_approval_policy(
            &row.try_get::<String, _>("approval_policy")
                .map_err(persistence_error)?,
        )?,
        working_folder_grants: grants,
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
        removed_at: optional_timestamp(row.try_get("removed_at").map_err(persistence_error)?)?,
    })
}

async fn read_participant(
    pool: &SqlitePool,
    participant_id: &ChatParticipantId,
) -> ChatResult<ChatParticipantRead> {
    let row = sqlx::query(
        "SELECT participant_kind, display_name, normalized_handle,
                avatar_schema_version, avatar_data, revision, archived_at
         FROM chat_participants WHERE id = ?",
    )
    .bind(participant_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Chat participant was not found",
            true,
        )
    })?;
    Ok(ChatParticipantRead {
        id: participant_id.clone(),
        kind: parse_participant_kind(
            &row.try_get::<String, _>("participant_kind")
                .map_err(persistence_error)?,
        )?,
        display_name: row.try_get("display_name").map_err(persistence_error)?,
        handle: row
            .try_get("normalized_handle")
            .map_err(persistence_error)?,
        avatar: VersionedJson {
            schema_version: u32_value(
                row.try_get("avatar_schema_version")
                    .map_err(persistence_error)?,
            )?,
            value: parse_json(row.try_get("avatar_data").map_err(persistence_error)?)?,
        },
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
        archived_at: optional_timestamp(row.try_get("archived_at").map_err(persistence_error)?)?,
    })
}

async fn read_teammate(
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
) -> ChatResult<ChatAiTeammateRead> {
    let row = sqlx::query(
        "SELECT purpose, instructions, latest_policy_revision, configuration_state,
                (SELECT count(*) FROM chat_conversation_memberships membership
                 WHERE membership.participant_id = teammate.participant_id
                   AND membership.removed_at IS NULL) AS channel_count
         FROM chat_ai_teammates teammate WHERE participant_id = ?",
    )
    .bind(teammate_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "AI teammate was not found", true))?;
    let latest_revision = u64_value(
        row.try_get("latest_policy_revision")
            .map_err(persistence_error)?,
    )?;
    Ok(ChatAiTeammateRead {
        participant: read_participant(pool, teammate_id).await?,
        purpose: row.try_get("purpose").map_err(persistence_error)?,
        instructions: row.try_get("instructions").map_err(persistence_error)?,
        configuration_state: parse_configuration_state(
            &row.try_get::<String, _>("configuration_state")
                .map_err(persistence_error)?,
        )?,
        latest_policy: if latest_revision == 0 {
            None
        } else {
            Some(read_policy(pool, teammate_id, latest_revision).await?)
        },
        channel_count: u64_value(row.try_get("channel_count").map_err(persistence_error)?)?,
    })
}

async fn read_policy(
    pool: &SqlitePool,
    teammate_id: &ChatParticipantId,
    revision: u64,
) -> ChatResult<ChatTeammatePolicyRead> {
    let row = sqlx::query(
        "SELECT id, provider_instance_id, model_selection_schema_version,
                model_selection_data, effort, speed, provider_options_schema_version,
                provider_options_data, created_at
         FROM chat_teammate_policy_revisions
         WHERE teammate_id = ? AND revision = ?",
    )
    .bind(teammate_id.as_str())
    .bind(i64_value(revision)?)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Teammate policy was not found",
            true,
        )
    })?;
    let selection = parse_model_selection(
        row.try_get("model_selection_data")
            .map_err(persistence_error)?,
    )?;
    Ok(ChatTeammatePolicyRead {
        id: ChatTeammatePolicyRevisionId::new(
            row.try_get::<String, _>("id").map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        teammate_id: teammate_id.clone(),
        revision,
        provider_instance_id: ProviderInstanceId::new(
            row.try_get::<String, _>("provider_instance_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        provider_managed_model: selection.provider_managed_model,
        model_id: selection.model_id,
        model_options: selection.model_options,
        effort: row.try_get("effort").map_err(persistence_error)?,
        speed: row.try_get("speed").map_err(persistence_error)?,
        provider_options: VersionedJson {
            schema_version: u32_value(
                row.try_get("provider_options_schema_version")
                    .map_err(persistence_error)?,
            )?,
            value: parse_json(
                row.try_get("provider_options_data")
                    .map_err(persistence_error)?,
            )?,
        },
        created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
    })
}

async fn read_message(
    pool: &SqlitePool,
    item_id: &ChatConversationItemId,
) -> ChatResult<ChatMessageRead> {
    let row = sqlx::query(
        "SELECT item.conversation_id, item.reply_thread_id, item.ordinal, item.created_at,
                message.author_participant_id, message.current_revision_id,
                message.edited_at, revision.revision, revision.normalized_markdown,
                revision.rich_content_schema_version, revision.rich_content_data
         FROM chat_conversation_items item
         JOIN chat_communication_messages message ON message.item_id = item.id
         JOIN chat_communication_message_revisions revision
           ON revision.id = message.current_revision_id
         WHERE item.id = ?",
    )
    .bind(item_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Chat message was not found", true))?;
    let revision_id = ChatMessageRevisionId::new(
        row.try_get::<String, _>("current_revision_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let participant_id = ChatParticipantId::new(
        row.try_get::<String, _>("author_participant_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let mentions = read_mentions(pool, &revision_id).await?;
    let attachment_ids = sqlx::query_scalar::<_, String>(
        "SELECT attachment_id FROM chat_communication_attachment_references
         WHERE message_revision_id = ? ORDER BY ordinal",
    )
    .bind(revision_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?
    .into_iter()
    .map(ChatAttachmentId::new)
    .collect::<Result<Vec<_>, _>>()
    .map_err(identifier_error)?;
    let resource_references = read_resource_references(pool, &revision_id).await?;
    let reply_thread_id = row
        .try_get::<Option<String>, _>("reply_thread_id")
        .map_err(persistence_error)?
        .map(ChatReplyThreadId::new)
        .transpose()
        .map_err(identifier_error)?;
    let root_thread_id = if reply_thread_id.is_none() {
        sqlx::query_scalar::<_, String>("SELECT id FROM chat_reply_threads WHERE root_item_id = ?")
            .bind(item_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?
            .map(ChatReplyThreadId::new)
            .transpose()
            .map_err(identifier_error)?
    } else {
        None
    };
    Ok(ChatMessageRead {
        item_id: item_id.clone(),
        conversation_id: ChatConversationId::new(
            row.try_get::<String, _>("conversation_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        reply_thread_id,
        revision_id,
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
        author: read_participant(pool, &participant_id).await?,
        normalized_markdown: row
            .try_get("normalized_markdown")
            .map_err(persistence_error)?,
        rich_content: VersionedJson {
            schema_version: u32_value(
                row.try_get("rich_content_schema_version")
                    .map_err(persistence_error)?,
            )?,
            value: parse_json(
                row.try_get("rich_content_data")
                    .map_err(persistence_error)?,
            )?,
        },
        mentions,
        attachment_ids,
        resource_references,
        reply_thread: match root_thread_id {
            Some(thread_id) => Some(read_reply_thread_summary(pool, &thread_id).await?),
            None => None,
        },
        ordinal: u64_value(row.try_get("ordinal").map_err(persistence_error)?)?,
        edited_at: optional_timestamp(row.try_get("edited_at").map_err(persistence_error)?)?,
        created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
    })
}

async fn read_reply_thread_summary(
    pool: &SqlitePool,
    reply_thread_id: &ChatReplyThreadId,
) -> ChatResult<ChatReplyThreadSummaryRead> {
    let row = sqlx::query(
        "SELECT reply_count, last_activity_at
         FROM chat_reply_threads WHERE id = ?",
    )
    .bind(reply_thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Reply thread was not found", true))?;
    let participants = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT message.author_participant_id
         FROM chat_conversation_items item
         JOIN chat_communication_messages message ON message.item_id = item.id
         WHERE item.reply_thread_id = ?
         ORDER BY item.ordinal DESC LIMIT 3",
    )
    .bind(reply_thread_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let mut participant_reads = Vec::with_capacity(participants.len());
    for participant_id in participants {
        participant_reads.push(
            read_participant(
                pool,
                &ChatParticipantId::new(participant_id).map_err(identifier_error)?,
            )
            .await?,
        );
    }
    let reply_count = u64_value(row.try_get("reply_count").map_err(persistence_error)?)?;
    let read_ordinal: i64 = sqlx::query_scalar(
        "SELECT coalesce((
            SELECT last_read_reply_ordinal FROM chat_reply_thread_read_cursors
            WHERE reply_thread_id = ? AND participant_id = ?
         ), 0)",
    )
    .bind(reply_thread_id.as_str())
    .bind(LOCAL_PARTICIPANT_ID)
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    Ok(ChatReplyThreadSummaryRead {
        id: reply_thread_id.clone(),
        reply_count,
        last_activity_at: timestamp(row.try_get("last_activity_at").map_err(persistence_error)?)?,
        participants: participant_reads,
        unread: reply_count > u64_value(read_ordinal)?,
        work_state: read_active_or_latest_assignment(pool, reply_thread_id)
            .await?
            .map(|assignment| assignment.state),
    })
}

async fn read_reply_thread_page(
    pool: &SqlitePool,
    reply_thread_id: &ChatReplyThreadId,
    before: Option<i64>,
    limit: u32,
) -> ChatResult<ChatReplyThreadPageRead> {
    let row = sqlx::query("SELECT root_item_id, revision FROM chat_reply_threads WHERE id = ?")
        .bind(reply_thread_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(ChatErrorCode::NotFound, "Reply thread was not found", true)
        })?;
    let root_id = ChatConversationItemId::new(
        row.try_get::<String, _>("root_item_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let reply_rows = sqlx::query(
        "SELECT id, ordinal FROM chat_conversation_items
         WHERE reply_thread_id = ? AND item_kind = 'message'
           AND (? IS NULL OR ordinal < ?)
         ORDER BY ordinal DESC LIMIT ?",
    )
    .bind(reply_thread_id.as_str())
    .bind(before)
    .bind(before)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    let previous_cursor = reply_rows
        .last()
        .map(|reply| reply.try_get::<i64, _>("ordinal"))
        .transpose()
        .map_err(persistence_error)?
        .filter(|_| reply_rows.len() == limit as usize)
        .map(|ordinal| ordinal.to_string());
    let mut replies = Vec::with_capacity(reply_rows.len());
    for reply in reply_rows.into_iter().rev() {
        let item_id = ChatConversationItemId::new(
            reply
                .try_get::<String, _>("id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?;
        replies.push(read_message(pool, &item_id).await?);
    }
    let runs = read_agent_runs(pool, reply_thread_id).await?;
    let summary = read_reply_thread_summary(pool, reply_thread_id).await?;
    sqlx::query(
        "INSERT INTO chat_reply_thread_read_cursors
            (reply_thread_id, participant_id, last_read_reply_ordinal, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(reply_thread_id, participant_id) DO UPDATE SET
            last_read_reply_ordinal = max(
                last_read_reply_ordinal,
                excluded.last_read_reply_ordinal
            ),
            updated_at = excluded.updated_at",
    )
    .bind(reply_thread_id.as_str())
    .bind(LOCAL_PARTICIPANT_ID)
    .bind(i64_value(summary.reply_count)?)
    .bind(now_timestamp()?.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(ChatReplyThreadPageRead {
        thread: ChatReplyThreadSummaryRead {
            unread: false,
            ..summary
        },
        root_message: read_message(pool, &root_id).await?,
        replies,
        assignment: read_active_or_latest_assignment(pool, reply_thread_id).await?,
        agent_runs: runs,
        previous_cursor,
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
    })
}

async fn read_assignment(
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
) -> ChatResult<ChatWorkAssignmentRead> {
    let row = sqlx::query(
        "SELECT reply_thread_id, teammate_id, triggering_message_item_id,
                previous_assignment_id, state, state_reason, revision, settled_at,
                created_at, updated_at
         FROM chat_work_assignments WHERE id = ?",
    )
    .bind(assignment_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Work assignment was not found",
            true,
        )
    })?;
    let teammate_id = ChatParticipantId::new(
        row.try_get::<String, _>("teammate_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    Ok(ChatWorkAssignmentRead {
        id: assignment_id.clone(),
        reply_thread_id: ChatReplyThreadId::new(
            row.try_get::<String, _>("reply_thread_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        teammate: read_participant(pool, &teammate_id).await?,
        triggering_message_item_id: ChatConversationItemId::new(
            row.try_get::<String, _>("triggering_message_item_id")
                .map_err(persistence_error)?,
        )
        .map_err(identifier_error)?,
        previous_assignment_id: row
            .try_get::<Option<String>, _>("previous_assignment_id")
            .map_err(persistence_error)?
            .map(ChatWorkAssignmentId::new)
            .transpose()
            .map_err(identifier_error)?,
        state: parse_work_state(
            &row.try_get::<String, _>("state")
                .map_err(persistence_error)?,
        )?,
        state_reason: row.try_get("state_reason").map_err(persistence_error)?,
        revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
        settled_at: optional_timestamp(row.try_get("settled_at").map_err(persistence_error)?)?,
        created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
        updated_at: timestamp(row.try_get("updated_at").map_err(persistence_error)?)?,
    })
}

async fn read_active_or_latest_assignment(
    pool: &SqlitePool,
    reply_thread_id: &ChatReplyThreadId,
) -> ChatResult<Option<ChatWorkAssignmentRead>> {
    let id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM chat_work_assignments
         WHERE reply_thread_id = ?
         ORDER BY
           CASE WHEN state IN (
             'queued', 'working', 'waiting_for_answer', 'waiting_for_approval', 'ready_for_review'
           ) THEN 0 ELSE 1 END,
           created_at DESC
         LIMIT 1",
    )
    .bind(reply_thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    match id {
        Some(id) => Ok(Some(
            read_assignment(
                pool,
                &ChatWorkAssignmentId::new(id).map_err(identifier_error)?,
            )
            .await?,
        )),
        None => Ok(None),
    }
}

async fn read_agent_runs(
    pool: &SqlitePool,
    reply_thread_id: &ChatReplyThreadId,
) -> ChatResult<Vec<ChatAgentRunRead>> {
    let rows = sqlx::query(
        "SELECT run.id, run.assignment_id, run.project_id, run.working_folder_id,
                run.teammate_policy_revision_id, policy.effort, run.provider_turn_id,
                run.provider_thread_id, run.state,
                run.run_ordinal, run.created_at, run.updated_at
         FROM chat_agent_runs run
         JOIN chat_work_assignments assignment ON assignment.id = run.assignment_id
         JOIN chat_teammate_policy_revisions policy ON policy.id = run.teammate_policy_revision_id
         WHERE assignment.reply_thread_id = ?
         ORDER BY assignment.created_at, run.run_ordinal",
    )
    .bind(reply_thread_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(ChatAgentRunRead {
                id: ChatAgentRunId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
                    .map_err(identifier_error)?,
                assignment_id: ChatWorkAssignmentId::new(
                    row.try_get::<String, _>("assignment_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                project_id: row.try_get("project_id").map_err(persistence_error)?,
                working_folder_id: ProjectWorkingFolderId::new(
                    row.try_get::<String, _>("working_folder_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                teammate_policy_revision_id: ChatTeammatePolicyRevisionId::new(
                    row.try_get::<String, _>("teammate_policy_revision_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                effort: row.try_get("effort").map_err(persistence_error)?,
                provider_execution_turn_id: ChatTurnId::new(
                    row.try_get::<String, _>("provider_turn_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                provider_execution_thread_id: row
                    .try_get::<Option<String>, _>("provider_thread_id")
                    .map_err(persistence_error)?
                    .map(ChatThreadId::new)
                    .transpose()
                    .map_err(identifier_error)?,
                state: row.try_get("state").map_err(persistence_error)?,
                run_ordinal: u64_value(row.try_get("run_ordinal").map_err(persistence_error)?)?,
                created_at: timestamp(row.try_get("created_at").map_err(persistence_error)?)?,
                updated_at: timestamp(row.try_get("updated_at").map_err(persistence_error)?)?,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct ResolvedInvocation {
    teammate_id: ChatParticipantId,
    active_assignment: Option<ChatWorkAssignmentRead>,
    latest_assignment: Option<ChatWorkAssignmentRead>,
}

struct AssignmentWrite {
    assignment_id: Option<ChatWorkAssignmentId>,
    input_queued: bool,
}

async fn resolve_invoked_teammate(
    pool: &SqlitePool,
    conversation_id: &ChatConversationId,
    reply_thread_id: Option<&ChatReplyThreadId>,
    mentions: &[ChatParticipantMentionInput],
    post_without_invoking: bool,
) -> ChatResult<Option<ResolvedInvocation>> {
    let mut ai_mentions = Vec::new();
    for mention in mentions {
        let participant = read_participant(pool, &mention.participant_id).await?;
        if participant.kind != mention.participant_kind {
            return Err(ChatError::validation(
                "participantMentions",
                "A participant mention has stale identity data",
            ));
        }
        if participant.kind == ChatParticipantKind::AiTeammate {
            ai_mentions.push(mention.participant_id.clone());
        }
    }
    ai_mentions.sort();
    ai_mentions.dedup();
    if ai_mentions.len() > 1 {
        return Err(ChatError::validation(
            "participantMentions",
            "Assign one AI teammate at a time",
        ));
    }
    let latest_assignment = match reply_thread_id {
        Some(reply_thread_id) => read_active_or_latest_assignment(pool, reply_thread_id).await?,
        None => None,
    };
    let active_assignment = latest_assignment
        .as_ref()
        .filter(|assignment| assignment.state.is_active())
        .cloned();
    if post_without_invoking {
        return Ok(None);
    }
    let teammate_id = ai_mentions.into_iter().next().or_else(|| {
        latest_assignment
            .as_ref()
            .map(|assignment| assignment.teammate.id.clone())
    });
    let Some(teammate_id) = teammate_id else {
        return Ok(None);
    };
    if active_assignment
        .as_ref()
        .is_some_and(|assignment| assignment.teammate.id != teammate_id)
    {
        return Err(ChatError::validation(
            "participantMentions",
            "This reply thread already has an active AI teammate",
        ));
    }
    require_addressable_teammate(pool, conversation_id, &teammate_id).await?;
    Ok(Some(ResolvedInvocation {
        teammate_id,
        active_assignment,
        latest_assignment,
    }))
}

async fn persist_assignment_routing(
    transaction: &mut Transaction<'_, Sqlite>,
    reply_thread_id: &ChatReplyThreadId,
    message_item_id: &ChatConversationItemId,
    invocation: Option<&ResolvedInvocation>,
    now: &UtcTimestamp,
) -> ChatResult<AssignmentWrite> {
    let Some(invocation) = invocation else {
        return Ok(AssignmentWrite {
            assignment_id: None,
            input_queued: false,
        });
    };
    if let Some(active) = &invocation.active_assignment {
        let ordinal: i64 = sqlx::query_scalar(
            "SELECT coalesce(max(ordinal), 0) + 1
             FROM chat_work_assignment_inputs WHERE assignment_id = ?",
        )
        .bind(active.id.as_str())
        .fetch_one(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        let routing_kind = if active.state == ChatWorkAssignmentState::Working {
            "steer"
        } else {
            "queued_continuation"
        };
        sqlx::query(
            "INSERT INTO chat_work_assignment_inputs
                (id, assignment_id, message_item_id, ordinal, routing_kind, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(new_id("assignment-input"))
        .bind(active.id.as_str())
        .bind(message_item_id.as_str())
        .bind(ordinal)
        .bind(routing_kind)
        .bind(now.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        return Ok(AssignmentWrite {
            assignment_id: Some(active.id.clone()),
            input_queued: true,
        });
    }
    let assignment_id = work_assignment_id()?;
    let previous_id = invocation
        .latest_assignment
        .as_ref()
        .map(|assignment| &assignment.id);
    sqlx::query(
        "INSERT INTO chat_work_assignments
            (id, reply_thread_id, teammate_id, triggering_message_item_id,
             previous_assignment_id, state, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'queued', ?, ?)",
    )
    .bind(assignment_id.as_str())
    .bind(reply_thread_id.as_str())
    .bind(invocation.teammate_id.as_str())
    .bind(message_item_id.as_str())
    .bind(previous_id.map(ChatWorkAssignmentId::as_str))
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_work_assignment_inputs
            (id, assignment_id, message_item_id, ordinal, routing_kind, created_at)
         VALUES (?, ?, ?, 1, ?, ?)",
    )
    .bind(new_id("assignment-input"))
    .bind(assignment_id.as_str())
    .bind(message_item_id.as_str())
    .bind(if previous_id.is_some() {
        "follow_up"
    } else {
        "trigger"
    })
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    freeze_context_package(
        transaction,
        &assignment_id,
        reply_thread_id,
        message_item_id,
        &invocation.teammate_id,
        now,
    )
    .await?;
    sqlx::query(
        "INSERT INTO chat_assignment_dispatch_jobs
            (id, assignment_id, state, available_at, created_at, updated_at)
         VALUES (?, ?, 'queued', ?, ?, ?)",
    )
    .bind(new_id("dispatch"))
    .bind(assignment_id.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(AssignmentWrite {
        assignment_id: Some(assignment_id),
        input_queued: false,
    })
}

async fn freeze_context_package(
    transaction: &mut Transaction<'_, Sqlite>,
    assignment_id: &ChatWorkAssignmentId,
    reply_thread_id: &ChatReplyThreadId,
    triggering_message_item_id: &ChatConversationItemId,
    teammate_id: &ChatParticipantId,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let thread_row = sqlx::query(
        "SELECT thread.conversation_id, thread.root_item_id, channel.project_id
         FROM chat_reply_threads thread
         JOIN chat_channels channel ON channel.conversation_id = thread.conversation_id
         WHERE thread.id = ?",
    )
    .bind(reply_thread_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let conversation_id = ChatConversationId::new(
        thread_row
            .try_get::<String, _>("conversation_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let project_id: String = thread_row
        .try_get("project_id")
        .map_err(persistence_error)?;
    let root_item_id = ChatConversationItemId::new(
        thread_row
            .try_get::<String, _>("root_item_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let source_rows = sqlx::query(
        "SELECT item.id, item.ordinal, revision.revision, revision.normalized_markdown,
                CASE
                  WHEN item.id = ? THEN 'trigger'
                  WHEN item.id = ? THEN 'thread_root'
                  ELSE 'thread_reply'
                END AS source_kind
         FROM chat_conversation_items item
         JOIN chat_communication_messages message ON message.item_id = item.id
         JOIN chat_communication_message_revisions revision
           ON revision.id = message.current_revision_id
         WHERE item.reply_thread_id = ? OR item.id IN (?, ?)
         ORDER BY CASE WHEN item.id = ? THEN 0 ELSE 1 END, item.ordinal DESC
         LIMIT ?",
    )
    .bind(triggering_message_item_id.as_str())
    .bind(root_item_id.as_str())
    .bind(reply_thread_id.as_str())
    .bind(root_item_id.as_str())
    .bind(triggering_message_item_id.as_str())
    .bind(triggering_message_item_id.as_str())
    .bind(i64::try_from(MAX_THREAD_CONTEXT_REPLIES + 2).unwrap_or(52))
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let channel_rows = sqlx::query(
        "SELECT item.id, item.ordinal, revision.revision, revision.normalized_markdown
         FROM chat_conversation_items item
         JOIN chat_communication_messages message ON message.item_id = item.id
         JOIN chat_communication_message_revisions revision
           ON revision.id = message.current_revision_id
         WHERE item.conversation_id = ? AND item.reply_thread_id IS NULL
           AND item.id != ?
         ORDER BY item.ordinal DESC LIMIT ?",
    )
    .bind(conversation_id.as_str())
    .bind(root_item_id.as_str())
    .bind(i64::try_from(MAX_CHANNEL_CONTEXT_MESSAGES).unwrap_or(20))
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let thread_total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat_conversation_items WHERE reply_thread_id = ?",
    )
    .bind(reply_thread_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let channel_total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat_conversation_items
         WHERE conversation_id = ? AND reply_thread_id IS NULL AND id != ?",
    )
    .bind(conversation_id.as_str())
    .bind(root_item_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let mut sources = Vec::new();
    for row in source_rows.into_iter().rev() {
        sources.push(ContextSource {
            id: row.try_get("id").map_err(persistence_error)?,
            kind: row.try_get("source_kind").map_err(persistence_error)?,
            revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
            text: row
                .try_get("normalized_markdown")
                .map_err(persistence_error)?,
        });
    }
    for row in channel_rows.into_iter().rev() {
        sources.push(ContextSource {
            id: row.try_get("id").map_err(persistence_error)?,
            kind: "channel_message".to_string(),
            revision: u64_value(row.try_get("revision").map_err(persistence_error)?)?,
            text: row
                .try_get("normalized_markdown")
                .map_err(persistence_error)?,
        });
    }
    let mut serialized = String::new();
    for source in &sources {
        let line = format!("[{}:{}]\n{}\n\n", source.kind, source.id, source.text);
        if serialized.len() + line.len() > MAX_MESSAGE_BYTES {
            if source.id == triggering_message_item_id.as_str() {
                serialized = truncate_utf8(&line, MAX_MESSAGE_BYTES).to_string();
            }
            continue;
        }
        serialized.push_str(&line);
    }
    let context_id = new_id("context");
    let context_hash = sha256_hex(serialized.as_bytes());
    sqlx::query(
        "INSERT INTO chat_assignment_context_packages
            (id, assignment_id, revision, triggering_message_item_id,
             serialized_text, serialized_bytes, excluded_thread_reply_count,
             excluded_channel_message_count, sha256, created_at)
         VALUES (?, ?, 1, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&context_id)
    .bind(assignment_id.as_str())
    .bind(triggering_message_item_id.as_str())
    .bind(&serialized)
    .bind(i64::try_from(serialized.len()).unwrap_or(i64::MAX))
    .bind((thread_total - i64::try_from(MAX_THREAD_CONTEXT_REPLIES).unwrap_or(50)).max(0))
    .bind((channel_total - i64::try_from(MAX_CHANNEL_CONTEXT_MESSAGES).unwrap_or(20)).max(0))
    .bind(context_hash)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for (ordinal, source) in sources.iter().enumerate() {
        sqlx::query(
            "INSERT OR IGNORE INTO chat_assignment_context_sources
                (context_package_id, source_kind, source_id, source_revision,
                 content_sha256, ordinal)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&context_id)
        .bind(&source.kind)
        .bind(&source.id)
        .bind(i64_value(source.revision)?)
        .bind(sha256_hex(source.text.as_bytes()))
        .bind(i64::try_from(ordinal).unwrap_or(i64::MAX))
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    let authorization = sqlx::query(
        "SELECT membership.approval_policy, grant_row.working_folder_id
         FROM chat_conversation_memberships membership
         JOIN chat_teammate_working_folder_grants grant_row
           ON grant_row.conversation_id = membership.conversation_id
          AND grant_row.teammate_id = membership.participant_id
          AND grant_row.is_default = 1
          AND grant_row.revoked_at IS NULL
         WHERE membership.conversation_id = ?
           AND membership.participant_id = ?
           AND membership.removed_at IS NULL",
    )
    .bind(conversation_id.as_str())
    .bind(teammate_id.as_str())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::Permission,
            "The teammate needs a default working folder in this channel",
            true,
        )
    })?;
    let approval_policy: String = authorization
        .try_get("approval_policy")
        .map_err(persistence_error)?;
    let default_working_folder_id: String = authorization
        .try_get("working_folder_id")
        .map_err(persistence_error)?;
    let policy_revision: i64 = sqlx::query_scalar(
        "SELECT latest_policy_revision FROM chat_ai_teammates WHERE participant_id = ?",
    )
    .bind(teammate_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    if policy_revision < 1 {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The teammate needs a published execution policy",
            true,
        ));
    }
    let policy_id: String = sqlx::query_scalar(
        "SELECT id FROM chat_teammate_policy_revisions
         WHERE teammate_id = ? AND revision = ?",
    )
    .bind(teammate_id.as_str())
    .bind(policy_revision)
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_assignment_authorization_decisions
            (id, assignment_id, context_package_id, teammate_policy_revision_id,
             conversation_id, working_folder_id, approval_policy, decision_state,
             reason, policy_data, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'allowed', '', '{}', ?)",
    )
    .bind(new_id("authorization"))
    .bind(assignment_id.as_str())
    .bind(&context_id)
    .bind(policy_id)
    .bind(conversation_id.as_str())
    .bind(default_working_folder_id)
    .bind(approval_policy)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let _ = project_id;
    Ok(())
}

struct ContextSource {
    id: String,
    kind: String,
    revision: u64,
    text: String,
}

struct CommunicationMessageWrite<'a> {
    item_id: &'a ChatConversationItemId,
    revision_id: &'a ChatMessageRevisionId,
    conversation_id: &'a ChatConversationId,
    reply_thread_id: Option<&'a ChatReplyThreadId>,
    ordinal: i64,
    request: &'a PostChatMessageCommand,
    now: &'a UtcTimestamp,
}

async fn insert_communication_message(
    transaction: &mut Transaction<'_, Sqlite>,
    write: CommunicationMessageWrite<'_>,
) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_conversation_items
            (id, conversation_id, reply_thread_id, item_kind, ordinal, created_at)
         VALUES (?, ?, ?, 'message', ?, ?)",
    )
    .bind(write.item_id.as_str())
    .bind(write.conversation_id.as_str())
    .bind(write.reply_thread_id.map(ChatReplyThreadId::as_str))
    .bind(write.ordinal)
    .bind(write.now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_communication_messages
            (item_id, author_participant_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(write.item_id.as_str())
    .bind(LOCAL_PARTICIPANT_ID)
    .bind(write.now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_communication_message_revisions
            (id, message_item_id, revision, normalized_markdown,
             rich_content_schema_version, rich_content_data, created_at)
         VALUES (?, ?, 1, ?, ?, ?, ?)",
    )
    .bind(write.revision_id.as_str())
    .bind(write.item_id.as_str())
    .bind(write.request.normalized_markdown.trim())
    .bind(i64::from(write.request.rich_content.schema_version))
    .bind(json_object(&write.request.rich_content, "richContent")?)
    .bind(write.now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for (index, mention) in write.request.participant_mentions.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_participant_mentions
                (id, message_revision_id, participant_id, participant_kind,
                 handle_snapshot, label_snapshot, start_offset, end_offset)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("mention:{}:{index}", write.revision_id.as_str()))
        .bind(write.revision_id.as_str())
        .bind(mention.participant_id.as_str())
        .bind(wire_participant_kind(mention.participant_kind))
        .bind(mention.handle_snapshot.as_deref())
        .bind(&mention.label_snapshot)
        .bind(i64_value(mention.start_offset)?)
        .bind(i64_value(mention.end_offset)?)
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    for (index, attachment_id) in write.request.attachment_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_communication_attachment_references
                (message_revision_id, attachment_id, ordinal, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(write.revision_id.as_str())
        .bind(attachment_id.as_str())
        .bind(i64::try_from(index).unwrap_or(i64::MAX))
        .bind(write.now.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    for (index, resource) in write.request.resource_references.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_communication_resource_references
                (id, message_revision_id, working_folder_id, reference_kind,
                 relative_path, display_label, ordinal)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!(
            "resource-reference:{}:{index}",
            write.revision_id.as_str()
        ))
        .bind(write.revision_id.as_str())
        .bind(resource.working_folder_id.as_str())
        .bind(&resource.kind)
        .bind(&resource.relative_path)
        .bind(&resource.display_label)
        .bind(i64::try_from(index).unwrap_or(i64::MAX))
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    sqlx::query("UPDATE chat_communication_messages SET current_revision_id = ? WHERE item_id = ?")
        .bind(write.revision_id.as_str())
        .bind(write.item_id.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    Ok(())
}

async fn insert_channel_copy(
    transaction: &mut Transaction<'_, Sqlite>,
    conversation_id: &ChatConversationId,
    request: &PostChatMessageCommand,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let item_id = conversation_item_id()?;
    let revision_id = message_revision_id()?;
    let ordinal = next_item_ordinal(transaction, conversation_id, None).await?;
    insert_communication_message(
        transaction,
        CommunicationMessageWrite {
            item_id: &item_id,
            revision_id: &revision_id,
            conversation_id,
            reply_thread_id: None,
            ordinal,
            request,
            now,
        },
    )
    .await?;
    let thread_id = reply_thread_id()?;
    sqlx::query(
        "INSERT INTO chat_reply_threads
            (id, conversation_id, root_item_id, last_activity_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(thread_id.as_str())
    .bind(conversation_id.as_str())
    .bind(item_id.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn next_item_ordinal(
    transaction: &mut Transaction<'_, Sqlite>,
    conversation_id: &ChatConversationId,
    reply_thread_id: Option<&ChatReplyThreadId>,
) -> ChatResult<i64> {
    sqlx::query_scalar(
        "SELECT coalesce(max(ordinal), 0) + 1
         FROM chat_conversation_items
         WHERE conversation_id = ?
           AND ((? IS NULL AND reply_thread_id IS NULL) OR reply_thread_id = ?)",
    )
    .bind(conversation_id.as_str())
    .bind(reply_thread_id.map(ChatReplyThreadId::as_str))
    .bind(reply_thread_id.map(ChatReplyThreadId::as_str))
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)
}

async fn require_reply_thread(
    pool: &SqlitePool,
    reply_thread_id: &ChatReplyThreadId,
    conversation_id: &ChatConversationId,
) -> ChatResult<ChatReplyThreadId> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM chat_reply_threads WHERE id = ? AND conversation_id = ?
         )",
    )
    .bind(reply_thread_id.as_str())
    .bind(conversation_id.as_str())
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if exists == 0 {
        return Err(ChatError::validation(
            "replyThreadId",
            "Reply thread does not belong to the selected channel",
        ));
    }
    Ok(reply_thread_id.clone())
}

async fn require_addressable_teammate(
    pool: &SqlitePool,
    conversation_id: &ChatConversationId,
    teammate_id: &ChatParticipantId,
) -> ChatResult<()> {
    let row = sqlx::query(
        "SELECT membership.addressable, membership.removed_at,
                participant.archived_at, teammate.configuration_state,
                teammate.latest_policy_revision,
                (SELECT count(*) FROM chat_teammate_working_folder_grants grant_row
                 WHERE grant_row.conversation_id = membership.conversation_id
                   AND grant_row.teammate_id = membership.participant_id
                   AND grant_row.is_default = 1 AND grant_row.revoked_at IS NULL) AS defaults
         FROM chat_conversation_memberships membership
         JOIN chat_participants participant ON participant.id = membership.participant_id
         JOIN chat_ai_teammates teammate ON teammate.participant_id = membership.participant_id
         WHERE membership.conversation_id = ? AND membership.participant_id = ?",
    )
    .bind(conversation_id.as_str())
    .bind(teammate_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::validation(
            "participantMentions",
            "The AI teammate is not a member of this channel",
        )
    })?;
    let available = row
        .try_get::<i64, _>("addressable")
        .map_err(persistence_error)?
        != 0
        && row
            .try_get::<Option<String>, _>("removed_at")
            .map_err(persistence_error)?
            .is_none()
        && row
            .try_get::<Option<String>, _>("archived_at")
            .map_err(persistence_error)?
            .is_none()
        && row
            .try_get::<String, _>("configuration_state")
            .map_err(persistence_error)?
            == "healthy"
        && row
            .try_get::<i64, _>("latest_policy_revision")
            .map_err(persistence_error)?
            > 0
        && row
            .try_get::<i64, _>("defaults")
            .map_err(persistence_error)?
            == 1;
    if !available {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The AI teammate needs setup before it can be assigned",
            true,
        ));
    }
    Ok(())
}

async fn upsert_membership_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    teammate_id: &ChatParticipantId,
    membership: &ChatTeammateMembershipInput,
    expected_revision: Option<u64>,
    now: &UtcTimestamp,
) -> ChatResult<ChatConversationId> {
    if !membership
        .working_folder_ids
        .iter()
        .any(|id| id == &membership.default_working_folder_id)
    {
        return Err(ChatError::validation(
            "defaultWorkingFolderId",
            "The default folder must be included in the allowed folders",
        ));
    }
    let unique = membership
        .working_folder_ids
        .iter()
        .map(ProjectWorkingFolderId::as_str)
        .collect::<BTreeSet<_>>();
    if unique.len() != membership.working_folder_ids.len() || unique.is_empty() {
        return Err(ChatError::validation(
            "workingFolderIds",
            "Choose one or more distinct allowed working folders",
        ));
    }
    let row = sqlx::query(
        "SELECT channel.conversation_id, channel.project_id
         FROM chat_channels channel
         WHERE channel.id = ? AND channel.archived_at IS NULL",
    )
    .bind(membership.channel_id.as_str())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Active Chat channel was not found",
            true,
        )
    })?;
    let conversation_id = ChatConversationId::new(
        row.try_get::<String, _>("conversation_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let project_id: String = row.try_get("project_id").map_err(persistence_error)?;
    let teammate_exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM chat_ai_teammates teammate
            JOIN chat_participants participant ON participant.id = teammate.participant_id
            WHERE teammate.participant_id = ? AND participant.archived_at IS NULL
         )",
    )
    .bind(teammate_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    if teammate_exists == 0 {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "AI teammate was not found",
            true,
        ));
    }
    for folder_id in &membership.working_folder_ids {
        let folder_exists: i64 = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM project_working_folders
                WHERE id = ? AND project_id = ? AND archived_at IS NULL
             )",
        )
        .bind(folder_id.as_str())
        .bind(&project_id)
        .fetch_one(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        if folder_exists == 0 {
            return Err(ChatError::validation(
                "workingFolderIds",
                "Every allowed folder must be active and belong to the channel project",
            ));
        }
    }
    match expected_revision {
        Some(expected_revision) => {
            let updated = sqlx::query(
                "UPDATE chat_conversation_memberships
                 SET addressable = ?, approval_policy = ?, removed_at = NULL,
                     revision = revision + 1, updated_at = ?
                 WHERE conversation_id = ? AND participant_id = ? AND revision = ?",
            )
            .bind(membership.addressable)
            .bind(wire_approval_policy(membership.approval_policy))
            .bind(now.as_str())
            .bind(conversation_id.as_str())
            .bind(teammate_id.as_str())
            .bind(i64_value(expected_revision)?)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            if updated.rows_affected() != 1 {
                return Err(ChatError::new(
                    ChatErrorCode::StaleRevision,
                    "The channel membership changed before the update",
                    true,
                ));
            }
        }
        None => {
            sqlx::query(
                "INSERT INTO chat_conversation_memberships
                    (conversation_id, participant_id, membership_role, addressable,
                     approval_policy, created_at, updated_at)
                 VALUES (?, ?, 'member', ?, ?, ?, ?)
                 ON CONFLICT(conversation_id, participant_id) DO UPDATE SET
                    addressable = excluded.addressable,
                    approval_policy = excluded.approval_policy,
                    removed_at = NULL,
                    revision = chat_conversation_memberships.revision + 1,
                    updated_at = excluded.updated_at",
            )
            .bind(conversation_id.as_str())
            .bind(teammate_id.as_str())
            .bind(membership.addressable)
            .bind(wire_approval_policy(membership.approval_policy))
            .bind(now.as_str())
            .bind(now.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
        }
    }
    sqlx::query(
        "UPDATE chat_teammate_working_folder_grants
         SET revoked_at = ?, is_default = 0
         WHERE conversation_id = ? AND teammate_id = ? AND revoked_at IS NULL",
    )
    .bind(now.as_str())
    .bind(conversation_id.as_str())
    .bind(teammate_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for folder_id in &membership.working_folder_ids {
        sqlx::query(
            "INSERT INTO chat_teammate_working_folder_grants
                (conversation_id, teammate_id, project_id, working_folder_id,
                 is_default, created_at, revoked_at)
             VALUES (?, ?, ?, ?, ?, ?, NULL)
             ON CONFLICT(conversation_id, teammate_id, working_folder_id) DO UPDATE SET
                project_id = excluded.project_id,
                is_default = excluded.is_default,
                revoked_at = NULL",
        )
        .bind(conversation_id.as_str())
        .bind(teammate_id.as_str())
        .bind(&project_id)
        .bind(folder_id.as_str())
        .bind(folder_id == &membership.default_working_folder_id)
        .bind(now.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    Ok(conversation_id)
}

async fn insert_policy_revision(
    transaction: &mut Transaction<'_, Sqlite>,
    teammate_id: &ChatParticipantId,
    revision: u64,
    policy: &ChatTeammatePolicyInput,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let selection = serde_json::to_string(&StoredModelSelection {
        provider_managed_model: policy.provider_managed_model,
        model_id: policy.model_id.clone(),
        model_options: policy.model_options.clone(),
    })
    .map_err(serialization_error)?;
    sqlx::query(
        "INSERT INTO chat_teammate_policy_revisions
            (id, teammate_id, revision, provider_instance_id,
             model_selection_schema_version, model_selection_data, effort, speed,
             provider_options_schema_version, provider_options_data,
             interaction_mode, created_at)
         VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, 'build', ?)",
    )
    .bind(new_id("teammate-policy"))
    .bind(teammate_id.as_str())
    .bind(i64_value(revision)?)
    .bind(policy.provider_instance_id.as_str())
    .bind(selection)
    .bind(policy.effort.as_deref())
    .bind(policy.speed.as_deref())
    .bind(i64::from(policy.provider_options.schema_version))
    .bind(json_object(&policy.provider_options, "providerOptions")?)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn read_mentions(
    pool: &SqlitePool,
    revision_id: &ChatMessageRevisionId,
) -> ChatResult<Vec<ChatParticipantMentionRead>> {
    let rows = sqlx::query(
        "SELECT participant_id, participant_kind, handle_snapshot,
                label_snapshot, start_offset, end_offset
         FROM chat_participant_mentions WHERE message_revision_id = ?
         ORDER BY start_offset",
    )
    .bind(revision_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(ChatParticipantMentionRead {
                participant_id: ChatParticipantId::new(
                    row.try_get::<String, _>("participant_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                participant_kind: parse_participant_kind(
                    &row.try_get::<String, _>("participant_kind")
                        .map_err(persistence_error)?,
                )?,
                handle_snapshot: row.try_get("handle_snapshot").map_err(persistence_error)?,
                label_snapshot: row.try_get("label_snapshot").map_err(persistence_error)?,
                start_offset: u64_value(row.try_get("start_offset").map_err(persistence_error)?)?,
                end_offset: u64_value(row.try_get("end_offset").map_err(persistence_error)?)?,
            })
        })
        .collect()
}

async fn read_resource_references(
    pool: &SqlitePool,
    revision_id: &ChatMessageRevisionId,
) -> ChatResult<Vec<ChatResourceReferenceRead>> {
    let rows = sqlx::query(
        "SELECT working_folder_id, reference_kind, relative_path, display_label
         FROM chat_communication_resource_references
         WHERE message_revision_id = ? ORDER BY ordinal",
    )
    .bind(revision_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(ChatResourceReferenceRead {
                working_folder_id: ProjectWorkingFolderId::new(
                    row.try_get::<String, _>("working_folder_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                kind: row.try_get("reference_kind").map_err(persistence_error)?,
                relative_path: row.try_get("relative_path").map_err(persistence_error)?,
                display_label: row.try_get("display_label").map_err(persistence_error)?,
            })
        })
        .collect()
}

async fn read_post_receipt(
    pool: &SqlitePool,
    client_command_id: &ChatCommandId,
) -> ChatResult<Option<PostChatMessageResult>> {
    let row = sqlx::query(
        "SELECT state, result_data FROM chat_organizational_command_receipts
         WHERE client_command_id = ?",
    )
    .bind(client_command_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let state: String = row.try_get("state").map_err(persistence_error)?;
    if state != "completed" {
        return Err(ChatError::new(
            ChatErrorCode::Busy,
            "This message command is still being processed",
            true,
        ));
    }
    let data: String = row
        .try_get::<Option<String>, _>("result_data")
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::Persistence,
                "Message receipt is incomplete",
                true,
            )
        })?;
    serde_json::from_str(&data)
        .map(Some)
        .map_err(serialization_error)
}

async fn set_assignment_state(
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
    expected_revision: u64,
    state: ChatWorkAssignmentState,
    reason: Option<&str>,
) -> ChatResult<ChatWorkAssignmentRead> {
    let now = now_timestamp()?;
    let updated = sqlx::query(
        "UPDATE chat_work_assignments
         SET state = ?, state_reason = ?, settled_at = ?,
             revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(wire_work_state(state))
    .bind(reason)
    .bind((!state.is_active()).then(|| now.as_str()))
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .bind(i64_value(expected_revision)?)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "The assignment changed before the update",
            true,
        ));
    }
    if !state.is_active() {
        sqlx::query(
            "UPDATE chat_assignment_dispatch_jobs SET state = ?, updated_at = ?
             WHERE assignment_id = ? AND state IN ('queued', 'claimed')",
        )
        .bind(if state == ChatWorkAssignmentState::Cancelled {
            "cancelled"
        } else {
            "failed"
        })
        .bind(now.as_str())
        .bind(assignment_id.as_str())
        .execute(pool)
        .await
        .map_err(persistence_error)?;
    }
    read_assignment(pool, assignment_id).await
}

async fn dispatch_assignment_job(
    app: tauri::AppHandle,
    db_url: String,
    assignment_id: ChatWorkAssignmentId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let now = now_timestamp()?;
    let claim_token = new_id("dispatch-claim");
    let claimed = sqlx::query(
        "UPDATE chat_assignment_dispatch_jobs
         SET state = 'claimed', claimed_at = ?, claim_token = ?,
             attempt_count = attempt_count + 1, updated_at = ?
         WHERE assignment_id = ? AND state = 'queued' AND available_at <= ?",
    )
    .bind(now.as_str())
    .bind(&claim_token)
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if claimed.rows_affected() != 1 {
        return Ok(());
    }
    let result =
        dispatch_claimed_assignment(app, db_url, &pool, &assignment_id, &claim_token).await;
    if let Err(error) = &result {
        fail_assignment_dispatch(&pool, &assignment_id, &claim_token, error).await?;
    }
    result.map(|_| ())
}

async fn dispatch_claimed_assignment(
    app: tauri::AppHandle,
    db_url: String,
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
    claim_token: &str,
) -> ChatResult<()> {
    let row = sqlx::query(
        "SELECT
            assignment.reply_thread_id,
            assignment.teammate_id,
            assignment.triggering_message_item_id,
            assignment.state,
            context.serialized_text,
            authorization.id AS authorization_id,
            authorization.teammate_policy_revision_id,
            authorization.conversation_id,
            authorization.working_folder_id,
            authorization.approval_policy,
            authorization.decision_state,
            policy.revision AS policy_revision,
            teammate.instructions,
            channel.project_id,
            project.status AS project_status,
            membership.addressable,
            membership.removed_at,
            participant.archived_at,
            grant_row.revoked_at AS grant_revoked_at,
            job.claim_token
         FROM chat_work_assignments assignment
         JOIN chat_assignment_context_packages context
           ON context.assignment_id = assignment.id AND context.revision = 1
         JOIN chat_assignment_authorization_decisions authorization
           ON authorization.assignment_id = assignment.id
         JOIN chat_teammate_policy_revisions policy
           ON policy.id = authorization.teammate_policy_revision_id
         JOIN chat_ai_teammates teammate ON teammate.participant_id = assignment.teammate_id
         JOIN chat_participants participant ON participant.id = assignment.teammate_id
         JOIN chat_channels channel ON channel.conversation_id = authorization.conversation_id
         JOIN projects project ON project.id = channel.project_id
         LEFT JOIN chat_conversation_memberships membership
           ON membership.conversation_id = authorization.conversation_id
          AND membership.participant_id = assignment.teammate_id
         LEFT JOIN chat_teammate_working_folder_grants grant_row
           ON grant_row.conversation_id = authorization.conversation_id
          AND grant_row.teammate_id = assignment.teammate_id
          AND grant_row.working_folder_id = authorization.working_folder_id
         JOIN chat_assignment_dispatch_jobs job ON job.assignment_id = assignment.id
         WHERE assignment.id = ?",
    )
    .bind(assignment_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Work assignment was not found",
            true,
        )
    })?;
    if row
        .try_get::<String, _>("claim_token")
        .map_err(persistence_error)?
        != claim_token
    {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Assignment dispatch ownership changed",
            true,
        ));
    }
    if row
        .try_get::<String, _>("state")
        .map_err(persistence_error)?
        != "queued"
    {
        return Err(ChatError::new(
            ChatErrorCode::InvalidStateTransition,
            "Only queued assignments can start",
            true,
        ));
    }
    let membership_active = row
        .try_get::<Option<i64>, _>("addressable")
        .map_err(persistence_error)?
        == Some(1)
        && row
            .try_get::<Option<String>, _>("removed_at")
            .map_err(persistence_error)?
            .is_none()
        && row
            .try_get::<Option<String>, _>("archived_at")
            .map_err(persistence_error)?
            .is_none()
        && row
            .try_get::<Option<String>, _>("grant_revoked_at")
            .map_err(persistence_error)?
            .is_none();
    if !membership_active {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "The teammate no longer has access to this channel and working folder",
            true,
        ));
    }
    if row
        .try_get::<String, _>("project_status")
        .map_err(persistence_error)?
        == "archived"
    {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Restore the project before starting queued AI work",
            true,
        ));
    }
    if row
        .try_get::<String, _>("decision_state")
        .map_err(persistence_error)?
        != "allowed"
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "The assignment authorization decision blocks execution",
            true,
        ));
    }
    let teammate_id = ChatParticipantId::new(
        row.try_get::<String, _>("teammate_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let policy_revision = u64_value(row.try_get("policy_revision").map_err(persistence_error)?)?;
    let policy = read_policy(pool, &teammate_id, policy_revision).await?;
    let working_folder_id = ProjectWorkingFolderId::new(
        row.try_get::<String, _>("working_folder_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let reply_thread_id = ChatReplyThreadId::new(
        row.try_get::<String, _>("reply_thread_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let triggering_message_item_id = ChatConversationItemId::new(
        row.try_get::<String, _>("triggering_message_item_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let continuation = sqlx::query(
        "SELECT thread.id, thread.revision
         FROM chat_agent_runs run
         JOIN chat_work_assignments previous_assignment
           ON previous_assignment.id = run.assignment_id
         JOIN chat_threads thread ON thread.id = run.provider_thread_id
         WHERE previous_assignment.reply_thread_id = ?
           AND previous_assignment.id != ?
           AND run.working_folder_id = ?
           AND thread.provider_instance_id = ?
           AND thread.archived_at IS NULL
           AND thread.state NOT IN ('closed', 'error')
         ORDER BY previous_assignment.created_at DESC, run.run_ordinal DESC
         LIMIT 1",
    )
    .bind(reply_thread_id.as_str())
    .bind(assignment_id.as_str())
    .bind(working_folder_id.as_str())
    .bind(policy.provider_instance_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    let existing_thread_id = continuation
        .as_ref()
        .map(|thread| thread.try_get::<String, _>("id"))
        .transpose()
        .map_err(persistence_error)?
        .map(ChatThreadId::new)
        .transpose()
        .map_err(identifier_error)?;
    let expected_thread_revision = continuation
        .as_ref()
        .map(|thread| thread.try_get::<i64, _>("revision"))
        .transpose()
        .map_err(persistence_error)?
        .map(u64_value)
        .transpose()?;
    let provider_execution_thread_id = existing_thread_id
        .clone()
        .unwrap_or(ChatThreadId::new(new_id("provider-execution")).map_err(identifier_error)?);
    let attachment_ids = sqlx::query_scalar::<_, String>(
        "SELECT attachment.attachment_id
         FROM chat_work_assignments assignment
         JOIN chat_communication_messages message
           ON message.item_id = assignment.triggering_message_item_id
         JOIN chat_communication_attachment_references attachment
           ON attachment.message_revision_id = message.current_revision_id
         WHERE assignment.id = ? ORDER BY attachment.ordinal",
    )
    .bind(assignment_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?
    .into_iter()
    .map(ChatAttachmentId::new)
    .collect::<Result<Vec<_>, _>>()
    .map_err(identifier_error)?;
    let mentions = sqlx::query(
        "SELECT resource.relative_path, resource.reference_kind
         FROM chat_communication_messages message
         JOIN chat_communication_resource_references resource
           ON resource.message_revision_id = message.current_revision_id
         WHERE message.item_id = ? ORDER BY resource.ordinal",
    )
    .bind(triggering_message_item_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?
    .into_iter()
    .map(|resource| {
        Ok(WorkspaceMentionReference {
            relative_path: resource
                .try_get("relative_path")
                .map_err(persistence_error)?,
            kind: resource
                .try_get("reference_kind")
                .map_err(persistence_error)?,
        })
    })
    .collect::<ChatResult<Vec<_>>>()?;
    let run_ordinal: i64 = sqlx::query_scalar(
        "SELECT coalesce(max(run_ordinal), 0) + 1 FROM chat_agent_runs WHERE assignment_id = ?",
    )
    .bind(assignment_id.as_str())
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    let approval_policy = parse_approval_policy(
        &row.try_get::<String, _>("approval_policy")
            .map_err(persistence_error)?,
    )?;
    let request = super::send_commands::SendChatTurnCommand {
        command: ChatCommandContext {
            client_command_id: ChatCommandId::new(format!(
                "assignment-dispatch:{}:{}",
                assignment_id.as_str(),
                run_ordinal
            ))
            .map_err(identifier_error)?,
            expected_thread_revision,
        },
        working_folder_id,
        thread_id: existing_thread_id,
        new_thread_id: (continuation.is_none()).then_some(provider_execution_thread_id),
        execution_environment_id: None,
        turn_id: ChatTurnId::new(new_id("provider-turn")).map_err(identifier_error)?,
        message_id: ChatMessageId::new(new_id("provider-message")).map_err(identifier_error)?,
        provider_instance_id: policy.provider_instance_id,
        provider_managed_model: policy.provider_managed_model,
        model_id: policy.model_id,
        model_options: policy.model_options,
        modes: TurnModeSnapshot {
            safety_mode: approval_policy_to_safety(approval_policy),
            interaction_mode: InteractionMode::Build,
        },
        prompt: row.try_get("serialized_text").map_err(persistence_error)?,
        attachment_ids,
        mentions,
        developer_instructions: Some(
            row.try_get::<String, _>("instructions")
                .map_err(persistence_error)?,
        ),
        organizational_run: Some(super::send_commands::OrganizationalRunBinding {
            run_id: ChatAgentRunId::new(new_id("agent-run")).map_err(identifier_error)?,
            assignment_id: assignment_id.clone(),
            teammate_policy_revision_id: policy.id,
            authorization_decision_id: row
                .try_get("authorization_id")
                .map_err(persistence_error)?,
            run_ordinal: u64_value(run_ordinal)?,
        }),
    };
    super::send_commands::chat_send_turn(app, db_url, request).await?;
    Ok(())
}

pub(crate) async fn mark_agent_run_dispatched(
    pool: &SqlitePool,
    binding: &super::send_commands::OrganizationalRunBinding,
    launch_error: Option<&ChatError>,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    if let Some(error) = launch_error {
        sqlx::query(
            "UPDATE chat_agent_runs
             SET state = 'failed', settled_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(binding.run_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_work_assignments
             SET state = 'failed', state_reason = ?, settled_at = ?,
                 revision = revision + 1, updated_at = ? WHERE id = ?",
        )
        .bind(&error.message)
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(binding.assignment_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_assignment_dispatch_jobs
             SET state = 'failed', last_error = ?, updated_at = ? WHERE assignment_id = ?",
        )
        .bind(&error.message)
        .bind(now.as_str())
        .bind(binding.assignment_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        insert_dispatch_failure_message(
            &mut transaction,
            &binding.assignment_id,
            binding.run_id.as_str(),
            &error.message,
            now,
        )
        .await?;
    } else {
        sqlx::query(
            "UPDATE chat_agent_runs SET state = 'working', started_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(binding.run_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_work_assignments
             SET state = 'working', state_reason = NULL,
                 revision = revision + 1, updated_at = ? WHERE id = ?",
        )
        .bind(now.as_str())
        .bind(binding.assignment_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_assignment_dispatch_jobs
             SET state = 'completed', last_error = NULL, updated_at = ?
             WHERE assignment_id = ?",
        )
        .bind(now.as_str())
        .bind(binding.assignment_id.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    transaction.commit().await.map_err(persistence_error)
}

async fn fail_assignment_dispatch(
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
    claim_token: &str,
    error: &ChatError,
) -> ChatResult<()> {
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_assignment_dispatch_jobs
         SET state = 'failed', last_error = ?, updated_at = ?
         WHERE assignment_id = ? AND claim_token = ?",
    )
    .bind(&error.message)
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .bind(claim_token)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_work_assignments
         SET state = 'failed', state_reason = ?, settled_at = ?,
             revision = revision + 1, updated_at = ?
         WHERE id = ? AND state = 'queued'",
    )
    .bind(&error.message)
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    insert_dispatch_failure_message(
        &mut transaction,
        assignment_id,
        claim_token,
        &error.message,
        &now,
    )
    .await?;
    transaction.commit().await.map_err(persistence_error)
}

async fn insert_dispatch_failure_message(
    transaction: &mut Transaction<'_, Sqlite>,
    assignment_id: &ChatWorkAssignmentId,
    claim_token: &str,
    message: &str,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let row = sqlx::query(
        "SELECT assignment.reply_thread_id, assignment.teammate_id, thread.conversation_id
         FROM chat_work_assignments assignment
         JOIN chat_reply_threads thread ON thread.id = assignment.reply_thread_id
         WHERE assignment.id = ?",
    )
    .bind(assignment_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let reply_thread_id: String = row.try_get("reply_thread_id").map_err(persistence_error)?;
    let teammate_id: String = row.try_get("teammate_id").map_err(persistence_error)?;
    let conversation_id: String = row.try_get("conversation_id").map_err(persistence_error)?;
    let hash = sha256_hex(format!("{}:{claim_token}", assignment_id.as_str()).as_bytes());
    let item_id = format!("organizational-item:{hash}");
    let revision_id = format!("organizational-revision:{hash}");
    let ordinal: i64 = sqlx::query_scalar(
        "SELECT coalesce(max(ordinal), 0) + 1 FROM chat_conversation_items
         WHERE reply_thread_id = ?",
    )
    .bind(&reply_thread_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_conversation_items
            (id, conversation_id, reply_thread_id, item_kind, ordinal, created_at)
         VALUES (?, ?, ?, 'message', ?, ?)",
    )
    .bind(&item_id)
    .bind(&conversation_id)
    .bind(&reply_thread_id)
    .bind(ordinal)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_communication_messages
            (item_id, author_participant_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&item_id)
    .bind(&teammate_id)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let rich_content = json!({
        "type": "agent_update",
        "updateKind": "failure",
        "assignmentId": assignment_id,
        "agentRunId": null,
        "payload": { "launchFailure": true },
    });
    let rich_content_data = serde_json::to_string(&rich_content).map_err(serialization_error)?;
    sqlx::query(
        "INSERT INTO chat_communication_message_revisions
            (id, message_item_id, revision, normalized_markdown,
             rich_content_schema_version, rich_content_data, created_at)
         VALUES (?, ?, 1, ?, 1, ?, ?)",
    )
    .bind(&revision_id)
    .bind(&item_id)
    .bind(message)
    .bind(&rich_content_data)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query("UPDATE chat_communication_messages SET current_revision_id = ? WHERE item_id = ?")
        .bind(&revision_id)
        .bind(&item_id)
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_work_semantic_updates
            (item_id, assignment_id, update_kind, payload_data, created_at)
         VALUES (?, ?, 'failure', ?, ?)",
    )
    .bind(&item_id)
    .bind(assignment_id.as_str())
    .bind(&rich_content_data)
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_reply_threads SET reply_count = reply_count + 1,
             last_activity_at = ?, revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(&reply_thread_id)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_conversations SET last_activity_at = ?, revision = revision + 1,
             updated_at = ? WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(&conversation_id)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn deliver_assignment_input(
    app: tauri::AppHandle,
    db_url: String,
    assignment_id: ChatWorkAssignmentId,
    message_item_id: ChatConversationItemId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url.clone()).await?;
    let row = sqlx::query(
        "SELECT run.provider_thread_id, revision.normalized_markdown
         FROM chat_agent_runs run
         JOIN chat_work_assignment_inputs input ON input.assignment_id = run.assignment_id
         JOIN chat_communication_messages message ON message.item_id = input.message_item_id
         JOIN chat_communication_message_revisions revision
           ON revision.id = message.current_revision_id
         WHERE run.assignment_id = ? AND input.message_item_id = ?
           AND run.state = 'working' AND input.delivery_state = 'pending'
         ORDER BY run.run_ordinal DESC LIMIT 1",
    )
    .bind(assignment_id.as_str())
    .bind(message_item_id.as_str())
    .fetch_optional(&pool)
    .await
    .map_err(persistence_error)?;
    let Some(row) = row else {
        return Ok(());
    };
    let thread_id = ChatThreadId::new(
        row.try_get::<String, _>("provider_thread_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    super::send_commands::chat_steer_turn(
        app,
        db_url,
        super::send_commands::SteerChatTurnCommand {
            command: ChatCommandContext {
                client_command_id: ChatCommandId::new(format!(
                    "assignment-steer:{}:{}",
                    assignment_id.as_str(),
                    message_item_id.as_str()
                ))
                .map_err(identifier_error)?,
                expected_thread_revision: None,
            },
            thread_id,
            message_id: ChatMessageId::new(new_id("provider-steer-message"))
                .map_err(identifier_error)?,
            prompt: row
                .try_get("normalized_markdown")
                .map_err(persistence_error)?,
        },
    )
    .await?;
    sqlx::query(
        "UPDATE chat_work_assignment_inputs SET delivery_state = 'delivered', delivered_at = ?
         WHERE assignment_id = ? AND message_item_id = ? AND delivery_state = 'pending'",
    )
    .bind(now_timestamp()?.as_str())
    .bind(assignment_id.as_str())
    .bind(message_item_id.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

fn approval_policy_to_safety(value: ChatApprovalPolicy) -> SafetyMode {
    match value {
        ChatApprovalPolicy::AskForApproval => SafetyMode::AskForApproval,
        ChatApprovalPolicy::ApproveForMe => SafetyMode::ApproveForMe,
        ChatApprovalPolicy::FullAccess => SafetyMode::FullAccess,
        ChatApprovalPolicy::Custom => SafetyMode::Custom,
    }
}

pub(crate) async fn project_provider_event_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    runtime: &super::events::CanonicalRuntimeEvent,
) -> ChatResult<bool> {
    let run = sqlx::query(
        "SELECT run.id AS run_id, run.assignment_id, assignment.reply_thread_id,
                assignment.teammate_id
         FROM chat_agent_runs run
         JOIN chat_work_assignments assignment ON assignment.id = run.assignment_id
         WHERE (
             (? IS NOT NULL AND run.provider_turn_id = ?)
             OR (? IS NULL AND run.provider_thread_id = ? AND run.state IN ('starting', 'working', 'waiting'))
         )
         ORDER BY run.run_ordinal DESC LIMIT 1",
    )
    .bind(runtime.turn_id.as_ref().map(ChatTurnId::as_str))
    .bind(runtime.turn_id.as_ref().map(ChatTurnId::as_str))
    .bind(runtime.turn_id.as_ref().map(ChatTurnId::as_str))
    .bind(runtime.thread_id.as_str())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let Some(run) = run else {
        return Ok(false);
    };
    let run_id: String = run.try_get("run_id").map_err(persistence_error)?;
    let assignment_id = ChatWorkAssignmentId::new(
        run.try_get::<String, _>("assignment_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let reply_thread_id = ChatReplyThreadId::new(
        run.try_get::<String, _>("reply_thread_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let teammate_id = ChatParticipantId::new(
        run.try_get::<String, _>("teammate_id")
            .map_err(persistence_error)?,
    )
    .map_err(identifier_error)?;
    let target = ProjectionTarget {
        run_id: &run_id,
        assignment_id: &assignment_id,
        reply_thread_id: &reply_thread_id,
        teammate_id: &teammate_id,
    };
    let projected = match &runtime.event {
        super::events::CanonicalEvent::PlanUpdated(event) => {
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "plan",
                &event.markdown,
                json!({ "steps": event.steps }),
            )
            .await?
        }
        super::events::CanonicalEvent::ProposedPlanCompleted(event) => {
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "plan",
                &event.markdown,
                json!({ "planId": event.plan_id }),
            )
            .await?
        }
        super::events::CanonicalEvent::RequestOpened(event) => {
            let markdown = match event.detail.as_deref() {
                Some(detail) if !detail.trim().is_empty() => {
                    format!("{}\n\n{}", event.title, detail)
                }
                _ => event.title.clone(),
            };
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                "waiting_for_approval",
                "waiting",
                None,
                runtime.created_at.as_str(),
            )
            .await?;
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "approval",
                &markdown,
                json!({
                    "requestId": event.request_id,
                    "kind": event.kind,
                    "allowedDecisions": event.allowed_decisions,
                    "safePayload": event.safe_payload,
                }),
            )
            .await?
        }
        super::events::CanonicalEvent::UserInputRequested(event) => {
            let markdown = event
                .questions
                .iter()
                .map(|question| question.question.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                "waiting_for_answer",
                "waiting",
                None,
                runtime.created_at.as_str(),
            )
            .await?;
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "question",
                &markdown,
                json!({
                    "requestId": event.request_id,
                    "questions": event.questions,
                }),
            )
            .await?
        }
        super::events::CanonicalEvent::RequestResolved(_)
        | super::events::CanonicalEvent::UserInputResolved(_) => {
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                "working",
                "working",
                None,
                runtime.created_at.as_str(),
            )
            .await?;
            true
        }
        super::events::CanonicalEvent::TurnCompleted(event) => {
            let markdown: Option<String> = sqlx::query_scalar(
                "SELECT normalized_markdown
                 FROM chat_messages
                 WHERE thread_id = ? AND turn_id = ? AND role = 'assistant'
                 ORDER BY sequence_anchor DESC, created_at DESC LIMIT 1",
            )
            .bind(runtime.thread_id.as_str())
            .bind(runtime.turn_id.as_ref().map(ChatTurnId::as_str))
            .fetch_optional(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            let state = if event.changed_files.is_empty() {
                "completed"
            } else {
                "ready_for_review"
            };
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                state,
                "completed",
                None,
                runtime.created_at.as_str(),
            )
            .await?;
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                if event.changed_files.is_empty() {
                    "result"
                } else {
                    "review"
                },
                markdown
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("Work completed."),
                json!({
                    "stopReason": event.stop_reason,
                    "usage": event.usage,
                    "changedFiles": event.changed_files,
                    "providerExecutionThreadId": runtime.thread_id,
                    "providerExecutionTurnId": runtime.turn_id,
                }),
            )
            .await?
        }
        super::events::CanonicalEvent::TurnAborted(event) => {
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                "failed",
                "failed",
                Some(&event.reason),
                runtime.created_at.as_str(),
            )
            .await?;
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "failure",
                &event.reason,
                json!({ "recoverable": event.recoverable }),
            )
            .await?
        }
        super::events::CanonicalEvent::RuntimeError(event) if !event.recoverable => {
            update_assignment_and_run_state(
                transaction,
                &assignment_id,
                &run_id,
                "failed",
                "failed",
                Some(&event.message),
                runtime.created_at.as_str(),
            )
            .await?;
            insert_projected_teammate_message(
                transaction,
                runtime,
                &target,
                "failure",
                &event.message,
                json!({ "code": event.code, "safeDetails": event.safe_details }),
            )
            .await?
        }
        _ => false,
    };
    Ok(projected)
}

struct ProjectionTarget<'a> {
    run_id: &'a str,
    assignment_id: &'a ChatWorkAssignmentId,
    reply_thread_id: &'a ChatReplyThreadId,
    teammate_id: &'a ChatParticipantId,
}

async fn insert_projected_teammate_message(
    transaction: &mut Transaction<'_, Sqlite>,
    runtime: &super::events::CanonicalRuntimeEvent,
    target: &ProjectionTarget<'_>,
    update_kind: &str,
    markdown: &str,
    payload: Value,
) -> ChatResult<bool> {
    let event_hash = sha256_hex(runtime.event_id.as_str().as_bytes());
    let item_id = format!("organizational-item:{event_hash}");
    let exists: i64 =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chat_conversation_items WHERE id = ?)")
            .bind(&item_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(persistence_error)?;
    if exists != 0 {
        return Ok(false);
    }
    let conversation_id: String =
        sqlx::query_scalar("SELECT conversation_id FROM chat_reply_threads WHERE id = ?")
            .bind(target.reply_thread_id.as_str())
            .fetch_one(&mut **transaction)
            .await
            .map_err(persistence_error)?;
    let ordinal: i64 = sqlx::query_scalar(
        "SELECT coalesce(max(ordinal), 0) + 1
         FROM chat_conversation_items WHERE reply_thread_id = ?",
    )
    .bind(target.reply_thread_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let revision_id = format!("organizational-revision:{event_hash}");
    sqlx::query(
        "INSERT INTO chat_conversation_items
            (id, conversation_id, reply_thread_id, item_kind, ordinal, created_at)
         VALUES (?, ?, ?, 'message', ?, ?)",
    )
    .bind(&item_id)
    .bind(&conversation_id)
    .bind(target.reply_thread_id.as_str())
    .bind(ordinal)
    .bind(runtime.created_at.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_communication_messages
            (item_id, author_participant_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&item_id)
    .bind(target.teammate_id.as_str())
    .bind(runtime.created_at.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let rich_content = json!({
        "type": "agent_update",
        "updateKind": update_kind,
        "assignmentId": target.assignment_id,
        "agentRunId": target.run_id,
        "payload": payload,
    });
    sqlx::query(
        "INSERT INTO chat_communication_message_revisions
            (id, message_item_id, revision, normalized_markdown,
             rich_content_schema_version, rich_content_data, created_at)
         VALUES (?, ?, 1, ?, 1, ?, ?)",
    )
    .bind(&revision_id)
    .bind(&item_id)
    .bind(markdown)
    .bind(serde_json::to_string(&rich_content).map_err(serialization_error)?)
    .bind(runtime.created_at.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query("UPDATE chat_communication_messages SET current_revision_id = ? WHERE item_id = ?")
        .bind(&revision_id)
        .bind(&item_id)
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_work_semantic_updates
            (item_id, assignment_id, update_kind, payload_data, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&item_id)
    .bind(target.assignment_id.as_str())
    .bind(update_kind)
    .bind(serde_json::to_string(&rich_content).map_err(serialization_error)?)
    .bind(runtime.created_at.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_reply_threads
         SET reply_count = reply_count + 1, last_activity_at = ?,
             revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(runtime.created_at.as_str())
    .bind(runtime.created_at.as_str())
    .bind(target.reply_thread_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_conversations
         SET last_activity_at = ?, revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(runtime.created_at.as_str())
    .bind(runtime.created_at.as_str())
    .bind(&conversation_id)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(true)
}

async fn update_assignment_and_run_state(
    transaction: &mut Transaction<'_, Sqlite>,
    assignment_id: &ChatWorkAssignmentId,
    run_id: &str,
    assignment_state: &str,
    run_state: &str,
    reason: Option<&str>,
    occurred_at: &str,
) -> ChatResult<()> {
    let terminal = matches!(assignment_state, "completed" | "failed" | "cancelled");
    sqlx::query(
        "UPDATE chat_work_assignments
         SET state = ?, state_reason = ?, settled_at = CASE WHEN ? THEN ? ELSE NULL END,
             revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(assignment_state)
    .bind(reason)
    .bind(terminal)
    .bind(occurred_at)
    .bind(occurred_at)
    .bind(assignment_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let run_terminal = matches!(run_state, "completed" | "failed" | "cancelled");
    sqlx::query(
        "UPDATE chat_agent_runs
         SET state = ?, settled_at = CASE WHEN ? THEN ? ELSE NULL END, updated_at = ?
         WHERE id = ?",
    )
    .bind(run_state)
    .bind(run_terminal)
    .bind(occurred_at)
    .bind(occurred_at)
    .bind(run_id)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredModelSelection {
    provider_managed_model: bool,
    model_id: Option<ModelId>,
    model_options: Vec<ModelOptionSelection>,
}

fn parse_model_selection(data: String) -> ChatResult<StoredModelSelection> {
    serde_json::from_str(&data).map_err(serialization_error)
}

fn validate_policy(app: &tauri::AppHandle, policy: &ChatTeammatePolicyInput) -> ChatResult<()> {
    if policy.provider_managed_model == policy.model_id.is_some() {
        return Err(ChatError::validation(
            "modelId",
            "Choose either provider-managed model selection or one explicit model",
        ));
    }
    if let Some(effort) = policy.effort.as_deref() {
        if !matches!(
            effort,
            "none" | "minimal" | "low" | "medium" | "high" | "xhigh"
        ) {
            return Err(ChatError::validation(
                "effort",
                "Teammate effort is invalid",
            ));
        }
    }
    if let Some(speed) = policy.speed.as_deref() {
        if !matches!(speed, "standard" | "fast") {
            return Err(ChatError::validation("speed", "Teammate speed is invalid"));
        }
    }
    super::settings_commands::read_provider(app, &policy.provider_instance_id)?;
    Ok(())
}

fn validate_message_request(request: &PostChatMessageCommand) -> ChatResult<()> {
    let markdown = request.normalized_markdown.trim();
    if markdown.is_empty()
        && request.attachment_ids.is_empty()
        && request.resource_references.is_empty()
    {
        return Err(ChatError::validation(
            "normalizedMarkdown",
            "Message content is required",
        ));
    }
    if request.normalized_markdown.len() > MAX_MESSAGE_BYTES {
        return Err(ChatError::validation(
            "normalizedMarkdown",
            "Message content exceeds the 128 KiB limit",
        ));
    }
    json_object(&request.rich_content, "richContent")?;
    let mut ranges = BTreeSet::new();
    for mention in &request.participant_mentions {
        if mention.start_offset >= mention.end_offset
            || mention.end_offset > request.normalized_markdown.len() as u64
            || mention.label_snapshot.trim().is_empty()
            || mention.label_snapshot.chars().count() > 160
            || !ranges.insert((mention.start_offset, mention.end_offset))
        {
            return Err(ChatError::validation(
                "participantMentions",
                "Participant mention ranges are invalid",
            ));
        }
    }
    let mut attachments = BTreeSet::new();
    if request
        .attachment_ids
        .iter()
        .any(|id| !attachments.insert(id.as_str()))
    {
        return Err(ChatError::validation(
            "attachmentIds",
            "Attachment IDs must be distinct",
        ));
    }
    for resource in &request.resource_references {
        if !matches!(resource.kind.as_str(), "file" | "folder")
            || !safe_relative_path(&resource.relative_path)
            || resource.display_label.trim().is_empty()
        {
            return Err(ChatError::validation(
                "resourceReferences",
                "A file or folder reference is invalid",
            ));
        }
    }
    Ok(())
}

fn validate_display_name(value: &str) -> ChatResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 160 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "displayName",
            "Teammate name must contain 1 to 160 characters",
        ));
    }
    Ok(value.to_string())
}

fn validate_handle(value: &str) -> ChatResult<String> {
    let value = value.trim().trim_start_matches('@').to_lowercase();
    if value.is_empty()
        || value.chars().count() > 80
        || value.chars().any(|character| {
            !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '-')
        })
    {
        return Err(ChatError::validation(
            "handle",
            "Teammate handle can use lowercase letters, numbers, periods, underscores, and hyphens",
        ));
    }
    Ok(value)
}

fn validate_profile_text(value: &str, maximum: usize, field: &str) -> ChatResult<()> {
    if value.len() > maximum {
        return Err(ChatError::validation(
            field,
            "Teammate profile text is too long",
        ));
    }
    Ok(())
}

fn json_object(value: &VersionedJson, field: &str) -> ChatResult<String> {
    if !value.value.is_object() {
        return Err(ChatError::validation(field, "Value must be a JSON object"));
    }
    serde_json::to_string(&value.value).map_err(serialization_error)
}

fn parse_json(data: String) -> ChatResult<Value> {
    serde_json::from_str(&data).map_err(serialization_error)
}

fn parse_cursor(cursor: Option<&str>) -> ChatResult<Option<i64>> {
    cursor
        .map(|value| {
            value
                .parse::<i64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| ChatError::validation("cursor", "Chat page cursor is invalid"))
        })
        .transpose()
}

fn normalized_fts_query(query: &str) -> ChatResult<String> {
    let terms = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .take(20)
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>();
    if terms.is_empty() || query.len() > 500 {
        return Err(ChatError::validation(
            "query",
            "Message search query is invalid",
        ));
    }
    Ok(terms.join(" AND "))
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4_096
        && !path.starts_with('/')
        && !path.starts_with("../")
        && !path.contains("/../")
        && !path.ends_with("/..")
        && !path.contains('\\')
}

fn conversation_item_id() -> ChatResult<ChatConversationItemId> {
    ChatConversationItemId::new(new_id("conversation-item")).map_err(identifier_error)
}

fn message_revision_id() -> ChatResult<ChatMessageRevisionId> {
    ChatMessageRevisionId::new(new_id("message-revision")).map_err(identifier_error)
}

fn reply_thread_id() -> ChatResult<ChatReplyThreadId> {
    ChatReplyThreadId::new(new_id("reply-thread")).map_err(identifier_error)
}

fn work_assignment_id() -> ChatResult<ChatWorkAssignmentId> {
    ChatWorkAssignmentId::new(new_id("assignment")).map_err(identifier_error)
}

fn default_addressable() -> bool {
    true
}

fn parse_participant_kind(value: &str) -> ChatResult<ChatParticipantKind> {
    match value {
        "local_user" => Ok(ChatParticipantKind::LocalUser),
        "ai_teammate" => Ok(ChatParticipantKind::AiTeammate),
        "human" => Ok(ChatParticipantKind::Human),
        _ => Err(stored_value_error("participant kind")),
    }
}

fn parse_configuration_state(value: &str) -> ChatResult<ChatTeammateConfigurationState> {
    match value {
        "healthy" => Ok(ChatTeammateConfigurationState::Healthy),
        "needs_setup" => Ok(ChatTeammateConfigurationState::NeedsSetup),
        "provider_unavailable" => Ok(ChatTeammateConfigurationState::ProviderUnavailable),
        "folder_access_missing" => Ok(ChatTeammateConfigurationState::FolderAccessMissing),
        _ => Err(stored_value_error("teammate configuration state")),
    }
}

fn parse_approval_policy(value: &str) -> ChatResult<ChatApprovalPolicy> {
    match value {
        "ask_for_approval" => Ok(ChatApprovalPolicy::AskForApproval),
        "approve_for_me" => Ok(ChatApprovalPolicy::ApproveForMe),
        "full_access" => Ok(ChatApprovalPolicy::FullAccess),
        "custom" => Ok(ChatApprovalPolicy::Custom),
        _ => Err(stored_value_error("approval policy")),
    }
}

fn parse_work_state(value: &str) -> ChatResult<ChatWorkAssignmentState> {
    match value {
        "queued" => Ok(ChatWorkAssignmentState::Queued),
        "working" => Ok(ChatWorkAssignmentState::Working),
        "waiting_for_answer" => Ok(ChatWorkAssignmentState::WaitingForAnswer),
        "waiting_for_approval" => Ok(ChatWorkAssignmentState::WaitingForApproval),
        "ready_for_review" => Ok(ChatWorkAssignmentState::ReadyForReview),
        "completed" => Ok(ChatWorkAssignmentState::Completed),
        "failed" => Ok(ChatWorkAssignmentState::Failed),
        "cancelled" => Ok(ChatWorkAssignmentState::Cancelled),
        _ => Err(stored_value_error("assignment state")),
    }
}

fn wire_participant_kind(value: ChatParticipantKind) -> &'static str {
    match value {
        ChatParticipantKind::LocalUser => "local_user",
        ChatParticipantKind::AiTeammate => "ai_teammate",
        ChatParticipantKind::Human => "human",
    }
}

fn wire_approval_policy(value: ChatApprovalPolicy) -> &'static str {
    match value {
        ChatApprovalPolicy::AskForApproval => "ask_for_approval",
        ChatApprovalPolicy::ApproveForMe => "approve_for_me",
        ChatApprovalPolicy::FullAccess => "full_access",
        ChatApprovalPolicy::Custom => "custom",
    }
}

fn wire_work_state(value: ChatWorkAssignmentState) -> &'static str {
    match value {
        ChatWorkAssignmentState::Queued => "queued",
        ChatWorkAssignmentState::Working => "working",
        ChatWorkAssignmentState::WaitingForAnswer => "waiting_for_answer",
        ChatWorkAssignmentState::WaitingForApproval => "waiting_for_approval",
        ChatWorkAssignmentState::ReadyForReview => "ready_for_review",
        ChatWorkAssignmentState::Completed => "completed",
        ChatWorkAssignmentState::Failed => "failed",
        ChatWorkAssignmentState::Cancelled => "cancelled",
    }
}

fn u32_value(value: i64) -> ChatResult<u32> {
    u32::try_from(value).map_err(|_| stored_value_error("schema version"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn truncate_utf8(value: &str, maximum: usize) -> &str {
    if value.len() <= maximum {
        return value;
    }
    let mut boundary = maximum;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}

fn stored_value_error(label: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        format!("Stored Chat {label} is invalid"),
        false,
    )
}

fn new_id(prefix: &str) -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}:{nanos:032x}{sequence:016x}")
}

fn serialization_error<E>(_error: E) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat JSON data is invalid",
        false,
    )
}

fn map_teammate_write_error(error: sqlx::Error) -> ChatError {
    let detail = error.to_string();
    if detail.contains("chat_participants.normalized_handle") {
        ChatError::validation("handle", "This teammate handle is already in use")
    } else {
        persistence_error(error)
    }
}

fn map_command_receipt_error(error: sqlx::Error) -> ChatError {
    if error
        .to_string()
        .contains("chat_organizational_command_receipts.client_command_id")
    {
        ChatError::new(
            ChatErrorCode::Conflict,
            "This message command was already accepted",
            true,
        )
    } else {
        persistence_error(error)
    }
}

#[allow(dead_code)]
fn current_timestamp_string() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn migrated_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();
        crate::db::run_migrations(&pool).await.unwrap();
        pool
    }

    fn message(markdown: &str) -> PostChatMessageCommand {
        PostChatMessageCommand {
            client_command_id: ChatCommandId::new("command:test").unwrap(),
            channel_id: ChatChannelId::new("channel:test").unwrap(),
            reply_thread_id: None,
            normalized_markdown: markdown.to_string(),
            rich_content: VersionedJson {
                schema_version: 1,
                value: json!({ "type": "doc", "content": [] }),
            },
            attachment_ids: Vec::new(),
            participant_mentions: Vec::new(),
            resource_references: Vec::new(),
            post_without_invoking: false,
            also_send_to_channel: false,
        }
    }

    #[test]
    fn plain_at_text_is_valid_but_does_not_invoke_a_teammate() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_pool().await;
            let request = message("Please ask @ganbaru to review this.");
            validate_message_request(&request).unwrap();
            let invocation = resolve_invoked_teammate(
                &pool,
                &ChatConversationId::new("conversation:test").unwrap(),
                None,
                &request.participant_mentions,
                false,
            )
            .await
            .unwrap();
            assert!(invocation.is_none());
        });
    }

    #[test]
    fn two_structured_ai_mentions_are_rejected_before_dispatch() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_pool().await;
            for index in 1..=2 {
                sqlx::query(
                    "INSERT INTO chat_participants
                        (id, participant_kind, display_name, normalized_handle,
                         created_at, updated_at)
                     VALUES (?, 'ai_teammate', ?, ?, ?, ?)",
                )
                .bind(format!("participant:agent-{index}"))
                .bind(format!("Agent {index}"))
                .bind(format!("agent-{index}"))
                .bind("2026-08-01T00:00:00.000Z")
                .bind("2026-08-01T00:00:00.000Z")
                .execute(&pool)
                .await
                .unwrap();
            }
            let mentions = (1..=2)
                .map(|index| ChatParticipantMentionInput {
                    participant_id: ChatParticipantId::new(format!("participant:agent-{index}"))
                        .unwrap(),
                    participant_kind: ChatParticipantKind::AiTeammate,
                    handle_snapshot: Some(format!("agent-{index}")),
                    label_snapshot: format!("Agent {index}"),
                    start_offset: u64::try_from((index - 1) * 9).unwrap(),
                    end_offset: u64::try_from(index * 9 - 1).unwrap(),
                })
                .collect::<Vec<_>>();
            let error = resolve_invoked_teammate(
                &pool,
                &ChatConversationId::new("conversation:test").unwrap(),
                None,
                &mentions,
                false,
            )
            .await
            .unwrap_err();
            assert_eq!(error.code, ChatErrorCode::Validation);
            assert_eq!(error.field.as_deref(), Some("participantMentions"));
        });
    }

    #[test]
    fn message_validation_rejects_duplicate_atomic_mention_ranges() {
        let mut request = message("@ganbaru");
        let mention = ChatParticipantMentionInput {
            participant_id: ChatParticipantId::new("participant:ganbaru").unwrap(),
            participant_kind: ChatParticipantKind::AiTeammate,
            handle_snapshot: Some("ganbaru".to_string()),
            label_snapshot: "Ganbaru".to_string(),
            start_offset: 0,
            end_offset: 8,
        };
        request.participant_mentions = vec![mention.clone(), mention];
        let error = validate_message_request(&request).unwrap_err();
        assert_eq!(error.field.as_deref(), Some("participantMentions"));
    }

    #[test]
    fn fts_queries_are_bounded_and_quote_each_term() {
        assert_eq!(
            normalized_fts_query("review exact-folder").unwrap(),
            "\"review\" AND \"exact-folder\""
        );
        assert!(normalized_fts_query("   ").is_err());
        assert!(normalized_fts_query(&"x".repeat(501)).is_err());
    }
}
