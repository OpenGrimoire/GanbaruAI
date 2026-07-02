use super::local_user;
use super::models::{
    NoteCommentAnchorCreate, NoteCommentAnchorDto, NoteCommentAnchorRow, NoteCommentCreate,
    NoteCommentDto, NoteCommentRow, NoteCommentThreadDto, NoteCommentThreadRow, NoteCommentUpdate,
    NoteParent,
};
use super::validation::{require_uuid, rich_text_items_plain_text, validate_comment_rich_text};
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

const COMMENT_ANCHOR_MAX_TEXT_LENGTH: usize = 2000;
const COMMENT_ANCHOR_MAX_CONTEXT_LENGTH: usize = 120;

struct CommentParentTarget {
    page_id: String,
    parent_type: &'static str,
    parent_page_id: Option<String>,
    parent_block_id: Option<String>,
}

pub(in crate::notes) async fn list_comments(
    pool: &SqlitePool,
    page_id: &str,
    include_resolved: bool,
) -> Result<Vec<NoteCommentThreadDto>, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    ensure_active_page(pool, page_id).await?;
    let thread_rows = if include_resolved {
        sqlx::query_as::<_, NoteCommentThreadRow>(
            "SELECT *
             FROM notes_comment_threads
             WHERE page_id = ?
               AND EXISTS (
                   SELECT 1
                   FROM notes_comments AS comment
                   WHERE comment.thread_id = notes_comment_threads.id
                     AND comment.deleted_at IS NULL
               )
               AND (
                   parent_type = 'page_id'
                   OR EXISTS (
                       SELECT 1
                       FROM notes_blocks AS block
                       WHERE block.id = notes_comment_threads.parent_block_id
                         AND block.in_trash = 0
                   )
               )
             ORDER BY created_time ASC, id ASC",
        )
        .bind(page_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NoteCommentThreadRow>(
            "SELECT *
             FROM notes_comment_threads
             WHERE page_id = ?
               AND status = 'open'
               AND EXISTS (
                   SELECT 1
                   FROM notes_comments AS comment
                   WHERE comment.thread_id = notes_comment_threads.id
                     AND comment.deleted_at IS NULL
               )
               AND (
                   parent_type = 'page_id'
                   OR EXISTS (
                       SELECT 1
                       FROM notes_blocks AS block
                       WHERE block.id = notes_comment_threads.parent_block_id
                         AND block.in_trash = 0
                   )
               )
             ORDER BY created_time ASC, id ASC",
        )
        .bind(page_id)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| format!("list notes comment threads: {e}"))?;
    thread_dtos(pool, thread_rows).await
}

