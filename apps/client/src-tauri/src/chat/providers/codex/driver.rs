//! Codex provider driver over app-server stdio.

use super::home::*;
use super::normalizer::{CodexEventNormalizer, CodexRouteState};
use super::protocol::*;
use super::session::*;
use super::transport::{CodexRpcConnection, CodexRpcFailure};
use crate::chat::events::{
    CanonicalEvent, NotificationEvent, SessionConfiguredEvent, SessionExitedEvent,
    SessionStartedEvent,
};
use crate::chat::models::*;
use crate::chat::process::{spawn_provider_process, ProviderProcessConfig};
use crate::chat::providers::{
    DriverFuture, DriverOperationContext, ProviderDriver, ProviderEventSink,
};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio::task::JoinHandle;

const CODEX_STDERR_LIMIT_BYTES: usize = 256 * 1024;
const SESSION_GRACEFUL_STOP: Duration = Duration::from_millis(500);
const SESSION_FORCE_STOP: Duration = Duration::from_secs(2);
const PROVIDER_THREAD_NOTIFICATION_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_MODEL_PAGES: usize = 64;
const MAX_MODELS: usize = 2_048;
const MAX_HISTORY_ITEMS: usize = 100;
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub struct CodexProviderDriver {
    configuration: ProviderInstanceConfig,
    settings: CodexProviderSettings,
    live: Option<CodexLiveSession>,
    cached_models: Option<ProviderModelCatalog>,
    #[cfg(test)]
    connection_factory: Option<TestConnectionFactory>,
}

#[cfg(test)]
type TestConnectionFactory =
    Arc<dyn Fn(&Path) -> ChatResult<(CodexRpcConnection, CodexHomeLayout)> + Send + Sync>;

struct CodexLiveSession {
    connection: CodexRpcConnection,
    router_task: JoinHandle<()>,
    route: Arc<Mutex<CodexRouteState>>,
    pending_requests: PendingCodexRequests,
    normalizer: Arc<CodexEventNormalizer>,
    sink: Arc<dyn ProviderEventSink>,
    expected_shutdown: Arc<AtomicBool>,
    terminal_error: Arc<Mutex<Option<ChatError>>>,
    session_id: ProviderSessionId,
    workspace: PathBuf,
    effective_model: String,
    refresh_mcp_before_turn: bool,
}

impl Drop for CodexLiveSession {
    fn drop(&mut self) {
        self.expected_shutdown.store(true, Ordering::Release);
        self.router_task.abort();
    }
}

struct ProbeSnapshot {
    initialize: InitializeResponse,
    account: AccountReadResponse,
    models: Vec<ProviderModel>,
}

impl CodexProviderDriver {
    pub fn new(configuration: ProviderInstanceConfig) -> ChatResult<Self> {
        if configuration.family_id.as_str() != "codex" {
            return Err(ChatError::validation(
                "familyId",
                "Codex driver requires the Codex provider family",
            ));
        }
        let settings = CodexProviderSettings::parse(&configuration)?;
        Ok(Self {
            configuration,
            settings,
            live: None,
            cached_models: None,
            #[cfg(test)]
            connection_factory: None,
        })
    }

    #[cfg(test)]
    pub(super) fn set_connection_factory(&mut self, factory: TestConnectionFactory) {
        self.connection_factory = Some(factory);
    }

    pub fn metadata_read() -> ProviderFamilyMetadataRead {
        ProviderFamilyMetadataRead {
            family_id: ProviderFamilyId::new("codex")
                .expect("static Codex family ID must be valid"),
            display_name: "Codex".to_string(),
            configuration_schema_version: 1,
            supported_platforms: vec![
                "linux".to_string(),
                "windows".to_string(),
                "macos".to_string(),
            ],
            minimum_tested_cli_version: None,
            default_executable_candidates: vec!["codex".to_string()],
            implementation_status: ProviderImplementationStatus::Available,
            potential_capabilities: codex_capability_kinds(),
            unavailable_reason: None,
        }
    }

    async fn probe_snapshot(&self, context: &DriverOperationContext) -> ChatResult<ProbeSnapshot> {
        let working_directory = canonical_current_directory()?;
        let (mut connection, layout) = self.open_connection(&working_directory)?;
        let client = connection.client();
        let result = async {
            let initialize = client
                .request("initialize", initialize_params(), context)
                .await
                .map_err(|error| error.to_chat_error("initialize"))?;
            let initialize: InitializeResponse = decode_response(initialize, "initialize response")
                .map_err(|error| error.to_chat_error("initialize"))?;
            verify_reported_home(&layout, &initialize.codex_home)?;
            client
                .notify("initialized", json!({}))
                .await
                .map_err(|error| error.to_chat_error("initialized notification"))?;
            let account = client
                .request("account/read", json!({}), context)
                .await
                .map_err(|error| error.to_chat_error("account probe"))?;
            let account: AccountReadResponse = decode_response(account, "account response")
                .map_err(|error| error.to_chat_error("account probe"))?;
            let models = if account.account.is_some() || !account.requires_openai_auth {
                self.fetch_models(&client, context).await?
            } else {
                Vec::new()
            };
            Ok(ProbeSnapshot {
                initialize,
                account,
                models,
            })
        }
        .await;
        let stop_result = connection
            .stop(SESSION_GRACEFUL_STOP, SESSION_FORCE_STOP)
            .await;
        match result {
            Ok(snapshot) => {
                stop_result?;
                Ok(snapshot)
            }
            Err(error) => Err(error),
        }
    }

