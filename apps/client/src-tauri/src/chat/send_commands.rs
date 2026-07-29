//! Durable send, steer, approval, and structured-input command routing.

use super::credentials::{materialize_provider_environment, PlatformCredentialStore};
use super::device_state::{full_access_is_trusted, read_active_device_scope};
use super::events::{CanonicalEvent, CanonicalRuntimeEvent};
use super::ingestion::{ChatEventIngestor, TauriChatChangeEmitter};
use super::models::*;
use super::providers::{
    DriverCancellation, DriverFuture, DriverOperationContext, ProviderDriver,
    ProviderDriverFactory, ProviderDriverRegistry, ProviderEventSink,
};
use super::repository::events::AppendCanonicalEventRequest;
use super::repository::receipts::{
    claim_command_receipt, complete_command_receipt, read_command_receipt, CommandReceiptClaim,
    CommandReceiptRead, CommandReceiptState,
};
use super::repository::{attachments, reads, workspaces};
use super::runtime::{ChatRuntimeRegistry, ThreadRuntimeOwner};
use super::workspace::{
    authorize_workspace, AuthorizedWorkingFolder, WorkingFolderAuthorizationOperation,
};
use crate::projects::working_folders::read_active_working_folder_scope;
use crate::{db_path, vault};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::Mutex;

const PROVIDER_START_TIMEOUT: Duration = Duration::from_secs(45);
const TURN_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendChatTurnCommand {
    pub command: ChatCommandContext,
    pub working_folder_id: ProjectWorkingFolderId,
    pub thread_id: Option<ChatThreadId>,
    pub new_thread_id: Option<ChatThreadId>,
    pub turn_id: ChatTurnId,
    pub message_id: ChatMessageId,
    pub provider_instance_id: ProviderInstanceId,
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    pub model_options: Vec<ModelOptionSelection>,
    pub modes: TurnModeSnapshot,
    pub prompt: String,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub mentions: Vec<WorkspaceMentionReference>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendChatTurnResult {
    pub thread: ChatThreadShellRead,
    pub dispatch: Option<TurnDispatchReceipt>,
    pub launch_error: Option<ChatError>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteerChatTurnCommand {
    pub command: ChatCommandContext,
    pub thread_id: ChatThreadId,
    pub message_id: ChatMessageId,
    pub prompt: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveChatApprovalCommand {
    pub command: ChatCommandContext,
    pub thread_id: ChatThreadId,
    pub request_id: ChatRequestId,
    pub provider_request_id: ProviderRequestId,
    pub decision: ApprovalDecision,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveChatUserInputCommand {
    pub command: ChatCommandContext,
    pub thread_id: ChatThreadId,
    pub request_id: ChatRequestId,
    pub provider_request_id: ProviderRequestId,
    pub answers: Vec<UserInputAnswer>,
}

#[tauri::command]
pub async fn chat_send_turn(
    app: tauri::AppHandle,
    db_url: String,
    request: SendChatTurnCommand,
) -> ChatResult<SendChatTurnResult> {
    validate_prompt(
        &request.prompt,
        !request.attachment_ids.is_empty() || !request.mentions.is_empty(),
    )?;
    validate_explicit_model(request.provider_managed_model, request.model_id.as_ref())?;
    validate_mentions(&request.mentions)?;
    let thread_id = match (&request.thread_id, &request.new_thread_id) {
        (Some(thread_id), _) => thread_id.clone(),
        (None, Some(thread_id)) => thread_id.clone(),
        (None, None) => {
            return Err(ChatError::validation(
                "newThreadId",
                "A new Chat thread ID is required",
            ))
        }
    };
    let pool = chat_pool(app.clone(), db_url).await?;
    if let Some(receipt) = read_command_receipt(&pool, &request.command.client_command_id).await? {
        return replay_send_receipt(&pool, &thread_id, &receipt).await;
    }
    let logical_workspace = workspaces::read_workspace(&pool, &request.working_folder_id).await?;
    require_project_accepts_ai_work(&pool, &logical_workspace.project_id).await?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    let working_folder_scope =
        read_active_working_folder_scope(&app).map_err(device_state_error)?;
    let authorized = authorize_workspace(
        &logical_workspace,
        &working_folder_scope,
        WorkingFolderAuthorizationOperation::ProviderStart,
    )?;
    if matches!(
        request.modes.safety_mode,
        SafetyMode::FullAccess | SafetyMode::Custom
    ) && !full_access_is_trusted(
        &scope,
        &request.provider_instance_id,
        &request.working_folder_id,
    ) {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Broad permissions are not trusted for this provider and workspace",
            true,
        ));
    }
    validate_send_mentions(&authorized, &request.mentions)?;
    let provider = super::settings_commands::read_provider(&app, &request.provider_instance_id)?;
    validate_provider_selection(&provider, &request)?;
    let configuration = materialize_provider_environment(
        &provider.configuration,
        &PlatformCredentialStore::default(),
    )?;
    let existing = match request.thread_id.as_ref() {
        Some(thread_id) => Some(read_thread_runtime_data(&pool, thread_id).await?),
        None => None,
    };
    if let Some(existing) = &existing {
        if existing.working_folder_id != request.working_folder_id
            || existing.provider_instance_id != request.provider_instance_id
        {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Changing workspace or provider requires a new Chat thread",
                true,
            ));
        }
    }
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(thread_id.clone())?;
    let mut new_driver = None;
    let continuation_group_id = if let Some(existing) = &existing {
        existing.continuation_group_id.clone()
    } else {
        let mut driver = ProviderDriverRegistry.create_driver(configuration.clone())?;
        validate_modes(&driver.capabilities(), request.modes)?;
        let continuation = driver
            .derive_continuation_group(
                ContinuationGroupRequest {
                    provider_instance_id: request.provider_instance_id.clone(),
                    normalized_provider_home: configuration.provider_home.clone(),
                    account_identity: provider
                        .last_probe
                        .as_ref()
                        .and_then(|probe| probe.account_label.clone()),
                    server_identity: None,
                    provider_fields: BTreeMap::new(),
                },
                &operation_context("derive-continuation", PROVIDER_START_TIMEOUT),
            )
            .await?;
        new_driver = Some(driver);
        continuation
    };
    let attachment_references = read_attachment_references(
        &app,
        &pool,
        &request.working_folder_id,
        &request.attachment_ids,
    )
    .await?;
    let persistence_now = now_timestamp()?;
    persist_user_turn(PersistUserTurnContext {
        pool: &pool,
        workspace: &logical_workspace,
        thread_id: &thread_id,
        existing: existing.as_ref(),
        continuation_group_id: &continuation_group_id,
        provider_family_id: &provider.configuration.family_id,
        request: &request,
        attachments: &attachment_references,
        now: &persistence_now,
    })
    .await?;
    ensure_pre_turn_checkpoint(
        &pool,
        &authorized,
        &thread_id,
        &request.turn_id,
        &persistence_now,
    )
    .await;

    let operation = async {
        let session = ensure_session(EnsureSessionContext {
            app: &app,
            pool: &pool,
            owner: &owner,
            new_driver,
            configuration,
            workspace: &authorized,
            thread_id: &thread_id,
            existing: existing.as_ref(),
            continuation_group_id: &continuation_group_id,
            request: &request,
        })
        .await?;
        owner
            .send_turn(
                SendTurnRequest {
                    command: request.command.clone(),
                    session_id: session.session_id,
                    turn_id: request.turn_id.clone(),
                    prompt: request.prompt.clone(),
                    attachments: attachment_references,
                    mentions: request.mentions.clone(),
                    model_id: request.model_id.clone(),
                    model_options: request.model_options.clone(),
                    modes: request.modes,
                    developer_instructions: None,
                },
                operation_context("send-turn", TURN_OPERATION_TIMEOUT),
            )
            .await
    }
    .await;
    let (dispatch, launch_error) = match operation {
        Ok(dispatch) => (Some(dispatch), None),
        Err(error) => {
            mark_turn_dispatch_failed(
                &pool,
                &thread_id,
                &request.turn_id,
                &error,
                &now_timestamp()?,
            )
            .await?;
            (None, Some(error))
        }
    };
    let result = SendChatTurnResult {
        thread: reads::read_thread_shell(&pool, &thread_id).await?,
        dispatch,
        launch_error,
    };
    let receipt_result = versioned_value(&result)?;
    complete_command_receipt(
        &pool,
        &request.command.client_command_id,
        CommandReceiptState::Completed,
        Some(&receipt_result),
        None,
        &now_timestamp()?,
    )
    .await?;
    Ok(result)
}

