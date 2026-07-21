//! Codex app-server notification normalization.

use crate::chat::events::*;
use crate::chat::models::*;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_PROVIDER_TEXT_BYTES: usize = 64 * 1024;
const MAX_UNKNOWN_KEYS: usize = 32;

#[derive(Clone, Debug)]
pub struct CodexRouteState {
    pub provider_thread_id: Option<String>,
    pub active_chat_turn_id: Option<ChatTurnId>,
    pub active_provider_turn_id: Option<String>,
    pub session_state: ProviderSessionState,
    pub thread_state: ChatThreadState,
    pub modes: TurnModeSnapshot,
    pub effective_model_id: Option<ModelId>,
    pub stream_indexes: HashMap<(String, ContentStreamKind), u32>,
}

impl CodexRouteState {
    pub fn new(modes: TurnModeSnapshot, model_id: Option<ModelId>) -> Self {
        Self {
            provider_thread_id: None,
            active_chat_turn_id: None,
            active_provider_turn_id: None,
            session_state: ProviderSessionState::Starting,
            thread_state: ChatThreadState::Active,
            modes,
            effective_model_id: model_id,
            stream_indexes: HashMap::new(),
        }
    }
}

pub struct CodexEventNormalizer {
    provider_instance_id: ProviderInstanceId,
    thread_id: ChatThreadId,
    session_id: ProviderSessionId,
    next_event_id: AtomicU64,
    next_fallback_id: AtomicU64,
}

