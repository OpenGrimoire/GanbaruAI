use super::models::{NoteBlockUpdate, NoteBlockWrite, NotePageCreate, NotePageUpdate, NoteParent};
use serde_json::Value;

pub(in crate::notes) const NOTE_BLOCK_TYPES: &[&str] = &[
    "paragraph",
    "heading_1",
    "heading_2",
    "heading_3",
    "heading_4",
    "bulleted_list_item",
    "numbered_list_item",
    "to_do",
    "toggle",
    "callout",
    "quote",
    "child_page",
    "child_database",
    "breadcrumb",
    "table_of_contents",
    "column_list",
    "column",
    "table",
    "table_row",
    "tab",
    "image",
    "video",
    "audio",
    "file",
    "pdf",
    "bookmark",
    "link_preview",
    "synced_block",
    "template",
    "button",
    "embed",
    "equation",
    "divider",
    "code",
    "unsupported",
];

const TEXT_BLOCK_TYPES: &[&str] = &[
    "paragraph",
    "heading_1",
    "heading_2",
    "heading_3",
    "heading_4",
    "bulleted_list_item",
    "numbered_list_item",
    "toggle",
    "callout",
    "quote",
];

const NOTE_COLORS: &[&str] = &[
    "default",
    "gray",
    "brown",
    "orange",
    "yellow",
    "green",
    "blue",
    "purple",
    "pink",
    "red",
    "gray_background",
    "brown_background",
    "orange_background",
    "yellow_background",
    "green_background",
    "blue_background",
    "purple_background",
    "pink_background",
    "red_background",
];

const NOTE_ICON_COLORS: &[&str] = &[
    "gray",
    "lightgray",
    "brown",
    "yellow",
    "orange",
    "green",
    "blue",
    "purple",
    "pink",
    "red",
];

const IMAGE_EXTENSIONS: &[&str] = &[
    ".bmp", ".gif", ".heic", ".jpeg", ".jpg", ".png", ".svg", ".tif", ".tiff",
];
const AUDIO_EXTENSIONS: &[&str] = &[".mp3", ".wav", ".ogg", ".oga", ".m4a"];
const VIDEO_EXTENSIONS: &[&str] = &[
    ".amv", ".asf", ".avi", ".f4v", ".flv", ".gifv", ".mkv", ".mov", ".mpg", ".mpeg", ".mpv",
    ".mp4", ".m4v", ".qt", ".wmv",
];

pub(in crate::notes) fn require_uuid(value: &str, field: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return Err(format!("{field} must be a UUID"));
    }
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if *byte != b'-' {
                return Err(format!("{field} must be a UUID"));
            }
            continue;
        }
        if !byte.is_ascii_hexdigit() {
            return Err(format!("{field} must be a UUID"));
        }
    }
    Ok(())
}

pub(in crate::notes) fn validate_page_create(page: &NotePageCreate) -> Result<(), String> {
    require_uuid(&page.id, "id")?;
    require_uuid(&page.first_block_id, "first_block_id")?;
    if page.id.trim() == page.first_block_id.trim() {
        return Err("first_block_id must not match id".to_string());
    }
    if let Some(after_block_id) = &page.after_block_id {
        require_uuid(after_block_id, "after_block_id")?;
    }
    validate_parent(&page.parent)
}

pub(in crate::notes) fn validate_page_update(update: &NotePageUpdate) -> Result<(), String> {
    if let Some(parent) = &update.parent {
        validate_parent(parent)?;
    }
    if let Some(properties) = &update.properties {
        validate_json_object(properties, "properties")?;
    }
    if let Some(icon) = update.icon.value() {
        validate_icon_value(icon, "icon")?;
    }
    if let Some(cover) = update.cover.value() {
        validate_page_cover_value(cover)?;
    }
    Ok(())
}

pub(in crate::notes) fn validate_parent(parent: &NoteParent) -> Result<(), String> {
    match parent {
        NoteParent::Workspace { workspace } => {
            if *workspace {
                Ok(())
            } else {
                Err("workspace parent must set workspace to true".to_string())
            }
        }
        NoteParent::PageId { page_id } => require_uuid(page_id, "page_id"),
        NoteParent::BlockId { block_id } => require_uuid(block_id, "block_id"),
    }
}

pub(in crate::notes) fn validate_block_write(block: &NoteBlockWrite) -> Result<(), String> {
    require_uuid(&block.id, "id")?;
    validate_block_type(&block.block_type)?;
    let payload = block
        .payload()
        .ok_or_else(|| format!("{} payload is required", block.block_type))?;
    validate_block_payload(&block.block_type, payload)
}

pub(in crate::notes) fn validate_block_update(
    current_type: &str,
    update: &NoteBlockUpdate,
) -> Result<(String, Value), String> {
    let block_type = update
        .block_type
        .as_deref()
        .unwrap_or(current_type)
        .trim()
        .to_string();
    validate_block_type(&block_type)?;
    let payload = update
        .payload_for(&block_type)
        .ok_or_else(|| format!("{block_type} payload is required"))?
        .clone();
    validate_block_payload(&block_type, &payload)?;
    Ok((block_type, payload))
}

