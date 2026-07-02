use super::models::{
    NoteBlockDto, NoteBlockRow, NoteLoadedPage, NotePageBreadcrumbItemDto, NotePageDto,
    NotePageRow, NotePaginatedBlockList, NoteSidebarPageList, NoteSidebarPagesRequest,
};
use super::validation::{require_uuid, validate_page_size};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::{HashMap, HashSet};

const DEFAULT_PAGE_SIZE: i64 = 50;

pub(in crate::notes) async fn list_pages(pool: &SqlitePool) -> Result<Vec<NotePageDto>, String> {
    list_pages_by_state(pool, false, Some(false)).await
}

pub(in crate::notes) async fn list_trashed_pages(
    pool: &SqlitePool,
) -> Result<Vec<NotePageDto>, String> {
    list_pages_by_state(pool, true, None).await
}

pub(in crate::notes) async fn list_archived_pages(
    pool: &SqlitePool,
) -> Result<Vec<NotePageDto>, String> {
    list_pages_by_state(pool, false, Some(true)).await
}

pub(in crate::notes) async fn list_sidebar_pages(
    pool: &SqlitePool,
    request: NoteSidebarPagesRequest,
) -> Result<NoteSidebarPageList, String> {
    let expanded_page_ids = normalize_request_page_ids(request.expanded_page_ids);
    let mut seed_page_ids = normalize_request_page_ids(request.seed_page_ids);
    if let Some(selected_page_id) = request
        .selected_page_id
        .as_deref()
        .and_then(normalize_request_page_id)
        .filter(|page_id| !seed_page_ids.contains(page_id))
    {
        seed_page_ids.push(selected_page_id);
    }

    let mut rows_by_id = HashMap::<String, NotePageRow>::new();
    push_unique_page_rows(&mut rows_by_id, fetch_sidebar_root_page_rows(pool).await?);
    push_unique_page_rows(
        &mut rows_by_id,
        fetch_active_page_rows_by_ids(pool, &seed_page_ids).await?,
    );
    for page_id in &seed_page_ids {
        push_unique_page_rows(
            &mut rows_by_id,
            fetch_active_ancestor_page_rows(pool, page_id).await?,
        );
    }
    push_unique_page_rows(
        &mut rows_by_id,
        fetch_active_child_page_rows(pool, &expanded_page_ids).await?,
    );

    let mut rows = rows_by_id.into_values().collect::<Vec<_>>();
    sort_page_rows(&mut rows);
    let loaded_page_ids = rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let page_ids_with_children =
        fetch_active_parent_page_ids_with_children(pool, &loaded_page_ids).await?;
    let (missing_parent_page_ids, trashed_parent_page_ids) =
        fetch_unavailable_parent_page_ids(pool, &rows, &loaded_page_ids).await?;

    NoteSidebarPageList::new(
        rows,
        page_ids_with_children,
        missing_parent_page_ids,
        trashed_parent_page_ids,
    )
}

pub(in crate::notes) async fn get_page_breadcrumb(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<Vec<NotePageBreadcrumbItemDto>, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    let mut crumbs = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor = Some(page_id.to_string());
    while let Some(current_page_id) = cursor {
        if !seen.insert(current_page_id.clone()) {
            break;
        }
        let Some(row) = fetch_page_row_any_state(pool, &current_page_id).await? else {
            crumbs.push(NotePageBreadcrumbItemDto::missing(current_page_id));
            break;
        };
        let current = row.id == page_id;
        if current && (row.in_trash != 0 || row.archived != 0) {
            return Err("notes page not found".to_string());
        }
        let parent_page_id = if row.parent_type == "page_id" {
            row.parent_page_id.clone()
        } else {
            None
        };
        let crumb = if row.in_trash != 0 {
            NotePageBreadcrumbItemDto::unavailable(row, "trashed")
        } else if row.archived != 0 {
            NotePageBreadcrumbItemDto::unavailable(row, "archived")
        } else {
            NotePageBreadcrumbItemDto::active(row, current)
        };
        crumbs.push(crumb);
        cursor = parent_page_id;
    }
    crumbs.reverse();
    Ok(crumbs)
}

