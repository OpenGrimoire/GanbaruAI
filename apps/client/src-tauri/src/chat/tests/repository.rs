use crate::chat::events::{
    CanonicalEvent, CanonicalRuntimeEvent, ContentDeltaEvent, ItemLifecycleEvent,
    RequestOpenedEvent, TurnCompletedEvent, TurnStartedEvent,
};
use crate::chat::ingestion::{ChatChangeEmitter, ChatEventIngestor};
use crate::chat::models::{
    ActivityStatus, CanonicalItemKind, CanonicalRequestKind, ChatAttachmentId, ChatCommandId,
    ChatEventId, ChatThreadId, ChatTurnId, ChatTurnState, ChatWorkspaceId, ContentStreamKind,
    InteractionMode, ProviderFamilyId, ProviderInstanceId, ProviderRequestId, SafetyMode,
    TurnModeSnapshot, UtcTimestamp, VersionedJson,
};
use crate::chat::models::{ChatChangeNotification, ChatError, ChatResult};
use crate::chat::repository::attachments::{
    import_attachment, run_due_attachment_cleanup, ChatAttachmentKind,
};
use crate::chat::repository::drafts::{delete_draft, read_draft, save_draft, ChatDraftWrite};
use crate::chat::repository::events::{
    append_canonical_event, read_canonical_events, AppendCanonicalEventRequest,
};
use crate::chat::repository::lifecycle::{
    resolve_project_deletion, set_project_chat_archived, set_thread_archived, set_thread_read,
    LinkedChatDeletionDecision,
};
use crate::chat::repository::reads::{
    parse_timeline_cursor, read_project_shells, read_thread_shells, read_timeline_page,
    search_thread_titles,
};
use crate::chat::repository::rebuild::rebuild_thread_projections;
use crate::chat::repository::receipts::{
    claim_command_receipt, complete_command_receipt, CommandReceiptClaim, CommandReceiptState,
};
use crate::chat::repository::recovery::recover_orphaned_turns;
use crate::chat::repository::workspaces::{create_workspace, list_workspaces, rename_workspace};
use crate::chat::workspace::CreateChatWorkspaceRequest;
use sqlx::Row;
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct RecordingEmitter(Mutex<Vec<ChatChangeNotification>>);

impl ChatChangeEmitter for RecordingEmitter {
    fn emit(&self, notification: &ChatChangeNotification) -> ChatResult<()> {
        self.0
            .lock()
            .map_err(|_| ChatError::driver_unavailable("emitter lock"))?
            .push(notification.clone());
        Ok(())
    }
}

const NOW: &str = "2026-07-20T12:00:00Z";

pub(crate) async fn pool_with_thread() -> sqlx::SqlitePool {
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
    sqlx::query(
        "INSERT INTO chat_workspaces
            (id, display_name, repository_kind, created_at, updated_at)
         VALUES ('workspace-1', 'Standalone', 'none', ?, ?)",
    )
    .bind(NOW)
    .bind(NOW)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO chat_threads
            (id, workspace_id, title, provider_family_id, provider_instance_id,
             continuation_group_id, safety_mode, interaction_mode, state,
             last_activity_at, created_at, updated_at)
         VALUES ('thread-1', 'workspace-1', 'Chat', 'codex', 'codex-personal',
                 'continuation-1', 'supervised', 'build', 'idle', ?, ?, ?)",
    )
    .bind(NOW)
    .bind(NOW)
    .bind(NOW)
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn content_event(event_id: &str, delta: &str) -> AppendCanonicalEventRequest {
    AppendCanonicalEventRequest {
        runtime: CanonicalRuntimeEvent {
            schema_version: 1,
            event_id: ChatEventId::new(event_id).unwrap(),
            provider_family_id: ProviderFamilyId::new("codex").unwrap(),
            provider_instance_id: ProviderInstanceId::new("codex-personal").unwrap(),
            thread_id: ChatThreadId::new("thread-1").unwrap(),
            created_at: UtcTimestamp::new(NOW).unwrap(),
            turn_id: None,
            provider_turn_id: None,
            provider_item_id: None,
            provider_request_id: None,
            provider_task_id: None,
            provider_reference: None,
            event: CanonicalEvent::ContentDelta(ContentDeltaEvent {
                item_id: "assistant-message-1".to_string(),
                stream_kind: ContentStreamKind::AssistantText,
                content_index: 0,
                delta: delta.to_string(),
            }),
            redacted_diagnostic: None,
        },
        ingested_at: UtcTimestamp::new(NOW).unwrap(),
        diagnostic_expires_at: None,
    }
}

