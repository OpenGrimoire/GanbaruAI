//! Logical Chat workspaces and device-local binding authorization.

use super::device_state::{
    read_active_device_scope, update_active_device_scope, ChatDeviceScope,
    ChatWorkspaceBindingState,
};
use super::models::{
    ChatError, ChatErrorCode, ChatResult, ChatWorkspaceId, RepositoryKind, UtcTimestamp,
};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use tauri::Runtime;

const MAX_WORKSPACE_NAME_BYTES: usize = 240;
const MAX_PROJECT_ID_BYTES: usize = 1_024;
const MAX_GIT_CONFIG_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceBindingStatus {
    Unbound,
    Available,
    Missing,
    RepositoryMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceAuthorizationOperation {
    ProviderStart,
    FileRead,
    MentionResolution,
    TerminalStart,
    Diff,
    Restore,
}

impl WorkspaceAuthorizationOperation {
    pub const ALL: [Self; 6] = [
        Self::ProviderStart,
        Self::FileRead,
        Self::MentionResolution,
        Self::TerminalStart,
        Self::Diff,
        Self::Restore,
    ];
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicalChatWorkspace {
    pub id: ChatWorkspaceId,
    pub project_id: Option<String>,
    pub display_name: String,
    pub repository_kind: RepositoryKind,
    pub repository_identity: Option<String>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
    pub archived_at: Option<UtcTimestamp>,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChatWorkspaceRequest {
    pub id: ChatWorkspaceId,
    pub project_id: Option<String>,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkspaceRead {
    pub workspace: LogicalChatWorkspace,
    pub binding_status: WorkspaceBindingStatus,
    pub canonical_path: Option<String>,
    pub last_verified_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryProbe {
    pub kind: RepositoryKind,
    pub identity: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedWorkspace {
    pub workspace_id: ChatWorkspaceId,
    pub canonical_path: PathBuf,
    pub repository_kind: RepositoryKind,
    pub repository_identity: Option<String>,
}

/// Temporary Phase 2 catalog. Phase 3 replaces this implementation with the
/// SQLite repository while retaining the same command and authorization API.
#[derive(Default)]
pub struct ChatWorkspaceCatalogState {
    workspaces: Mutex<BTreeMap<ChatWorkspaceId, LogicalChatWorkspace>>,
}

impl ChatWorkspaceCatalogState {
    pub fn list(&self) -> ChatResult<Vec<LogicalChatWorkspace>> {
        let workspaces = self.lock()?;
        Ok(workspaces.values().cloned().collect())
    }

    pub fn get(&self, id: &ChatWorkspaceId) -> ChatResult<LogicalChatWorkspace> {
        let workspaces = self.lock()?;
        workspaces.get(id).cloned().ok_or_else(|| not_found(id))
    }

    pub fn create(&self, request: CreateChatWorkspaceRequest) -> ChatResult<LogicalChatWorkspace> {
        validate_workspace_name(&request.display_name)?;
        validate_project_id(request.project_id.as_deref())?;
        let mut workspaces = self.lock()?;
        if workspaces.contains_key(&request.id) {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Chat workspace ID already exists",
                true,
            ));
        }
        ensure_unique_name(
            &workspaces,
            None,
            request.project_id.as_deref(),
            &request.display_name,
        )?;
        let now = now_timestamp()?;
        let workspace = LogicalChatWorkspace {
            id: request.id,
            project_id: request.project_id,
            display_name: request.display_name.trim().to_string(),
            repository_kind: RepositoryKind::None,
            repository_identity: None,
            created_at: now.clone(),
            updated_at: now,
            archived_at: None,
            revision: 1,
        };
        workspaces.insert(workspace.id.clone(), workspace.clone());
        Ok(workspace)
    }

    pub fn rename(
        &self,
        id: &ChatWorkspaceId,
        display_name: &str,
    ) -> ChatResult<LogicalChatWorkspace> {
        validate_workspace_name(display_name)?;
        let mut workspaces = self.lock()?;
        let project_id = workspaces
            .get(id)
            .ok_or_else(|| not_found(id))?
            .project_id
            .clone();
        ensure_unique_name(&workspaces, Some(id), project_id.as_deref(), display_name)?;
        let workspace = workspaces.get_mut(id).ok_or_else(|| not_found(id))?;
        workspace.display_name = display_name.trim().to_string();
        touch(workspace)?;
        Ok(workspace.clone())
    }

    pub fn set_archived(
        &self,
        id: &ChatWorkspaceId,
        archived: bool,
    ) -> ChatResult<LogicalChatWorkspace> {
        let mut workspaces = self.lock()?;
        let workspace = workspaces.get_mut(id).ok_or_else(|| not_found(id))?;
        workspace.archived_at = archived.then(now_timestamp).transpose()?;
        touch(workspace)?;
        Ok(workspace.clone())
    }

    fn record_repository(
        &self,
        id: &ChatWorkspaceId,
        probe: &RepositoryProbe,
    ) -> ChatResult<LogicalChatWorkspace> {
        let mut workspaces = self.lock()?;
        let workspace = workspaces.get_mut(id).ok_or_else(|| not_found(id))?;
        if workspace.repository_identity.is_some()
            && workspace.repository_identity != probe.identity
        {
            return Err(repository_mismatch());
        }
        if workspace.repository_kind != RepositoryKind::None
            && workspace.repository_kind != probe.kind
        {
            return Err(repository_mismatch());
        }
        workspace.repository_kind = probe.kind;
        workspace.repository_identity.clone_from(&probe.identity);
        touch(workspace)?;
        Ok(workspace.clone())
    }

    fn lock(
        &self,
    ) -> ChatResult<std::sync::MutexGuard<'_, BTreeMap<ChatWorkspaceId, LogicalChatWorkspace>>>
    {
        self.workspaces.lock().map_err(|_| {
            ChatError::new(
                ChatErrorCode::Internal,
                "Chat workspace catalog is unavailable",
                true,
            )
        })
    }
}

pub fn read_workspaces(
    catalog: &ChatWorkspaceCatalogState,
    scope: &ChatDeviceScope,
) -> ChatResult<Vec<ChatWorkspaceRead>> {
    catalog
        .list()?
        .into_iter()
        .map(|workspace| workspace_read(workspace, scope))
        .collect()
}

pub fn bind_workspace_path<R: Runtime>(
    app: &tauri::AppHandle<R>,
    catalog: &ChatWorkspaceCatalogState,
    workspace_id: &ChatWorkspaceId,
    selected_path: &Path,
) -> ChatResult<ChatWorkspaceRead> {
    let workspace = catalog.get(workspace_id)?;
    let canonical_path = canonical_existing_directory(selected_path)?;
    let probe = probe_repository(&canonical_path)?;
    if workspace.repository_identity.is_some() && workspace.repository_identity != probe.identity {
        return Err(repository_mismatch());
    }
    if workspace.repository_kind != RepositoryKind::None && workspace.repository_kind != probe.kind
    {
        return Err(repository_mismatch());
    }
    let canonical_path_text = path_to_string(&canonical_path)?;
    let binding = ChatWorkspaceBindingState {
        canonical_path: canonical_path_text,
        repository_kind: probe.kind,
        repository_identity: probe.identity.clone(),
        last_verified_at: now_timestamp()?,
    };
    update_active_device_scope(app, |scope| {
        scope
            .workspace_bindings
            .insert(workspace_id.clone(), binding.clone());
        Ok(())
    })
    .map_err(device_state_error)?;
    let workspace = catalog.record_repository(workspace_id, &probe)?;
    workspace_read(
        workspace,
        &read_active_device_scope(app).map_err(device_state_error)?,
    )
}

pub fn authorize_workspace(
    workspace: &LogicalChatWorkspace,
    scope: &ChatDeviceScope,
    _operation: WorkspaceAuthorizationOperation,
) -> ChatResult<AuthorizedWorkspace> {
    let binding = scope.workspace_bindings.get(&workspace.id).ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "This device has no folder binding for the Chat workspace",
            true,
        )
    })?;
    let canonical_path = canonical_existing_directory(Path::new(&binding.canonical_path))?;
    if path_to_string(&canonical_path)? != binding.canonical_path {
        return Err(ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "The Chat workspace binding is stale and must be located again",
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
    Ok(AuthorizedWorkspace {
        workspace_id: workspace.id.clone(),
        canonical_path,
        repository_kind: probe.kind,
        repository_identity: probe.identity,
    })
}

pub fn resolve_workspace_relative_path(
    authorized: &AuthorizedWorkspace,
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
    workspace_id: &ChatWorkspaceId,
) -> ChatResult<()> {
    update_active_device_scope(app, |scope| {
        scope.workspace_bindings.remove(workspace_id);
        Ok(())
    })
    .map_err(device_state_error)
}

fn workspace_read(
    workspace: LogicalChatWorkspace,
    scope: &ChatDeviceScope,
) -> ChatResult<ChatWorkspaceRead> {
    let binding = scope.workspace_bindings.get(&workspace.id);
    let binding_status = match binding {
        None => WorkspaceBindingStatus::Unbound,
        Some(binding) => match canonical_existing_directory(Path::new(&binding.canonical_path)) {
            Err(_) => WorkspaceBindingStatus::Missing,
            Ok(path) => match probe_repository(&path) {
                Ok(probe)
                    if probe.kind == workspace.repository_kind
                        && probe.identity == workspace.repository_identity
                        && probe.kind == binding.repository_kind
                        && probe.identity == binding.repository_identity =>
                {
                    WorkspaceBindingStatus::Available
                }
                _ => WorkspaceBindingStatus::RepositoryMismatch,
            },
        },
    };
    Ok(ChatWorkspaceRead {
        workspace,
        binding_status,
        canonical_path: binding.map(|value| value.canonical_path.clone()),
        last_verified_at: binding.map(|value| value.last_verified_at.clone()),
    })
}

fn canonical_existing_directory(path: &Path) -> ChatResult<PathBuf> {
    if !path.is_absolute() {
        return Err(ChatError::validation(
            "workspacePath",
            "Chat workspace folder must be absolute",
        ));
    }
    let metadata = fs::metadata(path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Chat workspace folder is missing or unreadable",
            true,
        )
    })?;
    if !metadata.is_dir() {
        return Err(ChatError::validation(
            "workspacePath",
            "Chat workspace path must be a directory",
        ));
    }
    fs::canonicalize(path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Permission,
            "Chat workspace folder could not be canonicalized",
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
    })
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

fn ensure_unique_name(
    workspaces: &BTreeMap<ChatWorkspaceId, LogicalChatWorkspace>,
    except_id: Option<&ChatWorkspaceId>,
    project_id: Option<&str>,
    display_name: &str,
) -> ChatResult<()> {
    let normalized = display_name.trim();
    if workspaces.values().any(|workspace| {
        except_id != Some(&workspace.id)
            && workspace.project_id.as_deref() == project_id
            && workspace.display_name.eq_ignore_ascii_case(normalized)
    }) {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "A Chat workspace with this name already exists in the same project context",
            true,
        ));
    }
    Ok(())
}

