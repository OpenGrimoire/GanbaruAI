//! Typed builders and bounded decoders for Claude Code's SDK JSONL protocol.

use crate::chat::models::*;
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const MINIMUM_CLAUDE_VERSION: ClaudeVersion = ClaudeVersion::new(2, 1, 170);
const MAX_PROMPT_BYTES: usize = 4 * 1024 * 1024;
const MAX_DEVELOPER_INSTRUCTIONS_BYTES: usize = 64 * 1024;
const MAX_TEXT_ATTACHMENT_BYTES: usize = 128 * 1024;
const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_MODEL_ID_BYTES: usize = 256;
const MAX_MODEL_OPTIONS: usize = 32;
const MAX_LAUNCH_ARGUMENTS: usize = 128;
const MAX_LAUNCH_ARGUMENT_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClaudeVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl ClaudeVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for ClaudeVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClaudeInitializeResponse {
    #[serde(default)]
    pub commands: Vec<ClaudeCommand>,
    #[serde(default)]
    pub models: Vec<ClaudeModel>,
    pub account: Option<ClaudeAccount>,
    #[serde(default)]
    pub output_style: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCommand {
    pub name: String,
    pub description: Option<String>,
    pub argument_hint: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeModel {
    pub value: String,
    pub resolved_model: Option<String>,
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub supports_effort: bool,
    #[serde(default)]
    pub supported_effort_levels: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeAccount {
    pub email: Option<String>,
    pub organization: Option<String>,
    pub subscription_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeResumeCursor {
    pub session_uuid: String,
    pub last_assistant_uuid: Option<String>,
    pub turn_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClaudeCompatibilityEvidence {
    pub version: ClaudeVersion,
    pub partial_messages: bool,
    pub session_uuid: bool,
    pub resume: bool,
    pub approval_callback: bool,
    pub structured_questions: bool,
    pub interrupt: bool,
    pub model_change: bool,
    pub native_plan: bool,
}

impl ClaudeCompatibilityEvidence {
    pub fn supports_native_transport(self) -> bool {
        self.version >= MINIMUM_CLAUDE_VERSION
            && self.partial_messages
            && self.session_uuid
            && self.resume
            && self.approval_callback
            && self.structured_questions
            && self.interrupt
            && self.model_change
            && self.native_plan
    }
}

pub fn parse_version(value: &str) -> ChatResult<ClaudeVersion> {
    let token = value
        .split_whitespace()
        .find(|token| {
            token
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_digit())
        })
        .ok_or_else(unsupported_version)?;
    let core = token.split_once('-').map_or(token, |(core, _)| core);
    let mut parts = core.split('.');
    let major = parse_version_part(parts.next())?;
    let minor = parse_version_part(parts.next())?;
    let patch = parse_version_part(parts.next())?;
    if parts.next().is_some() {
        return Err(unsupported_version());
    }
    Ok(ClaudeVersion::new(major, minor, patch))
}

pub fn ensure_supported_version(value: ClaudeVersion) -> ChatResult<()> {
    if value < MINIMUM_CLAUDE_VERSION {
        return Err(ChatError::new(
            ChatErrorCode::UnsupportedVersion,
            format!(
                "Claude Code {value} lacks the complete interactive SDK protocol; version {MINIMUM_CLAUDE_VERSION} or newer is required"
            ),
            true,
        ));
    }
    Ok(())
}

fn parse_version_part(value: Option<&str>) -> ChatResult<u64> {
    value
        .and_then(|part| part.parse::<u64>().ok())
        .ok_or_else(unsupported_version)
}

#[derive(Clone, Copy, Debug)]
pub struct ClaudeLaunchOptions<'a> {
    pub fresh_session_uuid: Option<&'a str>,
    pub resume_session_uuid: Option<&'a str>,
    pub last_assistant_uuid: Option<&'a str>,
    pub model: Option<&'a ModelId>,
    pub effort: Option<&'a str>,
    pub fast_mode: Option<bool>,
    pub modes: TurnModeSnapshot,
}

pub fn launch_arguments(
    prefix: Vec<String>,
    configured: &[String],
    options: ClaudeLaunchOptions<'_>,
) -> ChatResult<Vec<String>> {
    validate_launch_arguments(configured)?;
    let mut arguments = prefix;
    arguments.extend([
        "-p".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--input-format".to_string(),
        "stream-json".to_string(),
        "--permission-prompt-tool".to_string(),
        "stdio".to_string(),
        "--include-partial-messages".to_string(),
        "--replay-user-messages".to_string(),
    ]);
    let permission_mode = permission_mode(options.modes);
    arguments.push("--permission-mode".to_string());
    arguments.push(permission_mode.to_string());
    if permission_mode == "bypassPermissions" {
        arguments.push("--allow-dangerously-skip-permissions".to_string());
    }
    if options.fresh_session_uuid.is_some() && options.resume_session_uuid.is_some() {
        return Err(ChatError::validation(
            "sessionUuid",
            "Claude session cannot be both fresh and resumed",
        ));
    }
    if let Some(session_uuid) = options.fresh_session_uuid {
        validate_uuid(session_uuid, "sessionUuid")?;
        arguments.push("--session-id".to_string());
        arguments.push(session_uuid.to_string());
    }
    if let Some(session_uuid) = options.resume_session_uuid {
        validate_uuid(session_uuid, "sessionUuid")?;
        arguments.push("--resume".to_string());
        arguments.push(session_uuid.to_string());
    }
    if let Some(last_assistant_uuid) = options.last_assistant_uuid {
        validate_uuid(last_assistant_uuid, "lastAssistantUuid")?;
        arguments.push("--resume-session-at".to_string());
        arguments.push(last_assistant_uuid.to_string());
    }
    if let Some(model) = options.model {
        validate_model_id(model.as_str())?;
        arguments.push("--model".to_string());
        arguments.push(model.as_str().to_string());
    }
    if let Some(effort) = options.effort {
        if !matches!(effort, "low" | "medium" | "high" | "xhigh" | "max") {
            return Err(ChatError::validation(
                "modelOptions",
                "Claude effort value is unsupported",
            ));
        }
        arguments.push("--effort".to_string());
        arguments.push(effort.to_string());
    }
    if let Some(fast_mode) = options.fast_mode {
        arguments.push("--settings".to_string());
        arguments.push(json!({ "fastMode": fast_mode }).to_string());
    }
    arguments.extend(configured.iter().cloned());
    Ok(arguments)
}

pub fn permission_mode(modes: TurnModeSnapshot) -> &'static str {
    if modes.interaction_mode == InteractionMode::Plan {
        return "plan";
    }
    match modes.safety_mode {
        SafetyMode::Supervised => "default",
        SafetyMode::AutoAcceptEdits => "acceptEdits",
        SafetyMode::FullAccess => "bypassPermissions",
    }
}

pub fn initialize_request() -> Value {
    json!({
        "type": "control_request",
        "request_id": "ganbaru-initialize",
        "request": {
            "subtype": "initialize",
            "supportedDialogKinds": [],
        },
    })
}

pub fn control_request(request_id: &str, request: Value) -> ChatResult<Value> {
    validate_protocol_id(request_id, "requestId")?;
    if !request.is_object() {
        return Err(protocol_error("control request body"));
    }
    Ok(json!({
        "type": "control_request",
        "request_id": request_id,
        "request": request,
    }))
}

pub fn control_success(request_id: &str, response: Value) -> ChatResult<Value> {
    validate_protocol_id(request_id, "requestId")?;
    if !response.is_object() {
        return Err(protocol_error("control response body"));
    }
    Ok(json!({
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": request_id,
            "response": response,
        },
    }))
}

pub fn control_error(request_id: &str, message: &str) -> ChatResult<Value> {
    validate_protocol_id(request_id, "requestId")?;
    Ok(json!({
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": request_id,
            "error": bounded_text(message, 2_048),
        },
    }))
}

