#[cfg(not(any(target_os = "android", target_os = "ios")))]
use super::NoteJsonGraphExportSaveDto;
use super::{NoteJsonGraphExportDto, NoteJsonGraphExportRequest};
use sqlx::SqlitePool;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::path::PathBuf;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{AppHandle, Runtime};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::{DialogExt, FilePath};

pub async fn export_graph(
    pool: &SqlitePool,
    request: NoteJsonGraphExportRequest,
) -> Result<NoteJsonGraphExportDto, String> {
    ganbaru_notes::notes::json_graph_export::export_graph(pool, request).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn pick_and_write_graph<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    request: NoteJsonGraphExportRequest,
) -> Result<NoteJsonGraphExportSaveDto, String> {
    let export = export_graph(pool, request).await?;
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Export Notes JSON graph")
        .set_file_name(&export.file_name)
        .add_filter("JSON graph", &["json"])
        .blocking_save_file()
        .map(dialog_path)
        .transpose()?
    else {
        return Ok(NoteJsonGraphExportSaveDto::canceled());
    };
    ganbaru_notes::notes::json_graph_export::write_graph(&path, export)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|error| format!("selected path is not a local file: {error}"))
}