pub(in crate::notes) fn validate_block_type(block_type: &str) -> Result<(), String> {
    if NOTE_BLOCK_TYPES.contains(&block_type) {
        Ok(())
    } else {
        Err(format!("unsupported block type: {block_type}"))
    }
}

pub(in crate::notes) fn block_type_supports_children(block_type: &str) -> bool {
    matches!(
        block_type,
        "paragraph"
            | "bulleted_list_item"
            | "numbered_list_item"
            | "to_do"
            | "toggle"
            | "callout"
            | "quote"
            | "child_database"
            | "column_list"
            | "column"
            | "table"
            | "tab"
            | "template"
            | "button"
    )
}

pub(in crate::notes) fn block_payload_supports_children(block_type: &str, payload: &Value) -> bool {
    if matches!(
        block_type,
        "heading_1" | "heading_2" | "heading_3" | "heading_4"
    ) {
        return payload
            .get("is_toggleable")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    }
    if block_type == "synced_block" {
        return matches!(payload.get("synced_from"), Some(Value::Null));
    }
    block_type_supports_children(block_type)
}

pub(in crate::notes) fn validate_sort_order(value: f64) -> Result<(), String> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err("sort_order must be non-negative".to_string())
    }
}

pub(in crate::notes) fn validate_page_size(page_size: i64) -> Result<(), String> {
    if (1..=100).contains(&page_size) {
        Ok(())
    } else {
        Err("page_size must be between 1 and 100".to_string())
    }
}

pub(in crate::notes) fn validate_children_count(count: usize) -> Result<(), String> {
    if (1..=100).contains(&count) {
        Ok(())
    } else {
        Err("children must include between 1 and 100 blocks".to_string())
    }
}

pub(in crate::notes) fn validate_duplicate_block_count(count: usize) -> Result<(), String> {
    if (1..=100).contains(&count) {
        Ok(())
    } else {
        Err("duplicated_block_ids must include between 1 and 100 blocks".to_string())
    }
}

pub(in crate::notes) fn validate_comment_rich_text(rich_text: &[Value]) -> Result<(), String> {
    if !(1..=100).contains(&rich_text.len()) {
        return Err("comment rich_text must include between 1 and 100 items".to_string());
    }
    for item in rich_text {
        validate_rich_text_item(item)?;
    }
    if rich_text_items_plain_text(rich_text).trim().is_empty() {
        return Err("comment rich_text must not be empty".to_string());
    }
    Ok(())
}

pub(in crate::notes) fn validate_block_payload(
    block_type: &str,
    payload: &Value,
) -> Result<(), String> {
    validate_json_object(payload, block_type)?;
    if TEXT_BLOCK_TYPES.contains(&block_type) {
        validate_rich_text_payload(payload, block_type)?;
        return Ok(());
    }
    match block_type {
        "to_do" => {
            validate_rich_text_payload(payload, block_type)?;
            match payload.get("checked") {
                Some(Value::Bool(_)) => Ok(()),
                _ => Err("to_do.checked must be a boolean".to_string()),
            }
        }
        "code" => {
            validate_rich_text_payload(payload, block_type)?;
            match payload.get("caption") {
                Some(Value::Array(caption)) => {
                    for item in caption {
                        validate_rich_text_item(item)?;
                    }
                }
                _ => return Err("code.caption must be a rich text array".to_string()),
            }
            match payload.get("language") {
                Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
                _ => Err("code.language is required".to_string()),
            }
        }
        "table_of_contents" => validate_table_of_contents_payload(payload),
        "column_list" => Ok(()),
        "column" => validate_column_payload(payload),
        "table" => validate_table_payload(payload),
        "table_row" => validate_table_row_payload(payload),
        "tab" => validate_empty_object_payload(payload, "tab"),
        "image" | "video" | "audio" | "file" | "pdf" => validate_media_payload(block_type, payload),
        "child_page" => validate_child_page_payload(payload),
        "child_database" => validate_child_database_payload(payload),
        "bookmark" => validate_bookmark_payload(payload),
        "link_preview" => validate_link_preview_payload(payload),
        "synced_block" => validate_synced_block_payload(payload),
        "template" => validate_template_payload(payload),
        "button" => validate_button_payload(payload),
        "embed" => validate_embed_payload(payload),
        "equation" => validate_equation_payload(payload),
        "breadcrumb" | "divider" => Ok(()),
        "unsupported" => validate_unsupported_payload(payload),
        _ => Err(format!("unsupported block type: {block_type}")),
    }
}

fn validate_json_object(value: &Value, field: &str) -> Result<(), String> {
    if value.is_object() {
        Ok(())
    } else {
        Err(format!("{field} must be an object"))
    }
}

fn validate_empty_object_payload(value: &Value, field: &str) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{field} must be an object"))?;
    if object.is_empty() {
        Ok(())
    } else {
        Err(format!("{field} must be an empty object"))
    }
}