impl CodexEventNormalizer {
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
            next_fallback_id: AtomicU64::new(1),
        }
    }

    pub fn normalize_notification(
        &self,
        state: &mut CodexRouteState,
        method: &str,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = params.as_object();
        let provider_thread = object.and_then(|value| text(value, "threadId"));
        if let (Some(expected), Some(actual)) =
            (state.provider_thread_id.as_deref(), provider_thread)
        {
            if expected != actual && is_child_thread_lifecycle(method) {
                return Ok(Vec::new());
            }
        }
        match method {
            "thread/started" => self.thread_started(state, params),
            "thread/status/changed" => self.thread_status_changed(state, params),
            "thread/name/updated" => self.thread_metadata(state, params),
            "thread/tokenUsage/updated" => self.thread_usage(state, params),
            "turn/started" => self.turn_started(state, params),
            "turn/completed" => self.turn_completed(state, params),
            "turn/plan/updated" => self.plan_updated(state, params),
            "turn/diff/updated" | "item/fileChange/patchUpdated" => {
                self.diff_updated(state, method, params)
            }
            "item/started" => self.item_lifecycle(state, params, false),
            "item/completed" => self.item_lifecycle(state, params, true),
            "item/agentMessage/delta" => self.content_delta(
                state,
                params,
                ContentStreamKind::AssistantText,
                "delta",
                None,
            ),
            "item/reasoning/summaryTextDelta" => self.content_delta(
                state,
                params,
                ContentStreamKind::ReasoningSummary,
                "delta",
                Some("summaryIndex"),
            ),
            "item/reasoning/textDelta" => self.content_delta(
                state,
                params,
                ContentStreamKind::ReasoningText,
                "delta",
                Some("contentIndex"),
            ),
            "item/plan/delta" => self.proposed_plan_delta(state, params),
            "item/commandExecution/outputDelta" => self.content_delta(
                state,
                params,
                ContentStreamKind::CommandOutput,
                "delta",
                None,
            ),
            "item/fileChange/outputDelta" => self.content_delta(
                state,
                params,
                ContentStreamKind::FileChangeOutput,
                "delta",
                None,
            ),
            "item/mcpToolCall/progress" => self.tool_progress(state, params),
            "hook/started" => self.hook_lifecycle(state, params, false),
            "hook/completed" => self.hook_lifecycle(state, params, true),
            "thread/compacted" => self.context_compacted(state, params),
            "account/updated" | "account/login/completed" => {
                self.authentication_status(state, method, params)
            }
            "account/rateLimits/updated" => self.rate_limit_status(state, params),
            "mcpServer/startupStatus/updated" => self.mcp_status(state, params),
            "mcpServer/oauthLogin/completed" => self.mcp_oauth(state, params),
            "model/rerouted" => self.model_rerouted(state, params),
            "configWarning" => self.notification(
                state,
                params,
                "codex_configuration_warning",
                CanonicalNotificationKind::Configuration,
            ),
            "deprecationNotice" => self.notification(
                state,
                params,
                "codex_deprecation",
                CanonicalNotificationKind::Deprecation,
            ),
            "warning"
            | "guardianWarning"
            | "windows/worldWritableWarning"
            | "windowsSandbox/setupCompleted"
            | "model/verification"
            | "model/safetyBuffering/updated" => self.notification(
                state,
                params,
                "codex_runtime_warning",
                CanonicalNotificationKind::Runtime,
            ),
            "error" => self.runtime_error(state, params),
            "thread/closed" => self.session_closed(state, params),
            "serverRequest/resolved"
            | "thread/archived"
            | "thread/unarchived"
            | "thread/deleted"
            | "item/reasoning/summaryPartAdded"
            | "item/commandExecution/terminalInteraction"
            | "item/autoApprovalReview/started"
            | "item/autoApprovalReview/completed" => Ok(Vec::new()),
            _ => Ok(vec![self.event(
                state,
                method,
                route_turn(state, object),
                None,
                None,
                CanonicalEvent::Unknown(UnknownEvent {
                    source_type: method.to_string(),
                    summary: "Codex emitted an unsupported notification".to_string(),
                    safe_payload: bounded_shape(&params),
                }),
            )?]),
        }
    }

    fn thread_started(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "thread/started")?;
        let thread = object
            .get("thread")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("thread/started.thread"))?;
        let provider_thread_id = required_text(thread, "id")?.to_string();
        state.provider_thread_id = Some(provider_thread_id.clone());
        let provider_id = ProviderThreadId::new(provider_thread_id.clone())
            .unwrap_or_else(|_| self.fallback_provider_thread_id());
        let provider_title = text(thread, "name")
            .or_else(|| text(thread, "preview"))
            .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES));
        let resume_cursor = VersionedJson {
            schema_version: 1,
            value: json!({ "threadId": provider_thread_id }),
        };
        Ok(vec![
            self.event(
                state,
                "thread/started",
                None,
                None,
                None,
                CanonicalEvent::ThreadStarted(ThreadStartedEvent {
                    provider_thread_id: provider_id.clone(),
                    title: None,
                }),
            )?,
            self.event(
                state,
                "thread/started",
                None,
                None,
                None,
                CanonicalEvent::ThreadMetadataUpdated(ThreadMetadataUpdatedEvent {
                    title: None,
                    provider_thread_id: Some(provider_id),
                    resume_cursor: Some(resume_cursor),
                    metadata: provider_title.map(|title| VersionedJson {
                        schema_version: 1,
                        value: json!({ "providerTitle": title }),
                    }),
                }),
            )?,
        ])
    }

    fn thread_status_changed(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "thread/status/changed")?;
        let status = object
            .get("status")
            .and_then(status_text)
            .unwrap_or("active");
        let next = match status {
            "idle" => ChatThreadState::Idle,
            "error" | "failed" => ChatThreadState::Error,
            "closed" => ChatThreadState::Closed,
            _ => ChatThreadState::Active,
        };
        let previous = state.thread_state;
        state.thread_state = next;
        Ok(vec![self.event(
            state,
            "thread/status/changed",
            None,
            None,
            None,
            CanonicalEvent::ThreadStateChanged(ThreadStateChangedEvent {
                previous_state: previous,
                state: next,
                reason: None,
            }),
        )?])
    }

    fn thread_metadata(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "thread/name/updated")?;
        let provider_thread_id = state
            .provider_thread_id
            .as_deref()
            .and_then(|value| ProviderThreadId::new(value.to_string()).ok());
        Ok(vec![self.event(
            state,
            "thread/name/updated",
            None,
            None,
            None,
            CanonicalEvent::ThreadMetadataUpdated(ThreadMetadataUpdatedEvent {
                // Provider names are metadata only. Ganbaru owns the visible title.
                title: None,
                provider_thread_id,
                resume_cursor: None,
                metadata: text(object, "name").map(|name| VersionedJson {
                    schema_version: 1,
                    value: json!({ "providerName": bounded_text(name, MAX_PROVIDER_TEXT_BYTES) }),
                }),
            }),
        )?])
    }

    fn thread_usage(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "thread/tokenUsage/updated")?;
        let usage = object
            .get("tokenUsage")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("thread/tokenUsage/updated.tokenUsage"))?;
        let total = usage
            .get("total")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("thread/tokenUsage/updated.total"))?;
        let event = ThreadUsageUpdatedEvent {
            input_tokens: unsigned(total, "inputTokens"),
            output_tokens: unsigned(total, "outputTokens"),
            cached_input_tokens: unsigned(total, "cachedInputTokens"),
            context_tokens: unsigned(total, "totalTokens"),
            context_limit: unsigned(usage, "modelContextWindow"),
            cost: None,
        };
        Ok(vec![self.event(
            state,
            "thread/tokenUsage/updated",
            route_turn(state, Some(object)),
            None,
            None,
            CanonicalEvent::ThreadUsageUpdated(event),
        )?])
    }

    fn turn_started(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "turn/started")?;
        let turn = object
            .get("turn")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("turn/started.turn"))?;
        let provider_turn_id = required_text(turn, "id")?.to_string();
        state.active_provider_turn_id = Some(provider_turn_id.clone());
        state.session_state = ProviderSessionState::Active;
        let chat_turn_id = state.active_chat_turn_id.clone();
        let provider_turn = provider_identifier::<ProviderTurnId>(&provider_turn_id)
            .unwrap_or_else(|| self.fallback_provider_turn_id());
        Ok(vec![self.event(
            state,
            "turn/started",
            chat_turn_id,
            Some(provider_turn.clone()),
            None,
            CanonicalEvent::TurnStarted(TurnStartedEvent {
                provider_turn_id: Some(provider_turn),
                state: ChatTurnState::Active,
                modes: state.modes,
                model_id: state.effective_model_id.clone(),
                model_options: Vec::new(),
            }),
        )?])
    }

    fn turn_completed(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "turn/completed")?;
        let turn = object
            .get("turn")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("turn/completed.turn"))?;
        let provider_turn_id = required_text(turn, "id")?;
        let provider_turn = provider_identifier::<ProviderTurnId>(provider_turn_id);
        let status = required_text(turn, "status")?;
        let turn_state = match status {
            "completed" => ChatTurnState::Completed,
            "interrupted" => ChatTurnState::Interrupted,
            _ => ChatTurnState::Failed,
        };
        let stop_reason = turn
            .get("error")
            .and_then(Value::as_object)
            .and_then(|error| text(error, "message"))
            .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES));
        let chat_turn = state.active_chat_turn_id.clone();
        state.session_state = ProviderSessionState::Ready;
        state.active_provider_turn_id = None;
        state.stream_indexes.clear();
        Ok(vec![self.event(
            state,
            "turn/completed",
            chat_turn,
            provider_turn,
            None,
            CanonicalEvent::TurnCompleted(TurnCompletedEvent {
                state: turn_state,
                stop_reason,
                usage: None,
                changed_files: Vec::new(),
            }),
        )?])
    }

    fn plan_updated(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "turn/plan/updated")?;
        let steps = object
            .get("plan")
            .and_then(Value::as_array)
            .ok_or_else(|| protocol_error("turn/plan/updated.plan"))?
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                let value = value.as_object()?;
                let step = text(value, "step")?;
                Some(PlanStep {
                    id: format!("codex-plan-step-{index}"),
                    text: bounded_text(step, MAX_PROVIDER_TEXT_BYTES),
                    status: match text(value, "status") {
                        Some("completed") => ActivityStatus::Completed,
                        Some("inProgress") => ActivityStatus::Active,
                        _ => ActivityStatus::Pending,
                    },
                })
            })
            .collect::<Vec<_>>();
        let markdown = steps
            .iter()
            .map(|step| format!("- {}", step.text))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(vec![self.event(
            state,
            "turn/plan/updated",
            route_turn(state, Some(object)),
            None,
            None,
            CanonicalEvent::PlanUpdated(PlanUpdatedEvent { markdown, steps }),
        )?])
    }

    fn diff_updated(
        &self,
        state: &CodexRouteState,
        method: &str,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, method)?;
        let diff = text(object, "diff")
            .or_else(|| text(object, "patch"))
            .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES));
        Ok(vec![self.event(
            state,
            method,
            route_turn(state, Some(object)),
            provider_turn_from(object),
            provider_item_from(object),
            CanonicalEvent::DiffUpdated(DiffUpdatedEvent {
                source: "codex".to_string(),
                files: Vec::new(),
                provider_diff: diff,
            }),
        )?])
    }

    fn item_lifecycle(
        &self,
        state: &CodexRouteState,
        params: Value,
        completed: bool,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let method = if completed {
            "item/completed"
        } else {
            "item/started"
        };
        let object = required_object(&params, method)?;
        let item = object
            .get("item")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("item lifecycle item"))?;
        let raw_id = required_text(item, "id")?;
        let item_id = provider_identifier::<ProviderItemId>(raw_id)
            .unwrap_or_else(|| self.fallback_provider_item_id());
        let raw_kind = required_text(item, "type")?;
        let kind = item_kind(raw_kind);
        if completed && kind == CanonicalItemKind::Plan {
            if let Some(plan) = text(item, "text") {
                return Ok(vec![self.event(
                    state,
                    method,
                    route_turn(state, Some(object)),
                    provider_turn_from(object),
                    Some(item_id.clone()),
                    CanonicalEvent::ProposedPlanCompleted(ProposedPlanCompletedEvent {
                        plan_id: item_id.as_str().to_string(),
                        markdown: bounded_text(plan, MAX_PROVIDER_TEXT_BYTES),
                    }),
                )?]);
            }
        }
        let status = item_status(item, completed);
        let title = item_title(item, raw_kind);
        let detail = item_detail(item, raw_kind);
        let metadata = item_safe_metadata(item, raw_kind);
        let lifecycle = ItemLifecycleEvent {
            item_id: item_id.as_str().to_string(),
            kind,
            status,
            title,
            detail,
            safe_metadata: metadata,
        };
        let mut events = vec![self.event(
            state,
            method,
            route_turn(state, Some(object)),
            provider_turn_from(object),
            Some(item_id.clone()),
            if completed {
                CanonicalEvent::ItemCompleted(lifecycle)
            } else {
                CanonicalEvent::ItemStarted(lifecycle)
            },
        )?];
        if raw_kind == "collabAgentToolCall" {
            events.push(
                self.event(
                    state,
                    method,
                    route_turn(state, Some(object)),
                    provider_turn_from(object),
                    Some(item_id.clone()),
                    CanonicalEvent::TaskLifecycle(TaskLifecycleEvent {
                        task_id: item_id.as_str().to_string(),
                        parent_task_id: None,
                        status,
                        title: text(item, "tool").unwrap_or("Codex task").to_string(),
                        detail: text(item, "prompt")
                            .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
                        safe_metadata: None,
                    }),
                )?,
            );
        }
        Ok(events)
    }

    fn content_delta(
        &self,
        state: &mut CodexRouteState,
        params: Value,
        kind: ContentStreamKind,
        delta_key: &str,
        index_key: Option<&str>,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "content delta")?;
        let raw_item = required_text(object, "itemId")?;
        let item_id = provider_identifier::<ProviderItemId>(raw_item)
            .unwrap_or_else(|| self.fallback_provider_item_id());
        let delta = bounded_text(required_text(object, delta_key)?, MAX_PROVIDER_TEXT_BYTES);
        if delta.is_empty() {
            return Ok(Vec::new());
        }
        let content_index = index_key
            .and_then(|key| unsigned(object, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or_else(|| {
                *state
                    .stream_indexes
                    .entry((item_id.as_str().to_string(), kind))
                    .or_insert(0)
            });
        Ok(vec![self.event(
            state,
            "content/delta",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            Some(item_id.clone()),
            CanonicalEvent::ContentDelta(ContentDeltaEvent {
                item_id: item_id.as_str().to_string(),
                stream_kind: kind,
                content_index,
                delta,
            }),
        )?])
    }

    fn proposed_plan_delta(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "item/plan/delta")?;
        let raw_item = required_text(object, "itemId")?;
        let item_id = provider_identifier::<ProviderItemId>(raw_item)
            .unwrap_or_else(|| self.fallback_provider_item_id());
        let index = *state
            .stream_indexes
            .entry((item_id.as_str().to_string(), ContentStreamKind::PlanText))
            .or_insert(0);
        Ok(vec![self.event(
            state,
            "item/plan/delta",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            Some(item_id.clone()),
            CanonicalEvent::ProposedPlanDelta(ProposedPlanDeltaEvent {
                plan_id: item_id.as_str().to_string(),
                delta: bounded_text(required_text(object, "delta")?, MAX_PROVIDER_TEXT_BYTES),
                content_index: index,
            }),
        )?])
    }

    fn tool_progress(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "item/mcpToolCall/progress")?;
        let item_id = required_text(object, "itemId")?;
        Ok(vec![self.event(
            state,
            "item/mcpToolCall/progress",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            provider_identifier::<ProviderItemId>(item_id),
            CanonicalEvent::ToolProgress(ToolProgressEvent {
                tool_id: item_id.to_string(),
                status: ActivityStatus::Active,
                title: "MCP tool".to_string(),
                progress: None,
                summary: text(object, "message")
                    .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
            }),
        )?])
    }

    fn hook_lifecycle(
        &self,
        state: &CodexRouteState,
        params: Value,
        completed: bool,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "hook lifecycle")?;
        let run = object
            .get("run")
            .and_then(Value::as_object)
            .ok_or_else(|| protocol_error("hook lifecycle run"))?;
        let id = required_text(run, "id")?;
        Ok(vec![self.event(
            state,
            if completed {
                "hook/completed"
            } else {
                "hook/started"
            },
            route_turn(state, Some(object)),
            provider_turn_from(object),
            None,
            CanonicalEvent::HookLifecycle(HookLifecycleEvent {
                hook_id: id.to_string(),
                status: if completed {
                    status_text(run.get("status").unwrap_or(&Value::Null))
                        .map(activity_status)
                        .unwrap_or(ActivityStatus::Completed)
                } else {
                    ActivityStatus::Active
                },
                title: text(run, "eventName").unwrap_or("Codex hook").to_string(),
                detail: text(run, "statusMessage")
                    .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
            }),
        )?])
    }

    fn context_compacted(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "thread/compacted")?;
        let item_id = self.fallback_provider_item_id();
        Ok(vec![self.event(
            state,
            "thread/compacted",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            Some(item_id.clone()),
            CanonicalEvent::ItemCompleted(ItemLifecycleEvent {
                item_id: item_id.as_str().to_string(),
                kind: CanonicalItemKind::ContextCompaction,
                status: ActivityStatus::Completed,
                title: Some("Context compacted".to_string()),
                detail: None,
                safe_metadata: None,
            }),
        )?])
    }

    fn authentication_status(
        &self,
        state: &CodexRouteState,
        method: &str,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, method)?;
        let account = object.get("account").and_then(Value::as_object);
        let authenticated = account.is_some()
            || object
                .get("success")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        let account_label = account
            .and_then(|value| text(value, "email").or_else(|| text(value, "type")))
            .map(str::to_string);
        Ok(vec![self.event(
            state,
            method,
            None,
            None,
            None,
            CanonicalEvent::AuthenticationStatus(AuthenticationStatusEvent {
                authenticated,
                account_label,
                action_required: !authenticated,
                detail: None,
            }),
        )?])
    }

    fn rate_limit_status(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "account/rateLimits/updated")?;
        let limits = object.get("rateLimits").unwrap_or(&Value::Null);
        Ok(vec![self.event(
            state,
            "account/rateLimits/updated",
            None,
            None,
            None,
            CanonicalEvent::RateLimitStatus(RateLimitStatusEvent {
                limited: limits
                    .get("limitReached")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                resets_at: None,
                detail: None,
                provider_data: bounded_shape(limits),
            }),
        )?])
    }

    fn mcp_status(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "mcpServer/startupStatus/updated")?;
        let server_id = text(object, "serverName")
            .or_else(|| text(object, "serverId"))
            .unwrap_or("unknown");
        let status = text(object, "status")
            .map(activity_status)
            .unwrap_or(ActivityStatus::Unknown);
        Ok(vec![self.event(
            state,
            "mcpServer/startupStatus/updated",
            None,
            None,
            None,
            CanonicalEvent::McpStatus(McpStatusEvent {
                server_id: server_id.to_string(),
                status,
                detail: text(object, "message")
                    .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
            }),
        )?])
    }

    fn mcp_oauth(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "mcpServer/oauthLogin/completed")?;
        Ok(vec![self.event(
            state,
            "mcpServer/oauthLogin/completed",
            None,
            None,
            None,
            CanonicalEvent::McpOauthCompleted(McpOauthCompletedEvent {
                server_id: text(object, "serverName").unwrap_or("unknown").to_string(),
                successful: object
                    .get("success")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                detail: text(object, "error")
                    .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
            }),
        )?])
    }

    fn model_rerouted(
        &self,
        state: &mut CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "model/rerouted")?;
        let requested = ModelId::new(required_text(object, "fromModel")?.to_string())
            .map_err(|_| protocol_error("model/rerouted.fromModel"))?;
        let effective = ModelId::new(required_text(object, "toModel")?.to_string())
            .map_err(|_| protocol_error("model/rerouted.toModel"))?;
        state.effective_model_id = Some(effective.clone());
        Ok(vec![self.event(
            state,
            "model/rerouted",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            None,
            CanonicalEvent::ModelRerouted(ModelReroutedEvent {
                requested_model_id: requested,
                effective_model_id: effective,
                reason: object.get("reason").map(provider_reason),
            }),
        )?])
    }

    fn notification(
        &self,
        state: &CodexRouteState,
        params: Value,
        code: &str,
        kind: CanonicalNotificationKind,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, code)?;
        let title = text(object, "summary")
            .or_else(|| text(object, "message"))
            .unwrap_or("Codex warning");
        let notification = NotificationEvent {
            code: code.to_string(),
            title: bounded_text(title, MAX_PROVIDER_TEXT_BYTES),
            detail: text(object, "details")
                .or_else(|| text(object, "detail"))
                .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES)),
        };
        let event = match kind {
            CanonicalNotificationKind::Configuration => {
                CanonicalEvent::ConfigurationWarning(notification)
            }
            CanonicalNotificationKind::Deprecation => {
                CanonicalEvent::DeprecationNotice(notification)
            }
            CanonicalNotificationKind::Runtime => CanonicalEvent::RuntimeWarning(notification),
        };
        Ok(vec![self.event(state, code, None, None, None, event)?])
    }

    fn runtime_error(
        &self,
        state: &CodexRouteState,
        params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        let object = required_object(&params, "error")?;
        let error = object.get("error").and_then(Value::as_object);
        let message = error
            .and_then(|value| text(value, "message"))
            .unwrap_or("Codex turn failed");
        Ok(vec![self.event(
            state,
            "error",
            route_turn(state, Some(object)),
            provider_turn_from(object),
            None,
            CanonicalEvent::RuntimeError(RuntimeErrorEvent {
                code: "codex_runtime_error".to_string(),
                message: bounded_text(message, MAX_PROVIDER_TEXT_BYTES),
                recoverable: object
                    .get("willRetry")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                safe_details: error.map(|value| VersionedJson {
                    schema_version: 1,
                    value: json!({ "keys": bounded_keys(value) }),
                }),
            }),
        )?])
    }

    fn session_closed(
        &self,
        state: &mut CodexRouteState,
        _params: Value,
    ) -> ChatResult<Vec<CanonicalRuntimeEvent>> {
        state.session_state = ProviderSessionState::Stopped;
        Ok(vec![self.event(
            state,
            "thread/closed",
            None,
            None,
            None,
            CanonicalEvent::SessionExited(SessionExitedEvent {
                session_id: self.session_id.clone(),
                expected: false,
                exit_code: None,
                reason: Some("Codex closed the thread".to_string()),
            }),
        )?])
    }

    pub fn malformed_event(
        &self,
        state: &CodexRouteState,
        reason: &str,
        byte_length: usize,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        self.event(
            state,
            "malformed",
            None,
            None,
            None,
            CanonicalEvent::RuntimeError(RuntimeErrorEvent {
                code: "codex_malformed_protocol".to_string(),
                message: bounded_text(reason, MAX_PROVIDER_TEXT_BYTES),
                recoverable: true,
                safe_details: Some(VersionedJson {
                    schema_version: 1,
                    value: json!({ "byteLength": byte_length }),
                }),
            }),
        )
    }

    pub fn request_event(
        &self,
        state: &CodexRouteState,
        provider_request_id: ProviderRequestId,
        provider_turn_id: Option<ProviderTurnId>,
        provider_item_id: Option<ProviderItemId>,
        turn_id: Option<ChatTurnId>,
        event: CanonicalEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        let mut runtime = self.event(
            state,
            "server/request",
            turn_id,
            provider_turn_id,
            provider_item_id,
            event,
        )?;
        runtime.provider_request_id = Some(provider_request_id);
        Ok(runtime)
    }

    pub(super) fn session_id(&self) -> ProviderSessionId {
        self.session_id.clone()
    }

    pub(super) fn event(
        &self,
        _state: &CodexRouteState,
        method: &str,
        turn_id: Option<ChatTurnId>,
        provider_turn_id: Option<ProviderTurnId>,
        provider_item_id: Option<ProviderItemId>,
        event: CanonicalEvent,
    ) -> ChatResult<CanonicalRuntimeEvent> {
        let sequence = self.next_event_id.fetch_add(1, Ordering::Relaxed);
        Ok(CanonicalRuntimeEvent {
            schema_version: CANONICAL_EVENT_SCHEMA_VERSION,
            event_id: ChatEventId::new(format!("{}:{sequence}", self.session_id.as_str()))
                .map_err(identifier_error)?,
            provider_family_id: ProviderFamilyId::new("codex").map_err(identifier_error)?,
            provider_instance_id: self.provider_instance_id.clone(),
            thread_id: self.thread_id.clone(),
            created_at: now_utc()?,
            turn_id,
            provider_turn_id,
            provider_item_id,
            provider_request_id: None,
            provider_task_id: None,
            provider_reference: Some(VersionedJson {
                schema_version: 1,
                value: json!({ "method": bounded_text(method, 256) }),
            }),
            event,
            redacted_diagnostic: None,
        })
    }

    fn fallback_provider_thread_id(&self) -> ProviderThreadId {
        ProviderThreadId::new(self.fallback_id("thread"))
            .expect("bounded fallback provider thread ID must be valid")
    }

    fn fallback_provider_turn_id(&self) -> ProviderTurnId {
        ProviderTurnId::new(self.fallback_id("turn"))
            .expect("bounded fallback provider turn ID must be valid")
    }

    fn fallback_provider_item_id(&self) -> ProviderItemId {
        ProviderItemId::new(self.fallback_id("item"))
            .expect("bounded fallback provider item ID must be valid")
    }

    fn fallback_id(&self, kind: &str) -> String {
        format!(
            "codex-{kind}-{}",
            self.next_fallback_id.fetch_add(1, Ordering::Relaxed)
        )
    }
}