async fn require_project_accepts_ai_work(pool: &SqlitePool, project_id: &str) -> ChatResult<()> {
    let status: Option<String> = sqlx::query_scalar("SELECT status FROM projects WHERE id = ?")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            ChatError::new(
                ChatErrorCode::Persistence,
                "Project state could not be read",
                true,
            )
        })?;
    match status.as_deref() {
        Some("archived") => Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Restore the archived project before starting new AI work",
            true,
        )),
        Some(_) => Ok(()),
        None => Err(ChatError::new(
            ChatErrorCode::NotFound,
            "The Chat project was not found",
            true,
        )),
    }
}

#[tauri::command]
pub async fn chat_steer_turn(
    app: tauri::AppHandle,
    db_url: String,
    request: SteerChatTurnCommand,
) -> ChatResult<DriverOperationReceipt> {
    validate_prompt(&request.prompt, false)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(request.thread_id.clone())?;
    let snapshot = owner.snapshot()?;
    if !snapshot.capabilities.supports(ProviderCapability::Steering) {
        return Err(ChatError::new(
            ChatErrorCode::CapabilityUnsupported,
            "This provider does not support in-turn steering",
            true,
        ));
    }
    let session_id = snapshot.session_id.ok_or_else(runtime_not_running)?;
    let turn_id = snapshot.active_turn_id.ok_or_else(runtime_not_running)?;
    let command_id = request.command.client_command_id.clone();
    match claim_command_receipt(
        &pool,
        &command_id,
        &request.thread_id,
        "steer_turn",
        request.command.expected_thread_revision,
        &now_timestamp()?,
    )
    .await?
    {
        CommandReceiptClaim::Replay(receipt) => return replay_driver_receipt(receipt),
        CommandReceiptClaim::Claimed(_) => {}
    }
    if let Err(error) = persist_steer_message(&pool, &request, &turn_id, &now_timestamp()?).await {
        return complete_driver_operation(&pool, &command_id, Err(error)).await;
    }
    let result = owner
        .steer_turn(
            SteerTurnRequest {
                command: request.command,
                session_id,
                turn_id,
                prompt: request.prompt,
            },
            operation_context("steer-turn", TURN_OPERATION_TIMEOUT),
        )
        .await;
    complete_driver_operation(&pool, &command_id, result).await
}

#[tauri::command]
pub async fn chat_resolve_approval(
    app: tauri::AppHandle,
    db_url: String,
    request: ResolveChatApprovalCommand,
) -> ChatResult<DriverOperationReceipt> {
    let pool = chat_pool(app.clone(), db_url).await?;
    validate_pending_request(
        &pool,
        &request.thread_id,
        &request.request_id,
        &request.provider_request_id,
        "approval",
    )
    .await?;
    validate_approval_decision(&pool, &request.request_id, &request.decision).await?;
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(request.thread_id.clone())?;
    let session_id = owner
        .snapshot()?
        .session_id
        .ok_or_else(runtime_not_running)?;
    match claim_command_receipt(
        &pool,
        &request.command.client_command_id,
        &request.thread_id,
        "resolve_approval",
        request.command.expected_thread_revision,
        &now_timestamp()?,
    )
    .await?
    {
        CommandReceiptClaim::Replay(receipt) => return replay_driver_receipt(receipt),
        CommandReceiptClaim::Claimed(_) => {}
    }
    let command_id = request.command.client_command_id.clone();
    let result = owner
        .resolve_approval(
            ResolveApprovalRequest {
                command: request.command,
                session_id,
                request_id: request.request_id,
                provider_request_id: request.provider_request_id,
                decision: request.decision,
            },
            operation_context("resolve-approval", TURN_OPERATION_TIMEOUT),
        )
        .await;
    complete_driver_operation(&pool, &command_id, result).await
}

#[tauri::command]
pub async fn chat_resolve_user_input(
    app: tauri::AppHandle,
    db_url: String,
    request: ResolveChatUserInputCommand,
) -> ChatResult<DriverOperationReceipt> {
    validate_answers(&request.answers)?;
    let pool = chat_pool(app.clone(), db_url).await?;
    validate_pending_request(
        &pool,
        &request.thread_id,
        &request.request_id,
        &request.provider_request_id,
        "user_input",
    )
    .await?;
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(request.thread_id.clone())?;
    let session_id = owner
        .snapshot()?
        .session_id
        .ok_or_else(runtime_not_running)?;
    match claim_command_receipt(
        &pool,
        &request.command.client_command_id,
        &request.thread_id,
        "resolve_user_input",
        request.command.expected_thread_revision,
        &now_timestamp()?,
    )
    .await?
    {
        CommandReceiptClaim::Replay(receipt) => return replay_driver_receipt(receipt),
        CommandReceiptClaim::Claimed(_) => {}
    }
    let now = now_timestamp()?;
    let answer_persistence = sqlx::query(
        "INSERT INTO chat_user_input_drafts (request_id, answers_data, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(request_id) DO UPDATE SET answers_data = excluded.answers_data,
             updated_at = excluded.updated_at",
    )
    .bind(request.request_id.as_str())
    .bind(serde_json::to_string(&request.answers).map_err(json_error)?)
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error);
    let command_id = request.command.client_command_id.clone();
    if let Err(error) = answer_persistence {
        return complete_driver_operation(&pool, &command_id, Err(error)).await;
    }
    let request_id = request.request_id.clone();
    let result = owner
        .resolve_user_input(
            ResolveUserInputRequest {
                command: request.command,
                session_id,
                request_id: request.request_id.clone(),
                provider_request_id: request.provider_request_id,
                answers: request.answers,
            },
            operation_context("resolve-user-input", TURN_OPERATION_TIMEOUT),
        )
        .await;
    let result = complete_driver_operation(&pool, &command_id, result).await?;
    sqlx::query("DELETE FROM chat_user_input_drafts WHERE request_id = ?")
        .bind(request_id.as_str())
        .execute(&pool)
        .await
        .map_err(persistence_error)?;
    Ok(result)
}

struct EnsureSessionContext<'a> {
    app: &'a tauri::AppHandle,
    pool: &'a SqlitePool,
    owner: &'a Arc<ThreadRuntimeOwner>,
    new_driver: Option<Box<dyn ProviderDriver>>,
    configuration: ProviderInstanceConfig,
    workspace: &'a super::workspace::AuthorizedWorkingFolder,
    thread_id: &'a ChatThreadId,
    existing: Option<&'a ThreadRuntimeData>,
    continuation_group_id: &'a ContinuationGroupId,
    request: &'a SendChatTurnCommand,
}

