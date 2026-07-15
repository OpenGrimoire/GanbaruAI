use super::*;
use crate::db_path::connect_sqlite;

fn connection_error(error: String) -> MusicLibraryError {
    MusicLibraryError::runtime("open music library", error)
}

#[tauri::command]
pub async fn music_library_upsert_item(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicLibraryItemWrite,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::upsert_library_item(&pool, request).await
}

#[tauri::command]
pub async fn music_library_upsert_local_location(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicLocalLocationWrite,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::upsert_local_location(&pool, request).await
}

#[tauri::command]
pub async fn music_library_start_local_refresh(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicLocalRefreshRequest,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    let progress = super::local_refresh::prepare(&pool, &request).await?;
    let refresh_pool = pool.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _ = tauri::async_runtime::block_on(super::local_refresh::run_prepared(
            &refresh_pool,
            request,
        ));
    });
    Ok(progress)
}

#[tauri::command]
pub async fn music_library_refresh_progress(
    app: tauri::AppHandle,
    db_url: String,
    job_id: String,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::local_refresh::progress(&pool, &job_id).await
}

#[tauri::command]
pub async fn music_library_cancel_refresh(
    app: tauri::AppHandle,
    db_url: String,
    job_id: String,
    cancelled_at: i64,
) -> MusicLibraryResult<MusicRefreshJobProgress> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::local_refresh::cancel(&pool, &job_id, cancelled_at).await
}

#[tauri::command]
pub async fn music_library_upsert_youtube_video(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicYouTubeVideoWrite,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::youtube::upsert_video(&pool, request).await
}

#[tauri::command]
pub async fn music_library_apply_youtube_playlist_snapshot(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicYouTubePlaylistSnapshotWrite,
) -> MusicLibraryResult<MusicYouTubeSnapshotResult> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::youtube::apply_playlist_snapshot(&pool, request).await
}

#[tauri::command]
pub async fn music_library_report_youtube_source_failure(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicYouTubeSourceFailureWrite,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::youtube::report_source_failure(&pool, request).await
}

#[tauri::command]
pub async fn music_library_create_relink_plan(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicRelinkPlanRequest,
) -> MusicLibraryResult<MusicRelinkPlanSummary> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::relink::create_plan(&pool, request).await
}

#[tauri::command]
pub async fn music_library_relink_plan_entries(
    app: tauri::AppHandle,
    db_url: String,
    plan_id: String,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<MusicRelinkPlanWindow> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::relink::plan_entries(&pool, &plan_id, offset, limit).await
}

#[tauri::command]
pub async fn music_library_apply_relink_plan(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicRelinkApplyRequest,
) -> MusicLibraryResult<MusicRelinkPlanSummary> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::relink::apply_plan(&pool, request).await
}

#[tauri::command]
pub async fn music_library_cancel_relink_plan(
    app: tauri::AppHandle,
    db_url: String,
    plan_id: String,
    cancelled_at: i64,
) -> MusicLibraryResult<MusicRelinkPlanSummary> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::relink::cancel_plan(&pool, &plan_id, cancelled_at).await
}

#[tauri::command]
pub async fn music_library_source_removal_impact(
    app: tauri::AppHandle,
    db_url: String,
    collection_id: String,
) -> MusicLibraryResult<MusicSourceRemovalImpact> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::source_lifecycle::removal_impact(&pool, &collection_id).await
}

#[tauri::command]
pub async fn music_library_remove_source(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicSourceRemovalRequest,
) -> MusicLibraryResult<MusicSourceRemovalImpact> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::source_lifecycle::remove_source(&pool, request).await
}

#[tauri::command]
pub async fn music_library_restore_source(
    app: tauri::AppHandle,
    db_url: String,
    collection_id: String,
    expected_version: i64,
    restored_at: i64,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::source_lifecycle::restore_source(&pool, &collection_id, expected_version, restored_at)
        .await
}

#[tauri::command]
pub async fn music_library_create_playlist(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicPlaylistCreate,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::create_playlist(&pool, request).await
}

#[tauri::command]
pub async fn music_library_update_playlist(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicPlaylistUpdate,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::update_playlist(&pool, request).await
}