#[derive(Clone, Copy)]
enum CanonicalNotificationKind {
    Configuration,
    Deprecation,
    Runtime,
}

trait ProviderIdentifier: Sized {
    fn parse(value: String) -> Result<Self, String>;
}

impl ProviderIdentifier for ProviderTurnId {
    fn parse(value: String) -> Result<Self, String> {
        Self::new(value)
    }
}

impl ProviderIdentifier for ProviderItemId {
    fn parse(value: String) -> Result<Self, String> {
        Self::new(value)
    }
}

fn provider_identifier<T: ProviderIdentifier>(value: &str) -> Option<T> {
    T::parse(value.to_string()).ok()
}

fn required_object<'a>(value: &'a Value, label: &str) -> ChatResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| protocol_error(label))
}

fn required_text<'a>(object: &'a Map<String, Value>, key: &str) -> ChatResult<&'a str> {
    text(object, key).ok_or_else(|| protocol_error(key))
}

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

fn unsigned(object: &Map<String, Value>, key: &str) -> Option<u64> {
    object.get(key).and_then(Value::as_u64)
}

fn status_text(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.as_object().and_then(|value| text(value, "type")))
}

fn route_turn(state: &CodexRouteState, object: Option<&Map<String, Value>>) -> Option<ChatTurnId> {
    let provider_turn = object.and_then(|value| text(value, "turnId"));
    match (provider_turn, state.active_provider_turn_id.as_deref()) {
        (Some(actual), Some(expected)) if actual != expected => None,
        _ => state.active_chat_turn_id.clone(),
    }
}

