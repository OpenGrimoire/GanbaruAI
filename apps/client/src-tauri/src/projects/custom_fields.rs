use super::history::{bool_to_string, insert_task_change_event_owned};
use super::models::{
    ProjectCustomFieldOptionRow, ProjectCustomFieldRow, ProjectCustomFieldValueUpdate,
};
use super::validation::validate_date;
use std::collections::HashSet;

async fn custom_field_for_task_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field_id: &str,
) -> Result<ProjectCustomFieldRow, String> {
    sqlx::query_as::<_, ProjectCustomFieldRow>(
        "SELECT cf.*
         FROM project_custom_fields cf
         JOIN project_tasks pt ON pt.project_id = cf.project_id
         WHERE pt.id = ? AND cf.id = ?",
    )
    .bind(task_id.trim())
    .bind(field_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project custom field: {e}"))?
    .ok_or_else(|| "custom field must belong to the task project".to_string())
}

async fn custom_field_by_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
) -> Result<ProjectCustomFieldRow, String> {
    sqlx::query_as::<_, ProjectCustomFieldRow>("SELECT * FROM project_custom_fields WHERE id = ?")
        .bind(field_id.trim())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load project custom field: {e}"))?
        .ok_or_else(|| "project custom field not found".to_string())
}

async fn custom_field_option_with_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    option_id: &str,
) -> Result<(ProjectCustomFieldOptionRow, ProjectCustomFieldRow), String> {
    let option = sqlx::query_as::<_, ProjectCustomFieldOptionRow>(
        "SELECT * FROM project_custom_field_options WHERE id = ?",
    )
    .bind(option_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load project custom field option: {e}"))?
    .ok_or_else(|| "project custom field option not found".to_string())?;
    let field = custom_field_by_id(tx, &option.field_id).await?;
    Ok((option, field))
}

pub(in crate::projects) async fn ensure_custom_field_accepts_options_in_pool(
    pool: &sqlx::SqlitePool,
    field_id: &str,
) -> Result<(), String> {
    let field_type = sqlx::query_scalar::<_, String>(
        "SELECT field_type FROM project_custom_fields WHERE id = ?",
    )
    .bind(field_id.trim())
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load project custom field: {e}"))?
    .ok_or_else(|| "project custom field not found".to_string())?;
    if field_type != "select" && field_type != "multi_select" {
        return Err("custom field options require select or multi-select field".to_string());
    }
    Ok(())
}

async fn ensure_options_belong_to_field(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
    option_ids: &[String],
) -> Result<Vec<String>, String> {
    let mut seen = HashSet::new();
    let mut unique_option_ids = Vec::new();
    for option_id in option_ids {
        let trimmed = option_id.trim().to_string();
        if seen.insert(trimmed.clone()) {
            unique_option_ids.push(trimmed);
        }
    }
    for option_id in &unique_option_ids {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM project_custom_field_options
             WHERE id = ? AND field_id = ?",
        )
        .bind(option_id)
        .bind(field_id.trim())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("load project custom field option: {e}"))?;
        if exists == 0 {
            return Err("custom field option must belong to the field".to_string());
        }
    }
    Ok(unique_option_ids)
}

fn custom_field_history_name(field: &ProjectCustomFieldRow) -> String {
    format!("custom_field:{}", field.name)
}

