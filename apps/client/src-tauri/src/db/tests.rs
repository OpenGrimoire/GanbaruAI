use super::run_migrations;
use sqlx::{Row, SqlitePool};

async fn migrated_memory_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .unwrap();
    run_migrations(&pool).await.unwrap();
    pool
}

async fn insert_event(pool: &SqlitePool) {
    sqlx::query(
        "INSERT INTO calendar_events (id, title, start_time, end_time)
         VALUES ('event-1', 'Focus block', '2026-05-23T09:00:00Z', '2026-05-23T10:00:00Z')",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_open_run(
    pool: &SqlitePool,
    id: &str,
) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query(
        "INSERT INTO pomodoro_runs
            (id, event_id, original_event_id, event_date, planned_start, planned_end,
             started_at, rhythm_kind, rhythm_source, preset_key, last_heartbeat, start_trigger)
         VALUES (?, 'event-1', 'event-1', '2026-05-23',
                 '2026-05-23T09:00:00Z', '2026-05-23T10:00:00Z',
                 '2026-05-23T09:00:00Z', 'count', 'preset', 'adaptive',
                 '2026-05-23T09:00:00Z', 'manual')",
    )
    .bind(id)
    .execute(pool)
    .await
}

#[test]
fn schema_does_not_create_json_storage_columns() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        let rows = sqlx::query(
            "SELECT m.name AS table_name, p.name AS column_name
             FROM sqlite_schema AS m, pragma_table_info(m.name) AS p
             WHERE m.type = 'table'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let forbidden = [
            "pause_log",
            "raw_jcal",
            "skip_ranges_json",
            "break_source_json",
            "notifications",
            "exceptions",
            "categories",
            "geo",
            "rdate",
            "extended_properties",
            "organizer",
        ];
        let forbidden_tables = ["pomodoro_sessions"];
        for row in rows {
            let table_name: String = row.try_get("table_name").unwrap();
            let column_name: String = row.try_get("column_name").unwrap();
            assert!(
                !forbidden_tables.contains(&table_name.as_str()),
                "{table_name} should not be created as persisted storage",
            );
            assert!(
                !forbidden.contains(&column_name.as_str()) && !column_name.ends_with("_json"),
                "{table_name}.{column_name} should be normalized, not JSON storage",
            );
        }
    });
}

#[test]
fn schema_creates_normalized_calendar_archive_tables() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let archive_tables = [
            "calendar_events_archive",
            "calendar_event_archive_pomodoro_configs",
            "calendar_event_archive_pomodoro_config_count_rhythms",
            "calendar_event_archive_pomodoro_config_sequence_steps",
            "calendar_event_archive_notifications",
            "calendar_event_archive_exdates",
            "calendar_event_archive_rdates",
            "calendar_event_archive_categories",
            "calendar_event_archive_extended_properties",
            "calendar_event_archive_organizers",
            "calendar_event_archive_attendees",
            "calendar_event_archive_alarms",
            "calendar_event_archive_overrides",
            "calendar_event_archive_override_extended_properties",
        ];
        for table in archive_tables {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                    .bind(table)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{table} should exist");
        }
    });
}

#[test]
fn schema_creates_normalized_notes_database_tables() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for table in [
            "notes_databases",
            "notes_data_sources",
            "notes_database_views",
            "notes_data_source_relation_links",
            "notes_data_source_rollup_cache",
            "notes_data_source_templates",
            "notes_data_source_template_blocks",
        ] {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                    .bind(table)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{table} should exist");
        }

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title, properties)
             VALUES ('page-a', 'workspace', 'Inbox', '{}')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'database-a',
                'page-a',
                'page_id',
                'page-a',
                'child_database',
                '{\"title\":\"Tasks\"}',
                'Tasks',
                1000
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_databases (
                id,
                parent_type,
                parent_page_id,
                title,
                title_rich_text,
                description
             )
             VALUES ('database-a', 'page_id', 'page-a', 'Tasks', '[]', '[]')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_data_sources (
                id,
                database_id,
                title,
                title_rich_text,
                description,
                properties
             )
             VALUES ('source-a', 'database-a', 'Tasks', '[]', '[]', '{}')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_database_views (
                id,
                database_id,
                data_source_id,
                name,
                type,
                sorts
             )
             VALUES ('view-a', 'database-a', 'source-a', 'Table', 'table', '[]')",
        )
        .execute(&pool)
        .await
        .unwrap();

        assert!(sqlx::query(
            "INSERT INTO notes_database_views (
                id,
                database_id,
                data_source_id,
                name,
                type,
                sorts
             )
             VALUES ('view-b', 'database-a', 'source-a', 'Bad', 'kanbanish', '[]')",
        )
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_enforces_notes_folder_placement_invariants() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let folder_table: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'notes_folders'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(folder_table, Some(1));
        let folder_column: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM pragma_table_info('notes_pages') WHERE name = 'folder_id'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(folder_column, Some(1));

        sqlx::query("INSERT INTO project_groups (id, name) VALUES ('folder-group', 'Folders')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name)
             VALUES ('folder-project-a', 'folder-group', 'A'),
                    ('folder-project-b', 'folder-group', 'B')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_folders (id, project_id, name)
             VALUES ('folder-a', 'folder-project-a', 'A'),
                    ('folder-b', 'folder-project-b', 'B')",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "INSERT INTO notes_folders (id, project_id, parent_folder_id, name)
             VALUES ('folder-cross', 'folder-project-a', 'folder-b', 'Cross project')",
        )
        .execute(&pool)
        .await
        .is_err());

        sqlx::query(
            "INSERT INTO notes_folders (id, project_id, parent_folder_id, name)
             VALUES ('folder-child', 'folder-project-a', 'folder-a', 'Child')",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "UPDATE notes_folders SET parent_folder_id = 'folder-child' WHERE id = 'folder-a'",
        )
        .execute(&pool)
        .await
        .is_err());

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, folder_id, title, properties)
             VALUES (
                 'folder-page',
                 'workspace',
                 'folder-a',
                 'Folder page',
                 json_object('__ganbaru_project_id', 'folder-project-a')
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, folder_id, title, properties)
             VALUES (
                 'wrong-project-page',
                 'workspace',
                 'folder-a',
                 'Wrong project',
                 json_object('__ganbaru_project_id', 'folder-project-b')
             )",
        )
        .execute(&pool)
        .await
        .is_err());
        assert!(sqlx::query(
            "INSERT INTO notes_pages (
                id, parent_type, parent_page_id, folder_id, title, properties
             ) VALUES (
                 'nested-folder-page',
                 'page_id',
                 'folder-page',
                 'folder-a',
                 'Invalid nested folder page',
                 json_object('__ganbaru_project_id', 'folder-project-a')
             )",
        )
        .execute(&pool)
        .await
        .is_err());
        assert!(sqlx::query(
            "UPDATE notes_pages
             SET properties = json_object('__ganbaru_project_id', 'folder-project-b')
             WHERE id = 'folder-page'",
        )
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_validates_project_notes_default_open_mode() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::query(
            "INSERT INTO project_groups (id, name, icon, sort_order)
             VALUES ('group-notes-mode', 'Notes mode', 'folder', 100)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name, icon, sort_order)
             VALUES ('project-notes-mode', 'group-notes-mode', 'Notes mode', 'folder', 100)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let default_mode: Option<String> = sqlx::query_scalar(
            "SELECT notes_default_open_mode FROM projects WHERE id = 'project-notes-mode'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(default_mode, None);

        sqlx::query(
            "UPDATE projects SET notes_default_open_mode = 'side' WHERE id = 'project-notes-mode'",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "UPDATE projects SET notes_default_open_mode = 'invalid' WHERE id = 'project-notes-mode'",
        )
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_creates_notes_local_user_identity() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let row = sqlx::query(
            "SELECT id, display_name
             FROM notes_local_users
             ORDER BY created_time ASC, id ASC
             LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let id: String = row.try_get("id").unwrap();
        let display_name: String = row.try_get("display_name").unwrap();
        assert_ne!(id, "local-user");
        assert_eq!(display_name, "You");

        let created_by_column: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             FROM pragma_table_info('notes_page_history_snapshots')
             WHERE name = 'created_by'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(created_by_column, Some(1));
    });
}

