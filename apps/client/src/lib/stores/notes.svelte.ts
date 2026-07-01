import {
  appendNotesBlockChildren,
  archiveNotesPage,
  createNotesChildPageFromBlock,
  createNotesComment,
  createNotesPage,
  deleteNotesComment,
  duplicateNotesPage,
  duplicateNotesBlock,
  getNotesBlockChildren,
  listNotesBacklinks,
  listNotesComments,
  listNotesPages,
  listArchivedNotesPages,
  listTrashedNotesPages,
  loadNotesPage,
  moveNotesPage,
  moveNotesBlock,
  permanentlyDeleteNotesPage,
  resolveNotesCommentThread,
  searchNotes,
  trashNotesBlock,
  trashNotesPage,
  updateNotesComment,
  updateNotesBlock,
  updateNotesPage,
} from "$lib/api/notes";
import {
  collectLoadedBlockSubtreeIds,
  createDuplicateBlockRequest,
} from "$lib/notes/block-duplicate";
import { planNotesPlainTextPaste } from "$lib/notes/block-clipboard";
import {
  applyBlockUpdate,
  blockConvertedToType,
  blockPlainText,
  blockWithBookmark,
  blockWithCodeLanguage,
  blockWithDateMention,
  blockWithEmbedUrl,
  blockWithEquationExpression,
  blockWithHeadingToggleable,
  blockWithHeadingToggleOpen,
  blockWithInlineEquation,
  blockWithLinkPreviewUrl,
  blockWithMedia,
  blockWithPageMention,
  blockWithTextAnnotations,
  blockWithTableCell,
  blockWithTextLink,
  blockWithText,
  blockWithToggleOpen,
  blockWithTodoChecked,
  createBlockUpdate,
  createBlockWrite,
  createColumnPayload,
  createEmptyTableRowPayload,
  createRichText,
  DEFAULT_TABLE_ROW_COUNT,
  DEFAULT_TABLE_WIDTH,
  type NotesHeadingBlockType,
} from "$lib/notes/block-factory";
import type { NotesRichTextAnnotationPatch } from "$lib/notes/rich-text";
import { notesCommentParentKey } from "$lib/notes/comments";
import { blockWithColor } from "$lib/notes/block-color";
import type { NotesBlockLinkTarget, NotesPageLinkTarget } from "$lib/notes/block-link";
import {
  buildNotesChildIdsByParent,
  flattenNotesBlockChildren,
  flattenNotesBlockTree,
  parentIdForBlock,
  planDeleteBlock,
  planDropBlockWithinSiblings,
  planMergeWithPrevious,
  planMoveBlockWithinSiblings,
  planNestBlock,
  planOutdentBlock,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import {
  nextSelectedNotesPageId,
  notesSelectedPageConfigKey,
  parseStoredNotesPageId,
} from "$lib/notes/page-selection";
import {
  notesFavoritePageIdsConfigKey,
  notesRecentPageIdsConfigKey,
  parseStoredNotesPageIdList,
  recordRecentNotesPageId,
  setNotesPageFavoriteId,
} from "$lib/notes/page-navigation";
import {
  notesSidebarCollapsedPageIdsConfigKey,
  parseStoredNotesSidebarCollapsedPageIds,
} from "$lib/notes/page-tree";
import type {
  NotesBlock,
  NotesBacklink,
  NotesBlockWrite,
  NotesColumnBlockItems,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesBlockUpdate,
  NotesButtonInsertPosition,
  NotesColor,
  NotesParagraphBlock,
  NotesDateMentionValue,
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
import { getConfigKey, setConfigKey } from "$lib/vault/config";

interface PendingBlockSave {
  timer: ReturnType<typeof setTimeout>;
  update: NotesBlockUpdate;
}

type NotesViewMode = "pages" | "archive" | "trash";

const BLOCK_SAVE_DEBOUNCE_MS = 350;
const CHILDREN_PAGE_SIZE = 100;
const selectedPageConfigKey = notesSelectedPageConfigKey();
const favoritePageIdsConfigKey = notesFavoritePageIdsConfigKey();
const recentPageIdsConfigKey = notesRecentPageIdsConfigKey();
const sidebarCollapsedPageIdsConfigKey = notesSidebarCollapsedPageIdsConfigKey();

let pages = $state<NotesPage[]>([]);
let archivedPages = $state<NotesPage[]>([]);
let trashedPages = $state<NotesPage[]>([]);
let selectedPageId = $state<string | null>(
  parseStoredNotesPageId(getConfigKey<unknown>(selectedPageConfigKey, undefined)),
);
let favoritePageIds = $state<string[]>(
  parseStoredNotesPageIdList(getConfigKey<unknown>(favoritePageIdsConfigKey, undefined)),
);
let recentPageIds = $state<string[]>(
  parseStoredNotesPageIdList(getConfigKey<unknown>(recentPageIdsConfigKey, undefined)),
);
let sidebarCollapsedPageIds = $state<string[]>(
  parseStoredNotesSidebarCollapsedPageIds(
    getConfigKey<unknown>(sidebarCollapsedPageIdsConfigKey, undefined),
  ),
);
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
let focusBlockId = $state<string | null>(null);
let focusRequestId = $state(0);
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
const pendingBlockSaves = new Map<string, PendingBlockSave>();

function treeState(): NotesTreeState {
  return { blocksById, childIdsByParentId };
}

function saveSelectedPageId(pageId: string | null): void {
  selectedPageId = pageId;
  setConfigKey(selectedPageConfigKey, pageId ?? undefined);
}

function recordRecentPage(pageId: string): void {
  const next = recordRecentNotesPageId(recentPageIds, pageId);
  recentPageIds = next;
  setConfigKey(recentPageIdsConfigKey, next.length > 0 ? next : undefined);
}

function requestBlockFocus(blockId: string | null): void {
  focusBlockId = blockId;
  focusRequestId += 1;
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
    } else {
      loadedPage = null;
      backlinks = [];
      backlinksError = null;
      commentThreads = [];
      activeCommentParent = null;
      commentsError = null;
      blocksById = {};
      childIdsByParentId = {};
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
    return;
  }
  loading = true;
  loadError = null;
  try {
    await loadPageTree(pageId);
    recordRecentPage(pageId);
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
  requestBlockFocus(firstBlockId);
}

function setSidebarPageCollapsed(pageId: string, collapsed: boolean): void {
  const next = collapsed
    ? [...new Set([...sidebarCollapsedPageIds, pageId])]
    : sidebarCollapsedPageIds.filter((candidate) => candidate !== pageId);
  sidebarCollapsedPageIds = next;
  setConfigKey(sidebarCollapsedPageIdsConfigKey, next.length > 0 ? next : undefined);
}

function setPageFavorited(pageId: string, favorited: boolean): void {
  const next = setNotesPageFavoriteId(favoritePageIds, pageId, favorited);
  favoritePageIds = next;
  setConfigKey(favoritePageIdsConfigKey, next.length > 0 ? next : undefined);
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
  setConfigKey(favoritePageIdsConfigKey, nextFavoritePageIds.length > 0 ? nextFavoritePageIds : undefined);
  setConfigKey(recentPageIdsConfigKey, nextRecentPageIds.length > 0 ? nextRecentPageIds : undefined);
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
  return selectedPageId ? flattenNotesBlockTree(treeState(), selectedPageId) : [];
}

function closestAncestorBlockOfType(blockId: string, type: NotesBlockType): NotesBlock | null {
  let parent = blocksById[blockId]?.parent;
  while (parent?.type === "block_id") {
    const parentBlock = blocksById[parent.block_id];
    if (!parentBlock) return null;
    if (parentBlock.type === type) return parentBlock;
    parent = parentBlock.parent;
  }
  return null;
}

function tabLabelAncestorForBlock(blockId: string): NotesParagraphBlock | null {
  let parent = blocksById[blockId]?.parent;
  while (parent?.type === "block_id") {
    const parentBlock = blocksById[parent.block_id];
    if (!parentBlock) return null;
    if (
      parentBlock.type === "paragraph"
      && parentBlock.parent.type === "block_id"
      && blocksById[parentBlock.parent.block_id]?.type === "tab"
    ) {
      return parentBlock;
    }
    parent = parentBlock.parent;
  }
  return null;
}

function flatBlockItemsForBlockContext(blockId: string): NotesBlockTreeItem[] {
  const column = closestAncestorBlockOfType(blockId, "column");
  if (column) return flattenNotesBlockChildren(treeState(), column.id, 0);
  const tabLabel = tabLabelAncestorForBlock(blockId);
  if (tabLabel) return flattenNotesBlockChildren(treeState(), tabLabel.id, 0);
  return flatBlockItems();
}

function blockById(blockId: string): NotesBlock | undefined {
  return blocksById[blockId];
}

function tableRowsForBlock(blockId: string): NotesTableRowBlock[] {
  return (childIdsByParentId[blockId] ?? [])
    .map((childId) => blocksById[childId])
    .filter((block): block is NotesTableRowBlock => block?.type === "table_row");
}

function columnItemsForBlock(blockId: string): NotesColumnBlockItems[] {
  return (childIdsByParentId[blockId] ?? [])
    .map((childId) => blocksById[childId])
    .filter((block): block is NotesColumnBlockItems["column"] => block?.type === "column")
    .map((column) => ({
      column,
      items: flattenNotesBlockChildren(treeState(), column.id, 0),
    }));
}

function tabItemsForBlock(blockId: string): NotesTabBlockItems[] {
  return (childIdsByParentId[blockId] ?? [])
    .map((childId) => blocksById[childId])
    .filter((block): block is NotesParagraphBlock => block?.type === "paragraph")
    .map((label) => ({
      label,
      items: flattenNotesBlockChildren(treeState(), label.id, 0),
    }));
}

function previousBlockType(blockId: string): NotesBlockType | null {
  const items = flatBlockItemsForBlockContext(blockId);
  const item = items.find((candidate) => candidate.block.id === blockId);
  if (!item?.previousVisibleId) return null;
  return blocksById[item.previousVisibleId]?.type ?? null;
}

function localApplyBlockUpdate(blockId: string, update: NotesBlockUpdate): void {
  const block = blocksById[blockId];
  if (!block) return;
  replaceBlock(applyBlockUpdate(block, update));
}

async function saveBlockNow(blockId: string, update: NotesBlockUpdate): Promise<void> {
  const saved = await updateNotesBlock(blockId, update);
  replaceBlock(saved);
}

function scheduleBlockSave(blockId: string, update: NotesBlockUpdate): void {
  const pending = pendingBlockSaves.get(blockId);
  if (pending) clearTimeout(pending.timer);
  const timer = setTimeout(() => {
    pendingBlockSaves.delete(blockId);
    saveBlockNow(blockId, update).catch((error) => {
      loadError = error instanceof Error ? error.message : String(error);
      if (selectedPageId) void loadPageTree(selectedPageId);
    });
  }, BLOCK_SAVE_DEBOUNCE_MS);
  pendingBlockSaves.set(blockId, { timer, update });
}

async function flushBlockSave(blockId: string): Promise<void> {
  const pending = pendingBlockSaves.get(blockId);
  if (!pending) return;
  clearTimeout(pending.timer);
  pendingBlockSaves.delete(blockId);
  await saveBlockNow(blockId, pending.update);
}

async function flushPendingBlockSaves(): Promise<void> {
  const blockIds = [...pendingBlockSaves.keys()];
  await Promise.all(blockIds.map((blockId) => flushBlockSave(blockId)));
}

async function updateBlockText(blockId: string, text: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  const update = blockWithText(block, text);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function insertPageMention(
  blockId: string,
  start: number,
  end: number,
  pageId: string,
  title: string,
  href: string | null,
): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await flushBlockSave(blockId);
  const update = blockWithPageMention(block, start, end, pageId, title, href);
  localApplyBlockUpdate(blockId, update);
  await saveBlockNow(blockId, update);
  await reloadBacklinks();
}

async function insertDateMention(
  blockId: string,
  start: number,
  end: number,
  date: NotesDateMentionValue,
  title: string,
): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await flushBlockSave(blockId);
  const update = blockWithDateMention(block, start, end, date, title);
  localApplyBlockUpdate(blockId, update);
  await saveBlockNow(blockId, update);
}

async function insertInlineEquation(
  blockId: string,
  start: number,
  end: number,
  expression: string,
): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await flushBlockSave(blockId);
  const update = blockWithInlineEquation(block, start, end, expression);
  localApplyBlockUpdate(blockId, update);
  await saveBlockNow(blockId, update);
}

