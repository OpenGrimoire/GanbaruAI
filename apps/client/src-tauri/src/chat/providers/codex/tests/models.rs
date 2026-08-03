use super::*;

#[test]
fn custom_model_keeps_its_optional_display_label() {
    let model = custom_provider_model("exact-model-id", Some("Team model")).unwrap();

    assert_eq!(model.id.as_str(), "exact-model-id");
    assert_eq!(model.display_name, "Team model");
    assert!(model.custom);
    assert_eq!(model.availability, ModelAvailability::Unknown);
}

#[test]
fn safety_modes_map_to_exact_codex_policies() {
    assert_eq!(
        safety_settings(SafetyMode::AskForApproval),
        Some(CodexSafetySettings {
            approval_policy: "on-request",
            approvals_reviewer: "user",
            sandbox: "workspace-write",
            turn_sandbox_type: "workspaceWrite",
        })
    );
    assert_eq!(
        safety_settings(SafetyMode::ApproveForMe),
        Some(CodexSafetySettings {
            approval_policy: "on-request",
            approvals_reviewer: "auto_review",
            sandbox: "workspace-write",
            turn_sandbox_type: "workspaceWrite",
        })
    );
    assert_eq!(
        safety_settings(SafetyMode::FullAccess),
        Some(CodexSafetySettings {
            approval_policy: "never",
            approvals_reviewer: "user",
            sandbox: "danger-full-access",
            turn_sandbox_type: "dangerFullAccess",
        })
    );
    assert_eq!(safety_settings(SafetyMode::Custom), None);

    let custom = thread_open_params(
        None,
        Path::new("/workspace"),
        modes(SafetyMode::Custom, InteractionMode::Build),
        None,
        None,
    )
    .unwrap();
    assert!(custom.get("approvalPolicy").is_none());
    assert!(custom.get("approvalsReviewer").is_none());
    assert!(custom.get("sandbox").is_none());
}

#[test]
fn turn_builder_preserves_model_traits_modes_and_verified_images() {
    let workspace = TestDirectory::new("turn-builder");
    fs::write(workspace.path().join("prompt.png"), b"redacted image").unwrap();
    let request: SendTurnRequest = serde_json::from_value(json!({
        "command": { "clientCommandId": "command-1", "expectedThreadRevision": 3 },
        "sessionId": "session-1",
        "turnId": "chat-turn-1",
        "prompt": "Implement the change",
        "attachments": [{
            "attachmentId": "attachment-1",
            "kind": "image",
            "displayName": "prompt.png",
            "managedRelativePath": "prompt.png",
            "resourceUri": "ganbaru://chat/resource/attachment-1",
            "mimeType": "image/png",
            "byteSize": 14,
            "localPath": workspace.path().join("prompt.png").to_string_lossy(),
            "textContent": null
        }],
        "mentions": [],
        "modelId": "gpt-5.4",
        "modelOptions": [
            { "key": "reasoning_effort", "value": { "kind": "choice", "value": "high" } },
            { "key": "service_tier", "value": { "kind": "choice", "value": "fast" } }
        ],
        "modes": { "safetyMode": "approve_for_me", "interactionMode": "plan" },
        "developerInstructions": "Use the repository conventions."
    }))
    .unwrap();
    let params = turn_start_params(
        "provider-thread-1",
        workspace.path(),
        "fallback-model",
        &request,
        None,
    )
    .unwrap();

    assert_eq!(params["approvalPolicy"], "on-request");
    assert_eq!(params["approvalsReviewer"], "auto_review");
    assert_eq!(params["sandboxPolicy"]["type"], "workspaceWrite");
    assert_eq!(params["sandboxPolicy"]["writableRoots"], json!([]));
    assert_eq!(params["sandboxPolicy"]["networkAccess"], false);
    assert_eq!(params["model"], "gpt-5.4");
    assert_eq!(params["effort"], "high");
    assert_eq!(params["serviceTier"], "fast");
    assert_eq!(params["collaborationMode"]["mode"], "plan");
    assert_eq!(
        params["collaborationMode"]["settings"]["developer_instructions"],
        "Use the repository conventions."
    );
    assert_eq!(params["input"][0]["type"], "text");
    assert_eq!(params["input"][1]["type"], "localImage");
    assert_eq!(
        params["input"][1]["path"],
        workspace
            .path()
            .join("prompt.png")
            .to_string_lossy()
            .as_ref()
    );

    let mut standard_request = request.clone();
    standard_request.model_options[1].value = ModelOptionValue::Choice("standard".to_string());
    let standard_params = turn_start_params(
        "provider-thread-1",
        workspace.path(),
        "fallback-model",
        &standard_request,
        None,
    )
    .unwrap();
    assert!(standard_params.get("serviceTier").is_none());
    assert!(params["input"][1].get("url").is_none());

    let mut custom_request = request.clone();
    custom_request.modes.safety_mode = SafetyMode::Custom;
    let custom_safety = CodexCustomSafetySettings {
        approval_policy: json!("on-request"),
        approvals_reviewer: "user".to_string(),
        sandbox_policy: None,
        permissions: Some("project-edit".to_string()),
    };
    let custom_params = turn_start_params(
        "provider-thread-1",
        workspace.path(),
        "fallback-model",
        &custom_request,
        Some(&custom_safety),
    )
    .unwrap();
    assert_eq!(custom_params["permissions"], "project-edit");
    assert!(custom_params.get("sandboxPolicy").is_none());

    let mut escaping = request;
    escaping.attachments[0].local_path = Some("../outside.png".to_string());
    assert!(turn_start_params(
        "provider-thread-1",
        workspace.path(),
        "fallback-model",
        &escaping,
        None,
    )
    .is_err());
}

