use super::helpers::*;

#[test]
fn database_row_pages_are_real_pages_with_page_lifecycle() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "done_checkbox".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        let loaded = data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Write spec".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        let loaded_json = serde_json::to_value(&loaded).unwrap();
        assert_eq!(loaded_json["page"]["parent"]["type"], "data_source_id");
        assert_eq!(
            loaded_json["page"]["parent"]["data_source_id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            loaded_json["page"]["properties"]["Name"]["title"][0]["plain_text"],
            "Write spec"
        );
        assert_eq!(loaded_json["page"]["properties"]["Done"]["checkbox"], false);
        assert_eq!(loaded_json["blocks"]["results"][0]["type"], "paragraph");

        let sidebar_pages = reads::list_sidebar_pages(
            &pool,
            NoteSidebarPagesRequest {
                expanded_page_ids: vec![],
                seed_page_ids: vec![],
                selected_page_id: None,
            },
        )
        .await
        .unwrap();
        let sidebar_json = serde_json::to_value(sidebar_pages).unwrap();
        assert!(!sidebar_json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["id"] == PAGE_B));

        let rows = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);

        let search_results = search::search(&pool, "Write", Some(10), false)
            .await
            .unwrap();
        let search_json = serde_json::to_value(search_results).unwrap();
        assert!(search_json
            .as_array()
            .unwrap()
            .iter()
            .any(|result| result["page"]["id"] == PAGE_B));

        let duplicated = writes::duplicate_page(&pool, PAGE_B, NoteDuplicatePage { title: None })
            .await
            .unwrap();
        let duplicated_json = serde_json::to_value(&duplicated).unwrap();
        let duplicate_id = duplicated_json["page"]["id"].as_str().unwrap().to_string();
        assert_ne!(duplicate_id, PAGE_B);
        assert_eq!(
            duplicated_json["page"]["parent"]["data_source_id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            duplicated_json["page"]["properties"]["Name"]["title"][0]["plain_text"],
            "Write spec"
        );

        writes::trash_page(&pool, PAGE_B, true).await.unwrap();
        let rows_after_trash = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows_after_trash.len(), 1);
        let trashed = reads::list_trashed_pages(&pool).await.unwrap();
        let trashed_json = serde_json::to_value(trashed).unwrap();
        assert!(trashed_json
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["id"] == PAGE_B));

        let restored = writes::trash_page(&pool, PAGE_B, false).await.unwrap();
        let restored_json = serde_json::to_value(restored).unwrap();
        assert_eq!(restored_json["parent"]["data_source_id"], DATA_SOURCE_A);
        let rows_after_restore = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert_eq!(rows_after_restore.len(), 2);

        sqlx::query("UPDATE notes_data_sources SET in_trash = 1 WHERE id = ?")
            .bind(DATA_SOURCE_A)
            .execute(&pool)
            .await
            .unwrap();
        let hidden_rows = data_source_rows::list_data_source_row_pages(&pool, DATA_SOURCE_A)
            .await
            .unwrap();
        assert!(hidden_rows.is_empty());
        let hidden_search = search::search(&pool, "Write", Some(10), false)
            .await
            .unwrap();
        let hidden_search_json = serde_json::to_value(hidden_search).unwrap();
        assert!(!hidden_search_json
            .as_array()
            .unwrap()
            .iter()
            .any(|result| result["page"]["parent"]["type"] == "data_source_id"));
    });
}

#[test]
fn editable_database_table_view_persists_cells_configuration_filters_and_sorts() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;
        databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: None,
                cover: None,
            },
        )
        .await
        .unwrap();
        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "type": "number",
                        "number": { "format": "number" }
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "low", "name": "Low", "color": "blue" },
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                    "priority".to_string(),
                    "done_checkbox".to_string(),
                ],
                hidden_property_ids: vec![],
            },
        )
        .await
        .unwrap();

        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_B.to_string(),
                title: "Beta".to_string(),
                first_block_id: BLOCK_B.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();
        data_source_rows::create_data_source_row_page(
            &pool,
            DATA_SOURCE_A,
            NoteDataSourceRowPageCreate {
                id: PAGE_C.to_string(),
                title: "Alpha".to_string(),
                first_block_id: BLOCK_C.to_string(),
                properties: None,
            },
        )
        .await
        .unwrap();

        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "details".to_string(),
                value: json!("Write backend"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(5),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("High"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_B,
            NoteDataSourceRowPropertyUpdate {
                property_id: "done_checkbox".to_string(),
                value: json!(true),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "details".to_string(),
                value: json!("Write frontend"),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "estimate".to_string(),
                value: json!(2),
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_row_property(
            &pool,
            DATA_SOURCE_A,
            PAGE_C,
            NoteDataSourceRowPropertyUpdate {
                property_id: "priority".to_string(),
                value: json!("Low"),
            },
        )
        .await
        .unwrap();

        let table = data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTableViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "title".to_string(),
                    condition: "contains".to_string(),
                    value: Some(json!("a")),
                }],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "priority".to_string(),
                        "done_checkbox".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({
                        "title": 260,
                        "estimate": 160,
                        "priority": 180,
                        "done_checkbox": 112,
                        "details": 240
                    }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let table_json = serde_json::to_value(table).unwrap();
        assert_eq!(
            table_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["details"])
        );
        assert_eq!(
            table_json["view"]["configuration"]["table"]["column_widths"]["estimate"],
            160
        );
        assert_eq!(
            table_json["view"]["configuration"]["table"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(
            table_json["view"]["filter"]["filters"][0]["property_id"],
            "title"
        );
        assert_eq!(table_json["view"]["sorts"][0]["property_id"], "estimate");
        assert_eq!(table_json["rows"].as_array().unwrap().len(), 2);
        assert_eq!(table_json["rows"][0]["id"], PAGE_B);
        assert_eq!(table_json["rows"][0]["properties"]["Estimate"]["number"], 5);
        assert_eq!(
            table_json["rows"][0]["properties"]["Priority"]["select"]["name"],
            "High"
        );
        assert_eq!(
            table_json["rows"][0]["properties"]["Details"]["rich_text"][0]["plain_text"],
            "Write backend"
        );

        let filtered = data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            None,
            None,
            NoteDataSourceTableViewUpdate {
                filter: vec![NoteDataSourceTableFilter {
                    property_id: "done_checkbox".to_string(),
                    condition: "checked".to_string(),
                    value: None,
                }],
                sorts: vec![],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "priority".to_string(),
                        "done_checkbox".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({
                        "title": 260,
                        "estimate": 160,
                        "priority": 180,
                        "done_checkbox": 112,
                        "details": 240
                    }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();
        let filtered_json = serde_json::to_value(filtered).unwrap();
        assert_eq!(filtered_json["rows"].as_array().unwrap().len(), 1);
        assert_eq!(filtered_json["rows"][0]["id"], PAGE_B);
    });
}
