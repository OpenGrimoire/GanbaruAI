//! SQLite read models for organizational Chat state.

use super::super::models::*;
use super::common::{
    parse_approval_policy, parse_configuration_state, parse_json, parse_participant_kind,
    parse_work_state, u32_value,
};
use super::workflow::{parse_model_selection, read_mentions, read_resource_references};
use super::{
    i64_value, identifier_error, now_timestamp, optional_timestamp, persistence_error, timestamp,
    u64_value, LOCAL_PARTICIPANT_ID,
};
use sqlx::{Row, SqlitePool};

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

pub(super) async fn read_membership(
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

pub(super) async fn read_participant(
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

pub(super) async fn read_teammate(
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

pub(super) async fn read_policy(
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

pub(super) async fn read_message(
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

pub(super) async fn read_reply_thread_summary(
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

pub(super) async fn read_reply_thread_page(
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

pub(super) async fn read_assignment(
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

pub(super) async fn read_active_or_latest_assignment(
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

pub(super) async fn read_agent_runs(
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
