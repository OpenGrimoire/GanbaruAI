//! Native composer resources, workspace mentions, and interactive request commands.

use super::device_state::{read_active_device_scope, update_active_device_scope};
use super::events::{
    AccountStatusEvent, CanonicalEvent, RateLimitStatusEvent, ThreadUsageUpdatedEvent,
};
use super::models::{
    ChatAttachmentId, ChatCommandContext, ChatCommandId, ChatError, ChatErrorCode, ChatResult,
    ChatThreadId, DriverOperationReceipt, InterruptTurnRequest, ProjectWorkingFolderId,
    ProviderCapabilities, ProviderInstanceId, ProviderSessionState, UtcTimestamp, VersionedJson,
};
use super::repository::receipts::{
    claim_command_receipt, complete_command_receipt, CommandReceiptClaim, CommandReceiptRead,
    CommandReceiptState,
};
use super::repository::{attachments, workspaces};
use super::runtime::ChatRuntimeRegistry;
use super::workspace::{
    authorize_workspace, resolve_workspace_relative_path, WorkingFolderAuthorizationOperation,
};
use crate::projects::working_folders::read_active_working_folder_scope;
use crate::{db_path, vault};
use base64::{engine::general_purpose, Engine as _};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};

const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
const MAX_IMAGE_COUNT: usize = 8;
const MAX_PATH_RESULTS: u32 = 100;
const MAX_PATH_SCAN: usize = 20_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportChatImageRequest {
    pub attachment_id: ChatAttachmentId,
    pub working_folder_id: ProjectWorkingFolderId,
    pub display_name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickChatImagesRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportChatTextSnippetRequest {
    pub working_folder_id: ProjectWorkingFolderId,
    pub attachment_id: ChatAttachmentId,
    pub display_name: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderPathRead {
    pub relative_path: String,
    pub display_name: String,
    pub kind: String,
    pub ignored: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkingFolderPathPage {
    pub entries: Vec<ProjectWorkingFolderPathRead>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPromptCatalogEntry {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub kind: String,
    pub stale: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPendingRequestRead {
    pub id: String,
    pub turn_id: Option<String>,
    pub provider_request_id: String,
    pub request_kind: String,
    pub safe_display: VersionedJson,
    pub allowed_decisions: VersionedJson,
    pub opened_at: UtcTimestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatQueuedFollowupRead {
    pub id: String,
    pub thread_id: ChatThreadId,
    pub text: String,
    pub provider_instance_id: ProviderInstanceId,
    pub model_selection: VersionedJson,
    pub safety_mode: super::models::SafetyMode,
    pub interaction_mode: super::models::InteractionMode,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub mentions: VersionedJson,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatInteractionStateRead {
    pub session_state: ProviderSessionState,
    pub active_turn_id: Option<String>,
    pub capabilities: ProviderCapabilities,
    pub pending_request: Option<ChatPendingRequestRead>,
    pub queued_followup: Option<ChatQueuedFollowupRead>,
    pub usage: Option<ThreadUsageUpdatedEvent>,
    pub account_status: Option<AccountStatusEvent>,
    pub rate_limit_status: Option<RateLimitStatusEvent>,
    pub automatic_compaction_reported: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveQueuedFollowupRequest {
    pub id: String,
    pub thread_id: ChatThreadId,
    pub text: String,
    pub provider_instance_id: ProviderInstanceId,
    pub model_selection: VersionedJson,
    pub safety_mode: super::models::SafetyMode,
    pub interaction_mode: super::models::InteractionMode,
    pub attachment_ids: Vec<ChatAttachmentId>,
    pub mentions: VersionedJson,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatUserInputDraftRead {
    pub request_id: String,
    pub answers: VersionedJson,
    pub updated_at: UtcTimestamp,
}

#[tauri::command]
pub async fn chat_import_image(
    app: tauri::AppHandle,
    db_url: String,
    request: ImportChatImageRequest,
) -> ChatResult<attachments::ChatAttachmentRead> {
    if request.bytes.len() > MAX_IMAGE_BYTES {
        return Err(ChatError::validation(
            "image",
            "Chat images must be 20 MiB or smaller",
        ));
    }
    let pool = chat_pool(app.clone(), db_url).await?;
    require_workspace(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await?;
    let vault_root = vault::active_vault_path(&app).map_err(vault_error)?;
    let now = now_timestamp()?;
    attachments::import_attachment_bytes(
        &pool,
        &vault_root,
        attachments::AttachmentBytesImport {
            working_folder_id: &request.working_folder_id,
            attachment_id: request.attachment_id,
            display_name: request.display_name,
            bytes: &request.bytes,
            requested_kind: attachments::ChatAttachmentKind::Image,
            now: &now,
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_pick_images(
    app: tauri::AppHandle,
    db_url: String,
    request: PickChatImagesRequest,
) -> ChatResult<Vec<attachments::ChatAttachmentRead>> {
    if request.attachment_ids.is_empty() || request.attachment_ids.len() > MAX_IMAGE_COUNT {
        return Err(ChatError::validation(
            "attachmentIds",
            "Choose between one and eight Chat images",
        ));
    }
    let title = request.title.trim();
    if title.is_empty() || title.len() > 160 || title.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "title",
            "Image picker title is invalid",
        ));
    }
    let (sender, mut receiver) = tauri::async_runtime::channel(1);
    app.dialog()
        .file()
        .set_title(title)
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
        .pick_files(move |selection| {
            let result = selection
                .unwrap_or_default()
                .into_iter()
                .map(file_path_to_path)
                .collect::<ChatResult<Vec<_>>>();
            let _ = sender.try_send(result);
        });
    let selected = receiver.recv().await.ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::Internal,
            "Image picker did not respond",
            true,
        )
    })??;
    if selected.len() > request.attachment_ids.len() {
        return Err(ChatError::validation(
            "images",
            "More images were selected than the available attachment slots",
        ));
    }
    let pool = chat_pool(app.clone(), db_url).await?;
    require_workspace(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await?;
    let vault_root = vault::active_vault_path(&app).map_err(vault_error)?;
    let mut total_bytes = 0_u64;
    for path in &selected {
        let metadata = fs::symlink_metadata(path).map_err(attachment_io_error)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > MAX_IMAGE_BYTES as u64
        {
            return Err(ChatError::validation(
                "image",
                "Chat images must be 20 MiB or smaller",
            ));
        }
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| ChatError::validation("images", "Selected images are too large"))?;
        if total_bytes > 50 * 1024 * 1024 {
            return Err(ChatError::validation(
                "images",
                "Selected Chat images must total 50 MiB or less",
            ));
        }
    }
    let mut imported = Vec::with_capacity(selected.len());
    for (path, attachment_id) in selected.into_iter().zip(request.attachment_ids) {
        imported.push(
            attachments::import_attachment(
                &pool,
                &vault_root,
                &request.working_folder_id,
                attachment_id,
                &path,
                attachments::ChatAttachmentKind::Image,
                &now_timestamp()?,
            )
            .await?,
        );
    }
    Ok(imported)
}

#[tauri::command]
pub async fn chat_attachment_data_url(
    app: tauri::AppHandle,
    db_url: String,
    attachment_id: ChatAttachmentId,
) -> ChatResult<String> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let attachment = attachments::read_attachment(&pool, &attachment_id)
        .await?
        .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Chat image was not found", true))?;
    if attachment.kind != attachments::ChatAttachmentKind::Image {
        return Err(ChatError::validation(
            "attachmentId",
            "Attachment is not an image",
        ));
    }
    let path = managed_attachment_path(&app, &attachment.managed_relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(attachment_io_error)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() != attachment.byte_size
        || metadata.len() > MAX_IMAGE_BYTES as u64
    {
        return Err(attachment_io_error(()));
    }
    let bytes = fs::read(path).map_err(attachment_io_error)?;
    Ok(format!(
        "data:{};base64,{}",
        attachment.mime_type,
        general_purpose::STANDARD.encode(bytes)
    ))
}

#[tauri::command]
pub async fn chat_read_attachments(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    attachment_ids: Vec<ChatAttachmentId>,
) -> ChatResult<Vec<attachments::ChatAttachmentRead>> {
    if attachment_ids.len() > 20 {
        return Err(ChatError::validation(
            "attachmentIds",
            "Too many Chat attachments were requested",
        ));
    }
    let pool = chat_pool(app, db_url).await?;
    let mut result = Vec::with_capacity(attachment_ids.len());
    for id in attachment_ids {
        let attachment = attachments::read_attachment(&pool, &id)
            .await?
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "Chat attachment was not found",
                    true,
                )
            })?;
        if attachment.working_folder_id != working_folder_id {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Chat attachment belongs to another workspace",
                false,
            ));
        }
        result.push(attachment);
    }
    Ok(result)
}

#[tauri::command]
pub async fn chat_import_text_snippet(
    app: tauri::AppHandle,
    db_url: String,
    request: ImportChatTextSnippetRequest,
) -> ChatResult<attachments::ChatAttachmentRead> {
    if request.text.is_empty() || request.text.len() > 128 * 1024 || request.text.contains('\0') {
        return Err(ChatError::validation(
            "text",
            "Chat text context must be between 1 byte and 128 KiB",
        ));
    }
    let pool = chat_pool(app.clone(), db_url).await?;
    require_workspace(
        &app,
        &pool,
        &request.working_folder_id,
        WorkingFolderAuthorizationOperation::FileRead,
    )
    .await?;
    let now = now_timestamp()?;
    attachments::import_attachment_bytes(
        &pool,
        &vault::active_vault_path(&app).map_err(vault_error)?,
        attachments::AttachmentBytesImport {
            working_folder_id: &request.working_folder_id,
            attachment_id: request.attachment_id,
            display_name: request.display_name,
            bytes: request.text.as_bytes(),
            requested_kind: attachments::ChatAttachmentKind::TextSnippet,
            now: &now,
        },
    )
    .await
}

#[tauri::command]
pub async fn chat_search_working_folder_paths(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    query: String,
    include_ignored: bool,
    cursor: Option<String>,
    limit: u32,
) -> ChatResult<ProjectWorkingFolderPathPage> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(
        &app,
        &pool,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::MentionResolution,
    )
    .await?;
    let query = query.trim().to_lowercase();
    if query.len() > 500 || query.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "query",
            "Workspace path query is invalid",
        ));
    }
    let offset = parse_offset(cursor.as_deref())?;
    let visible = git_visible_paths(&authorized.canonical_path).unwrap_or_default();
    let mut entries = scan_paths(&authorized.canonical_path, include_ignored, &visible)?;
    entries.retain(|entry| fuzzy_path_matches(&entry.relative_path, &query));
    entries.sort_by(|left, right| {
        path_rank(&left.relative_path, &query)
            .cmp(&path_rank(&right.relative_path, &query))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    let page_size = limit.clamp(1, MAX_PATH_RESULTS) as usize;
    let page = entries
        .into_iter()
        .skip(offset)
        .take(page_size + 1)
        .collect::<Vec<_>>();
    let has_more = page.len() > page_size;
    Ok(ProjectWorkingFolderPathPage {
        entries: page.into_iter().take(page_size).collect(),
        next_cursor: has_more.then(|| (offset + page_size).to_string()),
    })
}

#[tauri::command]
pub async fn chat_validate_working_folder_mentions(
    app: tauri::AppHandle,
    db_url: String,
    working_folder_id: ProjectWorkingFolderId,
    relative_paths: Vec<String>,
) -> ChatResult<()> {
    if relative_paths.len() > 100 {
        return Err(ChatError::validation(
            "mentions",
            "Too many workspace mentions",
        ));
    }
    let pool = chat_pool(app.clone(), db_url).await?;
    let authorized = require_workspace(
        &app,
        &pool,
        &working_folder_id,
        WorkingFolderAuthorizationOperation::MentionResolution,
    )
    .await?;
    for path in relative_paths {
        if workspace_mention_is_safety_excluded(&path) {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "This workspace path is excluded from Chat context",
                false,
            ));
        }
        resolve_workspace_relative_path(&authorized, &path)?;
    }
    Ok(())
}

