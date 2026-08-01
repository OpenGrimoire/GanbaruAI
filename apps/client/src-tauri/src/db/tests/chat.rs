use super::helpers::migrated_memory_pool;
use sqlx::Row;

const NOW: &str = "2026-07-20T12:00:00Z";
const ADD_CHAT_DRAFT_RICH_CONTENT: &str =
    include_str!("../../../migrations/20260728032141_add_chat_draft_rich_content.sql");
const ADD_CHAT_REVIEW_COMMENT_SOURCES: &str =
    include_str!("../../../migrations/20260730193000_add_chat_review_comment_sources.sql");

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

async fn insert_working_folder(pool: &sqlx::SqlitePool, id: &str, project_id: &str) {
    sqlx::query(
        "INSERT INTO project_working_folders
            (id, project_id, display_name, kind, repository_kind, created_at, updated_at)
         VALUES (?, ?, ?, 'external', 'none', ?, ?)",
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
    working_folder_id: &str,
    project_id: &str,
) {
    sqlx::query(
        "INSERT INTO chat_threads
            (id, working_folder_id, project_id, title, provider_family_id,
             provider_instance_id, continuation_group_id, safety_mode,
             interaction_mode, state, last_activity_at, created_at, updated_at)
         VALUES (?, ?, ?, 'Implement Chat', 'codex', 'codex-personal',
                 'continuation-1', 'ask_for_approval', 'build', 'idle', ?, ?, ?)",
    )
    .bind(id)
    .bind(working_folder_id)
    .bind(project_id)
    .bind(NOW)
    .bind(NOW)
    .bind(NOW)
    .execute(pool)
    .await
    .unwrap();
}

#[test]
fn built_in_projects_have_one_protected_managed_working_folder() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(&pool)
            .await
            .unwrap();
        let managed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM project_working_folders WHERE kind = 'managed' AND archived_at IS NULL",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(managed, projects);
        let invalid_paths: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM project_working_folders
             WHERE kind = 'managed' AND managed_relative_path != ('projects/' || project_id)",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(invalid_paths, 0);
        let archive = sqlx::query(
            "UPDATE project_working_folders SET archived_at = ? WHERE id = 'working-folder-routine-learning'",
        )
        .bind(NOW)
        .execute(&pool)
        .await;
        assert!(archive.is_err());
        let delete = sqlx::query(
            "DELETE FROM project_working_folders WHERE id = 'working-folder-routine-learning'",
        )
        .execute(&pool)
        .await;
        assert!(delete.is_err());
    });
}

#[test]
fn schema_creates_chat_tables_indexes_and_no_device_paths() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for object in [
            "project_working_folders",
            "chat_threads",
            "chat_turns",
            "chat_messages",
            "chat_activities",
            "chat_pending_requests",
            "chat_plans",
            "chat_drafts",
            "chat_attachments",
            "chat_attachment_references",
            "chat_queued_followups",
            "chat_queued_attachment_references",
            "chat_user_input_drafts",
            "chat_events",
            "chat_command_receipts",
            "chat_checkpoints",
            "chat_cleanup_queue",
            "chat_checkpoint_failures",
            "chat_restore_previews",
            "chat_restore_operations",
            "chat_terminal_attachment_contexts",
            "chat_execution_environments",
            "chat_thread_relations",
            "chat_worktrees",
            "chat_review_comments",
            "chat_resources",
            "chat_resource_thread_references",
            "chat_preview_tabs",
            "chat_browser_artifacts",
            "chat_provider_cleanup_jobs",
            "chat_terminal_layouts",
            "chat_source_control_state",
            "chat_channels",
            "chat_channel_sessions",
            "idx_chat_threads_active_project",
            "idx_chat_threads_active_working_folder",
            "idx_chat_threads_archived",
            "idx_chat_threads_title_search",
            "idx_chat_events_thread_sequence",
            "idx_chat_messages_thread_sequence",
            "idx_chat_activities_thread_sequence",
            "idx_chat_pending_requests_unresolved",
            "idx_chat_turns_thread_ordinal",
            "idx_chat_checkpoints_thread_turn",
            "idx_chat_attachment_references_message",
            "idx_chat_queued_followups_active",
            "idx_chat_queued_attachment_references_attachment",
            "idx_chat_checkpoint_failures_thread",
            "idx_chat_restore_previews_thread",
            "idx_chat_restore_operations_thread",
            "idx_chat_events_valid_thread_sequence",
            "idx_chat_threads_execution_environment",
            "idx_chat_worktrees_cleanup",
            "idx_chat_review_comments_thread_path",
            "idx_chat_review_comments_thread_queue",
            "idx_chat_resources_workspace_kind",
            "idx_chat_browser_artifacts_thread",
            "idx_chat_provider_cleanup_jobs_retry",
            "idx_chat_channels_project_name",
            "idx_chat_channels_project_default",
            "idx_chat_channels_active_project",
            "idx_chat_channel_sessions_current",
            "idx_chat_channel_sessions_thread",
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
            "project_working_folders",
            "chat_threads",
            "chat_turns",
            "chat_messages",
            "chat_events",
            "chat_execution_environments",
            "chat_worktrees",
            "chat_resources",
            "chat_browser_artifacts",
        ] {
            let columns = sqlx::query(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .fetch_all(&pool)
                .await
                .unwrap()
                .into_iter()
                .map(|row| row.get::<String, _>("name"))
                .collect::<Vec<_>>();
            assert!(!columns.iter().any(|column| {
                column.contains("absolute_path")
                    || column.contains("canonical_path")
                    || column.contains("credential")
                    || column.contains("secret")
            }));
        }
    });
}