#[tauri::command]
pub async fn music_library_duplicate_playlist(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicPlaylistDuplicate,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::duplicate_playlist(&pool, request).await
}

#[tauri::command]
pub async fn music_library_playlist_delete_impact(
    app: tauri::AppHandle,
    db_url: String,
    playlist_id: String,
) -> MusicLibraryResult<MusicPlaylistDeleteImpact> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::playlist_delete_impact(&pool, &playlist_id).await
}

#[tauri::command]
pub async fn music_library_delete_playlist(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicPlaylistDelete,
) -> MusicLibraryResult<MusicPlaylistDeleteImpact> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::delete_playlist(&pool, request).await
}

#[tauri::command]
pub async fn music_library_set_review_state(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicReviewWrite,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::set_review_state(&pool, request).await
}

#[tauri::command]
pub async fn music_library_upsert_memberships(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicBulkMembershipWrite,
) -> MusicLibraryResult<Vec<MusicWriteReceipt>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::upsert_memberships(&pool, request).await
}

#[tauri::command]
pub async fn music_library_remove_memberships(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicMembershipRemove,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::remove_memberships(&pool, request).await
}

#[tauri::command]
pub async fn music_library_upsert_snooze(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicSnoozeWrite,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::upsert_snooze(&pool, request).await
}

#[tauri::command]
pub async fn music_library_remove_snooze(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicSnoozeRemove,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::remove_snooze(&pool, request).await
}

#[tauri::command]
pub async fn music_library_reset_statistics(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicStatisticsReset,
) -> MusicLibraryResult<()> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::reset_statistics(&pool, request).await
}

#[tauri::command]
pub async fn music_library_item_window(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicItemWindowRequest,
) -> MusicLibraryResult<MusicItemWindow> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::item_window(&pool, request).await
}

#[tauri::command]
pub async fn music_library_playlist_summaries(
    app: tauri::AppHandle,
    db_url: String,
    now_ms: i64,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<Vec<MusicPlaylistSummary>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::playlist_summaries(&pool, now_ms, offset, limit).await
}

#[tauri::command]
pub async fn music_library_source_summaries(
    app: tauri::AppHandle,
    db_url: String,
    now_ms: i64,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<Vec<MusicSourceSummary>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::source_summaries(&pool, now_ms, offset, limit).await
}

#[tauri::command]
pub async fn music_library_issues(
    app: tauri::AppHandle,
    db_url: String,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<Vec<MusicIssue>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::issues(&pool, offset, limit).await
}

#[tauri::command]
pub async fn music_library_inspector_detail(
    app: tauri::AppHandle,
    db_url: String,
    item_id: String,
) -> MusicLibraryResult<MusicInspectorDetail> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::inspector_detail(&pool, &item_id).await
}

#[tauri::command]
pub async fn music_library_rebuild_search_index(
    app: tauri::AppHandle,
    db_url: String,
    rebuilt_at: i64,
) -> MusicLibraryResult<MusicSearchRebuildResult> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::search::rebuild(&pool, rebuilt_at).await
}

#[tauri::command]
pub async fn music_library_local_roots(
    app: tauri::AppHandle,
    db_url: String,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<Vec<MusicLocalRoot>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::local_roots(&pool, offset, limit).await
}

#[tauri::command]
pub async fn music_library_source_collections(
    app: tauri::AppHandle,
    db_url: String,
    offset: i64,
    limit: i64,
) -> MusicLibraryResult<Vec<MusicSourceCollection>> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::source_collections(&pool, offset, limit).await
}

#[tauri::command]
pub async fn music_library_playlist_detail(
    app: tauri::AppHandle,
    db_url: String,
    playlist_id: String,
) -> MusicLibraryResult<MusicPlaylist> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::queries::playlist_detail(&pool, &playlist_id).await
}

#[tauri::command]
pub async fn music_library_upsert_source_collection(
    app: tauri::AppHandle,
    db_url: String,
    request: MusicCollectionWrite,
) -> MusicLibraryResult<MusicWriteReceipt> {
    let pool = connect_sqlite(app, db_url)
        .await
        .map_err(connection_error)?;
    super::writes::upsert_source_collection(&pool, request).await
}
