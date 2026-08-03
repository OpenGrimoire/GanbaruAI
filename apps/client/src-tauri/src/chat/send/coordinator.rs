//! Readable send-turn coordinator across validation, persistence, and runtime launch.

use super::checkpoints::ensure_pre_turn_checkpoint;
use super::persistence::{
    mark_turn_dispatch_failed, persist_user_turn, read_attachment_references,
    read_thread_runtime_data, replay_send_receipt, PersistUserTurnContext,
};
use super::session::{ensure_session, EnsureSessionContext};
use super::support::{
    chat_pool, device_state_error, now_timestamp, operation_context, versioned_value,
    PROVIDER_START_TIMEOUT, TURN_OPERATION_TIMEOUT,
};
use super::validation::{
    validate_explicit_model, validate_mentions, validate_modes, validate_prompt,
    validate_provider_selection, validate_send_mentions,
};
use crate::chat::agent_runs::TurnOrigin;
use crate::chat::credentials::{materialize_provider_environment, PlatformCredentialStore};
use crate::chat::device_state::{full_access_is_trusted, read_active_device_scope};
use crate::chat::models::*;
use crate::chat::providers::{ProviderDriverFactory, ProviderDriverRegistry};
use crate::chat::repository::receipts::{
    complete_command_receipt, read_command_receipt, CommandReceiptState,
};
use crate::chat::repository::{reads, workspaces};
use crate::chat::runtime::ChatRuntimeRegistry;
use crate::chat::send_commands::{SendChatTurnCommand, SendChatTurnResult};
use crate::chat::workspace::WorkingFolderAuthorizationOperation;
use crate::vault;
use sqlx::SqlitePool;
use std::collections::BTreeMap;
use tauri::Manager;

pub(crate) async fn send_turn(
    app: tauri::AppHandle,
    db_url: String,
    request: SendChatTurnCommand,
    origin: TurnOrigin,
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
    let existing = match request.thread_id.as_ref() {
        Some(existing_thread_id) => {
            Some(read_thread_runtime_data(&pool, existing_thread_id).await?)
        }
        None => None,
    };
    let logical_workspace = workspaces::read_workspace(&pool, &request.working_folder_id).await?;
    require_project_accepts_ai_work(&pool, &logical_workspace.project_id).await?;
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    let authorized = crate::chat::workspace_commands::authorize_working_folder(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::ProviderStart,
    )
    .await?;
    let selected_environment = existing
        .as_ref()
        .and_then(|data| data.execution_environment_id.as_deref())
        .or(request.execution_environment_id.as_deref());
    let authorized = crate::chat::execution_environment::resolve_environment_workspace(
        &app,
        &pool,
        authorized,
        selected_environment,
    )
    .await?;
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
    let provider =
        crate::chat::settings_commands::read_provider(&app, &request.provider_instance_id)?;
    validate_provider_selection(&provider, &request)?;
    let mut configuration = materialize_provider_environment(
        &provider.configuration,
        &PlatformCredentialStore::default(),
    )?;
    let resource_endpoint = app
        .state::<crate::chat::internal_mcp::InternalMcpRegistry>()
        .ensure_thread_endpoint(
            app.clone(),
            pool.clone(),
            vault::active_vault_path(&app).map_err(|_| {
                ChatError::new(
                    ChatErrorCode::Persistence,
                    "Active Ganbaru folder is unavailable",
                    true,
                )
            })?,
            thread_id.clone(),
        )
        .await?;
    configuration.internal_mcp = Some(ProviderInternalMcpConfig {
        name: "ganbaru-chat".to_string(),
        url: resource_endpoint.url,
        bearer_token: resource_endpoint.bearer_token,
    });
    if let Some(existing) = &existing {
        if existing.working_folder_id != request.working_folder_id
            || existing.provider_instance_id != request.provider_instance_id
            || request
                .execution_environment_id
                .as_deref()
                .is_some_and(|requested| {
                    existing.execution_environment_id.as_deref() != Some(requested)
                })
        {
            return Err(ChatError::new(
                ChatErrorCode::Conflict,
                "Changing workspace, provider, or execution environment requires a new Chat thread",
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
    let mutation_registry =
        app.state::<crate::chat::workspace_mutation::ChatWorkspaceMutationRegistry>();
    let reservation = mutation_registry.begin_provider_turn(
        &authorized.canonical_path,
        &thread_id,
        &request.turn_id,
    )?;
    let persistence = persist_user_turn(PersistUserTurnContext {
        pool: &pool,
        workspace: &logical_workspace,
        thread_id: &thread_id,
        existing: existing.as_ref(),
        continuation_group_id: &continuation_group_id,
        provider_family_id: &provider.configuration.family_id,
        request: &request,
        origin: &origin,
        attachments: &attachment_references,
        now: &persistence_now,
    })
    .await;
    persistence?;
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
        let reservation = reservation.handoff_to_runtime()?;
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
                    developer_instructions: origin.developer_instructions().map(str::to_string),
                },
                reservation,
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
    if let Some(binding) = origin.run() {
        crate::chat::agent_runs::settle_launch(
            &pool,
            binding,
            launch_error.as_ref(),
            &now_timestamp()?,
        )
        .await?;
    }
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
