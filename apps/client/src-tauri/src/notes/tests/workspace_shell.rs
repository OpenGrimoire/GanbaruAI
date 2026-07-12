use super::helpers::*;
use crate::notes::models::NotePageSummaryWindowRequest;

#[test]
fn empty_workspace_shell_contains_summaries_only() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        let shell = workspace_shell::load_workspace_shell(
            &pool,
            NoteWorkspaceShellRequest {
                project_id: None,
                expanded_page_ids: Vec::new(),
                seed_page_ids: Vec::new(),
                selected_page_id: None,
                page_cursor: None,
                folder_cursor: None,
                destination_candidates: false,
                page_query: None,
            },
        )
        .await
        .expect("load shell");
        let value = serde_json::to_value(shell).expect("serialize shell");

        assert_eq!(value["total_page_count"], 0);
        assert_eq!(value["pages"], json!([]));
        assert!(value.get("blocks").is_none());
        assert!(value.get("properties").is_none());
        assert!(value.get("history").is_none());
    });
}

#[test]
fn dense_workspace_shell_stays_within_row_and_byte_caps() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::raw_sql(
            "WITH RECURSIVE sequence(value) AS (
                 SELECT 1
                 UNION ALL
                 SELECT value + 1 FROM sequence WHERE value < 10000
             )
             INSERT INTO notes_pages (id, parent_type, title, properties)
             SELECT printf('00000000-0000-4000-8000-%012d', value),
                    'workspace',
                    printf('Page %05d', value),
                    json_object('title', json_object('id', 'title', 'type', 'title', 'title', json_array()))
             FROM sequence;",
        )
        .execute(&pool)
        .await
        .expect("seed dense pages");

        let shell = workspace_shell::load_workspace_shell(
            &pool,
            NoteWorkspaceShellRequest {
                project_id: None,
                expanded_page_ids: Vec::new(),
                seed_page_ids: Vec::new(),
                selected_page_id: None,
                page_cursor: None,
                folder_cursor: None,
                destination_candidates: false,
                page_query: None,
            },
        )
        .await
        .expect("load dense shell");
        let bytes = serde_json::to_vec(&shell).expect("serialize dense shell");
        let value = serde_json::to_value(shell).expect("inspect dense shell");

        assert_eq!(value["total_page_count"], 10_000);
        assert_eq!(value["pages"].as_array().map(Vec::len), Some(50));
        assert!(bytes.len() < 128 * 1024, "shell was {} bytes", bytes.len());
        let first_page = &value["pages"][0];
        assert!(first_page.get("properties").is_none());
        assert!(first_page.get("cover").is_none());
    });
}

#[test]
fn workspace_shell_keyset_is_stable_for_equal_times_and_titles() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::raw_sql(
            "WITH RECURSIVE sequence(value) AS (
                 SELECT 1 UNION ALL SELECT value + 1 FROM sequence WHERE value < 120
             )
             INSERT INTO notes_pages (id, parent_type, title, properties, last_edited_time)
             SELECT printf('00000000-0000-4000-8000-%012d', value), 'workspace', 'Same title',
                    json_object('title', json_object('id', 'title', 'type', 'title', 'title', json_array())),
                    '2026-07-11T00:00:00.000Z'
             FROM sequence;",
        )
        .execute(&pool)
        .await
        .expect("seed equal page keys");

        let request = |cursor| NoteWorkspaceShellRequest {
            project_id: None,
            expanded_page_ids: Vec::new(),
            seed_page_ids: Vec::new(),
            selected_page_id: None,
            page_cursor: cursor,
            folder_cursor: Some("end".to_string()),
            destination_candidates: false,
            page_query: None,
        };
        let first = serde_json::to_value(
            workspace_shell::load_workspace_shell(&pool, request(None))
                .await
                .unwrap(),
        )
        .unwrap();
        let cursor = first["next_page_cursor"].as_str().unwrap().to_string();
        let second = serde_json::to_value(
            workspace_shell::load_workspace_shell(&pool, request(Some(cursor)))
                .await
                .unwrap(),
        )
        .unwrap();
        let first_ids = first["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["id"].as_str().unwrap())
            .collect::<std::collections::HashSet<_>>();
        let second_ids = second["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["id"].as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(first_ids.len(), 50);
        assert_eq!(second_ids.len(), 50);
        assert!(second_ids.iter().all(|id| !first_ids.contains(id)));
        let plan = sqlx::query("EXPLAIN QUERY PLAN SELECT id FROM notes_pages WHERE in_trash = 0 AND archived = 0 ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC LIMIT 50")
            .fetch_all(&pool).await.unwrap();
        let details = plan
            .iter()
            .map(|row| sqlx::Row::get::<String, _>(row, "detail"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            details.contains("idx_notes_pages_active"),
            "unexpected plan: {details}"
        );
    });
}

#[test]
fn archive_summary_window_filters_and_pages_without_hidden_page_json() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        sqlx::raw_sql(
            "WITH RECURSIVE sequence(value) AS (
                 SELECT 1 UNION ALL SELECT value + 1 FROM sequence WHERE value < 75
             )
             INSERT INTO notes_pages (id, parent_type, title, properties, cover, archived, last_edited_time)
             SELECT printf('20000000-0000-4000-8000-%012d', value), 'workspace',
                    CASE WHEN value <= 60 THEN 'Match same' ELSE 'Other' END,
                    json_object('title', json_object('id', 'title', 'type', 'title', 'title', json_array()), 'secret', printf('payload-%d', value)),
                    json_object('type', 'external', 'external', json_object('url', 'https://example.com/private')),
                    1, '2026-07-11T00:00:00.000Z'
             FROM sequence;",
        )
        .execute(&pool)
        .await
        .unwrap();
        let first = serde_json::to_value(
            reads::list_archived_page_window(
                &pool,
                NotePageSummaryWindowRequest {
                    cursor: None,
                    query: Some("match".to_string()),
                    page_size: Some(25),
                },
            )
            .await
            .unwrap(),
        )
        .unwrap();
        let cursor = first["next_cursor"].as_str().unwrap();
        let second = serde_json::to_value(
            reads::list_archived_page_window(
                &pool,
                NotePageSummaryWindowRequest {
                    cursor: Some(cursor.to_string()),
                    query: Some("match".to_string()),
                    page_size: Some(25),
                },
            )
            .await
            .unwrap(),
        )
        .unwrap();
        let first_ids = first["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|page| page["id"].as_str().unwrap())
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(first["total_count"], 60);
        assert_eq!(first_ids.len(), 25);
        assert_eq!(second["pages"].as_array().unwrap().len(), 25);
        assert!(second["pages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|page| !first_ids.contains(page["id"].as_str().unwrap())));
        assert!(first["pages"][0].get("properties").is_none());
        assert!(first["pages"][0].get("cover").is_none());
    });
}
