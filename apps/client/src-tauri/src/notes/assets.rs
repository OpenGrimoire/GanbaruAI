use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use std::path::{Component, Path};

pub(in crate::notes) const NOTES_ASSET_SOURCE_LOCAL_UPLOAD: &str = "local_upload";
pub(in crate::notes) const NOTES_ASSET_STATE_AVAILABLE: &str = "available";

const NOTES_ASSET_PAGE_ICON_PREFIX: &str = "notes/page-icons/";
const NOTES_ASSET_PAGE_COVER_PREFIX: &str = "notes/page-covers/";
const NOTES_ASSET_FILE_PREFIX: &str = "notes/files/";
const NOTES_ASSET_ALLOWED_IMAGE_CONTENT_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp"];
const NOTES_ASSET_ALLOWED_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];
const NOTES_ASSET_SOURCE_TYPES: &[&str] = &[
    "local_upload",
    "generated",
    "imported",
    "external_reference",
];
const NOTES_ASSET_STORAGE_STATES: &[&str] = &["available", "missing"];

pub(in crate::notes) struct NotesManagedAssetWrite<'a> {
    pub(in crate::notes) relative_path: &'a str,
    pub(in crate::notes) original_name: Option<&'a str>,
    pub(in crate::notes) content_type: &'a str,
    pub(in crate::notes) byte_size: i64,
    pub(in crate::notes) sha256: &'a str,
    pub(in crate::notes) source_type: &'a str,
    pub(in crate::notes) storage_state: &'a str,
    pub(in crate::notes) missing_at: Option<&'a str>,
}

pub(in crate::notes) async fn upsert_managed_asset_tx(
    tx: &mut Transaction<'_, Sqlite>,
    asset: NotesManagedAssetWrite<'_>,
) -> Result<(), String> {
    let kind = validate_managed_asset(&asset)?;
    let original_name = asset
        .original_name
        .map(str::trim)
        .filter(|name| !name.is_empty());
    sqlx::query(
        "INSERT INTO notes_assets (
            id,
            asset_path,
            kind,
            source_type,
            original_name,
            content_type,
            byte_size,
            sha256,
            storage_state,
            missing_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(asset_path)
         DO UPDATE SET
            kind = excluded.kind,
            source_type = excluded.source_type,
            original_name = COALESCE(excluded.original_name, notes_assets.original_name),
            content_type = excluded.content_type,
            byte_size = excluded.byte_size,
            sha256 = excluded.sha256,
            storage_state = excluded.storage_state,
            missing_at = excluded.missing_at,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(asset.relative_path.trim())
    .bind(asset.relative_path.trim())
    .bind(kind)
    .bind(asset.source_type.trim())
    .bind(original_name)
    .bind(asset.content_type.trim())
    .bind(asset.byte_size)
    .bind(asset.sha256.trim())
    .bind(asset.storage_state.trim())
    .bind(asset.missing_at.map(str::trim))
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("record notes asset: {e}"))?;
    Ok(())
}

pub(in crate::notes) async fn sync_page_asset_references_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
    icon_is_set: bool,
    icon: Option<&Value>,
    cover_is_set: bool,
    cover: Option<&Value>,
) -> Result<(), String> {
    if icon_is_set {
        replace_page_asset_reference_tx(
            tx,
            page_id,
            "page_icon",
            icon,
            "icon",
            NOTES_ASSET_PAGE_ICON_PREFIX,
        )
        .await?;
    }
    if cover_is_set {
        replace_page_asset_reference_tx(
            tx,
            page_id,
            "page_cover",
            cover,
            "cover",
            NOTES_ASSET_PAGE_COVER_PREFIX,
        )
        .await?;
    }
    Ok(())
}

