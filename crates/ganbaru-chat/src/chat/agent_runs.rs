//! Durable bridge between organizational assignments and provider turns.

use super::models::{
    ChatAgentRunId, ChatError, ChatErrorCode, ChatResult, ChatTeammatePolicyRevisionId,
    ChatThreadId, ChatTurnId, ChatWorkAssignmentId, ProjectWorkingFolderId, UtcTimestamp,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};

#[derive(Clone, Debug)]
pub struct AgentRunBinding {
    pub run_id: ChatAgentRunId,
    pub assignment_id: ChatWorkAssignmentId,
    pub teammate_policy_revision_id: ChatTeammatePolicyRevisionId,
    pub authorization_decision_id: String,
    pub run_ordinal: u64,
}

#[derive(Clone, Debug, Default)]
pub enum TurnOrigin {
    #[default]
    Direct,
    Assignment {
        developer_instructions: String,
        run: AgentRunBinding,
    },
}

impl TurnOrigin {
    pub fn developer_instructions(&self) -> Option<&str> {
        match self {
            Self::Direct => None,
            Self::Assignment {
                developer_instructions,
                ..
            } => Some(developer_instructions),
        }
    }

    pub fn run(&self) -> Option<&AgentRunBinding> {
        match self {
            Self::Direct => None,
            Self::Assignment { run, .. } => Some(run),
        }
    }
}

pub struct StartingAgentRun<'a> {
    pub binding: &'a AgentRunBinding,
    pub project_id: &'a str,
    pub working_folder_id: &'a ProjectWorkingFolderId,
    pub provider_turn_id: &'a ChatTurnId,
    pub provider_thread_id: &'a ChatThreadId,
    pub now: &'a UtcTimestamp,
}

pub async fn insert_starting_run(
    transaction: &mut Transaction<'_, Sqlite>,
    run: StartingAgentRun<'_>,
) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_agent_runs
            (id, assignment_id, project_id, working_folder_id,
             teammate_policy_revision_id, authorization_decision_id,
             provider_turn_id, provider_thread_id, state, run_ordinal, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'starting', ?, ?, ?)",
    )
    .bind(run.binding.run_id.as_str())
    .bind(run.binding.assignment_id.as_str())
    .bind(run.project_id)
    .bind(run.working_folder_id.as_str())
    .bind(run.binding.teammate_policy_revision_id.as_str())
    .bind(&run.binding.authorization_decision_id)
    .bind(run.provider_turn_id.as_str())
    .bind(run.provider_thread_id.as_str())
    .bind(i64_value(run.binding.run_ordinal)?)
    .bind(run.now.as_str())
    .bind(run.now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

pub async fn settle_launch(
    pool: &sqlx::SqlitePool,
    binding: &AgentRunBinding,
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

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value).map_err(|_| ChatError::validation("number", "Value is too large"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat agent-run persistence failed",
        true,
    )
}

fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat agent-run content could not be encoded",
        false,
    )
}
