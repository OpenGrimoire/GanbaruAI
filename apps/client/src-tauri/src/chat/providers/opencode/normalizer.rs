//! OpenCode event normalization into provider-neutral Chat events.

use super::protocol::{protocol_error, validate_identifier, MAX_EVENT_DATA_BYTES};
use crate::chat::events::*;
use crate::chat::models::*;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_DEDUPLICATION_EVENTS: usize = 4_096;
const MAX_TEXT_BYTES: usize = 2 * 1024 * 1024;
const MAX_SAFE_COLLECTION: usize = 256;
const MAX_QUESTIONS: usize = 16;
const MAX_OPTIONS: usize = 64;

#[derive(Clone, Debug)]
pub struct OpenCodeRouteState {
    pub provider_thread_id: String,
    pub active_turn_id: Option<ChatTurnId>,
    pub session_state: ProviderSessionState,
    pub modes: TurnModeSnapshot,
    message_roles: HashMap<String, String>,
    rollback_message_id: Option<String>,
    part_text: HashMap<String, String>,
    completed_parts: HashSet<String>,
    pending_permissions: HashSet<String>,
    pending_questions: HashMap<String, Vec<OpenCodeQuestionMapping>>,
    deduplication_order: VecDeque<String>,
    deduplication_set: HashSet<String>,
}

#[derive(Clone, Debug)]
struct OpenCodeQuestionMapping {
    id: String,
    option_labels: Vec<String>,
    free_form_allowed: bool,
}

impl OpenCodeRouteState {
    pub fn new(provider_thread_id: String, modes: TurnModeSnapshot) -> Self {
        Self {
            provider_thread_id,
            active_turn_id: None,
            session_state: ProviderSessionState::Ready,
            modes,
            message_roles: HashMap::new(),
            rollback_message_id: None,
            part_text: HashMap::new(),
            completed_parts: HashSet::new(),
            pending_permissions: HashSet::new(),
            pending_questions: HashMap::new(),
            deduplication_order: VecDeque::new(),
            deduplication_set: HashSet::new(),
        }
    }

    pub fn has_permission(&self, request_id: &ProviderRequestId) -> bool {
        self.pending_permissions.contains(request_id.as_str())
    }

    pub fn question_answers(
        &self,
        request_id: &ProviderRequestId,
        answers: &[UserInputAnswer],
    ) -> ChatResult<Vec<Vec<String>>> {
        let questions = self
            .pending_questions
            .get(request_id.as_str())
            .ok_or_else(|| {
                ChatError::new(
                    ChatErrorCode::Conflict,
                    "OpenCode question is no longer pending in this session",
                    false,
                )
            })?;
        if answers.len() != questions.len() {
            return Err(ChatError::validation(
                "answers",
                "OpenCode question answers are incomplete",
            ));
        }
        questions
            .iter()
            .map(|question| {
                let answer = answers
                    .iter()
                    .find(|answer| answer.question_id == question.id)
                    .ok_or_else(|| {
                        ChatError::validation(
                            "answers",
                            "OpenCode question answer does not match the prompt",
                        )
                    })?;
                let mut values = Vec::new();
                for option_id in &answer.selected_option_ids {
                    let index = option_id
                        .strip_prefix("option-")
                        .and_then(|value| value.parse::<usize>().ok())
                        .ok_or_else(|| {
                            ChatError::validation("answers", "OpenCode question option is invalid")
                        })?;
                    let label = question.option_labels.get(index).ok_or_else(|| {
                        ChatError::validation("answers", "OpenCode question option is unavailable")
                    })?;
                    values.push(label.clone());
                }
                if let Some(text) = answer
                    .free_form_text
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    if !question.free_form_allowed || text.len() > 4096 || text.contains('\0') {
                        return Err(ChatError::validation(
                            "answers",
                            "OpenCode free-form question answer is invalid",
                        ));
                    }
                    values.push(text.to_string());
                }
                Ok(values)
            })
            .collect()
    }

    fn observe(&mut self, value: &Value) -> ChatResult<bool> {
        let encoded = serde_json::to_vec(value).map_err(|_| protocol_error("event envelope"))?;
        if encoded.len() > MAX_EVENT_DATA_BYTES {
            return Err(protocol_error("event envelope size"));
        }
        let fingerprint = format!("{:x}", Sha256::digest(encoded));
        if self.deduplication_set.contains(&fingerprint) {
            return Ok(false);
        }
        self.deduplication_set.insert(fingerprint.clone());
        self.deduplication_order.push_back(fingerprint);
        if self.deduplication_order.len() > MAX_DEDUPLICATION_EVENTS {
            if let Some(expired) = self.deduplication_order.pop_front() {
                self.deduplication_set.remove(&expired);
            }
        }
        Ok(true)
    }
}

pub struct OpenCodeEventNormalizer {
    provider_instance_id: ProviderInstanceId,
    thread_id: ChatThreadId,
    session_id: ProviderSessionId,
    next_event_id: AtomicU64,
}

