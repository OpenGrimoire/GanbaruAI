use super::helpers::migrated_memory_pool;

#[test]
fn schema_creates_normalized_quick_notes_storage() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        for table in [
            "quick_note_tags",
            "quick_notes",
            "quick_note_text_runs",
            "quick_notes_search_fts",
        ] {
            let exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE name = ?")
                    .bind(table)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(exists, 1, "missing {table}");
        }
        let invalid_color =
            sqlx::query("INSERT INTO quick_notes (id, color) VALUES ('bad-color', 32)")
                .execute(&pool)
                .await;
        assert!(invalid_color.is_err());
        let invalid_run = sqlx::query(
            "INSERT INTO quick_note_text_runs (note_id, sort_order, content) VALUES ('missing', 0, 'text')",
        )
        .execute(&pool)
        .await;
        assert!(invalid_run.is_err());
        sqlx::query("INSERT INTO quick_note_tags (id, name, sort_order) VALUES ('tag', 'Work', 0)")
            .execute(&pool)
            .await
            .unwrap();
        let duplicate_name = sqlx::query(
            "INSERT INTO quick_note_tags (id, name, sort_order) VALUES ('tag-2', 'work', 1)",
        )
        .execute(&pool)
        .await;
        assert!(duplicate_name.is_err());
        let invalid_order = sqlx::query(
            "INSERT INTO quick_note_tags (id, name, sort_order) VALUES ('tag-3', 'Later', 9)",
        )
        .execute(&pool)
        .await;
        assert!(invalid_order.is_err());
    });
}
