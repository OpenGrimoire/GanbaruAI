//! Authorized commands for runtime terminals and explicit terminal context.

use super::device_state::read_active_device_scope;
use super::models::{
    ChatAttachmentId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatWorkspaceId,
    UtcTimestamp,
};
use super::repository::{attachments, workspaces};
use super::terminal::{
    ChatTerminalCloseResult, ChatTerminalCreateInput, ChatTerminalRead, ChatTerminalRegistry,
    ChatTerminalSnapshotRead,
};
use super::workspace::{authorize_workspace, AuthorizedWorkspace, WorkspaceAuthorizationOperation};
use crate::{db_path, vault};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tauri::Manager;

const MAX_CONTEXT_BYTES: usize = 128 * 1024;
const MAX_CONTEXT_PREVIEW_BYTES: usize = 4 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatTerminalRequest {
    pub terminal_id: String,
    pub thread_id: ChatThreadId,
    pub workspace_id: ChatWorkspaceId,
    pub name: String,
    pub columns: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTerminalInputRequest {
    pub terminal_id: String,
    pub thread_id: ChatThreadId,
    pub workspace_id: ChatWorkspaceId,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTerminalResizeRequest {
    pub terminal_id: String,
    pub thread_id: ChatThreadId,
    pub workspace_id: ChatWorkspaceId,
    pub columns: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportChatTerminalContextRequest {
    pub terminal_id: String,
    pub thread_id: ChatThreadId,
    pub workspace_id: ChatWorkspaceId,
    pub attachment_id: ChatAttachmentId,
    pub source_kind: String,
    pub text: String,
    pub start_output_sequence: Option<u64>,
    pub end_output_sequence: Option<u64>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatTerminalContextRead {
    pub attachment_id: ChatAttachmentId,
    pub terminal_id: String,
    pub terminal_name: String,
    pub captured_at: UtcTimestamp,
    pub byte_size: u64,
    pub line_count: u64,
    pub preview: String,
    pub truncated: bool,
}

#[tauri::command]
pub async fn chat_list_terminals(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<Vec<ChatTerminalRead>> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &thread_id, &workspace_id).await?;
    app.state::<ChatTerminalRegistry>()
        .list(&thread_id, &workspace_id)
}

#[tauri::command]
pub async fn chat_terminal_create(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatTerminalRequest,
) -> ChatResult<ChatTerminalSnapshotRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized =
        authorize_thread_workspace(&app, &pool, &request.thread_id, &request.workspace_id).await?;
    app.state::<ChatTerminalRegistry>().create(
        app.clone(),
        ChatTerminalCreateInput {
            terminal_id: request.terminal_id,
            thread_id: request.thread_id,
            workspace_id: request.workspace_id,
            workspace_path: authorized.canonical_path,
            name: request.name,
            columns: request.columns,
            rows: request.rows,
        },
    )
}

#[tauri::command]
pub async fn chat_terminal_snapshot(
    app: tauri::AppHandle,
    db_url: String,
    terminal_id: String,
    thread_id: ChatThreadId,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatTerminalSnapshotRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &thread_id, &workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(&terminal_id, &thread_id, &workspace_id)?;
    registry.snapshot(&terminal_id)
}

#[tauri::command]
pub async fn chat_terminal_input(
    app: tauri::AppHandle,
    db_url: String,
    request: ChatTerminalInputRequest,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &request.thread_id, &request.workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(
        &request.terminal_id,
        &request.thread_id,
        &request.workspace_id,
    )?;
    registry.input(&request.terminal_id, &request.text)
}

#[tauri::command]
pub async fn chat_terminal_resize(
    app: tauri::AppHandle,
    db_url: String,
    request: ChatTerminalResizeRequest,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &request.thread_id, &request.workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(
        &request.terminal_id,
        &request.thread_id,
        &request.workspace_id,
    )?;
    registry.resize(&request.terminal_id, request.columns, request.rows)
}

#[tauri::command]
pub async fn chat_terminal_rename(
    app: tauri::AppHandle,
    db_url: String,
    terminal_id: String,
    thread_id: ChatThreadId,
    workspace_id: ChatWorkspaceId,
    name: String,
) -> ChatResult<ChatTerminalRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &thread_id, &workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(&terminal_id, &thread_id, &workspace_id)?;
    registry.rename(&terminal_id, &name)
}

#[tauri::command]
pub async fn chat_terminal_restart(
    app: tauri::AppHandle,
    db_url: String,
    terminal_id: String,
    thread_id: ChatThreadId,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatTerminalSnapshotRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = authorize_thread_workspace(&app, &pool, &thread_id, &workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(&terminal_id, &thread_id, &workspace_id)?;
    registry.restart(app.clone(), &terminal_id, &authorized.canonical_path)
}

#[tauri::command]
pub async fn chat_terminal_close(
    app: tauri::AppHandle,
    db_url: String,
    terminal_id: String,
    thread_id: ChatThreadId,
    workspace_id: ChatWorkspaceId,
    confirmed: bool,
) -> ChatResult<ChatTerminalCloseResult> {
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &thread_id, &workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(&terminal_id, &thread_id, &workspace_id)?;
    registry.close(&terminal_id, confirmed)
}

#[tauri::command]
pub async fn chat_terminal_import_context(
    app: tauri::AppHandle,
    db_url: String,
    request: ImportChatTerminalContextRequest,
) -> ChatResult<ChatTerminalContextRead> {
    validate_context(&request)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    authorize_thread_workspace(&app, &pool, &request.thread_id, &request.workspace_id).await?;
    let registry = app.state::<ChatTerminalRegistry>();
    registry.require_scope(
        &request.terminal_id,
        &request.thread_id,
        &request.workspace_id,
    )?;
    let terminal = registry.snapshot(&request.terminal_id)?.terminal;
    let now = now_timestamp()?;
    let line_count = if request.text.is_empty() {
        0
    } else {
        request.text.lines().count() as u64
    };
    let display_name = format!("{} terminal context.txt", terminal.name);
    let attachment = attachments::import_attachment_bytes(
        &pool,
        &vault::active_vault_path(&app).map_err(vault_error)?,
        attachments::AttachmentBytesImport {
            workspace_id: &request.workspace_id,
            attachment_id: request.attachment_id.clone(),
            display_name,
            bytes: request.text.as_bytes(),
            requested_kind: attachments::ChatAttachmentKind::TextSnippet,
            now: &now,
        },
    )
    .await?;
    sqlx::query(
        "INSERT INTO chat_terminal_attachment_contexts
            (attachment_id, terminal_runtime_id, terminal_name_snapshot, source_kind,
             start_output_sequence, end_output_sequence, line_count, truncated, captured_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(request.attachment_id.as_str())
    .bind(&request.terminal_id)
    .bind(&terminal.name)
    .bind(&request.source_kind)
    .bind(request.start_output_sequence.map(i64_value).transpose()?)
    .bind(request.end_output_sequence.map(i64_value).transpose()?)
    .bind(i64_value(line_count)?)
    .bind(request.truncated)
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    Ok(ChatTerminalContextRead {
        attachment_id: attachment.id,
        terminal_id: request.terminal_id,
        terminal_name: terminal.name,
        captured_at: now,
        byte_size: attachment.byte_size,
        line_count,
        preview: bounded_preview(&request.text),
        truncated: request.truncated,
    })
}

async fn authorize_thread_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    workspace_id: &ChatWorkspaceId,
) -> ChatResult<AuthorizedWorkspace> {
    let row =
        sqlx::query("SELECT workspace_id FROM chat_threads WHERE id = ? AND state != 'closed'")
            .bind(thread_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?
            .ok_or_else(|| {
                ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true)
            })?;
    let stored_workspace: String = row.try_get("workspace_id").map_err(persistence_error)?;
    if stored_workspace != workspace_id.as_str() {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Chat terminal workspace does not match the thread",
            false,
        ));
    }
    let workspace = workspaces::read_workspace(pool, workspace_id).await?;
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    authorize_workspace(
        &workspace,
        &scope,
        WorkspaceAuthorizationOperation::TerminalStart,
    )
}

fn validate_context(request: &ImportChatTerminalContextRequest) -> ChatResult<()> {
    if !matches!(
        request.source_kind.as_str(),
        "selection" | "last_command_output"
    ) || request.text.is_empty()
        || request.text.len() > MAX_CONTEXT_BYTES
        || request.text.contains('\0')
        || request
            .start_output_sequence
            .zip(request.end_output_sequence)
            .is_some_and(|(start, end)| start > end)
    {
        return Err(ChatError::validation(
            "terminalContext",
            "Terminal context selection is invalid",
        ));
    }
    Ok(())
}

fn bounded_preview(text: &str) -> String {
    if text.len() <= MAX_CONTEXT_PREVIEW_BYTES {
        return text.to_string();
    }
    let mut boundary = MAX_CONTEXT_PREVIEW_BYTES;
    while !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    text[..boundary].to_string()
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

fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be read",
        true,
    )
}

