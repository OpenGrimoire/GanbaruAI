use super::helpers::*;

#[test]
fn page_breadcrumb_resolves_unloaded_ancestor_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_A.to_string(),
                title: "Root".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_A.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Child".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_C).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json.as_array().unwrap().len(), 3);
        assert_eq!(breadcrumb_json[0]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[0]["status"], "active");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[1]["status"], "active");
        assert_eq!(breadcrumb_json[2]["id"], PAGE_C);
        assert_eq!(breadcrumb_json[2]["current"], true);
    });
}

#[test]
fn page_breadcrumb_marks_unavailable_ancestors() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_A.to_string(),
                title: "Archived root".to_string(),
                parent: workspace_parent(),
                first_block_id: BLOCK_A.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_B.to_string(),
                title: "Trashed child".to_string(),
                parent: page_parent(PAGE_A),
                first_block_id: BLOCK_B.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();
        writes::create_page(
            &pool,
            NotePageCreate {
                id: PAGE_C.to_string(),
                title: "Leaf".to_string(),
                parent: page_parent(PAGE_B),
                first_block_id: BLOCK_C.to_string(),
                after_block_id: None,
                properties: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE notes_pages SET archived = 1 WHERE id = ?")
            .bind(PAGE_A)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("UPDATE notes_pages SET in_trash = 1 WHERE id = ?")
            .bind(PAGE_B)
            .execute(&pool)
            .await
            .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_C).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json[0]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[0]["status"], "archived");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[1]["status"], "trashed");
        assert_eq!(breadcrumb_json[2]["id"], PAGE_C);
        assert_eq!(breadcrumb_json[2]["status"], "active");
    });
}

#[test]
fn page_breadcrumb_marks_missing_ancestor_rows() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        sqlx::raw_sql("PRAGMA foreign_keys=OFF")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE notes_pages
             SET parent_type = 'page_id', parent_page_id = ?
             WHERE id = ?",
        )
        .bind(PAGE_B)
        .bind(PAGE_A)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();

        let breadcrumb = reads::get_page_breadcrumb(&pool, PAGE_A).await.unwrap();
        let breadcrumb_json = serde_json::to_value(breadcrumb).unwrap();
        assert_eq!(breadcrumb_json.as_array().unwrap().len(), 2);
        assert_eq!(breadcrumb_json[0]["id"], PAGE_B);
        assert_eq!(breadcrumb_json[0]["status"], "missing");
        assert_eq!(breadcrumb_json[1]["id"], PAGE_A);
        assert_eq!(breadcrumb_json[1]["current"], true);
    });
}
