use super::*;

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
pub(in crate::chat::workspace_files) fn secure_directory_entries(
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
pub(in crate::chat::workspace_files) fn secure_workspace_file(
    root: &Path,
    relative_path: &str,
) -> ChatResult<SecureWorkspaceFile> {
    let parent = secure_workspace_parent(root, relative_path).map_err(file_error)?;
    open_regular_file_at(&parent)
        .map(|file| SecureWorkspaceFile { file })
        .map_err(file_error)
}

#[cfg(unix)]
pub(in crate::chat::workspace_files) struct SecureWorkspaceParent {
    directory: File,
    file_name: CString,
}

#[cfg(unix)]
pub(in crate::chat::workspace_files) fn secure_workspace_parent(
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
pub(in crate::chat::workspace_files) fn open_regular_file_at(
    parent: &SecureWorkspaceParent,
) -> std::io::Result<File> {
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
fn rename_noreplace_at(
    parent: &SecureWorkspaceParent,
    source: &CString,
    destination: &CString,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameat2(
            parent.directory.as_raw_fd(),
            source.as_ptr(),
            parent.directory.as_raw_fd(),
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn rename_noreplace_at(
    parent: &SecureWorkspaceParent,
    source: &CString,
    destination: &CString,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameatx_np(
            parent.directory.as_raw_fd(),
            source.as_ptr(),
            parent.directory.as_raw_fd(),
            destination.as_ptr(),
            libc::RENAME_EXCL,
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
fn rename_noreplace_at(
    _parent: &SecureWorkspaceParent,
    _source: &CString,
    _destination: &CString,
) -> std::io::Result<()> {
    Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
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

#[cfg(unix)]
pub(in crate::chat::workspace_files) fn workspace_regular_file_permissions(
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

#[cfg(unix)]
pub(in crate::chat::workspace_files) fn create_workspace_file_exclusively(
    root: &Path,
    relative_path: &str,
    bytes: &[u8],
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
    if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
        drop(file);
        return if unlink_at(&parent, &parent.file_name).is_ok() {
            Err(workspace_file_write_error())
        } else {
            Err(workspace_file_recovery_error())
        };
    }
    Ok(())
}

#[cfg(unix)]
pub(in crate::chat::workspace_files) fn delete_workspace_file_atomically(
    root: &Path,
    relative_path: &str,
    expected_revision: &str,
) -> ChatResult<()> {
    let parent =
        secure_workspace_parent(root, relative_path).map_err(|_| workspace_file_write_error())?;
    let mut current = open_regular_file_at(&parent).map_err(|_| workspace_file_write_error())?;
    let (current_revision, _) = revision_and_permissions(&mut current, relative_path)?;
    if current_revision != expected_revision {
        return Err(stale_workspace_file_error());
    }
    let metadata = current
        .metadata()
        .map_err(|_| workspace_file_write_error())?;
    let identity = (metadata.dev(), metadata.ino());
    let generation = WORKSPACE_FILE_WRITE_GENERATION.fetch_add(1, Ordering::Relaxed);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let backup_name = CString::new(format!(
        ".ganbaru.{}.{}.{}.backup",
        std::process::id(),
        generation,
        nonce,
    ))
    .map_err(|_| workspace_file_write_error())?;
    rename_noreplace_at(&parent, &parent.file_name, &backup_name)
        .map_err(|_| workspace_file_write_error())?;
    let verification = (|| {
        let descriptor = unsafe {
            libc::openat(
                parent.directory.as_raw_fd(),
                backup_name.as_ptr(),
                libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_RDONLY,
            )
        };
        if descriptor < 0 {
            return Err(workspace_file_write_error());
        }
        let mut displaced = unsafe { File::from_raw_fd(descriptor) };
        let displaced_metadata = displaced
            .metadata()
            .map_err(|_| workspace_file_write_error())?;
        if !displaced_metadata.is_file()
            || (displaced_metadata.dev(), displaced_metadata.ino()) != identity
        {
            return Err(stale_workspace_file_error());
        }
        let (revision, _) = revision_and_permissions(&mut displaced, relative_path)?;
        if revision != expected_revision {
            return Err(stale_workspace_file_error());
        }
        Ok(())
    })();
    if let Err(error) = verification {
        return if rename_noreplace_at(&parent, &backup_name, &parent.file_name).is_ok() {
            Err(error)
        } else {
            Err(workspace_file_recovery_error())
        };
    }
    unlink_at(&parent, &backup_name).map_err(|_| workspace_file_recovery_error())
}

#[cfg(unix)]
pub(in crate::chat::workspace_files) fn write_workspace_text_atomically(
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