async function updateBlockTextLink(
  blockId: string,
  start: number,
  end: number,
  url: string | null,
): Promise<void> {
  await flushBlockSave(blockId);
  const block = blocksById[blockId];
  if (!block) return;
  const update = blockWithTextLink(block, start, end, url);
  localApplyBlockUpdate(blockId, update);
  await saveBlockNow(blockId, update);
  await reloadBacklinks();
}

async function updateBlockTextAnnotations(
  blockId: string,
  start: number,
  end: number,
  patch: NotesRichTextAnnotationPatch,
): Promise<void> {
  await flushBlockSave(blockId);
  const block = blocksById[blockId];
  if (!block) return;
  const update = blockWithTextAnnotations(block, start, end, patch);
  localApplyBlockUpdate(blockId, update);
  await saveBlockNow(blockId, update);
}

async function updateBookmark(blockId: string, url: string, caption: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block || block.type !== "bookmark") return;
  const update = blockWithBookmark(block, url, caption);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function updateEmbedUrl(blockId: string, url: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block || block.type !== "embed") return;
  const update = blockWithEmbedUrl(block, url);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function updateLinkPreviewUrl(blockId: string, url: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block || block.type !== "link_preview") return;
  const update = blockWithLinkPreviewUrl(block, url);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function updateEquationExpression(blockId: string, expression: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block || block.type !== "equation") return;
  const update = blockWithEquationExpression(block, expression);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function updateMedia(
  blockId: string,
  url: string,
  caption: string,
  name?: string,
): Promise<void> {
  const block = blocksById[blockId];
  if (
    !block
    || !["image", "video", "audio", "file", "pdf"].includes(block.type)
  ) {
    return;
  }
  const update = blockWithMedia(block, url, caption, name);
  localApplyBlockUpdate(blockId, update);
  scheduleBlockSave(blockId, update);
}