fn canonical_request(
    event_id: &str,
    turn_id: Option<&str>,
    event: CanonicalEvent,
) -> AppendCanonicalEventRequest {
    AppendCanonicalEventRequest {
        runtime: CanonicalRuntimeEvent {
            schema_version: 1,
            event_id: ChatEventId::new(event_id).unwrap(),
            provider_family_id: ProviderFamilyId::new("codex").unwrap(),
            provider_instance_id: ProviderInstanceId::new("codex-personal").unwrap(),
            thread_id: ChatThreadId::new("thread-1").unwrap(),
            created_at: UtcTimestamp::new(NOW).unwrap(),
            turn_id: turn_id.map(|value| ChatTurnId::new(value).unwrap()),
            provider_turn_id: None,
            provider_item_id: None,
            provider_request_id: None,
            provider_task_id: None,
            provider_reference: None,
            event,
            redacted_diagnostic: None,
        },
        ingested_at: UtcTimestamp::new(NOW).unwrap(),
        diagnostic_expires_at: None,
    }
}

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
fn projection_failure_rolls_back_event_and_thread_advance() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let mut request = content_event("event-invalid", "ignored");
        request.runtime.event =
            CanonicalEvent::TurnStarted(crate::chat::events::TurnStartedEvent {
                provider_turn_id: None,
                state: crate::chat::models::ChatTurnState::Active,
                modes: crate::chat::models::TurnModeSnapshot {
                    safety_mode: crate::chat::models::SafetyMode::Supervised,
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

#[test]
fn command_receipts_replay_every_idempotent_command_kind() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let thread_id = ChatThreadId::new("thread-1").unwrap();
        let now = UtcTimestamp::new(NOW).unwrap();
        for command_kind in [
            "send",
            "approval",
            "answer",
            "interrupt",
            "restore",
            "delete",
        ] {
            let command_id = ChatCommandId::new(format!("command-{command_kind}")).unwrap();
            let claim =
                claim_command_receipt(&pool, &command_id, &thread_id, command_kind, Some(1), &now)
                    .await
                    .unwrap();
            assert!(matches!(claim, CommandReceiptClaim::Claimed(_)));
            let result = VersionedJson {
                schema_version: 7,
                value: serde_json::json!({ "commandKind": command_kind, "future": true }),
            };
            complete_command_receipt(
                &pool,
                &command_id,
                CommandReceiptState::Completed,
                Some(&result),
                None,
                &now,
            )
            .await
            .unwrap();
            let replay =
                claim_command_receipt(&pool, &command_id, &thread_id, command_kind, Some(1), &now)
                    .await
                    .unwrap();
            let CommandReceiptClaim::Replay(receipt) = replay else {
                panic!("duplicate command should replay its original receipt");
            };
            assert_eq!(receipt.result, Some(result));
        }
    });
}

#[test]
fn command_receipts_reject_cross_thread_and_changed_command_reuse() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        sqlx::query(
            "INSERT INTO chat_threads
                (id, workspace_id, title, provider_family_id, provider_instance_id,
                 continuation_group_id, safety_mode, interaction_mode, state,
                 last_activity_at, created_at, updated_at)
             VALUES ('thread-2', 'workspace-1', 'Other', 'codex', 'codex-personal',
                     'continuation-1', 'supervised', 'build', 'idle', ?, ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        let command_id = ChatCommandId::new("command-boundary").unwrap();
        let thread_1 = ChatThreadId::new("thread-1").unwrap();
        let thread_2 = ChatThreadId::new("thread-2").unwrap();
        let now = UtcTimestamp::new(NOW).unwrap();
        claim_command_receipt(&pool, &command_id, &thread_1, "approval", Some(1), &now)
            .await
            .unwrap();

        assert!(
            claim_command_receipt(&pool, &command_id, &thread_2, "approval", Some(1), &now,)
                .await
                .is_err()
        );
        assert!(
            claim_command_receipt(&pool, &command_id, &thread_1, "answer", Some(1), &now,)
                .await
                .is_err()
        );
    });
}

