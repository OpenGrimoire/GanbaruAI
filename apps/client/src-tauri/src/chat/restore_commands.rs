//! Coordinated checkpoint restore preview and execution.

use super::checkpoints::{
    current_git_snapshot, diff_files, read_stored_checkpoint, restore_git_snapshot,
    verify_checkpoint, ChatChangedFileRead, CurrentGitSnapshot, StoredCheckpoint,
};
use super::events::{CanonicalEvent, ThreadRevertedEvent};
use super::models::{
    ChatCheckpointId, ChatCommandContext, ChatError, ChatErrorCode, ChatResult, ChatThreadId,
    ChatTurnId, InterruptTurnRequest, ProjectWorkingFolderId, ProviderCapability, RollbackRequest,
    UtcTimestamp, VersionedJson,
};
use super::providers::{DriverCancellation, DriverOperationContext};
use super::repository::receipts::{
    claim_command_receipt, complete_command_receipt, CommandReceiptClaim, CommandReceiptRead,
    CommandReceiptState,
};
use super::runtime::ChatRuntimeRegistry;
use super::workspace::{AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation};
use crate::db_path;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};
use tauri::Manager;

const RESTORE_PREVIEW_LIFETIME: chrono::Duration = chrono::Duration::minutes(15);
const RESTORE_PHASE_SAFE_RETRY: u8 = 0;
const RESTORE_PHASE_PROVIDER_ROLLBACK: u8 = 1;
const RESTORE_PHASE_WORKSPACE_RESTORE: u8 = 2;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewChatRestoreRequest {
    pub thread_id: ChatThreadId,
    pub checkpoint_id: ChatCheckpointId,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteChatRestoreRequest {
    pub command: ChatCommandContext,
    pub thread_id: ChatThreadId,
    pub preview_id: String,
    pub confirmed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRestorePreviewRead {
    pub preview_id: String,
    pub checkpoint_id: ChatCheckpointId,
    pub expires_at: UtcTimestamp,
    pub files: Vec<ChatChangedFileRead>,
    pub staged_changes: bool,
    pub provider_rollback: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRestoreResultRead {
    pub checkpoint_id: ChatCheckpointId,
    pub reverted_turn_ids: Vec<ChatTurnId>,
    pub provider_history_action: String,
    pub recovery_state: String,
    pub thread_revision: u64,
}

struct StoredRestorePreview {
    id: String,
    thread_id: ChatThreadId,
    checkpoint_id: ChatCheckpointId,
    expected_thread_revision: u64,
    repository_identity: String,
    current: CurrentGitSnapshot,
    expires_at: UtcTimestamp,
}

#[tauri::command]
pub async fn chat_preview_checkpoint_restore(
    app: tauri::AppHandle,
    db_url: String,
    request: PreviewChatRestoreRequest,
) -> ChatResult<ChatRestorePreviewRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let (working_folder_id, authorized, revision) =
        authorize_thread(&app, &pool, &request.thread_id).await?;
    let target = read_stored_checkpoint(&pool, &request.checkpoint_id).await?;
    if target.thread_id != request.thread_id
        || authorized.repository_identity.as_deref() != Some(&target.repository_identity)
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Checkpoint belongs to another thread or repository",
            false,
        ));
    }
    let authorized_for_git = authorized.clone();
    let target_for_git = target.clone();
    let (current, files) = tauri::async_runtime::spawn_blocking(move || {
        verify_checkpoint(&authorized_for_git, &target_for_git)?;
        let current = current_git_snapshot(&authorized_for_git.canonical_path)?;
        require_head_context(&current, &target_for_git)?;
        let current_checkpoint = transient_checkpoint(&target_for_git, &current);
        let files = diff_files(
            &authorized_for_git.canonical_path,
            &current_checkpoint,
            &target_for_git,
        )?;
        Ok::<_, ChatError>((current, files))
    })
    .await
    .map_err(|_| restore_worker_error())??;
    let now = now_timestamp()?;
    let expires_at = add_duration(&now, RESTORE_PREVIEW_LIFETIME)?;
    let preview_id = preview_id(&request.thread_id, &request.checkpoint_id, &current, &now);
    let files_data = serde_json::to_string(&files).map_err(json_error)?;
    sqlx::query(
        "INSERT INTO chat_restore_previews
            (id, thread_id, checkpoint_id, expected_thread_revision, repository_identity,
             head_oid, head_ref, current_worktree_tree_oid, current_index_tree_oid,
             current_index_fingerprint, affected_files_data, state, expires_at, created_at,
             updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'ready', ?, ?, ?)",
    )
    .bind(&preview_id)
    .bind(request.thread_id.as_str())
    .bind(request.checkpoint_id.as_str())
    .bind(i64_value(revision)?)
    .bind(target.repository_identity)
    .bind(current.head_oid.as_deref())
    .bind(current.head_ref.as_deref())
    .bind(&current.worktree_tree_oid)
    .bind(&current.index_tree_oid)
    .bind(&current.index_fingerprint)
    .bind(files_data)
    .bind(expires_at.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    let capabilities = app
        .state::<ChatRuntimeRegistry>()
        .owner(request.thread_id)?
        .snapshot()?
        .capabilities;
    let staged_changes = current.index_tree_oid != target.index_tree_oid;
    let mut warnings = Vec::new();
    if staged_changes {
        warnings.push("The real staging state will be restored to the checkpoint.".to_string());
    }
    if !capabilities.supports(ProviderCapability::NativeRollback) {
        warnings.push(
            "The provider cannot align native history, so the restored conversation will require a fork."
                .to_string(),
        );
    }
    if working_folder_id != authorized.working_folder_id {
        return Err(restore_worker_error());
    }
    Ok(ChatRestorePreviewRead {
        preview_id,
        checkpoint_id: request.checkpoint_id,
        expires_at,
        files,
        staged_changes,
        provider_rollback: if capabilities.supports(ProviderCapability::NativeRollback) {
            "supported"
        } else {
            "unsupported"
        }
        .to_string(),
        warnings,
    })
}

