use super::*;

#[tauri::command]
pub async fn notes_preview_project_history_restore<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
) -> Result<NotesProjectHistoryRestorePlanDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    restore::preview_restore(&pool, &project_id, &version_id).await
}

#[tauri::command]
pub async fn notes_restore_project_history_version<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
) -> Result<NotesProjectHistoryVersionDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    restore::restore_version(&pool, &project_id, &version_id).await
}