async fn list_pages_by_state(
    pool: &SqlitePool,
    in_trash: bool,
    archived: Option<bool>,
) -> Result<Vec<NotePageDto>, String> {
    let rows = if let Some(archived) = archived {
        sqlx::query_as::<_, NotePageRow>(
            "SELECT *
             FROM notes_pages
             WHERE in_trash = ? AND archived = ?
             ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC",
        )
        .bind(if in_trash { 1_i64 } else { 0_i64 })
        .bind(if archived { 1_i64 } else { 0_i64 })
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NotePageRow>(
            "SELECT *
             FROM notes_pages
             WHERE in_trash = ?
             ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC",
        )
        .bind(if in_trash { 1_i64 } else { 0_i64 })
        .fetch_all(pool)
        .await
    }
    .map_err(|e| format!("list notes pages: {e}"))?;
    rows.into_iter().map(NotePageDto::new).collect()
}

fn normalize_request_page_ids(page_ids: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    page_ids
        .into_iter()
        .filter_map(|page_id| normalize_request_page_id(&page_id))
        .filter(|page_id| seen.insert(page_id.clone()))
        .collect()
}

fn normalize_request_page_id(page_id: &str) -> Option<String> {
    let trimmed = page_id.trim();
    require_uuid(trimmed, "page_id").ok()?;
    Some(trimmed.to_string())
}

fn push_unique_page_rows(rows_by_id: &mut HashMap<String, NotePageRow>, rows: Vec<NotePageRow>) {
    for row in rows {
        rows_by_id.entry(row.id.clone()).or_insert(row);
    }
}