#[tauri::command]
pub fn chat_list_prompt_catalog(
    app: tauri::AppHandle,
    provider_instance_id: ProviderInstanceId,
) -> ChatResult<Vec<ChatPromptCatalogEntry>> {
    let settings = super::settings_commands::read_provider(&app, &provider_instance_id)?;
    let stale = settings
        .last_probe
        .as_ref()
        .is_none_or(|probe| probe.state != super::models::ProbeState::Healthy);
    let mut entries = Vec::new();
    if settings.configuration.family_id.as_str() == "codex" {
        for (value, label, description) in [
            (
                "/compact",
                "Compact context",
                "Ask Codex to compact the active context",
            ),
            (
                "/review",
                "Review changes",
                "Start a provider-native code review",
            ),
            (
                "/status",
                "Provider status",
                "Show provider session and account status",
            ),
            ("/mcp", "MCP status", "Show configured MCP server status"),
        ] {
            entries.push(ChatPromptCatalogEntry {
                value: value.to_string(),
                label: label.to_string(),
                description: Some(description.to_string()),
                kind: "command".to_string(),
                stale,
            });
        }
    }
    if let Some(home) = provider_skill_home(&settings.configuration) {
        entries.extend(read_skill_entries(&home, stale)?);
    }
    Ok(entries)
}

