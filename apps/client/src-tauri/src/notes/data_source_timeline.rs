use super::data_source_board::{
    board_schema, canonical_filter, canonical_sorts, generated_uuid_tx,
    load_active_data_source_and_database_tx, load_active_row_pages_tx, normalized_row_for_schema,
    parse_json, row_matches_filters, sort_rows, stored_filters, stored_sorts, BoardProperty,
};
use super::data_source_views;
use super::models::{
    NoteDataSourceRow, NoteDataSourceTimelineConfigurationUpdate, NoteDataSourceTimelineViewDto,
    NoteDataSourceTimelineViewUpdate, NoteDatabaseViewRow, NotePageRow,
};
use chrono::NaiveDate;
use serde_json::{json, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashSet;

const DEFAULT_TIMELINE_VIEW_NAME: &str = "Timeline";
const MAX_TIMELINE_CONFIGURATION_BYTES: usize = 50 * 1024;
const TIMELINE_ROW_OPEN_MODES: &[&str] = &["full_page", "side_panel"];
const TIMELINE_GROUP_PROPERTY_TYPES: &[&str] = &[
    "status",
    "select",
    "multi_select",
    "checkbox",
    "people",
    "relation",
    "date",
];

pub(in crate::notes) async fn get_data_source_timeline_view(
    pool: &SqlitePool,
    data_source_id: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<NoteDataSourceTimelineViewDto, String> {
    data_source_views::validate_view_scope(data_source_id, database_id, view_id)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source timeline read: {e}"))?;
    let dto = load_timeline_view_tx(&mut tx, data_source_id, database_id, view_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source timeline read: {e}"))?;
    Ok(dto)
}

pub(in crate::notes) async fn update_data_source_timeline_view(
    pool: &SqlitePool,
    data_source_id: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
    update: NoteDataSourceTimelineViewUpdate,
) -> Result<NoteDataSourceTimelineViewDto, String> {
    data_source_views::validate_view_scope(data_source_id, database_id, view_id)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source timeline view update: {e}"))?;
    let (data_source, _database) =
        load_active_data_source_and_database_tx(&mut tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let property_ids: HashSet<String> = schema.iter().map(|property| property.id.clone()).collect();
    let filter = canonical_filter(&update.filter, &property_ids)?;
    let sorts = canonical_sorts(&update.sorts, &property_ids)?;
    let configuration = canonical_timeline_configuration(&update.configuration, &schema)?;
    let view =
        ensure_timeline_view_row_tx(&mut tx, &data_source, database_id, view_id, &schema).await?;
    sqlx::query(
        "UPDATE notes_database_views
         SET filter = ?,
             sorts = ?,
             configuration = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(filter.map(|value| value.to_string()))
    .bind(sorts.to_string())
    .bind(configuration.to_string())
    .bind(&view.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update notes data source timeline view: {e}"))?;
    let dto = load_timeline_view_tx(&mut tx, data_source_id, database_id, Some(&view.id)).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source timeline view update: {e}"))?;
    Ok(dto)
}

async fn load_timeline_view_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<NoteDataSourceTimelineViewDto, String> {
    let (data_source, database) =
        load_active_data_source_and_database_tx(tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let view = ensure_timeline_view_row_tx(tx, &data_source, database_id, view_id, &schema).await?;
    let configuration = timeline_configuration(view.configuration.as_deref(), &schema)?;
    let filters = stored_filters(view.filter.as_deref())?;
    let sorts = stored_sorts(&view.sorts)?;
    let mut rows = load_active_row_pages_tx(tx, data_source_id).await?;
    rows = rows
        .into_iter()
        .map(|row| normalized_row_for_schema(row, &schema))
        .collect::<Result<Vec<_>, _>>()?;
    rows.retain(|row| row_matches_filters(row, &schema, &filters));
    if let Some(date_property_id) = configuration.date_property_id.as_deref() {
        rows.retain(|row| row_overlaps_range(row, date_property_id, &configuration.range));
    } else {
        rows.clear();
    }
    sort_rows(&mut rows, &schema, &sorts);
    NoteDataSourceTimelineViewDto::new(data_source, database, view, rows)
}

async fn ensure_timeline_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source: &NoteDataSourceRow,
    database_id: Option<&str>,
    view_id: Option<&str>,
    schema: &[BoardProperty],
) -> Result<NoteDatabaseViewRow, String> {
    if let Some(view) = load_timeline_view_row_tx(tx, &data_source.id, database_id, view_id).await?
    {
        return Ok(view);
    }
    let database_id = data_source_views::scoped_database_id(data_source, database_id);
    let id = generated_uuid_tx(tx).await?;
    let sort_order = data_source_views::next_view_sort_order_tx(tx, database_id).await?;
    let (range_start, range_end) = current_month_range_tx(tx).await?;
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
         VALUES (?, ?, ?, ?, 'timeline', '[]', ?, ?)",
    )
    .bind(&id)
    .bind(database_id)
    .bind(&data_source.id)
    .bind(DEFAULT_TIMELINE_VIEW_NAME)
    .bind(default_timeline_configuration(schema, &range_start, &range_end).to_string())
    .bind(sort_order)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert notes timeline view: {e}"))?;
    load_timeline_view_row_tx(tx, &data_source.id, Some(database_id), Some(&id))
        .await?
        .ok_or_else(|| "inserted timeline view was not found".to_string())
}

async fn load_timeline_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<Option<NoteDatabaseViewRow>, String> {
    data_source_views::load_scoped_view_row_tx(tx, data_source_id, "timeline", database_id, view_id)
        .await
}

async fn current_month_range_tx(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(String, String), String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT date('now', 'start of month'), date('now', 'start of month', '+1 month', '-1 day')",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("prepare notes timeline default range: {e}"))
}

fn default_timeline_configuration(
    schema: &[BoardProperty],
    range_start: &str,
    range_end: &str,
) -> Value {
    let date_property_id = default_date_property_id(schema);
    let visible_property_ids =
        visible_timeline_property_ids(schema, date_property_id.as_deref(), None);
    json!({
        "type": "timeline",
        "timeline": {
            "date_property_id": date_property_id,
            "group_property_id": Value::Null,
            "group_order": [],
            "hidden_group_ids": [],
            "range_start": range_start,
            "range_end": range_end,
            "visible_property_ids": visible_property_ids,
            "row_open_mode": "side_panel"
        }
    })
}

fn canonical_timeline_configuration(
    update: &NoteDataSourceTimelineConfigurationUpdate,
    schema: &[BoardProperty],
) -> Result<Value, String> {
    let date_property_id = canonical_date_property_id(update.date_property_id.as_deref(), schema)?;
    let group_property_id =
        canonical_group_property_id(update.group_property_id.as_deref(), schema)?;
    let range = canonical_range(&update.range_start, &update.range_end)?;
    let visible_property_ids = canonical_visible_property_ids(
        &update.visible_property_ids,
        schema,
        date_property_id.as_deref(),
        group_property_id.as_deref(),
    );
    let row_open_mode = update.row_open_mode.trim();
    if !TIMELINE_ROW_OPEN_MODES.contains(&row_open_mode) {
        return Err("timeline row_open_mode is not supported".to_string());
    }
    let value = json!({
        "type": "timeline",
        "timeline": {
            "date_property_id": date_property_id,
            "group_property_id": group_property_id,
            "group_order": unique_strings(&update.group_order),
            "hidden_group_ids": unique_strings(&update.hidden_group_ids),
            "range_start": range.start.to_string(),
            "range_end": range.end.to_string(),
            "visible_property_ids": visible_property_ids,
            "row_open_mode": row_open_mode
        }
    });
    if value.to_string().len() > MAX_TIMELINE_CONFIGURATION_BYTES {
        return Err("timeline configuration must not exceed 50KB".to_string());
    }
    Ok(value)
}

fn timeline_configuration(
    configuration: Option<&str>,
    schema: &[BoardProperty],
) -> Result<TimelineConfiguration, String> {
    let value = configuration
        .map(|configuration| parse_json(configuration, "timeline view configuration"))
        .transpose()?
        .ok_or_else(|| "timeline view configuration is required".to_string())?;
    let timeline = value
        .get("timeline")
        .and_then(Value::as_object)
        .ok_or_else(|| "timeline view configuration must contain timeline".to_string())?;
    let date_property_id = timeline
        .get("date_property_id")
        .and_then(Value::as_str)
        .filter(|id| is_date_property_id(id, schema))
        .map(str::to_string)
        .or_else(|| default_date_property_id(schema));
    if let Some(group_property_id) = timeline.get("group_property_id").and_then(Value::as_str) {
        validate_group_property_id(group_property_id, schema)?;
    }
    let range_start = timeline
        .get("range_start")
        .and_then(Value::as_str)
        .ok_or_else(|| "timeline range_start is required".to_string())?;
    let range_end = timeline
        .get("range_end")
        .and_then(Value::as_str)
        .ok_or_else(|| "timeline range_end is required".to_string())?;
    let range = canonical_range(range_start, range_end)?;
    let row_open_mode = timeline
        .get("row_open_mode")
        .and_then(Value::as_str)
        .unwrap_or("side_panel");
    if !TIMELINE_ROW_OPEN_MODES.contains(&row_open_mode) {
        return Err("timeline row_open_mode is not supported".to_string());
    }
    Ok(TimelineConfiguration {
        date_property_id,
        range,
    })
}

fn canonical_date_property_id(
    raw_property_id: Option<&str>,
    schema: &[BoardProperty],
) -> Result<Option<String>, String> {
    let Some(property_id) = raw_property_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Ok(None);
    };
    let property = schema
        .iter()
        .find(|property| property.id == property_id)
        .ok_or_else(|| "timeline date property references an unknown property".to_string())?;
    if property.property_type != "date" {
        return Err("timeline date property must be a date property".to_string());
    }
    Ok(Some(property_id.to_string()))
}

fn canonical_group_property_id(
    raw_property_id: Option<&str>,
    schema: &[BoardProperty],
) -> Result<Option<String>, String> {
    let Some(property_id) = raw_property_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Ok(None);
    };
    validate_group_property_id(property_id, schema)?;
    Ok(Some(property_id.to_string()))
}

fn validate_group_property_id(property_id: &str, schema: &[BoardProperty]) -> Result<(), String> {
    let property = schema
        .iter()
        .find(|property| property.id == property_id)
        .ok_or_else(|| "timeline group property references an unknown property".to_string())?;
    if !TIMELINE_GROUP_PROPERTY_TYPES.contains(&property.property_type.as_str()) {
        return Err("timeline group property type is not supported".to_string());
    }
    Ok(())
}

fn default_date_property_id(schema: &[BoardProperty]) -> Option<String> {
    schema
        .iter()
        .find(|property| property.property_type == "date")
        .map(|property| property.id.clone())
}

fn is_date_property_id(property_id: &str, schema: &[BoardProperty]) -> bool {
    schema
        .iter()
        .any(|property| property.id == property_id && property.property_type == "date")
}

fn visible_timeline_property_ids(
    schema: &[BoardProperty],
    date_property_id: Option<&str>,
    group_property_id: Option<&str>,
) -> Vec<String> {
    schema
        .iter()
        .filter(|property| property.property_type != "title")
        .filter(|property| Some(property.id.as_str()) != date_property_id)
        .filter(|property| Some(property.id.as_str()) != group_property_id)
        .take(4)
        .map(|property| property.id.clone())
        .collect()
}

fn canonical_visible_property_ids(
    property_ids: &[String],
    schema: &[BoardProperty],
    date_property_id: Option<&str>,
    group_property_id: Option<&str>,
) -> Vec<String> {
    let known: HashSet<&str> = schema.iter().map(|property| property.id.as_str()).collect();
    let mut seen = HashSet::new();
    let mut visible = Vec::new();
    for property_id in property_ids {
        let property_id = property_id.trim();
        if property_id.is_empty()
            || property_id == "title"
            || Some(property_id) == date_property_id
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

fn canonical_range(range_start: &str, range_end: &str) -> Result<TimelineRange, String> {
    let start = parse_iso_date(range_start, "timeline range_start")?;
    let end = parse_iso_date(range_end, "timeline range_end")?;
    if end < start {
        return Err("timeline range_end must be on or after range_start".to_string());
    }
    Ok(TimelineRange { start, end })
}

fn row_overlaps_range(row: &NotePageRow, date_property_id: &str, range: &TimelineRange) -> bool {
    let Ok(properties) = parse_json(&row.properties, "row page properties") else {
        return false;
    };
    let Some(date_payload) = properties
        .as_object()
        .and_then(|properties| {
            properties.values().find_map(|property| {
                let object = property.as_object()?;
                if object.get("id").and_then(Value::as_str)? != date_property_id {
                    return None;
                }
                if object.get("type").and_then(Value::as_str)? != "date" {
                    return None;
                }
                object.get("date")
            })
        })
        .and_then(Value::as_object)
    else {
        return false;
    };
    let Some(start) = date_payload
        .get("start")
        .and_then(Value::as_str)
        .and_then(date_prefix)
    else {
        return false;
    };
    let end = date_payload
        .get("end")
        .and_then(Value::as_str)
        .and_then(date_prefix)
        .unwrap_or(start);
    start <= range.end && end >= range.start
}

fn date_prefix(value: &str) -> Option<NaiveDate> {
    let prefix = value.get(..10)?;
    parse_iso_date(prefix, "date").ok()
}

fn parse_iso_date(value: &str, label: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| format!("{label} must be an ISO date"))
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

struct TimelineConfiguration {
    date_property_id: Option<String>,
    range: TimelineRange,
}

struct TimelineRange {
    start: NaiveDate,
    end: NaiveDate,
}
