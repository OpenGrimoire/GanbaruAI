//! Projection-backed commands for Chat thread navigation and lifecycle actions.

use super::models::{
    ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatThreadShellRead, ChatWorkspaceId,
    UtcTimestamp,
};
use super::repository::{lifecycle, reads};
use crate::db_path;
use chrono::{SecondsFormat, Utc};
use sqlx::SqlitePool;
use std::time::{Duration, SystemTime};

const PERMANENT_DELETE_CLEANUP_GRACE: Duration = Duration::from_secs(24 * 60 * 60);

#[tauri::command]
pub async fn chat_list_project_shells(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<Vec<reads::ChatProjectShellRead>> {
    reads::read_project_shells(&chat_pool(app, db_url).await?).await
}

#[tauri::command]
pub async fn chat_list_threads(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: Option<ChatWorkspaceId>,
    archived: bool,
) -> ChatResult<Vec<ChatThreadShellRead>> {
    reads::read_thread_shells(
        &chat_pool(app, db_url).await?,
        workspace_id.as_ref(),
        archived,
    )
    .await
}

#[tauri::command]
pub async fn chat_search_thread_titles(
    app: tauri::AppHandle,
    db_url: String,
    query: String,
    archived: Option<bool>,
    limit: u32,
) -> ChatResult<Vec<ChatThreadShellRead>> {
    reads::search_thread_titles(&chat_pool(app, db_url).await?, &query, archived, limit).await
}

#[tauri::command]
pub async fn chat_rename_thread(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    title: String,
    expected_revision: u64,
) -> ChatResult<ChatThreadShellRead> {
    let pool = chat_pool(app, db_url).await?;
    lifecycle::rename_thread(
        &pool,
        &thread_id,
        &title,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    reads::read_thread_shell(&pool, &thread_id).await
}

#[tauri::command]
pub async fn chat_set_thread_read(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    read: bool,
    expected_revision: u64,
) -> ChatResult<ChatThreadShellRead> {
    let pool = chat_pool(app, db_url).await?;
    lifecycle::set_thread_read(
        &pool,
        &thread_id,
        read,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    reads::read_thread_shell(&pool, &thread_id).await
}

#[tauri::command]
pub async fn chat_archive_thread(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    expected_revision: u64,
) -> ChatResult<ChatThreadShellRead> {
    set_thread_archived(app, db_url, thread_id, expected_revision, true).await
}

#[tauri::command]
pub async fn chat_restore_thread(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    expected_revision: u64,
) -> ChatResult<ChatThreadShellRead> {
    set_thread_archived(app, db_url, thread_id, expected_revision, false).await
}

#[tauri::command]
pub async fn chat_delete_thread_permanently(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    expected_revision: u64,
    confirmed_title: String,
) -> ChatResult<()> {
    let pool = chat_pool(app, db_url).await?;
    let shell = reads::read_thread_shell(&pool, &thread_id).await?;
    if shell.title != confirmed_title {
        return Err(ChatError::validation(
            "confirmedTitle",
            "The confirmation title does not match the Chat thread",
        ));
    }
    let now = SystemTime::now();
    let cleanup = now
        .checked_add(PERMANENT_DELETE_CLEANUP_GRACE)
        .ok_or_else(|| ChatError::new(ChatErrorCode::Internal, "create cleanup deadline", false))?;
    lifecycle::permanently_delete_thread(
        &pool,
        &thread_id,
        expected_revision,
        &timestamp(cleanup)?,
        &timestamp(now)?,
    )
    .await
}

async fn set_thread_archived(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    expected_revision: u64,
    archived: bool,
) -> ChatResult<ChatThreadShellRead> {
    let pool = chat_pool(app, db_url).await?;
    lifecycle::set_thread_archived(
        &pool,
        &thread_id,
        archived,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    reads::read_thread_shell(&pool, &thread_id).await
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    timestamp(SystemTime::now())
}

fn timestamp(value: SystemTime) -> ChatResult<UtcTimestamp> {
    let value: chrono::DateTime<Utc> = value.into();
    UtcTimestamp::new(value.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}
