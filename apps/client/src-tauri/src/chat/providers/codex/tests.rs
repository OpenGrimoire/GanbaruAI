use super::driver::{
    confirmed_resume_not_found, validated_app_server_arguments, CodexProviderDriver,
};
use super::home::*;
use super::normalizer::{CodexEventNormalizer, CodexRouteState};
use super::protocol::*;
use super::session::{
    resolve_codex_approval, resolve_codex_user_input, CodexApprovalResponse, PendingCodexRequest,
    PendingCodexRequestKind, PendingCodexRequests,
};
use super::transport::{CodexInboundMessage, CodexRpcConnection, CodexRpcFailure};
use crate::chat::events::{CanonicalEvent, CanonicalRuntimeEvent};
use crate::chat::models::*;
use crate::chat::providers::{
    DriverCancellation, DriverFuture, DriverOperationContext, ProviderDriver, ProviderEventSink,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(1);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "ganbaru-codex-{label}-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct RecordingSink {
    events: Mutex<Vec<CanonicalRuntimeEvent>>,
}

impl RecordingSink {
    fn events(&self) -> Vec<CanonicalRuntimeEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl ProviderEventSink for RecordingSink {
    fn emit<'a>(&'a self, event: CanonicalRuntimeEvent) -> DriverFuture<'a, ()> {
        Box::pin(async move {
            self.events.lock().unwrap().push(event);
            Ok(())
        })
    }
}

fn context(operation_id: &str) -> DriverOperationContext {
    DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline: Instant::now() + Duration::from_secs(2),
        cancellation: DriverCancellation::default(),
    }
}

fn identifier<T>(value: &str, constructor: impl FnOnce(String) -> Result<T, String>) -> T {
    constructor(value.to_string()).unwrap()
}

fn configuration(shared_home: &Path, shadow_home: Option<&Path>) -> ProviderInstanceConfig {
    serde_json::from_value(json!({
        "schemaVersion": 1,
        "instanceId": "codex-instance-1",
        "familyId": "codex",
        "label": "Codex",
        "accentColor": null,
        "enabled": true,
        "executable": "codex",
        "providerHome": shared_home,
        "launchArguments": [],
        "environment": {},
        "credentialReferences": {},
        "visibleModelIds": [],
        "favoriteModelIds": [],
        "providerConfig": {
            "schemaVersion": 1,
            "value": {
                "shadowHomePath": shadow_home,
                "refreshMcpBeforeTurn": true,
                "allowCustomModels": false,
                "customModelIds": [],
                "customModelLabels": {}
            }
        }
    }))
    .unwrap()
}

#[test]
fn custom_model_keeps_its_optional_display_label() {
    let model = custom_provider_model("exact-model-id", Some("Team model")).unwrap();

    assert_eq!(model.id.as_str(), "exact-model-id");
    assert_eq!(model.display_name, "Team model");
    assert!(model.custom);
    assert_eq!(model.availability, ModelAvailability::Unknown);
}

