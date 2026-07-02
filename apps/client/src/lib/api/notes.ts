import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import {
  mapNotesBacklinkDto,
  mapNotesBlockDto,
  mapNotesBlockListDto,
  mapNotesCommentThreadDto,
  mapNotesCreatedDatabaseDto,
  mapNotesDataSourceBoardViewDto,
  mapNotesDataSourceGalleryViewDto,
  mapNotesDataSourceListViewDto,
  mapNotesDataSourceSchemaDto,
  mapNotesDataSourceTableViewDto,
  mapNotesLoadedPageDto,
  mapNotesPageBreadcrumbItemDto,
  mapNotesPageHistorySettingsDto,
  mapNotesPageHistorySnapshotDto,
  mapNotesPageDto,
  mapNotesPageTemplateDto,
  mapNotesSearchResultDto,
  mapNotesSidebarPageListDto,
} from "$lib/notes/notion-mappers";
import type {
  NotesAppendBlockChildrenRequest,
  NotesBacklink,
  NotesBlock,
  NotesChildPageFromBlockCreate,
  NotesCommentCreate,
  NotesCommentThread,
  NotesCommentUpdate,
  NotesCreatedDatabase,
  NotesDatabaseCreateRequest,
  NotesDataSourceBoardRowMove,
  NotesDataSourceBoardView,
  NotesDataSourceBoardViewUpdate,
  NotesDataSourceGalleryView,
  NotesDataSourceGalleryViewUpdate,
  NotesDataSourceListView,
  NotesDataSourceListViewUpdate,
  NotesDataSourceRowPageCreateRequest,
  NotesDataSourceRowPropertyUpdate,
  NotesDataSourceSchema,
  NotesDataSourceSchemaUpdate,
  NotesDataSourceTableView,
  NotesDataSourceTableViewUpdate,
  NotesDuplicatePageRequest,
  NotesDuplicateBlockRequest,
  NotesDuplicateBlocksRequest,
  NotesBlockUpdate,
  NotesLoadedPage,
  NotesMovePageRequest,
  NotesMoveBlockRequest,
  NotesMoveBlocksRequest,
  NotesPage,
  NotesPageBreadcrumbItem,
  NotesPageCreate,
  NotesPageHistoryCopyBlocksRequest,
  NotesPageHistorySettings,
  NotesPageHistorySettingsUpdate,
  NotesPageHistorySnapshot,
  NotesPageTemplate,
  NotesPageTemplateApplyRequest,
  NotesPageTemplateCreateFromPageRequest,
  NotesPageTemplateDuplicateRequest,
  NotesPageTemplateUpdateRequest,
  NotesPageUpdate,
  NotesPaginatedBlockList,
  NotesSearchResult,
  NotesSidebarPageList,
  NotesSidebarPagesRequest,
  NotesTrashBlocksRequest,
} from "$lib/notes/types";

export async function listNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_pages", { dbUrl });
  if (!Array.isArray(rows)) throw new Error("notes_list_pages returned a non-array payload");
  return rows.map(mapNotesPageDto);
}

export async function listTrashedNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_trashed_pages", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_trashed_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function listArchivedNotesPages(): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_archived_pages", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_archived_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function listNotesSidebarPages(
  request: NotesSidebarPagesRequest,
): Promise<NotesSidebarPageList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesSidebarPageListDto(
    await invoke<unknown>("notes_list_sidebar_pages", { dbUrl, request }),
  );
}

export async function listNotesBacklinks(pageId: string): Promise<NotesBacklink[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_backlinks", { dbUrl, pageId });
  if (!Array.isArray(rows)) throw new Error("notes_list_backlinks returned a non-array payload");
  return rows.map(mapNotesBacklinkDto);
}

export async function getNotesPageBreadcrumb(pageId: string): Promise<NotesPageBreadcrumbItem[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_get_page_breadcrumb", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_get_page_breadcrumb returned a non-array payload");
  }
  return rows.map(mapNotesPageBreadcrumbItemDto);
}

export async function searchNotes(
  query: string,
  pageSize = 20,
): Promise<NotesSearchResult[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_search", { dbUrl, query, pageSize });
  if (!Array.isArray(rows)) throw new Error("notes_search returned a non-array payload");
  return rows.map(mapNotesSearchResultDto);
}