#[tauri::command]
pub async fn chat_execute_checkpoint_restore(
    app: tauri::AppHandle,
    db_url: String,
    request: ExecuteChatRestoreRequest,
) -> ChatResult<ChatRestoreResultRead> {
    if !request.confirmed {
        return Err(ChatError::validation(
            "confirmed",
            "Checkpoint restore requires explicit confirmation",
        ));
    }
    let pool = chat_pool(app.clone(), db_url).await?;
    let now = now_timestamp()?;
    match claim_command_receipt(
        &pool,
        &request.command.client_command_id,
        &request.thread_id,
        "restore_checkpoint",
        request.command.expected_thread_revision,
        &now,
    )
    .await?
    {
        CommandReceiptClaim::Replay(receipt) => return replay_restore(receipt),
        CommandReceiptClaim::Claimed(_) => {}
    }
    let result = execute_restore(&app, &pool, &request, &now).await;
    match result {
        Ok(result) => {
            let value = versioned_value(&result)?;
            complete_command_receipt(
                &pool,
                &request.command.client_command_id,
                CommandReceiptState::Completed,
                Some(&value),
                None,
                &now_timestamp()?,
            )
            .await?;
            Ok(result)
        }
        Err(error) => {
            let value = versioned_error(&error)?;
            complete_command_receipt(
                &pool,
                &request.command.client_command_id,
                CommandReceiptState::Failed,
                None,
                Some(&value),
                &now_timestamp()?,
            )
            .await?;
            Err(error)
        }
    }
}

