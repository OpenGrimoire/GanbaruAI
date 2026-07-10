mod bundles;
mod restore;
mod scope;

use super::{local_user, writes};
use crate::db_path::connect_sqlite;
use bundles::{garbage_collect_bundles_tx, load_bundle_tx, store_bundle_tx};
use scope::{load_project_graph, ProjectHistoryGraph};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use std::collections::{BTreeMap, HashSet};
use tauri::{AppHandle, Runtime};

const DEFAULT_RETENTION_DAYS: i64 = 30;
const HISTORY_SCHEMA_VERSION: i64 = 1;
const ACTIVE_CHECKPOINT_MINUTES: i64 = 10;
const IDLE_CHECKPOINT_MINUTES: i64 = 2;
const DEFAULT_PAGE_SIZE: i64 = 40;
const MAX_PAGE_SIZE: i64 = 100;
const SUPPORTED_RETENTION_DAYS: [i64; 6] = [0, 7, 30, 90, 180, 365];

#[cfg(test)]
fn checkpoint_is_due(
    first_dirty_seconds_ago: i64,
    last_dirty_seconds_ago: i64,
    force_checkpoint: bool,
) -> bool {
    force_checkpoint
        || first_dirty_seconds_ago >= ACTIVE_CHECKPOINT_MINUTES * 60
        || last_dirty_seconds_ago >= IDLE_CHECKPOINT_MINUTES * 60
}

#[cfg(test)]
fn normalize_legacy_retention_days(value: Option<i64>) -> i64 {
    match value {
        None => 365,
        Some(days) if days <= 7 => 7,
        Some(days) if days <= 30 => 30,
        Some(days) if days <= 90 => 90,
        Some(days) if days <= 180 => 180,
        Some(_) => 365,
    }
}

pub(in crate::notes) async fn store_page_history_blocks_tx(
    tx: &mut Transaction<'_, Sqlite>,
    blocks_json: &str,
) -> Result<String, String> {
    store_bundle_tx(tx, "row", blocks_json.as_bytes()).await
}

pub(in crate::notes) async fn load_page_history_blocks_tx(
    tx: &mut Transaction<'_, Sqlite>,
    hash: &str,
) -> Result<String, String> {
    let raw = load_bundle_tx(tx, hash, "row").await?;
    String::from_utf8(raw).map_err(|_| "Notes page history block bundle is not UTF-8".to_string())
}

pub(in crate::notes) fn page_history_blocks_hash(blocks_json: &str) -> String {
    bundles::sha256_hex(blocks_json.as_bytes())
}

