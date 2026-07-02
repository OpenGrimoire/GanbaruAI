use super::models::{parent_columns, NoteLoadedPage, NoteParent};
use super::validation::{
    plain_text_from_payload, validate_block_payload, validate_parent, validate_sort_order,
};
use super::{reads, writes};
use serde_json::{json, Value};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashSet;

pub(super) struct ImportBlock {
    pub(super) id: String,
    pub(super) block_type: &'static str,
    pub(super) payload: Value,
    pub(super) source_object_id: Option<String>,
    pub(super) children: Vec<ImportBlock>,
}

impl ImportBlock {
    pub(super) fn new(
        block_type: &'static str,
        payload: Value,
        source_object_id: Option<String>,
        children: Vec<ImportBlock>,
    ) -> Self {
        Self {
            id: String::new(),
            block_type,
            payload,
            source_object_id,
            children,
        }
    }
}

pub(super) struct ImportedPageCreate<'a> {
    pub(super) parent: &'a NoteParent,
    pub(super) after_block_id: Option<&'a str>,
    pub(super) title: &'a str,
    pub(super) source_provider: &'a str,
    pub(super) source_object_id: Option<&'a str>,
    pub(super) blocks: Vec<ImportBlock>,
}

pub(super) async fn create_imported_page(
    pool: &SqlitePool,
    mut request: ImportedPageCreate<'_>,
) -> Result<NoteLoadedPage, String> {
    validate_parent(request.parent)?;
    if matches!(request.parent, NoteParent::DataSourceId { .. }) {
        return Err("document imports cannot create database row pages".to_string());
    }
    if request.blocks.is_empty() {
        request.blocks.push(ImportBlock::new(
            "paragraph",
            json!({
                "rich_text": [empty_rich_text()],
                "color": "default"
            }),
            None,
            Vec::new(),
        ));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("begin {} import: {e}", request.source_provider))?;
    writes::validate_page_parent_exists(&mut tx, request.parent).await?;
    let mut reserved_ids = HashSet::new();
    let page_id = writes::new_note_id(&mut tx, &mut reserved_ids).await?;
    let block_ids = new_note_ids(
        &mut tx,
        &mut reserved_ids,
        count_import_blocks(&request.blocks),
    )
    .await?;
    let mut block_ids = block_ids.into_iter();
    assign_block_ids(&mut request.blocks, &mut block_ids);
    insert_import_page(&mut tx, &page_id, &request).await?;
    insert_import_child_page_block(&mut tx, &page_id, &request).await?;
    let flattened_blocks = flatten_blocks(&page_id, &request.blocks);
    for block in flattened_blocks {
        insert_raw_block_with_id(&mut tx, &request, block).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("commit {} import: {e}", request.source_provider))?;
    reads::load_page(pool, &page_id).await
}

pub(super) fn count_import_blocks(blocks: &[ImportBlock]) -> usize {
    blocks
        .iter()
        .map(|block| 1 + count_import_blocks(&block.children))
        .sum()
}

async fn new_note_ids(
    tx: &mut Transaction<'_, Sqlite>,
    reserved_ids: &mut HashSet<String>,
    count: usize,
) -> Result<Vec<String>, String> {
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(writes::new_note_id(tx, reserved_ids).await?);
    }
    Ok(ids)
}

fn assign_block_ids(blocks: &mut [ImportBlock], ids: &mut impl Iterator<Item = String>) {
    for block in blocks {
        block.id = ids
            .next()
            .expect("generated import block ids must cover every parsed block");
        assign_block_ids(&mut block.children, ids);
    }
}