#[test]
fn shell_search_timeline_and_archive_reads_stay_lightweight() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(&pool, content_event("event-shell", "Searchable response"))
            .await
            .unwrap();
        let projects = read_project_shells(&pool).await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].active_thread_count, 1);
        let shells = read_thread_shells(&pool, None, false).await.unwrap();
        assert_eq!(shells.len(), 1);
        assert_eq!(shells[0].message_count, 1);
        let search = search_thread_titles(&pool, "cha", Some(false), 20)
            .await
            .unwrap();
        assert_eq!(search.len(), 1);
        let page = read_timeline_page(&pool, &ChatThreadId::new("thread-1").unwrap(), None, 20)
            .await
            .unwrap();
        assert_eq!(page.items.len(), 1);

        let revision = set_thread_read(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            false,
            2,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        let revision = set_thread_archived(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            true,
            revision,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        assert!(read_thread_shells(&pool, None, false)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            read_thread_shells(&pool, None, true).await.unwrap().len(),
            1
        );
        set_thread_archived(
            &pool,
            &ChatThreadId::new("thread-1").unwrap(),
            false,
            revision,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(
            read_thread_shells(&pool, None, false).await.unwrap().len(),
            1
        );
    });
}

#[test]
fn timeline_cursor_does_not_skip_rows_that_share_a_sequence() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(&pool, content_event("event-page", "Answer"))
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO chat_activities
                (id, thread_id, sequence_anchor, item_kind, status, title,
                 source_event_type, created_at, updated_at)
             VALUES ('activity-same-sequence', 'thread-1', 1, 'web_search',
                     'completed', 'Search', 'item_completed', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let thread_id = ChatThreadId::new("thread-1").unwrap();
        let newest = read_timeline_page(&pool, &thread_id, None, 1)
            .await
            .unwrap();
        let cursor = parse_timeline_cursor(newest.previous_cursor.as_deref().unwrap()).unwrap();
        let older = read_timeline_page(&pool, &thread_id, Some(&cursor), 1)
            .await
            .unwrap();

        let ids = [
            newest.items[0].activity_id.as_str(),
            older.items[0].activity_id.as_str(),
        ]
        .into_iter()
        .collect::<HashSet<_>>();
        assert_eq!(ids.len(), 2);
        assert!(older.previous_cursor.is_none());
    });
}