    async fn fetch_models(
        &self,
        client: &super::transport::CodexRpcClient,
        context: &DriverOperationContext,
    ) -> ChatResult<Vec<ProviderModel>> {
        let mut models = Vec::new();
        let mut cursor: Option<String> = None;
        let mut pages = 0;
        loop {
            if pages >= MAX_MODEL_PAGES || models.len() >= MAX_MODELS {
                return Err(ChatError::new(
                    ChatErrorCode::Protocol,
                    "Codex model catalog exceeds the supported bounds",
                    true,
                ));
            }
            let params = cursor
                .as_ref()
                .map(|cursor| json!({ "cursor": cursor }))
                .unwrap_or_else(|| json!({}));
            let response = client
                .request("model/list", params, context)
                .await
                .map_err(|error| error.to_chat_error("model discovery"))?;
            let response: ModelListResponse = decode_response(response, "model list response")
                .map_err(|error| error.to_chat_error("model discovery"))?;
            for model in response.data {
                if models.len() >= MAX_MODELS {
                    return Err(ChatError::new(
                        ChatErrorCode::Protocol,
                        "Codex model catalog exceeds the supported bounds",
                        true,
                    ));
                }
                models.push(provider_model(model)?);
            }
            pages += 1;
            cursor = response.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        if self.settings.allow_custom_models {
            let mut known = models
                .iter()
                .map(|model| model.id.as_str().to_string())
                .collect::<BTreeSet<_>>();
            for custom in &self.settings.custom_model_ids {
                if known.insert(custom.clone()) {
                    models.push(custom_provider_model(
                        custom,
                        self.settings
                            .custom_model_labels
                            .get(custom)
                            .map(String::as_str),
                    )?);
                }
            }
        }
        Ok(models)
    }

    fn open_connection(
        &self,
        working_directory: &Path,
    ) -> ChatResult<(CodexRpcConnection, CodexHomeLayout)> {
        #[cfg(test)]
        if let Some(factory) = self.connection_factory.as_ref() {
            return factory(working_directory);
        }
        let mut layout = resolve_codex_home_layout(&self.configuration, &self.settings)?;
        materialize_codex_shadow_home(&mut layout)?;
        verify_codex_shadow_home(&layout)?;
        let environment = codex_process_environment(&self.configuration, &layout)?;
        let executable = resolve_codex_executable(&self.configuration.executable, &environment)?;
        let mut arguments = executable.prefix_arguments;
        arguments.push("app-server".to_string());
        arguments.extend(validated_app_server_arguments(
            &self.configuration.launch_arguments,
        )?);
        let process = spawn_provider_process(ProviderProcessConfig {
            executable: executable.executable,
            arguments,
            working_directory: working_directory.to_path_buf(),
            environment,
            stderr_limit_bytes: CODEX_STDERR_LIMIT_BYTES,
        })?;
        Ok((CodexRpcConnection::from_process(process)?, layout))
    }

    async fn open_session(
        &mut self,
        input: SessionOpenInput,
        event_sink: Arc<dyn ProviderEventSink>,
        context: &DriverOperationContext,
    ) -> ChatResult<ProviderSessionSnapshot> {
        if self.live.is_some() {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Codex session is already running",
                true,
            ));
        }
        if input.provider_instance_id() != &self.configuration.instance_id {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Codex provider instance does not match the session request",
                false,
            ));
        }
        let workspace = canonical_verified_workspace(input.workspace())?;
        let (mut connection, layout) = self.open_connection(&workspace)?;
        let continuation_group_id = layout.continuation_group()?;
        input.verify_continuation(&continuation_group_id)?;
        let client = connection.client();
        let initialize = client
            .request("initialize", initialize_params(), context)
            .await
            .map_err(|error| error.to_chat_error("initialize"))?;
        let initialize: InitializeResponse = decode_response(initialize, "initialize response")
            .map_err(|error| error.to_chat_error("initialize"))?;
        verify_reported_home(&layout, &initialize.codex_home)?;
        client
            .notify("initialized", json!({}))
            .await
            .map_err(|error| error.to_chat_error("initialized notification"))?;

        let session_id = new_session_id(&self.configuration.instance_id)?;
        let requested_model = input.model_id();
        let route = Arc::new(Mutex::new(CodexRouteState::new(
            input.modes(),
            requested_model.cloned(),
        )));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let normalizer = Arc::new(CodexEventNormalizer::new(
            self.configuration.instance_id.clone(),
            input.thread_id().clone(),
            session_id.clone(),
        ));
        let (provider_thread_sender, provider_thread_receiver) = watch::channel(None);
        let expected_shutdown = Arc::new(AtomicBool::new(false));
        let terminal_error = Arc::new(Mutex::new(None));
        let inbound = connection.take_inbound()?;
        let router_task = spawn_codex_router(CodexRouterResources {
            client: client.clone(),
            inbound,
            normalizer: Arc::clone(&normalizer),
            route: Arc::clone(&route),
            pending_requests: Arc::clone(&pending_requests),
            sink: Arc::clone(&event_sink),
            provider_thread_sender,
            expected_shutdown: Arc::clone(&expected_shutdown),
            terminal_error: Arc::clone(&terminal_error),
        });

        let developer_instructions = input.developer_instructions();
        let start_params = thread_open_params(
            None,
            &workspace,
            input.modes(),
            requested_model,
            developer_instructions,
        )?;
        let (response, resumed) = match input.resume_provider_thread_id() {
            Some(provider_thread_id) => {
                let params = thread_open_params(
                    Some(provider_thread_id),
                    &workspace,
                    input.modes(),
                    requested_model,
                    developer_instructions,
                )?;
                match client.request("thread/resume", params, context).await {
                    Ok(response) => (response, true),
                    Err(error) if confirmed_resume_not_found(&error) => {
                        let state = route.lock().map_err(|_| driver_state_error())?.clone();
                        let warning = normalizer.event(
                            &state,
                            "thread/resume",
                            None,
                            None,
                            None,
                            CanonicalEvent::RuntimeWarning(NotificationEvent {
                                code: "codex_resume_not_found_fresh_start".to_string(),
                                title: "Codex continuation was not found".to_string(),
                                detail: Some(
                                    "A fresh native Codex thread was started while Ganbaru history was preserved."
                                        .to_string(),
                                ),
                            }),
                        )?;
                        event_sink.emit(warning).await?;
                        (
                            client
                                .request("thread/start", start_params, context)
                                .await
                                .map_err(|error| error.to_chat_error("fresh thread fallback"))?,
                            false,
                        )
                    }
                    Err(error) => return Err(error.to_chat_error("thread resume")),
                }
            }
            None => (
                client
                    .request("thread/start", start_params, context)
                    .await
                    .map_err(|error| error.to_chat_error("thread start"))?,
                false,
            ),
        };
        let response: ThreadOpenResponse = decode_response(response, "thread open response")
            .map_err(|error| error.to_chat_error("thread open"))?;
        verify_effective_safety(input.modes().safety_mode, &response)?;
        let provider_thread_id = ProviderThreadId::new(response.thread.id.clone())
            .map_err(|_| protocol_identifier_error("provider thread"))?;
        let observed_provider_thread = wait_for_provider_thread(
            provider_thread_receiver.clone(),
            provider_thread_id.as_str(),
            context,
        )
        .await?;
        if observed_provider_thread != provider_thread_id.as_str() {
            return Err(ChatError::new(
                ChatErrorCode::Protocol,
                "Codex thread notification did not match the open response",
                false,
            ));
        }
        let effective_model = ModelId::new(response.model.clone())
            .map_err(|_| protocol_identifier_error("effective model"))?;
        {
            let mut state = route.lock().map_err(|_| driver_state_error())?;
            state.provider_thread_id = Some(provider_thread_id.as_str().to_string());
            state.effective_model_id = Some(effective_model.clone());
            state.session_state = ProviderSessionState::Ready;
        }
        let started_at = now_utc()?;
        let resume_cursor = VersionedJson {
            schema_version: 1,
            value: json!({ "threadId": provider_thread_id.as_str() }),
        };
        let capabilities = codex_capabilities();
        let state = route.lock().map_err(|_| driver_state_error())?.clone();
        event_sink
            .emit(normalizer.event(
                &state,
                if resumed {
                    "session/resumed"
                } else {
                    "session/started"
                },
                None,
                None,
                None,
                CanonicalEvent::SessionStarted(SessionStartedEvent {
                    session_id: session_id.clone(),
                    state: ProviderSessionState::Ready,
                    provider_thread_id: Some(provider_thread_id.clone()),
                    resume_cursor: Some(resume_cursor.clone()),
                    effective_modes: input.modes(),
                    capability_overrides: capabilities.clone(),
                }),
            )?)
            .await?;
        event_sink
            .emit(normalizer.event(
                &state,
                "session/configured",
                None,
                None,
                None,
                CanonicalEvent::SessionConfigured(SessionConfiguredEvent {
                    session_id: session_id.clone(),
                    effective_modes: input.modes(),
                    effective_model_id: Some(effective_model.clone()),
                    effective_model_options: Vec::new(),
                }),
            )?)
            .await?;
        self.live = Some(CodexLiveSession {
            connection,
            router_task,
            route,
            pending_requests,
            normalizer,
            sink: event_sink,
            expected_shutdown,
            terminal_error,
            session_id: session_id.clone(),
            workspace,
            effective_model: effective_model.as_str().to_string(),
            refresh_mcp_before_turn: self.settings.refresh_mcp_before_turn
                || self
                    .configuration
                    .launch_arguments
                    .iter()
                    .any(|argument| argument.contains("mcp_servers.")),
        });
        Ok(ProviderSessionSnapshot {
            session_id,
            state: ProviderSessionState::Ready,
            provider_thread_id: Some(provider_thread_id),
            continuation_group_id,
            resume_cursor: Some(resume_cursor),
            effective_modes: input.modes(),
            capabilities,
            started_at,
        })
    }

    fn live_mut(&mut self, session_id: &ProviderSessionId) -> ChatResult<&mut CodexLiveSession> {
        let live = self.live.as_mut().ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::DriverUnavailable,
                "Codex session is not running",
                true,
            )
        })?;
        if &live.session_id != session_id {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Codex session identity does not match the active session",
                false,
            ));
        }
        if let Some(error) = live
            .terminal_error
            .lock()
            .map_err(|_| driver_state_error())?
            .clone()
        {
            return Err(error);
        }
        Ok(live)
    }
}

