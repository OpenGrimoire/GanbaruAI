//! Cursor ACP session startup, configuration, and inbound request routing.

use super::driver::AcpProviderFlavor;
use super::interactions::*;
use super::normalizer::{CursorEventNormalizer, CursorRouteState};
use super::protocol::*;
use super::transport::{AcpInboundMessage, AcpRpcClient, AcpRpcConnection, AcpRpcFailure};
use crate::chat::events::*;
use crate::chat::models::*;
use crate::chat::providers::{DriverOperationContext, ProviderEventSink};
use agent_client_protocol::schema::{
    v1::{
        ClientCapabilities, CreateTerminalRequest, CreateTerminalResponse, FileSystemCapabilities,
        HttpHeader, Implementation, InitializeRequest,
        InitializeResponse as OfficialInitializeResponse, KillTerminalRequest,
        KillTerminalResponse, McpServer, McpServerHttp, ReadTextFileRequest, ReadTextFileResponse,
        ReleaseTerminalRequest, ReleaseTerminalResponse, TerminalExitStatus, TerminalOutputRequest,
        TerminalOutputResponse, WaitForTerminalExitRequest, WaitForTerminalExitResponse,
        WriteTextFileRequest, WriteTextFileResponse,
    },
    ProtocolVersion,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot, Notify};
use tokio::task::JoinHandle;

const MAX_ACP_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ACP_FILE_LINES: u32 = 100_000;
const DEFAULT_ACP_TERMINAL_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_ACP_TERMINAL_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
static NEXT_ACP_FILE_WRITE: AtomicU64 = AtomicU64::new(1);
static NEXT_ACP_TERMINAL: AtomicU64 = AtomicU64::new(1);

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
    pub terminal_callbacks: AcpTerminalCallbacks,
}

#[derive(Clone)]
pub struct AcpTerminalCallbacks {
    workspace: PathBuf,
    session_id: String,
    terminals: Arc<Mutex<HashMap<String, AcpTerminal>>>,
}

#[derive(Clone)]
struct AcpTerminal {
    state: Arc<Mutex<AcpTerminalState>>,
    kill: mpsc::Sender<()>,
    completed: Arc<Notify>,
}

#[derive(Clone, Default)]
struct AcpTerminalState {
    output: Vec<u8>,
    output_limit: usize,
    truncated: bool,
    exit_status: Option<TerminalExitStatus>,
}

impl AcpTerminalCallbacks {
    pub fn new(workspace: PathBuf, session_id: String) -> Self {
        Self {
            workspace,
            session_id,
            terminals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn terminal(&self, id: &str) -> ChatResult<AcpTerminal> {
        self.terminals
            .lock()
            .map_err(|_| driver_state_error())?
            .get(id)
            .cloned()
            .ok_or_else(callback_permission_error)
    }

    fn verify_session(&self, session_id: &str) -> ChatResult<()> {
        require_acp_session(session_id, &self.session_id)
    }

    fn shutdown(&self) {
        if let Ok(mut terminals) = self.terminals.lock() {
            for terminal in terminals.values() {
                let _ = terminal.kill.try_send(());
            }
            terminals.clear();
        }
    }
}

pub type PendingPromptCompletions = Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>;

#[derive(Clone, Copy)]
pub struct AcpRequestedConfiguration<'a> {
    pub modes: TurnModeSnapshot,
    pub model_id: Option<&'a ModelId>,
    pub model_options: &'a [ModelOptionSelection],
}

pub struct AcpSessionInitialization<'a> {
    pub flavor: AcpProviderFlavor,
    pub grok_uses_api_key: bool,
    pub connection: &'a AcpRpcConnection,
    pub workspace: &'a str,
    pub resume_session_id: Option<&'a str>,
    pub requested: AcpRequestedConfiguration<'a>,
    pub internal_mcp: Option<&'a ProviderInternalMcpConfig>,
    pub context: &'a DriverOperationContext,
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
    initialize_provider_session(AcpSessionInitialization {
        flavor: AcpProviderFlavor::Cursor,
        grok_uses_api_key: false,
        connection,
        workspace,
        resume_session_id,
        requested: AcpRequestedConfiguration {
            modes,
            model_id,
            model_options,
        },
        internal_mcp: None,
        context,
    })
    .await
}

