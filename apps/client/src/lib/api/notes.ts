import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import {
  mapNotesBacklinkDto,
  mapNotesBlockDto,
  mapNotesBlockListDto,
  mapNotesCommentThreadDto,
  mapNotesLoadedPageDto,
  mapNotesPageDto,
  mapNotesSearchResultDto,
} from "$lib/notes/notion-mappers";
import type {
  NotesAppendBlockChildrenRequest,
  NotesBacklink,
  NotesBlock,
  NotesChildPageFromBlockCreate,
  NotesCommentCreate,
  NotesCommentThread,
  NotesCommentUpdate,
  NotesDuplicatePageRequest,
  NotesDuplicateBlockRequest,
  NotesBlockUpdate,
  NotesLoadedPage,
  NotesMovePageRequest,
  NotesMoveBlockRequest,
  NotesPage,
  NotesPageCreate,
  NotesPageUpdate,
  NotesPaginatedBlockList,
  NotesSearchResult,
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

export async function listNotesBacklinks(pageId: string): Promise<NotesBacklink[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<unknown>("notes_list_backlinks", { dbUrl, pageId });
  if (!Array.isArray(rows)) throw new Error("notes_list_backlinks returned a non-array payload");
  return rows.map(mapNotesBacklinkDto);
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

export async function moveNotesBlock(
  blockId: string,
  request: NotesMoveBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_move_block", { dbUrl, blockId, request }));
}

export async function duplicateNotesBlock(
  blockId: string,
  request: NotesDuplicateBlockRequest,
): Promise<NotesBlock> {
  const dbUrl = await ensureDbUrl();
  return mapNotesBlockDto(await invoke<unknown>("notes_duplicate_block", { dbUrl, blockId, request }));
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
