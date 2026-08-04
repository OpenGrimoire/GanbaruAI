//! Bounded, workspace-authorized file browsing and preview commands.

use super::models::{ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId, RepositoryKind};
use super::workspace::{
    resolve_workspace_relative_path, AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::db_path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
#[cfg(windows)]
use std::path::PathBuf;
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_SHARE_READ, FILE_SHARE_WRITE,
};

#[cfg(unix)]
use std::ffi::{CStr, CString};
#[cfg(unix)]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

mod platform;

use platform::{
    create_workspace_file_exclusively, secure_directory_entries, secure_workspace_file,
    workspace_regular_file_permissions, write_workspace_text_atomically,
};
#[cfg(all(test, unix))]
use platform::{open_regular_file_at, secure_workspace_parent};

const MAX_DIRECTORY_ENTRIES: usize = 5_000;
const MAX_PREVIEW_BYTES: u64 = 1024 * 1024;
const MAX_RELATIVE_PATH_BYTES: usize = 4_096;
static WORKSPACE_FILE_WRITE_GENERATION: AtomicU64 = AtomicU64::new(0);
static WORKSPACE_FILE_WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderFileEntry {
    pub relative_path: String,
    pub display_name: String,
    pub kind: String,
    pub ignored: bool,
    pub byte_size: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderDirectoryRead {
    pub relative_path: String,
    pub entries: Vec<ProjectWorkingFolderFileEntry>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderFilePreview {
    pub relative_path: String,
    pub display_name: String,
    pub text: Option<String>,
    pub line_count: Option<u64>,
    pub byte_size: u64,
    pub binary: bool,
    pub oversized: bool,
    pub content_revision: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectWorkingFolderFileRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub relative_path: String,
    pub contents: String,
    pub expected_revision: String,
    pub execution_environment_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectWorkingFolderFileCopyRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub source_relative_path: String,
    pub target_relative_path: String,
    pub contents: String,
    pub execution_environment_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecreateProjectWorkingFolderFileRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub relative_path: String,
    pub contents: String,
    pub confirmed: bool,
    pub execution_environment_id: Option<String>,
}

#[tauri::command]
pub async fn project_list_working_folder_directory(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
    include_ignored: bool,
    execution_environment_id: Option<String>,
) -> ChatResult<ProjectWorkingFolderDirectoryRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &working_folder_id).await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        execution_environment_id.as_deref(),
    )
    .await?;
    list_workspace_directory(&authorized, &relative_path, include_ignored)
}

#[tauri::command]
pub async fn project_preview_working_folder_file(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
    execution_environment_id: Option<String>,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &working_folder_id).await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        execution_environment_id.as_deref(),
    )
    .await?;
    preview_workspace_file(&authorized, &relative_path)
}

#[tauri::command]
pub async fn project_save_working_folder_file(
    app: tauri::AppHandle,
    observers: tauri::State<'_, super::workspace_observer::ChatWorkspaceObserverRegistry>,
    mutations: tauri::State<'_, super::workspace_mutation::ChatWorkspaceMutationRegistry>,
    db_url: String,
    request: SaveProjectWorkingFolderFileRequest,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace_for(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileWrite,
    )
    .await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    let _mutation = mutations.try_mutation(&authorized.canonical_path)?;
    let saved = save_workspace_file(
        &authorized,
        &request.relative_path,
        &request.contents,
        &request.expected_revision,
    )?;
    observers.invalidate_paths(
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
        vec![request.relative_path],
        false,
    );
    Ok(saved)
}

#[tauri::command]
pub async fn project_save_working_folder_file_copy(
    app: tauri::AppHandle,
    observers: tauri::State<'_, super::workspace_observer::ChatWorkspaceObserverRegistry>,
    mutations: tauri::State<'_, super::workspace_mutation::ChatWorkspaceMutationRegistry>,
    db_url: String,
    request: SaveProjectWorkingFolderFileCopyRequest,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace_for(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileWrite,
    )
    .await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    let _mutation = mutations.try_mutation(&authorized.canonical_path)?;
    let saved = save_workspace_file_copy(
        &authorized,
        &request.source_relative_path,
        &request.target_relative_path,
        &request.contents,
    )?;
    observers.invalidate_paths(
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
        vec![request.target_relative_path],
        false,
    );
    Ok(saved)
}