fn modes(safety_mode: SafetyMode, interaction_mode: InteractionMode) -> TurnModeSnapshot {
    TurnModeSnapshot {
        safety_mode,
        interaction_mode,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FixtureScenario {
    Healthy,
    ResumeMissing,
    AuthenticationRequired,
    MalformedInitialize,
    CommandApproval,
    StructuredQuestion,
}

#[derive(Default)]
struct AppServerFixtureState {
    received: Mutex<Vec<Value>>,
    closed: AtomicBool,
}

impl AppServerFixtureState {
    fn received(&self) -> Vec<Value> {
        self.received.lock().unwrap().clone()
    }
}

fn fixture_driver(
    workspace: &Path,
    scenario: FixtureScenario,
) -> (CodexProviderDriver, Arc<AppServerFixtureState>) {
    let config = configuration(workspace, None);
    let mut driver = CodexProviderDriver::new(config).unwrap();
    let state = Arc::new(AppServerFixtureState::default());
    let factory_state = Arc::clone(&state);
    let home = workspace.to_path_buf();
    driver.set_connection_factory(Arc::new(move |_working_directory| {
        let (client_reader, server_writer) = tokio::io::duplex(64 * 1024);
        let (server_reader, client_writer) = tokio::io::duplex(64 * 1024);
        tokio::spawn(run_app_server_fixture(
            server_reader,
            server_writer,
            home.clone(),
            scenario,
            Arc::clone(&factory_state),
        ));
        Ok((
            CodexRpcConnection::from_test_io(client_reader, client_writer),
            CodexHomeLayout {
                shared_home: home.clone(),
                effective_home: home.clone(),
                shadowed: false,
            },
        ))
    }));
    (driver, state)
}

async fn write_fixture_message<W>(writer: &mut W, message: Value)
where
    W: AsyncWrite + Unpin,
{
    let mut bytes = serde_json::to_vec(&message).unwrap();
    bytes.push(b'\n');
    writer.write_all(&bytes).await.unwrap();
    writer.flush().await.unwrap();
}

async fn run_app_server_fixture<R, W>(
    reader: R,
    mut writer: W,
    home: PathBuf,
    scenario: FixtureScenario,
    state: Arc<AppServerFixtureState>,
) where
    R: tokio::io::AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut resume_failed = false;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let message: Value = serde_json::from_str(&line).unwrap();
        state.received.lock().unwrap().push(message.clone());
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            continue;
        };
        let Some(id) = message.get("id").cloned() else {
            continue;
        };
        match method {
            "initialize" if scenario == FixtureScenario::MalformedInitialize => {
                write_fixture_message(
                    &mut writer,
                    json!({ "id": id, "result": { "invalid": true } }),
                )
                .await;
            }
            "initialize" => {
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": {
                            "userAgent": "codex-cli/0.144.6",
                            "codexHome": home,
                            "platformFamily": "unix",
                            "platformOs": "linux"
                        }
                    }),
                )
                .await;
            }
            "account/read" => {
                let authentication_required = scenario == FixtureScenario::AuthenticationRequired;
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": {
                            "account": if authentication_required {
                                Value::Null
                            } else {
                                json!({ "type": "chatgpt", "email": "redacted@example.test", "planType": "test" })
                            },
                            "requiresOpenaiAuth": true
                        }
                    }),
                )
                .await;
            }
            "model/list" => {
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": {
                            "data": [{
                                "id": "gpt-5.4",
                                "model": "gpt-5.4",
                                "displayName": "GPT-5.4",
                                "description": "Fixture model",
                                "hidden": false,
                                "isDefault": true,
                                "defaultReasoningEffort": "medium",
                                "supportedReasoningEfforts": [{
                                    "reasoningEffort": "medium",
                                    "description": "Balanced"
                                }],
                                "inputModalities": ["text", "image"],
                                "serviceTiers": [],
                                "defaultServiceTier": null,
                                "supportsPersonality": false,
                                "upgrade": null
                            }],
                            "nextCursor": null
                        }
                    }),
                )
                .await;
            }
            "config/read" => {
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": {
                            "config": {
                                "approval_policy": "on-request",
                                "approvals_reviewer": "user",
                                "sandbox_mode": "workspace-write",
                                "sandbox_workspace_write": {
                                    "writable_roots": [],
                                    "network_access": false,
                                    "exclude_tmpdir_env_var": false,
                                    "exclude_slash_tmp": false
                                }
                            }
                        }
                    }),
                )
                .await;
            }
            "thread/resume" if scenario == FixtureScenario::ResumeMissing && !resume_failed => {
                resume_failed = true;
                write_fixture_message(
                    &mut writer,
                    json!({ "id": id, "error": { "code": -32602, "message": "Thread not found" } }),
                )
                .await;
            }
            "thread/start" | "thread/resume" => {
                let thread_id = message["params"]["threadId"]
                    .as_str()
                    .unwrap_or("fixture-thread-new");
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": {
                            "thread": { "id": thread_id },
                            "model": "gpt-5.4",
                            "approvalPolicy": message["params"]["approvalPolicy"].as_str().unwrap_or("on-request"),
                            "approvalsReviewer": message["params"]["approvalsReviewer"].as_str().unwrap_or("user"),
                            "sandbox": {
                                "type": match message["params"]["sandbox"].as_str() {
                                    Some("read-only") => "readOnly",
                                    Some("workspace-write") => "workspaceWrite",
                                    Some("danger-full-access") => "dangerFullAccess",
                                    _ => "workspaceWrite"
                                }
                            }
                        }
                    }),
                )
                .await;
                write_fixture_message(
                    &mut writer,
                    json!({
                        "method": "thread/started",
                        "params": { "thread": { "id": thread_id, "name": "Fixture provider title" } }
                    }),
                )
                .await;
            }
            "config/mcpServer/reload" => {
                write_fixture_message(&mut writer, json!({ "id": id, "result": {} })).await;
            }
            "turn/start" => {
                write_fixture_message(
                    &mut writer,
                    json!({
                        "id": id,
                        "result": { "turn": { "id": "fixture-provider-turn", "status": "inProgress" } }
                    }),
                )
                .await;
                write_fixture_message(
                    &mut writer,
                    json!({
                        "method": "turn/started",
                        "params": {
                            "threadId": message["params"]["threadId"],
                            "turn": { "id": "fixture-provider-turn", "status": "inProgress" }
                        }
                    }),
                )
                .await;
                if scenario == FixtureScenario::CommandApproval {
                    write_fixture_message(
                        &mut writer,
                        json!({
                            "id": "fixture-approval",
                            "method": "item/commandExecution/requestApproval",
                            "params": {
                                "threadId": message["params"]["threadId"],
                                "turnId": "fixture-provider-turn",
                                "itemId": "fixture-command-item",
                                "command": "pnpm test",
                                "reason": "Run focused tests"
                            }
                        }),
                    )
                    .await;
                }
                if scenario == FixtureScenario::StructuredQuestion {
                    write_fixture_message(
                        &mut writer,
                        json!({
                            "id": "fixture-question",
                            "method": "item/tool/requestUserInput",
                            "params": {
                                "threadId": message["params"]["threadId"],
                                "turnId": "fixture-provider-turn",
                                "itemId": "fixture-question-item",
                                "questions": [{
                                    "id": "strategy",
                                    "header": "Strategy",
                                    "question": "Which strategy should be used?",
                                    "isOther": true,
                                    "isSecret": false,
                                    "options": [{ "label": "Focused", "description": "Small scope" }]
                                }]
                            }
                        }),
                    )
                    .await;
                }
            }
            "turn/interrupt" => {
                write_fixture_message(&mut writer, json!({ "id": id, "result": {} })).await;
                write_fixture_message(
                    &mut writer,
                    json!({
                        "method": "turn/completed",
                        "params": {
                            "threadId": message["params"]["threadId"],
                            "turn": { "id": message["params"]["turnId"], "status": "interrupted", "items": [] }
                        }
                    }),
                )
                .await;
            }
            "thread/read" => {
                write_fixture_message(
                    &mut writer,
                    json!({ "id": id, "result": { "thread": { "turns": [] } } }),
                )
                .await;
            }
            _ => {
                write_fixture_message(&mut writer, json!({ "id": id, "result": {} })).await;
            }
        }
    }
    state.closed.store(true, Ordering::Release);
}

