//! Cursor provider driver and live-session ownership.

use super::executable::*;
use super::interactions::PendingCursorRequests;
use super::normalizer::{CursorEventNormalizer, CursorRouteState};
use super::protocol::*;
use super::session::*;
use super::transport::AcpRpcConnection;
use crate::chat::events::*;
use crate::chat::models::*;
use crate::chat::providers::{DriverOperationContext, ProviderEventSink};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::task::JoinHandle;

pub const SESSION_GRACEFUL_STOP: Duration = Duration::from_millis(500);
pub const SESSION_FORCE_STOP: Duration = Duration::from_secs(2);
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

pub struct CursorProviderDriver {
    pub(super) configuration: ProviderInstanceConfig,
    pub(super) settings: CursorProviderSettings,
    pub(super) live: Option<CursorLiveSession>,
    pub(super) cached_models: Option<ProviderModelCatalog>,
    #[cfg(test)]
    connection_factory: Option<TestConnectionFactory>,
}

#[cfg(test)]
type TestConnectionFactory =
    Arc<dyn Fn(&Path) -> ChatResult<(AcpRpcConnection, CursorAbout)> + Send + Sync>;

pub(super) struct CursorLiveSession {
    pub connection: AcpRpcConnection,
    pub router_task: JoinHandle<()>,
    pub prompt_task: Option<JoinHandle<()>>,
    pub route: Arc<Mutex<CursorRouteState>>,
    pub setup: Arc<Mutex<AcpSessionSetup>>,
    pub pending: PendingCursorRequests,
    pub normalizer: Arc<CursorEventNormalizer>,
    pub capabilities: ProviderCapabilities,
    pub sink: Arc<dyn ProviderEventSink>,
    pub expected_shutdown: Arc<AtomicBool>,
    pub terminal_error: Arc<Mutex<Option<ChatError>>>,
    pub session_id: ProviderSessionId,
}

impl Drop for CursorLiveSession {
    fn drop(&mut self) {
        self.expected_shutdown.store(true, Ordering::Release);
        self.router_task.abort();
        if let Some(task) = self.prompt_task.as_ref() {
            task.abort();
        }
    }
}

pub(super) enum CursorSessionInput {
    Fresh(StartSessionRequest),
    Resume(ResumeSessionRequest),
}

impl CursorSessionInput {
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