#[tauri::command]
pub async fn project_recreate_working_folder_file(
    app: tauri::AppHandle,
    observers: tauri::State<'_, super::workspace_observer::ChatWorkspaceObserverRegistry>,
    mutations: tauri::State<'_, super::workspace_mutation::ChatWorkspaceMutationRegistry>,
    db_url: String,
    request: RecreateProjectWorkingFolderFileRequest,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace_for(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileWrite,
    )
    .await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        request.execution_environment_id.as_deref(),
    )
    .await?;
    let _mutation = mutations.try_mutation(&authorized.canonical_path)?;
    let recreated = recreate_workspace_file(
        &authorized,
        &request.relative_path,
        &request.contents,
        request.confirmed,
    )?;
    observers.invalidate_paths(
        &request.working_folder_id,
        request.execution_environment_id.as_deref(),
        vec![request.relative_path],
        false,
    );
    Ok(recreated)
}

#[tauri::command]
pub async fn project_open_working_folder_file(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
    execution_environment_id: Option<String>,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &working_folder_id).await?;
    let authorized = super::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        execution_environment_id.as_deref(),
    )
    .await?;
    let path = resolve_workspace_relative_path(&authorized, &relative_path)?;
    super::workspace::open_authorized_path(&authorized, &path)
}

pub fn list_workspace_directory(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
    include_ignored: bool,
) -> ChatResult<ProjectWorkingFolderDirectoryRead> {
    validate_optional_relative_path(relative_path)?;
    let (directory_entries, truncated) =
        secure_directory_entries(&authorized.canonical_path, relative_path)?;
    let mut entries = Vec::with_capacity(directory_entries.len());
    for entry in directory_entries {
        let display_name = entry.display_name;
        let child_relative = if relative_path.is_empty() {
            display_name.clone()
        } else {
            format!("{relative_path}/{display_name}")
        };
        if child_relative.len() > MAX_RELATIVE_PATH_BYTES
            || child_relative.chars().any(char::is_control)
            || safety_excluded(&child_relative)
        {
            continue;
        }
        entries.push(ProjectWorkingFolderFileEntry {
            ignored: common_ignored(&child_relative),
            relative_path: child_relative,
            display_name,
            kind: if entry.directory { "directory" } else { "file" }.to_string(),
            byte_size: entry.byte_size,
        });
    }
    if authorized.repository_kind == RepositoryKind::Git {
        let git_ignored = git_ignored_paths(
            &authorized.canonical_path,
            entries.iter().map(|entry| entry.relative_path.as_str()),
        );
        for entry in &mut entries {
            entry.ignored |= git_ignored.contains(&entry.relative_path);
        }
    }
    if !include_ignored {
        entries.retain(|entry| !entry.ignored);
    }
    entries.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| {
                left.display_name
                    .to_lowercase()
                    .cmp(&right.display_name.to_lowercase())
            })
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    Ok(ProjectWorkingFolderDirectoryRead {
        relative_path: relative_path.to_string(),
        entries,
        truncated,
    })
}

