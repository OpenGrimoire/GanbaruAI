//! Typed Codex app-server request builders and response decoders.

use super::transport::CodexRpcFailure;
use crate::chat::models::{
    ChatError, ChatResult, InteractionMode, ModelAvailability, ModelChoiceOption, ModelId,
    ModelOptionDefinition, ModelOptionSelection, ModelOptionValue, ProviderCapability,
    ProviderModel, SafetyMode, SendTurnRequest, TurnModeSnapshot,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const MAX_PROMPT_BYTES: usize = 4 * 1024 * 1024;
const MAX_DEVELOPER_INSTRUCTIONS_BYTES: usize = 64 * 1024;
const MAX_MODEL_ID_BYTES: usize = 256;
const MAX_MODEL_OPTIONS: usize = 32;
const STANDARD_SERVICE_TIER: &str = "standard";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub user_agent: String,
    pub codex_home: PathBuf,
    pub platform_family: String,
    pub platform_os: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadOpenResponse {
    pub thread: CodexThread,
    pub model: String,
    pub approval_policy: Value,
    pub sandbox: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThread {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnStartResponse {
    pub turn: CodexTurn,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexTurn {
    pub id: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountReadResponse {
    pub account: Option<CodexAccount>,
    pub requires_openai_auth: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAccount {
    #[serde(rename = "type")]
    pub account_type: String,
    pub email: Option<String>,
    pub plan_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelListResponse {
    pub data: Vec<CodexModel>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModel {
    pub id: String,
    pub model: String,
    pub display_name: String,
    pub description: String,
    pub hidden: bool,
    pub is_default: bool,
    pub default_reasoning_effort: String,
    pub supported_reasoning_efforts: Vec<CodexReasoningEffort>,
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub service_tiers: Vec<CodexServiceTier>,
    pub default_service_tier: Option<String>,
    #[serde(default)]
    pub supports_personality: bool,
    pub upgrade: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexReasoningEffort {
    pub reasoning_effort: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CodexServiceTier {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodexSafetySettings {
    pub approval_policy: &'static str,
    pub sandbox: &'static str,
    pub turn_sandbox_type: &'static str,
}

pub fn safety_settings(mode: SafetyMode) -> CodexSafetySettings {
    match mode {
        SafetyMode::Supervised => CodexSafetySettings {
            approval_policy: "untrusted",
            sandbox: "read-only",
            turn_sandbox_type: "readOnly",
        },
        SafetyMode::AutoAcceptEdits => CodexSafetySettings {
            approval_policy: "on-request",
            sandbox: "workspace-write",
            turn_sandbox_type: "workspaceWrite",
        },
        SafetyMode::FullAccess => CodexSafetySettings {
            approval_policy: "never",
            sandbox: "danger-full-access",
            turn_sandbox_type: "dangerFullAccess",
        },
    }
}

pub fn initialize_params() -> Value {
    json!({
        "clientInfo": {
            "name": "ganbaru_ai_desktop",
            "title": "Ganbaru AI",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "experimentalApi": true,
        },
    })
}

pub fn thread_open_params(
    provider_thread_id: Option<&str>,
    workspace: &Path,
    modes: TurnModeSnapshot,
    model_id: Option<&ModelId>,
    developer_instructions: Option<&str>,
) -> ChatResult<Value> {
    validate_optional_text(
        developer_instructions,
        MAX_DEVELOPER_INSTRUCTIONS_BYTES,
        "developerInstructions",
    )?;
    let safety = safety_settings(modes.safety_mode);
    let mut params = Map::from_iter([
        (
            "cwd".to_string(),
            Value::String(workspace.to_string_lossy().into_owned()),
        ),
        (
            "approvalPolicy".to_string(),
            Value::String(safety.approval_policy.to_string()),
        ),
        (
            "sandbox".to_string(),
            Value::String(safety.sandbox.to_string()),
        ),
    ]);
    if let Some(provider_thread_id) = provider_thread_id {
        params.insert(
            "threadId".to_string(),
            Value::String(provider_thread_id.to_string()),
        );
    }
    if let Some(model_id) = model_id {
        validate_model_id(model_id.as_str())?;
        params.insert(
            "model".to_string(),
            Value::String(model_id.as_str().to_string()),
        );
    }
    if let Some(instructions) = developer_instructions {
        params.insert(
            "developerInstructions".to_string(),
            Value::String(instructions.to_string()),
        );
    }
    Ok(Value::Object(params))
}

pub fn turn_start_params(
    provider_thread_id: &str,
    _workspace: &Path,
    fallback_model: &str,
    request: &SendTurnRequest,
) -> ChatResult<Value> {
    if request.prompt.len() > MAX_PROMPT_BYTES || request.prompt.contains('\0') {
        return Err(ChatError::validation(
            "prompt",
            "Codex prompt exceeds the supported limit",
        ));
    }
    validate_optional_text(
        request.developer_instructions.as_deref(),
        MAX_DEVELOPER_INSTRUCTIONS_BYTES,
        "developerInstructions",
    )?;
    let mut input = Vec::new();
    if !request.prompt.is_empty() {
        input.push(json!({ "type": "text", "text": request.prompt }));
    }
    for attachment in &request.attachments {
        match attachment.kind.as_str() {
            "image" => {
                let path = attachment.local_path.as_deref().ok_or_else(|| {
                    ChatError::validation("attachments", "Codex image attachment is unavailable")
                })?;
                let path = Path::new(path);
                if !path.is_absolute() || !path.is_file() {
                    return Err(ChatError::validation(
                        "attachments",
                        "Codex image attachment is unavailable",
                    ));
                }
                input.push(json!({
                    "type": "localImage",
                    "path": path.to_string_lossy(),
                }));
            }
            "text_snippet" => {
                let text = attachment.text_content.as_deref().ok_or_else(|| {
                    ChatError::validation("attachments", "Codex text context is unavailable")
                })?;
                if text.len() > 128 * 1024 || text.contains('\0') {
                    return Err(ChatError::validation(
                        "attachments",
                        "Codex text context exceeds the supported limit",
                    ));
                }
                input.push(json!({ "type": "text", "text": text }));
            }
            _ => {
                return Err(ChatError::unsupported(
                    "Codex prompt attachment kind is unsupported",
                ));
            }
        }
    }
    if input.is_empty() {
        return Err(ChatError::validation(
            "prompt",
            "Codex turn requires text or an image",
        ));
    }
    let (effort, service_tier) = parse_model_options(&request.model_options)?;
    let safety = safety_settings(request.modes.safety_mode);
    let mut params = Map::from_iter([
        (
            "threadId".to_string(),
            Value::String(provider_thread_id.to_string()),
        ),
        ("input".to_string(), Value::Array(input)),
        (
            "approvalPolicy".to_string(),
            Value::String(safety.approval_policy.to_string()),
        ),
        (
            "sandboxPolicy".to_string(),
            json!({ "type": safety.turn_sandbox_type }),
        ),
        (
            "clientUserMessageId".to_string(),
            Value::String(request.turn_id.as_str().to_string()),
        ),
    ]);
    if let Some(model_id) = request.model_id.as_ref() {
        validate_model_id(model_id.as_str())?;
        params.insert(
            "model".to_string(),
            Value::String(model_id.as_str().to_string()),
        );
    }
    if let Some(effort) = effort.as_deref() {
        params.insert("effort".to_string(), Value::String(effort.to_string()));
    }
    if let Some(service_tier) = service_tier.filter(|tier| tier != STANDARD_SERVICE_TIER) {
        params.insert("serviceTier".to_string(), Value::String(service_tier));
    }
    let collaboration_model = request
        .model_id
        .as_ref()
        .map(|model| model.as_str())
        .unwrap_or(fallback_model);
    validate_model_id(collaboration_model)?;
    let mut collaboration_settings = Map::from_iter([(
        "model".to_string(),
        Value::String(collaboration_model.to_string()),
    )]);
    if let Some(effort) = effort {
        collaboration_settings.insert("reasoning_effort".to_string(), Value::String(effort));
    }
    if let Some(instructions) = request.developer_instructions.as_ref() {
        collaboration_settings.insert(
            "developer_instructions".to_string(),
            Value::String(instructions.clone()),
        );
    }
    params.insert(
        "collaborationMode".to_string(),
        json!({
            "mode": match request.modes.interaction_mode {
                InteractionMode::Build => "default",
                InteractionMode::Plan => "plan",
            },
            "settings": collaboration_settings,
        }),
    );
    Ok(Value::Object(params))
}

pub fn decode_response<T>(value: Value, label: &str) -> Result<T, CodexRpcFailure>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value)
        .map_err(|_| CodexRpcFailure::Malformed(format!("{label} has an invalid shape")))
}

pub fn provider_model(model: CodexModel) -> ChatResult<ProviderModel> {
    validate_model_id(&model.id)?;
    let reasoning_options = model
        .supported_reasoning_efforts
        .into_iter()
        .map(|effort| ModelChoiceOption {
            label: effort.reasoning_effort.clone(),
            value: effort.reasoning_effort,
            description: nonempty(effort.description),
        })
        .collect::<Vec<_>>();
    let mut service_tier_options = model
        .service_tiers
        .into_iter()
        .map(|tier| ModelChoiceOption {
            value: tier.id,
            label: tier.name,
            description: nonempty(tier.description),
        })
        .collect::<Vec<_>>();
    if !service_tier_options.is_empty()
        && !service_tier_options
            .iter()
            .any(|tier| tier.value == STANDARD_SERVICE_TIER)
    {
        service_tier_options.insert(
            0,
            ModelChoiceOption {
                value: STANDARD_SERVICE_TIER.to_string(),
                label: "Standard".to_string(),
                description: Some("Default speed and usage".to_string()),
            },
        );
    }
    let mut options = Vec::new();
    if !reasoning_options.is_empty() {
        options.push(ModelOptionDefinition::Choice {
            key: "reasoning_effort".to_string(),
            label: "Reasoning effort".to_string(),
            description: None,
            options: reasoning_options,
            default_value: Some(model.default_reasoning_effort),
        });
    }
    if !service_tier_options.is_empty() {
        options.push(ModelOptionDefinition::Choice {
            key: "service_tier".to_string(),
            label: "Service tier".to_string(),
            description: None,
            options: service_tier_options,
            default_value: Some(
                model
                    .default_service_tier
                    .unwrap_or_else(|| STANDARD_SERVICE_TIER.to_string()),
            ),
        });
    }
    let mut capabilities = vec![
        ProviderCapability::NativeResume,
        ProviderCapability::NativePlan,
        ProviderCapability::DynamicModelChange,
        ProviderCapability::Approvals,
        ProviderCapability::StructuredQuestions,
        ProviderCapability::ReasoningSummaries,
        ProviderCapability::StructuredPlans,
        ProviderCapability::ContextUsage,
    ];
    if model.input_modalities.iter().any(|value| value == "image") {
        capabilities.push(ProviderCapability::Images);
    }
    Ok(ProviderModel {
        id: ModelId::new(model.id).map_err(identifier_error)?,
        display_name: if model.display_name.trim().is_empty() {
            model.model
        } else {
            model.display_name
        },
        description: nonempty(model.description),
        context_limit: None,
        availability: if model.hidden {
            ModelAvailability::Unavailable
        } else if model.upgrade.is_some() {
            ModelAvailability::Deprecated
        } else {
            ModelAvailability::Available
        },
        capabilities,
        options,
        custom: false,
    })
}

pub fn custom_provider_model(
    model_id: &str,
    display_name: Option<&str>,
) -> ChatResult<ProviderModel> {
    validate_model_id(model_id)?;
    Ok(ProviderModel {
        id: ModelId::new(model_id.to_string()).map_err(identifier_error)?,
        display_name: display_name.unwrap_or(model_id).to_string(),
        description: Some("Custom Codex model ID".to_string()),
        context_limit: None,
        availability: ModelAvailability::Unknown,
        capabilities: Vec::new(),
        options: Vec::new(),
        custom: true,
    })
}

fn parse_model_options(
    options: &[ModelOptionSelection],
) -> ChatResult<(Option<String>, Option<String>)> {
    if options.len() > MAX_MODEL_OPTIONS {
        return Err(ChatError::validation(
            "modelOptions",
            "Codex model options exceed the supported limit",
        ));
    }
    let mut effort = None;
    let mut service_tier = None;
    for option in options {
        let ModelOptionValue::Choice(value) = &option.value else {
            return Err(ChatError::validation(
                "modelOptions",
                "Codex model options must use a single choice",
            ));
        };
        validate_model_id(value)?;
        match option.key.as_str() {
            "reasoning_effort" if effort.is_none() => effort = Some(value.clone()),
            "service_tier" if service_tier.is_none() => service_tier = Some(value.clone()),
            _ => {
                return Err(ChatError::validation(
                    "modelOptions",
                    "Codex model option is unsupported or duplicated",
                ))
            }
        }
    }
    Ok((effort, service_tier))
}

fn validate_model_id(value: &str) -> ChatResult<()> {
    if value.trim().is_empty()
        || value.len() > MAX_MODEL_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(ChatError::validation(
            "modelId",
            "Codex model identifier is invalid",
        ));
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, maximum: usize, field: &str) -> ChatResult<()> {
    if value.is_some_and(|value| value.len() > maximum || value.contains('\0')) {
        return Err(ChatError::validation(
            field,
            "Codex text configuration exceeds the supported limit",
        ));
    }
    Ok(())
}

fn nonempty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn identifier_error(_error: String) -> ChatError {
    ChatError::new(
        crate::chat::models::ChatErrorCode::Protocol,
        "Codex returned an invalid identifier",
        false,
    )
}