async fn insert_import_page(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
    request: &ImportedPageCreate<'_>,
) -> Result<(), String> {
    let (parent_type, parent_page_id, parent_block_id) = parent_columns(request.parent);
    sqlx::query(
        "INSERT INTO notes_pages (
            id,
            parent_type,
            parent_page_id,
            parent_block_id,
            title,
            properties,
            source_provider,
            source_object_id
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(page_id)
    .bind(parent_type)
    .bind(parent_page_id)
    .bind(parent_block_id)
    .bind(request.title)
    .bind(writes::page_title_properties(request.title).to_string())
    .bind(request.source_provider)
    .bind(request.source_object_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("create {} import page: {e}", request.source_provider))?;
    Ok(())
}

async fn insert_import_child_page_block(
    tx: &mut Transaction<'_, Sqlite>,
    page_id: &str,
    request: &ImportedPageCreate<'_>,
) -> Result<(), String> {
    if matches!(request.parent, NoteParent::Workspace { .. }) {
        return Ok(());
    }
    let parent = writes::resolve_block_parent(tx, request.parent).await?;
    let sort_order = writes::next_sort_orders(tx, &parent, request.after_block_id, 1).await?[0];
    insert_raw_block_with_id(
        tx,
        request,
        RawBlockInsert {
            id: page_id.to_string(),
            page_id: page_id.to_string(),
            parent: RawBlockParent {
                parent_type: parent.parent_type,
                parent_page_id: parent.parent_page_id.clone(),
                parent_block_id: parent.parent_block_id.clone(),
            },
            block_type: "child_page",
            payload: json!({ "title": request.title }),
            sort_order,
            has_children: false,
            source_object_id: request.source_object_id.map(str::to_string),
        },
    )
    .await?;
    writes::refresh_parent_has_children(tx, &parent).await?;
    writes::touch_page(tx, &parent.page_id).await
}

fn flatten_blocks(page_id: &str, blocks: &[ImportBlock]) -> Vec<RawBlockInsert> {
    let mut flattened = Vec::new();
    flatten_children(
        page_id,
        RawBlockParent {
            parent_type: "page_id",
            parent_page_id: Some(page_id.to_string()),
            parent_block_id: None,
        },
        blocks,
        &mut flattened,
    );
    flattened
}

fn flatten_children(
    page_id: &str,
    parent: RawBlockParent,
    blocks: &[ImportBlock],
    output: &mut Vec<RawBlockInsert>,
) {
    for (index, block) in blocks.iter().enumerate() {
        output.push(RawBlockInsert {
            id: block.id.clone(),
            page_id: page_id.to_string(),
            parent: parent.clone(),
            block_type: block.block_type,
            payload: block.payload.clone(),
            sort_order: 1000.0 + index as f64 * 1000.0,
            has_children: !block.children.is_empty(),
            source_object_id: block.source_object_id.clone(),
        });
        flatten_children(
            page_id,
            RawBlockParent {
                parent_type: "block_id",
                parent_page_id: None,
                parent_block_id: Some(block.id.clone()),
            },
            &block.children,
            output,
        );
    }
}

async fn insert_raw_block_with_id(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ImportedPageCreate<'_>,
    block: RawBlockInsert,
) -> Result<(), String> {
    validate_sort_order(block.sort_order)?;
    validate_block_payload(block.block_type, &block.payload)?;
    let plain_text = plain_text_from_payload(block.block_type, &block.payload);
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
            sort_order,
            source_provider,
            source_object_id
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(block.id)
    .bind(block.page_id)
    .bind(block.parent.parent_type)
    .bind(block.parent.parent_page_id)
    .bind(block.parent.parent_block_id)
    .bind(if block.has_children { 1 } else { 0 })
    .bind(block.block_type)
    .bind(block.payload.to_string())
    .bind(plain_text)
    .bind(block.sort_order)
    .bind(request.source_provider)
    .bind(block.source_object_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| format!("insert {} import block: {e}", request.source_provider))?;
    Ok(())
}

fn empty_rich_text() -> Value {
    json!({
        "type": "text",
        "text": {
            "content": "",
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
        "plain_text": "",
        "href": null
    })
}

#[derive(Clone)]
struct RawBlockParent {
    parent_type: &'static str,
    parent_page_id: Option<String>,
    parent_block_id: Option<String>,
}

struct RawBlockInsert {
    id: String,
    page_id: String,
    parent: RawBlockParent,
    block_type: &'static str,
    payload: Value,
    sort_order: f64,
    has_children: bool,
    source_object_id: Option<String>,
}