#[test]
fn schema_creates_notes_comment_thread_read_state() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for table in ["notes_comment_thread_reads", "notes_local_users"] {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                    .bind(table)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{table} should exist");
        }

        let user_id_fk: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             FROM pragma_foreign_key_list('notes_comment_thread_reads')
             WHERE \"table\" = 'notes_local_users'
               AND \"from\" = 'user_id'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(user_id_fk, Some(1));
    });
}

#[test]
fn schema_creates_notes_mention_notifications() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                .bind("notes_mention_notifications")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert_eq!(exists, Some(1));

        let page_fk: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             FROM pragma_foreign_key_list('notes_mention_notifications')
             WHERE \"table\" = 'notes_pages'
               AND \"from\" = 'page_id'",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert_eq!(page_fk, Some(1));

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title, properties)
             VALUES ('page-a', 'workspace', 'Inbox', '{}')",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "INSERT INTO notes_mention_notifications (
                id,
                source_type,
                source_id,
                page_id,
                kind,
                target_type,
                status,
                fingerprint
             )
             VALUES (
                'notification-a',
                'block',
                'block-a',
                'page-a',
                'reminder',
                'date',
                'stale',
                'fingerprint-a'
             )",
        )
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_creates_notes_suggestions() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                .bind("notes_suggestions")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert_eq!(exists, Some(1));

        for column in ["created_by", "accepted_by", "rejected_by"] {
            let user_fk: Option<i64> = sqlx::query_scalar(
                "SELECT 1
                 FROM pragma_foreign_key_list('notes_suggestions')
                 WHERE \"table\" = 'notes_local_users'
                   AND \"from\" = ?",
            )
            .bind(column)
            .fetch_optional(&pool)
            .await
            .unwrap();
            assert_eq!(
                user_fk,
                Some(1),
                "{column} should reference notes_local_users"
            );
        }

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title, properties)
             VALUES ('page-a', 'workspace', 'Inbox', '{}')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'block-a',
                'page-a',
                'page_id',
                'page-a',
                'paragraph',
                json_object('rich_text', json_array()),
                'Original',
                1
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        let user_id: String = sqlx::query_scalar(
            "SELECT id FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(sqlx::query(
            "INSERT INTO notes_suggestions (
                id,
                page_id,
                block_id,
                created_by,
                display_name,
                range_start,
                range_end,
                original_text,
                proposed_text,
                accepted_at,
                accepted_by
             )
             VALUES (
                'suggestion-a',
                'page-a',
                'block-a',
                ?,
                json_object('type', 'user', 'resolved_name', 'You'),
                0,
                8,
                'Original',
                'Changed',
                strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                ?
             )",
        )
        .bind(&user_id)
        .bind(&user_id)
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_creates_notes_collaboration_operations() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                .bind("notes_collaboration_operations")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert_eq!(exists, Some(1));

        for table in ["notes_local_users"] {
            let fk: Option<i64> = sqlx::query_scalar(
                "SELECT 1
                 FROM pragma_foreign_key_list('notes_collaboration_operations')
                 WHERE \"table\" = ?",
            )
            .bind(table)
            .fetch_optional(&pool)
            .await
            .unwrap();
            assert_eq!(fk, Some(1), "{table} should be referenced");
        }
        for table in ["notes_pages", "notes_blocks"] {
            let fk: Option<i64> = sqlx::query_scalar(
                "SELECT 1
                 FROM pragma_foreign_key_list('notes_collaboration_operations')
                 WHERE \"table\" = ?",
            )
            .bind(table)
            .fetch_optional(&pool)
            .await
            .unwrap();
            assert_eq!(fk, None, "{table} must not delete append-only operations");
        }

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title, properties)
             VALUES ('page-a', 'workspace', 'Inbox', '{}')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'block-a',
                'page-a',
                'page_id',
                'page-a',
                'paragraph',
                json_object('rich_text', json_array()),
                'Original',
                1
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        let user_id: String = sqlx::query_scalar(
            "SELECT id FROM notes_local_users ORDER BY created_time ASC, id ASC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_collaboration_operations (
                id,
                entity_type,
                entity_id,
                operation_type,
                page_id,
                block_id,
                actor_id,
                actor_display_name,
                base_version,
                entity_version,
                conflict_policy,
                payload
             )
             VALUES (
                'operation-a',
                'suggestion',
                'suggestion-a',
                'suggestion_create',
                'page-a',
                'block-a',
                ?,
                json_object('type', 'user', 'resolved_name', 'You'),
                0,
                1,
                'append_only',
                json_object('schema_version', 1)
             )",
        )
        .bind(&user_id)
        .execute(&pool)
        .await
        .unwrap();

        assert!(sqlx::query(
            "INSERT INTO notes_collaboration_operations (
                id,
                entity_type,
                entity_id,
                operation_type,
                page_id,
                actor_id,
                actor_display_name,
                base_version,
                entity_version,
                conflict_policy,
                payload
             )
             VALUES (
                'operation-b',
                'suggestion',
                'suggestion-b',
                'suggestion_accept',
                'page-a',
                ?,
                json_object('type', 'user', 'resolved_name', 'You'),
                0,
                1,
                'append_only',
                json_object('schema_version', 1)
             )",
        )
        .bind(&user_id)
        .execute(&pool)
        .await
        .is_err());

        sqlx::query("DELETE FROM notes_pages WHERE id = 'page-a'")
            .execute(&pool)
            .await
            .unwrap();
        let retained_operations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notes_collaboration_operations WHERE id = 'operation-a'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(retained_operations, 1);
    });
}

