//! Device-local bindings and selection state for project working folders.

use crate::chat::models::{ProjectWorkingFolderId, RepositoryKind, UtcTimestamp};
use crate::vault::{active_vault_id, read_app_state, update_app_state, vault_device_id};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tauri::Runtime;

pub const WORKING_FOLDER_DEVICE_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderBindingState {
    pub canonical_path: String,
    pub repository_kind: RepositoryKind,
    pub repository_identity: Option<String>,
    pub last_verified_at: UtcTimestamp,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingFolderDeviceScope {
    #[serde(default)]
    pub bindings: BTreeMap<ProjectWorkingFolderId, ProjectWorkingFolderBindingState>,
    #[serde(default)]
    pub last_selected_by_project: BTreeMap<String, ProjectWorkingFolderId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingFolderDeviceState {
    pub schema_version: u32,
    #[serde(default)]
    pub vaults: BTreeMap<String, BTreeMap<String, WorkingFolderDeviceScope>>,
}

impl Default for WorkingFolderDeviceState {
    fn default() -> Self {
        Self {
            schema_version: WORKING_FOLDER_DEVICE_STATE_SCHEMA_VERSION,
            vaults: BTreeMap::new(),
        }
    }
}

impl WorkingFolderDeviceState {
    pub(crate) fn scope(
        &self,
        vault_id: &str,
        device_id: &str,
    ) -> Option<&WorkingFolderDeviceScope> {
        self.vaults
            .get(vault_id)
            .and_then(|devices| devices.get(device_id))
    }

    pub(crate) fn scope_mut(
        &mut self,
        vault_id: &str,
        device_id: &str,
    ) -> &mut WorkingFolderDeviceScope {
        self.vaults
            .entry(vault_id.to_string())
            .or_default()
            .entry(device_id.to_string())
            .or_default()
    }
}

pub fn read_active_working_folder_scope<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<WorkingFolderDeviceScope, String> {
    let vault_id = active_vault_id(app)?;
    let device_id = vault_device_id(app.clone())?;
    let state = read_app_state(app)?;
    Ok(state
        .project_working_folders
        .scope(&vault_id, &device_id)
        .cloned()
        .unwrap_or_default())
}

pub fn update_active_working_folder_scope<R: Runtime, T>(
    app: &tauri::AppHandle<R>,
    update: impl FnOnce(&mut WorkingFolderDeviceScope) -> Result<T, String>,
) -> Result<T, String> {
    let vault_id = active_vault_id(app)?;
    let device_id = vault_device_id(app.clone())?;
    update_app_state(app, |state| {
        if state.project_working_folders.schema_version
            != WORKING_FOLDER_DEVICE_STATE_SCHEMA_VERSION
        {
            return Err("project working-folder device state is unsupported".to_string());
        }
        update(
            state
                .project_working_folders
                .scope_mut(&vault_id, &device_id),
        )
    })
}
