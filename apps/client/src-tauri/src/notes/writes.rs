use super::models::{
    page_parent_columns, parent_columns, NoteAppendBlockChildren, NoteBlockDto, NoteBlockRow,
    NoteBlockUpdate, NoteBlockWrite, NoteChildPageFromBlockCreate, NoteDuplicateBlock,
    NoteDuplicateBlocks, NoteDuplicatePage, NoteDuplicatedBlockId, NoteLoadedPage, NoteMoveBlock,
    NoteMoveBlocks, NoteMovePage, NotePageCreate, NotePageDto, NotePageRow, NotePageUpdate,
    NotePaginatedBlockList, NoteParent, NoteTrashBlocks, OptionalJsonValue,
};
use super::validation::{
    block_payload_supports_children, plain_text_from_payload, require_uuid, validate_block_update,
    validate_block_write, validate_children_count, validate_duplicate_block_count,
    validate_page_create, validate_page_update, validate_parent, validate_sort_order,
};
use super::{data_source_rollups, history, mention_notifications, reads};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet, VecDeque};

const DEFAULT_BLOCK_SORT_STEP: f64 = 1000.0;

pub(in crate::notes) struct ParentTarget {
    pub(in crate::notes) parent_type: &'static str,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) parent_block_type: Option<String>,
    pub(in crate::notes) page_id: String,
}

struct DuplicatePagePlan {
    source_page: NotePageRow,
    duplicate_id: String,
    duplicate_title: String,
    parent: NoteParent,
    blocks: Vec<NoteBlockRow>,
    is_root: bool,
}

type PageParentColumns = (String, Option<String>, Option<String>, Option<String>);

pub(in crate::notes) async fn create_page(
    pool: &SqlitePool,
    page: NotePageCreate,
) -> Result<NoteLoadedPage, String> {
    validate_page_create(&page)?;
    if matches!(&page.parent, NoteParent::DataSourceId { .. }) {
        return Err("data source pages must be created with the row page command".to_string());
    }
    let title = page.title.trim().to_string();
    let properties = page_title_properties(&title);
    let (parent_type, parent_page_id, parent_block_id) = parent_columns(&page.parent);
    let first_payload = default_text_payload("");
    let first_plain_text = plain_text_from_payload("paragraph", &first_payload);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes page create: {e}"))?;
    validate_page_parent_exists(&mut tx, &page.parent).await?;
    let child_page_parent = match &page.parent {
        NoteParent::Workspace { .. } => None,
        _ => Some(resolve_block_parent(&mut tx, &page.parent).await?),
    };
    let child_page_sort_order = if let Some(parent) = &child_page_parent {
        Some(next_sort_orders(&mut tx, parent, page.after_block_id.as_deref(), 1).await?[0])
    } else {
        None
    };
    if let Some(parent) = &child_page_parent {
        history::record_page_snapshot_tx(&mut tx, &parent.page_id, "create_child_page").await?;
    }
    sqlx::query(
        "INSERT INTO notes_pages (
            id,
            parent_type,
            parent_page_id,
            parent_block_id,
            title,
            properties
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(page.id.trim())
    .bind(parent_type)
    .bind(parent_page_id)
    .bind(parent_block_id)
    .bind(&title)
    .bind(properties.to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create notes page: {e}"))?;
    sqlx::query(
        "INSERT INTO notes_blocks (
            id,
            page_id,
            parent_type,
            parent_page_id,
            type,
            payload,
            plain_text,
            sort_order
         )
         VALUES (?, ?, 'page_id', ?, 'paragraph', ?, ?, 1000)",
    )
    .bind(page.first_block_id.trim())
    .bind(page.id.trim())
    .bind(page.id.trim())
    .bind(first_payload.to_string())
    .bind(first_plain_text)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create initial notes block: {e}"))?;
    if let Some(parent) = child_page_parent {
        let child_payload = child_page_payload(&title);
        let child_plain_text = plain_text_from_payload("child_page", &child_payload);
        let sort_order = child_page_sort_order
            .ok_or_else(|| "child page sort order was not prepared".to_string())?;
        validate_sort_order(sort_order)?;
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, ?, ?, ?, 'child_page', ?, ?, ?)",
        )
        .bind(page.id.trim())
        .bind(&parent.page_id)
        .bind(parent.parent_type)
        .bind(&parent.parent_page_id)
        .bind(&parent.parent_block_id)
        .bind(child_payload.to_string())
        .bind(child_plain_text)
        .bind(sort_order)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("create notes child page block: {e}"))?;
        refresh_parent_has_children(&mut tx, &parent).await?;
        touch_page(&mut tx, &parent.page_id).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit notes page create: {e}"))?;
    reads::load_page(pool, page.id.trim()).await
}

pub(in crate::notes) async fn create_child_page_from_block(
    pool: &SqlitePool,
    block_id: &str,
    request: NoteChildPageFromBlockCreate,
) -> Result<NoteLoadedPage, String> {
    let block_id = block_id.trim();
    require_uuid(block_id, "block_id")?;
    require_uuid(&request.first_block_id, "first_block_id")?;
    if block_id == request.first_block_id.trim() {
        return Err("first_block_id must not match block_id".to_string());
    }
    let current = reads::get_block_row(pool, block_id, false).await?;
    if current.block_type == "child_page" {
        return reads::load_page(pool, block_id).await;
    }
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| current.plain_text.trim())
        .to_string();
    let page_parent_type = if current.parent_type == "page_id" {
        "page_id"
    } else {
        "block_id"
    };
    let child_payload = child_page_payload(&title);
    let child_plain_text = plain_text_from_payload("child_page", &child_payload);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes child page create: {e}"))?;
    let existing_page: Option<i64> = sqlx::query_scalar("SELECT 1 FROM notes_pages WHERE id = ?")
        .bind(block_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("check notes child page id: {e}"))?;
    if existing_page.is_some() {
        return Err("notes page already exists for block".to_string());
    }
    history::record_page_snapshot_tx(&mut tx, &current.page_id, "create_child_page_from_block")
        .await?;
    sqlx::query(
        "INSERT INTO notes_pages (
            id,
            parent_type,
            parent_page_id,
            parent_block_id,
            title,
            properties
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(block_id)
    .bind(page_parent_type)
    .bind(&current.parent_page_id)
    .bind(&current.parent_block_id)
    .bind(&title)
    .bind(page_title_properties(&title).to_string())
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("create notes child page: {e}"))?;

    let child_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notes_blocks WHERE parent_block_id = ? AND in_trash = 0",
    )
    .bind(block_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("count notes child page block children: {e}"))?;
    if child_count == 0 {
        let first_payload = default_text_payload("");
        let first_plain_text = plain_text_from_payload("paragraph", &first_payload);
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, 'page_id', ?, 'paragraph', ?, ?, 1000)",
        )
        .bind(request.first_block_id.trim())
        .bind(block_id)
        .bind(block_id)
        .bind(first_payload.to_string())
        .bind(first_plain_text)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("create initial notes child page block: {e}"))?;
    } else {
        sqlx::query(
            "WITH RECURSIVE subtree(id) AS (
                SELECT id FROM notes_blocks WHERE parent_block_id = ? AND in_trash = 0
                UNION ALL
                SELECT child.id
                FROM notes_blocks AS child
                JOIN subtree ON child.parent_block_id = subtree.id
                WHERE child.in_trash = 0
             )
             UPDATE notes_blocks
             SET page_id = ?
             WHERE id IN (SELECT id FROM subtree)",
        )
        .bind(block_id)
        .bind(block_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("move notes child page descendant blocks: {e}"))?;
        sqlx::query(
            "UPDATE notes_blocks
             SET parent_type = 'page_id',
                 parent_page_id = ?,
                 parent_block_id = NULL
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .bind(block_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("move notes child page root blocks: {e}"))?;
    }

    sqlx::query(
        "UPDATE notes_blocks
         SET has_children = 0,
             type = 'child_page',
             payload = ?,
             plain_text = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND in_trash = 0",
    )
    .bind(child_payload.to_string())
    .bind(child_plain_text)
    .bind(block_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("convert notes block to child page: {e}"))?;
    touch_page(&mut tx, &current.page_id).await?;
    touch_page(&mut tx, block_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit notes child page create: {e}"))?;
    reads::load_page(pool, block_id).await
}

pub(in crate::notes) async fn duplicate_page(
    pool: &SqlitePool,
    page_id: &str,
    request: NoteDuplicatePage,
) -> Result<NoteLoadedPage, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin duplicate notes page: {e}"))?;
    let root_page = load_page_row(&mut tx, page_id).await?;
    if matches!(root_page.parent_type.as_str(), "page_id" | "block_id") {
        let source_block = load_child_page_block_row(&mut tx, page_id).await?;
        history::record_page_snapshot_tx(&mut tx, &source_block.page_id, "duplicate_page").await?;
    }
    let root_duplicate_title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or(root_page.title.trim())
        .to_string();
    let mut reserved_ids = HashSet::new();
    let mut block_ids = HashMap::new();
    let root_duplicate_id = new_note_id(&mut tx, &mut reserved_ids).await?;
    let mut plans = Vec::new();
    let mut queue = VecDeque::from([(
        page_id.to_string(),
        root_duplicate_id.clone(),
        root_duplicate_title,
        None::<NoteParent>,
        true,
    )]);
    while let Some((source_page_id, duplicate_id, title_override, parent_override, is_root)) =
        queue.pop_front()
    {
        let source_page = load_page_row(&mut tx, &source_page_id).await?;
        let blocks = load_page_block_subtree_rows(&mut tx, &source_page_id).await?;
        for row in &blocks {
            if row.block_type == "child_page" {
                let child_page = load_page_row(&mut tx, &row.id).await?;
                let child_duplicate_id = new_note_id(&mut tx, &mut reserved_ids).await?;
                block_ids.insert(row.id.clone(), child_duplicate_id.clone());
                let duplicate_parent =
                    duplicate_page_parent_for_child_block(row, &duplicate_id, &block_ids)?;
                queue.push_back((
                    child_page.id,
                    child_duplicate_id,
                    child_page.title,
                    Some(duplicate_parent),
                    false,
                ));
            } else if !block_ids.contains_key(&row.id) {
                let duplicate_block_id = new_note_id(&mut tx, &mut reserved_ids).await?;
                block_ids.insert(row.id.clone(), duplicate_block_id);
            }
        }
        let parent = match parent_override {
            Some(parent) => parent,
            None => page_parent_from_columns(
                &source_page.parent_type,
                source_page.parent_page_id.clone(),
                source_page.parent_block_id.clone(),
                source_page.parent_data_source_id.clone(),
            )?,
        };
        plans.push(DuplicatePagePlan {
            source_page,
            duplicate_id,
            duplicate_title: title_override,
            parent,
            blocks,
            is_root,
        });
    }
    let duplicate_titles = plans
        .iter()
        .map(|plan| (plan.source_page.id.clone(), plan.duplicate_title.clone()))
        .collect::<HashMap<_, _>>();
    let mut inserted_block_ids = HashSet::new();
    for plan in &plans {
        insert_duplicated_page(&mut tx, plan).await?;
        if plan.is_root {
            insert_root_duplicate_child_page_block(&mut tx, plan).await?;
        }
        insert_duplicated_page_blocks(
            &mut tx,
            plan,
            &block_ids,
            &duplicate_titles,
            &mut inserted_block_ids,
        )
        .await?;
    }
    refresh_duplicated_has_children(&mut tx, &inserted_block_ids).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit duplicate notes page: {e}"))?;
    reads::load_page(pool, &root_duplicate_id).await
}

