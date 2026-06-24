use base64::{engine::general_purpose, Engine as _};
use reqwest::redirect::Policy;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{db_path::connect_sqlite, vault};

const PROJECT_ICON_MAX_DISPLAY_MEGABYTES: usize = 3;
const PROJECT_ICON_MAX_BYTES: usize = PROJECT_ICON_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const PROJECT_ICON_DIR: &str = "project-icons";
const PROJECT_ICON_ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIconAsset {
    pub relative_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProjectIconImageKind {
    Png,
    Jpeg,
    Webp,
}

impl ProjectIconImageKind {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }

    fn mime_type(self) -> &'static str {
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

fn project_icon_start_directory<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().picture_dir().ok().filter(|path| path.is_dir())
}

fn active_project_icon_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(vault::active_vault_path(app)?
        .join("assets")
        .join(PROJECT_ICON_DIR))
}

fn asset_path_for_relative<R: Runtime>(
    app: &AppHandle<R>,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let file_name = validate_project_icon_relative_path(relative_path)?;
    Ok(active_project_icon_dir(app)?.join(file_name))
}

fn validate_project_icon_relative_path(relative_path: &str) -> Result<&str, String> {
    let relative_path = relative_path.trim();
    let prefix = format!("{PROJECT_ICON_DIR}/");
    let file_name = relative_path
        .strip_prefix(&prefix)
        .ok_or_else(|| "project icon path must stay under project-icons".to_string())?;
    if file_name.is_empty() {
        return Err("project icon path is missing a file name".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err("project icon path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("project icon path cannot contain nested or parent paths".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !PROJECT_ICON_ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(project_icon_unsupported_type_error());
    }
    Ok(file_name)
}

fn project_icon_unsupported_type_error() -> String {
    "Use PNG, JPG, or WebP. SVG is blocked for security because it can contain interactive or external content.".to_string()
}

fn sniff_project_icon_kind(bytes: &[u8]) -> Result<ProjectIconImageKind, String> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Ok(ProjectIconImageKind::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Ok(ProjectIconImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(ProjectIconImageKind::Webp);
    }
    Err(project_icon_unsupported_type_error())
}

fn ensure_project_icon_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("project icon image is empty".to_string());
    }
    if bytes.len() > PROJECT_ICON_MAX_BYTES {
        return Err(project_icon_size_limit_error());
    }
    Ok(())
}

fn project_icon_size_limit_label() -> String {
    format!("{PROJECT_ICON_MAX_DISPLAY_MEGABYTES} MB")
}

