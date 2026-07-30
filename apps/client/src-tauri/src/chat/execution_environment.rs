//! Device-local Chat execution environments and safe Git worktree lifecycle.

use super::device_state::{read_active_device_scope, update_active_device_scope};
use super::git_service;
use super::models::{
    ChatError, ChatErrorCode, ChatResult, ChatThreadId, ProjectWorkingFolderId, UtcTimestamp,
};
use super::repository::workspaces;
use super::workspace::{
    authorize_workspace, AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::db_path;
use crate::projects::working_folders::read_active_working_folder_scope;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{Row, SqlitePool};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

const WORKTREE_DIRECTORY: &str = "chat-worktrees";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatExecutionEnvironmentKind {
    CurrentFolder,
    Worktree,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatExecutionEnvironmentRead {
    pub id: String,
    pub working_folder_id: ProjectWorkingFolderId,
    pub kind: ChatExecutionEnvironmentKind,
    pub display_name: String,
    pub lifecycle_state: String,
    pub branch_name: Option<String>,
    pub base_reference: Option<String>,
    pub remote_name: Option<String>,
    pub cleanup_state: Option<String>,
    pub local_path: Option<String>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatWorktreeRequest {
    pub environment_id: String,
    pub working_folder_id: ProjectWorkingFolderId,
    pub display_name: String,
    pub branch_name: String,
    pub base_reference: String,
    pub remote_name: Option<String>,
    pub fetch_remote: bool,
}

#[tauri::command]
pub async fn chat_list_execution_environments(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<Vec<ChatExecutionEnvironmentRead>> {
    let pool = chat_pool(&app, db_url).await?;
    let paths = read_active_device_scope(&app)
        .map_err(device_state_error)?
        .execution_environment_paths;
    let rows = sqlx::query(
        "SELECT environment.id, environment.working_folder_id, environment.kind,
                environment.display_name, environment.lifecycle_state, environment.created_at,
                environment.updated_at, worktree.branch_name, worktree.base_reference,
                worktree.remote_name, worktree.cleanup_state
         FROM chat_execution_environments environment
         LEFT JOIN chat_worktrees worktree
           ON worktree.execution_environment_id = environment.id
         WHERE environment.working_folder_id = ? AND environment.archived_at IS NULL
         ORDER BY environment.kind, environment.created_at, environment.id",
    )
    .bind(working_folder_id.as_str())
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter()
        .map(|row| parse_environment(row, &paths))
        .collect()
}

#[tauri::command]
pub async fn chat_create_worktree_environment(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatWorktreeRequest,
) -> ChatResult<ChatExecutionEnvironmentRead> {
    validate_id(&request.environment_id)?;
    validate_label(&request.display_name, "displayName", 240)?;
    validate_ref(&request.branch_name, "branchName")?;
    validate_ref(&request.base_reference, "baseReference")?;
    if let Some(remote) = request.remote_name.as_deref() {
        validate_ref(remote, "remoteName")?;
    }
    let pool = chat_pool(&app, db_url).await?;
    let root = authorized_workspace(&app, &pool, &request.working_folder_id).await?;
    if request.fetch_remote {
        git_service::fetch(&root.canonical_path, request.remote_name.as_deref()).await?;
    }
    let path = worktree_path(&app, &request.environment_id)?;
    if path.exists() {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Chat worktree directory already exists",
            true,
        ));
    }
    let path_text = path
        .to_str()
        .ok_or_else(|| ChatError::validation("environmentId", "Worktree path is unsupported"))?
        .to_string();
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_execution_environments
            (id, working_folder_id, kind, display_name, repository_identity,
             lifecycle_state, created_at, updated_at)
         VALUES (?, ?, 'worktree', ?, ?, 'creating', ?, ?)",
    )
    .bind(&request.environment_id)
    .bind(request.working_folder_id.as_str())
    .bind(request.display_name.trim())
    .bind(&root.repository_identity)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_worktrees
            (execution_environment_id, branch_name, base_reference, remote_name,
             cleanup_policy, cleanup_state, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'ask', 'retained', ?, ?)",
    )
    .bind(&request.environment_id)
    .bind(&request.branch_name)
    .bind(&request.base_reference)
    .bind(&request.remote_name)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)?;
    update_active_device_scope(&app, |scope| {
        scope
            .execution_environment_paths
            .insert(request.environment_id.clone(), path_text.clone());
        Ok(())
    })
    .map_err(device_state_error)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    if let Err(error) = git_service::add_worktree(
        &root.canonical_path,
        &path_text,
        &request.branch_name,
        &request.base_reference,
    )
    .await
    {
        mark_environment_failure(&pool, &request.environment_id, &error.message).await?;
        return Err(error);
    }
    let head = git_service::head_object_id(&path).await?;
    sqlx::query(
        "UPDATE chat_execution_environments SET lifecycle_state = 'available', updated_at = ?
         WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(&request.environment_id)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_worktrees SET head_object_id = ?, updated_at = ?
         WHERE execution_environment_id = ?",
    )
    .bind(head)
    .bind(now.as_str())
    .bind(&request.environment_id)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    read_environment(&app, &pool, &request.environment_id).await
}