pub(in crate::notes) async fn move_page(
    pool: &SqlitePool,
    page_id: &str,
    request: NoteMovePage,
) -> Result<NoteLoadedPage, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    validate_parent(&request.parent)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin move notes page: {e}"))?;
    let source_page = load_page_row(&mut tx, page_id).await?;
    let child_block = load_child_page_block_row_any(&mut tx, page_id).await?;
    let old_parent = child_block
        .as_ref()
        .filter(|block| block.in_trash == 0)
        .map(parent_target_from_block_row);
    let new_parent = resolve_page_move_parent(&mut tx, page_id, &request.parent).await?;
    let (parent_type, parent_page_id, parent_block_id, parent_data_source_id) =
        page_parent_columns(&request.parent);
    let mut history_page_ids = HashSet::from([page_id.to_string()]);
    if let Some(parent) = &old_parent {
        history_page_ids.insert(parent.page_id.clone());
    }
    if let Some(parent) = &new_parent {
        history_page_ids.insert(parent.page_id.clone());
    }
    for history_page_id in history_page_ids {
        history::record_page_snapshot_tx(&mut tx, &history_page_id, "move_page").await?;
    }
    sqlx::query(
        "UPDATE notes_pages
         SET parent_type = ?,
             parent_page_id = ?,
             parent_block_id = ?,
             parent_data_source_id = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(parent_type)
    .bind(parent_page_id)
    .bind(parent_block_id)
    .bind(parent_data_source_id)
    .bind(page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("move notes page: {e}"))?;
    match &new_parent {
        Some(parent) => {
            upsert_moved_child_page_block(
                &mut tx,
                page_id,
                &source_page.title,
                parent,
                child_block,
            )
            .await?;
            refresh_parent_has_children(&mut tx, parent).await?;
            touch_page(&mut tx, &parent.page_id).await?;
        }
        None => {
            if child_block
                .as_ref()
                .is_some_and(|block| block.in_trash == 0)
            {
                sqlx::query(
                    "UPDATE notes_blocks
                     SET in_trash = 1,
                         last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ? AND type = 'child_page'",
                )
                .bind(page_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("hide moved notes child page block: {e}"))?;
            }
        }
    }
    if let Some(parent) = &old_parent {
        refresh_parent_has_children(&mut tx, parent).await?;
        touch_page(&mut tx, &parent.page_id).await?;
    }
    touch_page(&mut tx, page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit move notes page: {e}"))?;
    reads::load_page(pool, page_id).await
}

pub(in crate::notes) async fn update_page(
    pool: &SqlitePool,
    page_id: &str,
    update: NotePageUpdate,
) -> Result<NotePageDto, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    validate_page_update(&update)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin notes page update: {e}"))?;
    history::record_page_snapshot_tx(&mut tx, page_id, "update_page").await?;
    if let Some(parent) = &update.parent {
        validate_page_parent_exists(&mut tx, parent).await?;
        let (parent_type, parent_page_id, parent_block_id, parent_data_source_id) =
            page_parent_columns(parent);
        sqlx::query(
            "UPDATE notes_pages
             SET parent_type = ?,
                 parent_page_id = ?,
                 parent_block_id = ?,
                 parent_data_source_id = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND in_trash = 0 AND archived = 0",
        )
        .bind(parent_type)
        .bind(parent_page_id)
        .bind(parent_block_id)
        .bind(parent_data_source_id)
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update notes page parent: {e}"))?;
    }
    if let Some(title) = update.title {
        let title = title.trim().to_string();
        let current_page = load_page_row(&mut tx, page_id).await?;
        let properties = match update.properties.clone() {
            Some(properties) => properties.to_string(),
            None => page_row_properties_for_title(&current_page, &title)?,
        };
        sqlx::query(
            "UPDATE notes_pages
             SET title = ?,
                 properties = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND in_trash = 0 AND archived = 0",
        )
        .bind(&title)
        .bind(properties)
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update notes page title: {e}"))?;
        let child_payload = child_page_payload(&title);
        sqlx::query(
            "UPDATE notes_blocks
             SET payload = ?,
                 plain_text = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND type = 'child_page'",
        )
        .bind(child_payload.to_string())
        .bind(plain_text_from_payload("child_page", &child_payload))
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update notes child page block title: {e}"))?;
    } else if let Some(properties) = update.properties {
        sqlx::query(
            "UPDATE notes_pages
             SET properties = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND in_trash = 0 AND archived = 0",
        )
        .bind(properties.to_string())
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update notes page properties: {e}"))?;
    }
    if update.icon.is_set() || update.cover.is_set() {
        let (icon_is_set, icon) = json_field_update(&update.icon);
        let (cover_is_set, cover) = json_field_update(&update.cover);
        sqlx::query(
            "UPDATE notes_pages
             SET icon = CASE WHEN ? THEN ? ELSE icon END,
                 cover = CASE WHEN ? THEN ? ELSE cover END,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND in_trash = 0 AND archived = 0",
        )
        .bind(icon_is_set)
        .bind(icon)
        .bind(cover_is_set)
        .bind(cover)
        .bind(page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update notes page media: {e}"))?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit notes page update: {e}"))?;
    reads::get_page(pool, page_id, false).await
}

fn json_field_update(value: &OptionalJsonValue) -> (i64, Option<String>) {
    if value.is_set() {
        (1, value.storage_value())
    } else {
        (0, None)
    }
}

async fn load_page_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<NotePageRow, String> {
    sqlx::query_as::<_, NotePageRow>(
        "SELECT * FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes page row: {e}"))?
    .ok_or_else(|| "notes page not found".to_string())
}

fn page_parent_from_columns(
    parent_type: &str,
    parent_page_id: Option<String>,
    parent_block_id: Option<String>,
    parent_data_source_id: Option<String>,
) -> Result<NoteParent, String> {
    match parent_type {
        "workspace" => Ok(NoteParent::Workspace { workspace: true }),
        "page_id" => parent_page_id
            .map(|page_id| NoteParent::PageId { page_id })
            .ok_or_else(|| "page parent row is missing parent_page_id".to_string()),
        "block_id" => parent_block_id
            .map(|block_id| NoteParent::BlockId { block_id })
            .ok_or_else(|| "page parent row is missing parent_block_id".to_string()),
        "data_source_id" => parent_data_source_id
            .map(|data_source_id| NoteParent::DataSourceId { data_source_id })
            .ok_or_else(|| "page parent row is missing parent_data_source_id".to_string()),
        _ => Err("page parent row has unsupported parent_type".to_string()),
    }
}

fn duplicate_page_parent_for_child_block(
    block: &NoteBlockRow,
    duplicate_page_id: &str,
    block_ids: &HashMap<String, String>,
) -> Result<NoteParent, String> {
    match block.parent_type.as_str() {
        "page_id" => Ok(NoteParent::PageId {
            page_id: duplicate_page_id.to_string(),
        }),
        "block_id" => {
            let source_parent_id = block
                .parent_block_id
                .as_ref()
                .ok_or_else(|| "child page block is missing its parent".to_string())?;
            let duplicate_parent_id = block_ids
                .get(source_parent_id)
                .cloned()
                .ok_or_else(|| "child page block parent was not duplicated".to_string())?;
            Ok(NoteParent::BlockId {
                block_id: duplicate_parent_id,
            })
        }
        _ => Err("child page block has unsupported parent_type".to_string()),
    }
}

async fn insert_duplicated_page(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    plan: &DuplicatePagePlan,
) -> Result<(), String> {
    validate_parent(&plan.parent)?;
    let (parent_type, parent_page_id, parent_block_id, parent_data_source_id) =
        page_parent_columns(&plan.parent);
    let properties = if plan.duplicate_title == plan.source_page.title {
        plan.source_page.properties.clone()
    } else {
        page_row_properties_for_title(&plan.source_page, &plan.duplicate_title)?
    };
    sqlx::query(
        "INSERT INTO notes_pages (
            id,
            parent_type,
            parent_page_id,
            parent_block_id,
            parent_data_source_id,
            title,
            properties,
            icon,
            cover
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&plan.duplicate_id)
    .bind(parent_type)
    .bind(parent_page_id)
    .bind(parent_block_id)
    .bind(parent_data_source_id)
    .bind(&plan.duplicate_title)
    .bind(properties)
    .bind(&plan.source_page.icon)
    .bind(&plan.source_page.cover)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("duplicate notes page: {e}"))?;
    Ok(())
}

async fn insert_root_duplicate_child_page_block(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    plan: &DuplicatePagePlan,
) -> Result<(), String> {
    if plan.source_page.parent_type == "workspace"
        || plan.source_page.parent_type == "data_source_id"
    {
        return Ok(());
    }
    let source_block = load_child_page_block_row(tx, &plan.source_page.id).await?;
    let parent = parent_target_from_block_row(&source_block);
    let sort_order = next_sort_orders(tx, &parent, Some(source_block.id.as_str()), 1).await?[0];
    validate_sort_order(sort_order)?;
    let payload = child_page_payload(&plan.duplicate_title);
    sqlx::query(
        "INSERT INTO notes_blocks (
            id,
            page_id,
            parent_type,
            parent_page_id,
            parent_block_id,
            type,
            payload,
            plain_text,
            sort_order
         )
         VALUES (?, ?, ?, ?, ?, 'child_page', ?, ?, ?)",
    )
    .bind(&plan.duplicate_id)
    .bind(&source_block.page_id)
    .bind(parent.parent_type)
    .bind(&parent.parent_page_id)
    .bind(&parent.parent_block_id)
    .bind(payload.to_string())
    .bind(plain_text_from_payload("child_page", &payload))
    .bind(sort_order)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("duplicate notes child page block: {e}"))?;
    refresh_parent_has_children(tx, &parent).await?;
    touch_page(tx, &source_block.page_id).await?;
    Ok(())
}

async fn insert_duplicated_page_blocks(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    plan: &DuplicatePagePlan,
    block_ids: &HashMap<String, String>,
    duplicate_titles: &HashMap<String, String>,
    inserted_block_ids: &mut HashSet<String>,
) -> Result<(), String> {
    for row in &plan.blocks {
        let duplicate_id = block_ids
            .get(&row.id)
            .ok_or_else(|| "duplicated page block id is missing".to_string())?;
        let (parent_type, parent_page_id, parent_block_id) = if row.parent_type == "page_id" {
            ("page_id", Some(plan.duplicate_id.clone()), None)
        } else {
            let source_parent_id = row
                .parent_block_id
                .as_ref()
                .ok_or_else(|| "duplicated block is missing its parent".to_string())?;
            let duplicate_parent_id = block_ids
                .get(source_parent_id)
                .cloned()
                .ok_or_else(|| "duplicated block parent id is missing".to_string())?;
            ("block_id", None, Some(duplicate_parent_id))
        };
        let (payload, plain_text) = if row.block_type == "child_page" {
            let title = duplicate_titles
                .get(&row.id)
                .ok_or_else(|| "duplicated child page title is missing".to_string())?;
            let payload = child_page_payload(title);
            (
                payload.to_string(),
                plain_text_from_payload("child_page", &payload),
            )
        } else {
            (row.payload.clone(), row.plain_text.clone())
        };
        validate_sort_order(row.sort_order)?;
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(duplicate_id)
        .bind(&plan.duplicate_id)
        .bind(parent_type)
        .bind(parent_page_id)
        .bind(parent_block_id)
        .bind(row.has_children)
        .bind(&row.block_type)
        .bind(payload)
        .bind(plain_text)
        .bind(row.sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("duplicate notes page block: {e}"))?;
        inserted_block_ids.insert(duplicate_id.clone());
    }
    Ok(())
}

pub(in crate::notes) fn parent_target_from_block_row(row: &NoteBlockRow) -> ParentTarget {
    ParentTarget {
        parent_type: if row.parent_type == "page_id" {
            "page_id"
        } else {
            "block_id"
        },
        parent_page_id: row.parent_page_id.clone(),
        parent_block_id: row.parent_block_id.clone(),
        parent_block_type: None,
        page_id: row.page_id.clone(),
    }
}