pub fn decode_initialize(value: Value) -> ChatResult<ClaudeInitializeResponse> {
    serde_json::from_value(value).map_err(|_| protocol_error("initialize response"))
}

pub fn build_user_message(request: &SendTurnRequest) -> ChatResult<Value> {
    if request.prompt.len() > MAX_PROMPT_BYTES || request.prompt.contains('\0') {
        return Err(ChatError::validation(
            "prompt",
            "Claude prompt exceeds the supported limit",
        ));
    }
    if request
        .developer_instructions
        .as_deref()
        .is_some_and(|value| value.len() > MAX_DEVELOPER_INSTRUCTIONS_BYTES || value.contains('\0'))
    {
        return Err(ChatError::validation(
            "developerInstructions",
            "Claude developer instructions exceed the supported limit",
        ));
    }
    validate_model_options(&request.model_options)?;
    let mut content = Vec::new();
    let mut prompt = request.prompt.clone();
    if !request.mentions.is_empty() {
        let paths = request
            .mentions
            .iter()
            .map(|mention| format!("@{}", mention.relative_path))
            .collect::<Vec<_>>()
            .join("\n");
        prompt.push_str("\n\nReferenced workspace paths:\n");
        prompt.push_str(&paths);
    }
    if let Some(instructions) = request.developer_instructions.as_deref() {
        prompt.push_str("\n\nAdditional Ganbaru instructions:\n");
        prompt.push_str(instructions);
    }
    if !prompt.is_empty() {
        content.push(json!({ "type": "text", "text": prompt }));
    }
    for attachment in &request.attachments {
        match attachment.kind.as_str() {
            "image" => content.push(image_content(attachment)?),
            "text_snippet" => {
                let text = attachment.text_content.as_deref().ok_or_else(|| {
                    ChatError::validation("attachments", "Claude text context is unavailable")
                })?;
                if text.len() > MAX_TEXT_ATTACHMENT_BYTES || text.contains('\0') {
                    return Err(ChatError::validation(
                        "attachments",
                        "Claude text context exceeds the supported limit",
                    ));
                }
                content.push(json!({ "type": "text", "text": text }));
            }
            _ => {
                return Err(ChatError::unsupported(
                    "Claude prompt attachment kind is unsupported",
                ));
            }
        }
    }
    if content.is_empty() {
        return Err(ChatError::validation(
            "prompt",
            "Claude turn requires text or an image",
        ));
    }
    Ok(json!({
        "type": "user",
        "message": { "role": "user", "content": content },
        "parent_tool_use_id": Value::Null,
        "session_id": "",
    }))
}