fn start_request(workspace: &Path) -> StartSessionRequest {
    serde_json::from_value(json!({
        "threadId": "chat-thread-fixture",
        "workspace": {
            "workspaceId": "workspace-fixture",
            "canonicalPath": workspace,
            "repositoryKind": "none",
            "repositoryIdentity": null
        },
        "providerInstanceId": "codex-instance-1",
        "modes": { "safetyMode": "ask_for_approval", "interactionMode": "build" },
        "modelId": "gpt-5.4",
        "modelOptions": []
    }))
    .unwrap()
}

fn fixture_turn(session_id: &ProviderSessionId, interaction_mode: &str) -> SendTurnRequest {
    serde_json::from_value(json!({
        "command": { "clientCommandId": "fixture-send", "expectedThreadRevision": 2 },
        "sessionId": session_id,
        "turnId": "chat-turn-fixture",
        "prompt": "Implement the fixture change",
        "attachments": [],
        "mentions": [],
        "modelId": "gpt-5.4",
        "modelOptions": [{
            "key": "reasoning_effort",
            "value": { "kind": "choice", "value": "high" }
        }],
        "modes": { "safetyMode": "ask_for_approval", "interactionMode": interaction_mode },
        "developerInstructions": "Fixture instructions"
    }))
    .unwrap()
}