pub fn preview_workspace_file(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    validate_required_relative_path(relative_path)?;
    if safety_excluded(relative_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "This workspace file is excluded from Chat preview",
            false,
        ));
    }
    let path = Path::new(relative_path);
    let mut file = secure_workspace_file(&authorized.canonical_path, relative_path)?;
    let metadata = file.metadata().map_err(file_error)?;
    if !metadata.is_file() {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path is not a regular file",
        ));
    }
    let display_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace filename is unsupported"))?
        .to_string();
    if metadata.len() > MAX_PREVIEW_BYTES {
        return Ok(ProjectWorkingFolderFilePreview {
            relative_path: relative_path.to_string(),
            display_name,
            text: None,
            line_count: None,
            byte_size: metadata.len(),
            binary: false,
            oversized: true,
            content_revision: None,
        });
    }
    let modified = metadata.modified().ok();
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or_default());
    (&mut file)
        .take(MAX_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(file_error)?;
    let after = file.metadata().map_err(file_error)?;
    if bytes.len() as u64 != metadata.len()
        || after.len() != metadata.len()
        || modified.is_some() && after.modified().ok() != modified
    {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Workspace file changed during preview",
            true,
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .ok()
        .filter(|_| !bytes.contains(&0))
        .map(ToOwned::to_owned);
    let binary = text.is_none();
    let content_revision = text
        .as_ref()
        .map(|_| workspace_file_revision(relative_path, &bytes));
    let line_count = text.as_ref().map(|value| {
        if value.is_empty() {
            0
        } else {
            value.lines().count() as u64
        }
    });
    Ok(ProjectWorkingFolderFilePreview {
        relative_path: relative_path.to_string(),
        display_name,
        text,
        line_count,
        byte_size: metadata.len(),
        binary,
        oversized: false,
        content_revision,
    })
}

pub fn save_workspace_file(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
    contents: &str,
    expected_revision: &str,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    validate_required_relative_path(relative_path)?;
    if safety_excluded(relative_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "This workspace file is excluded from Chat editing",
            false,
        ));
    }
    if contents.len() as u64 > MAX_PREVIEW_BYTES || contents.contains('\0') {
        return Err(ChatError::validation(
            "contents",
            "Workspace files must be UTF-8 text no larger than 1 MiB",
        ));
    }
    let _guard = WORKSPACE_FILE_WRITE_LOCK
        .lock()
        .map_err(|_| workspace_file_write_error())?;
    let current = preview_workspace_file(authorized, relative_path)?;
    let current_revision = current.content_revision.as_deref().ok_or_else(|| {
        ChatError::validation("relativePath", "Workspace file is not editable text")
    })?;
    if current_revision != expected_revision {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "The workspace file changed outside Ganbaru. Reload or compare it before saving.",
            true,
        ));
    }
    write_workspace_text_atomically(
        &authorized.canonical_path,
        relative_path,
        contents,
        expected_revision,
    )?;
    preview_workspace_file(authorized, relative_path)
}

pub fn save_workspace_file_copy(
    authorized: &AuthorizedWorkingFolder,
    source_relative_path: &str,
    target_relative_path: &str,
    contents: &str,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    validate_required_relative_path(source_relative_path)?;
    validate_required_relative_path(target_relative_path)?;
    if source_relative_path == target_relative_path {
        return Err(ChatError::validation(
            "targetRelativePath",
            "Save-copy path must be different from the edited file",
        ));
    }
    if safety_excluded(source_relative_path) || safety_excluded(target_relative_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "This workspace path is excluded from Chat editing",
            false,
        ));
    }
    if contents.len() as u64 > MAX_PREVIEW_BYTES || contents.contains('\0') {
        return Err(ChatError::validation(
            "contents",
            "Workspace files must be UTF-8 text no larger than 1 MiB",
        ));
    }
    let _guard = WORKSPACE_FILE_WRITE_LOCK
        .lock()
        .map_err(|_| workspace_file_write_error())?;
    let source_permissions = workspace_regular_file_permissions(
        &authorized.canonical_path,
        source_relative_path,
        "sourceRelativePath",
    )?;
    create_workspace_file_exclusively(
        &authorized.canonical_path,
        target_relative_path,
        contents,
        Some(source_permissions),
        "Save-copy target already exists",
    )?;
    preview_workspace_file(authorized, target_relative_path)
}

