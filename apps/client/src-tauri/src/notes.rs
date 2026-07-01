use crate::db_path::connect_sqlite;
use tauri::{AppHandle, Runtime};

mod comments;
mod models;
mod reads;
mod undo_state;
mod validation;
mod writes;

pub use models::*;

#[tauri::command]
pub async fn notes_list_pages<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Vec<NotePageDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::list_pages(&pool).await
}

#[tauri::command]
pub async fn notes_list_trashed_pages<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Vec<NotePageDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::list_trashed_pages(&pool).await
}

#[tauri::command]
pub async fn notes_list_archived_pages<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Vec<NotePageDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::list_archived_pages(&pool).await
}

#[tauri::command]
pub async fn notes_list_backlinks<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<Vec<NoteBacklinkDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::list_backlinks(&pool, &page_id).await
}

#[tauri::command]
pub async fn notes_search<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    query: String,
    page_size: Option<i64>,
) -> Result<Vec<NoteSearchResultDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::search(&pool, &query, page_size).await
}

#[tauri::command]
pub async fn notes_list_comments<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    include_resolved: Option<bool>,
) -> Result<Vec<NoteCommentThreadDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    comments::list_comments(&pool, &page_id, include_resolved.unwrap_or(false)).await
}

#[tauri::command]
pub async fn notes_create_comment<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteCommentCreate,
) -> Result<NoteCommentThreadDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    comments::create_comment(&pool, request).await
}

#[tauri::command]
pub async fn notes_update_comment<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    comment_id: String,
    update: NoteCommentUpdate,
) -> Result<NoteCommentThreadDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    comments::update_comment(&pool, &comment_id, update).await
}

#[tauri::command]
pub async fn notes_delete_comment<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    comment_id: String,
) -> Result<NoteCommentThreadDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    comments::delete_comment(&pool, &comment_id).await
}

#[tauri::command]
pub async fn notes_resolve_comment_thread<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    discussion_id: String,
    resolved: Option<bool>,
) -> Result<NoteCommentThreadDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    comments::resolve_comment_thread(&pool, &discussion_id, resolved.unwrap_or(true)).await
}

#[tauri::command]
pub async fn notes_create_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page: NotePageCreate,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::create_page(&pool, page).await
}

#[tauri::command]
pub async fn notes_create_child_page_from_block<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_id: String,
    request: NoteChildPageFromBlockCreate,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::create_child_page_from_block(&pool, &block_id, request).await
}

#[tauri::command]
pub async fn notes_duplicate_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    request: NoteDuplicatePage,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::duplicate_page(&pool, &page_id, request).await
}

#[tauri::command]
pub async fn notes_move_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    request: NoteMovePage,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::move_page(&pool, &page_id, request).await
}

#[tauri::command]
pub async fn notes_update_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    update: NotePageUpdate,
) -> Result<NotePageDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::update_page(&pool, &page_id, update).await
}

#[tauri::command]
pub async fn notes_trash_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    in_trash: Option<bool>,
) -> Result<NotePageDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::trash_page(&pool, &page_id, in_trash.unwrap_or(true)).await
}

#[tauri::command]
pub async fn notes_archive_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    archived: Option<bool>,
) -> Result<NotePageDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::archive_page(&pool, &page_id, archived.unwrap_or(true)).await
}

#[tauri::command]
pub async fn notes_permanently_delete_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<Vec<String>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::permanently_delete_page(&pool, &page_id).await
}

#[tauri::command]
pub async fn notes_load_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::load_page(&pool, &page_id).await
}

#[tauri::command]
pub async fn notes_get_block_children<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    parent_id: String,
    start_cursor: Option<String>,
    page_size: Option<i64>,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::get_block_children(&pool, &parent_id, start_cursor.as_deref(), page_size).await
}

#[tauri::command]
pub async fn notes_append_block_children<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteAppendBlockChildren,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::append_block_children(&pool, request).await
}

#[tauri::command]
pub async fn notes_update_block<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_id: String,
    update: NoteBlockUpdate,
) -> Result<NoteBlockDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::update_block(&pool, &block_id, update).await
}

#[tauri::command]
pub async fn notes_trash_block<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_id: String,
    in_trash: Option<bool>,
) -> Result<NoteBlockDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::trash_block(&pool, &block_id, in_trash.unwrap_or(true)).await
}

#[tauri::command]
pub async fn notes_move_block<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_id: String,
    request: NoteMoveBlock,
) -> Result<NoteBlockDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::move_block(&pool, &block_id, request).await
}

#[tauri::command]
pub async fn notes_duplicate_block<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    block_id: String,
    request: NoteDuplicateBlock,
) -> Result<NoteBlockDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::duplicate_block(&pool, &block_id, request).await
}

#[tauri::command]
pub async fn notes_load_undo_state<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<Option<String>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    undo_state::load_undo_state(&pool, &page_id).await
}

#[tauri::command]
pub async fn notes_save_undo_state<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    state_json: String,
) -> Result<(), String> {
    let pool = connect_sqlite(app, db_url).await?;
    undo_state::save_undo_state(&pool, &page_id, &state_json).await
}

#[tauri::command]
pub async fn notes_clear_undo_state<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<(), String> {
    let pool = connect_sqlite(app, db_url).await?;
    undo_state::clear_undo_state(&pool, &page_id).await
}

#[cfg(test)]
mod tests;