pub(in crate::notes) async fn sync_block_asset_reference_tx(
    tx: &mut Transaction<'_, Sqlite>,
    block_id: &str,
    page_id: &str,
    block_type: &str,
    payload: &Value,
) -> Result<(), String> {
    if !matches!(block_type, "image" | "video" | "audio" | "file" | "pdf") {
        return Ok(());
    }
    sqlx::query(
        "DELETE FROM notes_asset_references
         WHERE owner_type = 'block' AND owner_id = ? AND role = 'block_file'",
    )
    .bind(block_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("clear notes block asset reference: {e}"))?;

    let Some(asset) = media_local_file_asset(payload, block_type)? else {
        return Ok(());
    };
    let asset_path = asset.relative_path.trim().to_string();
    upsert_managed_asset_tx(tx, asset).await?;
    sqlx::query(
        "INSERT INTO notes_asset_references (
            asset_id,
            owner_type,
            owner_id,
            page_id,
            block_id,
            role
         )
         VALUES (?, 'block', ?, ?, ?, 'block_file')
         ON CONFLICT(asset_id, owner_type, owner_id, role)
         DO UPDATE SET
            page_id = excluded.page_id,
            block_id = excluded.block_id,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(asset_path)
    .bind(block_id)
    .bind(page_id)
    .bind(block_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("record notes block asset reference: {e}"))?;
    Ok(())
}

async fn replace_page_asset_reference_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
    role: &str,
    value: Option<&Value>,
    field: &str,
    expected_prefix: &str,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM notes_asset_references
         WHERE owner_type = 'page' AND owner_id = ? AND role = ?",
    )
    .bind(page_id)
    .bind(role)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("clear notes page asset reference: {e}"))?;

    let Some(asset) = page_local_file_asset(value, field, expected_prefix)? else {
        return Ok(());
    };
    let asset_path = asset.relative_path.trim().to_string();
    upsert_managed_asset_tx(tx, asset).await?;
    sqlx::query(
        "INSERT INTO notes_asset_references (
            asset_id,
            owner_type,
            owner_id,
            page_id,
            role
         )
         VALUES (?, 'page', ?, ?, ?)
         ON CONFLICT(asset_id, owner_type, owner_id, role)
         DO UPDATE SET
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(asset_path)
    .bind(page_id)
    .bind(page_id)
    .bind(role)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("record notes page asset reference: {e}"))?;
    Ok(())
}

fn page_local_file_asset<'a>(
    value: Option<&'a Value>,
    field: &str,
    expected_prefix: &str,
) -> Result<Option<NotesManagedAssetWrite<'a>>, String> {
    let Some(Value::Object(object)) = value else {
        return Ok(None);
    };
    if object.get("type").and_then(Value::as_str) != Some("file") {
        return Ok(None);
    }
    let Some(file) = object.get("file").and_then(Value::as_object) else {
        return Ok(None);
    };
    let Some(relative_path) = file.get("ganbaru_asset_path").and_then(Value::as_str) else {
        return Ok(None);
    };
    if !relative_path.trim().starts_with(expected_prefix) {
        return Err(format!(
            "{field}.file.ganbaru_asset_path must stay under the expected managed asset directory"
        ));
    }
    let content_type = file
        .get("content_type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{field}.file.content_type must be a string"))?;
    let byte_size = file
        .get("byte_size")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{field}.file.byte_size must be an integer"))?;
    let sha256 = file
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{field}.file.sha256 must be a string"))?;
    Ok(Some(NotesManagedAssetWrite {
        relative_path,
        original_name: file.get("name").and_then(Value::as_str),
        content_type,
        byte_size,
        sha256,
        source_type: NOTES_ASSET_SOURCE_LOCAL_UPLOAD,
        storage_state: NOTES_ASSET_STATE_AVAILABLE,
        missing_at: None,
    }))
}

fn media_local_file_asset<'a>(
    payload: &'a Value,
    block_type: &str,
) -> Result<Option<NotesManagedAssetWrite<'a>>, String> {
    let Value::Object(object) = payload else {
        return Ok(None);
    };
    if object.get("type").and_then(Value::as_str) != Some("file") {
        return Ok(None);
    }
    let Some(file) = object.get("file").and_then(Value::as_object) else {
        return Ok(None);
    };
    let Some(relative_path) = file.get("ganbaru_asset_path").and_then(Value::as_str) else {
        return Ok(None);
    };
    if !relative_path.trim().starts_with(NOTES_ASSET_FILE_PREFIX) {
        return Err(format!(
            "{block_type}.file.ganbaru_asset_path must stay under the managed Notes file directory"
        ));
    }
    let content_type = file
        .get("content_type")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{block_type}.file.content_type must be a string"))?;
    let byte_size = file
        .get("byte_size")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{block_type}.file.byte_size must be an integer"))?;
    let sha256 = file
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{block_type}.file.sha256 must be a string"))?;
    Ok(Some(NotesManagedAssetWrite {
        relative_path,
        original_name: file.get("name").and_then(Value::as_str),
        content_type,
        byte_size,
        sha256,
        source_type: NOTES_ASSET_SOURCE_LOCAL_UPLOAD,
        storage_state: NOTES_ASSET_STATE_AVAILABLE,
        missing_at: None,
    }))
}