pub async fn initialize_provider_session(
    initialization: AcpSessionInitialization<'_>,
) -> ChatResult<AcpStartedSession> {
    let AcpSessionInitialization {
        flavor,
        grok_uses_api_key,
        connection,
        workspace,
        resume_session_id,
        requested,
        internal_mcp,
        context,
    } = initialization;
    let client = connection.client();
    let provider_name = flavor.display_name();
    let filesystem = FileSystemCapabilities::new()
        .read_text_file(true)
        .write_text_file(true);
    let capabilities = ClientCapabilities::new()
        .fs(filesystem)
        .terminal(true)
        .meta(serde_json::Map::from_iter([(
            "parameterizedModelPicker".to_string(),
            Value::Bool(true),
        )]));
    let initialize_request = InitializeRequest::new(ProtocolVersion::V1)
        .client_capabilities(capabilities)
        .client_info(Implementation::new("ganbaru-ai", env!("CARGO_PKG_VERSION")));
    let initialize_request = serde_json::to_value(initialize_request)
        .map_err(|_| protocol_error("ACP initialization request"))?;
    let initialize_value = client
        .request("initialize", initialize_request, context)
        .await
        .map_err(|error| error.to_chat_error("initialization"))?;
    let negotiated: OfficialInitializeResponse =
        serde_json::from_value(initialize_value.clone())
            .map_err(|_| protocol_error("ACP initialization response"))?;
    if negotiated.protocol_version != ProtocolVersion::V1 {
        return Err(ChatError::new(
            ChatErrorCode::UnsupportedVersion,
            format!(
                "{provider_name} negotiated unsupported ACP protocol version {}",
                negotiated.protocol_version
            ),
            true,
        ));
    }
    let initialize = parse_initialize(initialize_value)?;
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
    let mcp_servers = acp_mcp_servers(internal_mcp)?;
    let (mut setup, session_id) = if let Some(session_id) = resume_session_id {
        if !initialize.agent_capabilities.load_session {
            return Err(ChatError::unsupported(format!(
                "{provider_name} did not advertise ACP session loading"
            )));
        }
        let result = client
            .request(
                "session/load",
                json!({ "sessionId": session_id, "cwd": workspace, "mcpServers": mcp_servers }),
                context,
            )
            .await
            .map_err(|error| session_load_error(error, session_id))?;
        (parse_session_setup(result, false)?, session_id.to_string())
    } else {
        let result = client
            .request(
                "session/new",
                json!({ "cwd": workspace, "mcpServers": mcp_servers }),
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

fn acp_mcp_servers(config: Option<&ProviderInternalMcpConfig>) -> ChatResult<Value> {
    let servers = config
        .map(|server| {
            vec![McpServer::Http(
                McpServerHttp::new(server.name.clone(), server.url.clone()).headers(vec![
                    HttpHeader::new("Authorization", format!("Bearer {}", server.bearer_token)),
                ]),
            )]
        })
        .unwrap_or_default();
    serde_json::to_value(servers)
        .map_err(|_| ChatError::validation("internalMcp", "ACP MCP server is invalid"))
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
    resources.terminal_callbacks.shutdown();
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
        "fs/read_text_file" => handle_read_text_file(resources, rpc_id, params).await,
        "fs/write_text_file" => handle_write_text_file(resources, rpc_id, params).await,
        "terminal/create" => handle_terminal_create(resources, rpc_id, params).await,
        "terminal/output" => handle_terminal_output(resources, rpc_id, params).await,
        "terminal/wait_for_exit" => handle_terminal_wait(resources, rpc_id, params).await,
        "terminal/kill" => handle_terminal_kill(resources, rpc_id, params).await,
        "terminal/release" => handle_terminal_release(resources, rpc_id, params).await,
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

async fn handle_read_text_file(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: ReadTextFileRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => return reject_callback(resources, rpc_id, "Invalid file read request").await,
    };
    let result = (|| {
        let (workspace, session_id) = route_workspace(resources)?;
        require_acp_session(&request.session_id.to_string(), &session_id)?;
        let path = verified_existing_file(&workspace, &request.path)?;
        let metadata = fs::metadata(&path).map_err(|_| callback_permission_error())?;
        if metadata.len() > MAX_ACP_FILE_BYTES {
            return Err(ChatError::new(
                ChatErrorCode::Protocol,
                "ACP file read exceeds the supported limit",
                true,
            ));
        }
        let text = fs::read_to_string(path).map_err(|_| callback_permission_error())?;
        if text.contains('\0') {
            return Err(ChatError::validation("path", "ACP file is not text"));
        }
        let start = request.line.unwrap_or(1).max(1).saturating_sub(1) as usize;
        let limit = request
            .limit
            .unwrap_or(MAX_ACP_FILE_LINES)
            .min(MAX_ACP_FILE_LINES) as usize;
        let content = text
            .lines()
            .skip(start)
            .take(limit)
            .collect::<Vec<_>>()
            .join("\n");
        serde_json::to_value(ReadTextFileResponse::new(content))
            .map_err(|_| protocol_error("ACP file read response"))
    })();
    match result {
        Ok(response) => resources
            .client
            .respond(rpc_id, response)
            .await
            .map_err(|error| error.to_chat_error("file read response")),
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn handle_write_text_file(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: WriteTextFileRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => return reject_callback(resources, rpc_id, "Invalid file write request").await,
    };
    let result = (|| {
        let (workspace, session_id) = route_workspace(resources)?;
        require_acp_session(&request.session_id.to_string(), &session_id)?;
        if request.content.len() as u64 > MAX_ACP_FILE_BYTES || request.content.contains('\0') {
            return Err(ChatError::validation(
                "content",
                "ACP file write exceeds the supported text limit",
            ));
        }
        let path = verified_write_path(&workspace, &request.path)?;
        atomic_write_text(&path, request.content.as_bytes())?;
        serde_json::to_value(WriteTextFileResponse::new())
            .map_err(|_| protocol_error("ACP file write response"))
    })();
    match result {
        Ok(response) => resources
            .client
            .respond(rpc_id, response)
            .await
            .map_err(|error| error.to_chat_error("file write response")),
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn handle_terminal_create(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: CreateTerminalRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => {
            return reject_callback(resources, rpc_id, "Invalid terminal create request").await
        }
    };
    let result = create_acp_terminal(&resources.terminal_callbacks, request).await;
    match result {
        Ok(terminal_id) => {
            let response = serde_json::to_value(CreateTerminalResponse::new(terminal_id))
                .map_err(|_| protocol_error("ACP terminal create response"))?;
            resources
                .client
                .respond(rpc_id, response)
                .await
                .map_err(|error| error.to_chat_error("terminal create response"))
        }
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn handle_terminal_output(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: TerminalOutputRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => {
            return reject_callback(resources, rpc_id, "Invalid terminal output request").await
        }
    };
    let result = (|| {
        resources
            .terminal_callbacks
            .verify_session(&request.session_id.to_string())?;
        let terminal = resources
            .terminal_callbacks
            .terminal(&request.terminal_id.to_string())?;
        terminal_output_response(&terminal)
    })();
    match result {
        Ok(response) => resources
            .client
            .respond(rpc_id, response)
            .await
            .map_err(|error| error.to_chat_error("terminal output response")),
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn handle_terminal_wait(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: WaitForTerminalExitRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => return reject_callback(resources, rpc_id, "Invalid terminal wait request").await,
    };
    resources
        .terminal_callbacks
        .verify_session(&request.session_id.to_string())?;
    let terminal = resources
        .terminal_callbacks
        .terminal(&request.terminal_id.to_string())?;
    let client = resources.client.clone();
    tokio::spawn(async move {
        let exit_status = loop {
            let notified = terminal.completed.notified();
            let current = {
                let state = match terminal.state.lock() {
                    Ok(state) => state,
                    Err(_) => return,
                };
                state.exit_status.clone()
            };
            if let Some(status) = current {
                break status;
            }
            notified.await;
        };
        let Ok(response) = serde_json::to_value(WaitForTerminalExitResponse::new(exit_status))
        else {
            return;
        };
        let _ = client.respond(rpc_id, response).await;
    });
    Ok(())
}

async fn handle_terminal_kill(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: KillTerminalRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => return reject_callback(resources, rpc_id, "Invalid terminal kill request").await,
    };
    let result = (|| {
        resources
            .terminal_callbacks
            .verify_session(&request.session_id.to_string())?;
        let terminal = resources
            .terminal_callbacks
            .terminal(&request.terminal_id.to_string())?;
        terminal
            .kill
            .try_send(())
            .map_err(|_| callback_permission_error())?;
        serde_json::to_value(KillTerminalResponse::new())
            .map_err(|_| protocol_error("ACP terminal kill response"))
    })();
    match result {
        Ok(response) => resources
            .client
            .respond(rpc_id, response)
            .await
            .map_err(|error| error.to_chat_error("terminal kill response")),
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn handle_terminal_release(
    resources: &CursorRouterResources,
    rpc_id: Value,
    params: Value,
) -> ChatResult<()> {
    let request: ReleaseTerminalRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(_) => {
            return reject_callback(resources, rpc_id, "Invalid terminal release request").await
        }
    };
    let result = (|| {
        resources
            .terminal_callbacks
            .verify_session(&request.session_id.to_string())?;
        let terminal = resources
            .terminal_callbacks
            .terminals
            .lock()
            .map_err(|_| driver_state_error())?
            .remove(&request.terminal_id.to_string())
            .ok_or_else(callback_permission_error)?;
        let _ = terminal.kill.try_send(());
        serde_json::to_value(ReleaseTerminalResponse::new())
            .map_err(|_| protocol_error("ACP terminal release response"))
    })();
    match result {
        Ok(response) => resources
            .client
            .respond(rpc_id, response)
            .await
            .map_err(|error| error.to_chat_error("terminal release response")),
        Err(error) => reject_callback(resources, rpc_id, &error.message).await,
    }
}

async fn create_acp_terminal(
    callbacks: &AcpTerminalCallbacks,
    request: CreateTerminalRequest,
) -> ChatResult<String> {
    callbacks.verify_session(&request.session_id.to_string())?;
    validate_terminal_command(&request)?;
    let cwd = match request.cwd.as_deref() {
        Some(path) => verified_directory(&callbacks.workspace, path)?,
        None => callbacks.workspace.clone(),
    };
    let output_limit = request
        .output_byte_limit
        .and_then(|limit| usize::try_from(limit).ok())
        .unwrap_or(DEFAULT_ACP_TERMINAL_OUTPUT_BYTES)
        .clamp(1, MAX_ACP_TERMINAL_OUTPUT_BYTES);
    let mut command = Command::new(&request.command);
    command
        .args(&request.args)
        .current_dir(cwd)
        .envs(request.env.iter().map(|value| (&value.name, &value.value)))
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| {
        ChatError::new(
            ChatErrorCode::DriverUnavailable,
            "ACP terminal command could not be started",
            true,
        )
    })?;
    let stdout = child.stdout.take().ok_or_else(callback_permission_error)?;
    let stderr = child.stderr.take().ok_or_else(callback_permission_error)?;
    let terminal_id = format!(
        "ganbaru-acp-terminal-{}",
        NEXT_ACP_TERMINAL.fetch_add(1, Ordering::Relaxed)
    );
    let state = Arc::new(Mutex::new(AcpTerminalState {
        output: Vec::new(),
        output_limit,
        truncated: false,
        exit_status: None,
    }));
    let completed = Arc::new(Notify::new());
    let (kill, mut kill_receiver) = mpsc::channel(1);
    let terminal = AcpTerminal {
        state: Arc::clone(&state),
        kill,
        completed: Arc::clone(&completed),
    };
    callbacks
        .terminals
        .lock()
        .map_err(|_| driver_state_error())?
        .insert(terminal_id.clone(), terminal);
    let stdout_task = tokio::spawn(capture_terminal_output(stdout, Arc::clone(&state)));
    let stderr_task = tokio::spawn(capture_terminal_output(stderr, Arc::clone(&state)));
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(40));
        let status = loop {
            tokio::select! {
                _ = kill_receiver.recv() => {
                    let _ = child.kill().await;
                    break child.wait().await.ok();
                }
                _ = interval.tick() => match child.try_wait() {
                    Ok(Some(status)) => break Some(status),
                    Ok(None) => {}
                    Err(_) => break None,
                }
            }
        };
        let _ = stdout_task.await;
        let _ = stderr_task.await;
        if let Ok(mut state) = state.lock() {
            state.exit_status = Some(match status.and_then(|status| status.code()) {
                Some(code) if code >= 0 => TerminalExitStatus::new().exit_code(code as u32),
                _ => TerminalExitStatus::new().signal("terminated".to_string()),
            });
        }
        completed.notify_waiters();
    });
    Ok(terminal_id)
}

async fn capture_terminal_output<R: tokio::io::AsyncRead + Unpin>(
    mut reader: R,
    state: Arc<Mutex<AcpTerminalState>>,
) {
    let mut buffer = [0u8; 8 * 1024];
    loop {
        let read = match reader.read(&mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(read) => read,
        };
        let Ok(mut state) = state.lock() else { break };
        append_terminal_bytes(&mut state, &buffer[..read]);
    }
}

fn append_terminal_bytes(state: &mut AcpTerminalState, bytes: &[u8]) {
    state.output.extend_from_slice(bytes);
    let excess = state.output.len().saturating_sub(state.output_limit);
    if excess > 0 {
        state.output.drain(..excess);
        state.truncated = true;
    }
}

fn terminal_output_response(terminal: &AcpTerminal) -> ChatResult<Value> {
    let state = terminal.state.lock().map_err(|_| driver_state_error())?;
    let output = String::from_utf8_lossy(&state.output).into_owned();
    let response =
        TerminalOutputResponse::new(output, state.truncated).exit_status(state.exit_status.clone());
    serde_json::to_value(response).map_err(|_| protocol_error("ACP terminal output response"))
}

fn validate_terminal_command(request: &CreateTerminalRequest) -> ChatResult<()> {
    if request.command.is_empty()
        || request.command.len() > 4_096
        || request.command.chars().any(|character| character == '\0')
        || request.args.len() > 10_000
        || request
            .args
            .iter()
            .any(|argument| argument.len() > 65_536 || argument.contains('\0'))
        || request.env.len() > 1_024
        || request.env.iter().any(|value| {
            value.name.is_empty()
                || value.name.len() > 1_024
                || value
                    .name
                    .chars()
                    .any(|character| matches!(character, '=' | '\0'))
                || value.value.len() > 65_536
                || value.value.contains('\0')
        })
    {
        return Err(ChatError::validation(
            "command",
            "ACP terminal command is invalid",
        ));
    }
    Ok(())
}

fn verified_directory(workspace: &Path, requested: &Path) -> ChatResult<PathBuf> {
    let relative = verified_relative_path(workspace, requested)?;
    reject_symlink_components(workspace, relative, false)?;
    let canonical = fs::canonicalize(requested).map_err(|_| callback_permission_error())?;
    if !canonical.starts_with(workspace) || !canonical.is_dir() {
        return Err(callback_permission_error());
    }
    Ok(canonical)
}

fn route_workspace(resources: &CursorRouterResources) -> ChatResult<(PathBuf, String)> {
    let route = resources.route.lock().map_err(|_| driver_state_error())?;
    Ok((route.workspace.clone(), route.provider_thread_id.clone()))
}

fn require_acp_session(actual: &str, expected: &str) -> ChatResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(callback_permission_error())
    }
}

fn verified_existing_file(workspace: &Path, requested: &Path) -> ChatResult<PathBuf> {
    let relative = verified_relative_path(workspace, requested)?;
    reject_symlink_components(workspace, relative, false)?;
    let canonical = fs::canonicalize(requested).map_err(|_| callback_permission_error())?;
    if !canonical.starts_with(workspace) || !canonical.is_file() {
        return Err(callback_permission_error());
    }
    Ok(canonical)
}

fn verified_write_path(workspace: &Path, requested: &Path) -> ChatResult<PathBuf> {
    let relative = verified_relative_path(workspace, requested)?;
    reject_symlink_components(workspace, relative, true)?;
    let parent = requested.parent().ok_or_else(callback_permission_error)?;
    let canonical_parent = fs::canonicalize(parent).map_err(|_| callback_permission_error())?;
    if !canonical_parent.starts_with(workspace) {
        return Err(callback_permission_error());
    }
    if requested.exists() {
        let metadata = fs::symlink_metadata(requested).map_err(|_| callback_permission_error())?;
        if !metadata.file_type().is_file() {
            return Err(callback_permission_error());
        }
    }
    Ok(requested.to_path_buf())
}

fn verified_relative_path<'a>(workspace: &Path, requested: &'a Path) -> ChatResult<&'a Path> {
    if !requested.is_absolute() {
        return Err(callback_permission_error());
    }
    let relative = requested
        .strip_prefix(workspace)
        .map_err(|_| callback_permission_error())?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(callback_permission_error());
    }
    Ok(relative)
}