pub(in crate::notes) async fn trash_page(
    pool: &SqlitePool,
    page_id: &str,
    in_trash: bool,
) -> Result<NotePageDto, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin trash notes page: {e}"))?;
    let page_exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM notes_pages WHERE id = ?")
        .bind(page_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("load notes page before trash: {e}"))?;
    if page_exists.is_none() {
        return Err("notes page not found".to_string());
    }
    history::record_page_snapshot_tx(&mut tx, page_id, "trash_page").await?;
    let child_parents = load_external_child_page_block_parents(&mut tx, page_id).await?;
    let root_child_page_block_visible = if in_trash {
        false
    } else {
        repair_page_parent_for_active_restore(&mut tx, page_id).await?
    };
    let result = sqlx::query(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         UPDATE notes_pages
         SET in_trash = ?,
             archived = 0,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id IN (SELECT id FROM page_subtree)",
    )
    .bind(page_id)
    .bind(if in_trash { 1_i64 } else { 0_i64 })
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("trash notes page subtree: {e}"))?;
    sqlx::query(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         UPDATE notes_blocks
         SET in_trash = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id IN (SELECT id FROM page_subtree) AND type = 'child_page'",
    )
    .bind(page_id)
    .bind(if in_trash { 1_i64 } else { 0_i64 })
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("trash notes child page blocks: {e}"))?;
    set_root_child_page_block_visibility(&mut tx, page_id, root_child_page_block_visible).await?;
    for parent in child_parents {
        refresh_parent_has_children(&mut tx, &parent).await?;
        touch_page(&mut tx, &parent.page_id).await?;
    }
    data_source_rollups::invalidate_rollup_cache_for_page_tx(&mut tx, page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit trash notes page: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("notes page not found".to_string());
    }
    reads::get_page(pool, page_id, true).await
}

pub(in crate::notes) async fn permanently_delete_page(
    pool: &SqlitePool,
    page_id: &str,
) -> Result<Vec<String>, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin permanent notes page delete: {e}"))?;
    let root_trash_state: Option<i64> =
        sqlx::query_scalar("SELECT in_trash FROM notes_pages WHERE id = ?")
            .bind(page_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| format!("load notes page before permanent delete: {e}"))?;
    match root_trash_state {
        Some(1) => {}
        Some(_) => return Err("notes page must be in trash before permanent delete".to_string()),
        None => return Err("notes page not found".to_string()),
    }
    let deleted_page_ids: Vec<String> = sqlx::query_scalar(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         SELECT id FROM page_subtree ORDER BY id ASC",
    )
    .bind(page_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("load permanent notes page delete subtree: {e}"))?;
    let child_parents = load_external_child_page_block_parents(&mut tx, page_id).await?;
    data_source_rollups::invalidate_rollup_cache_for_page_tx(&mut tx, page_id).await?;
    sqlx::query(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         DELETE FROM notes_blocks
         WHERE type = 'child_page' AND id IN (SELECT id FROM page_subtree)",
    )
    .bind(page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("delete notes child page blocks permanently: {e}"))?;
    sqlx::query(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         DELETE FROM notes_pages
         WHERE id IN (SELECT id FROM page_subtree)",
    )
    .bind(page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("delete notes pages permanently: {e}"))?;
    for parent in child_parents {
        refresh_parent_has_children(&mut tx, &parent).await?;
        touch_page(&mut tx, &parent.page_id).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit permanent notes page delete: {e}"))?;
    Ok(deleted_page_ids)
}

async fn repair_page_parent_for_active_restore(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<bool, String> {
    let row: Option<PageParentColumns> = sqlx::query_as(
        "SELECT parent_type, parent_page_id, parent_block_id, parent_data_source_id
         FROM notes_pages
         WHERE id = ?",
    )
    .bind(page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load restored notes page parent: {e}"))?;
    let Some((parent_type, parent_page_id, parent_block_id, parent_data_source_id)) = row else {
        return Err("notes page not found".to_string());
    };
    let parent_is_active = match parent_type.as_str() {
        "workspace" => return Ok(false),
        "page_id" => {
            let Some(parent_page_id) = parent_page_id else {
                move_page_to_workspace_parent(tx, page_id).await?;
                return Ok(false);
            };
            active_page_parent_exists(tx, &parent_page_id).await?
        }
        "block_id" => {
            let Some(parent_block_id) = parent_block_id else {
                move_page_to_workspace_parent(tx, page_id).await?;
                return Ok(false);
            };
            active_block_parent_exists(tx, &parent_block_id).await?
        }
        "data_source_id" => {
            let Some(parent_data_source_id) = parent_data_source_id else {
                move_page_to_workspace_parent(tx, page_id).await?;
                return Ok(false);
            };
            active_data_source_parent_exists(tx, &parent_data_source_id).await?
        }
        _ => false,
    };
    if parent_is_active {
        return Ok(true);
    }
    move_page_to_workspace_parent(tx, page_id).await?;
    Ok(false)
}

async fn active_page_parent_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent_page_id: &str,
) -> Result<bool, String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1
         FROM notes_pages
         WHERE id = ? AND in_trash = 0 AND archived = 0",
    )
    .bind(parent_page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check restored notes page parent: {e}"))?;
    Ok(exists.is_some())
}

async fn active_block_parent_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent_block_id: &str,
) -> Result<bool, String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1
         FROM notes_blocks AS block
         JOIN notes_pages AS page ON page.id = block.page_id
         WHERE block.id = ?
           AND block.in_trash = 0
           AND page.in_trash = 0
           AND page.archived = 0",
    )
    .bind(parent_block_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check restored notes block parent: {e}"))?;
    Ok(exists.is_some())
}

async fn active_data_source_parent_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent_data_source_id: &str,
) -> Result<bool, String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1
         FROM notes_data_sources AS data_source
         JOIN notes_databases AS database ON database.id = data_source.database_id
         WHERE data_source.id = ?
           AND data_source.in_trash = 0
           AND database.in_trash = 0",
    )
    .bind(parent_data_source_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check restored notes data source parent: {e}"))?;
    Ok(exists.is_some())
}

async fn move_page_to_workspace_parent(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE notes_pages
         SET parent_type = 'workspace',
             parent_page_id = NULL,
             parent_block_id = NULL,
             parent_data_source_id = NULL,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(page_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("move restored notes page to workspace: {e}"))?;
    Ok(())
}

async fn set_root_child_page_block_visibility(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
    visible: bool,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE notes_blocks
         SET in_trash = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND type = 'child_page'",
    )
    .bind(if visible { 0_i64 } else { 1_i64 })
    .bind(page_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("sync restored notes child page block: {e}"))?;
    Ok(())
}

async fn load_external_child_page_block_parents(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<Vec<ParentTarget>, String> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN page_subtree AS parent ON child.parent_page_id = parent.id
            UNION
            SELECT child.id
            FROM notes_pages AS child
            JOIN notes_blocks AS parent_block ON child.parent_block_id = parent_block.id
            JOIN page_subtree AS parent ON parent_block.page_id = parent.id
            UNION
            SELECT child_block.id
            FROM notes_blocks AS child_block
            JOIN page_subtree AS parent ON child_block.page_id = parent.id
            WHERE child_block.type = 'child_page'
         )
         SELECT DISTINCT block.page_id,
                         block.parent_type,
                         block.parent_page_id,
                         block.parent_block_id
         FROM notes_blocks AS block
         WHERE block.type = 'child_page'
           AND block.id IN (SELECT id FROM page_subtree)
           AND block.page_id NOT IN (SELECT id FROM page_subtree)",
    )
    .bind(page_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load external notes child page block parents: {e}"))?;
    Ok(rows
        .into_iter()
        .map(
            |(parent_page_id, parent_type, parent_parent_page_id, parent_block_id)| ParentTarget {
                parent_type: if parent_type == "page_id" {
                    "page_id"
                } else {
                    "block_id"
                },
                parent_page_id: parent_parent_page_id,
                parent_block_id,
                parent_block_type: None,
                page_id: parent_page_id,
            },
        )
        .collect())
}

