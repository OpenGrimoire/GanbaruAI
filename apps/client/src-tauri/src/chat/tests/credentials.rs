use crate::chat::{
    credentials::{
        CredentialStore, CredentialStoreAvailability, CredentialStoreError,
        CredentialStoreErrorCode, CredentialStoreOperation, SecretValue,
    },
    models::{CredentialReferenceId, ProviderInstanceConfig},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

const SENTINEL_SECRET: &str = "ganbaru-chat-secret-sentinel-47b1";

#[derive(Clone, Debug, Default)]
struct MemoryCredentialStore {
    values: Arc<Mutex<BTreeMap<String, String>>>,
}

impl CredentialStore for MemoryCredentialStore {
    fn availability(&self) -> CredentialStoreAvailability {
        CredentialStoreAvailability::Available
    }

    fn read(
        &self,
        reference: &CredentialReferenceId,
    ) -> Result<Option<SecretValue>, CredentialStoreError> {
        self.values
            .lock()
            .expect("memory credential mutex")
            .get(reference.as_str())
            .cloned()
            .map(SecretValue::new)
            .transpose()
    }

    fn replace(
        &self,
        reference: &CredentialReferenceId,
        value: &SecretValue,
    ) -> Result<(), CredentialStoreError> {
        self.values
            .lock()
            .expect("memory credential mutex")
            .insert(reference.to_string(), value.expose().to_string());
        Ok(())
    }

    fn remove(&self, reference: &CredentialReferenceId) -> Result<bool, CredentialStoreError> {
        Ok(self
            .values
            .lock()
            .expect("memory credential mutex")
            .remove(reference.as_str())
            .is_some())
    }
}

#[test]
fn credential_store_replaces_reads_and_removes_by_opaque_reference() {
    let store = MemoryCredentialStore::default();
    let reference = CredentialReferenceId::new("credential-1").unwrap();
    let first = SecretValue::new("first-secret").unwrap();
    let replacement = SecretValue::new("replacement-secret").unwrap();

    assert_eq!(store.availability(), CredentialStoreAvailability::Available);
    assert_eq!(store.read(&reference).unwrap(), None);
    store.replace(&reference, &first).unwrap();
    assert_eq!(store.read(&reference).unwrap(), Some(first));
    store.replace(&reference, &replacement).unwrap();
    assert_eq!(store.read(&reference).unwrap(), Some(replacement));
    assert!(store.remove(&reference).unwrap());
    assert!(!store.remove(&reference).unwrap());
}

#[test]
fn secret_values_and_errors_format_without_secret_material() {
    let value = SecretValue::new(SENTINEL_SECRET).unwrap();
    let error = CredentialStoreError {
        code: CredentialStoreErrorCode::AccessDenied,
        operation: CredentialStoreOperation::Read,
    };
    let formatted = format!("{value:?} {error:?} {error}");

    assert!(!formatted.contains(SENTINEL_SECRET));
    assert!(formatted.contains("REDACTED"));
}

#[test]
fn serialized_provider_configuration_contains_only_opaque_references() {
    let store = MemoryCredentialStore::default();
    let reference = CredentialReferenceId::new("credential-1").unwrap();
    let secret = SecretValue::new(SENTINEL_SECRET).unwrap();
    store.replace(&reference, &secret).unwrap();
    let input = serde_json::json!({
        "schemaVersion": 1,
        "instanceId": "provider-instance-1",
        "familyId": "codex",
        "label": "Local provider",
        "accentColor": null,
        "enabled": true,
        "executable": "codex",
        "providerHome": null,
        "launchArguments": [],
        "environment": {},
        "credentialReferences": { "API_TOKEN": "credential-1" },
        "visibleModelIds": [],
        "favoriteModelIds": [],
        "providerConfig": { "schemaVersion": 1, "value": {} }
    });
    let configuration: ProviderInstanceConfig = serde_json::from_value(input).unwrap();
    let serialized = serde_json::to_string(&configuration).unwrap();
    let diagnostic = format!("{configuration:?} {:?}", store.read(&reference).unwrap());

    assert!(!serialized.contains(SENTINEL_SECRET));
    assert!(!diagnostic.contains(SENTINEL_SECRET));
    assert!(serialized.contains("credential-1"));
}
