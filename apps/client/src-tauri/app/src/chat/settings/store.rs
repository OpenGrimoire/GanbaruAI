use super::{
    config_io_error, config_shape_error, device_state_error, mapping::provider_instance_read,
    provider_not_found,
};
use crate::chat::config::{
    parse_chat_config_branch, replace_chat_config_branch, ChatPortableProviderConfig,
    ChatVaultConfig,
};
use crate::chat::credentials::{CredentialStore, PlatformCredentialStore};
use crate::chat::device_state::read_active_device_scope;
use crate::chat::models::{ChatError, ChatErrorCode, ChatResult, ProviderInstanceId};
use crate::chat::providers::ProviderDriverRegistry;
use crate::chat::settings_commands::{ChatSettingsRead, ChatSettingsState};
use crate::vault;
use serde_json::Value;

pub(crate) fn read_settings(app: &tauri::AppHandle) -> ChatResult<ChatSettingsRead> {
    let configuration = read_chat_config(app)?;
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    let provider_instances = configuration
        .providers
        .iter()
        .map(|portable| provider_instance_read(portable, &scope.provider_instances))
        .collect();
    Ok(ChatSettingsRead {
        configuration,
        provider_families: ProviderDriverRegistry.list_metadata(),
        provider_instances,
        credential_store_availability: PlatformCredentialStore::default().availability(),
        last_selected_thread_id: scope.preferences.last_selected_thread_id,
    })
}

pub(crate) fn mutate_chat_config(
    app: &tauri::AppHandle,
    state: &ChatSettingsState,
    mutate: impl FnOnce(&mut ChatVaultConfig) -> ChatResult<()>,
) -> ChatResult<ChatVaultConfig> {
    let _guard = state.mutation_lock.lock().map_err(|_| {
        ChatError::new(
            ChatErrorCode::Internal,
            "Chat settings are unavailable",
            true,
        )
    })?;
    let raw = vault::vault_read_config(app.clone()).map_err(config_io_error)?;
    let mut root: Value = serde_json::from_str(&raw).map_err(|_| config_shape_error())?;
    let mut config = parse_chat_config_branch(&root)?;
    mutate(&mut config)?;
    config.validate()?;
    replace_chat_config_branch(&mut root, config.clone())?;
    let serialized = serde_json::to_string_pretty(&root).map_err(|_| config_shape_error())?;
    vault::vault_write_config(app.clone(), serialized).map_err(config_io_error)?;
    Ok(config)
}

pub(crate) fn read_chat_config(app: &tauri::AppHandle) -> ChatResult<ChatVaultConfig> {
    let raw = vault::vault_read_config(app.clone()).map_err(config_io_error)?;
    let root: Value = serde_json::from_str(&raw).map_err(|_| config_shape_error())?;
    parse_chat_config_branch(&root)
}

pub(crate) fn provider_mut<'a>(
    config: &'a mut ChatVaultConfig,
    instance_id: &ProviderInstanceId,
) -> ChatResult<&'a mut ChatPortableProviderConfig> {
    config
        .providers
        .iter_mut()
        .find(|provider| &provider.instance_id == instance_id)
        .ok_or_else(provider_not_found)
}
