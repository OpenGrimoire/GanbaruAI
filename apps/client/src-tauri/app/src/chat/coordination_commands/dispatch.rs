//! Assignment dispatch and live provider delivery.

use super::super::models::*;
use super::common::{new_id, parse_approval_policy, serialization_error, sha256_hex};
use super::reads::read_policy;
use super::{chat_pool, identifier_error, now_timestamp, persistence_error, u64_value};
use serde_json::json;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

pub(super) async fn dispatch_assignment_job(
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
    let result = dispatch_claimed_assignment(
        app.clone(),
        db_url.clone(),
        &pool,
        &assignment_id,
        &claim_token,
    )
    .await;
    if let Err(error) = result {
        fail_assignment_dispatch(&pool, &assignment_id, &claim_token, &error).await?;
        return Err(error);
    }
    mark_initial_assignment_input_delivered(&pool, &assignment_id).await?;
    deliver_pending_assignment_inputs(app, db_url, &pool, &assignment_id).await
}

async fn mark_initial_assignment_input_delivered(
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
) -> ChatResult<()> {
    let now = now_timestamp()?;
    sqlx::query(
        "UPDATE chat_work_assignment_inputs
         SET delivery_state = 'delivered', delivered_at = ?
         WHERE assignment_id = ? AND routing_kind IN ('trigger', 'follow_up')
           AND delivery_state = 'pending'",
    )
    .bind(now.as_str())
    .bind(assignment_id.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

pub(super) async fn deliver_pending_assignment_inputs(
    app: tauri::AppHandle,
    db_url: String,
    pool: &SqlitePool,
    assignment_id: &ChatWorkAssignmentId,
) -> ChatResult<()> {
    let message_ids = sqlx::query_scalar::<_, String>(
        "SELECT message_item_id FROM chat_work_assignment_inputs
         WHERE assignment_id = ? AND routing_kind IN ('steer', 'queued_continuation')
           AND delivery_state = 'pending'
         ORDER BY ordinal",
    )
    .bind(assignment_id.as_str())
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    for message_id in message_ids {
        deliver_assignment_input(
            app.clone(),
            db_url.clone(),
            assignment_id.clone(),
            ChatConversationItemId::new(message_id).map_err(identifier_error)?,
        )
        .await?;
    }
    Ok(())
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
    let developer_instructions: String = row.try_get("instructions").map_err(persistence_error)?;
    let request = super::super::send_commands::SendChatTurnCommand {
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
    };
    super::super::turns::send_turn(
        app,
        db_url,
        super::super::turns::SendChatTurnInvocation {
            command: request,
            origin: super::super::agent_runs::TurnOrigin::Assignment {
                developer_instructions,
                run: super::super::agent_runs::AgentRunBinding {
                    run_id: ChatAgentRunId::new(new_id("agent-run")).map_err(identifier_error)?,
                    assignment_id: assignment_id.clone(),
                    teammate_policy_revision_id: policy.id,
                    authorization_decision_id: row
                        .try_get("authorization_id")
                        .map_err(persistence_error)?,
                    run_ordinal: u64_value(run_ordinal)?,
                },
            },
        },
    )
    .await?;
    Ok(())
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

pub(super) async fn deliver_assignment_input(
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
    super::super::turns::steer_turn(
        app,
        db_url,
        super::super::send_commands::SteerChatTurnCommand {
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
