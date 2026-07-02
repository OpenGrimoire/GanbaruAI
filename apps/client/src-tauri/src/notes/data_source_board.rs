use super::models::{
    NoteDataSourceBoardConfigurationUpdate, NoteDataSourceBoardGroupDto,
    NoteDataSourceBoardRowMove, NoteDataSourceBoardViewDto, NoteDataSourceBoardViewUpdate,
    NoteDataSourceRow, NoteDataSourceRowPropertyUpdate, NoteDataSourceTableFilter,
    NoteDataSourceTableSort, NoteDatabaseRow, NoteDatabaseViewRow, NotePageRow,
};
use super::validation::require_uuid;
use super::{data_source_table, writes};
use serde_json::{json, Map, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

const DEFAULT_BOARD_VIEW_NAME: &str = "Board";
const MAX_FILTERS: usize = 10;
const MAX_SORTS: usize = 5;
const MAX_BOARD_CONFIGURATION_BYTES: usize = 50 * 1024;
const MAX_FILTER_TEXT_CHARS: usize = 200;
const BOARD_EMPTY_GROUP_ID: &str = "__empty__";
const BOARD_UNGROUPED_ID: &str = "__ungrouped__";
const BOARD_ROW_OPEN_MODES: &[&str] = &["full_page", "side_panel"];
const FILTER_CONDITIONS: &[&str] = &[
    "contains",
    "equals",
    "is_empty",
    "is_not_empty",
    "checked",
    "unchecked",
];
const BOARD_GROUP_PROPERTY_TYPES: &[&str] = &[
    "status",
    "select",
    "multi_select",
    "checkbox",
    "people",
    "relation",
    "date",
];

pub(in crate::notes) async fn get_data_source_board_view(
    pool: &SqlitePool,
    data_source_id: &str,
) -> Result<NoteDataSourceBoardViewDto, String> {
    require_uuid(data_source_id, "data_source_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source board read: {e}"))?;
    let dto = load_board_view_tx(&mut tx, data_source_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source board read: {e}"))?;
    Ok(dto)
}

pub(in crate::notes) async fn update_data_source_board_view(
    pool: &SqlitePool,
    data_source_id: &str,
    update: NoteDataSourceBoardViewUpdate,
) -> Result<NoteDataSourceBoardViewDto, String> {
    require_uuid(data_source_id, "data_source_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes data source board view update: {e}"))?;
    let (data_source, _database) =
        load_active_data_source_and_database_tx(&mut tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let property_ids: HashSet<String> = schema.iter().map(|property| property.id.clone()).collect();
    let filter = canonical_filter(&update.filter, &property_ids)?;
    let sorts = canonical_sorts(&update.sorts, &property_ids)?;
    let configuration = canonical_board_configuration(&update.configuration, &schema)?;
    ensure_board_view_row_tx(&mut tx, &data_source, &schema).await?;
    sqlx::query(
        "UPDATE notes_database_views
         SET filter = ?,
             sorts = ?,
             configuration = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE data_source_id = ? AND type = 'board'",
    )
    .bind(filter.map(|value| value.to_string()))
    .bind(sorts.to_string())
    .bind(configuration.to_string())
    .bind(data_source_id.trim())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update notes data source board view: {e}"))?;
    let dto = load_board_view_tx(&mut tx, data_source_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes data source board view update: {e}"))?;
    Ok(dto)
}

pub(in crate::notes) async fn move_data_source_board_row(
    pool: &SqlitePool,
    data_source_id: &str,
    request: NoteDataSourceBoardRowMove,
) -> Result<NoteDataSourceBoardViewDto, String> {
    require_uuid(data_source_id, "data_source_id")?;
    require_uuid(&request.page_id, "page_id")?;
    let group_id = request.group_id.trim();
    if group_id.is_empty() {
        return Err("board group_id is required".to_string());
    }
    let group_property = {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| format!("begin notes data source board row move: {e}"))?;
        let (data_source, _) =
            load_active_data_source_and_database_tx(&mut tx, data_source_id).await?;
        let schema = board_schema(&parse_json(
            &data_source.properties,
            "data source properties",
        )?)?;
        let view = ensure_board_view_row_tx(&mut tx, &data_source, &schema).await?;
        let configuration = board_configuration(view.configuration.as_deref(), &schema)?;
        let group_property_id = configuration
            .group_property_id
            .as_deref()
            .ok_or_else(|| "board view has no group property".to_string())?;
        let property = schema
            .iter()
            .find(|property| property.id == group_property_id)
            .cloned()
            .ok_or_else(|| "board group property was not found".to_string())?;
        tx.commit()
            .await
            .map_err(|e| format!("commit notes data source board row move read: {e}"))?;
        property
    };
    let value = board_move_value(&group_property, group_id)?;
    data_source_table::update_data_source_row_property(
        pool,
        data_source_id,
        &request.page_id,
        NoteDataSourceRowPropertyUpdate {
            property_id: group_property.id,
            value,
        },
    )
    .await?;
    get_data_source_board_view(pool, data_source_id).await
}

async fn load_board_view_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
) -> Result<NoteDataSourceBoardViewDto, String> {
    let (data_source, database) =
        load_active_data_source_and_database_tx(tx, data_source_id).await?;
    let schema = board_schema(&parse_json(
        &data_source.properties,
        "data source properties",
    )?)?;
    let view = ensure_board_view_row_tx(tx, &data_source, &schema).await?;
    let configuration = board_configuration(view.configuration.as_deref(), &schema)?;
    let filters = stored_filters(view.filter.as_deref())?;
    let sorts = stored_sorts(&view.sorts)?;
    let mut rows = load_active_row_pages_tx(tx, data_source_id).await?;
    rows = rows
        .into_iter()
        .map(|row| normalized_row_for_schema(row, &schema))
        .collect::<Result<Vec<_>, _>>()?;
    rows.retain(|row| row_matches_filters(row, &schema, &filters));
    sort_rows(&mut rows, &schema, &sorts);
    let groups = board_groups(&schema, &configuration, rows)?;
    NoteDataSourceBoardViewDto::new(data_source, database, view, groups)
}