#[test]
fn chat_workspace_schema_preserves_attachments_and_scopes_resources_to_threads() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project(&pool).await;
        insert_working_folder(&pool, "folder-1", "project-1").await;
        insert_thread(&pool, "thread-1", "folder-1", "project-1").await;
        sqlx::query(
            "INSERT INTO chat_messages
                (id, thread_id, sequence_anchor, role, normalized_markdown, streaming_state, created_at, updated_at)
             VALUES ('message-1', 'thread-1', 0, 'user', 'Inspect this image', 'complete', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_attachments
                (id, working_folder_id, kind, original_display_name, mime_type, byte_size,
                 sha256, managed_relative_path, signature_kind, created_at)
             VALUES ('resource-1', 'folder-1', 'image', 'screen.png', 'image/png', 8,
                     'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                     'assets/chat/attachments/resource-1.png', 'png', ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_attachment_references
                (id, attachment_id, message_id, created_at)
             VALUES ('reference-1', 'resource-1', 'message-1', ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let resource: (String, String, String) = sqlx::query_as(
            "SELECT resource_uri, managed_relative_path, integrity_state
             FROM chat_resources WHERE id = 'resource-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(resource.0, "ganbaru://chat/resource/resource-1");
        assert_eq!(resource.1, "assets/chat/attachments/resource-1.png");
        assert_eq!(resource.2, "verified");
        let visible: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_resource_thread_references
             WHERE resource_id = 'resource-1' AND thread_id = 'thread-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(visible, 1);

        sqlx::query("DELETE FROM chat_threads WHERE id = 'thread-1'")
            .execute(&pool)
            .await
            .unwrap();
        let resource_remains: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chat_resources WHERE id = 'resource-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let references_remain: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM chat_resource_thread_references WHERE resource_id = 'resource-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(resource_remains, 1);
        assert_eq!(references_remain, 0);
    });
}

#[test]
fn chat_workspace_schema_records_forks_worktrees_reviews_and_cleanup_failures() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project(&pool).await;
        insert_working_folder(&pool, "folder-1", "project-1").await;
        insert_thread(&pool, "thread-parent", "folder-1", "project-1").await;
        insert_thread(&pool, "thread-child", "folder-1", "project-1").await;
        sqlx::query(
            "INSERT INTO chat_thread_relations
                (child_thread_id, parent_thread_id, relation_kind, created_at)
             VALUES ('thread-child', 'thread-parent', 'fork', ?)",
        )
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_execution_environments
                (id, working_folder_id, kind, display_name, lifecycle_state, created_at, updated_at)
             VALUES ('worktree-1', 'folder-1', 'worktree', 'Feature worktree', 'cleanup_failed', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_worktrees
                (execution_environment_id, branch_name, base_reference, cleanup_state,
                 cleanup_error_code, cleanup_error_detail, created_at, updated_at)
             VALUES ('worktree-1', 'feat/workspace', 'origin/dev', 'failed',
                     'dirty_worktree', 'Worktree has local changes', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_review_comments
                (id, thread_id, relative_path, content_revision, start_line, end_line,
                 selected_text, comment_text, created_at, updated_at)
             VALUES ('review-1', 'thread-child', 'src/main.rs',
                     'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                     4, 5, 'unsafe block', 'Can this stay safe?', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        let environment: String = sqlx::query_scalar(
            "SELECT execution_environment_id FROM chat_threads WHERE id = 'thread-child'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let cleanup: (String, String) = sqlx::query_as(
            "SELECT cleanup_state, cleanup_error_code FROM chat_worktrees WHERE execution_environment_id = 'worktree-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(environment, "current-folder:folder-1");
        assert_eq!(
            cleanup,
            ("failed".to_string(), "dirty_worktree".to_string())
        );
    });
}

#[test]
fn chat_draft_rich_content_migration_preserves_existing_plain_text() {
    tauri::async_runtime::block_on(async {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE chat_drafts (
                id TEXT PRIMARY KEY NOT NULL,
                text TEXT NOT NULL DEFAULT ''
            ) STRICT",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO chat_drafts (id, text) VALUES ('draft-1', 'Keep this')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::raw_sql(ADD_CHAT_DRAFT_RICH_CONTENT)
            .execute(&pool)
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT text, rich_content_schema_version, rich_content_data
             FROM chat_drafts WHERE id = 'draft-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<String, _>("text"), "Keep this");
        assert_eq!(
            row.get::<Option<i64>, _>("rich_content_schema_version"),
            None
        );
        assert_eq!(row.get::<Option<String>, _>("rich_content_data"), None);
    });
}

#[test]
fn chat_review_comment_sources_migration_preserves_legacy_rows_and_enforces_constraints() {
    tauri::async_runtime::block_on(async {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE chat_review_comments (
                id TEXT PRIMARY KEY NOT NULL,
                thread_id TEXT NOT NULL,
                turn_id TEXT,
                relative_path TEXT NOT NULL,
                content_revision TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                start_column INTEGER NOT NULL DEFAULT 1,
                end_line INTEGER NOT NULL,
                end_column INTEGER NOT NULL DEFAULT 1,
                selected_text TEXT NOT NULL DEFAULT '',
                comment_text TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'open',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                resolved_at TEXT
            ) STRICT",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO chat_review_comments
                (id, thread_id, relative_path, content_revision, start_line, end_line,
                 selected_text, comment_text, created_at, updated_at)
             VALUES ('review-legacy', 'thread-1', 'src/main.rs',
                     'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
                     4, 5, 'unsafe block', 'Can this stay safe?', ?, ?)",
        )
        .bind(NOW)
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::raw_sql(ADD_CHAT_REVIEW_COMMENT_SOURCES)
            .execute(&pool)
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT selected_text, comment_text, source_kind, source_data,
                    review_revision, snapshot_id, file_id, selection_side,
                    previous_relative_path, applicability, queued_for_send
             FROM chat_review_comments WHERE id = 'review-legacy'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<String, _>("selected_text"), "unsafe block");
        assert_eq!(row.get::<String, _>("comment_text"), "Can this stay safe?");
        assert_eq!(row.get::<String, _>("source_kind"), "file");
        assert_eq!(row.get::<Option<String>, _>("source_data"), None);
        assert_eq!(row.get::<Option<String>, _>("review_revision"), None);
        assert_eq!(row.get::<Option<String>, _>("snapshot_id"), None);
        assert_eq!(row.get::<Option<String>, _>("file_id"), None);
        assert_eq!(row.get::<String, _>("selection_side"), "file");
        assert_eq!(row.get::<Option<String>, _>("previous_relative_path"), None);
        assert_eq!(row.get::<String, _>("applicability"), "current");
        assert_eq!(row.get::<i64, _>("queued_for_send"), 0);

        for invalid_update in [
            "UPDATE chat_review_comments SET source_kind = 'unsupported' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET source_data = 'not-json' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET review_revision = 'short' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET snapshot_id = '' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET file_id = '' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET selection_side = 'both' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET previous_relative_path = '../secret' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET applicability = 'unknown' WHERE id = 'review-legacy'",
            "UPDATE chat_review_comments SET queued_for_send = 2 WHERE id = 'review-legacy'",
        ] {
            assert!(
                sqlx::query(invalid_update).execute(&pool).await.is_err(),
                "constraint should reject: {invalid_update}"
            );
        }

        let queue_index: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_schema
             WHERE type = 'index' AND name = 'idx_chat_review_comments_thread_queue'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(queue_index, Some(1));
    });
}

