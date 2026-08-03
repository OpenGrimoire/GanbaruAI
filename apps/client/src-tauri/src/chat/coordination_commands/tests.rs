use super::common::{normalized_fts_query, validate_message_request};
use super::scheduling::{claim_scheduled_message_for_immediate_send, validate_scheduled_for};
use super::*;
use serde_json::json;
use sqlx::{Row, SqlitePool};

async fn migrated_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    crate::db::run_migrations(&pool).await.unwrap();
    pool
}

fn message(markdown: &str) -> PostChatMessageCommand {
    PostChatMessageCommand {
        client_command_id: ChatCommandId::new("command:test").unwrap(),
        channel_id: ChatChannelId::new("channel:test").unwrap(),
        reply_thread_id: None,
        normalized_markdown: markdown.to_string(),
        rich_content: VersionedJson {
            schema_version: 1,
            value: json!({ "type": "doc", "content": [] }),
        },
        attachment_ids: Vec::new(),
        participant_mentions: Vec::new(),
        resource_references: Vec::new(),
        also_send_to_channel: false,
    }
}

#[test]
fn scheduled_delivery_requires_a_near_future_time_within_one_year() {
    let now = UtcTimestamp::new("2026-08-03T00:00:00.000Z").unwrap();

    assert!(validate_scheduled_for(
        &UtcTimestamp::new("2026-08-03T00:00:30.000Z").unwrap(),
        &now,
    )
    .is_ok());
    assert_eq!(
        validate_scheduled_for(
            &UtcTimestamp::new("2026-08-03T00:00:29.999Z").unwrap(),
            &now,
        )
        .unwrap_err()
        .field
        .as_deref(),
        Some("scheduledFor"),
    );
    assert_eq!(
        validate_scheduled_for(
            &UtcTimestamp::new("2027-08-05T00:00:00.000Z").unwrap(),
            &now,
        )
        .unwrap_err()
        .field
        .as_deref(),
        Some("scheduledFor"),
    );
}

#[test]
fn immediate_scheduled_delivery_claim_is_atomic() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        let now = UtcTimestamp::new("2026-08-03T00:00:00.000Z").unwrap();
        sqlx::query("INSERT INTO project_groups (id, name) VALUES ('group:test', 'Test')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name)
             VALUES ('project:test', 'group:test', 'Test')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_conversations
                (id, project_id, conversation_kind, last_activity_at, created_at, updated_at)
             VALUES ('conversation:test', 'project:test', 'channel', ?, ?, ?)",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_channels
                (id, project_id, conversation_id, name, created_at, updated_at)
             VALUES ('channel:test', 'project:test', 'conversation:test', 'general', ?, ?)",
        )
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&pool)
        .await
        .unwrap();
        let request_data = serde_json::to_string(&message("Send this now")).unwrap();
        sqlx::query(
            "INSERT INTO chat_scheduled_messages
                (id, client_command_id, channel_id, request_data, scheduled_for,
                 available_at, created_at, updated_at)
             VALUES ('scheduled:test', 'command:test', 'channel:test', ?, ?, ?, ?, ?)",
        )
        .bind(&request_data)
        .bind("2026-08-04T00:00:00.000Z")
        .bind("2026-08-04T00:00:00.000Z")
        .bind(now.as_str())
        .bind(now.as_str())
        .execute(&pool)
        .await
        .unwrap();
        let scheduled_message_id = ChatScheduledMessageId::new("scheduled:test").unwrap();
        let claimed = claim_scheduled_message_for_immediate_send(
            &pool,
            &scheduled_message_id,
            &now,
            "claim:first",
        )
        .await
        .unwrap();
        assert_eq!(claimed, request_data);
        let row = sqlx::query(
            "SELECT state, attempt_count, claim_token
             FROM chat_scheduled_messages WHERE id = 'scheduled:test'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<String, _>("state"), "dispatching");
        assert_eq!(row.get::<i64, _>("attempt_count"), 1);
        assert_eq!(row.get::<String, _>("claim_token"), "claim:first");
        let duplicate = claim_scheduled_message_for_immediate_send(
            &pool,
            &scheduled_message_id,
            &now,
            "claim:second",
        )
        .await
        .unwrap_err();
        assert_eq!(duplicate.code, ChatErrorCode::Busy);
    });
}

#[test]
fn plain_at_text_is_valid_but_does_not_invoke_a_teammate() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        let request = message("Please ask @ganbaru to review this.");
        validate_message_request(&request).unwrap();
        let invocation = resolve_invoked_teammate(
            &pool,
            &ChatConversationId::new("conversation:test").unwrap(),
            None,
            &request.participant_mentions,
        )
        .await
        .unwrap();
        assert!(invocation.is_none());
    });
}

#[test]
fn two_structured_ai_mentions_are_rejected_before_dispatch() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        for index in 1..=2 {
            sqlx::query(
                "INSERT INTO chat_participants
                    (id, participant_kind, display_name, normalized_handle,
                     created_at, updated_at)
                 VALUES (?, 'ai_teammate', ?, ?, ?, ?)",
            )
            .bind(format!("participant:agent-{index}"))
            .bind(format!("Agent {index}"))
            .bind(format!("agent-{index}"))
            .bind("2026-08-01T00:00:00.000Z")
            .bind("2026-08-01T00:00:00.000Z")
            .execute(&pool)
            .await
            .unwrap();
        }
        let mentions = (1..=2)
            .map(|index| ChatParticipantMentionInput {
                participant_id: ChatParticipantId::new(format!("participant:agent-{index}"))
                    .unwrap(),
                participant_kind: ChatParticipantKind::AiTeammate,
                handle_snapshot: Some(format!("agent-{index}")),
                label_snapshot: format!("Agent {index}"),
                start_offset: u64::try_from((index - 1) * 9).unwrap(),
                end_offset: u64::try_from(index * 9 - 1).unwrap(),
            })
            .collect::<Vec<_>>();
        let error = resolve_invoked_teammate(
            &pool,
            &ChatConversationId::new("conversation:test").unwrap(),
            None,
            &mentions,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, ChatErrorCode::Validation);
        assert_eq!(error.field.as_deref(), Some("participantMentions"));
    });
}

#[test]
fn message_validation_rejects_duplicate_atomic_mention_ranges() {
    let mut request = message("@ganbaru");
    let mention = ChatParticipantMentionInput {
        participant_id: ChatParticipantId::new("participant:ganbaru").unwrap(),
        participant_kind: ChatParticipantKind::AiTeammate,
        handle_snapshot: Some("ganbaru".to_string()),
        label_snapshot: "Ganbaru".to_string(),
        start_offset: 0,
        end_offset: 8,
    };
    request.participant_mentions = vec![mention.clone(), mention];
    let error = validate_message_request(&request).unwrap_err();
    assert_eq!(error.field.as_deref(), Some("participantMentions"));
}

#[test]
fn fts_queries_are_bounded_and_quote_each_term() {
    assert_eq!(
        normalized_fts_query("review exact-folder").unwrap(),
        "\"review\" AND \"exact-folder\""
    );
    assert!(normalized_fts_query("   ").is_err());
    assert!(normalized_fts_query(&"x".repeat(501)).is_err());
}
