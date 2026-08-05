//! Shared validation, wire conversion, and identifier helpers.

use super::super::channel_commands::{identifier_error, persistence_error};
use super::super::coordination::contracts::*;
use super::super::models::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const MAX_MESSAGE_BYTES: usize = 128 * 1024;

pub(super) fn has_thread_eligible_mention(request: &PostChatMessageCommand) -> bool {
    !request.participant_mentions.is_empty()
}

pub(super) fn validate_policy(
    app: &tauri::AppHandle,
    policy: &ChatTeammatePolicyInput,
) -> ChatResult<()> {
    if policy.provider_managed_model == policy.model_id.is_some() {
        return Err(ChatError::validation(
            "modelId",
            "Choose either provider-managed model selection or one explicit model",
        ));
    }
    if let Some(effort) = policy.effort.as_deref() {
        if !matches!(
            effort,
            "none" | "minimal" | "low" | "medium" | "high" | "xhigh"
        ) {
            return Err(ChatError::validation(
                "effort",
                "Teammate effort is invalid",
            ));
        }
    }
    if let Some(speed) = policy.speed.as_deref() {
        if !matches!(speed, "standard" | "fast") {
            return Err(ChatError::validation("speed", "Teammate speed is invalid"));
        }
    }
    super::super::settings_commands::read_provider(app, &policy.provider_instance_id)?;
    Ok(())
}

pub(super) fn validate_message_request(request: &PostChatMessageCommand) -> ChatResult<()> {
    let markdown = request.normalized_markdown.trim();
    if markdown.is_empty()
        && request.attachment_ids.is_empty()
        && request.resource_references.is_empty()
    {
        return Err(ChatError::validation(
            "normalizedMarkdown",
            "Message content is required",
        ));
    }
    if request.normalized_markdown.len() > MAX_MESSAGE_BYTES {
        return Err(ChatError::validation(
            "normalizedMarkdown",
            "Message content exceeds the 128 KiB limit",
        ));
    }
    json_object(&request.rich_content, "richContent")?;
    let mut ranges = BTreeSet::new();
    for mention in &request.participant_mentions {
        if mention.start_offset >= mention.end_offset
            || mention.end_offset > request.normalized_markdown.len() as u64
            || mention.label_snapshot.trim().is_empty()
            || mention.label_snapshot.chars().count() > 160
            || !ranges.insert((mention.start_offset, mention.end_offset))
        {
            return Err(ChatError::validation(
                "participantMentions",
                "Participant mention ranges are invalid",
            ));
        }
    }
    let mut attachments = BTreeSet::new();
    if request
        .attachment_ids
        .iter()
        .any(|id| !attachments.insert(id.as_str()))
    {
        return Err(ChatError::validation(
            "attachmentIds",
            "Attachment IDs must be distinct",
        ));
    }
    for resource in &request.resource_references {
        if !matches!(resource.kind.as_str(), "file" | "folder")
            || !safe_relative_path(&resource.relative_path)
            || resource.display_label.trim().is_empty()
        {
            return Err(ChatError::validation(
                "resourceReferences",
                "A file or folder reference is invalid",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_display_name(value: &str) -> ChatResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 160 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            "displayName",
            "Teammate name must contain 1 to 160 characters",
        ));
    }
    Ok(value.to_string())
}

pub(super) fn validate_handle(value: &str) -> ChatResult<String> {
    let value = value.trim().trim_start_matches('@').to_lowercase();
    if value.is_empty()
        || value.chars().count() > 80
        || value.chars().any(|character| {
            !character.is_ascii_alphanumeric() && !matches!(character, '.' | '_' | '-')
        })
    {
        return Err(ChatError::validation(
            "handle",
            "Teammate handle can use lowercase letters, numbers, periods, underscores, and hyphens",
        ));
    }
    Ok(value)
}

pub(super) fn validate_profile_text(value: &str, maximum: usize, field: &str) -> ChatResult<()> {
    if value.len() > maximum {
        return Err(ChatError::validation(
            field,
            "Teammate profile text is too long",
        ));
    }
    Ok(())
}

pub(super) fn json_object(value: &VersionedJson, field: &str) -> ChatResult<String> {
    if !value.value.is_object() {
        return Err(ChatError::validation(field, "Value must be a JSON object"));
    }
    serde_json::to_string(&value.value).map_err(serialization_error)
}

pub(super) fn parse_json(data: String) -> ChatResult<Value> {
    serde_json::from_str(&data).map_err(serialization_error)
}

pub(super) fn parse_cursor(cursor: Option<&str>) -> ChatResult<Option<i64>> {
    cursor
        .map(|value| {
            value
                .parse::<i64>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| ChatError::validation("cursor", "Chat page cursor is invalid"))
        })
        .transpose()
}

pub(super) fn normalized_fts_query(query: &str) -> ChatResult<String> {
    let terms = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .take(20)
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>();
    if terms.is_empty() || query.len() > 500 {
        return Err(ChatError::validation(
            "query",
            "Message search query is invalid",
        ));
    }
    Ok(terms.join(" AND "))
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4_096
        && !path.starts_with('/')
        && !path.starts_with("../")
        && !path.contains("/../")
        && !path.ends_with("/..")
        && !path.contains('\\')
}

pub(super) fn conversation_item_id() -> ChatResult<ChatConversationItemId> {
    ChatConversationItemId::new(new_id("conversation-item")).map_err(identifier_error)
}