#[test]
fn chat_permission_columns_accept_only_current_modes() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for table in ["chat_threads", "chat_turns", "chat_drafts"] {
            let sql: String = sqlx::query_scalar(
                "SELECT sql FROM sqlite_schema WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .unwrap();
            for mode in [
                "ask_for_approval",
                "approve_for_me",
                "full_access",
                "custom",
            ] {
                assert!(sql.contains(mode), "{table} should accept {mode}");
            }
            assert!(
                !sql.contains("supervised"),
                "{table} should reject legacy modes"
            );
            assert!(
                !sql.contains("auto_accept_edits"),
                "{table} should reject legacy modes"
            );
        }
    });
}

#[test]
fn working_folder_tool_schema_tracks_restore_invalidation_and_exact_cleanup() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for (table, expected) in [
            (
                "chat_checkpoints",
                vec![
                    "checkpoint_kind",
                    "turn_id",
                    "index_commit_oid",
                    "index_tree_oid",
                    "worktree_tree_oid",
                    "head_oid",
                    "head_ref",
                    "index_fingerprint",
                    "invalidated_at",
                    "invalidated_by_checkpoint_id",
                ],
            ),
            ("chat_turns", vec!["invalidated_at", "invalidation_reason"]),
            ("chat_events", vec!["invalidated_at", "invalidation_reason"]),
            (
                "chat_cleanup_queue",
                vec!["working_folder_id", "expected_object_id"],
            ),
        ] {
            let columns = sqlx::query(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .fetch_all(&pool)
                .await
                .unwrap()
                .into_iter()
                .map(|row| row.get::<String, _>("name"))
                .collect::<Vec<_>>();
            for column in expected {
                assert!(
                    columns.iter().any(|value| value == column),
                    "{table}.{column} should exist"
                );
            }
        }
    });
}