fn image_content(attachment: &PromptAttachmentReference) -> ChatResult<Value> {
    let path = attachment.local_path.as_deref().ok_or_else(|| {
        ChatError::validation("attachments", "Claude image attachment is unavailable")
    })?;
    let path = Path::new(path);
    let metadata = fs::metadata(path).map_err(|_| {
        ChatError::validation("attachments", "Claude image attachment is unavailable")
    })?;
    if !path.is_absolute() || !metadata.is_file() || metadata.len() > MAX_IMAGE_BYTES {
        return Err(ChatError::validation(
            "attachments",
            "Claude image attachment is invalid or oversized",
        ));
    }
    let mime = attachment
        .mime_type
        .as_deref()
        .filter(|value| {
            matches!(
                *value,
                "image/png" | "image/jpeg" | "image/gif" | "image/webp"
            )
        })
        .ok_or_else(|| ChatError::validation("attachments", "Claude image type is unsupported"))?;
    let bytes = fs::read(path).map_err(|_| {
        ChatError::validation("attachments", "Claude image attachment could not be read")
    })?;
    Ok(json!({
        "type": "image",
        "source": {
            "type": "base64",
            "media_type": mime,
            "data": general_purpose::STANDARD.encode(bytes),
        },
    }))
}

pub fn parse_resume_cursor(value: &VersionedJson) -> ChatResult<ClaudeResumeCursor> {
    if value.schema_version != 1 {
        return Err(ChatError::validation(
            "resumeCursor.schemaVersion",
            "Claude resume cursor schema is unsupported",
        ));
    }
    let object = value
        .value
        .as_object()
        .ok_or_else(|| ChatError::validation("resumeCursor", "Claude resume cursor is invalid"))?;
    let session_uuid = required_text(object, "sessionUuid")?.to_string();
    validate_uuid(&session_uuid, "resumeCursor.sessionUuid")?;
    let last_assistant_uuid = object
        .get("lastAssistantUuid")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(uuid) = last_assistant_uuid.as_deref() {
        validate_uuid(uuid, "resumeCursor.lastAssistantUuid")?;
    }
    let turn_count = object.get("turnCount").and_then(Value::as_u64).unwrap_or(0);
    Ok(ClaudeResumeCursor {
        session_uuid,
        last_assistant_uuid,
        turn_count,
    })
}