pub(in crate::notes) async fn archive_page(
    pool: &SqlitePool,
    page_id: &str,
    archived: bool,
) -> Result<NotePageDto, String> {
    let page_id = page_id.trim();
    require_uuid(page_id, "page_id")?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin archive notes page: {e}"))?;
    let child_block = load_child_page_block_row_any(&mut tx, page_id).await?;
    let child_parent = child_block.as_ref().map(parent_target_from_block_row);
    history::record_page_snapshot_tx(&mut tx, page_id, "archive_page").await?;
    let root_child_page_block_visible = if archived {
        false
    } else {
        repair_page_parent_for_active_restore(&mut tx, page_id).await?
    };
    let result = sqlx::query(
        "UPDATE notes_pages
         SET archived = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND in_trash = 0",
    )
    .bind(if archived { 1_i64 } else { 0_i64 })
    .bind(page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("archive notes page: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("notes page not found".to_string());
    }
    sqlx::query(
        "UPDATE notes_blocks
         SET in_trash = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND type = 'child_page'",
    )
    .bind(if root_child_page_block_visible {
        0_i64
    } else {
        1_i64
    })
    .bind(page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("archive notes child page block: {e}"))?;
    if let Some(parent) = child_parent {
        refresh_parent_has_children(&mut tx, &parent).await?;
        touch_page(&mut tx, &parent.page_id).await?;
    }
    data_source_rollups::invalidate_rollup_cache_for_page_tx(&mut tx, page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit archive notes page: {e}"))?;
    reads::get_page(pool, page_id, true).await
}

pub(in crate::notes) async fn append_block_children(
    pool: &SqlitePool,
    request: NoteAppendBlockChildren,
) -> Result<NotePaginatedBlockList, String> {
    validate_parent(&request.parent)?;
    validate_children_count(request.children.len())?;
    for child in &request.children {
        validate_block_write(child)?;
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin append notes blocks: {e}"))?;
    let parent = resolve_block_parent(&mut tx, &request.parent).await?;
    validate_children_for_parent(&parent, &request.children)?;
    let sort_orders = next_sort_orders(
        &mut tx,
        &parent,
        request.after.as_deref(),
        request.children.len(),
    )
    .await?;
    history::record_page_snapshot_tx(&mut tx, &parent.page_id, "append_block_children").await?;
    let mut inserted_ids = Vec::with_capacity(request.children.len());
    for (child, sort_order) in request.children.iter().zip(sort_orders) {
        insert_block(&mut tx, &parent, child, sort_order).await?;
        inserted_ids.push(child.id.trim().to_string());
    }
    refresh_parent_has_children(&mut tx, &parent).await?;
    touch_page(&mut tx, &parent.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit append notes blocks: {e}"))?;
    load_blocks_by_ids(pool, inserted_ids).await
}

pub(in crate::notes) async fn update_block(
    pool: &SqlitePool,
    block_id: &str,
    update: NoteBlockUpdate,
) -> Result<NoteBlockDto, String> {
    let block_id = block_id.trim();
    require_uuid(block_id, "block_id")?;
    let current = reads::get_block_row(pool, block_id, false).await?;
    let (block_type, payload) = validate_block_update(&current.block_type, &update)?;
    validate_block_update_parent(pool, &current, &block_type, &payload).await?;
    validate_block_update_children(pool, block_id, &current.block_type, &block_type, &payload)
        .await?;
    let plain_text = plain_text_from_payload(&block_type, &payload);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin update notes block: {e}"))?;
    history::record_page_snapshot_tx(&mut tx, &current.page_id, "update_block").await?;
    let result = sqlx::query(
        "UPDATE notes_blocks
         SET type = ?,
             payload = ?,
             plain_text = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND in_trash = 0",
    )
    .bind(&block_type)
    .bind(payload.to_string())
    .bind(&plain_text)
    .bind(block_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("update notes block: {e}"))?;
    if result.rows_affected() == 0 {
        return Err("notes block not found".to_string());
    }
    if block_type == "child_page" {
        let title = payload
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default();
        sqlx::query(
            "UPDATE notes_pages
             SET title = ?,
                 properties = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(title)
        .bind(page_title_properties(title).to_string())
        .bind(block_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("update child page title from block: {e}"))?;
    }
    mention_notifications::sync_block_tx(
        &mut tx,
        block_id,
        &current.page_id,
        &block_type,
        &payload,
        &plain_text,
    )
    .await?;
    touch_page(&mut tx, &current.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit update notes block: {e}"))?;
    reads::get_block(pool, block_id, false).await
}

pub(in crate::notes) async fn trash_block(
    pool: &SqlitePool,
    block_id: &str,
    in_trash: bool,
) -> Result<NoteBlockDto, String> {
    let block_id = block_id.trim();
    require_uuid(block_id, "block_id")?;
    let current = reads::get_block_row(pool, block_id, true).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin trash notes block: {e}"))?;
    history::record_page_snapshot_tx(&mut tx, &current.page_id, "trash_block").await?;
    set_block_subtree_trash(&mut tx, block_id, in_trash).await?;
    let parent = ParentTarget {
        parent_type: if current.parent_type == "page_id" {
            "page_id"
        } else {
            "block_id"
        },
        parent_page_id: current.parent_page_id.clone(),
        parent_block_id: current.parent_block_id.clone(),
        parent_block_type: None,
        page_id: current.page_id.clone(),
    };
    refresh_parent_has_children(&mut tx, &parent).await?;
    touch_page(&mut tx, &current.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit trash notes block: {e}"))?;
    reads::get_block(pool, block_id, true).await
}

pub(in crate::notes) async fn trash_blocks(
    pool: &SqlitePool,
    request: NoteTrashBlocks,
) -> Result<NotePaginatedBlockList, String> {
    let in_trash = request.in_trash.unwrap_or(true);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin trash notes blocks: {e}"))?;
    let root_ids = normalize_selection_root_ids(&mut tx, &request.block_ids, true).await?;
    let mut root_rows = Vec::with_capacity(root_ids.len());
    let mut parents = Vec::with_capacity(root_ids.len());
    let mut touched_pages = HashSet::new();
    for block_id in &root_ids {
        let row = load_block_row_in_tx(&mut tx, block_id, true).await?;
        parents.push(parent_target_from_block_row(&row));
        touched_pages.insert(row.page_id.clone());
        root_rows.push(row);
    }
    for page_id in &touched_pages {
        history::record_page_snapshot_tx(&mut tx, page_id, "trash_blocks").await?;
    }
    for block_id in &root_ids {
        set_block_subtree_trash(&mut tx, block_id, in_trash).await?;
    }
    for parent in &parents {
        refresh_parent_has_children(&mut tx, parent).await?;
    }
    for page_id in &touched_pages {
        touch_page(&mut tx, page_id).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit trash notes blocks: {e}"))?;
    load_blocks_by_ids_with_trash(
        pool,
        root_rows.into_iter().map(|row| row.id).collect(),
        true,
    )
    .await
}

pub(in crate::notes) async fn move_block(
    pool: &SqlitePool,
    block_id: &str,
    request: NoteMoveBlock,
) -> Result<NoteBlockDto, String> {
    let block_id = block_id.trim();
    require_uuid(block_id, "block_id")?;
    validate_parent(&request.parent)?;
    let current = reads::get_block_row(pool, block_id, false).await?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin move notes block: {e}"))?;
    let old_parent = ParentTarget {
        parent_type: if current.parent_type == "page_id" {
            "page_id"
        } else {
            "block_id"
        },
        parent_page_id: current.parent_page_id.clone(),
        parent_block_id: current.parent_block_id.clone(),
        parent_block_type: None,
        page_id: current.page_id.clone(),
    };
    let new_parent = resolve_block_parent(&mut tx, &request.parent).await?;
    let current_payload: Value = serde_json::from_str(&current.payload)
        .map_err(|e| format!("parse moved block payload: {e}"))?;
    validate_block_for_parent(&new_parent, &current.block_type, &current_payload)?;
    ensure_not_moving_into_self(&mut tx, block_id, &new_parent).await?;
    ensure_not_moving_into_subtree_page(&mut tx, block_id, &new_parent.page_id).await?;
    if request.after.is_some() && request.before.is_some() {
        return Err("move request cannot include both after and before".to_string());
    }
    let sort_order = if let Some(before) = request.before.as_deref() {
        sort_order_before(&mut tx, &new_parent, before).await?
    } else {
        next_sort_orders(&mut tx, &new_parent, request.after.as_deref(), 1).await?[0]
    };
    validate_sort_order(sort_order)?;
    history::record_page_snapshot_tx(&mut tx, &current.page_id, "move_block").await?;
    if current.page_id != new_parent.page_id {
        history::record_page_snapshot_tx(&mut tx, &new_parent.page_id, "move_block").await?;
    }
    sqlx::query(
        "UPDATE notes_blocks
         SET page_id = ?,
             parent_type = ?,
             parent_page_id = ?,
             parent_block_id = ?,
             sort_order = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ? AND in_trash = 0",
    )
    .bind(&new_parent.page_id)
    .bind(new_parent.parent_type)
    .bind(&new_parent.parent_page_id)
    .bind(&new_parent.parent_block_id)
    .bind(sort_order)
    .bind(block_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("move notes block: {e}"))?;
    sqlx::query(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE parent_block_id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         UPDATE notes_blocks
         SET page_id = ?
         WHERE id IN (SELECT id FROM subtree)",
    )
    .bind(block_id)
    .bind(&new_parent.page_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("move notes block descendants: {e}"))?;
    update_block_comment_thread_pages(&mut tx, block_id, &new_parent.page_id).await?;
    if current.block_type == "child_page" {
        sqlx::query(
            "UPDATE notes_pages
             SET parent_type = ?,
                 parent_page_id = ?,
                 parent_block_id = ?,
                 parent_data_source_id = NULL,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(new_parent.parent_type)
        .bind(&new_parent.parent_page_id)
        .bind(&new_parent.parent_block_id)
        .bind(block_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("move notes child page parent: {e}"))?;
    }
    refresh_parent_has_children(&mut tx, &old_parent).await?;
    refresh_parent_has_children(&mut tx, &new_parent).await?;
    touch_page(&mut tx, &new_parent.page_id).await?;
    if old_parent.page_id != new_parent.page_id {
        touch_page(&mut tx, &old_parent.page_id).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit move notes block: {e}"))?;
    reads::get_block(pool, block_id, false).await
}

pub(in crate::notes) async fn move_blocks(
    pool: &SqlitePool,
    request: NoteMoveBlocks,
) -> Result<NotePaginatedBlockList, String> {
    validate_parent(&request.parent)?;
    if request.after.is_some() && request.before.is_some() {
        return Err("move request cannot include both after and before".to_string());
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin move notes blocks: {e}"))?;
    let root_ids = normalize_selection_root_ids(&mut tx, &request.block_ids, false).await?;
    let mut root_rows = Vec::with_capacity(root_ids.len());
    let mut old_parents = Vec::with_capacity(root_ids.len());
    for block_id in &root_ids {
        let row = load_block_row_in_tx(&mut tx, block_id, false).await?;
        old_parents.push(parent_target_from_block_row(&row));
        root_rows.push(row);
    }
    let new_parent = resolve_block_parent(&mut tx, &request.parent).await?;
    ensure_insert_anchor_outside_selection(
        &mut tx,
        request.after.as_deref().or(request.before.as_deref()),
        &root_ids,
    )
    .await?;
    for row in &root_rows {
        let payload: Value = serde_json::from_str(&row.payload)
            .map_err(|e| format!("parse moved block payload: {e}"))?;
        validate_block_for_parent(&new_parent, &row.block_type, &payload)?;
        ensure_not_moving_into_self(&mut tx, &row.id, &new_parent).await?;
        ensure_not_moving_into_subtree_page(&mut tx, &row.id, &new_parent.page_id).await?;
    }
    let sort_orders = if let Some(before) = request.before.as_deref() {
        sort_orders_before(&mut tx, &new_parent, before, root_rows.len()).await?
    } else {
        next_sort_orders(
            &mut tx,
            &new_parent,
            request.after.as_deref(),
            root_rows.len(),
        )
        .await?
    };
    let mut history_page_ids = HashSet::from([new_parent.page_id.clone()]);
    for row in &root_rows {
        history_page_ids.insert(row.page_id.clone());
    }
    for page_id in history_page_ids {
        history::record_page_snapshot_tx(&mut tx, &page_id, "move_blocks").await?;
    }
    for (row, sort_order) in root_rows.iter().zip(sort_orders) {
        validate_sort_order(sort_order)?;
        sqlx::query(
            "UPDATE notes_blocks
             SET page_id = ?,
                 parent_type = ?,
                 parent_page_id = ?,
                 parent_block_id = ?,
                 sort_order = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ? AND in_trash = 0",
        )
        .bind(&new_parent.page_id)
        .bind(new_parent.parent_type)
        .bind(&new_parent.parent_page_id)
        .bind(&new_parent.parent_block_id)
        .bind(sort_order)
        .bind(&row.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("move notes block: {e}"))?;
        sqlx::query(
            "WITH RECURSIVE subtree(id) AS (
                SELECT id FROM notes_blocks WHERE parent_block_id = ?
                UNION ALL
                SELECT notes_blocks.id
                FROM notes_blocks
                JOIN subtree ON notes_blocks.parent_block_id = subtree.id
             )
             UPDATE notes_blocks
             SET page_id = ?
             WHERE id IN (SELECT id FROM subtree)",
        )
        .bind(&row.id)
        .bind(&new_parent.page_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("move notes block descendants: {e}"))?;
        update_block_comment_thread_pages(&mut tx, &row.id, &new_parent.page_id).await?;
        if row.block_type == "child_page" {
            sqlx::query(
                "UPDATE notes_pages
                 SET parent_type = ?,
                     parent_page_id = ?,
                     parent_block_id = ?,
                     parent_data_source_id = NULL,
                     last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE id = ?",
            )
            .bind(new_parent.parent_type)
            .bind(&new_parent.parent_page_id)
            .bind(&new_parent.parent_block_id)
            .bind(&row.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("move notes child page parent: {e}"))?;
        }
    }
    for parent in &old_parents {
        refresh_parent_has_children(&mut tx, parent).await?;
        if parent.page_id != new_parent.page_id {
            touch_page(&mut tx, &parent.page_id).await?;
        }
    }
    refresh_parent_has_children(&mut tx, &new_parent).await?;
    touch_page(&mut tx, &new_parent.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit move notes blocks: {e}"))?;
    load_blocks_by_ids(pool, root_ids).await
}

pub(in crate::notes) async fn duplicate_block(
    pool: &SqlitePool,
    block_id: &str,
    request: NoteDuplicateBlock,
) -> Result<NoteBlockDto, String> {
    let block_id = block_id.trim();
    require_uuid(block_id, "block_id")?;
    validate_duplicate_block_count(request.duplicated_block_ids.len())?;
    let mut duplicate_ids = HashMap::with_capacity(request.duplicated_block_ids.len());
    let mut seen_duplicate_ids = HashSet::with_capacity(request.duplicated_block_ids.len());
    for pair in request.duplicated_block_ids {
        let source_id = pair.source_id.trim().to_string();
        let duplicate_id = pair.duplicate_id.trim().to_string();
        require_uuid(&source_id, "source_id")?;
        require_uuid(&duplicate_id, "duplicate_id")?;
        if !seen_duplicate_ids.insert(duplicate_id.clone()) {
            return Err("duplicate_id values must be unique".to_string());
        }
        if duplicate_ids.insert(source_id, duplicate_id).is_some() {
            return Err("source_id values must be unique".to_string());
        }
    }
    let duplicate_root_id = duplicate_ids
        .get(block_id)
        .cloned()
        .ok_or_else(|| "duplicated_block_ids must include the source block".to_string())?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin duplicate notes block: {e}"))?;
    let source_rows = load_block_subtree_rows(&mut tx, block_id).await?;
    if source_rows.is_empty() {
        return Err("notes block not found".to_string());
    }
    if source_rows
        .first()
        .map(|row| row.block_type.as_str())
        .is_some_and(|block_type| block_type == "child_page")
    {
        return Err("child_page blocks must be duplicated through page duplication".to_string());
    }
    if source_rows.len() != duplicate_ids.len() {
        return Err("duplicated_block_ids must match the source block subtree".to_string());
    }
    let source_id_set = source_rows
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    for source_id in duplicate_ids.keys() {
        if !source_id_set.contains(source_id.as_str()) {
            return Err("duplicated_block_ids must match the source block subtree".to_string());
        }
    }
    for duplicate_id in &seen_duplicate_ids {
        if source_id_set.contains(duplicate_id.as_str()) {
            return Err("duplicate_id values must not match source_id values".to_string());
        }
    }
    let source_root = source_rows
        .first()
        .ok_or_else(|| "notes block not found".to_string())?;
    let root_parent = ParentTarget {
        parent_type: if source_root.parent_type == "page_id" {
            "page_id"
        } else {
            "block_id"
        },
        parent_page_id: source_root.parent_page_id.clone(),
        parent_block_id: source_root.parent_block_id.clone(),
        parent_block_type: None,
        page_id: source_root.page_id.clone(),
    };
    let root_sort_order =
        next_sort_orders(&mut tx, &root_parent, Some(source_root.id.as_str()), 1).await?[0];
    history::record_page_snapshot_tx(&mut tx, &source_root.page_id, "duplicate_block").await?;
    for row in &source_rows {
        let duplicate_id = duplicate_ids.get(&row.id).ok_or_else(|| {
            "duplicated_block_ids must match the source block subtree".to_string()
        })?;
        let (parent_type, parent_page_id, parent_block_id, sort_order) =
            if row.id == source_root.id {
                (
                    root_parent.parent_type,
                    root_parent.parent_page_id.clone(),
                    root_parent.parent_block_id.clone(),
                    root_sort_order,
                )
            } else {
                let source_parent_id = row.parent_block_id.as_ref().ok_or_else(|| {
                    "duplicated descendant block is missing its parent".to_string()
                })?;
                let duplicate_parent_id = duplicate_ids.get(source_parent_id).ok_or_else(|| {
                    "duplicated_block_ids must include every descendant parent".to_string()
                })?;
                (
                    "block_id",
                    None,
                    Some(duplicate_parent_id.clone()),
                    row.sort_order,
                )
            };
        validate_sort_order(sort_order)?;
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(duplicate_id)
        .bind(&source_root.page_id)
        .bind(parent_type)
        .bind(parent_page_id)
        .bind(parent_block_id)
        .bind(row.has_children)
        .bind(&row.block_type)
        .bind(&row.payload)
        .bind(&row.plain_text)
        .bind(sort_order)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("duplicate notes block: {e}"))?;
    }
    refresh_duplicated_has_children(&mut tx, &seen_duplicate_ids).await?;
    duplicate_block_comment_threads(&mut tx, &duplicate_ids, &source_root.page_id).await?;
    touch_page(&mut tx, &source_root.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit duplicate notes block: {e}"))?;
    reads::get_block(pool, &duplicate_root_id, false).await
}

pub(in crate::notes) async fn duplicate_blocks(
    pool: &SqlitePool,
    request: NoteDuplicateBlocks,
) -> Result<NotePaginatedBlockList, String> {
    validate_parent(&request.parent)?;
    if request.after.is_some() && request.before.is_some() {
        return Err("duplicate request cannot include both after and before".to_string());
    }
    validate_duplicate_block_count(request.duplicated_block_ids.len())?;
    let (duplicate_ids, seen_duplicate_ids) = duplicate_block_id_map(request.duplicated_block_ids)?;
    let include_trashed_sources = request.include_trashed_sources.unwrap_or(false);
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin duplicate notes blocks: {e}"))?;
    let root_ids =
        normalize_selection_root_ids(&mut tx, &request.block_ids, include_trashed_sources).await?;
    let destination_parent = resolve_block_parent(&mut tx, &request.parent).await?;
    let mut source_rows = Vec::new();
    let mut root_rows = Vec::with_capacity(root_ids.len());
    for block_id in &root_ids {
        let subtree_rows =
            load_block_subtree_rows_with_trash(&mut tx, block_id, include_trashed_sources).await?;
        let source_root = subtree_rows
            .first()
            .ok_or_else(|| "notes block not found".to_string())?;
        if source_root.block_type == "child_page" {
            return Err(
                "child_page blocks must be duplicated through page duplication".to_string(),
            );
        }
        let payload: Value = serde_json::from_str(&source_root.payload)
            .map_err(|e| format!("parse duplicated block payload: {e}"))?;
        validate_block_for_parent(&destination_parent, &source_root.block_type, &payload)?;
        root_rows.push(source_root.clone());
        source_rows.extend(subtree_rows);
    }
    if source_rows.len() != duplicate_ids.len() {
        return Err("duplicated_block_ids must match the source block subtrees".to_string());
    }
    let source_id_set = source_rows
        .iter()
        .map(|row| row.id.as_str())
        .collect::<HashSet<_>>();
    for source_id in duplicate_ids.keys() {
        if !source_id_set.contains(source_id.as_str()) {
            return Err("duplicated_block_ids must match the source block subtrees".to_string());
        }
    }
    for duplicate_id in &seen_duplicate_ids {
        if source_id_set.contains(duplicate_id.as_str()) {
            return Err("duplicate_id values must not match source_id values".to_string());
        }
    }
    let root_id_set = root_ids.iter().map(String::as_str).collect::<HashSet<_>>();
    let root_sort_orders = if let Some(before) = request.before.as_deref() {
        sort_orders_before(&mut tx, &destination_parent, before, root_rows.len()).await?
    } else {
        next_sort_orders(
            &mut tx,
            &destination_parent,
            request.after.as_deref(),
            root_rows.len(),
        )
        .await?
    };
    let root_sort_order_by_source = root_rows
        .iter()
        .zip(root_sort_orders)
        .map(|(row, sort_order)| (row.id.as_str(), sort_order))
        .collect::<HashMap<_, _>>();
    history::record_page_snapshot_tx(&mut tx, &destination_parent.page_id, "duplicate_blocks")
        .await?;
    for row in &source_rows {
        let duplicate_id = duplicate_ids.get(&row.id).ok_or_else(|| {
            "duplicated_block_ids must match the source block subtrees".to_string()
        })?;
        let (parent_type, parent_page_id, parent_block_id, sort_order) =
            if root_id_set.contains(row.id.as_str()) {
                (
                    destination_parent.parent_type,
                    destination_parent.parent_page_id.clone(),
                    destination_parent.parent_block_id.clone(),
                    *root_sort_order_by_source
                        .get(row.id.as_str())
                        .ok_or_else(|| "duplicated root sort order is missing".to_string())?,
                )
            } else {
                let source_parent_id = row.parent_block_id.as_ref().ok_or_else(|| {
                    "duplicated descendant block is missing its parent".to_string()
                })?;
                let duplicate_parent_id = duplicate_ids.get(source_parent_id).ok_or_else(|| {
                    "duplicated_block_ids must include every descendant parent".to_string()
                })?;
                (
                    "block_id",
                    None,
                    Some(duplicate_parent_id.clone()),
                    row.sort_order,
                )
            };
        validate_sort_order(sort_order)?;
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(duplicate_id)
        .bind(&destination_parent.page_id)
        .bind(parent_type)
        .bind(parent_page_id)
        .bind(parent_block_id)
        .bind(row.has_children)
        .bind(&row.block_type)
        .bind(&row.payload)
        .bind(&row.plain_text)
        .bind(sort_order)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("duplicate notes block: {e}"))?;
    }
    refresh_duplicated_has_children(&mut tx, &seen_duplicate_ids).await?;
    duplicate_block_comment_threads(&mut tx, &duplicate_ids, &destination_parent.page_id).await?;
    refresh_parent_has_children(&mut tx, &destination_parent).await?;
    touch_page(&mut tx, &destination_parent.page_id).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit duplicate notes blocks: {e}"))?;
    let duplicate_root_ids = root_ids
        .iter()
        .map(|source_id| {
            duplicate_ids
                .get(source_id)
                .cloned()
                .ok_or_else(|| "duplicated root block id is missing".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    load_blocks_by_ids(pool, duplicate_root_ids).await
}

fn duplicate_block_id_map(
    pairs: Vec<NoteDuplicatedBlockId>,
) -> Result<(HashMap<String, String>, HashSet<String>), String> {
    let mut duplicate_ids = HashMap::with_capacity(pairs.len());
    let mut seen_duplicate_ids = HashSet::with_capacity(pairs.len());
    for pair in pairs {
        let source_id = pair.source_id.trim().to_string();
        let duplicate_id = pair.duplicate_id.trim().to_string();
        require_uuid(&source_id, "source_id")?;
        require_uuid(&duplicate_id, "duplicate_id")?;
        if !seen_duplicate_ids.insert(duplicate_id.clone()) {
            return Err("duplicate_id values must be unique".to_string());
        }
        if duplicate_ids.insert(source_id, duplicate_id).is_some() {
            return Err("source_id values must be unique".to_string());
        }
    }
    Ok((duplicate_ids, seen_duplicate_ids))
}

async fn normalize_selection_root_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_ids: &[String],
    include_trashed: bool,
) -> Result<Vec<String>, String> {
    validate_children_count(block_ids.len())?;
    let mut seen = HashSet::with_capacity(block_ids.len());
    let mut normalized = Vec::with_capacity(block_ids.len());
    for raw_id in block_ids {
        let block_id = raw_id.trim().to_string();
        require_uuid(&block_id, "block_id")?;
        if !seen.insert(block_id.clone()) {
            return Err("block_ids must be unique".to_string());
        }
        normalized.push(block_id);
    }
    let selected = normalized.iter().cloned().collect::<HashSet<_>>();
    let mut roots = Vec::with_capacity(normalized.len());
    for block_id in normalized {
        let row = load_block_row_in_tx(tx, &block_id, include_trashed).await?;
        if selected_ancestor_exists(tx, row.parent_block_id.as_deref(), &selected).await? {
            continue;
        }
        roots.push(block_id);
    }
    if roots.is_empty() {
        return Err("block_ids must include at least one root block".to_string());
    }
    Ok(roots)
}

async fn selected_ancestor_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent_block_id: Option<&str>,
    selected: &HashSet<String>,
) -> Result<bool, String> {
    let mut current = parent_block_id.map(str::to_string);
    while let Some(block_id) = current {
        if selected.contains(&block_id) {
            return Ok(true);
        }
        current = sqlx::query_scalar::<_, Option<String>>(
            "SELECT parent_block_id
             FROM notes_blocks
             WHERE id = ?",
        )
        .bind(&block_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("load selected notes block ancestor: {e}"))?
        .flatten();
    }
    Ok(false)
}

async fn load_block_row_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    include_trashed: bool,
) -> Result<NoteBlockRow, String> {
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
         WHERE id = ? AND (? = 1 OR in_trash = 0)",
    )
    .bind(block_id)
    .bind(if include_trashed { 1_i64 } else { 0_i64 })
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes block row: {e}"))?
    .ok_or_else(|| "notes block not found".to_string())
}

async fn set_block_subtree_trash(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    in_trash: bool,
) -> Result<(), String> {
    sqlx::query(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         UPDATE notes_blocks
         SET in_trash = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id IN (SELECT id FROM subtree)",
    )
    .bind(block_id)
    .bind(if in_trash { 1_i64 } else { 0_i64 })
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("trash notes block subtree: {e}"))?;
    sqlx::query(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         UPDATE notes_pages
         SET in_trash = ?,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id IN (
             SELECT notes_blocks.id
             FROM notes_blocks
             JOIN subtree ON subtree.id = notes_blocks.id
             WHERE notes_blocks.type = 'child_page'
         )",
    )
    .bind(block_id)
    .bind(if in_trash { 1_i64 } else { 0_i64 })
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("trash notes child pages: {e}"))?;
    Ok(())
}

async fn load_block_subtree_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
) -> Result<Vec<NoteBlockRow>, String> {
    load_block_subtree_rows_with_trash(tx, block_id, false).await
}

async fn load_block_subtree_rows_with_trash(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    include_trashed: bool,
) -> Result<Vec<NoteBlockRow>, String> {
    sqlx::query_as::<_, NoteBlockRow>(
        "WITH RECURSIVE subtree(id, path) AS (
            SELECT id, printf('%020.6f:%s', sort_order, id)
            FROM notes_blocks
            WHERE id = ? AND (? = 1 OR in_trash = 0)
            UNION ALL
            SELECT child.id, subtree.path || '/' || printf('%020.6f:%s', child.sort_order, child.id)
            FROM notes_blocks AS child
            JOIN subtree ON child.parent_block_id = subtree.id
            WHERE ? = 1 OR child.in_trash = 0
         )
         SELECT
            notes_blocks.id,
            notes_blocks.page_id,
            notes_blocks.parent_type,
            notes_blocks.parent_page_id,
            notes_blocks.parent_block_id,
            notes_blocks.has_children,
            notes_blocks.in_trash,
            notes_blocks.type AS block_type,
            notes_blocks.payload,
            notes_blocks.plain_text,
            notes_blocks.sort_order,
            notes_blocks.source_provider,
            notes_blocks.source_object_id,
            notes_blocks.source_last_edited_time,
            notes_blocks.created_time,
            notes_blocks.last_edited_time
         FROM notes_blocks
         JOIN subtree ON subtree.id = notes_blocks.id
         ORDER BY subtree.path ASC",
    )
    .bind(block_id)
    .bind(if include_trashed { 1_i64 } else { 0_i64 })
    .bind(if include_trashed { 1_i64 } else { 0_i64 })
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load notes block subtree: {e}"))
}

async fn load_page_block_subtree_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<Vec<NoteBlockRow>, String> {
    sqlx::query_as::<_, NoteBlockRow>(
        "WITH RECURSIVE subtree(id, path) AS (
            SELECT id, printf('%020.6f:%s', sort_order, id)
            FROM notes_blocks
            WHERE parent_type = 'page_id' AND parent_page_id = ? AND in_trash = 0
            UNION ALL
            SELECT child.id, subtree.path || '/' || printf('%020.6f:%s', child.sort_order, child.id)
            FROM notes_blocks AS child
            JOIN subtree ON child.parent_block_id = subtree.id
            WHERE child.in_trash = 0
         )
         SELECT
            notes_blocks.id,
            notes_blocks.page_id,
            notes_blocks.parent_type,
            notes_blocks.parent_page_id,
            notes_blocks.parent_block_id,
            notes_blocks.has_children,
            notes_blocks.in_trash,
            notes_blocks.type AS block_type,
            notes_blocks.payload,
            notes_blocks.plain_text,
            notes_blocks.sort_order,
            notes_blocks.source_provider,
            notes_blocks.source_object_id,
            notes_blocks.source_last_edited_time,
            notes_blocks.created_time,
            notes_blocks.last_edited_time
         FROM notes_blocks
         JOIN subtree ON subtree.id = notes_blocks.id
         ORDER BY subtree.path ASC",
    )
    .bind(page_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| format!("load notes page block subtree: {e}"))
}

async fn load_child_page_block_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
) -> Result<NoteBlockRow, String> {
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
         WHERE id = ? AND type = 'child_page' AND in_trash = 0",
    )
    .bind(block_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes child page block: {e}"))?
    .ok_or_else(|| "child page block not found".to_string())
}

async fn load_child_page_block_row_any(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
) -> Result<Option<NoteBlockRow>, String> {
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
         WHERE id = ? AND type = 'child_page'",
    )
    .bind(block_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("load notes child page block: {e}"))
}

async fn resolve_page_move_parent(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
    parent: &NoteParent,
) -> Result<Option<ParentTarget>, String> {
    match parent {
        NoteParent::Workspace { .. } => Ok(None),
        NoteParent::PageId {
            page_id: parent_page_id,
        } => {
            let parent_page_id = parent_page_id.trim();
            require_uuid(parent_page_id, "parent.page_id")?;
            if parent_page_id == page_id {
                return Err("page cannot be moved under itself".to_string());
            }
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
            )
            .bind(parent_page_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load moved page parent: {e}"))?;
            if exists.is_none() {
                return Err("parent page not found".to_string());
            }
            ensure_page_not_moved_under_descendant(tx, page_id, parent_page_id).await?;
            Ok(Some(ParentTarget {
                parent_type: "page_id",
                parent_page_id: Some(parent_page_id.to_string()),
                parent_block_id: None,
                parent_block_type: None,
                page_id: parent_page_id.to_string(),
            }))
        }
        NoteParent::BlockId { .. } => {
            Err("pages can only be moved to workspace or another page".to_string())
        }
        NoteParent::DataSourceId { .. } => {
            Err("pages can only be moved to workspace or another page".to_string())
        }
    }
}

async fn ensure_page_not_moved_under_descendant(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
    target_page_id: &str,
) -> Result<(), String> {
    let is_descendant: Option<i64> = sqlx::query_scalar(
        "WITH RECURSIVE page_subtree(id) AS (
            SELECT id FROM notes_pages WHERE id = ?
            UNION ALL
            SELECT child.id
            FROM notes_pages AS child
            LEFT JOIN notes_blocks AS parent_block ON parent_block.id = child.parent_block_id
            JOIN page_subtree ON (
                child.parent_type = 'page_id'
                AND child.parent_page_id = page_subtree.id
            ) OR (
                child.parent_type = 'block_id'
                AND parent_block.page_id = page_subtree.id
            )
            WHERE child.in_trash = 0
         )
         SELECT 1 FROM page_subtree WHERE id = ? AND id <> ? LIMIT 1",
    )
    .bind(page_id)
    .bind(target_page_id)
    .bind(page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check moved page descendants: {e}"))?;
    if is_descendant.is_some() {
        return Err("page cannot be moved under its descendant".to_string());
    }
    Ok(())
}

async fn upsert_moved_child_page_block(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
    title: &str,
    parent: &ParentTarget,
    existing_block: Option<NoteBlockRow>,
) -> Result<(), String> {
    let sort_order = next_sort_orders(tx, parent, None, 1).await?[0];
    validate_sort_order(sort_order)?;
    let payload = child_page_payload(title);
    let plain_text = plain_text_from_payload("child_page", &payload);
    if let Some(block) = existing_block {
        if block.block_type != "child_page" {
            return Err("page block id is not a child_page block".to_string());
        }
        sqlx::query(
            "UPDATE notes_blocks
             SET page_id = ?,
                 parent_type = 'page_id',
                 parent_page_id = ?,
                 parent_block_id = NULL,
                 has_children = 0,
                 in_trash = 0,
                 type = 'child_page',
                 payload = ?,
                 plain_text = ?,
                 sort_order = ?,
                 last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?",
        )
        .bind(&parent.page_id)
        .bind(&parent.parent_page_id)
        .bind(payload.to_string())
        .bind(plain_text)
        .bind(sort_order)
        .bind(page_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("move notes child page block: {e}"))?;
    } else {
        sqlx::query(
            "INSERT INTO notes_blocks (
                id,
                page_id,
                parent_type,
                parent_page_id,
                parent_block_id,
                has_children,
                type,
                payload,
                plain_text,
                sort_order
             )
             VALUES (?, ?, 'page_id', ?, NULL, 0, 'child_page', ?, ?, ?)",
        )
        .bind(page_id)
        .bind(&parent.page_id)
        .bind(&parent.parent_page_id)
        .bind(payload.to_string())
        .bind(plain_text)
        .bind(sort_order)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("create moved notes child page block: {e}"))?;
    }
    Ok(())
}

