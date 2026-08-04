use super::{NoteHtmlArchiveSaveDto, NoteHtmlExportDto, NoteHtmlExportRequest};
use crate::vault;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};

pub async fn export_page(
    pool: &SqlitePool,
    request: NoteHtmlExportRequest,
) -> Result<NoteHtmlExportDto, String> {
    ganbaru_notes::notes::html_export::export_page(pool, request).await
}

pub async fn pick_and_write_archive<R: Runtime>(
    app: &AppHandle<R>,
    pool: &SqlitePool,
    request: NoteHtmlExportRequest,
) -> Result<NoteHtmlArchiveSaveDto, String> {
    let mut archive = ganbaru_notes::notes::html_export::export_archive(pool, request).await?;
    let Some(path) = app
        .dialog()
        .file()
        .set_title("Export Notes HTML archive")
        .set_file_name(&archive.default_file_name)
        .add_filter("Zip archive", &["zip"])
        .blocking_save_file()
        .map(dialog_path)
        .transpose()?
    else {
        return Ok(NoteHtmlArchiveSaveDto::canceled());
    };
    let asset_root = vault::active_vault_path(app)?.join("assets");
    ganbaru_notes::notes::html_export_archive::write_archive(&asset_root, &path, &mut archive)
}

fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|error| format!("selected path is not a local file: {error}"))
}