#[test]
fn custom_permissions_resolve_profiles_and_legacy_sandbox_settings() {
    let profile = custom_safety_settings(ConfigReadResponse {
        config: json!({
            "approval_policy": "on-request",
            "approvals_reviewer": "auto_review",
            "default_permissions": "project-edit"
        }),
    })
    .unwrap();
    assert_eq!(profile.permissions.as_deref(), Some("project-edit"));
    assert_eq!(profile.sandbox_policy, None);

    let writable_root = TestDirectory::new("custom-writable-root");
    let writable_root_value = writable_root.path().to_string_lossy().into_owned();
    let legacy = custom_safety_settings(ConfigReadResponse {
        config: json!({
            "approval_policy": "untrusted",
            "approvals_reviewer": "user",
            "sandbox_mode": "workspace-write",
            "sandbox_workspace_write": {
                "writable_roots": [writable_root_value],
                "network_access": true,
                "exclude_tmpdir_env_var": true,
                "exclude_slash_tmp": false
            }
        }),
    })
    .unwrap();
    assert_eq!(legacy.permissions, None);
    assert_eq!(
        legacy.sandbox_policy.as_ref().unwrap()["type"],
        "workspaceWrite"
    );
    assert_eq!(
        legacy.sandbox_policy.as_ref().unwrap()["networkAccess"],
        true
    );
    assert_eq!(
        legacy.sandbox_policy.as_ref().unwrap()["writableRoots"],
        json!([writable_root.path().to_string_lossy()])
    );
}

#[test]
fn model_catalog_keeps_options_and_image_capability() {
    let model: CodexModel = serde_json::from_value(json!({
        "id": "gpt-5.4",
        "model": "gpt-5.4",
        "displayName": "GPT-5.4",
        "description": "Current coding model",
        "hidden": false,
        "isDefault": true,
        "defaultReasoningEffort": "medium",
        "supportedReasoningEfforts": [
            { "reasoningEffort": "medium", "description": "Balanced" },
            { "reasoningEffort": "high", "description": "Deeper" }
        ],
        "inputModalities": ["text", "image"],
        "serviceTiers": [{ "id": "fast", "name": "Fast", "description": "Low latency" }],
        "defaultServiceTier": "fast",
        "supportsPersonality": true,
        "upgrade": null
    }))
    .unwrap();
    let model = provider_model(model).unwrap();

    assert_eq!(model.id.as_str(), "gpt-5.4");
    assert!(model.capabilities.contains(&ProviderCapability::Images));
    assert!(model.capabilities.contains(&ProviderCapability::NativePlan));
    assert!(model
        .capabilities
        .contains(&ProviderCapability::StructuredPlans));
    assert_eq!(model.options.len(), 2);
    let ModelOptionDefinition::Choice {
        options,
        default_value,
        ..
    } = &model.options[1]
    else {
        panic!("service tier must be a choice option");
    };
    assert_eq!(
        options
            .iter()
            .map(|option| option.value.as_str())
            .collect::<Vec<_>>(),
        ["standard", "fast"]
    );
    assert_eq!(default_value.as_deref(), Some("fast"));
}
