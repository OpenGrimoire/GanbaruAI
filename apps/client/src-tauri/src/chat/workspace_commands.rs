//! Tauri commands for project-owned working-folder management.

use super::models::{ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId, UtcTimestamp};
use super::repository::workspaces as repository;
use super::workspace::{
    authorize_workspace, ensure_managed_working_folder_binding, open_authorized_workspace,
    prepare_workspace_binding, remove_active_device_binding, store_active_device_binding,
    validate_external_folder_outside_vault, workspace_read, CreateProjectWorkingFolderRequest,
    ProjectWorkingFolderRead, WorkingFolderAuthorizationOperation, WorkingFolderKind,
};
use crate::db_path;
use crate::projects::working_folders::{
    read_active_working_folder_scope, update_active_working_folder_scope,
};
use chrono::{SecondsFormat, Utc};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};

#[tauri::command]
pub async fn projects_list_working_folders(
    app: tauri::AppHandle,
    db_url: String,
) -> ChatResult<Vec<ProjectWorkingFolderRead>> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspaces = repository::list_workspaces(&pool).await?;
    for workspace in &workspaces {
        ensure_managed_working_folder_binding(&app, workspace)?;
    }
    let scope = read_active_working_folder_scope(&app).map_err(device_state_error)?;
    workspaces
        .into_iter()
        .map(|workspace| workspace_read(workspace, &scope))
        .collect()
}

#[tauri::command]
pub async fn projects_add_external_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    mut request: CreateProjectWorkingFolderRequest,
    title: String,
) -> ChatResult<Option<ProjectWorkingFolderRead>> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let Some(selection) = pick_workspace_folder(&app, &title, None).await? else {
        return Ok(None);
    };
    if request.display_name.trim().is_empty() {
        request.display_name = selection
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("External folder")
            .to_string();
    }
    validate_external_folder_outside_vault(&app, &selection)?;
    ensure_unique_project_path(&pool, &app, &request.project_id, None, &selection).await?;
    let workspace = repository::create_workspace(&pool, &request, &now_timestamp()?).await?;
    let (probe, binding) = prepare_workspace_binding(&workspace, &selection)?;
    let workspace = repository::set_workspace_repository(
        &pool,
        &workspace.id,
        probe.kind,
        probe.identity.as_deref(),
        &now_timestamp()?,
    )
    .await?;
    store_active_device_binding(&app, &workspace.id, binding)?;
    read_workspace(&app, workspace).map(Some)
}

#[tauri::command]
pub async fn projects_rename_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    display_name: String,
    expected_revision: u64,
) -> ChatResult<ProjectWorkingFolderRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::rename_workspace(
        &pool,
        &working_folder_id,
        &display_name,
        expected_revision,
        &now_timestamp()?,
    )
    .await?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn projects_locate_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    title: String,
) -> ChatResult<Option<ProjectWorkingFolderRead>> {
    pick_and_bind_workspace(&app, &db_url, &working_folder_id, &title).await
}

#[tauri::command]
pub async fn projects_rebind_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    title: String,
) -> ChatResult<Option<ProjectWorkingFolderRead>> {
    pick_and_bind_workspace(&app, &db_url, &working_folder_id, &title).await
}

#[tauri::command]
pub async fn projects_unbind_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<ProjectWorkingFolderRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::read_workspace(&pool, &working_folder_id).await?;
    if workspace.kind == WorkingFolderKind::Managed {
        return Err(ChatError::validation(
            "workingFolderId",
            "Managed project working folders cannot be unbound",
        ));
    }
    app.state::<super::terminal::ChatTerminalRegistry>()
        .shutdown_workspace(&working_folder_id)?;
    remove_active_device_binding(&app, &working_folder_id)?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn projects_archive_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    expected_revision: u64,
) -> ChatResult<ProjectWorkingFolderRead> {
    set_workspace_archived(app, db_url, working_folder_id, expected_revision, true).await
}

#[tauri::command]
pub async fn projects_restore_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    expected_revision: u64,
) -> ChatResult<ProjectWorkingFolderRead> {
    set_workspace_archived(app, db_url, working_folder_id, expected_revision, false).await
}

#[tauri::command]
pub async fn projects_open_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::read_workspace(&pool, &working_folder_id).await?;
    let scope = read_active_working_folder_scope(&app).map_err(device_state_error)?;
    let authorized = authorize_workspace(
        &workspace,
        &scope,
        WorkingFolderAuthorizationOperation::FileRead,
    )?;
    open_authorized_workspace(&authorized)
}

#[tauri::command]
pub async fn projects_remove_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    app.state::<super::terminal::ChatTerminalRegistry>()
        .shutdown_workspace(&working_folder_id)?;
    repository::remove_external_working_folder(&pool, &working_folder_id).await?;
    remove_active_device_binding(&app, &working_folder_id)
}

