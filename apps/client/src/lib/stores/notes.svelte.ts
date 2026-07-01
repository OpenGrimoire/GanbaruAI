import {
  archiveNotesPage,
  createNotesChildPageFromBlock,
  createNotesComment,
  createNotesPage,
  deleteNotesComment,
  duplicateNotesPage,
  getNotesBlockChildren,
  listNotesBacklinks,
  listNotesComments,
  listNotesPages,
  listArchivedNotesPages,
  listTrashedNotesPages,
  loadNotesPage,
  moveNotesPage,
  permanentlyDeleteNotesPage,
  resolveNotesCommentThread,
  searchNotes,
  trashNotesPage,
  updateNotesComment,
  updateNotesPage,
} from "$lib/api/notes";
import { blockPlainText, createRichText } from "$lib/notes/block-factory";
import { notesCommentParentKey } from "$lib/notes/comments";
import type { NotesBlockLinkTarget, NotesPageLinkTarget } from "$lib/notes/block-link";
import {
  buildNotesChildIdsByParent,
  parentIdForBlock,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import { nextSelectedNotesPageId } from "$lib/notes/page-selection";
import {
  recordRecentNotesPageId,
  setNotesPageFavoriteId,
} from "$lib/notes/page-navigation";
import { nextNotesFocusRequest, type NotesFocusRequest } from "$lib/notes/editor-focus";
import { createNotesBlockActions } from "./notes-store-block-actions";
import { createNotesUndoController } from "./notes-store-undo";
import {
  flatNotesBlockItems,
  flatNotesBlockItemsForContext,
  isOnlyNotesBlockInContext,
  notesColumnItemsForBlock,
  notesTabItemsForBlock,
  notesTableRowsForBlock,
  notesTreeState,
  previousNotesBlockType,
  type NotesBlockTreeSnapshot,
} from "./notes-store-block-tree";
import {
  initialNotesFavoritePageIds,
  initialNotesRecentPageIds,
  initialNotesSelectedPageId,
  initialNotesSidebarCollapsedPageIds,
  saveNotesFavoritePageIds,
  saveNotesRecentPageIds,
  saveNotesSelectedPageId,
  saveNotesSidebarCollapsedPageIds,
} from "./notes-store-page-state";
import { createNotesBlockPersistence } from "./notes-store-persistence";
import type {
  NotesBlock,
  NotesBacklink,
  NotesColumnBlockItems,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesCommentParent,
  NotesCommentThread,
  NotesLoadedPage,
  NotesPage,
  NotesPageCover,
  NotesPageIcon,
  NotesParent,
  NotesSearchResult,
  NotesTabBlockItems,
  NotesTableRowBlock,
} from "$lib/notes/types";

type NotesViewMode = "pages" | "archive" | "trash";

const BLOCK_SAVE_DEBOUNCE_MS = 350;
const CHILDREN_PAGE_SIZE = 100;

let pages = $state<NotesPage[]>([]);
let archivedPages = $state<NotesPage[]>([]);
let trashedPages = $state<NotesPage[]>([]);
let selectedPageId = $state<string | null>(initialNotesSelectedPageId());
let favoritePageIds = $state<string[]>(initialNotesFavoritePageIds());
let recentPageIds = $state<string[]>(initialNotesRecentPageIds());
let sidebarCollapsedPageIds = $state<string[]>(initialNotesSidebarCollapsedPageIds());
let loadedPage = $state<NotesPage | null>(null);
let backlinks = $state<NotesBacklink[]>([]);
let commentThreads = $state<NotesCommentThread[]>([]);
let activeCommentParent = $state<NotesCommentParent | null>(null);
let blocksById = $state<Record<string, NotesBlock>>({});
let childIdsByParentId = $state<Record<string, string[]>>({});
let loaded = $state(false);
let loading = $state(false);
let loadError = $state<string | null>(null);
let viewMode = $state<NotesViewMode>("pages");
let archiveLoaded = $state(false);
let archiveLoading = $state(false);
let archiveError = $state<string | null>(null);
let trashLoaded = $state(false);
let trashLoading = $state(false);
let trashError = $state<string | null>(null);
let focusRequest = $state<NotesFocusRequest>({ blockId: null, requestId: 0 });
let loadRequestId = 0;
let archiveRequestId = 0;
let trashRequestId = 0;
let backlinksRequestId = 0;
let commentsRequestId = 0;
let searchRequestId = 0;
let backlinksLoading = $state(false);
let backlinksError = $state<string | null>(null);
let commentsLoading = $state(false);
let commentsError = $state<string | null>(null);
let commentsIncludeResolved = $state(false);
let searchResults = $state<NotesSearchResult[]>([]);
let searchLoading = $state(false);
let searchError = $state<string | null>(null);

function blockTreeSnapshot(): NotesBlockTreeSnapshot {
  return { selectedPageId, blocksById, childIdsByParentId };
}

function treeState(): NotesTreeState {
  return notesTreeState(blockTreeSnapshot());
}

function saveSelectedPageId(pageId: string | null): void {
  selectedPageId = pageId;
  saveNotesSelectedPageId(pageId);
}

function recordRecentPage(pageId: string): void {
  const next = recordRecentNotesPageId(recentPageIds, pageId);
  recentPageIds = next;
  saveNotesRecentPageIds(next);
}

function requestBlockFocus(blockId: string | null): void {
  focusRequest = nextNotesFocusRequest(focusRequest, blockId);
}

function replacePages(nextPages: NotesPage[]): void {
  pages = [...nextPages];
}

function replaceBlock(block: NotesBlock): void {
  blocksById = { ...blocksById, [block.id]: block };
}

function setLoadedPageFromLoaded(loaded: NotesLoadedPage): void {
  loadedPage = loaded.page;
  const blocks = loaded.blocks.results;
  blocksById = Object.fromEntries(blocks.map((block) => [block.id, block]));
  childIdsByParentId = buildNotesChildIdsByParent(blocks);
}

async function loadAllChildrenForVisibleTree(): Promise<void> {
  let queue = Object.values(blocksById).filter(
    (block) => block.has_children && block.type !== "child_page",
  );
  const visited = new Set<string>();
  while (queue.length > 0) {
    const block = queue.shift();
    if (!block || visited.has(block.id)) continue;
    visited.add(block.id);
    let cursor: string | null = null;
    const loadedChildren: NotesBlock[] = [];
    do {
      const page = await getNotesBlockChildren(block.id, cursor, CHILDREN_PAGE_SIZE);
      loadedChildren.push(...page.results);
      cursor = page.next_cursor;
    } while (cursor);
    if (loadedChildren.length === 0) continue;
    blocksById = {
      ...blocksById,
      ...Object.fromEntries(loadedChildren.map((child) => [child.id, child])),
    };
    childIdsByParentId = {
      ...childIdsByParentId,
      [block.id]: loadedChildren.map((child) => child.id),
    };
    queue = [
      ...queue,
      ...loadedChildren.filter((child) => child.has_children && child.type !== "child_page"),
    ];
  }
}

async function reloadPages(): Promise<void> {
  replacePages(await listNotesPages());
}

async function reloadArchivedPages(): Promise<void> {
  const requestId = ++archiveRequestId;
  archiveLoading = true;
  archiveError = null;
  try {
    const nextPages = await listArchivedNotesPages();
    if (requestId !== archiveRequestId) return;
    archivedPages = [...nextPages];
    archiveLoaded = true;
  } catch (error) {
    if (requestId !== archiveRequestId) return;
    archiveError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === archiveRequestId) archiveLoading = false;
  }
}

