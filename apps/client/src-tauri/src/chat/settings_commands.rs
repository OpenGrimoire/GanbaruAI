//! Narrow provider and Chat preference commands for the existing Settings UI.

use super::config::{
    parse_chat_config_branch, replace_chat_config_branch, ChatBehaviorPreferences,
    ChatPanelPreferences, ChatPortableProviderConfig, ChatVaultConfig, RememberedComposerSelection,
};
use super::credentials::{
    materialize_provider_environment, CredentialStore, CredentialStoreAvailability,
    PlatformCredentialStore, SecretValue,
};
use super::device_state::{
    read_active_device_scope, update_active_device_scope, ChatProviderDeviceState,
};
use super::models::{
    ChatError, ChatErrorCode, ChatResult, ChatThreadId, ChatWorkspaceId, CredentialReferenceId,
    ModelId, ProbeState, ProviderFamilyMetadataRead, ProviderInstanceConfig, ProviderInstanceId,
    ProviderModelCatalog, ProviderProbeResult,
};
use super::providers::{
    DriverCancellation, DriverOperationContext, ProviderDriverFactory, ProviderDriverRegistry,
};
use crate::vault;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri_plugin_dialog::{DialogExt, FilePath};

const PROVIDER_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Default)]
pub struct ChatSettingsState {
    mutation_lock: Mutex<()>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderInstanceRequest {
    pub configuration: ProviderInstanceConfig,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInstanceRead {
    pub configuration: ProviderInstanceConfig,
    pub last_probe: Option<ProviderProbeResult>,
    pub last_successful_probe_at: Option<super::models::UtcTimestamp>,
    pub model_catalog: Option<ProviderModelCatalog>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSettingsRead {
    pub configuration: ChatVaultConfig,
    pub provider_families: Vec<ProviderFamilyMetadataRead>,
    pub provider_instances: Vec<ProviderInstanceRead>,
    pub credential_store_availability: CredentialStoreAvailability,
    pub last_selected_thread_id: Option<ChatThreadId>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveProviderResult {
    pub removed: bool,
    pub credential_cleanup_failed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSetupTestRead {
    pub probe: ProviderProbeResult,
    pub model_catalog: Option<ProviderModelCatalog>,
}

#[tauri::command]
pub fn chat_read_settings(app: tauri::AppHandle) -> ChatResult<ChatSettingsRead> {
    let configuration = read_chat_config(&app)?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
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

#[tauri::command]
pub fn chat_set_last_selected_thread(
    app: tauri::AppHandle,
    thread_id: Option<ChatThreadId>,
) -> ChatResult<()> {
    update_active_device_scope(&app, |scope| {
        scope.preferences.last_selected_thread_id = thread_id;
        Ok(())
    })
    .map_err(device_state_error)
}

#[tauri::command]
pub fn chat_save_provider(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    request: SaveProviderInstanceRequest,
) -> ChatResult<ProviderInstanceRead> {
    ProviderDriverRegistry.create_driver(request.configuration.clone())?;
    let previous = match read_provider(&app, &request.configuration.instance_id) {
        Ok(read) => Some(read.configuration),
        Err(error) if error.code == ChatErrorCode::NotFound => None,
        Err(error) => return Err(error),
    };
    let runtime_changed = previous
        .as_ref()
        .is_none_or(|previous| provider_runtime_changed(previous, &request.configuration));
    let portable = portable_configuration(&request.configuration);
    let device = device_configuration(&request.configuration);
    mutate_chat_config(&app, &state, |config| {
        if let Some(existing) = config
            .providers
            .iter_mut()
            .find(|candidate| candidate.instance_id == portable.instance_id)
        {
            *existing = portable.clone();
        } else {
            config.providers.push(portable.clone());
        }
        Ok(())
    })?;
    update_active_device_scope(&app, |scope| {
        let existing = scope
            .provider_instances
            .entry(request.configuration.instance_id.clone())
            .or_default();
        existing.executable_path = device.executable_path.clone();
        existing.provider_home_path = device.provider_home_path.clone();
        if runtime_changed {
            existing.last_probe = None;
            if let Some(catalog) = existing.model_catalog.as_mut() {
                catalog.stale = true;
            }
        }
        Ok(())
    })
    .map_err(device_state_error)?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    Ok(provider_instance_read(&portable, &scope.provider_instances))
}

#[tauri::command]
pub fn chat_set_provider_enabled(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    instance_id: ProviderInstanceId,
    enabled: bool,
) -> ChatResult<ProviderInstanceRead> {
    mutate_chat_config(&app, &state, |config| {
        provider_mut(config, &instance_id)?.enabled = enabled;
        Ok(())
    })?;
    read_provider(&app, &instance_id)
}

#[tauri::command]
pub fn chat_remove_provider(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    instance_id: ProviderInstanceId,
) -> ChatResult<RemoveProviderResult> {
    let current = read_chat_config(&app)?;
    let provider = current
        .providers
        .iter()
        .find(|candidate| candidate.instance_id == instance_id)
        .cloned()
        .ok_or_else(provider_not_found)?;
    mutate_chat_config(&app, &state, |config| {
        config
            .providers
            .retain(|candidate| candidate.instance_id != instance_id);
        config
            .remembered_selections
            .retain(|selection| selection.provider_instance_id != instance_id);
        config
            .workspace_provider_preferences
            .retain(|_, provider_id| provider_id != &instance_id);
        Ok(())
    })?;
    update_active_device_scope(&app, |scope| {
        scope.provider_instances.remove(&instance_id);
        Ok(())
    })
    .map_err(device_state_error)?;
    let store = PlatformCredentialStore::default();
    let mut credential_cleanup_failed = false;
    for reference in provider.credential_references.values() {
        if store.remove(reference).is_err() {
            credential_cleanup_failed = true;
        }
    }
    Ok(RemoveProviderResult {
        removed: true,
        credential_cleanup_failed,
    })
}

#[tauri::command]
pub async fn chat_test_provider(
    request: SaveProviderInstanceRequest,
) -> ChatResult<ProviderSetupTestRead> {
    let configuration = materialize_provider_environment(
        &request.configuration,
        &PlatformCredentialStore::default(),
    )?;
    let mut driver = ProviderDriverRegistry.create_driver(configuration)?;
    let probe = driver.probe(&operation_context("test-provider")).await?;
    let model_catalog = if probe.state == ProbeState::Healthy {
        Some(
            driver
                .discover_models(&operation_context("test-provider-models"))
                .await?,
        )
    } else {
        None
    };
    Ok(ProviderSetupTestRead {
        probe,
        model_catalog,
    })
}

#[tauri::command]
pub async fn chat_probe_provider(
    app: tauri::AppHandle,
    instance_id: ProviderInstanceId,
) -> ChatResult<ProviderProbeResult> {
    let read = read_provider(&app, &instance_id)?;
    let configuration =
        materialize_provider_environment(&read.configuration, &PlatformCredentialStore::default())?;
    let mut driver = ProviderDriverRegistry.create_driver(configuration)?;
    let probe = driver.probe(&operation_context("probe-provider")).await?;
    update_active_device_scope(&app, |scope| {
        let device = scope.provider_instances.entry(instance_id).or_default();
        device.last_probe = Some(probe.clone());
        if probe.state == ProbeState::Healthy {
            device.last_successful_probe_at = Some(probe.checked_at.clone());
        }
        Ok(())
    })
    .map_err(device_state_error)?;
    Ok(probe)
}

#[tauri::command]
pub async fn chat_refresh_provider_models(
    app: tauri::AppHandle,
    instance_id: ProviderInstanceId,
) -> ChatResult<ProviderModelCatalog> {
    let read = read_provider(&app, &instance_id)?;
    let configuration =
        materialize_provider_environment(&read.configuration, &PlatformCredentialStore::default())?;
    let mut driver = ProviderDriverRegistry.create_driver(configuration)?;
    let catalog = driver
        .discover_models(&operation_context("discover-provider-models"))
        .await?;
    update_active_device_scope(&app, |scope| {
        scope
            .provider_instances
            .entry(instance_id)
            .or_default()
            .model_catalog = Some(catalog.clone());
        Ok(())
    })
    .map_err(device_state_error)?;
    Ok(catalog)
}

#[tauri::command]
pub fn chat_update_provider_models(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    instance_id: ProviderInstanceId,
    visible_model_ids: Vec<ModelId>,
    favorite_model_ids: Vec<ModelId>,
) -> ChatResult<ProviderInstanceRead> {
    let visible = unique_model_ids(visible_model_ids);
    let favorites = unique_model_ids(favorite_model_ids);
    mutate_chat_config(&app, &state, |config| {
        let provider = provider_mut(config, &instance_id)?;
        provider.visible_model_ids = visible;
        provider.favorite_model_ids = favorites;
        Ok(())
    })?;
    read_provider(&app, &instance_id)
}

#[tauri::command]
pub fn chat_update_behavior(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    behavior: ChatBehaviorPreferences,
) -> ChatResult<ChatVaultConfig> {
    mutate_chat_config(&app, &state, |config| {
        config.behavior = behavior;
        Ok(())
    })
}

#[tauri::command]
pub fn chat_update_panels(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    panels: ChatPanelPreferences,
) -> ChatResult<ChatVaultConfig> {
    mutate_chat_config(&app, &state, |config| {
        config.panels = panels;
        Ok(())
    })
}

#[tauri::command]
pub fn chat_set_workspace_provider_preference(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    workspace_id: ChatWorkspaceId,
    instance_id: Option<ProviderInstanceId>,
) -> ChatResult<ChatVaultConfig> {
    mutate_chat_config(&app, &state, |config| {
        match instance_id {
            Some(instance_id) => {
                if !config
                    .providers
                    .iter()
                    .any(|provider| provider.instance_id == instance_id)
                {
                    return Err(provider_not_found());
                }
                config
                    .workspace_provider_preferences
                    .insert(workspace_id, instance_id);
            }
            None => {
                config.workspace_provider_preferences.remove(&workspace_id);
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn chat_remember_composer_selection(
    app: tauri::AppHandle,
    state: tauri::State<'_, ChatSettingsState>,
    selection: RememberedComposerSelection,
) -> ChatResult<ChatVaultConfig> {
    mutate_chat_config(&app, &state, |config| {
        if !config
            .providers
            .iter()
            .any(|provider| provider.instance_id == selection.provider_instance_id)
        {
            return Err(provider_not_found());
        }
        config.remembered_selections.retain(|existing| {
            existing.workspace_id != selection.workspace_id
                || existing.provider_instance_id != selection.provider_instance_id
        });
        config.remembered_selections.push(selection);
        Ok(())
    })
}

#[tauri::command]
pub fn chat_replace_credential(
    app: tauri::AppHandle,
    reference_id: CredentialReferenceId,
    secret: String,
) -> ChatResult<()> {
    let secret = SecretValue::new(secret).map_err(credential_error)?;
    PlatformCredentialStore::default()
        .replace(&reference_id, &secret)
        .map_err(credential_error)?;
    invalidate_provider_state_for_credential(&app, &reference_id)
}

#[tauri::command]
pub fn chat_remove_credential(
    app: tauri::AppHandle,
    reference_id: CredentialReferenceId,
) -> ChatResult<bool> {
    let removed = PlatformCredentialStore::default()
        .remove(&reference_id)
        .map_err(credential_error)?;
    invalidate_provider_state_for_credential(&app, &reference_id)?;
    Ok(removed)
}

#[tauri::command]
pub async fn chat_pick_provider_executable(
    app: tauri::AppHandle,
    title: String,
) -> ChatResult<Option<String>> {
    pick_local_path(&app, false, validate_picker_title(&title)?).await
}

#[tauri::command]
pub async fn chat_pick_provider_home(
    app: tauri::AppHandle,
    title: String,
) -> ChatResult<Option<String>> {
    pick_local_path(&app, true, validate_picker_title(&title)?).await
}

async fn pick_local_path(
    app: &tauri::AppHandle,
    directory: bool,
    title: &str,
) -> ChatResult<Option<String>> {
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    let picker = app.dialog().file().set_title(title);
    let callback = move |selection: Option<FilePath>| {
        let result = selection.map(file_path_to_local_path).transpose();
        let _ = sender.try_send(result);
    };
    if directory {
        picker.pick_folder(callback);
    } else {
        picker.pick_file(callback);
    }
    let selected = receiver.recv().await.ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::Internal,
            "Native picker did not respond",
            true,
        )
    })??;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let metadata = std::fs::metadata(&selected).map_err(|_| {
        ChatError::new(
            ChatErrorCode::NotFound,
            "Selected provider path is missing",
            true,
        )
    })?;
    if (directory && !metadata.is_dir()) || (!directory && !metadata.is_file()) {
        return Err(ChatError::validation(
            "providerPath",
            "Selected provider path has the wrong type",
        ));
    }
    let canonical = std::fs::canonicalize(selected).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Permission,
            "Selected provider path could not be resolved",
            true,
        )
    })?;
    canonical
        .to_str()
        .map(|value| Some(value.to_string()))
        .ok_or_else(|| ChatError::validation("providerPath", "Selected path is not supported"))
}

fn validate_picker_title(title: &str) -> ChatResult<&str> {
    let title = title.trim();
    if title.is_empty() || title.len() > 160 || title.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "title",
            "Native picker title is invalid",
        ));
    }
    Ok(title)
}

fn file_path_to_local_path(path: FilePath) -> ChatResult<PathBuf> {
    path.into_path()
        .map_err(|_| ChatError::validation("providerPath", "Selected path is not local"))
}

pub(crate) fn read_provider(
    app: &tauri::AppHandle,
    instance_id: &ProviderInstanceId,
) -> ChatResult<ProviderInstanceRead> {
    let config = read_chat_config(app)?;
    let portable = config
        .providers
        .iter()
        .find(|candidate| &candidate.instance_id == instance_id)
        .ok_or_else(provider_not_found)?;
    let scope = read_active_device_scope(app).map_err(device_state_error)?;
    Ok(provider_instance_read(portable, &scope.provider_instances))
}

fn provider_instance_read(
    portable: &ChatPortableProviderConfig,
    device_instances: &BTreeMap<ProviderInstanceId, ChatProviderDeviceState>,
) -> ProviderInstanceRead {
    let device = device_instances.get(&portable.instance_id);
    ProviderInstanceRead {
        configuration: ProviderInstanceConfig {
            schema_version: portable.schema_version,
            instance_id: portable.instance_id.clone(),
            family_id: portable.family_id.clone(),
            label: portable.label.clone(),
            accent_color: portable.accent_color.clone(),
            enabled: portable.enabled,
            executable: device
                .and_then(|entry| entry.executable_path.clone())
                .unwrap_or_default(),
            provider_home: device.and_then(|entry| entry.provider_home_path.clone()),
            launch_arguments: portable.launch_arguments.clone(),
            environment: portable.environment.clone(),
            credential_references: portable.credential_references.clone(),
            visible_model_ids: portable.visible_model_ids.clone(),
            favorite_model_ids: portable.favorite_model_ids.clone(),
            provider_config: portable.provider_config.clone(),
            unknown_fields: portable.unknown_fields.clone(),
        },
        last_probe: device.and_then(|entry| entry.last_probe.clone()),
        last_successful_probe_at: device.and_then(|entry| entry.last_successful_probe_at.clone()),
        model_catalog: device.and_then(|entry| entry.model_catalog.clone()),
    }
}

fn portable_configuration(config: &ProviderInstanceConfig) -> ChatPortableProviderConfig {
    ChatPortableProviderConfig {
        schema_version: config.schema_version,
        instance_id: config.instance_id.clone(),
        family_id: config.family_id.clone(),
        label: config.label.clone(),
        accent_color: config.accent_color.clone(),
        enabled: config.enabled,
        launch_arguments: config.launch_arguments.clone(),
        environment: config.environment.clone(),
        credential_references: config.credential_references.clone(),
        visible_model_ids: config.visible_model_ids.clone(),
        favorite_model_ids: config.favorite_model_ids.clone(),
        provider_config: config.provider_config.clone(),
        unknown_fields: config.unknown_fields.clone(),
    }
}

fn device_configuration(config: &ProviderInstanceConfig) -> ChatProviderDeviceState {
    ChatProviderDeviceState {
        executable_path: (!config.executable.trim().is_empty())
            .then(|| config.executable.trim().to_string()),
        provider_home_path: config
            .provider_home
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        last_probe: None,
        last_successful_probe_at: None,
        model_catalog: None,
    }
}

fn provider_runtime_changed(
    previous: &ProviderInstanceConfig,
    next: &ProviderInstanceConfig,
) -> bool {
    previous.family_id != next.family_id
        || previous.executable != next.executable
        || previous.provider_home != next.provider_home
        || previous.launch_arguments != next.launch_arguments
        || previous.environment != next.environment
        || previous.credential_references != next.credential_references
        || previous.provider_config != next.provider_config
}

fn invalidate_provider_state_for_credential(
    app: &tauri::AppHandle,
    reference_id: &CredentialReferenceId,
) -> ChatResult<()> {
    let affected = read_chat_config(app)?
        .providers
        .into_iter()
        .filter(|provider| {
            provider
                .credential_references
                .values()
                .any(|reference| reference == reference_id)
        })
        .map(|provider| provider.instance_id)
        .collect::<BTreeSet<_>>();
    if affected.is_empty() {
        return Ok(());
    }
    update_active_device_scope(app, |scope| {
        for instance_id in &affected {
            if let Some(provider) = scope.provider_instances.get_mut(instance_id) {
                provider.last_probe = None;
                if let Some(catalog) = provider.model_catalog.as_mut() {
                    catalog.stale = true;
                }
            }
        }
        Ok(())
    })
    .map_err(device_state_error)
}

fn mutate_chat_config(
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

fn read_chat_config(app: &tauri::AppHandle) -> ChatResult<ChatVaultConfig> {
    let raw = vault::vault_read_config(app.clone()).map_err(config_io_error)?;
    let root: Value = serde_json::from_str(&raw).map_err(|_| config_shape_error())?;
    parse_chat_config_branch(&root)
}

fn provider_mut<'a>(
    config: &'a mut ChatVaultConfig,
    instance_id: &ProviderInstanceId,
) -> ChatResult<&'a mut ChatPortableProviderConfig> {
    config
        .providers
        .iter_mut()
        .find(|provider| &provider.instance_id == instance_id)
        .ok_or_else(provider_not_found)
}

fn unique_model_ids(values: Vec<ModelId>) -> Vec<ModelId> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.as_str().to_string()))
        .collect()
}

fn operation_context(operation_id: &str) -> DriverOperationContext {
    DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline: Instant::now() + PROVIDER_OPERATION_TIMEOUT,
        cancellation: DriverCancellation::default(),
    }
}

fn provider_not_found() -> ChatError {
    ChatError::new(
        ChatErrorCode::NotFound,
        "Chat provider instance was not found",
        true,
    )
}

fn device_state_error(_error: String) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be updated",
        true,
    )
}

fn config_io_error(_error: String) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat settings could not be persisted",
        true,
    )
}

fn config_shape_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat settings file is invalid",
        false,
    )
}

fn credential_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat credential operation failed",
        true,
    )
}
