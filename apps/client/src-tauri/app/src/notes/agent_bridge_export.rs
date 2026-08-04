use super::{NoteAgentBridgeExportDto, NoteAgentBridgeExportRequest, NoteAgentBridgeExportSaveDto};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};

pub async fn export_bridge(
    pool: &SqlitePool,
    request: NoteAgentBridgeExportRequest,
) -> Result<NoteAgentBridgeExportDto, String> {
    ganbaru_notes::notes::agent_bridge_export::export_bridge(pool, request).await
}

pub async fn pick_and_write_bridge<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    request: NoteAgentBridgeExportRequest,
) -> Result<NoteAgentBridgeExportSaveDto, String> {
    let export = export_bridge(pool, request).await?;
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Export agent bridge markdown")
        .set_file_name(&export.file_name)
        .add_filter("Markdown", &["md"])
        .blocking_save_file()
        .map(dialog_path)
        .transpose()?
    else {
        return Ok(NoteAgentBridgeExportSaveDto::canceled());
    };
    ganbaru_notes::notes::agent_bridge_export::write_bridge(&path, export)
}

fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|error| format!("selected path is not a local file: {error}"))
}
