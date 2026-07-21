use crate::chat::events::{
    CanonicalEvent, CanonicalRuntimeEvent, CanonicalStoredEvent, CANONICAL_EVENT_SCHEMA_VERSION,
};
use crate::chat::models::{
    ChatChangeNotification, ChatError, ChatErrorCode, ChatResult, ChatThreadId, UtcTimestamp,
    VersionedJson,
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

#[derive(Clone, Debug)]
pub struct AppendCanonicalEventRequest {
    pub runtime: CanonicalRuntimeEvent,
    pub ingested_at: UtcTimestamp,
    pub diagnostic_expires_at: Option<UtcTimestamp>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppendCanonicalEventResult {
    pub event: CanonicalStoredEvent,
    pub notification: ChatChangeNotification,
}

pub async fn append_canonical_event(
    pool: &SqlitePool,
    request: AppendCanonicalEventRequest,
) -> ChatResult<AppendCanonicalEventResult> {
    if request.runtime.schema_version != CANONICAL_EVENT_SCHEMA_VERSION {
        return Err(ChatError::validation(
            "event.schemaVersion",
            "Canonical event schema is unsupported",
        ));
    }
    if request.runtime.redacted_diagnostic.is_some() != request.diagnostic_expires_at.is_some() {
        return Err(ChatError::validation(
            "event.redactedDiagnostic",
            "Redacted diagnostics require an explicit retention deadline",
        ));
    }
    let mut transaction = pool.begin().await.map_err(persistence_error)?;
    let thread = sqlx::query("SELECT revision, last_event_sequence FROM chat_threads WHERE id = ?")
        .bind(request.runtime.thread_id.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(persistence_error)?
        .ok_or_else(|| {
            ChatError::new(ChatErrorCode::NotFound, "Chat thread was not found", true)
        })?;
    let revision = u64_column(&thread, "revision")?;
    let sequence = u64_column(&thread, "last_event_sequence")?
        .checked_add(1)
        .ok_or_else(|| {
            ChatError::new(
                ChatErrorCode::Internal,
                "Chat event sequence overflow",
                false,
            )
        })?;
    let payload = serde_json::to_string(&request.runtime.event).map_err(serialization_error)?;
    let provider_reference = versioned_parts(request.runtime.provider_reference.as_ref())?;
    let diagnostic = versioned_parts(request.runtime.redacted_diagnostic.as_ref())?;

    sqlx::query(
        "INSERT INTO chat_events
            (id, thread_id, sequence, event_schema_version, turn_id, provider_turn_id,
             provider_item_id, provider_request_id, provider_task_id, provider_family_id,
             provider_instance_id, event_type, payload_schema_version, payload_data,
             provider_reference_schema_version, provider_reference_data, created_at,
             ingested_at, redacted_diagnostic_schema_version, redacted_diagnostic_data,
             diagnostic_expires_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(request.runtime.event_id.as_str())
    .bind(request.runtime.thread_id.as_str())
    .bind(i64_value(sequence)?)
    .bind(i64::from(request.runtime.schema_version))
    .bind(request.runtime.turn_id.as_ref().map(|value| value.as_str()))
    .bind(
        request
            .runtime
            .provider_turn_id
            .as_ref()
            .map(|value| value.as_str()),
    )
    .bind(
        request
            .runtime
            .provider_item_id
            .as_ref()
            .map(|value| value.as_str()),
    )
    .bind(
        request
            .runtime
            .provider_request_id
            .as_ref()
            .map(|value| value.as_str()),
    )
    .bind(
        request
            .runtime
            .provider_task_id
            .as_ref()
            .map(|value| value.as_str()),
    )
    .bind(request.runtime.provider_family_id.as_str())
    .bind(request.runtime.provider_instance_id.as_str())
    .bind(event_type(&request.runtime.event)?)
    .bind(i64::from(request.runtime.schema_version))
    .bind(payload)
    .bind(provider_reference.0)
    .bind(provider_reference.1)
    .bind(request.runtime.created_at.as_str())
    .bind(request.ingested_at.as_str())
    .bind(diagnostic.0)
    .bind(diagnostic.1)
    .bind(
        request
            .diagnostic_expires_at
            .as_ref()
            .map(UtcTimestamp::as_str),
    )
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;

    let changed_projection_keys =
        apply_projection(&mut transaction, sequence, &request.runtime).await?;
    let next_revision = revision.checked_add(1).ok_or_else(|| {
        ChatError::new(
            ChatErrorCode::Internal,
            "Chat thread revision overflow",
            false,
        )
    })?;
    let updated = sqlx::query(
        "UPDATE chat_threads
         SET last_event_sequence = ?, last_projected_sequence = ?, revision = ?,
             last_activity_at = ?, updated_at = ?
         WHERE id = ? AND revision = ? AND last_event_sequence = ?",
    )
    .bind(i64_value(sequence)?)
    .bind(i64_value(sequence)?)
    .bind(i64_value(next_revision)?)
    .bind(request.runtime.created_at.as_str())
    .bind(request.ingested_at.as_str())
    .bind(request.runtime.thread_id.as_str())
    .bind(i64_value(revision)?)
    .bind(i64_value(sequence - 1)?)
    .execute(&mut *transaction)
    .await
    .map_err(persistence_error)?;
    if updated.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::StaleRevision,
            "Chat thread changed while the event was being appended",
            true,
        ));
    }
    transaction.commit().await.map_err(persistence_error)?;
    let notification_thread_id = request.runtime.thread_id.clone();

    Ok(AppendCanonicalEventResult {
        event: CanonicalStoredEvent {
            sequence,
            ingested_at: request.ingested_at,
            runtime: request.runtime,
        },
        notification: ChatChangeNotification {
            thread_id: notification_thread_id,
            sequence,
            revision: next_revision,
            changed_projection_keys,
        },
    })
}

pub async fn read_canonical_events(
    pool: &SqlitePool,
    thread_id: &ChatThreadId,
    after_sequence: u64,
) -> ChatResult<Vec<CanonicalStoredEvent>> {
    let rows = sqlx::query(
        "SELECT id, thread_id, sequence, event_schema_version, turn_id, provider_turn_id,
                provider_item_id, provider_request_id, provider_task_id,
                provider_family_id, provider_instance_id, payload_data,
                provider_reference_schema_version, provider_reference_data,
                created_at, ingested_at, redacted_diagnostic_schema_version,
                redacted_diagnostic_data
         FROM chat_events
         WHERE thread_id = ? AND sequence > ?
         ORDER BY sequence ASC, id ASC",
    )
    .bind(thread_id.as_str())
    .bind(i64_value(after_sequence)?)
    .fetch_all(pool)
    .await
    .map_err(persistence_error)?;
    rows.into_iter().map(row_to_event).collect()
}

pub(super) async fn apply_projection(
    transaction: &mut Transaction<'_, Sqlite>,
    sequence: u64,
    runtime: &CanonicalRuntimeEvent,
) -> ChatResult<Vec<String>> {
    let sequence = i64_value(sequence)?;
    let thread_id = runtime.thread_id.as_str();
    let mut changed = Vec::new();
    match &runtime.event {
        CanonicalEvent::ThreadStarted(event) => {
            sqlx::query(
                "UPDATE chat_threads
                 SET provider_thread_id = ?, title = COALESCE(?, title), title_source = CASE
                     WHEN ? IS NULL THEN title_source ELSE 'provider' END
                 WHERE id = ?",
            )
            .bind(event.provider_thread_id.as_str())
            .bind(event.title.as_deref())
            .bind(event.title.as_deref())
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("thread".to_string());
        }
        CanonicalEvent::ThreadStateChanged(event) => {
            sqlx::query("UPDATE chat_threads SET state = ? WHERE id = ?")
                .bind(wire_literal(&event.state)?)
                .bind(thread_id)
                .execute(&mut **transaction)
                .await
                .map_err(persistence_error)?;
            changed.push("thread".to_string());
        }
        CanonicalEvent::ThreadMetadataUpdated(event) => {
            let resume = versioned_parts(event.resume_cursor.as_ref())?;
            sqlx::query(
                "UPDATE chat_threads
                 SET title = COALESCE(?, title),
                     title_source = CASE WHEN ? IS NULL THEN title_source ELSE 'provider' END,
                     provider_thread_id = COALESCE(?, provider_thread_id),
                     resume_cursor_schema_version = COALESCE(?, resume_cursor_schema_version),
                     resume_cursor_data = COALESCE(?, resume_cursor_data)
                 WHERE id = ?",
            )
            .bind(event.title.as_deref())
            .bind(event.title.as_deref())
            .bind(
                event
                    .provider_thread_id
                    .as_ref()
                    .map(|value| value.as_str()),
            )
            .bind(resume.0)
            .bind(resume.1)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("thread".to_string());
        }
        CanonicalEvent::TurnStarted(event) => {
            let turn_id = required_turn_id(runtime)?;
            let ordinal: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(ordinal), -1) + 1 FROM chat_turns WHERE thread_id = ?",
            )
            .bind(thread_id)
            .fetch_one(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            let model_selection = serde_json::to_string(&json!({
                "modelId": event.model_id,
                "options": event.model_options,
            }))
            .map_err(serialization_error)?;
            sqlx::query(
                "INSERT INTO chat_turns
                    (id, thread_id, ordinal, provider_turn_id, state, started_at,
                     model_selection_data, safety_mode, interaction_mode, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                    provider_turn_id = excluded.provider_turn_id,
                    state = excluded.state,
                    started_at = excluded.started_at,
                    model_selection_data = excluded.model_selection_data,
                    safety_mode = excluded.safety_mode,
                    interaction_mode = excluded.interaction_mode,
                    updated_at = excluded.updated_at",
            )
            .bind(turn_id)
            .bind(thread_id)
            .bind(ordinal)
            .bind(event.provider_turn_id.as_ref().map(|value| value.as_str()))
            .bind(wire_literal(&event.state)?)
            .bind(runtime.created_at.as_str())
            .bind(model_selection)
            .bind(wire_literal(&event.modes.safety_mode)?)
            .bind(wire_literal(&event.modes.interaction_mode)?)
            .bind(runtime.created_at.as_str())
            .bind(runtime.created_at.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            sqlx::query(
                "UPDATE chat_threads SET latest_turn_state = ?, state = 'active' WHERE id = ?",
            )
            .bind(wire_literal(&event.state)?)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.extend(["turns".to_string(), "thread".to_string()]);
        }
        CanonicalEvent::TurnCompleted(event) => {
            let turn_id = required_turn_id(runtime)?;
            let usage = event
                .usage
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(serialization_error)?;
            let files = serde_json::to_string(&event.changed_files).map_err(serialization_error)?;
            let updated = sqlx::query(
                "UPDATE chat_turns
                 SET state = ?, completed_at = ?, stop_reason = ?,
                     usage_schema_version = CASE WHEN ? IS NULL THEN NULL ELSE 1 END,
                     usage_data = ?, changed_file_summary_schema_version = 1,
                     changed_file_summary_data = ?, updated_at = ?
                 WHERE id = ? AND thread_id = ?",
            )
            .bind(wire_literal(&event.state)?)
            .bind(runtime.created_at.as_str())
            .bind(event.stop_reason.as_deref())
            .bind(usage.as_deref())
            .bind(usage.as_deref())
            .bind(files)
            .bind(runtime.created_at.as_str())
            .bind(turn_id)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            if updated.rows_affected() != 1 {
                return Err(ChatError::validation(
                    "event.turnId",
                    "Canonical turn completion references an unknown turn",
                ));
            }
            sqlx::query(
                "UPDATE chat_threads SET latest_turn_state = ?, state = 'idle' WHERE id = ?",
            )
            .bind(wire_literal(&event.state)?)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.extend(["turns".to_string(), "thread".to_string()]);
        }
        CanonicalEvent::TurnAborted(event) => {
            let turn_id = required_turn_id(runtime)?;
            let updated = sqlx::query(
                "UPDATE chat_turns SET state = ?, completed_at = ?, stop_reason = ?, updated_at = ?
                 WHERE id = ? AND thread_id = ?",
            )
            .bind(wire_literal(&event.state)?)
            .bind(runtime.created_at.as_str())
            .bind(&event.reason)
            .bind(runtime.created_at.as_str())
            .bind(turn_id)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            if updated.rows_affected() != 1 {
                return Err(ChatError::validation(
                    "event.turnId",
                    "Canonical turn abort references an unknown turn",
                ));
            }
            sqlx::query(
                "UPDATE chat_threads SET latest_turn_state = ?, state = 'idle' WHERE id = ?",
            )
            .bind(wire_literal(&event.state)?)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.extend(["turns".to_string(), "thread".to_string()]);
        }
        CanonicalEvent::ContentDelta(event) => {
            let item_id = &event.item_id;
            let stream_kind = wire_literal(&event.stream_kind)?;
            if stream_kind == "assistant_text" {
                sqlx::query(
                    "INSERT INTO chat_messages
                        (id, thread_id, turn_id, sequence_anchor, role, normalized_markdown,
                         streaming_state, provider_item_id, created_at, updated_at)
                     VALUES (?, ?, ?, ?, 'assistant', ?, 'streaming', ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET
                        normalized_markdown = chat_messages.normalized_markdown || excluded.normalized_markdown,
                        updated_at = excluded.updated_at",
                )
                .bind(item_id)
                .bind(thread_id)
                .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
                .bind(sequence)
                .bind(&event.delta)
                .bind(item_id)
                .bind(runtime.created_at.as_str())
                .bind(runtime.created_at.as_str())
                .execute(&mut **transaction)
                .await
                .map_err(persistence_error)?;
                let message_count: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM chat_messages WHERE thread_id = ?")
                        .bind(thread_id)
                        .fetch_one(&mut **transaction)
                        .await
                        .map_err(persistence_error)?;
                sqlx::query(
                    "UPDATE chat_threads
                     SET message_count = ?,
                         latest_preview = (
                             SELECT substr(normalized_markdown, -2000)
                             FROM chat_messages
                             WHERE id = ?
                         )
                     WHERE id = ?",
                )
                .bind(message_count)
                .bind(item_id)
                .bind(thread_id)
                .execute(&mut **transaction)
                .await
                .map_err(persistence_error)?;
                changed.extend(["messages".to_string(), "thread".to_string()]);
            } else {
                upsert_activity_detail(
                    transaction,
                    sequence,
                    runtime,
                    item_id,
                    &stream_kind,
                    &event.delta,
                )
                .await?;
                changed.push("activities".to_string());
            }
        }
        CanonicalEvent::ItemStarted(event)
        | CanonicalEvent::ItemUpdated(event)
        | CanonicalEvent::ItemCompleted(event) => {
            let metadata = event
                .safe_metadata
                .as_ref()
                .map(|value| serde_json::to_string(&value.value))
                .transpose()
                .map_err(serialization_error)?
                .unwrap_or_else(|| "{}".to_string());
            sqlx::query(
                "INSERT INTO chat_activities
                    (id, thread_id, turn_id, sequence_anchor, item_kind, status, title,
                     detail, provider_item_id, safe_metadata_data, started_at,
                     source_event_type, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET status = excluded.status,
                    title = excluded.title, detail = excluded.detail,
                    safe_metadata_data = excluded.safe_metadata_data,
                    updated_at = excluded.updated_at",
            )
            .bind(&event.item_id)
            .bind(thread_id)
            .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
            .bind(sequence)
            .bind(wire_literal(&event.kind)?)
            .bind(wire_literal(&event.status)?)
            .bind(event.title.as_deref().unwrap_or(""))
            .bind(event.detail.as_deref())
            .bind(&event.item_id)
            .bind(metadata)
            .bind(runtime.created_at.as_str())
            .bind(event_type(&runtime.event)?)
            .bind(runtime.created_at.as_str())
            .bind(runtime.created_at.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("activities".to_string());
        }
        CanonicalEvent::RequestOpened(event) => {
            sqlx::query(
                "INSERT INTO chat_pending_requests
                    (id, thread_id, turn_id, provider_request_id, request_kind,
                     safe_display_schema_version, safe_display_data,
                     allowed_decisions_schema_version, allowed_decisions_data,
                     opened_sequence, opened_at)
                 VALUES (?, ?, ?, ?, 'approval', ?, ?, 1, ?, ?, ?)",
            )
            .bind(event.request_id.as_str())
            .bind(thread_id)
            .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
            .bind(event.request_id.as_str())
            .bind(i64::from(event.safe_payload.schema_version))
            .bind(serde_json::to_string(&event.safe_payload.value).map_err(serialization_error)?)
            .bind(serde_json::to_string(&event.allowed_decisions).map_err(serialization_error)?)
            .bind(sequence)
            .bind(runtime.created_at.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("pending_requests".to_string());
        }
        CanonicalEvent::UserInputRequested(event) => {
            sqlx::query(
                "INSERT INTO chat_pending_requests
                    (id, thread_id, turn_id, provider_request_id, request_kind,
                     safe_display_data, allowed_decisions_data, opened_sequence, opened_at)
                 VALUES (?, ?, ?, ?, 'user_input', ?, '[]', ?, ?)",
            )
            .bind(event.request_id.as_str())
            .bind(thread_id)
            .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
            .bind(event.request_id.as_str())
            .bind(serde_json::to_string(&event.questions).map_err(serialization_error)?)
            .bind(sequence)
            .bind(runtime.created_at.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("pending_requests".to_string());
        }
        CanonicalEvent::RequestResolved(event) => {
            resolve_request(
                transaction,
                runtime,
                event.request_id.as_str(),
                wire_literal(&event.state)?,
                &event.decision,
            )
            .await?;
            changed.push("pending_requests".to_string());
        }
        CanonicalEvent::UserInputResolved(event) => {
            resolve_request(
                transaction,
                runtime,
                event.request_id.as_str(),
                wire_literal(&event.state)?,
                &event.answers,
            )
            .await?;
            changed.push("pending_requests".to_string());
        }
        CanonicalEvent::ProposedPlanDelta(event) => {
            sqlx::query(
                "INSERT INTO chat_plans
                    (id, thread_id, origin_turn_id, sequence_anchor, markdown,
                     state, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, 'proposed', ?, ?)
                 ON CONFLICT(id) DO UPDATE SET markdown = chat_plans.markdown || excluded.markdown,
                    updated_at = excluded.updated_at",
            )
            .bind(&event.plan_id)
            .bind(thread_id)
            .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
            .bind(sequence)
            .bind(&event.delta)
            .bind(runtime.created_at.as_str())
            .bind(runtime.created_at.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("plans".to_string());
        }
        CanonicalEvent::ProposedPlanCompleted(event) => {
            sqlx::query(
                "UPDATE chat_plans SET markdown = ?, updated_at = ?
                 WHERE id = ? AND thread_id = ?",
            )
            .bind(&event.markdown)
            .bind(runtime.created_at.as_str())
            .bind(&event.plan_id)
            .bind(thread_id)
            .execute(&mut **transaction)
            .await
            .map_err(persistence_error)?;
            changed.push("plans".to_string());
        }
        _ => {}
    }
    Ok(changed)
}

async fn upsert_activity_detail(
    transaction: &mut Transaction<'_, Sqlite>,
    sequence: i64,
    runtime: &CanonicalRuntimeEvent,
    item_id: &str,
    item_kind: &str,
    delta: &str,
) -> ChatResult<()> {
    sqlx::query(
        "INSERT INTO chat_activities
            (id, thread_id, turn_id, sequence_anchor, item_kind, status, title,
             detail, provider_item_id, source_event_type, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 'active', '', ?, ?, 'content_delta', ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            detail = COALESCE(chat_activities.detail, '') || excluded.detail,
            updated_at = excluded.updated_at",
    )
    .bind(item_id)
    .bind(runtime.thread_id.as_str())
    .bind(runtime.turn_id.as_ref().map(|value| value.as_str()))
    .bind(sequence)
    .bind(item_kind)
    .bind(delta)
    .bind(item_id)
    .bind(runtime.created_at.as_str())
    .bind(runtime.created_at.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    Ok(())
}

async fn resolve_request<T: Serialize>(
    transaction: &mut Transaction<'_, Sqlite>,
    runtime: &CanonicalRuntimeEvent,
    request_id: &str,
    state: String,
    resolution: &T,
) -> ChatResult<()> {
    let resolution = serde_json::to_string(resolution).map_err(serialization_error)?;
    let result = sqlx::query(
        "UPDATE chat_pending_requests
         SET resolution_state = ?, resolution_schema_version = 1,
             resolution_data = ?, resolved_at = ?
         WHERE thread_id = ? AND provider_request_id = ? AND resolution_state = 'open'",
    )
    .bind(state)
    .bind(resolution)
    .bind(runtime.created_at.as_str())
    .bind(runtime.thread_id.as_str())
    .bind(request_id)
    .execute(&mut **transaction)
    .await
    .map_err(persistence_error)?;
    if result.rows_affected() != 1 {
        return Err(ChatError::new(
            ChatErrorCode::Conflict,
            "Chat request is no longer open",
            true,
        ));
    }
    Ok(())
}

fn row_to_event(row: sqlx::sqlite::SqliteRow) -> ChatResult<CanonicalStoredEvent> {
    let schema_version = u32_column(&row, "event_schema_version")?;
    let event: CanonicalEvent =
        serde_json::from_str(row.try_get("payload_data").map_err(persistence_error)?)
            .map_err(serialization_error)?;
    let runtime = CanonicalRuntimeEvent {
        schema_version,
        event_id: crate::chat::models::ChatEventId::new(string_column(&row, "id")?)
            .map_err(|_| corrupt_data())?,
        provider_family_id: crate::chat::models::ProviderFamilyId::new(string_column(
            &row,
            "provider_family_id",
        )?)
        .map_err(|_| corrupt_data())?,
        provider_instance_id: crate::chat::models::ProviderInstanceId::new(string_column(
            &row,
            "provider_instance_id",
        )?)
        .map_err(|_| corrupt_data())?,
        thread_id: ChatThreadId::new(string_column(&row, "thread_id")?)
            .map_err(|_| corrupt_data())?,
        created_at: UtcTimestamp::new(string_column(&row, "created_at")?)
            .map_err(|_| corrupt_data())?,
        turn_id: optional_identifier(&row, "turn_id", crate::chat::models::ChatTurnId::new)?,
        provider_turn_id: optional_identifier(
            &row,
            "provider_turn_id",
            crate::chat::models::ProviderTurnId::new,
        )?,
        provider_item_id: optional_identifier(
            &row,
            "provider_item_id",
            crate::chat::models::ProviderItemId::new,
        )?,
        provider_request_id: optional_identifier(
            &row,
            "provider_request_id",
            crate::chat::models::ProviderRequestId::new,
        )?,
        provider_task_id: optional_identifier(
            &row,
            "provider_task_id",
            crate::chat::models::ProviderTaskId::new,
        )?,
        provider_reference: read_versioned(
            &row,
            "provider_reference_schema_version",
            "provider_reference_data",
        )?,
        event,
        redacted_diagnostic: read_versioned(
            &row,
            "redacted_diagnostic_schema_version",
            "redacted_diagnostic_data",
        )?,
    };
    Ok(CanonicalStoredEvent {
        sequence: u64_column(&row, "sequence")?,
        ingested_at: UtcTimestamp::new(string_column(&row, "ingested_at")?)
            .map_err(|_| corrupt_data())?,
        runtime,
    })
}

fn required_turn_id(runtime: &CanonicalRuntimeEvent) -> ChatResult<&str> {
    runtime
        .turn_id
        .as_ref()
        .map(|value| value.as_str())
        .ok_or_else(|| ChatError::validation("event.turnId", "Turn event requires a turn ID"))
}

fn event_type(event: &CanonicalEvent) -> ChatResult<String> {
    serde_json::to_value(event)
        .map_err(serialization_error)?
        .get("type")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(corrupt_data)
}

fn wire_literal<T: Serialize>(value: &T) -> ChatResult<String> {
    serde_json::to_value(value)
        .map_err(serialization_error)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(corrupt_data)
}

fn versioned_parts(value: Option<&VersionedJson>) -> ChatResult<(Option<i64>, Option<String>)> {
    value
        .map(|value| {
            Ok((
                Some(i64::from(value.schema_version)),
                Some(serde_json::to_string(&value.value).map_err(serialization_error)?),
            ))
        })
        .unwrap_or(Ok((None, None)))
}

fn read_versioned(
    row: &sqlx::sqlite::SqliteRow,
    version_column: &str,
    data_column: &str,
) -> ChatResult<Option<VersionedJson>> {
    let version: Option<i64> = row.try_get(version_column).map_err(persistence_error)?;
    let data: Option<String> = row.try_get(data_column).map_err(persistence_error)?;
    match (version, data) {
        (None, None) => Ok(None),
        (Some(version), Some(data)) => Ok(Some(VersionedJson {
            schema_version: u32::try_from(version).map_err(|_| corrupt_data())?,
            value: serde_json::from_str(&data).map_err(serialization_error)?,
        })),
        _ => Err(corrupt_data()),
    }
}

fn optional_identifier<T>(
    row: &sqlx::sqlite::SqliteRow,
    column: &str,
    create: impl FnOnce(String) -> Result<T, String>,
) -> ChatResult<Option<T>> {
    row.try_get::<Option<String>, _>(column)
        .map_err(persistence_error)?
        .map(create)
        .transpose()
        .map_err(|_| corrupt_data())
}

fn string_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<String> {
    row.try_get(column).map_err(persistence_error)
}

fn u32_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<u32> {
    let value: i64 = row.try_get(column).map_err(persistence_error)?;
    u32::try_from(value).map_err(|_| corrupt_data())
}

fn u64_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> ChatResult<u64> {
    let value: i64 = row.try_get(column).map_err(persistence_error)?;
    u64::try_from(value).map_err(|_| corrupt_data())
}

fn i64_value(value: u64) -> ChatResult<i64> {
    i64::try_from(value).map_err(|_| {
        ChatError::validation(
            "sequence",
            "Chat sequence exceeds the supported storage range",
        )
    })
}

fn persistence_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Chat persistence operation failed",
        true,
    )
}

fn serialization_error<T>(_error: T) -> ChatError {
    ChatError::new(
        ChatErrorCode::Internal,
        "Chat data could not be serialized",
        false,
    )
}

fn corrupt_data() -> ChatError {
    ChatError::new(
        ChatErrorCode::Persistence,
        "Stored Chat data is invalid",
        false,
    )
}