async fn ensure_session(context: EnsureSessionContext<'_>) -> ChatResult<ProviderSessionSnapshot> {
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
    super::diagnostics_commands::prune_expired_diagnostics(pool).await?;
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

fn reuse_existing_session(
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
    workspace: super::workspace::AuthorizedWorkingFolder,
    ingestor: Mutex<ChatEventIngestor>,
}

impl DurableChatEventSink {
    fn new(
        app: tauri::AppHandle,
        pool: SqlitePool,
        emitter: Arc<dyn super::ingestion::ChatChangeEmitter>,
        workspace: super::workspace::AuthorizedWorkingFolder,
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
            let settled_turn = matches!(
                &event.event,
                CanonicalEvent::TurnCompleted(_) | CanonicalEvent::TurnAborted(_)
            )
            .then(|| (event.thread_id.clone(), event.turn_id.clone()))
            .and_then(|(thread_id, turn_id)| turn_id.map(|turn_id| (thread_id, turn_id)));
            let diagnostic_expires_at =
                super::diagnostics_commands::attach_opt_in_diagnostic(&self.app, &mut event)?;
            self.ingestor
                .lock()
                .await
                .ingest(AppendCanonicalEventRequest {
                    runtime: event,
                    ingested_at: now_timestamp()?,
                    diagnostic_expires_at,
                })
                .await?;
            if let Some((thread_id, turn_id)) = settled_turn {
                ensure_post_turn_checkpoint(
                    &self.pool,
                    &self.workspace,
                    &thread_id,
                    &turn_id,
                    &now_timestamp()?,
                )
                .await;
            }
            Ok(())
        })
    }

    fn flush(&self) -> DriverFuture<'_, ()> {
        Box::pin(async move { self.ingestor.lock().await.flush().await })
    }
}

async fn ensure_pre_turn_checkpoint(
    pool: &SqlitePool,
    workspace: &super::workspace::AuthorizedWorkingFolder,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
    now: &UtcTimestamp,
) {
    if workspace.repository_kind != RepositoryKind::Git {
        return;
    }
    let ordinal = match turn_ordinal(pool, thread_id, turn_id).await {
        Ok(value) => value,
        Err(error) => {
            record_checkpoint_failure(pool, thread_id, Some(turn_id), "pre_turn", &error, now)
                .await;
            return;
        }
    };
    let existing: Result<Option<String>, _> = sqlx::query_scalar(
        "SELECT id FROM chat_checkpoints
         WHERE thread_id = ? AND turn_count = ? AND status = 'available'
           AND invalidated_at IS NULL",
    )
    .bind(thread_id.as_str())
    .bind(i64::try_from(ordinal).unwrap_or(i64::MAX))
    .fetch_optional(pool)
    .await;
    if let Ok(Some(checkpoint_id)) = existing {
        let _ = sqlx::query(
            "UPDATE chat_turns SET pre_checkpoint_id = ? WHERE id = ? AND thread_id = ?",
        )
        .bind(&checkpoint_id)
        .bind(turn_id.as_str())
        .bind(thread_id.as_str())
        .execute(pool)
        .await;
        update_user_checkpoint_context(pool, turn_id, &checkpoint_id).await;
        return;
    }
    let kind = if ordinal == 0 {
        super::checkpoints::CheckpointKind::Initial
    } else {
        super::checkpoints::CheckpointKind::PreTurn
    };
    match super::checkpoints::capture_and_store(
        pool,
        workspace,
        thread_id,
        Some(turn_id),
        ordinal,
        kind,
        now,
    )
    .await
    {
        Ok(checkpoint) => {
            update_user_checkpoint_context(pool, turn_id, checkpoint.id.as_str()).await
        }
        Err(error) => {
            let failure_kind = if ordinal == 0 { "initial" } else { "pre_turn" };
            record_checkpoint_failure(pool, thread_id, Some(turn_id), failure_kind, &error, now)
                .await;
        }
    }
}

async fn ensure_post_turn_checkpoint(
    pool: &SqlitePool,
    workspace: &super::workspace::AuthorizedWorkingFolder,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
    now: &UtcTimestamp,
) {
    if workspace.repository_kind != RepositoryKind::Git {
        return;
    }
    let ordinal = match turn_ordinal(pool, thread_id, turn_id).await {
        Ok(value) => value,
        Err(error) => {
            record_checkpoint_failure(pool, thread_id, Some(turn_id), "post_turn", &error, now)
                .await;
            return;
        }
    };
    if let Err(error) = super::checkpoints::capture_and_store(
        pool,
        workspace,
        thread_id,
        Some(turn_id),
        ordinal.saturating_add(1),
        super::checkpoints::CheckpointKind::PostTurn,
        now,
    )
    .await
    {
        if error.code != ChatErrorCode::Conflict {
            record_checkpoint_failure(pool, thread_id, Some(turn_id), "post_turn", &error, now)
                .await;
        }
    }
}

async fn turn_ordinal(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
) -> ChatResult<u64> {
    let ordinal: i64 =
        sqlx::query_scalar("SELECT ordinal FROM chat_turns WHERE id = ? AND thread_id = ?")
            .bind(turn_id.as_str())
            .bind(thread_id.as_str())
            .fetch_optional(pool)
            .await
            .map_err(persistence_error)?
            .ok_or_else(|| {
                ChatError::new(ChatErrorCode::NotFound, "Chat turn was not found", true)
            })?;
    u64::try_from(ordinal).map_err(|_| corrupt_data())
}

async fn update_user_checkpoint_context(
    pool: &SqlitePool,
    turn_id: &ChatTurnId,
    checkpoint_id: &str,
) {
    let _ = sqlx::query(
        "UPDATE chat_messages
         SET content_metadata_data = json_set(content_metadata_data, '$.preCheckpointId', ?)
         WHERE turn_id = ? AND role = 'user'",
    )
    .bind(checkpoint_id)
    .bind(turn_id.as_str())
    .execute(pool)
    .await;
}

