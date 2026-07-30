//! Durable inline review comments and immutable composer context.

use super::models::{
    ChatAttachmentId, ChatError, ChatErrorCode, ChatResult, ChatThreadId, ProjectWorkingFolderId,
    UtcTimestamp,
};
use super::repository::attachments;
use crate::{db_path, vault};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

const MAX_PATH_BYTES: usize = 4_096;
const MAX_COMMENT_BYTES: usize = 65_536;
const MAX_SELECTED_TEXT_BYTES: usize = 1_048_576;
const MAX_REVIEW_CONTEXT_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatReviewCommentRequest {
    pub id: String,
    pub thread_id: ChatThreadId,
    pub relative_path: String,
    pub content_revision: String,
    pub start_line: u64,
    pub start_column: u64,
    pub end_line: u64,
    pub end_column: u64,
    pub selected_text: String,
    pub comment_text: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachChatReviewCommentRequest {
    pub thread_id: ChatThreadId,
    pub comment_id: String,
    pub attachment_id: ChatAttachmentId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatReviewCommentState {
    Open,
    Resolved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReviewCommentRead {
    pub id: String,
    pub thread_id: ChatThreadId,
    pub relative_path: String,
    pub content_revision: String,
    pub start_line: u64,
    pub start_column: u64,
    pub end_line: u64,
    pub end_column: u64,
    pub selected_text: String,
    pub comment_text: String,
    pub state: ChatReviewCommentState,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub resolved_at: Option<UtcTimestamp>,
}

#[tauri::command]
pub async fn chat_list_review_comments(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    include_resolved: bool,
) -> ChatResult<Vec<ChatReviewCommentRead>> {
    let pool = chat_pool(app, db_url).await?;
    require_thread_workspace(&pool, &thread_id).await?;
    let rows = sqlx::query(
        "SELECT id, thread_id, relative_path, content_revision, start_line, start_column,
                end_line, end_column, selected_text, comment_text, state, created_at,
                updated_at, resolved_at
         FROM chat_review_comments
         WHERE thread_id = ? AND (? OR state = 'open')
         ORDER BY relative_path, start_line, start_column, created_at, id",
    )
    .bind(thread_id.as_str())
    .bind(include_resolved)
    .fetch_all(&pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter().map(parse_comment).collect()
}

#[tauri::command]
pub async fn chat_create_review_comment(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatReviewCommentRequest,
) -> ChatResult<ChatReviewCommentRead> {
    validate_create_request(&request)?;
    let pool = chat_pool(app, db_url).await?;
    require_thread_workspace(&pool, &request.thread_id).await?;
    let now = now_timestamp()?;
    sqlx::query(
        "INSERT INTO chat_review_comments
            (id, thread_id, relative_path, content_revision, start_line, start_column,
             end_line, end_column, selected_text, comment_text, state, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?, ?)",
    )
    .bind(&request.id)
    .bind(request.thread_id.as_str())
    .bind(&request.relative_path)
    .bind(&request.content_revision)
    .bind(i64_value(request.start_line, "startLine")?)
    .bind(i64_value(request.start_column, "startColumn")?)
    .bind(i64_value(request.end_line, "endLine")?)
    .bind(i64_value(request.end_column, "endColumn")?)
    .bind(&request.selected_text)
    .bind(request.comment_text.trim())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    read_comment(&pool, &request.thread_id, &request.id).await
}

#[tauri::command]
pub async fn chat_set_review_comment_resolved(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    comment_id: String,
    resolved: bool,
) -> ChatResult<ChatReviewCommentRead> {
    validate_id(&comment_id)?;
    let pool = chat_pool(app, db_url).await?;
    require_thread_workspace(&pool, &thread_id).await?;
    let now = now_timestamp()?;
    let result = sqlx::query(
        "UPDATE chat_review_comments
         SET state = CASE WHEN ? THEN 'resolved' ELSE 'open' END,
             resolved_at = CASE WHEN ? THEN ? ELSE NULL END,
             updated_at = ?
         WHERE id = ? AND thread_id = ?",
    )
    .bind(resolved)
    .bind(resolved)
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(&comment_id)
    .bind(thread_id.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    if result.rows_affected() == 0 {
        return Err(ChatError::new(
            ChatErrorCode::NotFound,
            "Review comment was not found",
            true,
        ));
    }
    read_comment(&pool, &thread_id, &comment_id).await
}

#[tauri::command]
pub async fn chat_attach_review_comment(
    app: tauri::AppHandle,
    db_url: String,
    request: AttachChatReviewCommentRequest,
) -> ChatResult<attachments::ChatAttachmentRead> {
    validate_id(&request.comment_id)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let working_folder_id = require_thread_workspace(&pool, &request.thread_id).await?;
    let comment = read_comment(&pool, &request.thread_id, &request.comment_id).await?;
    let context = bounded_review_context(&comment);
    let now = now_timestamp()?;
    attachments::import_attachment_bytes(
        &pool,
        &vault::active_vault_path(&app).map_err(vault_error)?,
        attachments::AttachmentBytesImport {
            working_folder_id: &working_folder_id,
            attachment_id: request.attachment_id,
            display_name: format!("{} review comment.txt", comment.relative_path),
            bytes: context.as_bytes(),
            requested_kind: attachments::ChatAttachmentKind::TextSnippet,
            now: &now,
        },
    )
    .await
}

async fn require_thread_workspace(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<ProjectWorkingFolderId> {
    let row = sqlx::query(
        "SELECT working_folder_id FROM chat_threads WHERE id = ? AND state != 'closed'",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Active Chat thread was not found",
            true,
        )
    })?;
    ProjectWorkingFolderId::new(
        row.try_get::<String, _>("working_folder_id")
            .map_err(persistence_error)?,
    )
    .map_err(|_| persistence_error("invalid working folder ID"))
}

async fn read_comment(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    comment_id: &str,
) -> ChatResult<ChatReviewCommentRead> {
    let row = sqlx::query(
        "SELECT id, thread_id, relative_path, content_revision, start_line, start_column,
                end_line, end_column, selected_text, comment_text, state, created_at,
                updated_at, resolved_at
         FROM chat_review_comments WHERE id = ? AND thread_id = ?",
    )
    .bind(comment_id)
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Review comment was not found",
            true,
        )
    })?;
    parse_comment(row)
}

fn parse_comment(row: sqlx::sqlite::SqliteRow) -> ChatResult<ChatReviewCommentRead> {
    let state = match row
        .try_get::<String, _>("state")
        .map_err(persistence_error)?
        .as_str()
    {
        "open" => ChatReviewCommentState::Open,
        "resolved" => ChatReviewCommentState::Resolved,
        _ => return Err(persistence_error("invalid review comment state")),
    };
    Ok(ChatReviewCommentRead {
        id: row.try_get("id").map_err(persistence_error)?,
        thread_id: ChatThreadId::new(
            row.try_get::<String, _>("thread_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| persistence_error("invalid thread ID"))?,
        relative_path: row.try_get("relative_path").map_err(persistence_error)?,
        content_revision: row.try_get("content_revision").map_err(persistence_error)?,
        start_line: u64_value(row.try_get("start_line").map_err(persistence_error)?)?,
        start_column: u64_value(row.try_get("start_column").map_err(persistence_error)?)?,
        end_line: u64_value(row.try_get("end_line").map_err(persistence_error)?)?,
        end_column: u64_value(row.try_get("end_column").map_err(persistence_error)?)?,
        selected_text: row.try_get("selected_text").map_err(persistence_error)?,
        comment_text: row.try_get("comment_text").map_err(persistence_error)?,
        state,
        created_at: timestamp(&row, "created_at")?,
        updated_at: timestamp(&row, "updated_at")?,
        resolved_at: row
            .try_get::<Option<String>, _>("resolved_at")
            .map_err(persistence_error)?
            .map(UtcTimestamp::new)
            .transpose()
            .map_err(|_| persistence_error("invalid resolved timestamp"))?,
    })
}

fn validate_create_request(request: &CreateChatReviewCommentRequest) -> ChatResult<()> {
    validate_id(&request.id)?;
    let path = request.relative_path.as_str();
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.starts_with("../")
        || path.contains("/../")
        || path.contains('\\')
        || path.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "relativePath",
            "Review path is invalid",
        ));
    }
    if request.content_revision.len() < 16
        || request.content_revision.len() > 128
        || request.content_revision.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "contentRevision",
            "Review content revision is invalid",
        ));
    }
    if request.start_line == 0
        || request.start_column == 0
        || request.end_line < request.start_line
        || request.end_column == 0
        || (request.end_line == request.start_line && request.end_column < request.start_column)
    {
        return Err(ChatError::validation("range", "Review range is invalid"));
    }
    if request.selected_text.len() > MAX_SELECTED_TEXT_BYTES || request.selected_text.contains('\0')
    {
        return Err(ChatError::validation(
            "selectedText",
            "Review selection is invalid",
        ));
    }
    let comment = request.comment_text.trim();
    if comment.is_empty() || comment.len() > MAX_COMMENT_BYTES || comment.contains('\0') {
        return Err(ChatError::validation(
            "commentText",
            "Review comment is invalid",
        ));
    }
    Ok(())
}

