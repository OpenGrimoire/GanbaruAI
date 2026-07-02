use super::data_source_relations;
use super::models::{
    NoteBacklinkDto, NoteBlockDto, NoteBlockRow, NoteCommentRow, NoteLoadedPage,
    NotePageBreadcrumbItemDto, NotePageDto, NotePageRow, NotePaginatedBlockList,
    NoteSearchResultDto, NoteSidebarPageList, NoteSidebarPagesRequest,
};
use super::validation::{require_uuid, validate_page_size};
use serde_json::Value;
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

pub(in crate::notes) async fn list_backlinks(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<Vec<NoteBacklinkDto>, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    if !page_exists(pool, page_id).await? {
        return Err("notes page not found".to_string());
    }
    let rows = sqlx::query_as::<_, NoteBlockRow>(
        "SELECT
            block.id,
            block.page_id,
            block.parent_type,
            block.parent_page_id,
            block.parent_block_id,
            block.has_children,
            block.in_trash,
            block.type AS block_type,
            block.payload,
            block.plain_text,
            block.sort_order,
            block.source_provider,
            block.source_object_id,
            block.source_last_edited_time,
            block.created_time,
            block.last_edited_time
         FROM notes_blocks AS block
         JOIN notes_pages AS source_page ON source_page.id = block.page_id
         WHERE block.in_trash = 0
           AND source_page.in_trash = 0
           AND source_page.archived = 0
           AND (
               (block.type = 'child_page' AND block.id = ?)
               OR block.payload LIKE '%' || ? || '%'
           )
         ORDER BY source_page.last_edited_time DESC,
                  block.sort_order ASC,
                  block.id ASC",
    )
    .bind(page_id)
    .bind(page_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("list notes backlinks: {e}"))?;
    let mut backlinks = Vec::new();
    for row in rows {
        let reference_type = backlink_reference_type(&row, page_id)?;
        let Some(reference_type) = reference_type else {
            continue;
        };
        let source_page = get_page(pool, &row.page_id, false).await?;
        let snippet = backlink_snippet(&row);
        backlinks.push(NoteBacklinkDto::new(
            source_page,
            row,
            reference_type.to_string(),
            snippet,
        ));
    }
    for relation in data_source_relations::relation_backlinks(pool, page_id).await? {
        let source_page = get_page(pool, &relation.source_page_id, false).await?;
        backlinks.push(NoteBacklinkDto::database_relation(
            source_page,
            relation.source_page_id,
            relation.source_property_id,
            relation.source_property_name,
            relation.target_page_id,
            relation.created_time,
            relation.last_edited_time,
        ));
    }
    Ok(backlinks)
}