fn validate_page_cover_value(value: &Value) -> Result<(), String> {
    validate_json_object(value, "cover")?;
    let Some(Value::String(source_type)) = value.get("type") else {
        return Err("cover.type must be a string".to_string());
    };
    match source_type.as_str() {
        "external" => {
            let external = value
                .get("external")
                .and_then(Value::as_object)
                .ok_or_else(|| "cover.external must be an object".to_string())?;
            let Some(Value::String(url)) = external.get("url") else {
                return Err("cover.external.url must be a string".to_string());
            };
            validate_media_url("image", url, false, "cover.external.url")
        }
        "file" => {
            let file = value
                .get("file")
                .and_then(Value::as_object)
                .ok_or_else(|| "cover.file must be an object".to_string())?;
            let Some(Value::String(url)) = file.get("url") else {
                return Err("cover.file.url must be a string".to_string());
            };
            validate_media_url("image", url, false, "cover.file.url")?;
            match file.get("expiry_time") {
                Some(Value::String(expiry_time)) if !contains_control_characters(expiry_time) => {
                    Ok(())
                }
                Some(Value::String(_)) => {
                    Err("cover.file.expiry_time must not contain control characters".to_string())
                }
                _ => Err("cover.file.expiry_time must be a string".to_string()),
            }
        }
        "file_upload" => {
            let file_upload = value
                .get("file_upload")
                .and_then(Value::as_object)
                .ok_or_else(|| "cover.file_upload must be an object".to_string())?;
            let Some(Value::String(id)) = file_upload.get("id") else {
                return Err("cover.file_upload.id must be a string".to_string());
            };
            require_uuid(id, "cover.file_upload.id")
        }
        _ => Err("cover.type must be file, external, or file_upload".to_string()),
    }
}

fn validate_table_of_contents_payload(payload: &Value) -> Result<(), String> {
    validate_optional_color(payload, "table_of_contents.color")
}

fn validate_child_page_payload(payload: &Value) -> Result<(), String> {
    match payload.get("title") {
        Some(Value::String(title)) if !contains_control_characters(title) => Ok(()),
        Some(Value::String(_)) => {
            Err("child_page.title must not contain control characters".to_string())
        }
        _ => Err("child_page.title must be a string".to_string()),
    }
}

fn validate_child_database_payload(payload: &Value) -> Result<(), String> {
    match payload.get("title") {
        Some(Value::String(title)) if !contains_control_characters(title) => Ok(()),
        Some(Value::String(_)) => {
            Err("child_database.title must not contain control characters".to_string())
        }
        _ => Err("child_database.title must be a string".to_string()),
    }
}

fn validate_column_payload(payload: &Value) -> Result<(), String> {
    match payload.get("width_ratio") {
        None => Ok(()),
        Some(Value::Number(width)) => match width.as_f64() {
            Some(value) if value > 0.0 && value <= 1.0 => Ok(()),
            _ => Err("column.width_ratio must be greater than 0 and no more than 1".to_string()),
        },
        _ => Err("column.width_ratio must be a number".to_string()),
    }
}

fn validate_table_payload(payload: &Value) -> Result<(), String> {
    match payload.get("table_width") {
        Some(Value::Number(width)) => match width.as_i64() {
            Some(value) if (1..=100).contains(&value) => {}
            _ => return Err("table.table_width must be between 1 and 100".to_string()),
        },
        _ => return Err("table.table_width must be an integer".to_string()),
    }
    match payload.get("has_column_header") {
        Some(Value::Bool(_)) => {}
        _ => return Err("table.has_column_header must be a boolean".to_string()),
    }
    match payload.get("has_row_header") {
        Some(Value::Bool(_)) => {}
        _ => return Err("table.has_row_header must be a boolean".to_string()),
    }
    Ok(())
}

fn validate_table_row_payload(payload: &Value) -> Result<(), String> {
    let cells = match payload.get("cells") {
        Some(Value::Array(cells)) if (1..=100).contains(&cells.len()) => cells,
        Some(Value::Array(_)) => {
            return Err("table_row.cells must include between 1 and 100 cells".to_string())
        }
        _ => return Err("table_row.cells must be an array".to_string()),
    };
    for cell in cells {
        let Value::Array(items) = cell else {
            return Err("table_row.cells entries must be rich text arrays".to_string());
        };
        for item in items {
            validate_rich_text_item(item)?;
        }
    }
    Ok(())
}

fn validate_media_payload(block_type: &str, payload: &Value) -> Result<(), String> {
    match payload.get("caption") {
        Some(Value::Array(caption)) => {
            for item in caption {
                validate_rich_text_item(item)?;
            }
        }
        None if block_type != "file" => {}
        _ => return Err(format!("{block_type}.caption must be a rich text array")),
    }
    match payload.get("name") {
        None => {}
        Some(Value::String(name)) if !contains_control_characters(name) => {}
        Some(Value::String(_)) => {
            return Err(format!(
                "{block_type}.name must not contain control characters"
            ))
        }
        _ => return Err(format!("{block_type}.name must be a string")),
    }
    let Some(Value::String(source_type)) = payload.get("type") else {
        return Err(format!("{block_type}.type must be a string"));
    };
    match source_type.as_str() {
        "external" => {
            let external = payload
                .get("external")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("{block_type}.external must be an object"))?;
            let Some(Value::String(url)) = external.get("url") else {
                return Err(format!("{block_type}.external.url must be a string"));
            };
            validate_media_url(block_type, url, true, &format!("{block_type}.external.url"))
        }
        "file" => {
            let file = payload
                .get("file")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("{block_type}.file must be an object"))?;
            let Some(Value::String(url)) = file.get("url") else {
                return Err(format!("{block_type}.file.url must be a string"));
            };
            validate_media_url(block_type, url, false, &format!("{block_type}.file.url"))?;
            match file.get("expiry_time") {
                Some(Value::String(expiry_time)) if !contains_control_characters(expiry_time) => {
                    Ok(())
                }
                Some(Value::String(_)) => Err(format!(
                    "{block_type}.file.expiry_time must not contain control characters"
                )),
                _ => Err(format!("{block_type}.file.expiry_time must be a string")),
            }
        }
        "file_upload" => {
            let file_upload = payload
                .get("file_upload")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("{block_type}.file_upload must be an object"))?;
            let Some(Value::String(id)) = file_upload.get("id") else {
                return Err(format!("{block_type}.file_upload.id must be a string"));
            };
            require_uuid(id, &format!("{block_type}.file_upload.id"))
        }
        _ => Err(format!(
            "{block_type}.type must be file, external, or file_upload"
        )),
    }
}