async fn custom_field_value_label(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field: &ProjectCustomFieldRow,
) -> Result<Option<String>, String> {
    match field.field_type.as_str() {
        "text" | "url" => sqlx::query_scalar::<_, String>(
            "SELECT text_value
             FROM project_custom_field_values
             WHERE task_id = ? AND field_id = ?",
        )
        .bind(task_id)
        .bind(&field.id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load custom field text value: {e}")),
        "number" => {
            let value = sqlx::query_scalar::<_, f64>(
                "SELECT number_value
                 FROM project_custom_field_values
                 WHERE task_id = ? AND field_id = ?",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load custom field number value: {e}"))?;
            Ok(value.map(|number| {
                if number.fract() == 0.0 {
                    format!("{number:.0}")
                } else {
                    number.to_string()
                }
            }))
        }
        "date" => sqlx::query_scalar::<_, String>(
            "SELECT date_value
             FROM project_custom_field_values
             WHERE task_id = ? AND field_id = ?",
        )
        .bind(task_id)
        .bind(&field.id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load custom field date value: {e}")),
        "checkbox" => {
            let value = sqlx::query_scalar::<_, i64>(
                "SELECT checkbox_value
                 FROM project_custom_field_values
                 WHERE task_id = ? AND field_id = ?",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load custom field checkbox value: {e}"))?;
            Ok(value.map(|checked| bool_to_string(checked != 0)))
        }
        "select" | "multi_select" => {
            let option_names = sqlx::query_scalar::<_, String>(
                "SELECT opt.name
                 FROM project_custom_field_option_values val
                 JOIN project_custom_field_options opt ON opt.id = val.option_id
                 WHERE val.task_id = ? AND val.field_id = ?
                 ORDER BY opt.sort_order ASC, opt.name ASC",
            )
            .bind(task_id)
            .bind(&field.id)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| format!("load custom field option values: {e}"))?;
            if option_names.is_empty() {
                Ok(None)
            } else {
                Ok(Some(option_names.join(", ")))
            }
        }
        _ => Err("unsupported custom field type".to_string()),
    }
}

fn ensure_custom_field_value_payload_matches_type(
    field_type: &str,
    value: &ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
    let has_text = value
        .text_value
        .as_ref()
        .is_some_and(|text| !text.trim().is_empty());
    let has_date = value
        .date_value
        .as_ref()
        .is_some_and(|date| !date.trim().is_empty());
    match field_type {
        "text" | "url" => {
            if value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("text custom fields only accept text values".to_string());
            }
        }
        "number" => {
            if value.text_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("number custom fields only accept number values".to_string());
            }
        }
        "date" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.checkbox_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("date custom fields only accept date values".to_string());
            }
        }
        "checkbox" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || !value.option_ids.is_empty()
            {
                return Err("checkbox custom fields only accept checkbox values".to_string());
            }
        }
        "select" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
                || value.option_ids.len() > 1
            {
                return Err("select custom fields accept one option".to_string());
            }
        }
        "multi_select" => {
            if value.text_value.is_some()
                || value.number_value.is_some()
                || value.date_value.is_some()
                || value.checkbox_value.is_some()
            {
                return Err("multi-select custom fields only accept option values".to_string());
            }
        }
        _ => return Err("unsupported custom field type".to_string()),
    }
    if field_type != "text" && field_type != "url" && has_text {
        return Err("text_value is not valid for this custom field".to_string());
    }
    if field_type != "date" && has_date {
        return Err("date_value is not valid for this custom field".to_string());
    }
    Ok(())
}

async fn upsert_scalar_custom_field_value(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    task_id: &str,
    field_id: &str,
    text_value: Option<&str>,
    number_value: Option<f64>,
    date_value: Option<&str>,
    checkbox_value: Option<i64>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO project_custom_field_values (
            task_id, field_id, text_value, number_value, date_value, checkbox_value
         )
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(task_id, field_id) DO UPDATE SET
            text_value = excluded.text_value,
            number_value = excluded.number_value,
            date_value = excluded.date_value,
            checkbox_value = excluded.checkbox_value,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(task_id)
    .bind(field_id)
    .bind(text_value)
    .bind(number_value)
    .bind(date_value)
    .bind(checkbox_value)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("update custom field value: {e}"))?;
    Ok(())
}

