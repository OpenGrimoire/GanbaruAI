//! Bounded access to Markdown files in project-owned working folders.

use crate::chat::models::{ChatError, ChatErrorCode, ChatResult, ProjectWorkingFolderId};
use crate::chat::repository::workspaces;
use crate::chat::workspace::{
    authorize_workspace, ensure_managed_working_folder_binding, open_authorized_path,
    AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::db_path::connect_sqlite;
use crate::projects::working_folders::read_active_working_folder_scope;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Runtime;

const MAX_SCAN_DEPTH: usize = 16;
const MAX_SCAN_ENTRIES: usize = 5_000;
const MAX_RELATIVE_PATH_BYTES: usize = 4_096;
const MAX_MARKDOWN_FILE_BYTES: u64 = 4 * 1024 * 1024;
const EXCLUDED_DIRECTORIES: &[&str] = &[
    ".git",
    ".cache",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "vendor",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotesWorkingMarkdownNodeKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesWorkingMarkdownNode {
    pub kind: NotesWorkingMarkdownNodeKind,
    pub name: String,
    pub relative_path: String,
    pub children: Vec<NotesWorkingMarkdownNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesWorkingMarkdownRoot {
    pub working_folder_id: ProjectWorkingFolderId,
    pub display_name: String,
    pub source_kind: String,
    pub display_path: String,
    pub nodes: Vec<NotesWorkingMarkdownNode>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesWorkingMarkdownTreeRead {
    pub roots: Vec<NotesWorkingMarkdownRoot>,
    pub unavailable_working_folder_ids: Vec<ProjectWorkingFolderId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesWorkingMarkdownFileRead {
    pub working_folder_id: ProjectWorkingFolderId,
    pub relative_path: String,
    pub content: String,
    pub revision: String,
    pub byte_size: u64,
}

struct ScanBudget {
    visited_entries: usize,
    truncated: bool,
}

#[tauri::command]
pub async fn notes_list_working_markdown<R: Runtime>(
    app: tauri::AppHandle<R>,
    db_url: String,
    project_id: String,
) -> ChatResult<NotesWorkingMarkdownTreeRead> {
    validate_project_id(&project_id)?;
    let pool = connect_sqlite(app.clone(), db_url)
        .await
        .map_err(persistence_error)?;
    let folders = workspaces::list_workspaces(&pool).await?;
    for folder in folders
        .iter()
        .filter(|folder| folder.project_id == project_id && folder.archived_at.is_none())
    {
        ensure_managed_working_folder_binding(&app, folder)?;
    }
    let scope = read_active_working_folder_scope(&app).map_err(device_state_error)?;
    let mut roots = Vec::new();
    let mut unavailable = Vec::new();
    for folder in folders
        .into_iter()
        .filter(|folder| folder.project_id == project_id && folder.archived_at.is_none())
    {
        let authorized = match authorize_workspace(
            &folder,
            &scope,
            WorkingFolderAuthorizationOperation::FileRead,
        ) {
            Ok(authorized) => authorized,
            Err(_) => {
                unavailable.push(folder.id);
                continue;
            }
        };
        let mut budget = ScanBudget {
            visited_entries: 0,
            truncated: false,
        };
        let nodes = scan_directory(&authorized.canonical_path, Path::new(""), 0, &mut budget)?;
        if nodes.is_empty() {
            continue;
        }
        roots.push(NotesWorkingMarkdownRoot {
            working_folder_id: folder.id,
            display_name: folder.display_name,
            source_kind: match folder.kind {
                crate::chat::workspace::WorkingFolderKind::Managed => "managed".to_string(),
                crate::chat::workspace::WorkingFolderKind::External => "external".to_string(),
            },
            display_path: folder
                .managed_relative_path
                .unwrap_or_else(|| authorized.canonical_path.to_string_lossy().into_owned()),
            nodes,
            truncated: budget.truncated,
        });
    }
    Ok(NotesWorkingMarkdownTreeRead {
        roots,
        unavailable_working_folder_ids: unavailable,
    })
}

#[tauri::command]
pub async fn notes_read_working_markdown<R: Runtime>(
    app: tauri::AppHandle<R>,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
) -> ChatResult<NotesWorkingMarkdownFileRead> {
    let authorized = require_folder(
        &app,
        db_url,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await?;
    let path = resolve_markdown_file(&authorized, &relative_path)?;
    let bytes = read_bounded_markdown(&path)?;
    let content = String::from_utf8(bytes.clone()).map_err(|_| invalid_utf8_error())?;
    Ok(NotesWorkingMarkdownFileRead {
        working_folder_id,
        relative_path: normalize_relative_path(&relative_path)?,
        content,
        revision: revision_digest(&bytes),
        byte_size: bytes.len() as u64,
    })
}

#[tauri::command]
pub async fn notes_save_working_markdown<R: Runtime>(
    app: tauri::AppHandle<R>,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
    content: String,
    expected_revision: String,
) -> ChatResult<NotesWorkingMarkdownFileRead> {
    if content.len() as u64 > MAX_MARKDOWN_FILE_BYTES {
        return Err(file_too_large_error());
    }
    let authorized = require_folder(
        &app,
        db_url,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::FileWrite,
    )
    .await?;
    let path = resolve_markdown_file(&authorized, &relative_path)?;
    let current = read_bounded_markdown(&path)?;
    ensure_expected_revision(&current, &expected_revision)?;
    write_atomic(&path, content.as_bytes())?;
    let bytes = read_bounded_markdown(&path)?;
    Ok(NotesWorkingMarkdownFileRead {
        working_folder_id,
        relative_path: normalize_relative_path(&relative_path)?,
        content: String::from_utf8(bytes.clone()).map_err(|_| invalid_utf8_error())?,
        revision: revision_digest(&bytes),
        byte_size: bytes.len() as u64,
    })
}

#[tauri::command]
pub async fn notes_open_working_markdown<R: Runtime>(
    app: tauri::AppHandle<R>,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_path: String,
) -> ChatResult<()> {
    let authorized = require_folder(
        &app,
        db_url,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await?;
    let path = resolve_markdown_file(&authorized, &relative_path)?;
    open_authorized_path(&authorized, &path)
}

async fn require_folder<R: Runtime>(
    app: &tauri::AppHandle<R>,
    db_url: String,
    working_folder_id: &ProjectWorkingFolderId,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<AuthorizedWorkingFolder> {
    let pool = connect_sqlite(app.clone(), db_url)
        .await
        .map_err(persistence_error)?;
    let folder = workspaces::read_workspace(&pool, working_folder_id).await?;
    if folder.archived_at.is_some() {
        return Err(ChatError::validation(
            "workingFolderId",
            "Archived project working folders cannot be edited",
        ));
    }
    ensure_managed_working_folder_binding(app, &folder)?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    authorize_workspace(&folder, &scope, operation)
}

fn scan_directory(
    root: &Path,
    relative_directory: &Path,
    depth: usize,
    budget: &mut ScanBudget,
) -> ChatResult<Vec<NotesWorkingMarkdownNode>> {
    if depth > MAX_SCAN_DEPTH {
        budget.truncated = true;
        return Ok(Vec::new());
    }
    let directory = root.join(relative_directory);
    let mut entries = Vec::new();
    for entry in fs::read_dir(&directory).map_err(file_error)? {
        if budget.visited_entries >= MAX_SCAN_ENTRIES {
            budget.truncated = true;
            break;
        }
        budget.visited_entries += 1;
        let entry = entry.map_err(file_error)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(file_error)?;
        if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if name.chars().any(char::is_control) {
            continue;
        }
        let relative = relative_directory.join(&name);
        let Some(relative_path) = relative.to_str().map(normalize_separator) else {
            continue;
        };
        if relative_path.len() > MAX_RELATIVE_PATH_BYTES {
            budget.truncated = true;
            continue;
        }
        if metadata.is_dir() {
            if is_excluded_directory(&name) {
                continue;
            }
            let children = scan_directory(root, &relative, depth + 1, budget)?;
            if !children.is_empty() {
                entries.push(NotesWorkingMarkdownNode {
                    kind: NotesWorkingMarkdownNodeKind::Directory,
                    name,
                    relative_path,
                    children,
                });
            }
        } else if is_markdown_name(&name) && metadata.len() <= MAX_MARKDOWN_FILE_BYTES {
            entries.push(NotesWorkingMarkdownNode {
                kind: NotesWorkingMarkdownNodeKind::File,
                name,
                relative_path,
                children: Vec::new(),
            });
        } else if is_markdown_name(&name) {
            budget.truncated = true;
        }
    }
    entries.sort_by(|left, right| {
        node_rank(&left.kind)
            .cmp(&node_rank(&right.kind))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(entries)
}

fn resolve_markdown_file(
    authorized: &AuthorizedWorkingFolder,
    relative_path: &str,
) -> ChatResult<PathBuf> {
    let normalized = normalize_relative_path(relative_path)?;
    if !is_markdown_name(
        Path::new(&normalized)
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default(),
    ) {
        return Err(ChatError::validation(
            "relativePath",
            "Only Markdown files can be opened in project Notes",
        ));
    }
    let mut candidate = authorized.canonical_path.clone();
    for component in Path::new(&normalized).components() {
        let Component::Normal(segment) = component else {
            return Err(path_validation_error());
        };
        candidate.push(segment);
        let metadata = fs::symlink_metadata(&candidate).map_err(file_error)?;
        if metadata.file_type().is_symlink() {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Symbolic links are not available in project Notes",
                false,
            ));
        }
    }
    let canonical = fs::canonicalize(&candidate).map_err(file_error)?;
    if !canonical.starts_with(&authorized.canonical_path) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Markdown path resolves outside the working folder",
            false,
        ));
    }
    let metadata = fs::metadata(&canonical).map_err(file_error)?;
    if !metadata.is_file() {
        return Err(ChatError::validation(
            "relativePath",
            "Markdown path must identify a file",
        ));
    }
    Ok(canonical)
}

fn normalize_relative_path(relative_path: &str) -> ChatResult<String> {
    if relative_path.is_empty()
        || relative_path.len() > MAX_RELATIVE_PATH_BYTES
        || relative_path.chars().any(char::is_control)
    {
        return Err(path_validation_error());
    }
    let path = Path::new(relative_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(path_validation_error());
    }
    path.to_str()
        .map(normalize_separator)
        .ok_or_else(path_validation_error)
}

fn read_bounded_markdown(path: &Path) -> ChatResult<Vec<u8>> {
    let metadata = fs::metadata(path).map_err(file_error)?;
    if metadata.len() > MAX_MARKDOWN_FILE_BYTES {
        return Err(file_too_large_error());
    }
    fs::read(path).map_err(file_error)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> ChatResult<()> {
    let parent = path.parent().ok_or_else(path_validation_error)?;
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(path_validation_error)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| file_error("clock"))?
        .as_nanos();
    let temporary = parent.join(format!(".{file_name}.ganbaru-{nonce}.tmp"));
    let result = (|| -> ChatResult<()> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(file_error)?;
        file.write_all(bytes).map_err(file_error)?;
        file.sync_all().map_err(file_error)?;
        fs::rename(&temporary, path).map_err(file_error)?;
        if let Ok(directory) = fs::File::open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn revision_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn ensure_expected_revision(current: &[u8], expected_revision: &str) -> ChatResult<()> {
    if revision_digest(current) == expected_revision {
        return Ok(());
    }
    Err(ChatError::new(
        ChatErrorCode::StaleRevision,
        "The Markdown file changed outside Ganbaru AI",
        true,
    ))
}

fn is_markdown_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn is_excluded_directory(name: &str) -> bool {
    EXCLUDED_DIRECTORIES
        .iter()
        .any(|excluded| name.eq_ignore_ascii_case(excluded))
}

fn node_rank(kind: &NotesWorkingMarkdownNodeKind) -> u8 {
    match kind {
        NotesWorkingMarkdownNodeKind::Directory => 0,
        NotesWorkingMarkdownNodeKind::File => 1,
    }
}

fn normalize_separator(path: &str) -> String {
    path.replace('\\', "/")
}

fn validate_project_id(project_id: &str) -> ChatResult<()> {
    if project_id.trim().is_empty()
        || project_id.len() > 1_024
        || project_id.chars().any(char::is_control)
    {
        return Err(ChatError::validation("projectId", "project ID is invalid"));
    }
    Ok(())
}

fn path_validation_error() -> ChatError {
    ChatError::validation("relativePath", "Markdown relative path is invalid")
}

fn invalid_utf8_error() -> ChatError {
    ChatError::validation("relativePath", "Markdown file must contain UTF-8 text")
}

fn file_too_large_error() -> ChatError {
    ChatError::validation(
        "relativePath",
        "Markdown files larger than 4 MiB are not available in project Notes",
    )
}

fn file_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "The Markdown file or directory could not be accessed",
        true,
    )
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Project working-folder persistence failed",
        true,
    )
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
    use super::*;

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ganbaru-working-markdown-{label}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn scan_prunes_empty_and_excluded_directories() {
        let root = test_root("pruning");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/guides")).unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
        fs::create_dir_all(root.join("node_modules/package")).unwrap();
        fs::write(root.join("docs/guides/start.md"), "# Start\n").unwrap();
        fs::write(root.join("docs/ignore.txt"), "ignored").unwrap();
        fs::write(root.join("node_modules/package/readme.md"), "ignored").unwrap();
        let mut budget = ScanBudget {
            visited_entries: 0,
            truncated: false,
        };
        let nodes = scan_directory(&root, Path::new(""), 0, &mut budget).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].relative_path, "docs");
        assert_eq!(nodes[0].children[0].relative_path, "docs/guides");
        assert_eq!(
            nodes[0].children[0].children[0].relative_path,
            "docs/guides/start.md"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn revision_save_refuses_stale_content() {
        let first = revision_digest(b"first");
        let second = revision_digest(b"second");
        assert_ne!(first, second);
        assert!(first.starts_with("sha256:"));
        assert!(ensure_expected_revision(b"first", &first).is_ok());
        assert!(ensure_expected_revision(b"second", &first).is_err());
    }

    #[test]
    fn relative_path_rejects_traversal_and_control_characters() {
        assert!(normalize_relative_path("../secret.md").is_err());
        assert!(normalize_relative_path("docs/\0secret.md").is_err());
        assert_eq!(
            normalize_relative_path("docs/readme.md").unwrap(),
            "docs/readme.md"
        );
    }

    #[test]
    fn scan_reports_depth_and_file_size_truncation() {
        let root = test_root("caps");
        let _ = fs::remove_dir_all(&root);
        let mut deep = root.clone();
        for index in 0..=MAX_SCAN_DEPTH {
            deep.push(format!("level-{index}"));
        }
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("too-deep.md"), "hidden").unwrap();
        fs::write(
            root.join("too-large.md"),
            vec![b'a'; MAX_MARKDOWN_FILE_BYTES as usize + 1],
        )
        .unwrap();
        let mut budget = ScanBudget {
            visited_entries: 0,
            truncated: false,
        };
        let nodes = scan_directory(&root, Path::new(""), 0, &mut budget).unwrap();
        assert!(nodes.is_empty());
        assert!(budget.truncated);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bounded_read_rejects_non_utf8_and_atomic_write_replaces_content() {
        let root = test_root("read-write");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("notes.md");
        fs::write(&path, [0xff, 0xfe]).unwrap();
        let bytes = read_bounded_markdown(&path).unwrap();
        assert!(String::from_utf8(bytes).is_err());
        write_atomic(&path, b"# Updated\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Updated\n");
        assert!(!fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains(".ganbaru-")));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn scan_and_resolution_reject_symbolic_links() {
        use crate::chat::models::RepositoryKind;
        use std::os::unix::fs::symlink;

        let root = test_root("symlink-root");
        let outside = test_root("symlink-outside");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("outside.md"), "secret").unwrap();
        symlink(outside.join("outside.md"), root.join("linked.md")).unwrap();
        let mut budget = ScanBudget {
            visited_entries: 0,
            truncated: false,
        };
        assert!(scan_directory(&root, Path::new(""), 0, &mut budget)
            .unwrap()
            .is_empty());
        let authorized = AuthorizedWorkingFolder {
            working_folder_id: ProjectWorkingFolderId::new("folder").unwrap(),
            canonical_path: fs::canonicalize(&root).unwrap(),
            repository_kind: RepositoryKind::None,
            repository_identity: None,
        };
        assert!(resolve_markdown_file(&authorized, "linked.md").is_err());
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
    }
}
