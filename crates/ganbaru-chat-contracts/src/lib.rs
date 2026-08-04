//! Provider-neutral Chat commands, events, configuration, and wire models.

pub mod chat;

pub use chat::{config, events, models};

#[cfg(test)]
mod tests {
    use crate::chat::models::{ChatError, ChatErrorCode};
    use serde_json::json;

    #[test]
    fn chat_errors_keep_the_stable_wire_shape() {
        let error = ChatError::new(
            ChatErrorCode::DriverUnavailable,
            "Provider unavailable",
            true,
        );

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({
                "code": "driver_unavailable",
                "message": "Provider unavailable",
                "field": null,
                "recoverable": true,
                "details": null
            })
        );
    }
}