pub(in crate::notes) async fn garbage_collect_history_storage_tx(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), String> {
    garbage_collect_bundles_tx(tx).await
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectHistoryManifest {
    schema_version: i64,
    project_id: String,
    rows_by_table: BTreeMap<String, Vec<String>>,
    asset_ids: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesProjectHistoryVersionDto {
    id: String,
    project_id: String,
    manifest_hash: String,
    reason: String,
    created_by: String,
    display_name: Value,
    changed_note_summary: String,
    page_count: i64,
    active_page_count: i64,
    archived_page_count: i64,
    deleted_page_count: i64,
    created_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesProjectHistoryVersionListDto {
    versions: Vec<NotesProjectHistoryVersionDto>,
    next_cursor_time: Option<String>,
    next_cursor_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesHistoryRetentionImpactDto {
    version_count: i64,
    stored_bytes: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesProjectHistoryTreeDto {
    version: NotesProjectHistoryVersionDto,
    pages: Vec<NotesHistoricalPageSummaryDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesHistoricalPageSummaryDto {
    id: String,
    title: String,
    parent_page_id: Option<String>,
    parent_data_source_id: Option<String>,
    in_trash: bool,
    archived: bool,
    icon: Option<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesHistoricalPageDto {
    id: String,
    title: String,
    properties: Value,
    icon: Option<Value>,
    cover: Option<Value>,
    in_trash: bool,
    archived: bool,
    blocks: Vec<Value>,
    databases: Vec<Value>,
    data_sources: Vec<Value>,
    database_views: Vec<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotesProjectHistoryRestorePlanDto {
    version_id: String,
    remove_count: i64,
    recreate_count: i64,
    change_count: i64,
    copy_count: i64,
    safety_version_will_be_created: bool,
}

fn validate_project_id(project_id: &str) -> Result<String, String> {
    let project_id = project_id.trim();
    if project_id.is_empty() || project_id.len() > 120 {
        return Err("project_id is invalid".to_string());
    }
    Ok(project_id.to_string())
}

pub(super) fn validate_retention_days(retention_days: i64) -> Result<(), String> {
    if !SUPPORTED_RETENTION_DAYS.contains(&retention_days) {
        return Err("Notes history retention must be off, 7, 30, 90, 180, or 365 days".to_string());
    }
    Ok(())
}

async fn new_version_id_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<String, String> {
    let mut reserved_ids = HashSet::new();
    for _ in 0..32 {
        let id = writes::new_note_id(tx, &mut reserved_ids).await?;
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM notes_project_history_versions WHERE id = ?")
                .bind(&id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| format!("check Notes project history version id: {e}"))?;
        if exists.is_none() {
            return Ok(id);
        }
    }
    Err("could not generate a Notes project history version id".to_string())
}

async fn store_graph_tx(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
    graph: &ProjectHistoryGraph,
) -> Result<(String, Vec<String>, i64), String> {
    let mut rows_by_table = BTreeMap::new();
    let mut bundle_hashes = HashSet::new();
    for (table, rows) in &graph.rows_by_table {
        let mut hashes = Vec::with_capacity(rows.len());
        for row in rows {
            let raw = serde_json::to_vec(row)
                .map_err(|e| format!("serialize Notes history row for {table}: {e}"))?;
            let hash = store_bundle_tx(tx, "row", &raw).await?;
            bundle_hashes.insert(hash.clone());
            hashes.push(hash);
        }
        rows_by_table.insert(table.clone(), hashes);
    }
    let manifest = ProjectHistoryManifest {
        schema_version: HISTORY_SCHEMA_VERSION,
        project_id: project_id.to_string(),
        rows_by_table,
        asset_ids: graph.asset_ids.clone(),
    };
    let raw = serde_json::to_vec(&manifest)
        .map_err(|e| format!("serialize Notes project history manifest: {e}"))?;
    let uncompressed_bytes = i64::try_from(raw.len())
        .map_err(|_| "Notes project history manifest is too large".to_string())?;
    let manifest_hash = store_bundle_tx(tx, "manifest", &raw).await?;
    let mut hashes = bundle_hashes.into_iter().collect::<Vec<_>>();
    hashes.sort();
    Ok((manifest_hash, hashes, uncompressed_bytes))
}

async fn load_manifest_tx(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
    version_id: &str,
) -> Result<(ProjectHistoryManifest, NotesProjectHistoryVersionDto), String> {
    let row = sqlx::query(
        "SELECT id, project_id, manifest_hash, reason, created_by, display_name,
                changed_note_summary, page_count, active_page_count,
                archived_page_count, deleted_page_count, created_time
         FROM notes_project_history_versions
         WHERE id = ? AND project_id = ?",
    )
    .bind(version_id)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load Notes project history version: {e}"))?
    .ok_or_else(|| "Notes project history version not found".to_string())?;
    let version = version_from_row(&row)?;
    let raw = load_bundle_tx(tx, &version.manifest_hash, "manifest").await?;
    let manifest: ProjectHistoryManifest = serde_json::from_slice(&raw)
        .map_err(|e| format!("parse Notes project history manifest: {e}"))?;
    if manifest.schema_version != HISTORY_SCHEMA_VERSION || manifest.project_id != project_id {
        return Err("Notes project history manifest metadata is invalid".to_string());
    }
    Ok((manifest, version))
}

async fn load_manifest_rows_tx(
    tx: &mut Transaction<'_, Sqlite>,
    manifest: &ProjectHistoryManifest,
) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let mut rows_by_table = BTreeMap::new();
    for (table, hashes) in &manifest.rows_by_table {
        let mut rows = Vec::with_capacity(hashes.len());
        for hash in hashes {
            let raw = load_bundle_tx(tx, hash, "row").await?;
            let row = serde_json::from_slice(&raw)
                .map_err(|e| format!("parse Notes history row for {table}: {e}"))?;
            rows.push(row);
        }
        rows_by_table.insert(table.clone(), rows);
    }
    Ok(rows_by_table)
}

fn version_from_row(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<NotesProjectHistoryVersionDto, String> {
    let display_name_json: String = row
        .try_get("display_name")
        .map_err(|e| format!("read Notes history display name: {e}"))?;
    let display_name = serde_json::from_str(&display_name_json)
        .map_err(|e| format!("parse Notes history display name: {e}"))?;
    Ok(NotesProjectHistoryVersionDto {
        id: row.try_get("id").map_err(|e| e.to_string())?,
        project_id: row.try_get("project_id").map_err(|e| e.to_string())?,
        manifest_hash: row.try_get("manifest_hash").map_err(|e| e.to_string())?,
        reason: row.try_get("reason").map_err(|e| e.to_string())?,
        created_by: row.try_get("created_by").map_err(|e| e.to_string())?,
        display_name,
        changed_note_summary: row
            .try_get("changed_note_summary")
            .map_err(|e| e.to_string())?,
        page_count: row.try_get("page_count").map_err(|e| e.to_string())?,
        active_page_count: row
            .try_get("active_page_count")
            .map_err(|e| e.to_string())?,
        archived_page_count: row
            .try_get("archived_page_count")
            .map_err(|e| e.to_string())?,
        deleted_page_count: row
            .try_get("deleted_page_count")
            .map_err(|e| e.to_string())?,
        created_time: row.try_get("created_time").map_err(|e| e.to_string())?,
    })
}

async fn effective_retention_days_tx(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
) -> Result<i64, String> {
    let value: Option<i64> = sqlx::query_scalar(
        "SELECT COALESCE(
             notes_history_retention_days,
             (SELECT retention_days FROM notes_page_history_settings WHERE id = 1),
             ?
         )
         FROM projects
         WHERE id = ?",
    )
    .bind(DEFAULT_RETENTION_DAYS)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load effective Notes history retention: {e}"))?;
    let retention_days = value.ok_or_else(|| "project not found".to_string())?;
    validate_retention_days(retention_days)?;
    Ok(retention_days)
}

pub(super) async fn prune_project_history_tx(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
) -> Result<i64, String> {
    let retention_days = effective_retention_days_tx(tx, project_id).await?;
    let result = sqlx::query(
        "DELETE FROM notes_project_history_versions
         WHERE project_id = ?
           AND (
               ? = 0
               OR created_time < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' days')
           )",
    )
    .bind(project_id)
    .bind(retention_days)
    .bind(retention_days)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("prune Notes project history: {e}"))?;
    if retention_days == 0 {
        sqlx::query("DELETE FROM notes_project_history_dirty WHERE project_id = ?")
            .bind(project_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| format!("clear disabled Notes project history dirty state: {e}"))?;
    }
    garbage_collect_bundles_tx(tx).await?;
    i64::try_from(result.rows_affected())
        .map_err(|_| "pruned Notes history count is too large".to_string())
}

async fn create_checkpoint(
    pool: &SqlitePool,
    project_id: &str,
    reason: &str,
    actor_id: Option<&str>,
    actor_display_name: Option<&str>,
    changed_note_summary: &str,
) -> Result<Option<NotesProjectHistoryVersionDto>, String> {
    let project_id = validate_project_id(project_id)?;
    let mut retention_tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin Notes project history retention check: {e}"))?;
    if effective_retention_days_tx(&mut retention_tx, &project_id).await? == 0 {
        prune_project_history_tx(&mut retention_tx, &project_id).await?;
        retention_tx
            .commit()
            .await
            .map_err(|e| format!("commit disabled Notes project history cleanup: {e}"))?;
        return Ok(None);
    }
    retention_tx
        .commit()
        .await
        .map_err(|e| format!("commit Notes project history retention check: {e}"))?;
    let graph = load_project_graph(pool, &project_id).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin Notes project history checkpoint: {e}"))?;
    if effective_retention_days_tx(&mut tx, &project_id).await? == 0 {
        prune_project_history_tx(&mut tx, &project_id).await?;
        tx.commit()
            .await
            .map_err(|e| format!("commit disabled Notes project history cleanup: {e}"))?;
        return Ok(None);
    }
    let local_user = if actor_id.is_none() || actor_display_name.is_none() {
        Some(local_user::current_local_user_tx(&mut tx).await?)
    } else {
        None
    };
    let resolved_actor_id = actor_id
        .or_else(|| local_user.as_ref().map(|user| user.id.as_str()))
        .ok_or_else(|| "Notes history actor is unavailable".to_string())?;
    let resolved_display_name = actor_display_name
        .or_else(|| local_user.as_ref().map(|user| user.display_name.as_str()))
        .ok_or_else(|| "Notes history display name is unavailable".to_string())?;
    let (manifest_hash, bundle_hashes, manifest_bytes) =
        store_graph_tx(&mut tx, &project_id, &graph).await?;
    let latest_hash: Option<String> = sqlx::query_scalar(
        "SELECT manifest_hash
         FROM notes_project_history_versions
         WHERE project_id = ?
         ORDER BY created_time DESC, id DESC
         LIMIT 1",
    )
    .bind(&project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| format!("load latest Notes project history version: {e}"))?;
    if latest_hash.as_deref() == Some(&manifest_hash) {
        sqlx::query("DELETE FROM notes_project_history_dirty WHERE project_id = ?")
            .bind(&project_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("clear unchanged Notes project history dirty state: {e}"))?;
        garbage_collect_bundles_tx(&mut tx).await?;
        tx.commit()
            .await
            .map_err(|e| format!("commit unchanged Notes project checkpoint: {e}"))?;
        return Ok(None);
    }
    let version_id = new_version_id_tx(&mut tx).await?;
    let display_name_json = local_user::comment_display_name_json(resolved_display_name);
    sqlx::query(
        "INSERT INTO notes_project_history_versions (
            id, project_id, manifest_hash, reason, created_by, display_name,
            changed_note_summary, page_count, active_page_count,
            archived_page_count, deleted_page_count, manifest_uncompressed_bytes
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(&project_id)
    .bind(&manifest_hash)
    .bind(reason)
    .bind(resolved_actor_id)
    .bind(display_name_json)
    .bind(changed_note_summary)
    .bind(graph.page_count)
    .bind(graph.active_page_count)
    .bind(graph.archived_page_count)
    .bind(graph.deleted_page_count)
    .bind(manifest_bytes)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("insert Notes project history version: {e}"))?;
    for hash in bundle_hashes {
        sqlx::query(
            "INSERT INTO notes_project_history_bundle_references (version_id, bundle_hash)
             VALUES (?, ?)",
        )
        .bind(&version_id)
        .bind(hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("pin Notes project history bundle: {e}"))?;
    }
    for asset_id in &graph.asset_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO notes_project_history_asset_pins (version_id, asset_id)
             VALUES (?, ?)",
        )
        .bind(&version_id)
        .bind(asset_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("pin Notes project history asset: {e}"))?;
    }
    sqlx::query("DELETE FROM notes_project_history_dirty WHERE project_id = ?")
        .bind(&project_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("clear Notes project history dirty state: {e}"))?;
    prune_project_history_tx(&mut tx, &project_id).await?;
    let row = sqlx::query(
        "SELECT id, project_id, manifest_hash, reason, created_by, display_name,
                changed_note_summary, page_count, active_page_count,
                archived_page_count, deleted_page_count, created_time
         FROM notes_project_history_versions
         WHERE id = ?",
    )
    .bind(&version_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("reload Notes project history version: {e}"))?;
    let version = version_from_row(&row)?;
    tx.commit()
        .await
        .map_err(|e| format!("commit Notes project history checkpoint: {e}"))?;
    Ok(Some(version))
}

pub(in crate::notes) async fn mark_page_dirty_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
    summary: &str,
    force_checkpoint: bool,
) -> Result<(), String> {
    let project_id = resolve_project_id_for_page_tx(tx, page_id).await?;
    let Some(project_id) = project_id else {
        return Ok(());
    };
    if effective_retention_days_tx(tx, &project_id).await? == 0 {
        return Ok(());
    }
    let local_user = local_user::current_local_user_tx(tx).await?;
    let display_name = local_user::comment_display_name_json(&local_user.display_name);
    sqlx::query(
        "INSERT INTO notes_project_history_dirty (
            project_id, first_dirty_at, last_dirty_at, actor_id,
            actor_display_name, changed_note_summary, force_checkpoint
         ) VALUES (
            ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?, ?, ?, ?
         )
         ON CONFLICT(project_id) DO UPDATE SET
            last_dirty_at = excluded.last_dirty_at,
            actor_id = excluded.actor_id,
            actor_display_name = excluded.actor_display_name,
            changed_note_summary = CASE
                WHEN trim(excluded.changed_note_summary) <> ''
                THEN excluded.changed_note_summary
                ELSE notes_project_history_dirty.changed_note_summary
            END,
            force_checkpoint = MAX(
                notes_project_history_dirty.force_checkpoint,
                excluded.force_checkpoint
            )",
    )
    .bind(&project_id)
    .bind(&local_user.id)
    .bind(display_name)
    .bind(summary.trim())
    .bind(i64::from(force_checkpoint))
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("mark Notes project history dirty: {e}"))?;
    Ok(())
}