fn validate_managed_asset(asset: &NotesManagedAssetWrite<'_>) -> Result<&'static str, String> {
    let relative_path = asset.relative_path.trim();
    validate_managed_asset_path(relative_path)?;
    let kind = asset_kind_for_content_type(asset.content_type.trim())?;
    let is_page_media_asset = relative_path.starts_with(NOTES_ASSET_PAGE_ICON_PREFIX)
        || relative_path.starts_with(NOTES_ASSET_PAGE_COVER_PREFIX);
    if is_page_media_asset && kind != "image" {
        return Err("page icon and cover assets must be images".to_string());
    }
    validate_asset_size(asset.byte_size)?;
    validate_sha256(asset.sha256.trim())?;
    validate_allowed_value(
        asset.source_type.trim(),
        NOTES_ASSET_SOURCE_TYPES,
        "notes asset source_type",
    )?;
    validate_allowed_value(
        asset.storage_state.trim(),
        NOTES_ASSET_STORAGE_STATES,
        "notes asset storage_state",
    )?;
    match (asset.storage_state.trim(), asset.missing_at.map(str::trim)) {
        ("available", None) => {}
        ("missing", Some(missing_at)) if !missing_at.is_empty() => {
            validate_no_control_characters(missing_at, "notes asset missing_at")?;
        }
        ("available", Some(_)) => {
            return Err("available notes assets must not have missing_at".to_string())
        }
        ("missing", _) => return Err("missing notes assets must have missing_at".to_string()),
        _ => {}
    }
    if let Some(original_name) = asset.original_name.map(str::trim) {
        validate_no_control_characters(original_name, "notes asset original_name")?;
    }
    Ok(kind)
}

fn validate_managed_asset_path(relative_path: &str) -> Result<(), String> {
    let prefix = [
        NOTES_ASSET_PAGE_ICON_PREFIX,
        NOTES_ASSET_PAGE_COVER_PREFIX,
        NOTES_ASSET_FILE_PREFIX,
    ]
    .iter()
    .find(|prefix| relative_path.starts_with(**prefix))
    .ok_or_else(|| {
        "notes asset path must stay under a managed Notes asset directory".to_string()
    })?;
    let file_name = relative_path.strip_prefix(*prefix).ok_or_else(|| {
        "notes asset path must stay under a managed Notes asset directory".to_string()
    })?;
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err("notes asset path cannot contain nested or parent paths".to_string());
    }
    let path = Path::new(file_name);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("notes asset path cannot contain nested or parent paths".to_string());
    }
    if (*prefix == NOTES_ASSET_PAGE_ICON_PREFIX || *prefix == NOTES_ASSET_PAGE_COVER_PREFIX)
        && !has_allowed_image_extension(path)
    {
        return Err("page icon and cover assets must be PNG, JPG, or WebP".to_string());
    }
    Ok(())
}

fn asset_kind_for_content_type(content_type: &str) -> Result<&'static str, String> {
    validate_mime_type(content_type)?;
    if NOTES_ASSET_ALLOWED_IMAGE_CONTENT_TYPES.contains(&content_type) {
        return Ok("image");
    }
    if content_type == "image/svg+xml" {
        return Err("SVG assets are blocked because they can contain active content".to_string());
    }
    if content_type.starts_with("video/") {
        return Ok("video");
    }
    if content_type.starts_with("audio/") {
        return Ok("audio");
    }
    if content_type == "application/pdf" {
        return Ok("pdf");
    }
    Ok("file")
}