fn validate_media_url(
    block_type: &str,
    url: &str,
    allow_empty: bool,
    field: &str,
) -> Result<(), String> {
    if url.trim().is_empty() {
        return if allow_empty {
            Ok(())
        } else {
            Err(format!("{field} must not be empty"))
        };
    }
    if contains_control_characters(url) {
        return Err(format!("{field} must not contain control characters"));
    }
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| format!("{field} must be a supported HTTPS {block_type} URL"))?;
    if parsed.scheme() != "https" || !media_url_matches_block_type(block_type, &parsed) {
        return Err(format!(
            "{field} must be a supported HTTPS {block_type} URL"
        ));
    }
    Ok(())
}

fn media_url_matches_block_type(block_type: &str, url: &reqwest::Url) -> bool {
    match block_type {
        "file" => true,
        "pdf" => path_has_extension(url, &[".pdf"]),
        "image" => path_has_extension(url, IMAGE_EXTENSIONS),
        "audio" => path_has_extension(url, AUDIO_EXTENSIONS),
        "video" => path_has_extension(url, VIDEO_EXTENSIONS) || is_youtube_video_url(url),
        _ => false,
    }
}

fn path_has_extension(url: &reqwest::Url, extensions: &[&str]) -> bool {
    let path = url.path().to_ascii_lowercase();
    extensions.iter().any(|extension| path.ends_with(extension))
}

fn is_youtube_video_url(url: &reqwest::Url) -> bool {
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host != "www.youtube.com" && host != "youtube.com" {
        return false;
    }
    if url.path() == "/watch" {
        return url
            .query_pairs()
            .any(|(key, value)| key == "v" && !value.is_empty());
    }
    url.path().starts_with("/embed/")
}

fn validate_bookmark_payload(payload: &Value) -> Result<(), String> {
    match payload.get("caption") {
        Some(Value::Array(caption)) => {
            for item in caption {
                validate_rich_text_item(item)?;
            }
        }
        _ => return Err("bookmark.caption must be a rich text array".to_string()),
    }
    match payload.get("url") {
        Some(Value::String(url)) if is_bookmark_url_string(url) => Ok(()),
        Some(Value::String(_)) => {
            Err("bookmark.url must not contain control characters".to_string())
        }
        _ => Err("bookmark.url must be a string".to_string()),
    }
}

fn is_bookmark_url_string(url: &str) -> bool {
    !url.chars().any(char::is_control)
}

fn validate_embed_payload(payload: &Value) -> Result<(), String> {
    match payload.get("url") {
        Some(Value::String(url)) if !url.chars().any(char::is_control) => Ok(()),
        Some(Value::String(_)) => Err("embed.url must not contain control characters".to_string()),
        _ => Err("embed.url must be a string".to_string()),
    }
}

fn validate_link_preview_payload(payload: &Value) -> Result<(), String> {
    match payload.get("url") {
        Some(Value::String(url)) if !url.chars().any(char::is_control) => Ok(()),
        Some(Value::String(_)) => {
            Err("link_preview.url must not contain control characters".to_string())
        }
        _ => Err("link_preview.url must be a string".to_string()),
    }
}

fn validate_synced_block_payload(payload: &Value) -> Result<(), String> {
    match payload.get("synced_from") {
        Some(Value::Null) => Ok(()),
        Some(Value::Object(synced_from)) => {
            match synced_from.get("type") {
                Some(Value::String(source_type)) if source_type == "block_id" => {}
                Some(Value::String(_)) => {
                    return Err("synced_block.synced_from.type must be block_id".to_string())
                }
                _ => return Err("synced_block.synced_from.type must be a string".to_string()),
            }
            match synced_from.get("block_id") {
                Some(Value::String(block_id)) => {
                    require_uuid(block_id, "synced_block.synced_from.block_id")
                }
                _ => Err("synced_block.synced_from.block_id must be a string".to_string()),
            }
        }
        Some(_) => Err("synced_block.synced_from must be null or an object".to_string()),
        None => Err("synced_block.synced_from is required".to_string()),
    }
}

