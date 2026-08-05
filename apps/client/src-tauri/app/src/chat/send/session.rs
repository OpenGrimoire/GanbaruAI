//! Provider session startup and serialized durable event ingestion.

use super::checkpoints::ensure_post_turn_checkpoint;
use super::persistence::ThreadRuntimeData;
use super::support::{now_timestamp, operation_context, PROVIDER_START_TIMEOUT};
use super::validation::validate_modes;
use crate::chat::credentials::{materialize_provider_environment, PlatformCredentialStore};
use crate::chat::events::{CanonicalEvent, CanonicalRuntimeEvent, ChangedFileSummary};
use crate::chat::ingestion::{ChatEventIngestor, TauriChatChangeEmitter};
use crate::chat::models::{
    ChatError, ChatErrorCode, ChatResult, ChatThreadId, ContinuationGroupId,
    ProviderInstanceConfig, ProviderSessionSnapshot, ProviderSessionState, ResumeSessionRequest,
    StartSessionRequest, VerifiedWorkspaceContext,
};
use crate::chat::providers::{
    DriverFuture, ProviderDriver, ProviderDriverFactory, ProviderDriverRegistry, ProviderEventSink,
};
use crate::chat::repository::events::AppendCanonicalEventRequest;
use crate::chat::runtime::ThreadRuntimeOwner;
use crate::chat::send_commands::SendChatTurnCommand;
use crate::chat::workspace::AuthorizedWorkingFolder;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

const MAX_PROVIDER_CHANGED_FILES: usize = 512;
const MAX_PROVIDER_CHANGED_FILE_PATH_BYTES: usize = 4_096;

pub(super) struct EnsureSessionContext<'a> {
    pub(super) app: &'a tauri::AppHandle,
    pub(super) pool: &'a SqlitePool,
    pub(super) owner: &'a Arc<ThreadRuntimeOwner>,
    pub(super) new_driver: Option<Box<dyn ProviderDriver>>,
    pub(super) configuration: ProviderInstanceConfig,
    pub(super) workspace: &'a AuthorizedWorkingFolder,
    pub(super) thread_id: &'a ChatThreadId,
    pub(super) existing: Option<&'a ThreadRuntimeData>,
    pub(super) continuation_group_id: &'a ContinuationGroupId,
    pub(super) request: &'a SendChatTurnCommand,
}

pub(super) async fn ensure_session_with_executable_recovery(
    context: EnsureSessionContext<'_>,
) -> ChatResult<ProviderSessionSnapshot> {
    let EnsureSessionContext {
        app,
        pool,
        owner,
        new_driver,
        configuration,
        workspace,
        thread_id,
        existing,
        continuation_group_id,
        request,
    } = context;
    let internal_mcp = configuration.internal_mcp.clone();
    let initial = ensure_session(EnsureSessionContext {
        app,
        pool,
        owner,
        new_driver,
        configuration,
        workspace,
        thread_id,
        existing,
        continuation_group_id,
        request,
    })
    .await;
    let error = match initial {
        Ok(session) => return Ok(session),
        Err(error) if error.code == ChatErrorCode::ExecutableMissing => error,
        Err(error) => return Err(error),
    };
    let repaired_probe = crate::chat::settings_commands::chat_probe_provider(
        app.clone(),
        request.provider_instance_id.clone(),
    )
    .await;
    if !repaired_probe
        .as_ref()
        .is_ok_and(|probe| probe.state != crate::chat::models::ProbeState::ExecutableMissing)
    {
        return Err(error);
    }
    let repaired_provider =
        crate::chat::settings_commands::read_provider(app, &request.provider_instance_id)?;
    let mut repaired_configuration = materialize_provider_environment(
        &repaired_provider.configuration,
        &PlatformCredentialStore::default(),
    )?;
    repaired_configuration.internal_mcp = internal_mcp;
    let retry = ensure_session(EnsureSessionContext {
        app,
        pool,
        owner,
        new_driver: None,
        configuration: repaired_configuration,
        workspace,
        thread_id,
        existing,
        continuation_group_id,
        request,
    })
    .await;
    if retry
        .as_ref()
        .is_err_and(|retry_error| retry_error.code == ChatErrorCode::ExecutableMissing)
    {
        let _ = crate::chat::settings_commands::chat_probe_provider(
            app.clone(),
            request.provider_instance_id.clone(),
        )
        .await;
    }
    retry
}

