use super::super::models::{ProjectWorkingFolderId, RepositoryKind, UtcTimestamp};
use super::super::workspace::{
    authorize_workspace, probe_repository, resolve_workspace_relative_path, workspace_read,
    ProjectWorkingFolder, WorkingFolderAuthorizationOperation, WorkingFolderBindingStatus,
    WorkingFolderKind,
};
use crate::projects::working_folders::{
    ProjectWorkingFolderBindingState, WorkingFolderDeviceScope,
};
use std::fs;
use std::path::{Path, PathBuf};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let unique = format!(
            "ganbaru-chat-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("test clock should follow the Unix epoch")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("test directory should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn git_repository(label: &str, remote: &str) -> Self {
        let directory = Self::new(label);
        fs::create_dir(directory.path().join(".git"))
            .expect("Git metadata directory should be created");
        fs::write(
            directory.path().join(".git/config"),
            format!(
                "[core]\n\trepositoryformatversion = 0\n[remote \"origin\"]\n\turl = {remote}\n"
            ),
        )
        .expect("Git config should be written");
        fs::write(
            directory.path().join(".git/HEAD"),
            "ref: refs/heads/feat/chat\n",
        )
        .expect("Git HEAD should be written");
        directory
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn timestamp() -> UtcTimestamp {
    UtcTimestamp::new("2026-07-20T12:00:00Z").expect("timestamp should be valid")
}

fn workspace(id: &str, kind: RepositoryKind, identity: Option<String>) -> ProjectWorkingFolder {
    ProjectWorkingFolder {
        id: ProjectWorkingFolderId::new(id).expect("workspace ID should be valid"),
        project_id: "project-1".to_string(),
        display_name: "Frontend".to_string(),
        kind: WorkingFolderKind::External,
        managed_relative_path: None,
        sort_order: 10,
        repository_kind: kind,
        repository_identity: identity,
        created_at: timestamp(),
        updated_at: timestamp(),
        archived_at: None,
        revision: 1,
    }
}

fn binding(
    path: &Path,
    kind: RepositoryKind,
    identity: Option<String>,
) -> ProjectWorkingFolderBindingState {
    ProjectWorkingFolderBindingState {
        canonical_path: path
            .to_str()
            .expect("test path should be UTF-8")
            .to_string(),
        repository_kind: kind,
        repository_identity: identity,
        last_verified_at: timestamp(),
    }
}

#[test]
fn non_git_workspace_is_authorized_for_every_guarded_operation() {
    let directory = TestDirectory::new("non-git");
    let workspace = workspace("workspace-1", RepositoryKind::None, None);
    let mut scope = WorkingFolderDeviceScope::default();
    scope.bindings.insert(
        workspace.id.clone(),
        binding(directory.path(), RepositoryKind::None, None),
    );

    for operation in WorkingFolderAuthorizationOperation::ALL {
        let authorized = authorize_workspace(&workspace, &scope, operation).unwrap();
        assert_eq!(authorized.canonical_path, directory.path());
    }
}

#[test]
fn another_device_without_a_binding_remains_unbound() {
    let read = workspace_read(
        workspace("working-folder-1", RepositoryKind::None, None),
        &WorkingFolderDeviceScope::default(),
    )
    .unwrap();
    assert_eq!(read.binding_status, WorkingFolderBindingStatus::Unbound);
    assert_eq!(read.canonical_path, None);
}

#[test]
fn missing_and_stale_bindings_are_rejected() {
    let missing = std::env::temp_dir().join("ganbaru-chat-definitely-missing");
    let workspace = workspace("workspace-1", RepositoryKind::None, None);
    let mut scope = WorkingFolderDeviceScope::default();
    scope.bindings.insert(
        workspace.id.clone(),
        binding(&missing, RepositoryKind::None, None),
    );
    assert!(authorize_workspace(
        &workspace,
        &scope,
        WorkingFolderAuthorizationOperation::ProviderStart
    )
    .is_err());

    let directory = TestDirectory::new("stale");
    let noncanonical = directory.path().join(".");
    scope.bindings.insert(
        workspace.id.clone(),
        binding(&noncanonical, RepositoryKind::None, None),
    );
    assert!(authorize_workspace(
        &workspace,
        &scope,
        WorkingFolderAuthorizationOperation::ProviderStart
    )
    .is_err());
}

#[test]
fn repository_identity_is_stable_and_redacts_remote_credentials() {
    let first = TestDirectory::git_repository(
        "first-clone",
        "https://alice:secret-token@example.com/Owner/Repository.git",
    );
    let second =
        TestDirectory::git_repository("second-clone", "git@example.com:owner/repository.git");
    let first_probe = probe_repository(first.path()).unwrap();
    let second_probe = probe_repository(second.path()).unwrap();

    assert_eq!(first_probe.kind, RepositoryKind::Git);
    assert_eq!(first_probe.identity, second_probe.identity);
    assert_eq!(first_probe.current_branch.as_deref(), Some("feat/chat"));
    let identity = first_probe.identity.unwrap();
    assert!(!identity.contains("alice"));
    assert!(!identity.contains("secret-token"));
    assert!(!identity.contains("example.com"));
}

#[test]
fn repository_mismatch_blocks_authorization() {
    let first = TestDirectory::git_repository("repo-a", "https://example.com/owner/a.git");
    let second = TestDirectory::git_repository("repo-b", "https://example.com/owner/b.git");
    let first_probe = probe_repository(first.path()).unwrap();
    let second_probe = probe_repository(second.path()).unwrap();
    let workspace = workspace(
        "workspace-1",
        RepositoryKind::Git,
        first_probe.identity.clone(),
    );
    let mut scope = WorkingFolderDeviceScope::default();
    scope.bindings.insert(
        workspace.id.clone(),
        binding(second.path(), RepositoryKind::Git, second_probe.identity),
    );

    let error = authorize_workspace(
        &workspace,
        &scope,
        WorkingFolderAuthorizationOperation::Restore,
    )
    .unwrap_err();
    assert!(error.message.contains("different repository"));
}

#[test]
fn traversal_and_absolute_paths_are_rejected() {
    let directory = TestDirectory::new("paths");
    let authorized = super::super::workspace::AuthorizedWorkingFolder {
        working_folder_id: ProjectWorkingFolderId::new("workspace-1").unwrap(),
        canonical_path: directory.path().to_path_buf(),
        repository_kind: RepositoryKind::None,
        repository_identity: None,
    };

    assert!(resolve_workspace_relative_path(&authorized, "../outside.txt").is_err());
    assert!(resolve_workspace_relative_path(&authorized, "/etc/passwd").is_err());
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_rejected_at_resolution_time() {
    use std::os::unix::fs::symlink;

    let workspace_directory = TestDirectory::new("symlink-workspace");
    let outside_directory = TestDirectory::new("symlink-outside");
    fs::write(outside_directory.path().join("secret.txt"), "outside")
        .expect("outside file should be created");
    symlink(
        outside_directory.path(),
        workspace_directory.path().join("escape"),
    )
    .expect("test symlink should be created");
    let authorized = super::super::workspace::AuthorizedWorkingFolder {
        working_folder_id: ProjectWorkingFolderId::new("workspace-1").unwrap(),
        canonical_path: workspace_directory.path().to_path_buf(),
        repository_kind: RepositoryKind::None,
        repository_identity: None,
    };

    let error = resolve_workspace_relative_path(&authorized, "escape/secret.txt").unwrap_err();
    assert_eq!(
        error.message,
        "Workspace path resolves outside the bound folder"
    );
}
