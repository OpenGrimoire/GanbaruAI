//! Bounded, workspace-authorized file browsing and preview commands.

use super::models::{ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId, RepositoryKind};
use super::repository::workspaces;
use super::workspace::{
    authorize_workspace, resolve_workspace_relative_path, AuthorizedWorkingFolder,
    WorkingFolderAuthorizationOperation,
};
use crate::db_path;
use crate::projects::working_folders::read_active_working_folder_scope;
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
    let workspace = workspaces::read_workspace(pool, working_folder_id).await?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    authorize_workspace(&workspace, &scope, operation)
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

struct SecureDirectoryEntry {
    display_name: String,
    directory: bool,
    byte_size: Option<u64>,
}

struct SecureWorkspaceFile {
    file: File,
    #[cfg(windows)]
    _parent_handles: Vec<File>,
}

impl SecureWorkspaceFile {
    fn metadata(&self) -> std::io::Result<fs::Metadata> {
        self.file.metadata()
    }
}

impl Read for SecureWorkspaceFile {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buffer)
    }
}

#[cfg(unix)]
fn secure_workspace_directory(root: &Path, relative_path: &str) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let mut directory = options.open(root)?;
    for component in Path::new(relative_path).components() {
        let Component::Normal(name) = component else {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        };
        let name = CString::new(name.as_bytes())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
        let descriptor = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_RDONLY,
            )
        };
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error());
        }
        directory = unsafe { File::from_raw_fd(descriptor) };
    }
    Ok(directory)
}

#[cfg(unix)]
struct SecureDirectoryStream(*mut libc::DIR);

#[cfg(unix)]
impl Drop for SecureDirectoryStream {
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.0);
        }
    }
}

#[cfg(unix)]
fn secure_directory_entries(
    root: &Path,
    relative_path: &str,
) -> ChatResult<(Vec<SecureDirectoryEntry>, bool)> {
    let directory = secure_workspace_directory(root, relative_path).map_err(file_error)?;
    let duplicated = unsafe { libc::dup(directory.as_raw_fd()) };
    if duplicated < 0 {
        return Err(file_error(std::io::Error::last_os_error()));
    }
    let stream = unsafe { libc::fdopendir(duplicated) };
    if stream.is_null() {
        unsafe {
            libc::close(duplicated);
        }
        return Err(file_error(std::io::Error::last_os_error()));
    }
    let stream = SecureDirectoryStream(stream);
    let mut entries = Vec::new();
    let mut truncated = false;
    loop {
        let raw = unsafe { libc::readdir(stream.0) };
        if raw.is_null() {
            break;
        }
        let name = unsafe { CStr::from_ptr((*raw).d_name.as_ptr()) };
        if name.to_bytes() == b"." || name.to_bytes() == b".." {
            continue;
        }
        let Ok(display_name) = name.to_str() else {
            continue;
        };
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let status = unsafe {
            libc::fstatat(
                directory.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if status != 0 {
            continue;
        }
        let stat = unsafe { stat.assume_init() };
        let kind = stat.st_mode & libc::S_IFMT;
        let directory = kind == libc::S_IFDIR;
        let regular = kind == libc::S_IFREG;
        if !directory && !regular {
            continue;
        }
        entries.push(SecureDirectoryEntry {
            display_name: display_name.to_string(),
            directory,
            byte_size: regular.then(|| u64::try_from(stat.st_size).unwrap_or_default()),
        });
        if entries.len() >= MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }
    }
    Ok((entries, truncated))
}

#[cfg(unix)]
fn secure_workspace_file(root: &Path, relative_path: &str) -> ChatResult<SecureWorkspaceFile> {
    let parent = secure_workspace_parent(root, relative_path).map_err(file_error)?;
    open_regular_file_at(&parent)
        .map(|file| SecureWorkspaceFile { file })
        .map_err(file_error)
}

#[cfg(windows)]
fn windows_metadata_is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0
}

#[cfg(windows)]
fn windows_open_directory(path: &Path) -> ChatResult<File> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0 | FILE_FLAG_BACKUP_SEMANTICS.0);
    let file = options.open(path).map_err(file_error)?;
    let metadata = file.metadata().map_err(file_error)?;
    if !metadata.is_dir() || windows_metadata_is_reparse(&metadata) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace directory is unavailable or uses a reparse point",
            false,
        ));
    }
    Ok(file)
}

#[cfg(windows)]
fn windows_workspace_directory_chain(
    root: &Path,
    relative_path: &str,
) -> ChatResult<(PathBuf, Vec<File>)> {
    let mut path = root.to_path_buf();
    let mut handles = vec![windows_open_directory(&path)?];
    for component in Path::new(relative_path).components() {
        let Component::Normal(name) = component else {
            return Err(ChatError::validation(
                "relativePath",
                "Workspace path must be a normalized relative path",
            ));
        };
        path.push(name);
        handles.push(windows_open_directory(&path)?);
    }
    Ok((path, handles))
}

