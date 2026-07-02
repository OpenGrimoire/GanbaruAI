use super::data_source_board::{
    board_schema, canonical_filter, canonical_sorts, generated_uuid_tx,
    load_active_data_source_and_database_tx, load_active_row_pages_tx, normalized_row_for_schema,
    parse_json, row_matches_filters, sort_rows, stored_filters, stored_sorts, BoardProperty,
};
use super::models::{
    NoteDataSourceListConfigurationUpdate, NoteDataSourceListViewDto, NoteDataSourceListViewUpdate,
    NoteDataSourceRow, NoteDatabaseViewRow,
};
use super::validation::require_uuid;
use serde_json::{json, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashSet;

const DEFAULT_LIST_VIEW_NAME: &str = "List";
const MAX_LIST_CONFIGURATION_BYTES: usize = 50 * 1024;
const LIST_ROW_OPEN_MODES: &[&str] = &["full_page", "side_panel"];
const LIST_GROUP_PROPERTY_TYPES: &[&str] = &[
    "status",
    "select",
    "multi_select",
    "checkbox",
    "people",
    "relation",
    "date",
];

pub(in crate::notes) async fn get_data_source_list_view(
    pool: &SqlitePool,
    data_source_id: &str,
) -> Result<NoteDataSourceListViewDto, String> {
    require_uuid(data_source_id, "data_source_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source list read: {e}"))?;
    let dto = load_list_view_tx(&mut tx, data_source_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source list read: {e}"))?;
    Ok(dto)
}

pub(in crate::notes) async fn update_data_source_list_view(
    pool: &SqlitePool,
    data_source_id: &str,
    update: NoteDataSourceListViewUpdate,
) -> Result<NoteDataSourceListViewDto, String> {
    require_uuid(data_source_id, "data_source_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source list view update: {e}"))?;
    let (data_source, _database) =
        load_active_data_source_and_database_tx(&mut tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let property_ids: HashSet<String> = schema.iter().map(|property| property.id.clone()).collect();
    let filter = canonical_filter(&update.filter, &property_ids)?;
    let sorts = canonical_sorts(&update.sorts, &property_ids)?;
    let configuration = canonical_list_configuration(&update.configuration, &schema)?;
    ensure_list_view_row_tx(&mut tx, &data_source, &schema).await?;
    sqlx::query(
        "UPDATE notes_database_views
         SET filter = ?,
             sorts = ?,
             configuration = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE data_source_id = ? AND type = 'list'",
    )
    .bind(filter.map(|value| value.to_string()))
    .bind(sorts.to_string())
    .bind(configuration.to_string())
    .bind(data_source_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update notes data source list view: {e}"))?;
    let dto = load_list_view_tx(&mut tx, data_source_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source list view update: {e}"))?;
    Ok(dto)
}

async fn load_list_view_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
) -> Result<NoteDataSourceListViewDto, String> {
    let (data_source, database) =
        load_active_data_source_and_database_tx(tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let view = ensure_list_view_row_tx(tx, &data_source, &schema).await?;
    validate_list_configuration(view.configuration.as_deref(), &schema)?;
    let filters = stored_filters(view.filter.as_deref())?;
    let sorts = stored_sorts(&view.sorts)?;
    let mut rows = load_active_row_pages_tx(tx, data_source_id).await?;
    rows = rows
        .into_iter()
        .map(|row| normalized_row_for_schema(row, &schema))
        .collect::<Result<Vec<_>, _>>()?;
    rows.retain(|row| row_matches_filters(row, &schema, &filters));
    sort_rows(&mut rows, &schema, &sorts);
    NoteDataSourceListViewDto::new(data_source, database, view, rows)
}

async fn ensure_list_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source: &NoteDataSourceRow,
    schema: &[BoardProperty],
) -> Result<NoteDatabaseViewRow, String> {
    if let Some(view) = load_list_view_row_tx(tx, &data_source.id).await? {
        return Ok(view);
    }
    let id = generated_uuid_tx(tx).await?;
    let sort_order: f64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 1)
         FROM notes_database_views
         WHERE database_id = ?",
    )
    .bind(&data_source.database_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("prepare notes list view order: {e}"))?;
    sqlx::query(
        "INSERT INTO notes_database_views (
            id,
            database_id,
            data_source_id,
            name,
            type,
            sorts,
            configuration,
            sort_order
         )
         VALUES (?, ?, ?, ?, 'list', '[]', ?, ?)",
    )
    .bind(&id)
    .bind(&data_source.database_id)
    .bind(&data_source.id)
    .bind(DEFAULT_LIST_VIEW_NAME)
    .bind(default_list_configuration(schema).to_string())
    .bind(sort_order)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert notes list view: {e}"))?;
    load_list_view_row_tx(tx, &data_source.id)
        .await?
        .ok_or_else(|| "inserted list view was not found".to_string())
}