#[test]
fn schema_rejects_invalid_calendar_values() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        assert!(sqlx::query(
            "INSERT INTO calendars (id, name, source, created_at, updated_at)
             VALUES ('bad-source', 'Bad', 'web', '2026-05-23T00:00:00Z', '2026-05-23T00:00:00Z')",
        )
        .execute(&pool)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO calendar_events
                (id, title, start_time, end_time, timezone, calendar_id, all_day)
             VALUES ('bad-bool', 'Bad', '2026-05-23T09:00:00Z',
                     '2026-05-23T10:00:00Z', 'UTC', 'local', 2)",
        )
        .execute(&pool)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO calendar_events
                (id, title, start_time, end_time, timezone, calendar_id, color)
             VALUES ('bad-color', 'Bad', '2026-05-23T09:00:00Z',
                     '2026-05-23T10:00:00Z', 'UTC', 'local', 32)",
        )
        .execute(&pool)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO calendar_events
                (id, title, start_time, end_time, timezone, calendar_id, priority)
             VALUES ('bad-priority', 'Bad', '2026-05-23T09:00:00Z',
                     '2026-05-23T10:00:00Z', 'UTC', 'local', 10)",
        )
        .execute(&pool)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO calendar_events
                (id, title, start_time, end_time, timezone, calendar_id, geo_lat)
             VALUES ('bad-geo', 'Bad', '2026-05-23T09:00:00Z',
                     '2026-05-23T10:00:00Z', 'UTC', 'local', 25.0)",
        )
        .execute(&pool)
        .await
        .is_err());

        assert!(sqlx::query(
            "INSERT INTO calendar_events
                (id, title, start_time, end_time, timezone, calendar_id)
             VALUES ('bad-fk', 'Bad', '2026-05-23T09:00:00Z',
                     '2026-05-23T10:00:00Z', 'UTC', 'missing')",
        )
        .execute(&pool)
        .await
        .is_err());
    });
}

#[test]
fn schema_accepts_current_pomodoro_preset_keys() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;

        sqlx::query(
            "INSERT INTO pomodoro_configs
                (event_id, rhythm_kind, rhythm_source, preset_key, idle_timeout_minutes)
             VALUES ('event-1', 'count', 'preset', 'adaptive', 3)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "UPDATE pomodoro_configs SET preset_key = 'balanced' WHERE event_id = 'event-1'",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_runs
                (id, event_id, original_event_id, event_date, planned_start, planned_end,
                 started_at, rhythm_kind, rhythm_source, preset_key, last_heartbeat, start_trigger)
             VALUES ('run-balanced', 'event-1', 'event-1', '2026-05-23',
                     '2026-05-23T09:00:00Z', '2026-05-23T10:00:00Z',
                     '2026-05-23T09:00:00Z', 'count', 'preset', 'balanced',
                     '2026-05-23T09:00:00Z', 'manual')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO calendar_events_archive
                (id, source_event_id, archived_at, title, start_time, end_time,
                 calendar_id, created_at, updated_at)
             VALUES ('archive-1', 'event-1', '2026-05-23T11:00:00Z', 'Focus block',
                     '2026-05-23T09:00:00Z', '2026-05-23T10:00:00Z',
                     'local', '2026-05-23T08:00:00Z', '2026-05-23T08:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO calendar_event_archive_pomodoro_configs
                (archive_event_id, rhythm_kind, rhythm_source, preset_key, idle_timeout_minutes)
             VALUES ('archive-1', 'count', 'preset', 'adaptive', 3)",
        )
        .execute(&pool)
        .await
        .unwrap();
    });
}