async fn record_checkpoint_failure(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: Option<&ChatTurnId>,
    kind: &str,
    error: &ChatError,
    now: &UtcTimestamp,
) {
    let id = format!(
        "checkpoint-failure:{}:{}:{}",
        thread_id.as_str(),
        turn_id.map(ChatTurnId::as_str).unwrap_or("initial"),
        kind
    );
    let _ = sqlx::query(
        "INSERT OR REPLACE INTO chat_checkpoint_failures
            (id, thread_id, turn_id, checkpoint_kind, error_code, detail, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(thread_id.as_str())
    .bind(turn_id.map(ChatTurnId::as_str))
    .bind(kind)
    .bind(format!("{:?}", error.code).to_lowercase())
    .bind(&error.message)
    .bind(now.as_str())
    .execute(pool)
    .await;
}

#[derive(Clone, Debug)]
struct ThreadRuntimeData {
    working_folder_id: ProjectWorkingFolderId,
    provider_instance_id: ProviderInstanceId,
    continuation_group_id: ContinuationGroupId,
    provider_thread_id: Option<ProviderThreadId>,
    resume_cursor: Option<VersionedJson>,
    revision: u64,
}

async fn read_thread_runtime_data(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<ThreadRuntimeData> {
    let row = sqlx::query(
        "SELECT working_folder_id, provider_instance_id, continuation_group_id,
                provider_thread_id, resume_cursor_schema_version, resume_cursor_data, revision
         FROM chat_threads WHERE id = ? AND archived_at IS NULL AND state != 'closed'",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true))?;
    let resume_cursor = match (
        row.try_get::<Option<i64>, _>("resume_cursor_schema_version")
            .map_err(persistence_error)?,
        row.try_get::<Option<String>, _>("resume_cursor_data")
            .map_err(persistence_error)?,
    ) {
        (None, None) => None,
        (Some(version), Some(data)) => Some(VersionedJson {
            schema_version: u32::try_from(version).map_err(|_| corrupt_data())?,
            value: serde_json::from_str(&data).map_err(json_error)?,
        }),
        _ => return Err(corrupt_data()),
    };
    Ok(ThreadRuntimeData {
        working_folder_id: ProjectWorkingFolderId::new(
            row.try_get::<String, _>("working_folder_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        provider_instance_id: ProviderInstanceId::new(
            row.try_get::<String, _>("provider_instance_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        continuation_group_id: ContinuationGroupId::new(
            row.try_get::<String, _>("continuation_group_id")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
        provider_thread_id: row
            .try_get::<Option<String>, _>("provider_thread_id")
            .map_err(persistence_error)?
            .map(ProviderThreadId::new)
            .transpose()
            .map_err(|_| corrupt_data())?,
        resume_cursor,
        revision: u64::try_from(
            row.try_get::<i64, _>("revision")
                .map_err(persistence_error)?,
        )
        .map_err(|_| corrupt_data())?,
    })
}

async fn read_attachment_references(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
    attachment_ids: &[ChatAttachmentId],
) -> ChatResult<Vec<PromptAttachmentReference>> {
    if attachment_ids.len() > 8 {
        return Err(ChatError::validation(
            "attachments",
            "Too many Chat attachments",
        ));
    }
    let mut result = Vec::with_capacity(attachment_ids.len());
    let mut total_bytes = 0_u64;
    let vault_root = vault::active_vault_path(app).map_err(|_| {
        ChatError::new(
            ChatErrorCode::Persistence,
            "Active Ganbaru folder is unavailable",
            true,
        )
    })?;
    for id in attachment_ids {
        let attachment = attachments::read_attachment(pool, id)
            .await?
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "Chat attachment was not found",
                    true,
                )
            })?;
        if &attachment.working_folder_id != working_folder_id {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Chat attachment belongs to another workspace",
                false,
            ));
        }
        total_bytes = total_bytes
            .checked_add(attachment.byte_size)
            .ok_or_else(|| {
                ChatError::validation("attachments", "Chat attachment total is too large")
            })?;
        if total_bytes > 50 * 1024 * 1024 {
            return Err(ChatError::validation(
                "attachments",
                "Chat attachments must total 50 MiB or less",
            ));
        }
        let (local_path, bytes) =
            attachments::read_managed_attachment_bytes(&vault_root, &attachment)?;
        let (local_path, text_content) = match attachment.kind {
            attachments::ChatAttachmentKind::Image => (
                Some(
                    local_path
                        .to_str()
                        .ok_or_else(|| {
                            ChatError::validation(
                                "attachments",
                                "Managed attachment path is unsupported",
                            )
                        })?
                        .to_string(),
                ),
                None,
            ),
            attachments::ChatAttachmentKind::TextSnippet => (
                None,
                Some(String::from_utf8(bytes).map_err(|_| {
                    ChatError::validation("attachments", "Text context is invalid")
                })?),
            ),
        };
        result.push(PromptAttachmentReference {
            attachment_id: id.clone(),
            kind: match attachment.kind {
                attachments::ChatAttachmentKind::Image => "image",
                attachments::ChatAttachmentKind::TextSnippet => "text_snippet",
            }
            .to_string(),
            display_name: attachment.original_display_name,
            managed_relative_path: attachment.managed_relative_path,
            mime_type: Some(attachment.mime_type),
            byte_size: attachment.byte_size,
            local_path,
            text_content,
        });
    }
    Ok(result)
}

struct PersistUserTurnContext<'a> {
    pool: &'a SqlitePool,
    workspace: &'a super::workspace::ProjectWorkingFolder,
    thread_id: &'a ChatThreadId,
    existing: Option<&'a ThreadRuntimeData>,
    continuation_group_id: &'a ContinuationGroupId,
    provider_family_id: &'a ProviderFamilyId,
    request: &'a SendChatTurnCommand,
    attachments: &'a [PromptAttachmentReference],
    now: &'a UtcTimestamp,
}

async fn persist_user_turn(context: PersistUserTurnContext<'_>) -> ChatResult<()> {
    let PersistUserTurnContext {
        pool,
        workspace,
        thread_id,
        existing,
        continuation_group_id,
        provider_family_id,
        request,
        attachments,
        now,
    } = context;
    let model_selection = serde_json::to_string(&json!({
        "modelId": request.model_id,
        "providerManaged": request.provider_managed_model,
        "options": request.model_options,
    }))
    .map_err(json_error)?;
    let user_context = serde_json::to_string(&json!({
        "attachments": attachments.iter().map(|attachment| json!({
            "id": attachment.attachment_id,
            "displayName": attachment.display_name,
            "kind": attachment.kind,
            "mimeType": attachment.mime_type,
            "byteSize": attachment.byte_size,
            "status": "managed",
        })).collect::<Vec<_>>(),
        "mentions": request.mentions,
        "terminalContext": attachments.iter().filter(|attachment| attachment.kind == "text_snippet").map(|attachment| json!({
            "attachmentId": attachment.attachment_id,
            "displayName": attachment.display_name,
            "byteSize": attachment.byte_size,
        })).collect::<Vec<_>>(),
        "preCheckpointId": null,
    }))
    .map_err(json_error)?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    if let Some(existing) = existing {
        if request
            .command
            .expected_thread_revision
            .is_some_and(|revision| revision != existing.revision)
        {
            return Err(ChatError::new(
                ChatErrorCode::StaleRevision,
                "Chat thread changed before send",
                true,
            ));
        }
        let duplicate: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM chat_command_receipts WHERE client_command_id = ?)",
        )
        .bind(request.command.client_command_id.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(persistence_error)?;
        if duplicate {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "This Chat send command was already submitted",
                true,
            ));
        }
    } else {
        let title = prompt_title(&request.prompt);
        sqlx::query(
            "INSERT INTO chat_threads
                (id, working_folder_id, project_id, title, provider_family_id,
                 provider_instance_id, continuation_group_id, model_selection_data,
                 safety_mode, interaction_mode, state, latest_turn_state,
                 last_activity_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', 'pending', ?, ?, ?)",
        )
        .bind(thread_id.as_str())
        .bind(workspace.id.as_str())
        .bind(&workspace.project_id)
        .bind(title)
        .bind(provider_family_id.as_str())
        .bind(request.provider_instance_id.as_str())
        .bind(continuation_group_id.as_str())
        .bind(&model_selection)
        .bind(wire_safety(request.modes.safety_mode))
        .bind(wire_interaction(request.modes.interaction_mode))
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    let ordinal: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(ordinal), -1) + 1 FROM chat_turns WHERE thread_id = ?",
    )
    .bind(thread_id.as_str())
    .fetch_one(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    let sequence: i64 =
        sqlx::query_scalar("SELECT last_event_sequence FROM chat_threads WHERE id = ?")
            .bind(thread_id.as_str())
            .fetch_one(&mut *transaction)
            .await
            .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_turns
            (id, thread_id, ordinal, user_message_id, state, model_selection_data,
             safety_mode, interaction_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'pending', ?, ?, ?, ?, ?)",
    )
    .bind(request.turn_id.as_str())
    .bind(thread_id.as_str())
    .bind(ordinal)
    .bind(request.message_id.as_str())
    .bind(&model_selection)
    .bind(wire_safety(request.modes.safety_mode))
    .bind(wire_interaction(request.modes.interaction_mode))
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_messages
            (id, thread_id, turn_id, sequence_anchor, role, normalized_markdown,
             streaming_state, content_metadata_data, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'user', ?, 'complete', ?, ?, ?)",
    )
    .bind(request.message_id.as_str())
    .bind(thread_id.as_str())
    .bind(request.turn_id.as_str())
    .bind(sequence)
    .bind(&request.prompt)
    .bind(user_context)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    for (index, attachment) in attachments.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_attachment_references
                (id, attachment_id, message_id, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(format!("message:{}:{index}", request.message_id.as_str()))
        .bind(attachment.attachment_id.as_str())
        .bind(request.message_id.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    sqlx::query(
        "INSERT INTO chat_command_receipts
            (client_command_id, thread_id, command_kind, submitted_revision,
             state, created_at, updated_at)
         VALUES (?, ?, 'send_turn', ?, 'accepted', ?, ?)",
    )
    .bind(request.command.client_command_id.as_str())
    .bind(thread_id.as_str())
    .bind(
        request
            .command
            .expected_thread_revision
            .map(i64_value)
            .transpose()?,
    )
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_threads SET model_selection_data = ?, safety_mode = ?,
                interaction_mode = ?, state = 'active', latest_turn_state = 'pending',
                latest_preview = ?, message_count = message_count + 1,
                revision = revision + 1, last_activity_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(model_selection)
    .bind(wire_safety(request.modes.safety_mode))
    .bind(wire_interaction(request.modes.interaction_mode))
    .bind(prompt_preview(&request.prompt))
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)
}