pub(super) async fn ensure_session(
    context: EnsureSessionContext<'_>,
) -> ChatResult<ProviderSessionSnapshot> {
    let EnsureSessionContext {
        app,
        pool,
        owner,
        new_driver,
        configuration,
        workspace,
        thread_id,
        existing,
        continuation_group_id,
        request,
    } = context;
    let snapshot = owner.snapshot()?;
    if reuse_existing_session(snapshot.session_id.is_some(), snapshot.session_state)? {
        let session_id = snapshot.session_id.ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::InvalidStateTransition,
                "Chat provider session identity is unavailable",
                true,
            )
        })?;
        validate_modes(&snapshot.capabilities, request.modes)?;
        return Ok(ProviderSessionSnapshot {
            session_id,
            state: snapshot.session_state,
            provider_thread_id: existing.and_then(|value| value.provider_thread_id.clone()),
            continuation_group_id: continuation_group_id.clone(),
            resume_cursor: existing.and_then(|value| value.resume_cursor.clone()),
            effective_modes: request.modes,
            capabilities: snapshot.capabilities,
            started_at: now_timestamp()?,
        });
    }
    let driver = match new_driver {
        Some(driver) => driver,
        None => ProviderDriverRegistry.create_driver(configuration)?,
    };
    validate_modes(&driver.capabilities(), request.modes)?;
    let verified = VerifiedWorkspaceContext {
        working_folder_id: workspace.working_folder_id.clone(),
        canonical_path: workspace
            .canonical_path
            .to_str()
            .ok_or_else(|| ChatError::validation("workspace", "Workspace path is not supported"))?
            .to_string(),
        repository_kind: workspace.repository_kind,
        repository_identity: workspace.repository_identity.clone(),
    };
    crate::chat::diagnostics_commands::prune_expired_diagnostics(pool).await?;
    let sink: Arc<dyn ProviderEventSink> = Arc::new(DurableChatEventSink::new(
        app.clone(),
        pool.clone(),
        Arc::new(TauriChatChangeEmitter::new(app.clone())),
        workspace.clone(),
    ));
    if let Some(existing) = existing {
        if let (Some(provider_thread_id), Some(resume_cursor)) = (
            existing.provider_thread_id.clone(),
            existing.resume_cursor.clone(),
        ) {
            return owner
                .resume_session(
                    driver,
                    ResumeSessionRequest {
                        thread_id: thread_id.clone(),
                        workspace: verified,
                        provider_instance_id: request.provider_instance_id.clone(),
                        provider_thread_id,
                        continuation_group_id: continuation_group_id.clone(),
                        resume_cursor,
                        modes: request.modes,
                    },
                    sink,
                    operation_context("resume-session", PROVIDER_START_TIMEOUT),
                )
                .await;
        }
        if existing.provider_thread_id.is_some() {
            return Err(ChatError::new(
                ChatErrorCode::ResumeNotFound,
                "Native continuation data is missing. Start a fresh thread instead",
                true,
            ));
        }
    }
    owner
        .start_session(
            driver,
            StartSessionRequest {
                thread_id: thread_id.clone(),
                workspace: verified,
                provider_instance_id: request.provider_instance_id.clone(),
                modes: request.modes,
                model_id: request.model_id.clone(),
                model_options: request.model_options.clone(),
            },
            sink,
            operation_context("start-session", PROVIDER_START_TIMEOUT),
        )
        .await
}

pub(super) fn reuse_existing_session(
    has_session_id: bool,
    session_state: ProviderSessionState,
) -> ChatResult<bool> {
    if session_state == ProviderSessionState::Ready {
        if has_session_id {
            return Ok(true);
        }
        return Err(ChatError::new(
            ChatErrorCode::InvalidStateTransition,
            "Chat provider session identity is unavailable",
            true,
        ));
    }
    if matches!(
        session_state,
        ProviderSessionState::Stopped | ProviderSessionState::Failed
    ) {
        return Ok(false);
    }
    Err(ChatError::new(
        ChatErrorCode::InvalidStateTransition,
        "Chat provider session is not ready",
        true,
    ))
}

struct DurableChatEventSink {
    app: tauri::AppHandle,
    pool: SqlitePool,
    workspace: AuthorizedWorkingFolder,
    ingestor: Mutex<ChatEventIngestor>,
}

impl DurableChatEventSink {
    fn new(
        app: tauri::AppHandle,
        pool: SqlitePool,
        emitter: Arc<dyn crate::chat::ingestion::ChatChangeEmitter>,
        workspace: AuthorizedWorkingFolder,
    ) -> Self {
        Self {
            app,
            ingestor: Mutex::new(ChatEventIngestor::new(pool.clone(), emitter)),
            pool,
            workspace,
        }
    }
}