impl ProviderDriver for CodexProviderDriver {
    fn metadata(&self) -> ProviderFamilyMetadataRead {
        Self::metadata_read()
    }

    fn instance_configuration(&self) -> &ProviderInstanceConfig {
        &self.configuration
    }

    fn capabilities(&self) -> ProviderCapabilities {
        codex_capabilities()
    }

    fn probe<'a>(
        &'a mut self,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ProviderProbeResult> {
        Box::pin(async move {
            let checked_at = now_utc()?;
            match self.probe_snapshot(context).await {
                Ok(snapshot) => {
                    let authenticated = snapshot.account.account.is_some()
                        || !snapshot.account.requires_openai_auth;
                    let state = if authenticated {
                        ProbeState::Healthy
                    } else {
                        ProbeState::AuthenticationRequired
                    };
                    let account_label = snapshot.account.account.as_ref().map(|account| {
                        account
                            .email
                            .clone()
                            .unwrap_or_else(|| account.account_type.clone())
                    });
                    self.cached_models = Some(ProviderModelCatalog {
                        instance_id: self.configuration.instance_id.clone(),
                        models: snapshot.models,
                        source: ModelCatalogSource::Provider,
                        discovered_at: checked_at.clone(),
                        stale: false,
                    });
                    Ok(ProviderProbeResult {
                        instance_id: self.configuration.instance_id.clone(),
                        state,
                        version: parse_user_agent_version(&snapshot.initialize.user_agent),
                        account_label,
                        capabilities: codex_capabilities(),
                        checked_at,
                        detail: (!authenticated)
                            .then_some("Codex requires authentication".to_string()),
                    })
                }
                Err(error) if context.is_cancelled() => Err(error),
                Err(error) => Ok(ProviderProbeResult {
                    instance_id: self.configuration.instance_id.clone(),
                    state: probe_state_for_error(error.code),
                    version: None,
                    account_label: None,
                    capabilities: codex_capabilities(),
                    checked_at,
                    detail: Some(probe_detail(error.code).to_string()),
                }),
            }
        })
    }

    fn discover_models<'a>(
        &'a mut self,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ProviderModelCatalog> {
        Box::pin(async move {
            let snapshot = self.probe_snapshot(context).await?;
            if snapshot.account.account.is_none() && snapshot.account.requires_openai_auth {
                return Err(ChatError::new(
                    ChatErrorCode::AuthenticationRequired,
                    "Codex authentication is required before model discovery",
                    true,
                ));
            }
            let catalog = ProviderModelCatalog {
                instance_id: self.configuration.instance_id.clone(),
                models: snapshot.models,
                source: ModelCatalogSource::Provider,
                discovered_at: now_utc()?,
                stale: false,
            };
            self.cached_models = Some(catalog.clone());
            Ok(catalog)
        })
    }

    fn derive_continuation_group<'a>(
        &'a mut self,
        request: ContinuationGroupRequest,
        _context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ContinuationGroupId> {
        Box::pin(async move {
            let layout = resolve_codex_home_layout(&self.configuration, &self.settings)?;
            if let Some(expected) = request.normalized_provider_home.as_deref() {
                let expected = std::fs::canonicalize(expected).map_err(|_| {
                    ChatError::validation(
                        "normalizedProviderHome",
                        "Codex continuation home is unavailable",
                    )
                })?;
                if expected != layout.shared_home {
                    return Err(ChatError::new(
                        ChatErrorCode::Conflict,
                        "Codex home change requires a thread fork",
                        true,
                    ));
                }
            }
            layout.continuation_group()
        })
    }

    fn start_session<'a>(
        &'a mut self,
        request: StartSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ProviderSessionSnapshot> {
        Box::pin(async move {
            self.open_session(SessionOpenInput::Fresh(request), event_sink, context)
                .await
        })
    }

    fn resume_session<'a>(
        &'a mut self,
        request: ResumeSessionRequest,
        event_sink: Arc<dyn ProviderEventSink>,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ProviderSessionSnapshot> {
        Box::pin(async move {
            self.open_session(SessionOpenInput::Resume(request), event_sink, context)
                .await
        })
    }

    fn send_turn<'a>(
        &'a mut self,
        request: SendTurnRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, TurnDispatchReceipt> {
        Box::pin(async move {
            let live = self.live_mut(&request.session_id)?;
            let client = live.connection.client();
            if live.refresh_mcp_before_turn {
                client
                    .request("config/mcpServer/reload", json!({}), context)
                    .await
                    .map_err(|error| error.to_chat_error("MCP refresh"))?;
            }
            let provider_thread_id = live
                .route
                .lock()
                .map_err(|_| driver_state_error())?
                .provider_thread_id
                .clone()
                .ok_or_else(|| protocol_identifier_error("provider thread"))?;
            let params = turn_start_params(
                &provider_thread_id,
                &live.workspace,
                &live.effective_model,
                &request,
            )?;
            {
                let mut state = live.route.lock().map_err(|_| driver_state_error())?;
                state.active_chat_turn_id = Some(request.turn_id.clone());
                state.active_provider_turn_id = None;
                state.modes = request.modes;
                if let Some(model) = request.model_id.clone() {
                    state.effective_model_id = Some(model);
                }
            }
            let response = match client.request("turn/start", params, context).await {
                Ok(response) => response,
                Err(error) => {
                    if let Ok(mut state) = live.route.lock() {
                        state.active_chat_turn_id = None;
                    }
                    return Err(error.to_chat_error("turn start"));
                }
            };
            let response: TurnStartResponse = decode_response(response, "turn start response")
                .map_err(|error| error.to_chat_error("turn start"))?;
            let provider_turn_id = ProviderTurnId::new(response.turn.id.clone())
                .map_err(|_| protocol_identifier_error("provider turn"))?;
            if response.turn.status != "inProgress" {
                return Err(ChatError::new(
                    ChatErrorCode::Protocol,
                    "Codex did not accept the turn as in progress",
                    true,
                ));
            }
            {
                let mut state = live.route.lock().map_err(|_| driver_state_error())?;
                if let Some(observed) = state.active_provider_turn_id.as_deref() {
                    if observed != provider_turn_id.as_str() {
                        return Err(ChatError::new(
                            ChatErrorCode::Protocol,
                            "Codex turn notification did not match the response",
                            false,
                        ));
                    }
                }
                state.active_provider_turn_id = Some(provider_turn_id.as_str().to_string());
                state.session_state = ProviderSessionState::Active;
            }
            if let Some(model) = request.model_id.as_ref() {
                live.effective_model = model.as_str().to_string();
            }
            Ok(TurnDispatchReceipt {
                turn_id: request.turn_id,
                state: ChatTurnState::Active,
                provider_turn_id: Some(provider_turn_id),
                accepted_at: now_utc()?,
            })
        })
    }

    fn steer_turn<'a>(
        &'a mut self,
        request: SteerTurnRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async move {
            if request.prompt.is_empty() || request.prompt.len() > 4 * 1024 * 1024 {
                return Err(ChatError::validation(
                    "prompt",
                    "Codex steering text is invalid",
                ));
            }
            let live = self.live_mut(&request.session_id)?;
            let state = live.route.lock().map_err(|_| driver_state_error())?.clone();
            let provider_thread_id = state
                .provider_thread_id
                .ok_or_else(|| protocol_identifier_error("provider thread"))?;
            let provider_turn_id = state
                .active_provider_turn_id
                .ok_or_else(|| ChatError::invalid_transition("Codex has no active turn"))?;
            live.connection
                .client()
                .request(
                    "turn/steer",
                    json!({
                        "threadId": provider_thread_id,
                        "expectedTurnId": provider_turn_id,
                        "input": [{ "type": "text", "text": request.prompt }],
                        "clientUserMessageId": request.command.client_command_id.as_str(),
                    }),
                    context,
                )
                .await
                .map_err(|error| error.to_chat_error("turn steer"))?;
            Ok(operation_receipt(context, "Codex accepted steering input"))
        })
    }

    fn interrupt_turn<'a>(
        &'a mut self,
        request: InterruptTurnRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async move {
            let live = self.live_mut(&request.session_id)?;
            interrupt_live_turn(live, context).await?;
            Ok(operation_receipt(context, "Codex turn interrupt requested"))
        })
    }

    fn resolve_approval<'a>(
        &'a mut self,
        request: ResolveApprovalRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async move {
            let live = self.live_mut(&request.session_id)?;
            resolve_codex_approval(
                &live.connection.client(),
                &live.pending_requests,
                &live.normalizer,
                &live.route,
                &live.sink,
                &request,
                context,
            )
            .await?;
            Ok(DriverOperationReceipt {
                accepted: true,
                operation_id: request.command.client_command_id.as_str().to_string(),
                detail: Some("Codex approval response accepted".to_string()),
            })
        })
    }

    fn resolve_user_input<'a>(
        &'a mut self,
        request: ResolveUserInputRequest,
        _context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async move {
            let live = self.live_mut(&request.session_id)?;
            resolve_codex_user_input(
                &live.connection.client(),
                &live.pending_requests,
                &live.normalizer,
                &live.route,
                &live.sink,
                &request,
            )
            .await?;
            Ok(DriverOperationReceipt {
                accepted: true,
                operation_id: request.command.client_command_id.as_str().to_string(),
                detail: Some("Codex user input response accepted".to_string()),
            })
        })
    }

    fn rollback<'a>(
        &'a mut self,
        _request: RollbackRequest,
        _context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async {
            Err(ChatError::unsupported(
                "Codex native rollback is not supported",
            ))
        })
    }

    fn read_history<'a>(
        &'a mut self,
        request: ReadHistoryRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, ProviderHistoryPage> {
        Box::pin(async move {
            if request.cursor.is_some() {
                return Err(ChatError::unsupported(
                    "Codex history cursor paging is not yet required by the driver contract",
                ));
            }
            let live = self.live_mut(&request.session_id)?;
            let provider_thread_id = live
                .route
                .lock()
                .map_err(|_| driver_state_error())?
                .provider_thread_id
                .clone()
                .ok_or_else(|| protocol_identifier_error("provider thread"))?;
            let response = live
                .connection
                .client()
                .request(
                    "thread/read",
                    json!({ "threadId": provider_thread_id, "includeTurns": true }),
                    context,
                )
                .await
                .map_err(|error| error.to_chat_error("thread history"))?;
            provider_history(response, request.limit)
        })
    }

    fn stop_session<'a>(
        &'a mut self,
        request: StopSessionRequest,
        context: &'a DriverOperationContext,
    ) -> DriverFuture<'a, DriverOperationReceipt> {
        Box::pin(async move {
            let Some(mut live) = self.live.take() else {
                return Ok(operation_receipt(
                    context,
                    "Codex session was already stopped",
                ));
            };
            if live.session_id != request.session_id {
                self.live = Some(live);
                return Err(ChatError::new(
                    ChatErrorCode::Conflict,
                    "Codex session identity does not match the active session",
                    false,
                ));
            }
            live.expected_shutdown.store(true, Ordering::Release);
            let _ = interrupt_live_turn(&mut live, context).await;
            live.sink.flush().await?;
            let state = live.route.lock().map_err(|_| driver_state_error())?.clone();
            live.sink
                .emit(live.normalizer.event(
                    &state,
                    "session/exited",
                    None,
                    None,
                    None,
                    CanonicalEvent::SessionExited(SessionExitedEvent {
                        session_id: live.session_id.clone(),
                        expected: true,
                        exit_code: None,
                        reason: Some("Codex session stopped".to_string()),
                    }),
                )?)
                .await?;
            live.sink.flush().await?;
            live.connection
                .stop(
                    if request.force {
                        Duration::ZERO
                    } else {
                        SESSION_GRACEFUL_STOP
                    },
                    SESSION_FORCE_STOP,
                )
                .await?;
            let _ = tokio::time::timeout(SESSION_GRACEFUL_STOP, &mut live.router_task).await;
            if !live.router_task.is_finished() {
                live.router_task.abort();
            }
            Ok(operation_receipt(context, "Codex session stopped"))
        })
    }
}

