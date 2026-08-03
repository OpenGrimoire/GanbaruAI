use super::*;

#[test]
fn event_append_updates_projection_sequence_and_revision_atomically() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let first = append_canonical_event(&pool, content_event("event-1", "Hello "))
            .await
            .unwrap();
        let second = append_canonical_event(&pool, content_event("event-2", "world"))
            .await
            .unwrap();

        assert_eq!(first.event.sequence, 1);
        assert_eq!(first.notification.revision, 2);
        assert_eq!(second.event.sequence, 2);
        assert_eq!(second.notification.revision, 3);
        let thread: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT revision, last_event_sequence, last_projected_sequence, message_count
             FROM chat_threads WHERE id = 'thread-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(thread, (3, 2, 2, 1));
        let text: String = sqlx::query_scalar(
            "SELECT normalized_markdown FROM chat_messages WHERE id = 'assistant-message-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(text, "Hello world");

        let replay = read_canonical_events(&pool, &ChatThreadId::new("thread-1").unwrap(), 0)
            .await
            .unwrap();
        assert_eq!(replay.len(), 2);
        assert_eq!(replay[0].runtime, first.event.runtime);
        assert_eq!(replay[1].runtime, second.event.runtime);
    });
}

#[test]
fn completed_activity_preserves_streamed_command_output() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(
            &pool,
            canonical_request(
                "event-command-start",
                None,
                CanonicalEvent::ItemStarted(ItemLifecycleEvent {
                    item_id: "command-1".to_string(),
                    kind: CanonicalItemKind::CommandExecution,
                    status: ActivityStatus::Active,
                    title: Some("printf 4".to_string()),
                    detail: None,
                    safe_metadata: None,
                }),
            ),
        )
        .await
        .unwrap();
        let mut output = content_event("event-command-output", "4\n");
        let CanonicalEvent::ContentDelta(delta) = &mut output.runtime.event else {
            unreachable!()
        };
        delta.item_id = "command-1".to_string();
        delta.stream_kind = ContentStreamKind::CommandOutput;
        append_canonical_event(&pool, output).await.unwrap();
        append_canonical_event(
            &pool,
            canonical_request(
                "event-command-complete",
                None,
                CanonicalEvent::ItemCompleted(ItemLifecycleEvent {
                    item_id: "command-1".to_string(),
                    kind: CanonicalItemKind::CommandExecution,
                    status: ActivityStatus::Completed,
                    title: Some("printf 4".to_string()),
                    detail: None,
                    safe_metadata: None,
                }),
            ),
        )
        .await
        .unwrap();

        let detail: String =
            sqlx::query_scalar("SELECT detail FROM chat_activities WHERE id = 'command-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(detail, "4\n");

        sqlx::query("UPDATE chat_activities SET detail = NULL WHERE id = 'command-1'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql(include_str!(
            "../../../../migrations/20260727223218_restore_chat_activity_output.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();
        let restored: String =
            sqlx::query_scalar("SELECT detail FROM chat_activities WHERE id = 'command-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(restored, "4\n");
    });
}

#[test]
fn projections_keep_assistant_phase_and_turn_diff_data() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(
            &pool,
            canonical_request(
                "event-turn-start",
                Some("turn-typed-items"),
                CanonicalEvent::TurnStarted(TurnStartedEvent {
                    provider_turn_id: None,
                    state: ChatTurnState::Active,
                    modes: TurnModeSnapshot {
                        safety_mode: SafetyMode::AskForApproval,
                        interaction_mode: InteractionMode::Build,
                    },
                    model_id: None,
                    model_options: Vec::new(),
                }),
            ),
        )
        .await
        .unwrap();
        append_canonical_event(
            &pool,
            canonical_request(
                "event-final-answer",
                Some("turn-typed-items"),
                CanonicalEvent::ItemCompleted(ItemLifecycleEvent {
                    item_id: "answer-1".to_string(),
                    kind: CanonicalItemKind::AssistantMessage,
                    status: ActivityStatus::Completed,
                    title: Some("Assistant message".to_string()),
                    detail: Some("Final answer".to_string()),
                    safe_metadata: Some(VersionedJson {
                        schema_version: 1,
                        value: serde_json::json!({ "phase": "final_answer" }),
                    }),
                }),
            ),
        )
        .await
        .unwrap();
        let changed_file = ChangedFileSummary {
            relative_path: "src/main.rs".to_string(),
            previous_relative_path: None,
            additions: Some(2),
            deletions: Some(1),
            binary: false,
            status: "modified".to_string(),
        };
        append_canonical_event(
            &pool,
            canonical_request(
                "event-turn-diff",
                Some("turn-typed-items"),
                CanonicalEvent::DiffUpdated(DiffUpdatedEvent {
                    source: "codex".to_string(),
                    files: vec![changed_file.clone()],
                    provider_diff: None,
                }),
            ),
        )
        .await
        .unwrap();
        append_canonical_event(
            &pool,
            canonical_request(
                "event-turn-complete",
                Some("turn-typed-items"),
                CanonicalEvent::TurnCompleted(TurnCompletedEvent {
                    state: ChatTurnState::Completed,
                    stop_reason: None,
                    usage: None,
                    changed_files: Vec::new(),
                }),
            ),
        )
        .await
        .unwrap();

        let message = sqlx::query(
            "SELECT normalized_markdown, streaming_state, content_metadata_data
             FROM chat_messages WHERE id = 'answer-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            message.get::<String, _>("normalized_markdown"),
            "Final answer"
        );
        assert_eq!(message.get::<String, _>("streaming_state"), "complete");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &message.get::<String, _>("content_metadata_data")
            )
            .unwrap()["phase"],
            "final_answer"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM chat_activities WHERE id = 'answer-1'"
            )
            .fetch_one(&pool)
            .await
            .unwrap(),
            0
        );
        let page = read_timeline_page(&pool, &ChatThreadId::new("thread-1").unwrap(), None, 50)
            .await
            .unwrap();
        assert_eq!(page.turns[0].changed_files, vec![changed_file]);
    });
}

#[test]
fn projection_failure_rolls_back_event_and_thread_advance() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let mut request = content_event("event-invalid", "ignored");
        request.runtime.event =
            CanonicalEvent::TurnStarted(crate::chat::events::TurnStartedEvent {
                provider_turn_id: None,
                state: crate::chat::models::ChatTurnState::Active,
                modes: crate::chat::models::TurnModeSnapshot {
                    safety_mode: crate::chat::models::SafetyMode::AskForApproval,
                    interaction_mode: crate::chat::models::InteractionMode::Build,
                },
                model_id: None,
                model_options: Vec::new(),
            });

        assert!(append_canonical_event(&pool, request).await.is_err());
        let event_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chat_events")
            .fetch_one(&pool)
            .await
            .unwrap();
        let thread = sqlx::query(
            "SELECT revision, last_event_sequence FROM chat_threads WHERE id = 'thread-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(event_count, 0);
        assert_eq!(thread.get::<i64, _>("revision"), 1);
        assert_eq!(thread.get::<i64, _>("last_event_sequence"), 0);
    });
}