async fn resolve_project_id_for_page_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar(
        "WITH RECURSIVE ancestors(id, parent_page_id, properties) AS (
             SELECT page.id,
                    COALESCE(
                        page.parent_page_id,
                        (SELECT block.page_id FROM notes_blocks AS block
                         WHERE block.id = page.parent_block_id),
                        (SELECT COALESCE(database.parent_page_id, database_block.page_id)
                         FROM notes_data_sources AS data_source
                         JOIN notes_databases AS database ON database.id = data_source.database_id
                         LEFT JOIN notes_blocks AS database_block ON database_block.id = database.id
                         WHERE data_source.id = page.parent_data_source_id)
                    ),
                    page.properties
             FROM notes_pages AS page
             WHERE page.id = ?
             UNION ALL
             SELECT parent.id,
                    COALESCE(
                        parent.parent_page_id,
                        (SELECT block.page_id FROM notes_blocks AS block
                         WHERE block.id = parent.parent_block_id),
                        (SELECT COALESCE(database.parent_page_id, database_block.page_id)
                         FROM notes_data_sources AS data_source
                         JOIN notes_databases AS database ON database.id = data_source.database_id
                         LEFT JOIN notes_blocks AS database_block ON database_block.id = database.id
                         WHERE data_source.id = parent.parent_data_source_id)
                    ),
                    parent.properties
             FROM notes_pages AS parent
             JOIN ancestors AS child ON child.parent_page_id = parent.id
         )
         SELECT project.id
         FROM ancestors
         JOIN projects AS project
           ON project.id = trim(json_extract(ancestors.properties, '$.__ganbaru_project_id'))
         LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("resolve Notes history project for page: {e}"))
}

