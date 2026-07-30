//! Project-owned working folders and device-local binding authorization.

use super::models::{
    ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId, RepositoryKind, UtcTimestamp,
};
use crate::projects::working_folders::{
    read_active_working_folder_scope, update_active_working_folder_scope,
    ProjectWorkingFolderBindingState, WorkingFolderDeviceScope,
};
use crate::vault;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tauri::Runtime;

const MAX_GIT_CONFIG_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingFolderBindingStatus {
    Unbound,
    Available,
    Missing,
    RepositoryMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingFolderAuthorizationOperation {
    ProviderStart,
    FileRead,
    FileWrite,
    MentionResolution,
    TerminalStart,
    Git,
    Diff,
    Restore,
}

impl WorkingFolderAuthorizationOperation {
    pub const ALL: [Self; 8] = [
        Self::ProviderStart,
        Self::FileRead,
        Self::FileWrite,
        Self::MentionResolution,
        Self::TerminalStart,
        Self::Git,
        Self::Diff,
        Self::Restore,
    ];
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolder {
    pub id: ProjectWorkingFolderId,
    pub project_id: String,
    pub display_name: String,
    pub kind: WorkingFolderKind,
    pub managed_relative_path: Option<String>,
    pub sort_order: u64,
    pub repository_kind: RepositoryKind,
    pub repository_identity: Option<String>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub archived_at: Option<UtcTimestamp>,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectWorkingFolderRequest {
    pub id: ProjectWorkingFolderId,
    pub project_id: String,
    pub display_name: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingFolderKind {
    Managed,
    External,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderRead {
    pub working_folder: ProjectWorkingFolder,
    pub binding_status: WorkingFolderBindingStatus,
    pub canonical_path: Option<String>,
    pub last_verified_at: Option<UtcTimestamp>,
    pub current_branch: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryProbe {
    pub kind: RepositoryKind,
    pub identity: Option<String>,
    pub current_branch: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedWorkingFolder {
    pub working_folder_id: ProjectWorkingFolderId,
    pub canonical_path: PathBuf,
    pub repository_kind: RepositoryKind,
    pub repository_identity: Option<String>,
}

pub(crate) fn prepare_workspace_binding(
    workspace: &ProjectWorkingFolder,
    selected_path: &Path,
) -> ChatResult<(RepositoryProbe, ProjectWorkingFolderBindingState)> {
    let canonical_path = canonical_existing_directory(selected_path)?;
    let probe = probe_repository(&canonical_path)?;
    if workspace.repository_identity.is_some() && workspace.repository_identity != probe.identity {
        return Err(repository_mismatch());
    }
    if workspace.repository_kind != RepositoryKind::None && workspace.repository_kind != probe.kind
    {
        return Err(repository_mismatch());
    }
    let binding = ProjectWorkingFolderBindingState {
        canonical_path: path_to_string(&canonical_path)?,
        repository_kind: probe.kind,
        repository_identity: probe.identity.clone(),
        last_verified_at: now_timestamp()?,
    };
    Ok((probe, binding))
}

pub(crate) fn ensure_managed_working_folder_binding<R: Runtime>(
    app: &tauri::AppHandle<R>,
    working_folder: &ProjectWorkingFolder,
) -> ChatResult<()> {
    if working_folder.kind != WorkingFolderKind::Managed {
        return Ok(());
    }
    let existing = read_active_working_folder_scope(app)
        .map_err(device_state_error)?
        .bindings
        .contains_key(&working_folder.id);
    if existing {
        return Ok(());
    }
    let relative_path = working_folder
        .managed_relative_path
        .as_deref()
        .ok_or_else(path_validation_error)?;
    let expected = format!("projects/{}", working_folder.project_id);
    if relative_path != expected {
        return Err(path_validation_error());
    }
    let vault_root = vault::active_vault_path(app).map_err(device_state_error)?;
    let folder_path = vault_root.join(relative_path);
    fs::create_dir_all(&folder_path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Permission,
            "The managed project working folder could not be created",
            true,
        )
    })?;
    let (_, binding) = prepare_workspace_binding(working_folder, &folder_path)?;
    store_active_device_binding(app, &working_folder.id, binding)
}

pub(crate) fn validate_external_folder_outside_vault<R: Runtime>(
    app: &tauri::AppHandle<R>,
    selected_path: &Path,
) -> ChatResult<()> {
    let selected = canonical_existing_directory(selected_path)?;
    let vault_root = vault::active_vault_path(app).map_err(device_state_error)?;
    if paths_overlap(&selected, &vault_root) {
        return Err(ChatError::validation(
            "workingFolderPath",
            "External project working folders cannot overlap the active Ganbaru AI folder",
        ));
    }
    Ok(())
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

pub(crate) fn store_active_device_binding<R: Runtime>(
    app: &tauri::AppHandle<R>,
    working_folder_id: &ProjectWorkingFolderId,
    binding: ProjectWorkingFolderBindingState,
) -> ChatResult<()> {
    update_active_working_folder_scope(app, |scope| {
        scope.bindings.insert(working_folder_id.clone(), binding);
        Ok(())
    })
    .map_err(device_state_error)
}

pub fn authorize_workspace(
    workspace: &ProjectWorkingFolder,
    scope: &WorkingFolderDeviceScope,
    _operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<AuthorizedWorkingFolder> {
    let binding = scope.bindings.get(&workspace.id).ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "This device has no binding for the project working folder",
            true,
        )
    })?;
    let canonical_path = canonical_existing_directory(Path::new(&binding.canonical_path))?;
    if path_to_string(&canonical_path)? != binding.canonical_path {
        return Err(ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "The project working-folder binding is stale and must be located again",
            true,
        ));
    }
    let probe = probe_repository(&canonical_path)?;
    if probe.kind != binding.repository_kind
        || probe.identity != binding.repository_identity
        || probe.kind != workspace.repository_kind
        || probe.identity != workspace.repository_identity
    {
        return Err(repository_mismatch());
    }
    Ok(AuthorizedWorkingFolder {
        working_folder_id: workspace.id.clone(),
        canonical_path,
        repository_kind: probe.kind,
        repository_identity: probe.identity,
    })
}

pub fn resolve_workspace_relative_path(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
) -> ChatResult<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.as_os_str().is_empty() || relative.is_absolute() {
        return Err(path_validation_error());
    }
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(path_validation_error());
        }
    }
    let resolved = fs::canonicalize(authorized.canonical_path.join(relative)).map_err(|_| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Workspace path does not exist",
            true,
        )
    })?;
    if !resolved.starts_with(&authorized.canonical_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace path resolves outside the bound folder",
            false,
        ));
    }
    Ok(resolved)
}