async function reloadTrashedPages(): Promise<void> {
  const requestId = ++trashRequestId;
  trashLoading = true;
  trashError = null;
  try {
    const nextPages = await listTrashedNotesPages();
    if (requestId !== trashRequestId) return;
    trashedPages = [...nextPages];
    trashLoaded = true;
  } catch (error) {
    if (requestId !== trashRequestId) return;
    trashError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === trashRequestId) trashLoading = false;
  }
}

async function loadPageTree(pageId: string): Promise<void> {
  const loaded = await loadNotesPage(pageId);
  setLoadedPageFromLoaded(loaded);
  await loadAllChildrenForVisibleTree();
  await reloadBacklinks(pageId);
  await reloadComments(pageId);
}

async function loadPageTreeForUndo(pageId: string): Promise<void> {
  viewMode = "pages";
  saveSelectedPageId(pageId);
  await loadPageTree(pageId);
  recordRecentPage(pageId);
}

async function reloadBacklinks(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++backlinksRequestId;
  if (!pageId) {
    backlinks = [];
    backlinksError = null;
    backlinksLoading = false;
    return;
  }
  backlinksLoading = true;
  backlinksError = null;
  try {
    const nextBacklinks = await listNotesBacklinks(pageId);
    if (requestId !== backlinksRequestId) return;
    backlinks = [...nextBacklinks];
  } catch (error) {
    if (requestId !== backlinksRequestId) return;
    backlinks = [];
    backlinksError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === backlinksRequestId) backlinksLoading = false;
  }
}