enum SessionOpenInput {
    Fresh(StartSessionRequest),
    Resume(ResumeSessionRequest),
}

impl SessionOpenInput {
    fn provider_instance_id(&self) -> &ProviderInstanceId {
        match self {
            Self::Fresh(request) => &request.provider_instance_id,
            Self::Resume(request) => &request.provider_instance_id,
        }
    }

    fn thread_id(&self) -> &ChatThreadId {
        match self {
            Self::Fresh(request) => &request.thread_id,
            Self::Resume(request) => &request.thread_id,
        }
    }

    fn workspace(&self) -> &VerifiedWorkspaceContext {
        match self {
            Self::Fresh(request) => &request.workspace,
            Self::Resume(request) => &request.workspace,
        }
    }

    fn modes(&self) -> TurnModeSnapshot {
        match self {
            Self::Fresh(request) => request.modes,
            Self::Resume(request) => request.modes,
        }
    }

    fn model_id(&self) -> Option<&ModelId> {
        match self {
            Self::Fresh(request) => request.model_id.as_ref(),
            Self::Resume(_) => None,
        }
    }

    fn developer_instructions(&self) -> Option<&str> {
        None
    }

    fn resume_provider_thread_id(&self) -> Option<&str> {
        match self {
            Self::Fresh(_) => None,
            Self::Resume(request) => Some(request.provider_thread_id.as_str()),
        }
    }