#[test]
fn durable_draft_preserves_unknown_json_and_attachment_references() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let vault_root = std::env::temp_dir().join(format!(
            "ganbaru-chat-draft-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&vault_root).unwrap();
        let source = vault_root.join("source.txt");
        fs::write(&source, b"explicit context").unwrap();
        let attachment_id = ChatAttachmentId::new("attachment-draft").unwrap();
        import_attachment(
            &pool,
            &vault_root,
            &ChatWorkspaceId::new("workspace-1").unwrap(),
            attachment_id.clone(),
            &source,
            ChatAttachmentKind::TextSnippet,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        let draft = ChatDraftWrite {
            id: "draft-1".to_string(),
            workspace_id: ChatWorkspaceId::new("workspace-1").unwrap(),
            thread_id: Some(ChatThreadId::new("thread-1").unwrap()),
            text: "Keep this".to_string(),
            attachment_ids: vec![attachment_id],
            mentions: VersionedJson {
                schema_version: 88,
                value: serde_json::json!([{ "futureMention": true }]),
            },
            provider_instance_id: Some(ProviderInstanceId::new("codex-personal").unwrap()),
            model_selection: Some(VersionedJson {
                schema_version: 91,
                value: serde_json::json!({ "futureModel": "x" }),
            }),
            safety_mode: Some(SafetyMode::Supervised),
            interaction_mode: Some(InteractionMode::Build),
            sent_snapshot: None,
            updated_at: UtcTimestamp::new(NOW).unwrap(),
        };
        assert_eq!(save_draft(&pool, &draft).await.unwrap(), draft);
        assert_eq!(read_draft(&pool, "draft-1").await.unwrap(), Some(draft));
        delete_draft(&pool, "draft-1", &UtcTimestamp::new(NOW).unwrap())
            .await
            .unwrap();
        let unreferenced: String = sqlx::query_scalar(
            "SELECT unreferenced_at FROM chat_attachments WHERE id = 'attachment-draft'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unreferenced, NOW);
        fs::remove_dir_all(vault_root).unwrap();
    });
}

#[test]
fn canonical_events_rebuild_equivalent_projections() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let modes = TurnModeSnapshot {
            safety_mode: SafetyMode::Supervised,
            interaction_mode: InteractionMode::Build,
        };
        append_canonical_event(
            &pool,
            canonical_request(
                "event-turn-start",
                Some("turn-1"),
                CanonicalEvent::TurnStarted(TurnStartedEvent {
                    provider_turn_id: None,
                    state: ChatTurnState::Active,
                    modes,
                    model_id: None,
                    model_options: Vec::new(),
                }),
            ),
        )
        .await
        .unwrap();
        let mut message = content_event("event-message", "Rebuild me");
        message.runtime.turn_id = Some(ChatTurnId::new("turn-1").unwrap());
        append_canonical_event(&pool, message).await.unwrap();
        append_canonical_event(
            &pool,
            canonical_request(
                "event-activity",
                Some("turn-1"),
                CanonicalEvent::ItemStarted(ItemLifecycleEvent {
                    item_id: "activity-1".to_string(),
                    kind: CanonicalItemKind::CommandExecution,
                    status: ActivityStatus::Active,
                    title: Some("Compile".to_string()),
                    detail: None,
                    safe_metadata: Some(VersionedJson {
                        schema_version: 12,
                        value: serde_json::json!({ "future": true }),
                    }),
                }),
            ),
        )
        .await
        .unwrap();
        append_canonical_event(
            &pool,
            canonical_request(
                "event-turn-complete",
                Some("turn-1"),
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
        append_canonical_event(
            &pool,
            canonical_request(
                "event-request",
                Some("turn-1"),
                CanonicalEvent::RequestOpened(RequestOpenedEvent {
                    request_id: ProviderRequestId::new("request-1").unwrap(),
                    kind: CanonicalRequestKind::CommandExecution,
                    title: "Approve".to_string(),
                    detail: None,
                    allowed_decisions: Vec::new(),
                    safe_payload: VersionedJson {
                        schema_version: 44,
                        value: serde_json::json!({ "future": "preserved" }),
                    },
                }),
            ),
        )
        .await
        .unwrap();
        let before: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT 'message', id, normalized_markdown FROM chat_messages
             UNION ALL SELECT 'activity', id, safe_metadata_data FROM chat_activities
             UNION ALL SELECT 'turn', id, state FROM chat_turns
             UNION ALL SELECT 'request', id, safe_display_data FROM chat_pending_requests
             ORDER BY 1, 2",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            rebuild_thread_projections(&pool, &ChatThreadId::new("thread-1").unwrap())
                .await
                .unwrap(),
            5
        );
        let after: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT 'message', id, normalized_markdown FROM chat_messages
             UNION ALL SELECT 'activity', id, safe_metadata_data FROM chat_activities
             UNION ALL SELECT 'turn', id, state FROM chat_turns
             UNION ALL SELECT 'request', id, safe_display_data FROM chat_pending_requests
             ORDER BY 1, 2",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(after, before);
    });
}

#[test]
fn attachment_cleanup_retries_and_never_removes_referenced_files() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let vault_root = std::env::temp_dir().join(format!(
            "ganbaru-chat-cleanup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&vault_root).unwrap();
        let source = vault_root.join("source.txt");
        fs::write(&source, b"retained context").unwrap();
        let attachment_id = ChatAttachmentId::new("attachment-cleanup").unwrap();
        let imported = import_attachment(
            &pool,
            &vault_root,
            &ChatWorkspaceId::new("workspace-1").unwrap(),
            attachment_id.clone(),
            &source,
            ChatAttachmentKind::TextSnippet,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_cleanup_queue
                (id, cleanup_kind, exact_target, state, not_before, created_at, updated_at)
             VALUES ('cleanup-retained', 'attachment_file', ?, 'failed', ?, ?, ?)",
        )
        .bind(&imported.managed_relative_path)
        .bind(NOW)
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_drafts
                (id, workspace_id, text, mentions_data, updated_at)
             VALUES ('draft-retained', 'workspace-1', '', '[]', ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_attachment_references
                (id, attachment_id, draft_id, created_at)
             VALUES ('reference-retained', ?, 'draft-retained', ?)",
        )
        .bind(attachment_id.as_str())
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(
            run_due_attachment_cleanup(&pool, &vault_root, &UtcTimestamp::new(NOW).unwrap())
                .await
                .unwrap(),
            1
        );
        let managed_path = vault_root.join(&imported.managed_relative_path);
        assert!(
            managed_path.exists(),
            "referenced attachment files must be retained"
        );
        let state: String = sqlx::query_scalar(
            "SELECT deletion_state FROM chat_attachments WHERE id = 'attachment-cleanup'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(state, "active", "referenced metadata must remain active");
        fs::remove_dir_all(vault_root).unwrap();
    });
}