pub fn recreate_workspace_file(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
    contents: &str,
    confirmed: bool,
) -> ChatResult<ProjectWorkingFolderFilePreview> {
    if !confirmed {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Recreating a deleted workspace file requires confirmation",
            true,
        ));
    }
    validate_required_relative_path(relative_path)?;
    if safety_excluded(relative_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "This workspace path is excluded from Chat editing",
            false,
        ));
    }
    if contents.len() as u64 > MAX_PREVIEW_BYTES || contents.contains('\0') {
        return Err(ChatError::validation(
            "contents",
            "Workspace files must be UTF-8 text no larger than 1 MiB",
        ));
    }
    let _guard = WORKSPACE_FILE_WRITE_LOCK
        .lock()
        .map_err(|_| workspace_file_write_error())?;
    create_workspace_file_exclusively(
        &authorized.canonical_path,
        relative_path,
        contents,
        None,
        "Workspace file already exists. Reload it before saving.",
    )?;
    preview_workspace_file(authorized, relative_path)
}

async fn require_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
) -> ChatResult<AuthorizedWorkingFolder> {
    require_workspace_for(
        app,
        pool,
        working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await
}

pub(crate) async fn require_workspace_for(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<AuthorizedWorkingFolder> {
    super::workspace_commands::authorize_working_folder(app, pool, working_folder_id, operation)
        .await
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn validate_optional_relative_path(value: &str) -> ChatResult<()> {
    if value.is_empty() {
        Ok(())
    } else {
        validate_required_relative_path(value)
    }
}

fn validate_required_relative_path(value: &str) -> ChatResult<()> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > MAX_RELATIVE_PATH_BYTES
        || path.is_absolute()
        || value.contains('\u{005c}')
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path must be a normalized relative path",
        ));
    }
    Ok(())
}

fn safety_excluded(relative_path: &str) -> bool {
    super::interaction_commands::workspace_mention_is_safety_excluded(relative_path)
}

fn common_ignored(relative_path: &str) -> bool {
    relative_path.split('/').any(|segment| {
        let ganbaru_temporary = segment.starts_with('.')
            && segment.contains(".ganbaru.")
            && (segment.ends_with(".tmp") || segment.ends_with(".backup"));
        matches!(
            segment,
            ".git" | ".cache" | ".turbo" | "node_modules" | "target" | "dist" | "build"
        ) || ganbaru_temporary
    })
}

pub(crate) fn observer_excluded(relative_path: &str) -> bool {
    safety_excluded(relative_path) || common_ignored(relative_path)
}

fn git_ignored_paths<'a>(root: &Path, paths: impl Iterator<Item = &'a str>) -> HashSet<String> {
    let mut input = Vec::new();
    for path in paths {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
    }
    if input.is_empty() {
        return HashSet::new();
    }
    let mut child = match Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["check-ignore", "-z", "--stdin"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return HashSet::new(),
    };
    if child
        .stdin
        .take()
        .is_none_or(|mut stdin| stdin.write_all(&input).is_err())
    {
        return HashSet::new();
    }
    let Ok(output) = child.wait_with_output() else {
        return HashSet::new();
    };
    if !matches!(output.status.code(), Some(0 | 1)) || output.stdout.len() > input.len() {
        return HashSet::new();
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .filter_map(|value| std::str::from_utf8(value).ok().map(ToOwned::to_owned))
        .collect()
}

fn workspace_file_revision(relative_path: &str, bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"workspace-file-v1\0");
    digest.update(relative_path.as_bytes());
    digest.update(b"\0");
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn file_error(error: std::io::Error) -> ChatError {
    if error.kind() == std::io::ErrorKind::NotFound {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Workspace file or directory no longer exists",
            true,
        )
    } else {
        ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file could not be read safely",
            true,
        )
    }
}

fn workspace_file_write_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Workspace file could not be saved safely",
        true,
    )
}

fn workspace_file_recovery_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Conflict,
        "Workspace file recovery could not finish safely. A partial file or Ganbaru recovery artifact remains in the workspace.",
        true,
    )
}

fn stale_workspace_file_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Conflict,
        "The workspace file changed outside Ganbaru. Reload or compare it before saving.",
        true,
    )
}

#[cfg(test)]
mod tests;