export async function listNotesPageTemplates(): Promise<NotesPageTemplate[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_page_templates", { dbUrl });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_page_templates returned a non-array payload");
  }
  return rows.map(mapNotesPageTemplateDto);
}

export async function createNotesPageTemplateFromPage(
  request: NotesPageTemplateCreateFromPageRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_create_page_template_from_page", { dbUrl, request }),
  );
}

export async function applyNotesPageTemplate(
  templateId: string,
  request: NotesPageTemplateApplyRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_apply_page_template", { dbUrl, templateId, request }),
  );
}

export async function updateNotesPageTemplate(
  templateId: string,
  update: NotesPageTemplateUpdateRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_update_page_template", { dbUrl, templateId, update }),
  );
}

export async function duplicateNotesPageTemplate(
  templateId: string,
  request: NotesPageTemplateDuplicateRequest,
): Promise<NotesPageTemplate> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageTemplateDto(
    await invoke<unknown>("notes_duplicate_page_template", { dbUrl, templateId, request }),
  );
}

export async function deleteNotesPageTemplate(templateId: string): Promise<string> {
  const dbUrl = await ensureDbUrl();
  const deletedTemplateId = await invoke<unknown>("notes_delete_page_template", { dbUrl, templateId });
  if (typeof deletedTemplateId !== "string") {
    throw new Error("notes_delete_page_template returned an invalid template id");
  }
  return deletedTemplateId;
}

export async function getNotesPageHistorySettings(): Promise<NotesPageHistorySettings> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageHistorySettingsDto(
    await invoke<unknown>("notes_get_page_history_settings", { dbUrl }),
  );
}

export async function updateNotesPageHistorySettings(
  update: NotesPageHistorySettingsUpdate,
): Promise<NotesPageHistorySettings> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageHistorySettingsDto(
    await invoke<unknown>("notes_update_page_history_settings", { dbUrl, update }),
  );
}

export async function listNotesPageHistorySnapshots(
  pageId: string,
): Promise<NotesPageHistorySnapshot[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_page_history_snapshots", { dbUrl, pageId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_page_history_snapshots returned a non-array payload");
  }
  return rows.map(mapNotesPageHistorySnapshotDto);
}

export async function loadNotesPageHistorySnapshot(
  pageId: string,
  snapshotId: string,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_load_page_history_snapshot", { dbUrl, pageId, snapshotId }),
  );
}

export async function restoreNotesPageHistorySnapshot(
  pageId: string,
  snapshotId: string,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_restore_page_history_snapshot", { dbUrl, pageId, snapshotId }),
  );
}

export async function copyNotesPageHistoryBlocks(
  pageId: string,
  snapshotId: string,
  request: NotesPageHistoryCopyBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(
    await invoke<unknown>("notes_copy_page_history_blocks", { dbUrl, pageId, snapshotId, request }),
  );
}

export async function listNotesComments(
  pageId: string,
  includeResolved = false,
): Promise<NotesCommentThread[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_comments", { dbUrl, pageId, includeResolved });
  if (!Array.isArray(rows)) throw new Error("notes_list_comments returned a non-array payload");
  return rows.map(mapNotesCommentThreadDto);
}

export async function createNotesComment(
  request: NotesCommentCreate,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCommentThreadDto(await invoke<unknown>("notes_create_comment", { dbUrl, request }));
}

export async function updateNotesComment(
  commentId: string,
  update: NotesCommentUpdate,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCommentThreadDto(
    await invoke<unknown>("notes_update_comment", { dbUrl, commentId, update }),
  );
}

export async function deleteNotesComment(commentId: string): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCommentThreadDto(
    await invoke<unknown>("notes_delete_comment", { dbUrl, commentId }),
  );
}

export async function resolveNotesCommentThread(
  discussionId: string,
  resolved = true,
): Promise<NotesCommentThread> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCommentThreadDto(
    await invoke<unknown>("notes_resolve_comment_thread", { dbUrl, discussionId, resolved }),
  );
}

export async function createNotesPage(page: NotesPageCreate): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invoke<unknown>("notes_create_page", { dbUrl, page }));
}