impl OpenCodeEventNormalizer {
    pub fn new(
        provider_instance_id: ProviderInstanceId,
        thread_id: ChatThreadId,
        session_id: ProviderSessionId,
    ) -> Self {
        Self {
            provider_instance_id,
            thread_id,
            session_id,
            next_event_id: AtomicU64::new(1),
        }
    }

    pub fn normalize(
        &self,
        state: &mut OpenCodeRouteState,
        envelope: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        if !state.observe(&envelope)? {
            return Ok(Vec::new());
        }
        let object = envelope
            .as_object()
            .ok_or_else(|| protocol_error("event envelope"))?;
        let event_type = text(object, "type").ok_or_else(|| protocol_error("event type"))?;
        let properties = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("event properties"))?;
        if event_type != "server.connected"
            && event_type != "server.heartbeat"
            && properties
                .get("sessionID")
                .and_then(Value::as_str)
                .is_some_and(|session_id| session_id != state.provider_thread_id)
        {
            return Ok(Vec::new());
        }
        match event_type {
            "server.connected" | "server.heartbeat" => Ok(Vec::new()),
            "session.updated" => self.session_updated(state, properties),
            "session.status" => self.session_status(state, properties),
            "session.error" => self.session_error(state, properties),
            "session.diff" => self.session_diff(state, properties),
            "message.updated" => self.message_updated(state, properties),
            "message.removed" => {
                if let Some(message_id) = text(properties, "messageID") {
                    state.message_roles.remove(message_id);
                }
                Ok(Vec::new())
            }
            "message.part.delta" => self.part_delta(state, properties),
            "message.part.updated" => self.part_updated(state, properties),
            "permission.asked" => self.permission_asked(state, properties),
            "permission.replied" => self.permission_replied(state, properties),
            "question.asked" => self.question_asked(state, properties),
            "question.replied" => self.question_replied(state, properties, false),
            "question.rejected" => self.question_replied(state, properties, true),
            "todo.updated" => self.todo_updated(state, properties),
            "mcp.status" | "mcp.updated" => self.mcp_status(state, properties),
            unknown => Ok(vec![self.event(
                state,
                unknown,
                None,
                None,
                CanonicalEvent::Unknown(UnknownEvent {
                    source_type: format!("opencode/{unknown}"),
                    summary: "OpenCode emitted an unsupported event".to_string(),
                    safe_payload: Some(safe_shape(&Value::Object(properties.clone()))),
                }),
            )?]),
        }
    }

    pub fn external_event(
        &self,
        state: &OpenCodeRouteState,
        source: &str,
        event: CanonicalEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        self.event(state, source, None, None, event)
    }

    fn session_updated(
        &self,
        state: &OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let info = properties
            .get("info")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("session update"))?;
        let title = text(info, "title")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(|title| bounded(title, 512));
        Ok(vec![self.event(
            state,
            "session.updated",
            None,
            None,
            CanonicalEvent::ThreadMetadataUpdated(ThreadMetadataUpdatedEvent {
                title,
                provider_thread_id: ProviderThreadId::new(state.provider_thread_id.clone()).ok(),
                resume_cursor: None,
                metadata: Some(VersionedJson {
                    schema_version: 1,
                    value: json!({
                        "directory": text(info, "directory").map(|value| bounded(value, 4096)),
                        "version": text(info, "version").map(|value| bounded(value, 128)),
                    }),
                }),
            }),
        )?])
    }

    fn session_status(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let status = properties
            .get("status")
            .and_then(Value::as_object)
            .and_then(|status| text(status, "type"))
            .ok_or_else(|| protocol_error("session status"))?;
        match status {
            "busy" => {
                let previous = state.session_state;
                state.session_state = ProviderSessionState::Active;
                if previous == state.session_state {
                    return Ok(Vec::new());
                }
                Ok(vec![self.event(
                    state,
                    "session.status",
                    None,
                    None,
                    CanonicalEvent::SessionStateChanged(SessionStateChangedEvent {
                        session_id: self.session_id.clone(),
                        previous_state: previous,
                        state: state.session_state,
                        reason: None,
                    }),
                )?])
            }
            "idle" => {
                let previous = state.session_state;
                state.session_state = ProviderSessionState::Ready;
                let mut events = Vec::new();
                if previous != state.session_state {
                    events.push(self.event(
                        state,
                        "session.status",
                        None,
                        None,
                        CanonicalEvent::SessionStateChanged(SessionStateChangedEvent {
                            session_id: self.session_id.clone(),
                            previous_state: previous,
                            state: state.session_state,
                            reason: None,
                        }),
                    )?);
                }
                if let Some(turn_id) = state.active_turn_id.take() {
                    events.push(self.event(
                        state,
                        "session.status",
                        None,
                        Some(turn_id),
                        CanonicalEvent::TurnCompleted(TurnCompletedEvent {
                            state: ChatTurnState::Completed,
                            stop_reason: Some("idle".to_string()),
                            usage: None,
                            changed_files: Vec::new(),
                        }),
                    )?);
                }
                Ok(events)
            }
            "retry" => {
                let detail = properties
                    .get("status")
                    .and_then(Value::as_object)
                    .and_then(|status| text(status, "message"))
                    .map(|message| bounded(message, 4096));
                Ok(vec![self.event(
                    state,
                    "session.status",
                    None,
                    None,
                    CanonicalEvent::RuntimeWarning(NotificationEvent {
                        code: "opencode_retry".to_string(),
                        title: "OpenCode is retrying".to_string(),
                        detail,
                    }),
                )?])
            }
            unknown => Ok(vec![self.unknown(
                state,
                "session.status",
                unknown,
                properties,
            )?]),
        }
    }

    fn session_error(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let message = properties
            .get("error")
            .and_then(Value::as_object)
            .and_then(|error| text(error, "message"))
            .map(|message| bounded(message, 4096))
            .unwrap_or_else(|| "OpenCode reported a provider error".to_string());
        let mut events = vec![self.event(
            state,
            "session.error",
            None,
            None,
            CanonicalEvent::RuntimeError(RuntimeErrorEvent {
                code: "opencode_provider_error".to_string(),
                message: message.clone(),
                recoverable: true,
                safe_details: properties.get("error").map(safe_shape),
            }),
        )?];
        if let Some(turn_id) = state.active_turn_id.take() {
            state.session_state = ProviderSessionState::Ready;
            events.push(self.event(
                state,
                "session.error",
                None,
                Some(turn_id),
                CanonicalEvent::TurnCompleted(TurnCompletedEvent {
                    state: ChatTurnState::Failed,
                    stop_reason: Some(message),
                    usage: None,
                    changed_files: Vec::new(),
                }),
            )?);
        }
        Ok(events)
    }

    fn message_updated(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let info = properties
            .get("info")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("message update"))?;
        let id = identifier(info, "id", "message ID")?;
        let role = text(info, "role").ok_or_else(|| protocol_error("message role"))?;
        if !matches!(role, "user" | "assistant") {
            return Err(protocol_error("message role"));
        }
        state.message_roles.insert(id.to_string(), role.to_string());
        if role != "assistant" {
            return Ok(Vec::new());
        }
        state.rollback_message_id = Some(id.to_string());
        let model_id = text(info, "modelID").and_then(|model| ModelId::new(model.to_string()).ok());
        let provider_id = text(info, "providerID").unwrap_or_default();
        let effective_model = model_id.and_then(|model| {
            if provider_id.is_empty() {
                Some(model)
            } else {
                ModelId::new(format!("{provider_id}/{}", model.as_str())).ok()
            }
        });
        let Some(effective_model) = effective_model else {
            return Ok(Vec::new());
        };
        Ok(vec![self.event(
            state,
            "message.updated",
            ProviderItemId::new(id.to_string()).ok(),
            None,
            CanonicalEvent::SessionConfigured(SessionConfiguredEvent {
                session_id: self.session_id.clone(),
                effective_modes: state.modes,
                effective_model_id: Some(effective_model),
                effective_model_options: text(info, "variant")
                    .map(|variant| ModelOptionSelection {
                        key: "variant".to_string(),
                        value: ModelOptionValue::Choice(bounded(variant, 256)),
                    })
                    .into_iter()
                    .collect(),
            }),
        )?])
    }

    fn part_delta(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let part_id = identifier(properties, "partID", "part ID")?;
        let delta = text(properties, "delta").ok_or_else(|| protocol_error("part delta"))?;
        if delta.is_empty() {
            return Ok(Vec::new());
        }
        let current = state.part_text.entry(part_id.to_string()).or_default();
        if current.len().saturating_add(delta.len()) > MAX_TEXT_BYTES {
            return Err(protocol_error("part text size"));
        }
        current.push_str(delta);
        let stream_kind = match text(properties, "field") {
            Some("reasoning") => ContentStreamKind::ReasoningText,
            _ => ContentStreamKind::AssistantText,
        };
        Ok(vec![self.event(
            state,
            "message.part.delta",
            ProviderItemId::new(part_id.to_string()).ok(),
            None,
            CanonicalEvent::ContentDelta(ContentDeltaEvent {
                item_id: part_id.to_string(),
                stream_kind,
                content_index: 0,
                delta: bounded(delta, MAX_TEXT_BYTES),
            }),
        )?])
    }

    fn part_updated(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let part = properties
            .get("part")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("message part"))?;
        let part_type = text(part, "type").ok_or_else(|| protocol_error("part type"))?;
        match part_type {
            "text" | "reasoning" => self.text_part(state, part, part_type),
            "tool" => self.tool_part(state, part),
            "file" => self.file_part(state, part),
            "patch" => self.patch_part(state, part),
            "step-finish" => self.step_finish(state, part),
            "subtask" => self.subtask(state, part),
            "compaction" => self.compaction(state, part),
            "retry" => Ok(vec![self.event(
                state,
                "message.part.updated",
                part_id(part),
                None,
                CanonicalEvent::RuntimeWarning(NotificationEvent {
                    code: "opencode_retry".to_string(),
                    title: "OpenCode retried a model request".to_string(),
                    detail: part.get("error").map(safe_shape).and_then(|value| {
                        serde_json::to_string(&value.value)
                            .ok()
                            .map(|value| bounded(&value, 4096))
                    }),
                }),
            )?]),
            "snapshot" | "step-start" | "agent" => Ok(Vec::new()),
            unknown => Ok(vec![self.unknown(
                state,
                "message.part.updated",
                unknown,
                part,
            )?]),
        }
    }

    fn text_part(
        &self,
        state: &mut OpenCodeRouteState,
        part: &Map<String, Value>,
        part_type: &str,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(part, "id", "part ID")?;
        let text_value = text(part, "text").ok_or_else(|| protocol_error("part text"))?;
        if text_value.len() > MAX_TEXT_BYTES {
            return Err(protocol_error("part text size"));
        }
        let previous = state.part_text.get(id).cloned().unwrap_or_default();
        let delta = if text_value.starts_with(&previous) {
            &text_value[previous.len()..]
        } else if previous.starts_with(text_value) {
            ""
        } else {
            text_value
        };
        state
            .part_text
            .insert(id.to_string(), text_value.to_string());
        let kind = if part_type == "reasoning" {
            CanonicalItemKind::Reasoning
        } else {
            CanonicalItemKind::AssistantMessage
        };
        let mut events = Vec::new();
        if previous.is_empty() && !text_value.is_empty() {
            events.push(self.item_event(state, id, kind, ActivityStatus::Active, None, None)?);
        }
        if !delta.is_empty() {
            events.push(self.event(
                state,
                "message.part.updated",
                ProviderItemId::new(id.to_string()).ok(),
                None,
                CanonicalEvent::ContentDelta(ContentDeltaEvent {
                    item_id: id.to_string(),
                    stream_kind: if part_type == "reasoning" {
                        ContentStreamKind::ReasoningText
                    } else {
                        ContentStreamKind::AssistantText
                    },
                    content_index: 0,
                    delta: delta.to_string(),
                }),
            )?);
        }
        let completed = part
            .get("time")
            .and_then(Value::as_object)
            .and_then(|time| time.get("end"))
            .is_some_and(Value::is_number);
        if completed && state.completed_parts.insert(id.to_string()) {
            events.push(self.item_event(state, id, kind, ActivityStatus::Completed, None, None)?);
        }
        Ok(events)
    }

    fn tool_part(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = text(part, "callID")
            .or_else(|| text(part, "id"))
            .ok_or_else(|| protocol_error("tool call ID"))?;
        validate_identifier(id, "tool call ID")?;
        let tool = text(part, "tool").ok_or_else(|| protocol_error("tool name"))?;
        let state_value = part
            .get("state")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("tool state"))?;
        let status = match text(state_value, "status") {
            Some("pending") => ActivityStatus::Pending,
            Some("running") => ActivityStatus::Active,
            Some("completed") => ActivityStatus::Completed,
            Some("error") => ActivityStatus::Failed,
            _ => ActivityStatus::Unknown,
        };
        let kind = tool_kind(tool);
        let title = text(state_value, "title")
            .map(|title| bounded(title, 512))
            .or_else(|| Some(bounded(tool, 256)));
        let detail = state_value
            .get("output")
            .and_then(Value::as_str)
            .or_else(|| text(state_value, "error"))
            .map(|detail| bounded(detail, MAX_TEXT_BYTES));
        let input = state_value.get("input").map(safe_shape);
        Ok(vec![self.item_lifecycle_event(
            state,
            id,
            ItemLifecycleEvent {
                item_id: id.to_string(),
                kind,
                status,
                title,
                detail,
                safe_metadata: Some(VersionedJson {
                    schema_version: 1,
                    value: json!({
                        "tool": bounded(tool, 256),
                        "input": input.map(|value| value.value),
                    }),
                }),
            },
        )?])
    }

    fn file_part(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(part, "id", "file part ID")?;
        let title = text(part, "filename")
            .map(|value| bounded(value, 1024))
            .unwrap_or_else(|| "File context".to_string());
        Ok(vec![self.item_event(
            state,
            id,
            CanonicalItemKind::FileChange,
            ActivityStatus::Completed,
            Some(title),
            None,
        )?])
    }

    fn patch_part(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let files = part
            .get("files")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("patch files"))?;
        let summaries = changed_files_from_paths(files)?;
        Ok(vec![self.event(
            state,
            "message.part.updated",
            part_id(part),
            None,
            CanonicalEvent::DiffUpdated(DiffUpdatedEvent {
                source: "opencode".to_string(),
                files: summaries,
                provider_diff: None,
            }),
        )?])
    }

    fn step_finish(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let tokens = part.get("tokens").and_then(Value::as_object);
        let cache = tokens
            .and_then(|tokens| tokens.get("cache"))
            .and_then(Value::as_object);
        let usage = ThreadUsageUpdatedEvent {
            input_tokens: tokens
                .and_then(|tokens| tokens.get("input"))
                .and_then(Value::as_u64),
            output_tokens: tokens
                .and_then(|tokens| tokens.get("output"))
                .and_then(Value::as_u64),
            cached_input_tokens: cache
                .and_then(|cache| cache.get("read"))
                .and_then(Value::as_u64),
            context_tokens: tokens
                .and_then(|tokens| tokens.get("total"))
                .and_then(Value::as_u64),
            context_limit: None,
            cost: part
                .get("cost")
                .and_then(Value::as_f64)
                .map(|amount| ProviderAttributedCost {
                    amount,
                    currency: "USD".to_string(),
                    provider_reported: true,
                }),
        };
        Ok(vec![self.event(
            state,
            "message.part.updated",
            part_id(part),
            None,
            CanonicalEvent::ThreadUsageUpdated(usage),
        )?])
    }

    fn subtask(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(part, "id", "subtask ID")?;
        Ok(vec![self.event(
            state,
            "message.part.updated",
            ProviderItemId::new(id.to_string()).ok(),
            None,
            CanonicalEvent::TaskLifecycle(TaskLifecycleEvent {
                task_id: id.to_string(),
                parent_task_id: None,
                status: ActivityStatus::Active,
                title: text(part, "description")
                    .map(|value| bounded(value, 512))
                    .unwrap_or_else(|| "OpenCode subtask".to_string()),
                detail: text(part, "prompt").map(|value| bounded(value, 4096)),
                safe_metadata: Some(safe_shape(&Value::Object(part.clone()))),
            }),
        )?])
    }

    fn compaction(
        &self,
        state: &OpenCodeRouteState,
        part: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(part, "id", "compaction ID")?;
        Ok(vec![self.item_event(
            state,
            id,
            CanonicalItemKind::ContextCompaction,
            ActivityStatus::Completed,
            Some("Context compacted".to_string()),
            None,
        )?])
    }

    fn session_diff(
        &self,
        state: &OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let diff = properties
            .get("diff")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("session diff"))?;
        if diff.len() > MAX_SAFE_COLLECTION {
            return Err(protocol_error("session diff size"));
        }
        let mut files = Vec::new();
        for entry in diff {
            let entry = entry
                .as_object()
                .ok_or_else(|| protocol_error("session diff entry"))?;
            let path = text(entry, "file")
                .or_else(|| text(entry, "path"))
                .ok_or_else(|| protocol_error("session diff path"))?;
            files.push(ChangedFileSummary {
                relative_path: bounded(path, 4096),
                previous_relative_path: text(entry, "before").map(|value| bounded(value, 4096)),
                additions: entry.get("additions").and_then(Value::as_u64),
                deletions: entry.get("deletions").and_then(Value::as_u64),
                binary: entry
                    .get("binary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                status: text(entry, "status").unwrap_or("modified").to_string(),
            });
        }
        Ok(vec![self.event(
            state,
            "session.diff",
            None,
            None,
            CanonicalEvent::DiffUpdated(DiffUpdatedEvent {
                source: "opencode".to_string(),
                files,
                provider_diff: None,
            }),
        )?])
    }

    fn permission_asked(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(properties, "id", "permission ID")?;
        state.pending_permissions.insert(id.to_string());
        let permission =
            text(properties, "permission").ok_or_else(|| protocol_error("permission kind"))?;
        let patterns = properties
            .get("patterns")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("permission patterns"))?;
        if patterns.len() > MAX_SAFE_COLLECTION
            || patterns.iter().any(|value| value.as_str().is_none())
        {
            return Err(protocol_error("permission patterns"));
        }
        let mut decisions = vec![
            ApprovalDecisionOption {
                id: "once".to_string(),
                label: "Allow once".to_string(),
                decision_kind: ApprovalDecisionKind::AllowOnce,
                description: None,
            },
            ApprovalDecisionOption {
                id: "reject".to_string(),
                label: "Deny".to_string(),
                decision_kind: ApprovalDecisionKind::Deny,
                description: None,
            },
        ];
        if properties
            .get("always")
            .and_then(Value::as_array)
            .is_some_and(|always| !always.is_empty())
        {
            decisions.insert(
                1,
                ApprovalDecisionOption {
                    id: "always".to_string(),
                    label: "Allow for session".to_string(),
                    decision_kind: ApprovalDecisionKind::AllowSession,
                    description: None,
                },
            );
        }
        let detail = patterns
            .iter()
            .filter_map(Value::as_str)
            .map(|value| bounded(value, 4096))
            .collect::<Vec<_>>()
            .join("\n");
        let request_id =
            ProviderRequestId::new(id.to_string()).map_err(|_| protocol_error("permission ID"))?;
        Ok(vec![self.event_with_request(
            state,
            "permission.asked",
            None,
            None,
            Some(request_id.clone()),
            CanonicalEvent::RequestOpened(RequestOpenedEvent {
                request_id,
                kind: request_kind(permission),
                title: format!("OpenCode requested {permission}"),
                detail: (!detail.is_empty()).then_some(detail),
                allowed_decisions: decisions,
                safe_payload: safe_shape(&Value::Object(properties.clone())),
            }),
        )?])
    }

    fn permission_replied(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(properties, "requestID", "permission reply ID")?;
        state.pending_permissions.remove(id);
        let reply = text(properties, "reply").ok_or_else(|| protocol_error("permission reply"))?;
        let decision_kind = match reply {
            "once" => ApprovalDecisionKind::AllowOnce,
            "always" => ApprovalDecisionKind::AllowSession,
            "reject" => ApprovalDecisionKind::Deny,
            _ => return Err(protocol_error("permission reply")),
        };
        let request_id = ProviderRequestId::new(id.to_string())
            .map_err(|_| protocol_error("permission reply ID"))?;
        Ok(vec![self.event_with_request(
            state,
            "permission.replied",
            None,
            None,
            Some(request_id.clone()),
            CanonicalEvent::RequestResolved(RequestResolvedEvent {
                request_id,
                state: RequestResolutionState::Resolved,
                decision: Some(ApprovalDecision {
                    kind: decision_kind,
                    provider_option_id: Some(reply.to_string()),
                    updated_tool_input: None,
                }),
            }),
        )?])
    }

    fn question_asked(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(properties, "id", "question request ID")?;
        let questions = properties
            .get("questions")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("questions"))?;
        if questions.is_empty() || questions.len() > MAX_QUESTIONS {
            return Err(protocol_error("questions"));
        }
        let mut normalized = Vec::with_capacity(questions.len());
        let mut mappings = Vec::with_capacity(questions.len());
        for (index, question) in questions.iter().enumerate() {
            let question = question
                .as_object()
                .ok_or_else(|| protocol_error("question"))?;
            let prompt = text(question, "question")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| protocol_error("question prompt"))?;
            let options = question
                .get("options")
                .and_then(Value::as_array)
                .ok_or_else(|| protocol_error("question options"))?;
            if options.len() > MAX_OPTIONS {
                return Err(protocol_error("question options"));
            }
            let mut normalized_options = Vec::with_capacity(options.len());
            let mut option_labels = Vec::with_capacity(options.len());
            for (option_index, option) in options.iter().enumerate() {
                let option = option
                    .as_object()
                    .ok_or_else(|| protocol_error("question option"))?;
                let label = text(option, "label")
                    .filter(|label| !label.trim().is_empty())
                    .ok_or_else(|| protocol_error("question option label"))?;
                option_labels.push(bounded(label, 512));
                normalized_options.push(UserInputOption {
                    id: format!("option-{option_index}"),
                    label: bounded(label, 512),
                    description: text(option, "description")
                        .map(|description| bounded(description, 2048)),
                });
            }
            let question_id = format!("question-{index}");
            let free_form_allowed = question
                .get("custom")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            mappings.push(OpenCodeQuestionMapping {
                id: question_id.clone(),
                option_labels,
                free_form_allowed,
            });
            normalized.push(UserInputQuestion {
                id: question_id,
                header: text(question, "header").map(|header| bounded(header, 256)),
                question: bounded(prompt, 4096),
                options: normalized_options,
                multiple: question
                    .get("multiple")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                free_form_allowed,
                required: true,
            });
        }
        let request_id = ProviderRequestId::new(id.to_string())
            .map_err(|_| protocol_error("question request ID"))?;
        state.pending_questions.insert(id.to_string(), mappings);
        Ok(vec![self.event_with_request(
            state,
            "question.asked",
            None,
            None,
            Some(request_id.clone()),
            CanonicalEvent::UserInputRequested(UserInputRequestedEvent {
                request_id,
                questions: normalized,
            }),
        )?])
    }

    fn question_replied(
        &self,
        state: &mut OpenCodeRouteState,
        properties: &Map<String, Value>,
        rejected: bool,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = identifier(properties, "requestID", "question reply ID")?;
        state.pending_questions.remove(id);
        let answers = if rejected {
            Vec::new()
        } else {
            properties
                .get("answers")
                .and_then(Value::as_array)
                .ok_or_else(|| protocol_error("question answers"))?
                .iter()
                .enumerate()
                .map(|(index, answer)| {
                    let selected = answer
                        .as_array()
                        .ok_or_else(|| protocol_error("question answer"))?
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(|value| bounded(value, 512))
                                .ok_or_else(|| protocol_error("question answer value"))
                        })
                        .collect::<ChatResult<Vec<_>>>()?;
                    Ok(UserInputAnswer {
                        question_id: format!("question-{index}"),
                        selected_option_ids: selected,
                        free_form_text: None,
                    })
                })
                .collect::<ChatResult<Vec<_>>>()?
        };
        let request_id = ProviderRequestId::new(id.to_string())
            .map_err(|_| protocol_error("question reply ID"))?;
        Ok(vec![self.event_with_request(
            state,
            if rejected {
                "question.rejected"
            } else {
                "question.replied"
            },
            None,
            None,
            Some(request_id.clone()),
            CanonicalEvent::UserInputResolved(UserInputResolvedEvent {
                request_id,
                state: if rejected {
                    RequestResolutionState::Interrupted
                } else {
                    RequestResolutionState::Resolved
                },
                answers,
            }),
        )?])
    }

    fn todo_updated(
        &self,
        state: &OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let todos = properties
            .get("todos")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("todo list"))?;
        if todos.len() > MAX_SAFE_COLLECTION {
            return Err(protocol_error("todo list size"));
        }
        let mut steps = Vec::new();
        for (index, todo) in todos.iter().enumerate() {
            let todo = todo.as_object().ok_or_else(|| protocol_error("todo"))?;
            let content = text(todo, "content")
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| protocol_error("todo content"))?;
            steps.push(PlanStep {
                id: text(todo, "id")
                    .map(|value| bounded(value, 256))
                    .unwrap_or_else(|| format!("opencode-todo-{index}")),
                text: bounded(content, 4096),
                status: activity_status(text(todo, "status")),
            });
        }
        let markdown = steps
            .iter()
            .map(|step| {
                format!(
                    "- [{}] {}",
                    if step.status == ActivityStatus::Completed {
                        "x"
                    } else {
                        " "
                    },
                    step.text
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(vec![self.event(
            state,
            "todo.updated",
            None,
            None,
            CanonicalEvent::PlanUpdated(PlanUpdatedEvent { markdown, steps }),
        )?])
    }

    fn mcp_status(
        &self,
        state: &OpenCodeRouteState,
        properties: &Map<String, Value>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let id = text(properties, "name")
            .or_else(|| text(properties, "id"))
            .ok_or_else(|| protocol_error("MCP server ID"))?;
        validate_identifier(id, "MCP server ID")?;
        Ok(vec![self.event(
            state,
            "mcp.status",
            None,
            None,
            CanonicalEvent::McpStatus(McpStatusEvent {
                server_id: id.to_string(),
                status: activity_status(text(properties, "status")),
                detail: text(properties, "message").map(|value| bounded(value, 4096)),
            }),
        )?])
    }

    fn item_event(
        &self,
        state: &OpenCodeRouteState,
        id: &str,
        kind: CanonicalItemKind,
        status: ActivityStatus,
        title: Option<String>,
        detail: Option<String>,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        let item = ItemLifecycleEvent {
            item_id: id.to_string(),
            kind,
            status,
            title,
            detail,
            safe_metadata: None,
        };
        self.item_lifecycle_event(state, id, item)
    }

    fn item_lifecycle_event(
        &self,
        state: &OpenCodeRouteState,
        id: &str,
        item: ItemLifecycleEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        validate_identifier(id, "item ID")?;
        let status = item.status;
        let event = match status {
            ActivityStatus::Pending | ActivityStatus::Active => CanonicalEvent::ItemStarted(item),
            ActivityStatus::Completed | ActivityStatus::Failed | ActivityStatus::Interrupted => {
                CanonicalEvent::ItemCompleted(item)
            }
            _ => CanonicalEvent::ItemUpdated(item),
        };
        self.event(
            state,
            "message.part.updated",
            ProviderItemId::new(id.to_string()).ok(),
            None,
            event,
        )
    }

    fn unknown(
        &self,
        state: &OpenCodeRouteState,
        source: &str,
        unknown: &str,
        value: &Map<String, Value>,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        self.event(
            state,
            source,
            None,
            None,
            CanonicalEvent::Unknown(UnknownEvent {
                source_type: format!("opencode/{unknown}"),
                summary: "OpenCode emitted an unsupported protocol value".to_string(),
                safe_payload: Some(safe_shape(&Value::Object(value.clone()))),
            }),
        )
    }

    fn event(
        &self,
        state: &OpenCodeRouteState,
        source: &str,
        provider_item_id: Option<ProviderItemId>,
        turn_id: Option<ChatTurnId>,
        event: CanonicalEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        self.event_with_request(state, source, provider_item_id, turn_id, None, event)
    }

    fn event_with_request(
        &self,
        state: &OpenCodeRouteState,
        source: &str,
        provider_item_id: Option<ProviderItemId>,
        turn_id: Option<ChatTurnId>,
        provider_request_id: Option<ProviderRequestId>,
        event: CanonicalEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        let sequence = self.next_event_id.fetch_add(1, Ordering::Relaxed);
        Ok(CanonicalRuntimeEvent {
            schema_version: CANONICAL_EVENT_SCHEMA_VERSION,
            event_id: ChatEventId::new(format!("{}:event:{sequence}", self.session_id.as_str()))
                .map_err(|_| protocol_error("event ID"))?,
            provider_family_id: ProviderFamilyId::new("opencode")
                .map_err(|_| protocol_error("provider family"))?,
            provider_instance_id: self.provider_instance_id.clone(),
            thread_id: self.thread_id.clone(),
            created_at: now_utc()?,
            turn_id: turn_id.or_else(|| state.active_turn_id.clone()),
            provider_turn_id: state
                .active_turn_id
                .as_ref()
                .and_then(|turn| ProviderTurnId::new(format!("opencode-{}", turn.as_str())).ok()),
            provider_item_id,
            provider_request_id,
            provider_task_id: None,
            provider_reference: Some(VersionedJson {
                schema_version: 1,
                value: match state.rollback_message_id.as_deref() {
                    Some(message_id) => json!({
                        "messageId": message_id,
                        "partId": null,
                        "source": source,
                        "sessionId": state.provider_thread_id,
                    }),
                    None => json!({ "source": source, "sessionId": state.provider_thread_id }),
                },
            }),
            event,
            redacted_diagnostic: None,
        })
    }
}

fn changed_files_from_paths(values: &[Value]) -> ChatResult<Vec<ChangedFileSummary>> {
    if values.len() > MAX_SAFE_COLLECTION {
        return Err(protocol_error("patch file count"));
    }
    values
        .iter()
        .map(|value| {
            let path = value
                .as_str()
                .ok_or_else(|| protocol_error("patch file path"))?;
            Ok(ChangedFileSummary {
                relative_path: bounded(path, 4096),
                previous_relative_path: None,
                additions: None,
                deletions: None,
                binary: false,
                status: "modified".to_string(),
            })
        })
        .collect()
}

fn part_id(part: &Map<String, Value>) -> Option<ProviderItemId> {
    text(part, "id").and_then(|id| ProviderItemId::new(id.to_string()).ok())
}

fn tool_kind(tool: &str) -> CanonicalItemKind {
    let tool = tool.to_ascii_lowercase();
    if tool.contains("bash") || tool.contains("shell") || tool.contains("command") {
        CanonicalItemKind::CommandExecution
    } else if tool.contains("edit") || tool.contains("write") || tool.contains("patch") {
        CanonicalItemKind::FileChange
    } else if tool.contains("web") || tool.contains("search") {
        CanonicalItemKind::WebSearch
    } else if tool.contains("mcp") {
        CanonicalItemKind::McpToolCall
    } else if tool.contains("task") || tool.contains("agent") {
        CanonicalItemKind::CollaborationTask
    } else {
        CanonicalItemKind::DynamicToolCall
    }
}

fn request_kind(permission: &str) -> CanonicalRequestKind {
    match permission {
        "bash" => CanonicalRequestKind::CommandExecution,
        "edit" => CanonicalRequestKind::FileChange,
        "read" => CanonicalRequestKind::FileRead,
        "external_directory" => CanonicalRequestKind::FileRead,
        _ => CanonicalRequestKind::DynamicTool,
    }
}

fn activity_status(status: Option<&str>) -> ActivityStatus {
    match status {
        Some("pending") => ActivityStatus::Pending,
        Some("running" | "in_progress" | "active" | "busy" | "connected") => ActivityStatus::Active,
        Some("completed" | "complete" | "done" | "idle") => ActivityStatus::Completed,
        Some("failed" | "error" | "disconnected") => ActivityStatus::Failed,
        Some("cancelled" | "canceled") => ActivityStatus::Interrupted,
        _ => ActivityStatus::Unknown,
    }
}

fn identifier<'a>(object: &'a Map<String, Value>, key: &str, label: &str) -> ChatResult<&'a str> {
    let value = text(object, key).ok_or_else(|| protocol_error(label))?;
    validate_identifier(value, label)?;
    Ok(value)
}

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn bounded(value: &str, maximum: usize) -> String {
    if value.len() <= maximum {
        return value.to_string();
    }
    let mut end = maximum;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn safe_shape(value: &Value) -> VersionedJson {
    VersionedJson {
        schema_version: 1,
        value: safe_value(value, 0),
    }
}

fn safe_value(value: &Value, depth: usize) -> Value {
    if depth >= 6 {
        return Value::String("[nested]".to_string());
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
        Value::String(value) => Value::String(bounded(value, 4_096)),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .take(MAX_SAFE_COLLECTION)
                .map(|value| safe_value(value, depth + 1))
                .collect(),
        ),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .take(MAX_SAFE_COLLECTION)
                .filter(|(key, _)| !sensitive_key(key))
                .map(|(key, value)| (key.clone(), safe_value(value, depth + 1)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
    }
}

fn sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    [
        "authorization",
        "password",
        "token",
        "secret",
        "credential",
        "api_key",
    ]
    .iter()
    .any(|needle| key.contains(needle))
}

fn now_utc() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .map_err(|_| protocol_error("timestamp"))
}