async function reloadComments(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++commentsRequestId;
  if (!pageId) {
    commentThreads = [];
    activeCommentParent = null;
    commentsError = null;
    commentsLoading = false;
    return;
  }
  commentsLoading = true;
  commentsError = null;
  try {
    const nextThreads = await listNotesComments(pageId, commentsIncludeResolved);
    if (requestId !== commentsRequestId) return;
    commentThreads = [...nextThreads];
  } catch (error) {
    if (requestId !== commentsRequestId) return;
    commentThreads = [];
    commentsError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === commentsRequestId) commentsLoading = false;
  }
}

function updateCommentThread(thread: NotesCommentThread): void {
  if (thread.comments.length === 0 || (thread.status === "resolved" && !commentsIncludeResolved)) {
    commentThreads = commentThreads.filter((candidate) => candidate.id !== thread.id);
    return;
  }
  const existingIndex = commentThreads.findIndex((candidate) => candidate.id === thread.id);
  if (existingIndex === -1) {
    commentThreads = [...commentThreads, thread];
    return;
  }
  commentThreads = commentThreads.map((candidate) => (candidate.id === thread.id ? thread : candidate));
}

function commentParentForSelectedPage(): NotesCommentParent | null {
  return selectedPageId ? { type: "page_id", page_id: selectedPageId } : null;
}

function setActiveCommentParent(parent: NotesCommentParent | null): void {
  activeCommentParent = parent;
}

async function startBlockComment(blockId: string): Promise<void> {
  if (!blocksById[blockId]) return;
  await flushBlockSave(blockId);
  activeCommentParent = { type: "block_id", block_id: blockId };
  requestBlockFocus(blockId);
}

async function createComment(
  text: string,
  parent: NotesCommentParent | null = activeCommentParent ?? commentParentForSelectedPage(),
): Promise<void> {
  const content = text.trim();
  if (!content || !parent) return;
  if (parent.type === "block_id") {
    await flushBlockSave(parent.block_id);
  }
  const thread = await createNotesComment({
    id: crypto.randomUUID(),
    parent,
    rich_text: [createRichText(content)],
  });
  updateCommentThread(thread);
  activeCommentParent = null;
}

async function replyToCommentThread(discussionId: string, text: string): Promise<void> {
  const content = text.trim();
  if (!content) return;
  const thread = await createNotesComment({
    id: crypto.randomUUID(),
    discussion_id: discussionId,
    rich_text: [createRichText(content)],
  });
  updateCommentThread(thread);
}

async function updateComment(commentId: string, text: string): Promise<void> {
  const content = text.trim();
  if (!content) return;
  const thread = await updateNotesComment(commentId, {
    rich_text: [createRichText(content)],
  });
  updateCommentThread(thread);
}

async function deleteComment(commentId: string): Promise<void> {
  const thread = await deleteNotesComment(commentId);
  updateCommentThread(thread);
}

async function setCommentThreadResolved(
  discussionId: string,
  resolved: boolean,
): Promise<void> {
  const thread = await resolveNotesCommentThread(discussionId, resolved);
  updateCommentThread(thread);
}

async function setCommentsIncludeResolved(includeResolved: boolean): Promise<void> {
  commentsIncludeResolved = includeResolved;
  await reloadComments();
}