async fn new_note_id(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    reserved_ids: &mut HashSet<String>,
) -> Result<String, String> {
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
        .map_err(|e| format!("generate notes id: {e}"))?;
        require_uuid(&id, "generated_id")?;
        if reserved_ids.contains(&id) {
            continue;
        }
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1
             WHERE EXISTS (SELECT 1 FROM notes_pages WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_blocks WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_comment_threads WHERE id = ?)
                OR EXISTS (SELECT 1 FROM notes_comments WHERE id = ?)",
        )
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("check generated notes id: {e}"))?;
        if exists.is_none() {
            reserved_ids.insert(id.clone());
            return Ok(id);
        }
    }
    Err("could not generate a unique notes id".to_string())
}

async fn refresh_duplicated_has_children(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_ids: &HashSet<String>,
) -> Result<(), String> {
    for block_id in block_ids {
        sqlx::query(
            "UPDATE notes_blocks
             SET has_children = CASE
                 WHEN EXISTS (
                     SELECT 1 FROM notes_blocks AS child
                     WHERE child.parent_block_id = notes_blocks.id
                       AND child.in_trash = 0
                 )
                 THEN 1 ELSE 0 END
             WHERE id = ?",
        )
        .bind(block_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| format!("refresh duplicated notes block children: {e}"))?;
    }
    Ok(())
}