pub(in crate::notes) async fn page_history_enabled_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
) -> Result<bool, String> {
    if let Some(project_id) = resolve_project_id_for_page_tx(tx, page_id).await? {
        return Ok(effective_retention_days_tx(tx, &project_id).await? > 0);
    }
    let retention_days: Option<i64> =
        sqlx::query_scalar("SELECT retention_days FROM notes_page_history_settings WHERE id = 1")
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load global Notes history retention: {e}"))?;
    let retention_days = retention_days.unwrap_or(DEFAULT_RETENTION_DAYS);
    validate_retention_days(retention_days)?;
    Ok(retention_days > 0)
}

pub(in crate::notes) async fn create_safety_checkpoint_for_page(
    pool: &SqlitePool,
    page_id: &str,
    reason: &str,
) -> Result<(), String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin Notes safety checkpoint project lookup: {e}"))?;
    let project_id = resolve_project_id_for_page_tx(&mut tx, page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit Notes safety checkpoint project lookup: {e}"))?;
    if let Some(project_id) = project_id {
        create_checkpoint(pool, &project_id, reason, None, None, "Safety checkpoint").await?;
    }
    Ok(())
}

pub(in crate::notes) async fn mark_data_source_dirty_tx(
    tx: &mut Transaction<'_, Sqlite>,
    data_source_id: &str,
    summary: &str,
    force_checkpoint: bool,
) -> Result<(), String> {
    let page_id: Option<String> = sqlx::query_scalar(
        "SELECT page_id
         FROM (
             SELECT database.parent_page_id AS page_id, 1 AS priority
             FROM notes_data_sources AS data_source
             JOIN notes_databases AS database ON database.id = data_source.database_id
             WHERE data_source.id = ? AND database.parent_page_id IS NOT NULL
             UNION ALL
             SELECT block.page_id AS page_id, 2 AS priority
             FROM notes_data_sources AS data_source
             JOIN notes_databases AS database ON database.id = data_source.database_id
             JOIN notes_blocks AS block ON block.id = database.id
             WHERE data_source.id = ?
             UNION ALL
             SELECT row_page.id AS page_id, 3 AS priority
             FROM notes_pages AS row_page
             WHERE row_page.parent_data_source_id = ?
         )
         ORDER BY priority
         LIMIT 1",
    )
    .bind(data_source_id)
    .bind(data_source_id)
    .bind(data_source_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("resolve Notes history page for data source: {e}"))?;
    if let Some(page_id) = page_id {
        mark_page_dirty_tx(tx, &page_id, summary, force_checkpoint).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn notes_initialize_project_history<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
) -> Result<Option<NotesProjectHistoryVersionDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    create_checkpoint(
        &pool,
        &project_id,
        "baseline",
        None,
        None,
        "Initial version",
    )
    .await
}

#[tauri::command]
pub async fn notes_flush_due_project_history<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<i64, String> {
    let pool = connect_sqlite(app, db_url).await?;
    flush_due_checkpoints(&pool).await
}