export async function createNotesChildPageFromBlock(
  blockId: string,
  request: NotesChildPageFromBlockCreate,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_create_child_page_from_block", { dbUrl, blockId, request }),
  );
}

export async function createNotesDatabase(
  request: NotesDatabaseCreateRequest,
): Promise<NotesCreatedDatabase> {
  const dbUrl = await ensureDbUrl();
  return mapNotesCreatedDatabaseDto(
    await invoke<unknown>("notes_create_database", { dbUrl, request }),
  );
}

export async function getNotesDataSourceSchema(
  dataSourceId: string,
): Promise<NotesDataSourceSchema> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceSchemaDto(
    await invoke<unknown>("notes_get_data_source_schema", { dbUrl, dataSourceId }),
  );
}

export async function updateNotesDataSourceSchema(
  dataSourceId: string,
  update: NotesDataSourceSchemaUpdate,
): Promise<NotesDataSourceSchema> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceSchemaDto(
    await invoke<unknown>("notes_update_data_source_schema", { dbUrl, dataSourceId, update }),
  );
}

export async function listNotesDataSourceRowPages(
  dataSourceId: string,
): Promise<NotesPage[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_data_source_row_pages", { dbUrl, dataSourceId });
  if (!Array.isArray(rows)) {
    throw new Error("notes_list_data_source_row_pages returned a non-array payload");
  }
  return rows.map(mapNotesPageDto);
}

export async function createNotesDataSourceRowPage(
  dataSourceId: string,
  request: NotesDataSourceRowPageCreateRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(
    await invoke<unknown>("notes_create_data_source_row_page", {
      dbUrl,
      dataSourceId,
      request,
    }),
  );
}

export async function getNotesDataSourceTableView(
  dataSourceId: string,
): Promise<NotesDataSourceTableView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTableViewDto(
    await invoke<unknown>("notes_get_data_source_table_view", { dbUrl, dataSourceId }),
  );
}

export async function updateNotesDataSourceTableView(
  dataSourceId: string,
  update: NotesDataSourceTableViewUpdate,
): Promise<NotesDataSourceTableView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceTableViewDto(
    await invoke<unknown>("notes_update_data_source_table_view", {
      dbUrl,
      dataSourceId,
      update,
    }),
  );
}

export async function getNotesDataSourceBoardView(
  dataSourceId: string,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invoke<unknown>("notes_get_data_source_board_view", { dbUrl, dataSourceId }),
  );
}

export async function updateNotesDataSourceBoardView(
  dataSourceId: string,
  update: NotesDataSourceBoardViewUpdate,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invoke<unknown>("notes_update_data_source_board_view", {
      dbUrl,
      dataSourceId,
      update,
    }),
  );
}

export async function moveNotesDataSourceBoardRow(
  dataSourceId: string,
  request: NotesDataSourceBoardRowMove,
): Promise<NotesDataSourceBoardView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceBoardViewDto(
    await invoke<unknown>("notes_move_data_source_board_row", {
      dbUrl,
      dataSourceId,
      request,
    }),
  );
}

export async function getNotesDataSourceGalleryView(
  dataSourceId: string,
): Promise<NotesDataSourceGalleryView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceGalleryViewDto(
    await invoke<unknown>("notes_get_data_source_gallery_view", { dbUrl, dataSourceId }),
  );
}

export async function updateNotesDataSourceGalleryView(
  dataSourceId: string,
  update: NotesDataSourceGalleryViewUpdate,
): Promise<NotesDataSourceGalleryView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceGalleryViewDto(
    await invoke<unknown>("notes_update_data_source_gallery_view", {
      dbUrl,
      dataSourceId,
      update,
    }),
  );
}

export async function updateNotesDataSourceRowProperty(
  dataSourceId: string,
  pageId: string,
  update: NotesDataSourceRowPropertyUpdate,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(
    await invoke<unknown>("notes_update_data_source_row_property", {
      dbUrl,
      dataSourceId,
      pageId,
      update,
    }),
  );
}

export async function getNotesDataSourceListView(
  dataSourceId: string,
): Promise<NotesDataSourceListView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceListViewDto(
    await invoke<unknown>("notes_get_data_source_list_view", { dbUrl, dataSourceId }),
  );
}