    fn verify_continuation(&self, actual: &ContinuationGroupId) -> ChatResult<()> {
        let Self::Resume(request) = self else {
            return Ok(());
        };
        if &request.continuation_group_id != actual {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Codex home is incompatible with this thread. Fork the thread to continue.",
                true,
            ));
        }
        if request.resume_cursor.schema_version != 1
            || request
                .resume_cursor
                .value
                .get("threadId")
                .and_then(Value::as_str)
                != Some(request.provider_thread_id.as_str())
        {
            return Err(ChatError::validation(
                "resumeCursor",
                "Codex resume cursor is invalid",
            ));
        }
        Ok(())
    }
}

fn codex_capability_kinds() -> Vec<ProviderCapability> {
    vec![
        ProviderCapability::NativeResume,
        ProviderCapability::NativePlan,
        ProviderCapability::Steering,
        ProviderCapability::DynamicModelChange,
        ProviderCapability::Images,
        ProviderCapability::FileReferences,
        ProviderCapability::Skills,
        ProviderCapability::SlashCommands,
        ProviderCapability::Approvals,
        ProviderCapability::StructuredQuestions,
        ProviderCapability::ReasoningSummaries,
        ProviderCapability::StructuredPlans,
        ProviderCapability::ContextUsage,
        ProviderCapability::McpStatus,
        ProviderCapability::AccountStatus,
        ProviderCapability::RateLimitStatus,
        ProviderCapability::ProviderDiffs,
        ProviderCapability::TaskActivity,
    ]
}