async function updateTableCell(
  rowBlockId: string,
  columnIndex: number,
  text: string,
): Promise<void> {
  const block = blocksById[rowBlockId];
  if (!block || block.type !== "table_row") return;
  const update = blockWithTableCell(block, columnIndex, text);
  localApplyBlockUpdate(rowBlockId, update);
  scheduleBlockSave(rowBlockId, update);
}

async function replaceBlockWithUpdate(blockId: string, update: NotesBlockUpdate): Promise<void> {
  await flushBlockSave(blockId);
  localApplyBlockUpdate(blockId, update);
  const saved = await updateNotesBlock(blockId, update);
  replaceBlock(saved);
}

async function convertBlock(blockId: string, type: NotesBlockType, clearText = false): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  if (type === "child_page") {
    await createChildPageFromBlock(blockId);
    return;
  }
  if (type === "table") {
    await createTableFromBlock(blockId);
    return;
  }
  if (type === "column_list") {
    await createColumnListFromBlock(blockId);
    return;
  }
  if (type === "tab") {
    await createTabFromBlock(blockId);
    return;
  }
  if (block.type === "child_page") return;
  if (block.type === "table" || block.type === "table_row") return;
  if (block.type === "column_list" || block.type === "column") return;
  if (block.type === "tab") return;
  const update = clearText ? createBlockUpdate(type, "") : blockConvertedToType(block, type);
  await replaceBlockWithUpdate(blockId, update);
  requestBlockFocus(type === "divider" ? null : blockId);
}

