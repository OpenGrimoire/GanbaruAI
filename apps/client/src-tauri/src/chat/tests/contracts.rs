use crate::chat::{
    events::{CanonicalEvent, ContentDeltaEvent},
    models::{ChatThreadId, ContentStreamKind, ModelOptionDefinition, SafetyMode, UtcTimestamp},
};
use serde_json::json;

#[test]
fn identifiers_reject_empty_oversized_and_control_values() {
    assert!(ChatThreadId::new("").is_err());
    assert!(ChatThreadId::new("thread\n1").is_err());
    assert!(ChatThreadId::new("x".repeat(1_025)).is_err());
    assert_eq!(ChatThreadId::new("thread-1").unwrap().as_str(), "thread-1");
}

#[test]
fn timestamps_require_rfc3339_utc_values() {
    assert!(UtcTimestamp::new("2026-07-20T12:00:00Z").is_ok());
    assert!(UtcTimestamp::new("2026-07-20T06:00:00-06:00").is_err());
    assert!(UtcTimestamp::new("2026-07-20 12:00:00").is_err());
}

#[test]
fn safety_modes_keep_normative_wire_literals() {
    assert_eq!(
        serde_json::to_value(SafetyMode::AutoAcceptEdits).unwrap(),
        json!("auto_accept_edits")
    );
}

#[test]
fn canonical_events_use_a_stable_tagged_shape() {
    let event = CanonicalEvent::ContentDelta(ContentDeltaEvent {
        item_id: "assistant-1".to_string(),
        stream_kind: ContentStreamKind::AssistantText,
        content_index: 0,
        delta: "Hello".to_string(),
    });

    assert_eq!(
        serde_json::to_value(event).unwrap(),
        json!({
            "type": "content_delta",
            "payload": {
                "itemId": "assistant-1",
                "streamKind": "assistant_text",
                "contentIndex": 0,
                "delta": "Hello"
            }
        })
    );
}

#[test]
fn unknown_model_options_preserve_bounded_canonical_data() {
    let input = json!({
        "kind": "unknown",
        "key": "futureTrait",
        "label": "Future trait",
        "rawKind": "distribution",
        "schemaVersion": 3,
        "data": { "shape": "balanced", "providerField": [1, 2] }
    });
    let option: ModelOptionDefinition = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(option).unwrap(), input);
}