#[test]
fn logical_workspace_mutations_are_sqlite_backed_and_revision_checked() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let request = CreateChatWorkspaceRequest {
            id: ChatWorkspaceId::new("workspace-2").unwrap(),
            project_id: None,
            display_name: "Second workspace".to_string(),
        };
        let created = create_workspace(&pool, &request, &UtcTimestamp::new(NOW).unwrap())
            .await
            .unwrap();
        assert_eq!(created.revision, 1);
        let renamed = rename_workspace(
            &pool,
            &request.id,
            "Persistent workspace",
            1,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(renamed.revision, 2);
        assert_eq!(list_workspaces(&pool).await.unwrap().len(), 2);
        assert!(rename_workspace(
            &pool,
            &request.id,
            "Stale rename",
            1,
            &UtcTimestamp::new(NOW).unwrap(),
        )
        .await
        .is_err());
    });
}

#[test]
fn project_archive_preserves_chat_and_deletion_requires_detach_or_delete() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        sqlx::query("INSERT INTO project_groups (id, name) VALUES ('group-chat', 'Chat')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO projects (id, group_id, name) VALUES ('project-chat', 'group-chat', 'Chat')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE chat_workspaces SET project_id = 'project-chat' WHERE id = 'workspace-1'",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("UPDATE chat_threads SET project_id = 'project-chat' WHERE id = 'thread-1'")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            set_project_chat_archived(
                &pool,
                "project-chat",
                true,
                &UtcTimestamp::new(NOW).unwrap()
            )
            .await
            .unwrap(),
            1
        );
        let thread_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_threads WHERE project_id = 'project-chat'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(thread_count, 1);
        assert!(
            sqlx::query("DELETE FROM projects WHERE id = 'project-chat'")
                .execute(&pool)
                .await
                .is_err()
        );
        assert_eq!(
            resolve_project_deletion(
                &pool,
                "project-chat",
                LinkedChatDeletionDecision::Detach,
                &UtcTimestamp::new(NOW).unwrap(),
                &UtcTimestamp::new(NOW).unwrap(),
            )
            .await
            .unwrap(),
            1
        );
        let detached: Option<String> =
            sqlx::query_scalar("SELECT project_id FROM chat_threads WHERE id = 'thread-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(detached, None);
        sqlx::query("DELETE FROM projects WHERE id = 'project-chat'")
            .execute(&pool)
            .await
            .unwrap();
    });
}

#[test]
fn ingestion_batches_adjacent_deltas_and_notifies_only_after_commit() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let emitter = Arc::new(RecordingEmitter::default());
        let mut ingestor = ChatEventIngestor::new(pool.clone(), emitter.clone());
        ingestor
            .ingest(content_event("event-batch-1", "Hello "))
            .await
            .unwrap();
        ingestor
            .ingest(content_event("event-batch-2", "world"))
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_events")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0
        );
        assert!(emitter.0.lock().unwrap().is_empty());
        ingestor.flush().await.unwrap();
        let text: String = sqlx::query_scalar(
            "SELECT normalized_markdown FROM chat_messages WHERE id = 'assistant-message-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(text, "Hello world");
        let notifications = emitter.0.lock().unwrap();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].sequence, 1);
    });
}

