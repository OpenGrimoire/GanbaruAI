//! Assignment routing, frozen context, and durable organizational messages.

use super::super::coordination::contracts::*;
use super::super::models::*;
use super::common::{
    conversation_item_id, has_thread_eligible_mention, json_object, message_revision_id, new_id,
    parse_participant_kind, reply_thread_id, serialization_error, wire_approval_policy,
    wire_participant_kind, wire_work_state, work_assignment_id,
};
use super::context::freeze_context_package;
use super::reads::{
    read_active_or_latest_assignment, read_assignment, read_message, read_participant,
};
use super::{
    i64_value, identifier_error, now_timestamp, persistence_error, u64_value,
    StoredPostMessageReceipt, LOCAL_PARTICIPANT_ID,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(super) struct ResolvedInvocation {
    pub(super) teammate_id: ChatParticipantId,
    pub(super) active_assignment: Option<ChatWorkAssignmentRead>,
    pub(super) latest_assignment: Option<ChatWorkAssignmentRead>,
}

pub(super) struct AssignmentWrite {
    pub(super) assignment_id: Option<ChatWorkAssignmentId>,
    pub(super) input_queued: bool,
}

pub(super) async fn resolve_invoked_teammate(
    pool: &SqlitePool,
    conversation_id: &ChatConversationId,
    reply_thread_id: Option<&ChatReplyThreadId>,
    mentions: &[ChatParticipantMentionInput],
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
        .filter(|assignment| assignment.state.accepts_continuation())
        .cloned();
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

pub(super) async fn persist_assignment_routing(
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
    if let Some(previous) = invocation
        .latest_assignment
        .as_ref()
        .filter(|assignment| assignment.state == ChatWorkAssignmentState::ReadyForReview)
    {
        let settled = sqlx::query(
            "UPDATE chat_work_assignments
             SET state = 'completed', settled_at = ?, revision = revision + 1, updated_at = ?
             WHERE id = ? AND state = 'ready_for_review'",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(previous.id.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
        if settled.rows_affected() != 1 {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "The reviewed assignment changed before the follow-up was created",
                true,
            ));
        }
    }
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

pub(super) struct CommunicationMessageWrite<'a> {
    pub(super) item_id: &'a ChatConversationItemId,
    pub(super) revision_id: &'a ChatMessageRevisionId,
    pub(super) conversation_id: &'a ChatConversationId,
    pub(super) reply_thread_id: Option<&'a ChatReplyThreadId>,
    pub(super) ordinal: i64,
    pub(super) request: &'a PostChatMessageCommand,
    pub(super) now: &'a UtcTimestamp,
}

pub(super) async fn insert_communication_message(
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
                 label_snapshot, start_offset, end_offset)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("mention:{}:{index}", write.revision_id.as_str()))
        .bind(write.revision_id.as_str())
        .bind(mention.participant_id.as_str())
        .bind(wire_participant_kind(mention.participant_kind))
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

pub(super) async fn insert_channel_copy(
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
    if has_thread_eligible_mention(request) {
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
    }
    Ok(())
}

pub(super) async fn next_item_ordinal(
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

pub(super) async fn require_reply_thread(
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

pub(super) async fn require_addressable_teammate(
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

pub(super) async fn upsert_membership_in_transaction(
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

pub(super) async fn insert_policy_revision(
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

pub(super) async fn read_mentions(
    pool: &SqlitePool,
    revision_id: &ChatMessageRevisionId,
) -> ChatResult<Vec<ChatParticipantMentionRead>> {
    let rows = sqlx::query(
        "SELECT participant_id, participant_kind, label_snapshot,
                start_offset, end_offset
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
                label_snapshot: row.try_get("label_snapshot").map_err(persistence_error)?,
                start_offset: u64_value(row.try_get("start_offset").map_err(persistence_error)?)?,
                end_offset: u64_value(row.try_get("end_offset").map_err(persistence_error)?)?,
            })
        })
        .collect()
}

pub(super) async fn read_resource_references(
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

pub(super) async fn read_post_receipt(
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
    if let Ok(result) = serde_json::from_str::<PostChatMessageResult>(&data) {
        return Ok(Some(result));
    }
    let locator =
        serde_json::from_str::<StoredPostMessageReceipt>(&data).map_err(serialization_error)?;
    let StoredPostMessageReceipt::Locator {
        message_item_id,
        reply_thread_id,
        assignment_id,
        assignment_input_queued,
    } = locator;
    let message = read_message(pool, &message_item_id).await?;
    let assignment = match assignment_id.as_ref() {
        Some(assignment_id) => Some(read_assignment(pool, assignment_id).await?),
        None => match reply_thread_id.as_ref() {
            Some(reply_thread_id) => {
                read_active_or_latest_assignment(pool, reply_thread_id).await?
            }
            None => None,
        },
    };
    Ok(Some(PostChatMessageResult {
        message,
        reply_thread_id,
        assignment,
        assignment_input_queued,
    }))
}

pub(super) async fn set_assignment_state(
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoredModelSelection {
    pub(super) provider_managed_model: bool,
    pub(super) model_id: Option<ModelId>,
    pub(super) model_options: Vec<ModelOptionSelection>,
}

pub(super) fn parse_model_selection(data: String) -> ChatResult<StoredModelSelection> {
    serde_json::from_str(&data).map_err(serialization_error)
}
