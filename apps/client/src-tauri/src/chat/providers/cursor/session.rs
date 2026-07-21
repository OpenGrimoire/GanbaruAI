//! Cursor ACP session startup, configuration, and inbound request routing.

use super::interactions::*;
use super::normalizer::{CursorEventNormalizer, CursorRouteState};
use super::protocol::*;
use super::transport::{AcpInboundMessage, AcpRpcClient, AcpRpcConnection, AcpRpcFailure};
use crate::chat::events::*;
use crate::chat::models::*;
use crate::chat::providers::{DriverOperationContext, ProviderEventSink};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
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
    let client = connection.client();
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
    if initialize
        .auth_methods
        .iter()
        .any(|method| method.id == CURSOR_AUTH_METHOD)
    {
        client
            .request(
                "authenticate",
                json!({ "methodId": CURSOR_AUTH_METHOD }),
                context,
            )
            .await
            .map_err(|error| authentication_error(error, "authentication"))?;
    } else if !initialize.auth_methods.is_empty() {
        return Err(ChatError::new(
            ChatErrorCode::AuthenticationRequired,
            "Cursor requires an authentication method Ganbaru does not support",
            true,
        ));
    }
    let (mut setup, session_id) = if let Some(session_id) = resume_session_id {
        if !initialize.agent_capabilities.load_session {
            return Err(ChatError::unsupported(
                "Cursor did not advertise ACP session loading",
            ));
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
    apply_configuration(
        &client,
        &session_id,
        &mut setup,
        modes,
        model_id,
        model_options,
        context,
    )
    .await?;
    let models = parse_available_models(
        client
            .request("cursor/list_available_models", json!({}), context)
            .await
            .map_err(|error| error.to_chat_error("model discovery"))?,
    )?;
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
    let updates = resolve_configuration_updates(&setup.config_options, model_id, model_options)?;
    let requested_mode = setup
        .modes
        .as_ref()
        .and_then(|available| find_mode(available, modes.interaction_mode))
        .map(|mode| mode.id.clone());
    if requested_mode.is_none() {
        if modes.interaction_mode == InteractionMode::Plan {
            return Err(ChatError::unsupported(
                "Cursor did not advertise a native Plan or Architect mode",
            ));
        }
        if setup.modes.is_some() {
            return Err(ChatError::unsupported(
                "Cursor did not advertise a compatible Build mode",
            ));
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
                    format!("Cursor rejected a model setting: {error}"),
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
                        format!("Cursor rejected the requested mode: {error}"),
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
                    format!("Cursor rejected the requested mode: {error}"),
                ));
            }
            mode_state.current_mode_id = mode_id;
        }
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
        let events = resources
            .route
            .lock()
            .map_err(|_| driver_state_error())
            .and_then(|mut state| {
                resources
                    .normalizer
                    .interrupted(&mut state, "Cursor ACP transport closed")
            })
            .unwrap_or_default();
        emit_all(&resources.sink, events).await;
        set_terminal_error(
            &resources.terminal_error,
            ChatError::new(
                ChatErrorCode::TransportUnavailable,
                "Cursor ACP transport closed unexpectedly",
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
                    summary: "Cursor emitted an unsupported ACP notification".to_string(),
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

async fn handle_request(
    resources: &CursorRouterResources,
    rpc_id: Value,
    method: &str,
    params: Value,
) -> ChatResult<()> {
    match method {
        "session/request_permission" => handle_permission(resources, rpc_id, params).await,
        "cursor/ask_question" => handle_question(resources, rpc_id, params).await,
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
                        summary: "Cursor requested an unsupported ACP extension".to_string(),
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
                "Cursor request is no longer pending",
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
            "Cursor emitted a duplicate or excessive pending request",
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
                format!("Cursor session {session_id} was not found"),
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
                "Cursor authentication is required",
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
        "Cursor driver state is unavailable",
        false,
    )
}