pub fn resume_cursor(cursor: &ClaudeResumeCursor) -> VersionedJson {
    VersionedJson {
        schema_version: 1,
        value: json!({
            "sessionUuid": cursor.session_uuid,
            "lastAssistantUuid": cursor.last_assistant_uuid,
            "turnCount": cursor.turn_count,
        }),
    }
}

pub fn provider_models(
    models: Vec<ClaudeModel>,
    custom_ids: &[String],
    custom_labels: &BTreeMap<String, String>,
) -> ChatResult<Vec<ProviderModel>> {
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for model in models {
        validate_model_id(&model.value)?;
        if !seen.insert(model.value.clone()) {
            continue;
        }
        let display_name = claude_model_display_name(&model);
        let options = claude_model_options(
            model.supports_effort,
            &model.supported_effort_levels,
            claude_model_supports_fast_mode(&model),
        );
        output.push(ProviderModel {
            id: ModelId::new(model.value.clone()).map_err(|_| protocol_error("model ID"))?,
            display_name: if display_name.is_empty() {
                model.value
            } else {
                display_name
            },
            description: model.description.filter(|value| !value.trim().is_empty()),
            context_limit: None,
            availability: ModelAvailability::Available,
            capabilities: claude_model_capabilities(),
            options,
            custom: false,
        });
    }
    for id in custom_ids {
        if !seen.insert(id.clone()) {
            continue;
        }
        validate_model_id(id)?;
        let display_name = custom_labels.get(id).cloned().unwrap_or_else(|| id.clone());
        let supports_fast_mode = [id.as_str(), display_name.as_str()]
            .into_iter()
            .any(claude_identity_supports_fast_mode);
        output.push(ProviderModel {
            id: ModelId::new(id.clone()).map_err(|_| protocol_error("custom model ID"))?,
            display_name,
            description: Some("Custom Claude model ID".to_string()),
            context_limit: None,
            availability: ModelAvailability::Unknown,
            capabilities: claude_model_capabilities(),
            options: claude_model_options(
                true,
                &["low", "medium", "high", "xhigh", "max"].map(str::to_string),
                supports_fast_mode,
            ),
            custom: true,
        });
    }
    Ok(output)
}

fn claude_model_capabilities() -> Vec<ProviderCapability> {
    vec![
        ProviderCapability::NativeResume,
        ProviderCapability::NativePlan,
        ProviderCapability::DynamicModelChange,
        ProviderCapability::Images,
        ProviderCapability::FileReferences,
        ProviderCapability::Approvals,
        ProviderCapability::StructuredQuestions,
        ProviderCapability::ReasoningSummaries,
        ProviderCapability::StructuredPlans,
        ProviderCapability::ContextUsage,
        ProviderCapability::CostReporting,
        ProviderCapability::TaskActivity,
    ]
}