async fn validate_page_parent_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &NoteParent,
) -> Result<(), String> {
    match parent {
        NoteParent::Workspace { .. } => Ok(()),
        NoteParent::PageId { page_id } => {
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
            )
            .bind(page_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load parent page: {e}"))?;
            exists
                .map(|_| ())
                .ok_or_else(|| "parent page not found".to_string())
        }
        NoteParent::BlockId { block_id } => {
            let exists: Option<i64> =
                sqlx::query_scalar("SELECT 1 FROM notes_blocks WHERE id = ? AND in_trash = 0")
                    .bind(block_id)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(|e| format!("load parent block: {e}"))?;
            exists
                .map(|_| ())
                .ok_or_else(|| "parent block not found".to_string())
        }
        NoteParent::DataSourceId { data_source_id } => {
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1
                 FROM notes_data_sources AS data_source
                 JOIN notes_databases AS database ON database.id = data_source.database_id
                 WHERE data_source.id = ?
                   AND data_source.in_trash = 0
                   AND database.in_trash = 0",
            )
            .bind(data_source_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load parent data source: {e}"))?;
            exists
                .map(|_| ())
                .ok_or_else(|| "parent data source not found".to_string())
        }
    }
}

pub(in crate::notes) async fn resolve_block_parent(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &NoteParent,
) -> Result<ParentTarget, String> {
    match parent {
        NoteParent::Workspace { .. } => Err("blocks cannot be parented by workspace".to_string()),
        NoteParent::PageId { page_id } => {
            let page_id = page_id.trim();
            require_uuid(page_id, "page_id")?;
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1 FROM notes_pages WHERE id = ? AND in_trash = 0 AND archived = 0",
            )
            .bind(page_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load block parent page: {e}"))?;
            if exists.is_none() {
                return Err("parent page not found".to_string());
            }
            Ok(ParentTarget {
                parent_type: "page_id",
                parent_page_id: Some(page_id.to_string()),
                parent_block_id: None,
                parent_block_type: None,
                page_id: page_id.to_string(),
            })
        }
        NoteParent::BlockId { block_id } => {
            let block_id = block_id.trim();
            require_uuid(block_id, "block_id")?;
            let parent: Option<(String, String, String)> = sqlx::query_as(
                "SELECT page_id, type, payload FROM notes_blocks WHERE id = ? AND in_trash = 0",
            )
            .bind(block_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| format!("load block parent block: {e}"))?;
            let (page_id, block_type, payload) =
                parent.ok_or_else(|| "parent block not found".to_string())?;
            let payload: Value = serde_json::from_str(&payload)
                .map_err(|e| format!("parse parent block payload: {e}"))?;
            if !block_payload_supports_children(&block_type, &payload) {
                return Err(format!("{block_type} blocks cannot have children"));
            }
            Ok(ParentTarget {
                parent_type: "block_id",
                parent_page_id: None,
                parent_block_id: Some(block_id.to_string()),
                parent_block_type: Some(block_type),
                page_id,
            })
        }
        NoteParent::DataSourceId { .. } => {
            Err("blocks cannot be parented by data sources".to_string())
        }
    }
}

