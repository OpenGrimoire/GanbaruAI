//! Cursor ACP session startup, configuration, and inbound request routing.

use super::driver::AcpProviderFlavor;
use super::interactions::*;
use super::normalizer::{CursorEventNormalizer, CursorRouteState};
use super::protocol::*;
use super::transport::{AcpInboundMessage, AcpRpcClient, AcpRpcConnection, AcpRpcFailure};
use crate::chat::events::*;
use crate::chat::models::*;
use crate::chat::providers::{DriverOperationContext, ProviderEventSink};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct AcpStartedSession {
    pub initialize: AcpInitializeResponse,
    pub setup: AcpSessionSetup,
    pub session_id: String,
    pub models: Vec<ProviderModel>,
}

pub struct CursorRouterResources {
    pub client: AcpRpcClient,
    pub inbound: mpsc::Receiver<AcpInboundMessage>,
    pub normalizer: Arc<CursorEventNormalizer>,
    pub route: Arc<Mutex<CursorRouteState>>,
    pub pending: PendingCursorRequests,
    pub sink: Arc<dyn ProviderEventSink>,
    pub expected_shutdown: Arc<AtomicBool>,
    pub terminal_error: Arc<Mutex<Option<ChatError>>>,
    pub flavor: AcpProviderFlavor,
    pub prompt_completions: PendingPromptCompletions,
}

pub type PendingPromptCompletions = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

#[derive(Clone, Copy)]
pub struct AcpRequestedConfiguration<'a> {
    pub modes: TurnModeSnapshot,
    pub model_id: Option<&'a ModelId>,
    pub model_options: &'a [ModelOptionSelection],
}

pub async fn initialize_session(
    connection: &AcpRpcConnection,
    workspace: &str,
    resume_session_id: Option<&str>,
    modes: TurnModeSnapshot,
    model_id: Option<&ModelId>,
    model_options: &[ModelOptionSelection],
    context: &DriverOperationContext,
) -> ChatResult<AcpStartedSession> {
    initialize_provider_session(
        AcpProviderFlavor::Cursor,
        false,
        connection,
        workspace,
        resume_session_id,
        AcpRequestedConfiguration {
            modes,
            model_id,
            model_options,
        },
        context,
    )
    .await
}