pub(in crate::notes) async fn load_active_data_source_and_database_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
) -> Result<(NoteDataSourceRow, NoteDatabaseRow), String> {
    let data_source = sqlx::query_as::<_, NoteDataSourceRow>(
        "SELECT data_source.*
         FROM notes_data_sources AS data_source
         JOIN notes_databases AS database ON database.id = data_source.database_id
         WHERE data_source.id = ?
           AND data_source.in_trash = 0
           AND database.in_trash = 0",
    )
    .bind(data_source_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes data source for board: {e}"))?
    .ok_or_else(|| "data source not found".to_string())?;
    let database = sqlx::query_as::<_, NoteDatabaseRow>(
        "SELECT * FROM notes_databases WHERE id = ? AND in_trash = 0",
    )
    .bind(&data_source.database_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("load notes database for board: {e}"))?;
    Ok((data_source, database))
}

async fn ensure_board_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source: &NoteDataSourceRow,
    schema: &[BoardProperty],
) -> Result<NoteDatabaseViewRow, String> {
    if let Some(view) = load_board_view_row_tx(tx, &data_source.id).await? {
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
    .map_err(|e| format!("prepare notes board view order: {e}"))?;
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
         VALUES (?, ?, ?, ?, 'board', '[]', ?, ?)",
    )
    .bind(&id)
    .bind(&data_source.database_id)
    .bind(&data_source.id)
    .bind(DEFAULT_BOARD_VIEW_NAME)
    .bind(default_board_configuration(schema).to_string())
    .bind(sort_order)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert notes board view: {e}"))?;
    load_board_view_row_tx(tx, &data_source.id)
        .await?
        .ok_or_else(|| "inserted board view was not found".to_string())
}

async fn load_board_view_row_tx(
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
         WHERE data_source_id = ? AND type = 'board'
         ORDER BY sort_order ASC, created_time ASC, id ASC
         LIMIT 1",
    )
    .bind(data_source_id.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes board view: {e}"))
}

pub(in crate::notes) async fn load_active_row_pages_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
) -> Result<Vec<NotePageRow>, String> {
    sqlx::query_as::<_, NotePageRow>(
        "SELECT page.*
         FROM notes_pages AS page
         WHERE page.parent_type = 'data_source_id'
           AND page.parent_data_source_id = ?
           AND page.in_trash = 0
           AND page.archived = 0
         ORDER BY page.last_edited_time DESC, page.title COLLATE NOCASE ASC, page.id ASC",
    )
    .bind(data_source_id.trim())
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load notes data source board rows: {e}"))
}