fn validate_workspace_name(value: &str) -> ChatResult<()> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_WORKSPACE_NAME_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "displayName",
            "Chat workspace name is invalid",
        ));
    }
    Ok(())
}

fn validate_project_id(value: Option<&str>) -> ChatResult<()> {
    if value.is_some_and(|project_id| {
        let project_id = project_id.trim();
        project_id.is_empty()
            || project_id.len() > MAX_PROJECT_ID_BYTES
            || project_id.chars().any(char::is_control)
    }) {
        return Err(ChatError::validation("projectId", "project ID is invalid"));
    }
    Ok(())
}

fn touch(workspace: &mut LogicalChatWorkspace) -> ChatResult<()> {
    workspace.updated_at = now_timestamp()?;
    workspace.revision = workspace.revision.saturating_add(1);
    Ok(())
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
            "Chat workspace path contains unsupported characters",
        )
    })
}

fn not_found(id: &ChatWorkspaceId) -> ChatError {
    ChatError::new(
        ChatErrorCode::NotFound,
        format!("Chat workspace {id} was not found"),
        true,
    )
}

fn repository_mismatch() -> ChatError {
    ChatError::new(
        ChatErrorCode::ConfigurationInvalid,
        "The selected folder belongs to a different repository. Create a new logical workspace for it.",
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

pub fn open_authorized_workspace(authorized: &AuthorizedWorkspace) -> ChatResult<()> {
    spawn_file_manager(&authorized.canonical_path).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Internal,
            "The Chat workspace folder could not be opened",
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
