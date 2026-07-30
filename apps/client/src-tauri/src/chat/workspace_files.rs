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
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

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
    pub language: Option<String>,
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
    save_workspace_file(
        &authorized,
        &request.relative_path,
        &request.contents,
        &request.expected_revision,
    )
}

#[tauri::command]
pub async fn project_save_working_folder_file_copy(
    app: tauri::AppHandle,
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
    save_workspace_file_copy(
        &authorized,
        &request.source_relative_path,
        &request.target_relative_path,
        &request.contents,
    )
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
    let directory = if relative_path.is_empty() {
        authorized.canonical_path.clone()
    } else {
        resolve_workspace_relative_path(authorized, relative_path)?
    };
    if !fs::metadata(&directory).map_err(file_error)?.is_dir() {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path is not a directory",
        ));
    }
    let mut entries = Vec::new();
    let mut truncated = false;
    for entry in fs::read_dir(&directory).map_err(file_error)? {
        let entry = entry.map_err(file_error)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(file_error)?;
        if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
            continue;
        }
        let Some(display_name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
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
            kind: if metadata.is_dir() {
                "directory"
            } else {
                "file"
            }
            .to_string(),
            byte_size: metadata.is_file().then_some(metadata.len()),
        });
        if entries.len() >= MAX_DIRECTORY_ENTRIES {
            truncated = true;
            break;
        }
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
    let requested_path = authorized.canonical_path.join(relative_path);
    let requested_metadata = fs::symlink_metadata(&requested_path).map_err(file_error)?;
    if requested_metadata.file_type().is_symlink() {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace symlinks are not available for Chat preview",
            false,
        ));
    }
    let path = resolve_workspace_relative_path(authorized, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(file_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
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
            language: language_for_path(&path),
            text: None,
            line_count: None,
            byte_size: metadata.len(),
            binary: false,
            oversized: true,
            content_revision: None,
        });
    }
    let bytes = fs::read(&path).map_err(file_error)?;
    if bytes.len() as u64 != metadata.len() {
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
        language: language_for_path(&path),
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
    let path = resolve_workspace_relative_path(authorized, relative_path)?;
    write_workspace_text_atomically(&path, contents)?;
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
    let source_path = resolve_workspace_relative_path(authorized, source_relative_path)?;
    let source_metadata =
        fs::symlink_metadata(&source_path).map_err(|_| workspace_file_write_error())?;
    if source_metadata.file_type().is_symlink() || !source_metadata.is_file() {
        return Err(ChatError::validation(
            "sourceRelativePath",
            "Source workspace path is not a regular file",
        ));
    }
    let target_relative = Path::new(target_relative_path);
    let parent_relative = target_relative.parent().unwrap_or_else(|| Path::new(""));
    let parent = if parent_relative.as_os_str().is_empty() {
        authorized.canonical_path.clone()
    } else {
        fs::canonicalize(authorized.canonical_path.join(parent_relative)).map_err(|_| {
            ChatError::validation(
                "targetRelativePath",
                "Save-copy parent directory does not exist",
            )
        })?
    };
    if !parent.starts_with(&authorized.canonical_path) || !parent.is_dir() {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Save-copy path resolves outside the working folder",
            false,
        ));
    }
    let file_name = target_relative
        .file_name()
        .ok_or_else(|| ChatError::validation("targetRelativePath", "Save-copy path is invalid"))?;
    let target = parent.join(file_name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                ChatError::new(
                    ChatErrorCode::Conflict,
                    "Save-copy target already exists",
                    true,
                )
            } else {
                workspace_file_write_error()
            }
        })?;
    let write_result = file
        .write_all(contents.as_bytes())
        .and_then(|_| file.sync_all());
    if write_result.is_err() {
        drop(file);
        let _ = fs::remove_file(&target);
        return Err(workspace_file_write_error());
    }
    fs::set_permissions(&target, source_metadata.permissions())
        .map_err(|_| workspace_file_write_error())?;
    preview_workspace_file(authorized, target_relative_path)
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

async fn require_workspace_for(
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

fn safety_excluded(relative_path: &str) -> bool {
    super::interaction_commands::workspace_mention_is_safety_excluded(relative_path)
}

fn common_ignored(relative_path: &str) -> bool {
    relative_path.split('/').any(|segment| {
        matches!(
            segment,
            ".git" | ".cache" | ".turbo" | "node_modules" | "target" | "dist" | "build"
        )
    })
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

fn language_for_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let language = match extension.as_str() {
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "go" => "go",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "rs" => "rust",
        "ts" | "tsx" | "mts" | "cts" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "svelte" => "svelte",
        "vue" => "vue",
        "json" => "json",
        "md" | "mdx" => "markdown",
        "css" | "scss" | "sass" | "less" => "css",
        "html" | "htm" | "xml" => "html",
        "py" => "python",
        "rb" => "ruby",
        "swift" => "swift",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        _ => return None,
    };
    Some(language.to_string())
}

fn workspace_file_revision(relative_path: &str, bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"workspace-file-v1\0");
    digest.update(relative_path.as_bytes());
    digest.update(b"\0");
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn write_workspace_text_atomically(path: &Path, contents: &str) -> ChatResult<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| workspace_file_write_error())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ChatError::validation(
            "relativePath",
            "Workspace path is not an editable regular file",
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
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(metadata.permissions().mode());
        }
        let mut file = options
            .open(&temporary)
            .map_err(|_| workspace_file_write_error())?;
        file.write_all(contents.as_bytes())
            .map_err(|_| workspace_file_write_error())?;
        file.sync_all().map_err(|_| workspace_file_write_error())?;
        replace_workspace_file(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(windows))]
fn replace_workspace_file(temporary: &Path, target: &Path) -> ChatResult<()> {
    fs::rename(temporary, target).map_err(|_| workspace_file_write_error())
}

#[cfg(windows)]
fn replace_workspace_file(temporary: &Path, target: &Path) -> ChatResult<()> {
    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let backup = target.with_extension(format!(
        "ganbaru.{}.{}.backup",
        std::process::id(),
        generation
    ));
    fs::rename(target, &backup).map_err(|_| workspace_file_write_error())?;
    if fs::rename(temporary, target).is_err() {
        let _ = fs::rename(&backup, target);
        return Err(workspace_file_write_error());
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

fn file_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "Workspace file could not be read safely",
        true,
    )
}

fn workspace_file_write_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Workspace file could not be saved safely",
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
    fn common_code_extensions_receive_editor_languages() {
        let cases = [
            ("component.tsx", "typescript"),
            ("component.jsx", "javascript"),
            ("styles.scss", "css"),
            ("main.go", "go"),
            ("header.hpp", "cpp"),
            ("view.vue", "vue"),
        ];
        for (path, expected) in cases {
            assert_eq!(
                language_for_path(Path::new(path)).as_deref(),
                Some(expected)
            );
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
        assert_eq!(text.language.as_deref(), Some("rust"));
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
        }
    }
}