async function search(query: string, pageSize = 20): Promise<void> {
  const trimmed = query.trim();
  const requestId = ++searchRequestId;
  if (!trimmed) {
    searchResults = [];
    searchError = null;
    searchLoading = false;
    return;
  }
  searchLoading = true;
  searchError = null;
  try {
    const results = await searchNotes(trimmed, pageSize);
    if (requestId !== searchRequestId) return;
    searchResults = [...results];
  } catch (error) {
    if (requestId !== searchRequestId) return;
    searchResults = [];
    searchError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === searchRequestId) searchLoading = false;
  }
}

async function load(): Promise<void> {
  const requestId = ++loadRequestId;
  loading = true;
  loadError = null;
  try {
    await reloadPages();
    if (requestId !== loadRequestId) return;
    const nextSelected = selectedPageId && pages.some((page) => page.id === selectedPageId)
      ? selectedPageId
      : pages[0]?.id ?? null;
    saveSelectedPageId(nextSelected);
    if (nextSelected) {
      await loadPageTree(nextSelected);
      await undoController.hydrate(nextSelected);
    } else {
      loadedPage = null;
      backlinks = [];
      backlinksError = null;
      commentThreads = [];
      activeCommentParent = null;
      commentsError = null;
      blocksById = {};
      childIdsByParentId = {};
      await undoController.hydrate(null);
    }
    loaded = true;
  } catch (error) {
    if (requestId !== loadRequestId) return;
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === loadRequestId) loading = false;
  }
}

async function ensureLoaded(): Promise<void> {
  if (loaded || loading) return;
  await load();
}

async function selectPage(pageId: string | null): Promise<void> {
  if (selectedPageId === pageId && (!pageId || loadedPage?.id === pageId)) return;
  saveSelectedPageId(pageId);
  if (!pageId) {
    loadedPage = null;
    backlinks = [];
    backlinksError = null;
    commentThreads = [];
    activeCommentParent = null;
    commentsError = null;
    blocksById = {};
    childIdsByParentId = {};
    await undoController.hydrate(null);
    return;
  }
  loading = true;
  loadError = null;
  try {
    await loadPageTree(pageId);
    recordRecentPage(pageId);
    await undoController.hydrate(pageId);
  } catch (error) {
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    loading = false;
  }
}

async function createPage(title: string): Promise<void> {
  await createPageWithParent(title, { type: "workspace", workspace: true });
}

async function createSubpage(parentPageId: string, title: string): Promise<void> {
  setSidebarPageCollapsed(parentPageId, false);
  await createPageWithParent(title, { type: "page_id", page_id: parentPageId });
}

async function createPageWithParent(title: string, parent: NotesParent): Promise<void> {
  const pageId = crypto.randomUUID();
  const firstBlockId = crypto.randomUUID();
  const loaded = await createNotesPage({
    id: pageId,
    title,
    parent,
    first_block_id: firstBlockId,
    after_block_id: null,
  });
  await reloadPages();
  if (!pages.some((page) => page.id === loaded.page.id)) {
    pages = [loaded.page, ...pages];
  }
  viewMode = "pages";
  saveSelectedPageId(loaded.page.id);
  recordRecentPage(loaded.page.id);
  setLoadedPageFromLoaded(loaded);
  await reloadBacklinks(loaded.page.id);
  await reloadComments(loaded.page.id);
  await undoController.hydrate(loaded.page.id);
  requestBlockFocus(firstBlockId);
}

function setSidebarPageCollapsed(pageId: string, collapsed: boolean): void {
  const next = collapsed
    ? [...new Set([...sidebarCollapsedPageIds, pageId])]
    : sidebarCollapsedPageIds.filter((candidate) => candidate !== pageId);
  sidebarCollapsedPageIds = next;
  saveNotesSidebarCollapsedPageIds(next);
}

function setPageFavorited(pageId: string, favorited: boolean): void {
  const next = setNotesPageFavoriteId(favoritePageIds, pageId, favorited);
  favoritePageIds = next;
  saveNotesFavoritePageIds(next);
}

