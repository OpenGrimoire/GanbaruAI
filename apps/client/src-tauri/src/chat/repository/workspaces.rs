use crate::chat::models::{
    ChatError, ChatErrorCode, ChatResult, ChatWorkspaceId, RepositoryKind, UtcTimestamp,
};
use crate::chat::workspace::{CreateChatWorkspaceRequest, LogicalChatWorkspace};
use sqlx::{Row, SqlitePool};

pub async fn list_workspaces(pool: &SqlitePool) -> ChatResult<Vec<LogicalChatWorkspace>> {
    let rows = sqlx::query(
        "SELECT id, project_id, display_name, repository_kind, repository_identity,
                created_at, updated_at, archived_at, revision
         FROM chat_workspaces ORDER BY project_id IS NULL, project_id, display_name COLLATE NOCASE, id",
    )
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter().map(row_to_workspace).collect()
}

pub async fn read_workspace(
    pool: &SqlitePool,
    id: &ChatWorkspaceId,
) -> ChatResult<LogicalChatWorkspace> {
    sqlx::query(
        "SELECT id, project_id, display_name, repository_kind, repository_identity,
                created_at, updated_at, archived_at, revision
         FROM chat_workspaces WHERE id = ?",
    )
    .bind(id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .map(row_to_workspace)
    .transpose()?
    .ok_or_else(not_found)
}

pub async fn create_workspace(
    pool: &SqlitePool,
    request: &CreateChatWorkspaceRequest,
    now: &UtcTimestamp,
) -> ChatResult<LogicalChatWorkspace> {
    validate_name(&request.display_name)?;
    sqlx::query(
        "INSERT INTO chat_workspaces
            (id, project_id, display_name, repository_kind, created_at, updated_at)
         VALUES (?, ?, ?, 'none', ?, ?)",
    )
    .bind(request.id.as_str())
    .bind(request.project_id.as_deref())
    .bind(request.display_name.trim())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    read_workspace(pool, &request.id).await
}

pub async fn rename_workspace(
    pool: &SqlitePool,
    id: &ChatWorkspaceId,
    display_name: &str,
    expected_revision: u64,
    now: &UtcTimestamp,
) -> ChatResult<LogicalChatWorkspace> {
    validate_name(display_name)?;
    let result = sqlx::query(
        "UPDATE chat_workspaces SET display_name = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(display_name.trim())
    .bind(now.as_str())
    .bind(id.as_str())
    .bind(i64::try_from(expected_revision).unwrap_or(i64::MAX))
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    require_updated(pool, id, result.rows_affected()).await
}

pub async fn set_workspace_repository(
    pool: &SqlitePool,
    id: &ChatWorkspaceId,
    kind: RepositoryKind,
    identity: Option<&str>,
    now: &UtcTimestamp,
) -> ChatResult<LogicalChatWorkspace> {
    if (kind == RepositoryKind::Git) != identity.is_some() {
        return Err(ChatError::validation(
            "repositoryIdentity",
            "Chat repository identity is inconsistent",
        ));
    }
    let result = sqlx::query(
        "UPDATE chat_workspaces SET repository_kind = ?, repository_identity = ?,
                revision = revision + 1, updated_at = ? WHERE id = ?
           AND (repository_identity IS NULL OR repository_identity IS ?)",
    )
    .bind(match kind {
        RepositoryKind::Git => "git",
        RepositoryKind::None => "none",
    })
    .bind(identity)
    .bind(now.as_str())
    .bind(id.as_str())
    .bind(identity)
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    require_updated(pool, id, result.rows_affected()).await
}

pub async fn set_workspace_archived(
    pool: &SqlitePool,
    id: &ChatWorkspaceId,
    archived: bool,
    expected_revision: u64,
    now: &UtcTimestamp,
) -> ChatResult<LogicalChatWorkspace> {
    let result = sqlx::query(
        "UPDATE chat_workspaces SET archived_at = ?, revision = revision + 1, updated_at = ?
         WHERE id = ? AND revision = ?",
    )
    .bind(archived.then_some(now.as_str()))
    .bind(now.as_str())
    .bind(id.as_str())
    .bind(i64::try_from(expected_revision).unwrap_or(i64::MAX))
    .execute(pool)
    .await
    .map_err(persistence_error)?;
    require_updated(pool, id, result.rows_affected()).await
}

async fn require_updated(
    pool: &SqlitePool,
    id: &ChatWorkspaceId,
    rows: u64,
) -> ChatResult<LogicalChatWorkspace> {
    if rows == 1 {
        return read_workspace(pool, id).await;
    }
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chat_workspaces WHERE id = ?)")
            .bind(id.as_str())
            .fetch_one(pool)
            .await
            .map_err(persistence_error)?;
    Err(if exists {
        ChatError::new(
            ChatErrorCode::StaleRevision,
            "Chat workspace revision is stale",
            true,
        )
    } else {
        not_found()
    })
}

fn row_to_workspace(row: sqlx::sqlite::SqliteRow) -> ChatResult<LogicalChatWorkspace> {
    let kind: String = row.try_get("repository_kind").map_err(persistence_error)?;
    Ok(LogicalChatWorkspace {
        id: ChatWorkspaceId::new(row.try_get::<String, _>("id").map_err(persistence_error)?)
            .map_err(|_| corrupt_data())?,
        project_id: row.try_get("project_id").map_err(persistence_error)?,
        display_name: row.try_get("display_name").map_err(persistence_error)?,
        repository_kind: match kind.as_str() {
            "git" => RepositoryKind::Git,
            "none" => RepositoryKind::None,
            _ => return Err(corrupt_data()),
        },
        repository_identity: row
            .try_get("repository_identity")
            .map_err(persistence_error)?,
        created_at: timestamp(
            row.try_get::<String, _>("created_at")
                .map_err(persistence_error)?,
        )?,
        updated_at: timestamp(
            row.try_get::<String, _>("updated_at")
                .map_err(persistence_error)?,
        )?,
        archived_at: row
            .try_get::<Option<String>, _>("archived_at")
            .map_err(persistence_error)?
            .map(timestamp)
            .transpose()?,
        revision: u64::try_from(
            row.try_get::<i64, _>("revision")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
    })
}

fn validate_name(value: &str) -> ChatResult<()> {
    if value.trim().is_empty() || value.trim().len() > 240 {
        Err(ChatError::validation(
            "displayName",
            "Chat workspace name is invalid",
        ))
    } else {
        Ok(())
    }
}
fn timestamp(value: String) -> ChatResult<UtcTimestamp> {
    UtcTimestamp::new(value).map_err(|_| corrupt_data())
}
fn not_found() -> ChatError {
    ChatError::new(
        ChatErrorCode::NotFound,
        "Chat workspace was not found",
        true,
    )
}
fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat workspace persistence failed",
        true,
    )
}
fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat workspace is invalid",
        false,
    )
}