fn sort_page_rows(rows: &mut [NotePageRow]) {
    rows.sort_by(|left, right| {
        right
            .last_edited_time
            .cmp(&left.last_edited_time)
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
}

async fn fetch_sidebar_root_page_rows(pool: &SqlitePool) -> Result<Vec<NotePageRow>, String> {
    sqlx::query_as::<_, NotePageRow>(
        "SELECT *
         FROM notes_pages
         WHERE in_trash = 0
           AND archived = 0
           AND parent_type <> 'page_id'
           AND parent_type <> 'data_source_id'
         ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("list notes sidebar root pages: {e}"))
}

async fn fetch_active_page_rows_by_ids(
    pool: &SqlitePool,
    page_ids: &[String],
) -> Result<Vec<NotePageRow>, String> {
    if page_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT * FROM notes_pages WHERE in_trash = 0 AND archived = 0 AND id IN (",
    );
    let mut separated = query.separated(", ");
    for page_id in page_ids {
        separated.push_bind(page_id);
    }
    query.push(") ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC");
    query
        .build_query_as::<NotePageRow>()
        .fetch_all(pool)
        .await
        .map_err(|e| format!("list notes sidebar pages by id: {e}"))
}

async fn fetch_active_child_page_rows(
    pool: &SqlitePool,
    parent_page_ids: &[String],
) -> Result<Vec<NotePageRow>, String> {
    if parent_page_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT * FROM notes_pages
         WHERE in_trash = 0
           AND archived = 0
           AND parent_type = 'page_id'
           AND parent_page_id IN (",
    );
    let mut separated = query.separated(", ");
    for page_id in parent_page_ids {
        separated.push_bind(page_id);
    }
    query.push(") ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC");
    query
        .build_query_as::<NotePageRow>()
        .fetch_all(pool)
        .await
        .map_err(|e| format!("list notes sidebar child pages: {e}"))
}

async fn fetch_active_ancestor_page_rows(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<Vec<NotePageRow>, String> {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut cursor = Some(page_id.to_string());
    while let Some(current_page_id) = cursor {
        let Some(row) = fetch_page_row_any_state(pool, &current_page_id).await? else {
            break;
        };
        if !seen.insert(row.id.clone()) {
            break;
        }
        let parent_page_id = if row.parent_type == "page_id" {
            row.parent_page_id.clone()
        } else {
            None
        };
        if row.id != page_id && row.in_trash == 0 && row.archived == 0 {
            rows.push(row);
        } else if row.in_trash != 0 || row.archived != 0 {
            break;
        }
        cursor = parent_page_id;
    }
    Ok(rows)
}

async fn fetch_page_row_any_state(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<Option<NotePageRow>, String> {
    sqlx::query_as::<_, NotePageRow>(
        "SELECT *
         FROM notes_pages
         WHERE id = ?
         LIMIT 1",
    )
    .bind(page_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load notes page metadata: {e}"))
}

async fn fetch_active_parent_page_ids_with_children(
    pool: &SqlitePool,
    page_ids: &[String],
) -> Result<Vec<String>, String> {
    if page_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT DISTINCT parent_page_id
         FROM notes_pages
         WHERE in_trash = 0
           AND archived = 0
           AND parent_type = 'page_id'
           AND parent_page_id IN (",
    );
    let mut separated = query.separated(", ");
    for page_id in page_ids {
        separated.push_bind(page_id);
    }
    query.push(") ORDER BY parent_page_id ASC");
    let rows = query
        .build_query_scalar::<String>()
        .fetch_all(pool)
        .await
        .map_err(|e| format!("list notes sidebar child page markers: {e}"))?;
    Ok(rows)
}

async fn fetch_unavailable_parent_page_ids(
    pool: &SqlitePool,
    rows: &[NotePageRow],
    loaded_page_ids: &[String],
) -> Result<(Vec<String>, Vec<String>), String> {
    let loaded_page_id_set = loaded_page_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut checked_parent_ids = HashSet::new();
    let mut missing_parent_page_ids = Vec::new();
    let mut trashed_parent_page_ids = Vec::new();
    for row in rows {
        if row.parent_type != "page_id" {
            continue;
        }
        let Some(parent_page_id) = row.parent_page_id.as_deref() else {
            continue;
        };
        if loaded_page_id_set.contains(parent_page_id)
            || !checked_parent_ids.insert(parent_page_id.to_string())
        {
            continue;
        }
        match fetch_page_row_any_state(pool, parent_page_id).await? {
            None => missing_parent_page_ids.push(parent_page_id.to_string()),
            Some(parent) if parent.in_trash != 0 => {
                trashed_parent_page_ids.push(parent_page_id.to_string());
            }
            Some(parent) if parent.archived != 0 => {
                missing_parent_page_ids.push(parent_page_id.to_string());
            }
            Some(_) => {}
        }
    }
    missing_parent_page_ids.sort();
    trashed_parent_page_ids.sort();
    Ok((missing_parent_page_ids, trashed_parent_page_ids))
}

pub(in crate::notes) async fn load_page(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<NoteLoadedPage, String> {
    require_uuid(page_id.trim(), "page_id")?;
    let page = get_page(pool, page_id.trim(), false).await?;
    let blocks =
        get_page_block_children(pool, page_id.trim(), None, Some(DEFAULT_PAGE_SIZE)).await?;
    Ok(NoteLoadedPage::new(page, blocks))
}

pub(in crate::notes) async fn get_page(
    pool: &SqlitePool,
    page_id: &str,
    include_trash: bool,
) -> Result<NotePageDto, String> {
    let row = if include_trash {
        sqlx::query_as::<_, NotePageRow>("SELECT * FROM notes_pages WHERE id = ?")
            .bind(page_id)
            .fetch_optional(pool)
            .await
    } else {
        sqlx::query_as::<_, NotePageRow>(
            "SELECT * FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
        )
        .bind(page_id)
        .fetch_optional(pool)
        .await
    }
    .map_err(|e| format!("load notes page: {e}"))?
    .ok_or_else(|| "notes page not found".to_string())?;
    NotePageDto::new(row)
}

pub(in crate::notes) async fn get_block(
    pool: &SqlitePool,
    block_id: &str,
    include_trash: bool,
) -> Result<NoteBlockDto, String> {
    let row = get_block_row(pool, block_id, include_trash).await?;
    NoteBlockDto::new(row)
}

pub(in crate::notes) async fn get_block_row(
    pool: &SqlitePool,
    block_id: &str,
    include_trash: bool,
) -> Result<NoteBlockRow, String> {
    let row = if include_trash {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE id = ?",
        )
        .bind(block_id)
        .fetch_optional(pool)
        .await
    } else {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_optional(pool)
        .await
    }
    .map_err(|e| format!("load notes block: {e}"))?
    .ok_or_else(|| "notes block not found".to_string())?;
    Ok(row)
}

pub(in crate::notes) async fn get_block_children(
    pool: &SqlitePool,
    parent_id: &str,
    start_cursor: Option<&str>,
    page_size: Option<i64>,
) -> Result<NotePaginatedBlockList, String> {
    let parent_id = parent_id.trim();
    require_uuid(parent_id, "parent_id")?;
    if page_exists(pool, parent_id).await? {
        return get_page_block_children(pool, parent_id, start_cursor, page_size).await;
    }
    let parent = get_block_row(pool, parent_id, false).await?;
    get_child_blocks_by_parent_block(pool, &parent.id, start_cursor, page_size).await
}

pub(in crate::notes) async fn get_page_block_children(
    pool: &SqlitePool,
    page_id: &str,
    start_cursor: Option<&str>,
    page_size: Option<i64>,
) -> Result<NotePaginatedBlockList, String> {
    get_child_blocks_by_parent_page(pool, page_id, start_cursor, page_size).await
}

pub(in crate::notes) async fn page_exists(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<bool, String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(page_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("check notes page: {e}"))?;
    Ok(exists.is_some())
}

async fn get_child_blocks_by_parent_page(
    pool: &SqlitePool,
    page_id: &str,
    start_cursor: Option<&str>,
    page_size: Option<i64>,
) -> Result<NotePaginatedBlockList, String> {
    let cursor_order = cursor_sort_order(pool, start_cursor).await?;
    let size = normalized_page_size(page_size)?;
    let rows = if let Some((sort_order, cursor_id)) = cursor_order {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE parent_type = 'page_id'
               AND parent_page_id = ?
               AND in_trash = 0
               AND (sort_order > ? OR (sort_order = ? AND id > ?))
             ORDER BY sort_order ASC, id ASC
             LIMIT ?",
        )
        .bind(page_id)
        .bind(sort_order)
        .bind(sort_order)
        .bind(cursor_id)
        .bind(size + 1)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE parent_type = 'page_id'
               AND parent_page_id = ?
               AND in_trash = 0
             ORDER BY sort_order ASC, id ASC
             LIMIT ?",
        )
        .bind(page_id)
        .bind(size + 1)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| format!("load page block children: {e}"))?;
    paginated_rows(rows, size as usize)
}

async fn get_child_blocks_by_parent_block(
    pool: &SqlitePool,
    block_id: &str,
    start_cursor: Option<&str>,
    page_size: Option<i64>,
) -> Result<NotePaginatedBlockList, String> {
    let cursor_order = cursor_sort_order(pool, start_cursor).await?;
    let size = normalized_page_size(page_size)?;
    let rows = if let Some((sort_order, cursor_id)) = cursor_order {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE parent_type = 'block_id'
               AND parent_block_id = ?
               AND in_trash = 0
               AND (sort_order > ? OR (sort_order = ? AND id > ?))
             ORDER BY sort_order ASC, id ASC
             LIMIT ?",
        )
        .bind(block_id)
        .bind(sort_order)
        .bind(sort_order)
        .bind(cursor_id)
        .bind(size + 1)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NoteBlockRow>(
            "SELECT
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                in_trash,
                type AS block_type,
                payload,
                plain_text,
                sort_order,
                source_provider,
                source_object_id,
                source_last_edited_time,
                created_time,
                last_edited_time
             FROM notes_blocks
             WHERE parent_type = 'block_id'
               AND parent_block_id = ?
               AND in_trash = 0
             ORDER BY sort_order ASC, id ASC
             LIMIT ?",
        )
        .bind(block_id)
        .bind(size + 1)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| format!("load block children: {e}"))?;
    paginated_rows(rows, size as usize)
}

async fn cursor_sort_order(
    pool: &SqlitePool,
    start_cursor: Option<&str>,
) -> Result<Option<(f64, String)>, String> {
    let Some(cursor) = start_cursor
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    require_uuid(cursor, "start_cursor")?;
    let row: Option<(f64, String)> =
        sqlx::query_as("SELECT sort_order, id FROM notes_blocks WHERE id = ? AND in_trash = 0")
            .bind(cursor)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("load notes cursor: {e}"))?;
    row.ok_or_else(|| "start_cursor block not found".to_string())
        .map(Some)
}

fn normalized_page_size(page_size: Option<i64>) -> Result<i64, String> {
    let size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    validate_page_size(size)?;
    Ok(size)
}

fn paginated_rows(
    rows: Vec<NoteBlockRow>,
    page_size: usize,
) -> Result<NotePaginatedBlockList, String> {
    let has_more = rows.len() > page_size;
    let visible_rows = rows.into_iter().take(page_size).collect::<Vec<_>>();
    let next_cursor = if has_more {
        visible_rows.last().map(|row| row.id.clone())
    } else {
        None
    };
    let results = visible_rows
        .into_iter()
        .map(NoteBlockDto::new)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NotePaginatedBlockList::new(results, next_cursor, has_more))
}
