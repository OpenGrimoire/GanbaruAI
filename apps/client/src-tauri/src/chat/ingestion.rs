use crate::chat::events::{CanonicalEvent, CanonicalRuntimeEvent};
use crate::chat::models::{
    ChatChangeNotification, ChatError, ChatErrorCode, ChatResult, UtcTimestamp,
};
use crate::chat::repository::events::{append_canonical_event, AppendCanonicalEventRequest};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{Emitter, Runtime};

const MAX_CANONICAL_EVENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_DELTA_BATCH_BYTES: usize = 64 * 1024;
pub const CHAT_CHANGE_EVENT: &str = "chat://change";

pub trait ChatChangeEmitter: Send + Sync {
    fn emit(&self, notification: &ChatChangeNotification) -> ChatResult<()>;
}

pub struct TauriChatChangeEmitter<R: Runtime> {
    app: tauri::AppHandle<R>,
}

impl<R: Runtime> TauriChatChangeEmitter<R> {
    pub fn new(app: tauri::AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> ChatChangeEmitter for TauriChatChangeEmitter<R> {
    fn emit(&self, notification: &ChatChangeNotification) -> ChatResult<()> {
        self.app.emit(CHAT_CHANGE_EVENT, notification).map_err(|_| {
            ChatError::new(
                ChatErrorCode::Internal,
                "Chat change notification delivery failed",
                true,
            )
        })
    }
}

pub struct ChatEventIngestor {
    pool: SqlitePool,
    emitter: Arc<dyn ChatChangeEmitter>,
    pending_delta: Option<AppendCanonicalEventRequest>,
}

impl ChatEventIngestor {
    pub fn new(pool: SqlitePool, emitter: Arc<dyn ChatChangeEmitter>) -> Self {
        Self {
            pool,
            emitter,
            pending_delta: None,
        }
    }

    pub async fn ingest(&mut self, request: AppendCanonicalEventRequest) -> ChatResult<()> {
        validate_event(&request.runtime)?;
        if matches!(request.runtime.event, CanonicalEvent::ContentDelta(_)) {
            if let Some(pending) = self.pending_delta.as_mut() {
                if merge_adjacent_delta(pending, &request) {
                    return Ok(());
                }
            }
            self.flush().await?;
            self.pending_delta = Some(request);
            return Ok(());
        }
        self.flush().await?;
        self.append_and_notify(request).await
    }

    pub async fn flush(&mut self) -> ChatResult<()> {
        if let Some(request) = self.pending_delta.take() {
            match append_canonical_event(&self.pool, request.clone()).await {
                Ok(result) => self.emitter.emit(&result.notification)?,
                Err(error) => {
                    self.pending_delta = Some(request);
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    async fn append_and_notify(&self, request: AppendCanonicalEventRequest) -> ChatResult<()> {
        let result = append_canonical_event(&self.pool, request).await?;
        self.emitter.emit(&result.notification)
    }
}

fn merge_adjacent_delta(
    pending: &mut AppendCanonicalEventRequest,
    next: &AppendCanonicalEventRequest,
) -> bool {
    if pending.runtime.thread_id != next.runtime.thread_id
        || pending.runtime.turn_id != next.runtime.turn_id
        || pending.runtime.provider_instance_id != next.runtime.provider_instance_id
    {
        return false;
    }
    let (CanonicalEvent::ContentDelta(pending_delta), CanonicalEvent::ContentDelta(next_delta)) =
        (&mut pending.runtime.event, &next.runtime.event)
    else {
        return false;
    };
    if pending_delta.item_id != next_delta.item_id
        || pending_delta.stream_kind != next_delta.stream_kind
        || pending_delta.content_index != next_delta.content_index
        || pending_delta.delta.len() + next_delta.delta.len() > MAX_DELTA_BATCH_BYTES
    {
        return false;
    }
    pending_delta.delta.push_str(&next_delta.delta);
    pending.runtime.created_at = next.runtime.created_at.clone();
    pending.ingested_at = next.ingested_at.clone();
    true
}

fn validate_event(event: &CanonicalRuntimeEvent) -> ChatResult<()> {
    let encoded = serde_json::to_vec(event).map_err(serialization_error)?;
    if encoded.len() > MAX_CANONICAL_EVENT_BYTES {
        return Err(ChatError::validation(
            "event",
            "Canonical Chat event exceeds the ingestion limit",
        ));
    }
    if let Some(diagnostic) = &event.redacted_diagnostic {
        validate_redacted_value(&diagnostic.value)?;
    }
    Ok(())
}

fn validate_redacted_value(value: &Value) -> ChatResult<()> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let normalized = key.to_ascii_lowercase();
                if [
                    "authorization",
                    "token",
                    "password",
                    "secret",
                    "api_key",
                    "apiKey",
                ]
                .iter()
                .any(|blocked| normalized.contains(&blocked.to_ascii_lowercase()))
                {
                    return Err(unredacted_diagnostic());
                }
                validate_redacted_value(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_redacted_value(value)?;
            }
        }
        Value::String(value) => {
            let normalized = value.to_ascii_lowercase();
            if normalized.contains("/home/")
                || normalized.contains("/root/")
                || normalized.contains("\\users\\")
                || normalized.contains("/users/")
                || normalized.contains("bearer ")
            {
                return Err(unredacted_diagnostic());
            }
        }
        _ => {}
    }
    Ok(())
}

fn unredacted_diagnostic() -> ChatError {
    ChatError::validation(
        "event.redactedDiagnostic",
        "Chat diagnostic contains prohibited sensitive fields",
    )
}
fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        "Canonical Chat event could not be encoded",
        false,
    )
}

pub fn ingestion_timestamp(value: &str) -> ChatResult<UtcTimestamp> {
    UtcTimestamp::new(value.to_string())
        .map_err(|_| ChatError::validation("ingestedAt", "Chat ingestion timestamp is invalid"))
}