async fn execute_restore(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &ExecuteChatRestoreRequest,
    now: &UtcTimestamp,
) -> ChatResult<ChatRestoreResultRead> {
    let preview = read_preview(pool, &request.preview_id).await?;
    if preview.thread_id != request.thread_id
        || preview.expected_thread_revision
            != request
                .command
                .expected_thread_revision
                .unwrap_or(preview.expected_thread_revision)
        || preview.expires_at.as_str() < now.as_str()
    {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Checkpoint restore preview is stale",
            true,
        ));
    }
    let (_, authorized, revision) = authorize_thread(app, pool, &request.thread_id).await?;
    if revision != preview.expected_thread_revision
        || authorized.repository_identity.as_deref() != Some(&preview.repository_identity)
    {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Thread or repository changed after the restore preview",
            true,
        ));
    }
    let target = read_stored_checkpoint(pool, &preview.checkpoint_id).await?;
    let current = {
        let authorized = authorized.clone();
        let target = target.clone();
        tauri::async_runtime::spawn_blocking(move || {
            verify_checkpoint(&authorized, &target)?;
            let current = current_git_snapshot(&authorized.canonical_path)?;
            require_head_context(&current, &target)?;
            Ok::<_, ChatError>(current)
        })
        .await
        .map_err(|_| restore_worker_error())??
    };
    if current.worktree_tree_oid != preview.current.worktree_tree_oid
        || current.index_tree_oid != preview.current.index_tree_oid
        || current.index_fingerprint != preview.current.index_fingerprint
        || current.head_oid != preview.current.head_oid
        || current.head_ref != preview.current.head_ref
    {
        mark_preview_state(pool, &preview.id, "stale", now).await?;
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Workspace changed after the restore preview",
            true,
        ));
    }
    mark_preview_state(pool, &preview.id, "executing", now).await?;
    let failure_phase = AtomicU8::new(RESTORE_PHASE_SAFE_RETRY);
    let execution: ChatResult<ChatRestoreResultRead> = async {
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(request.thread_id.clone())?;
    let snapshot = owner.snapshot()?;
    if let (Some(session_id), Some(turn_id)) =
        (snapshot.session_id.clone(), snapshot.active_turn_id.clone())
    {
        owner
            .interrupt_turn(
                InterruptTurnRequest {
                    command: request.command.clone(),
                    session_id,
                    turn_id,
                },
                operation_context("restore-interrupt", Duration::from_secs(15)),
            )
            .await?;
    }
    let _mutation_guard = app
        .state::<super::workspace_mutation::ChatWorkspaceMutationRegistry>()
        .mutation_with_timeout(&authorized.canonical_path, Duration::from_secs(30))
        .await?;
    let current = {
        let authorized = authorized.clone();
        let target = target.clone();
        tauri::async_runtime::spawn_blocking(move || {
            verify_checkpoint(&authorized, &target)?;
            let current = current_git_snapshot(&authorized.canonical_path)?;
            require_head_context(&current, &target)?;
            Ok::<_, ChatError>(current)
        })
        .await
        .map_err(|_| restore_worker_error())??
    };
    if current.worktree_tree_oid != preview.current.worktree_tree_oid
        || current.index_tree_oid != preview.current.index_tree_oid
        || current.index_fingerprint != preview.current.index_fingerprint
        || current.head_oid != preview.current.head_oid
        || current.head_ref != preview.current.head_ref
    {
        mark_preview_state(pool, &preview.id, "stale", now).await?;
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Workspace changed while the active coding agent was stopping",
            true,
        ));
    }
    let refreshed = owner.snapshot()?;
    let target_turn_id = target_turn_id(pool, &request.thread_id, target.turn_count).await?;
    let provider_cursor = match target_turn_id.as_ref() {
        Some(turn_id) => stored_provider_rollback_cursor(pool, &request.thread_id, turn_id).await?,
        None => None,
    };
    let provider_history_action = if refreshed
        .capabilities
        .supports(ProviderCapability::NativeRollback)
        && refreshed.session_id.is_some()
        && refreshed.session_state == super::models::ProviderSessionState::Ready
        && provider_cursor.is_some()
    {
        failure_phase.store(RESTORE_PHASE_PROVIDER_ROLLBACK, Ordering::Release);
        owner
            .rollback(
                RollbackRequest {
                    command: request.command.clone(),
                    session_id: refreshed
                        .session_id
                        .clone()
                        .ok_or_else(restore_worker_error)?,
                    checkpoint_id: Some(target.id.clone()),
                    provider_cursor,
                    target_turn_id,
                },
                operation_context("restore-provider-rollback", Duration::from_secs(30)),
            )
            .await?;
        "rolled_back"
    } else {
        "fork_required"
    };
    if provider_history_action == "fork_required" {
        failure_phase.store(RESTORE_PHASE_WORKSPACE_RESTORE, Ordering::Release);
    }
    let root = authorized.canonical_path.clone();
    let current_for_restore = current.clone();
    let target_for_restore = target.clone();
    let restore_result = tauri::async_runtime::spawn_blocking(move || {
        restore_git_snapshot(&root, &current_for_restore, &target_for_restore)
    })
    .await
    .map_err(|_| restore_worker_error())?;
    if let Err(error) = restore_result {
        record_restore_operation(
            pool,
            request,
            &target,
            provider_history_action,
            "recovery_required",
            Some(&error),
            now,
        )
        .await?;
        mark_preview_state(pool, &preview.id, "failed", now).await?;
        return Err(ChatError::new(
            ChatErrorCode::Internal,
            "Workspace restore failed after provider coordination. Checkpoint refs were retained for recovery.",
            true,
        ));
    }
    if provider_history_action == "fork_required" && refreshed.session_id.is_some() {
        let _ = owner
            .stop_session(
                true,
                operation_context("restore-provider-fork", Duration::from_secs(30)),
            )
            .await;
        if owner.snapshot()?.session_id.is_some() {
            let error = ChatError::new(
                ChatErrorCode::Internal,
                "The workspace was restored, but the previous provider session could not be detached",
                true,
            );
            record_restore_operation(
                pool,
                request,
                &target,
                provider_history_action,
                "recovery_required",
                Some(&error),
                now,
            )
            .await?;
            mark_preview_state(pool, &preview.id, "failed", now).await?;
            return Err(error);
        }
    }
    let reverted_turn_ids = match persist_restore(
        pool,
        request,
        &preview,
        &target,
        provider_history_action,
        now,
    )
    .await
    {
        Ok(turn_ids) => turn_ids,
        Err(error) => {
            let _ = record_restore_operation(
                pool,
                request,
                &target,
                provider_history_action,
                "recovery_required",
                Some(&error),
                now,
            )
            .await;
            let _ = mark_preview_state(pool, &preview.id, "failed", now).await;
            return Err(ChatError::new(
                ChatErrorCode::Persistence,
                "The workspace was restored, but Chat history reconciliation failed. Checkpoint refs were retained for recovery.",
                true,
            ));
        }
    };
    mark_preview_state(pool, &preview.id, "completed", now).await?;
    let thread_revision: i64 = sqlx::query_scalar("SELECT revision FROM chat_threads WHERE id = ?")
        .bind(request.thread_id.as_str())
        .fetch_one(pool)
        .await
        .map_err(persistence_error)?;
    Ok(ChatRestoreResultRead {
        checkpoint_id: target.id.clone(),
        reverted_turn_ids,
        provider_history_action: provider_history_action.to_string(),
        recovery_state: "complete".to_string(),
        thread_revision: u64::try_from(thread_revision).map_err(|_| corrupt_data())?,
    })
    }
    .await;
    if let Err(error) = execution.as_ref() {
        settle_executing_preview_failure(
            pool,
            request,
            &preview,
            &target,
            failure_phase.load(Ordering::Acquire),
            error,
            now,
        )
        .await?;
    }
    execution
}