async fn persist_steer_message(
    pool: &SqlitePool,
    request: &SteerChatTurnCommand,
    turn_id: &ChatTurnId,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let sequence: i64 =
        sqlx::query_scalar("SELECT last_event_sequence FROM chat_threads WHERE id = ?")
            .bind(request.thread_id.as_str())
            .fetch_one(&mut *transaction)
            .await
            .map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_messages
            (id, thread_id, turn_id, sequence_anchor, role, normalized_markdown,
             streaming_state, content_metadata_data, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'user', ?, 'complete', ?, ?, ?)",
    )
    .bind(request.message_id.as_str())
    .bind(request.thread_id.as_str())
    .bind(turn_id.as_str())
    .bind(sequence)
    .bind(&request.prompt)
    .bind(json!({ "steer": true }).to_string())
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_threads SET latest_preview = ?, message_count = message_count + 1,
                revision = revision + 1, last_activity_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(prompt_preview(&request.prompt))
    .bind(now.as_str())
    .bind(now.as_str())
    .bind(request.thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)
}

async fn mark_turn_dispatch_failed(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    turn_id: &ChatTurnId,
    error: &ChatError,
    now: &UtcTimestamp,
) -> ChatResult<()> {
    let error_data = serde_json::to_string(error).map_err(json_error)?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_turns SET state = 'failed', completed_at = ?, stop_reason = ?,
                error_schema_version = 1, error_data = ?, updated_at = ?
         WHERE id = ? AND thread_id = ?",
    )
    .bind(now.as_str())
    .bind(&error.message)
    .bind(error_data)
    .bind(now.as_str())
    .bind(turn_id.as_str())
    .bind(thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query(
        "UPDATE chat_threads SET state = 'error', latest_turn_state = 'failed',
                revision = revision + 1, updated_at = ? WHERE id = ?",
    )
    .bind(now.as_str())
    .bind(thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    transaction.commit().await.map_err(persistence_error)
}

async fn replay_send_receipt(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    receipt: &CommandReceiptRead,
) -> ChatResult<SendChatTurnResult> {
    if receipt.thread_id != *thread_id || receipt.command_kind != "send_turn" {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Chat command ID was already used for a different operation",
            false,
        ));
    }
    match receipt.state {
        CommandReceiptState::Accepted => Err(ChatError::new(
            ChatErrorCode::Busy,
            "This Chat send command is still being processed",
            true,
        )),
        CommandReceiptState::Failed => Err(receipt_error(receipt)?),
        CommandReceiptState::Completed => {
            let mut result: SendChatTurnResult = serde_json::from_value(
                receipt
                    .result
                    .as_ref()
                    .ok_or_else(corrupt_data)?
                    .value
                    .clone(),
            )
            .map_err(json_error)?;
            result.thread = reads::read_thread_shell(pool, thread_id).await?;
            Ok(result)
        }
    }
}

fn replay_driver_receipt(receipt: CommandReceiptRead) -> ChatResult<DriverOperationReceipt> {
    match receipt.state {
        CommandReceiptState::Accepted => Err(ChatError::new(
            ChatErrorCode::Busy,
            "This Chat command is still being processed",
            true,
        )),
        CommandReceiptState::Failed => Err(receipt_error(&receipt)?),
        CommandReceiptState::Completed => {
            serde_json::from_value(receipt.result.ok_or_else(corrupt_data)?.value)
                .map_err(json_error)
        }
    }
}

async fn complete_driver_operation(
    pool: &SqlitePool,
    command_id: &ChatCommandId,
    result: ChatResult<DriverOperationReceipt>,
) -> ChatResult<DriverOperationReceipt> {
    match result {
        Ok(receipt) => {
            let value = versioned_value(&receipt)?;
            complete_command_receipt(
                pool,
                command_id,
                CommandReceiptState::Completed,
                Some(&value),
                None,
                &now_timestamp()?,
            )
            .await?;
            Ok(receipt)
        }
        Err(error) => {
            let value = versioned_value(&error)?;
            complete_command_receipt(
                pool,
                command_id,
                CommandReceiptState::Failed,
                None,
                Some(&value),
                &now_timestamp()?,
            )
            .await?;
            Err(error)
        }
    }
}

fn receipt_error(receipt: &CommandReceiptRead) -> ChatResult<ChatError> {
    serde_json::from_value(
        receipt
            .error
            .as_ref()
            .ok_or_else(corrupt_data)?
            .value
            .clone(),
    )
    .map_err(json_error)
}

fn versioned_value<T: Serialize>(value: &T) -> ChatResult<VersionedJson> {
    Ok(VersionedJson {
        schema_version: 1,
        value: serde_json::to_value(value).map_err(json_error)?,
    })
}

async fn validate_pending_request(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    request_id: &ChatRequestId,
    provider_request_id: &ProviderRequestId,
    request_kind: &str,
) -> ChatResult<()> {
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM chat_pending_requests
            WHERE id = ? AND thread_id = ? AND provider_request_id = ?
              AND request_kind = ? AND resolution_state = 'open'
         )",
    )
    .bind(request_id.as_str())
    .bind(thread_id.as_str())
    .bind(provider_request_id.as_str())
    .bind(request_kind)
    .fetch_one(pool)
    .await
    .map_err(persistence_error)?;
    if valid {
        Ok(())
    } else {
        Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Chat request is stale or belongs to another thread",
            false,
        ))
    }
}

