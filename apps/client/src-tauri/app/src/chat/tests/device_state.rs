use crate::chat::{
    device_state::{
        full_access_is_trusted, set_full_access_trust, ChatDeviceScope, ChatDeviceState,
        CHAT_DEVICE_STATE_SCHEMA_VERSION, DEFAULT_DIAGNOSTIC_RETENTION_DAYS,
    },
    models::{ProjectWorkingFolderId, ProviderInstanceId, RepositoryKind, UtcTimestamp},
};
use crate::projects::working_folders::{
    ProjectWorkingFolderBindingState, WORKING_FOLDER_DEVICE_STATE_SCHEMA_VERSION,
};
use ganbaru_working_folders::WorkingFolderDeviceState;

#[test]
fn bindings_are_scoped_by_vault_and_device() {
    let mut state = WorkingFolderDeviceState::default();
    let working_folder_id = ProjectWorkingFolderId::new("workspace-1").unwrap();
    let binding = ProjectWorkingFolderBindingState {
        canonical_path: "/mnt/work/ganbaru".to_string(),
        filesystem_identity: Some("filesystem-sha256:folder".to_string()),
        repository_kind: RepositoryKind::Git,
        repository_identity: Some("git:example/ganbaru".to_string()),
        repository_storage_identity: Some("filesystem-sha256:git".to_string()),
        last_verified_at: UtcTimestamp::new("2026-07-20T12:00:00Z").unwrap(),
    };

    let first_device = state.scope_mut("vault-1", "device-1");
    first_device
        .bindings
        .insert(working_folder_id.clone(), binding.clone());

    assert_eq!(
        state
            .scope("vault-1", "device-1")
            .and_then(|scope| scope.bindings.get(&working_folder_id)),
        Some(&binding)
    );
    assert!(state.scope("vault-1", "device-2").is_none());
    assert!(state.scope("vault-2", "device-1").is_none());
}

#[test]
fn working_folder_device_state_round_trips_typed_map_keys() {
    let mut state = WorkingFolderDeviceState::default();
    state.scope_mut("vault-1", "device-1").bindings.insert(
        ProjectWorkingFolderId::new("workspace-1").unwrap(),
        ProjectWorkingFolderBindingState {
            canonical_path: "/mnt/work/ganbaru".to_string(),
            filesystem_identity: Some("filesystem-sha256:folder".to_string()),
            repository_kind: RepositoryKind::Git,
            repository_identity: Some("git:example/ganbaru".to_string()),
            repository_storage_identity: Some("filesystem-sha256:git".to_string()),
            last_verified_at: UtcTimestamp::new("2026-07-20T12:00:00Z").unwrap(),
        },
    );

    let serialized = serde_json::to_value(&state).unwrap();
    let restored: WorkingFolderDeviceState = serde_json::from_value(serialized).unwrap();

    assert_eq!(restored, state);
    assert_eq!(
        restored.schema_version,
        WORKING_FOLDER_DEVICE_STATE_SCHEMA_VERSION
    );
}

#[test]
fn legacy_working_folder_binding_defaults_new_identity_fields() {
    let binding: ProjectWorkingFolderBindingState = serde_json::from_value(serde_json::json!({
        "canonicalPath": "/mnt/work/ganbaru",
        "repositoryKind": "git",
        "repositoryIdentity": "git-sha256:legacy",
        "lastVerifiedAt": "2026-07-20T12:00:00Z"
    }))
    .unwrap();

    assert_eq!(binding.filesystem_identity, None);
    assert_eq!(binding.repository_storage_identity, None);
}

#[test]
fn missing_legacy_chat_state_defaults_to_the_current_schema() {
    let restored: crate::vault::VaultAppState = serde_json::from_value(serde_json::json!({
        "activeVaultPath": null,
        "recentVaultPaths": []
    }))
    .unwrap();

    assert_eq!(
        restored.chat.schema_version,
        CHAT_DEVICE_STATE_SCHEMA_VERSION
    );
    assert!(restored.chat.vaults.is_empty());
}

#[test]
fn legacy_device_scope_defaults_to_disabled_bounded_diagnostics() {
    let scope: ChatDeviceScope = serde_json::from_value(serde_json::json!({})).unwrap();
    assert!(!scope.diagnostics.capture_enabled);
    assert_eq!(
        scope.diagnostics.retention_days,
        DEFAULT_DIAGNOSTIC_RETENTION_DAYS
    );
}

#[test]
fn full_access_trust_never_crosses_provider_workspace_vault_or_device_boundaries() {
    let mut state = ChatDeviceState::default();
    let trusted_provider = ProviderInstanceId::new("codex-personal").unwrap();
    let incompatible_provider = ProviderInstanceId::new("codex-other-home").unwrap();
    let trusted_workspace = ProjectWorkingFolderId::new("workspace-1").unwrap();
    let new_workspace = ProjectWorkingFolderId::new("workspace-2").unwrap();
    let timestamp = UtcTimestamp::new("2026-07-21T12:00:00Z").unwrap();
    set_full_access_trust(
        state.scope_mut("vault-1", "device-1"),
        trusted_provider.clone(),
        trusted_workspace.clone(),
        Some(timestamp),
    );

    let trusted_scope = state.scope("vault-1", "device-1").unwrap();
    assert!(full_access_is_trusted(
        trusted_scope,
        &trusted_provider,
        &trusted_workspace,
    ));
    assert!(!full_access_is_trusted(
        trusted_scope,
        &trusted_provider,
        &new_workspace,
    ));
    assert!(!full_access_is_trusted(
        trusted_scope,
        &incompatible_provider,
        &trusted_workspace,
    ));
    assert!(!full_access_is_trusted(
        state.scope_mut("vault-1", "device-2"),
        &trusted_provider,
        &trusted_workspace,
    ));
    assert!(!full_access_is_trusted(
        state.scope_mut("vault-2", "device-1"),
        &trusted_provider,
        &trusted_workspace,
    ));

    set_full_access_trust(
        state.scope_mut("vault-1", "device-1"),
        trusted_provider.clone(),
        trusted_workspace.clone(),
        None,
    );
    assert!(!full_access_is_trusted(
        state.scope("vault-1", "device-1").unwrap(),
        &trusted_provider,
        &trusted_workspace,
    ));
}