fn provider_turn_from(object: &Map<String, Value>) -> Option<ProviderTurnId> {
    text(object, "turnId").and_then(provider_identifier)
}

fn provider_item_from(object: &Map<String, Value>) -> Option<ProviderItemId> {
    text(object, "itemId").and_then(provider_identifier)
}

fn item_kind(value: &str) -> CanonicalItemKind {
    match value {
        "userMessage" => CanonicalItemKind::UserMessage,
        "agentMessage" => CanonicalItemKind::AssistantMessage,
        "reasoning" => CanonicalItemKind::Reasoning,
        "plan" => CanonicalItemKind::Plan,
        "commandExecution" => CanonicalItemKind::CommandExecution,
        "fileChange" => CanonicalItemKind::FileChange,
        "mcpToolCall" => CanonicalItemKind::McpToolCall,
        "dynamicToolCall" => CanonicalItemKind::DynamicToolCall,
        "collabAgentToolCall" | "subAgentActivity" => CanonicalItemKind::CollaborationTask,
        "webSearch" => CanonicalItemKind::WebSearch,
        "imageView" | "imageGeneration" => CanonicalItemKind::ImageView,
        "enteredReviewMode" | "exitedReviewMode" => CanonicalItemKind::ReviewTransition,
        "contextCompaction" => CanonicalItemKind::ContextCompaction,
        _ => CanonicalItemKind::Unknown,
    }
}