async function createTableFromBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  const block = blocksById[blockId];
  if (!block || block.type === "child_page" || block.type === "table_row") return;
  if ((childIdsByParentId[blockId] ?? []).length > 0 && block.type !== "table") return;
  await replaceBlockWithUpdate(blockId, createBlockUpdate("table", ""));
  if (tableRowsForBlock(blockId).length === 0) {
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: blockId },
      after: null,
      children: Array.from({ length: DEFAULT_TABLE_ROW_COUNT }, () => ({
        id: crypto.randomUUID(),
        type: "table_row" as const,
        table_row: createEmptyTableRowPayload(DEFAULT_TABLE_WIDTH),
      })),
    });
  }
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

async function createColumnListFromBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  const block = blocksById[blockId];
  if (
    !block
    || block.type === "child_page"
    || block.type === "table_row"
    || block.type === "column"
  ) {
    return;
  }
  if ((childIdsByParentId[blockId] ?? []).length > 0 && block.type !== "column_list") return;
  await replaceBlockWithUpdate(blockId, createBlockUpdate("column_list", ""));
  if (columnItemsForBlock(blockId).length === 0) {
    const leftColumnId = crypto.randomUUID();
    const rightColumnId = crypto.randomUUID();
    const leftBlockId = crypto.randomUUID();
    const rightBlockId = crypto.randomUUID();
    const columns: NotesBlockWrite[] = [
      {
        id: leftColumnId,
        type: "column",
        column: createColumnPayload(0.5),
      },
      {
        id: rightColumnId,
        type: "column",
        column: createColumnPayload(0.5),
      },
    ];
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: blockId },
      after: null,
      children: columns,
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: leftColumnId },
      after: null,
      children: [createBlockWrite(leftBlockId, "paragraph")],
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: rightColumnId },
      after: null,
      children: [createBlockWrite(rightBlockId, "paragraph")],
    });
    await loadPageTree(selectedPageId);
    requestBlockFocus(leftBlockId);
    return;
  }
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