async fn load_list_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
) -> Result<Option<NoteDatabaseViewRow>, String> {
    sqlx::query_as::<_, NoteDatabaseViewRow>(
        "SELECT id,
                database_id,
                data_source_id,
                name,
                type AS view_type,
                filter,
                sorts,
                configuration,
                source_provider,
                source_object_id,
                source_workspace_id,
                source_last_edited_time,
                url,
                created_time,
                last_edited_time
         FROM notes_database_views
         WHERE data_source_id = ? AND type = 'list'
         ORDER BY sort_order ASC, created_time ASC, id ASC
         LIMIT 1",
    )
    .bind(data_source_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes list view: {e}"))
}

fn default_list_configuration(schema: &[BoardProperty]) -> Value {
    json!({
        "type": "list",
        "list": {
            "group_property_id": null,
            "group_order": [],
            "hidden_group_ids": [],
            "visible_property_ids": visible_list_property_ids(schema, None),
            "row_open_mode": "side_panel"
        }
    })
}

fn canonical_list_configuration(
    update: &NoteDataSourceListConfigurationUpdate,
    schema: &[BoardProperty],
) -> Result<Value, String> {
    let group_property_id =
        canonical_group_property_id(update.group_property_id.as_deref(), schema)?;
    let visible_property_ids = canonical_visible_property_ids(
        &update.visible_property_ids,
        schema,
        group_property_id.as_deref(),
    );
    let row_open_mode = update.row_open_mode.trim();
    if !LIST_ROW_OPEN_MODES.contains(&row_open_mode) {
        return Err("list row_open_mode is not supported".to_string());
    }
    let value = json!({
        "type": "list",
        "list": {
            "group_property_id": group_property_id,
            "group_order": unique_strings(&update.group_order),
            "hidden_group_ids": unique_strings(&update.hidden_group_ids),
            "visible_property_ids": visible_property_ids,
            "row_open_mode": row_open_mode
        }
    });
    if value.to_string().len() > MAX_LIST_CONFIGURATION_BYTES {
        return Err("list configuration must not exceed 50KB".to_string());
    }
    Ok(value)
}

fn validate_list_configuration(
    configuration: Option<&str>,
    schema: &[BoardProperty],
) -> Result<(), String> {
    let value = configuration
        .map(|configuration| parse_json(configuration, "list view configuration"))
        .transpose()?
        .unwrap_or_else(|| default_list_configuration(schema));
    let list = value
        .get("list")
        .and_then(Value::as_object)
        .ok_or_else(|| "list view configuration must contain list".to_string())?;
    canonical_group_property_id(
        list.get("group_property_id").and_then(Value::as_str),
        schema,
    )?;
    let row_open_mode = list
        .get("row_open_mode")
        .and_then(Value::as_str)
        .unwrap_or("side_panel");
    if !LIST_ROW_OPEN_MODES.contains(&row_open_mode) {
        return Err("list row_open_mode is not supported".to_string());
    }
    Ok(())
}

fn canonical_group_property_id(
    raw_property_id: Option<&str>,
    schema: &[BoardProperty],
) -> Result<Option<String>, String> {
    let Some(property_id) = raw_property_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Ok(None);
    };
    let property = schema
        .iter()
        .find(|property| property.id == property_id)
        .ok_or_else(|| "list group property references an unknown property".to_string())?;
    if !LIST_GROUP_PROPERTY_TYPES.contains(&property.property_type.as_str()) {
        return Err("list group property type is not supported".to_string());
    }
    Ok(Some(property_id.to_string()))
}

fn visible_list_property_ids(
    schema: &[BoardProperty],
    group_property_id: Option<&str>,
) -> Vec<String> {
    schema
        .iter()
        .filter(|property| property.property_type != "title")
        .filter(|property| Some(property.id.as_str()) != group_property_id)
        .take(4)
        .map(|property| property.id.clone())
        .collect()
}

fn canonical_visible_property_ids(
    property_ids: &[String],
    schema: &[BoardProperty],
    group_property_id: Option<&str>,
) -> Vec<String> {
    let known: HashSet<&str> = schema.iter().map(|property| property.id.as_str()).collect();
    let mut seen = HashSet::new();
    let mut visible = Vec::new();
    for property_id in property_ids {
        let property_id = property_id.trim();
        if property_id.is_empty()
            || property_id == "title"
            || Some(property_id) == group_property_id
            || !known.contains(property_id)
        {
            continue;
        }
        if seen.insert(property_id.to_string()) {
            visible.push(property_id.to_string());
        }
    }
    visible
}

fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() || !seen.insert(value.to_string()) {
            continue;
        }
        unique.push(value.to_string());
    }
    unique
}
