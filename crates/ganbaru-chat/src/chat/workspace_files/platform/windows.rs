use super::*;

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
pub(in crate::chat::workspace_files) fn secure_workspace_file(
    root: &Path,
    relative_path: &str,
) -> ChatResult<SecureWorkspaceFile> {
    let (parent_handles, file) = windows_open_regular_file(root, relative_path)?;
    Ok(SecureWorkspaceFile {
        file,
        _parent_handles: parent_handles,
    })
}

#[cfg(windows)]
pub(in crate::chat::workspace_files) fn secure_directory_entries(
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

#[cfg(windows)]
pub(in crate::chat::workspace_files) fn workspace_regular_file_permissions(
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

#[cfg(windows)]
pub(in crate::chat::workspace_files) fn create_workspace_file_exclusively(
    root: &Path,
    relative_path: &str,
    bytes: &[u8],
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
    if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
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

#[cfg(windows)]
pub(in crate::chat::workspace_files) fn delete_workspace_file_atomically(
    root: &Path,
    relative_path: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    let relative = Path::new(relative_path);
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new(""));
    let parent_relative = parent_relative
        .to_str()
        .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is unsupported"))?;
    let (parent, parent_handles) = windows_workspace_directory_chain(root, parent_relative)?;
    let target = parent.join(
        relative
            .file_name()
            .ok_or_else(|| ChatError::validation("relativePath", "Workspace path is invalid"))?,
    );
    let (_current_parents, mut current) = windows_open_regular_file(root, relative_path)?;
    let (current_revision, _) = revision_and_permissions(&mut current, relative_path)?;
    if current_revision != expected_revision {
        return Err(stale_workspace_file_error());
    }
    drop(current);
    let (_, backup, _) = windows_replacement_paths(&parent);
    fs::rename(&target, &backup).map_err(|_| workspace_file_write_error())?;
    let displaced_matches = windows_revision_for_path(&backup, relative_path)
        .is_ok_and(|revision| revision == expected_revision);
    if !displaced_matches {
        if fs::symlink_metadata(&target).is_err() && fs::rename(&backup, &target).is_ok() {
            return Err(stale_workspace_file_error());
        }
        return Err(workspace_file_recovery_error());
    }
    let removed = fs::remove_file(&backup).map_err(|_| workspace_file_recovery_error());
    drop(parent_handles);
    removed
}

#[cfg(windows)]
pub(in crate::chat::workspace_files) fn write_workspace_text_atomically(
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
    use ::windows::core::PCWSTR;
    use ::windows::Win32::Storage::FileSystem::{ReplaceFileW, REPLACE_FILE_FLAGS};
    use std::os::windows::ffi::OsStrExt;

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