fn codex_capabilities() -> ProviderCapabilities {
    ProviderCapabilities {
        entries: codex_capability_kinds()
            .into_iter()
            .map(|capability| ProviderCapabilitySupport {
                capability,
                supported: true,
                explanation: None,
            })
            .collect(),
    }
}

pub(super) fn validated_app_server_arguments(arguments: &[String]) -> ChatResult<Vec<String>> {
    let mut output = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--strict-config" {
            output.push(argument.clone());
            index += 1;
            continue;
        }
        if argument.starts_with("--config=")
            || argument.starts_with("-c=")
            || argument.starts_with("--enable=")
            || argument.starts_with("--disable=")
        {
            output.push(argument.clone());
            index += 1;
            continue;
        }
        if matches!(
            argument.as_str(),
            "--config" | "-c" | "--enable" | "--disable"
        ) {
            let value = arguments.get(index + 1).ok_or_else(|| {
                ChatError::validation("launchArguments", "Codex option requires a value")
            })?;
            if value.is_empty() || value.starts_with('-') {
                return Err(ChatError::validation(
                    "launchArguments",
                    "Codex option value is invalid",
                ));
            }
            output.push(argument.clone());
            output.push(value.clone());
            index += 2;
            continue;
        }
        return Err(ChatError::validation(
            "launchArguments",
            "Codex app-server accepts only configuration and feature arguments",
        ));
    }
    Ok(output)
}