fn validate_equation_payload(payload: &Value) -> Result<(), String> {
    match payload.get("expression") {
        Some(Value::String(expression)) if !contains_control_characters(expression) => Ok(()),
        Some(Value::String(_)) => {
            Err("equation.expression must not contain control characters".to_string())
        }
        _ => Err("equation.expression must be a string".to_string()),
    }
}

fn validate_unsupported_payload(payload: &Value) -> Result<(), String> {
    validate_optional_display_string(payload.get("block_type"), "unsupported.block_type")?;
    validate_optional_display_string(payload.get("source_type"), "unsupported.source_type")?;
    match payload.get("raw") {
        None | Some(Value::Object(_)) => {}
        _ => return Err("unsupported.raw must be an object".to_string()),
    }
    match payload.get("warnings") {
        None => {}
        Some(Value::Array(warnings)) => {
            for (index, warning) in warnings.iter().enumerate() {
                validate_optional_display_string(
                    Some(warning),
                    &format!("unsupported.warnings[{index}]"),
                )?;
            }
        }
        _ => return Err("unsupported.warnings must be an array".to_string()),
    }
    Ok(())
}

fn validate_optional_display_string(value: Option<&Value>, field: &str) -> Result<(), String> {
    match value {
        None => Ok(()),
        Some(Value::String(text)) if text.trim().is_empty() => {
            Err(format!("{field} must not be empty"))
        }
        Some(Value::String(text)) if contains_control_characters(text) => {
            Err(format!("{field} must not contain control characters"))
        }
        Some(Value::String(_)) => Ok(()),
        _ => Err(format!("{field} must be a string")),
    }
}

fn contains_control_characters(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_control() && character != '\n' && character != '\t')
}

fn validate_rich_text_payload(payload: &Value, field: &str) -> Result<(), String> {
    validate_optional_color(payload, &format!("{field}.color"))?;
    validate_optional_toggle_open(payload, field)?;
    validate_optional_heading_toggle_fields(payload, field)?;
    validate_callout_icon(payload, field)?;
    validate_optional_paragraph_icon(payload, field)?;
    match payload.get("rich_text") {
        Some(Value::Array(items)) => {
            for item in items {
                validate_rich_text_item(item)?;
            }
            Ok(())
        }
        _ => Err(format!("{field}.rich_text must be an array")),
    }
}

fn validate_optional_paragraph_icon(payload: &Value, field: &str) -> Result<(), String> {
    if field != "paragraph" {
        return Ok(());
    }
    match payload.get("icon") {
        None | Some(Value::Null) => Ok(()),
        Some(icon) => validate_icon_value(icon, "paragraph.icon"),
    }
}

fn validate_template_payload(payload: &Value) -> Result<(), String> {
    if payload.get("color").is_some() {
        return Err("template.color is not supported".to_string());
    }
    if payload.get("children").is_some() {
        return Err("template.children must be stored as child blocks".to_string());
    }
    match payload.get("rich_text") {
        Some(Value::Array(items)) => {
            for item in items {
                validate_rich_text_item(item)?;
            }
            Ok(())
        }
        _ => Err("template.rich_text must be an array".to_string()),
    }
}

fn validate_button_payload(payload: &Value) -> Result<(), String> {
    if payload.get("children").is_some() {
        return Err("button.children must be stored as child blocks".to_string());
    }
    match payload.get("rich_text") {
        Some(Value::Array(items)) => {
            for item in items {
                validate_rich_text_item(item)?;
            }
        }
        _ => return Err("button.rich_text must be an array".to_string()),
    }
    match payload.get("icon") {
        Some(Value::Null) => {}
        Some(icon) => validate_icon_value(icon, "button.icon")?,
        None => return Err("button.icon is required".to_string()),
    }
    match payload.get("actions") {
        Some(Value::Array(actions)) if (1..=10).contains(&actions.len()) => {
            for (index, action) in actions.iter().enumerate() {
                validate_button_action(action, index)?;
            }
            Ok(())
        }
        Some(Value::Array(_)) => {
            Err("button.actions must include between 1 and 10 actions".to_string())
        }
        _ => Err("button.actions must be an array".to_string()),
    }
}

fn validate_button_action(action: &Value, index: usize) -> Result<(), String> {
    validate_json_object(action, &format!("button.actions[{index}]"))?;
    match action.get("type") {
        Some(Value::String(value)) if value == "insert_blocks" => {}
        _ => {
            return Err(format!(
                "button.actions[{index}].type must be insert_blocks"
            ))
        }
    }
    match action.get("source") {
        Some(Value::String(value)) if value == "children" => {}
        _ => return Err(format!("button.actions[{index}].source must be children")),
    }
    match action.get("position") {
        Some(Value::String(value))
            if matches!(
                value.as_str(),
                "below_button" | "above_button" | "top_of_page" | "bottom_of_page"
            ) =>
        {
            Ok(())
        }
        _ => Err(format!(
            "button.actions[{index}].position must be a supported button insert position"
        )),
    }
}

fn validate_callout_icon(payload: &Value, field: &str) -> Result<(), String> {
    if field != "callout" {
        return Ok(());
    }
    match payload.get("icon") {
        Some(Value::Null) => Ok(()),
        Some(Value::Object(icon)) => validate_icon_object(icon, "callout.icon"),
        _ => Err("callout.icon must be an icon object or null".to_string()),
    }
}

