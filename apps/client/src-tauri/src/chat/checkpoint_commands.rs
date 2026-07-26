//! Authorized checkpoint diff and cleanup commands.

use super::checkpoints::{
    delete_exact_ref, diff_files, file_diff, read_stored_checkpoint, verify_checkpoint,
    ChatChangedFileRead, ChatCheckpointFileDiffRead,
};
use super::events::ChangedFileSummary;
use super::models::UtcTimestamp;
use super::models::{
    ChatCheckpointId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatTurnId,
    ProjectWorkingFolderId,
};
use super::repository::workspaces;
use super::workspace::{
    authorize_workspace, AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::db_path;
use crate::projects::working_folders::read_active_working_folder_scope;
use chrono::{SecondsFormat, Utc};
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCheckpointDiffRead {
    pub scope: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub pre_checkpoint_id: Option<ChatCheckpointId>,
    pub post_checkpoint_id: Option<ChatCheckpointId>,
    pub files: Vec<ChatChangedFileRead>,
    pub additions: u64,
    pub deletions: u64,
    pub provider_mismatch: bool,
}

#[tauri::command]
pub async fn chat_read_checkpoint_diff(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    scope: String,
    turn_id: Option<ChatTurnId>,
) -> ChatResult<ChatCheckpointDiffRead> {
    validate_scope(&scope)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let (working_folder_id, authorized) = authorize_thread(&app, &pool, &thread_id).await?;
    let provider_files = provider_files(&pool, &thread_id, turn_id.as_ref(), &scope).await?;
    let Some((pre_id, post_id)) =
        checkpoint_pair(&pool, &thread_id, turn_id.as_ref(), &scope).await?
    else {
        return Ok(unavailable_diff(
            scope,
            provider_files,
            "A settled Git checkpoint pair is not available yet",
        ));
    };
    let pre = read_stored_checkpoint(&pool, &pre_id).await?;
    let post = read_stored_checkpoint(&pool, &post_id).await?;
    if pre.thread_id != thread_id
        || post.thread_id != thread_id
        || pre.repository_identity != post.repository_identity
        || authorized.working_folder_id != working_folder_id
    {
        return Ok(unavailable_diff(
            scope,
            provider_files,
            "Checkpoint identity does not match this thread",
        ));
    }
    let authorized_for_git = authorized.clone();
    let pre_for_git = pre.clone();
    let post_for_git = post.clone();
    let git_files = tauri::async_runtime::spawn_blocking(move || {
        verify_checkpoint(&authorized_for_git, &pre_for_git)?;
        verify_checkpoint(&authorized_for_git, &post_for_git)?;
        diff_files(
            &authorized_for_git.canonical_path,
            &pre_for_git,
            &post_for_git,
        )
    })
    .await
    .map_err(|_| checkpoint_command_error())??;
    let files = merge_provider_and_git(provider_files, git_files);
    let additions = files.iter().filter_map(|file| file.additions).sum();
    let deletions = files.iter().filter_map(|file| file.deletions).sum();
    let provider_mismatch = files
        .iter()
        .any(|file| file.provider_reported != file.git_observed);
    Ok(ChatCheckpointDiffRead {
        scope,
        available: true,
        unavailable_reason: None,
        pre_checkpoint_id: Some(pre_id),
        post_checkpoint_id: Some(post_id),
        files,
        additions,
        deletions,
        provider_mismatch,
    })
}

#[tauri::command]
pub async fn chat_read_checkpoint_file_diff(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    pre_checkpoint_id: ChatCheckpointId,
    post_checkpoint_id: ChatCheckpointId,
    relative_path: String,
    ignore_whitespace: bool,
) -> ChatResult<ChatCheckpointFileDiffRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let (_, authorized) = authorize_thread(&app, &pool, &thread_id).await?;
    let pre = read_stored_checkpoint(&pool, &pre_checkpoint_id).await?;
    let post = read_stored_checkpoint(&pool, &post_checkpoint_id).await?;
    if pre.thread_id != thread_id || post.thread_id != thread_id {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Checkpoint pair belongs to another Chat thread",
            false,
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        verify_checkpoint(&authorized, &pre)?;
        verify_checkpoint(&authorized, &post)?;
        file_diff(
            &authorized.canonical_path,
            &pre,
            &post,
            &relative_path,
            ignore_whitespace,
        )
    })
    .await
    .map_err(|_| checkpoint_command_error())?
}