#[test]
fn ingestion_splits_delta_batches_at_the_memory_bound() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let emitter = Arc::new(RecordingEmitter::default());
        let mut ingestor = ChatEventIngestor::new(pool.clone(), emitter.clone());
        let first = "a".repeat(40 * 1024);
        let second = "b".repeat(40 * 1024);
        ingestor
            .ingest(content_event("event-bounded-1", &first))
            .await
            .unwrap();
        ingestor
            .ingest(content_event("event-bounded-2", &second))
            .await
            .unwrap();
        ingestor.flush().await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_events")
                .fetch_one(&pool)
                .await
                .unwrap(),
            2
        );
        let text: String = sqlx::query_scalar(
            "SELECT normalized_markdown FROM chat_messages WHERE id = 'assistant-message-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(text, format!("{first}{second}"));
        let preview: String =
            sqlx::query_scalar("SELECT latest_preview FROM chat_threads WHERE id = 'thread-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(preview, "b".repeat(2000));
        assert_eq!(emitter.0.lock().unwrap().len(), 2);
    });
}

#[test]
fn ingestion_retains_a_pending_batch_after_append_failure() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let emitter = Arc::new(RecordingEmitter::default());
        let mut ingestor = ChatEventIngestor::new(pool.clone(), emitter.clone());
        ingestor
            .ingest(content_event("event-retry", "retained"))
            .await
            .unwrap();
        sqlx::query(
            "CREATE TRIGGER reject_chat_event
             BEFORE INSERT ON chat_events
             BEGIN SELECT RAISE(ABORT, 'fixture rejection'); END",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(ingestor.flush().await.is_err());
        assert!(emitter.0.lock().unwrap().is_empty());
        sqlx::query("DROP TRIGGER reject_chat_event")
            .execute(&pool)
            .await
            .unwrap();
        ingestor.flush().await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chat_events")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
        assert_eq!(emitter.0.lock().unwrap().len(), 1);
    });
}

#[test]
fn ingestion_rejects_diagnostics_that_are_not_redacted() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        let emitter = Arc::new(RecordingEmitter::default());
        let mut ingestor = ChatEventIngestor::new(pool, emitter);
        let mut request = content_event("event-secret", "ignored");
        request.runtime.redacted_diagnostic = Some(VersionedJson {
            schema_version: 1,
            value: serde_json::json!({ "authorization": "Bearer private" }),
        });
        request.diagnostic_expires_at = Some(UtcTimestamp::new("2026-07-21T12:00:00Z").unwrap());
        assert!(ingestor.ingest(request).await.is_err());

        let mut request = content_event("event-home-path", "ignored");
        request.runtime.redacted_diagnostic = Some(VersionedJson {
            schema_version: 1,
            value: serde_json::json!({ "detail": "c:\\users\\person\\secret.txt" }),
        });
        request.diagnostic_expires_at = Some(UtcTimestamp::new("2026-07-21T12:00:00Z").unwrap());
        assert!(ingestor.ingest(request).await.is_err());
    });
}

#[test]
fn startup_recovery_interrupts_only_turns_without_proven_resumability() {
    tauri::async_runtime::block_on(async {
        let pool = pool_with_thread().await;
        append_canonical_event(
            &pool,
            canonical_request(
                "event-orphan-start",
                Some("turn-orphan"),
                CanonicalEvent::TurnStarted(TurnStartedEvent {
                    provider_turn_id: None,
                    state: ChatTurnState::Active,
                    modes: TurnModeSnapshot {
                        safety_mode: SafetyMode::Supervised,
                        interaction_mode: InteractionMode::Build,
                    },
                    model_id: None,
                    model_options: Vec::new(),
                }),
            ),
        )
        .await
        .unwrap();
        let mut resumable = HashSet::new();
        resumable.insert(ChatThreadId::new("thread-1").unwrap());
        assert_eq!(
            recover_orphaned_turns(&pool, &resumable, &UtcTimestamp::new(NOW).unwrap())
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            recover_orphaned_turns(&pool, &HashSet::new(), &UtcTimestamp::new(NOW).unwrap())
                .await
                .unwrap(),
            1
        );
        let state: String =
            sqlx::query_scalar("SELECT state FROM chat_turns WHERE id = 'turn-orphan'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(state, "interrupted");
        let event_type: String =
            sqlx::query_scalar("SELECT event_type FROM chat_events ORDER BY sequence DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(event_type, "turn_aborted");
    });
}