    fn provider_instance_id(&self) -> &ProviderInstanceId {
        match self {
            Self::Fresh(request) => &request.provider_instance_id,
            Self::Resume(request) => &request.provider_instance_id,
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

    fn model_options(&self) -> &[ModelOptionSelection] {
        match self {
            Self::Fresh(request) => &request.model_options,
            Self::Resume(_) => &[],
        }
    }

    fn resume_cursor(&self) -> ChatResult<Option<AcpResumeCursor>> {
        match self {
            Self::Fresh(_) => Ok(None),
            Self::Resume(request) => parse_resume_cursor(&request.resume_cursor).map(Some),
        }
    }

    fn expected_continuation(&self) -> Option<&ContinuationGroupId> {
        match self {
            Self::Fresh(_) => None,
            Self::Resume(request) => Some(&request.continuation_group_id),
        }
    }

    fn is_resume(&self) -> bool {
        matches!(self, Self::Resume(_))
    }
}

impl CursorProviderDriver {
    pub fn new(configuration: ProviderInstanceConfig) -> ChatResult<Self> {
        if configuration.family_id.as_str() != "cursor" {
            return Err(ChatError::validation(
                "familyId",
                "Cursor driver requires the Cursor provider family",
            ));
        }
        let settings = CursorProviderSettings::parse(&configuration)?;
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
            family_id: ProviderFamilyId::new("cursor")
                .expect("static Cursor family ID must be valid"),
            display_name: "Cursor".to_string(),
            configuration_schema_version: 1,
            supported_platforms: vec![
                "linux".to_string(),
                "windows".to_string(),
                "macos".to_string(),
            ],
            minimum_tested_cli_version: Some(MINIMUM_CURSOR_VERSION.to_string()),
            default_executable_candidates: vec!["cursor-agent".to_string(), "agent".to_string()],
            implementation_status: ProviderImplementationStatus::Available,
            potential_capabilities: cursor_capability_kinds(),
            unavailable_reason: None,
        }
    }

    pub(super) async fn open_connection(
        &self,
        workspace: &Path,
    ) -> ChatResult<(AcpRpcConnection, CursorAbout)> {
        #[cfg(test)]
        if let Some(factory) = self.connection_factory.as_ref() {
            return factory(workspace);
        }
        let environment = process_environment(&self.configuration)?;
        let executable = resolve_executable(&self.configuration.executable, &environment)?;
        let about = probe_about(&executable, workspace, environment).await?;
        ensure_supported_version(about.version)?;
        let process = spawn_connection_process(&self.configuration, &self.settings, workspace)?;
        Ok((AcpRpcConnection::from_process(process)?, about))
    }

    pub(super) async fn probe_snapshot(
        &self,
        context: &DriverOperationContext,
    ) -> ChatResult<(AcpStartedSession, CursorAbout)> {
        let workspace = canonical_current_directory()?;
        let (mut connection, about) = self.open_connection(&workspace).await?;
        let result = initialize_session(
            &connection,
            workspace.to_string_lossy().as_ref(),
            None,
            TurnModeSnapshot {
                safety_mode: SafetyMode::Supervised,
                interaction_mode: InteractionMode::Build,
            },
            None,
            &[],
            context,
        )
        .await;
        let stop = connection
            .stop(SESSION_GRACEFUL_STOP, SESSION_FORCE_STOP)
            .await;
        match result {
            Ok(started) => {
                stop?;
                Ok((started, about))
            }
            Err(error) => Err(error),
        }
    }

    pub(super) async fn open_session(
        &mut self,
        input: CursorSessionInput,
        sink: Arc<dyn ProviderEventSink>,
        context: &DriverOperationContext,
    ) -> ChatResult<ProviderSessionSnapshot> {
        if self.live.is_some() {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Cursor session is already running",
                true,
            ));
        }
        if input.provider_instance_id() != &self.configuration.instance_id {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Cursor provider instance does not match the session request",
                false,
            ));
        }
        let workspace = canonical_verified_workspace(input.workspace())?;
        let cursor = input.resume_cursor()?;
        let (mut connection, about) = self.open_connection(&workspace).await?;
        let group = continuation_group(
            &self.configuration,
            &self.settings,
            about.account_label.as_deref(),
        )?;
        if input
            .expected_continuation()
            .is_some_and(|expected| expected != &group)
        {
            let _ = connection
                .stop(SESSION_GRACEFUL_STOP, SESSION_FORCE_STOP)
                .await;
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Cursor account context change requires a thread fork",
                true,
            ));
        }
        let started = match initialize_session(
            &connection,
            workspace.to_string_lossy().as_ref(),
            cursor.as_ref().map(|cursor| cursor.session_id.as_str()),
            input.modes(),
            input.model_id(),
            input.model_options(),
            context,
        )
        .await
        {
            Ok(started) => started,
            Err(error) => {
                let _ = connection
                    .stop(SESSION_GRACEFUL_STOP, SESSION_FORCE_STOP)
                    .await;
                return Err(error);
            }
        };
        let local_session_id = new_session_id(&self.configuration.instance_id)?;
        let provider_thread_id = ProviderThreadId::new(started.session_id.clone())
            .map_err(|_| protocol_error("session ID"))?;
        let capabilities = negotiated_capabilities(&started.initialize, &started.setup);
        let route = Arc::new(Mutex::new(CursorRouteState::new(
            started.session_id.clone(),
            input.modes(),
            input.model_id().cloned(),
            started.setup.config_options.clone(),
            workspace,
        )));
        let setup = Arc::new(Mutex::new(started.setup));
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let normalizer = Arc::new(CursorEventNormalizer::new(
            self.configuration.instance_id.clone(),
            input.thread_id().clone(),
            local_session_id.clone(),
        ));
        let expected_shutdown = Arc::new(AtomicBool::new(false));
        let terminal_error = Arc::new(Mutex::new(None));
        let inbound = connection.take_inbound()?;
        let router_task = spawn_cursor_router(CursorRouterResources {
            client: connection.client(),
            inbound,
            normalizer: Arc::clone(&normalizer),
            route: Arc::clone(&route),
            pending: Arc::clone(&pending),
            sink: Arc::clone(&sink),
            expected_shutdown: Arc::clone(&expected_shutdown),
            terminal_error: Arc::clone(&terminal_error),
        });
        let state = route.lock().map_err(|_| driver_state_error())?.clone();
        let cursor = resume_cursor(&started.session_id);
        sink.emit(normalizer.external_event(
            &state,
            if input.is_resume() {
                "session/resumed"
            } else {
                "session/started"
            },
            None,
            CanonicalEvent::SessionStarted(SessionStartedEvent {
                session_id: local_session_id.clone(),
                state: ProviderSessionState::Ready,
                provider_thread_id: Some(provider_thread_id.clone()),
                resume_cursor: Some(cursor.clone()),
                effective_modes: input.modes(),
                capability_overrides: capabilities.clone(),
            }),
        )?)
        .await?;
        sink.emit(normalizer.external_event(
            &state,
            "session/configured",
            None,
            CanonicalEvent::SessionConfigured(SessionConfiguredEvent {
                session_id: local_session_id.clone(),
                effective_modes: input.modes(),
                effective_model_id: input.model_id().cloned(),
                effective_model_options: input.model_options().to_vec(),
            }),
        )?)
        .await?;
        self.cached_models = Some(ProviderModelCatalog {
            instance_id: self.configuration.instance_id.clone(),
            models: started.models,
            source: ModelCatalogSource::Provider,
            discovered_at: now_utc()?,
            stale: false,
        });
        let started_at = now_utc()?;
        self.live = Some(CursorLiveSession {
            connection,
            router_task,
            prompt_task: None,
            route,
            setup,
            pending,
            normalizer,
            capabilities: capabilities.clone(),
            sink,
            expected_shutdown,
            terminal_error,
            session_id: local_session_id.clone(),
        });
        Ok(ProviderSessionSnapshot {
            session_id: local_session_id,
            state: ProviderSessionState::Ready,
            provider_thread_id: Some(provider_thread_id),
            continuation_group_id: group,
            resume_cursor: Some(cursor),
            effective_modes: input.modes(),
            capabilities,
            started_at,
        })
    }

    pub(super) fn live_mut(
        &mut self,
        session_id: &ProviderSessionId,
    ) -> ChatResult<&mut CursorLiveSession> {
        let live = self
            .live
            .as_mut()
            .ok_or_else(|| ChatError::driver_unavailable("Cursor session is not running"))?;
        if &live.session_id != session_id {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Cursor session identity does not match the active session",
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

fn canonical_verified_workspace(input: &VerifiedWorkspaceContext) -> ChatResult<PathBuf> {
    let path = PathBuf::from(&input.canonical_path);
    if !path.is_absolute() || !path.is_dir() {
        return Err(ChatError::validation(
            "workspace.canonicalPath",
            "Cursor workspace is unavailable",
        ));
    }
    std::fs::canonicalize(path).map_err(|_| {
        ChatError::validation("workspace.canonicalPath", "Cursor workspace is unavailable")
    })
}

pub(super) fn canonical_current_directory() -> ChatResult<PathBuf> {
    let current = std::env::current_dir().map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Cursor probe directory is unavailable",
            true,
        )
    })?;
    std::fs::canonicalize(current).map_err(|_| {
        ChatError::new(
            ChatErrorCode::ConfigurationInvalid,
            "Cursor probe directory is unavailable",
            true,
        )
    })
}

pub(super) fn potential_capabilities() -> ProviderCapabilities {
    ProviderCapabilities {
        entries: cursor_capability_kinds()
            .into_iter()
            .map(|capability| ProviderCapabilitySupport {
                capability,
                supported: true,
                explanation: None,
            })
            .collect(),
    }
}

pub(super) fn new_session_id(instance: &ProviderInstanceId) -> ChatResult<ProviderSessionId> {
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| driver_state_error())?
        .as_nanos();
    ProviderSessionId::new(format!(
        "cursor-{}-{}-{created}-{}",
        instance.as_str(),
        std::process::id(),
        NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
    ))
    .map_err(|_| protocol_error("local session ID"))
}

pub(super) fn now_utc() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .map_err(|_| protocol_error("timestamp"))
}