pub(in crate::notes) async fn create_comment(
    pool: &SqlitePool,
    request: NoteCommentCreate,
) -> Result<NoteCommentThreadDto, String> {
    require_uuid(request.id.trim(), "id")?;
    validate_comment_rich_text(&request.rich_text)?;
    match (&request.parent, &request.discussion_id) {
        (Some(_), Some(_)) => {
            return Err("provide either parent or discussion_id, not both".to_string())
        }
        (None, None) => return Err("parent or discussion_id is required".to_string()),
        _ => {}
    }
    if request.anchor.is_some() && request.discussion_id.is_some() {
        return Err("inline comment anchors can only start new block comment threads".to_string());
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes comment create: {e}"))?;
    let thread_id = if let Some(parent) = request.parent {
        let parent = resolve_comment_parent(&mut tx, &parent).await?;
        let thread_id = new_comment_thread_id(&mut tx).await?;
        sqlx::query(
            "INSERT INTO notes_comment_threads (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id
             )
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&thread_id)
        .bind(&parent.page_id)
        .bind(parent.parent_type)
        .bind(parent.parent_page_id.as_deref())
        .bind(parent.parent_block_id.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("create notes comment thread: {e}"))?;
        if let Some(anchor) = request.anchor.as_ref() {
            insert_comment_anchor_row(&mut tx, &thread_id, &parent, anchor).await?;
        }
        thread_id
    } else {
        let thread_id = request
            .discussion_id
            .as_deref()
            .map(str::trim)
            .ok_or_else(|| "discussion_id is required".to_string())?;
        require_uuid(thread_id, "discussion_id")?;
        let row = load_thread_row(&mut tx, thread_id).await?;
        if row.status != "open" {
            return Err("resolved comment threads cannot receive replies".to_string());
        }
        thread_id.to_string()
    };
    insert_comment_row(&mut tx, request.id.trim(), &thread_id, &request.rich_text).await?;
    touch_thread(&mut tx, &thread_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes comment create: {e}"))?;
    load_thread(pool, &thread_id).await
}

pub(in crate::notes) async fn update_comment(
    pool: &SqlitePool,
    comment_id: &str,
    update: NoteCommentUpdate,
) -> Result<NoteCommentThreadDto, String> {
    let comment_id = comment_id.trim();
    require_uuid(comment_id, "comment_id")?;
    validate_comment_rich_text(&update.rich_text)?;
    let plain_text = rich_text_items_plain_text(&update.rich_text);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes comment update: {e}"))?;
    let row = load_comment_row(&mut tx, comment_id).await?;
    let thread = load_thread_row(&mut tx, &row.thread_id).await?;
    ensure_thread_target_active(&mut tx, &thread).await?;
    sqlx::query(
        "UPDATE notes_comments
         SET rich_text = ?,
             plain_text = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(Value::Array(update.rich_text).to_string())
    .bind(plain_text)
    .bind(comment_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update notes comment: {e}"))?;
    touch_thread(&mut tx, &row.thread_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes comment update: {e}"))?;
    load_thread(pool, &row.thread_id).await
}

pub(in crate::notes) async fn delete_comment(
    pool: &SqlitePool,
    comment_id: &str,
) -> Result<NoteCommentThreadDto, String> {
    let comment_id = comment_id.trim();
    require_uuid(comment_id, "comment_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes comment delete: {e}"))?;
    let row = load_comment_row(&mut tx, comment_id).await?;
    sqlx::query(
        "UPDATE notes_comments
         SET deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(comment_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("delete notes comment: {e}"))?;
    touch_thread(&mut tx, &row.thread_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes comment delete: {e}"))?;
    load_thread(pool, &row.thread_id).await
}

pub(in crate::notes) async fn resolve_comment_thread(
    pool: &SqlitePool,
    discussion_id: &str,
    resolved: bool,
) -> Result<NoteCommentThreadDto, String> {
    let discussion_id = discussion_id.trim();
    require_uuid(discussion_id, "discussion_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes comment thread resolve: {e}"))?;
    let thread = load_thread_row(&mut tx, discussion_id).await?;
    ensure_thread_target_active(&mut tx, &thread).await?;
    if resolved {
        let local_user = local_user::current_local_user_tx(&mut tx).await?;
        sqlx::query(
            "UPDATE notes_comment_threads
             SET status = 'resolved',
                 resolved_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                 resolved_by = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(&local_user.id)
        .bind(discussion_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("resolve notes comment thread: {e}"))?;
    } else {
        sqlx::query(
            "UPDATE notes_comment_threads
             SET status = 'open',
                 resolved_at = NULL,
                 resolved_by = NULL,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(discussion_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("reopen notes comment thread: {e}"))?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit notes comment thread resolve: {e}"))?;
    load_thread(pool, discussion_id).await
}

async fn load_thread(pool: &SqlitePool, thread_id: &str) -> Result<NoteCommentThreadDto, String> {
    let thread = sqlx::query_as::<_, NoteCommentThreadRow>(
        "SELECT * FROM notes_comment_threads WHERE id = ?",
    )
    .bind(thread_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load notes comment thread: {e}"))?
    .ok_or_else(|| "notes comment thread not found".to_string())?;
    let comments = load_comment_dtos(pool, &thread).await?;
    let anchor = load_comment_anchor_dto(pool, thread_id).await?;
    NoteCommentThreadDto::new(thread, comments, anchor)
}

async fn thread_dtos(
    pool: &SqlitePool,
    threads: Vec<NoteCommentThreadRow>,
) -> Result<Vec<NoteCommentThreadDto>, String> {
    let mut dtos = Vec::with_capacity(threads.len());
    for thread in threads {
        let comments = load_comment_dtos(pool, &thread).await?;
        let anchor = load_comment_anchor_dto(pool, &thread.id).await?;
        dtos.push(NoteCommentThreadDto::new(thread, comments, anchor)?);
    }
    Ok(dtos)
}

async fn load_comment_anchor_dto(
    pool: &SqlitePool,
    thread_id: &str,
) -> Result<Option<NoteCommentAnchorDto>, String> {
    let row = sqlx::query_as::<_, NoteCommentAnchorRow>(
        "SELECT *
         FROM notes_comment_thread_anchors
         WHERE thread_id = ?",
    )
    .bind(thread_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load notes inline comment anchor: {e}"))?;
    Ok(row.map(NoteCommentAnchorDto::new))
}

async fn load_comment_dtos(
    pool: &SqlitePool,
    thread: &NoteCommentThreadRow,
) -> Result<Vec<NoteCommentDto>, String> {
    let rows = sqlx::query_as::<_, NoteCommentRow>(
        "SELECT *
         FROM notes_comments
         WHERE thread_id = ? AND deleted_at IS NULL
         ORDER BY created_time ASC, id ASC",
    )
    .bind(&thread.id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("list notes comments: {e}"))?;
    rows.into_iter()
        .map(|row| NoteCommentDto::new(thread, row))
        .collect()
}

async fn resolve_comment_parent(
    tx: &mut Transaction<'_, Sqlite>,
    parent: &NoteParent,
) -> Result<CommentParentTarget, String> {
    match parent {
        NoteParent::Workspace { .. } => {
            Err("comments can only be parented by pages or blocks".to_string())
        }
        NoteParent::DataSourceId { .. } => {
            Err("comments can only be parented by pages or blocks".to_string())
        }
        NoteParent::PageId { page_id } => {
            require_uuid(page_id, "parent.page_id")?;
            ensure_active_page_tx(tx, page_id).await?;
            Ok(CommentParentTarget {
                page_id: page_id.clone(),
                parent_type: "page_id",
                parent_page_id: Some(page_id.clone()),
                parent_block_id: None,
            })
        }
        NoteParent::BlockId { block_id } => {
            require_uuid(block_id, "parent.block_id")?;
            let (page_id,): (String,) = sqlx::query_as(
                "SELECT block.page_id
                 FROM notes_blocks AS block
                 JOIN notes_pages AS page ON page.id = block.page_id
                 WHERE block.id = ?
                   AND block.in_trash = 0
                   AND page.in_trash = 0
                   AND page.archived = 0",
            )
            .bind(block_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load notes comment block parent: {e}"))?
            .ok_or_else(|| "notes block not found".to_string())?;
            Ok(CommentParentTarget {
                page_id,
                parent_type: "block_id",
                parent_page_id: None,
                parent_block_id: Some(block_id.clone()),
            })
        }
    }
}

async fn insert_comment_row(
    tx: &mut Transaction<'_, Sqlite>,
    comment_id: &str,
    thread_id: &str,
    rich_text: &[Value],
) -> Result<(), String> {
    let plain_text = rich_text_items_plain_text(rich_text);
    let local_user = local_user::current_local_user_tx(tx).await?;
    sqlx::query(
        "INSERT INTO notes_comments (
            id,
            thread_id,
            rich_text,
            plain_text,
            created_by,
            display_name
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(comment_id)
    .bind(thread_id)
    .bind(Value::Array(rich_text.to_vec()).to_string())
    .bind(plain_text)
    .bind(&local_user.id)
    .bind(local_user::comment_display_name_json(
        &local_user.display_name,
    ))
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("create notes comment: {e}"))?;
    Ok(())
}

async fn insert_comment_anchor_row(
    tx: &mut Transaction<'_, Sqlite>,
    thread_id: &str,
    parent: &CommentParentTarget,
    anchor: &NoteCommentAnchorCreate,
) -> Result<(), String> {
    let block_id = parent
        .parent_block_id
        .as_deref()
        .ok_or_else(|| "inline comment anchors require a block parent".to_string())?;
    validate_comment_anchor(anchor)?;
    let plain_text: String = sqlx::query_scalar(
        "SELECT plain_text
         FROM notes_blocks
         WHERE id = ? AND page_id = ? AND in_trash = 0",
    )
    .bind(block_id)
    .bind(&parent.page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes inline comment anchor block: {e}"))?
    .ok_or_else(|| "notes block not found".to_string())?;
    if !plain_text.contains(anchor.text.as_str()) {
        return Err("inline comment anchor text must exist in the block".to_string());
    }
    sqlx::query(
        "INSERT INTO notes_comment_thread_anchors (
            thread_id,
            page_id,
            block_id,
            start_offset,
            end_offset,
            anchor_text,
            prefix_text,
            suffix_text
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(thread_id)
    .bind(&parent.page_id)
    .bind(block_id)
    .bind(anchor.start)
    .bind(anchor.end)
    .bind(anchor.text.as_str())
    .bind(anchor.prefix.as_str())
    .bind(anchor.suffix.as_str())
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("create notes inline comment anchor: {e}"))?;
    Ok(())
}

fn validate_comment_anchor(anchor: &NoteCommentAnchorCreate) -> Result<(), String> {
    if anchor.start < 0 || anchor.end <= anchor.start {
        return Err("inline comment anchor range is invalid".to_string());
    }
    if anchor.text.trim().is_empty() {
        return Err("inline comment anchor text is required".to_string());
    }
    if anchor.text.chars().count() > COMMENT_ANCHOR_MAX_TEXT_LENGTH {
        return Err("inline comment anchor text is too long".to_string());
    }
    if anchor.prefix.chars().count() > COMMENT_ANCHOR_MAX_CONTEXT_LENGTH
        || anchor.suffix.chars().count() > COMMENT_ANCHOR_MAX_CONTEXT_LENGTH
    {
        return Err("inline comment anchor context is too long".to_string());
    }
    Ok(())
}

async fn load_thread_row(
    tx: &mut Transaction<'_, Sqlite>,
    thread_id: &str,
) -> Result<NoteCommentThreadRow, String> {
    sqlx::query_as::<_, NoteCommentThreadRow>("SELECT * FROM notes_comment_threads WHERE id = ?")
        .bind(thread_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load notes comment thread: {e}"))?
        .ok_or_else(|| "notes comment thread not found".to_string())
}

async fn load_comment_row(
    tx: &mut Transaction<'_, Sqlite>,
    comment_id: &str,
) -> Result<NoteCommentRow, String> {
    sqlx::query_as::<_, NoteCommentRow>(
        "SELECT * FROM notes_comments WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(comment_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes comment: {e}"))?
    .ok_or_else(|| "notes comment not found".to_string())
}

async fn touch_thread(tx: &mut Transaction<'_, Sqlite>, thread_id: &str) -> Result<(), String> {
    sqlx::query(
        "UPDATE notes_comment_threads
         SET last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(thread_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("touch notes comment thread: {e}"))?;
    Ok(())
}

async fn ensure_active_page(pool: &SqlitePool, page_id: &str) -> Result<(), String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(page_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("check notes page for comments: {e}"))?;
    if exists.is_some() {
        Ok(())
    } else {
        Err("notes page not found".to_string())
    }
}

async fn ensure_active_page_tx(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
) -> Result<(), String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check notes page for comments: {e}"))?;
    if exists.is_some() {
        Ok(())
    } else {
        Err("notes page not found".to_string())
    }
}

async fn ensure_thread_target_active(
    tx: &mut Transaction<'_, Sqlite>,
    thread: &NoteCommentThreadRow,
) -> Result<(), String> {
    match thread.parent_type.as_str() {
        "page_id" => {
            let page_id = thread
                .parent_page_id
                .as_deref()
                .ok_or_else(|| "comment thread parent page is missing".to_string())?;
            ensure_active_page_tx(tx, page_id).await
        }
        "block_id" => {
            let block_id = thread
                .parent_block_id
                .as_deref()
                .ok_or_else(|| "comment thread parent block is missing".to_string())?;
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1
                 FROM notes_blocks AS block
                 JOIN notes_pages AS page ON page.id = block.page_id
                 WHERE block.id = ?
                   AND block.in_trash = 0
                   AND page.in_trash = 0
                   AND page.archived = 0",
            )
            .bind(block_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("check notes block for comments: {e}"))?;
            if exists.is_some() {
                Ok(())
            } else {
                Err("notes block not found".to_string())
            }
        }
        _ => Err("invalid comment parent".to_string()),
    }
}

async fn new_comment_thread_id(tx: &mut Transaction<'_, Sqlite>) -> Result<String, String> {
    for _ in 0..32 {
        let id: String = sqlx::query_scalar(
            "SELECT lower(hex(randomblob(4))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(2))) || '-' ||
                    lower(hex(randomblob(6)))",
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| format!("generate notes comment thread id: {e}"))?;
        require_uuid(&id, "generated_thread_id")?;
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             WHERE EXISTS (SELECT 1 FROM notes_comment_threads WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_comments WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_pages WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_blocks WHERE id = ?)",
        )
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("check generated notes comment thread id: {e}"))?;
        if exists.is_none() {
            return Ok(id);
        }
    }
    Err("could not generate a unique notes comment thread id".to_string())
}