fn validate_icon_value(value: &Value, field: &str) -> Result<(), String> {
    match value {
        Value::Object(icon) => validate_icon_object(icon, field),
        _ => Err(format!("{field} must be an icon object")),
    }
}

fn validate_icon_object(icon: &serde_json::Map<String, Value>, field: &str) -> Result<(), String> {
    let Some(Value::String(icon_type)) = icon.get("type") else {
        return Err(format!("{field}.type must be a string"));
    };
    match icon_type.as_str() {
        "emoji" => match icon.get("emoji") {
            Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
            _ => Err(format!("{field}.emoji must be a non-empty string")),
        },
        "custom_emoji" => match icon.get("custom_emoji") {
            Some(Value::Object(_)) => Ok(()),
            _ => Err(format!("{field}.custom_emoji must be an object")),
        },
        "icon" => validate_native_icon(icon, field),
        "external" => match icon.get("external").and_then(Value::as_object) {
            Some(external) => match external.get("url") {
                Some(Value::String(url)) if !url.trim().is_empty() => Ok(()),
                _ => Err(format!("{field}.external.url must be a non-empty string")),
            },
            None => Err(format!("{field}.external must be an object")),
        },
        "file" => match icon.get("file") {
            Some(Value::Object(_)) => Ok(()),
            _ => Err(format!("{field}.file must be an object")),
        },
        _ => Err(format!("{field}.type must be a supported Notion icon type")),
    }
}

fn validate_native_icon(icon: &serde_json::Map<String, Value>, field: &str) -> Result<(), String> {
    let native = icon
        .get("icon")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{field}.icon must be an object"))?;
    match native.get("name") {
        Some(Value::String(name)) if !name.trim().is_empty() => {}
        _ => return Err(format!("{field}.icon.name must be a non-empty string")),
    }
    match native.get("color") {
        None => Ok(()),
        Some(Value::String(color)) if NOTE_ICON_COLORS.contains(&color.as_str()) => Ok(()),
        Some(Value::String(_)) => Err(format!(
            "{field}.icon.color must be a supported Notion icon color"
        )),
        _ => Err(format!("{field}.icon.color must be a string")),
    }
}

fn validate_optional_toggle_open(payload: &Value, field: &str) -> Result<(), String> {
    if field != "toggle" {
        return Ok(());
    }
    match payload.get("ganbaru_open") {
        None | Some(Value::Bool(_)) => Ok(()),
        _ => Err("toggle.ganbaru_open must be a boolean".to_string()),
    }
}

fn validate_optional_heading_toggle_fields(payload: &Value, field: &str) -> Result<(), String> {
    if !matches!(field, "heading_1" | "heading_2" | "heading_3" | "heading_4") {
        return Ok(());
    }
    match payload.get("is_toggleable") {
        None | Some(Value::Bool(_)) => {}
        _ => return Err(format!("{field}.is_toggleable must be a boolean")),
    }
    match payload.get("ganbaru_open") {
        None | Some(Value::Bool(_)) => Ok(()),
        _ => Err(format!("{field}.ganbaru_open must be a boolean")),
    }
}

fn validate_rich_text_item(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "rich text item must be an object".to_string())?;
    match object.get("type") {
        Some(Value::String(kind)) if kind == "text" => validate_text_rich_text_item(object)?,
        Some(Value::String(kind)) if kind == "mention" => validate_mention_rich_text_item(object)?,
        Some(Value::String(kind)) if kind == "equation" => {
            validate_equation_rich_text_item(object)?;
        }
        Some(Value::String(_)) => {
            return Err("rich text item type must be text, mention, or equation".to_string())
        }
        _ => return Err("rich text item type must be a string".to_string()),
    }
    validate_rich_text_common_fields(object)?;
    Ok(())
}

fn validate_text_rich_text_item(object: &serde_json::Map<String, Value>) -> Result<(), String> {
    let text = object
        .get("text")
        .and_then(Value::as_object)
        .ok_or_else(|| "rich text text payload is required".to_string())?;
    match text.get("content") {
        Some(Value::String(_)) => {}
        _ => return Err("rich text text.content must be a string".to_string()),
    }
    match text.get("link") {
        None | Some(Value::Null) => {}
        Some(Value::Object(link)) => {
            let url = link
                .get("url")
                .and_then(Value::as_str)
                .ok_or_else(|| "rich text text.link.url must be a string".to_string())?;
            validate_rich_text_url(url, "rich text text.link.url")?;
        }
        _ => return Err("rich text text.link must be null or an object".to_string()),
    }
    Ok(())
}

fn validate_equation_rich_text_item(object: &serde_json::Map<String, Value>) -> Result<(), String> {
    let expression = object
        .get("equation")
        .and_then(|equation| equation.get("expression"))
        .and_then(Value::as_str)
        .ok_or_else(|| "rich text equation.expression must be a string".to_string())?;
    validate_inline_equation_expression(expression, "rich text equation.expression")
}