async function createChildPageFromBlock(blockId: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block || block.type === "child_page") return;
  await flushBlockSave(blockId);
  const firstBlockId = crypto.randomUUID();
  const loaded = await createNotesChildPageFromBlock(blockId, {
    first_block_id: firstBlockId,
    title: blockPlainText(block).trim(),
  });
  await reloadPages();
  if (!pages.some((page) => page.id === loaded.page.id)) {
    pages = [loaded.page, ...pages];
  }
  viewMode = "pages";
  saveSelectedPageId(loaded.page.id);
  recordRecentPage(loaded.page.id);
  setLoadedPageFromLoaded(loaded);
  await reloadBacklinks(loaded.page.id);
  await reloadComments(loaded.page.id);
  await undoController.hydrate(loaded.page.id);
  requestBlockFocus(loaded.blocks.results[0]?.id ?? firstBlockId);
}

async function renamePage(pageId: string, title: string): Promise<void> {
  const trimmedTitle = title.trim();
  const page = await updateNotesPage(pageId, { title: trimmedTitle });
  pages = pages.map((item) => (item.id === page.id ? page : item));
  if (loadedPage?.id === page.id) loadedPage = page;
}

async function duplicatePage(pageId: string, title: string): Promise<void> {
  await flushPendingBlockSaves();
  const loaded = await duplicateNotesPage(pageId, { title });
  await reloadPages();
  if (!pages.some((page) => page.id === loaded.page.id)) {
    pages = [loaded.page, ...pages];
  }
  if (loaded.page.parent.type === "page_id") {
    setSidebarPageCollapsed(loaded.page.parent.page_id, false);
  }
  viewMode = "pages";
  saveSelectedPageId(loaded.page.id);
  recordRecentPage(loaded.page.id);
  setLoadedPageFromLoaded(loaded);
  await reloadBacklinks(loaded.page.id);
  await reloadComments(loaded.page.id);
  await undoController.hydrate(loaded.page.id);
  requestBlockFocus(loaded.blocks.results[0]?.id ?? null);
}

async function movePage(pageId: string, parent: NotesParent): Promise<void> {
  await flushPendingBlockSaves();
  const loaded = await moveNotesPage(pageId, { parent });
  await reloadPages();
  if (!pages.some((page) => page.id === loaded.page.id)) {
    pages = [loaded.page, ...pages];
  }
  if (loaded.page.parent.type === "page_id") {
    setSidebarPageCollapsed(loaded.page.parent.page_id, false);
  }
  viewMode = "pages";
  saveSelectedPageId(loaded.page.id);
  recordRecentPage(loaded.page.id);
  setLoadedPageFromLoaded(loaded);
  await loadAllChildrenForVisibleTree();
  await reloadBacklinks(loaded.page.id);
  await reloadComments(loaded.page.id);
  await undoController.hydrate(loaded.page.id);
  requestBlockFocus(loaded.blocks.results[0]?.id ?? null);
}

async function updatePageIcon(pageId: string, icon: NotesPageIcon | null): Promise<void> {
  const page = await updateNotesPage(pageId, { icon });
  pages = pages.map((item) => (item.id === page.id ? page : item));
  if (loadedPage?.id === page.id) loadedPage = page;
}

async function updatePageCover(pageId: string, cover: NotesPageCover | null): Promise<void> {
  const page = await updateNotesPage(pageId, { cover });
  pages = pages.map((item) => (item.id === page.id ? page : item));
  if (loadedPage?.id === page.id) loadedPage = page;
}

async function trashPage(pageId: string): Promise<void> {
  const trashedPage = await trashNotesPage(pageId, true);
  if (trashLoaded) {
    trashedPages = [trashedPage, ...trashedPages.filter((page) => page.id !== pageId)];
  }
  if (archiveLoaded) {
    archivedPages = archivedPages.filter((page) => page.id !== pageId);
  }
  const nextSelected = nextSelectedNotesPageId(pages, pageId);
  pages = pages.filter((page) => page.id !== pageId);
  await selectPage(nextSelected);
}

async function archivePage(pageId: string): Promise<void> {
  const archivedPage = await archiveNotesPage(pageId, true);
  if (archiveLoaded) {
    archivedPages = [archivedPage, ...archivedPages.filter((page) => page.id !== pageId)];
  }
  const nextSelected = nextSelectedNotesPageId(pages, pageId);
  pages = pages.filter((page) => page.id !== pageId);
  await selectPage(nextSelected);
}

