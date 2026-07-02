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

const PAGE_ICON_MAX_DISPLAY_MEGABYTES: usize = 3;
const PAGE_ICON_MAX_BYTES: usize = PAGE_ICON_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const PAGE_ICON_DIR: &str = "notes/page-icons";
const PAGE_ICON_ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotePageIconAssetDto {
    pub relative_path: String,
    pub original_name: Option<String>,
    pub content_type: String,
    pub byte_size: i64,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageIconImageKind {
    Png,
    Jpeg,
    Webp,
}

impl PageIconImageKind {
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

fn page_icon_start_directory<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().picture_dir().ok().filter(|path| path.is_dir())
}

fn active_page_icon_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(vault::active_vault_path(app)?
        .join("assets")
        .join(PAGE_ICON_DIR))
}

fn asset_path_for_relative<R: Runtime>(
    app: &AppHandle<R>,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let file_name = validate_page_icon_relative_path(relative_path)?;
    Ok(active_page_icon_dir(app)?.join(file_name))
}

fn validate_page_icon_relative_path(relative_path: &str) -> Result<&str, String> {
    let relative_path = relative_path.trim();
    let prefix = format!("{PAGE_ICON_DIR}/");
    let file_name = relative_path
        .strip_prefix(&prefix)
        .ok_or_else(|| "page icon path must stay under notes/page-icons".to_string())?;
    if file_name.is_empty() {
        return Err("page icon path is missing a file name".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err("page icon path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("page icon path cannot contain nested or parent paths".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !PAGE_ICON_ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(page_icon_unsupported_type_error());
    }
    Ok(file_name)
}

fn page_icon_unsupported_type_error() -> String {
    "Use PNG, JPG, or WebP. SVG is blocked for security because it can contain interactive or external content.".to_string()
}

fn sniff_page_icon_kind(bytes: &[u8]) -> Result<PageIconImageKind, String> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Ok(PageIconImageKind::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Ok(PageIconImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(PageIconImageKind::Webp);
    }
    Err(page_icon_unsupported_type_error())
}

fn ensure_page_icon_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("page icon image is empty".to_string());
    }
    if bytes.len() > PAGE_ICON_MAX_BYTES {
        return Err(format!(
            "page icon image exceeds the {PAGE_ICON_MAX_DISPLAY_MEGABYTES} MB limit"
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
        .ok_or_else(|| "page icon target has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create page icon directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "page icon target has no file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let tmp_path = parent.join(format!("{file_name}.tmp"));
    {
        let mut file = fs::File::create(&tmp_path).map_err(|e| format!("write page icon: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("write page icon: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("sync page icon: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("save page icon: {e}"))
}

fn read_file_capped(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("inspect page icon image: {e}"))?;
    if !metadata.is_file() {
        return Err("page icon image path must be a file".to_string());
    }
    if metadata.len() > PAGE_ICON_MAX_BYTES as u64 {
        return Err(format!(
            "page icon image exceeds the {PAGE_ICON_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    let bytes = fs::read(path).map_err(|e| format!("read page icon image: {e}"))?;
    ensure_page_icon_size(&bytes)?;
    Ok(bytes)
}

fn decode_page_icon_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let trimmed = data_url.trim();
    let Some((metadata, payload)) = trimmed.split_once(',') else {
        return Err("page icon data URL is malformed".to_string());
    };
    if !metadata.starts_with("data:image/") || !metadata.ends_with(";base64") {
        return Err("page icon data URL must be a base64 image".to_string());
    }
    let bytes = general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| format!("decode page icon data URL: {e}"))?;
    ensure_page_icon_size(&bytes)?;
    Ok(bytes)
}

fn page_icon_data_url(bytes: &[u8]) -> Result<String, String> {
    ensure_page_icon_size(bytes)?;
    let kind = sniff_page_icon_kind(bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        kind.content_type(),
        general_purpose::STANDARD.encode(bytes)
    ))
}

async fn save_page_icon_bytes<R: Runtime>(
    app: &AppHandle<R>,
    db_url: String,
    bytes: Vec<u8>,
    original_name: Option<String>,
) -> Result<NotePageIconAssetDto, String> {
    ensure_page_icon_size(&bytes)?;
    let kind = sniff_page_icon_kind(&bytes)?;
    let sha256 = hex_hash(&bytes);
    let file_name = format!("{}.{}", sha256, kind.extension());
    let relative_path = format!("{PAGE_ICON_DIR}/{file_name}");
    let path = active_page_icon_dir(app)?.join(file_name);
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
        .map_err(|e| format!("begin page icon asset record: {e}"))?;
    sqlx::query(
        "INSERT INTO notes_page_icon_assets
            (id, asset_path, original_name, content_type, byte_size, sha256)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(asset_path)
         DO UPDATE SET
            original_name = COALESCE(excluded.original_name, notes_page_icon_assets.original_name),
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(&sha256)
    .bind(&relative_path)
    .bind(&original_name)
    .bind(kind.content_type())
    .bind(bytes.len() as i64)
    .bind(&sha256)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("record page icon asset: {e}"))?;
    assets::upsert_managed_asset_tx(
        &mut tx,
        NotesManagedAssetWrite {
            relative_path: &relative_path,
            original_name: original_name.as_deref(),
            content_type: kind.content_type(),
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
        .map_err(|e| format!("commit page icon asset record: {e}"))?;
    Ok(NotePageIconAssetDto {
        relative_path,
        original_name,
        content_type: kind.content_type().to_string(),
        byte_size: bytes.len() as i64,
        sha256,
    })
}

#[tauri::command]
pub async fn notes_pick_page_icon_file<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Option<NotePageIconAssetDto>, String> {
    let mut picker = app
        .dialog()
        .file()
        .set_title("Upload page icon")
        .add_filter("Image", PAGE_ICON_ALLOWED_EXTENSIONS);
    if let Some(directory) = page_icon_start_directory(&app) {
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
    save_page_icon_bytes(&app, db_url, bytes, original_name)
        .await
        .map(Some)
}

#[tauri::command]
pub async fn notes_save_page_icon_data_url<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    data_url: String,
    original_name: Option<String>,
) -> Result<NotePageIconAssetDto, String> {
    let bytes = decode_page_icon_data_url(&data_url)?;
    save_page_icon_bytes(&app, db_url, bytes, original_name).await
}

#[tauri::command]
pub fn notes_page_icon_asset_data_url<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<String, String> {
    let path = asset_path_for_relative(&app, &relative_path)?;
    let bytes = read_file_capped(&path)?;
    page_icon_data_url(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_page_icon_kind_accepts_supported_images() {
        assert_eq!(
            sniff_page_icon_kind(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).unwrap(),
            PageIconImageKind::Png,
        );
        assert_eq!(
            sniff_page_icon_kind(&[0xff, 0xd8, 0xff, 0xdb]).unwrap(),
            PageIconImageKind::Jpeg,
        );
        assert_eq!(
            sniff_page_icon_kind(b"RIFFxxxxWEBP").unwrap(),
            PageIconImageKind::Webp,
        );
    }

    #[test]
    fn validate_page_icon_relative_path_rejects_escape_paths() {
        assert!(validate_page_icon_relative_path(
            "notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        )
        .is_ok());
        assert!(validate_page_icon_relative_path("project-icons/a.png").is_err());
        assert!(validate_page_icon_relative_path("notes/page-icons/../a.png").is_err());
        assert!(validate_page_icon_relative_path("notes/page-icons/nested/a.png").is_err());
        assert!(validate_page_icon_relative_path("notes/page-icons/a.svg").is_err());
    }
}