async function createTabFromBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  const block = blocksById[blockId];
  if (
    !block
    || block.type === "child_page"
    || block.type === "table_row"
    || block.type === "column"
  ) {
    return;
  }
  if ((childIdsByParentId[blockId] ?? []).length > 0 && block.type !== "tab") return;
  const firstLabel = blockPlainText(block).trim() || "Tab 1";
  await replaceBlockWithUpdate(blockId, createBlockUpdate("tab", ""));
  if (tabItemsForBlock(blockId).length === 0) {
    const firstLabelId = crypto.randomUUID();
    const secondLabelId = crypto.randomUUID();
    const firstContentId = crypto.randomUUID();
    const secondContentId = crypto.randomUUID();
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: blockId },
      after: null,
      children: [
        createBlockWrite(firstLabelId, "paragraph", firstLabel),
        createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
      ],
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: firstLabelId },
      after: null,
      children: [createBlockWrite(firstContentId, "paragraph")],
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: secondLabelId },
      after: null,
      children: [createBlockWrite(secondContentId, "paragraph")],
    });
    await loadPageTree(selectedPageId);
    requestBlockFocus(firstContentId);
    return;
  }
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

async function toggleTodo(blockId: string, checked: boolean): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await replaceBlockWithUpdate(blockId, blockWithTodoChecked(block, checked));
}

async function updateCodeLanguage(blockId: string, language: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await replaceBlockWithUpdate(blockId, blockWithCodeLanguage(block, language));
}

async function updateBlockColor(blockId: string, color: NotesColor): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await replaceBlockWithUpdate(blockId, blockWithColor(block, color));
}

