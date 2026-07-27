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

use crate::vault;

const PROFILE_IMAGE_MAX_DISPLAY_MEGABYTES: usize = 3;
const PROFILE_IMAGE_MAX_BYTES: usize = PROFILE_IMAGE_MAX_DISPLAY_MEGABYTES * 1024 * 1024;
const PROFILE_IMAGE_DIR: &str = "profile";
const PROFILE_IMAGE_ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileImageAsset {
    pub relative_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProfileImageKind {
    Png,
    Jpeg,
    Webp,
}

impl ProfileImageKind {
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
        .map_err(|error| format!("selected path is not a local file: {error}"))
}

fn profile_image_start_directory<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().picture_dir().ok().filter(|path| path.is_dir())
}

fn active_profile_image_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(vault::active_vault_path(app)?
        .join("assets")
        .join(PROFILE_IMAGE_DIR))
}

fn validate_profile_image_relative_path(relative_path: &str) -> Result<&str, String> {
    let relative_path = relative_path.trim();
    let prefix = format!("{PROFILE_IMAGE_DIR}/");
    let file_name = relative_path
        .strip_prefix(&prefix)
        .ok_or_else(|| "profile image path must stay under profile".to_string())?;
    if file_name.is_empty() {
        return Err("profile image path is missing a file name".to_string());
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err("profile image path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("profile image path cannot contain nested or parent paths".to_string());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !PROFILE_IMAGE_ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(profile_image_unsupported_type_error());
    }
    Ok(file_name)
}

fn profile_image_unsupported_type_error() -> String {
    "Use PNG, JPG, or WebP. SVG is blocked for security because it can contain interactive or external content.".to_string()
}

fn sniff_profile_image_kind(bytes: &[u8]) -> Result<ProfileImageKind, String> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Ok(ProfileImageKind::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Ok(ProfileImageKind::Jpeg);
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(ProfileImageKind::Webp);
    }
    Err(profile_image_unsupported_type_error())
}

fn ensure_profile_image_size(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err("profile image is empty".to_string());
    }
    if bytes.len() > PROFILE_IMAGE_MAX_BYTES {
        return Err(format!(
            "profile image exceeds the {PROFILE_IMAGE_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    Ok(())
}

fn read_file_capped(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("inspect profile image: {error}"))?;
    if !metadata.is_file() {
        return Err("profile image path must be a file".to_string());
    }
    if metadata.len() > PROFILE_IMAGE_MAX_BYTES as u64 {
        return Err(format!(
            "profile image exceeds the {PROFILE_IMAGE_MAX_DISPLAY_MEGABYTES} MB limit"
        ));
    }
    let bytes = fs::read(path).map_err(|error| format!("read profile image: {error}"))?;
    ensure_profile_image_size(&bytes)?;
    Ok(bytes)
}

fn content_hash(bytes: &[u8]) -> String {
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
        .ok_or_else(|| "profile image target has no parent".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("create profile image directory: {error}"))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "profile image target has no file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let temporary_path = parent.join(format!("{file_name}.tmp"));
    {
        let mut file = fs::File::create(&temporary_path)
            .map_err(|error| format!("write profile image: {error}"))?;
        file.write_all(bytes)
            .map_err(|error| format!("write profile image: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("sync profile image: {error}"))?;
    }
    fs::rename(&temporary_path, path).map_err(|error| format!("save profile image: {error}"))
}

fn save_profile_image_bytes<R: Runtime>(
    app: &AppHandle<R>,
    bytes: Vec<u8>,
) -> Result<ProfileImageAsset, String> {
    ensure_profile_image_size(&bytes)?;
    let kind = sniff_profile_image_kind(&bytes)?;
    let file_name = format!("{}.{}", content_hash(&bytes), kind.extension());
    let relative_path = format!("{PROFILE_IMAGE_DIR}/{file_name}");
    let path = active_profile_image_dir(app)?.join(file_name);
    if !path.exists() {
        write_binary_file_atomically(&path, &bytes)?;
    }
    Ok(ProfileImageAsset { relative_path })
}

fn profile_image_asset_path<R: Runtime>(
    app: &AppHandle<R>,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let file_name = validate_profile_image_relative_path(relative_path)?;
    Ok(active_profile_image_dir(app)?.join(file_name))
}

#[tauri::command]
pub async fn profile_image_pick_file<R: Runtime>(
    app: AppHandle<R>,
    title: String,
) -> Result<Option<ProfileImageAsset>, String> {
    let mut picker = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter("Image", PROFILE_IMAGE_ALLOWED_EXTENSIONS);
    if let Some(directory) = profile_image_start_directory(&app) {
        picker = picker.set_directory(directory);
    }
    let Some(path) = picker.blocking_pick_file().map(dialog_path).transpose()? else {
        return Ok(None);
    };
    let bytes = read_file_capped(&path)?;
    save_profile_image_bytes(&app, bytes).map(Some)
}

#[tauri::command]
pub fn profile_image_asset_data_url<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<String, String> {
    let path = profile_image_asset_path(&app, &relative_path)?;
    let bytes = read_file_capped(&path)?;
    let kind = sniff_profile_image_kind(&bytes)?;
    Ok(format!(
        "data:{};base64,{}",
        kind.mime_type(),
        general_purpose::STANDARD.encode(bytes)
    ))
}

#[tauri::command]
pub fn profile_image_delete_file<R: Runtime>(
    app: AppHandle<R>,
    relative_path: String,
) -> Result<(), String> {
    let path = profile_image_asset_path(&app, &relative_path)?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("delete profile image: {error}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_image_kind_accepts_supported_images() {
        assert_eq!(
            sniff_profile_image_kind(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]).unwrap(),
            ProfileImageKind::Png
        );
        assert_eq!(
            sniff_profile_image_kind(&[0xff, 0xd8, 0xff, 0xdb]).unwrap(),
            ProfileImageKind::Jpeg
        );
        assert_eq!(
            sniff_profile_image_kind(b"RIFFxxxxWEBPmore").unwrap(),
            ProfileImageKind::Webp
        );
    }

    #[test]
    fn profile_image_kind_rejects_unsafe_or_unknown_images() {
        assert!(
            sniff_profile_image_kind(br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#).is_err()
        );
        assert!(sniff_profile_image_kind(b"GIF89amore").is_err());
        assert!(sniff_profile_image_kind(b"not-image").is_err());
    }

    #[test]
    fn profile_image_relative_path_stays_in_managed_directory() {
        assert_eq!(
            validate_profile_image_relative_path(
                "profile/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
            )
            .unwrap(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png"
        );
        assert!(validate_profile_image_relative_path("../outside.png").is_err());
        assert!(validate_profile_image_relative_path("profile/../outside.png").is_err());
        assert!(validate_profile_image_relative_path("profile/nested/image.png").is_err());
        assert!(validate_profile_image_relative_path("profile/image.svg").is_err());
    }
}