pub(in crate::notes) async fn flush_due_checkpoints(pool: &SqlitePool) -> Result<i64, String> {
    let rows = sqlx::query(
        "SELECT project_id, actor_id, actor_display_name, changed_note_summary
         FROM notes_project_history_dirty
         WHERE force_checkpoint = 1
            OR first_dirty_at <= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' minutes')
            OR last_dirty_at <= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' minutes')
         ORDER BY first_dirty_at, project_id",
    )
    .bind(ACTIVE_CHECKPOINT_MINUTES)
    .bind(IDLE_CHECKPOINT_MINUTES)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("list due Notes project history checkpoints: {e}"))?;
    let mut created = 0_i64;
    for row in rows {
        let display_name_json: String = row
            .try_get("actor_display_name")
            .map_err(|e| e.to_string())?;
        let display_name: Value = serde_json::from_str(&display_name_json)
            .map_err(|e| format!("parse Notes history dirty actor: {e}"))?;
        let resolved_name = display_name
            .get("resolved_name")
            .and_then(Value::as_str)
            .unwrap_or("You");
        let project_id: String = row.try_get("project_id").map_err(|e| e.to_string())?;
        let actor_id: String = row.try_get("actor_id").map_err(|e| e.to_string())?;
        let summary: String = row
            .try_get("changed_note_summary")
            .map_err(|e| e.to_string())?;
        let version = create_checkpoint(
            pool,
            &project_id,
            "checkpoint",
            Some(&actor_id),
            Some(resolved_name),
            &summary,
        )
        .await?;
        if version.is_some() {
            created += 1;
        }
    }
    let mut maintenance_tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin Notes history startup maintenance: {e}"))?;
    super::history::cleanup_history_retention_tx(&mut maintenance_tx).await?;
    let project_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM projects ORDER BY id")
        .fetch_all(&mut *maintenance_tx)
        .await
        .map_err(|e| format!("list projects for Notes history maintenance: {e}"))?;
    for project_id in project_ids {
        prune_project_history_tx(&mut maintenance_tx, &project_id).await?;
    }
    maintenance_tx
        .commit()
        .await
        .map_err(|e| format!("commit Notes history startup maintenance: {e}"))?;
    Ok(created)
}

#[tauri::command]
pub async fn notes_list_project_history_versions<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    cursor_time: Option<String>,
    cursor_id: Option<String>,
    page_size: Option<i64>,
) -> Result<NotesProjectHistoryVersionListDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let project_id = validate_project_id(&project_id)?;
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let rows = sqlx::query(
        "SELECT id, project_id, manifest_hash, reason, created_by, display_name,
                changed_note_summary, page_count, active_page_count,
                archived_page_count, deleted_page_count, created_time
         FROM notes_project_history_versions
         WHERE project_id = ?
           AND (? IS NULL OR created_time < ? OR (created_time = ? AND id < ?))
         ORDER BY created_time DESC, id DESC
         LIMIT ?",
    )
    .bind(&project_id)
    .bind(cursor_time.as_deref())
    .bind(cursor_time.as_deref())
    .bind(cursor_time.as_deref())
    .bind(cursor_id.as_deref())
    .bind(page_size + 1)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("list Notes project history versions: {e}"))?;
    let has_more = rows.len() > page_size as usize;
    let mut versions = rows
        .iter()
        .take(page_size as usize)
        .map(version_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let (next_cursor_time, next_cursor_id) = if has_more {
        versions
            .last()
            .map(|version| (Some(version.created_time.clone()), Some(version.id.clone())))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };
    Ok(NotesProjectHistoryVersionListDto {
        versions: std::mem::take(&mut versions),
        next_cursor_time,
        next_cursor_id,
    })
}

#[tauri::command]
pub async fn notes_load_project_history_tree<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
) -> Result<NotesProjectHistoryTreeDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let project_id = validate_project_id(&project_id)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let (manifest, version) = load_manifest_tx(&mut tx, &project_id, &version_id).await?;
    let rows = load_manifest_rows_tx(&mut tx, &manifest).await?;
    let pages = rows
        .get("notes_pages")
        .into_iter()
        .flatten()
        .map(page_summary_from_value)
        .collect::<Result<Vec<_>, _>>()?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(NotesProjectHistoryTreeDto { version, pages })
}

fn page_summary_from_value(row: &Value) -> Result<NotesHistoricalPageSummaryDto, String> {
    Ok(NotesHistoricalPageSummaryDto {
        id: json_string(row, "id")?,
        title: json_string(row, "title")?,
        parent_page_id: json_optional_string(row, "parent_page_id"),
        parent_data_source_id: json_optional_string(row, "parent_data_source_id"),
        in_trash: json_bool(row, "in_trash"),
        archived: json_bool(row, "archived"),
        icon: row.get("icon").filter(|value| !value.is_null()).cloned(),
    })
}

fn json_string(row: &Value, key: &str) -> Result<String, String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Notes history row is missing {key}"))
}

