use super::*;
use crate::projects::custom_fields::update_custom_field_value_with_history;

#[test]
fn custom_field_value_rejects_fields_from_another_project() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project_graph_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO projects (id, group_id, name, icon, sort_order)
             VALUES ('project-b', 'group-a', 'Project B', 'lucide:folder', 200)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
             VALUES ('field-b', 'project-b', 'Client note', 'text', 100)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let value = ProjectCustomFieldValueUpdate {
            task_id: "task-a".to_string(),
            field_id: "field-b".to_string(),
            text_value: Some("Wrong project".to_string()),
            number_value: None,
            date_value: None,
            checkbox_value: None,
            option_ids: Vec::new(),
        };
        let mut tx = pool.begin().await.unwrap();
        let result = update_custom_field_value_with_history(&mut tx, &value).await;

        assert_eq!(
            result,
            Err("custom field must belong to the task project".to_string())
        );
    });
}

#[test]
fn custom_field_value_rejects_options_from_another_field() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project_graph_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
             VALUES
                ('field-a', 'project-a', 'Phase', 'select', 100),
                ('field-b', 'project-a', 'Risk', 'select', 200)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO project_custom_field_options (id, field_id, name, sort_order)
             VALUES ('option-b', 'field-b', 'High', 100)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let value = ProjectCustomFieldValueUpdate {
            task_id: "task-a".to_string(),
            field_id: "field-a".to_string(),
            text_value: None,
            number_value: None,
            date_value: None,
            checkbox_value: None,
            option_ids: vec!["option-b".to_string()],
        };
        let mut tx = pool.begin().await.unwrap();
        let result = update_custom_field_value_with_history(&mut tx, &value).await;

        assert_eq!(
            result,
            Err("custom field option must belong to the field".to_string())
        );
    });
}

#[test]
fn custom_field_value_records_task_history() {
    tauri::async_runtime::block_on(async {
        let pool = migrated_memory_pool().await;
        insert_project_graph_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO project_custom_fields (id, project_id, name, field_type, sort_order)
             VALUES ('field-a', 'project-a', 'Risk', 'text', 100)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let value = ProjectCustomFieldValueUpdate {
            task_id: "task-a".to_string(),
            field_id: "field-a".to_string(),
            text_value: Some("High".to_string()),
            number_value: None,
            date_value: None,
            checkbox_value: None,
            option_ids: Vec::new(),
        };
        let mut tx = pool.begin().await.unwrap();
        update_custom_field_value_with_history(&mut tx, &value)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let stored_value: String = sqlx::query_scalar(
            "SELECT text_value
             FROM project_custom_field_values
             WHERE task_id = 'task-a' AND field_id = 'field-a'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let history_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM project_task_change_events
             WHERE task_id = 'task-a'
               AND event_type = 'updated'
               AND field_name = 'custom_field:Risk'
               AND old_value IS NULL
               AND new_value = 'High'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(stored_value, "High");
        assert_eq!(history_count, 1);
    });
}