fn validate_children_for_parent(
    parent: &ParentTarget,
    children: &[NoteBlockWrite],
) -> Result<(), String> {
    for child in children {
        let payload = child
            .payload()
            .ok_or_else(|| format!("{} payload is required", child.block_type))?;
        validate_block_for_parent(parent, &child.block_type, payload)?;
    }
    Ok(())
}

fn validate_block_for_parent(
    parent: &ParentTarget,
    block_type: &str,
    payload: &Value,
) -> Result<(), String> {
    validate_block_type_for_parent(parent, block_type)?;
    if block_type == "paragraph"
        && payload.get("icon").is_some()
        && parent.parent_block_type.as_deref() != Some("tab")
    {
        return Err("paragraph.icon is only supported for tab labels".to_string());
    }
    Ok(())
}

fn validate_block_type_for_parent(parent: &ParentTarget, block_type: &str) -> Result<(), String> {
    if parent.parent_block_type.as_deref() == Some("column_list") {
        if block_type == "column" {
            return Ok(());
        }
        return Err(format!(
            "{block_type} blocks cannot be children of column_list blocks"
        ));
    }
    if parent.parent_block_type.as_deref() == Some("table") {
        if block_type == "table_row" {
            return Ok(());
        }
        return Err(format!(
            "{block_type} blocks cannot be children of table blocks"
        ));
    }
    if parent.parent_block_type.as_deref() == Some("tab") {
        if block_type == "paragraph" {
            return Ok(());
        }
        return Err(format!(
            "{block_type} blocks cannot be children of tab blocks"
        ));
    }
    if block_type == "table_row" {
        return Err("table_row blocks must be children of table blocks".to_string());
    }
    if block_type == "column" {
        return Err("column blocks must be children of column_list blocks".to_string());
    }
    Ok(())
}

async fn validate_block_update_parent(
    pool: &SqlitePool,
    current: &NoteBlockRow,
    target_block_type: &str,
    target_payload: &Value,
) -> Result<(), String> {
    let parent_block_type = if let Some(parent_block_id) = &current.parent_block_id {
        sqlx::query_scalar::<_, String>(
            "SELECT type FROM notes_blocks WHERE id = ? AND in_trash = 0",
        )
        .bind(parent_block_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("load notes block parent type: {e}"))?
    } else {
        None
    };
    let parent = ParentTarget {
        parent_type: if current.parent_type == "page_id" {
            "page_id"
        } else {
            "block_id"
        },
        parent_page_id: current.parent_page_id.clone(),
        parent_block_id: current.parent_block_id.clone(),
        parent_block_type,
        page_id: current.page_id.clone(),
    };
    validate_block_for_parent(&parent, target_block_type, target_payload)
}

async fn validate_block_update_children(
    pool: &SqlitePool,
    block_id: &str,
    current_block_type: &str,
    target_block_type: &str,
    target_payload: &Value,
) -> Result<(), String> {
    if target_block_type == "table" {
        ensure_existing_children_have_type(pool, block_id, "table_row", "table").await?;
    }
    if target_block_type == "column_list" {
        ensure_existing_children_have_type(pool, block_id, "column", "column_list").await?;
    }
    if target_block_type == "tab" {
        ensure_existing_children_have_type(pool, block_id, "paragraph", "tab").await?;
    }
    if current_block_type == "table" && target_block_type != "table" {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_blocks
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count notes table rows: {e}"))?;
        if child_count > 0 {
            return Err(format!(
                "table blocks with rows cannot be converted to {target_block_type} blocks"
            ));
        }
    }
    if current_block_type == "column_list" && target_block_type != "column_list" {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_blocks
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count notes column children: {e}"))?;
        if child_count > 0 {
            return Err(format!(
                "column_list blocks with columns cannot be converted to {target_block_type} blocks"
            ));
        }
    }
    if current_block_type == "tab" && target_block_type != "tab" {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_blocks
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count notes tab children: {e}"))?;
        if child_count > 0 {
            return Err(format!(
                "tab blocks with labels cannot be converted to {target_block_type} blocks"
            ));
        }
    }
    if matches!(
        target_block_type,
        "heading_1" | "heading_2" | "heading_3" | "heading_4"
    ) && !block_payload_supports_children(target_block_type, target_payload)
    {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_blocks
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count notes heading children: {e}"))?;
        if child_count > 0 {
            return Err(format!(
                "{target_block_type} blocks with children must stay toggleable"
            ));
        }
    }
    if target_block_type == "synced_block"
        && !block_payload_supports_children(target_block_type, target_payload)
    {
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notes_blocks
             WHERE parent_block_id = ? AND in_trash = 0",
        )
        .bind(block_id)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("count notes synced block children: {e}"))?;
        if child_count > 0 {
            return Err("synced_block blocks with children must stay original".to_string());
        }
    }
    Ok(())
}

async fn ensure_existing_children_have_type(
    pool: &SqlitePool,
    block_id: &str,
    allowed_child_type: &str,
    parent_type: &str,
) -> Result<(), String> {
    let invalid_child_type: Option<String> = sqlx::query_scalar(
        "SELECT type
         FROM notes_blocks
         WHERE parent_block_id = ?
           AND in_trash = 0
           AND type <> ?
         LIMIT 1",
    )
    .bind(block_id)
    .bind(allowed_child_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("load notes {parent_type} child type: {e}"))?;
    if invalid_child_type.is_some() {
        return Err(format!(
            "{parent_type} blocks can only contain {allowed_child_type} blocks"
        ));
    }
    Ok(())
}

async fn insert_block(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    block: &NoteBlockWrite,
    sort_order: f64,
) -> Result<(), String> {
    validate_sort_order(sort_order)?;
    let payload = block
        .payload()
        .ok_or_else(|| format!("{} payload is required", block.block_type))?;
    let plain_text = plain_text_from_payload(&block.block_type, payload);
    sqlx::query(
        "INSERT INTO notes_blocks (
            id,
            page_id,
            parent_type,
            parent_page_id,
            parent_block_id,
            type,
            payload,
            plain_text,
            sort_order
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(block.id.trim())
    .bind(&parent.page_id)
    .bind(parent.parent_type)
    .bind(&parent.parent_page_id)
    .bind(&parent.parent_block_id)
    .bind(block.block_type.trim())
    .bind(payload.to_string())
    .bind(&plain_text)
    .bind(sort_order)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert notes block: {e}"))?;
    mention_notifications::sync_block_tx(
        tx,
        block.id.trim(),
        &parent.page_id,
        block.block_type.trim(),
        payload,
        &plain_text,
    )
    .await?;
    Ok(())
}

pub(in crate::notes) async fn next_sort_orders(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    after: Option<&str>,
    count: usize,
) -> Result<Vec<f64>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if let Some(after_id) = after.map(str::trim).filter(|value| !value.is_empty()) {
        require_uuid(after_id, "after")?;
        let after_row: Option<(f64,)> = if parent.parent_type == "page_id" {
            sqlx::query_as(
                "SELECT sort_order
                 FROM notes_blocks
                 WHERE id = ?
                   AND parent_type = 'page_id'
                   AND parent_page_id = ?
                   AND page_id = ?
                   AND in_trash = 0",
            )
            .bind(after_id)
            .bind(&parent.parent_page_id)
            .bind(&parent.page_id)
            .fetch_optional(&mut **tx)
            .await
        } else {
            sqlx::query_as(
                "SELECT sort_order
                 FROM notes_blocks
                 WHERE id = ?
                   AND parent_type = 'block_id'
                   AND parent_block_id = ?
                   AND page_id = ?
                   AND in_trash = 0",
            )
            .bind(after_id)
            .bind(&parent.parent_block_id)
            .bind(&parent.page_id)
            .fetch_optional(&mut **tx)
            .await
        }
        .map_err(|e| format!("load notes after block: {e}"))?;
        let after_order = after_row
            .map(|row| row.0)
            .ok_or_else(|| "after block not found".to_string())?;
        let next_order = next_sibling_order_after(tx, parent, after_order, after_id).await?;
        let step = match next_order {
            Some(next_order) if next_order > after_order => {
                (next_order - after_order) / (count as f64 + 1.0)
            }
            _ => DEFAULT_BLOCK_SORT_STEP,
        };
        return Ok((1..=count)
            .map(|index| after_order + step * index as f64)
            .collect());
    }
    let max_order: Option<f64> = if parent.parent_type == "page_id" {
        sqlx::query_scalar(
            "SELECT MAX(sort_order)
             FROM notes_blocks
             WHERE parent_type = 'page_id'
               AND parent_page_id = ?
               AND in_trash = 0",
        )
        .bind(&parent.parent_page_id)
        .fetch_one(&mut **tx)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT MAX(sort_order)
             FROM notes_blocks
             WHERE parent_type = 'block_id'
               AND parent_block_id = ?
               AND in_trash = 0",
        )
        .bind(&parent.parent_block_id)
        .fetch_one(&mut **tx)
        .await
    }
    .map_err(|e| format!("load notes max sort order: {e}"))?;
    let base = max_order.unwrap_or(0.0);
    Ok((1..=count)
        .map(|index| base + DEFAULT_BLOCK_SORT_STEP * index as f64)
        .collect())
}

async fn next_sibling_order_after(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    after_order: f64,
    after_id: &str,
) -> Result<Option<f64>, String> {
    let next_order: Option<f64> = if parent.parent_type == "page_id" {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE parent_type = 'page_id'
               AND parent_page_id = ?
               AND in_trash = 0
               AND (sort_order > ? OR (sort_order = ? AND id > ?))
             ORDER BY sort_order ASC, id ASC
             LIMIT 1",
        )
        .bind(&parent.parent_page_id)
        .bind(after_order)
        .bind(after_order)
        .bind(after_id)
        .fetch_optional(&mut **tx)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE parent_type = 'block_id'
               AND parent_block_id = ?
               AND in_trash = 0
               AND (sort_order > ? OR (sort_order = ? AND id > ?))
             ORDER BY sort_order ASC, id ASC
             LIMIT 1",
        )
        .bind(&parent.parent_block_id)
        .bind(after_order)
        .bind(after_order)
        .bind(after_id)
        .fetch_optional(&mut **tx)
        .await
    }
    .map_err(|e| format!("load next notes sibling: {e}"))?;
    Ok(next_order)
}

async fn sort_order_before(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    before_id: &str,
) -> Result<f64, String> {
    let before_id = before_id.trim();
    require_uuid(before_id, "before")?;
    let before_order: f64 = if parent.parent_type == "page_id" {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE id = ?
               AND parent_type = 'page_id'
               AND parent_page_id = ?
               AND page_id = ?
               AND in_trash = 0",
        )
        .bind(before_id)
        .bind(&parent.parent_page_id)
        .bind(&parent.page_id)
        .fetch_optional(&mut **tx)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE id = ?
               AND parent_type = 'block_id'
               AND parent_block_id = ?
               AND page_id = ?
               AND in_trash = 0",
        )
        .bind(before_id)
        .bind(&parent.parent_block_id)
        .bind(&parent.page_id)
        .fetch_optional(&mut **tx)
        .await
    }
    .map_err(|e| format!("load notes before block: {e}"))?
    .ok_or_else(|| "before block not found".to_string())?;
    let previous_order = previous_sibling_order_before(tx, parent, before_order, before_id).await?;
    Ok(match previous_order {
        Some(previous_order) if previous_order < before_order => {
            previous_order + (before_order - previous_order) / 2.0
        }
        _ if before_order > 0.0 => before_order / 2.0,
        _ => 0.0,
    })
}