#[test]
fn schema_keeps_pomodoro_foreign_key_targets() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let references = [
            ("pomodoro_config_count_rhythms", "pomodoro_configs"),
            ("pomodoro_config_sequence_steps", "pomodoro_configs"),
            ("pomodoro_runs", "pomodoro_runs"),
            ("pomodoro_run_count_rhythms", "pomodoro_runs"),
            ("pomodoro_run_sequence_steps", "pomodoro_runs"),
            ("pomodoro_segments", "pomodoro_runs"),
            ("pomodoro_run_events", "pomodoro_runs"),
            (
                "calendar_event_archive_pomodoro_config_count_rhythms",
                "calendar_event_archive_pomodoro_configs",
            ),
            (
                "calendar_event_archive_pomodoro_config_sequence_steps",
                "calendar_event_archive_pomodoro_configs",
            ),
            ("pomodoro_run_adaptive_snapshots", "pomodoro_runs"),
            ("pomodoro_adaptive_context_snapshots", "pomodoro_runs"),
            ("pomodoro_adaptive_decisions", "pomodoro_runs"),
            ("pomodoro_adaptive_planned_blocks", "pomodoro_runs"),
            ("pomodoro_adaptive_assignments", "pomodoro_runs"),
            ("doomscrolling_block_events", "pomodoro_runs"),
        ];

        for (child_table, target_table) in references {
            let query = format!(
                "SELECT COUNT(*) AS count
                 FROM pragma_foreign_key_list('{child_table}')
                 WHERE \"table\" = '{target_table}'",
            );
            let count: i64 = sqlx::query_scalar(&query).fetch_one(&pool).await.unwrap();
            assert!(count > 0, "{child_table} should reference {target_table}",);
        }
    });
}

#[test]
fn schema_allows_only_one_open_pomodoro_run() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;

        insert_open_run(&pool, "run-1").await.unwrap();
        assert!(insert_open_run(&pool, "run-2").await.is_err());

        sqlx::query(
            "UPDATE pomodoro_runs
             SET ended_at = '2026-05-23T09:30:00Z', end_reason = 'stopped'
             WHERE id = 'run-1'",
        )
        .execute(&pool)
        .await
        .unwrap();

        insert_open_run(&pool, "run-2").await.unwrap();
    });
}

#[test]
fn schema_allows_only_one_active_pomodoro_segment() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, status)
             VALUES ('segment-1', 'event-1', '2026-05-23', 'run-1', 1, 'focus',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:40:00Z',
                     '2026-05-23T09:00:00Z', 'active')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let second_active = sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, status)
             VALUES ('segment-2', 'event-1', '2026-05-23', 'run-1', 1, 'short_break',
                     '2026-05-23T09:40:00Z', '2026-05-23T09:45:00Z',
                     '2026-05-23T09:40:00Z', 'active')",
        )
        .execute(&pool)
        .await;

        assert!(second_active.is_err());
    });
}

#[test]
fn schema_accepts_focus_failed_pomodoro_history() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, actual_end, status, end_reason)
             VALUES ('segment-1', 'event-1', '2026-05-23', 'run-1', 1, 'focus',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:40:00Z',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:10:00Z',
                     'interrupted', 'focus_failed')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_run_events
                (id, run_id, segment_id, event_type, occurred_at, phase, reason, duration_seconds)
             VALUES ('event-1', 'run-1', 'segment-1', 'focus_failed',
                     '2026-05-23T09:11:00Z', 'focus', 'long_idle', 60)",
        )
        .execute(&pool)
        .await
        .unwrap();
    });
}

#[test]
fn deleting_calendar_event_preserves_pomodoro_segments() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, actual_end, status, end_reason)
             VALUES ('segment-1', 'event-1', '2026-05-23', 'run-1', 1, 'focus',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:40:00Z',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:20:00Z',
                     'interrupted', 'stopped')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("DELETE FROM calendar_events WHERE id = 'event-1'")
            .execute(&pool)
            .await
            .unwrap();

        let segment_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM pomodoro_segments WHERE id = 'segment-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let event_id: Option<String> =
            sqlx::query_scalar("SELECT event_id FROM pomodoro_segments WHERE id = 'segment-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(segment_count, 1);
        assert_eq!(event_id, None);
    });
}

#[test]
fn schema_allows_only_one_open_pause_per_segment() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();
        sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, status)
             VALUES ('segment-1', 'event-1', '2026-05-23', 'run-1', 1, 'focus',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:40:00Z',
                     '2026-05-23T09:00:00Z', 'active')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_pauses (id, segment_id, started_at, reason)
             VALUES ('pause-1', 'segment-1', '2026-05-23T09:10:00Z', 'manual')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let second_open_pause = sqlx::query(
            "INSERT INTO pomodoro_pauses (id, segment_id, started_at, reason)
             VALUES ('pause-2', 'segment-1', '2026-05-23T09:15:00Z', 'idle')",
        )
        .execute(&pool)
        .await;

        assert!(second_open_pause.is_err());
    });
}

#[test]
fn schema_creates_doomscrolling_usage_samples() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO doomscrolling_usage_samples
                (id, source_type, source_key, display_name, started_at, elapsed_seconds, local_date, created_at)
             VALUES ('sample-1', 'website', 'youtube.com', 'youtube.com', 1779923600000, 30, '2026-05-28', 1779923630000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid = sqlx::query(
            "INSERT INTO doomscrolling_usage_samples
                (id, source_type, source_key, started_at, elapsed_seconds, local_date, created_at)
             VALUES ('sample-2', 'website', 'youtube.com', 1779923600000, 0, '2026-05-28', 1779923630000)",
        )
        .execute(&pool)
        .await;

        assert!(invalid.is_err());
    });
}