fn claude_model_display_name(model: &ClaudeModel) -> String {
    let display_name = model.display_name.trim();
    if display_name.is_empty() {
        return String::new();
    }
    if model.value == "default" {
        if let Some(resolved) = model
            .resolved_model
            .as_deref()
            .and_then(resolved_claude_model_label)
        {
            return resolved;
        }
    }
    if display_name
        .chars()
        .any(|character| character.is_ascii_digit())
    {
        return display_name.to_string();
    }
    if let Some(versioned_name) = model
        .description
        .as_deref()
        .and_then(|description| description.split('·').next())
        .map(str::trim)
        .filter(|candidate| {
            candidate.len() > display_name.len()
                && candidate
                    .get(..display_name.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(display_name))
                && candidate
                    .get(display_name.len()..)
                    .is_some_and(|suffix| suffix.chars().next().is_some_and(char::is_whitespace))
        })
    {
        return versioned_name.to_string();
    }
    display_name.to_string()
}

fn resolved_claude_model_label(model_id: &str) -> Option<String> {
    let without_context = model_id.strip_suffix("[1m]").unwrap_or(model_id);
    let mut parts = without_context.strip_prefix("claude-")?.split('-');
    let family = parts.next()?;
    if family.is_empty()
        || !family
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return None;
    }
    let version = parts
        .take(2)
        .take_while(|part| {
            !part.is_empty()
                && part.len() <= 2
                && part.chars().all(|character| character.is_ascii_digit())
        })
        .collect::<Vec<_>>();
    if version.is_empty() {
        return None;
    }
    let mut label = family.to_string();
    if let Some(first) = label.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    label.push(' ');
    label.push_str(&version.join("."));
    Some(label)
}

fn claude_model_options(
    supports_effort: bool,
    supported_effort_levels: &[String],
    supports_fast_mode: bool,
) -> Vec<ModelOptionDefinition> {
    let mut definitions = Vec::new();
    if supports_effort {
        let mut seen = BTreeSet::new();
        let levels = supported_effort_levels
            .iter()
            .filter(|level| matches!(level.as_str(), "low" | "medium" | "high" | "xhigh" | "max"))
            .filter(|level| seen.insert(level.as_str()))
            .map(|level| ModelChoiceOption {
                value: level.clone(),
                label: level.clone(),
                description: None,
            })
            .collect::<Vec<_>>();
        if !levels.is_empty() {
            let default_value = levels
                .iter()
                .find(|level| level.value == "high")
                .map(|level| level.value.clone());
            definitions.push(ModelOptionDefinition::Choice {
                key: "effort".to_string(),
                label: "Effort".to_string(),
                description: Some("Provider-supported reasoning effort".to_string()),
                options: levels,
                default_value,
            });
        }
    }
    if supports_fast_mode {
        definitions.push(ModelOptionDefinition::Boolean {
            key: "fastMode".to_string(),
            label: "Fast mode".to_string(),
            description: Some("Lower latency with higher usage cost".to_string()),
            default_value: Some(false),
        });
    }
    definitions
}

fn claude_model_supports_fast_mode(model: &ClaudeModel) -> bool {
    [
        Some(model.value.as_str()),
        model.resolved_model.as_deref(),
        Some(model.display_name.as_str()),
        model.description.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(claude_identity_supports_fast_mode)
}

fn claude_identity_supports_fast_mode(identity: &str) -> bool {
    let normalized = identity.trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "default" | "opus") {
        return true;
    }
    let Some(opus_index) = normalized.find("opus") else {
        return false;
    };
    let mut version_parts = normalized[opus_index + "opus".len()..]
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u64>().ok());
    let Some(major) = version_parts.next() else {
        return false;
    };
    let minor = version_parts.next().unwrap_or_default();
    (major, minor) >= (4, 6)
}

pub fn selected_effort(options: &[ModelOptionSelection]) -> ChatResult<Option<String>> {
    validate_model_options(options)?;
    for option in options {
        if option.key == "effort" {
            let ModelOptionValue::Choice(value) = &option.value else {
                return Err(ChatError::validation(
                    "modelOptions",
                    "Claude effort must use a single choice",
                ));
            };
            return Ok(Some(value.clone()));
        }
    }
    Ok(None)
}