#[tauri::command]
pub async fn chat_read_interaction_state(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
) -> ChatResult<ChatInteractionStateRead> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(thread_id.clone())?;
    let snapshot = owner.snapshot()?;
    Ok(ChatInteractionStateRead {
        session_state: snapshot.session_state,
        active_turn_id: snapshot.active_turn_id.map(|value| value.into_inner()),
        capabilities: snapshot.capabilities,
        pending_request: read_pending_request(&pool, &thread_id).await?,
        queued_followup: read_queued_followup(&pool, &thread_id).await?,
        usage: read_latest_usage(&pool, &thread_id).await?,
        account_status: read_latest_account_status(&pool, &thread_id).await?,
        rate_limit_status: read_latest_rate_limit_status(&pool, &thread_id).await?,
        automatic_compaction_reported: automatic_compaction_reported(&pool, &thread_id).await?,
    })
}

#[tauri::command]
pub fn chat_set_full_access_trust(
    app: tauri::AppHandle,
    provider_instance_id: ProviderInstanceId,
    working_folder_id: ProjectWorkingFolderId,
    trusted: bool,
) -> ChatResult<bool> {
    let timestamp = now_timestamp()?;
    update_active_device_scope(&app, |scope| {
        super::device_state::set_full_access_trust(
            scope,
            provider_instance_id,
            working_folder_id,
            trusted.then_some(timestamp),
        );
        Ok(())
    })
    .map_err(device_state_error)?;
    Ok(trusted)
}

#[tauri::command]
pub fn chat_has_full_access_trust(
    app: tauri::AppHandle,
    provider_instance_id: ProviderInstanceId,
    working_folder_id: ProjectWorkingFolderId,
) -> ChatResult<bool> {
    let scope = read_active_device_scope(&app).map_err(device_state_error)?;
    Ok(super::device_state::full_access_is_trusted(
        &scope,
        &provider_instance_id,
        &working_folder_id,
    ))
}