async fn validate_approval_decision(
    pool: &SqlitePool,
    request_id: &ChatRequestId,
    decision: &ApprovalDecision,
) -> ChatResult<()> {
    let data: String = sqlx::query_scalar(
        "SELECT allowed_decisions_data FROM chat_pending_requests
         WHERE id = ? AND resolution_state = 'open'",
    )
    .bind(request_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::Conflict, "Chat approval is stale", false))?;
    let choices: serde_json::Value = serde_json::from_str(&data).map_err(json_error)?;
    let decision_kind = match decision.kind {
        ApprovalDecisionKind::AllowOnce => "allow_once",
        ApprovalDecisionKind::AllowSession => "allow_session",
        ApprovalDecisionKind::Deny => "deny",
        ApprovalDecisionKind::Cancel => "cancel",
    };
    let Some(provider_option_id) = decision.provider_option_id.as_deref() else {
        return Err(ChatError::new(
            ChatErrorCode::Permission,
            "Approval decision is missing its provider option",
            false,
        ));
    };
    let offered = choices.as_array().is_some_and(|choices| {
        choices.iter().any(|choice| {
            choice.get("id").and_then(serde_json::Value::as_str) == Some(provider_option_id)
                && choice
                    .get("decisionKind")
                    .and_then(serde_json::Value::as_str)
                    == Some(decision_kind)
        })
    });
    if offered {
        Ok(())
    } else {
        Err(ChatError::new(
            ChatErrorCode::Permission,
            "Approval decision was not offered by the provider",
            false,
        ))
    }
}

fn validate_provider_selection(
    provider: &super::settings_commands::ProviderInstanceRead,
    request: &SendChatTurnCommand,
) -> ChatResult<()> {
    if !provider.configuration.enabled
        || provider
            .last_probe
            .as_ref()
            .is_none_or(|probe| probe.state != ProbeState::Healthy)
    {
        return Err(ChatError::new(
            ChatErrorCode::DriverUnavailable,
            "Selected Chat provider is not ready",
            true,
        ));
    }
    let family = provider.configuration.family_id.as_str();
    let permission_mode_supported = match request.modes.safety_mode {
        SafetyMode::AskForApproval | SafetyMode::FullAccess => true,
        SafetyMode::ApproveForMe => matches!(family, "codex" | "claude"),
        SafetyMode::Custom => matches!(family, "codex" | "claude" | "cursor" | "grok" | "opencode"),
    };
    if !permission_mode_supported {
        return Err(ChatError::validation(
            "modes.safetyMode",
            "Selected provider cannot implement this permission mode",
        ));
    }
    if let Some(model_id) = request.model_id.as_ref() {
        let model = provider
            .model_catalog
            .as_ref()
            .and_then(|catalog| catalog.models.iter().find(|model| &model.id == model_id))
            .ok_or_else(|| ChatError::validation("modelId", "Selected model is unavailable"))?;
        if !matches!(
            model.availability,
            ModelAvailability::Available | ModelAvailability::Stale
        ) {
            return Err(ChatError::validation(
                "modelId",
                "Selected model is unavailable",
            ));
        }
        validate_model_options(&model.options, &request.model_options)?;
    } else if provider
        .model_catalog
        .as_ref()
        .is_some_and(|catalog| !catalog.models.is_empty())
    {
        return Err(ChatError::validation(
            "modelId",
            "This provider exposes explicit model choices",
        ));
    } else if !request.model_options.is_empty() {
        return Err(ChatError::validation(
            "modelOptions",
            "Provider-managed model selection cannot include model traits",
        ));
    }
    Ok(())
}

fn validate_model_options(
    definitions: &[ModelOptionDefinition],
    selections: &[ModelOptionSelection],
) -> ChatResult<()> {
    if selections.len() > 100 {
        return Err(ChatError::validation(
            "modelOptions",
            "Too many model traits",
        ));
    }
    let mut keys = BTreeSet::new();
    for selection in selections {
        if !keys.insert(selection.key.as_str()) {
            return Err(ChatError::validation(
                "modelOptions",
                "Model trait keys must be unique",
            ));
        }
        let definition = definitions.iter().find(|definition| match definition {
            ModelOptionDefinition::Boolean { key, .. }
            | ModelOptionDefinition::Choice { key, .. }
            | ModelOptionDefinition::MultipleChoice { key, .. }
            | ModelOptionDefinition::IntegerRange { key, .. }
            | ModelOptionDefinition::Text { key, .. }
            | ModelOptionDefinition::Unknown { key, .. } => key == &selection.key,
        });
        let valid = match (definition, &selection.value) {
            (Some(ModelOptionDefinition::Boolean { .. }), ModelOptionValue::Boolean(_)) => true,
            (
                Some(ModelOptionDefinition::Choice { options, .. }),
                ModelOptionValue::Choice(value),
            ) => options.iter().any(|option| &option.value == value),
            (
                Some(ModelOptionDefinition::MultipleChoice { options, .. }),
                ModelOptionValue::MultipleChoice(values),
            ) => {
                let unique = values.iter().collect::<BTreeSet<_>>();
                unique.len() == values.len()
                    && values
                        .iter()
                        .all(|value| options.iter().any(|option| &option.value == value))
            }
            (
                Some(ModelOptionDefinition::IntegerRange {
                    minimum,
                    maximum,
                    step,
                    ..
                }),
                ModelOptionValue::Integer(value),
            ) => *step > 0 && value >= minimum && value <= maximum && (value - minimum) % step == 0,
            (
                Some(ModelOptionDefinition::Text { allow_empty, .. }),
                ModelOptionValue::Text(value),
            ) => (*allow_empty || !value.is_empty()) && value.len() <= 100_000,
            (Some(ModelOptionDefinition::Unknown { .. }), ModelOptionValue::Unknown(_)) => true,
            (None, ModelOptionValue::Unknown(_)) => true,
            _ => false,
        };
        if !valid {
            return Err(ChatError::validation(
                "modelOptions",
                "A selected model trait is invalid or unavailable",
            ));
        }
    }
    Ok(())
}

fn validate_modes(capabilities: &ProviderCapabilities, modes: TurnModeSnapshot) -> ChatResult<()> {
    if modes.interaction_mode == InteractionMode::Plan
        && !capabilities.supports(ProviderCapability::NativePlan)
    {
        return Err(ChatError::new(
            ChatErrorCode::CapabilityUnsupported,
            "Selected provider does not support native Plan mode",
            true,
        ));
    }
    Ok(())
}

fn validate_explicit_model(provider_managed: bool, model_id: Option<&ModelId>) -> ChatResult<()> {
    if provider_managed == model_id.is_some() {
        return Err(ChatError::validation(
            "model",
            "Choose exactly one model or the provider-managed model state",
        ));
    }
    Ok(())
}

fn validate_prompt(prompt: &str, has_context: bool) -> ChatResult<()> {
    if (!has_context && prompt.trim().is_empty()) || prompt.len() > 16_777_216 {
        return Err(ChatError::validation("prompt", "Chat prompt is invalid"));
    }
    Ok(())
}

fn validate_mentions(mentions: &[WorkspaceMentionReference]) -> ChatResult<()> {
    if mentions.len() > 100
        || mentions.iter().any(|mention| {
            !matches!(mention.kind.as_str(), "file" | "directory")
                || mention.relative_path.is_empty()
                || mention.relative_path.len() > 4_096
                || super::interaction_commands::workspace_mention_is_safety_excluded(
                    &mention.relative_path,
                )
        })
    {
        return Err(ChatError::validation(
            "mentions",
            "Chat mentions are invalid",
        ));
    }
    Ok(())
}