pub(in crate::notes) async fn generated_uuid_tx(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<String, String> {
    let id: String = sqlx::query_scalar(
        "SELECT lower(hex(randomblob(4))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(6)))",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("generate board view id: {e}"))?;
    require_uuid(&id, "generated_board_view_id")?;
    Ok(id)
}

#[derive(Clone)]
pub(in crate::notes) struct BoardProperty {
    pub(in crate::notes) key: String,
    pub(in crate::notes) id: String,
    pub(in crate::notes) property_type: String,
    pub(in crate::notes) schema: Value,
}

#[derive(Clone)]
struct BoardConfiguration {
    group_property_id: Option<String>,
    group_order: Vec<String>,
    hidden_group_ids: HashSet<String>,
}

struct BoardGroupDraft {
    id: String,
    name: String,
    color: String,
    hidden: bool,
    rows: Vec<NotePageRow>,
}

pub(in crate::notes) fn board_schema(properties: &Value) -> Result<Vec<BoardProperty>, String> {
    let object = properties
        .as_object()
        .ok_or_else(|| "data source properties must be an object".to_string())?;
    let mut schema = Vec::with_capacity(object.len());
    for (key, value) in object {
        let property = value
            .as_object()
            .ok_or_else(|| "data source property must be an object".to_string())?;
        schema.push(BoardProperty {
            key: key.clone(),
            id: read_string_field(property, "id", "property.id")?.to_string(),
            property_type: read_string_field(property, "type", "property.type")?.to_string(),
            schema: value.clone(),
        });
    }
    Ok(schema)
}

fn default_board_configuration(schema: &[BoardProperty]) -> Value {
    let group_property_id = default_group_property_id(schema);
    json!({
        "type": "board",
        "board": {
            "group_property_id": group_property_id,
            "group_order": [],
            "hidden_group_ids": [],
            "visible_property_ids": visible_board_property_ids(schema, group_property_id.as_deref()),
            "row_open_mode": "full_page"
        }
    })
}

fn canonical_board_configuration(
    update: &NoteDataSourceBoardConfigurationUpdate,
    schema: &[BoardProperty],
) -> Result<Value, String> {
    let property_ids: HashSet<&str> = schema.iter().map(|property| property.id.as_str()).collect();
    let group_property_id = match update.group_property_id.as_deref().map(str::trim) {
        Some("") | None => None,
        Some(id) => {
            let property = schema
                .iter()
                .find(|property| property.id == id)
                .ok_or_else(|| "board group property references an unknown property".to_string())?;
            if !BOARD_GROUP_PROPERTY_TYPES.contains(&property.property_type.as_str()) {
                return Err("board group property type is not supported".to_string());
            }
            Some(id.to_string())
        }
    };
    let group_order = unique_strings(&update.group_order);
    let hidden_group_ids = unique_strings(&update.hidden_group_ids);
    let mut visible_property_ids = Vec::new();
    let mut seen = HashSet::new();
    for id in &update.visible_property_ids {
        let id = id.trim();
        if id.is_empty() || id == "title" || !property_ids.contains(id) {
            continue;
        }
        if seen.insert(id.to_string()) {
            visible_property_ids.push(id.to_string());
        }
    }
    let row_open_mode = update.row_open_mode.trim();
    if !BOARD_ROW_OPEN_MODES.contains(&row_open_mode) {
        return Err("board row_open_mode is not supported".to_string());
    }
    let value = json!({
        "type": "board",
        "board": {
            "group_property_id": group_property_id,
            "group_order": group_order,
            "hidden_group_ids": hidden_group_ids,
            "visible_property_ids": visible_property_ids,
            "row_open_mode": row_open_mode
        }
    });
    if value.to_string().len() > MAX_BOARD_CONFIGURATION_BYTES {
        return Err("board configuration must not exceed 50KB".to_string());
    }
    Ok(value)
}

fn board_configuration(
    configuration: Option<&str>,
    schema: &[BoardProperty],
) -> Result<BoardConfiguration, String> {
    let value = configuration
        .map(|configuration| parse_json(configuration, "board view configuration"))
        .transpose()?
        .unwrap_or_else(|| default_board_configuration(schema));
    let board = value
        .get("board")
        .and_then(Value::as_object)
        .ok_or_else(|| "board view configuration must contain board".to_string())?;
    let group_property_id = board
        .get("group_property_id")
        .and_then(Value::as_str)
        .filter(|id| schema.iter().any(|property| property.id == *id))
        .map(str::to_string)
        .or_else(|| default_group_property_id(schema));
    let group_order = string_array(board.get("group_order")).unwrap_or_default();
    let hidden_group_ids = string_array(board.get("hidden_group_ids"))
        .unwrap_or_default()
        .into_iter()
        .collect();
    Ok(BoardConfiguration {
        group_property_id,
        group_order,
        hidden_group_ids,
    })
}

fn default_group_property_id(schema: &[BoardProperty]) -> Option<String> {
    BOARD_GROUP_PROPERTY_TYPES.iter().find_map(|property_type| {
        schema
            .iter()
            .find(|property| property.property_type == *property_type)
            .map(|property| property.id.clone())
    })
}

fn visible_board_property_ids(
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

fn board_groups(
    schema: &[BoardProperty],
    configuration: &BoardConfiguration,
    rows: Vec<NotePageRow>,
) -> Result<Vec<NoteDataSourceBoardGroupDto>, String> {
    let group_property = configuration
        .group_property_id
        .as_deref()
        .and_then(|id| schema.iter().find(|property| property.id == id));
    let mut groups = initial_groups(group_property, configuration);
    for row in rows {
        let group_ids = group_property
            .map(|property| row_group_ids(&row, property))
            .unwrap_or_else(|| vec![BOARD_UNGROUPED_ID.to_string()]);
        for group_id in group_ids {
            if !groups.contains_key(&group_id) {
                let (name, color) = dynamic_group_label(&group_id, group_property);
                groups.insert(
                    group_id.clone(),
                    BoardGroupDraft {
                        id: group_id.clone(),
                        name,
                        color,
                        hidden: false,
                        rows: Vec::new(),
                    },
                );
            }
            if let Some(group) = groups.get_mut(&group_id) {
                group.rows.push(row.clone());
            }
        }
    }
    let mut ordered = Vec::new();
    for group_id in &configuration.group_order {
        if let Some(group) = groups.remove(group_id) {
            ordered.push(group);
        }
    }
    ordered.extend(groups.into_values());
    ordered
        .into_iter()
        .map(|group| {
            NoteDataSourceBoardGroupDto::new(
                group.id,
                group.name,
                group.color,
                group.hidden,
                group.rows,
            )
        })
        .collect()
}

fn initial_groups(
    group_property: Option<&BoardProperty>,
    configuration: &BoardConfiguration,
) -> BTreeMap<String, BoardGroupDraft> {
    let mut groups = BTreeMap::new();
    let Some(property) = group_property else {
        groups.insert(
            BOARD_UNGROUPED_ID.to_string(),
            group_draft(BOARD_UNGROUPED_ID, "Ungrouped", "default", configuration),
        );
        return groups;
    };
    match property.property_type.as_str() {
        "select" | "multi_select" | "status" => {
            for option in property_options(property) {
                groups.insert(
                    option.id.clone(),
                    group_draft(&option.id, &option.name, &option.color, configuration),
                );
            }
            groups.insert(
                BOARD_EMPTY_GROUP_ID.to_string(),
                group_draft(BOARD_EMPTY_GROUP_ID, "No value", "default", configuration),
            );
        }
        "checkbox" => {
            groups.insert(
                "false".to_string(),
                group_draft("false", "Unchecked", "gray", configuration),
            );
            groups.insert(
                "true".to_string(),
                group_draft("true", "Checked", "green", configuration),
            );
        }
        "date" | "people" | "relation" => {
            groups.insert(
                BOARD_EMPTY_GROUP_ID.to_string(),
                group_draft(BOARD_EMPTY_GROUP_ID, "No value", "default", configuration),
            );
        }
        _ => {
            groups.insert(
                BOARD_UNGROUPED_ID.to_string(),
                group_draft(BOARD_UNGROUPED_ID, "Ungrouped", "default", configuration),
            );
        }
    }
    groups
}

fn group_draft(
    id: &str,
    name: &str,
    color: &str,
    configuration: &BoardConfiguration,
) -> BoardGroupDraft {
    BoardGroupDraft {
        id: id.to_string(),
        name: name.to_string(),
        color: color.to_string(),
        hidden: configuration.hidden_group_ids.contains(id),
        rows: Vec::new(),
    }
}

struct BoardOption {
    id: String,
    name: String,
    color: String,
}

fn property_options(property: &BoardProperty) -> Vec<BoardOption> {
    property
        .schema
        .get(&property.property_type)
        .and_then(|config| config.get("options"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .filter_map(|option| {
            Some(BoardOption {
                id: option.get("id")?.as_str()?.to_string(),
                name: option.get("name")?.as_str()?.to_string(),
                color: option
                    .get("color")
                    .and_then(Value::as_str)
                    .unwrap_or("default")
                    .to_string(),
            })
        })
        .collect()
}

fn row_group_ids(row: &NotePageRow, property: &BoardProperty) -> Vec<String> {
    match property.property_type.as_str() {
        "select" | "status" => row_property_payload(row, property)
            .and_then(|payload| {
                payload
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .filter(|id| !id.is_empty())
            .map(|id| vec![id])
            .unwrap_or_else(|| vec![BOARD_EMPTY_GROUP_ID.to_string()]),
        "multi_select" => {
            let ids: Vec<String> = row_property_payload(row, property)
                .and_then(|payload| payload.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
                .collect();
            if ids.is_empty() {
                vec![BOARD_EMPTY_GROUP_ID.to_string()]
            } else {
                ids
            }
        }
        "checkbox" => row_property_checked(row, property)
            .map(|checked| checked.to_string())
            .into_iter()
            .collect(),
        "date" => row_property_payload(row, property)
            .and_then(|payload| {
                payload
                    .get("start")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .filter(|date| !date.trim().is_empty())
            .map(|date| vec![date])
            .unwrap_or_else(|| vec![BOARD_EMPTY_GROUP_ID.to_string()]),
        "people" | "relation" => {
            let ids: Vec<String> = row_property_payload(row, property)
                .and_then(|payload| payload.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
                .collect();
            if ids.is_empty() {
                vec![BOARD_EMPTY_GROUP_ID.to_string()]
            } else {
                ids
            }
        }
        _ => vec![BOARD_UNGROUPED_ID.to_string()],
    }
}

fn dynamic_group_label(group_id: &str, group_property: Option<&BoardProperty>) -> (String, String) {
    if group_id == BOARD_EMPTY_GROUP_ID {
        return ("No value".to_string(), "default".to_string());
    }
    if let Some(property) = group_property {
        if property.property_type == "date" {
            return (group_id.to_string(), "blue".to_string());
        }
        if property.property_type == "people" {
            return (format!("Person {group_id}"), "purple".to_string());
        }
        if property.property_type == "relation" {
            return (format!("Related {group_id}"), "default".to_string());
        }
    }
    (group_id.to_string(), "default".to_string())
}

fn board_move_value(property: &BoardProperty, group_id: &str) -> Result<Value, String> {
    match property.property_type.as_str() {
        "select" | "status" => {
            if group_id == BOARD_EMPTY_GROUP_ID {
                Ok(Value::Null)
            } else {
                option_name_by_id(property, group_id).map(Value::String)
            }
        }
        "multi_select" => {
            if group_id == BOARD_EMPTY_GROUP_ID {
                Ok(Value::String(String::new()))
            } else {
                option_name_by_id(property, group_id).map(Value::String)
            }
        }
        "checkbox" => match group_id {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err("checkbox board group must be checked or unchecked".to_string()),
        },
        "date" => {
            if group_id == BOARD_EMPTY_GROUP_ID {
                Ok(Value::Null)
            } else {
                Ok(Value::String(group_id.to_string()))
            }
        }
        "people" => {
            if group_id == BOARD_EMPTY_GROUP_ID {
                Ok(Value::Array(Vec::new()))
            } else {
                Err("people board groups can only move cards to no value".to_string())
            }
        }
        "relation" => Err("relation board moves require normalized relation links".to_string()),
        _ => Err("board group property type is not movable".to_string()),
    }
}

fn option_name_by_id(property: &BoardProperty, group_id: &str) -> Result<String, String> {
    property_options(property)
        .into_iter()
        .find(|option| option.id == group_id || option.name.eq_ignore_ascii_case(group_id))
        .map(|option| option.name)
        .ok_or_else(|| "board target group option was not found".to_string())
}

pub(in crate::notes) fn canonical_filter(
    filters: &[NoteDataSourceTableFilter],
    property_ids: &HashSet<String>,
) -> Result<Option<Value>, String> {
    if filters.len() > MAX_FILTERS {
        return Err("board filters are limited to 10".to_string());
    }
    let mut canonical = Vec::new();
    for filter in filters {
        let property_id = filter.property_id.trim();
        if property_id.is_empty() {
            continue;
        }
        if !property_ids.contains(property_id) {
            return Err("board filter references an unknown property".to_string());
        }
        let condition = filter.condition.trim();
        if !FILTER_CONDITIONS.contains(&condition) {
            return Err("board filter condition is not supported".to_string());
        }
        let value = canonical_filter_value(condition, filter.value.as_ref())?;
        canonical.push(json!({
            "property_id": property_id,
            "condition": condition,
            "value": value
        }));
    }
    if canonical.is_empty() {
        Ok(None)
    } else {
        Ok(Some(json!({
            "type": "and",
            "filters": canonical
        })))
    }
}

fn canonical_filter_value(condition: &str, value: Option<&Value>) -> Result<Value, String> {
    if matches!(
        condition,
        "is_empty" | "is_not_empty" | "checked" | "unchecked"
    ) {
        return Ok(Value::Null);
    }
    let Some(value) = value else {
        return Ok(Value::Null);
    };
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(value.clone()),
        Value::String(text) => Ok(Value::String(validate_text(
            text.trim(),
            "board filter value",
            MAX_FILTER_TEXT_CHARS,
        )?)),
        _ => Err("board filter value must be a scalar".to_string()),
    }
}

pub(in crate::notes) fn canonical_sorts(
    sorts: &[NoteDataSourceTableSort],
    property_ids: &HashSet<String>,
) -> Result<Value, String> {
    if sorts.len() > MAX_SORTS {
        return Err("board sorts are limited to 5".to_string());
    }
    let mut seen = HashSet::new();
    let mut canonical = Vec::new();
    for sort in sorts {
        let property_id = sort.property_id.trim();
        if property_id.is_empty() {
            continue;
        }
        if !property_ids.contains(property_id) {
            return Err("board sort references an unknown property".to_string());
        }
        if !seen.insert(property_id.to_string()) {
            return Err("board sorts must not repeat properties".to_string());
        }
        let direction = sort.direction.trim();
        if direction != "ascending" && direction != "descending" {
            return Err("board sort direction is not supported".to_string());
        }
        canonical.push(json!({
            "property_id": property_id,
            "direction": direction
        }));
    }
    Ok(Value::Array(canonical))
}

pub(in crate::notes) fn stored_filters(
    filter: Option<&str>,
) -> Result<Vec<NoteDataSourceTableFilter>, String> {
    let Some(filter) = filter else {
        return Ok(Vec::new());
    };
    let value = parse_json(filter, "database board filter")?;
    let filters = value
        .get("filters")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    serde_json::from_value(filters).map_err(|e| format!("parse board filters: {e}"))
}

pub(in crate::notes) fn stored_sorts(sorts: &str) -> Result<Vec<NoteDataSourceTableSort>, String> {
    let value = parse_json(sorts, "database board sorts")?;
    serde_json::from_value(value).map_err(|e| format!("parse board sorts: {e}"))
}

pub(in crate::notes) fn normalized_row_for_schema(
    mut row: NotePageRow,
    schema: &[BoardProperty],
) -> Result<NotePageRow, String> {
    let current = parse_json(&row.properties, "row page properties")?;
    let (title, properties) = normalized_row_properties(schema, &current, &row.title)?;
    row.title = title;
    row.properties = properties.to_string();
    Ok(row)
}

fn normalized_row_properties(
    schema: &[BoardProperty],
    current: &Value,
    fallback_title: &str,
) -> Result<(String, Value), String> {
    let current_object = current
        .as_object()
        .ok_or_else(|| "row page properties must be an object".to_string())?;
    let mut title = fallback_title.to_string();
    let mut next = Map::new();
    for property in schema {
        let value = existing_property_value(current_object, property)
            .and_then(|value| canonical_stored_property_value(property, value).ok())
            .unwrap_or_else(|| default_property_value(property, fallback_title));
        if property.property_type == "title" {
            title = title_from_property_value(&value).unwrap_or_else(|| fallback_title.to_string());
        }
        next.insert(property.key.clone(), value);
    }
    Ok((title, Value::Object(next)))
}

fn existing_property_value<'a>(
    current: &'a Map<String, Value>,
    property: &BoardProperty,
) -> Option<&'a Value> {
    current
        .get(&property.key)
        .filter(|value| property_value_matches_schema(property, value))
        .or_else(|| {
            current
                .values()
                .find(|value| property_value_matches_schema(property, value))
        })
}

fn property_value_matches_schema(property: &BoardProperty, value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("id").and_then(Value::as_str) == Some(property.id.as_str())
        && object.get("type").and_then(Value::as_str) == Some(property.property_type.as_str())
}

fn canonical_stored_property_value(
    property: &BoardProperty,
    value: &Value,
) -> Result<Value, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "row property value must be an object".to_string())?;
    if object.get("id").and_then(Value::as_str) != Some(property.id.as_str()) {
        return Err("row property id does not match schema".to_string());
    }
    if object.get("type").and_then(Value::as_str) != Some(property.property_type.as_str()) {
        return Err("row property type does not match schema".to_string());
    }
    let payload = object
        .get(&property.property_type)
        .ok_or_else(|| "row property is missing its typed value".to_string())?;
    Ok(json!({
        "id": property.id,
        "type": property.property_type,
        property.property_type.clone(): canonical_property_payload(&property.property_type, payload)?
    }))
}

fn default_property_value(property: &BoardProperty, title: &str) -> Value {
    let payload = match property.property_type.as_str() {
        "title" => Value::Array(vec![writes::rich_text(title)]),
        "rich_text" | "multi_select" | "files" | "people" | "relation" => Value::Array(Vec::new()),
        "number" | "select" | "status" | "date" | "url" | "email" | "phone_number"
        | "created_time" | "created_by" | "last_edited_time" | "last_edited_by" | "place" => {
            Value::Null
        }
        "checkbox" => Value::Bool(false),
        "unique_id" => json!({
            "number": null,
            "prefix": property
                .schema
                .get("unique_id")
                .and_then(|config| config.get("prefix"))
                .cloned()
                .unwrap_or(Value::Null)
        }),
        _ => Value::Null,
    };
    json!({
        "id": property.id,
        "type": property.property_type,
        property.property_type.clone(): payload
    })
}

fn canonical_property_payload(property_type: &str, value: &Value) -> Result<Value, String> {
    match property_type {
        "title" | "rich_text" | "multi_select" | "files" | "people" | "relation" => {
            if value.is_array() {
                Ok(value.clone())
            } else {
                Err(format!("{property_type} property must be an array"))
            }
        }
        "number" => {
            if value.is_null() || value.is_number() {
                Ok(value.clone())
            } else {
                Err("number property must be a number or null".to_string())
            }
        }
        "select" | "status" | "date" | "created_by" | "last_edited_by" | "unique_id" | "place" => {
            if value.is_null() || value.is_object() {
                Ok(value.clone())
            } else {
                Err(format!(
                    "{property_type} property must be an object or null"
                ))
            }
        }
        "checkbox" => value
            .as_bool()
            .map(Value::Bool)
            .ok_or_else(|| "checkbox property must be boolean".to_string()),
        "url" | "email" | "phone_number" | "created_time" | "last_edited_time" => {
            if value.is_null() {
                return Ok(Value::Null);
            }
            Ok(Value::String(validate_text(
                value
                    .as_str()
                    .ok_or_else(|| format!("{property_type} property must be text or null"))?,
                property_type,
                MAX_FILTER_TEXT_CHARS,
            )?))
        }
        other => Err(format!("unsupported row property type: {other}")),
    }
}

pub(in crate::notes) fn row_matches_filters(
    row: &NotePageRow,
    schema: &[BoardProperty],
    filters: &[NoteDataSourceTableFilter],
) -> bool {
    filters.iter().all(|filter| {
        let Some(property) = schema
            .iter()
            .find(|property| property.id == filter.property_id)
        else {
            return true;
        };
        let text = row_property_plain_text(row, property);
        match filter.condition.as_str() {
            "contains" => filter
                .value
                .as_ref()
                .and_then(Value::as_str)
                .map(|value| text.to_lowercase().contains(&value.to_lowercase()))
                .unwrap_or(true),
            "equals" => filter
                .value
                .as_ref()
                .map(|value| scalar_filter_text(value).to_lowercase() == text.to_lowercase())
                .unwrap_or_else(|| text.is_empty()),
            "is_empty" => text.trim().is_empty(),
            "is_not_empty" => !text.trim().is_empty(),
            "checked" => row_property_checked(row, property) == Some(true),
            "unchecked" => row_property_checked(row, property) == Some(false),
            _ => true,
        }
    })
}

pub(in crate::notes) fn sort_rows(
    rows: &mut [NotePageRow],
    schema: &[BoardProperty],
    sorts: &[NoteDataSourceTableSort],
) {
    rows.sort_by(|left, right| {
        for sort in sorts {
            let Some(property) = schema
                .iter()
                .find(|property| property.id == sort.property_id)
            else {
                continue;
            };
            let ordering = compare_row_property(left, right, property);
            if ordering != Ordering::Equal {
                return if sort.direction == "descending" {
                    ordering.reverse()
                } else {
                    ordering
                };
            }
        }
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn compare_row_property(
    left: &NotePageRow,
    right: &NotePageRow,
    property: &BoardProperty,
) -> Ordering {
    match property.property_type.as_str() {
        "number" => compare_optional_f64(
            row_property_number(left, property),
            row_property_number(right, property),
        ),
        "checkbox" => compare_optional_bool(
            row_property_checked(left, property),
            row_property_checked(right, property),
        ),
        _ => row_property_plain_text(left, property)
            .to_lowercase()
            .cmp(&row_property_plain_text(right, property).to_lowercase()),
    }
}

fn compare_optional_f64(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_bool(left: Option<bool>, right: Option<bool>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn row_property_value(row: &NotePageRow, property: &BoardProperty) -> Option<Value> {
    let properties = parse_json(&row.properties, "row page properties").ok()?;
    properties.get(&property.key).cloned()
}

fn row_property_payload(row: &NotePageRow, property: &BoardProperty) -> Option<Value> {
    row_property_value(row, property)?
        .get(&property.property_type)
        .cloned()
}

fn row_property_plain_text(row: &NotePageRow, property: &BoardProperty) -> String {
    match property.property_type.as_str() {
        "title" | "rich_text" => row_property_payload(row, property)
            .and_then(|payload| payload.as_array().cloned())
            .map(|items| rich_text_plain_text(&items))
            .unwrap_or_default(),
        "number" => row_property_number(row, property)
            .map(|number| number.to_string())
            .unwrap_or_default(),
        "checkbox" => row_property_checked(row, property)
            .map(|checked| checked.to_string())
            .unwrap_or_default(),
        "select" | "status" => row_property_payload(row, property)
            .and_then(|payload| {
                payload
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default(),
        "multi_select" | "people" | "relation" => row_property_payload(row, property)
            .and_then(|payload| payload.as_array().cloned())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        item.get("name")
                            .or_else(|| item.get("id"))
                            .and_then(Value::as_str)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default(),
        "date" => row_property_payload(row, property)
            .and_then(|payload| {
                payload
                    .get("start")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default(),
        "url" | "email" | "phone_number" => row_property_payload(row, property)
            .and_then(|payload| payload.as_str().map(str::to_string))
            .unwrap_or_default(),
        "created_time" => row.created_time.clone(),
        "last_edited_time" => row.last_edited_time.clone(),
        "place" => row_property_payload(row, property)
            .and_then(|payload| {
                payload
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn row_property_number(row: &NotePageRow, property: &BoardProperty) -> Option<f64> {
    row_property_payload(row, property)?.as_f64()
}

fn row_property_checked(row: &NotePageRow, property: &BoardProperty) -> Option<bool> {
    row_property_payload(row, property)?.as_bool()
}

fn rich_text_plain_text(items: &[Value]) -> String {
    let mut text = String::new();
    for item in items {
        if let Some(plain_text) = item.get("plain_text").and_then(Value::as_str) {
            text.push_str(plain_text);
        } else if let Some(content) = item
            .get("text")
            .and_then(|value| value.get("content"))
            .and_then(Value::as_str)
        {
            text.push_str(content);
        }
    }
    text
}

fn title_from_property_value(value: &Value) -> Option<String> {
    value
        .get("title")?
        .as_array()
        .map(|items| rich_text_plain_text(items))
}

fn unique_strings(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if seen.insert(value.to_string()) {
            result.push(value.to_string());
        }
    }
    result
}

fn string_array(value: Option<&Value>) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| "board configuration list must be an array".to_string())?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| "board configuration list item must be text".to_string())
        })
        .collect()
}

fn scalar_filter_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        _ => String::new(),
    }
}

fn validate_text(value: &str, label: &str, max_chars: usize) -> Result<String, String> {
    if value.chars().any(char::is_control) {
        return Err(format!("{label} must not contain control characters"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label} is too long"));
    }
    Ok(value.to_string())
}

pub(in crate::notes) fn parse_json(value: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(value).map_err(|e| format!("parse {label}: {e}"))
}

fn read_string_field<'a>(
    object: &'a Map<String, Value>,
    key: &str,
    label: &str,
) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} must be a string"))
}
