use super::helpers::*;

#[test]
fn create_local_database_inside_notes_page() {
    crate::test_block_on(async {
        let pool = migrated_memory_pool().await;
        create_page(&pool, PAGE_A, BLOCK_A).await;

        let created = databases::create_database(
            &pool,
            NoteDatabaseCreate {
                id: DATABASE_A.to_string(),
                data_source_id: DATA_SOURCE_A.to_string(),
                view_id: DATABASE_VIEW_A.to_string(),
                title: "Tasks".to_string(),
                parent: Some(page_parent(PAGE_A)),
                after_block_id: Some(BLOCK_A.to_string()),
                replace_block_id: None,
                icon: Some(json!({
                    "type": "icon",
                    "icon": {
                        "name": "table",
                        "color": "blue"
                    }
                })),
                cover: None,
            },
        )
        .await
        .unwrap();

        let created_json = serde_json::to_value(created).unwrap();
        assert_eq!(created_json["database"]["id"], DATABASE_A);
        assert_eq!(created_json["database"]["parent"]["page_id"], PAGE_A);
        assert_eq!(
            created_json["database"]["data_sources"][0]["id"],
            DATA_SOURCE_A
        );
        assert_eq!(
            created_json["data_source"]["parent"]["database_id"],
            DATABASE_A
        );
        assert_eq!(created_json["view"]["type"], "table");
        assert_eq!(created_json["block"]["type"], "child_database");
        assert_eq!(
            created_json["block"]["child_database"]["database_id"],
            DATABASE_A
        );

        let properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_data_sources WHERE id = ?")
                .bind(DATA_SOURCE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let properties_json: serde_json::Value = serde_json::from_str(&properties).unwrap();
        assert_eq!(properties_json["Name"]["type"], "title");

        let children = reads::get_block_children(&pool, PAGE_A, None, Some(10))
            .await
            .unwrap();
        let children_json = serde_json::to_value(children).unwrap();
        assert_eq!(children_json["results"][0]["id"], BLOCK_A);
        assert_eq!(children_json["results"][1]["id"], DATABASE_A);
        assert_eq!(children_json["results"][1]["type"], "child_database");
    });
}

#[test]
fn linked_database_view_shares_source_and_keeps_view_settings_independent() {
    crate::test_block_on(async {
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
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                ],
                hidden_property_ids: vec!["details".to_string()],
            },
        )
        .await
        .unwrap();
        data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(DATABASE_A),
            Some(DATABASE_VIEW_A),
            NoteDataSourceTableViewUpdate {
                filter: vec![],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "estimate".to_string(),
                    direction: "descending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "details".to_string(),
                        "estimate".to_string(),
                    ],
                    hidden_property_ids: vec!["details".to_string()],
                    column_widths: json!({ "title": 260, "details": 180, "estimate": 120 }),
                    row_open_mode: "side_panel".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let linked = databases::create_linked_database_view(
            &pool,
            NoteLinkedDatabaseCreate {
                id: LINKED_DATABASE_A.to_string(),
                view_id: LINKED_DATABASE_VIEW_A.to_string(),
                source_block_id: DATABASE_A.to_string(),
                title: Some("Task mirror".to_string()),
            },
        )
        .await
        .unwrap();
        let linked_json = serde_json::to_value(linked).unwrap();
        assert_eq!(linked_json["database"]["id"], LINKED_DATABASE_A);
        assert_eq!(
            linked_json["data_source"]["parent"]["database_id"],
            DATABASE_A
        );
        assert_eq!(
            linked_json["view"]["parent"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(linked_json["view"]["data_source_id"], DATA_SOURCE_A);
        assert_eq!(
            linked_json["block"]["child_database"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(
            linked_json["block"]["child_database"]["data_source_id"],
            DATA_SOURCE_A
        );

        let linked_data_source_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM notes_data_sources WHERE database_id = ?")
                .bind(LINKED_DATABASE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(linked_data_source_count, 0);

        data_source_table::update_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
            NoteDataSourceTableViewUpdate {
                filter: vec![],
                sorts: vec![NoteDataSourceTableSort {
                    property_id: "title".to_string(),
                    direction: "ascending".to_string(),
                }],
                configuration: NoteDataSourceTableConfigurationUpdate {
                    property_order: vec![
                        "title".to_string(),
                        "estimate".to_string(),
                        "details".to_string(),
                    ],
                    hidden_property_ids: vec!["estimate".to_string()],
                    column_widths: json!({ "title": 320, "details": 180, "estimate": 120 }),
                    row_open_mode: "full_page".to_string(),
                },
            },
        )
        .await
        .unwrap();

        let source_table = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(DATABASE_A),
            Some(DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let source_json = serde_json::to_value(source_table).unwrap();
        assert_eq!(
            source_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["details"])
        );
        assert_eq!(
            source_json["view"]["configuration"]["table"]["row_open_mode"],
            "side_panel"
        );
        assert_eq!(source_json["view"]["sorts"][0]["property_id"], "estimate");

        let linked_table = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let linked_table_json = serde_json::to_value(linked_table).unwrap();
        assert_eq!(
            linked_table_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["estimate"])
        );
        assert_eq!(
            linked_table_json["view"]["configuration"]["table"]["row_open_mode"],
            "full_page"
        );
        assert_eq!(
            linked_table_json["view"]["sorts"][0]["property_id"],
            "title"
        );

        data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_VIEW_A),
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
                    "Due": {
                        "id": "due",
                        "name": "Due",
                        "type": "date",
                        "date": {}
                    }
                }),
                property_order: vec![
                    "title".to_string(),
                    "estimate".to_string(),
                    "details".to_string(),
                    "due".to_string(),
                ],
                hidden_property_ids: vec!["estimate".to_string()],
            },
        )
        .await
        .unwrap();

        let reloaded_linked = data_source_table::get_data_source_table_view(
            &pool,
            DATA_SOURCE_A,
            Some(LINKED_DATABASE_A),
            Some(LINKED_DATABASE_VIEW_A),
        )
        .await
        .unwrap();
        let reloaded_linked_json = serde_json::to_value(reloaded_linked).unwrap();
        assert_eq!(
            reloaded_linked_json["data_source"]["properties"]["Due"]["type"],
            "date"
        );
        assert_eq!(
            reloaded_linked_json["view"]["parent"]["database_id"],
            LINKED_DATABASE_A
        );
        assert_eq!(
            reloaded_linked_json["view"]["configuration"]["table"]["hidden_property_ids"],
            json!(["estimate"])
        );
    });
}