pub fn remove_active_device_binding<R: Runtime>(
    app: &tauri::AppHandle<R>,
    working_folder_id: &ProjectWorkingFolderId,
) -> ChatResult<()> {
    update_active_working_folder_scope(app, |scope| {
        scope.bindings.remove(working_folder_id);
        Ok(())
    })
    .map_err(device_state_error)
}

pub(crate) fn workspace_read(
    workspace: ProjectWorkingFolder,
    scope: &WorkingFolderDeviceScope,
) -> ChatResult<ProjectWorkingFolderRead> {
    let binding = scope.bindings.get(&workspace.id);
    let (binding_status, current_branch) = match binding {
        None => (WorkingFolderBindingStatus::Unbound, None),
        Some(binding) => match canonical_existing_directory(Path::new(&binding.canonical_path)) {
            Err(_) => (WorkingFolderBindingStatus::Missing, None),
            Ok(path) => match probe_repository(&path) {
                Ok(probe)
                    if probe.kind == workspace.repository_kind
                        && probe.identity == workspace.repository_identity
                        && probe.kind == binding.repository_kind
                        && probe.identity == binding.repository_identity =>
                {
                    (WorkingFolderBindingStatus::Available, probe.current_branch)
                }
                _ => (WorkingFolderBindingStatus::RepositoryMismatch, None),
            },
        },
    };
    Ok(ProjectWorkingFolderRead {
        working_folder: workspace,
        binding_status,
        canonical_path: binding.map(|value| value.canonical_path.clone()),
        last_verified_at: binding.map(|value| value.last_verified_at.clone()),
        current_branch,
    })
}