async function unarchivePage(pageId: string): Promise<void> {
  const restoredPage = await archiveNotesPage(pageId, false);
  archivedPages = archivedPages.filter((page) => page.id !== pageId);
  await reloadPages();
  if (!pages.some((page) => page.id === restoredPage.id)) {
    pages = [restoredPage, ...pages];
  }
  viewMode = "pages";
  saveSelectedPageId(restoredPage.id);
  await loadPageTree(restoredPage.id);
  recordRecentPage(restoredPage.id);
  await undoController.hydrate(restoredPage.id);
  requestBlockFocus(null);
}

async function restorePage(pageId: string): Promise<void> {
  const restoredPage = await trashNotesPage(pageId, false);
  trashedPages = trashedPages.filter((page) => page.id !== pageId);
  await reloadPages();
  if (!pages.some((page) => page.id === restoredPage.id)) {
    pages = [restoredPage, ...pages];
  }
  viewMode = "pages";
  saveSelectedPageId(restoredPage.id);
  await loadPageTree(restoredPage.id);
  recordRecentPage(restoredPage.id);
  await undoController.hydrate(restoredPage.id);
  requestBlockFocus(null);
}

async function permanentlyDeletePage(pageId: string): Promise<void> {
  await flushPendingBlockSaves();
  const deletedPageIds = await permanentlyDeleteNotesPage(pageId);
  const deletedPageIdSet = new Set(deletedPageIds);
  pages = pages.filter((page) => !deletedPageIdSet.has(page.id));
  archivedPages = archivedPages.filter((page) => !deletedPageIdSet.has(page.id));
  trashedPages = trashedPages.filter((page) => !deletedPageIdSet.has(page.id));
  const nextFavoritePageIds = favoritePageIds.filter((id) => !deletedPageIdSet.has(id));
  const nextRecentPageIds = recentPageIds.filter((id) => !deletedPageIdSet.has(id));
  favoritePageIds = nextFavoritePageIds;
  recentPageIds = nextRecentPageIds;
  saveNotesFavoritePageIds(nextFavoritePageIds);
  saveNotesRecentPageIds(nextRecentPageIds);
  if (trashLoaded) {
    await reloadTrashedPages();
  }
  await reloadPages();
  if (selectedPageId && deletedPageIdSet.has(selectedPageId)) {
    await selectPage(pages[0]?.id ?? null);
  }
}

async function openArchive(): Promise<void> {
  viewMode = "archive";
  if (!archiveLoaded && !archiveLoading) {
    await reloadArchivedPages();
  }
}

function closeArchive(): void {
  viewMode = "pages";
}

async function openTrash(): Promise<void> {
  viewMode = "trash";
  if (!trashLoaded && !trashLoading) {
    await reloadTrashedPages();
  }
}

function closeTrash(): void {
  viewMode = "pages";
}

function flatBlockItems(): NotesBlockTreeItem[] {
  return flatNotesBlockItems(blockTreeSnapshot());
}

function flatBlockItemsForBlockContext(blockId: string): NotesBlockTreeItem[] {
  return flatNotesBlockItemsForContext(blockTreeSnapshot(), blockId);
}

function blockById(blockId: string): NotesBlock | undefined {
  return blocksById[blockId];
}

function tableRowsForBlock(blockId: string): NotesTableRowBlock[] {
  return notesTableRowsForBlock(blockTreeSnapshot(), blockId);
}

function columnItemsForBlock(blockId: string): NotesColumnBlockItems[] {
  return notesColumnItemsForBlock(blockTreeSnapshot(), blockId);
}

function tabItemsForBlock(blockId: string): NotesTabBlockItems[] {
  return notesTabItemsForBlock(blockTreeSnapshot(), blockId);
}

function previousBlockType(blockId: string): NotesBlockType | null {
  return previousNotesBlockType(blockTreeSnapshot(), blockId);
}

