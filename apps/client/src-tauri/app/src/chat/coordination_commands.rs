//! Durable organizational messages, reply threads, teammates, and work assignments.

use super::channel_commands::{
    chat_pool, i64_value, identifier_error, now_timestamp, optional_timestamp, persistence_error,
    timestamp, u64_value,
};
pub use super::coordination::contracts::*;
use super::models::*;
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, Transaction};

const LOCAL_PARTICIPANT_ID: &str = "participant:local-owner";
const MAX_CHANNEL_CONTEXT_MESSAGES: usize = 20;
const MAX_THREAD_CONTEXT_REPLIES: usize = 50;
const MAX_PAGE_SIZE: u32 = 100;
const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_SEARCH_RESULTS: u32 = 100;

mod assignments;
mod common;
mod context;
mod dispatch;
mod reads;
mod scheduling;
mod workflow;

use common::{
    conversation_item_id, json_object, map_command_receipt_error, map_teammate_write_error,
    message_revision_id, normalized_fts_query, parse_cursor, parse_participant_kind,
    reply_thread_id, serialization_error, validate_display_name, validate_handle,
    validate_message_request, validate_policy, validate_profile_text,
};
use dispatch::{deliver_assignment_input, dispatch_assignment_job};
pub(crate) use reads::read_memberships_for_conversation;
use reads::{
    read_active_or_latest_assignment, read_assignment, read_membership, read_message, read_policy,
    read_reply_thread_page, read_teammate,
};
use workflow::{
    insert_channel_copy, insert_communication_message, insert_policy_revision, next_item_ordinal,
    persist_assignment_routing, read_post_receipt, require_reply_thread, resolve_invoked_teammate,
    upsert_membership_in_transaction, CommunicationMessageWrite,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "receiptKind", rename_all = "snake_case")]
enum StoredPostMessageReceipt {
    Locator {
        message_item_id: ChatConversationItemId,
        reply_thread_id: ChatReplyThreadId,
        assignment_id: Option<ChatWorkAssignmentId>,
        assignment_input_queued: bool,
    },
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
    let receipt_locator = StoredPostMessageReceipt::Locator {
        message_item_id: item_id.clone(),
        reply_thread_id: new_reply_thread_id.clone(),
        assignment_id: assignment_write.assignment_id.clone(),
        assignment_input_queued: assignment_write.input_queued,
    };
    sqlx::query(
        "UPDATE chat_organizational_command_receipts
         SET state = 'completed', result_schema_version = 1, result_data = ?, updated_at = ?
         WHERE client_command_id = ? AND state = 'accepted'",
    )
    .bind(serde_json::to_string(&receipt_locator).map_err(serialization_error)?)
    .bind(now.as_str())
    .bind(request.client_command_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
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
pub async fn chat_schedule_message(
    app: tauri::AppHandle,
    db_url: String,
    request: ScheduleChatMessageCommand,
) -> ChatResult<ChatScheduledMessageRead> {
    scheduling::schedule_message(app, db_url, request).await
}

#[tauri::command]
pub async fn chat_list_scheduled_messages(
    app: tauri::AppHandle,
    db_url: String,
    channel_id: ChatChannelId,
    reply_thread_id: Option<ChatReplyThreadId>,
) -> ChatResult<Vec<ChatScheduledMessageRead>> {
    scheduling::list_scheduled_messages(app, db_url, channel_id, reply_thread_id).await
}

#[tauri::command]
pub async fn chat_cancel_scheduled_message(
    app: tauri::AppHandle,
    db_url: String,
    scheduled_message_id: ChatScheduledMessageId,
) -> ChatResult<()> {
    scheduling::cancel_scheduled_message(app, db_url, scheduled_message_id).await
}

#[tauri::command]
pub async fn chat_retry_scheduled_message(
    app: tauri::AppHandle,
    db_url: String,
    scheduled_message_id: ChatScheduledMessageId,
) -> ChatResult<ChatScheduledMessageRead> {
    scheduling::retry_scheduled_message(app, db_url, scheduled_message_id).await
}

#[tauri::command]
pub async fn chat_send_scheduled_message_now(
    app: tauri::AppHandle,
    db_url: String,
    scheduled_message_id: ChatScheduledMessageId,
) -> ChatResult<PostChatMessageResult> {
    scheduling::send_scheduled_message_now(app, db_url, scheduled_message_id).await
}

#[tauri::command]
pub async fn chat_dispatch_due_scheduled_messages(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<ChatScheduledMessageDispatchRead> {
    scheduling::dispatch_due_scheduled_messages(app, db_url).await
}

#[tauri::command]
pub async fn chat_recover_assignment_dispatch_jobs(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<u32> {
    assignments::recover_assignment_dispatch_jobs(app, db_url).await
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
            participant.id AS author_participant_id,
            participant.participant_kind AS author_participant_kind,
            participant.display_name AS author_display_name,
            snippet(chat_communication_search_fts, 4, '<mark>', '</mark>', '…', 24) AS excerpt,
            item.created_at
         FROM chat_communication_search_fts search
         JOIN chat_conversation_items item ON item.id = search.message_item_id
         JOIN chat_communication_messages message ON message.item_id = item.id
         JOIN chat_participants participant ON participant.id = message.author_participant_id
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
                author_participant_id: ChatParticipantId::new(
                    row.try_get::<String, _>("author_participant_id")
                        .map_err(persistence_error)?,
                )
                .map_err(identifier_error)?,
                author_kind: parse_participant_kind(
                    &row.try_get::<String, _>("author_participant_kind")
                        .map_err(persistence_error)?,
                )?,
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
    assignments::cancel_assignment(app, db_url, assignment_id, expected_revision).await
}

#[tauri::command]
pub async fn chat_retry_assignment(
    app: tauri::AppHandle,
    db_url: String,
    assignment_id: ChatWorkAssignmentId,
    expected_revision: u64,
) -> ChatResult<ChatWorkAssignmentRead> {
    assignments::retry_assignment(app, db_url, assignment_id, expected_revision).await
}

pub(crate) async fn project_provider_event_in_transaction(
    transaction: &mut Transaction<'_, Sqlite>,
    runtime: &super::events::CanonicalRuntimeEvent,
) -> ChatResult<bool> {
    super::coordination::projection::project_provider_event_in_transaction(transaction, runtime)
        .await
}

#[cfg(test)]
mod tests;