#[test]
fn schema_creates_pomodoro_adaptive_tables() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let adaptive_tables = [
            "pomodoro_adaptive_policies",
            "pomodoro_adaptive_policy_bounds",
            "pomodoro_adaptive_context_states",
            "pomodoro_adaptive_context_state_history",
            "pomodoro_adaptive_context_snapshots",
            "pomodoro_adaptive_context_snapshot_features",
            "pomodoro_adaptive_data_quality_flags",
            "pomodoro_adaptive_decisions",
            "pomodoro_adaptive_decision_values",
            "pomodoro_adaptive_decision_reasons",
            "pomodoro_adaptive_decision_state_scores",
            "pomodoro_run_adaptive_snapshots",
            "pomodoro_adaptive_planned_blocks",
            "pomodoro_adaptive_experiments",
            "pomodoro_adaptive_experiment_variants",
            "pomodoro_adaptive_assignments",
            "pomodoro_adaptive_outcomes",
            "doomscrolling_block_events",
            "doomscrolling_block_event_rule_snapshots",
        ];

        for table in adaptive_tables {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = ?")
                    .bind(table)
                    .fetch_optional(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, Some(1), "{table} should exist");
        }
    });
}

#[test]
fn schema_enforces_adaptive_policy_and_decision_integrity() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_policies
                (id, status, policy_version, model_version)
             VALUES ('policy-1', 'active', 1, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let second_active_policy = sqlx::query(
            "INSERT INTO pomodoro_adaptive_policies
                (id, status, policy_version, model_version)
             VALUES ('policy-2', 'active', 1, 1)",
        )
        .execute(&pool)
        .await;
        assert!(second_active_policy.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_policy_bounds
                (policy_id, parameter_key, min_value, max_value)
             VALUES ('policy-1', 'focus_duration_minutes', 15, 60)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_bounds = sqlx::query(
            "INSERT INTO pomodoro_adaptive_policy_bounds
                (policy_id, parameter_key, min_value, max_value)
             VALUES ('policy-1', 'short_break_minutes', 12, 3)",
        )
        .execute(&pool)
        .await;
        assert!(invalid_bounds.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_context_snapshots
                (id, run_id, local_started_at, time_of_day, session_position,
                 event_length, workload, energy)
             VALUES ('snapshot-1', 'run-1', '2026-05-23T09:00:00Z',
                     'morning', 'first', 'medium', 'low', 'unknown')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_context_snapshot_features
                (snapshot_id, feature_key, numeric_value, source_kind)
             VALUES ('snapshot-1', 'clean_focus_seconds', 2400, 'pomodoro')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_data_quality_flags (snapshot_id, flag)
             VALUES ('snapshot-1', 'diary_missing')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_decisions
                (id, policy_id, run_id, context_snapshot_id, opportunity_kind,
                 decision_mode, policy_version, model_version, occurred_at)
             VALUES ('decision-1', 'policy-1', 'run-1', 'snapshot-1',
                     'run_start', 'fallback', 1, 1, '2026-05-23T09:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_values
                (decision_id, value_key, previous_numeric_value, selected_numeric_value, value_unit)
             VALUES ('decision-1', 'focus_duration_minutes', 40, 40, 'minutes')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_bundle_value = sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_values
                (decision_id, value_key, previous_numeric_value, selected_numeric_value, value_unit)
             VALUES ('decision-1', 'rhythm_bundle', 0, 1, 'count')",
        )
        .execute(&pool)
        .await;
        assert!(invalid_bundle_value.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_reasons (decision_id, reason_code)
             VALUES ('decision-1', 'no_history')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_reason = sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_reasons (decision_id, reason_code)
             VALUES ('decision-1', 'raw_anxiety_label')",
        )
        .execute(&pool)
        .await;
        assert!(invalid_reason.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_state_scores
                (decision_id, readiness, strain, recovery_debt,
                 avoidance_pressure, momentum, confidence)
             VALUES ('decision-1', 0.2, 0.1, 0.1, 0.0, 0.2, 0.1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_score = sqlx::query(
            "INSERT INTO pomodoro_adaptive_decision_state_scores
                (decision_id, readiness, strain, recovery_debt,
                 avoidance_pressure, momentum, confidence)
             VALUES ('decision-bad', 1.2, 0.1, 0.1, 0.0, 0.2, 0.1)",
        )
        .execute(&pool)
        .await;
        assert!(invalid_score.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_run_adaptive_snapshots
                (run_id, policy_id, policy_version, model_version,
                 context_snapshot_id, decision_id)
             VALUES ('run-1', 'policy-1', 1, 1, 'snapshot-1', 'decision-1')",
        )
        .execute(&pool)
        .await
        .unwrap();
    });
}

#[test]
fn schema_records_adaptive_experiments_and_outcomes() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_policies
                (id, status, policy_version, model_version)
             VALUES ('policy-1', 'active', 1, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_context_snapshots
                (id, run_id, local_started_at, time_of_day, session_position,
                 event_length, workload, energy)
             VALUES ('snapshot-1', 'run-1', '2026-05-23T09:00:00Z',
                     'morning', 'first', 'medium', 'low', 'unknown')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_experiments
                (id, policy_id, parameter_key, assignment_unit, status, started_at)
             VALUES ('experiment-1', 'policy-1', 'focus_duration_minutes',
                     'run', 'active', '2026-05-23T09:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_experiments
                (id, policy_id, parameter_key, assignment_unit, status, started_at)
             VALUES ('experiment-bundle', 'policy-1', 'rhythm_bundle',
                     'run', 'active', '2026-05-23T09:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_bundle_bound = sqlx::query(
            "INSERT INTO pomodoro_adaptive_policy_bounds
                (policy_id, parameter_key, min_value, max_value)
             VALUES ('policy-1', 'rhythm_bundle', 0, 1)",
        )
        .execute(&pool)
        .await;
        assert!(invalid_bundle_bound.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_experiment_variants
                (experiment_id, variant_key, numeric_value, is_control)
             VALUES ('experiment-1', 'control', 40, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_experiment_variants
                (experiment_id, variant_key, numeric_value, is_control)
             VALUES ('experiment-1', 'shorter', 35, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let second_control = sqlx::query(
            "INSERT INTO pomodoro_adaptive_experiment_variants
                (experiment_id, variant_key, numeric_value, is_control)
             VALUES ('experiment-1', 'other-control', 45, 1)",
        )
        .execute(&pool)
        .await;
        assert!(second_control.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_assignments
                (id, experiment_id, variant_key, run_id,
                 context_snapshot_id, assignment_seed, assigned_at)
             VALUES ('assignment-1', 'experiment-1', 'shorter', 'run-1',
                     'snapshot-1', 'seed-1', '2026-05-23T09:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let invalid_assignment = sqlx::query(
            "INSERT INTO pomodoro_adaptive_assignments
                (id, experiment_id, variant_key, assignment_seed, assigned_at)
             VALUES ('assignment-bad', 'experiment-1', 'missing',
                     'seed-2', '2026-05-23T09:00:00Z')",
        )
        .execute(&pool)
        .await;
        assert!(invalid_assignment.is_err());

        sqlx::query(
            "INSERT INTO pomodoro_adaptive_outcomes
                (id, assignment_id, outcome_window, outcome_key,
                 boolean_value, measured_at)
             VALUES ('outcome-1', 'assignment-1', 'run',
                     'clean_focus_completed', 1, '2026-05-23T10:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();
    });
}

#[test]
fn schema_records_redacted_doomscrolling_block_events() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_event(&pool).await;
        insert_open_run(&pool, "run-1").await.unwrap();

        sqlx::query(
            "INSERT INTO pomodoro_segments
                (id, event_id, event_date, run_id, rhythm_position, phase,
                 planned_start, planned_end, actual_start, status)
             VALUES ('segment-1', 'event-1', '2026-05-23', 'run-1', 1, 'focus',
                     '2026-05-23T09:00:00Z', '2026-05-23T09:40:00Z',
                     '2026-05-23T09:00:00Z', 'active')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO doomscrolling_block_events
                (id, run_id, segment_id, occurred_at, source_type,
                 source_key, display_name, phase, decision, rule_id, category_id)
             VALUES ('block-1', 'run-1', 'segment-1', '2026-05-23T09:10:00Z',
                     'browser', 'reddit.com', 'reddit.com', 'focus',
                     'blocked', 'rule-1', 'social')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO doomscrolling_block_event_rule_snapshots
                (block_event_id, rule_id, rule_kind, rule_label,
                 environment_id, blocker_mode)
             VALUES ('block-1', 'rule-1', 'category', 'Social',
                     'env-1', 'blacklist')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let full_url = sqlx::query(
            "INSERT INTO doomscrolling_block_events
                (id, occurred_at, source_type, source_key, phase, decision)
             VALUES ('block-bad', '2026-05-23T09:11:00Z',
                     'browser', 'https://reddit.com/r/all', 'focus', 'blocked')",
        )
        .execute(&pool)
        .await;
        assert!(full_url.is_err());
    });
}

#[test]
fn schema_creates_project_custom_emojis() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO project_custom_emojis (id, name, asset_path, sort_order)
             VALUES ('emoji-1', 'Rocket',
                     'project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                     1000)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let asset_path: String =
            sqlx::query_scalar("SELECT asset_path FROM project_custom_emojis WHERE id = 'emoji-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            asset_path,
            "project-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        );

        for (id, name, asset_path, sort_order) in [
            (
                "bad-name",
                "",
                "project-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-prefix",
                "Bad",
                "other/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-parent",
                "Bad",
                "project-icons/../bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-nested",
                "Bad",
                "project-icons/nested/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                0,
            ),
            (
                "bad-order",
                "Bad",
                "project-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                -1,
            ),
        ] {
            assert!(
                sqlx::query(
                    "INSERT INTO project_custom_emojis (id, name, asset_path, sort_order)
                     VALUES (?, ?, ?, ?)",
                )
                .bind(id)
                .bind(name)
                .bind(asset_path)
                .bind(sort_order)
                .execute(&pool)
                .await
                .is_err(),
                "{id} should fail"
            );
        }
    });
}

#[test]
fn schema_creates_notes_page_icon_assets() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_page_icon_assets
                (id, asset_path, original_name, content_type, byte_size, sha256)
             VALUES (
                'asset-1',
                'notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'focus.png',
                'image/png',
                42,
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let asset_path: String = sqlx::query_scalar(
            "SELECT asset_path FROM notes_page_icon_assets WHERE id = 'asset-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            asset_path,
            "notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        );

        for (id, asset_path, content_type, byte_size, sha256) in [
            (
                "bad-prefix",
                "project-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-nested",
                "notes/page-icons/nested/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-type",
                "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.gif",
                "image/gif",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-size",
                "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                0,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-hash",
                "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "short",
            ),
        ] {
            let inserted = sqlx::query(
                "INSERT INTO notes_page_icon_assets
                    (id, asset_path, content_type, byte_size, sha256)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(asset_path)
            .bind(content_type)
            .bind(byte_size)
            .bind(sha256)
            .execute(&pool)
            .await;
            assert!(inserted.is_err());
        }
    });
}

#[test]
fn schema_creates_notes_page_cover_assets() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_page_cover_assets
                (id, asset_path, original_name, content_type, byte_size, sha256)
             VALUES (
                'asset-1',
                'notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'cover.png',
                'image/png',
                42,
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let asset_path: String = sqlx::query_scalar(
            "SELECT asset_path FROM notes_page_cover_assets WHERE id = 'asset-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            asset_path,
            "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png",
        );

        for (id, asset_path, content_type, byte_size, sha256) in [
            (
                "bad-prefix",
                "notes/page-icons/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-nested",
                "notes/page-covers/nested/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-type",
                "notes/page-covers/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.gif",
                "image/gif",
                1,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-size",
                "notes/page-covers/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                0,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
            (
                "bad-hash",
                "notes/page-covers/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.png",
                "image/png",
                1,
                "short",
            ),
        ] {
            let inserted = sqlx::query(
                "INSERT INTO notes_page_cover_assets
                    (id, asset_path, content_type, byte_size, sha256)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(asset_path)
            .bind(content_type)
            .bind(byte_size)
            .bind(sha256)
            .execute(&pool)
            .await;
            assert!(inserted.is_err());
        }
    });
}

#[test]
fn schema_creates_notes_assets_and_references() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title)
             VALUES ('page-1', 'workspace', 'Assets')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'block-1',
                'page-1',
                'page_id',
                'page-1',
                'paragraph',
                '{\"paragraph\":{\"rich_text\":[]}}',
                '',
                1000
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO notes_assets (
                id,
                asset_path,
                kind,
                source_type,
                original_name,
                content_type,
                byte_size,
                sha256
             )
             VALUES (
                'notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'image',
                'local_upload',
                'focus.png',
                'image/png',
                42,
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
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
             VALUES (
                'notes/files/report.pdf',
                'notes/files/report.pdf',
                'pdf',
                'imported',
                'report.pdf',
                'application/pdf',
                128,
                'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
                'missing',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO notes_asset_references (asset_id, owner_type, owner_id, page_id, role)
             VALUES (
                'notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'page',
                'page-1',
                'page-1',
                'page_icon'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_asset_references (
                asset_id,
                owner_type,
                owner_id,
                page_id,
                block_id,
                role
             )
             VALUES (
                'notes/files/report.pdf',
                'block',
                'block-1',
                'page-1',
                'block-1',
                'block_file'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let reference_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_asset_references")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(reference_count, 2);

        for (
            id,
            asset_path,
            kind,
            source_type,
            content_type,
            byte_size,
            sha256,
            storage_state,
            missing_at,
        ) in [
            (
                "bad-path",
                "notes/files/nested/file.pdf",
                "pdf",
                "imported",
                "application/pdf",
                1,
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "available",
                None::<&str>,
            ),
            (
                "bad-icon-type",
                "notes/page-icons/icon.svg",
                "image",
                "local_upload",
                "image/svg+xml",
                1,
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "available",
                None::<&str>,
            ),
            (
                "bad-content-type",
                "notes/files/file.bin",
                "file",
                "local_upload",
                "not-a-mime",
                1,
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "available",
                None::<&str>,
            ),
            (
                "bad-size",
                "notes/files/file.bin",
                "file",
                "local_upload",
                "application/octet-stream",
                0,
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "available",
                None::<&str>,
            ),
            (
                "bad-hash",
                "notes/files/file.bin",
                "file",
                "local_upload",
                "application/octet-stream",
                1,
                "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                "available",
                None::<&str>,
            ),
            (
                "bad-missing-state",
                "notes/files/file.bin",
                "file",
                "local_upload",
                "application/octet-stream",
                1,
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "missing",
                None::<&str>,
            ),
        ] {
            let inserted = sqlx::query(
                "INSERT INTO notes_assets (
                    id,
                    asset_path,
                    kind,
                    source_type,
                    content_type,
                    byte_size,
                    sha256,
                    storage_state,
                    missing_at
                 )
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(asset_path)
            .bind(kind)
            .bind(source_type)
            .bind(content_type)
            .bind(byte_size)
            .bind(sha256)
            .bind(storage_state)
            .bind(missing_at)
            .execute(&pool)
            .await;
            assert!(inserted.is_err(), "{id} should fail");
        }

        let invalid_reference = sqlx::query(
            "INSERT INTO notes_asset_references (asset_id, owner_type, owner_id, role)
             VALUES (
                'notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.png',
                'page',
                'page-1',
                'page_icon'
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_reference.is_err());
    });
}

#[test]
fn schema_creates_notes_search_fts_projection() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title)
             VALUES ('page-1', 'workspace', 'Searchable')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_search_index (
                id,
                source_type,
                page_id,
                title,
                body,
                metadata,
                source_last_edited_time
             )
             VALUES (
                'page:page-1',
                'page',
                'page-1',
                'Searchable',
                'Alpha project',
                'local metadata',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_search_fts (index_id, title, body, metadata)
             VALUES ('page:page-1', 'Searchable', 'Alpha project', 'local metadata')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_search_index_state (key, value)
             VALUES ('source_fingerprint', 'test')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let matched: String = sqlx::query_scalar(
            "SELECT index_id
             FROM notes_search_fts
             WHERE notes_search_fts MATCH 'alpha'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(matched, "page:page-1");

        let state: String =
            sqlx::query_scalar("SELECT value FROM notes_search_index_state WHERE key = ?")
                .bind("source_fingerprint")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(state, "test");
    });
}

#[test]
fn schema_creates_notes_backlink_index() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title)
             VALUES ('page-1', 'workspace', 'Source')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'block-1',
                'page-1',
                'page_id',
                'page-1',
                'paragraph',
                '{\"paragraph\":{\"rich_text\":[]}}',
                'Mention target',
                1000
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_backlink_index (
                id,
                target_type,
                target_id,
                source_type,
                source_page_id,
                source_block_id,
                reference_type,
                snippet,
                created_time,
                last_edited_time
             )
             VALUES (
                'block:block-1:page:target-page:page_mention',
                'page',
                'target-page',
                'block',
                'page-1',
                'block-1',
                'page_mention',
                'Mention target',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_backlink_index (
                id,
                target_type,
                target_id,
                target_object_type,
                source_type,
                source_page_id,
                source_block_id,
                reference_type,
                snippet,
                created_time,
                last_edited_time
             )
             VALUES (
                'block:block-1:local_object:task-1:local_object_mention',
                'local_object',
                'task-1',
                'project_task',
                'block',
                'page-1',
                'block-1',
                'local_object_mention',
                'Mention target',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_backlink_index_state (key, value)
             VALUES ('source_fingerprint', 'test')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let target_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_backlink_index
             WHERE target_type = 'page' AND target_id = 'target-page'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(target_count, 1);

        let invalid_local_object = sqlx::query(
            "INSERT INTO notes_backlink_index (
                id,
                target_type,
                target_id,
                source_type,
                source_page_id,
                source_block_id,
                reference_type,
                created_time,
                last_edited_time
             )
             VALUES (
                'invalid-local-object',
                'local_object',
                'task-2',
                'block',
                'page-1',
                'block-1',
                'local_object_mention',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_local_object.is_err());
    });
}

#[test]
fn schema_creates_notes_link_facts() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;

        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title)
             VALUES ('page-1', 'workspace', 'Source')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (
                'block-1',
                'page-1',
                'page_id',
                'page-1',
                'paragraph',
                '{\"paragraph\":{\"rich_text\":[]}}',
                'Task mention',
                1000
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_assets (
                id,
                asset_path,
                kind,
                source_type,
                original_name,
                content_type,
                byte_size,
                sha256
             )
             VALUES (
                'asset-1',
                'notes/files/asset-1.png',
                'image',
                'local_upload',
                'asset-1.png',
                'image/png',
                42,
                'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_link_facts (
                id,
                source_object_type,
                source_object_id,
                source_page_id,
                source_block_id,
                target_object_type,
                target_object_id,
                link_type,
                snippet,
                created_time,
                last_edited_time
             )
             VALUES (
                'fact-local-object',
                'block',
                'block-1',
                'page-1',
                'block-1',
                'project_task',
                'task-1',
                'local_object_mention',
                'Task mention',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_link_facts (
                id,
                source_object_type,
                source_object_id,
                source_page_id,
                source_block_id,
                target_object_type,
                target_object_id,
                target_asset_id,
                link_type,
                snippet,
                created_time,
                last_edited_time
             )
             VALUES (
                'fact-file',
                'block',
                'block-1',
                'page-1',
                'block-1',
                'file',
                'asset-1',
                'asset-1',
                'block_file',
                'diagram.png',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_link_facts_state (key, value)
             VALUES ('source_fingerprint', 'test')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let task_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_link_facts
             WHERE target_object_type = 'project_task'
               AND target_object_id = 'task-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(task_count, 1);

        let invalid_file = sqlx::query(
            "INSERT INTO notes_link_facts (
                id,
                source_object_type,
                source_object_id,
                source_page_id,
                source_block_id,
                target_object_type,
                target_object_id,
                link_type,
                snippet,
                created_time,
                last_edited_time
             )
             VALUES (
                'invalid-file',
                'block',
                'block-1',
                'page-1',
                'block-1',
                'file',
                'asset-1',
                'block_file',
                'diagram.png',
                '2026-07-02T12:00:00.000Z',
                '2026-07-02T12:00:00.000Z'
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_file.is_err());
    });
}