async fn persist_restore(
    pool: &SqlitePool,
    request: &ExecuteChatRestoreRequest,
    preview: &StoredRestorePreview,
    target: &StoredCheckpoint,
    provider_history_action: &str,
    now: &UtcTimestamp,
) -> ChatResult<Vec<ChatTurnId>> {
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let rows = sqlx::query(
        "SELECT id FROM chat_turns
         WHERE thread_id = ? AND ordinal >= ? AND invalidated_at IS NULL ORDER BY ordinal",
    )
    .bind(request.thread_id.as_str())
    .bind(i64_value(target.turn_count)?)
    .fetch_all(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    let reverted_turn_ids = rows
        .into_iter()
        .map(|row| {
            ChatTurnId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
                .map_err(|_| corrupt_data())
        })
        .collect::<ChatResult<Vec<_>>>()?;
    sqlx::query(
        "UPDATE chat_events SET invalidated_at = ?, invalidation_reason = 'checkpoint_restore'
         WHERE thread_id = ? AND invalidated_at IS NULL AND turn_id IN (
             SELECT id FROM chat_turns WHERE thread_id = ? AND ordinal >= ?
         )",
    )
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .bind(request.thread_id.as_str())
    .bind(i64_value(target.turn_count)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_turns SET invalidated_at = ?, invalidation_reason = 'checkpoint_restore'
         WHERE thread_id = ? AND ordinal >= ? AND invalidated_at IS NULL",
    )
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .bind(i64_value(target.turn_count)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_pending_requests SET resolution_state = 'stale', resolved_at = ?
         WHERE thread_id = ? AND resolution_state = 'open'",
    )
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_queued_followups SET state = 'cancelled', updated_at = ?
         WHERE thread_id = ? AND state = 'queued'",
    )
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    enqueue_invalidated_checkpoints(&mut transaction, &request.thread_id, target, now).await?;
    append_reverted_event(
        &mut transaction,
        request,
        target,
        &reverted_turn_ids,
        provider_history_action,
        now,
    )
    .await?;
    sqlx::query(
        "UPDATE chat_threads
         SET state = 'idle', latest_turn_state = (
                SELECT state FROM chat_turns
                WHERE thread_id = chat_threads.id AND invalidated_at IS NULL
                ORDER BY ordinal DESC LIMIT 1
             ),
             provider_thread_id = CASE WHEN ? = 'fork_required' THEN NULL ELSE provider_thread_id END,
             resume_cursor_schema_version = CASE
                 WHEN ? = 'fork_required' THEN NULL ELSE resume_cursor_schema_version END,
             resume_cursor_data = CASE
                 WHEN ? = 'fork_required' THEN NULL ELSE resume_cursor_data END,
             message_count = (
                SELECT COUNT(*) FROM chat_messages
                WHERE thread_id = chat_threads.id AND (
                    turn_id IS NULL OR EXISTS (
                        SELECT 1 FROM chat_turns
                        WHERE id = chat_messages.turn_id AND invalidated_at IS NULL
                    )
                )
             ),
             latest_preview = (
                SELECT substr(normalized_markdown, -2000) FROM chat_messages
                WHERE thread_id = chat_threads.id AND (
                    turn_id IS NULL OR EXISTS (
                        SELECT 1 FROM chat_turns
                        WHERE id = chat_messages.turn_id AND invalidated_at IS NULL
                    )
                )
                ORDER BY sequence_anchor DESC, id DESC LIMIT 1
             ),
             changed_file_summary_schema_version = NULL,
             changed_file_summary_data = NULL,
             revision = revision + 1, updated_at = ?, last_activity_at = ?
         WHERE id = ?",
    )
    .bind(provider_history_action)
    .bind(provider_history_action)
    .bind(provider_history_action)
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_restore_operations
            (id, thread_id, checkpoint_id, preview_id, provider_history_action,
             recovery_state, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'complete', ?, ?)",
    )
    .bind(format!(
        "restore-operation:{}",
        request.command.client_command_id.as_str()
    ))
    .bind(request.thread_id.as_str())
    .bind(target.id.as_str())
    .bind(&preview.id)
    .bind(provider_history_action)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)?;
    Ok(reverted_turn_ids)
}