async function updateToggleOpen(blockId: string, open: boolean): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  if (block.type === "toggle") {
    await replaceBlockWithUpdate(blockId, blockWithToggleOpen(block, open));
    return;
  }
  if (
    (block.type === "heading_1" && block.heading_1.is_toggleable === true)
    || (block.type === "heading_2" && block.heading_2.is_toggleable === true)
    || (block.type === "heading_3" && block.heading_3.is_toggleable === true)
    || (block.type === "heading_4" && block.heading_4.is_toggleable === true)
  ) {
    await replaceBlockWithUpdate(blockId, blockWithHeadingToggleOpen(block, open));
  }
}

async function convertBlockToToggleHeading(
  blockId: string,
  headingType: NotesHeadingBlockType,
): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  if (block.type === "child_page") return;
  if (block.type === "table" || block.type === "table_row") return;
  if (block.type === "column_list" || block.type === "column") return;
  await replaceBlockWithUpdate(blockId, blockWithHeadingToggleable(block, headingType, true));
  requestBlockFocus(blockId);
}

async function createSiblingAfter(blockId: string, type: NotesBlockType = "paragraph"): Promise<void> {
  const block = blocksById[blockId];
  if (!block || !selectedPageId) return;
  await flushBlockSave(blockId);
  const newBlockId = crypto.randomUUID();
  await appendNotesBlockChildren({
    parent: block.parent,
    after: blockId,
    children: [createBlockWrite(newBlockId, type)],
  });
  if (type === "tab") {
    const firstLabelId = crypto.randomUUID();
    const secondLabelId = crypto.randomUUID();
    const firstContentId = crypto.randomUUID();
    const secondContentId = crypto.randomUUID();
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: newBlockId },
      after: null,
      children: [
        createBlockWrite(firstLabelId, "paragraph", "Tab 1"),
        createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
      ],
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: firstLabelId },
      after: null,
      children: [createBlockWrite(firstContentId, "paragraph")],
    });
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: secondLabelId },
      after: null,
      children: [createBlockWrite(secondContentId, "paragraph")],
    });
    await loadPageTree(selectedPageId);
    requestBlockFocus(firstContentId);
    return;
  }
  await loadPageTree(selectedPageId);
  requestBlockFocus(type === "divider" ? null : newBlockId);
}

async function pastePlainTextIntoBlock(
  blockId: string,
  selectionStart: number,
  selectionEnd: number,
  plainText: string,
): Promise<boolean> {
  const block = blocksById[blockId];
  if (!block || !selectedPageId) return false;
  const plan = planNotesPlainTextPaste({
    currentBlockId: blockId,
    currentBlockType: block.type,
    currentText: blockPlainText(block),
    selectionStart,
    selectionEnd,
    plainText,
    createId: () => crypto.randomUUID(),
  });
  if (!plan) return false;
  await flushBlockSave(blockId);
  localApplyBlockUpdate(blockId, plan.currentUpdate);
  await saveBlockNow(blockId, plan.currentUpdate);
  if (plan.appendedBlocks.length > 0) {
    await appendNotesBlockChildren({
      parent: block.parent,
      after: blockId,
      children: plan.appendedBlocks,
    });
  }
  await loadPageTree(selectedPageId);
  requestBlockFocus(plan.focusBlockId);
  return true;
}

async function deleteBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(blockId);
  const plan = planDeleteBlock(flatBlockItemsForBlockContext(blockId), blockId);
  if (!plan) return;
  if (plan.keepOnlyBlockAsParagraph) {
    await replaceBlockWithUpdate(blockId, createBlockUpdate("paragraph", ""));
    requestBlockFocus(blockId);
    return;
  }
  if (plan.deleteBlockId) {
    await trashNotesBlock(plan.deleteBlockId, true);
    await loadPageTree(selectedPageId);
  }
  requestBlockFocus(plan.focusBlockId);
}

async function mergeBlockWithPrevious(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(blockId);
  const plan = planMergeWithPrevious(flatBlockItemsForBlockContext(blockId), blockId);
  if (!plan) return;
  const target = blocksById[plan.targetBlockId];
  if (!target) return;
  await replaceBlockWithUpdate(target.id, blockWithText(target, plan.mergedText));
  await trashNotesBlock(plan.sourceBlockId, true);
  await loadPageTree(selectedPageId);
  requestBlockFocus(target.id);
}