fn validate_id(value: &str) -> ChatResult<()> {
    if value.is_empty() || value.len() > 1_024 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "commentId",
            "Review comment ID is invalid",
        ));
    }
    Ok(())
}

fn bounded_review_context(comment: &ChatReviewCommentRead) -> String {
    let heading = format!(
        "Review comment\nPath: {}\nRange: {}:{} to {}:{}\nContent revision: {}\nComment: {}\n\nSelected text:\n",
        comment.relative_path,
        comment.start_line,
        comment.start_column,
        comment.end_line,
        comment.end_column,
        comment.content_revision,
        comment.comment_text,
    );
    if heading.len() >= MAX_REVIEW_CONTEXT_BYTES {
        return heading.chars().take(MAX_REVIEW_CONTEXT_BYTES).collect();
    }
    let maximum = MAX_REVIEW_CONTEXT_BYTES - heading.len();
    let mut boundary = comment.selected_text.len().min(maximum);
    while !comment.selected_text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{heading}{}", &comment.selected_text[..boundary])
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

fn i64_value(value: u64, field: &str) -> ChatResult<i64> {
    i64::try_from(value).map_err(|_| ChatError::validation(field, "Review range is too large"))
}

fn u64_value(value: i64) -> ChatResult<u64> {
    u64::try_from(value).map_err(|_| persistence_error("invalid review range"))
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
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
        "Chat review comment persistence failed",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_context_preserves_location_revision_and_comment() {
        let timestamp = UtcTimestamp::new("2026-07-29T12:00:00.000Z").expect("timestamp");
        let comment = ChatReviewCommentRead {
            id: "review:1".to_string(),
            thread_id: ChatThreadId::new("thread:1").expect("thread ID"),
            relative_path: "src/lib.rs".to_string(),
            content_revision: "0123456789abcdef".to_string(),
            start_line: 4,
            start_column: 2,
            end_line: 5,
            end_column: 8,
            selected_text: "let value = 1;".to_string(),
            comment_text: "Handle this error explicitly.".to_string(),
            state: ChatReviewCommentState::Open,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            resolved_at: None,
        };

        let context = bounded_review_context(&comment);
        assert!(context.contains("src/lib.rs"));
        assert!(context.contains("4:2 to 5:8"));
        assert!(context.contains("0123456789abcdef"));
        assert!(context.contains("Handle this error explicitly."));
        assert!(context.ends_with("let value = 1;"));
    }
}