pub(in crate::notes) async fn search(
    pool: &SqlitePool,
    query: &str,
    page_size: Option<i64>,
) -> Result<Vec<NoteSearchResultDto>, String> {
    let query = query.trim();
    validate_search_query(query)?;
    let page_size = normalized_page_size(page_size)? as usize;
    let pattern = format!("%{}%", escape_like_query(query));
    let page_rows = sqlx::query_as::<_, NotePageRow>(
        "SELECT page.*
         FROM notes_pages AS page
         WHERE page.in_trash = 0
           AND page.archived = 0
           AND page.title LIKE ? ESCAPE '\\'
           AND (
               page.parent_type != 'data_source_id'
               OR EXISTS (
                   SELECT 1
                   FROM notes_data_sources AS data_source
                   JOIN notes_databases AS database ON database.id = data_source.database_id
                   WHERE data_source.id = page.parent_data_source_id
                     AND data_source.in_trash = 0
                     AND database.in_trash = 0
               )
           )
         ORDER BY page.last_edited_time DESC, page.title COLLATE NOCASE ASC, page.id ASC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind((page_size * 2) as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("search notes pages: {e}"))?;
    let relation_page_rows =
        data_source_relations::relation_search_rows(pool, &pattern, (page_size * 2) as i64).await?;

    let block_rows = sqlx::query_as::<_, NoteBlockRow>(
        "SELECT
            block.id,
            block.page_id,
            block.parent_type,
            block.parent_page_id,
            block.parent_block_id,
            block.has_children,
            block.in_trash,
            block.type AS block_type,
            block.payload,
            block.plain_text,
            block.sort_order,
            block.source_provider,
            block.source_object_id,
            block.source_last_edited_time,
            block.created_time,
            block.last_edited_time
         FROM notes_blocks AS block
         JOIN notes_pages AS page ON page.id = block.page_id
         WHERE block.in_trash = 0
           AND page.in_trash = 0
           AND page.archived = 0
           AND (
               page.parent_type != 'data_source_id'
               OR EXISTS (
                   SELECT 1
                   FROM notes_data_sources AS data_source
                   JOIN notes_databases AS database ON database.id = data_source.database_id
                   WHERE data_source.id = page.parent_data_source_id
                     AND data_source.in_trash = 0
                     AND database.in_trash = 0
               )
           )
           AND block.plain_text LIKE ? ESCAPE '\\'
         ORDER BY block.last_edited_time DESC, block.sort_order ASC, block.id ASC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind((page_size * 3) as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("search notes blocks: {e}"))?;

    let comment_rows = sqlx::query_as::<_, NoteCommentRow>(
        "SELECT
            comment.id,
            comment.thread_id,
            comment.rich_text,
            comment.plain_text,
            comment.created_by,
            comment.display_name,
            comment.attachments,
            comment.deleted_at,
            comment.sync_version,
            comment.created_time,
            comment.last_edited_time
         FROM notes_comments AS comment
         JOIN notes_comment_threads AS thread ON thread.id = comment.thread_id
         JOIN notes_pages AS page ON page.id = thread.page_id
         LEFT JOIN notes_blocks AS target_block ON target_block.id = thread.parent_block_id
         WHERE comment.deleted_at IS NULL
           AND page.in_trash = 0
           AND page.archived = 0
           AND (
               page.parent_type != 'data_source_id'
               OR EXISTS (
                   SELECT 1
                   FROM notes_data_sources AS data_source
                   JOIN notes_databases AS database ON database.id = data_source.database_id
                   WHERE data_source.id = page.parent_data_source_id
                     AND data_source.in_trash = 0
                     AND database.in_trash = 0
               )
           )
           AND (thread.parent_block_id IS NULL OR target_block.in_trash = 0)
           AND comment.plain_text LIKE ? ESCAPE '\\'
         ORDER BY comment.last_edited_time DESC, comment.id ASC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind((page_size * 2) as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("search notes comments: {e}"))?;

    let normalized_query = normalize_search_text(query);
    let mut candidates = Vec::new();
    for row in page_rows {
        let score = search_score("page", &row.title, &normalized_query);
        let sort_time = row.last_edited_time.clone();
        let id = format!("page:{}", row.id);
        let snippet = search_snippet(&row.title, query);
        candidates.push(SearchCandidate {
            score,
            sort_time: sort_time.clone(),
            id,
            result: NoteSearchResultDto::page(NotePageDto::new(row)?, snippet, sort_time),
        });
    }
    for row in relation_page_rows {
        let score = search_score("page", &row.title, &normalized_query) + 2;
        let sort_time = row.last_edited_time.clone();
        let id = format!("relation-page:{}", row.id);
        let snippet = search_snippet(&row.title, query);
        candidates.push(SearchCandidate {
            score,
            sort_time: sort_time.clone(),
            id,
            result: NoteSearchResultDto::page(NotePageDto::new(row)?, snippet, sort_time),
        });
    }
    for row in block_rows {
        let source_page = get_page(pool, &row.page_id, false).await?;
        let score = search_score("block", &row.plain_text, &normalized_query);
        let sort_time = row.last_edited_time.clone();
        let id = format!("block:{}", row.id);
        let snippet = search_snippet(&row.plain_text, query);
        candidates.push(SearchCandidate {
            score,
            sort_time,
            id,
            result: NoteSearchResultDto::block(source_page, row, snippet),
        });
    }
    for row in comment_rows {
        let target = comment_thread_search_target(pool, &row.thread_id).await?;
        let source_page = get_page(pool, &target.page_id, false).await?;
        let score = search_score("comment", &row.plain_text, &normalized_query);
        let sort_time = row.last_edited_time.clone();
        let id = format!("comment:{}", row.id);
        let snippet = search_snippet(&row.plain_text, query);
        candidates.push(SearchCandidate {
            score,
            sort_time,
            id,
            result: NoteSearchResultDto::comment(source_page, row, target.block_id, snippet),
        });
    }
    candidates.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| right.sort_time.cmp(&left.sort_time))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(candidates
        .into_iter()
        .take(page_size)
        .map(|candidate| candidate.result)
        .collect())
}

struct SearchCandidate {
    score: i64,
    sort_time: String,
    id: String,
    result: NoteSearchResultDto,
}

fn validate_search_query(query: &str) -> Result<(), String> {
    if query.is_empty() {
        return Err("search query must not be empty".to_string());
    }
    if query.chars().count() > 200 {
        return Err("search query must be 200 characters or fewer".to_string());
    }
    if query
        .chars()
        .any(|character| character.is_control() && character != '\n' && character != '\t')
    {
        return Err("search query must not contain control characters".to_string());
    }
    Ok(())
}

fn escape_like_query(query: &str) -> String {
    let mut escaped = String::new();
    for character in query.chars() {
        if matches!(character, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn normalize_search_text(text: &str) -> String {
    text.trim().to_lowercase()
}

fn search_score(result_type: &str, text: &str, normalized_query: &str) -> i64 {
    let normalized_text = normalize_search_text(text);
    let type_offset = match result_type {
        "page" => 0,
        "block" => 3,
        "comment" => 6,
        _ => 9,
    };
    if normalized_text == normalized_query {
        type_offset
    } else if normalized_text.starts_with(normalized_query) {
        type_offset + 1
    } else {
        type_offset + 2
    }
}

fn search_snippet(text: &str, query: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 160 {
        return trimmed.to_string();
    }
    let normalized_text = trimmed.to_lowercase();
    let normalized_query = query.trim().to_lowercase();
    let match_index = normalized_text.find(&normalized_query).unwrap_or(0);
    let prefix = trimmed
        .char_indices()
        .take_while(|(index, _)| *index < match_index)
        .count();
    let start = prefix.saturating_sub(50);
    let snippet = trimmed.chars().skip(start).take(160).collect::<String>();
    if start == 0 {
        snippet
    } else {
        format!("...{snippet}")
    }
}

struct CommentThreadSearchTarget {
    page_id: String,
    block_id: Option<String>,
}

async fn comment_thread_search_target(
    pool: &SqlitePool,
    thread_id: &str,
) -> Result<CommentThreadSearchTarget, String> {
    let row: (String, Option<String>) =
        sqlx::query_as("SELECT page_id, parent_block_id FROM notes_comment_threads WHERE id = ?")
            .bind(thread_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("load notes comment thread target: {e}"))?;
    Ok(CommentThreadSearchTarget {
        page_id: row.0,
        block_id: row.1,
    })
}

fn backlink_reference_type(
    row: &NoteBlockRow,
    page_id: &str,
) -> Result<Option<&'static str>, String> {
    if row.block_type == "child_page" && row.id == page_id {
        return Ok(Some("child_page"));
    }
    let payload: Value = serde_json::from_str(&row.payload)
        .map_err(|e| format!("parse notes backlink block payload: {e}"))?;
    if payload_contains_page_mention(&payload, page_id) {
        return Ok(Some("page_mention"));
    }
    if payload_contains_notes_page_link(&payload, page_id) {
        return Ok(Some("link"));
    }
    Ok(None)
}

fn payload_contains_page_mention(value: &Value, page_id: &str) -> bool {
    match value {
        Value::Object(object) => {
            let is_page_mention = object
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|value_type| value_type == "mention")
                && object
                    .get("mention")
                    .and_then(|mention| mention.get("type"))
                    .and_then(Value::as_str)
                    .is_some_and(|mention_type| mention_type == "page")
                && object
                    .get("mention")
                    .and_then(|mention| mention.get("page"))
                    .and_then(|page| page.get("id"))
                    .and_then(Value::as_str)
                    .is_some_and(|id| notes_id_matches(id, page_id));
            is_page_mention
                || object
                    .values()
                    .any(|nested| payload_contains_page_mention(nested, page_id))
        }
        Value::Array(items) => items
            .iter()
            .any(|nested| payload_contains_page_mention(nested, page_id)),
        _ => false,
    }
}

fn payload_contains_notes_page_link(value: &Value, page_id: &str) -> bool {
    match value {
        Value::Object(object) => {
            let has_link = ["href", "url"].into_iter().any(|field| {
                object
                    .get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|url| notes_url_references_page(url, page_id))
            }) || object
                .get("link")
                .and_then(|link| link.get("url"))
                .and_then(Value::as_str)
                .is_some_and(|url| notes_url_references_page(url, page_id));
            has_link
                || object
                    .values()
                    .any(|nested| payload_contains_notes_page_link(nested, page_id))
        }
        Value::Array(items) => items
            .iter()
            .any(|nested| payload_contains_notes_page_link(nested, page_id)),
        _ => false,
    }
}

fn notes_url_references_page(url: &str, page_id: &str) -> bool {
    url.contains("#notes?")
        && (url.contains(&format!("page={page_id}")) || url.contains(&page_id.replace('-', "")))
}

fn notes_id_matches(value: &str, page_id: &str) -> bool {
    value == page_id || value == page_id.replace('-', "")
}

fn backlink_snippet(row: &NoteBlockRow) -> String {
    let trimmed = row.plain_text.trim();
    if trimmed.is_empty() {
        return row.block_type.clone();
    }
    trimmed.chars().take(160).collect()
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