impl ProviderEventSink for DurableChatEventSink {
    fn emit<'a>(&'a self, mut event: CanonicalRuntimeEvent) -> DriverFuture<'a, ()> {
        Box::pin(async move {
            normalize_changed_file_paths(&mut event.event, &self.workspace.canonical_path);
            let settled_turn = matches!(
                &event.event,
                CanonicalEvent::TurnCompleted(_) | CanonicalEvent::TurnAborted(_)
            )
            .then(|| (event.thread_id.clone(), event.turn_id.clone()))
            .and_then(|(thread_id, turn_id)| turn_id.map(|turn_id| (thread_id, turn_id)));
            let stopped_thread = matches!(&event.event, CanonicalEvent::SessionExited(_))
                .then(|| event.thread_id.clone());
            let ingestion: ChatResult<()> = async {
                let diagnostic_expires_at =
                    crate::chat::diagnostics_commands::attach_opt_in_diagnostic(
                        &self.app, &mut event,
                    )?;
                self.ingestor
                    .lock()
                    .await
                    .ingest(AppendCanonicalEventRequest {
                        runtime: event,
                        ingested_at: now_timestamp()?,
                        diagnostic_expires_at,
                    })
                    .await
            }
            .await;
            if let Some((thread_id, turn_id)) = settled_turn {
                if ingestion.is_ok() {
                    if let Ok(settled_at) = now_timestamp() {
                        ensure_post_turn_checkpoint(
                            &self.pool,
                            &self.workspace,
                            &thread_id,
                            &turn_id,
                            &settled_at,
                        )
                        .await;
                    }
                }
                self.app
                    .state::<crate::chat::workspace_mutation::ChatWorkspaceMutationRegistry>()
                    .finish_provider_turn(&thread_id, &turn_id);
            }
            if let Some(thread_id) = stopped_thread {
                self.app
                    .state::<crate::chat::workspace_mutation::ChatWorkspaceMutationRegistry>()
                    .finish_thread(&thread_id);
            }
            ingestion
        })
    }

    fn flush(&self) -> DriverFuture<'_, ()> {
        Box::pin(async move { self.ingestor.lock().await.flush().await })
    }
}

pub(super) fn normalize_changed_file_paths(
    event: &mut CanonicalEvent,
    workspace: &std::path::Path,
) {
    let files = match event {
        CanonicalEvent::DiffUpdated(event) => &mut event.files,
        CanonicalEvent::TurnCompleted(event) => &mut event.changed_files,
        _ => return,
    };
    let mut normalized: Vec<ChangedFileSummary> = Vec::with_capacity(files.len());
    for mut file in std::mem::take(files)
        .into_iter()
        .take(MAX_PROVIDER_CHANGED_FILES)
    {
        let Some(relative_path) = workspace_relative_provider_path(workspace, &file.relative_path)
        else {
            continue;
        };
        file.relative_path = relative_path;
        file.previous_relative_path = file
            .previous_relative_path
            .as_deref()
            .and_then(|path| workspace_relative_provider_path(workspace, path));
        if let Some(index) = normalized
            .iter()
            .position(|candidate| candidate.relative_path == file.relative_path)
        {
            normalized[index] = file;
        } else {
            normalized.push(file);
        }
    }
    *files = normalized;
}

fn workspace_relative_provider_path(
    workspace: &std::path::Path,
    provider_path: &str,
) -> Option<String> {
    if provider_path.is_empty()
        || provider_path.len() > MAX_PROVIDER_CHANGED_FILE_PATH_BYTES
        || provider_path.contains('\0')
        || (cfg!(not(windows)) && provider_path.contains('\\'))
        || is_windows_absolute_provider_path(provider_path)
        || provider_path.chars().any(char::is_control)
    {
        return None;
    }
    let path = std::path::Path::new(provider_path);
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace).ok()?
    } else {
        path
    };
    let mut components = Vec::new();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            return None;
        };
        components.push(component.to_str()?);
    }
    (!components.is_empty()).then(|| components.join("/"))
}

fn is_windows_absolute_provider_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes
        .first()
        .is_some_and(|value| value.is_ascii_alphabetic())
        && bytes.get(1) == Some(&b':')
        && bytes
            .get(2)
            .is_some_and(|separator| matches!(*separator, b'/' | b'\\'))
}