#[cfg(windows)]
fn windows_open_regular_file(root: &Path, relative_path: &str) -> ChatResult<(Vec<File>, File)> {
    use std::os::windows::fs::OpenOptionsExt;

    let relative = Path::new(relative_path);
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent = parent
        .to_str()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is unsupported"))?;
    let (parent_path, handles) = windows_workspace_directory_chain(root, parent)?;
    let name = relative
        .file_name()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is invalid"))?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);
    let file = options.open(parent_path.join(name)).map_err(file_error)?;
    let metadata = file.metadata().map_err(file_error)?;
    if !metadata.is_file() || windows_metadata_is_reparse(&metadata) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file is unavailable or uses a reparse point",
            false,
        ));
    }
    Ok((handles, file))
}

#[cfg(windows)]
fn secure_workspace_file(root: &Path, relative_path: &str) -> ChatResult<SecureWorkspaceFile> {
    let (parent_handles, file) = windows_open_regular_file(root, relative_path)?;
    Ok(SecureWorkspaceFile {
        file,
        _parent_handles: parent_handles,
    })
}

#[cfg(windows)]
fn secure_directory_entries(
    root: &Path,
    relative_path: &str,
) -> ChatResult<(Vec<SecureDirectoryEntry>, bool)> {
    use std::os::windows::fs::OpenOptionsExt;

    let (directory, _handles) = windows_workspace_directory_chain(root, relative_path)?;
    let mut entries = Vec::new();
    let mut truncated = false;
    for entry in fs::read_dir(&directory).map_err(file_error)? {
        let entry = entry.map_err(file_error)?;
        let Some(display_name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        let mut options = OpenOptions::new();
        options
            .read(true)
            .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0 | FILE_FLAG_BACKUP_SEMANTICS.0);
        let Ok(child) = options.open(entry.path()) else {
            continue;
        };
        let Ok(metadata) = child.metadata() else {
            continue;
        };
        if windows_metadata_is_reparse(&metadata) || (!metadata.is_file() && !metadata.is_dir()) {
            continue;
        }
        entries.push(SecureDirectoryEntry {
            display_name,
            directory: metadata.is_dir(),
            byte_size: metadata.is_file().then_some(metadata.len()),
        });
        if entries.len() >= MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }
    }
    Ok((entries, truncated))
}

#[cfg(not(any(unix, windows)))]
fn secure_workspace_file(root: &Path, relative_path: &str) -> ChatResult<SecureWorkspaceFile> {
    if workspace_path_contains_symlink(root, relative_path)? {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace symlinks are not available for Chat preview",
            false,
        ));
    }
    let path = root.join(relative_path);
    let canonical = path.canonicalize().map_err(file_error)?;
    if !canonical.starts_with(root) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file resolves outside the working folder",
            false,
        ));
    }
    File::open(canonical)
        .map(|file| SecureWorkspaceFile { file })
        .map_err(file_error)
}

#[cfg(not(any(unix, windows)))]
fn secure_directory_entries(
    root: &Path,
    relative_path: &str,
) -> ChatResult<(Vec<SecureDirectoryEntry>, bool)> {
    if !relative_path.is_empty() && workspace_path_contains_symlink(root, relative_path)? {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace symlinks are not available for Chat browsing",
            false,
        ));
    }
    let path = root.join(relative_path);
    let canonical = path.canonicalize().map_err(file_error)?;
    if !canonical.starts_with(root) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace directory resolves outside the working folder",
            false,
        ));
    }
    let mut entries = Vec::new();
    let mut truncated = false;
    for entry in fs::read_dir(canonical).map_err(file_error)? {
        let entry = entry.map_err(file_error)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(file_error)?;
        if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
            continue;
        }
        let Some(display_name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        entries.push(SecureDirectoryEntry {
            display_name,
            directory: metadata.is_dir(),
            byte_size: metadata.is_file().then_some(metadata.len()),
        });
        if entries.len() >= MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }
    }
    Ok((entries, truncated))
}