fn reject_symlink_components(
    workspace: &Path,
    relative: &Path,
    allow_missing_file: bool,
) -> ChatResult<()> {
    let mut current = workspace.to_path_buf();
    let component_count = relative.components().count();
    for (index, component) in relative.components().enumerate() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(callback_permission_error())
            }
            Ok(_) => {}
            Err(_) if allow_missing_file && index + 1 == component_count => {}
            Err(_) => return Err(callback_permission_error()),
        }
    }
    Ok(())
}

fn atomic_write_text(path: &Path, content: &[u8]) -> ChatResult<()> {
    let parent = path.parent().ok_or_else(callback_permission_error)?;
    let permissions = fs::metadata(path)
        .ok()
        .map(|metadata| metadata.permissions());
    let sequence = NEXT_ACP_FILE_WRITE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".ganbaru-acp-write-{}-{sequence}",
        std::process::id()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|_| callback_permission_error())?;
        file.write_all(content)
            .map_err(|_| callback_permission_error())?;
        file.sync_all().map_err(|_| callback_permission_error())?;
        if let Some(permissions) = permissions {
            fs::set_permissions(&temporary, permissions)
                .map_err(|_| callback_permission_error())?;
        }
        fs::rename(&temporary, path).map_err(|_| callback_permission_error())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