fn validate_send_mentions(
    authorized: &AuthorizedWorkingFolder,
    mentions: &[WorkspaceMentionReference],
) -> ChatResult<()> {
    for mention in mentions {
        let path =
            super::workspace::resolve_workspace_relative_path(authorized, &mention.relative_path)?;
        let metadata = std::fs::metadata(path).map_err(|_| {
            ChatError::new(
                ChatErrorCode::NotFound,
                "A mentioned workspace path is missing",
                true,
            )
        })?;
        if (mention.kind == "file") != metadata.is_file()
            || (mention.kind == "directory") != metadata.is_dir()
        {
            return Err(ChatError::validation(
                "mentions",
                "A workspace mention changed type before send",
            ));
        }
    }
    Ok(())
}

fn validate_answers(answers: &[UserInputAnswer]) -> ChatResult<()> {
    if answers.is_empty()
        || answers.len() > 100
        || answers.iter().any(|answer| {
            answer.question_id.is_empty()
                || answer.selected_option_ids.len() > 100
                || answer
                    .free_form_text
                    .as_ref()
                    .is_some_and(|text| text.len() > 100_000)
        })
    {
        return Err(ChatError::validation(
            "answers",
            "Chat input answers are invalid",
        ));
    }
    Ok(())
}

fn prompt_title(prompt: &str) -> String {
    prompt
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("New conversation")
        .trim()
        .chars()
        .take(80)
        .collect()
}

fn prompt_preview(prompt: &str) -> String {
    prompt.trim().chars().take(2_000).collect()
}

fn wire_safety(value: SafetyMode) -> &'static str {
    match value {
        SafetyMode::AskForApproval => "ask_for_approval",
        SafetyMode::ApproveForMe => "approve_for_me",
        SafetyMode::FullAccess => "full_access",
        SafetyMode::Custom => "custom",
    }
}

fn wire_interaction(value: InteractionMode) -> &'static str {
    match value {
        InteractionMode::Build => "build",
        InteractionMode::Plan => "plan",
    }
}

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value)
        .map_err(|_| ChatError::validation("revision", "Chat revision is too large"))
}

fn operation_context(operation_id: &str, timeout: Duration) -> DriverOperationContext {
    DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline: Instant::now() + timeout,
        cancellation: DriverCancellation::default(),
    }
}

fn now_timestamp() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(SecondsFormat::Millis, true))
        .map_err(|_| ChatError::new(ChatErrorCode::Internal, "create Chat timestamp", false))
}

async fn chat_pool(app: tauri::AppHandle, db_url: String) -> ChatResult<SqlitePool> {
    db_path::connect_sqlite(app, db_url)
        .await
        .map_err(|_| ChatError::new(ChatErrorCode::Persistence, "open Chat database", true))
}