fn json_optional_string(row: &Value, key: &str) -> Option<String> {
    row.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn json_bool(row: &Value, key: &str) -> bool {
    row.get(key)
        .and_then(Value::as_i64)
        .is_some_and(|value| value != 0)
}

#[tauri::command]
pub async fn notes_load_project_history_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
    page_id: String,
) -> Result<NotesHistoricalPageDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let project_id = validate_project_id(&project_id)?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let (manifest, _) = load_manifest_tx(&mut tx, &project_id, &version_id).await?;
    let rows = load_manifest_rows_tx(&mut tx, &manifest).await?;
    let page = rows
        .get("notes_pages")
        .into_iter()
        .flatten()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(page_id.as_str()))
        .ok_or_else(|| "Historical Notes page not found".to_string())?;
    let databases = rows_for_page(&rows, "notes_databases", "parent_page_id", &page_id);
    let database_ids = databases
        .iter()
        .filter_map(|row| json_optional_string(row, "id"))
        .collect::<HashSet<_>>();
    let data_sources = rows
        .get("notes_data_sources")
        .into_iter()
        .flatten()
        .filter(|row| {
            json_optional_string(row, "database_id").is_some_and(|id| database_ids.contains(&id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let data_source_ids = data_sources
        .iter()
        .filter_map(|row| json_optional_string(row, "id"))
        .collect::<HashSet<_>>();
    let database_views = rows
        .get("notes_database_views")
        .into_iter()
        .flatten()
        .filter(|row| {
            json_optional_string(row, "database_id").is_some_and(|id| database_ids.contains(&id))
                || json_optional_string(row, "data_source_id")
                    .is_some_and(|id| data_source_ids.contains(&id))
        })
        .cloned()
        .collect::<Vec<_>>();
    let result = NotesHistoricalPageDto {
        id: json_string(page, "id")?,
        title: json_string(page, "title")?,
        properties: page.get("properties").cloned().unwrap_or(Value::Null),
        icon: page.get("icon").filter(|value| !value.is_null()).cloned(),
        cover: page.get("cover").filter(|value| !value.is_null()).cloned(),
        in_trash: json_bool(page, "in_trash"),
        archived: json_bool(page, "archived"),
        blocks: rows_for_page(&rows, "notes_blocks", "page_id", &page_id),
        databases,
        data_sources,
        database_views,
    };
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(result)
}

fn rows_for_page(
    rows: &BTreeMap<String, Vec<Value>>,
    table: &str,
    key: &str,
    page_id: &str,
) -> Vec<Value> {
    rows.get(table)
        .into_iter()
        .flatten()
        .filter(|row| row.get(key).and_then(Value::as_str) == Some(page_id))
        .cloned()
        .collect()
}

#[tauri::command]
pub async fn notes_get_history_retention_impact<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: Option<String>,
    retention_days: i64,
) -> Result<NotesHistoryRetentionImpactDto, String> {
    validate_retention_days(retention_days)?;
    let pool = connect_sqlite(app, db_url).await?;
    let project_filter = project_id.as_deref().map(validate_project_id).transpose()?;
    let row = if let Some(project_id) = project_filter.as_deref() {
        sqlx::query(
            "WITH affected_versions AS (
                 SELECT id, manifest_hash
                 FROM notes_project_history_versions
                 WHERE project_id = ?
                   AND (
                       ? = 0
                       OR created_time < strftime(
                           '%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' days'
                       )
                   )
             ),
             affected_bundles AS (
                 SELECT manifest_hash AS hash FROM affected_versions
                 UNION
                 SELECT reference.bundle_hash
                 FROM notes_project_history_bundle_references AS reference
                 JOIN affected_versions AS version ON version.id = reference.version_id
             ),
             retained_bundles AS (
                 SELECT manifest_hash AS hash
                 FROM notes_project_history_versions
                 WHERE id NOT IN (SELECT id FROM affected_versions)
                 UNION
                 SELECT reference.bundle_hash
                 FROM notes_project_history_bundle_references AS reference
                 WHERE reference.version_id NOT IN (SELECT id FROM affected_versions)
             ),
             reclaimable_bundles AS (
                 SELECT hash FROM affected_bundles
                 EXCEPT SELECT hash FROM retained_bundles
             )
             SELECT (SELECT COUNT(*) FROM affected_versions) AS version_count,
                    COALESCE((
                        SELECT SUM(bundle.stored_bytes)
                        FROM notes_history_bundles AS bundle
                        JOIN reclaimable_bundles AS reclaimable ON reclaimable.hash = bundle.hash
                    ), 0) AS stored_bytes",
        )
        .bind(project_id)
        .bind(retention_days)
        .bind(retention_days)
        .fetch_one(&pool)
        .await
    } else {
        sqlx::query(
            "WITH affected_versions AS (
                 SELECT version.id, version.manifest_hash
                 FROM notes_project_history_versions AS version
                 JOIN projects AS project ON project.id = version.project_id
                 WHERE project.notes_history_retention_days IS NULL
                   AND (
                       ? = 0
                       OR version.created_time < strftime(
                           '%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' days'
                       )
                   )
             ),
             affected_bundles AS (
                 SELECT manifest_hash AS hash FROM affected_versions
                 UNION
                 SELECT reference.bundle_hash
                 FROM notes_project_history_bundle_references AS reference
                 JOIN affected_versions AS version ON version.id = reference.version_id
             ),
             retained_bundles AS (
                 SELECT manifest_hash AS hash
                 FROM notes_project_history_versions
                 WHERE id NOT IN (SELECT id FROM affected_versions)
                 UNION
                 SELECT reference.bundle_hash
                 FROM notes_project_history_bundle_references AS reference
                 WHERE reference.version_id NOT IN (SELECT id FROM affected_versions)
             ),
             reclaimable_bundles AS (
                 SELECT hash FROM affected_bundles
                 EXCEPT SELECT hash FROM retained_bundles
             )
             SELECT (SELECT COUNT(*) FROM affected_versions) AS version_count,
                    COALESCE((
                        SELECT SUM(bundle.stored_bytes)
                        FROM notes_history_bundles AS bundle
                        JOIN reclaimable_bundles AS reclaimable ON reclaimable.hash = bundle.hash
                    ), 0) AS stored_bytes",
        )
        .bind(retention_days)
        .bind(retention_days)
        .fetch_one(&pool)
        .await
    }
    .map_err(|e| format!("calculate Notes history retention impact: {e}"))?;
    let legacy_row = if let Some(project_id) = project_filter.as_deref() {
        sqlx::query(
            "WITH RECURSIVE ownership(page_id, parent_page_id, project_id) AS (
                 SELECT page.id,
                        COALESCE(
                            page.parent_page_id,
                            (SELECT block.page_id FROM notes_blocks AS block
                             WHERE block.id = page.parent_block_id),
                            (SELECT COALESCE(database.parent_page_id, database_block.page_id)
                             FROM notes_data_sources AS data_source
                             JOIN notes_databases AS database ON database.id = data_source.database_id
                             LEFT JOIN notes_blocks AS database_block ON database_block.id = database.id
                             WHERE data_source.id = page.parent_data_source_id)
                        ),
                        NULLIF(trim(json_extract(properties, '$.__ganbaru_project_id')), '')
                 FROM notes_pages AS page
                 UNION ALL
                 SELECT ownership.page_id,
                        parent.parent_page_id,
                        COALESCE(
                            ownership.project_id,
                            NULLIF(trim(json_extract(parent.properties, '$.__ganbaru_project_id')), '')
                        )
                 FROM ownership
                 JOIN notes_pages AS parent ON parent.id = ownership.parent_page_id
                 WHERE ownership.project_id IS NULL
             ),
             resolved AS (
                 SELECT page_id, MAX(project_id) AS project_id
                 FROM ownership
                 GROUP BY page_id
             )
             SELECT COUNT(*) AS version_count,
                    COALESCE(SUM(
                        COALESCE(bundle.stored_bytes, length(snapshot.blocks))
                        + length(snapshot.properties)
                        + length(snapshot.title)
                        + COALESCE(length(snapshot.icon), 0)
                        + COALESCE(length(snapshot.cover), 0)
                    ), 0) AS stored_bytes
             FROM notes_page_history_snapshots AS snapshot
             JOIN resolved ON resolved.page_id = snapshot.page_id
             LEFT JOIN notes_history_bundles AS bundle
               ON bundle.hash = snapshot.block_bundle_hash
             WHERE resolved.project_id = ?
               AND (
                   ? = 0
                   OR snapshot.created_time < strftime(
                       '%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' days'
                   )
               )",
        )
        .bind(project_id)
        .bind(retention_days)
        .bind(retention_days)
        .fetch_one(&pool)
        .await
    } else {
        sqlx::query(
            "WITH RECURSIVE ownership(page_id, parent_page_id, project_id) AS (
                 SELECT page.id,
                        COALESCE(
                            page.parent_page_id,
                            (SELECT block.page_id FROM notes_blocks AS block
                             WHERE block.id = page.parent_block_id),
                            (SELECT COALESCE(database.parent_page_id, database_block.page_id)
                             FROM notes_data_sources AS data_source
                             JOIN notes_databases AS database ON database.id = data_source.database_id
                             LEFT JOIN notes_blocks AS database_block ON database_block.id = database.id
                             WHERE data_source.id = page.parent_data_source_id)
                        ),
                        NULLIF(trim(json_extract(properties, '$.__ganbaru_project_id')), '')
                 FROM notes_pages AS page
                 UNION ALL
                 SELECT ownership.page_id,
                        parent.parent_page_id,
                        COALESCE(
                            ownership.project_id,
                            NULLIF(trim(json_extract(parent.properties, '$.__ganbaru_project_id')), '')
                        )
                 FROM ownership
                 JOIN notes_pages AS parent ON parent.id = ownership.parent_page_id
                 WHERE ownership.project_id IS NULL
             ),
             resolved AS (
                 SELECT page_id, MAX(project_id) AS project_id
                 FROM ownership
                 GROUP BY page_id
             )
             SELECT COUNT(*) AS version_count,
                    COALESCE(SUM(
                        COALESCE(bundle.stored_bytes, length(snapshot.blocks))
                        + length(snapshot.properties)
                        + length(snapshot.title)
                        + COALESCE(length(snapshot.icon), 0)
                        + COALESCE(length(snapshot.cover), 0)
                    ), 0) AS stored_bytes
             FROM notes_page_history_snapshots AS snapshot
             JOIN resolved ON resolved.page_id = snapshot.page_id
             JOIN projects AS project ON project.id = resolved.project_id
             LEFT JOIN notes_history_bundles AS bundle
               ON bundle.hash = snapshot.block_bundle_hash
             WHERE project.notes_history_retention_days IS NULL
               AND (
                   ? = 0
                   OR snapshot.created_time < strftime(
                       '%Y-%m-%dT%H:%M:%fZ', 'now', '-' || ? || ' days'
                   )
               )",
        )
        .bind(retention_days)
        .bind(retention_days)
        .fetch_one(&pool)
        .await
    }
    .map_err(|e| format!("calculate legacy Notes history retention impact: {e}"))?;
    let project_version_count: i64 = row.try_get("version_count").map_err(|e| e.to_string())?;
    let project_stored_bytes: i64 = row.try_get("stored_bytes").map_err(|e| e.to_string())?;
    let legacy_version_count: i64 = legacy_row
        .try_get("version_count")
        .map_err(|e| e.to_string())?;
    let legacy_stored_bytes: i64 = legacy_row
        .try_get("stored_bytes")
        .map_err(|e| e.to_string())?;
    Ok(NotesHistoryRetentionImpactDto {
        version_count: project_version_count.saturating_add(legacy_version_count),
        stored_bytes: project_stored_bytes.saturating_add(legacy_stored_bytes),
    })
}