async fn interrupt_live_turn(
    live: &mut CodexLiveSession,
    context: &DriverOperationContext,
) -> ChatResult<()> {
    let state = live.route.lock().map_err(|_| driver_state_error())?.clone();
    let (Some(provider_thread_id), Some(provider_turn_id)) =
        (state.provider_thread_id, state.active_provider_turn_id)
    else {
        return Ok(());
    };
    live.connection
        .client()
        .request(
            "turn/interrupt",
            json!({ "threadId": provider_thread_id, "turnId": provider_turn_id }),
            context,
        )
        .await
        .map_err(|error| error.to_chat_error("turn interrupt"))?;
    Ok(())
}

async fn wait_for_provider_thread(
    mut receiver: watch::Receiver<Option<String>>,
    expected: &str,
    context: &DriverOperationContext,
) -> ChatResult<String> {
    let timeout = context
        .deadline
        .saturating_duration_since(Instant::now())
        .min(PROVIDER_THREAD_NOTIFICATION_TIMEOUT);
    let wait = async {
        loop {
            if let Some(provider_thread) = receiver.borrow().clone() {
                return Ok(provider_thread);
            }
            receiver.changed().await.map_err(|_| {
                ChatError::new(
                    ChatErrorCode::TransportUnavailable,
                    "Codex thread notification stream closed",
                    true,
                )
            })?;
        }
    };
    let provider_thread = tokio::time::timeout(timeout, wait).await.map_err(|_| {
        ChatError::new(
            ChatErrorCode::Protocol,
            "Codex did not confirm the provider thread before the deadline",
            true,
        )
    })??;
    if provider_thread != expected {
        return Err(ChatError::new(
            ChatErrorCode::Protocol,
            "Codex confirmed a different provider thread",
            false,
        ));
    }
    Ok(provider_thread)
}

fn provider_history(response: Value, limit: u32) -> ChatResult<ProviderHistoryPage> {
    let limit = usize::try_from(limit)
        .unwrap_or(MAX_HISTORY_ITEMS)
        .min(MAX_HISTORY_ITEMS);
    let thread = response
        .get("thread")
        .and_then(Value::as_object)
        .ok_or_else(|| protocol_identifier_error("history thread"))?;
    let turns = thread
        .get("turns")
        .and_then(Value::as_array)
        .ok_or_else(|| protocol_identifier_error("history turns"))?;
    let mut items = Vec::new();
    for turn in turns {
        let Some(turn) = turn.as_object() else {
            return Err(protocol_identifier_error("history turn"));
        };
        let provider_turn_id = turn
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| ProviderTurnId::new(id.to_string()).ok());
        let Some(turn_items) = turn.get("items").and_then(Value::as_array) else {
            continue;
        };
        for item in turn_items {
            if items.len() >= limit {
                break;
            }
            let object = item
                .as_object()
                .ok_or_else(|| protocol_identifier_error("history item"))?;
            let provider_item_id = object
                .get("id")
                .and_then(Value::as_str)
                .and_then(|id| ProviderItemId::new(id.to_string()).ok());
            let kind = object
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            items.push(ProviderHistoryItem {
                provider_item_id,
                provider_turn_id: provider_turn_id.clone(),
                kind,
                data: VersionedJson {
                    schema_version: 1,
                    value: item.clone(),
                },
            });
        }
        if items.len() >= limit {
            break;
        }
    }
    Ok(ProviderHistoryPage {
        items,
        next_cursor: None,
    })
}