fn canonical_existing_directory(path: &Path) -> ChatResult<PathBuf> {
    if !path.is_absolute() {
        return Err(ChatError::validation(
            "workspacePath",
            "Project working-folder path must be absolute",
        ));
    }
    let metadata = fs::metadata(path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Project working folder is missing or unreadable",
            true,
        )
    })?;
    if !metadata.is_dir() {
        return Err(ChatError::validation(
            "workspacePath",
            "Project working-folder path must be a directory",
        ));
    }
    fs::canonicalize(path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Permission,
            "Project working folder could not be canonicalized",
            true,
        )
    })
}

pub(crate) fn probe_repository(path: &Path) -> ChatResult<RepositoryProbe> {
    let git_entry = path.join(".git");
    let metadata = match fs::symlink_metadata(&git_entry) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RepositoryProbe {
                kind: RepositoryKind::None,
                identity: None,
                current_branch: None,
            });
        }
        Err(_) => {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Repository metadata is unreadable",
                true,
            ));
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Repository metadata cannot be a symbolic link",
            false,
        ));
    }
    let git_directory = if metadata.is_dir() {
        fs::canonicalize(&git_entry).map_err(|_| repository_probe_error())?
    } else if metadata.is_file() {
        resolve_git_directory_file(path, &git_entry)?
    } else {
        return Err(repository_probe_error());
    };
    let config_path = git_directory.join("config");
    let config_metadata = fs::metadata(&config_path).map_err(|_| repository_probe_error())?;
    if !config_metadata.is_file() || config_metadata.len() > MAX_GIT_CONFIG_BYTES {
        return Err(repository_probe_error());
    }
    let config = fs::read_to_string(&config_path).map_err(|_| repository_probe_error())?;
    let current_branch = read_current_branch(&git_directory)?;
    let remote = origin_remote(&config).map(normalize_remote_identity);
    let mut hasher = Sha256::new();
    hasher.update(b"ganbaru-chat-repository-v1\0");
    match remote.filter(|value| !value.is_empty()) {
        Some(remote) => hasher.update(remote.as_bytes()),
        None => {
            hasher.update(b"device-local\0");
            hasher.update(path.as_os_str().as_encoded_bytes());
        }
    }
    Ok(RepositoryProbe {
        kind: RepositoryKind::Git,
        identity: Some(format!("git-sha256:{}", hex_digest(hasher.finalize()))),
        current_branch,
    })
}

fn read_current_branch(git_directory: &Path) -> ChatResult<Option<String>> {
    let head_path = git_directory.join("HEAD");
    let metadata = fs::metadata(&head_path).map_err(|_| repository_probe_error())?;
    if !metadata.is_file() || metadata.len() > 4_096 {
        return Err(repository_probe_error());
    }
    let head = fs::read_to_string(head_path).map_err(|_| repository_probe_error())?;
    let Some(reference) = head.trim().strip_prefix("ref: refs/heads/") else {
        return Ok(None);
    };
    if reference.is_empty() || reference.chars().any(char::is_control) {
        return Err(repository_probe_error());
    }
    Ok(Some(reference.to_string()))
}