fn item_status(item: &Map<String, Value>, completed: bool) -> ActivityStatus {
    if let Some(status) = item.get("status").and_then(status_text) {
        return activity_status(status);
    }
    if completed {
        ActivityStatus::Completed
    } else {
        ActivityStatus::Active
    }
}

fn activity_status(value: &str) -> ActivityStatus {
    match value {
        "pending" | "notStarted" => ActivityStatus::Pending,
        "inProgress" | "running" | "active" => ActivityStatus::Active,
        "waiting" => ActivityStatus::Waiting,
        "completed" | "success" | "succeeded" => ActivityStatus::Completed,
        "interrupted" | "cancelled" | "canceled" | "declined" => ActivityStatus::Interrupted,
        "failed" | "error" => ActivityStatus::Failed,
        _ => ActivityStatus::Unknown,
    }
}

fn item_title(item: &Map<String, Value>, kind: &str) -> Option<String> {
    let title = match kind {
        "commandExecution" => text(item, "command"),
        "mcpToolCall" => text(item, "tool"),
        "dynamicToolCall" => text(item, "tool"),
        "collabAgentToolCall" => text(item, "tool"),
        "webSearch" => text(item, "query"),
        "imageView" => Some("Viewed image"),
        "fileChange" => Some("File changes"),
        "reasoning" => Some("Reasoning"),
        "plan" => Some("Proposed plan"),
        "agentMessage" => Some("Assistant response"),
        "userMessage" => Some("User message"),
        "enteredReviewMode" => Some("Entered review"),
        "exitedReviewMode" => Some("Exited review"),
        _ => Some("Codex activity"),
    }?;
    Some(bounded_text(title, 512))
}

