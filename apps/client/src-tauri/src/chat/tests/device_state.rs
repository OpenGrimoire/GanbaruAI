use crate::chat::{
    device_state::{
        ChatDeviceState, ChatProviderDeviceState, ChatWorkspaceBindingState,
        CHAT_DEVICE_STATE_SCHEMA_VERSION,
    },
    models::{ChatWorkspaceId, ProviderInstanceId, RepositoryKind, UtcTimestamp},
};

#[test]
fn bindings_and_provider_paths_are_scoped_by_vault_and_device() {
    let mut state = ChatDeviceState::default();
    let workspace_id = ChatWorkspaceId::new("workspace-1").unwrap();
    let provider_id = ProviderInstanceId::new("codex-personal").unwrap();
    let binding = ChatWorkspaceBindingState {
        canonical_path: "/mnt/work/ganbaru".to_string(),
        repository_kind: RepositoryKind::Git,
        repository_identity: Some("git:example/ganbaru".to_string()),
        last_verified_at: UtcTimestamp::new("2026-07-20T12:00:00Z").unwrap(),
    };

    let first_device = state.scope_mut("vault-1", "device-1");
    first_device
        .workspace_bindings
        .insert(workspace_id.clone(), binding.clone());
    first_device.provider_instances.insert(
        provider_id.clone(),
        ChatProviderDeviceState {
            executable_path: Some("/usr/bin/codex".to_string()),
            provider_home_path: Some("/home/user/.codex".to_string()),
            last_probe: None,
        },
    );

    assert_eq!(
        state
            .scope("vault-1", "device-1")
            .and_then(|scope| scope.workspace_bindings.get(&workspace_id)),
        Some(&binding)
    );
    assert_eq!(
        state
            .scope("vault-1", "device-1")
            .and_then(|scope| scope.provider_instances.get(&provider_id))
            .and_then(|provider| provider.executable_path.as_deref()),
        Some("/usr/bin/codex")
    );
    assert!(state.scope("vault-1", "device-2").is_none());
    assert!(state.scope("vault-2", "device-1").is_none());
}

#[test]
fn device_state_round_trips_typed_map_keys() {
    let mut state = ChatDeviceState::default();
    state
        .scope_mut("vault-1", "device-1")
        .workspace_bindings
        .insert(
            ChatWorkspaceId::new("workspace-1").unwrap(),
            ChatWorkspaceBindingState {
                canonical_path: "/mnt/work/ganbaru".to_string(),
                repository_kind: RepositoryKind::Git,
                repository_identity: Some("git:example/ganbaru".to_string()),
                last_verified_at: UtcTimestamp::new("2026-07-20T12:00:00Z").unwrap(),
            },
        );

    let serialized = serde_json::to_value(&state).unwrap();
    let restored: ChatDeviceState = serde_json::from_value(serialized).unwrap();

    assert_eq!(restored, state);
    assert_eq!(restored.schema_version, CHAT_DEVICE_STATE_SCHEMA_VERSION);
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
