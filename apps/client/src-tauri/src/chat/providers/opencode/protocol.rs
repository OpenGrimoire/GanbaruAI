//! OpenCode resume cursors and bounded protocol helpers.

use crate::chat::models::{ChatError, ChatErrorCode, ChatResult, VersionedJson};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const OPENCODE_RESUME_SCHEMA_VERSION: u32 = 1;
pub const MAX_HTTP_BODY_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_EVENT_DATA_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeResumeCursor {
    pub session_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeRollbackCursor {
    pub message_id: String,
    #[serde(default)]
    pub part_id: Option<String>,
}

pub fn resume_cursor(session_id: &str) -> ChatResult<VersionedJson> {
    validate_identifier(session_id, "session ID")?;
    Ok(VersionedJson {
        schema_version: OPENCODE_RESUME_SCHEMA_VERSION,
        value: serde_json::to_value(OpenCodeResumeCursor {
            session_id: session_id.to_string(),
        })
        .map_err(|_| protocol_error("resume cursor"))?,
    })
}

pub fn parse_resume_cursor(value: &VersionedJson) -> ChatResult<OpenCodeResumeCursor> {
    if value.schema_version != OPENCODE_RESUME_SCHEMA_VERSION {
        return Err(ChatError::validation(
            "resumeCursor.schemaVersion",
            "OpenCode resume cursor schema is unsupported",
        ));
    }
    let cursor = serde_json::from_value::<OpenCodeResumeCursor>(value.value.clone())
        .map_err(|_| ChatError::validation("resumeCursor", "OpenCode resume cursor is invalid"))?;
    validate_identifier(&cursor.session_id, "session ID")?;
    Ok(cursor)
}

pub fn parse_rollback_cursor(value: &VersionedJson) -> ChatResult<OpenCodeRollbackCursor> {
    if value.schema_version != OPENCODE_RESUME_SCHEMA_VERSION {
        return Err(ChatError::validation(
            "providerCursor.schemaVersion",
            "OpenCode rollback cursor schema is unsupported",
        ));
    }
    let cursor =
        serde_json::from_value::<OpenCodeRollbackCursor>(value.value.clone()).map_err(|_| {
            ChatError::validation("providerCursor", "OpenCode rollback cursor is invalid")
        })?;
    validate_identifier(&cursor.message_id, "message ID")?;
    if let Some(part_id) = cursor.part_id.as_deref() {
        validate_identifier(part_id, "part ID")?;
    }
    Ok(cursor)
}

pub fn confirmed_not_found(status: u16, body: Option<&Value>) -> bool {
    if status == 404 {
        return true;
    }
    if status != 0 {
        return false;
    }
    body.and_then(Value::as_object)
        .and_then(|object| object.get("name"))
        .and_then(Value::as_str)
        == Some("NotFoundError")
}

pub fn validate_identifier(value: &str, label: &str) -> ChatResult<()> {
    if value.is_empty()
        || value.len() > 512
        || value.chars().any(|character| character.is_control())
    {
        return Err(protocol_error(label));
    }
    Ok(())
}

pub fn protocol_error(detail: &str) -> ChatError {
    ChatError::new(
        ChatErrorCode::Protocol,
        format!("OpenCode returned an invalid {detail}"),
        false,
    )
}
