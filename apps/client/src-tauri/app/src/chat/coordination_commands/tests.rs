use super::common::{has_thread_eligible_mention, normalized_fts_query, validate_message_request};
use super::scheduling::{claim_scheduled_message_for_immediate_send, validate_scheduled_for};
use super::*;
use serde_json::json;
use sqlx::{Row, SqlitePool};

const REMOVE_EMPTY_UNMENTIONED_CHAT_THREADS: &str =
    include_str!("../../../../migrations/20260805014734_remove_empty_unmentioned_chat_threads.sql");

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

async fn seed_review_assignment_with_stranded_replies(pool: &SqlitePool) {
    sqlx::raw_sql(
        "INSERT INTO project_groups (id, name) VALUES ('group:review', 'Review');
         INSERT INTO projects (id, group_id, name)
         VALUES ('project:review', 'group:review', 'Review');
         INSERT INTO project_working_folders
             (id, project_id, display_name, kind, managed_relative_path)
         VALUES ('folder:review', 'project:review', 'Review', 'managed', 'projects/review');
         UPDATE chat_conversations SET id = 'conversation:review'
         WHERE project_id = 'project:review' AND conversation_kind = 'channel';
         UPDATE chat_channels SET id = 'channel:review'
         WHERE project_id = 'project:review' AND is_default = 1;
         INSERT INTO chat_participants
             (id, participant_kind, display_name, normalized_handle, created_at, updated_at)
         VALUES (
             'participant:review-agent', 'ai_teammate', 'Ganbaru', 'ganbaru',
             '2026-08-04T17:00:00.000Z', '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_ai_teammates
             (participant_id, purpose, instructions, created_at, updated_at)
         VALUES (
             'participant:review-agent', 'Test', 'Test',
             '2026-08-04T17:00:00.000Z', '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_teammate_policy_revisions
             (id, teammate_id, revision, provider_instance_id,
              model_selection_data, created_at)
         VALUES (
             'policy:review', 'participant:review-agent', 1, 'provider:review',
             '{\"providerManagedModel\":true,\"modelId\":null,\"modelOptions\":[]}',
             '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_conversation_memberships
             (conversation_id, participant_id, approval_policy, created_at, updated_at)
         VALUES (
             'conversation:review', 'participant:review-agent', 'ask_for_approval',
             '2026-08-04T17:00:00.000Z', '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_teammate_working_folder_grants
             (conversation_id, teammate_id, project_id, working_folder_id,
              is_default, created_at)
         VALUES (
             'conversation:review', 'participant:review-agent', 'project:review',
             'folder:review', 1, '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_conversation_items
             (id, conversation_id, item_kind, ordinal, created_at)
         VALUES (
             'item:review-root', 'conversation:review', 'message', 1,
             '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_communication_messages
             (item_id, author_participant_id, created_at)
         VALUES (
             'item:review-root', 'participant:local-owner',
             '2026-08-04T17:00:00.000Z'
         );
         INSERT INTO chat_communication_message_revisions
             (id, message_item_id, revision, normalized_markdown, created_at)
         VALUES (
             'revision:review-root', 'item:review-root', 1, 'Create hello.py',
             '2026-08-04T17:00:00.000Z'
         );
         UPDATE chat_communication_messages
         SET current_revision_id = 'revision:review-root'
         WHERE item_id = 'item:review-root';
         INSERT INTO chat_reply_threads
             (id, conversation_id, root_item_id, reply_count, last_activity_at,
              created_at, updated_at)
         VALUES (
             'reply-thread:review', 'conversation:review', 'item:review-root', 2,
             '2026-08-04T17:02:00.000Z', '2026-08-04T17:00:00.000Z',
             '2026-08-04T17:02:00.000Z'
         );
         INSERT INTO chat_conversation_items
             (id, conversation_id, reply_thread_id, item_kind, ordinal, created_at)
         VALUES
             ('item:review-reply-1', 'conversation:review', 'reply-thread:review',
              'message', 1, '2026-08-04T17:01:00.000Z'),
             ('item:review-reply-2', 'conversation:review', 'reply-thread:review',
              'message', 2, '2026-08-04T17:02:00.000Z');
         INSERT INTO chat_communication_messages
             (item_id, author_participant_id, created_at)
         VALUES
             ('item:review-reply-1', 'participant:local-owner',
              '2026-08-04T17:01:00.000Z'),
             ('item:review-reply-2', 'participant:local-owner',
              '2026-08-04T17:02:00.000Z');
         INSERT INTO chat_communication_message_revisions
             (id, message_item_id, revision, normalized_markdown, created_at)
         VALUES
             ('revision:review-reply-1', 'item:review-reply-1', 1,
              'Ok, remove it now.', '2026-08-04T17:01:00.000Z'),
             ('revision:review-reply-2', 'item:review-reply-2', 1,
              '@ganbaru Ok, remove it now.', '2026-08-04T17:02:00.000Z');
         UPDATE chat_communication_messages
         SET current_revision_id = 'revision:review-reply-1'
         WHERE item_id = 'item:review-reply-1';
         UPDATE chat_communication_messages
         SET current_revision_id = 'revision:review-reply-2'
         WHERE item_id = 'item:review-reply-2';
         INSERT INTO chat_work_assignments
             (id, reply_thread_id, teammate_id, triggering_message_item_id,
              state, created_at, updated_at)
         VALUES (
             'assignment:review', 'reply-thread:review', 'participant:review-agent',
             'item:review-root', 'ready_for_review',
             '2026-08-04T17:00:00.000Z', '2026-08-04T17:02:00.000Z'
         );
         INSERT INTO chat_work_assignment_inputs
             (id, assignment_id, message_item_id, ordinal, routing_kind,
              delivery_state, created_at, delivered_at)
         VALUES
             ('input:review-root', 'assignment:review', 'item:review-root', 1,
              'trigger', 'delivered', '2026-08-04T17:00:00.000Z',
              '2026-08-04T17:00:00.000Z'),
             ('input:review-reply-1', 'assignment:review', 'item:review-reply-1', 2,
              'queued_continuation', 'pending', '2026-08-04T17:01:00.000Z', NULL),
             ('input:review-reply-2', 'assignment:review', 'item:review-reply-2', 3,
              'queued_continuation', 'pending', '2026-08-04T17:02:00.000Z', NULL);",
    )
    .execute(pool)
    .await
    .unwrap();
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
fn restart_rehomes_review_replies_into_one_ordered_follow_up_assignment() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        seed_review_assignment_with_stranded_replies(&pool).await;
        let now = UtcTimestamp::new("2026-08-04T17:03:00.000Z").unwrap();

        let recovered = assignments::recover_stranded_follow_up_assignments(&pool, &now)
            .await
            .unwrap();

        assert_eq!(recovered, 1);
        let old_state: String = sqlx::query_scalar(
            "SELECT state FROM chat_work_assignments WHERE id = 'assignment:review'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(old_state, "completed");
        let follow_up = sqlx::query(
            "SELECT id, triggering_message_item_id, previous_assignment_id, state
             FROM chat_work_assignments WHERE previous_assignment_id = 'assignment:review'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let follow_up_id = follow_up.get::<String, _>("id");
        assert_eq!(
            follow_up.get::<String, _>("triggering_message_item_id"),
            "item:review-reply-1"
        );
        assert_eq!(follow_up.get::<String, _>("state"), "queued");
        let inputs = sqlx::query(
            "SELECT message_item_id, ordinal, routing_kind, delivery_state
             FROM chat_work_assignment_inputs WHERE assignment_id = ? ORDER BY ordinal",
        )
        .bind(&follow_up_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(inputs.len(), 2);
        assert_eq!(
            inputs[0].get::<String, _>("message_item_id"),
            "item:review-reply-1"
        );
        assert_eq!(inputs[0].get::<String, _>("routing_kind"), "follow_up");
        assert_eq!(
            inputs[1].get::<String, _>("message_item_id"),
            "item:review-reply-2"
        );
        assert_eq!(
            inputs[1].get::<String, _>("routing_kind"),
            "queued_continuation"
        );
        assert!(inputs
            .iter()
            .all(|input| input.get::<String, _>("delivery_state") == "pending"));
        let queued_job: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM chat_assignment_dispatch_jobs
             WHERE assignment_id = ? AND state = 'queued'",
        )
        .bind(follow_up_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(queued_job, 1);
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
fn only_structured_participant_mentions_make_top_level_messages_thread_eligible() {
    let mut request = message("Ordinary channel message");
    assert!(!has_thread_eligible_mention(&request));

    request.normalized_markdown = "@ganbaru Please review this.".to_string();
    request.participant_mentions = vec![ChatParticipantMentionInput {
        participant_id: ChatParticipantId::new("participant:ganbaru").unwrap(),
        participant_kind: ChatParticipantKind::AiTeammate,
        handle_snapshot: Some("ganbaru".to_string()),
        label_snapshot: "Ganbaru".to_string(),
        start_offset: 0,
        end_offset: 8,
    }];

    assert!(has_thread_eligible_mention(&request));
}

#[test]
fn empty_unmentioned_thread_cleanup_preserves_mentioned_threads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        sqlx::raw_sql(
            "INSERT INTO project_groups (id, name) VALUES ('group:cleanup', 'Cleanup');
             INSERT INTO projects (id, group_id, name)
             VALUES ('project:cleanup', 'group:cleanup', 'Cleanup');
             INSERT INTO chat_conversations
                 (id, project_id, conversation_kind, last_activity_at,
                  created_at, updated_at)
             VALUES (
                 'conversation:cleanup', 'project:cleanup', 'channel',
                 '2026-08-04T18:00:00.000Z', '2026-08-04T18:00:00.000Z',
                 '2026-08-04T18:00:00.000Z'
             );
             INSERT INTO chat_channels
                 (id, project_id, conversation_id, name, created_at, updated_at)
             VALUES (
                 'channel:cleanup', 'project:cleanup', 'conversation:cleanup',
                 'cleanup', '2026-08-04T18:00:00.000Z',
                 '2026-08-04T18:00:00.000Z'
             );
             INSERT INTO chat_conversation_items
                 (id, conversation_id, item_kind, ordinal, created_at)
             VALUES
                 ('item:ordinary', 'conversation:cleanup', 'message', 1,
                  '2026-08-04T18:00:00.000Z'),
                 ('item:mentioned', 'conversation:cleanup', 'message', 2,
                  '2026-08-04T18:01:00.000Z'),
                 ('item:drafted', 'conversation:cleanup', 'message', 3,
                  '2026-08-04T18:02:00.000Z'),
                 ('item:scheduled', 'conversation:cleanup', 'message', 4,
                  '2026-08-04T18:03:00.000Z');
             INSERT INTO chat_communication_messages
                 (item_id, author_participant_id, created_at)
             VALUES
                 ('item:ordinary', 'participant:local-owner',
                  '2026-08-04T18:00:00.000Z'),
                 ('item:mentioned', 'participant:local-owner',
                  '2026-08-04T18:01:00.000Z'),
                 ('item:drafted', 'participant:local-owner',
                  '2026-08-04T18:02:00.000Z'),
                 ('item:scheduled', 'participant:local-owner',
                  '2026-08-04T18:03:00.000Z');
             INSERT INTO chat_communication_message_revisions
                 (id, message_item_id, revision, normalized_markdown, created_at)
             VALUES
                 ('revision:ordinary', 'item:ordinary', 1, 'Ordinary',
                  '2026-08-04T18:00:00.000Z'),
                 ('revision:mentioned', 'item:mentioned', 1, '@You Mentioned',
                  '2026-08-04T18:01:00.000Z');
             UPDATE chat_communication_messages
             SET current_revision_id = 'revision:ordinary'
             WHERE item_id = 'item:ordinary';
             UPDATE chat_communication_messages
             SET current_revision_id = 'revision:mentioned'
             WHERE item_id = 'item:mentioned';
             INSERT INTO chat_participant_mentions
                 (id, message_revision_id, participant_id, participant_kind,
                  label_snapshot, start_offset, end_offset)
             VALUES (
                 'mention:cleanup', 'revision:mentioned', 'participant:local-owner',
                 'local_user', 'You', 0, 4
             );
             INSERT INTO chat_reply_threads
                 (id, conversation_id, root_item_id, last_activity_at,
                  created_at, updated_at)
             VALUES
                 ('thread:ordinary', 'conversation:cleanup', 'item:ordinary',
                  '2026-08-04T18:00:00.000Z', '2026-08-04T18:00:00.000Z',
                  '2026-08-04T18:00:00.000Z'),
                 ('thread:mentioned', 'conversation:cleanup', 'item:mentioned',
                  '2026-08-04T18:01:00.000Z', '2026-08-04T18:01:00.000Z',
                  '2026-08-04T18:01:00.000Z'),
                 ('thread:drafted', 'conversation:cleanup', 'item:drafted',
                  '2026-08-04T18:02:00.000Z', '2026-08-04T18:02:00.000Z',
                  '2026-08-04T18:02:00.000Z'),
                 ('thread:scheduled', 'conversation:cleanup', 'item:scheduled',
                  '2026-08-04T18:03:00.000Z', '2026-08-04T18:03:00.000Z',
                  '2026-08-04T18:03:00.000Z');
             INSERT INTO chat_organizational_drafts
                 (id, participant_id, conversation_id, reply_thread_id,
                  created_at, updated_at)
             VALUES (
                 'draft:cleanup', 'participant:local-owner',
                 'conversation:cleanup', 'thread:drafted',
                 '2026-08-04T18:02:00.000Z', '2026-08-04T18:02:00.000Z'
             );
             INSERT INTO chat_scheduled_messages
                 (id, client_command_id, channel_id, reply_thread_id,
                  request_data, scheduled_for, available_at, created_at, updated_at)
             VALUES (
                 'scheduled:cleanup', 'command:scheduled-cleanup',
                 'channel:cleanup', 'thread:scheduled', '{}',
                 '2026-08-05T18:03:00.000Z', '2026-08-05T18:03:00.000Z',
                 '2026-08-04T18:03:00.000Z', '2026-08-04T18:03:00.000Z'
             );
             INSERT INTO chat_organizational_command_receipts
                 (client_command_id, command_kind, state, result_schema_version,
                  result_data, created_at, updated_at)
             VALUES (
                 'command:ordinary-cleanup', 'post_message', 'completed', 1,
                 '{\"message\":{\"itemId\":\"item:ordinary\"},\"replyThreadId\":\"thread:ordinary\"}',
                 '2026-08-04T18:00:00.000Z', '2026-08-04T18:00:00.000Z'
             );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::raw_sql(REMOVE_EMPTY_UNMENTIONED_CHAT_THREADS)
            .execute(&pool)
            .await
            .unwrap();

        let remaining = sqlx::query_scalar::<_, String>(
            "SELECT id FROM chat_reply_threads
             WHERE conversation_id = 'conversation:cleanup' ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            remaining,
            vec!["thread:drafted", "thread:mentioned", "thread:scheduled"]
        );
        let receipt_thread_id = sqlx::query_scalar::<_, Option<String>>(
            "SELECT json_extract(result_data, '$.replyThreadId')
             FROM chat_organizational_command_receipts
             WHERE client_command_id = 'command:ordinary-cleanup'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(receipt_thread_id, None);
    });
}

#[test]
fn channel_unread_count_excludes_local_messages() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_pool().await;
        sqlx::raw_sql(
            "INSERT INTO project_groups (id, name) VALUES ('group:unread', 'Unread');
             INSERT INTO projects (id, group_id, name)
             VALUES ('project:unread', 'group:unread', 'Unread');
             INSERT INTO chat_conversations
                 (id, project_id, conversation_kind, last_activity_at,
                  created_at, updated_at)
             VALUES (
                 'conversation:unread', 'project:unread', 'channel',
                 '2026-08-04T19:01:00.000Z', '2026-08-04T19:00:00.000Z',
                 '2026-08-04T19:01:00.000Z'
             );
             INSERT INTO chat_channels
                 (id, project_id, conversation_id, name, created_at, updated_at)
             VALUES (
                 'channel:unread', 'project:unread', 'conversation:unread',
                 'general', '2026-08-04T19:00:00.000Z',
                 '2026-08-04T19:01:00.000Z'
             );
             INSERT INTO chat_participants
                 (id, participant_kind, display_name, normalized_handle,
                  created_at, updated_at)
             VALUES (
                 'participant:collaborator', 'human', 'Collaborator',
                 'collaborator',
                 '2026-08-04T19:00:00.000Z', '2026-08-04T19:00:00.000Z'
             );
             INSERT INTO chat_conversation_items
                 (id, conversation_id, item_kind, ordinal, created_at)
             VALUES
                 ('item:local-unread', 'conversation:unread', 'message', 1,
                  '2026-08-04T19:00:00.000Z'),
                 ('item:incoming-unread', 'conversation:unread', 'message', 2,
                  '2026-08-04T19:01:00.000Z');
             INSERT INTO chat_communication_messages
                 (item_id, author_participant_id, created_at)
             VALUES
                 ('item:local-unread', 'participant:local-owner',
                  '2026-08-04T19:00:00.000Z'),
                 ('item:incoming-unread', 'participant:collaborator',
                  '2026-08-04T19:01:00.000Z');",
        )
        .execute(&pool)
        .await
        .unwrap();

        let channel = super::super::channel_commands::read_channel(
            &pool,
            &ChatChannelId::new("channel:unread").unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(channel.message_count, 2);
        assert_eq!(channel.unread_count, 1);
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