async fn wait_for_event(
    sink: &RecordingSink,
    predicate: impl Fn(&CanonicalRuntimeEvent) -> bool,
) -> CanonicalRuntimeEvent {
    for _ in 0..100 {
        if let Some(event) = sink.events().into_iter().find(&predicate) {
            return event;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("fixture event was not emitted");
}

async fn wait_for_fixture_message(
    fixture: &AppServerFixtureState,
    predicate: impl Fn(&Value) -> bool,
) -> Value {
    for _ in 0..100 {
        if let Some(message) = fixture.received().into_iter().find(&predicate) {
            return message;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("fixture message was not received");
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
    assert!(params["input"][1]["path"]
        .as_str()
        .unwrap()
        .starts_with(workspace.path().to_str().unwrap()));

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

#[test]
fn launch_arguments_are_configuration_only() {
    assert_eq!(
        validated_app_server_arguments(&[
            "--strict-config".to_string(),
            "--enable".to_string(),
            "responses_websockets".to_string(),
            "-c=model=redacted".to_string(),
        ])
        .unwrap()
        .len(),
        4
    );
    assert!(validated_app_server_arguments(&["exec".to_string()]).is_err());
    assert!(validated_app_server_arguments(&[
        "--config".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
    ])
    .is_err());
}

#[test]
fn only_confirmed_thread_not_found_errors_allow_fresh_fallback() {
    assert!(confirmed_resume_not_found(&CodexRpcFailure::Remote {
        code: -32602,
        message: "Thread not found".to_string(),
    }));
    assert!(!confirmed_resume_not_found(&CodexRpcFailure::Remote {
        code: -32602,
        message: "Authentication failed".to_string(),
    }));
    assert!(!confirmed_resume_not_found(&CodexRpcFailure::Closed));
}

#[test]
fn direct_and_shadow_homes_share_continuation_identity() {
    let shared = TestDirectory::new("shared-home");
    let shadow = TestDirectory::new("shadow-home");
    let direct_config = configuration(shared.path(), None);
    let shadow_config = configuration(shared.path(), Some(shadow.path()));
    let direct = resolve_codex_home_layout(
        &direct_config,
        &CodexProviderSettings::parse(&direct_config).unwrap(),
    )
    .unwrap();
    let shadowed = resolve_codex_home_layout(
        &shadow_config,
        &CodexProviderSettings::parse(&shadow_config).unwrap(),
    )
    .unwrap();

    assert_eq!(
        direct.continuation_group().unwrap(),
        shadowed.continuation_group().unwrap()
    );
    let other = TestDirectory::new("other-home");
    let other_config = configuration(other.path(), None);
    let other_layout = resolve_codex_home_layout(
        &other_config,
        &CodexProviderSettings::parse(&other_config).unwrap(),
    )
    .unwrap();
    assert_ne!(
        direct.continuation_group().unwrap(),
        other_layout.continuation_group().unwrap()
    );
}

#[cfg(unix)]
#[test]
fn shadow_home_links_shared_state_but_keeps_auth_private() {
    let shared = TestDirectory::new("materialized-shared");
    let shadow_parent = TestDirectory::new("materialized-shadow");
    let shadow = shadow_parent.path().join("effective");
    fs::write(shared.path().join("config.toml"), b"model = 'redacted'").unwrap();
    let config = configuration(shared.path(), Some(&shadow));
    let settings = CodexProviderSettings::parse(&config).unwrap();
    let mut layout = resolve_codex_home_layout(&config, &settings).unwrap();

    materialize_codex_shadow_home(&mut layout).unwrap();
    verify_codex_shadow_home(&layout).unwrap();
    assert!(fs::symlink_metadata(shadow.join("sessions"))
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(fs::symlink_metadata(shadow.join("config.toml"))
        .unwrap()
        .file_type()
        .is_symlink());
    fs::write(shadow.join("auth.json"), b"redacted private auth").unwrap();
    verify_codex_shadow_home(&layout).unwrap();
    assert!(!fs::symlink_metadata(shadow.join("auth.json"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn transport_correlates_out_of_order_responses_and_routes_messages() {
    tauri::async_runtime::block_on(async {
        let (client_reader, mut server_writer) = tokio::io::duplex(16 * 1024);
        let (server_reader, client_writer) = tokio::io::duplex(16 * 1024);
        let mut connection = CodexRpcConnection::from_test_io(client_reader, client_writer);
        let client = connection.client();
        let mut inbound = connection.take_inbound().unwrap();
        let mut requests = BufReader::new(server_reader);
        let first_client = client.clone();
        let first_context = context("first");
        let first = tokio::spawn(async move {
            first_client
                .request("test/first", json!({ "value": 1 }), &first_context)
                .await
        });
        let second_client = client.clone();
        let second_context = context("second");
        let second = tokio::spawn(async move {
            second_client
                .request("test/second", json!({ "value": 2 }), &second_context)
                .await
        });
        let mut request_lines = Vec::new();
        for _ in 0..2 {
            let mut line = String::new();
            requests.read_line(&mut line).await.unwrap();
            request_lines.push(serde_json::from_str::<Value>(&line).unwrap());
        }
        let first_request = request_lines
            .iter()
            .find(|request| request["method"] == "test/first")
            .unwrap();
        let second_request = request_lines
            .iter()
            .find(|request| request["method"] == "test/second")
            .unwrap();
        server_writer
            .write_all(
                format!(
                    "{{\"id\":{},\"result\":{{\"name\":\"second\"}}}}\n",
                    second_request["id"]
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        server_writer
            .write_all(
                format!(
                    "{{\"id\":{},\"result\":{{\"name\":\"first\"}}}}\n",
                    first_request["id"]
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        server_writer
            .write_all(b"{\"method\":\"notice/test\",\"params\":{\"ok\":true}}\n")
            .await
            .unwrap();
        server_writer
            .write_all(b"{\"id\":\"server-1\",\"method\":\"request/test\",\"params\":{}}\n")
            .await
            .unwrap();

        assert_eq!(first.await.unwrap().unwrap()["name"], "first");
        assert_eq!(second.await.unwrap().unwrap()["name"], "second");
        assert_eq!(
            inbound.recv().await.unwrap(),
            CodexInboundMessage::Notification {
                method: "notice/test".to_string(),
                params: json!({ "ok": true }),
            }
        );
        assert_eq!(
            inbound.recv().await.unwrap(),
            CodexInboundMessage::Request {
                id: json!("server-1"),
                method: "request/test".to_string(),
                params: json!({}),
            }
        );
        connection
            .stop(Duration::from_millis(10), Duration::from_millis(10))
            .await
            .unwrap();
    });
}

#[test]
fn transport_reports_malformed_input_without_echoing_raw_content() {
    tauri::async_runtime::block_on(async {
        let (client_reader, mut server_writer) = tokio::io::duplex(1024);
        let (_server_reader, client_writer) = tokio::io::duplex(1024);
        let mut connection = CodexRpcConnection::from_test_io(client_reader, client_writer);
        let mut inbound = connection.take_inbound().unwrap();
        server_writer
            .write_all(b"not-json-with-ganbaru-secret\n")
            .await
            .unwrap();

        let CodexInboundMessage::Malformed {
            reason,
            byte_length,
        } = inbound.recv().await.unwrap()
        else {
            panic!("expected malformed input");
        };
        assert_eq!(reason, "message is not valid JSON");
        assert_eq!(byte_length, 28);
        assert!(!reason.contains("ganbaru-secret"));
    });
}

#[test]
fn transport_honors_cancellation_while_waiting_for_response() {
    tauri::async_runtime::block_on(async {
        let (client_reader, _server_writer) = tokio::io::duplex(1024);
        let (_server_reader, client_writer) = tokio::io::duplex(1024);
        let _connection = CodexRpcConnection::from_test_io(client_reader, client_writer);
        let client = _connection.client();
        let cancellation = DriverCancellation::default();
        let request_context = DriverOperationContext {
            operation_id: "cancelled".to_string(),
            deadline: Instant::now() + Duration::from_secs(2),
            cancellation: cancellation.clone(),
        };
        let request = tokio::spawn(async move {
            client
                .request("test/cancel", json!({}), &request_context)
                .await
        });
        tokio::time::sleep(Duration::from_millis(30)).await;
        cancellation.cancel();
        assert_eq!(request.await.unwrap(), Err(CodexRpcFailure::Cancelled));
    });
}

#[test]
fn redacted_fixture_normalizes_lifecycle_content_plan_and_unknown_events() {
    let normalizer = CodexEventNormalizer::new(
        identifier("codex-instance-1", ProviderInstanceId::new),
        identifier("chat-thread-1", ChatThreadId::new),
        identifier("session-1", ProviderSessionId::new),
    );
    let mut state = CodexRouteState::new(
        modes(SafetyMode::AskForApproval, InteractionMode::Build),
        Some(identifier("gpt-5.4", ModelId::new)),
    );
    state.active_chat_turn_id = Some(identifier("chat-turn-1", ChatTurnId::new));
    let mut events = Vec::new();
    for line in include_str!("fixtures/notifications.jsonl").lines() {
        let envelope: Value = serde_json::from_str(line).unwrap();
        events.extend(
            normalizer
                .normalize_notification(
                    &mut state,
                    envelope["method"].as_str().unwrap(),
                    envelope["params"].clone(),
                )
                .unwrap(),
        );
    }

    let CanonicalEvent::ThreadStarted(thread_started) = &events[0].event else {
        panic!("expected thread start");
    };
    assert_eq!(
        thread_started.provider_thread_id.as_str(),
        "provider-thread-1"
    );
    assert!(thread_started.title.is_none());
    assert!(events.iter().any(|event| matches!(
        &event.event,
        CanonicalEvent::ContentDelta(delta)
            if delta.item_id == "provider-item-1"
                && delta.delta == "Hello from Codex"
    )));
    assert!(events.iter().any(|event| matches!(
        &event.event,
        CanonicalEvent::ContentDelta(delta)
            if delta.content_index == 2
                && delta.stream_kind == ContentStreamKind::ReasoningSummary
    )));
    assert!(events.iter().any(|event| matches!(
        &event.event,
        CanonicalEvent::PlanUpdated(plan) if plan.steps.len() == 2
    )));
    assert!(events.iter().any(|event| matches!(
        &event.event,
        CanonicalEvent::Unknown(unknown)
            if unknown.source_type == "future/redactedEvent"
    )));
    assert!(matches!(
        events.last().unwrap().event,
        CanonicalEvent::TurnCompleted(_)
    ));
    assert!(!serde_json::to_string(&events)
        .unwrap()
        .contains("ganbaru-codex-fixture-secret"));
}

#[test]
fn permission_approvals_return_only_requested_subset_and_scope() {
    tauri::async_runtime::block_on(async {
        let (client_reader, _server_writer) = tokio::io::duplex(4096);
        let (server_reader, client_writer) = tokio::io::duplex(4096);
        let _connection = CodexRpcConnection::from_test_io(client_reader, client_writer);
        let pending_id = identifier("provider-request-1", ProviderRequestId::new);
        let requested = json!({
            "fileSystem": { "write": ["/redacted/workspace"] },
            "network": { "enabled": true }
        });
        let pending: PendingCodexRequests = Arc::new(Mutex::new(HashMap::from([(
            pending_id.clone(),
            PendingCodexRequest {
                rpc_id: json!(61),
                provider_request_id: pending_id.clone(),
                chat_turn_id: Some(identifier("chat-turn-1", ChatTurnId::new)),
                provider_turn_id: Some(identifier("provider-turn-1", ProviderTurnId::new)),
                provider_item_id: Some(identifier("provider-item-1", ProviderItemId::new)),
                kind: PendingCodexRequestKind::Approval {
                    allowed_provider_decisions: vec![
                        "accept".to_string(),
                        "acceptForSession".to_string(),
                        "decline".to_string(),
                        "cancel".to_string(),
                    ],
                    response: CodexApprovalResponse::Permissions {
                        requested: requested.clone(),
                    },
                },
            },
        )])));
        let normalizer = CodexEventNormalizer::new(
            identifier("codex-instance-1", ProviderInstanceId::new),
            identifier("chat-thread-1", ChatThreadId::new),
            identifier("session-1", ProviderSessionId::new),
        );
        let route = Arc::new(Mutex::new(CodexRouteState::new(
            modes(SafetyMode::AskForApproval, InteractionMode::Build),
            None,
        )));
        let sink = Arc::new(RecordingSink::default());
        let sink_trait: Arc<dyn ProviderEventSink> = sink.clone();
        let request: ResolveApprovalRequest = serde_json::from_value(json!({
            "command": { "clientCommandId": "approval-command-1", "expectedThreadRevision": 4 },
            "sessionId": "session-1",
            "requestId": "chat-request-1",
            "providerRequestId": "provider-request-1",
            "decision": {
                "kind": "allow_session",
                "providerOptionId": "acceptForSession",
                "updatedToolInput": null
            }
        }))
        .unwrap();
        resolve_codex_approval(
            &_connection.client(),
            &pending,
            &normalizer,
            &route,
            &sink_trait,
            &request,
            &context("approve"),
        )
        .await
        .unwrap();
        let mut reader = BufReader::new(server_reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();

        assert_eq!(response["id"], 61);
        assert_eq!(response["result"]["scope"], "session");
        assert_eq!(response["result"]["permissions"], requested);
        assert!(matches!(
            sink.events().as_slice(),
            [CanonicalRuntimeEvent {
                event: CanonicalEvent::RequestResolved(_),
                ..
            }]
        ));
    });
}

#[test]
fn secret_structured_answers_are_sent_to_codex_but_not_canonicalized() {
    tauri::async_runtime::block_on(async {
        let (client_reader, _server_writer) = tokio::io::duplex(4096);
        let (server_reader, client_writer) = tokio::io::duplex(4096);
        let _connection = CodexRpcConnection::from_test_io(client_reader, client_writer);
        let pending_id = identifier("provider-request-secret", ProviderRequestId::new);
        let pending: PendingCodexRequests = Arc::new(Mutex::new(HashMap::from([(
            pending_id.clone(),
            PendingCodexRequest {
                rpc_id: json!(72),
                provider_request_id: pending_id,
                chat_turn_id: Some(identifier("chat-turn-1", ChatTurnId::new)),
                provider_turn_id: None,
                provider_item_id: None,
                kind: PendingCodexRequestKind::UserInput {
                    option_labels: HashMap::from([("token".to_string(), HashMap::new())]),
                    secret_question_ids: BTreeSet::from(["token".to_string()]),
                },
            },
        )])));
        let normalizer = CodexEventNormalizer::new(
            identifier("codex-instance-1", ProviderInstanceId::new),
            identifier("chat-thread-1", ChatThreadId::new),
            identifier("session-1", ProviderSessionId::new),
        );
        let route = Arc::new(Mutex::new(CodexRouteState::new(
            modes(SafetyMode::AskForApproval, InteractionMode::Build),
            None,
        )));
        let sink = Arc::new(RecordingSink::default());
        let sink_trait: Arc<dyn ProviderEventSink> = sink.clone();
        let request: ResolveUserInputRequest = serde_json::from_value(json!({
            "command": { "clientCommandId": "input-command-1", "expectedThreadRevision": 5 },
            "sessionId": "session-1",
            "requestId": "chat-request-secret",
            "providerRequestId": "provider-request-secret",
            "answers": [{
                "questionId": "token",
                "selectedOptionIds": [],
                "freeFormText": "ganbaru-sensitive-answer"
            }]
        }))
        .unwrap();
        resolve_codex_user_input(
            &_connection.client(),
            &pending,
            &normalizer,
            &route,
            &sink_trait,
            &request,
        )
        .await
        .unwrap();
        let mut reader = BufReader::new(server_reader);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert!(line.contains("ganbaru-sensitive-answer"));
        assert!(!serde_json::to_string(&sink.events())
            .unwrap()
            .contains("ganbaru-sensitive-answer"));
    });
}

#[test]
fn driver_fixture_covers_fresh_plan_interrupt_and_shutdown() {
    tauri::async_runtime::block_on(async {
        let workspace = TestDirectory::new("driver-fresh");
        let (mut driver, fixture) = fixture_driver(workspace.path(), FixtureScenario::Healthy);
        let sink = Arc::new(RecordingSink::default());
        let sink_trait: Arc<dyn ProviderEventSink> = sink.clone();
        let snapshot = driver
            .start_session(
                start_request(workspace.path()),
                sink_trait,
                &context("start"),
            )
            .await
            .unwrap();
        assert_eq!(
            snapshot.provider_thread_id.as_ref().unwrap().as_str(),
            "fixture-thread-new"
        );
        assert_eq!(snapshot.state, ProviderSessionState::Ready);
        assert_eq!(
            snapshot.resume_cursor.as_ref().unwrap().value["threadId"],
            "fixture-thread-new"
        );

        let receipt = driver
            .send_turn(fixture_turn(&snapshot.session_id, "plan"), &context("send"))
            .await
            .unwrap();
        assert_eq!(
            receipt.provider_turn_id.as_ref().unwrap().as_str(),
            "fixture-provider-turn"
        );
        driver
            .interrupt_turn(
                InterruptTurnRequest {
                    command: ChatCommandContext {
                        client_command_id: identifier("interrupt-command", ChatCommandId::new),
                        expected_thread_revision: Some(3),
                    },
                    session_id: snapshot.session_id.clone(),
                    turn_id: identifier("chat-turn-fixture", ChatTurnId::new),
                },
                &context("interrupt"),
            )
            .await
            .unwrap();
        wait_for_event(&sink, |event| {
            matches!(
                event.event,
                CanonicalEvent::TurnCompleted(ref turn)
                    if turn.state == ChatTurnState::Interrupted
            )
        })
        .await;
        driver
            .stop_session(
                StopSessionRequest {
                    session_id: snapshot.session_id,
                    force: false,
                },
                &context("stop"),
            )
            .await
            .unwrap();
        for _ in 0..100 {
            if fixture.closed.load(Ordering::Acquire) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(fixture.closed.load(Ordering::Acquire));

        let received = fixture.received();
        assert!(received
            .iter()
            .any(|message| message["method"] == "thread/start"));
        let turn = received
            .iter()
            .find(|message| message["method"] == "turn/start")
            .unwrap();
        assert_eq!(turn["params"]["collaborationMode"]["mode"], "plan");
        assert_eq!(turn["params"]["approvalPolicy"], "on-request");
        assert!(received
            .iter()
            .any(|message| message["method"] == "turn/interrupt"));
        assert!(received
            .iter()
            .any(|message| message["method"] == "config/mcpServer/reload"));
    });
}

#[test]
fn driver_fixture_covers_native_resume_and_confirmed_missing_fallback() {
    tauri::async_runtime::block_on(async {
        for (scenario, expected_thread, expects_fallback) in [
            (FixtureScenario::Healthy, "provider-thread-existing", false),
            (FixtureScenario::ResumeMissing, "fixture-thread-new", true),
        ] {
            let workspace = TestDirectory::new("driver-resume");
            let (mut driver, fixture) = fixture_driver(workspace.path(), scenario);
            let direct_config = configuration(workspace.path(), None);
            let layout = resolve_codex_home_layout(
                &direct_config,
                &CodexProviderSettings::parse(&direct_config).unwrap(),
            )
            .unwrap();
            let sink = Arc::new(RecordingSink::default());
            let sink_trait: Arc<dyn ProviderEventSink> = sink.clone();
            let request: ResumeSessionRequest = serde_json::from_value(json!({
                "threadId": "chat-thread-fixture",
                "workspace": {
                    "workspaceId": "workspace-fixture",
                    "canonicalPath": workspace.path(),
                    "repositoryKind": "none",
                    "repositoryIdentity": null
                },
                "providerInstanceId": "codex-instance-1",
                "providerThreadId": "provider-thread-existing",
                "continuationGroupId": layout.continuation_group().unwrap(),
                "resumeCursor": {
                    "schemaVersion": 1,
                    "value": { "threadId": "provider-thread-existing" }
                },
                "modes": { "safetyMode": "ask_for_approval", "interactionMode": "build" }
            }))
            .unwrap();
            let snapshot = driver
                .resume_session(request, sink_trait, &context("resume"))
                .await
                .unwrap();
            assert_eq!(
                snapshot.provider_thread_id.as_ref().unwrap().as_str(),
                expected_thread
            );
            assert_eq!(
                sink.events().iter().any(|event| matches!(
                    &event.event,
                    CanonicalEvent::RuntimeWarning(warning)
                        if warning.code == "codex_resume_not_found_fresh_start"
                )),
                expects_fallback
            );
            let received = fixture.received();
            assert!(received
                .iter()
                .any(|message| message["method"] == "thread/resume"));
            assert_eq!(
                received
                    .iter()
                    .any(|message| message["method"] == "thread/start"),
                expects_fallback
            );
            driver
                .stop_session(
                    StopSessionRequest {
                        session_id: snapshot.session_id,
                        force: true,
                    },
                    &context("stop-resume"),
                )
                .await
                .unwrap();
        }
    });
}

#[test]
fn driver_fixture_routes_native_approval_and_structured_question() {
    tauri::async_runtime::block_on(async {
        for scenario in [
            FixtureScenario::CommandApproval,
            FixtureScenario::StructuredQuestion,
        ] {
            let workspace = TestDirectory::new("driver-request");
            let (mut driver, fixture) = fixture_driver(workspace.path(), scenario);
            let sink = Arc::new(RecordingSink::default());
            let sink_trait: Arc<dyn ProviderEventSink> = sink.clone();
            let snapshot = driver
                .start_session(
                    start_request(workspace.path()),
                    sink_trait,
                    &context("start"),
                )
                .await
                .unwrap();
            driver
                .send_turn(
                    fixture_turn(&snapshot.session_id, "build"),
                    &context("send"),
                )
                .await
                .unwrap();

            match scenario {
                FixtureScenario::CommandApproval => {
                    let opened = wait_for_event(&sink, |event| {
                        matches!(event.event, CanonicalEvent::RequestOpened(_))
                    })
                    .await;
                    let provider_request_id = opened.provider_request_id.unwrap();
                    driver
                        .resolve_approval(
                            ResolveApprovalRequest {
                                command: ChatCommandContext {
                                    client_command_id: identifier(
                                        "approval-command",
                                        ChatCommandId::new,
                                    ),
                                    expected_thread_revision: Some(3),
                                },
                                session_id: snapshot.session_id.clone(),
                                request_id: identifier("chat-request", ChatRequestId::new),
                                provider_request_id,
                                decision: ApprovalDecision {
                                    kind: ApprovalDecisionKind::AllowOnce,
                                    provider_option_id: Some("accept".to_string()),
                                    updated_tool_input: None,
                                },
                            },
                            &context("approve"),
                        )
                        .await
                        .unwrap();
                    wait_for_event(&sink, |event| {
                        matches!(event.event, CanonicalEvent::RequestResolved(_))
                    })
                    .await;
                    let response = wait_for_fixture_message(&fixture, |message| {
                        message["id"] == "fixture-approval" && message.get("result").is_some()
                    })
                    .await;
                    assert_eq!(response["result"]["decision"], "accept");
                }
                FixtureScenario::StructuredQuestion => {
                    let opened = wait_for_event(&sink, |event| {
                        matches!(event.event, CanonicalEvent::UserInputRequested(_))
                    })
                    .await;
                    let provider_request_id = opened.provider_request_id.unwrap();
                    let CanonicalEvent::UserInputRequested(requested) = opened.event else {
                        unreachable!();
                    };
                    assert!(!requested.questions[0].multiple);
                    assert!(requested.questions[0].free_form_allowed);
                    driver
                        .resolve_user_input(
                            ResolveUserInputRequest {
                                command: ChatCommandContext {
                                    client_command_id: identifier(
                                        "question-command",
                                        ChatCommandId::new,
                                    ),
                                    expected_thread_revision: Some(3),
                                },
                                session_id: snapshot.session_id.clone(),
                                request_id: identifier("chat-question", ChatRequestId::new),
                                provider_request_id,
                                answers: vec![UserInputAnswer {
                                    question_id: "strategy".to_string(),
                                    selected_option_ids: vec!["q0-option-0".to_string()],
                                    free_form_text: None,
                                }],
                            },
                            &context("answer"),
                        )
                        .await
                        .unwrap();
                    wait_for_event(&sink, |event| {
                        matches!(event.event, CanonicalEvent::UserInputResolved(_))
                    })
                    .await;
                    let response = wait_for_fixture_message(&fixture, |message| {
                        message["id"] == "fixture-question" && message.get("result").is_some()
                    })
                    .await;
                    assert_eq!(
                        response["result"]["answers"]["strategy"]["answers"][0],
                        "Focused"
                    );
                }
                _ => unreachable!(),
            }
            driver
                .stop_session(
                    StopSessionRequest {
                        session_id: snapshot.session_id,
                        force: true,
                    },
                    &context("stop-request"),
                )
                .await
                .unwrap();
        }
    });
}

#[test]
fn driver_probe_distinguishes_authentication_and_protocol_failure() {
    tauri::async_runtime::block_on(async {
        for (scenario, expected_state) in [
            (
                FixtureScenario::AuthenticationRequired,
                ProbeState::AuthenticationRequired,
            ),
            (
                FixtureScenario::MalformedInitialize,
                ProbeState::UnsupportedVersion,
            ),
        ] {
            let workspace = TestDirectory::new("driver-probe");
            let (mut driver, _fixture) = fixture_driver(workspace.path(), scenario);
            let probe = driver.probe(&context("probe")).await.unwrap();
            assert_eq!(probe.state, expected_state);
        }
    });
}

#[test]
fn healthy_probe_retains_the_discovered_model_catalog() {
    tauri::async_runtime::block_on(async {
        let workspace = TestDirectory::new("driver-probe-models");
        let (mut driver, _fixture) = fixture_driver(workspace.path(), FixtureScenario::Healthy);

        let probe = driver.probe(&context("probe-models")).await.unwrap();
        let catalog = driver.cached_model_catalog().unwrap();

        assert_eq!(probe.state, ProbeState::Healthy);
        assert!(!catalog.models.is_empty());
        assert_eq!(catalog.instance_id, probe.instance_id);
    });
}

#[test]
fn process_environment_cannot_override_codex_home() {
    let shared = TestDirectory::new("environment-shared");
    let mut config = configuration(shared.path(), None);
    config.environment =
        BTreeMap::from([("CODEX_HOME".to_string(), "/untrusted/override".to_string())]);
    let layout = CodexHomeLayout {
        shared_home: shared.path().to_path_buf(),
        effective_home: shared.path().to_path_buf(),
        shadowed: false,
    };
    assert!(codex_process_environment(&config, &layout).is_err());
}

#[cfg(unix)]
#[test]
fn provider_path_falls_back_to_the_standard_pnpm_user_directory() {
    use std::os::unix::fs::PermissionsExt;

    let home = TestDirectory::new("provider-path-home");
    let bin = home.path().join(".local").join("share").join("pnpm");
    fs::create_dir_all(&bin).unwrap();
    let executable = bin.join("codex");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&executable, permissions).unwrap();
    let mut environment = BTreeMap::from([("PATH".to_string(), "/usr/bin".to_string())]);

    append_fallback_executable_directories(&mut environment, home.path());
    let resolved = resolve_codex_executable("codex", &environment).unwrap();

    assert_eq!(resolved.executable, executable);
    assert!(resolved.prefix_arguments.is_empty());
}