pub(super) fn message_revision_id() -> ChatResult<ChatMessageRevisionId> {
    ChatMessageRevisionId::new(new_id("message-revision")).map_err(identifier_error)
}

pub(super) fn reply_thread_id() -> ChatResult<ChatReplyThreadId> {
    ChatReplyThreadId::new(new_id("reply-thread")).map_err(identifier_error)
}

pub(super) fn work_assignment_id() -> ChatResult<ChatWorkAssignmentId> {
    ChatWorkAssignmentId::new(new_id("assignment")).map_err(identifier_error)
}

pub(super) fn parse_participant_kind(value: &str) -> ChatResult<ChatParticipantKind> {
    match value {
        "local_user" => Ok(ChatParticipantKind::LocalUser),
        "ai_teammate" => Ok(ChatParticipantKind::AiTeammate),
        "human" => Ok(ChatParticipantKind::Human),
        _ => Err(stored_value_error("participant kind")),
    }
}

pub(super) fn parse_configuration_state(value: &str) -> ChatResult<ChatTeammateConfigurationState> {
    match value {
        "healthy" => Ok(ChatTeammateConfigurationState::Healthy),
        "needs_setup" => Ok(ChatTeammateConfigurationState::NeedsSetup),
        "provider_unavailable" => Ok(ChatTeammateConfigurationState::ProviderUnavailable),
        "folder_access_missing" => Ok(ChatTeammateConfigurationState::FolderAccessMissing),
        _ => Err(stored_value_error("teammate configuration state")),
    }
}

pub(super) fn parse_approval_policy(value: &str) -> ChatResult<ChatApprovalPolicy> {
    match value {
        "ask_for_approval" => Ok(ChatApprovalPolicy::AskForApproval),
        "approve_for_me" => Ok(ChatApprovalPolicy::ApproveForMe),
        "full_access" => Ok(ChatApprovalPolicy::FullAccess),
        "custom" => Ok(ChatApprovalPolicy::Custom),
        _ => Err(stored_value_error("approval policy")),
    }
}

pub(super) fn parse_work_state(value: &str) -> ChatResult<ChatWorkAssignmentState> {
    match value {
        "queued" => Ok(ChatWorkAssignmentState::Queued),
        "working" => Ok(ChatWorkAssignmentState::Working),
        "waiting_for_answer" => Ok(ChatWorkAssignmentState::WaitingForAnswer),
        "waiting_for_approval" => Ok(ChatWorkAssignmentState::WaitingForApproval),
        "ready_for_review" => Ok(ChatWorkAssignmentState::ReadyForReview),
        "completed" => Ok(ChatWorkAssignmentState::Completed),
        "failed" => Ok(ChatWorkAssignmentState::Failed),
        "cancelled" => Ok(ChatWorkAssignmentState::Cancelled),
        _ => Err(stored_value_error("assignment state")),
    }
}

pub(super) fn wire_participant_kind(value: ChatParticipantKind) -> &'static str {
    match value {
        ChatParticipantKind::LocalUser => "local_user",
        ChatParticipantKind::AiTeammate => "ai_teammate",
        ChatParticipantKind::Human => "human",
    }
}

pub(super) fn wire_approval_policy(value: ChatApprovalPolicy) -> &'static str {
    match value {
        ChatApprovalPolicy::AskForApproval => "ask_for_approval",
        ChatApprovalPolicy::ApproveForMe => "approve_for_me",
        ChatApprovalPolicy::FullAccess => "full_access",
        ChatApprovalPolicy::Custom => "custom",
    }
}

pub(super) fn wire_work_state(value: ChatWorkAssignmentState) -> &'static str {
    match value {
        ChatWorkAssignmentState::Queued => "queued",
        ChatWorkAssignmentState::Working => "working",
        ChatWorkAssignmentState::WaitingForAnswer => "waiting_for_answer",
        ChatWorkAssignmentState::WaitingForApproval => "waiting_for_approval",
        ChatWorkAssignmentState::ReadyForReview => "ready_for_review",
        ChatWorkAssignmentState::Completed => "completed",
        ChatWorkAssignmentState::Failed => "failed",
        ChatWorkAssignmentState::Cancelled => "cancelled",
    }
}

pub(super) fn u32_value(value: i64) -> ChatResult<u32> {
    u32::try_from(value).map_err(|_| stored_value_error("schema version"))
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn truncate_utf8(value: &str, maximum: usize) -> &str {
    if value.len() <= maximum {
        return value;
    }
    let mut boundary = maximum;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}

fn stored_value_error(label: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        format!("Stored Chat {label} is invalid"),
        false,
    )
}

pub(super) fn new_id(prefix: &str) -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}:{nanos:032x}{sequence:016x}")
}

pub(super) fn serialization_error<E>(_error: E) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat JSON data is invalid",
        false,
    )
}

pub(super) fn map_teammate_write_error(error: sqlx::Error) -> ChatError {
    let detail = error.to_string();
    if detail.contains("chat_participants.normalized_handle") {
        ChatError::validation("handle", "This teammate handle is already in use")
    } else {
        persistence_error(error)
    }
}

pub(super) fn map_command_receipt_error(error: sqlx::Error) -> ChatError {
    if error
        .to_string()
        .contains("chat_organizational_command_receipts.client_command_id")
    {
        ChatError::new(
            ChatErrorCode::Conflict,
            "This message command was already accepted",
            true,
        )
    } else {
        persistence_error(error)
    }
}