#[tauri::command]
pub async fn notes_prune_project_history<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
) -> Result<i64, String> {
    let pool = connect_sqlite(app, db_url).await?;
    let project_id = validate_project_id(&project_id)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin Notes project history pruning: {e}"))?;
    super::history::cleanup_history_retention_tx(&mut tx).await?;
    let pruned = prune_project_history_tx(&mut tx, &project_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit Notes project history pruning: {e}"))?;
    Ok(pruned)
}

#[tauri::command]
pub async fn notes_preview_project_history_restore<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
) -> Result<NotesProjectHistoryRestorePlanDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    restore::preview_restore(&pool, &project_id, &version_id).await
}

#[tauri::command]
pub async fn notes_restore_project_history_version<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    project_id: String,
    version_id: String,
) -> Result<NotesProjectHistoryVersionDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    restore::restore_version(&pool, &project_id, &version_id).await
}

#[cfg(test)]
mod tests {
    use super::{checkpoint_is_due, create_checkpoint, normalize_legacy_retention_days, restore};
    use crate::db::run_migrations;
    use sqlx::SqlitePool;

    const PROJECT_ID: &str = "10101010-1010-4010-8010-101010101010";
    const PAGE_ID: &str = "20202020-2020-4020-8020-202020202020";
    const BLOCK_ID: &str = "30303030-3030-4030-8030-303030303030";
    const LATER_PAGE_ID: &str = "40404040-4040-4040-8040-404040404040";
    const LATER_BLOCK_ID: &str = "50505050-5050-4050-8050-505050505050";