async fn enqueue_invalidated_checkpoints(
    transaction: &mut Transaction<'_, Sqlite>,
    thread_id: &ChatThreadId,
    target: &StoredCheckpoint,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let rows = sqlx::query(
        "SELECT id, hidden_ref_name, git_object_id FROM chat_checkpoints
         WHERE thread_id = ? AND turn_count > ? AND status = 'available'",
    )
    .bind(thread_id.as_str())
    .bind(i64_value(target.turn_count)?)
    .fetch_all(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    for row in rows {
        let checkpoint_id: String = row.try_get("id").map_err(persistence_error)?;
        let reference: String = row.try_get("hidden_ref_name").map_err(persistence_error)?;
        let object_id: String = row.try_get("git_object_id").map_err(persistence_error)?;
        sqlx::query(
            "INSERT OR IGNORE INTO chat_cleanup_queue
                (id, source_thread_id, cleanup_kind, exact_target, repository_identity,
                 state, not_before, created_at, updated_at, working_folder_id, expected_object_id)
             VALUES (?, ?, 'checkpoint_ref', ?, ?, 'pending', ?, ?, ?, (
                SELECT working_folder_id FROM chat_threads WHERE id = ?
             ), ?)",
        )
        .bind(format!("cleanup:checkpoint:{checkpoint_id}"))
        .bind(thread_id.as_str())
        .bind(&reference)
        .bind(&target.repository_identity)
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(thread_id.as_str())
        .bind(&object_id)
        .execute(&mut **transaction)
        .await
        .map_err(persistence_error)?;
    }
    sqlx::query(
        "UPDATE chat_checkpoints
         SET status = 'invalid', invalidated_at = ?, invalidated_by_checkpoint_id = ?,
             cleanup_state = 'queued'
         WHERE thread_id = ? AND turn_count > ? AND status = 'available'",
    )
    .bind(now.as_str())
    .bind(target.id.as_str())
    .bind(thread_id.as_str())
    .bind(i64_value(target.turn_count)?)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn append_reverted_event(
    transaction: &mut Transaction<'_, Sqlite>,
    request: &ExecuteChatRestoreRequest,
    target: &StoredCheckpoint,
    reverted_turn_ids: &[ChatTurnId],
    provider_history_action: &str,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let thread = sqlx::query(
        "SELECT last_event_sequence, provider_family_id, provider_instance_id
         FROM chat_threads WHERE id = ?",
    )
    .bind(request.thread_id.as_str())
    .fetch_one(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    let sequence = u64::try_from(
        thread
            .try_get::<i64, _>("last_event_sequence")
            .map_err(persistence_error)?,
    )
    .map_err(|_| corrupt_data())?
    .saturating_add(1);
    let event = CanonicalEvent::ThreadReverted(ThreadRevertedEvent {
        checkpoint_id: target.id.clone(),
        reverted_turn_ids: reverted_turn_ids.to_vec(),
        provider_history_action: provider_history_action.to_string(),
    });
    sqlx::query(
        "INSERT INTO chat_events
            (id, thread_id, sequence, event_schema_version, provider_family_id,
             provider_instance_id, event_type, payload_schema_version, payload_data,
             created_at, ingested_at)
         VALUES (?, ?, ?, 1, ?, ?, 'thread_reverted', 1, ?, ?, ?)",
    )
    .bind(format!(
        "event:restore:{}",
        request.command.client_command_id.as_str()
    ))
    .bind(request.thread_id.as_str())
    .bind(i64_value(sequence)?)
    .bind(
        thread
            .try_get::<String, _>("provider_family_id")
            .map_err(persistence_error)?,
    )
    .bind(
        thread
            .try_get::<String, _>("provider_instance_id")
            .map_err(persistence_error)?,
    )
    .bind(serde_json::to_string(&event).map_err(json_error)?)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_activities
            (id, thread_id, sequence_anchor, item_kind, status, title, detail,
             safe_metadata_data, source_event_type, completed_at, created_at, updated_at)
         VALUES (?, ?, ?, 'notice', 'completed', 'thread_reverted', NULL, ?,
                 'thread_reverted', ?, ?, ?)",
    )
    .bind(format!(
        "activity:restore:{}",
        request.command.client_command_id.as_str()
    ))
    .bind(request.thread_id.as_str())
    .bind(i64_value(sequence)?)
    .bind(
        serde_json::to_string(&json!({
            "checkpointId": target.id,
            "revertedTurnCount": reverted_turn_ids.len(),
            "providerHistoryAction": provider_history_action,
        }))
        .map_err(json_error)?,
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_threads
         SET last_event_sequence = ?, last_projected_sequence = ?
         WHERE id = ?",
    )
    .bind(i64_value(sequence)?)
    .bind(i64_value(sequence)?)
    .bind(request.thread_id.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn read_preview(pool: &SqlitePool, preview_id: &str) -> ChatResult<StoredRestorePreview> {
    let row = sqlx::query(
        "SELECT id, thread_id, checkpoint_id, expected_thread_revision,
                repository_identity, head_oid, head_ref, current_worktree_tree_oid,
                current_index_tree_oid, current_index_fingerprint, expires_at
         FROM chat_restore_previews WHERE id = ? AND state = 'ready'",
    )
    .bind(preview_id)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Checkpoint restore preview is unavailable",
            true,
        )
    })?;
    Ok(StoredRestorePreview {
        id: row.try_get("id").map_err(persistence_error)?,
        thread_id: ChatThreadId::new(
            row.try_get::<String, _>("thread_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        checkpoint_id: ChatCheckpointId::new(
            row.try_get::<String, _>("checkpoint_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        expected_thread_revision: u64::try_from(
            row.try_get::<i64, _>("expected_thread_revision")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        repository_identity: row
            .try_get("repository_identity")
            .map_err(persistence_error)?,
        current: CurrentGitSnapshot {
            worktree_commit_oid: String::new(),
            worktree_tree_oid: row
                .try_get("current_worktree_tree_oid")
                .map_err(persistence_error)?,
            index_tree_oid: row
                .try_get("current_index_tree_oid")
                .map_err(persistence_error)?,
            index_fingerprint: row
                .try_get("current_index_fingerprint")
                .map_err(persistence_error)?,
            head_oid: row.try_get("head_oid").map_err(persistence_error)?,
            head_ref: row.try_get("head_ref").map_err(persistence_error)?,
        },
        expires_at: UtcTimestamp::new(
            row.try_get::<String, _>("expires_at")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
    })
}

async fn authorize_thread(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<(ProjectWorkingFolderId, AuthorizedWorkingFolder, u64)> {
    let (working_folder_id, _, authorized, revision) =
        super::execution_environment::authorize_thread_environment(
            app,
            pool,
            thread_id,
            WorkingFolderAuthorizationOperation::Restore,
        )
        .await?;
    Ok((working_folder_id, authorized, revision))
}

async fn target_turn_id(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_count: u64,
) -> ChatResult<Option<ChatTurnId>> {
    if turn_count == 0 {
        return Ok(None);
    }
    sqlx::query_scalar::<_, String>("SELECT id FROM chat_turns WHERE thread_id = ? AND ordinal = ?")
        .bind(thread_id.as_str())
        .bind(i64_value(turn_count - 1)?)
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .map(ChatTurnId::new)
        .transpose()
        .map_err(|_| corrupt_data())
}

async fn stored_provider_rollback_cursor(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
) -> ChatResult<Option<VersionedJson>> {
    let row = sqlx::query(
        "SELECT provider_reference_schema_version, provider_reference_data, provider_item_id
         FROM chat_events
         WHERE thread_id = ? AND turn_id = ? AND provider_family_id = 'opencode'
           AND invalidated_at IS NULL
           AND (
               (
                   provider_reference_schema_version IS NOT NULL
                   AND provider_reference_data IS NOT NULL
                   AND length(CAST(provider_reference_data AS BLOB)) <= 8192
                   AND json_type(provider_reference_data, '$.messageId') = 'text'
               )
               OR (event_type = 'session_configured' AND provider_item_id IS NOT NULL)
           )
         ORDER BY sequence DESC, id DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .bind(turn_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    let Some(row) = row else { return Ok(None) };
    let schema_version = row
        .try_get::<Option<i64>, _>("provider_reference_schema_version")
        .map_err(persistence_error)?;
    let data = row
        .try_get::<Option<String>, _>("provider_reference_data")
        .map_err(persistence_error)?;
    let message_id = row
        .try_get::<Option<String>, _>("provider_item_id")
        .map_err(persistence_error)?;
    stored_rollback_cursor(schema_version, data.as_deref(), message_id.as_deref()).map(Some)
}

fn stored_rollback_cursor(
    schema_version: Option<i64>,
    data: Option<&str>,
    fallback_message_id: Option<&str>,
) -> ChatResult<VersionedJson> {
    if let (Some(schema_version), Some(data)) = (schema_version, data) {
        return Ok(VersionedJson {
            schema_version: u32::try_from(schema_version).map_err(|_| corrupt_data())?,
            value: serde_json::from_str(data).map_err(json_error)?,
        });
    }
    let message_id = fallback_message_id.ok_or_else(corrupt_data)?;
    Ok(VersionedJson {
        schema_version: 1,
        value: json!({ "messageId": message_id, "partId": null }),
    })
}

async fn mark_preview_state(
    pool: &SqlitePool,
    preview_id: &str,
    state: &str,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    sqlx::query("UPDATE chat_restore_previews SET state = ?, updated_at = ? WHERE id = ?")
        .bind(state)
        .bind(now.as_str())
        .bind(preview_id)
        .execute(pool)
        .await
        .map_err(persistence_error)?;
    Ok(())
}

async fn settle_executing_preview_failure(
    pool: &SqlitePool,
    request: &ExecuteChatRestoreRequest,
    preview: &StoredRestorePreview,
    target: &StoredCheckpoint,
    failure_phase: u8,
    error: &ChatError,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let operation_id = format!(
        "restore-operation:{}",
        request.command.client_command_id.as_str()
    );
    let completed = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM chat_restore_operations
         WHERE id = ? AND recovery_state = 'complete'",
    )
    .bind(&operation_id)
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?
        == 1;
    let state = preview_state_after_failure(completed, failure_phase);
    let transition = sqlx::query(
        "UPDATE chat_restore_previews SET state = ?, updated_at = ?
         WHERE id = ? AND state = 'executing'",
    )
    .bind(state)
    .bind(now.as_str())
    .bind(&preview.id)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    if transition.rows_affected() == 0 || completed || failure_phase == RESTORE_PHASE_SAFE_RETRY {
        return Ok(());
    }
    let provider_action = if failure_phase == RESTORE_PHASE_PROVIDER_ROLLBACK {
        "rolled_back"
    } else {
        "fork_required"
    };
    sqlx::query(
        "INSERT OR IGNORE INTO chat_restore_operations
            (id, thread_id, checkpoint_id, preview_id, provider_history_action,
             recovery_state, error_code, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'recovery_required', ?, ?, ?)",
    )
    .bind(operation_id)
    .bind(request.thread_id.as_str())
    .bind(target.id.as_str())
    .bind(&preview.id)
    .bind(provider_action)
    .bind(format!("{:?}", error.code).to_lowercase())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

fn preview_state_after_failure(completed_operation: bool, failure_phase: u8) -> &'static str {
    if completed_operation {
        "completed"
    } else if failure_phase == RESTORE_PHASE_SAFE_RETRY {
        "ready"
    } else {
        "failed"
    }
}

async fn record_restore_operation(
    pool: &SqlitePool,
    request: &ExecuteChatRestoreRequest,
    target: &StoredCheckpoint,
    provider_action: &str,
    recovery_state: &str,
    error: Option<&ChatError>,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_restore_operations
            (id, thread_id, checkpoint_id, preview_id, provider_history_action,
             recovery_state, error_code, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(format!(
        "restore-operation:{}",
        request.command.client_command_id.as_str()
    ))
    .bind(request.thread_id.as_str())
    .bind(target.id.as_str())
    .bind(&request.preview_id)
    .bind(provider_action)
    .bind(recovery_state)
    .bind(error.map(|value| format!("{:?}", value.code).to_lowercase()))
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

fn transient_checkpoint(
    target: &StoredCheckpoint,
    current: &CurrentGitSnapshot,
) -> StoredCheckpoint {
    StoredCheckpoint {
        id: target.id.clone(),
        thread_id: target.thread_id.clone(),
        turn_count: target.turn_count,
        repository_identity: target.repository_identity.clone(),
        hidden_ref_name: target.hidden_ref_name.clone(),
        git_object_id: current.worktree_commit_oid.clone(),
        index_tree_oid: current.index_tree_oid.clone(),
        worktree_tree_oid: current.worktree_tree_oid.clone(),
        head_oid: current.head_oid.clone(),
        head_ref: current.head_ref.clone(),
    }
}

fn require_head_context(current: &CurrentGitSnapshot, target: &StoredCheckpoint) -> ChatResult<()> {
    if current.head_oid != target.head_oid || current.head_ref != target.head_ref {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Current Git HEAD context differs from the checkpoint",
            true,
        ));
    }
    Ok(())
}

fn replay_restore(receipt: CommandReceiptRead) -> ChatResult<ChatRestoreResultRead> {
    match receipt.state {
        CommandReceiptState::Completed => {
            let result = receipt.result.ok_or_else(corrupt_data)?;
            serde_json::from_value(serde_json::to_value(result.value).map_err(json_error)?)
                .map_err(json_error)
        }
        CommandReceiptState::Failed => Err(ChatError::new(
            ChatErrorCode::Conflict,
            "This checkpoint restore previously failed",
            true,
        )),
        CommandReceiptState::Accepted => Err(ChatError::new(
            ChatErrorCode::Busy,
            "This checkpoint restore is already running",
            true,
        )),
    }
}

fn versioned_value<T: Serialize>(value: &T) -> ChatResult<VersionedJson> {
    Ok(VersionedJson {
        schema_version: 1,
        value: serde_json::to_value(value).map_err(json_error)?,
    })
}

fn versioned_error(error: &ChatError) -> ChatResult<VersionedJson> {
    versioned_value(&json!({
        "code": format!("{:?}", error.code).to_lowercase(),
        "message": error.message,
        "recoverable": error.recoverable,
    }))
}

fn preview_id(
    thread_id: &ChatThreadId,
    checkpoint_id: &ChatCheckpointId,
    current: &CurrentGitSnapshot,
    now: &UtcTimestamp,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(thread_id.as_str());
    hasher.update(checkpoint_id.as_str());
    hasher.update(&current.worktree_tree_oid);
    hasher.update(&current.index_tree_oid);
    hasher.update(now.as_str());
    format!("restore-preview:{:x}", hasher.finalize())
}

fn add_duration(timestamp: &UtcTimestamp, duration: chrono::Duration) -> ChatResult<UtcTimestamp> {
    let parsed = chrono::DateTime::parse_from_rfc3339(timestamp.as_str())
        .map_err(|_| corrupt_data())?
        .with_timezone(&Utc)
        + duration;
    UtcTimestamp::new(parsed.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| corrupt_data())
}

fn operation_context(operation_id: &str, timeout: Duration) -> DriverOperationContext {
    DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline: Instant::now() + timeout,
        cancellation: DriverCancellation::default(),
    }
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value).map_err(|_| ChatError::validation("number", "Value is too large"))
}

fn restore_worker_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Checkpoint restore worker stopped",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Checkpoint restore persistence failed",
        true,
    )
}

fn json_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Checkpoint restore metadata is invalid",
        false,
    )
}

fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored checkpoint restore data is invalid",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_opencode_cursor_preserves_native_message_and_part_ids() {
        let cursor = stored_rollback_cursor(
            Some(1),
            Some(r#"{"messageId":"message-7","partId":"part-2","source":"message.updated"}"#),
            None,
        )
        .expect("stored cursor should parse");

        assert_eq!(cursor.schema_version, 1);
        assert_eq!(cursor.value["messageId"], "message-7");
        assert_eq!(cursor.value["partId"], "part-2");
    }

    #[test]
    fn legacy_opencode_message_reference_becomes_a_rollback_cursor() {
        let cursor = stored_rollback_cursor(None, None, Some("message-legacy"))
            .expect("legacy message reference should convert");

        assert_eq!(cursor.schema_version, 1);
        assert_eq!(cursor.value["messageId"], "message-legacy");
        assert!(cursor.value["partId"].is_null());
    }

    #[test]
    fn executing_preview_failures_never_remain_executing() {
        assert_eq!(
            preview_state_after_failure(false, RESTORE_PHASE_SAFE_RETRY),
            "ready"
        );
        assert_eq!(
            preview_state_after_failure(false, RESTORE_PHASE_PROVIDER_ROLLBACK),
            "failed"
        );
        assert_eq!(
            preview_state_after_failure(false, RESTORE_PHASE_WORKSPACE_RESTORE),
            "failed"
        );
        assert_eq!(
            preview_state_after_failure(true, RESTORE_PHASE_WORKSPACE_RESTORE),
            "completed"
        );
    }
}