fn resolve_git_directory_file(worktree: &Path, git_file: &Path) -> ChatResult<PathBuf> {
    let metadata = fs::metadata(git_file).map_err(|_| repository_probe_error())?;
    if metadata.len() > 4_096 {
        return Err(repository_probe_error());
    }
    let contents = fs::read_to_string(git_file).map_err(|_| repository_probe_error())?;
    let raw_path = contents
        .trim()
        .strip_prefix("gitdir:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(repository_probe_error)?;
    let path = Path::new(raw_path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        worktree.join(path)
    };
    fs::canonicalize(path).map_err(|_| repository_probe_error())
}

fn origin_remote(config: &str) -> Option<&str> {
    let mut in_origin = false;
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed.eq_ignore_ascii_case("[remote \"origin\"]");
            continue;
        }
        if in_origin {
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            if key.trim().eq_ignore_ascii_case("url") {
                return Some(value.trim());
            }
        }
    }
    None
}

fn normalize_remote_identity(remote: &str) -> String {
    let trimmed = remote.trim();
    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, remainder)| remainder)
        .unwrap_or(trimmed);
    let without_user = without_scheme
        .rsplit_once('@')
        .map(|(_, remainder)| remainder)
        .unwrap_or(without_scheme);
    without_user
        .replace(':', "/")
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_ascii_lowercase()
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write;
    bytes
        .as_ref()
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        })
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}

fn path_to_string(path: &Path) -> ChatResult<String> {
    path.to_str().map(ToOwned::to_owned).ok_or_else(|| {
        ChatError::validation(
            "workspacePath",
            "Project working-folder path contains unsupported characters",
        )
    })
}

fn repository_mismatch() -> ChatError {
    ChatError::new(
        ChatErrorCode::ConfigurationInvalid,
        "The selected folder belongs to a different repository. Rebind the project working folder or add it separately.",
        true,
    )
}

fn repository_probe_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::ConfigurationInvalid,
        "Git repository metadata is invalid or unreadable",
        true,
    )
}

fn path_validation_error() -> ChatError {
    ChatError::validation(
        "relativePath",
        "Workspace paths must be normalized relative paths",
    )
}

fn device_state_error(_error: String) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be updated",
        true,
    )
}

pub fn open_authorized_workspace(authorized: &AuthorizedWorkingFolder) -> ChatResult<()> {
    open_authorized_path(authorized, &authorized.canonical_path)
}

pub fn open_authorized_path(authorized: &AuthorizedWorkingFolder, path: &Path) -> ChatResult<()> {
    if !path.starts_with(&authorized.canonical_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Workspace path resolves outside the bound folder",
            false,
        ));
    }
    spawn_file_manager(path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Internal,
            "The project working-folder path could not be opened",
            true,
        )
    })
}

#[cfg(target_os = "linux")]
fn spawn_file_manager(path: &Path) -> std::io::Result<()> {
    spawn_file_manager_command("xdg-open", [path.as_os_str()])
}

#[cfg(target_os = "macos")]
fn spawn_file_manager(path: &Path) -> std::io::Result<()> {
    spawn_file_manager_command("open", [path.as_os_str()])
}

#[cfg(windows)]
fn spawn_file_manager(path: &Path) -> std::io::Result<()> {
    spawn_file_manager_command("explorer.exe", [path.as_os_str()])
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn spawn_file_manager(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "platform is unsupported",
    ))
}

fn spawn_file_manager_command<I, S>(program: &str, arguments: I) -> std::io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    std::process::Command::new(program)
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod overlap_tests {
    use super::paths_overlap;
    use std::path::Path;

    #[test]
    fn external_folder_overlap_rejects_vault_ancestors_descendants_and_identity() {
        let vault = Path::new("data").join("Ganbaru AI");
        let vault = vault.as_path();
        assert!(paths_overlap(vault, vault));
        assert!(paths_overlap(Path::new("data"), vault));
        assert!(paths_overlap(&vault.join("projects").join("repo"), vault));
        assert!(!paths_overlap(Path::new("work/repo"), vault));
    }
}