async fn reject_callback(
    resources: &CursorRouterResources,
    rpc_id: Value,
    message: &str,
) -> ChatResult<()> {
    resources
        .client
        .respond_error(rpc_id, -32602, message)
        .await
        .map_err(|error| error.to_chat_error("ACP callback rejection"))
}

fn callback_permission_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "ACP callback cannot access this workspace path",
        true,
    )
}

#[cfg(test)]
mod callback_tests {
    use super::{
        append_terminal_bytes, atomic_write_text, verified_existing_file, verified_write_path,
        AcpTerminalCallbacks, AcpTerminalState,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ganbaru-acp-callback-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn file_callbacks_reject_cross_workspace_and_preserve_atomic_contents() {
        let workspace = TestDirectory::new();
        let outside = TestDirectory::new();
        let file = workspace.0.join("source.txt");
        fs::write(&file, "before").expect("test file should be written");
        assert_eq!(
            verified_existing_file(&workspace.0, &file).expect("file should be authorized"),
            file
        );
        assert!(verified_existing_file(&workspace.0, &outside.0.join("secret.txt")).is_err());
        let write_path = verified_write_path(&workspace.0, &workspace.0.join("created.txt"))
            .expect("new file should be authorized");
        atomic_write_text(&write_path, b"after").expect("atomic write should succeed");
        assert_eq!(fs::read_to_string(write_path).unwrap(), "after");
    }

    #[cfg(unix)]
    #[test]
    fn file_callbacks_reject_symlink_substitution() {
        use std::os::unix::fs::symlink;
        let workspace = TestDirectory::new();
        let outside = TestDirectory::new();
        let outside_file = outside.0.join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();
        let link = workspace.0.join("link.txt");
        symlink(&outside_file, &link).unwrap();
        assert!(verified_existing_file(&workspace.0, &link).is_err());
        assert!(verified_write_path(&workspace.0, &link).is_err());
    }

    #[test]
    fn terminal_callbacks_bound_output_and_session_ownership() {
        let callbacks = AcpTerminalCallbacks::new(PathBuf::from("/workspace"), "session-a".into());
        assert!(callbacks.verify_session("session-a").is_ok());
        assert!(callbacks.verify_session("session-b").is_err());
        let mut state = AcpTerminalState {
            output: Vec::new(),
            output_limit: 4,
            truncated: false,
            exit_status: None,
        };
        append_terminal_bytes(&mut state, b"abcdef");
        assert_eq!(state.output, b"cdef");
        assert!(state.truncated);
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
