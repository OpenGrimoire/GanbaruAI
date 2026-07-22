//! Durable Tauri commands for logical Chat workspace management.

use super::device_state::read_active_device_scope;
use super::models::{ChatError, ChatErrorCode, ChatResult, ChatWorkspaceId, UtcTimestamp};
use super::repository::workspaces as repository;
use super::workspace::{
    authorize_workspace, open_authorized_workspace, prepare_workspace_binding,
    remove_active_device_binding, store_active_device_binding, workspace_read, ChatWorkspaceRead,
    CreateChatWorkspaceRequest, WorkspaceAuthorizationOperation,
};
use crate::db_path;
use chrono::{SecondsFormat, Utc};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command]
pub async fn chat_list_workspaces(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<Vec<ChatWorkspaceRead>> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    let workspaces = repository::list_workspaces(&pool).await?;
    workspaces
        .into_iter()
        .map(|workspace| workspace_read(workspace, &scope))
        .collect()
}

#[tauri::command]
pub async fn chat_create_workspace(
    app: tauri::AppHandle,
    db_url: String,
    request: CreateChatWorkspaceRequest,
) -> ChatResult<ChatWorkspaceRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::create_workspace(&pool, &request, &now_timestamp()?).await?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn chat_rename_workspace(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    display_name: String,
    expected_revision: u64,
) -> ChatResult<ChatWorkspaceRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::rename_workspace(
        &pool,
        &workspace_id,
        &display_name,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn chat_bind_workspace(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    title: String,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    pick_and_bind_workspace(&app, &db_url, &workspace_id, &title).await
}

#[tauri::command]
pub async fn chat_rebind_workspace(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    title: String,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    pick_and_bind_workspace(&app, &db_url, &workspace_id, &title).await
}

#[tauri::command]
pub async fn chat_remove_workspace_binding(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatWorkspaceRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::read_workspace(&pool, &workspace_id).await?;
    app.state::<super::terminal::ChatTerminalRegistry>()
        .shutdown_workspace(&workspace_id)?;
    remove_active_device_binding(&app, &workspace_id)?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn chat_archive_workspace(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    expected_revision: u64,
) -> ChatResult<ChatWorkspaceRead> {
    set_workspace_archived(app, db_url, workspace_id, expected_revision, true).await
}

#[tauri::command]
pub async fn chat_restore_workspace(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    expected_revision: u64,
) -> ChatResult<ChatWorkspaceRead> {
    set_workspace_archived(app, db_url, workspace_id, expected_revision, false).await
}

#[tauri::command]
pub async fn chat_open_workspace_folder(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::read_workspace(&pool, &workspace_id).await?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    let authorized = authorize_workspace(
        &workspace,
        &scope,
        WorkspaceAuthorizationOperation::FileRead,
    )?;
    open_authorized_workspace(&authorized)
}

async fn set_workspace_archived(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    expected_revision: u64,
    archived: bool,
) -> ChatResult<ChatWorkspaceRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::set_workspace_archived(
        &pool,
        &workspace_id,
        archived,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    read_workspace(&app, workspace)
}

async fn pick_and_bind_workspace(
    app: &tauri::AppHandle,
    db_url: &str,
    workspace_id: &ChatWorkspaceId,
    title: &str,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    let pool = chat_pool(app.clone(), db_url.to_string()).await?;
    let workspace = repository::read_workspace(&pool, workspace_id).await?;
    let start_directory = workspace_picker_start_directory(app, workspace_id);
    let Some(selection) = pick_workspace_folder(app, title, start_directory).await? else {
        return Ok(None);
    };
    app.state::<super::terminal::ChatTerminalRegistry>()
        .shutdown_workspace(workspace_id)?;
    let (probe, binding) = prepare_workspace_binding(&workspace, &selection)?;
    let workspace = repository::set_workspace_repository(
        &pool,
        workspace_id,
        probe.kind,
        probe.identity.as_deref(),
        &now_timestamp()?,
    )
    .await?;
    store_active_device_binding(app, workspace_id, binding)?;
    read_workspace(app, workspace).map(Some)
}

async fn pick_workspace_folder(
    app: &tauri::AppHandle,
    title: &str,
    start_directory: Option<PathBuf>,
) -> ChatResult<Option<PathBuf>> {
    let title = title.trim();
    if title.is_empty() || title.len() > 160 || title.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "title",
            "Workspace picker title is invalid",
        ));
    }
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let mut picker = app.dialog().file().set_title(title);
    if let Some(directory) = start_directory {
        picker = picker.set_directory(directory);
    }
    picker.pick_folder(move |selection| {
        let result = selection.map(file_path_to_path_buf).transpose();
        let _ = sender.try_send(result);
    });
    receiver
        .recv()
        .await
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::Internal,
                "Folder picker closed without a result",
                true,
            )
        })?
        .map_err(|_| {
            ChatError::validation(
                "workspacePath",
                "Selected Chat workspace is not a local folder",
            )
        })
}

fn workspace_picker_start_directory(
    app: &tauri::AppHandle,
    workspace_id: &ChatWorkspaceId,
) -> Option<PathBuf> {
    let bound_path = read_active_device_scope(app)
        .ok()
        .and_then(|scope| scope.workspace_bindings.get(workspace_id).cloned())
        .map(|binding| PathBuf::from(binding.canonical_path));
    preferred_workspace_picker_directory(
        bound_path.as_deref(),
        app.path().document_dir().ok().as_deref(),
    )
}

fn preferred_workspace_picker_directory(
    bound_path: Option<&Path>,
    documents_path: Option<&Path>,
) -> Option<PathBuf> {
    bound_path
        .and_then(|path| path.ancestors().find(|candidate| candidate.is_dir()))
        .or_else(|| documents_path.filter(|path| path.is_dir()))
        .map(Path::to_path_buf)
}

fn read_workspace(
    app: &tauri::AppHandle,
    workspace: super::workspace::LogicalChatWorkspace,
) -> ChatResult<ChatWorkspaceRead> {
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    workspace_read(workspace, &scope)
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

fn file_path_to_path_buf(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|_| "selected path is not local".to_string())
}

fn device_state_error(_error: String) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be read",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::preferred_workspace_picker_directory;
    use std::path::Path;

    #[test]
    fn workspace_picker_prefers_the_existing_bound_folder() {
        let current = std::env::current_dir().expect("read current directory");
        let documents = std::env::temp_dir();

        assert_eq!(
            preferred_workspace_picker_directory(Some(&current), Some(&documents)),
            Some(current),
        );
    }

    #[test]
    fn workspace_picker_falls_back_to_documents_without_a_binding() {
        let documents = std::env::temp_dir();

        assert_eq!(
            preferred_workspace_picker_directory(None, Some(&documents)),
            Some(documents),
        );
        assert_eq!(
            preferred_workspace_picker_directory(None, Some(Path::new("missing-documents"))),
            None,
        );
    }
}