#[cfg(not(any(unix, windows)))]
fn workspace_path_contains_symlink(root: &Path, relative_path: &str) -> ChatResult<bool> {
    let mut candidate = root.to_path_buf();
    for component in Path::new(relative_path).components() {
        let Component::Normal(name) = component else {
            return Err(ChatError::validation(
                "relativePath",
                "Workspace path must be a normalized relative path",
            ));
        };
        candidate.push(name);
        let metadata = fs::symlink_metadata(&candidate).map_err(file_error)?;
        if metadata.file_type().is_symlink() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(not(any(unix, windows)))]
fn workspace_parent_contains_symlink(root: &Path, relative_path: &str) -> ChatResult<bool> {
    let Some(parent) = Path::new(relative_path).parent() else {
        return Ok(false);
    };
    if parent.as_os_str().is_empty() {
        return Ok(false);
    }
    let parent = parent
        .to_str()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is unsupported"))?;
    workspace_path_contains_symlink(root, parent)
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

#[cfg(unix)]
struct SecureWorkspaceParent {
    directory: File,
    file_name: CString,
}

#[cfg(unix)]
fn secure_workspace_parent(
    root: &Path,
    relative_path: &str,
) -> std::io::Result<SecureWorkspaceParent> {
    use std::os::unix::fs::OpenOptionsExt;

    let relative = Path::new(relative_path);
    let mut root_options = OpenOptions::new();
    root_options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
    let mut directory = root_options.open(root)?;
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            let Component::Normal(name) = component else {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
            };
            let name = CString::new(name.as_bytes())
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
            let descriptor = unsafe {
                libc::openat(
                    directory.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_RDONLY,
                )
            };
            if descriptor < 0 {
                return Err(std::io::Error::last_os_error());
            }
            directory = unsafe { File::from_raw_fd(descriptor) };
        }
    }
    let file_name = relative
        .file_name()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    Ok(SecureWorkspaceParent {
        directory,
        file_name: CString::new(file_name.as_bytes())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?,
    })
}

#[cfg(unix)]
fn open_regular_file_at(parent: &SecureWorkspaceParent) -> std::io::Result<File> {
    let descriptor = unsafe {
        libc::openat(
            parent.directory.as_raw_fd(),
            parent.file_name.as_ptr(),
            libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_RDONLY,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_fd(descriptor) };
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    Ok(file)
}

#[cfg(unix)]
fn create_file_at(
    parent: &SecureWorkspaceParent,
    file_name: &CString,
    mode: u32,
) -> std::io::Result<File> {
    let descriptor = unsafe {
        libc::openat(
            parent.directory.as_raw_fd(),
            file_name.as_ptr(),
            libc::O_CLOEXEC | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_WRONLY,
            0o600,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_fd(descriptor) };
    if unsafe { libc::fchmod(file.as_raw_fd(), mode as libc::mode_t) } != 0 {
        let error = std::io::Error::last_os_error();
        drop(file);
        let _ = unlink_at(parent, file_name);
        return Err(error);
    }
    Ok(file)
}

#[cfg(unix)]
fn unlink_at(parent: &SecureWorkspaceParent, file_name: &CString) -> std::io::Result<()> {
    if unsafe { libc::unlinkat(parent.directory.as_raw_fd(), file_name.as_ptr(), 0) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn exchange_at(
    parent: &SecureWorkspaceParent,
    left: &CString,
    right: &CString,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameat2(
            parent.directory.as_raw_fd(),
            left.as_ptr(),
            parent.directory.as_raw_fd(),
            right.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn exchange_at(
    parent: &SecureWorkspaceParent,
    left: &CString,
    right: &CString,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameatx_np(
            parent.directory.as_raw_fd(),
            left.as_ptr(),
            parent.directory.as_raw_fd(),
            right.as_ptr(),
            libc::RENAME_SWAP,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    ))
))]
fn exchange_at(
    _parent: &SecureWorkspaceParent,
    _left: &CString,
    _right: &CString,
) -> std::io::Result<()> {
    Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
}

fn revision_and_permissions(
    file: &mut File,
    relative_path: &str,
) -> ChatResult<(String, fs::Permissions)> {
    let before = file.metadata().map_err(|_| workspace_file_write_error())?;
    if !before.is_file() || before.len() > MAX_PREVIEW_BYTES {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path is not bounded editable text",
        ));
    }
    let modified = before.modified().ok();
    file.seek(SeekFrom::Start(0))
        .map_err(|_| workspace_file_write_error())?;
    let mut bytes = Vec::with_capacity(usize::try_from(before.len()).unwrap_or_default());
    (&mut *file)
        .take(MAX_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| workspace_file_write_error())?;
    let after = file.metadata().map_err(|_| workspace_file_write_error())?;
    if before.len() != after.len()
        || bytes.len() as u64 != after.len()
        || modified.is_some() && after.modified().ok() != modified
    {
        return Err(stale_workspace_file_error());
    }
    Ok((
        workspace_file_revision(relative_path, &bytes),
        after.permissions(),
    ))
}

#[cfg(unix)]
fn workspace_regular_file_permissions(
    root: &Path,
    relative_path: &str,
    field: &str,
) -> ChatResult<fs::Permissions> {
    let parent = secure_workspace_parent(root, relative_path).map_err(|_| {
        ChatError::validation(field, "Workspace file parent is unavailable or symbolic")
    })?;
    open_regular_file_at(&parent)
        .and_then(|file| file.metadata())
        .map(|metadata| metadata.permissions())
        .map_err(|_| ChatError::validation(field, "Workspace path is not a regular file"))
}

#[cfg(windows)]
fn workspace_regular_file_permissions(
    root: &Path,
    relative_path: &str,
    field: &str,
) -> ChatResult<fs::Permissions> {
    let (_parents, file) = windows_open_regular_file(root, relative_path)
        .map_err(|_| ChatError::validation(field, "Workspace path is not a regular file"))?;
    file.metadata()
        .map(|metadata| metadata.permissions())
        .map_err(|_| ChatError::validation(field, "Workspace path is not a regular file"))
}

#[cfg(not(any(unix, windows)))]
fn workspace_regular_file_permissions(
    root: &Path,
    relative_path: &str,
    field: &str,
) -> ChatResult<fs::Permissions> {
    if workspace_path_contains_symlink(root, relative_path)? {
        return Err(ChatError::validation(
            field,
            "Workspace path cannot be symbolic",
        ));
    }
    let requested = root.join(relative_path);
    let metadata = fs::symlink_metadata(&requested)
        .map_err(|_| ChatError::validation(field, "Workspace path is unavailable"))?;
    let canonical = fs::canonicalize(&requested)
        .map_err(|_| ChatError::validation(field, "Workspace path is unavailable"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || !canonical.starts_with(root) {
        return Err(ChatError::validation(
            field,
            "Workspace path is not a regular file",
        ));
    }
    Ok(metadata.permissions())
}

#[cfg(unix)]
fn create_workspace_file_exclusively(
    root: &Path,
    relative_path: &str,
    contents: &str,
    permissions: Option<fs::Permissions>,
    conflict_message: &str,
) -> ChatResult<()> {
    let parent =
        secure_workspace_parent(root, relative_path).map_err(|_| workspace_file_write_error())?;
    let mode = permissions.map_or(0o600, |value| value.mode() & 0o777);
    let mut file = create_file_at(&parent, &parent.file_name, mode).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            ChatError::new(ChatErrorCode::Conflict, conflict_message, true)
        } else {
            workspace_file_write_error()
        }
    })?;
    if file
        .write_all(contents.as_bytes())
        .and_then(|_| file.sync_all())
        .is_err()
    {
        drop(file);
        return if unlink_at(&parent, &parent.file_name).is_ok() {
            Err(workspace_file_write_error())
        } else {
            Err(workspace_file_recovery_error())
        };
    }
    Ok(())
}