#[test]
fn schema_creates_strict_project_notes_history_storage() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let default_retention: i64 = sqlx::query_scalar(
            "SELECT retention_days FROM notes_page_history_settings WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(default_retention, 30);

        sqlx::query("INSERT INTO project_groups (id, name) VALUES ('group-1', 'Group')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name)
             VALUES ('project-1', 'group-1', 'Learning')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let inherited: Option<i64> = sqlx::query_scalar(
            "SELECT notes_history_retention_days FROM projects WHERE id = 'project-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(inherited, None);

        for days in [0_i64, 7, 30, 90, 180, 365] {
            sqlx::query(
                "UPDATE projects SET notes_history_retention_days = ? WHERE id = 'project-1'",
            )
            .bind(days)
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query("UPDATE notes_page_history_settings SET retention_days = ? WHERE id = 1")
                .bind(days)
                .execute(&pool)
                .await
                .unwrap();
        }
        assert!(sqlx::query(
            "UPDATE projects SET notes_history_retention_days = 14 WHERE id = 'project-1'",
        )
        .execute(&pool)
        .await
        .is_err());
        assert!(sqlx::query(
            "UPDATE notes_page_history_settings SET retention_days = NULL WHERE id = 1",
        )
        .execute(&pool)
        .await
        .is_err());
        assert!(sqlx::query(
            "UPDATE notes_page_history_settings SET retention_days = 366 WHERE id = 1",
        )
        .execute(&pool)
        .await
        .is_err());

        for table in [
            "notes_history_bundles",
            "notes_history_bundle_chunks",
            "notes_project_history_versions",
            "notes_project_history_bundle_references",
            "notes_project_history_asset_pins",
            "notes_project_history_dirty",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(exists, 1, "missing table {table}");
        }

        let operation_foreign_keys: Vec<String> = sqlx::query_scalar(
            "SELECT \"table\" FROM pragma_foreign_key_list('notes_collaboration_operations')",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(!operation_foreign_keys
            .iter()
            .any(|table| table == "notes_pages"));
        assert!(!operation_foreign_keys
            .iter()
            .any(|table| table == "notes_blocks"));
    });
}