fn validate_mention_rich_text_item(object: &serde_json::Map<String, Value>) -> Result<(), String> {
    let mention = object
        .get("mention")
        .and_then(Value::as_object)
        .ok_or_else(|| "rich text mention payload is required".to_string())?;
    match mention.get("type") {
        Some(Value::String(kind)) if kind == "page" => validate_page_mention(mention),
        Some(Value::String(kind)) if kind == "date" => validate_date_mention(mention),
        Some(Value::String(_)) => Err("rich text mention.type must be page or date".to_string()),
        _ => Err("rich text mention.type must be a string".to_string()),
    }
}

fn validate_page_mention(mention: &serde_json::Map<String, Value>) -> Result<(), String> {
    let page_id = mention
        .get("page")
        .and_then(|page| page.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "rich text mention.page.id must be a string".to_string())?;
    require_uuid(page_id, "rich text mention.page.id")
}

fn validate_date_mention(mention: &serde_json::Map<String, Value>) -> Result<(), String> {
    let date = mention
        .get("date")
        .and_then(Value::as_object)
        .ok_or_else(|| "rich text mention.date must be an object".to_string())?;
    let start = date
        .get("start")
        .and_then(Value::as_str)
        .ok_or_else(|| "rich text mention.date.start must be a string".to_string())?;
    validate_date_mention_boundary(start, "rich text mention.date.start")?;
    if let Some(end) = date.get("end") {
        if let Some(end_value) = end.as_str() {
            validate_date_mention_boundary(end_value, "rich text mention.date.end")?;
        } else if !end.is_null() {
            return Err("rich text mention.date.end must be null or a string".to_string());
        }
    }
    if let Some(time_zone) = date.get("time_zone") {
        if let Some(time_zone_value) = time_zone.as_str() {
            if time_zone_value.trim().is_empty() || contains_control_characters(time_zone_value) {
                return Err(
                    "rich text mention.date.time_zone must not be empty or contain control characters"
                        .to_string(),
                );
            }
            if time_zone_value.len() > 100 {
                return Err("rich text mention.date.time_zone is too long".to_string());
            }
        } else if !time_zone.is_null() {
            return Err("rich text mention.date.time_zone must be null or a string".to_string());
        }
    }
    if let Some(reminder) = date.get("ganbaru_reminder") {
        validate_date_mention_reminder(reminder)?;
    }
    Ok(())
}

fn validate_date_mention_reminder(value: &Value) -> Result<(), String> {
    if value.is_null() {
        return Ok(());
    }
    let reminder = value
        .as_object()
        .ok_or_else(|| "rich text mention.date.ganbaru_reminder must be an object".to_string())?;
    match reminder.get("enabled") {
        Some(Value::Bool(_)) => Ok(()),
        _ => Err("rich text mention.date.ganbaru_reminder.enabled must be a boolean".to_string()),
    }
}

fn validate_date_mention_boundary(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || contains_control_characters(value) {
        return Err(format!(
            "{field} must not be empty or contain control characters"
        ));
    }
    if value.len() > 80 {
        return Err(format!("{field} is too long"));
    }
    if !date_mention_boundary_looks_iso(value) {
        return Err(format!("{field} must be an ISO date or date-time"));
    }
    Ok(())
}

fn validate_inline_equation_expression(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || contains_control_characters(value) {
        return Err(format!(
            "{field} must not be empty or contain control characters"
        ));
    }
    if value.len() > 2048 {
        return Err(format!("{field} is too long"));
    }
    Ok(())
}

fn date_mention_boundary_looks_iso(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 10 {
        return false;
    }
    for (index, byte) in bytes.iter().take(10).enumerate() {
        match index {
            4 | 7 if *byte == b'-' => {}
            4 | 7 => return false,
            _ if byte.is_ascii_digit() => {}
            _ => return false,
        }
    }
    let month = two_digit_number(bytes[5], bytes[6]);
    let day = two_digit_number(bytes[8], bytes[9]);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }
    bytes.len() == 10 || bytes.get(10).is_some_and(|byte| *byte == b'T')
}

fn two_digit_number(tens: u8, ones: u8) -> u8 {
    (tens - b'0') * 10 + (ones - b'0')
}

fn validate_rich_text_common_fields(object: &serde_json::Map<String, Value>) -> Result<(), String> {
    let annotations = object
        .get("annotations")
        .and_then(Value::as_object)
        .ok_or_else(|| "rich text annotations are required".to_string())?;
    for field in ["bold", "italic", "strikethrough", "underline", "code"] {
        if !matches!(annotations.get(field), Some(Value::Bool(_))) {
            return Err(format!("rich text annotations.{field} must be a boolean"));
        }
    }
    validate_required_color(annotations.get("color"), "rich text annotations.color")?;
    match object.get("plain_text") {
        Some(Value::String(_)) => {}
        _ => return Err("rich text plain_text must be a string".to_string()),
    }
    if !matches!(
        object.get("href"),
        None | Some(Value::Null) | Some(Value::String(_))
    ) {
        return Err("rich text href must be null or a string".to_string());
    }
    if let Some(Value::String(href)) = object.get("href") {
        validate_rich_text_url(href, "rich text href")?;
    }
    Ok(())
}