pub async fn initialize_provider_session(
    flavor: AcpProviderFlavor,
    grok_uses_api_key: bool,
    connection: &AcpRpcConnection,
    workspace: &str,
    resume_session_id: Option<&str>,
    requested: AcpRequestedConfiguration<'_>,
    context: &DriverOperationContext,
) -> ChatResult<AcpStartedSession> {
    let client = connection.client();
    let provider_name = flavor.display_name();
    let initialize = parse_initialize(
        client
            .request(
                "initialize",
                json!({
                    "protocolVersion": ACP_PROTOCOL_VERSION,
                    "clientCapabilities": {
                        "fs": { "readTextFile": false, "writeTextFile": false },
                        "terminal": false,
                        "_meta": { "parameterizedModelPicker": true },
                    },
                    "clientInfo": {
                        "name": "ganbaru-ai",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                }),
                context,
            )
            .await
            .map_err(|error| error.to_chat_error("initialization"))?,
    )?;
    let preferred_auth_method = match flavor {
        AcpProviderFlavor::Cursor => CURSOR_AUTH_METHOD,
        AcpProviderFlavor::Grok => {
            if grok_uses_api_key {
                "xai.api_key"
            } else {
                "cached_token"
            }
        }
    };
    if initialize
        .auth_methods
        .iter()
        .any(|method| method.id == preferred_auth_method)
    {
        client
            .request(
                "authenticate",
                json!({ "methodId": preferred_auth_method }),
                context,
            )
            .await
            .map_err(|error| authentication_error(error, "authentication"))?;
    } else if !initialize.auth_methods.is_empty() {
        return Err(ChatError::new(
            ChatErrorCode::AuthenticationRequired,
            format!("{provider_name} requires an authentication method Ganbaru does not support"),
            true,
        ));
    }
    let (mut setup, session_id) = if let Some(session_id) = resume_session_id {
        if !initialize.agent_capabilities.load_session {
            return Err(ChatError::unsupported(format!(
                "{provider_name} did not advertise ACP session loading"
            )));
        }
        let result = client
            .request(
                "session/load",
                json!({ "sessionId": session_id, "cwd": workspace, "mcpServers": [] }),
                context,
            )
            .await
            .map_err(|error| session_load_error(error, session_id))?;
        (parse_session_setup(result, false)?, session_id.to_string())
    } else {
        let result = client
            .request(
                "session/new",
                json!({ "cwd": workspace, "mcpServers": [] }),
                context,
            )
            .await
            .map_err(|error| authentication_error(error, "session creation"))?;
        let setup = parse_session_setup(result, true)?;
        let session_id = setup
            .session_id
            .clone()
            .ok_or_else(|| protocol_error("new session ID"))?;
        (setup, session_id)
    };
    if flavor == AcpProviderFlavor::Grok {
        ensure_grok_model_state(&mut setup);
    }
    apply_provider_configuration(flavor, &client, &session_id, &mut setup, requested, context)
        .await?;
    let models = match flavor {
        AcpProviderFlavor::Cursor => parse_available_models(
            client
                .request("cursor/list_available_models", json!({}), context)
                .await
                .map_err(|error| error.to_chat_error("model discovery"))?,
        )?,
        AcpProviderFlavor::Grok => parse_acp_models(&setup)?,
    };
    Ok(AcpStartedSession {
        initialize,
        setup,
        session_id,
        models,
    })
}

pub async fn apply_configuration(
    client: &AcpRpcClient,
    session_id: &str,
    setup: &mut AcpSessionSetup,
    modes: TurnModeSnapshot,
    model_id: Option<&ModelId>,
    model_options: &[ModelOptionSelection],
    context: &DriverOperationContext,
) -> ChatResult<()> {
    apply_provider_configuration(
        AcpProviderFlavor::Cursor,
        client,
        session_id,
        setup,
        AcpRequestedConfiguration {
            modes,
            model_id,
            model_options,
        },
        context,
    )
    .await
}

pub async fn apply_provider_configuration(
    flavor: AcpProviderFlavor,
    client: &AcpRpcClient,
    session_id: &str,
    setup: &mut AcpSessionSetup,
    requested: AcpRequestedConfiguration<'_>,
    context: &DriverOperationContext,
) -> ChatResult<()> {
    let provider_name = flavor.display_name();
    if flavor == AcpProviderFlavor::Grok {
        if !requested.model_options.is_empty() {
            return Err(ChatError::validation(
                "modelOptions",
                "Grok does not advertise model options",
            ));
        }
        apply_grok_model(client, session_id, setup, requested.model_id, context).await?;
    }
    let updates = if flavor == AcpProviderFlavor::Cursor {
        resolve_configuration_updates(
            &setup.config_options,
            requested.model_id,
            requested.model_options,
        )?
    } else {
        Vec::new()
    };
    let requested_mode = setup
        .modes
        .as_ref()
        .and_then(|available| find_mode(available, requested.modes.interaction_mode))
        .map(|mode| mode.id.clone());
    if requested_mode.is_none() {
        if requested.modes.interaction_mode == InteractionMode::Plan {
            return Err(ChatError::unsupported(format!(
                "{provider_name} did not advertise a native Plan mode"
            )));
        }
        if setup.modes.is_some() {
            return Err(ChatError::unsupported(format!(
                "{provider_name} did not advertise a compatible Build mode"
            )));
        }
    }
    let mut applied = Vec::new();
    for update in &updates {
        let result = client
            .request(
                "session/set_config_option",
                config_request(session_id, &update.config_id, &update.value),
                context,
            )
            .await;
        match result {
            Ok(result) => {
                let parsed = parse_config_update_response(result, &update.config_id, &update.value);
                let options = match parsed {
                    Ok(options) => options,
                    Err(error) => {
                        let mut rollback = applied.clone();
                        rollback.push(update.clone());
                        rollback_configuration(client, session_id, &rollback, context).await;
                        return Err(error);
                    }
                };
                setup.config_options = options;
                applied.push(update.clone());
            }
            Err(error) => {
                rollback_configuration(client, session_id, &applied, context).await;
                return Err(ChatError::validation(
                    "modelOptions",
                    format!("{provider_name} rejected a model setting: {error}"),
                ));
            }
        }
    }
    if let (Some(mode_id), Some(mode_state)) = (requested_mode, setup.modes.as_mut()) {
        if mode_state.current_mode_id != mode_id {
            if let Some(mode_update) =
                resolve_mode_configuration_update(&setup.config_options, &mode_id)?
            {
                let result = client
                    .request(
                        "session/set_config_option",
                        config_request(session_id, &mode_update.config_id, &mode_update.value),
                        context,
                    )
                    .await;
                let options = match result {
                    Ok(result) => parse_config_update_response(
                        result,
                        &mode_update.config_id,
                        &mode_update.value,
                    ),
                    Err(error) => Err(ChatError::validation(
                        "interactionMode",
                        format!("{provider_name} rejected the requested mode: {error}"),
                    )),
                };
                match options {
                    Ok(options) => setup.config_options = options,
                    Err(error) => {
                        let mut rollback = applied.clone();
                        rollback.push(mode_update);
                        rollback_configuration(client, session_id, &rollback, context).await;
                        return Err(error);
                    }
                }
            } else if let Err(error) = client
                .request(
                    "session/set_mode",
                    json!({ "sessionId": session_id, "modeId": mode_id }),
                    context,
                )
                .await
            {
                rollback_configuration(client, session_id, &applied, context).await;
                return Err(ChatError::validation(
                    "interactionMode",
                    format!("{provider_name} rejected the requested mode: {error}"),
                ));
            }
            mode_state.current_mode_id = mode_id;
        }
    }
    Ok(())
}

async fn apply_grok_model(
    client: &AcpRpcClient,
    session_id: &str,
    setup: &mut AcpSessionSetup,
    model_id: Option<&ModelId>,
    context: &DriverOperationContext,
) -> ChatResult<()> {
    let Some(model_state) = setup.models.as_mut() else {
        if model_id.is_some() {
            return Err(ChatError::unsupported(
                "Grok did not advertise dynamic models",
            ));
        }
        return Ok(());
    };
    let requested = model_id
        .map(ModelId::as_str)
        .unwrap_or(model_state.current_model_id.as_str());
    if !model_state
        .available_models
        .iter()
        .any(|model| model.model_id == requested)
    {
        return Err(ChatError::validation(
            "modelId",
            "Grok model is not in the advertised catalog",
        ));
    }
    if model_state.current_model_id != requested {
        client
            .request(
                "session/set_model",
                json!({ "sessionId": session_id, "modelId": requested }),
                context,
            )
            .await
            .map_err(|error| {
                ChatError::validation(
                    "modelId",
                    format!("Grok rejected the requested model: {error}"),
                )
            })?;
        model_state.current_model_id = requested.to_string();
    }
    Ok(())
}

pub fn spawn_cursor_router(resources: CursorRouterResources) -> JoinHandle<()> {
    tokio::spawn(async move {
        route_inbound(resources).await;
    })
}

async fn route_inbound(mut resources: CursorRouterResources) {
    while let Some(message) = resources.inbound.recv().await {
        let result = match message {
            AcpInboundMessage::Notification { method, params } => {
                handle_notification(&resources, &method, params).await
            }
            AcpInboundMessage::Request { id, method, params } => {
                handle_request(&resources, id, &method, params).await
            }
            AcpInboundMessage::Malformed { reason, .. } => {
                Err(protocol_error(&format!("JSON-RPC message: {reason}")))
            }
            AcpInboundMessage::Closed => break,
        };
        if let Err(error) = result {
            set_terminal_error(&resources.terminal_error, error);
            break;
        }
    }
    if !resources.expected_shutdown.load(Ordering::Acquire) {
        let provider_name = resources.flavor.display_name();
        let events = resources
            .route
            .lock()
            .map_err(|_| driver_state_error())
            .and_then(|mut state| {
                resources
                    .normalizer
                    .interrupted(&mut state, &format!("{provider_name} ACP transport closed"))
            })
            .unwrap_or_default();
        emit_all(&resources.sink, events).await;
        set_terminal_error(
            &resources.terminal_error,
            ChatError::new(
                ChatErrorCode::TransportUnavailable,
                format!("{provider_name} ACP transport closed unexpectedly"),
                true,
            ),
        );
    }
}

async fn handle_notification(
    resources: &CursorRouterResources,
    method: &str,
    params: Value,
) -> ChatResult<()> {
    if resources.flavor == AcpProviderFlavor::Grok
        && matches!(
            method,
            "x.ai/session/prompt_complete" | "_x.ai/session/prompt_complete"
        )
    {
        return handle_grok_prompt_completion(resources, params);
    }
    let events = {
        let mut state = resources.route.lock().map_err(|_| driver_state_error())?;
        match method {
            "session/update" => resources
                .normalizer
                .normalize_session_update(&mut state, params)?,
            "cursor/update_todos" => vec![resources.normalizer.normalize_todos(&state, &params)?],
            _ => vec![resources.normalizer.external_event(
                &state,
                method,
                None,
                CanonicalEvent::Unknown(UnknownEvent {
                    source_type: format!("acp/notification/{method}"),
                    summary: format!(
                        "{} emitted an unsupported ACP notification",
                        resources.flavor.display_name()
                    ),
                    safe_payload: Some(safe_shape(&params)),
                }),
            )?],
        }
    };
    for event in events {
        resources.sink.emit(event).await?;
    }
    Ok(())
}

fn handle_grok_prompt_completion(
    resources: &CursorRouterResources,
    params: Value,
) -> ChatResult<()> {
    let object = params
        .as_object()
        .ok_or_else(|| protocol_error("Grok prompt completion"))?;
    let session_id = object
        .get("sessionId")
        .and_then(Value::as_str)
        .filter(|value| valid_identifier(value, 512))
        .ok_or_else(|| protocol_error("Grok prompt completion session ID"))?;
    let expected_session = resources
        .route
        .lock()
        .map_err(|_| driver_state_error())?
        .provider_thread_id
        .clone();
    if session_id != expected_session {
        return Err(protocol_error("Grok prompt completion session ID"));
    }
    let prompt_id = object
        .get("promptId")
        .and_then(Value::as_str)
        .filter(|value| valid_identifier(value, 512));
    let stop_reason = object
        .get("stopReason")
        .and_then(Value::as_str)
        .filter(|value| valid_identifier(value, 128))
        .unwrap_or("unknown");
    let sender = {
        let mut pending = resources
            .prompt_completions
            .lock()
            .map_err(|_| driver_state_error())?;
        if let Some(prompt_id) = prompt_id {
            pending.remove(prompt_id)
        } else if pending.len() == 1 {
            let only_id = pending.keys().next().cloned();
            only_id.and_then(|id| pending.remove(&id))
        } else if pending.is_empty() {
            None
        } else {
            return Err(protocol_error("ambiguous Grok prompt completion"));
        }
    };
    if let Some(sender) = sender {
        let _ = sender.send(json!({ "stopReason": stop_reason }));
    }
    Ok(())
}

async fn handle_request(
    resources: &CursorRouterResources,
    rpc_id: Value,
    method: &str,
    params: Value,
) -> ChatResult<()> {
    match method {
        "session/request_permission" => handle_permission(resources, rpc_id, params).await,
        "cursor/ask_question" => handle_question(resources, rpc_id, params).await,
        "x.ai/ask_user_question" | "_x.ai/ask_user_question"
            if resources.flavor == AcpProviderFlavor::Grok =>
        {
            handle_xai_question(resources, rpc_id, params).await
        }
        "cursor/create_plan" => {
            let event = {
                let state = resources.route.lock().map_err(|_| driver_state_error())?;
                resources
                    .normalizer
                    .normalize_create_plan(&state, &params)?
            };
            resources.sink.emit(event).await?;
            resources
                .client
                .respond(rpc_id, json!({ "accepted": true }))
                .await
                .map_err(|error| error.to_chat_error("plan response"))
        }
        _ => {
            let event = {
                let state = resources.route.lock().map_err(|_| driver_state_error())?;
                resources.normalizer.external_event(
                    &state,
                    method,
                    None,
                    CanonicalEvent::Unknown(UnknownEvent {
                        source_type: format!("acp/request/{method}"),
                        summary: format!(
                            "{} requested an unsupported ACP extension",
                            resources.flavor.display_name()
                        ),
                        safe_payload: Some(safe_shape(&params)),
                    }),
                )?
            };
            resources.sink.emit(event).await?;
            resources
                .client
                .respond_error(rpc_id, -32601, "Method not supported by Ganbaru AI")
                .await
                .map_err(|error| error.to_chat_error("extension rejection"))
        }
    }
}

async fn handle_xai_question(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let params_object = params
        .as_object()
        .ok_or_else(|| protocol_error("Grok question parameters"))?;
    let question_object = params_object
        .get("params")
        .and_then(Value::as_object)
        .unwrap_or(params_object);
    let question_session_id = question_object
        .get("sessionId")
        .and_then(Value::as_str)
        .ok_or_else(|| protocol_error("Grok question session ID"))?;
    let expected_session_id = resources
        .route
        .lock()
        .map_err(|_| driver_state_error())?
        .provider_thread_id
        .clone();
    if question_session_id != expected_session_id {
        return Err(protocol_error("Grok question session ID"));
    }
    let (parsed, questions) = parse_xai_question(&params)?;
    let event = {
        let state = resources.route.lock().map_err(|_| driver_state_error())?;
        resources.normalizer.external_event(
            &state,
            "x.ai/ask_user_question",
            Some(parsed.provider_request_id.clone()),
            CanonicalEvent::UserInputRequested(parsed.request),
        )?
    };
    resources.sink.emit(event).await?;
    insert_pending(
        &resources.pending,
        parsed.provider_request_id,
        PendingCursorRequest {
            rpc_id,
            kind: PendingCursorRequestKind::XaiUserInput { questions },
        },
    )
}

async fn handle_permission(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let (parsed, safety_mode) = {
        let state = resources.route.lock().map_err(|_| driver_state_error())?;
        (
            parse_permission(&params, &state.workspace)?,
            state.modes.safety_mode,
        )
    };
    let opened = {
        let state = resources.route.lock().map_err(|_| driver_state_error())?;
        resources.normalizer.external_event(
            &state,
            "session/request_permission",
            Some(parsed.provider_request_id.clone()),
            CanonicalEvent::RequestOpened(parsed.request.clone()),
        )?
    };
    resources.sink.emit(opened).await?;
    if let Some(option) = auto_permission_option(&parsed, safety_mode) {
        let decision = ApprovalDecision {
            kind: if option.kind == "allow_always" {
                ApprovalDecisionKind::AllowSession
            } else {
                ApprovalDecisionKind::AllowOnce
            },
            provider_option_id: Some(option.option_id.clone()),
            updated_tool_input: None,
        };
        resources
            .client
            .respond(
                rpc_id,
                json!({
                    "outcome": { "outcome": "selected", "optionId": option.option_id }
                }),
            )
            .await
            .map_err(|error| error.to_chat_error("automatic permission response"))?;
        let resolved = {
            let state = resources.route.lock().map_err(|_| driver_state_error())?;
            resources.normalizer.external_event(
                &state,
                "session/request_permission/resolved",
                Some(parsed.provider_request_id.clone()),
                CanonicalEvent::RequestResolved(RequestResolvedEvent {
                    request_id: parsed.provider_request_id,
                    state: RequestResolutionState::Resolved,
                    decision: Some(decision),
                }),
            )?
        };
        resources.sink.emit(resolved).await?;
        return Ok(());
    }
    insert_pending(
        &resources.pending,
        parsed.provider_request_id,
        PendingCursorRequest {
            rpc_id,
            kind: PendingCursorRequestKind::Approval {
                options: parsed.options,
            },
        },
    )
}

async fn handle_question(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let parsed = parse_question(&params)?;
    let event = {
        let state = resources.route.lock().map_err(|_| driver_state_error())?;
        resources.normalizer.external_event(
            &state,
            "cursor/ask_question",
            Some(parsed.provider_request_id.clone()),
            CanonicalEvent::UserInputRequested(parsed.request),
        )?
    };
    resources.sink.emit(event).await?;
    insert_pending(
        &resources.pending,
        parsed.provider_request_id,
        PendingCursorRequest {
            rpc_id,
            kind: PendingCursorRequestKind::UserInput {
                questions: parsed.questions,
            },
        },
    )
}

pub fn take_pending(
    pending: &PendingCursorRequests,
    request_id: &ProviderRequestId,
) -> ChatResult<PendingCursorRequest> {
    pending
        .lock()
        .map_err(|_| driver_state_error())?
        .remove(request_id)
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::Conflict,
                "ACP provider request is no longer pending",
                false,
            )
        })
}