#[tauri::command]
pub async fn chat_save_queued_followup(
    app: tauri::AppHandle,
    db_url: String,
    request: SaveQueuedFollowupRequest,
) -> ChatResult<ChatQueuedFollowupRead> {
    let text = request.text.trim();
    if text.is_empty() || text.len() > 16_777_216 || request.attachment_ids.len() > MAX_IMAGE_COUNT
    {
        return Err(ChatError::validation(
            "queue",
            "Queued follow-up is invalid",
        ));
    }
    if !request.mentions.value.is_array() {
        return Err(ChatError::validation(
            "mentions",
            "Queued mentions must be an array",
        ));
    }
    let model = request.model_selection.value.as_object().ok_or_else(|| {
        ChatError::validation("modelSelection", "Queued model selection is invalid")
    })?;
    let model_id = model.get("modelId").and_then(serde_json::Value::as_str);
    let provider_managed = model
        .get("providerManaged")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if provider_managed == model_id.is_some()
        || !model
            .get("options")
            .is_some_and(serde_json::Value::is_array)
    {
        return Err(ChatError::validation(
            "modelSelection",
            "Queued model selection is invalid",
        ));
    }
    let pool = chat_pool(app, db_url).await?;
    let thread = sqlx::query(
        "SELECT working_folder_id, provider_instance_id FROM chat_threads
         WHERE id = ? AND archived_at IS NULL",
    )
    .bind(request.thread_id.as_str())
    .fetch_optional(&pool)
    .await
    .map_err(persistence_error)?
    .ok_or_else(|| ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true))?;
    let working_folder_id: String = thread
        .try_get("working_folder_id")
        .map_err(persistence_error)?;
    let pinned_provider: String = thread
        .try_get("provider_instance_id")
        .map_err(persistence_error)?;
    if request.provider_instance_id.as_str() != pinned_provider {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Queued follow-up provider does not match the native thread",
            true,
        ));
    }
    let mut total_attachment_bytes = 0_u64;
    for attachment_id in &request.attachment_ids {
        let attachment = attachments::read_attachment(&pool, attachment_id)
            .await?
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::NotFound,
                    "Chat attachment was not found",
                    true,
                )
            })?;
        if attachment.working_folder_id.as_str() != working_folder_id {
            return Err(ChatError::new(
                ChatErrorCode::Permission,
                "Queued attachment belongs to another workspace",
                false,
            ));
        }
        total_attachment_bytes = total_attachment_bytes
            .checked_add(attachment.byte_size)
            .ok_or_else(|| {
                ChatError::validation("attachments", "Queued attachments are too large")
            })?;
        if total_attachment_bytes > 50 * 1024 * 1024 {
            return Err(ChatError::validation(
                "attachments",
                "Queued attachments must total 50 MiB or less",
            ));
        }
    }
    let attachment_ids = serde_json::to_string(&request.attachment_ids).map_err(json_error)?;
    let mentions = serde_json::to_string(&request.mentions.value).map_err(json_error)?;
    let now = now_timestamp()?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    sqlx::query(
        "INSERT INTO chat_queued_followups
            (id, thread_id, text, provider_instance_id, model_selection_schema_version,
             model_selection_data, safety_mode, interaction_mode, attachment_ids_data,
             mentions_schema_version, mentions_data, state, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'queued', ?, ?)
         ON CONFLICT(id) DO UPDATE SET text = excluded.text,
             provider_instance_id = excluded.provider_instance_id,
             model_selection_schema_version = excluded.model_selection_schema_version,
             model_selection_data = excluded.model_selection_data,
             safety_mode = excluded.safety_mode,
             interaction_mode = excluded.interaction_mode,
             attachment_ids_data = excluded.attachment_ids_data,
             mentions_schema_version = excluded.mentions_schema_version,
             mentions_data = excluded.mentions_data, state = 'queued',
             updated_at = excluded.updated_at",
    )
    .bind(&request.id)
    .bind(request.thread_id.as_str())
    .bind(&request.text)
    .bind(request.provider_instance_id.as_str())
    .bind(i64::from(request.model_selection.schema_version))
    .bind(serde_json::to_string(&request.model_selection.value).map_err(json_error)?)
    .bind(wire_safety_mode(request.safety_mode))
    .bind(wire_interaction_mode(request.interaction_mode))
    .bind(attachment_ids)
    .bind(i64::from(request.mentions.schema_version))
    .bind(mentions)
    .bind(now.as_str())
    .bind(now.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    sqlx::query("DELETE FROM chat_queued_attachment_references WHERE queued_followup_id = ?")
        .bind(&request.id)
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    for attachment_id in &request.attachment_ids {
        sqlx::query(
            "INSERT INTO chat_queued_attachment_references
                (queued_followup_id, attachment_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(&request.id)
        .bind(attachment_id.as_str())
        .bind(now.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(persistence_error)?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    read_queued_followup(&pool, &request.thread_id)
        .await?
        .ok_or_else(corrupt_data)
}

#[tauri::command]
pub async fn chat_cancel_queued_followup(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
) -> ChatResult<bool> {
    let pool = chat_pool(app, db_url).await?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let active_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM chat_queued_followups WHERE thread_id = ? AND state = 'queued'",
    )
    .bind(thread_id.as_str())
    .fetch_optional(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    let changed = sqlx::query(
        "UPDATE chat_queued_followups SET state = 'cancelled', updated_at = ?
         WHERE thread_id = ? AND state = 'queued'",
    )
    .bind(now_timestamp()?.as_str())
    .bind(thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?
    .rows_affected();
    if let Some(active_id) = active_id {
        sqlx::query("DELETE FROM chat_queued_attachment_references WHERE queued_followup_id = ?")
            .bind(active_id)
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    Ok(changed > 0)
}

#[tauri::command]
pub async fn chat_mark_queued_followup_dispatched(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    queued_followup_id: String,
) -> ChatResult<bool> {
    let pool = chat_pool(app, db_url).await?;
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let changed = sqlx::query(
        "UPDATE chat_queued_followups SET state = 'dispatched', updated_at = ?
         WHERE id = ? AND thread_id = ? AND state = 'queued'",
    )
    .bind(now_timestamp()?.as_str())
    .bind(&queued_followup_id)
    .bind(thread_id.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?
    .rows_affected();
    if changed > 0 {
        sqlx::query("DELETE FROM chat_queued_attachment_references WHERE queued_followup_id = ?")
            .bind(&queued_followup_id)
            .execute(&mut *transaction)
            .await
            .map_err(persistence_error)?;
    }
    transaction.commit().await.map_err(persistence_error)?;
    Ok(changed > 0)
}

#[tauri::command]
pub async fn chat_read_user_input_draft(
    app: tauri::AppHandle,
    db_url: String,
    request_id: String,
) -> ChatResult<Option<ChatUserInputDraftRead>> {
    read_user_input_draft(&chat_pool(app, db_url).await?, &request_id).await
}

async fn read_user_input_draft(
    pool: &SqlitePool,
    request_id: &str,
) -> ChatResult<Option<ChatUserInputDraftRead>> {
    let row = sqlx::query(
        "SELECT request_id, answers_schema_version, answers_data, updated_at
         FROM chat_user_input_drafts WHERE request_id = ?",
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    row.map(|row| {
        Ok(ChatUserInputDraftRead {
            request_id: row.try_get("request_id").map_err(persistence_error)?,
            answers: versioned_row(&row, "answers_schema_version", "answers_data")?,
            updated_at: UtcTimestamp::new(
                row.try_get::<String, _>("updated_at")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
        })
    })
    .transpose()
}

#[tauri::command]
pub async fn chat_save_user_input_draft(
    app: tauri::AppHandle,
    db_url: String,
    request_id: String,
    answers: VersionedJson,
) -> ChatResult<ChatUserInputDraftRead> {
    if !answers.value.is_array() {
        return Err(ChatError::validation(
            "answers",
            "Chat answer draft must be an array",
        ));
    }
    let pool = chat_pool(app, db_url).await?;
    let now = now_timestamp()?;
    sqlx::query(
        "INSERT INTO chat_user_input_drafts
            (request_id, answers_schema_version, answers_data, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(request_id) DO UPDATE SET
            answers_schema_version = excluded.answers_schema_version,
            answers_data = excluded.answers_data, updated_at = excluded.updated_at",
    )
    .bind(&request_id)
    .bind(i64::from(answers.schema_version))
    .bind(serde_json::to_string(&answers.value).map_err(json_error)?)
    .bind(now.as_str())
    .execute(&pool)
    .await
    .map_err(persistence_error)?;
    read_user_input_draft(&pool, &request_id)
        .await?
        .ok_or_else(corrupt_data)
}

#[tauri::command]
pub async fn chat_stop_session(
    app: tauri::AppHandle,
    db_url: String,
    thread_id: ChatThreadId,
    force: bool,
    client_command_id: ChatCommandId,
) -> ChatResult<DriverOperationReceipt> {
    let pool = chat_pool(app.clone(), db_url).await?;
    let owner = app
        .state::<ChatRuntimeRegistry>()
        .owner(thread_id.clone())?;
    let interrupt_context = if force {
        None
    } else {
        let snapshot = owner.snapshot()?;
        Some((
            snapshot.session_id.ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::InvalidStateTransition,
                    "Chat session is not running",
                    true,
                )
            })?,
            snapshot.active_turn_id.ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::InvalidStateTransition,
                    "Chat turn is not running",
                    true,
                )
            })?,
        ))
    };
    let command_kind = if force {
        "force_stop_session"
    } else {
        "interrupt_turn"
    };
    match claim_command_receipt(
        &pool,
        &client_command_id,
        &thread_id,
        command_kind,
        None,
        &now_timestamp()?,
    )
    .await?
    {
        CommandReceiptClaim::Replay(receipt) => return replay_driver_receipt(receipt),
        CommandReceiptClaim::Claimed(_) => {}
    }
    if force {
        let result = owner
            .stop_session(
                true,
                operation_context("ui-force-stop-session", Duration::from_secs(5)),
            )
            .await;
        return complete_driver_operation(&pool, &client_command_id, result).await;
    }
    let (session_id, turn_id) = interrupt_context.ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::InvalidStateTransition,
            "Chat provider session is not running",
            true,
        )
    })?;
    let result = owner
        .interrupt_turn(
            InterruptTurnRequest {
                command: ChatCommandContext {
                    client_command_id: client_command_id.clone(),
                    expected_thread_revision: None,
                },
                session_id,
                turn_id,
            },
            operation_context("ui-interrupt-turn", Duration::from_secs(15)),
        )
        .await;
    complete_driver_operation(&pool, &client_command_id, result).await
}

async fn require_workspace(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    working_folder_id: &ProjectWorkingFolderId,
    operation: WorkingFolderAuthorizationOperation,
) -> ChatResult<super::workspace::AuthorizedWorkingFolder> {
    let workspace = workspaces::read_workspace(pool, working_folder_id).await?;
    let scope = read_active_working_folder_scope(app).map_err(device_state_error)?;
    authorize_workspace(&workspace, &scope, operation)
}

async fn read_pending_request(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<ChatPendingRequestRead>> {
    let row = sqlx::query(
        "SELECT id, turn_id, provider_request_id, request_kind,
                safe_display_schema_version, safe_display_data,
                allowed_decisions_schema_version, allowed_decisions_data, opened_at
         FROM chat_pending_requests
         WHERE thread_id = ? AND resolution_state = 'open'
         ORDER BY opened_sequence DESC, id DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    row.map(|row| {
        Ok(ChatPendingRequestRead {
            id: row.try_get("id").map_err(persistence_error)?,
            turn_id: row.try_get("turn_id").map_err(persistence_error)?,
            provider_request_id: row
                .try_get("provider_request_id")
                .map_err(persistence_error)?,
            request_kind: row.try_get("request_kind").map_err(persistence_error)?,
            safe_display: versioned_row(&row, "safe_display_schema_version", "safe_display_data")?,
            allowed_decisions: versioned_row(
                &row,
                "allowed_decisions_schema_version",
                "allowed_decisions_data",
            )?,
            opened_at: UtcTimestamp::new(
                row.try_get::<String, _>("opened_at")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
        })
    })
    .transpose()
}

async fn read_queued_followup(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<ChatQueuedFollowupRead>> {
    let row = sqlx::query(
        "SELECT id, thread_id, text, provider_instance_id,
                model_selection_schema_version, model_selection_data,
                safety_mode, interaction_mode, attachment_ids_data,
                mentions_schema_version, mentions_data, created_at, updated_at
         FROM chat_queued_followups WHERE thread_id = ? AND state = 'queued'
         ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    row.map(|row| {
        let attachment_ids: Vec<ChatAttachmentId> = serde_json::from_str::<Vec<String>>(
            &row.try_get::<String, _>("attachment_ids_data")
                .map_err(persistence_error)?,
        )
        .map_err(json_error)?
        .into_iter()
        .map(ChatAttachmentId::new)
        .collect::<Result<_, _>>()
        .map_err(|_| corrupt_data())?;
        Ok(ChatQueuedFollowupRead {
            id: row.try_get("id").map_err(persistence_error)?,
            thread_id: ChatThreadId::new(
                row.try_get::<String, _>("thread_id")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
            text: row.try_get("text").map_err(persistence_error)?,
            provider_instance_id: ProviderInstanceId::new(
                row.try_get::<String, _>("provider_instance_id")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
            model_selection: versioned_row(
                &row,
                "model_selection_schema_version",
                "model_selection_data",
            )?,
            safety_mode: parse_safety_mode(
                &row.try_get::<String, _>("safety_mode")
                    .map_err(persistence_error)?,
            )?,
            interaction_mode: parse_interaction_mode(
                &row.try_get::<String, _>("interaction_mode")
                    .map_err(persistence_error)?,
            )?,
            attachment_ids,
            mentions: versioned_row(&row, "mentions_schema_version", "mentions_data")?,
            created_at: UtcTimestamp::new(
                row.try_get::<String, _>("created_at")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
            updated_at: UtcTimestamp::new(
                row.try_get::<String, _>("updated_at")
                    .map_err(persistence_error)?,
            )
            .map_err(|_| corrupt_data())?,
        })
    })
    .transpose()
}

async fn read_latest_event(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    event_type: &str,
) -> ChatResult<Option<CanonicalEvent>> {
    let payload: Option<String> = sqlx::query_scalar(
        "SELECT payload_data FROM chat_events
         WHERE thread_id = ? AND event_type = ?
         ORDER BY sequence DESC LIMIT 1",
    )
    .bind(thread_id.as_str())
    .bind(event_type)
    .fetch_optional(pool)
    .await
    .map_err(persistence_error)?;
    payload
        .map(|value| serde_json::from_str(&value).map_err(json_error))
        .transpose()
}

async fn read_latest_usage(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<ThreadUsageUpdatedEvent>> {
    match read_latest_event(pool, thread_id, "thread_usage_updated").await? {
        Some(CanonicalEvent::ThreadUsageUpdated(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(corrupt_data()),
    }
}

async fn read_latest_account_status(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<AccountStatusEvent>> {
    match read_latest_event(pool, thread_id, "account_status").await? {
        Some(CanonicalEvent::AccountStatus(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(corrupt_data()),
    }
}

async fn read_latest_rate_limit_status(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<Option<RateLimitStatusEvent>> {
    match read_latest_event(pool, thread_id, "rate_limit_status").await? {
        Some(CanonicalEvent::RateLimitStatus(value)) => Ok(Some(value)),
        None => Ok(None),
        _ => Err(corrupt_data()),
    }
}

async fn automatic_compaction_reported(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
) -> ChatResult<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM chat_events
            WHERE thread_id = ?
              AND event_type IN ('item_started', 'item_updated', 'item_completed')
              AND json_extract(payload_data, '$.payload.kind') = 'context_compaction'
         )",
    )
    .bind(thread_id.as_str())
    .fetch_one(pool)
    .await
    .map_err(persistence_error)
}

fn scan_paths(
    root: &Path,
    include_ignored: bool,
    visible: &BTreeSet<String>,
) -> ChatResult<Vec<ProjectWorkingFolderPathRead>> {
    let mut result = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .map_err(workspace_io_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(workspace_io_error)?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if result.len() >= MAX_PATH_SCAN {
                return Ok(result);
            }
            let path = entry.path();
            let file_type = entry.file_type().map_err(workspace_io_error)?;
            if file_type.is_symlink() || entry.file_name() == ".git" {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| workspace_io_error(()))?
                .to_string_lossy()
                .replace('\\', "/");
            if workspace_mention_is_safety_excluded(&relative) {
                continue;
            }
            let ignored = !visible.is_empty()
                && if file_type.is_dir() {
                    let prefix = format!("{relative}/");
                    !visible.iter().any(|entry| entry.starts_with(&prefix))
                } else {
                    !visible.contains(&relative)
                };
            if include_ignored || !ignored {
                result.push(ProjectWorkingFolderPathRead {
                    display_name: entry.file_name().to_string_lossy().into_owned(),
                    relative_path: relative,
                    kind: if file_type.is_dir() {
                        "directory"
                    } else {
                        "file"
                    }
                    .to_string(),
                    ignored,
                });
            }
            if file_type.is_dir() && (include_ignored || !ignored) {
                pending.push(path);
            }
        }
    }
    Ok(result)
}

pub(crate) fn workspace_mention_is_safety_excluded(relative_path: &str) -> bool {
    let first = relative_path.split('/').next().unwrap_or(relative_path);
    matches!(first, ".git" | ".ssh" | ".gnupg" | ".aws")
        || relative_path == ".env"
        || (relative_path.starts_with(".env.") && relative_path != ".env.example")
        || relative_path == ".codex/auth.json"
        || relative_path == ".claude/.credentials.json"
}

fn git_visible_paths(root: &Path) -> ChatResult<BTreeSet<String>> {
    if !root.join(".git").exists() {
        return Ok(BTreeSet::new());
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-co", "--exclude-standard", "-z"])
        .output()
        .map_err(workspace_io_error)?;
    if !output.status.success() || output.stdout.len() > 16 * 1024 * 1024 {
        return Err(workspace_io_error(()));
    }
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .filter_map(|path| String::from_utf8(path.to_vec()).ok())
        .collect())
}

fn read_skill_entries(home: &Path, stale: bool) -> ChatResult<Vec<ChatPromptCatalogEntry>> {
    let root = home.join("skills");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut pending = vec![(root, 0_u8)];
    let mut seen = BTreeSet::new();
    let mut scanned = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        if scanned >= 2_000 || entries.len() >= 500 {
            break;
        }
        for child in fs::read_dir(directory)
            .map_err(workspace_io_error)?
            .take(2_000_usize.saturating_sub(scanned))
        {
            scanned += 1;
            let child = child.map_err(workspace_io_error)?;
            if child.file_type().map_err(workspace_io_error)?.is_symlink()
                || !child.file_type().map_err(workspace_io_error)?.is_dir()
            {
                continue;
            }
            let skill_file = child.path().join("SKILL.md");
            if skill_file.is_file() {
                let name = child.file_name().to_string_lossy().into_owned();
                if name.is_empty()
                    || name.len() > 200
                    || name.chars().any(char::is_control)
                    || !seen.insert(name.clone())
                {
                    continue;
                }
                entries.push(ChatPromptCatalogEntry {
                    value: format!("${name}"),
                    label: name,
                    description: read_skill_description(&skill_file),
                    kind: "skill".to_string(),
                    stale,
                });
            } else if depth < 3 && entries.len() < 500 {
                pending.push((child.path(), depth + 1));
            }
        }
    }
    entries.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(entries)
}

fn read_skill_description(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.by_ref().take(16 * 1024).read_to_end(&mut bytes).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    text.lines()
        .take(80)
        .find_map(|line| line.trim().strip_prefix("description:"))
        .map(str::trim)
        .map(|value| value.trim_matches(['\'', '"']))
        .filter(|value| !value.is_empty() && value.len() <= 1_000)
        .map(str::to_string)
}

fn provider_skill_home(configuration: &super::models::ProviderInstanceConfig) -> Option<PathBuf> {
    if let Some(home) = configuration
        .provider_home
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(PathBuf::from(home));
    }
    if configuration.family_id.as_str() != "codex" {
        return None;
    }
    if let Some(home) = std::env::var_os("CODEX_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(home));
    }
    #[cfg(windows)]
    let home_names = ["USERPROFILE", "HOME"];
    #[cfg(not(windows))]
    let home_names = ["HOME", "USERPROFILE"];
    home_names
        .iter()
        .find_map(|name| std::env::var_os(name).filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .map(|home| home.join(".codex"))
}

fn wire_safety_mode(value: super::models::SafetyMode) -> &'static str {
    match value {
        super::models::SafetyMode::AskForApproval => "ask_for_approval",
        super::models::SafetyMode::ApproveForMe => "approve_for_me",
        super::models::SafetyMode::FullAccess => "full_access",
        super::models::SafetyMode::Custom => "custom",
    }
}

fn parse_safety_mode(value: &str) -> ChatResult<super::models::SafetyMode> {
    match value {
        "ask_for_approval" => Ok(super::models::SafetyMode::AskForApproval),
        "approve_for_me" => Ok(super::models::SafetyMode::ApproveForMe),
        "full_access" => Ok(super::models::SafetyMode::FullAccess),
        "custom" => Ok(super::models::SafetyMode::Custom),
        _ => Err(corrupt_data()),
    }
}

fn wire_interaction_mode(value: super::models::InteractionMode) -> &'static str {
    match value {
        super::models::InteractionMode::Build => "build",
        super::models::InteractionMode::Plan => "plan",
    }
}

fn parse_interaction_mode(value: &str) -> ChatResult<super::models::InteractionMode> {
    match value {
        "build" => Ok(super::models::InteractionMode::Build),
        "plan" => Ok(super::models::InteractionMode::Plan),
        _ => Err(corrupt_data()),
    }
}

fn managed_attachment_path(app: &tauri::AppHandle, relative_path: &str) -> ChatResult<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute()
        || !relative_path.starts_with("assets/chat/attachments/")
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(ChatError::validation(
            "attachmentPath",
            "Managed Chat attachment path is invalid",
        ));
    }
    Ok(vault::active_vault_path(app)
        .map_err(vault_error)?
        .join(relative))
}

fn fuzzy_path_matches(path: &str, query: &str) -> bool {
    if query.is_empty() || path.to_lowercase().contains(query) {
        return true;
    }
    let mut query_chars = query.chars();
    let mut wanted = query_chars.next();
    for character in path.chars().flat_map(char::to_lowercase) {
        if wanted == Some(character) {
            wanted = query_chars.next();
        }
    }
    wanted.is_none()
}

fn path_rank(path: &str, query: &str) -> (u8, usize, usize) {
    let normalized = path.to_lowercase();
    let name = normalized.rsplit('/').next().unwrap_or(&normalized);
    if name == query {
        (0, path.len(), 0)
    } else if name.starts_with(query) {
        (1, path.len(), 0)
    } else if let Some(index) = normalized.find(query) {
        (2, path.len(), index)
    } else {
        (3, path.len(), usize::MAX)
    }
}

fn parse_offset(cursor: Option<&str>) -> ChatResult<usize> {
    cursor
        .unwrap_or("0")
        .parse::<usize>()
        .map_err(|_| ChatError::validation("cursor", "Workspace search cursor is invalid"))
}

fn versioned_row(
    row: &sqlx::sqlite::SqliteRow,
    version: &str,
    data: &str,
) -> ChatResult<VersionedJson> {
    Ok(VersionedJson {
        schema_version: u32::try_from(row.try_get::<i64, _>(version).map_err(persistence_error)?)
            .map_err(|_| corrupt_data())?,
        value: serde_json::from_str(&row.try_get::<String, _>(data).map_err(persistence_error)?)
            .map_err(json_error)?,
    })
}

fn replay_driver_receipt(receipt: CommandReceiptRead) -> ChatResult<DriverOperationReceipt> {
    match receipt.state {
        CommandReceiptState::Accepted => Err(ChatError::new(
            ChatErrorCode::Busy,
            "This Chat command is still being processed",
            true,
        )),
        CommandReceiptState::Completed => {
            serde_json::from_value(receipt.result.ok_or_else(corrupt_data)?.value)
                .map_err(json_error)
        }
        CommandReceiptState::Failed => Err(serde_json::from_value(
            receipt.error.ok_or_else(corrupt_data)?.value,
        )
        .map_err(json_error)?),
    }
}

async fn complete_driver_operation(
    pool: &SqlitePool,
    command_id: &ChatCommandId,
    result: ChatResult<DriverOperationReceipt>,
) -> ChatResult<DriverOperationReceipt> {
    let now = now_timestamp()?;
    match result {
        Ok(receipt) => {
            let value = VersionedJson {
                schema_version: 1,
                value: serde_json::to_value(&receipt).map_err(json_error)?,
            };
            complete_command_receipt(
                pool,
                command_id,
                CommandReceiptState::Completed,
                Some(&value),
                None,
                &now,
            )
            .await?;
            Ok(receipt)
        }
        Err(error) => {
            let value = VersionedJson {
                schema_version: 1,
                value: serde_json::to_value(&error).map_err(json_error)?,
            };
            complete_command_receipt(
                pool,
                command_id,
                CommandReceiptState::Failed,
                None,
                Some(&value),
                &now,
            )
            .await?;
            Err(error)
        }
    }
}

fn file_path_to_path(value: FilePath) -> ChatResult<PathBuf> {
    value
        .into_path()
        .map_err(|_| ChatError::validation("image", "Selected image is not local"))
}

fn operation_context(
    operation_id: &str,
    timeout: Duration,
) -> super::providers::DriverOperationContext {
    super::providers::DriverOperationContext {
        operation_id: operation_id.to_string(),
        deadline: Instant::now() + timeout,
        cancellation: super::providers::DriverCancellation::default(),
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

fn device_state_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat device state could not be updated",
        true,
    )
}
fn vault_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat attachment folder is unavailable",
        true,
    )
}
fn attachment_io_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat image could not be read",
        true,
    )
}
fn workspace_io_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Permission,
        "Project working-folder paths could not be read",
        true,
    )
}
fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat interaction persistence failed",
        true,
    )
}
fn json_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat interaction data is invalid",
        false,
    )
}
fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat interaction is invalid",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_search_fuzzy_matching_and_safety_exclusions_are_bounded() {
        assert!(fuzzy_path_matches("src/calendar/view.ts", "scv"));
        assert!(
            path_rank("src/calendar", "calendar") < path_rank("docs/my-calendar.md", "calendar")
        );
        assert!(workspace_mention_is_safety_excluded(".git/config"));
        assert!(workspace_mention_is_safety_excluded(".env.local"));
        assert!(!workspace_mention_is_safety_excluded(".env.example"));
        assert!(!workspace_mention_is_safety_excluded("src/config.ts"));
        assert!(parse_offset(Some("not-a-number")).is_err());
    }

    #[test]
    fn provider_skill_catalog_reads_nested_bounded_metadata() {
        let root = std::env::temp_dir().join(format!(
            "ganbaru-chat-skill-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let skill = root.join("skills/.system/example");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: example\ndescription: A provider-scoped example\n---\n",
        )
        .unwrap();
        let entries = read_skill_entries(&root, false).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "$example");
        assert_eq!(
            entries[0].description.as_deref(),
            Some("A provider-scoped example")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
