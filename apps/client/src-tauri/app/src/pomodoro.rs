//! Tauri focus adapters. Vault path authorization stays at the application boundary.

use crate::db_path::connect_sqlite;
pub use ganbaru_focus::*;
use tauri::{AppHandle, Runtime};

/// Admit a desktop automatic start only from a fresh observation after its boundary.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub async fn pomodoro_can_start_automatically(boundary_epoch_ms: i64) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
        let observation = crate::notification::idle::get_idle_status();
        ganbaru_focus::admission::automatic_start_allowed(
            boundary_epoch_ms,
            now.timestamp_millis(),
            observation.idle_ms,
        )
    })
    .await
    .map_err(|error| format!("observe local focus admission: {error}"))
}

#[tauri::command]
pub async fn pomodoro_load_segments_for_events<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    event_ids: Vec<String>,
) -> Result<Vec<PomodoroSegmentRead>, String> {
    ganbaru_focus::pomodoro_load_segments_for_events(connect_sqlite(app, db_url).await?, event_ids)
        .await
}

#[tauri::command]
pub async fn pomodoro_load_adaptive_history<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    before: String,
    policy_id: String,
    segment_limit: i64,
) -> Result<PomodoroAdaptiveHistoryRead, String> {
    ganbaru_focus::pomodoro_load_adaptive_history(
        connect_sqlite(app, db_url).await?,
        before,
        policy_id,
        segment_limit,
    )
    .await
}

#[tauri::command]
pub async fn pomodoro_load_adaptive_replay_dataset<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    before: String,
    policy_id: String,
    limit: i64,
    history_segment_limit: Option<i64>,
) -> Result<PomodoroAdaptiveReplayDatasetRead, String> {
    ganbaru_focus::pomodoro_load_adaptive_replay_dataset(
        connect_sqlite(app, db_url).await?,
        before,
        policy_id,
        limit,
        history_segment_limit,
    )
    .await
}

#[tauri::command]
pub async fn pomodoro_start_run<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    run: PomodoroRunWrite,
    segment: PomodoroSegmentWrite,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_start_run(connect_sqlite(app, db_url).await?, run, segment).await
}

#[tauri::command]
pub async fn pomodoro_transition_run<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    transition: PomodoroTransitionRunWrite,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_transition_run(connect_sqlite(app, db_url).await?, transition).await
}

#[tauri::command]
pub async fn pomodoro_insert_segments<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    segments: Vec<PomodoroSegmentWrite>,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_insert_segments(connect_sqlite(app, db_url).await?, segments).await
}

#[tauri::command]
pub async fn pomodoro_insert_segment_with_adaptive_decision<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    segment: PomodoroSegmentWrite,
    adaptive_decision: PomodoroAdaptiveDecisionEnvelopeWrite,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_insert_segment_with_adaptive_decision(
        connect_sqlite(app, db_url).await?,
        segment,
        adaptive_decision,
    )
    .await
}

#[tauri::command]
pub async fn pomodoro_update_segments<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    segments: Vec<PomodoroSegmentUpdate>,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_update_segments(connect_sqlite(app, db_url).await?, segments).await
}

#[tauri::command]
pub async fn pomodoro_close_run<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    closure: PomodoroRunClosure,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_close_run(connect_sqlite(app, db_url).await?, closure).await
}

#[tauri::command]
pub async fn pomodoro_update_run_window<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    update: PomodoroRunWindowUpdate,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_update_run_window(connect_sqlite(app, db_url).await?, update).await
}

#[tauri::command]
pub async fn pomodoro_transfer_active_event_reference<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    transfer: PomodoroActiveEventReferenceTransfer,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_transfer_active_event_reference(
        connect_sqlite(app, db_url).await?,
        transfer,
    )
    .await
}

#[tauri::command]
pub async fn pomodoro_heartbeat<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    run_id: String,
    heartbeat_at: String,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_heartbeat(connect_sqlite(app, db_url).await?, run_id, heartbeat_at)
        .await
}

#[tauri::command]
pub async fn pomodoro_record_run_event<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    event: PomodoroRunEventWrite,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_record_run_event(connect_sqlite(app, db_url).await?, event).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub async fn pomodoro_recover_open_runs<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<(), String> {
    ganbaru_focus::pomodoro_recover_open_runs(connect_sqlite(app, db_url).await?).await
}

#[tauri::command]
#[cfg(any(test, target_os = "android", target_os = "ios"))]
pub async fn pomodoro_recover_mobile_run<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<PomodoroMobileRecoveryRead, String> {
    let now: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();
    let now_at = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    ganbaru_focus::pomodoro_recover_mobile_run(connect_sqlite(app, db_url).await?, now_at).await
}