#[tauri::command]
pub async fn chat_select_thread_execution_environment(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    environment_id: String,
) -> ChatResult<()> {
    validate_id(&environment_id)?;
    let pool = chat_pool(&app, db_url).await?;
    let result = sqlx::query(
        "UPDATE chat_threads
         SET execution_environment_id = ?, updated_at = ?, revision = revision + 1
         WHERE id = ? AND state != 'closed'
           AND NOT EXISTS (SELECT 1 FROM chat_turns WHERE thread_id = chat_threads.id)
           AND EXISTS (
             SELECT 1 FROM chat_execution_environments environment
             WHERE environment.id = ?
               AND environment.working_folder_id = chat_threads.working_folder_id
               AND environment.lifecycle_state = 'available'
           )",
    )
    .bind(&environment_id)
    .bind(now_timestamp()?.as_str())
    .bind(thread_id.as_str())
    .bind(&environment_id)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if result.rows_affected() == 0 {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Execution environment can only change before the first turn",
            true,
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn chat_read_thread_execution_environment(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
) -> ChatResult<Option<String>> {
    let pool = chat_pool(&app, db_url).await?;
    sqlx::query_scalar(
        "SELECT execution_environment_id FROM chat_threads WHERE id = ? AND state != 'closed'",
    )
    .bind(thread_id.as_str())
    .fetch_optional(&pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Active Chat thread was not found",
            true,
        )
    })
}

#[tauri::command]
pub async fn chat_remove_worktree_environment(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    environment_id: String,
    confirmed: bool,
) -> ChatResult<()> {
    if !confirmed {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Worktree removal requires explicit confirmation",
            true,
        ));
    }
    let pool = chat_pool(&app, db_url).await?;
    let root = authorized_workspace(&app, &pool, &working_folder_id).await?;
    let environment = read_environment(&app, &pool, &environment_id).await?;
    if environment.working_folder_id != working_folder_id
        || environment.kind != ChatExecutionEnvironmentKind::Worktree
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Worktree environment belongs to another workspace",
            true,
        ));
    }
    let path_text = environment.local_path.ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Worktree path is unavailable on this device",
            true,
        )
    })?;
    let status = git_service::status(Path::new(&path_text)).await?;
    if !status.files.is_empty() {
        sqlx::query(
            "UPDATE chat_worktrees SET cleanup_state = 'dirty', updated_at = ?
             WHERE execution_environment_id = ?",
        )
        .bind(now_timestamp()?.as_str())
        .bind(&environment_id)
        .execute(&pool)
        .await
        .map_err(persistence_error)?;
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Dirty worktrees are never removed automatically",
            true,
        ));
    }
    git_service::remove_worktree(&root.canonical_path, &path_text).await?;
    let now = now_timestamp()?;
    sqlx::query(
        "UPDATE chat_execution_environments
         SET lifecycle_state = 'removed', archived_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(&environment_id)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_worktrees
         SET cleanup_state = 'cleaned', removed_at = ?, updated_at = ?
         WHERE execution_environment_id = ?",
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(&environment_id)
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    update_active_device_scope(&app, |scope| {
        scope.execution_environment_paths.remove(&environment_id);
        Ok(())
    })
    .map_err(device_state_error)
}

pub async fn resolve_environment_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    authorized: AuthorizedWorkingFolder,
    environment_id: Option<&str>,
) -> ChatResult<AuthorizedWorkingFolder> {
    let Some(environment_id) = environment_id else {
        return Ok(authorized);
    };
    let environment = read_environment(app, pool, environment_id).await?;
    if environment.working_folder_id != authorized.working_folder_id
        || environment.lifecycle_state != "available"
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Execution environment is unavailable for this workspace",
            true,
        ));
    }
    if environment.kind == ChatExecutionEnvironmentKind::CurrentFolder {
        return Ok(authorized);
    }
    let path = environment.local_path.ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Worktree path is unavailable on this device",
            true,
        )
    })?;
    let canonical_path = PathBuf::from(path)
        .canonicalize()
        .map_err(|_| ChatError::new(ChatErrorCode::NotFound, "Chat worktree is missing", true))?;
    if !canonical_path.is_dir() {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "Chat worktree is missing",
            true,
        ));
    }
    Ok(AuthorizedWorkingFolder {
        canonical_path,
        ..authorized
    })
}

