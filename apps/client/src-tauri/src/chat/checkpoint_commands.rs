//! Durable checkpoint-ref cleanup command facade.

use super::checkpoints::run_checkpoint_cleanup;
use super::models::{ChatError, ChatErrorCode, ChatResult, UtcTimestamp};
use crate::db_path;
use chrono::{SecondsFormat, Utc};
use sqlx::SqlitePool;

#[tauri::command]
pub async fn chat_run_checkpoint_cleanup(app: tauri::AppHandle, db_url: String) -> ChatResult<u64> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let now = now_timestamp()?;
    run_checkpoint_cleanup(&app, &pool, &now).await
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