fn validate_mime_type(content_type: &str) -> Result<(), String> {
    if content_type.is_empty()
        || content_type.contains(' ')
        || content_type.contains(';')
        || content_type.matches('/').count() != 1
        || content_type.chars().any(char::is_control)
    {
        return Err("notes asset content_type must be a plain MIME type".to_string());
    }
    let Some((kind, subtype)) = content_type.split_once('/') else {
        return Err("notes asset content_type must be a plain MIME type".to_string());
    };
    if kind.is_empty() || subtype.is_empty() {
        return Err("notes asset content_type must be a plain MIME type".to_string());
    }
    Ok(())
}

fn validate_asset_size(byte_size: i64) -> Result<(), String> {
    if byte_size <= 0 {
        return Err("notes asset byte_size must be positive".to_string());
    }
    Ok(())
}

fn validate_sha256(sha256: &str) -> Result<(), String> {
    if sha256.len() != 64 || !sha256.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err("notes asset sha256 must be a SHA-256 hex digest".to_string());
    }
    if sha256.chars().any(|value| value.is_ascii_uppercase()) {
        return Err("notes asset sha256 must be lowercase".to_string());
    }
    Ok(())
}

fn validate_allowed_value(value: &str, allowed: &[&str], field: &str) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{field} is unsupported"))
    }
}

fn validate_no_control_characters(value: &str, field: &str) -> Result<(), String> {
    if value.chars().any(char::is_control) {
        Err(format!("{field} must not contain control characters"))
    } else {
        Ok(())
    }
}

fn has_allowed_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            NOTES_ASSET_ALLOWED_IMAGE_EXTENSIONS
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_asset(relative_path: &str) -> NotesManagedAssetWrite<'_> {
        NotesManagedAssetWrite {
            relative_path,
            original_name: Some("focus.png"),
            content_type: "image/png",
            byte_size: 42,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            source_type: NOTES_ASSET_SOURCE_LOCAL_UPLOAD,
            storage_state: NOTES_ASSET_STATE_AVAILABLE,
            missing_at: None,
        }
    }

    #[test]
    fn validate_managed_asset_accepts_supported_paths_and_types() {
        assert_eq!(
            validate_managed_asset(&valid_asset(
                "notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
            ))
            .unwrap(),
            "image",
        );
        let asset = NotesManagedAssetWrite {
            relative_path: "notes/files/report.pdf",
            original_name: Some("report.pdf"),
            content_type: "application/pdf",
            byte_size: 12,
            sha256: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            source_type: "imported",
            storage_state: "missing",
            missing_at: Some("2026-07-02T12:00:00.000Z"),
        };
        assert_eq!(validate_managed_asset(&asset).unwrap(), "pdf");
    }

    #[test]
    fn validate_managed_asset_rejects_unsafe_asset_shapes() {
        for (asset, expected) in [
            (
                NotesManagedAssetWrite {
                    relative_path: "notes/files/../secret.pdf",
                    ..valid_asset("notes/files/../secret.pdf")
                },
                "notes asset path cannot contain nested or parent paths",
            ),
            (
                NotesManagedAssetWrite {
                    relative_path: "notes/page-icons/icon.svg",
                    ..valid_asset("notes/page-icons/icon.svg")
                },
                "page icon and cover assets must be PNG, JPG, or WebP",
            ),
            (
                NotesManagedAssetWrite {
                    content_type: "image/svg+xml",
                    ..valid_asset("notes/files/icon.svg")
                },
                "SVG assets are blocked because they can contain active content",
            ),
            (
                NotesManagedAssetWrite {
                    byte_size: 0,
                    ..valid_asset("notes/files/empty.txt")
                },
                "notes asset byte_size must be positive",
            ),
            (
                NotesManagedAssetWrite {
                    sha256: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                    ..valid_asset("notes/files/file.bin")
                },
                "notes asset sha256 must be lowercase",
            ),
            (
                NotesManagedAssetWrite {
                    storage_state: "missing",
                    missing_at: None,
                    ..valid_asset("notes/files/file.bin")
                },
                "missing notes assets must have missing_at",
            ),
        ] {
            assert_eq!(
                validate_managed_asset(&asset).err().as_deref(),
                Some(expected)
            );
        }
    }
}