pub(in crate::projects) async fn update_custom_field_value_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    value: &ProjectCustomFieldValueUpdate,
) -> Result<(), String> {
    let field = custom_field_for_task_value(tx, &value.task_id, &value.field_id).await?;
    ensure_custom_field_value_payload_matches_type(&field.field_type, value)?;
    let old_value = custom_field_value_label(tx, &value.task_id, &field).await?;
    sqlx::query("DELETE FROM project_custom_field_values WHERE task_id = ? AND field_id = ?")
        .bind(value.task_id.trim())
        .bind(value.field_id.trim())
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("clear custom field value: {e}"))?;
    sqlx::query(
        "DELETE FROM project_custom_field_option_values WHERE task_id = ? AND field_id = ?",
    )
    .bind(value.task_id.trim())
    .bind(value.field_id.trim())
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("clear custom field option values: {e}"))?;

    match field.field_type.as_str() {
        "text" | "url" => {
            if let Some(text) = value.text_value.as_ref().map(|text| text.trim()) {
                if !text.is_empty() {
                    upsert_scalar_custom_field_value(
                        tx,
                        value.task_id.trim(),
                        value.field_id.trim(),
                        Some(text),
                        None,
                        None,
                        None,
                    )
                    .await?;
                }
            }
        }
        "number" => {
            if let Some(number) = value.number_value {
                upsert_scalar_custom_field_value(
                    tx,
                    value.task_id.trim(),
                    value.field_id.trim(),
                    None,
                    Some(number),
                    None,
                    None,
                )
                .await?;
            }
        }
        "date" => {
            if let Some(date) = value.date_value.as_ref().map(|date| date.trim()) {
                if !date.is_empty() {
                    validate_date(date, "date_value")?;
                    upsert_scalar_custom_field_value(
                        tx,
                        value.task_id.trim(),
                        value.field_id.trim(),
                        None,
                        None,
                        Some(date),
                        None,
                    )
                    .await?;
                }
            }
        }
        "checkbox" => {
            if let Some(checked) = value.checkbox_value {
                upsert_scalar_custom_field_value(
                    tx,
                    value.task_id.trim(),
                    value.field_id.trim(),
                    None,
                    None,
                    None,
                    Some(if checked { 1 } else { 0 }),
                )
                .await?;
            }
        }
        "select" | "multi_select" => {
            let option_ids =
                ensure_options_belong_to_field(tx, &field.id, &value.option_ids).await?;
            for option_id in option_ids {
                sqlx::query(
                    "INSERT INTO project_custom_field_option_values (task_id, field_id, option_id)
                     VALUES (?, ?, ?)",
                )
                .bind(value.task_id.trim())
                .bind(value.field_id.trim())
                .bind(option_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("update custom field option value: {e}"))?;
            }
        }
        _ => return Err("unsupported custom field type".to_string()),
    }

    let new_value = custom_field_value_label(tx, &value.task_id, &field).await?;
    if old_value != new_value {
        insert_task_change_event_owned(
            tx,
            value.task_id.trim(),
            "updated",
            &custom_field_history_name(&field),
            old_value,
            new_value,
            None,
        )
        .await?;
    }
    Ok(())
}

pub(in crate::projects) async fn delete_custom_field_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    field_id: &str,
) -> Result<(), String> {
    let field = custom_field_by_id(tx, field_id).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id FROM project_custom_field_values WHERE field_id = ?
         UNION
         SELECT task_id FROM project_custom_field_option_values WHERE field_id = ?",
    )
    .bind(field_id)
    .bind(field_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load custom field task values: {e}"))?;
    let mut old_values = Vec::new();
    for task_id in task_ids {
        let old_value = custom_field_value_label(tx, &task_id, &field).await?;
        old_values.push((task_id, old_value));
    }
    sqlx::query("DELETE FROM project_custom_fields WHERE id = ?")
        .bind(field_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project custom field: {e}"))?;
    for (task_id, old_value) in old_values {
        if old_value.is_some() {
            insert_task_change_event_owned(
                tx,
                &task_id,
                "updated",
                &custom_field_history_name(&field),
                old_value,
                None,
                None,
            )
            .await?;
        }
    }
    Ok(())
}

pub(in crate::projects) async fn delete_custom_field_option_with_history(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    option_id: &str,
) -> Result<(), String> {
    let (_option, field) = custom_field_option_with_field(tx, option_id).await?;
    let task_ids = sqlx::query_scalar::<_, String>(
        "SELECT task_id
         FROM project_custom_field_option_values
         WHERE option_id = ?
         ORDER BY created_at ASC",
    )
    .bind(option_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load custom field option tasks: {e}"))?;
    let mut old_values = Vec::new();
    for task_id in task_ids {
        let old_value = custom_field_value_label(tx, &task_id, &field).await?;
        old_values.push((task_id, old_value));
    }
    sqlx::query("DELETE FROM project_custom_field_options WHERE id = ?")
        .bind(option_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("delete project custom field option: {e}"))?;
    for (task_id, old_value) in old_values {
        let new_value = custom_field_value_label(tx, &task_id, &field).await?;
        if old_value != new_value {
            insert_task_change_event_owned(
                tx,
                &task_id,
                "updated",
                &custom_field_history_name(&field),
                old_value,
                new_value,
                None,
            )
            .await?;
        }
    }
    Ok(())
}
