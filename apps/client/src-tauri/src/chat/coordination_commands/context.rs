//! Frozen assignment context and authorization snapshots.

use super::super::models::*;
use super::common::{new_id, sha256_hex, truncate_utf8, MAX_MESSAGE_BYTES};
use super::{
    i64_value, identifier_error, persistence_error, u64_value, MAX_CHANNEL_CONTEXT_MESSAGES,
    MAX_THREAD_CONTEXT_REPLIES,
};
use sqlx::{Row, Sqlite, Transaction};

pub(super) async fn freeze_context_package(
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