async function nestBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(blockId);
  const plan = planNestBlock(treeState(), blockId);
  if (!plan) return;
  await moveNotesBlock(blockId, {
    parent: { type: "block_id", block_id: plan.parentId },
    after: plan.after,
  });
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

async function outdentBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(blockId);
  const plan = planOutdentBlock(treeState(), blockId);
  if (!plan) return;
  const parent: NotesParent = plan.parentId === selectedPageId
    ? { type: "page_id", page_id: selectedPageId }
    : { type: "block_id", block_id: plan.parentId };
  await moveNotesBlock(blockId, { parent, after: plan.after });
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

function parentFromMoveParentId(parentId: string): NotesParent | null {
  if (!selectedPageId) return null;
  return parentId === selectedPageId
    ? { type: "page_id", page_id: selectedPageId }
    : { type: "block_id", block_id: parentId };
}

async function moveBlockWithinSiblings(
  blockId: string,
  direction: "up" | "down",
): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(blockId);
  const plan = planMoveBlockWithinSiblings(treeState(), blockId, direction);
  if (!plan) return;
  const parent = parentFromMoveParentId(plan.parentId);
  if (!parent) return;
  await moveNotesBlock(blockId, { parent, after: plan.after, before: plan.before });
  await loadPageTree(selectedPageId);
  requestBlockFocus(blockId);
}

async function dropBlockWithinSiblings(
  sourceBlockId: string,
  targetBlockId: string,
  position: "before" | "after",
): Promise<void> {
  if (!selectedPageId) return;
  await flushBlockSave(sourceBlockId);
  const plan = planDropBlockWithinSiblings(treeState(), sourceBlockId, targetBlockId, position);
  if (!plan) return;
  const parent = parentFromMoveParentId(plan.parentId);
  if (!parent) return;
  await moveNotesBlock(sourceBlockId, { parent, after: plan.after, before: plan.before });
  await loadPageTree(selectedPageId);
  requestBlockFocus(sourceBlockId);
}

async function moveBlockToPage(blockId: string, pageId: string): Promise<void> {
  if (!selectedPageId || pageId === selectedPageId) return;
  const block = blocksById[blockId];
  if (!block || (block.type === "child_page" && block.id === pageId)) return;
  await flushBlockSave(blockId);
  await moveNotesBlock(blockId, {
    parent: { type: "page_id", page_id: pageId },
    after: null,
    before: null,
  });
  await reloadPages();
  await loadPageTree(selectedPageId);
  requestBlockFocus(null);
}

async function duplicateBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  if (blocksById[blockId]?.type === "child_page") return;
  await flushBlockSave(blockId);
  const request = createDuplicateBlockRequest(treeState(), blockId, () => crypto.randomUUID());
  if (request.duplicated_block_ids.length === 0) return;
  const duplicate = await duplicateNotesBlock(blockId, request);
  await loadPageTree(selectedPageId);
  requestBlockFocus(duplicate.id);
}

async function useTemplateBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  const template = blocksById[blockId];
  if (!template || template.type !== "template") return;
  const childIds = (childIdsByParentId[blockId] ?? [])
    .filter((childId) => {
      const child = blocksById[childId];
      return child !== undefined && !child.in_trash;
    });
  if (childIds.length === 0) return;
  const state = treeState();
  const templateContainsChildPage = childIds.some((childId) =>
    collectLoadedBlockSubtreeIds(state, childId).some((subtreeId) =>
      blocksById[subtreeId]?.type === "child_page"
    )
  );
  if (templateContainsChildPage) return;

  await flushPendingBlockSaves();
  let after = blockId;
  let firstDuplicateId: string | null = null;
  for (const childId of childIds) {
    const request = createDuplicateBlockRequest(state, childId, () => crypto.randomUUID());
    if (request.duplicated_block_ids.length === 0) continue;
    const duplicate = await duplicateNotesBlock(childId, request);
    await moveNotesBlock(duplicate.id, {
      parent: template.parent,
      after,
      before: null,
    });
    after = duplicate.id;
    firstDuplicateId ??= duplicate.id;
  }

  await loadPageTree(selectedPageId);
  requestBlockFocus(firstDuplicateId);
}