fn vault_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Active Ganbaru folder is unavailable",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat terminal context persistence failed",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(text: &str) -> ImportChatTerminalContextRequest {
        ImportChatTerminalContextRequest {
            terminal_id: "terminal:test".to_string(),
            thread_id: ChatThreadId::new("thread:test").expect("thread ID should be valid"),
            workspace_id: ChatWorkspaceId::new("workspace:test")
                .expect("workspace ID should be valid"),
            attachment_id: ChatAttachmentId::new("attachment:test")
                .expect("attachment ID should be valid"),
            source_kind: "selection".to_string(),
            text: text.to_string(),
            start_output_sequence: Some(1),
            end_output_sequence: Some(2),
            truncated: false,
        }
    }

    #[test]
    fn terminal_context_requires_bounded_ordered_text_and_known_sources() {
        assert!(validate_context(&request("context")).is_ok());
        assert!(validate_context(&request("")).is_err());
        let mut reversed = request("context");
        reversed.start_output_sequence = Some(3);
        assert!(validate_context(&reversed).is_err());
        let mut unknown = request("context");
        unknown.source_kind = "screen".to_string();
        assert!(validate_context(&unknown).is_err());
    }

    #[test]
    fn context_preview_stops_at_a_utf8_boundary() {
        let text = "é".repeat(MAX_CONTEXT_PREVIEW_BYTES);
        let preview = bounded_preview(&text);
        assert!(preview.is_char_boundary(preview.len()));
        assert!(preview.len() <= MAX_CONTEXT_PREVIEW_BYTES);
    }
}