#[tauri::command]
pub async fn chat_run_checkpoint_cleanup(app: tauri::AppHandle, db_url: String) -> ChatResult<u64> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let now = now_timestamp()?;
    let rows = sqlx::query(
        "SELECT id, working_folder_id, exact_target, repository_identity, expected_object_id
         FROM chat_cleanup_queue
         WHERE cleanup_kind = 'checkpoint_ref' AND state IN ('pending', 'failed')
           AND not_before <= ?
         ORDER BY not_before, id LIMIT 100",
    )
    .bind(now.as_str())
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    let mut completed = 0;
    for row in rows {
        let cleanup_id: String = row.try_get("id").map_err(persistence_error)?;
        let working_folder_id: Option<String> = row
            .try_get("working_folder_id")
            .map_err(persistence_error)?;
        let reference: String = row.try_get("exact_target").map_err(persistence_error)?;
        let repository_identity: Option<String> = row
            .try_get("repository_identity")
            .map_err(persistence_error)?;
        let expected_object_id: Option<String> = row
            .try_get("expected_object_id")
            .map_err(persistence_error)?;
        sqlx::query(
            "UPDATE chat_cleanup_queue
             SET state = 'running', attempts = attempts + 1, updated_at = ? WHERE id = ?",
        )
        .bind(now.as_str())
        .bind(&cleanup_id)
        .execute(&pool)
        .await
        .map_err(persistence_error)?;
        let result = match (working_folder_id, repository_identity, expected_object_id) {
            (Some(working_folder_id), Some(repository_identity), Some(expected_object_id)) => {
                let working_folder_id =
                    ProjectWorkingFolderId::new(working_folder_id).map_err(|_| corrupt_data())?;
                let workspace = workspaces::read_workspace(&pool, &working_folder_id).await?;
                let scope = read_active_working_folder_scope(&app).map_err(device_state_error)?;
                let authorized = authorize_workspace(
                    &workspace,
                    &scope,
                    WorkingFolderAuthorizationOperation::Restore,
                )?;
                if authorized.repository_identity.as_deref() != Some(&repository_identity) {
                    Err(ChatError::new(
                        ChatErrorCode::ConfigurationInvalid,
                        "Checkpoint cleanup repository identity changed",
                        true,
                    ))
                } else {
                    let root = authorized.canonical_path;
                    tauri::async_runtime::spawn_blocking(move || {
                        delete_exact_ref(&root, &reference, &expected_object_id)
                    })
                    .await
                    .map_err(|_| checkpoint_command_error())?
                }
            }
            _ => Err(corrupt_data()),
        };
        match result {
            Ok(()) => {
                sqlx::query(
                    "UPDATE chat_cleanup_queue
                     SET state = 'completed', last_error_code = NULL, updated_at = ? WHERE id = ?",
                )
                .bind(now.as_str())
                .bind(&cleanup_id)
                .execute(&pool)
                .await
                .map_err(persistence_error)?;
                completed += 1;
            }
            Err(error) => {
                sqlx::query(
                    "UPDATE chat_cleanup_queue
                     SET state = 'failed', last_error_code = ?, updated_at = ? WHERE id = ?",
                )
                .bind(format!("{:?}", error.code).to_lowercase())
                .bind(now.as_str())
                .bind(&cleanup_id)
                .execute(&pool)
                .await
                .map_err(persistence_error)?;
            }
        }
    }
    Ok(completed)
}

