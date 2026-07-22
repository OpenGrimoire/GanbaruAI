//! Bounded, workspace-authorized file browsing and preview commands.

use super::device_state::read_active_device_scope;
use super::models::{ChatError, ChatErrorCode, ChatResult, ChatWorkspaceId, RepositoryKind};
use super::repository::workspaces;
use super::workspace::{
    authorize_workspace, resolve_workspace_relative_path, AuthorizedWorkspace,
    WorkspaceAuthorizationOperation,
};
use crate::db_path;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Component, Path};
use std::process::{Command, Stdio};

const MAX_DIRECTORY_ENTRIES: usize = 5_000;
const MAX_PREVIEW_BYTES: u64 = 1024 * 1024;
const MAX_RELATIVE_PATH_BYTES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkspaceFileEntry {
    pub relative_path: String,
    pub display_name: String,
    pub kind: String,
    pub ignored: bool,
    pub byte_size: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkspaceDirectoryRead {
    pub relative_path: String,
    pub entries: Vec<ChatWorkspaceFileEntry>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatWorkspaceFilePreview {
    pub relative_path: String,
    pub display_name: String,
    pub language: Option<String>,
    pub text: Option<String>,
    pub line_count: Option<u64>,
    pub byte_size: u64,
    pub binary: bool,
    pub oversized: bool,
}

#[tauri::command]
pub async fn chat_list_workspace_directory(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    relative_path: String,
    include_ignored: bool,
) -> ChatResult<ChatWorkspaceDirectoryRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &workspace_id).await?;
    list_workspace_directory(&authorized, &relative_path, include_ignored)
}

#[tauri::command]
pub async fn chat_preview_workspace_file(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    relative_path: String,
) -> ChatResult<ChatWorkspaceFilePreview> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &workspace_id).await?;
    preview_workspace_file(&authorized, &relative_path)
}

#[tauri::command]
pub async fn chat_open_workspace_file(
    app: tauri::AppHandle,
    db_url: String,
    workspace_id: ChatWorkspaceId,
    relative_path: String,
) -> ChatResult<()> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(&app, &pool, &workspace_id).await?;
    let path = resolve_workspace_relative_path(&authorized, &relative_path)?;
    super::workspace::open_authorized_path(&authorized, &path)
}

pub fn list_workspace_directory(
    authorized: &AuthorizedWorkspace,
    relative_path: &str,
    include_ignored: bool,
) -> ChatResult<ChatWorkspaceDirectoryRead> {
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
        entries.push(ChatWorkspaceFileEntry {
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
    Ok(ChatWorkspaceDirectoryRead {
        relative_path: relative_path.to_string(),
        entries,
        truncated,
    })
}

pub fn preview_workspace_file(
    authorized: &AuthorizedWorkspace,
    relative_path: &str,
) -> ChatResult<ChatWorkspaceFilePreview> {
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
        return Ok(ChatWorkspaceFilePreview {
            relative_path: relative_path.to_string(),
            display_name,
            language: language_for_path(&path),
            text: None,
            line_count: None,
            byte_size: metadata.len(),
            binary: false,
            oversized: true,
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
    let line_count = text.as_ref().map(|value| {
        if value.is_empty() {
            0
        } else {
            value.lines().count() as u64
        }
    });
    Ok(ChatWorkspaceFilePreview {
        relative_path: relative_path.to_string(),
        display_name,
        language: language_for_path(&path),
        text,
        line_count,
        byte_size: metadata.len(),
        binary,
        oversized: false,
    })
}

async fn require_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    workspace_id: &ChatWorkspaceId,
) -> ChatResult<AuthorizedWorkspace> {
    let workspace = workspaces::read_workspace(pool, workspace_id).await?;
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    authorize_workspace(
        &workspace,
        &scope,
        WorkspaceAuthorizationOperation::FileRead,
    )
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

        fn authorized(&self, kind: RepositoryKind) -> AuthorizedWorkspace {
            AuthorizedWorkspace {
                workspace_id: ChatWorkspaceId::new("workspace:file-test")
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
}
