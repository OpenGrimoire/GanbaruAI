#[cfg(not(any(target_os = "android", target_os = "ios")))]
use super::NoteDataSourceCsvExportSaveDto;
use super::{NoteDataSourceCsvExportDto, NoteDataSourceCsvExportRequest};
use sqlx::SqlitePool;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::path::PathBuf;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{AppHandle, Runtime};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_dialog::{DialogExt, FilePath};

pub async fn export_csv(
    pool: &SqlitePool,
    data_source_id: &str,
    request: NoteDataSourceCsvExportRequest,
) -> Result<NoteDataSourceCsvExportDto, String> {
    ganbaru_notes::notes::data_source_csv_export::export_csv(pool, data_source_id, request).await
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub async fn pick_and_write_csv<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    data_source_id: &str,
    request: NoteDataSourceCsvExportRequest,
) -> Result<NoteDataSourceCsvExportSaveDto, String> {
    let export = export_csv(pool, data_source_id, request).await?;
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Export Notes database CSV")
        .set_file_name(&export.file_name)
        .add_filter("CSV", &["csv"])
        .blocking_save_file()
        .map(dialog_path)
        .transpose()?
    else {
        return Ok(NoteDataSourceCsvExportSaveDto::canceled());
    };
    ganbaru_notes::notes::data_source_csv_export::write_csv(&path, export)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|error| format!("selected path is not a local file: {error}"))
}