#[cfg(windows)]
fn create_workspace_file_exclusively(
    root: &Path,
    relative_path: &str,
    contents: &str,
    permissions: Option<fs::Permissions>,
    conflict_message: &str,
) -> ChatResult<()> {
    use std::os::windows::fs::OpenOptionsExt;

    let relative = Path::new(relative_path);
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent_relative = parent_relative
        .to_str()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is unsupported"))?;
    let (parent, _handles) = windows_workspace_directory_chain(root, parent_relative)?;
    let target = parent.join(
        relative
            .file_name()
            .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is invalid"))?,
    );
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);
    let mut file = options.open(&target).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            ChatError::new(ChatErrorCode::Conflict, conflict_message, true)
        } else {
            workspace_file_write_error()
        }
    })?;
    if file
        .write_all(contents.as_bytes())
        .and_then(|_| file.sync_all())
        .is_err()
    {
        drop(file);
        if fs::remove_file(&target).is_err() {
            return Err(workspace_file_recovery_error());
        }
        return Err(workspace_file_write_error());
    }
    if let Some(permissions) = permissions {
        if file.set_permissions(permissions).is_err() {
            drop(file);
            if fs::remove_file(&target).is_err() {
                return Err(workspace_file_recovery_error());
            }
            return Err(workspace_file_write_error());
        }
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn create_workspace_file_exclusively(
    root: &Path,
    relative_path: &str,
    contents: &str,
    permissions: Option<fs::Permissions>,
    conflict_message: &str,
) -> ChatResult<()> {
    if workspace_parent_contains_symlink(root, relative_path)? {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file parent cannot be symbolic",
            false,
        ));
    }
    let relative = Path::new(relative_path);
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent = if parent_relative.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        fs::canonicalize(root.join(parent_relative)).map_err(|_| workspace_file_write_error())?
    };
    if !parent.starts_with(root) || !parent.is_dir() {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file path resolves outside the working folder",
            false,
        ));
    }
    let target = parent.join(
        relative
            .file_name()
            .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is invalid"))?,
    );
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                ChatError::new(ChatErrorCode::Conflict, conflict_message, true)
            } else {
                workspace_file_write_error()
            }
        })?;
    if file
        .write_all(contents.as_bytes())
        .and_then(|_| file.sync_all())
        .is_err()
    {
        drop(file);
        let _ = fs::remove_file(&target);
        return Err(workspace_file_write_error());
    }
    if let Some(permissions) = permissions {
        if fs::set_permissions(&target, permissions).is_err() {
            drop(file);
            let _ = fs::remove_file(&target);
            return Err(workspace_file_write_error());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn write_workspace_text_atomically(
    root: &Path,
    relative_path: &str,
    contents: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    let parent =
        secure_workspace_parent(root, relative_path).map_err(|_| workspace_file_write_error())?;
    let mut current = open_regular_file_at(&parent).map_err(|_| workspace_file_write_error())?;
    let (current_revision, _) = revision_and_permissions(&mut current, relative_path)?;
    if current_revision != expected_revision {
        return Err(stale_workspace_file_error());
    }
    let current_metadata = current
        .metadata()
        .map_err(|_| workspace_file_write_error())?;
    let current_identity = (current_metadata.dev(), current_metadata.ino());

    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary_name = CString::new(format!(
        ".ganbaru.{}.{}.{}.tmp",
        std::process::id(),
        generation,
        nonce,
    ))
    .map_err(|_| workspace_file_write_error())?;
    let mut replacement = create_file_at(&parent, &temporary_name, 0o600)
        .map_err(|_| workspace_file_write_error())?;
    let write_result = replacement
        .write_all(contents.as_bytes())
        .and_then(|_| replacement.sync_all());
    if write_result.is_err() {
        drop(replacement);
        return if unlink_at(&parent, &temporary_name).is_ok() {
            Err(workspace_file_write_error())
        } else {
            Err(workspace_file_recovery_error())
        };
    }

    if exchange_at(&parent, &temporary_name, &parent.file_name).is_err() {
        drop(replacement);
        return if unlink_at(&parent, &temporary_name).is_ok() {
            Err(workspace_file_write_error())
        } else {
            Err(workspace_file_recovery_error())
        };
    }
    let commit = (|| {
        let mut displaced = {
            let descriptor = unsafe {
                libc::openat(
                    parent.directory.as_raw_fd(),
                    temporary_name.as_ptr(),
                    libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_RDONLY,
                )
            };
            if descriptor < 0 {
                return Err(workspace_file_write_error());
            }
            unsafe { File::from_raw_fd(descriptor) }
        };
        let displaced_metadata = displaced
            .metadata()
            .map_err(|_| workspace_file_write_error())?;
        if (displaced_metadata.dev(), displaced_metadata.ino()) != current_identity {
            return Err(stale_workspace_file_error());
        }
        let (displaced_revision, displaced_permissions) =
            revision_and_permissions(&mut displaced, relative_path)?;
        if displaced_revision != expected_revision {
            return Err(stale_workspace_file_error());
        }
        if unsafe {
            libc::fchmod(
                replacement.as_raw_fd(),
                (displaced_permissions.mode() & 0o777) as libc::mode_t,
            )
        } != 0
        {
            return Err(workspace_file_write_error());
        }
        replacement
            .sync_all()
            .map_err(|_| workspace_file_write_error())?;
        unlink_at(&parent, &temporary_name).map_err(|_| workspace_file_write_error())
    })();
    if let Err(error) = commit {
        if exchange_at(&parent, &temporary_name, &parent.file_name).is_ok() {
            return if unlink_at(&parent, &temporary_name).is_ok() {
                Err(error)
            } else {
                Err(workspace_file_recovery_error())
            };
        }
        return Err(workspace_file_recovery_error());
    }
    Ok(())
}

#[cfg(windows)]
fn write_workspace_text_atomically(
    root: &Path,
    relative_path: &str,
    contents: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    use std::os::windows::fs::OpenOptionsExt;

    let relative = Path::new(relative_path);
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent_relative = parent_relative
        .to_str()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is unsupported"))?;
    let (parent, _parent_handles) = windows_workspace_directory_chain(root, parent_relative)?;
    let target = parent.join(
        relative
            .file_name()
            .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is invalid"))?,
    );
    let (_current_parents, mut current) = windows_open_regular_file(root, relative_path)?;
    let (current_revision, permissions) = revision_and_permissions(&mut current, relative_path)?;
    if current_revision != expected_revision {
        return Err(stale_workspace_file_error());
    }

    let (temporary, backup, recovery) = windows_replacement_paths(&parent);
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);
    let mut replacement = options
        .open(&temporary)
        .map_err(|_| workspace_file_write_error())?;
    let prepared = replacement
        .write_all(contents.as_bytes())
        .and_then(|_| replacement.set_permissions(permissions))
        .and_then(|_| replacement.sync_all());
    drop(replacement);
    if prepared.is_err() {
        if fs::remove_file(&temporary).is_err() {
            return Err(workspace_file_recovery_error());
        }
        return Err(workspace_file_write_error());
    }
    drop(current);

    if windows_replace_file(&target, &temporary, &backup).is_err() {
        // ReplaceFileW can move the original to its backup before reporting failure.
        // Restore that documented partial state only after verifying the original bytes.
        let target_exists = fs::symlink_metadata(&target).is_ok();
        let backup_is_original = windows_revision_for_path(&backup, relative_path)
            .is_ok_and(|revision| revision == expected_revision);
        if !target_exists && backup_is_original {
            if fs::rename(&backup, &target).is_err() {
                return Err(workspace_file_recovery_error());
            }
        } else if !target_exists || fs::symlink_metadata(&backup).is_ok() {
            return Err(workspace_file_recovery_error());
        }
        if fs::remove_file(&temporary).is_err() {
            return Err(workspace_file_recovery_error());
        }
        return Err(workspace_file_write_error());
    }

    let displaced_revision = windows_revision_for_path(&backup, relative_path);
    if !displaced_revision
        .as_ref()
        .is_ok_and(|revision| revision == expected_revision)
    {
        if windows_replace_file(&target, &backup, &recovery).is_err() {
            return Err(workspace_file_recovery_error());
        }
        if fs::remove_file(&recovery).is_err() {
            return Err(workspace_file_recovery_error());
        }
        return Err(stale_workspace_file_error());
    }
    fs::remove_file(&backup).map_err(|_| workspace_file_recovery_error())
}

#[cfg(windows)]
fn windows_replacement_paths(parent: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stem = format!(".ganbaru.{}.{}.{}", std::process::id(), generation, nonce);
    (
        parent.join(format!("{stem}.tmp")),
        parent.join(format!("{stem}.backup")),
        parent.join(format!("{stem}.recovery")),
    )
}

#[cfg(windows)]
fn windows_replace_file(target: &Path, replacement: &Path, backup: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{ReplaceFileW, REPLACE_FILE_FLAGS};

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let target = wide(target);
    let replacement = wide(replacement);
    let backup = wide(backup);
    unsafe {
        ReplaceFileW(
            PCWSTR(target.as_ptr()),
            PCWSTR(replacement.as_ptr()),
            PCWSTR(backup.as_ptr()),
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )
    }
    .map_err(|error| std::io::Error::other(error.to_string()))
}

#[cfg(windows)]
fn windows_revision_for_path(path: &Path, relative_path: &str) -> ChatResult<String> {
    use std::os::windows::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options
        .read(true)
        .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0);
    let mut file = options
        .open(path)
        .map_err(|_| workspace_file_recovery_error())?;
    let metadata = file
        .metadata()
        .map_err(|_| workspace_file_recovery_error())?;
    if !metadata.is_file() || windows_metadata_is_reparse(&metadata) {
        return Err(workspace_file_recovery_error());
    }
    revision_and_permissions(&mut file, relative_path).map(|(revision, _)| revision)
}

