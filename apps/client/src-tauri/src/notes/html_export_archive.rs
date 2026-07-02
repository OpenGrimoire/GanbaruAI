use super::html_export::HtmlArchive;
use super::models::{NoteHtmlArchiveSaveDto, NoteHtmlExportDiagnosticDto};
use crate::vault;
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};

pub(super) fn pick_and_write_archive<R: Runtime>(
    app: &AppHandle<R>,
    archive: &mut HtmlArchive,
) -> Result<NoteHtmlArchiveSaveDto, String> {
    let Some(path) = pick_save_path(app, &archive.default_file_name)? else {
        return Ok(NoteHtmlArchiveSaveDto::canceled());
    };
    require_zip_extension(&path)?;
    write_zip_archive(app, &path, archive)?;
    Ok(NoteHtmlArchiveSaveDto::saved(archive.dto.clone()))
}

fn write_zip_archive<R: Runtime>(
    app: &AppHandle<R>,
    path: &Path,
    archive: &mut HtmlArchive,
) -> Result<(), String> {
    let tmp_path = temp_zip_path(path)?;
    let result = write_zip_archive_inner(app, &tmp_path, archive)
        .and_then(|()| fs::rename(&tmp_path, path).map_err(|e| format!("save HTML archive: {e}")));
    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

fn write_zip_archive_inner<R: Runtime>(
    app: &AppHandle<R>,
    path: &Path,
    archive: &mut HtmlArchive,
) -> Result<(), String> {
    use zip::write::{SimpleFileOptions, ZipWriter};
    use zip::CompressionMethod;

    let file = fs::File::create(path).map_err(|e| format!("create HTML archive: {e}"))?;
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut writer = ZipWriter::new(file);
    for file in &archive.dto.files {
        validate_archive_entry_path(&file.path)?;
        writer
            .start_file(&file.path, options)
            .map_err(|e| format!("write HTML archive entry: {e}"))?;
        writer
            .write_all(file.contents.as_bytes())
            .map_err(|e| format!("write HTML archive entry: {e}"))?;
    }
    let asset_root = vault::active_vault_path(app)?.join("assets");
    for asset in &archive.dto.assets {
        if !asset.exported {
            continue;
        }
        validate_archive_entry_path(&asset.archive_path)?;
        let source = asset_source_path(&asset_root, &asset.source_path)?;
        match fs::read(&source) {
            Ok(bytes) => {
                writer
                    .start_file(&asset.archive_path, options)
                    .map_err(|e| format!("write HTML archive asset: {e}"))?;
                writer
                    .write_all(&bytes)
                    .map_err(|e| format!("write HTML archive asset: {e}"))?;
            }
            Err(error) => archive
                .dto
                .diagnostics
                .push(NoteHtmlExportDiagnosticDto::new(
                    "html_export_asset_read_failed",
                    "warning",
                    None::<String>,
                    None::<String>,
                    Some(asset.id.clone()),
                    None::<String>,
                    format!("Managed asset could not be read: {error}"),
                )),
        }
    }
    let file = writer
        .finish()
        .map_err(|e| format!("finish HTML archive: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("sync HTML archive: {e}"))?;
    Ok(())
}

fn asset_source_path(root: &Path, source_path: &str) -> Result<PathBuf, String> {
    if source_path.starts_with('/') || source_path.contains('\\') || source_path.contains("..") {
        return Err("asset path is not safe for export".to_string());
    }
    let path = Path::new(source_path);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("asset path is not safe for export".to_string());
    }
    Ok(root.join(path))
}

fn validate_archive_entry_path(path: &str) -> Result<(), String> {
    if path.starts_with('/') || path.contains('\\') || path.contains("..") {
        return Err("archive entry path is not safe".to_string());
    }
    if Path::new(path)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("archive entry path is not safe".to_string());
    }
    Ok(())
}

fn pick_save_path<R: Runtime>(
    app: &AppHandle<R>,
    default_name: &str,
) -> Result<Option<PathBuf>, String> {
    app.dialog()
        .file()
        .set_title("Export Notes HTML archive")
        .set_file_name(default_name)
        .add_filter("Zip archive", &["zip"])
        .blocking_save_file()
        .map(dialog_path)
        .transpose()
}

fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|e| format!("selected path is not a local file: {e}"))
}

fn require_zip_extension(path: &Path) -> Result<(), String> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        Ok(())
    } else {
        Err("HTML archive export path must end in .zip".to_string())
    }
}

fn temp_zip_path(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| "HTML archive path has no file name".to_string())?
        .to_string_lossy();
    Ok(path.with_file_name(format!("{file_name}.tmp")))
}