fn validate_rich_text_url(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || contains_control_characters(value) {
        return Err(format!(
            "{field} must not be empty or contain control characters"
        ));
    }
    if value.len() > 2048 {
        return Err(format!("{field} is too long"));
    }
    let parsed = reqwest::Url::parse(value)
        .map_err(|_| format!("{field} must be a valid HTTP, HTTPS, or email URL"))?;
    match parsed.scheme() {
        "http" | "https" if parsed.host_str().is_some() => Ok(()),
        "mailto" if is_email_address_like(parsed.path()) => Ok(()),
        _ => Err(format!("{field} must be a valid HTTP, HTTPS, or email URL")),
    }
}

fn is_email_address_like(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value || trimmed.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = trimmed.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && domain.split('.').all(|segment| !segment.is_empty())
        && !domain.contains('@')
}

fn validate_optional_color(payload: &Value, field: &str) -> Result<(), String> {
    match payload.get("color") {
        None => Ok(()),
        Some(value) => validate_required_color(Some(value), field),
    }
}

fn validate_required_color(value: Option<&Value>, field: &str) -> Result<(), String> {
    match value {
        Some(Value::String(color)) if NOTE_COLORS.contains(&color.as_str()) => Ok(()),
        Some(Value::String(_)) => Err(format!("{field} must be a supported Notion color")),
        _ => Err(format!("{field} must be a string")),
    }
}

pub(in crate::notes) fn plain_text_from_payload(block_type: &str, payload: &Value) -> String {
    if matches!(
        block_type,
        "breadcrumb" | "table_of_contents" | "column_list" | "column" | "table" | "tab" | "divider"
    ) {
        return String::new();
    }
    if block_type == "table_row" {
        return table_row_plain_text(payload);
    }
    if matches!(block_type, "child_page" | "child_database") {
        return payload
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }
    if block_type == "bookmark" {
        return bookmark_plain_text(payload);
    }
    if matches!(block_type, "embed" | "link_preview") {
        return payload
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }
    if block_type == "synced_block" {
        return synced_block_plain_text(payload);
    }
    if block_type == "button" {
        return button_plain_text(payload);
    }
    if block_type == "equation" {
        return payload
            .get("expression")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }
    if matches!(block_type, "image" | "video" | "audio" | "file" | "pdf") {
        return media_plain_text(payload);
    }
    if block_type == "unsupported" {
        return unsupported_plain_text(payload);
    }
    payload
        .get("rich_text")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("plain_text")
                        .and_then(Value::as_str)
                        .or_else(|| item.get("text")?.get("content")?.as_str())
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

fn media_plain_text(payload: &Value) -> String {
    let caption = payload
        .get("caption")
        .and_then(Value::as_array)
        .map(|items| rich_text_items_plain_text(items))
        .unwrap_or_default();
    let name = payload
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let source = match payload.get("type").and_then(Value::as_str) {
        Some("external") => payload
            .get("external")
            .and_then(|external| external.get("url"))
            .and_then(Value::as_str)
            .unwrap_or_default(),
        Some("file") => payload
            .get("file")
            .and_then(|file| file.get("url"))
            .and_then(Value::as_str)
            .unwrap_or_default(),
        Some("file_upload") => payload
            .get("file_upload")
            .and_then(|file_upload| file_upload.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default(),
        _ => "",
    };
    [caption.as_str(), name, source]
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn unsupported_plain_text(payload: &Value) -> String {
    let block_type = payload
        .get("block_type")
        .and_then(Value::as_str)
        .or_else(|| payload.get("source_type").and_then(Value::as_str))
        .unwrap_or_default();
    let warnings = payload
        .get("warnings")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|warning| !warning.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    [block_type.trim(), warnings.trim()]
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

fn synced_block_plain_text(payload: &Value) -> String {
    match payload.get("synced_from") {
        Some(Value::Null) => "Synced block".to_string(),
        Some(Value::Object(synced_from)) => synced_from
            .get("block_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

fn button_plain_text(payload: &Value) -> String {
    payload
        .get("rich_text")
        .and_then(Value::as_array)
        .map(|items| rich_text_items_plain_text(items))
        .unwrap_or_else(|| "Button".to_string())
}

fn table_row_plain_text(payload: &Value) -> String {
    payload
        .get("cells")
        .and_then(Value::as_array)
        .map(|cells| {
            cells
                .iter()
                .filter_map(Value::as_array)
                .map(|items| rich_text_items_plain_text(items))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .unwrap_or_default()
}

fn bookmark_plain_text(payload: &Value) -> String {
    let caption = payload
        .get("caption")
        .and_then(Value::as_array)
        .map(|items| rich_text_items_plain_text(items))
        .unwrap_or_default();
    let url = payload
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match (caption.trim().is_empty(), url.trim().is_empty()) {
        (true, true) => String::new(),
        (true, false) => url.to_string(),
        (false, true) => caption,
        (false, false) => format!("{caption} {url}"),
    }
}

pub(in crate::notes) fn rich_text_items_plain_text(items: &[Value]) -> String {
    items
        .iter()
        .filter_map(|item| {
            item.get("plain_text")
                .and_then(Value::as_str)
                .or_else(|| item.get("text")?.get("content")?.as_str())
        })
        .collect::<Vec<_>>()
        .join("")
}
