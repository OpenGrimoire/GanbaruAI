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

use crate::{db_path::connect_sqlite, vault};

const PAGE_COVER_MAX_DISPLAY_MEGABYTES: usize = 10;
const PAGE_COVER_MAX_BYTES: usize = PAGE_COVER_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const PAGE_COVER_DIR: &str = "notes/page-covers";
const PAGE_COVER_ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotePageCoverAssetDto {
    pub relative_path: String,
    pub original_name: Option<String>,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageCoverImageKind {
    Png,
    Jpeg,
    Webp,
}

impl PageCoverImageKind {
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

fn page_cover_start_directory<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().picture_dir().ok().filter(|path| path.is_dir())
}

fn active_page_cover_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(vault::active_vault_path(app)?
        .join("assets")
        .join(PAGE_COVER_DIR))
}

fn asset_path_for_relative<R: Runtime>(
    app: &AppHandle<R>,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let file_name = validate_page_cover_relative_path(relative_path)?;
    Ok(active_page_cover_dir(app)?.join(file_name))
}

fn validate_page_cover_relative_path(relative_path: &str) -> Result<&str, String> {
    let relative_path = relative_path.trim();
    let prefix = format!("{PAGE_COVER_DIR}/");
    let file_name = relative_path
        .strip_prefix(&prefix)
        .ok_or_else(|| "page cover path must stay under notes/page-covers".to_string())?;
    if file_name.is_empty() {
        return Err("page cover path is missing a file name".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err("page cover path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("page cover path cannot contain nested or parent paths".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !PAGE_COVER_ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(page_cover_unsupported_type_error());
    }
    Ok(file_name)
}

fn page_cover_unsupported_type_error() -> String {
    "Use PNG, JPG, or WebP. SVG is blocked for security because it can contain interactive or external content.".to_string()
}

fn sniff_page_cover_kind(bytes: &[u8]) -> Result<PageCoverImageKind, String> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Ok(PageCoverImageKind::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Ok(PageCoverImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(PageCoverImageKind::Webp);
    }
    Err(page_cover_unsupported_type_error())
}

fn ensure_page_cover_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("page cover image is empty".to_string());
    }
    if bytes.len() > PAGE_COVER_MAX_BYTES {
        return Err(format!(
            "page cover image exceeds the {PAGE_COVER_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    Ok(())
}

fn hex_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn write_binary_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "page cover target has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create page cover directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "page cover target has no file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let tmp_path = parent.join(format!("{file_name}.tmp"));
    {
        let mut file = fs::File::create(&tmp_path).map_err(|e| format!("write page cover: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("write page cover: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("sync page cover: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("save page cover: {e}"))
}

fn read_file_capped(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("inspect page cover image: {e}"))?;
    if !metadata.is_file() {
        return Err("page cover image path must be a file".to_string());
    }
    if metadata.len() > PAGE_COVER_MAX_BYTES as u64 {
        return Err(format!(
            "page cover image exceeds the {PAGE_COVER_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read page cover image: {e}"))?;
    ensure_page_cover_size(&bytes)?;
    Ok(bytes)
}

fn decode_page_cover_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let trimmed = data_url.trim();
    let Some((metadata, payload)) = trimmed.split_once(',') else {
        return Err("page cover data URL is malformed".to_string());
    };
    if !metadata.starts_with("data:image/") || !metadata.ends_with(";base64") {
        return Err("page cover data URL must be a base64 image".to_string());
    }
    let bytes = general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| format!("decode page cover data URL: {e}"))?;
    ensure_page_cover_size(&bytes)?;
    Ok(bytes)
}

fn page_cover_data_url(bytes: &[u8]) -> Result<String, String> {
    ensure_page_cover_size(bytes)?;
    let kind = sniff_page_cover_kind(bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        kind.content_type(),
        general_purpose::STANDARD.encode(bytes)
    ))
}

async fn save_page_cover_bytes<R: Runtime>(
    app: &AppHandle<R>,
    db_url: String,
    bytes: Vec<u8>,
    original_name: Option<String>,
) -> Result<NotePageCoverAssetDto, String> {
    ensure_page_cover_size(&bytes)?;
    let kind = sniff_page_cover_kind(&bytes)?;
    let sha256 = hex_hash(&bytes);
    let file_name = format!("{}.{}", sha256, kind.extension());
    let relative_path = format!("{PAGE_COVER_DIR}/{file_name}");
    let path = active_page_cover_dir(app)?.join(file_name);
    if !path.exists() {
        write_binary_file_atomically(&path, &bytes)?;
    }
    let original_name = original_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());
    let pool = connect_sqlite(app.clone(), db_url).await?;
    sqlx::query(
        "INSERT INTO notes_page_cover_assets
            (id, asset_path, original_name, content_type, byte_size, sha256)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(asset_path)
         DO UPDATE SET
            original_name = COALESCE(excluded.original_name, notes_page_cover_assets.original_name),
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(&sha256)
    .bind(&relative_path)
    .bind(&original_name)
    .bind(kind.content_type())
    .bind(bytes.len() as i64)
    .bind(&sha256)
    .execute(&pool)
    .await
    .map_err(|e| format!("record page cover asset: {e}"))?;
    Ok(NotePageCoverAssetDto {
        relative_path,
        original_name,
        content_type: kind.content_type().to_string(),
        byte_size: bytes.len() as i64,
        sha256,
    })
}

#[tauri::command]
pub async fn notes_pick_page_cover_file<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Option<NotePageCoverAssetDto>, String> {
    let mut picker = app
        .dialog()
        .file()
        .set_title("Upload page cover")
        .add_filter("Image", PAGE_COVER_ALLOWED_EXTENSIONS);
    if let Some(directory) = page_cover_start_directory(&app) {
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
    save_page_cover_bytes(&app, db_url, bytes, original_name)
        .await
        .map(Some)
}

#[tauri::command]
pub async fn notes_save_page_cover_data_url<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    data_url: String,
    original_name: Option<String>,
) -> Result<NotePageCoverAssetDto, String> {
    let bytes = decode_page_cover_data_url(&data_url)?;
    save_page_cover_bytes(&app, db_url, bytes, original_name).await
}

#[tauri::command]
pub fn notes_page_cover_asset_data_url<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<String, String> {
    let path = asset_path_for_relative(&app, &relative_path)?;
    let bytes = read_file_capped(&path)?;
    page_cover_data_url(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_page_cover_kind_accepts_supported_images() {
        assert_eq!(
            sniff_page_cover_kind(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).unwrap(),
            PageCoverImageKind::Png,
        );
        assert_eq!(
            sniff_page_cover_kind(&[0xff, 0xd8, 0xff, 0xdb]).unwrap(),
            PageCoverImageKind::Jpeg,
        );
        assert_eq!(
            sniff_page_cover_kind(b"RIFFxxxxWEBP").unwrap(),
            PageCoverImageKind::Webp,
        );
    }

    #[test]
    fn validate_page_cover_relative_path_rejects_escape_paths() {
        assert!(validate_page_cover_relative_path(
            "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        )
        .is_ok());
        assert!(validate_page_cover_relative_path("notes/page-icons/a.png").is_err());
        assert!(validate_page_cover_relative_path("notes/page-covers/../a.png").is_err());
        assert!(validate_page_cover_relative_path("notes/page-covers/nested/a.png").is_err());
        assert!(validate_page_cover_relative_path("notes/page-covers/a.svg").is_err());
    }
}
