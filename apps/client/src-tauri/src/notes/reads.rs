use super::models::{
    NoteBacklinkDto, NoteBlockDto, NoteBlockRow, NoteCommentRow, NoteLoadedPage, NotePageDto,
    NotePageRow, NotePaginatedBlockList, NoteSearchResultDto,
};
use super::validation::{require_uuid, validate_page_size};
use serde_json::Value;
use sqlx::SqlitePool;

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
        "SELECT *
         FROM notes_pages
         WHERE in_trash = 0
           AND archived = 0
           AND title LIKE ? ESCAPE '\\'
         ORDER BY last_edited_time DESC, title COLLATE NOCASE ASC, id ASC
         LIMIT ?",
    )
    .bind(&pattern)
    .bind((page_size * 2) as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("search notes pages: {e}"))?;

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
            comment.created_time,
            comment.last_edited_time
         FROM notes_comments AS comment
         JOIN notes_comment_threads AS thread ON thread.id = comment.thread_id
         JOIN notes_pages AS page ON page.id = thread.page_id
         LEFT JOIN notes_blocks AS target_block ON target_block.id = thread.parent_block_id
         WHERE comment.deleted_at IS NULL
           AND page.in_trash = 0
           AND page.archived = 0
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