fn canonical_verified_workspace(workspace: &VerifiedWorkspaceContext) -> ChatResult<PathBuf> {
    let path = PathBuf::from(&workspace.canonical_path);
    if !path.is_absolute() || !path.is_dir() {
        return Err(ChatError::validation(
            "workspace",
            "Codex workspace binding is invalid",
        ));
    }
    std::fs::canonicalize(path).map_err(|_| {
        ChatError::validation("workspace", "Codex workspace could not be canonicalized")
    })
}

fn canonical_current_directory() -> ChatResult<PathBuf> {
    let current = std::env::current_dir().map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Codex probe working directory is unavailable",
            true,
        )
    })?;
    std::fs::canonicalize(current).map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Codex probe working directory is unavailable",
            true,
        )
    })
}

fn verify_reported_home(layout: &CodexHomeLayout, reported: &Path) -> ChatResult<()> {
    let reported = std::fs::canonicalize(reported).map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Codex reported an unavailable home directory",
            true,
        )
    })?;
    if reported != layout.effective_home {
        return Err(ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Codex app-server used a different home directory",
            true,
        ));
    }
    Ok(())
}

fn verify_effective_safety(mode: SafetyMode, response: &ThreadOpenResponse) -> ChatResult<()> {
    let expected = safety_settings(mode);
    if response.approval_policy.as_str() != Some(expected.approval_policy) {
        return Err(permission_error());
    }
    let sandbox = response
        .sandbox
        .get("type")
        .and_then(Value::as_str)
        .or_else(|| response.sandbox.as_str());
    if sandbox != Some(expected.turn_sandbox_type) && sandbox != Some(expected.sandbox) {
        return Err(permission_error());
    }
    Ok(())
}

pub(super) fn confirmed_resume_not_found(error: &CodexRpcFailure) -> bool {
    let CodexRpcFailure::Remote { code, message } = error else {
        return false;
    };
    if !matches!(*code, -32602 | -32000 | -32001) {
        return false;
    }
    let message = message.to_ascii_lowercase();
    message.contains("thread")
        && [
            "not found",
            "missing thread",
            "no such thread",
            "unknown thread",
            "does not exist",
            "no rollout found",
        ]
        .iter()
        .any(|fragment| message.contains(fragment))
}

fn new_session_id(instance_id: &ProviderInstanceId) -> ChatResult<ProviderSessionId> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    ProviderSessionId::new(format!(
        "codex-{}-{now:x}-{sequence:x}",
        instance_id.as_str()
    ))
    .map_err(|_| protocol_identifier_error("session"))
}

fn now_utc() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Internal,
            "Codex timestamp could not be created",
            false,
        )
    })
}

fn parse_user_agent_version(user_agent: &str) -> Option<String> {
    let (_, suffix) = user_agent.split_once('/')?;
    suffix.split_whitespace().next().map(str::to_string)
}

fn probe_state_for_error(code: ChatErrorCode) -> ProbeState {
    match code {
        ChatErrorCode::ExecutableMissing => ProbeState::ExecutableMissing,
        ChatErrorCode::UnsupportedVersion | ChatErrorCode::Protocol => {
            ProbeState::UnsupportedVersion
        }
        ChatErrorCode::AuthenticationRequired => ProbeState::AuthenticationRequired,
        ChatErrorCode::ConfigurationInvalid | ChatErrorCode::Validation => {
            ProbeState::ConfigurationInvalid
        }
        _ => ProbeState::TransportUnavailable,
    }
}

fn probe_detail(code: ChatErrorCode) -> &'static str {
    match probe_state_for_error(code) {
        ProbeState::ExecutableMissing => "Codex executable is unavailable",
        ProbeState::UnsupportedVersion => "Codex app-server protocol is unsupported",
        ProbeState::AuthenticationRequired => "Codex authentication is required",
        ProbeState::ConfigurationInvalid => "Codex configuration is invalid",
        ProbeState::TransportUnavailable => "Codex app-server transport is unavailable",
        ProbeState::Healthy => "Codex is ready",
    }
}

fn operation_receipt(context: &DriverOperationContext, detail: &str) -> DriverOperationReceipt {
    DriverOperationReceipt {
        accepted: true,
        operation_id: context.operation_id.clone(),
        detail: Some(detail.to_string()),
    }
}

fn permission_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "Codex did not confirm the requested approval and sandbox policy",
        false,
    )
}

fn protocol_identifier_error(label: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        format!("Codex returned an invalid {label} identifier"),
        false,
    )
}

fn driver_state_error() -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Codex driver state is unavailable",
        false,
    )
}