function activeChildIdsForBlock(blockId: string): string[] {
  return (childIdsByParentId[blockId] ?? []).filter((childId) => {
    const child = blocksById[childId];
    return child !== undefined && !child.in_trash;
  });
}

function loadedSubtreesContainChildPage(state: NotesTreeState, childIds: readonly string[]): boolean {
  return childIds.some((childId) =>
    collectLoadedBlockSubtreeIds(state, childId).some(
      (subtreeId) => blocksById[subtreeId]?.type === "child_page",
    ),
  );
}

function firstActiveSiblingId(parentId: string): string | null {
  return (
    (childIdsByParentId[parentId] ?? []).find((childId) => {
      const child = blocksById[childId];
      return child !== undefined && !child.in_trash;
    }) ?? null
  );
}

function buttonInsertTarget(
  blockId: string,
  position: NotesButtonInsertPosition,
): { parent: NotesParent; after: string | null; before: string | null } | null {
  const button = blocksById[blockId];
  if (!selectedPageId || !button || button.type !== "button") return null;
  if (position === "below_button") {
    return { parent: button.parent, after: blockId, before: null };
  }
  if (position === "above_button") {
    return { parent: button.parent, after: null, before: blockId };
  }
  const pageParent = { type: "page_id", page_id: selectedPageId } as const satisfies NotesParent;
  if (position === "bottom_of_page") {
    return { parent: pageParent, after: null, before: null };
  }
  return {
    parent: pageParent,
    after: null,
    before: firstActiveSiblingId(selectedPageId),
  };
}

async function insertLoadedChildSubtrees(
  childIds: readonly string[],
  target: { parent: NotesParent; after: string | null; before: string | null },
): Promise<string | null> {
  const state = treeState();
  let after = target.after;
  let before = target.before;
  let firstDuplicateId: string | null = null;
  for (const childId of childIds) {
    const request = createDuplicateBlockRequest(state, childId, () => crypto.randomUUID());
    if (request.duplicated_block_ids.length === 0) continue;
    const duplicate = await duplicateNotesBlock(childId, request);
    await moveNotesBlock(duplicate.id, {
      parent: target.parent,
      after,
      before,
    });
    after = duplicate.id;
    before = null;
    firstDuplicateId ??= duplicate.id;
  }
  return firstDuplicateId;
}

async function useButtonBlock(blockId: string): Promise<void> {
  if (!selectedPageId) return;
  const button = blocksById[blockId];
  if (!button || button.type !== "button") return;
  const action = button.button.actions.find((candidate) => candidate.type === "insert_blocks");
  if (!action) return;
  const childIds = activeChildIdsForBlock(blockId);
  if (childIds.length === 0) return;
  const state = treeState();
  if (loadedSubtreesContainChildPage(state, childIds)) return;
  const target = buttonInsertTarget(blockId, action.position);
  if (!target) return;

  await flushPendingBlockSaves();
  const firstDuplicateId = await insertLoadedChildSubtrees(childIds, target);
  await loadPageTree(selectedPageId);
  requestBlockFocus(firstDuplicateId);
}

function isOnlyBlock(blockId: string): boolean {
  const items = flatBlockItemsForBlockContext(blockId);
  return items.length === 1 && items[0]?.block.id === blockId;
}

function focusBlock(blockId: string): void {
  requestBlockFocus(blockId);
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
      return focusBlockId;
    },
    get focusRequestId(): number {
      return focusRequestId;
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
    updateBlockText,
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
    deleteBlock,
    mergeBlockWithPrevious,
    nestBlock,
    outdentBlock,
    moveBlockUp: (blockId: string) => moveBlockWithinSiblings(blockId, "up"),
    moveBlockDown: (blockId: string) => moveBlockWithinSiblings(blockId, "down"),
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
