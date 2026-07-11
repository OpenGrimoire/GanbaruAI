use super::helpers::*;

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