fn item_detail(item: &Map<String, Value>, kind: &str) -> Option<String> {
    let value = match kind {
        "commandExecution" => text(item, "aggregatedOutput").map(str::to_string),
        "agentMessage" | "plan" => text(item, "text").map(str::to_string),
        "webSearch" => item.get("action").map(provider_reason),
        "collabAgentToolCall" => text(item, "prompt").map(str::to_string),
        "imageView" => text(item, "path").map(str::to_string),
        _ => None,
    }?;
    Some(bounded_text(&value, MAX_PROVIDER_TEXT_BYTES))
}

fn item_safe_metadata(item: &Map<String, Value>, kind: &str) -> Option<VersionedJson> {
    let value = match kind {
        "commandExecution" => json!({
            "exitCode": item.get("exitCode"),
            "durationMs": item.get("durationMs"),
        }),
        "fileChange" => json!({
            "changeCount": item.get("changes").and_then(Value::as_array).map(Vec::len),
        }),
        "mcpToolCall" => json!({
            "server": text(item, "server"),
            "tool": text(item, "tool"),
            "durationMs": item.get("durationMs"),
        }),
        "dynamicToolCall" => json!({
            "tool": text(item, "tool"),
            "namespace": text(item, "namespace"),
            "durationMs": item.get("durationMs"),
        }),
        _ => return None,
    };
    Some(VersionedJson {
        schema_version: 1,
        value,
    })
}

