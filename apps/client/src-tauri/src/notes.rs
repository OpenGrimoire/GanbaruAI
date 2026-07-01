use crate::db_path::connect_sqlite;
use tauri::{AppHandle, Runtime};

mod comments;
mod history;
mod models;
mod page_cover_assets;
mod page_icon_assets;
mod reads;
mod templates;
mod undo_state;
mod validation;
mod writes;

pub use models::*;
pub use page_cover_assets::*;
pub use page_icon_assets::*;

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
pub async fn notes_list_sidebar_pages<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteSidebarPagesRequest,
) -> Result<NoteSidebarPageList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::list_sidebar_pages(&pool, request).await
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
pub async fn notes_get_page_breadcrumb<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<Vec<NotePageBreadcrumbItemDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    reads::get_page_breadcrumb(&pool, &page_id).await
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
pub async fn notes_list_page_templates<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<Vec<NotePageTemplateDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::list_page_templates(&pool).await
}

#[tauri::command]
pub async fn notes_create_page_template_from_page<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NotePageTemplateCreateFromPage,
) -> Result<NotePageTemplateDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::create_page_template_from_page(&pool, request).await
}

#[tauri::command]
pub async fn notes_apply_page_template<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    template_id: String,
    request: NotePageTemplateApply,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::apply_page_template(&pool, &template_id, request).await
}

#[tauri::command]
pub async fn notes_update_page_template<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    template_id: String,
    update: NotePageTemplateUpdate,
) -> Result<NotePageTemplateDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::update_page_template(&pool, &template_id, update).await
}

#[tauri::command]
pub async fn notes_duplicate_page_template<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    template_id: String,
    request: NotePageTemplateDuplicate,
) -> Result<NotePageTemplateDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::duplicate_page_template(&pool, &template_id, request).await
}

#[tauri::command]
pub async fn notes_delete_page_template<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    template_id: String,
) -> Result<String, String> {
    let pool = connect_sqlite(app, db_url).await?;
    templates::delete_page_template(&pool, &template_id).await
}

#[tauri::command]
pub async fn notes_get_page_history_settings<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
) -> Result<NotePageHistorySettingsDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::get_page_history_settings(&pool).await
}

#[tauri::command]
pub async fn notes_update_page_history_settings<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    update: NotePageHistorySettingsUpdate,
) -> Result<NotePageHistorySettingsDto, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::update_page_history_settings(&pool, update).await
}

#[tauri::command]
pub async fn notes_list_page_history_snapshots<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
) -> Result<Vec<NotePageHistorySnapshotDto>, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::list_page_history_snapshots(&pool, &page_id).await
}

#[tauri::command]
pub async fn notes_load_page_history_snapshot<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    snapshot_id: String,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::load_page_history_snapshot(&pool, &page_id, &snapshot_id).await
}

#[tauri::command]
pub async fn notes_restore_page_history_snapshot<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    snapshot_id: String,
) -> Result<NoteLoadedPage, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::restore_page_history_snapshot(&pool, &page_id, &snapshot_id).await
}

#[tauri::command]
pub async fn notes_copy_page_history_blocks<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    page_id: String,
    snapshot_id: String,
    request: NotePageHistoryCopyBlocks,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    history::copy_page_history_blocks(&pool, &page_id, &snapshot_id, request).await
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
pub async fn notes_trash_blocks<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteTrashBlocks,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::trash_blocks(&pool, request).await
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
pub async fn notes_move_blocks<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteMoveBlocks,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::move_blocks(&pool, request).await
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
pub async fn notes_duplicate_blocks<R: Runtime>(
    app: AppHandle<R>,
    db_url: String,
    request: NoteDuplicateBlocks,
) -> Result<NotePaginatedBlockList, String> {
    let pool = connect_sqlite(app, db_url).await?;
    writes::duplicate_blocks(&pool, request).await
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