#[tauri::command]
pub async fn projects_recreate_managed_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<ProjectWorkingFolderRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let workspace = repository::read_workspace(&pool, &working_folder_id).await?;
    if workspace.kind != WorkingFolderKind::Managed {
        return Err(ChatError::validation(
            "workingFolderId",
            "Only managed project working folders can be recreated",
        ));
    }
    remove_active_device_binding(&app, &working_folder_id)?;
    ensure_managed_working_folder_binding(&app, &workspace)?;
    read_workspace(&app, workspace)
}

#[tauri::command]
pub async fn projects_remember_working_folder(
    app: tauri::AppHandle,
    db_url: String,
    project_id: String,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let folder = repository::read_workspace(&pool, &working_folder_id).await?;
    if folder.project_id != project_id || folder.archived_at.is_some() {
        return Err(ChatError::validation(
            "workingFolderId",
            "The selected working folder is not active in this project",
        ));
    }
    update_active_working_folder_scope(&app, |scope| {
        scope
            .last_selected_by_project
            .insert(project_id, working_folder_id);
        Ok(())
    })
    .map_err(device_state_error)
}

#[tauri::command]
pub fn projects_last_working_folder(
    app: tauri::AppHandle,
    project_id: String,
) -> ChatResult<Option<ProjectWorkingFolderId>> {
    if project_id.trim().is_empty() || project_id.chars().any(char::is_control) {
        return Err(ChatError::validation("projectId", "project ID is invalid"));
    }
    Ok(read_active_working_folder_scope(&app)
        .map_err(device_state_error)?
        .last_selected_by_project
        .get(&project_id)
        .cloned())
}

async fn set_workspace_archived(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    expected_revision: u64,
    archived: bool,
) -> ChatResult<ProjectWorkingFolderRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let existing = repository::read_workspace(&pool, &working_folder_id).await?;
    if existing.kind == WorkingFolderKind::Managed {
        return Err(ChatError::validation(
            "workingFolderId",
            "Managed project working folders cannot be archived",
        ));
    }
    let workspace = repository::set_workspace_archived(
        &pool,
        &working_folder_id,
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
    working_folder_id: &ProjectWorkingFolderId,
    title: &str,
) -> ChatResult<Option<ProjectWorkingFolderRead>> {
    let pool = chat_pool(app.clone(), db_url.to_string()).await?;
    let workspace = repository::read_workspace(&pool, working_folder_id).await?;
    if workspace.kind == WorkingFolderKind::Managed {
        return Err(ChatError::validation(
            "workingFolderId",
            "Managed project working folders do not use external bindings",
        ));
    }
    let start_directory = workspace_picker_start_directory(app, working_folder_id);
    let Some(selection) = pick_workspace_folder(app, title, start_directory).await? else {
        return Ok(None);
    };
    validate_external_folder_outside_vault(app, &selection)?;
    ensure_unique_project_path(
        &pool,
        app,
        &workspace.project_id,
        Some(&workspace.id),
        &selection,
    )
    .await?;
    app.state::<super::terminal::ChatTerminalRegistry>()
        .shutdown_workspace(working_folder_id)?;
    let (probe, binding) = prepare_workspace_binding(&workspace, &selection)?;
    let workspace = repository::set_workspace_repository(
        &pool,
        working_folder_id,
        probe.kind,
        probe.identity.as_deref(),
        &now_timestamp()?,
    )
    .await?;
    store_active_device_binding(app, working_folder_id, binding)?;
    read_workspace(app, workspace).map(Some)
}

async fn ensure_unique_project_path(
    pool: &SqlitePool,
    app: &tauri::AppHandle,
    project_id: &str,
    except_id: Option<&ProjectWorkingFolderId>,
    selected_path: &Path,
) -> ChatResult<()> {
    let selected = std::fs::canonicalize(selected_path).map_err(|_| {
        ChatError::validation(
            "workingFolderPath",
            "Selected working folder is unavailable",
        )
    })?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    let siblings = repository::list_workspaces(pool).await?;
    let duplicate = siblings.iter().any(|candidate| {
        candidate.project_id == project_id
            && except_id != Some(&candidate.id)
            && scope.bindings.get(&candidate.id).is_some_and(|binding| {
                std::fs::canonicalize(&binding.canonical_path).is_ok_and(|bound| bound == selected)
            })
    });
    if duplicate {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "This folder is already assigned to the selected project",
            true,
        ));
    }
    Ok(())
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
            "Working-folder picker title is invalid",
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
                "workingFolderPath",
                "Selected project working folder is not a local folder",
            )
        })
}

fn workspace_picker_start_directory(
    app: &tauri::AppHandle,
    working_folder_id: &ProjectWorkingFolderId,
) -> Option<PathBuf> {
    let bound_path = read_active_working_folder_scope(app)
        .ok()
        .and_then(|scope| scope.bindings.get(working_folder_id).cloned())
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
    workspace: super::workspace::ProjectWorkingFolder,
) -> ChatResult<ProjectWorkingFolderRead> {
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
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
        "Project working-folder device state could not be read",
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