async fn authorized_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
) -> ChatResult<AuthorizedWorkingFolder> {
    let workspace = workspaces::read_workspace(pool, working_folder_id).await?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    authorize_workspace(&workspace, &scope, WorkingFolderAuthorizationOperation::Git)
}

async fn read_environment(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    environment_id: &str,
) -> ChatResult<ChatExecutionEnvironmentRead> {
    let paths = read_active_device_scope(app)
        .map_err(device_state_error)?
        .execution_environment_paths;
    let row = sqlx::query(
        "SELECT environment.id, environment.working_folder_id, environment.kind,
                environment.display_name, environment.lifecycle_state, environment.created_at,
                environment.updated_at, worktree.branch_name, worktree.base_reference,
                worktree.remote_name, worktree.cleanup_state
         FROM chat_execution_environments environment
         LEFT JOIN chat_worktrees worktree
           ON worktree.execution_environment_id = environment.id
         WHERE environment.id = ?",
    )
    .bind(environment_id)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Chat execution environment was not found",
            true,
        )
    })?;
    parse_environment(row, &paths)
}

fn parse_environment(
    row: sqlx::sqlite::SqliteRow,
    paths: &std::collections::BTreeMap<String, String>,
) -> ChatResult<ChatExecutionEnvironmentRead> {
    let id: String = row.try_get("id").map_err(persistence_error)?;
    let kind = match row
        .try_get::<String, _>("kind")
        .map_err(persistence_error)?
        .as_str()
    {
        "current_folder" => ChatExecutionEnvironmentKind::CurrentFolder,
        "worktree" => ChatExecutionEnvironmentKind::Worktree,
        _ => return Err(persistence_error("invalid environment kind")),
    };
    Ok(ChatExecutionEnvironmentRead {
        local_path: paths.get(&id).cloned(),
        id,
        working_folder_id: ProjectWorkingFolderId::new(
            row.try_get::<String, _>("working_folder_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| persistence_error("invalid working folder ID"))?,
        kind,
        display_name: row.try_get("display_name").map_err(persistence_error)?,
        lifecycle_state: row.try_get("lifecycle_state").map_err(persistence_error)?,
        branch_name: row.try_get("branch_name").map_err(persistence_error)?,
        base_reference: row.try_get("base_reference").map_err(persistence_error)?,
        remote_name: row.try_get("remote_name").map_err(persistence_error)?,
        cleanup_state: row.try_get("cleanup_state").map_err(persistence_error)?,
        created_at: timestamp(&row, "created_at")?,
        updated_at: timestamp(&row, "updated_at")?,
    })
}

async fn mark_environment_failure(
    pool: &SqlitePool,
    environment_id: &str,
    detail: &str,
) -> ChatResult<()> {
    let now = now_timestamp()?;
    sqlx::query(
        "UPDATE chat_execution_environments SET lifecycle_state = 'cleanup_failed', updated_at = ?
         WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(environment_id)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_worktrees
         SET cleanup_state = 'failed', cleanup_error_code = 'create_failed',
             cleanup_error_detail = ?, updated_at = ?
         WHERE execution_environment_id = ?",
    )
    .bind(detail.chars().take(2_000).collect::<String>())
    .bind(now.as_str())
    .bind(environment_id)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

fn worktree_path(app: &tauri::AppHandle, environment_id: &str) -> ChatResult<PathBuf> {
    let digest = format!("{:x}", sha2::Sha256::digest(environment_id.as_bytes()));
    app.path()
        .app_local_data_dir()
        .map(|root| root.join(WORKTREE_DIRECTORY).join(&digest[..24]))
        .map_err(device_state_error)
}

fn validate_id(value: &str) -> ChatResult<()> {
    validate_label(value, "environmentId", 2_048)
}

fn validate_label(value: &str, field: &str, maximum: usize) -> ChatResult<()> {
    let value = value.trim();
    if value.is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            field,
            "Execution environment value is invalid",
        ));
    }
    Ok(())
}

fn validate_ref(value: &str, field: &str) -> ChatResult<()> {
    validate_label(value, field, 1_024)?;
    if value.starts_with('-') || value.chars().any(char::is_whitespace) {
        return Err(ChatError::validation(field, "Git reference is invalid"));
    }
    Ok(())
}

fn timestamp(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<UtcTimestamp> {
    UtcTimestamp::new(
        row.try_get::<String, _>(column)
            .map_err(persistence_error)?,
    )
    .map_err(|_| persistence_error("invalid timestamp"))
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}

async fn chat_pool(app: &tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app.clone(), db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn io_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "Chat worktree directory could not be created",
        true,
    )
}

fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat execution environment device state failed",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat execution environment persistence failed",
        true,
    )
}