pub fn restore_pending(
    pending: &PendingCursorRequests,
    request_id: ProviderRequestId,
    request: PendingCursorRequest,
) {
    if let Ok(mut pending) = pending.lock() {
        pending.insert(request_id, request);
    }
}

fn insert_pending(
    pending: &PendingCursorRequests,
    request_id: ProviderRequestId,
    request: PendingCursorRequest,
) -> ChatResult<()> {
    let mut pending = pending.lock().map_err(|_| driver_state_error())?;
    if pending.len() >= 64 || pending.contains_key(&request_id) {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "ACP provider emitted a duplicate or excessive pending request",
            false,
        ));
    }
    pending.insert(request_id, request);
    Ok(())
}

async fn rollback_configuration(
    client: &AcpRpcClient,
    session_id: &str,
    applied: &[ResolvedConfigUpdate],
    context: &DriverOperationContext,
) {
    for update in applied.iter().rev() {
        let _ = client
            .request(
                "session/set_config_option",
                config_request(session_id, &update.config_id, &update.previous_value),
                context,
            )
            .await;
    }
}

fn config_request(session_id: &str, config_id: &str, value: &Value) -> Value {
    match value {
        Value::Bool(value) => json!({
            "sessionId": session_id,
            "configId": config_id,
            "type": "boolean",
            "value": value,
        }),
        Value::String(value) => json!({
            "sessionId": session_id,
            "configId": config_id,
            "value": value,
        }),
        _ => json!({
            "sessionId": session_id,
            "configId": config_id,
            "value": value,
        }),
    }
}