const {
  localApplyBlockUpdate,
  saveBlockNow,
  scheduleBlockSave,
  flushBlockSave,
  flushPendingBlockSaves,
} = createNotesBlockPersistence({
  readBlock: (blockId) => blocksById[blockId],
  replaceBlock,
  readSelectedPageId: () => selectedPageId,
  loadPageTree,
  setLoadError: (message) => {
    loadError = message;
  },
  debounceMs: BLOCK_SAVE_DEBOUNCE_MS,
});

const undoController = createNotesUndoController({
  readSelectedPageId: () => selectedPageId,
  readTreeState: treeState,
  loadPageTreeForUndo,
  requestBlockFocus,
  flushPendingBlockSaves,
  setLoadError: (message) => {
    loadError = message;
  },
});

const blockActions = createNotesBlockActions({
  readSelectedPageId: () => selectedPageId,
  readBlocksById: () => blocksById,
  readChildIdsByParentId: () => childIdsByParentId,
  treeState,
  blockById,
  flatBlockItemsForBlockContext,
  tableRowsForBlock,
  columnItemsForBlock,
  tabItemsForBlock,
  setSidebarPageCollapsed,
  requestBlockFocus,
  createChildPageFromBlock,
  loadPageTree,
  reloadPages,
  reloadBacklinks,
  localApplyBlockUpdate,
  saveBlockNow,
  scheduleBlockSave,
  flushBlockSave,
  flushPendingBlockSaves,
  createUndoSnapshot: undoController.snapshot,
  recordUndo: undoController.record,
});

const {
  updateBlockText, updateBlockRichText,
  insertPageMention,
  insertDateMention,
  insertInlineEquation,
  updateBlockTextLink,
  updateBlockTextAnnotations,
  updateBookmark,
  updateEmbedUrl,
  updateLinkPreviewUrl,
  updateEquationExpression,
  updateMedia,
  updateTableCell,
  convertBlock,
  toggleTodo,
  updateCodeLanguage,
  updateBlockColor,
  updateToggleOpen,
  convertBlockToToggleHeading,
  createSiblingAfter,
  pastePlainTextIntoBlock,
  pasteRichHtmlIntoBlock,
  deleteBlock,
  mergeBlockWithPrevious,
  nestBlock,
  outdentBlock,
  moveBlockUp,
  moveBlockDown,
  dropBlockWithinSiblings,
  moveBlockToPage,
  duplicateBlock,
  useTemplateBlock,
  useButtonBlock,
} = blockActions;

function isOnlyBlock(blockId: string): boolean {
  return isOnlyNotesBlockInContext(blockTreeSnapshot(), blockId);
}

function focusBlock(blockId: string): void {
  requestBlockFocus(blockId);
}

async function undoNotesEdit(): Promise<boolean> {
  try {
    return await undoController.undo();
  } catch (error) {
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  }
}

async function redoNotesEdit(): Promise<boolean> {
  try {
    return await undoController.redo();
  } catch (error) {
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  }
}

async function openBlockLink(target: NotesBlockLinkTarget): Promise<boolean> {
  return openNotesLink(target);
}

async function openNotesLink(target: NotesPageLinkTarget): Promise<boolean> {
  viewMode = "pages";
  await ensureLoaded();
  if (!pages.some((page) => page.id === target.pageId)) {
    await reloadPages();
  }
  if (!pages.some((page) => page.id === target.pageId)) return false;
  if (loadedPage?.id !== target.pageId) {
    await selectPage(target.pageId);
  }
  if (!target.blockId) return true;
  if (!blocksById[target.blockId]) return false;
  requestBlockFocus(target.blockId);
  return true;
}