pub fn selected_fast_mode(options: &[ModelOptionSelection]) -> ChatResult<Option<bool>> {
    validate_model_options(options)?;
    for option in options {
        if option.key == "fastMode" {
            let ModelOptionValue::Boolean(value) = &option.value else {
                return Err(ChatError::validation(
                    "modelOptions",
                    "Claude Fast mode must use a boolean",
                ));
            };
            return Ok(Some(*value));
        }
    }
    Ok(None)
}

fn validate_model_options(options: &[ModelOptionSelection]) -> ChatResult<()> {
    if options.len() > MAX_MODEL_OPTIONS {
        return Err(ChatError::validation(
            "modelOptions",
            "Claude model options exceed the supported limit",
        ));
    }
    let mut keys = BTreeSet::new();
    for option in options {
        if !keys.insert(option.key.as_str()) {
            return Err(ChatError::validation(
                "modelOptions",
                "Claude model option is unsupported or duplicated",
            ));
        }
        match option.key.as_str() {
            "effort" => {
                let ModelOptionValue::Choice(value) = &option.value else {
                    return Err(ChatError::validation(
                        "modelOptions",
                        "Claude effort must use a single choice",
                    ));
                };
                if !matches!(value.as_str(), "low" | "medium" | "high" | "xhigh" | "max") {
                    return Err(ChatError::validation(
                        "modelOptions",
                        "Claude effort value is unsupported",
                    ));
                }
            }
            "fastMode" if matches!(&option.value, ModelOptionValue::Boolean(_)) => {}
            _ => {
                return Err(ChatError::validation(
                    "modelOptions",
                    "Claude model option is unsupported or has the wrong type",
                ));
            }
        }
    }
    Ok(())
}

fn validate_launch_arguments(arguments: &[String]) -> ChatResult<()> {
    const PROTECTED: &[&str] = &[
        "-p",
        "--print",
        "--output-format",
        "--input-format",
        "--permission-prompt-tool",
        "--include-partial-messages",
        "--replay-user-messages",
        "--permission-mode",
        "--dangerously-skip-permissions",
        "--allow-dangerously-skip-permissions",
        "--resume",
        "--resume-session-at",
        "--session-id",
        "--model",
        "--effort",
        "--settings",
    ];
    if arguments.len() > MAX_LAUNCH_ARGUMENTS
        || arguments.iter().any(|argument| {
            argument.len() > MAX_LAUNCH_ARGUMENT_BYTES
                || argument.contains('\0')
                || PROTECTED.iter().any(|protected| {
                    argument == protected || argument.starts_with(&format!("{protected}="))
                })
        })
    {
        return Err(ChatError::validation(
            "launchArguments",
            "Claude launch arguments conflict with the managed transport",
        ));
    }
    Ok(())
}

fn validate_model_id(value: &str) -> ChatResult<()> {
    if value.trim().is_empty()
        || value.len() > MAX_MODEL_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "modelId",
            "Claude model identifier is invalid",
        ));
    }
    Ok(())
}

pub fn validate_uuid(value: &str, field: &str) -> ChatResult<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| bytes[index] == b'-')
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit());
    if !valid {
        return Err(ChatError::validation(field, "Claude UUID is invalid"));
    }
    Ok(())
}

fn validate_protocol_id(value: &str, field: &str) -> ChatResult<()> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(ChatError::validation(
            field,
            "Claude protocol identifier is invalid",
        ));
    }
    Ok(())
}

fn required_text<'a>(object: &'a Map<String, Value>, key: &str) -> ChatResult<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| protocol_error(key))
}

fn bounded_text(value: &str, maximum: usize) -> String {
    if value.len() <= maximum {
        return value.to_string();
    }
    let mut end = maximum;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn unsupported_version() -> ChatError {
    ChatError::new(
        ChatErrorCode::UnsupportedVersion,
        "Claude Code version could not be verified",
        true,
    )
}

pub fn protocol_error(label: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        format!("Claude returned an invalid {label}"),
        false,
    )
}