fn session_load_error(error: AcpRpcFailure, session_id: &str) -> ChatError {
    if let AcpRpcFailure::Remote { code, message } = &error {
        if confirmed_session_not_found(*code, message) {
            return ChatError::new(
                ChatErrorCode::ResumeNotFound,
                format!("ACP provider session {session_id} was not found"),
                true,
            );
        }
    }
    error.to_chat_error("session load")
}

fn authentication_error(error: AcpRpcFailure, operation: &str) -> ChatError {
    if let AcpRpcFailure::Remote { code, message } = &error {
        let lower = message.to_ascii_lowercase();
        if *code == -32000
            && (lower.contains("auth") || lower.contains("login") || lower.contains("credential"))
        {
            return ChatError::new(
                ChatErrorCode::AuthenticationRequired,
                "ACP provider authentication is required",
                true,
            );
        }
    }
    error.to_chat_error(operation)
}

async fn emit_all(sink: &Arc<dyn ProviderEventSink>, events: Vec<CanonicalRuntimeEvent>) {
    for event in events {
        let _ = sink.emit(event).await;
    }
    let _ = sink.flush().await;
}

fn set_terminal_error(target: &Arc<Mutex<Option<ChatError>>>, error: ChatError) {
    if let Ok(mut target) = target.lock() {
        if target.is_none() {
            *target = Some(error);
        }
    }
}

pub fn driver_state_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "ACP provider driver state is unavailable",
        false,
    )
}