export function getNotes() {
  return {
    get pages(): NotesPage[] {
      return pages;
    },
    get archivedPages(): NotesPage[] {
      return archivedPages;
    },
    get workspacePages(): NotesPage[] {
      return pages.filter((page) => page.parent.type === "workspace");
    },
    get favoritePageIds(): readonly string[] {
      return favoritePageIds;
    },
    get recentPageIds(): readonly string[] {
      return recentPageIds;
    },
    get sidebarCollapsedPageIds(): readonly string[] {
      return sidebarCollapsedPageIds;
    },
    get trashedPages(): NotesPage[] {
      return trashedPages;
    },
    get selectedPageId(): string | null {
      return selectedPageId;
    },
    get loadedPage(): NotesPage | null {
      return loadedPage;
    },
    get backlinks(): NotesBacklink[] {
      return backlinks;
    },
    get backlinksLoading(): boolean {
      return backlinksLoading;
    },
    get backlinksError(): string | null {
      return backlinksError;
    },
    get commentThreads(): NotesCommentThread[] {
      return commentThreads;
    },
    get commentsLoading(): boolean {
      return commentsLoading;
    },
    get commentsError(): string | null {
      return commentsError;
    },
    get commentsIncludeResolved(): boolean {
      return commentsIncludeResolved;
    },
    get activeCommentParent(): NotesCommentParent | null {
      return activeCommentParent;
    },
    get searchResults(): NotesSearchResult[] {
      return searchResults;
    },
    get searchLoading(): boolean {
      return searchLoading;
    },
    get searchError(): string | null {
      return searchError;
    },
    get blocksById(): Record<string, NotesBlock> {
      return blocksById;
    },
    get childIdsByParentId(): Record<string, string[]> {
      return childIdsByParentId;
    },
    get flatBlocks(): NotesBlockTreeItem[] {
      return flatBlockItems();
    },
    get loaded(): boolean {
      return loaded;
    },
    get loading(): boolean {
      return loading;
    },
    get loadError(): string | null {
      return loadError;
    },
    get viewMode(): NotesViewMode {
      return viewMode;
    },
    get archiveLoaded(): boolean {
      return archiveLoaded;
    },
    get archiveLoading(): boolean {
      return archiveLoading;
    },
    get archiveError(): string | null {
      return archiveError;
    },
    get trashLoaded(): boolean {
      return trashLoaded;
    },
    get trashLoading(): boolean {
      return trashLoading;
    },
    get trashError(): string | null {
      return trashError;
    },
    get focusBlockId(): string | null {
      return focusRequest.blockId;
    },
    get focusRequestId(): number {
      return focusRequest.requestId;
    },
    get canUndoNotesEdit(): boolean {
      return undoController.canUndo();
    },
    get canRedoNotesEdit(): boolean {
      return undoController.canRedo();
    },
    load,
    ensureLoaded,
    selectPage,
    createPage,
    createSubpage,
    createChildPageFromBlock,
    renamePage,
    duplicatePage,
    movePage,
    archivePage,
    unarchivePage,
    trashPage,
    restorePage,
    permanentlyDeletePage,
    openArchive,
    closeArchive,
    reloadArchivedPages,
    openTrash,
    closeTrash,
    reloadTrashedPages,
    reloadBacklinks,
    reloadComments,
    setCommentsIncludeResolved,
    setActiveCommentParent,
    startBlockComment,
    createComment,
    replyToCommentThread,
    updateComment,
    deleteComment,
    setCommentThreadResolved,
    search,
    notesCommentParentKey,
    blockById,
    tableRowsForBlock,
    columnItemsForBlock,
    tabItemsForBlock,
    parentIdForBlock,
    previousBlockType,
    isOnlyBlock,
    focusBlock,
    setPageFavorited,
    setSidebarPageCollapsed,
    updatePageIcon,
    updatePageCover,
    openBlockLink,
    openNotesLink,
    undoNotesEdit,
    redoNotesEdit,
    updateBlockText, updateBlockRichText,
    insertPageMention,
    insertDateMention,
    insertInlineEquation,
    updateBlockTextLink,
    updateBlockTextAnnotations,
    flushBlockSave,
    convertBlock,
    toggleTodo,
    updateCodeLanguage,
    updateBlockColor,
    updateToggleOpen,
    convertBlockToToggleHeading,
    createSiblingAfter,
    pastePlainTextIntoBlock,
    pasteRichHtmlIntoBlock,
    deleteBlock,
    mergeBlockWithPrevious,
    nestBlock,
    outdentBlock,
    moveBlockUp,
    moveBlockDown,
    dropBlockWithinSiblings,
    moveBlockToPage,
    duplicateBlock,
    useTemplateBlock,
    useButtonBlock,
    blockPlainText,
    updateBookmark,
    updateLinkPreviewUrl,
    updateEmbedUrl,
    updateEquationExpression,
    updateMedia,
    updateTableCell,
  };
}