fn project_icon_size_limit_error() -> String {
    format!(
        "project icon image exceeds the {} limit",
        project_icon_size_limit_label()
    )
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
        .ok_or_else(|| "project icon target has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("create project icon directory: {e}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "project icon target has no file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let tmp_path = parent.join(format!("{file_name}.tmp"));
    {
        let mut file =
            fs::File::create(&tmp_path).map_err(|e| format!("write project icon: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("write project icon: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("sync project icon: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("save project icon: {e}"))
}

fn save_project_icon_bytes<R: Runtime>(
    app: &AppHandle<R>,
    bytes: Vec<u8>,
) -> Result<ProjectIconAsset, String> {
    ensure_project_icon_size(&bytes)?;
    let kind = sniff_project_icon_kind(&bytes)?;
    let file_name = format!("{}.{}", hex_hash(&bytes), kind.extension());
    let relative_path = format!("{PROJECT_ICON_DIR}/{file_name}");
    let path = active_project_icon_dir(app)?.join(file_name);
    if !path.exists() {
        write_binary_file_atomically(&path, &bytes)?;
    }
    Ok(ProjectIconAsset { relative_path })
}

fn read_file_capped(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("inspect project icon image: {e}"))?;
    if !metadata.is_file() {
        return Err("project icon image path must be a file".to_string());
    }
    if metadata.len() > PROJECT_ICON_MAX_BYTES as u64 {
        return Err(project_icon_size_limit_error());
    }
    let bytes = fs::read(path).map_err(|e| format!("read project icon image: {e}"))?;
    ensure_project_icon_size(&bytes)?;
    Ok(bytes)
}

fn decode_project_icon_data_url(data_url: &str) -> Result<Vec<u8>, String> {
    let trimmed = data_url.trim();
    let Some((metadata, payload)) = trimmed.split_once(',') else {
        return Err("project icon data URL is malformed".to_string());
    };
    if !metadata.starts_with("data:image/") || !metadata.ends_with(";base64") {
        return Err("project icon data URL must be a base64 image".to_string());
    }
    let bytes = general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| format!("decode project icon data URL: {e}"))?;
    ensure_project_icon_size(&bytes)?;
    Ok(bytes)
}

fn project_icon_data_url(bytes: &[u8]) -> Result<String, String> {
    ensure_project_icon_size(bytes)?;
    let kind = sniff_project_icon_kind(bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        kind.mime_type(),
        general_purpose::STANDARD.encode(bytes)
    ))
}

async fn download_project_icon_url(url: &str) -> Result<Vec<u8>, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("invalid project icon URL: {e}"))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("project icon URL must use http or https".to_string());
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(Policy::limited(4))
        .build()
        .map_err(|e| format!("create project icon HTTP client: {e}"))?;
    let mut response = client
        .get(parsed)
        .send()
        .await
        .map_err(|e| format!("download project icon image: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("project icon URL returned {}", response.status()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > PROJECT_ICON_MAX_BYTES as u64)
    {
        return Err(project_icon_size_limit_error());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("read project icon image: {e}"))?
    {
        bytes.extend_from_slice(&chunk);
        if bytes.len() > PROJECT_ICON_MAX_BYTES {
            return Err(project_icon_size_limit_error());
        }
    }
    ensure_project_icon_size(&bytes)?;
    Ok(bytes)
}

#[tauri::command]
pub async fn project_icon_pick_image_file<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<ProjectIconAsset>, String> {
    let mut picker = app
        .dialog()
        .file()
        .set_title("Upload project icon")
        .add_filter("Image", PROJECT_ICON_ALLOWED_EXTENSIONS);
    if let Some(directory) = project_icon_start_directory(&app) {
        picker = picker.set_directory(directory);
    }
    let Some(path) = picker.blocking_pick_file().map(dialog_path).transpose()? else {
        return Ok(None);
    };
    let bytes = read_file_capped(&path)?;
    save_project_icon_bytes(&app, bytes).map(Some)
}

#[tauri::command]
pub fn project_icon_save_image_data_url<R: Runtime>(
    app: AppHandle<R>,
    data_url: String,
) -> Result<ProjectIconAsset, String> {
    let bytes = decode_project_icon_data_url(&data_url)?;
    save_project_icon_bytes(&app, bytes)
}

#[tauri::command]
pub async fn project_icon_download_image_url<R: Runtime>(
    app: AppHandle<R>,
    url: String,
) -> Result<ProjectIconAsset, String> {
    let bytes = download_project_icon_url(&url).await?;
    save_project_icon_bytes(&app, bytes)
}

#[tauri::command]
pub fn project_icon_asset_path<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<String, String> {
    let path = asset_path_for_relative(&app, &relative_path)?;
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "project icon path contains non-utf8 characters".to_string())
}

#[tauri::command]
pub fn project_icon_asset_data_url<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<String, String> {
    let path = asset_path_for_relative(&app, &relative_path)?;
    let bytes = read_file_capped(&path)?;
    project_icon_data_url(&bytes)
}

#[tauri::command]
pub async fn project_icon_delete_assets_if_unreferenced<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    relative_paths: Vec<String>,
) -> Result<(), String> {
    let mut candidates = HashSet::new();
    for relative_path in relative_paths {
        validate_project_icon_relative_path(&relative_path)?;
        candidates.insert(relative_path);
    }
    if candidates.is_empty() {
        return Ok(());
    }
    let pool = connect_sqlite(app.clone(), db_url).await?;
    let mut referenced = HashSet::new();
    for row in sqlx::query_scalar::<_, String>(
        "SELECT icon FROM project_groups
         UNION ALL SELECT icon FROM projects
         UNION ALL SELECT ('asset:' || asset_path) FROM project_custom_emojis",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("load project icon references: {e}"))?
    {
        if let Some(relative_path) = row.strip_prefix("asset:") {
            referenced.insert(relative_path.to_string());
        }
    }
    for relative_path in candidates.difference(&referenced) {
        let path = asset_path_for_relative(&app, relative_path)?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("delete project icon asset: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniff_project_icon_kind_accepts_supported_images() {
        assert_eq!(
            sniff_project_icon_kind(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).unwrap(),
            ProjectIconImageKind::Png,
        );
        assert_eq!(
            sniff_project_icon_kind(&[0xff, 0xd8, 0xff, 0xdb]).unwrap(),
            ProjectIconImageKind::Jpeg,
        );
        assert_eq!(
            sniff_project_icon_kind(b"RIFFxxxxWEBPmore").unwrap(),
            ProjectIconImageKind::Webp,
        );
    }

    #[test]
    fn sniff_project_icon_kind_rejects_unsupported_icon_images() {
        let expected = project_icon_unsupported_type_error();
        assert_eq!(
            sniff_project_icon_kind(b"GIF89amore").unwrap_err(),
            expected
        );
        assert_eq!(
            sniff_project_icon_kind(br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#)
                .unwrap_err(),
            expected,
        );
        assert_eq!(sniff_project_icon_kind(b"not-image").unwrap_err(), expected);
    }

    #[test]
    fn validate_project_icon_relative_path_rejects_escape_paths() {
        assert_eq!(
            validate_project_icon_relative_path("project-icons/abc.png").unwrap(),
            "abc.png",
        );
        assert!(validate_project_icon_relative_path("../abc.png").is_err());
        assert!(validate_project_icon_relative_path("project-icons/../abc.png").is_err());
        assert!(validate_project_icon_relative_path("project-icons/nested/abc.png").is_err());
        assert!(validate_project_icon_relative_path("project-icons/abc.gif").is_err());
        assert!(validate_project_icon_relative_path("project-icons/abc.svg").is_err());
        assert!(validate_project_icon_relative_path("project-icons/abc.txt").is_err());
    }

    #[test]
    fn decode_project_icon_data_url_requires_base64_image() {
        let data_url = format!(
            "data:image/png;base64,{}",
            general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
        );
        assert!(decode_project_icon_data_url(&data_url).is_ok());
        assert!(decode_project_icon_data_url("data:text/plain;base64,SGk=").is_err());
    }

    #[test]
    fn project_icon_data_url_uses_sniffed_image_mime_type() {
        let bytes = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        assert_eq!(
            project_icon_data_url(&bytes).unwrap(),
            format!(
                "data:image/png;base64,{}",
                general_purpose::STANDARD.encode(bytes)
            ),
        );
    }

    #[test]
    fn project_icon_size_limit_error_uses_human_units() {
        assert_eq!(
            project_icon_size_limit_error(),
            "project icon image exceeds the 3 MB limit",
        );
    }
}