#[cfg(not(any(unix, windows)))]
fn write_workspace_text_atomically(
    root: &Path,
    relative_path: &str,
    contents: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    if workspace_path_contains_symlink(root, relative_path)? {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file path cannot be symbolic",
            false,
        ));
    }
    let requested = root.join(relative_path);
    let metadata = fs::symlink_metadata(&requested).map_err(|_| workspace_file_write_error())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path is not an editable regular file",
        ));
    }
    let path = fs::canonicalize(&requested).map_err(|_| workspace_file_write_error())?;
    if !path.starts_with(root) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace file path resolves outside the working folder",
            false,
        ));
    }
    let parent = path.parent().ok_or_else(workspace_file_write_error)?;
    let file_name = path
        .file_name()
        .ok_or_else(workspace_file_write_error)?
        .to_string_lossy();
    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".{file_name}.ganbaru.{}.{}.tmp",
        std::process::id(),
        generation
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| workspace_file_write_error())?;
    let result = (|| {
        file.write_all(contents.as_bytes())
            .map_err(|_| workspace_file_write_error())?;
        file.sync_all().map_err(|_| workspace_file_write_error())?;
        let current = fs::read(&path).map_err(|_| workspace_file_write_error())?;
        if workspace_file_revision(relative_path, &current) != expected_revision {
            return Err(stale_workspace_file_error());
        }
        fs::set_permissions(&temporary, metadata.permissions())
            .map_err(|_| workspace_file_write_error())?;
        replace_workspace_file(&temporary, &path, relative_path, expected_revision)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(any(unix, windows)))]