async fn checkpoint_pair(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: Option<&ChatTurnId>,
    scope: &str,
) -> ChatResult<Option<(ChatCheckpointId, ChatCheckpointId)>> {
    let row = if scope == "current_turn" {
        match turn_id {
            Some(turn_id) => sqlx::query(
                "SELECT pre_checkpoint_id, post_checkpoint_id
                     FROM chat_turns WHERE id = ? AND thread_id = ? AND invalidated_at IS NULL",
            )
            .bind(turn_id.as_str())
            .bind(thread_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?,
            None => sqlx::query(
                "SELECT pre_checkpoint_id, post_checkpoint_id
                     FROM chat_turns WHERE thread_id = ? AND invalidated_at IS NULL
                     ORDER BY ordinal DESC LIMIT 1",
            )
            .bind(thread_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?,
        }
    } else {
        sqlx::query(
            "SELECT
                (SELECT id FROM chat_checkpoints
                 WHERE thread_id = ? AND status = 'available' AND invalidated_at IS NULL
                 ORDER BY turn_count ASC LIMIT 1) AS pre_checkpoint_id,
                (SELECT id FROM chat_checkpoints
                 WHERE thread_id = ? AND status = 'available' AND invalidated_at IS NULL
                 ORDER BY turn_count DESC LIMIT 1) AS post_checkpoint_id",
        )
        .bind(thread_id.as_str())
        .bind(thread_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
    };
    row.map(|row| {
        let pre: Option<String> = row
            .try_get("pre_checkpoint_id")
            .map_err(persistence_error)?;
        let post: Option<String> = row
            .try_get("post_checkpoint_id")
            .map_err(persistence_error)?;
        match (pre, post) {
            (Some(pre), Some(post)) if pre != post => Ok(Some((
                ChatCheckpointId::new(pre).map_err(|_| corrupt_data())?,
                ChatCheckpointId::new(post).map_err(|_| corrupt_data())?,
            ))),
            _ => Ok(None),
        }
    })
    .transpose()
    .map(Option::flatten)
}

async fn provider_files(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: Option<&ChatTurnId>,
    scope: &str,
) -> ChatResult<Vec<ChangedFileSummary>> {
    let rows = if scope == "current_turn" {
        match turn_id {
            Some(turn_id) => sqlx::query(
                "SELECT changed_file_summary_data FROM chat_turns
                     WHERE id = ? AND thread_id = ? AND invalidated_at IS NULL",
            )
            .bind(turn_id.as_str())
            .bind(thread_id.as_str())
            .fetch_all(pool)
            .await
            .map_err(persistence_error)?,
            None => sqlx::query(
                "SELECT changed_file_summary_data FROM chat_turns
                     WHERE thread_id = ? AND invalidated_at IS NULL
                     ORDER BY ordinal DESC LIMIT 1",
            )
            .bind(thread_id.as_str())
            .fetch_all(pool)
            .await
            .map_err(persistence_error)?,
        }
    } else {
        sqlx::query(
            "SELECT changed_file_summary_data FROM chat_turns
             WHERE thread_id = ? AND invalidated_at IS NULL ORDER BY ordinal",
        )
        .bind(thread_id.as_str())
        .fetch_all(pool)
        .await
        .map_err(persistence_error)?
    };
    let mut merged = BTreeMap::new();
    for row in rows {
        let data: Option<String> = row
            .try_get("changed_file_summary_data")
            .map_err(persistence_error)?;
        let Some(data) = data else {
            continue;
        };
        for file in serde_json::from_str::<Vec<ChangedFileSummary>>(&data).map_err(json_error)? {
            merged.insert(file.relative_path.clone(), file);
        }
    }
    Ok(merged.into_values().collect())
}

fn merge_provider_and_git(
    provider: Vec<ChangedFileSummary>,
    git: Vec<ChatChangedFileRead>,
) -> Vec<ChatChangedFileRead> {
    let mut merged = git
        .into_iter()
        .map(|file| (file.relative_path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    for file in provider {
        merged
            .entry(file.relative_path.clone())
            .and_modify(|entry| entry.provider_reported = true)
            .or_insert(ChatChangedFileRead {
                relative_path: file.relative_path,
                previous_relative_path: file.previous_relative_path,
                status: normalize_status(&file.status),
                additions: file.additions,
                deletions: file.deletions,
                binary: file.binary,
                provider_reported: true,
                git_observed: false,
            });
    }
    merged.into_values().collect()
}

async fn authorize_thread(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<(ProjectWorkingFolderId, AuthorizedWorkingFolder)> {
    let working_folder_id: String = sqlx::query_scalar(
        "SELECT working_folder_id FROM chat_threads WHERE id = ? AND state != 'closed'",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true))?;
    let working_folder_id =
        ProjectWorkingFolderId::new(working_folder_id).map_err(|_| corrupt_data())?;
    let workspace = workspaces::read_workspace(pool, &working_folder_id).await?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    let authorized = authorize_workspace(
        &workspace,
        &scope,
        WorkingFolderAuthorizationOperation::Diff,
    )?;
    Ok((working_folder_id, authorized))
}

fn unavailable_diff(
    scope: String,
    provider_files: Vec<ChangedFileSummary>,
    reason: &str,
) -> ChatCheckpointDiffRead {
    let files = provider_files
        .into_iter()
        .map(|file| ChatChangedFileRead {
            relative_path: file.relative_path,
            previous_relative_path: file.previous_relative_path,
            status: normalize_status(&file.status),
            additions: file.additions,
            deletions: file.deletions,
            binary: file.binary,
            provider_reported: true,
            git_observed: false,
        })
        .collect();
    ChatCheckpointDiffRead {
        scope,
        available: false,
        unavailable_reason: Some(reason.to_string()),
        pre_checkpoint_id: None,
        post_checkpoint_id: None,
        files,
        additions: 0,
        deletions: 0,
        provider_mismatch: false,
    }
}

fn normalize_status(status: &str) -> String {
    match status {
        "added" | "modified" | "deleted" | "renamed" | "type_changed" => status,
        _ => "unknown",
    }
    .to_string()
}

fn validate_scope(scope: &str) -> ChatResult<()> {
    if matches!(scope, "current_turn" | "entire_thread") {
        Ok(())
    } else {
        Err(ChatError::validation(
            "scope",
            "Checkpoint diff scope is invalid",
        ))
    }
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

fn checkpoint_command_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Checkpoint diff worker stopped",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat checkpoint read failed",
        true,
    )
}

fn json_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat changed-file metadata is invalid",
        false,
    )
}

fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be read",
        true,
    )
}

fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat checkpoint data is invalid",
        false,
    )
}