export async function updateNotesDataSourceListView(
  dataSourceId: string,
  update: NotesDataSourceListViewUpdate,
): Promise<NotesDataSourceListView> {
  const dbUrl = await ensureDbUrl();
  return mapNotesDataSourceListViewDto(
    await invoke<unknown>("notes_update_data_source_list_view", {
      dbUrl,
      dataSourceId,
      update,
    }),
  );
}

export async function duplicateNotesPage(
  pageId: string,
  request: NotesDuplicatePageRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invoke<unknown>("notes_duplicate_page", { dbUrl, pageId, request }));
}

export async function moveNotesPage(
  pageId: string,
  request: NotesMovePageRequest,
): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invoke<unknown>("notes_move_page", { dbUrl, pageId, request }));
}

export async function updateNotesPage(
  pageId: string,
  update: NotesPageUpdate,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(await invoke<unknown>("notes_update_page", { dbUrl, pageId, update }));
}

export async function trashNotesPage(
  pageId: string,
  inTrash = true,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(await invoke<unknown>("notes_trash_page", { dbUrl, pageId, inTrash }));
}

export async function archiveNotesPage(
  pageId: string,
  archived = true,
): Promise<NotesPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesPageDto(await invoke<unknown>("notes_archive_page", { dbUrl, pageId, archived }));
}

export async function permanentlyDeleteNotesPage(pageId: string): Promise<string[]> {
  const dbUrl = await ensureDbUrl();
  const deletedPageIds = await invoke<unknown>("notes_permanently_delete_page", { dbUrl, pageId });
  if (!Array.isArray(deletedPageIds)) {
    throw new Error("notes_permanently_delete_page returned a non-array payload");
  }
  if (!deletedPageIds.every((deletedPageId): deletedPageId is string => typeof deletedPageId === "string")) {
    throw new Error("notes_permanently_delete_page returned invalid page ids");
  }
  return deletedPageIds;
}

export async function loadNotesPage(pageId: string): Promise<NotesLoadedPage> {
  const dbUrl = await ensureDbUrl();
  return mapNotesLoadedPageDto(await invoke<unknown>("notes_load_page", { dbUrl, pageId }));
}

export async function getNotesBlockChildren(
  parentId: string,
  startCursor: string | null = null,
  pageSize = 50,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(
    await invoke<unknown>("notes_get_block_children", {
      dbUrl,
      parentId,
      startCursor,
      pageSize,
    }),
  );
}

export async function appendNotesBlockChildren(
  request: NotesAppendBlockChildrenRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(await invoke<unknown>("notes_append_block_children", { dbUrl, request }));
}

export async function updateNotesBlock(
  blockId: string,
  update: NotesBlockUpdate,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_update_block", { dbUrl, blockId, update }));
}

export async function trashNotesBlock(
  blockId: string,
  inTrash = true,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_trash_block", { dbUrl, blockId, inTrash }));
}

export async function trashNotesBlocks(
  request: NotesTrashBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(await invoke<unknown>("notes_trash_blocks", { dbUrl, request }));
}

export async function moveNotesBlock(
  blockId: string,
  request: NotesMoveBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_move_block", { dbUrl, blockId, request }));
}

export async function moveNotesBlocks(
  request: NotesMoveBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(await invoke<unknown>("notes_move_blocks", { dbUrl, request }));
}

export async function duplicateNotesBlock(
  blockId: string,
  request: NotesDuplicateBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_duplicate_block", { dbUrl, blockId, request }));
}

export async function duplicateNotesBlocks(
  request: NotesDuplicateBlocksRequest,
): Promise<NotesPaginatedBlockList> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockListDto(await invoke<unknown>("notes_duplicate_blocks", { dbUrl, request }));
}

export async function loadNotesUndoState(pageId: string): Promise<string | null> {
  const dbUrl = await ensureDbUrl();
  const state = await invoke<unknown>("notes_load_undo_state", { dbUrl, pageId });
  if (state === null) return null;
  if (typeof state !== "string") throw new Error("notes_load_undo_state returned invalid state");
  return state;
}

export async function saveNotesUndoState(
  pageId: string,
  stateJson: string,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("notes_save_undo_state", { dbUrl, pageId, stateJson });
}

export async function clearNotesUndoState(pageId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("notes_clear_undo_state", { dbUrl, pageId });
}
