//! Review ownership and working-folder authorization.

use super::super::models::{ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId};
use super::super::workspace::{AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation};
use super::contracts::{
    ApplyChatReviewActionRequest, ChatReviewActionResultRead, OpenChatReviewRequest,
};
use super::registry::ReviewSnapshot;
use super::validation::{source_requires_thread, thread_required};
use super::{default_context_lines, persistence_error};
use crate::chat::{execution_environment, workspace_commands};
use sqlx::{Row, SqlitePool};

pub(super) async fn require_request_ownership(
    pool: &SqlitePool,
    snapshot: &ReviewSnapshot,
    working_folder_id: &ProjectWorkingFolderId,
    execution_environment_id: Option<&str>,
) -> ChatResult<()> {
    let requested_environment =
        resolve_requested_environment_id(pool, working_folder_id, execution_environment_id).await?;
    if &snapshot.working_folder_id != working_folder_id
        || requested_environment != snapshot.environment_id
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Review snapshot belongs to another execution environment",
            false,
        ));
    }
    Ok(())
}

pub(super) async fn resolve_requested_environment_id(
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
    execution_environment_id: Option<&str>,
) -> ChatResult<String> {
    match execution_environment_id {
        Some(environment) => Ok(environment.to_string()),
        None => sqlx::query_scalar(
            "SELECT id FROM chat_execution_environments
             WHERE working_folder_id = ? AND kind = 'current_folder'
               AND lifecycle_state = 'available' AND archived_at IS NULL",
        )
        .bind(working_folder_id.as_str())
        .fetch_optional(pool)
        .await
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::ConfigurationInvalid,
                "The current-folder execution environment is unavailable",
                true,
            )
        }),
    }
}

pub(super) async fn authorize_review_request(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &OpenChatReviewRequest,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<(ProjectWorkingFolderId, String, AuthorizedWorkingFolder)> {
    if source_requires_thread(&request.source) {
        let thread_id = request.thread_id.as_ref().ok_or_else(thread_required)?;
        let (working_folder_id, environment_id, authorized, _) =
            execution_environment::authorize_thread_environment(app, pool, thread_id, operation)
                .await?;
        let requested_environment = resolve_requested_environment_id(
            pool,
            &request.working_folder_id,
            request.execution_environment_id.as_deref(),
        )
        .await?;
        if working_folder_id != request.working_folder_id || requested_environment != environment_id
        {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Review source belongs to another workspace or execution environment",
                false,
            ));
        }
        return Ok((working_folder_id, environment_id, authorized));
    }
    let environment_id = resolve_requested_environment_id(
        pool,
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    if let Some(thread_id) = request.thread_id.as_ref() {
        let row = sqlx::query(
            "SELECT working_folder_id, execution_environment_id FROM chat_threads
             WHERE id = ? AND state != 'closed'",
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
        let thread_working_folder: String = row
            .try_get("working_folder_id")
            .map_err(persistence_error)?;
        let thread_environment: Option<String> = row
            .try_get("execution_environment_id")
            .map_err(persistence_error)?;
        if thread_working_folder != request.working_folder_id.as_str()
            || thread_environment.as_deref() != Some(environment_id.as_str())
        {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Chat thread belongs to another workspace or execution environment",
                false,
            ));
        }
    }
    let authorized = workspace_commands::authorize_working_folder(
        app,
        pool,
        &request.working_folder_id,
        operation,
    )
    .await?;
    let authorized = execution_environment::resolve_environment_workspace(
        app,
        pool,
        authorized,
        Some(&environment_id),
    )
    .await?;
    Ok((
        request.working_folder_id.clone(),
        environment_id,
        authorized,
    ))
}

pub(super) async fn authorize_snapshot(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    snapshot: &ReviewSnapshot,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<()> {
    let request = OpenChatReviewRequest {
        thread_id: snapshot.thread_id.clone(),
        working_folder_id: snapshot.working_folder_id.clone(),
        execution_environment_id: Some(snapshot.environment_id.clone()),
        source: snapshot.source.clone(),
        ignore_whitespace: snapshot.ignore_whitespace,
        context_lines: snapshot.context_lines,
        preferred_relative_path: None,
    };
    let (working_folder_id, environment_id, authorized) =
        authorize_review_request(app, pool, &request, operation).await?;
    if working_folder_id != snapshot.working_folder_id
        || environment_id != snapshot.environment_id
        || authorized.canonical_path != snapshot.root
    {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Review snapshot belongs to another execution environment",
            false,
        ));
    }
    Ok(())
}

pub(super) async fn authorize_completed_action(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    request: &ApplyChatReviewActionRequest,
    result: &ChatReviewActionResultRead,
) -> ChatResult<()> {
    let completed_request = OpenChatReviewRequest {
        thread_id: request.thread_id.clone(),
        working_folder_id: request.working_folder_id.clone(),
        execution_environment_id: request.execution_environment_id.clone(),
        source: result.snapshot.source.clone(),
        ignore_whitespace: false,
        context_lines: default_context_lines(),
        preferred_relative_path: None,
    };
    authorize_review_request(
        app,
        pool,
        &completed_request,
        WorkingFolderAuthorizationOperation::Git,
    )
    .await
    .map(|_| ())
}
