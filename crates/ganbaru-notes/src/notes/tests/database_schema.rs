use super::helpers::*;

#[test]
fn update_local_data_source_schema() {
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

        let updated = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Name": {
                        "id": "title",
                        "name": "Name",
                        "description": "Row title",
                        "type": "title",
                        "title": {}
                    },
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "description": "",
                        "type": "rich_text",
                        "rich_text": {}
                    },
                    "Estimate": {
                        "id": "estimate",
                        "name": "Estimate",
                        "description": "",
                        "type": "number",
                        "number": { "format": "percent" }
                    },
                    "Priority": {
                        "id": "priority",
                        "name": "Priority",
                        "description": "",
                        "type": "select",
                        "select": {
                            "options": [
                                { "id": "low", "name": "Low", "color": "blue" },
                                { "id": "high", "name": "High", "color": "red" }
                            ]
                        }
                    },
                    "Tags": {
                        "id": "tags",
                        "name": "Tags",
                        "description": "",
                        "type": "multi_select",
                        "multi_select": {
                            "options": [
                                { "id": "home", "name": "Home", "color": "green" }
                            ]
                        }
                    },
                    "Status": {
                        "id": "status",
                        "name": "Status",
                        "description": "",
                        "type": "status",
                        "status": {
                            "options": [
                                {
                                    "id": "todo",
                                    "name": "Todo",
                                    "color": "default",
                                    "group": "To-do"
                                },
                                {
                                    "id": "doing",
                                    "name": "Doing",
                                    "color": "blue",
                                    "group": "In progress"
                                },
                                {
                                    "id": "done",
                                    "name": "Done",
                                    "color": "green",
                                    "group": "Complete"
                                }
                            ]
                        }
                    },
                    "Due": { "id": "due", "name": "Due", "type": "date", "date": {} },
                    "Done": {
                        "id": "done_checkbox",
                        "name": "Done",
                        "type": "checkbox",
                        "checkbox": {}
                    },
                    "URL": { "id": "url", "name": "URL", "type": "url", "url": {} },
                    "Email": { "id": "email", "name": "Email", "type": "email", "email": {} },
                    "Phone": {
                        "id": "phone",
                        "name": "Phone",
                        "type": "phone_number",
                        "phone_number": {}
                    },
                    "Files": { "id": "files", "name": "Files", "type": "files", "files": {} },
                    "People": {
                        "id": "people",
                        "name": "People",
                        "type": "people",
                        "people": {}
                    },
                    "Created": {
                        "id": "created",
                        "name": "Created",
                        "type": "created_time",
                        "created_time": {}
                    },
                    "Created by": {
                        "id": "created_by",
                        "name": "Created by",
                        "type": "created_by",
                        "created_by": {}
                    },
                    "Edited": {
                        "id": "edited",
                        "name": "Edited",
                        "type": "last_edited_time",
                        "last_edited_time": {}
                    },
                    "Edited by": {
                        "id": "edited_by",
                        "name": "Edited by",
                        "type": "last_edited_by",
                        "last_edited_by": {}
                    },
                    "Task ID": {
                        "id": "task_id",
                        "name": "Task ID",
                        "type": "unique_id",
                        "unique_id": { "prefix": "TASK" }
                    },
                    "Place": { "id": "place", "name": "Place", "type": "place", "place": {} }
                }),
                property_order: vec![
                    "title".to_string(),
                    "status".to_string(),
                    "priority".to_string(),
                    "details".to_string(),
                    "estimate".to_string(),
                    "tags".to_string(),
                    "due".to_string(),
                    "done_checkbox".to_string(),
                    "url".to_string(),
                    "email".to_string(),
                    "phone".to_string(),
                    "files".to_string(),
                    "people".to_string(),
                    "created".to_string(),
                    "created_by".to_string(),
                    "edited".to_string(),
                    "edited_by".to_string(),
                    "task_id".to_string(),
                    "place".to_string(),
                ],
                hidden_property_ids: vec!["details".to_string(), "files".to_string()],
            },
        )
        .await
        .unwrap();

        let updated_json = serde_json::to_value(updated).unwrap();
        assert_eq!(
            updated_json["data_source"]["properties"]["Priority"]["select"]["options"][1]["name"],
            "High"
        );
        assert_eq!(
            updated_json["data_source"]["properties"]["Estimate"]["number"]["format"],
            "percent"
        );
        assert_eq!(
            updated_json["data_source"]["properties"]["Task ID"]["unique_id"]["prefix"],
            "TASK"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["table"]["property_order"][1],
            "status"
        );
        assert_eq!(
            updated_json["view"]["configuration"]["table"]["hidden_property_ids"][0],
            "details"
        );

        let stored_properties: String =
            sqlx::query_scalar("SELECT properties FROM notes_data_sources WHERE id = ?")
                .bind(DATA_SOURCE_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let stored_properties_json: serde_json::Value =
            serde_json::from_str(&stored_properties).unwrap();
        assert_eq!(stored_properties_json["Place"]["type"], "place");

        let configuration: String =
            sqlx::query_scalar("SELECT configuration FROM notes_database_views WHERE id = ?")
                .bind(DATABASE_VIEW_A)
                .fetch_one(&pool)
                .await
                .unwrap();
        let configuration_json: serde_json::Value = serde_json::from_str(&configuration).unwrap();
        assert_eq!(
            configuration_json["table"]["hidden_property_ids"],
            json!(["details", "files"])
        );
    });
}

#[test]
fn update_local_data_source_schema_rejects_title_removal() {
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

        let result = data_source_schema::update_data_source_schema(
            &pool,
            DATA_SOURCE_A,
            None,
            NoteDataSourceSchemaUpdate {
                properties: json!({
                    "Details": {
                        "id": "details",
                        "name": "Details",
                        "type": "rich_text",
                        "rich_text": {}
                    }
                }),
                property_order: vec!["details".to_string()],
                hidden_property_ids: vec![],
            },
        )
        .await;

        assert_eq!(
            result.err().unwrap(),
            "data source schema must contain exactly one title property"
        );
    });
}