async fn sort_orders_before(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    before_id: &str,
    count: usize,
) -> Result<Vec<f64>, String> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let before_id = before_id.trim();
    require_uuid(before_id, "before")?;
    let before_order: f64 = if parent.parent_type == "page_id" {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE id = ?
               AND parent_type = 'page_id'
               AND parent_page_id = ?
               AND page_id = ?
               AND in_trash = 0",
        )
        .bind(before_id)
        .bind(&parent.parent_page_id)
        .bind(&parent.page_id)
        .fetch_optional(&mut **tx)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE id = ?
               AND parent_type = 'block_id'
               AND parent_block_id = ?
               AND page_id = ?
               AND in_trash = 0",
        )
        .bind(before_id)
        .bind(&parent.parent_block_id)
        .bind(&parent.page_id)
        .fetch_optional(&mut **tx)
        .await
    }
    .map_err(|e| format!("load notes before block: {e}"))?
    .ok_or_else(|| "before block not found".to_string())?;
    let previous_order = previous_sibling_order_before(tx, parent, before_order, before_id).await?;
    let (base, step) = match previous_order {
        Some(previous_order) if previous_order < before_order => (
            previous_order,
            (before_order - previous_order) / (count as f64 + 1.0),
        ),
        _ if before_order > 0.0 => (0.0, before_order / (count as f64 + 1.0)),
        _ => (0.0, 0.0),
    };
    Ok((1..=count)
        .map(|index| base + step * index as f64)
        .collect())
}

async fn previous_sibling_order_before(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
    before_order: f64,
    before_id: &str,
) -> Result<Option<f64>, String> {
    let previous_order: Option<f64> = if parent.parent_type == "page_id" {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE parent_type = 'page_id'
               AND parent_page_id = ?
               AND in_trash = 0
               AND (sort_order < ? OR (sort_order = ? AND id < ?))
             ORDER BY sort_order DESC, id DESC
             LIMIT 1",
        )
        .bind(&parent.parent_page_id)
        .bind(before_order)
        .bind(before_order)
        .bind(before_id)
        .fetch_optional(&mut **tx)
        .await
    } else {
        sqlx::query_scalar(
            "SELECT sort_order
             FROM notes_blocks
             WHERE parent_type = 'block_id'
               AND parent_block_id = ?
               AND in_trash = 0
               AND (sort_order < ? OR (sort_order = ? AND id < ?))
             ORDER BY sort_order DESC, id DESC
             LIMIT 1",
        )
        .bind(&parent.parent_block_id)
        .bind(before_order)
        .bind(before_order)
        .bind(before_id)
        .fetch_optional(&mut **tx)
        .await
    }
    .map_err(|e| format!("load previous notes sibling: {e}"))?;
    Ok(previous_order)
}

pub(in crate::notes) async fn refresh_parent_has_children(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    parent: &ParentTarget,
) -> Result<(), String> {
    let Some(block_id) = &parent.parent_block_id else {
        return Ok(());
    };
    sqlx::query(
        "UPDATE notes_blocks
         SET has_children = CASE
             WHEN EXISTS (
                 SELECT 1 FROM notes_blocks AS child
                 WHERE child.parent_block_id = notes_blocks.id
                   AND child.in_trash = 0
             )
             THEN 1 ELSE 0 END,
             last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(block_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("refresh notes parent has_children: {e}"))?;
    Ok(())
}

pub(in crate::notes) async fn touch_page(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    page_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE notes_pages
         SET last_edited_time = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?",
    )
    .bind(page_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("touch notes page: {e}"))?;
    Ok(())
}

async fn load_blocks_by_ids(
    pool: &SqlitePool,
    ids: Vec<String>,
) -> Result<NotePaginatedBlockList, String> {
    load_blocks_by_ids_with_trash(pool, ids, false).await
}

async fn load_blocks_by_ids_with_trash(
    pool: &SqlitePool,
    ids: Vec<String>,
    include_trashed: bool,
) -> Result<NotePaginatedBlockList, String> {
    let mut rows = Vec::with_capacity(ids.len());
    for id in ids {
        rows.push(reads::get_block_row(pool, &id, include_trashed).await?);
    }
    let results = rows
        .into_iter()
        .map(NoteBlockDto::new)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NotePaginatedBlockList::new(results, None, false))
}

async fn update_block_comment_thread_pages(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    page_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         UPDATE notes_comment_threads
         SET page_id = ?
         WHERE parent_type = 'block_id'
           AND parent_block_id IN (SELECT id FROM subtree)",
    )
    .bind(block_id)
    .bind(page_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("move notes block comment threads: {e}"))?;
    Ok(())
}

async fn duplicate_block_comment_threads(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    duplicate_ids: &HashMap<String, String>,
    page_id: &str,
) -> Result<(), String> {
    let mut reserved_ids = HashSet::new();
    for (source_id, duplicate_id) in duplicate_ids {
        let threads = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                String,
            ),
        >(
            "SELECT id, status, resolved_at, resolved_by, created_time, last_edited_time
             FROM notes_comment_threads
             WHERE parent_type = 'block_id' AND parent_block_id = ?",
        )
        .bind(source_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| format!("load notes block comment threads: {e}"))?;
        for (thread_id, status, resolved_at, resolved_by, created_time, last_edited_time) in threads
        {
            let duplicate_thread_id = new_note_id(tx, &mut reserved_ids).await?;
            sqlx::query(
                "INSERT INTO notes_comment_threads (
                    id,
                    page_id,
                    parent_type,
                    parent_page_id,
                    parent_block_id,
                    status,
                    resolved_at,
                    resolved_by,
                    created_time,
                    last_edited_time
                 )
                 VALUES (?, ?, 'block_id', NULL, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&duplicate_thread_id)
            .bind(page_id)
            .bind(duplicate_id)
            .bind(&status)
            .bind(&resolved_at)
            .bind(&resolved_by)
            .bind(&created_time)
            .bind(&last_edited_time)
            .execute(&mut **tx)
            .await
            .map_err(|e| format!("duplicate notes block comment thread: {e}"))?;
            let comments = sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    String,
                    String,
                    String,
                    Option<String>,
                    String,
                    String,
                ),
            >(
                "SELECT rich_text,
                        plain_text,
                        created_by,
                        display_name,
                        attachments,
                        deleted_at,
                        created_time,
                        last_edited_time
                 FROM notes_comments
                 WHERE thread_id = ?
                 ORDER BY created_time ASC, id ASC",
            )
            .bind(&thread_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| format!("load notes block comments: {e}"))?;
            for (
                rich_text,
                plain_text,
                created_by,
                display_name,
                attachments,
                deleted_at,
                comment_created_time,
                comment_last_edited_time,
            ) in comments
            {
                let duplicate_comment_id = new_note_id(tx, &mut reserved_ids).await?;
                sqlx::query(
                    "INSERT INTO notes_comments (
                        id,
                        thread_id,
                        rich_text,
                        plain_text,
                        created_by,
                        display_name,
                        attachments,
                        deleted_at,
                        created_time,
                        last_edited_time
                     )
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&duplicate_comment_id)
                .bind(&duplicate_thread_id)
                .bind(&rich_text)
                .bind(&plain_text)
                .bind(&created_by)
                .bind(&display_name)
                .bind(&attachments)
                .bind(&deleted_at)
                .bind(&comment_created_time)
                .bind(&comment_last_edited_time)
                .execute(&mut **tx)
                .await
                .map_err(|e| format!("duplicate notes block comment: {e}"))?;
            }
        }
    }
    Ok(())
}

async fn ensure_not_moving_into_self(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    parent: &ParentTarget,
) -> Result<(), String> {
    let Some(parent_block_id) = &parent.parent_block_id else {
        return Ok(());
    };
    if parent_block_id == block_id {
        return Err("block cannot be moved under itself".to_string());
    }
    let descendant: Option<i64> = sqlx::query_scalar(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE parent_block_id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         SELECT 1 FROM subtree WHERE id = ? LIMIT 1",
    )
    .bind(block_id)
    .bind(parent_block_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check notes move cycle: {e}"))?;
    if descendant.is_some() {
        return Err("block cannot be moved under its descendant".to_string());
    }
    Ok(())
}

async fn ensure_insert_anchor_outside_selection(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    anchor_id: Option<&str>,
    root_ids: &[String],
) -> Result<(), String> {
    let Some(anchor_id) = anchor_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    require_uuid(anchor_id, "anchor")?;
    for root_id in root_ids {
        let inside_subtree: Option<i64> = sqlx::query_scalar(
            "WITH RECURSIVE subtree(id) AS (
                SELECT id FROM notes_blocks WHERE id = ?
                UNION ALL
                SELECT notes_blocks.id
                FROM notes_blocks
                JOIN subtree ON notes_blocks.parent_block_id = subtree.id
             )
             SELECT 1 FROM subtree WHERE id = ? LIMIT 1",
        )
        .bind(root_id)
        .bind(anchor_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| format!("check notes selection anchor: {e}"))?;
        if inside_subtree.is_some() {
            return Err("insert anchor cannot be inside the selected block subtree".to_string());
        }
    }
    Ok(())
}

async fn ensure_not_moving_into_subtree_page(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    block_id: &str,
    destination_page_id: &str,
) -> Result<(), String> {
    let is_subtree_page: Option<i64> = sqlx::query_scalar(
        "WITH RECURSIVE subtree(id) AS (
            SELECT id FROM notes_blocks WHERE id = ?
            UNION ALL
            SELECT notes_blocks.id
            FROM notes_blocks
            JOIN subtree ON notes_blocks.parent_block_id = subtree.id
         )
         SELECT 1 FROM subtree WHERE id = ? LIMIT 1",
    )
    .bind(block_id)
    .bind(destination_page_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| format!("check notes block subtree pages: {e}"))?;
    if is_subtree_page.is_some() {
        return Err("block cannot be moved into a page contained by its subtree".to_string());
    }
    Ok(())
}

fn page_title_properties(title: &str) -> Value {
    json!({
        "title": {
            "id": "title",
            "type": "title",
            "title": [rich_text(title)]
        }
    })
}

fn page_row_properties_for_title(page: &NotePageRow, title: &str) -> Result<String, String> {
    if page.parent_type == "data_source_id" {
        data_source_page_properties_with_title(&page.properties, title)
    } else {
        Ok(page_title_properties(title).to_string())
    }
}

fn data_source_page_properties_with_title(properties: &str, title: &str) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(properties).map_err(|e| format!("parse row page properties: {e}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "row page properties must be an object".to_string())?;
    for property in object.values_mut() {
        let Some(property_object) = property.as_object_mut() else {
            continue;
        };
        if property_object.get("type").and_then(Value::as_str) == Some("title") {
            property_object.insert("title".to_string(), Value::Array(vec![rich_text(title)]));
            return Ok(value.to_string());
        }
    }
    Err("row page properties are missing a title property".to_string())
}

fn child_page_payload(title: &str) -> Value {
    json!({
        "title": title
    })
}

pub(in crate::notes) fn default_text_payload(text: &str) -> Value {
    json!({
        "rich_text": [rich_text(text)],
        "color": "default"
    })
}

pub(in crate::notes) fn rich_text(text: &str) -> Value {
    json!({
        "type": "text",
        "text": {
            "content": text,
            "link": null
        },
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
    })
}
