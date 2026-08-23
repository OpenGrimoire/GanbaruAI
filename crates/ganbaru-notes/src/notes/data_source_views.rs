use super::models::{NoteDataSourceRow, NoteDatabaseViewRow};
use super::validation::require_uuid;
use sqlx::{Sqlite, Transaction};

pub fn validate_view_scope(
    data_source_id: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<(), String> {
    require_uuid(data_source_id, "data_source_id")?;
    if let Some(database_id) = database_id {
        require_uuid(database_id, "database_id")?;
    }
    if let Some(view_id) = view_id {
        require_uuid(view_id, "view_id")?;
    }
    Ok(())
}

pub fn scoped_database_id<'a>(
    data_source: &'a NoteDataSourceRow,
    database_id: Option<&'a str>,
) -> &'a str {
    database_id.unwrap_or(&data_source.database_id)
}

pub async fn next_view_sort_order_tx(
    tx: &mut Transaction<'_, Sqlite>,
    database_id: &str,
) -> Result<f64, String> {
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order) + 1, 1)
         FROM notes_database_views
         WHERE database_id = ?",
    )
    .bind(database_id.trim())
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("prepare notes database view order: {e}"))
}

pub async fn generated_uuid_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<String, String> {
    let id: String = sqlx::query_scalar(
        "SELECT lower(hex(randomblob(4))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(2))) || '-' ||
                lower(hex(randomblob(6)))",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| format!("generate database view id: {e}"))?;
    require_uuid(&id, "generated_database_view_id")?;
    Ok(id)
}

pub async fn load_scoped_view_row_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
    view_type: &str,
    database_id: Option<&str>,
    view_id: Option<&str>,
) -> Result<Option<NoteDatabaseViewRow>, String> {
    let row = if let Some(view_id) = view_id {
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
             WHERE id = ? AND data_source_id = ? AND type = ?",
        )
        .bind(view_id.trim())
        .bind(data_source_id.trim())
        .bind(view_type)
        .fetch_optional(&mut **tx)
        .await
    } else if let Some(database_id) = database_id {
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
             WHERE database_id = ? AND data_source_id = ? AND type = ?
             ORDER BY sort_order ASC, created_time ASC, id ASC
             LIMIT 1",
        )
        .bind(database_id.trim())
        .bind(data_source_id.trim())
        .bind(view_type)
        .fetch_optional(&mut **tx)
        .await
    } else {
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
             WHERE data_source_id = ? AND type = ?
             ORDER BY sort_order ASC, created_time ASC, id ASC
             LIMIT 1",
        )
        .bind(data_source_id.trim())
        .bind(view_type)
        .fetch_optional(&mut **tx)
        .await
    };
    row.map_err(|e| format!("load notes {view_type} view: {e}"))
}