    async fn migrated_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::raw_sql("PRAGMA foreign_keys=ON")
            .execute(&pool)
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    async fn seed_project(pool: &SqlitePool) {
        sqlx::query("INSERT INTO project_groups (id, name) VALUES ('group-history', 'History')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO projects (id, group_id, name) VALUES (?, 'group-history', 'Learning')",
        )
        .bind(PROJECT_ID)
        .execute(pool)
        .await
        .unwrap();
        insert_project_page(pool, PAGE_ID, BLOCK_ID, "First version").await;
    }

    async fn insert_project_page(pool: &SqlitePool, page_id: &str, block_id: &str, text: &str) {
        sqlx::query(
            "INSERT INTO notes_pages (id, parent_type, title, properties)
             VALUES (?, 'workspace', ?, json_object('__ganbaru_project_id', ?, 'title', json_object()))",
        )
        .bind(page_id)
        .bind(text)
        .bind(PROJECT_ID)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO notes_blocks (
                id, page_id, parent_type, parent_page_id, type, payload, plain_text, sort_order
             ) VALUES (?, ?, 'page_id', ?, 'paragraph', ?, ?, 1000)",
        )
        .bind(block_id)
        .bind(page_id)
        .bind(page_id)
        .bind(
            serde_json::json!({
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": { "content": text, "link": null },
                        "annotations": {
                            "bold": false,
                            "italic": false,
                            "strikethrough": false,
                            "underline": false,
                            "code": false,
                            "color": "default"
                        },
                        "plain_text": text,
                        "href": null
                    }],
                    "color": "default"
                }
            })
            .to_string(),
        )
        .bind(text)
        .execute(pool)
        .await
        .unwrap();
    }

    #[test]
    fn legacy_retention_values_round_up_and_remove_forever() {
        assert_eq!(normalize_legacy_retention_days(Some(7)), 7);
        assert_eq!(normalize_legacy_retention_days(Some(8)), 30);
        assert_eq!(normalize_legacy_retention_days(Some(31)), 90);
        assert_eq!(normalize_legacy_retention_days(Some(91)), 180);
        assert_eq!(normalize_legacy_retention_days(Some(181)), 365);
        assert_eq!(normalize_legacy_retention_days(Some(365)), 365);
        assert_eq!(normalize_legacy_retention_days(Some(3_650)), 365);
        assert_eq!(normalize_legacy_retention_days(None), 365);
    }

    #[test]
    fn checkpoint_planner_handles_active_idle_restart_and_forced_flushes() {
        assert!(!checkpoint_is_due(599, 119, false));
        assert!(checkpoint_is_due(600, 1, false));
        assert!(checkpoint_is_due(30, 120, false));
        assert!(checkpoint_is_due(0, 0, true));
        assert!(checkpoint_is_due(3_600, 3_600, false));
    }

    #[test]
    fn checkpoints_deduplicate_rows_and_skip_unchanged_manifests() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_pool().await;
            seed_project(&pool).await;
            let first =
                create_checkpoint(&pool, PROJECT_ID, "baseline", None, None, "Initial version")
                    .await
                    .unwrap();
            assert!(first.is_some());
            let initial_bundle_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM notes_history_bundles")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let unchanged =
                create_checkpoint(&pool, PROJECT_ID, "checkpoint", None, None, "No change")
                    .await
                    .unwrap();
            assert!(unchanged.is_none());
            let unchanged_bundle_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM notes_history_bundles")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(unchanged_bundle_count, initial_bundle_count);

            sqlx::query(
                "UPDATE notes_blocks
                 SET plain_text = 'Second version',
                     payload = json_set(payload, '$.paragraph.rich_text[0].text.content', 'Second version')
                 WHERE id = ?",
            )
            .bind(BLOCK_ID)
            .execute(&pool)
            .await
            .unwrap();
            let changed = create_checkpoint(
                &pool,
                PROJECT_ID,
                "checkpoint",
                None,
                None,
                "Edited one note",
            )
            .await
            .unwrap();
            assert!(changed.is_some());
            let changed_bundle_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM notes_history_bundles")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(changed_bundle_count, initial_bundle_count + 2);
        });
    }

    #[test]
    fn disabled_project_history_prunes_versions_and_skips_new_checkpoints() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_pool().await;
            seed_project(&pool).await;
            assert!(create_checkpoint(
                &pool,
                PROJECT_ID,
                "baseline",
                None,
                None,
                "Initial version",
            )
            .await
            .unwrap()
            .is_some());

            sqlx::query(
                "UPDATE projects
                 SET notes_history_retention_days = 0
                 WHERE id = ?",
            )
            .bind(PROJECT_ID)
            .execute(&pool)
            .await
            .unwrap();
            assert!(create_checkpoint(
                &pool,
                PROJECT_ID,
                "checkpoint",
                None,
                None,
                "This version must not be stored",
            )
            .await
            .unwrap()
            .is_none());

            let version_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM notes_project_history_versions
                 WHERE project_id = ?",
            )
            .bind(PROJECT_ID)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(version_count, 0);
        });
    }

    #[test]
    fn project_restore_removes_later_notes_and_keeps_a_safety_version() {
        tauri::async_runtime::block_on(async {
            let pool = migrated_pool().await;
            seed_project(&pool).await;
            let baseline =
                create_checkpoint(&pool, PROJECT_ID, "baseline", None, None, "Initial version")
                    .await
                    .unwrap()
                    .unwrap();
            insert_project_page(&pool, LATER_PAGE_ID, LATER_BLOCK_ID, "Later note").await;
            restore::restore_version(&pool, PROJECT_ID, &baseline.id)
                .await
                .unwrap();
            let later_exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM notes_pages WHERE id = ?")
                    .bind(LATER_PAGE_ID)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(later_exists, 0);
            let versions: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM notes_project_history_versions WHERE project_id = ?",
            )
            .bind(PROJECT_ID)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(versions >= 3);
        });
    }
}
