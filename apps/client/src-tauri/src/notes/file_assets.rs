use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};

use super::assets::{
    self, NotesManagedAssetWrite, NOTES_ASSET_SOURCE_LOCAL_UPLOAD, NOTES_ASSET_STATE_AVAILABLE,
};
use crate::{db_path::connect_sqlite, vault};

const NOTES_FILE_MAX_DISPLAY_MEGABYTES: usize = 50;
const NOTES_FILE_MAX_BYTES: usize = NOTES_FILE_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const NOTES_FILE_PREVIEW_MAX_DISPLAY_MEGABYTES: i64 = 25;
const NOTES_FILE_PREVIEW_MAX_BYTES: i64 = NOTES_FILE_PREVIEW_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const NOTES_FILE_DIR: &str = "notes/files";
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg", "oga", "m4a"];
const VIDEO_EXTENSIONS: &[&str] = &[
    "amv", "asf", "avi", "f4v", "flv", "gifv", "mkv", "mov", "mpg", "mpeg", "mpv", "mp4", "m4v",
    "qt", "wmv",
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesFileAssetDto {
    pub relative_path: String,
    pub original_name: Option<String>,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
    pub kind: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotesFileAssetKind {
    Image,
    Video,
    Audio,
    Pdf,
    File,
}

impl NotesFileAssetKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Pdf => "pdf",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotesImageKind {
    Png,
    Jpeg,
    Webp,
}

impl NotesImageKind {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

fn dialog_path(path: FilePath) -> Result<PathBuf, String> {
    path.into_path()
        .map_err(|e| format!("selected path is not a local file: {e}"))
}

fn notes_file_start_directory<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().document_dir().ok().filter(|path| path.is_dir())
}

fn active_notes_file_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(vault::active_vault_path(app)?
        .join("assets")
        .join(NOTES_FILE_DIR))
}

fn asset_path_for_relative<R: Runtime>(
    app: &AppHandle<R>,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let file_name = validate_notes_file_relative_path(relative_path)?;
    Ok(active_notes_file_dir(app)?.join(file_name))
}

fn validate_notes_file_relative_path(relative_path: &str) -> Result<&str, String> {
    let relative_path = relative_path.trim();
    let prefix = format!("{NOTES_FILE_DIR}/");
    let file_name = relative_path
        .strip_prefix(&prefix)
        .ok_or_else(|| "notes file path must stay under notes/files".to_string())?;
    if file_name.is_empty() {
        return Err("notes file path is missing a file name".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err("notes file path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("notes file path cannot contain nested or parent paths".to_string());
    }
    Ok(file_name)
}

fn ensure_notes_file_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("notes file is empty".to_string());
    }
    if bytes.len() > NOTES_FILE_MAX_BYTES {
        return Err(format!(
            "notes file exceeds the {NOTES_FILE_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    Ok(())
}

fn read_file_capped(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("inspect notes file: {e}"))?;
    if !metadata.is_file() {
        return Err("notes file path must be a file".to_string());
    }
    if metadata.len() > NOTES_FILE_MAX_BYTES as u64 {
        return Err(format!(
            "notes file exceeds the {NOTES_FILE_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read notes file: {e}"))?;
    ensure_notes_file_size(&bytes)?;
    Ok(bytes)
}

fn read_preview_file_capped(path: &Path, byte_size: i64) -> Result<Vec<u8>, String> {
    if byte_size > NOTES_FILE_PREVIEW_MAX_BYTES {
        return Err(format!(
            "notes file exceeds the {NOTES_FILE_PREVIEW_MAX_DISPLAY_MEGABYTES} MB preview limit"
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read notes file preview: {e}"))?;
    if bytes.len() as i64 != byte_size {
        return Err("notes file size no longer matches its recorded metadata".to_string());
    }
    Ok(bytes)
}

fn write_binary_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "notes file target has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create notes file directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "notes file target has no file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let tmp_path = parent.join(format!("{file_name}.tmp"));
    {
        let mut file = fs::File::create(&tmp_path).map_err(|e| format!("write notes file: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("write notes file: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("sync notes file: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("save notes file: {e}"))
}

fn hex_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn block_type_kind(block_type: &str) -> Result<NotesFileAssetKind, String> {
    match block_type.trim() {
        "image" => Ok(NotesFileAssetKind::Image),
        "video" => Ok(NotesFileAssetKind::Video),
        "audio" => Ok(NotesFileAssetKind::Audio),
        "pdf" => Ok(NotesFileAssetKind::Pdf),
        "file" => Ok(NotesFileAssetKind::File),
        _ => Err("notes file assets are only supported for media and file blocks".to_string()),
    }
}

fn sniff_image_kind(bytes: &[u8]) -> Option<NotesImageKind> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Some(NotesImageKind::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some(NotesImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(NotesImageKind::Webp);
    }
    None
}

fn path_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.trim().to_ascii_lowercase())
        .filter(|extension| {
            !extension.is_empty()
                && extension.len() <= 16
                && extension.chars().all(|value| value.is_ascii_alphanumeric())
        })
}

fn extension_is(path: &Path, allowed: &[&str]) -> bool {
    path_extension(path).is_some_and(|extension| {
        allowed
            .iter()
            .any(|allowed_extension| extension.eq_ignore_ascii_case(allowed_extension))
    })
}

fn classify_selected_file(
    requested_kind: NotesFileAssetKind,
    path: &Path,
    bytes: &[u8],
) -> Result<(NotesFileAssetKind, &'static str, String), String> {
    match requested_kind {
        NotesFileAssetKind::Image => {
            let Some(image_kind) = sniff_image_kind(bytes) else {
                return Err("local image blocks support PNG, JPG, and WebP files".to_string());
            };
            Ok((
                NotesFileAssetKind::Image,
                image_kind.content_type(),
                image_kind.extension().to_string(),
            ))
        }
        NotesFileAssetKind::Pdf => {
            if !bytes.starts_with(b"%PDF-") || !extension_is(path, &["pdf"]) {
                return Err("local PDF blocks require a PDF file".to_string());
            }
            Ok((
                NotesFileAssetKind::Pdf,
                "application/pdf",
                "pdf".to_string(),
            ))
        }
        NotesFileAssetKind::Audio => {
            let Some(extension) = path_extension(path) else {
                return Err("local audio blocks require a supported audio extension".to_string());
            };
            if !AUDIO_EXTENSIONS.contains(&extension.as_str()) {
                return Err(
                    "local audio blocks support MP3, WAV, OGG, OGA, and M4A files".to_string(),
                );
            }
            Ok((
                NotesFileAssetKind::Audio,
                audio_content_type(&extension),
                extension,
            ))
        }
        NotesFileAssetKind::Video => {
            let Some(extension) = path_extension(path) else {
                return Err("local video blocks require a supported video extension".to_string());
            };
            if !VIDEO_EXTENSIONS.contains(&extension.as_str()) {
                return Err("local video blocks require a supported video file".to_string());
            }
            Ok((
                NotesFileAssetKind::Video,
                video_content_type(&extension),
                extension,
            ))
        }
        NotesFileAssetKind::File => {
            let extension = path_extension(path).unwrap_or_else(|| "bin".to_string());
            let content_type = generic_content_type(&extension);
            Ok((kind_for_content_type(content_type), content_type, extension))
        }
    }
}

fn audio_content_type(extension: &str) -> &'static str {
    match extension {
        "m4a" => "audio/mp4",
        "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        _ => "audio/mpeg",
    }
}

fn video_content_type(extension: &str) -> &'static str {
    match extension {
        "avi" => "video/x-msvideo",
        "flv" | "f4v" => "video/x-flv",
        "mkv" => "video/x-matroska",
        "mov" | "qt" => "video/quicktime",
        "mpeg" | "mpg" => "video/mpeg",
        "wmv" => "video/x-ms-wmv",
        _ => "video/mp4",
    }
}

fn generic_content_type(extension: &str) -> &'static str {
    match extension {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        extension if VIDEO_EXTENSIONS.contains(&extension) => video_content_type(extension),
        "json" => "application/json",
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        _ => "application/octet-stream",
    }
}

fn kind_for_content_type(content_type: &str) -> NotesFileAssetKind {
    if matches!(content_type, "image/png" | "image/jpeg" | "image/webp") {
        NotesFileAssetKind::Image
    } else if content_type.starts_with("video/") {
        NotesFileAssetKind::Video
    } else if content_type.starts_with("audio/") {
        NotesFileAssetKind::Audio
    } else if content_type == "application/pdf" {
        NotesFileAssetKind::Pdf
    } else {
        NotesFileAssetKind::File
    }
}

fn picker_extensions_for_kind(
    kind: NotesFileAssetKind,
) -> Option<(&'static str, &'static [&'static str])> {
    match kind {
        NotesFileAssetKind::Image => Some(("Image", IMAGE_EXTENSIONS)),
        NotesFileAssetKind::Video => Some(("Video", VIDEO_EXTENSIONS)),
        NotesFileAssetKind::Audio => Some(("Audio", AUDIO_EXTENSIONS)),
        NotesFileAssetKind::Pdf => Some(("PDF", &["pdf"])),
        NotesFileAssetKind::File => None,
    }
}

async fn save_notes_file_bytes<R: Runtime>(
    app: &AppHandle<R>,
    db_url: String,
    requested_kind: NotesFileAssetKind,
    source_path: &Path,
    bytes: Vec<u8>,
    original_name: Option<String>,
) -> Result<NotesFileAssetDto, String> {
    ensure_notes_file_size(&bytes)?;
    let (kind, content_type, extension) =
        classify_selected_file(requested_kind, source_path, &bytes)?;
    let sha256 = hex_hash(&bytes);
    let file_name = format!("{sha256}.{extension}");
    let relative_path = format!("{NOTES_FILE_DIR}/{file_name}");
    let path = active_notes_file_dir(app)?.join(file_name);
    if !path.exists() {
        write_binary_file_atomically(&path, &bytes)?;
    }
    let original_name = original_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());
    let pool = connect_sqlite(app.clone(), db_url).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes file asset record: {e}"))?;
    assets::upsert_managed_asset_tx(
        &mut tx,
        NotesManagedAssetWrite {
            relative_path: &relative_path,
            original_name: original_name.as_deref(),
            content_type,
            byte_size: bytes.len() as i64,
            sha256: &sha256,
            source_type: NOTES_ASSET_SOURCE_LOCAL_UPLOAD,
            storage_state: NOTES_ASSET_STATE_AVAILABLE,
            missing_at: None,
        },
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes file asset record: {e}"))?;
    Ok(NotesFileAssetDto {
        relative_path,
        original_name,
        content_type: content_type.to_string(),
        byte_size: bytes.len() as i64,
        sha256,
        kind: kind.as_str().to_string(),
    })
}

#[tauri::command]
pub async fn notes_pick_file_asset<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_type: String,
) -> Result<Option<NotesFileAssetDto>, String> {
    let requested_kind = block_type_kind(&block_type)?;
    let mut picker = app.dialog().file().set_title("Attach local file");
    if let Some((filter_name, extensions)) = picker_extensions_for_kind(requested_kind) {
        picker = picker.add_filter(filter_name, extensions);
    }
    if let Some(directory) = notes_file_start_directory(&app) {
        picker = picker.set_directory(directory);
    }
    let Some(path) = picker.blocking_pick_file().map(dialog_path).transpose()? else {
        return Ok(None);
    };
    let original_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned);
    let bytes = read_file_capped(&path)?;
    save_notes_file_bytes(&app, db_url, requested_kind, &path, bytes, original_name)
        .await
        .map(Some)
}

#[tauri::command]
pub async fn notes_file_asset_data_url<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    relative_path: String,
) -> Result<String, String> {
    let path = asset_path_for_relative(&app, &relative_path)?;
    let pool = connect_sqlite(app.clone(), db_url.clone()).await?;
    let asset: Option<(String, i64)> = sqlx::query_as(
        "SELECT content_type, byte_size
         FROM notes_assets
         WHERE asset_path = ? AND asset_path GLOB 'notes/files/*'",
    )
    .bind(relative_path.trim())
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("load notes file asset metadata: {e}"))?;
    let Some((content_type, byte_size)) = asset else {
        return Err("notes file asset not found".to_string());
    };
    let bytes = match read_preview_file_capped(&path, byte_size) {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ =
                assets::mark_managed_asset_storage_state(&pool, &relative_path, true, "notes file")
                    .await;
            return Err(error);
        }
    };
    assets::mark_managed_asset_storage_state(&pool, &relative_path, false, "notes file").await?;
    Ok(format!(
        "data:{content_type};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_local_media_files() {
        let png = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        assert_eq!(
            classify_selected_file(NotesFileAssetKind::Image, Path::new("image.png"), &png)
                .unwrap(),
            (NotesFileAssetKind::Image, "image/png", "png".to_string(),),
        );
        assert_eq!(
            classify_selected_file(
                NotesFileAssetKind::Pdf,
                Path::new("report.pdf"),
                b"%PDF-1.7"
            )
            .unwrap(),
            (
                NotesFileAssetKind::Pdf,
                "application/pdf",
                "pdf".to_string()
            ),
        );
        assert_eq!(
            classify_selected_file(
                NotesFileAssetKind::Audio,
                Path::new("focus.m4a"),
                b"not sniffed",
            )
            .unwrap(),
            (NotesFileAssetKind::Audio, "audio/mp4", "m4a".to_string()),
        );
        assert_eq!(
            classify_selected_file(
                NotesFileAssetKind::Video,
                Path::new("clip.mp4"),
                b"not sniffed",
            )
            .unwrap(),
            (NotesFileAssetKind::Video, "video/mp4", "mp4".to_string()),
        );
    }

    #[test]
    fn rejects_unsafe_or_mismatched_local_media_files() {
        assert_eq!(
            classify_selected_file(
                NotesFileAssetKind::Image,
                Path::new("image.svg"),
                b"<svg />"
            )
            .unwrap_err(),
            "local image blocks support PNG, JPG, and WebP files",
        );
        assert_eq!(
            classify_selected_file(
                NotesFileAssetKind::Pdf,
                Path::new("report.txt"),
                b"%PDF-1.7"
            )
            .unwrap_err(),
            "local PDF blocks require a PDF file",
        );
        assert_eq!(
            classify_selected_file(NotesFileAssetKind::Audio, Path::new("focus.exe"), b"")
                .unwrap_err(),
            "local audio blocks support MP3, WAV, OGG, OGA, and M4A files",
        );
        assert_eq!(
            validate_notes_file_relative_path("notes/files/nested/file.pdf").unwrap_err(),
            "notes file path cannot contain nested or parent paths",
        );
    }
}
