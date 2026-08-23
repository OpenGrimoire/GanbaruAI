use super::*;

pub(in crate::pomodoro) async fn normalized_close_run_ended_at(
    tx: &mut Transaction<'_, Sqlite>,
    closure: &PomodoroRunClosure,
) -> Result<String, String> {
    let active_start = sqlx::query_scalar::<_, String>(
        "SELECT actual_start
         FROM pomodoro_segments
         WHERE run_id = ? AND status = 'active'
         ORDER BY actual_start DESC
         LIMIT 1",
    )
    .bind(&closure.run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load active pomodoro segment start: {e}"))?;

    Ok(active_start
        .filter(|start| iso_is_before(&closure.ended_at, start))
        .unwrap_or_else(|| closure.ended_at.clone()))
}

pub(in crate::pomodoro) async fn load_segment_event_context(
    tx: &mut Transaction<'_, Sqlite>,
    segment_id: &str,
) -> Result<SegmentEventContext, String> {
    let row = sqlx::query(
        "SELECT run_id, phase, status, planned_end
         FROM pomodoro_segments
         WHERE id = ?",
    )
    .bind(segment_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("load pomodoro segment event context: {e}"))?;
    Ok(SegmentEventContext {
        run_id: row
            .try_get("run_id")
            .map_err(|e| format!("read segment run_id: {e}"))?,
        phase: row
            .try_get("phase")
            .map_err(|e| format!("read segment phase: {e}"))?,
        status: row
            .try_get("status")
            .map_err(|e| format!("read segment status: {e}"))?,
        planned_end: row
            .try_get("planned_end")
            .map_err(|e| format!("read segment planned_end: {e}"))?,
    })
}

pub(in crate::pomodoro) async fn load_existing_pauses(
    tx: &mut Transaction<'_, Sqlite>,
    segment_id: &str,
) -> Result<Vec<ExistingPause>, String> {
    let rows = sqlx::query(
        "SELECT started_at, ended_at, reason
         FROM pomodoro_pauses
         WHERE segment_id = ?
         ORDER BY started_at ASC",
    )
    .bind(segment_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load existing pomodoro pauses: {e}"))?;
    rows.into_iter()
        .map(|row| {
            Ok(ExistingPause {
                started_at: row
                    .try_get("started_at")
                    .map_err(|e| format!("read existing pause started_at: {e}"))?,
                ended_at: row
                    .try_get("ended_at")
                    .map_err(|e| format!("read existing pause ended_at: {e}"))?,
                reason: row
                    .try_get("reason")
                    .map_err(|e| format!("read existing pause reason: {e}"))?,
            })
        })
        .collect()
}

pub(in crate::pomodoro) async fn replace_segment_pauses(
    tx: &mut Transaction<'_, Sqlite>,
    segment_id: &str,
    pauses: &[PomodoroPauseWrite],
) -> Result<(), String> {
    sqlx::query("DELETE FROM pomodoro_pauses WHERE segment_id = ?")
        .bind(segment_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("clear pomodoro pauses: {e}"))?;

    for (index, pause) in pauses.iter().enumerate() {
        validate_pause(pause)?;
        sqlx::query(
            "INSERT INTO pomodoro_pauses
                (id, segment_id, started_at, ended_at, reason, detected_at)
             VALUES (lower(hex(randomblob(16))), ?, ?, ?, ?, ?)",
        )
        .bind(segment_id)
        .bind(&pause.started_at)
        .bind(&pause.ended_at)
        .bind(&pause.reason)
        .bind(if pause.reason == "idle" {
            Some(pause.started_at.as_str())
        } else {
            None
        })
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("insert pomodoro pause {index}: {e}"))?;
    }
    Ok(())
}

pub(in crate::pomodoro) async fn log_segment_update_events(
    tx: &mut Transaction<'_, Sqlite>,
    context: &SegmentEventContext,
    segment: &PomodoroSegmentUpdate,
    previous_pauses: &[ExistingPause],
) -> Result<(), String> {
    let occurred_at = segment
        .occurred_at
        .as_deref()
        .or(segment.actual_end.as_deref())
        .or(segment.actual_start.as_deref())
        .unwrap_or(&segment.planned_end);

    if context.status == "active" && segment.status == "completed" {
        insert_run_event_tx(
            tx,
            RunEventInsert {
                run_id: &context.run_id,
                segment_id: Some(&segment.id),
                event_type: "phase_complete",
                occurred_at,
                phase: Some(&context.phase),
                reason: segment.end_reason.as_deref(),
                duration_seconds: None,
            },
        )
        .await?;
    }

    if context.phase == "focus" && context.planned_end != segment.planned_end {
        insert_run_event_tx(
            tx,
            RunEventInsert {
                run_id: &context.run_id,
                segment_id: Some(&segment.id),
                event_type: "extend_focus",
                occurred_at,
                phase: Some(&context.phase),
                reason: None,
                duration_seconds: iso_seconds_between(&context.planned_end, &segment.planned_end),
            },
        )
        .await?;
    }

    for pause in &segment.pauses {
        let previous = previous_pauses
            .iter()
            .find(|old| old.started_at == pause.started_at && old.reason == pause.reason);
        if previous.is_none() {
            insert_pause_start_events(tx, &context.run_id, &segment.id, &context.phase, pause)
                .await?;
            if let Some(ended_at) = &pause.ended_at {
                insert_run_event_tx(
                    tx,
                    RunEventInsert {
                        run_id: &context.run_id,
                        segment_id: Some(&segment.id),
                        event_type: "pause_end",
                        occurred_at: ended_at,
                        phase: Some(&context.phase),
                        reason: Some(&pause.reason),
                        duration_seconds: None,
                    },
                )
                .await?;
            }
            continue;
        }

        if previous.and_then(|old| old.ended_at.as_ref()).is_none() && pause.ended_at.is_some() {
            let ended_at = pause.ended_at.as_deref().unwrap_or(occurred_at);
            insert_run_event_tx(
                tx,
                RunEventInsert {
                    run_id: &context.run_id,
                    segment_id: Some(&segment.id),
                    event_type: "pause_end",
                    occurred_at: ended_at,
                    phase: Some(&context.phase),
                    reason: Some(&pause.reason),
                    duration_seconds: None,
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub(in crate::pomodoro) async fn log_new_pause_events(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    segment: &PomodoroSegmentWrite,
) -> Result<(), String> {
    for pause in &segment.pauses {
        insert_pause_start_events(tx, run_id, &segment.id, &segment.phase, pause).await?;
        if let Some(ended_at) = &pause.ended_at {
            insert_run_event_tx(
                tx,
                RunEventInsert {
                    run_id,
                    segment_id: Some(&segment.id),
                    event_type: "pause_end",
                    occurred_at: ended_at,
                    phase: Some(&segment.phase),
                    reason: Some(&pause.reason),
                    duration_seconds: None,
                },
            )
            .await?;
        }
    }
    Ok(())
}

async fn insert_pause_start_events(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    segment_id: &str,
    phase: &str,
    pause: &PomodoroPauseWrite,
) -> Result<(), String> {
    insert_run_event_tx(
        tx,
        RunEventInsert {
            run_id,
            segment_id: Some(segment_id),
            event_type: "pause_start",
            occurred_at: &pause.started_at,
            phase: Some(phase),
            reason: Some(&pause.reason),
            duration_seconds: None,
        },
    )
    .await?;
    if pause.reason == "idle" {
        insert_run_event_tx(
            tx,
            RunEventInsert {
                run_id,
                segment_id: Some(segment_id),
                event_type: "idle_detected",
                occurred_at: &pause.started_at,
                phase: Some(phase),
                reason: Some(&pause.reason),
                duration_seconds: None,
            },
        )
        .await?;
    }
    if pause.reason == "suspend" {
        insert_run_event_tx(
            tx,
            RunEventInsert {
                run_id,
                segment_id: Some(segment_id),
                event_type: "suspend_detected",
                occurred_at: &pause.started_at,
                phase: Some(phase),
                reason: Some(&pause.reason),
                duration_seconds: None,
            },
        )
        .await?;
    }
    Ok(())
}

pub(in crate::pomodoro) async fn insert_run_event_tx(
    tx: &mut Transaction<'_, Sqlite>,
    event: RunEventInsert<'_>,
) -> Result<(), String> {
    validate_event_type(event.event_type)?;
    if let Some(phase) = event.phase {
        validate_phase(phase)?;
    }
    sqlx::query(
        "INSERT INTO pomodoro_run_events
            (id, run_id, segment_id, event_type, occurred_at, phase, reason, duration_seconds)
         VALUES (lower(hex(randomblob(16))), ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(event.run_id)
    .bind(event.segment_id)
    .bind(event.event_type)
    .bind(event.occurred_at)
    .bind(event.phase)
    .bind(event.reason)
    .bind(event.duration_seconds)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert pomodoro run event: {e}"))?;
    Ok(())
}