fn runtime_not_running() -> ChatError {
    ChatError::new(
        ChatErrorCode::InvalidStateTransition,
        "Chat provider session is not running",
        true,
    )
}
fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be read",
        true,
    )
}
fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat send persistence failed",
        true,
    )
}
fn json_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat send data is invalid",
        false,
    )
}
fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat send data is invalid",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::tests::repository::pool_with_thread;

    #[test]
    fn stopped_and_failed_sessions_restart_even_if_a_stale_identity_remains() {
        assert!(!reuse_existing_session(true, ProviderSessionState::Stopped).unwrap());
        assert!(!reuse_existing_session(true, ProviderSessionState::Failed).unwrap());
        assert!(reuse_existing_session(true, ProviderSessionState::Ready).unwrap());
        assert!(reuse_existing_session(true, ProviderSessionState::Stopping).is_err());
    }

    #[cfg(unix)]
    struct TestDirectory(std::path::PathBuf);

    #[cfg(unix)]
    impl TestDirectory {
        fn new(label: &str) -> Self {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT: AtomicU64 = AtomicU64::new(1);
            let path = std::env::temp_dir().join(format!(
                "ganbaru-chat-{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    #[cfg(unix)]
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn user_intent_and_receipt_commit_before_provider_dispatch() {
        tauri::async_runtime::block_on(async {
            let pool = pool_with_thread().await;
            let thread_id = ChatThreadId::new("thread-1").unwrap();
            let working_folder_id = ProjectWorkingFolderId::new("workspace-1").unwrap();
            let attachment_id = ChatAttachmentId::new("attachment-before-dispatch").unwrap();
            sqlx::query(
                "INSERT INTO chat_attachments
                    (id, working_folder_id, kind, original_display_name, mime_type, byte_size,
                     sha256, managed_relative_path, signature_kind, created_at)
                 VALUES (?, ?, 'image', 'prompt.png', 'image/png', 14, ?, ?, 'png', ?)",
            )
            .bind(attachment_id.as_str())
            .bind(working_folder_id.as_str())
            .bind("0".repeat(64))
            .bind("assets/chat/attachments/prompt.png")
            .bind("2026-07-21T12:00:00Z")
            .execute(&pool)
            .await
            .unwrap();
            let workspace = workspaces::read_workspace(&pool, &working_folder_id)
                .await
                .unwrap();
            let existing = read_thread_runtime_data(&pool, &thread_id).await.unwrap();
            let request = SendChatTurnCommand {
                command: ChatCommandContext {
                    client_command_id: ChatCommandId::new("send-before-dispatch").unwrap(),
                    expected_thread_revision: Some(existing.revision),
                },
                working_folder_id,
                thread_id: Some(thread_id.clone()),
                new_thread_id: None,
                turn_id: ChatTurnId::new("turn-before-dispatch").unwrap(),
                message_id: ChatMessageId::new("message-before-dispatch").unwrap(),
                provider_instance_id: ProviderInstanceId::new("codex-personal").unwrap(),
                provider_managed_model: true,
                model_id: None,
                model_options: Vec::new(),
                modes: TurnModeSnapshot {
                    safety_mode: SafetyMode::AskForApproval,
                    interaction_mode: InteractionMode::Build,
                },
                prompt: "  Preserve this exact prompt\n".to_string(),
                attachment_ids: vec![attachment_id.clone()],
                mentions: Vec::new(),
            };
            let attachments = [PromptAttachmentReference {
                attachment_id,
                kind: "image".to_string(),
                display_name: "prompt.png".to_string(),
                managed_relative_path: "assets/chat/attachments/prompt.png".to_string(),
                mime_type: Some("image/png".to_string()),
                byte_size: 14,
                local_path: Some("/vault/assets/chat/attachments/prompt.png".to_string()),
                text_content: None,
            }];
            let family_id = ProviderFamilyId::new("codex").unwrap();
            let persisted_at = UtcTimestamp::new("2026-07-21T12:00:00Z").unwrap();
            persist_user_turn(PersistUserTurnContext {
                pool: &pool,
                workspace: &workspace,
                thread_id: &thread_id,
                existing: Some(&existing),
                continuation_group_id: &existing.continuation_group_id,
                provider_family_id: &family_id,
                request: &request,
                attachments: &attachments,
                now: &persisted_at,
            })
            .await
            .unwrap();

            let stored: String = sqlx::query_scalar(
                "SELECT normalized_markdown FROM chat_messages WHERE id = 'message-before-dispatch'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let context: String = sqlx::query_scalar(
                "SELECT content_metadata_data FROM chat_messages
                 WHERE id = 'message-before-dispatch'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let receipt_state: String = sqlx::query_scalar(
                "SELECT state FROM chat_command_receipts WHERE client_command_id = 'send-before-dispatch'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            let turn_state: String = sqlx::query_scalar(
                "SELECT state FROM chat_turns WHERE id = 'turn-before-dispatch'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(stored, "  Preserve this exact prompt\n");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&context).unwrap()["attachments"][0]
                    ["kind"],
                "image"
            );
            assert_eq!(receipt_state, "accepted");
            assert_eq!(turn_state, "pending");
        });
    }

    #[test]
    fn model_options_require_declared_types_values_and_unique_keys() {
        let definitions = vec![
            ModelOptionDefinition::Choice {
                key: "effort".to_string(),
                label: "Effort".to_string(),
                description: None,
                options: vec![ModelChoiceOption {
                    value: "high".to_string(),
                    label: "High".to_string(),
                    description: None,
                }],
                default_value: None,
            },
            ModelOptionDefinition::IntegerRange {
                key: "budget".to_string(),
                label: "Budget".to_string(),
                description: None,
                minimum: 2,
                maximum: 10,
                step: 2,
                default_value: None,
            },
        ];
        assert!(validate_model_options(
            &definitions,
            &[
                ModelOptionSelection {
                    key: "effort".to_string(),
                    value: ModelOptionValue::Choice("high".to_string()),
                },
                ModelOptionSelection {
                    key: "budget".to_string(),
                    value: ModelOptionValue::Integer(6),
                },
            ],
        )
        .is_ok());
        for invalid in [
            vec![ModelOptionSelection {
                key: "effort".to_string(),
                value: ModelOptionValue::Choice("undeclared".to_string()),
            }],
            vec![ModelOptionSelection {
                key: "budget".to_string(),
                value: ModelOptionValue::Integer(5),
            }],
            vec![
                ModelOptionSelection {
                    key: "effort".to_string(),
                    value: ModelOptionValue::Choice("high".to_string()),
                },
                ModelOptionSelection {
                    key: "effort".to_string(),
                    value: ModelOptionValue::Choice("high".to_string()),
                },
            ],
        ] {
            assert!(validate_model_options(&definitions, &invalid).is_err());
        }
    }

    #[test]
    fn approval_validation_rejects_cross_thread_expired_and_fabricated_requests() {
        tauri::async_runtime::block_on(async {
            let pool = pool_with_thread().await;
            sqlx::query(
                "INSERT INTO chat_pending_requests
                    (id, thread_id, provider_request_id, request_kind,
                     safe_display_data, allowed_decisions_data, opened_sequence, opened_at)
                 VALUES ('approval-1', 'thread-1', 'provider-approval-1', 'approval',
                         '{}', ?, 1, '2026-07-21T12:00:00Z')",
            )
            .bind(
                serde_json::json!([{
                    "id": "allow-once",
                    "label": "Allow once",
                    "decisionKind": "allow_once",
                    "description": null
                }])
                .to_string(),
            )
            .execute(&pool)
            .await
            .unwrap();
            let thread = ChatThreadId::new("thread-1").unwrap();
            let request = ChatRequestId::new("approval-1").unwrap();
            let provider_request = ProviderRequestId::new("provider-approval-1").unwrap();

            validate_pending_request(&pool, &thread, &request, &provider_request, "approval")
                .await
                .unwrap();
            for (candidate_thread, candidate_request, candidate_provider) in [
                (
                    ChatThreadId::new("thread-other").unwrap(),
                    request.clone(),
                    provider_request.clone(),
                ),
                (
                    thread.clone(),
                    ChatRequestId::new("fabricated-request").unwrap(),
                    provider_request.clone(),
                ),
                (
                    thread.clone(),
                    request.clone(),
                    ProviderRequestId::new("fabricated-provider-request").unwrap(),
                ),
            ] {
                let error = validate_pending_request(
                    &pool,
                    &candidate_thread,
                    &candidate_request,
                    &candidate_provider,
                    "approval",
                )
                .await
                .unwrap_err();
                assert_eq!(error.code, ChatErrorCode::Conflict);
            }

            validate_approval_decision(
                &pool,
                &request,
                &ApprovalDecision {
                    kind: ApprovalDecisionKind::AllowOnce,
                    provider_option_id: Some("allow-once".to_string()),
                    updated_tool_input: None,
                },
            )
            .await
            .unwrap();
            for decision in [
                ApprovalDecision {
                    kind: ApprovalDecisionKind::AllowSession,
                    provider_option_id: Some("allow-once".to_string()),
                    updated_tool_input: None,
                },
                ApprovalDecision {
                    kind: ApprovalDecisionKind::AllowOnce,
                    provider_option_id: Some("fabricated-option".to_string()),
                    updated_tool_input: None,
                },
            ] {
                let error = validate_approval_decision(&pool, &request, &decision)
                    .await
                    .unwrap_err();
                assert_eq!(error.code, ChatErrorCode::Permission);
            }

            sqlx::query(
                "UPDATE chat_pending_requests
                 SET resolution_state = 'stale', resolved_at = '2026-07-21T12:01:00Z'
                 WHERE id = 'approval-1'",
            )
            .execute(&pool)
            .await
            .unwrap();
            let expired =
                validate_pending_request(&pool, &thread, &request, &provider_request, "approval")
                    .await
                    .unwrap_err();
            assert_eq!(expired.code, ChatErrorCode::Conflict);
        });
    }

    #[cfg(unix)]
    #[test]
    fn send_time_validation_rejects_file_and_directory_symlink_swaps() {
        use std::os::unix::fs::symlink;

        let workspace = TestDirectory::new("mention-swap-workspace");
        let outside = TestDirectory::new("mention-swap-outside");
        std::fs::write(workspace.0.join("selected.txt"), "inside").unwrap();
        std::fs::write(outside.0.join("secret.txt"), "outside").unwrap();
        std::fs::create_dir(workspace.0.join("selected-directory")).unwrap();
        std::fs::create_dir(outside.0.join("outside-directory")).unwrap();
        let authorized = AuthorizedWorkingFolder {
            working_folder_id: ProjectWorkingFolderId::new("workspace-1").unwrap(),
            canonical_path: workspace.0.clone(),
            repository_kind: RepositoryKind::None,
            repository_identity: None,
        };
        let file_mention = WorkspaceMentionReference {
            relative_path: "selected.txt".to_string(),
            kind: "file".to_string(),
        };
        let directory_mention = WorkspaceMentionReference {
            relative_path: "selected-directory".to_string(),
            kind: "directory".to_string(),
        };

        validate_send_mentions(&authorized, std::slice::from_ref(&file_mention)).unwrap();
        validate_send_mentions(&authorized, std::slice::from_ref(&directory_mention)).unwrap();

        std::fs::remove_file(workspace.0.join("selected.txt")).unwrap();
        symlink(
            outside.0.join("secret.txt"),
            workspace.0.join("selected.txt"),
        )
        .unwrap();
        std::fs::remove_dir(workspace.0.join("selected-directory")).unwrap();
        symlink(
            outside.0.join("outside-directory"),
            workspace.0.join("selected-directory"),
        )
        .unwrap();

        for mention in [file_mention, directory_mention] {
            let error = validate_send_mentions(&authorized, &[mention]).unwrap_err();
            assert_eq!(error.code, ChatErrorCode::Permission);
            assert!(!error.message.contains("secret.txt"));
        }
    }
}
