use super::helpers::migrated_memory_pool;
use sqlx::Row;

const NOW: &str = "2026-07-20T12:00:00Z";

async fn insert_project(pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO project_groups (id, name) VALUES ('group-1', 'Engineering')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO projects (id, group_id, name) VALUES ('project-1', 'group-1', 'Ganbaru')",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_workspace(pool: &sqlx::SqlitePool, id: &str, project_id: Option<&str>) {
    sqlx::query(
        "INSERT INTO chat_workspaces
            (id, project_id, display_name, repository_kind, created_at, updated_at)
         VALUES (?, ?, ?, 'none', ?, ?)",
    )
    .bind(id)
    .bind(project_id)
    .bind(format!("Workspace {id}"))
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_thread(
    pool: &sqlx::SqlitePool,
    id: &str,
    workspace_id: &str,
    project_id: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO chat_threads
            (id, workspace_id, project_id, title, provider_family_id,
             provider_instance_id, continuation_group_id, safety_mode,
             interaction_mode, state, last_activity_at, created_at, updated_at)
         VALUES (?, ?, ?, 'Implement Chat', 'codex', 'codex-personal',
                 'continuation-1', 'supervised', 'build', 'idle', ?, ?, ?)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(project_id)
    .bind(NOW)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .unwrap();
}

#[test]
fn schema_creates_chat_tables_indexes_and_no_device_paths() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for object in [
            "chat_workspaces",
            "chat_threads",
            "chat_turns",
            "chat_messages",
            "chat_activities",
            "chat_pending_requests",
            "chat_plans",
            "chat_drafts",
            "chat_attachments",
            "chat_attachment_references",
            "chat_events",
            "chat_command_receipts",
            "chat_checkpoints",
            "chat_cleanup_queue",
            "idx_chat_threads_active_project",
            "idx_chat_threads_archived",
            "idx_chat_threads_title_search",
            "idx_chat_events_thread_sequence",
            "idx_chat_messages_thread_sequence",
            "idx_chat_activities_thread_sequence",
            "idx_chat_pending_requests_unresolved",
            "idx_chat_turns_thread_ordinal",
            "idx_chat_checkpoints_thread_turn",
            "idx_chat_attachment_references_message",
        ] {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE name = ?")
                    .bind(object)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{object} should exist");
        }

        for table in [
            "chat_workspaces",
            "chat_threads",
            "chat_turns",
            "chat_messages",
            "chat_events",
        ] {
            let columns = sqlx::query(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .fetch_all(&pool)
                .await
                .unwrap()
                .into_iter()
                .map(|row| row.get::<String, _>("name"))
                .collect::<Vec<_>>();
            assert!(!columns.iter().any(|column| {
                column.contains("path")
                    || column.contains("folder")
                    || column.contains("directory")
                    || column.contains("credential")
                    || column.contains("secret")
            }));
        }
    });
}

#[test]
fn workspace_project_identity_and_delete_policies_are_explicit() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project(&pool).await;
        insert_workspace(&pool, "workspace-project", Some("project-1")).await;
        insert_workspace(&pool, "workspace-standalone", None).await;
        insert_thread(
            &pool,
            "thread-project",
            "workspace-project",
            Some("project-1"),
        )
        .await;
        insert_thread(&pool, "thread-standalone", "workspace-standalone", None).await;

        let mismatched = sqlx::query(
            "INSERT INTO chat_threads
                (id, workspace_id, project_id, title, provider_family_id,
                 provider_instance_id, continuation_group_id, safety_mode,
                 interaction_mode, state, last_activity_at, created_at, updated_at)
             VALUES ('thread-bad', 'workspace-project', NULL, 'Bad', 'codex',
                     'codex-personal', 'continuation-1', 'supervised', 'build',
                     'idle', ?, ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await;
        assert!(mismatched.is_err());

        let project_delete = sqlx::query("DELETE FROM projects WHERE id = 'project-1'")
            .execute(&pool)
            .await;
        assert!(project_delete.is_err());
        let workspace_delete =
            sqlx::query("DELETE FROM chat_workspaces WHERE id = 'workspace-standalone'")
                .execute(&pool)
                .await;
        assert!(workspace_delete.is_err());
    });
}

#[test]
fn event_sequences_and_unresolved_requests_are_unique() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_workspace(&pool, "workspace-1", None).await;
        insert_thread(&pool, "thread-1", "workspace-1", None).await;

        let skipped_sequence = sqlx::query(
            "INSERT INTO chat_events
                (id, thread_id, sequence, provider_family_id, provider_instance_id,
                 event_type, payload_schema_version, payload_data, created_at, ingested_at)
             VALUES ('event-2', 'thread-1', 2, 'codex', 'codex-personal',
                     'turn_started', 1, '{}', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await;
        assert!(skipped_sequence.is_err());

        sqlx::query(
            "INSERT INTO chat_events
                (id, thread_id, sequence, provider_family_id, provider_instance_id,
                 event_type, payload_schema_version, payload_data, created_at, ingested_at)
             VALUES ('event-1', 'thread-1', 1, 'codex', 'codex-personal',
                     'approval_requested', 999, '{\"future\":true}', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE chat_threads
             SET last_event_sequence = 1, last_projected_sequence = 1
             WHERE id = 'thread-1'",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO chat_pending_requests
                (id, thread_id, provider_request_id, request_kind,
                 safe_display_data, allowed_decisions_data, opened_sequence, opened_at)
             VALUES ('request-1', 'thread-1', 'provider-request-1', 'approval',
                     '{}', '[\"deny\"]', 1, ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        let duplicate_open = sqlx::query(
            "INSERT INTO chat_pending_requests
                (id, thread_id, provider_request_id, request_kind,
                 safe_display_data, allowed_decisions_data, opened_sequence, opened_at)
             VALUES ('request-2', 'thread-1', 'provider-request-1', 'approval',
                     '{}', '[\"deny\"]', 1, ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await;
        assert!(duplicate_open.is_err());

        let payload: String =
            sqlx::query_scalar("SELECT payload_data FROM chat_events WHERE id = 'event-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(payload, "{\"future\":true}");
    });
}

#[test]
fn cleanup_queue_survives_thread_deletion_and_requires_exact_refs() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_workspace(&pool, "workspace-1", None).await;
        insert_thread(&pool, "thread-1", "workspace-1", None).await;
        sqlx::query(
            "INSERT INTO chat_cleanup_queue
                (id, source_thread_id, cleanup_kind, exact_target,
                 repository_identity, not_before, created_at, updated_at)
             VALUES ('cleanup-1', 'thread-1', 'checkpoint_ref',
                     'refs/ganbaru-ai/chat/thread-1/checkpoint-1',
                     'git-sha256:repository', ?, ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("DELETE FROM chat_threads WHERE id = 'thread-1'")
            .execute(&pool)
            .await
            .unwrap();

        let target: String = sqlx::query_scalar(
            "SELECT exact_target FROM chat_cleanup_queue WHERE id = 'cleanup-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(target, "refs/ganbaru-ai/chat/thread-1/checkpoint-1");
    });
}