fn replace_workspace_file(
    temporary: &Path,
    target: &Path,
    relative_path: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let backup = target.with_extension(format!(
        "ganbaru.{}.{}.backup",
        std::process::id(),
        generation
    ));
    fs::rename(target, &backup).map_err(|_| workspace_file_write_error())?;
    if fs::rename(temporary, target).is_err() {
        return if fs::rename(&backup, target).is_ok() {
            Err(workspace_file_write_error())
        } else {
            Err(workspace_file_recovery_error())
        };
    }
    let displaced = fs::read(&backup).map_err(|_| workspace_file_write_error())?;
    if workspace_file_revision(relative_path, &displaced) != expected_revision {
        if fs::remove_file(target).is_ok() && fs::rename(&backup, target).is_ok() {
            return Err(stale_workspace_file_error());
        }
        return Err(workspace_file_recovery_error());
    }
    fs::remove_file(backup).map_err(|_| workspace_file_write_error())
}

fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be read",
        true,
    )
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be valid")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ganbaru-chat-file-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self(fs::canonicalize(path).expect("test path should canonicalize"))
        }

        fn authorized(&self, kind: RepositoryKind) -> AuthorizedWorkingFolder {
            AuthorizedWorkingFolder {
                working_folder_id: ProjectWorkingFolderId::new("workspace:file-test")
                    .expect("workspace ID should be valid"),
                canonical_path: self.0.clone(),
                repository_kind: kind,
                repository_identity: None,
            }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn listing_is_on_demand_and_respects_common_and_git_ignores() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("visible.txt"), "visible\n").expect("file should write");
        fs::write(directory.0.join("ignored.log"), "ignored\n").expect("file should write");
        fs::write(directory.0.join(".gitignore"), "ignored.log\n")
            .expect("ignore file should write");
        fs::create_dir(directory.0.join("node_modules")).expect("ignored directory should exist");
        let status = Command::new("git")
            .arg("-C")
            .arg(&directory.0)
            .args(["init", "-q"])
            .status()
            .expect("Git should start");
        assert!(status.success());
        let authorized = directory.authorized(RepositoryKind::Git);

        let visible =
            list_workspace_directory(&authorized, "", false).expect("listing should succeed");
        assert!(visible
            .entries
            .iter()
            .any(|entry| entry.relative_path == "visible.txt"));
        assert!(!visible
            .entries
            .iter()
            .any(|entry| entry.relative_path == "ignored.log"));
        assert!(!visible
            .entries
            .iter()
            .any(|entry| entry.relative_path == "node_modules"));
        let all = list_workspace_directory(&authorized, "", true)
            .expect("listing with ignored files should succeed");
        assert!(all
            .entries
            .iter()
            .any(|entry| entry.relative_path == "ignored.log" && entry.ignored));
        assert!(all
            .entries
            .iter()
            .any(|entry| entry.relative_path == "node_modules" && entry.ignored));
    }

    #[test]
    fn preview_bounds_text_and_rejects_binary_traversal_and_symlinks() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("sample.rs"), "fn main() {}\n").expect("file should write");
        fs::write(directory.0.join("binary.bin"), [0, 1, 2]).expect("binary should write");
        fs::write(
            directory.0.join("large.txt"),
            vec![b'a'; MAX_PREVIEW_BYTES as usize + 1],
        )
        .expect("large file should write");
        let authorized = directory.authorized(RepositoryKind::None);

        let text = preview_workspace_file(&authorized, "sample.rs").expect("text should preview");
        assert_eq!(text.line_count, Some(1));
        assert!(text.content_revision.is_some());
        assert!(
            preview_workspace_file(&authorized, "binary.bin")
                .expect("binary metadata should read")
                .binary
        );
        assert!(
            preview_workspace_file(&authorized, "large.txt")
                .expect("large metadata should read")
                .oversized
        );
        assert!(preview_workspace_file(&authorized, "../outside.txt").is_err());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(directory.0.join("sample.rs"), directory.0.join("link.rs"))
                .expect("symlink should be created");
            assert!(preview_workspace_file(&authorized, "link.rs").is_err());
        }
    }

    #[test]
    fn save_requires_the_current_revision_and_preserves_file_permissions() {
        let directory = TestDirectory::new();
        let path = directory.0.join("sample.rs");
        fs::write(&path, "fn before() {}\n").expect("file should write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o640))
                .expect("permissions should update");
        }
        let authorized = directory.authorized(RepositoryKind::None);
        let original =
            preview_workspace_file(&authorized, "sample.rs").expect("text should preview");
        let revision = original
            .content_revision
            .as_deref()
            .expect("text should have a revision");

        let saved = save_workspace_file(&authorized, "sample.rs", "fn after() {}\n", revision)
            .expect("matching revision should save");
        assert_eq!(saved.text.as_deref(), Some("fn after() {}\n"));
        assert_ne!(saved.content_revision, original.content_revision);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path)
                    .expect("metadata should read")
                    .permissions()
                    .mode()
                    & 0o777,
                0o640
            );
        }

        let error = save_workspace_file(&authorized, "sample.rs", "fn stale() {}\n", revision)
            .expect_err("stale revision should conflict");
        assert_eq!(error.code, ChatErrorCode::Conflict);
        assert_eq!(
            fs::read_to_string(path).expect("saved file should read"),
            "fn after() {}\n"
        );
    }

    #[test]
    fn save_copy_creates_a_new_file_without_overwriting_an_existing_target() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("sample.rs"), "fn original() {}\n")
            .expect("source file should write");
        let authorized = directory.authorized(RepositoryKind::None);

        let copy = save_workspace_file_copy(
            &authorized,
            "sample.rs",
            "sample.ganbaru-copy.rs",
            "fn local_edit() {}\n",
        )
        .expect("separate copy should save");
        assert_eq!(copy.text.as_deref(), Some("fn local_edit() {}\n"));
        assert_eq!(
            fs::read_to_string(directory.0.join("sample.rs")).expect("source should read"),
            "fn original() {}\n"
        );
        let error = save_workspace_file_copy(
            &authorized,
            "sample.rs",
            "sample.ganbaru-copy.rs",
            "fn overwritten() {}\n",
        )
        .expect_err("an existing copy must not be overwritten");
        assert_eq!(error.code, ChatErrorCode::Conflict);
    }

    #[test]
    fn recreate_requires_confirmation_and_never_overwrites_a_reappeared_file() {
        let directory = TestDirectory::new();
        let authorized = directory.authorized(RepositoryKind::None);

        assert!(
            recreate_workspace_file(&authorized, "restored.txt", "preserved\n", false).is_err()
        );
        let recreated = recreate_workspace_file(&authorized, "restored.txt", "preserved\n", true)
            .expect("confirmed recreation should succeed");
        assert_eq!(recreated.text.as_deref(), Some("preserved\n"));
        let conflict = recreate_workspace_file(&authorized, "restored.txt", "overwrite\n", true)
            .expect_err("recreation must not overwrite an existing file");
        assert_eq!(conflict.code, ChatErrorCode::Conflict);
        assert_eq!(
            fs::read_to_string(directory.0.join("restored.txt"))
                .expect("recreated file should read"),
            "preserved\n"
        );
    }

    #[test]
    fn save_rejects_binary_oversized_excluded_and_symbolic_files() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("binary.bin"), [0, 1, 2]).expect("binary should write");
        fs::create_dir(directory.0.join(".git")).expect("excluded directory should exist");
        fs::write(directory.0.join(".git/config"), "config\n").expect("excluded file should write");
        let authorized = directory.authorized(RepositoryKind::None);

        assert!(save_workspace_file(&authorized, "binary.bin", "text", "missing").is_err());
        assert!(save_workspace_file(&authorized, ".git/config", "text", "missing").is_err());
        assert!(save_workspace_file(
            &authorized,
            "binary.bin",
            &"x".repeat(MAX_PREVIEW_BYTES as usize + 1),
            "missing",
        )
        .is_err());

        #[cfg(unix)]
        {
            fs::write(directory.0.join("target.txt"), "target\n").expect("target should write");
            std::os::unix::fs::symlink(
                directory.0.join("target.txt"),
                directory.0.join("link.txt"),
            )
            .expect("symlink should be created");
            assert!(save_workspace_file(&authorized, "link.txt", "changed\n", "missing").is_err());

            fs::create_dir(directory.0.join("real-parent")).expect("real parent should be created");
            fs::write(directory.0.join("real-parent/nested.txt"), "nested\n")
                .expect("nested file should write");
            std::os::unix::fs::symlink(
                directory.0.join("real-parent"),
                directory.0.join("linked-parent"),
            )
            .expect("parent symlink should be created");
            assert!(preview_workspace_file(&authorized, "linked-parent/nested.txt").is_err());
            assert!(list_workspace_directory(&authorized, "linked-parent", true).is_err());
            assert!(save_workspace_file(
                &authorized,
                "linked-parent/nested.txt",
                "changed\n",
                "missing",
            )
            .is_err());
            assert!(
                recreate_workspace_file(&authorized, "linked-parent/new.txt", "new\n", true,)
                    .is_err()
            );
            assert_eq!(
                fs::read_to_string(directory.0.join("real-parent/nested.txt"))
                    .expect("nested file should remain readable"),
                "nested\n"
            );
            assert!(!directory.0.join("real-parent/new.txt").exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn descriptor_relative_file_open_does_not_follow_a_replaced_parent_path() {
        let directory = TestDirectory::new();
        let original = directory.0.join("original");
        let replacement = directory.0.join("replacement");
        fs::create_dir(&original).expect("original directory should be created");
        fs::create_dir(&replacement).expect("replacement directory should be created");
        fs::write(original.join("sample.txt"), "original\n")
            .expect("original file should be written");
        fs::write(replacement.join("sample.txt"), "replacement\n")
            .expect("replacement file should be written");

        let parent = secure_workspace_parent(&directory.0, "original/sample.txt")
            .expect("secure parent should open");
        let moved = directory.0.join("moved-original");
        fs::rename(&original, &moved).expect("original directory should move");
        std::os::unix::fs::symlink(&replacement, &original)
            .expect("replacement symlink should be created");

        let mut file = open_regular_file_at(&parent).expect("descriptor-relative file should open");
        let mut text = String::new();
        file.read_to_string(&mut text)
            .expect("descriptor-relative file should read");
        assert_eq!(text, "original\n");
    }
}
