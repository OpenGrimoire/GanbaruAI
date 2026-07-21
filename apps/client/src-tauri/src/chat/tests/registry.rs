use crate::chat::{
    models::{ProviderImplementationStatus, ProviderInstanceConfig},
    providers::{ProviderDriverFactory, ProviderDriverRegistry},
};
use serde_json::json;
use std::time::{Duration, Instant};

fn configuration(family_id: &str) -> ProviderInstanceConfig {
    serde_json::from_value(json!({
        "schemaVersion": 1,
        "instanceId": "provider-instance-1",
        "familyId": family_id,
        "label": "Local provider",
        "accentColor": null,
        "enabled": true,
        "executable": family_id,
        "providerHome": null,
        "launchArguments": [],
        "environment": {},
        "credentialReferences": {},
        "visibleModelIds": [],
        "favoriteModelIds": [],
        "providerConfig": { "schemaVersion": 7, "value": { "future": true } },
        "futureCommonField": { "nested": [1, 2, 3] }
    }))
    .unwrap()
}

#[test]
fn registry_lists_four_known_families_in_stable_order() {
    let metadata = ProviderDriverRegistry.list_metadata();
    let family_ids = metadata
        .iter()
        .map(|entry| entry.family_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(family_ids, ["codex", "claude", "cursor", "opencode"]);
    assert_eq!(
        metadata[0].implementation_status,
        ProviderImplementationStatus::Available
    );
    assert!(metadata[0].unavailable_reason.is_none());
    assert!(metadata[1..].iter().all(|entry| {
        entry.implementation_status == ProviderImplementationStatus::MetadataOnly
            && entry.unavailable_reason.is_some()
    }));
}

#[test]
fn unknown_family_and_configuration_round_trip_without_loss() {
    let input = serde_json::to_value(configuration("future-provider")).unwrap();
    let parsed: ProviderInstanceConfig = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(&parsed).unwrap(), input);

    let metadata = ProviderDriverRegistry.metadata(&parsed.family_id);
    assert_eq!(metadata.family_id.as_str(), "future-provider");
    assert_eq!(
        metadata.implementation_status,
        ProviderImplementationStatus::Unsupported
    );

    let driver = ProviderDriverRegistry.create_driver(parsed).unwrap();
    assert_eq!(
        driver.instance_configuration().family_id.as_str(),
        "future-provider"
    );
    assert_eq!(
        driver
            .instance_configuration()
            .unknown_fields
            .get("futureCommonField"),
        Some(&json!({ "nested": [1, 2, 3] }))
    );
}

#[test]
fn codex_driver_is_available_with_declared_capabilities() {
    let mut configuration = configuration("codex");
    configuration.executable = "ganbaru-nonexistent-codex-executable".to_string();
    configuration.provider_config = crate::chat::models::VersionedJson {
        schema_version: 1,
        value: json!({}),
    };
    let mut driver = ProviderDriverRegistry.create_driver(configuration).unwrap();
    assert!(!driver.capabilities().entries.is_empty());
    assert!(driver
        .capabilities()
        .entries
        .iter()
        .all(|entry| entry.supported));

    let context = crate::chat::providers::DriverOperationContext {
        operation_id: "probe-1".to_string(),
        deadline: Instant::now() + Duration::from_secs(1),
        cancellation: Default::default(),
    };
    let result = tauri::async_runtime::block_on(driver.probe(&context)).unwrap();
    assert_eq!(
        result.state,
        crate::chat::models::ProbeState::ExecutableMissing
    );
}