fn provider_reason(value: &Value) -> String {
    match value {
        Value::String(value) => bounded_text(value, MAX_PROVIDER_TEXT_BYTES),
        Value::Object(value) => text(value, "message")
            .or_else(|| text(value, "type"))
            .map(|value| bounded_text(value, MAX_PROVIDER_TEXT_BYTES))
            .unwrap_or_else(|| "Provider supplied structured detail".to_string()),
        _ => "Provider supplied structured detail".to_string(),
    }
}

fn is_child_thread_lifecycle(method: &str) -> bool {
    matches!(
        method,
        "thread/started"
            | "thread/status/changed"
            | "thread/archived"
            | "thread/unarchived"
            | "thread/closed"
            | "thread/name/updated"
            | "thread/tokenUsage/updated"
            | "turn/started"
            | "turn/completed"
            | "turn/plan/updated"
    )
}

fn bounded_shape(value: &Value) -> Option<VersionedJson> {
    let object = value.as_object()?;
    Some(VersionedJson {
        schema_version: 1,
        value: json!({ "keys": bounded_keys(object) }),
    })
}

fn bounded_keys(object: &Map<String, Value>) -> Vec<String> {
    object
        .keys()
        .take(MAX_UNKNOWN_KEYS)
        .map(|key| bounded_text(key, 256))
        .collect()
}

fn bounded_text(value: &str, maximum_bytes: usize) -> String {
    if value.len() <= maximum_bytes {
        return value.to_string();
    }
    let mut boundary = maximum_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value[..boundary].to_string()
}

fn now_utc() -> ChatResult<UtcTimestamp> {
    let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
    UtcTimestamp::new(now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .map_err(identifier_error)
}

fn protocol_error(field: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        format!("Codex protocol field '{field}' is invalid"),
        true,
    )
}

fn identifier_error(_error: String) -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Codex canonical event identity is invalid",
        false,
    )
}