#[test]
fn working_folder_project_identity_and_delete_policies_are_explicit() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project(&pool).await;
        insert_working_folder(&pool, "working-folder-project", "project-1").await;
        insert_thread(
            &pool,
            "thread-project",
            "working-folder-project",
            "project-1",
        )
        .await;

        let mismatched = sqlx::query(
            "INSERT INTO chat_threads
                (id, working_folder_id, project_id, title, provider_family_id,
                 provider_instance_id, continuation_group_id, safety_mode,
                 interaction_mode, state, last_activity_at, created_at, updated_at)
             VALUES ('thread-bad', 'working-folder-project', NULL, 'Bad', 'codex',
                     'codex-personal', 'continuation-1', 'ask_for_approval', 'build',
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
        let folder_delete =
            sqlx::query("DELETE FROM project_working_folders WHERE id = 'working-folder-project'")
                .execute(&pool)
                .await;
        assert!(folder_delete.is_err());
    });
}

#[test]
fn event_sequences_and_unresolved_requests_are_unique() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project(&pool).await;
        insert_working_folder(&pool, "working-folder-1", "project-1").await;
        insert_thread(&pool, "thread-1", "working-folder-1", "project-1").await;

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
        insert_project(&pool).await;
        insert_working_folder(&pool, "working-folder-1", "project-1").await;
        insert_thread(&pool, "thread-1", "working-folder-1", "project-1").await;
        sqlx::query(
            "INSERT INTO chat_cleanup_queue
                (id, working_folder_id, source_thread_id, cleanup_kind, exact_target,
                 repository_identity, not_before, created_at, updated_at)
             VALUES ('cleanup-1', 'working-folder-1', 'thread-1', 'checkpoint_ref',
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
