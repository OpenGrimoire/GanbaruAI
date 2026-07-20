//! Narrow Tauri commands for logical Chat workspace management.

use super::device_state::read_active_device_scope;
use super::models::{ChatError, ChatErrorCode, ChatResult, ChatWorkspaceId};
use super::workspace::{
    authorize_workspace, bind_workspace_path, open_authorized_workspace, read_workspaces,
    remove_active_device_binding, ChatWorkspaceCatalogState, ChatWorkspaceRead,
    CreateChatWorkspaceRequest, WorkspaceAuthorizationOperation,
};
use std::path::PathBuf;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command]
pub fn chat_list_workspaces(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
) -> ChatResult<Vec<ChatWorkspaceRead>> {
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    read_workspaces(&catalog, &scope)
}

#[tauri::command]
pub fn chat_create_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    request: CreateChatWorkspaceRequest,
) -> ChatResult<ChatWorkspaceRead> {
    let workspace = catalog.create(request)?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    read_workspaces(&catalog, &scope)?
        .into_iter()
        .find(|candidate| candidate.workspace.id == workspace.id)
        .ok_or_else(|| ChatError::new(ChatErrorCode::Internal, "read new Chat workspace", false))
}

#[tauri::command]
pub fn chat_rename_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
    display_name: String,
) -> ChatResult<ChatWorkspaceRead> {
    catalog.rename(&workspace_id, &display_name)?;
    read_workspace(&app, &catalog, &workspace_id)
}

#[tauri::command]
pub async fn chat_bind_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    pick_and_bind_workspace(&app, &catalog, &workspace_id).await
}

#[tauri::command]
pub async fn chat_rebind_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    pick_and_bind_workspace(&app, &catalog, &workspace_id).await
}

#[tauri::command]
pub fn chat_remove_workspace_binding(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatWorkspaceRead> {
    catalog.get(&workspace_id)?;
    remove_active_device_binding(&app, &workspace_id)?;
    read_workspace(&app, &catalog, &workspace_id)
}

#[tauri::command]
pub fn chat_archive_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatWorkspaceRead> {
    catalog.set_archived(&workspace_id, true)?;
    read_workspace(&app, &catalog, &workspace_id)
}

#[tauri::command]
pub fn chat_restore_workspace(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<ChatWorkspaceRead> {
    catalog.set_archived(&workspace_id, false)?;
    read_workspace(&app, &catalog, &workspace_id)
}

#[tauri::command]
pub fn chat_open_workspace_folder(
    app: tauri::AppHandle,
    catalog: tauri::State<'_, ChatWorkspaceCatalogState>,
    workspace_id: ChatWorkspaceId,
) -> ChatResult<()> {
    let workspace = catalog.get(&workspace_id)?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    let authorized = authorize_workspace(
        &workspace,
        &scope,
        WorkspaceAuthorizationOperation::FileRead,
    )?;
    open_authorized_workspace(&authorized)
}

async fn pick_and_bind_workspace(
    app: &tauri::AppHandle,
    catalog: &ChatWorkspaceCatalogState,
    workspace_id: &ChatWorkspaceId,
) -> ChatResult<Option<ChatWorkspaceRead>> {
    catalog.get(workspace_id)?;
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    app.dialog()
        .file()
        .set_title("Select a Chat workspace folder")
        .pick_folder(move |selection| {
            let result = selection.map(file_path_to_path_buf).transpose();
            let _ = sender.try_send(result);
        });
    let selection = receiver
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
        })?;
    selection
        .as_deref()
        .map(|path| bind_workspace_path(app, catalog, workspace_id, path))
        .transpose()
}

fn read_workspace(
    app: &tauri::AppHandle,
    catalog: &ChatWorkspaceCatalogState,
    workspace_id: &ChatWorkspaceId,
) -> ChatResult<ChatWorkspaceRead> {
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    read_workspaces(catalog, &scope)?
        .into_iter()
        .find(|candidate| &candidate.workspace.id == workspace_id)
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "Chat workspace was not found",
                true,
            )
        })
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
