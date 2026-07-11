import {
  acceptNotesSuggestion,
  addNotesPageAlias,
  applyNotesPageTemplate,
  archiveNotesPage,
  createNotesFolder,
  createNotesChildPageFromBlock,
  createNotesComment,
  createNotesPage,
  createNotesPageTemplateFromPage,
  createNotesSuggestion,
  deleteNotesComment,
  deleteNotesFolder,
  deleteNotesPageAlias,
  deleteNotesPageTemplate,
  duplicateNotesPage,
  duplicateNotesPageTemplate,
  getNotesLocalUser,
  getNotesBlockFrontier,
  getNotesBlockChildren,
  getNotesPageBreadcrumb,
  importNotesHtmlPage,
  importNotesNotionApi,
  importNotesNotionExportFolder,
  listNotesBacklinks,
  listNotesComments,
  listNotesFolders,
  listNotesPageAliases,
  listNotesPages,
  listNotesPageTemplates,
  listNotesSuggestions,
  listNotesUnresolvedLinks,
  listNotesDestinationCandidates,
  listNotesSidebarPages,
  loadNotesWorkspaceShell,
  listArchivedNotesPages,
  listTrashedNotesPages,
  loadNotesPage,
  openNotesPage,
  markNotesCommentThreadsRead,
  moveNotesPage,
  permanentlyDeleteNotesPage,
  resolveNotesCommentThread,
  resolveNotesUnresolvedLink,
  rejectNotesSuggestion,
  saveNotesAgentBridge,
  saveNotesHtmlArchive,
  saveNotesJsonGraph,
  searchNotes,
  trashNotesPage,
  updateNotesComment,
  updateNotesFolder,
  updateNotesLocalUser,
  updateNotesPage,
  updateNotesPageTemplate,
} from "$lib/api/notes";
import { invalidateNotesPageCoverAssetUrl } from "$lib/api/notes-page-covers";
import { invalidateNotesPageIconAssetUrl } from "$lib/api/notes-page-icons";
import { blockPlainText, createRichText } from "$lib/notes/block-factory";
import {
  notesCommentAnchorDraft,
  notesCommentParentKey,
  notesCommentParentMatches,
} from "$lib/notes/comments";
import {
  notesApplySuggestionToBlock,
  notesSuggestionCreateRequest,
  notesSuggestionDraft,
  type NotesSuggestionDraft,
} from "$lib/notes/suggestions";
import type { NotesBlockLinkTarget, NotesPageLinkTarget } from "$lib/notes/block-link";
import {
  buildNotesChildIdsByParent,
  parentIdForBlock,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import { nextSelectedNotesPageId } from "$lib/notes/page-selection";
import {
  notesPageOpenModeForSelection,
  notesDefaultOpenModeForProject,
  type NotesPageOpenMode,
} from "$lib/notes/page-open-mode";
import {
  recordRecentNotesPageId,
  setNotesPageFavoriteId,
} from "$lib/notes/page-navigation";
import {
  normalizeNotesProjectId,
  notesPageProjectId,
  notesPageProjectProperties,
  type NotesCreatePageOptions,
} from "$lib/notes/project-membership";
import { notesPageCoverAssetPath } from "$lib/notes/page-cover";
import { notesPageIconAssetPath } from "$lib/notes/page-icon";
import {
  nextNotesFocusRequest,
  planNotesInsertedBlockFocus,
  planNotesPageLoadFocus,
  type NotesFocusRequest,
} from "$lib/notes/editor-focus";
import type { NotesTextSelection } from "$lib/notes/editor-selection";
import type { NotesUndoSnapshot } from "$lib/notes/undo-history";
import {
  applyNotesPostMutationToTree,
  type NotesPostMutationResult,
  type NotesSidebarMetadataImpact,
} from "$lib/notes/post-mutation";
import { createNotesSidebarRefreshCoordinator } from "$lib/notes/sidebar-refresh-coordinator";
import { createNotesBlockActions } from "./notes-store-block-actions";
import { createNotesPageHistoryController } from "./notes-store-page-history.svelte";
import { createNotesUndoController } from "./notes-store-undo";
import { getPreferences } from "./preferences.svelte";
import { getProjects } from "./projects.svelte";
import {
  flatNotesBlockItems,
  flatNotesBlockItemsForContext,
  isOnlyNotesBlockInContext,
  notesColumnItemsForBlock,
  notesTabItemsForBlock,
  notesTableRowsForBlock,
  notesTreeState,
  notesTreeStateWithoutLeafBlock,
  previousNotesBlockType,
  type NotesBlockTreeSnapshot,
} from "./notes-store-block-tree";
import {
  initialNotesFavoritePageIds,
  initialNotesRecentPageIds,
  initialNotesSelectedPageId,
  initialNotesSidebarCollapsedFolderIds,
  initialNotesSidebarExpandedPageIds,
  saveNotesFavoritePageIds,
  saveNotesRecentPageIds,
  saveNotesSelectedPageId,
  saveNotesSidebarCollapsedFolderIds,
  saveNotesSidebarExpandedPageIds,
} from "./notes-store-page-state";
import { createNotesBlockPersistence } from "./notes-store-persistence";
import { invalidateNotesNotificationSchedule } from "$lib/notes/notification-schedule.svelte";
import type {
  NotesBlock,
  NotesBacklink,
  NotesCommentAnchorCreate,
  NotesColumnBlockItems,
  NotesFolder,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesCommentParent,
  NotesCommentThread,
  NotesLocalUser,
  NotesLoadedPage,
  NotesHtmlArchiveSaveResult,
  NotesHtmlExportRequest,
  NotesHtmlImportRequest,
  NotesHtmlImportResult,
  NotesAgentBridgeExportRequest,
  NotesAgentBridgeExportSaveResult,
  NotesJsonGraphExportRequest,
  NotesJsonGraphExportSaveResult,
  NotesPage,
  NotesNotionApiImportRequest,
  NotesNotionApiImportResult,
  NotesNotionExportImportRequest,
  NotesNotionExportImportResult,
  NotesPageAlias,
  NotesPageBreadcrumbItem,
  NotesPageCover,
  NotesPageHistorySettings,
  NotesPageHistorySnapshot,
  NotesPageIcon,
  NotesPageTemplate,
  NotesParent,
  NotesSearchResult,
  NotesSuggestion,
  NotesTabBlockItems,
  NotesTableRowBlock,
  NotesUnresolvedLink,
} from "$lib/notes/types";

type NotesViewMode = "pages" | "archive" | "trash";
interface NotesSelectPageOptions {
  openMode?: NotesPageOpenMode;
  focusBlockId?: string | null;
}

interface NotesLoadPageTreeOptions {
  focusOnLoad?: boolean;
  focusBlockId?: string | null;
}

type NotesOptionalSubsystem =
  | "templates"
  | "local-user"
  | "history-settings"
  | "undo"
  | "links"
  | "comments"
  | "suggestions"
  | "page-history"
  | "destinations";
type NotesPagePanelSubsystem = "links" | "comments" | "suggestions" | "page-history";

const BLOCK_SAVE_DEBOUNCE_MS = 350;
const CHILDREN_PAGE_SIZE = 100;

let pages = $state<NotesPage[]>([]);
let allPages = $state<NotesPage[]>([]);
let folders = $state<NotesFolder[]>([]);
let archivedPages = $state<NotesPage[]>([]);
let trashedPages = $state<NotesPage[]>([]);
let pageTemplates = $state<NotesPageTemplate[]>([]);
let selectedPageId = $state<string | null>(initialNotesSelectedPageId());
let pageOpenMode = $state<NotesPageOpenMode>("full");
let favoritePageIds = $state<string[]>(initialNotesFavoritePageIds());
let recentPageIds = $state<string[]>(initialNotesRecentPageIds());
let collapsedFolderIds = $state<string[]>(initialNotesSidebarCollapsedFolderIds());
let sidebarExpandedPageIds = $state<string[]>(initialNotesSidebarExpandedPageIds());
let sidebarPageIdsWithChildren = $state<string[]>([]);
let sidebarMissingParentPageIds = $state<string[]>([]);
let sidebarTrashedParentPageIds = $state<string[]>([]);
let loadedPage = $state<NotesPage | null>(null);
let pageBreadcrumbItems = $state<NotesPageBreadcrumbItem[]>([]);
let backlinks = $state<NotesBacklink[]>([]);
let pageAliases = $state<NotesPageAlias[]>([]);
let unresolvedLinks = $state<NotesUnresolvedLink[]>([]);
let linkResolutionPages = $state<NotesPage[]>([]);
let commentThreads = $state<NotesCommentThread[]>([]);
let activeCommentParent = $state<NotesCommentParent | null>(null);
let activeCommentAnchor = $state<NotesCommentAnchorCreate | null>(null);
let suggestions = $state<NotesSuggestion[]>([]);
let activeSuggestionDraft = $state<NotesSuggestionDraft | null>(null);
let blocksById = $state<Record<string, NotesBlock>>({});
let childIdsByParentId = $state<Record<string, string[]>>({});
let loaded = $state(false);
let loading = $state(false);
let primaryContentReady = $state(false);
let loadError = $state<string | null>(null);
let nextWorkspacePageCursor = $state<string | null>(null);
let nextWorkspaceFolderCursor = $state<string | null>(null);
let workspaceWindowLoading = $state(false);
let workspaceTotalPageCount = 0;
let workspaceTotalFolderCount = 0;
let viewMode = $state<NotesViewMode>("pages");
let archiveLoaded = $state(false);
let archiveLoading = $state(false);
let archiveError = $state<string | null>(null);
let trashLoaded = $state(false);
let trashLoading = $state(false);
let trashError = $state<string | null>(null);
let pageTemplatesLoading = $state(false);
let pageTemplatesError = $state<string | null>(null);
let focusRequest = $state<NotesFocusRequest>({
  blockId: null,
  requestId: 0,
  selection: null,
});
const START_OF_NOTES_BLOCK_SELECTION: NotesTextSelection = { start: 0, end: 0 };
let loadRequestId = 0;
let pageWorkGeneration = 0;
let loadPromise: Promise<void> | null = null;
const optionalSubsystemPromises = new Map<string, Promise<void>>();
const loadedOptionalSubsystems = new Set<string>();
const openPagePanelSubsystems = new Set<NotesPagePanelSubsystem>();
let archiveRequestId = 0;
let trashRequestId = 0;
let pageTemplatesRequestId = 0;
let backlinksRequestId = 0;
let pageAliasesRequestId = 0;
let unresolvedLinksRequestId = 0;
let linkResolutionPagesRequestId = 0;
let commentsRequestId = 0;
let suggestionsRequestId = 0;
let localUserRequestId = 0;
let searchRequestId = 0;
let backlinksLoading = $state(false);
let backlinksError = $state<string | null>(null);
let pageAliasesLoading = $state(false);
let pageAliasesError = $state<string | null>(null);
let unresolvedLinksLoading = $state(false);
let unresolvedLinksError = $state<string | null>(null);
let commentsLoading = $state(false);
let commentsError = $state<string | null>(null);
let commentsIncludeResolved = $state(false);
let suggestionsLoading = $state(false);
let suggestionsError = $state<string | null>(null);
let suggestionsIncludeDecided = $state(false);
let localUser = $state<NotesLocalUser | null>(null);
let localUserLoading = $state(false);
let localUserError = $state<string | null>(null);
let searchResults = $state<NotesSearchResult[]>([]);
let searchLoading = $state(false);
let searchError = $state<string | null>(null);
let searchIncludeResolvedComments = $state(false);
let titleFocusRequest = $state<{ pageId: string | null; requestId: number }>({
  pageId: null,
  requestId: 0,
});
let pageTitleDraft = $state<{ pageId: string; title: string } | null>(null);
const preferences = getPreferences();
const projects = getProjects();

function blockTreeSnapshot(): NotesBlockTreeSnapshot {
  return { selectedPageId, blocksById, childIdsByParentId };
}

function defaultNotesPageOpenMode(projectId: string | null = projects.selectedProjectId): NotesPageOpenMode {
  return notesDefaultOpenModeForProject(
    preferences.notesDefaultOpenMode,
    projects.projectById(projectId)?.notesDefaultOpenMode,
  );
}

function projectIdForPage(pageId: string): string | null {
  const page = allPages.find((candidate) => candidate.id === pageId)
    ?? (loadedPage?.id === pageId ? loadedPage : null);
  return page ? notesPageProjectId(page) : projects.selectedProjectId;
}

function treeState(): NotesTreeState {
  return notesTreeState(blockTreeSnapshot());
}

function saveSelectedPageId(pageId: string | null): void {
  selectedPageId = pageId;
  saveNotesSelectedPageId(pageId);
}

function showSelectedPageAs(openMode: NotesPageOpenMode): void {
  pageOpenMode = openMode;
}

function openSelectedPage(pageId: string, openMode: NotesPageOpenMode): void {
  viewMode = "pages";
  showSelectedPageAs(openMode);
  saveSelectedPageId(pageId);
}

function recordRecentPage(pageId: string): void {
  const next = recordRecentNotesPageId(recentPageIds, pageId);
  recentPageIds = next;
  saveNotesRecentPageIds(next);
}

function requestBlockFocus(
  blockId: string | null,
  selection: NotesTextSelection | null = null,
): void {
  focusRequest = nextNotesFocusRequest(focusRequest, blockId, selection);
}

function requestTitleFocus(pageId: string): void {
  titleFocusRequest = {
    pageId,
    requestId: titleFocusRequest.requestId + 1,
  };
}

function setPageTitleDraft(pageId: string, title: string): void {
  pageTitleDraft = { pageId, title };
}

function clearPageTitleDraft(pageId: string): void {
  if (pageTitleDraft?.pageId === pageId) pageTitleDraft = null;
}

function pageTitleDraftForPage(pageId: string): string | null {
  return pageTitleDraft?.pageId === pageId ? pageTitleDraft.title : null;
}

function replacePages(nextPages: NotesPage[]): void {
  pages = [...nextPages];
}

function replaceAllPages(nextPages: NotesPage[]): void {
  allPages = [...nextPages];
}

function replaceFolders(nextFolders: NotesFolder[]): void {
  folders = [...nextFolders];
}

function upsertFolder(folder: NotesFolder): void {
  folders = folders.some((item) => item.id === folder.id)
    ? folders.map((item) => (item.id === folder.id ? folder : item))
    : [...folders, folder];
}

function upsertPageInActiveCollections(page: NotesPage): void {
  pages = pages.some((item) => item.id === page.id)
    ? pages.map((item) => (item.id === page.id ? page : item))
    : [page, ...pages];
  allPages = allPages.some((item) => item.id === page.id)
    ? allPages.map((item) => (item.id === page.id ? page : item))
    : [page, ...allPages];
  invalidateNotesNotificationSchedule();
}

function removePagesFromActiveCollections(pageIds: ReadonlySet<string>): void {
  pages = pages.filter((page) => !pageIds.has(page.id));
  allPages = allPages.filter((page) => !pageIds.has(page.id));
  invalidateNotesNotificationSchedule();
}

function sidebarSeedPageIds(): string[] {
  return [...new Set([...favoritePageIds, ...recentPageIds])];
}

function replaceBlock(block: NotesBlock): void {
  blocksById = { ...blocksById, [block.id]: block };
}

function applyLocalUndoSnapshot(
  target: NotesUndoSnapshot,
  source: NotesUndoSnapshot,
): void {
  const targetIds = new Set(target.blocks.map((block) => block.id));
  const sourceIds = new Set(source.blocks.map((block) => block.id));
  const affectedIds = new Set([...targetIds, ...sourceIds]);
  const nextBlocksById = { ...blocksById };
  for (const blockId of sourceIds) {
    if (!targetIds.has(blockId)) delete nextBlocksById[blockId];
  }
  for (const block of target.blocks) {
    nextBlocksById[block.id] = block;
  }
  for (const blockId of affectedIds) markBlockLocallyChanged(blockId);
  blocksById = nextBlocksById;
  childIdsByParentId = buildNotesChildIdsByParent(Object.values(nextBlocksById));
}

function insertBlockAfter(block: NotesBlock, afterBlockId: string | null): void {
  const parentId = parentIdForBlock(block);
  const currentChildIds = (childIdsByParentId[parentId] ?? []).filter((id) => id !== block.id);
  const afterIndex = afterBlockId ? currentChildIds.indexOf(afterBlockId) : -1;
  const insertIndex = afterIndex >= 0 ? afterIndex + 1 : currentChildIds.length;
  childIdsByParentId = {
    ...childIdsByParentId,
    [parentId]: [
      ...currentChildIds.slice(0, insertIndex),
      block.id,
      ...currentChildIds.slice(insertIndex),
    ],
  };
  replaceBlock(block);
}

function removeLeafBlockLocally(blockId: string): boolean {
  const next = notesTreeStateWithoutLeafBlock(treeState(), blockId);
  if (!next) return false;
  markBlockLocallyChanged(blockId);
  blocksById = next.blocksById;
  childIdsByParentId = next.childIdsByParentId;
  return true;
}

function setLoadedPageFromLoaded(loaded: NotesLoadedPage): void {
  loadedPage = loaded.page;
  const blocks = loaded.blocks.results;
  blocksById = Object.fromEntries(blocks.map((block) => [block.id, block]));
  childIdsByParentId = buildNotesChildIdsByParent(blocks);
  primaryContentReady = true;
}

function requestLoadedPageFocus(
  loaded: NotesLoadedPage,
  focusBlockId: string | null = null,
): void {
  requestBlockFocus(
    planNotesPageLoadFocus(
      loaded.blocks.results.map((block) => block.id),
      focusBlockId,
    ),
  );
}

async function loadRemainingTopLevelBlocks(
  pageId: string = selectedPageId ?? "",
  requestId: number = pageWorkGeneration,
  initialCursor: string | null = null,
): Promise<void> {
  let cursor = initialCursor;
  while (cursor) {
    const page = await getNotesBlockChildren(pageId, cursor, CHILDREN_PAGE_SIZE);
    if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
    if (page.results.length > 0) {
      blocksById = {
        ...blocksById,
        ...Object.fromEntries(page.results.map((block) => [block.id, block])),
      };
      childIdsByParentId = buildNotesChildIdsByParent(Object.values(blocksById));
    }
    cursor = page.next_cursor;
  }
}

async function loadDescendantFrontiers(
  pageId: string = selectedPageId ?? "",
  requestId: number = pageWorkGeneration,
): Promise<void> {
  let frontier = Object.values(blocksById).filter(
    (block) => block.has_children && block.type !== "child_page",
  ).map((block) => block.id);
  const visited = new Set<string>();
  while (frontier.length > 0) {
    const parentIds = frontier.filter((id) => !visited.has(id));
    if (parentIds.length === 0) return;
    parentIds.forEach((id) => visited.add(id));
    const { blocks: loadedChildren } = await getNotesBlockFrontier(parentIds);
    if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
    if (loadedChildren.length === 0) return;
    blocksById = {
      ...blocksById,
      ...Object.fromEntries(loadedChildren.map((child) => [child.id, child])),
    };
    childIdsByParentId = buildNotesChildIdsByParent(Object.values(blocksById));
    frontier = loadedChildren
      .filter((child) => child.has_children && child.type !== "child_page")
      .map((child) => child.id);
  }
}

function queueDescendantHydration(
  pageId: string = selectedPageId ?? "",
  requestId: number = pageWorkGeneration,
): void {
  void loadDescendantFrontiers(pageId, requestId).catch((error) => {
    if (requestId === pageWorkGeneration && pageId === selectedPageId) {
      loadError = error instanceof Error ? error.message : String(error);
    }
  });
}

async function reloadPages(selectedPageIdOverride: string | null = selectedPageId): Promise<void> {
  const [sidebarPages, nextAllPages, nextFolders] = await Promise.all([
    listNotesSidebarPages({
      expanded_page_ids: [...sidebarExpandedPageIds],
      seed_page_ids: sidebarSeedPageIds(),
      selected_page_id: selectedPageIdOverride,
    }),
    listNotesPages(),
    listNotesFolders(),
  ]);
  replacePages(sidebarPages.pages);
  replaceAllPages(nextAllPages);
  replaceFolders(nextFolders);
  sidebarPageIdsWithChildren = [...sidebarPages.page_ids_with_children];
  sidebarMissingParentPageIds = [...sidebarPages.missing_parent_page_ids];
  sidebarTrashedParentPageIds = [...sidebarPages.trashed_parent_page_ids];
}

const sidebarRefreshCoordinator = createNotesSidebarRefreshCoordinator({
  refresh: async (_impact: Exclude<NotesSidebarMetadataImpact, "none">) => {
    await reloadPages();
  },
});

function applyPostMutation(result: NotesPostMutationResult): void {
  if (result.loadedPage !== undefined) {
    if (result.loadedPage) {
      setLoadedPageFromLoaded(result.loadedPage);
    } else {
      loadedPage = null;
      blocksById = {};
      childIdsByParentId = {};
    }
  }
  if (result.blocks || result.placements || result.removedBlockIds) {
    const next = applyNotesPostMutationToTree(treeState(), result);
    blocksById = { ...next.blocksById };
    childIdsByParentId = Object.fromEntries(
      Object.entries(next.childIdsByParentId).map(([parentId, childIds]) => [
        parentId,
        [...childIds],
      ]),
    );
    for (const block of result.blocks ?? []) markBlockLocallyChanged(block.id);
    for (const blockId of result.removedBlockIds ?? []) markBlockLocallyChanged(blockId);
  }
  for (const page of result.pages ?? []) upsertPageInActiveCollections(page);
  if (result.removedPageIds?.length) {
    removePagesFromActiveCollections(new Set(result.removedPageIds));
  }
  if (loadedPage) {
    const returnedLoadedPage = result.pages?.find((page) => page.id === loadedPage?.id);
    if (returnedLoadedPage) loadedPage = returnedLoadedPage;
  }
  sidebarRefreshCoordinator.schedule(result.sidebarImpact ?? "none");
}

async function reloadLinkResolutionPages(): Promise<void> {
  const requestId = ++linkResolutionPagesRequestId;
  const projectId = projects.selectedProjectId;
  try {
    const result = await listNotesDestinationCandidates(projectId);
    if (requestId !== linkResolutionPagesRequestId || projectId !== projects.selectedProjectId) return;
    linkResolutionPages = [...result.pages];
  } catch {
    if (requestId !== linkResolutionPagesRequestId || projectId !== projects.selectedProjectId) return;
    linkResolutionPages = [...allPages];
  }
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

async function reloadPageTemplates(): Promise<void> {
  const requestId = ++pageTemplatesRequestId;
  pageTemplatesLoading = true;
  pageTemplatesError = null;
  try {
    const nextTemplates = await listNotesPageTemplates();
    if (requestId !== pageTemplatesRequestId) return;
    pageTemplates = [...nextTemplates];
  } catch (error) {
    if (requestId !== pageTemplatesRequestId) return;
    pageTemplatesError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === pageTemplatesRequestId) pageTemplatesLoading = false;
  }
}

async function loadPageTree(pageId: string, options: NotesLoadPageTreeOptions = {}): Promise<void> {
  const requestId = ++pageWorkGeneration;
  primaryContentReady = false;
  const loaded = await openNotesPage(pageId);
  if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
  if (options.focusOnLoad) {
    requestLoadedPageFocus(loaded, options.focusBlockId ?? null);
  }
  setLoadedPageFromLoaded(loaded);
  pageBreadcrumbItems = [...loaded.breadcrumb];
  void (async () => {
    await loadRemainingTopLevelBlocks(pageId, requestId, loaded.blocks.next_cursor);
    if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
    queueDescendantHydration(pageId, requestId);
  })().catch((error) => {
    if (requestId === pageWorkGeneration && pageId === selectedPageId) {
      loadError = error instanceof Error ? error.message : String(error);
    }
  });
}

async function loadPageTreeForUndo(pageId: string): Promise<void> {
  openSelectedPage(pageId, pageOpenMode);
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
    if (requestId !== backlinksRequestId || pageId !== selectedPageId) return;
    backlinks = [...nextBacklinks];
  } catch (error) {
    if (requestId !== backlinksRequestId || pageId !== selectedPageId) return;
    backlinks = [];
    backlinksError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === backlinksRequestId) backlinksLoading = false;
  }
}

async function reloadPageAliases(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++pageAliasesRequestId;
  if (!pageId) {
    pageAliases = [];
    pageAliasesError = null;
    pageAliasesLoading = false;
    return;
  }
  pageAliasesLoading = true;
  pageAliasesError = null;
  try {
    const nextAliases = await listNotesPageAliases(pageId);
    if (requestId !== pageAliasesRequestId || pageId !== selectedPageId) return;
    pageAliases = [...nextAliases];
  } catch (error) {
    if (requestId !== pageAliasesRequestId || pageId !== selectedPageId) return;
    pageAliases = [];
    pageAliasesError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === pageAliasesRequestId) pageAliasesLoading = false;
  }
}

async function reloadUnresolvedLinks(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++unresolvedLinksRequestId;
  if (!pageId) {
    unresolvedLinks = [];
    unresolvedLinksError = null;
    unresolvedLinksLoading = false;
    return;
  }
  unresolvedLinksLoading = true;
  unresolvedLinksError = null;
  try {
    const nextLinks = await listNotesUnresolvedLinks(pageId);
    if (requestId !== unresolvedLinksRequestId || pageId !== selectedPageId) return;
    unresolvedLinks = [...nextLinks];
  } catch (error) {
    if (requestId !== unresolvedLinksRequestId || pageId !== selectedPageId) return;
    unresolvedLinks = [];
    unresolvedLinksError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === unresolvedLinksRequestId) unresolvedLinksLoading = false;
  }
}

async function reloadPageBreadcrumb(pageId: string | null = selectedPageId): Promise<void> {
  if (!pageId) {
    pageBreadcrumbItems = [];
    return;
  }
  pageBreadcrumbItems = [...await getNotesPageBreadcrumb(pageId)];
}

async function reloadComments(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++commentsRequestId;
  if (!pageId) {
    commentThreads = [];
    activeCommentParent = null;
    activeCommentAnchor = null;
    commentsError = null;
    commentsLoading = false;
    return;
  }
  commentsLoading = true;
  commentsError = null;
  try {
    const nextThreads = await listNotesComments(pageId, commentsIncludeResolved);
    if (requestId !== commentsRequestId || pageId !== selectedPageId) return;
    commentThreads = [...nextThreads];
  } catch (error) {
    if (requestId !== commentsRequestId || pageId !== selectedPageId) return;
    commentThreads = [];
    commentsError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === commentsRequestId) commentsLoading = false;
  }
}

async function reloadSuggestions(pageId: string | null = selectedPageId): Promise<void> {
  const requestId = ++suggestionsRequestId;
  if (!pageId) {
    suggestions = [];
    activeSuggestionDraft = null;
    suggestionsError = null;
    suggestionsLoading = false;
    return;
  }
  suggestionsLoading = true;
  suggestionsError = null;
  try {
    const nextSuggestions = await listNotesSuggestions(pageId, suggestionsIncludeDecided);
    if (requestId !== suggestionsRequestId || pageId !== selectedPageId) return;
    suggestions = [...nextSuggestions];
  } catch (error) {
    if (requestId !== suggestionsRequestId || pageId !== selectedPageId) return;
    suggestions = [];
    suggestionsError = error instanceof Error ? error.message : String(error);
  } finally {
    if (requestId === suggestionsRequestId) suggestionsLoading = false;
  }
}

async function loadLocalUser(): Promise<NotesLocalUser | null> {
  const requestId = ++localUserRequestId;
  localUserLoading = true;
  localUserError = null;
  try {
    const nextLocalUser = await getNotesLocalUser();
    if (requestId !== localUserRequestId) return localUser;
    localUser = nextLocalUser;
    return nextLocalUser;
  } catch (error) {
    if (requestId !== localUserRequestId) return localUser;
    localUserError = error instanceof Error ? error.message : String(error);
    return null;
  } finally {
    if (requestId === localUserRequestId) localUserLoading = false;
  }
}

async function updateLocalUserDisplayName(displayName: string): Promise<NotesLocalUser | null> {
  const requestId = ++localUserRequestId;
  localUserLoading = true;
  localUserError = null;
  try {
    const nextLocalUser = await updateNotesLocalUser({ display_name: displayName });
    if (requestId !== localUserRequestId) return localUser;
    localUser = nextLocalUser;
    await Promise.all([
      openPagePanelSubsystems.has("comments") ? reloadComments() : Promise.resolve(),
      openPagePanelSubsystems.has("suggestions") ? reloadSuggestions() : Promise.resolve(),
    ]);
    return nextLocalUser;
  } catch (error) {
    if (requestId !== localUserRequestId) return localUser;
    localUserError = error instanceof Error ? error.message : String(error);
    return null;
  } finally {
    if (requestId === localUserRequestId) localUserLoading = false;
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

function updateSuggestion(suggestion: NotesSuggestion): void {
  if (suggestion.status !== "open" && !suggestionsIncludeDecided) {
    suggestions = suggestions.filter((candidate) => candidate.id !== suggestion.id);
    return;
  }
  const existingIndex = suggestions.findIndex((candidate) => candidate.id === suggestion.id);
  if (existingIndex === -1) {
    suggestions = [...suggestions, suggestion];
    return;
  }
  suggestions = suggestions.map((candidate) => (
    candidate.id === suggestion.id ? suggestion : candidate
  ));
}

function commentParentForSelectedPage(): NotesCommentParent | null {
  return selectedPageId ? { type: "page_id", page_id: selectedPageId } : null;
}

function setActiveCommentParent(parent: NotesCommentParent | null): void {
  activeCommentParent = parent;
  activeCommentAnchor = null;
}

async function startBlockComment(blockId: string): Promise<void> {
  if (!blocksById[blockId]) return;
  await flushBlockSave(blockId);
  activeCommentParent = { type: "block_id", block_id: blockId };
  activeCommentAnchor = null;
  requestBlockFocus(blockId);
}

async function startInlineComment(blockId: string, start: number, end: number): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  const anchor = notesCommentAnchorDraft(blockPlainText(block), start, end);
  if (!anchor) return;
  await flushBlockSave(blockId);
  activeCommentParent = { type: "block_id", block_id: blockId };
  activeCommentAnchor = anchor;
  requestBlockFocus(blockId);
}

async function startInlineSuggestion(blockId: string, start: number, end: number): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  const draft = notesSuggestionDraft(block.id, blockPlainText(block), start, end);
  if (!draft) return;
  await flushBlockSave(blockId);
  activeSuggestionDraft = draft;
  requestBlockFocus(blockId);
}

function cancelSuggestionDraft(): void {
  activeSuggestionDraft = null;
}

async function createSuggestion(proposedText: string): Promise<void> {
  const draft = activeSuggestionDraft;
  if (!draft || proposedText === draft.original_text) return;
  await flushBlockSave(draft.block_id);
  const suggestion = await createNotesSuggestion(
    notesSuggestionCreateRequest(crypto.randomUUID(), draft, proposedText),
  );
  updateSuggestion(suggestion);
  activeSuggestionDraft = null;
  requestBlockFocus(draft.block_id);
}

async function acceptSuggestion(suggestionId: string): Promise<void> {
  const suggestion = suggestions.find((candidate) => candidate.id === suggestionId);
  if (!suggestion || suggestion.status !== "open") return;
  const block = blocksById[suggestion.block_id];
  if (!block) return;
  await flushBlockSave(suggestion.block_id);
  const plan = notesApplySuggestionToBlock(block, suggestion);
  if (!plan) {
    suggestionsError = "target_missing";
    return;
  }
  await blockActions.updateBlockRichText(suggestion.block_id, plan.richText);
  await flushBlockSave(suggestion.block_id);
  const updated = await acceptNotesSuggestion(suggestion.id);
  updateSuggestion(updated);
  requestBlockFocus(suggestion.block_id);
}

async function rejectSuggestion(suggestionId: string): Promise<void> {
  const suggestion = suggestions.find((candidate) => candidate.id === suggestionId);
  if (!suggestion || suggestion.status !== "open") return;
  await flushBlockSave(suggestion.block_id);
  const updated = await rejectNotesSuggestion(suggestion.id);
  updateSuggestion(updated);
  requestBlockFocus(suggestion.block_id);
}

async function setSuggestionsIncludeDecided(includeDecided: boolean): Promise<void> {
  suggestionsIncludeDecided = includeDecided;
  await reloadSuggestions();
}

async function addPageAlias(alias: string): Promise<void> {
  if (!selectedPageId) return;
  const content = alias.trim();
  if (!content) return;
  pageAliases = await addNotesPageAlias(selectedPageId, {
    id: crypto.randomUUID(),
    alias: content,
  });
  pageAliasesError = null;
  await Promise.all([
    reloadUnresolvedLinks(selectedPageId),
    reloadBacklinks(selectedPageId),
  ]);
}

async function deletePageAlias(aliasId: string): Promise<void> {
  if (!selectedPageId) return;
  pageAliases = await deleteNotesPageAlias(selectedPageId, aliasId);
  pageAliasesError = null;
  await Promise.all([
    reloadUnresolvedLinks(selectedPageId),
    reloadBacklinks(selectedPageId),
  ]);
}

async function resolveUnresolvedLink(linkId: string, targetPageId: string): Promise<void> {
  const targetId = targetPageId.trim();
  if (!targetId) return;
  unresolvedLinks = await resolveNotesUnresolvedLink(linkId, {
    target_page_id: targetId,
  });
  unresolvedLinksError = null;
  const sourcePageId = selectedPageId;
  await Promise.all([
    reloadLinkResolutionPages(),
    sourcePageId ? loadPageTree(sourcePageId) : Promise.resolve(),
  ]);
  sidebarRefreshCoordinator.schedule("visible-metadata");
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
  const anchor = activeCommentParent
    && notesCommentParentKey(activeCommentParent) === notesCommentParentKey(parent)
    ? activeCommentAnchor
    : null;
  const thread = await createNotesComment({
    id: crypto.randomUUID(),
    parent,
    anchor: anchor ?? undefined,
    rich_text: [createRichText(content)],
  });
  updateCommentThread(thread);
  activeCommentParent = null;
  activeCommentAnchor = null;
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

async function markCommentThreadsRead(discussionIds: readonly string[]): Promise<void> {
  if (!selectedPageId) return;
  const normalizedIds = [
    ...new Set(discussionIds.map((discussionId) => discussionId.trim()).filter(Boolean)),
  ];
  if (normalizedIds.length === 0) return;
  const nextThreads = await markNotesCommentThreadsRead({
    page_id: selectedPageId,
    discussion_ids: normalizedIds,
    include_resolved: commentsIncludeResolved,
  });
  commentThreads = [...nextThreads];
  commentsError = null;
}

async function markVisibleCommentThreadsRead(parent: NotesCommentParent | null = null): Promise<void> {
  const visibleThreads = parent
    ? commentThreads.filter((thread) => notesCommentParentMatches(thread.parent, parent))
    : commentThreads;
  await markCommentThreadsRead(
    visibleThreads.filter((thread) => thread.unread).map((thread) => thread.id),
  );
}

async function setCommentsIncludeResolved(includeResolved: boolean): Promise<void> {
  commentsIncludeResolved = includeResolved;
  await reloadComments();
}

async function search(
  query: string,
  pageSize = 20,
  includeResolvedComments = searchIncludeResolvedComments,
): Promise<void> {
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
    const results = await searchNotes(trimmed, pageSize, includeResolvedComments);
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

function setSearchIncludeResolvedComments(includeResolvedComments: boolean): void {
  searchIncludeResolvedComments = includeResolvedComments;
}

async function load(): Promise<void> {
  const requestId = ++loadRequestId;
  pageWorkGeneration += 1;
  openPagePanelSubsystems.clear();
  loadedOptionalSubsystems.clear();
  optionalSubsystemPromises.clear();
  backlinksRequestId += 1;
  pageAliasesRequestId += 1;
  unresolvedLinksRequestId += 1;
  linkResolutionPagesRequestId += 1;
  commentsRequestId += 1;
  suggestionsRequestId += 1;
  pageHistoryController.resetPageState();
  sidebarRefreshCoordinator.cancel();
  const requestedSelection = selectedPageId;
  loading = true;
  loadError = null;
  try {
    const shell = await loadNotesWorkspaceShell({
      project_id: projects.selectedProjectId,
      expanded_page_ids: [...sidebarExpandedPageIds],
      seed_page_ids: sidebarSeedPageIds(),
      selected_page_id: requestedSelection,
    });
    if (requestId !== loadRequestId) return;
    replacePages(shell.pages);
    replaceAllPages(shell.pages);
    replaceFolders(shell.folders);
    sidebarPageIdsWithChildren = [...shell.page_ids_with_children];
    sidebarMissingParentPageIds = [...shell.missing_parent_page_ids];
    sidebarTrashedParentPageIds = [...shell.trashed_parent_page_ids];
    nextWorkspacePageCursor = shell.next_page_cursor;
    nextWorkspaceFolderCursor = shell.next_folder_cursor;
    workspaceTotalPageCount = shell.total_page_count;
    workspaceTotalFolderCount = shell.total_folder_count;
    const selectionUnchanged = selectedPageId === requestedSelection;
    const nextSelected = selectionUnchanged ? shell.resolved_selected_page_id : selectedPageId;
    if (selectionUnchanged) saveSelectedPageId(nextSelected);
    loaded = true;
    if (nextSelected) {
      void loadPageTree(nextSelected, { focusOnLoad: true }).catch((error) => {
        if (selectedPageId === nextSelected) {
          loadError = error instanceof Error ? error.message : String(error);
        }
      });
    } else {
      loadedPage = null;
      pageBreadcrumbItems = [];
      backlinks = [];
      backlinksError = null;
      pageAliases = [];
      pageAliasesError = null;
      unresolvedLinks = [];
      unresolvedLinksError = null;
      linkResolutionPages = [];
      commentThreads = [];
      activeCommentParent = null;
      activeCommentAnchor = null;
      commentsError = null;
      suggestions = [];
      activeSuggestionDraft = null;
      suggestionsError = null;
      pageHistoryController.resetPageState();
      blocksById = {};
      childIdsByParentId = {};
    }
  } catch (error) {
    if (requestId !== loadRequestId) return;
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === loadRequestId) loading = false;
  }
}

async function loadMoreWorkspaceWindow(): Promise<void> {
  if (workspaceWindowLoading || (!nextWorkspacePageCursor && !nextWorkspaceFolderCursor)) return;
  workspaceWindowLoading = true;
  const pageCursor = nextWorkspacePageCursor;
  const folderCursor = nextWorkspaceFolderCursor;
  const requestId = loadRequestId;
  try {
    const shell = await loadNotesWorkspaceShell({
      project_id: projects.selectedProjectId,
      expanded_page_ids: [],
      seed_page_ids: [],
      selected_page_id: selectedPageId,
      page_cursor: pageCursor ?? `offset:${workspaceTotalPageCount}`,
      folder_cursor: folderCursor ?? `offset:${workspaceTotalFolderCount}`,
    });
    if (requestId !== loadRequestId) return;
    const mergedPages = [...new Map([...allPages, ...shell.pages].map((page) => [page.id, page])).values()];
    const mergedFolders = [...new Map([...folders, ...shell.folders].map((folder) => [folder.id, folder])).values()];
    replaceAllPages(mergedPages);
    replacePages(mergedPages);
    replaceFolders(mergedFolders);
    sidebarPageIdsWithChildren = [...new Set([
      ...sidebarPageIdsWithChildren,
      ...shell.page_ids_with_children,
    ])];
    nextWorkspacePageCursor = shell.next_page_cursor;
    nextWorkspaceFolderCursor = shell.next_folder_cursor;
  } finally {
    if (requestId === loadRequestId) workspaceWindowLoading = false;
  }
}

async function ensureLoaded(): Promise<void> {
  if (loaded) return;
  if (loadPromise) return loadPromise;
  loadPromise = load().finally(() => {
    loadPromise = null;
  });
  return loadPromise;
}

async function ensureOptionalSubsystem(
  subsystem: NotesOptionalSubsystem,
  pageId: string | null = selectedPageId,
): Promise<void> {
  const pageScoped = subsystem === "links"
    || subsystem === "comments"
    || subsystem === "suggestions"
    || subsystem === "page-history"
    || subsystem === "undo";
  if (pageScoped && !pageId) return;
  const generation = pageWorkGeneration;
  const key = pageScoped
    ? `${generation}:${subsystem}:${pageId}`
    : subsystem === "destinations"
      ? `${subsystem}:${projects.selectedProjectId ?? "workspace"}`
      : subsystem;
  if (loadedOptionalSubsystems.has(key)) return;
  const existing = optionalSubsystemPromises.get(key);
  if (existing) return existing;
  const promise = (async () => {
    switch (subsystem) {
      case "templates":
        await reloadPageTemplates();
        break;
      case "local-user":
        await loadLocalUser();
        break;
      case "history-settings":
        await pageHistoryController.loadSettings();
        break;
      case "undo":
        await undoController.hydrate(pageId);
        break;
      case "links":
        await Promise.all([
          reloadBacklinks(pageId),
          reloadPageAliases(pageId),
          reloadUnresolvedLinks(pageId),
          reloadLinkResolutionPages(),
        ]);
        break;
      case "comments":
        await Promise.all([loadLocalUser(), reloadComments(pageId)]);
        break;
      case "suggestions":
        await reloadSuggestions(pageId);
        break;
      case "page-history":
        if (pageId) await pageHistoryController.reloadSnapshots(pageId);
        break;
      case "destinations":
        await reloadLinkResolutionPages();
        break;
    }
    if (!pageScoped || (generation === pageWorkGeneration && pageId === selectedPageId)) {
      loadedOptionalSubsystems.add(key);
    }
  })().finally(() => {
    if (optionalSubsystemPromises.get(key) === promise) {
      optionalSubsystemPromises.delete(key);
    }
  });
  optionalSubsystemPromises.set(key, promise);
  return promise;
}

function setPagePanelSubsystemOpen(
  subsystem: NotesPagePanelSubsystem,
  open: boolean,
): void {
  if (open) {
    openPagePanelSubsystems.add(subsystem);
  } else {
    openPagePanelSubsystems.delete(subsystem);
  }
}

async function refreshOpenLinks(): Promise<void> {
  const pageId = selectedPageId;
  if (!pageId || !openPagePanelSubsystems.has("links")) return;
  const generation = pageWorkGeneration;
  await Promise.all([
    reloadBacklinks(pageId),
    reloadPageAliases(pageId),
    reloadUnresolvedLinks(pageId),
    reloadLinkResolutionPages(),
  ]);
  if (generation !== pageWorkGeneration || pageId !== selectedPageId) return;
  loadedOptionalSubsystems.add(`${generation}:links:${pageId}`);
}

async function selectPage(
  pageId: string | null,
  options: NotesSelectPageOptions = {},
): Promise<void> {
  const alreadyLoaded = selectedPageId === pageId && (!pageId || loadedPage?.id === pageId);
  const openMode = notesPageOpenModeForSelection({
    requestedOpenMode: options.openMode,
    currentOpenMode: pageOpenMode,
    defaultOpenMode: pageId
      ? defaultNotesPageOpenMode(projectIdForPage(pageId))
      : defaultNotesPageOpenMode(),
    hasOpenPage: selectedPageId !== null,
  });
  if (pageId) {
    openSelectedPage(pageId, openMode);
  } else {
    saveSelectedPageId(null);
  }
  if (alreadyLoaded) return;
  pageWorkGeneration += 1;
  for (const key of loadedOptionalSubsystems) {
    if (/^\d+:/.test(key)) loadedOptionalSubsystems.delete(key);
  }
  undoController.reset(pageId);
  pageHistoryController.resetPageState();
  openPagePanelSubsystems.clear();
  backlinksRequestId += 1;
  pageAliasesRequestId += 1;
  unresolvedLinksRequestId += 1;
  commentsRequestId += 1;
  suggestionsRequestId += 1;
  backlinks = [];
  pageAliases = [];
  unresolvedLinks = [];
  commentThreads = [];
  suggestions = [];
  if (!pageId) {
    loadedPage = null;
    primaryContentReady = false;
    pageBreadcrumbItems = [];
    backlinks = [];
    backlinksError = null;
    pageAliases = [];
    pageAliasesError = null;
    unresolvedLinks = [];
    unresolvedLinksError = null;
    linkResolutionPages = [];
    commentThreads = [];
    activeCommentParent = null;
    activeCommentAnchor = null;
    commentsError = null;
    suggestions = [];
    activeSuggestionDraft = null;
    suggestionsError = null;
    pageHistoryController.resetPageState();
    blocksById = {};
    childIdsByParentId = {};
    return;
  }
  loading = true;
  loadError = null;
  try {
    await loadPageTree(pageId, {
      focusOnLoad: true,
      focusBlockId: options.focusBlockId ?? null,
    });
    recordRecentPage(pageId);
  } catch (error) {
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    loading = false;
  }
}

async function createPage(title: string, options: NotesCreatePageOptions = {}): Promise<void> {
  await createPageWithParent(title, { type: "workspace", workspace: true }, options);
}

async function createSubpage(
  parentPageId: string,
  title: string,
  options: NotesCreatePageOptions = {},
): Promise<void> {
  setSidebarPageCollapsed(parentPageId, false);
  await createPageWithParent(title, { type: "page_id", page_id: parentPageId }, options);
}

function pageProjectIdForParent(
  parent: NotesParent,
  options: NotesCreatePageOptions,
): string | null {
  if ("projectId" in options) return normalizeNotesProjectId(options.projectId);
  const folderId = options.folderId?.trim();
  if (parent.type === "workspace" && folderId) {
    return normalizeNotesProjectId(
      folders.find((folder) => folder.id === folderId)?.project_id,
    );
  }
  if (parent.type !== "page_id") return null;
  const parentPage = allPages.find((page) => page.id === parent.page_id)
    ?? (loadedPage?.id === parent.page_id ? loadedPage : null);
  return parentPage ? notesPageProjectId(parentPage) : null;
}

async function createPageWithParent(
  title: string,
  parent: NotesParent,
  options: NotesCreatePageOptions = {},
): Promise<void> {
  const pageId = crypto.randomUUID();
  const firstBlockId = crypto.randomUUID();
  const projectId = pageProjectIdForParent(parent, options);
  const folderId = parent.type === "workspace"
    ? options.folderId?.trim() || null
    : null;
  const projectProperties = notesPageProjectProperties(projectId);
  const loaded = await createNotesPage({
    id: pageId,
    title,
    parent,
    folder_id: folderId,
    first_block_id: firstBlockId,
    after_block_id: null,
    properties: projectProperties,
  });
  if (loaded.page.folder_id) setFolderCollapsed(loaded.page.folder_id, false);
  await activateReturnedPage(loaded, "hierarchy", defaultNotesPageOpenMode(projectId));
  requestBlockFocus(null);
  requestTitleFocus(loaded.page.id);
}

async function activateReturnedPage(
  loaded: NotesLoadedPage,
  sidebarImpact: Exclude<NotesSidebarMetadataImpact, "none">,
  openMode: NotesPageOpenMode = defaultNotesPageOpenMode(notesPageProjectId(loaded.page)),
): Promise<void> {
  pageWorkGeneration += 1;
  undoController.reset(loaded.page.id);
  pageHistoryController.resetPageState();
  openPagePanelSubsystems.clear();
  openSelectedPage(loaded.page.id, openMode);
  recordRecentPage(loaded.page.id);
  applyPostMutation({
    loadedPage: loaded,
    pages: [loaded.page],
    sidebarImpact,
  });
  await reloadPageBreadcrumb(loaded.page.id);
}

async function importHtmlPage(
  input: Omit<NotesHtmlImportRequest, "parent"> & { parent?: NotesParent },
): Promise<NotesHtmlImportResult> {
  const result = await importNotesHtmlPage({
    ...input,
    parent: input.parent ?? { type: "workspace", workspace: true },
  });
  await activateReturnedPage(result.page, "hierarchy");
  queueDescendantHydration();
  requestPageLoadFocus();
  return result;
}

async function importNotionApi(
  input: Omit<NotesNotionApiImportRequest, "parent"> & { parent?: NotesParent },
): Promise<NotesNotionApiImportResult> {
  const result = await importNotesNotionApi({
    ...input,
    parent: input.parent ?? { type: "workspace", workspace: true },
  });
  await refreshAfterMultiPageImport(result.imported_pages);
  return result;
}

async function importNotionExportFolder(
  input: Omit<NotesNotionExportImportRequest, "parent"> & { parent?: NotesParent },
): Promise<NotesNotionExportImportResult> {
  const result = await importNotesNotionExportFolder({
    ...input,
    parent: input.parent ?? { type: "workspace", workspace: true },
  });
  await refreshAfterMultiPageImport(result.imported_pages);
  return result;
}

async function refreshAfterMultiPageImport(importedPages: NotesLoadedPage[]): Promise<void> {
  viewMode = "pages";
  const firstPage = importedPages[0] ?? null;
  for (const loaded of importedPages) {
    upsertPageInActiveCollections(loaded.page);
  }
  if (firstPage) {
    await activateReturnedPage(firstPage, "hierarchy");
    queueDescendantHydration();
    requestPageLoadFocus();
  } else {
    sidebarRefreshCoordinator.schedule("hierarchy");
  }
}

async function exportHtmlArchive(
  input: Omit<NotesHtmlExportRequest, "page_id"> = {},
): Promise<NotesHtmlArchiveSaveResult> {
  if (!selectedPageId) {
    throw new Error("No Notes page is selected");
  }
  return saveNotesHtmlArchive({
    ...input,
    page_id: selectedPageId,
  });
}

async function exportJsonGraph(
  input: NotesJsonGraphExportRequest = {},
): Promise<NotesJsonGraphExportSaveResult> {
  return saveNotesJsonGraph(input);
}

async function exportAgentBridge(
  input: Omit<NotesAgentBridgeExportRequest, "page_ids"> = {},
): Promise<NotesAgentBridgeExportSaveResult> {
  if (!selectedPageId) {
    throw new Error("No Notes page is selected");
  }
  return saveNotesAgentBridge({
    ...input,
    page_ids: [selectedPageId],
  });
}

async function applyPageTemplate(templateId: string, title?: string): Promise<void> {
  const loaded = await applyNotesPageTemplate(templateId, {
    parent: { type: "workspace", workspace: true },
    title: title?.trim() || null,
  });
  await activateReturnedPage(loaded, "hierarchy");
  queueDescendantHydration();
  requestPageLoadFocus();
}

async function createPageTemplateFromCurrentPage(name: string): Promise<void> {
  if (!loadedPage) return;
  await flushPendingBlockSaves();
  const template = await createNotesPageTemplateFromPage({
    id: crypto.randomUUID(),
    source_page_id: loadedPage.id,
    name,
  });
  pageTemplates = [template, ...pageTemplates.filter((item) => item.id !== template.id)];
}

async function updatePageTemplateFromCurrentPage(templateId: string): Promise<void> {
  if (!loadedPage) return;
  await flushPendingBlockSaves();
  const template = await updateNotesPageTemplate(templateId, {
    source_page_id: loadedPage.id,
  });
  pageTemplates = pageTemplates.map((item) => (item.id === template.id ? template : item));
}

async function renamePageTemplate(templateId: string, name: string): Promise<void> {
  const template = await updateNotesPageTemplate(templateId, { name });
  pageTemplates = pageTemplates.map((item) => (item.id === template.id ? template : item));
}

async function duplicatePageTemplate(templateId: string, name: string): Promise<void> {
  const template = await duplicateNotesPageTemplate(templateId, {
    id: crypto.randomUUID(),
    name,
  });
  pageTemplates = [template, ...pageTemplates];
}

async function deletePageTemplate(templateId: string): Promise<void> {
  const deletedTemplateId = await deleteNotesPageTemplate(templateId);
  pageTemplates = pageTemplates.filter((template) => template.id !== deletedTemplateId);
}

async function createFolder(
  projectId: string,
  name: string,
  parentFolderId: string | null,
): Promise<NotesFolder> {
  const normalizedProjectId = normalizeNotesProjectId(projectId);
  const normalizedName = name.trim();
  const normalizedParentFolderId = parentFolderId?.trim() || null;
  if (!normalizedProjectId) throw new Error("folder project id must not be empty");
  if (!normalizedName) throw new Error("folder name must not be empty");
  const folder = await createNotesFolder({
    id: crypto.randomUUID(),
    project_id: normalizedProjectId,
    parent_folder_id: normalizedParentFolderId,
    name: normalizedName,
  });
  upsertFolder(folder);
  if (normalizedParentFolderId) setFolderCollapsed(normalizedParentFolderId, false);
  return folder;
}

async function renameFolder(folderId: string, name: string): Promise<void> {
  const normalizedName = name.trim();
  if (!normalizedName) throw new Error("folder name must not be empty");
  const currentFolder = folders.find((folder) => folder.id === folderId);
  if (!currentFolder) throw new Error("notes folder not found");
  upsertFolder(await updateNotesFolder(folderId, {
    name: normalizedName,
    parent_folder_id: currentFolder.parent_folder_id,
  }));
}

async function moveFolder(folderId: string, parentFolderId: string | null): Promise<void> {
  const normalizedParentFolderId = parentFolderId?.trim() || null;
  const currentFolder = folders.find((folder) => folder.id === folderId);
  if (!currentFolder) throw new Error("notes folder not found");
  upsertFolder(
    await updateNotesFolder(folderId, {
      name: currentFolder.name,
      parent_folder_id: normalizedParentFolderId,
    }),
  );
  if (normalizedParentFolderId) setFolderCollapsed(normalizedParentFolderId, false);
}

async function deleteFolder(folderId: string): Promise<void> {
  const deletedFolderId = await deleteNotesFolder(folderId);
  folders = folders.filter((folder) => folder.id !== deletedFolderId);
  setFolderCollapsed(deletedFolderId, false);
  sidebarRefreshCoordinator.schedule("hierarchy");
}

function setFolderCollapsed(folderId: string, collapsed: boolean): void {
  const normalizedFolderId = folderId.trim();
  if (!normalizedFolderId) return;
  const next = collapsed
    ? [
        normalizedFolderId,
        ...collapsedFolderIds.filter((candidate) => candidate !== normalizedFolderId),
      ]
    : collapsedFolderIds.filter((candidate) => candidate !== normalizedFolderId);
  collapsedFolderIds = next;
  saveNotesSidebarCollapsedFolderIds(next);
}

function setSidebarPageCollapsed(pageId: string, collapsed: boolean): void {
  const normalizedPageId = pageId.trim();
  if (!normalizedPageId) return;
  const next = collapsed
    ? sidebarExpandedPageIds.filter((candidate) => candidate !== normalizedPageId)
    : [
        normalizedPageId,
        ...sidebarExpandedPageIds.filter((candidate) => candidate !== normalizedPageId),
      ];
  sidebarExpandedPageIds = next;
  saveNotesSidebarExpandedPageIds(next);
  if (!collapsed) void reloadPages();
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
  const projectProperties = notesPageProjectProperties(
    loadedPage ? notesPageProjectId(loadedPage) : null,
  );
  const loaded = await createNotesChildPageFromBlock(blockId, {
    first_block_id: firstBlockId,
    title: blockPlainText(block).trim(),
    properties: projectProperties,
  });
  await activateReturnedPage(loaded, "hierarchy");
  requestBlockFocus(
    planNotesInsertedBlockFocus([loaded.blocks.results[0]?.id, firstBlockId]),
    START_OF_NOTES_BLOCK_SELECTION,
  );
}

async function createChildPageAfterBlock(blockId: string): Promise<void> {
  const block = blocksById[blockId];
  if (!block) return;
  await flushBlockSave(blockId);
  const pageId = crypto.randomUUID();
  const firstBlockId = crypto.randomUUID();
  const projectProperties = notesPageProjectProperties(
    loadedPage ? notesPageProjectId(loadedPage) : null,
  );
  const loaded = await createNotesPage({
    id: pageId,
    title: "",
    parent: block.parent,
    folder_id: null,
    first_block_id: firstBlockId,
    after_block_id: blockId,
    properties: projectProperties,
  });
  if (block.parent.type === "page_id") {
    setSidebarPageCollapsed(block.parent.page_id, false);
  }
  await activateReturnedPage(loaded, "hierarchy");
  requestBlockFocus(
    planNotesInsertedBlockFocus([loaded.blocks.results[0]?.id, firstBlockId]),
    START_OF_NOTES_BLOCK_SELECTION,
  );
}

async function renamePage(pageId: string, title: string): Promise<void> {
  const trimmedTitle = title.trim();
  const page = await updateNotesPage(pageId, { title: trimmedTitle });
  applyPostMutation({ pages: [page], sidebarImpact: "visible-metadata" });
  if (selectedPageId === page.id) await reloadPageBreadcrumb(page.id);
}

async function duplicatePage(pageId: string, title: string): Promise<void> {
  await flushPendingBlockSaves();
  const loaded = await duplicateNotesPage(pageId, { title });
  if (loaded.page.parent.type === "page_id") {
    setSidebarPageCollapsed(loaded.page.parent.page_id, false);
  }
  if (loaded.page.folder_id) setFolderCollapsed(loaded.page.folder_id, false);
  await activateReturnedPage(loaded, "hierarchy");
  requestBlockFocus(planNotesPageLoadFocus(loaded.blocks.results.map((block) => block.id)));
}

async function movePage(pageId: string, parent: NotesParent): Promise<void> {
  await movePageWithPlacement(pageId, parent, null);
}

async function movePageToFolder(pageId: string, folderId: string | null): Promise<void> {
  const normalizedFolderId = folderId?.trim() || null;
  await movePageWithPlacement(
    pageId,
    { type: "workspace", workspace: true },
    normalizedFolderId,
  );
}

async function movePageWithPlacement(
  pageId: string,
  parent: NotesParent,
  folderId: string | null,
): Promise<void> {
  await flushPendingBlockSaves();
  const loaded = await moveNotesPage(pageId, { parent, folder_id: folderId });
  if (loaded.page.parent.type === "page_id") {
    setSidebarPageCollapsed(loaded.page.parent.page_id, false);
  }
  if (loaded.page.folder_id) setFolderCollapsed(loaded.page.folder_id, false);
  await activateReturnedPage(loaded, "hierarchy");
  queueDescendantHydration();
  requestPageLoadFocus();
}

async function updatePageIcon(pageId: string, icon: NotesPageIcon | null): Promise<void> {
  const previousPage = loadedPage?.id === pageId
    ? loadedPage
    : allPages.find((candidate) => candidate.id === pageId);
  const previousPath = notesPageIconAssetPath(previousPage?.icon ?? null);
  const page = await updateNotesPage(pageId, { icon });
  const nextPath = notesPageIconAssetPath(page.icon);
  if (previousPath && previousPath !== nextPath) {
    invalidateNotesPageIconAssetUrl(previousPath);
  }
  applyPostMutation({ pages: [page], sidebarImpact: "visible-metadata" });
}

async function updatePageCover(pageId: string, cover: NotesPageCover | null): Promise<void> {
  const previousPage = loadedPage?.id === pageId
    ? loadedPage
    : allPages.find((candidate) => candidate.id === pageId);
  const previousPath = notesPageCoverAssetPath(previousPage?.cover ?? null);
  const page = await updateNotesPage(pageId, { cover });
  const nextPath = notesPageCoverAssetPath(page.cover);
  if (previousPath && previousPath !== nextPath) {
    invalidateNotesPageCoverAssetUrl(previousPath);
  }
  applyPostMutation({ pages: [page] });
}

async function trashPage(pageId: string): Promise<void> {
  const trashedPage = await trashNotesPage(pageId, true);
  const preferredNextSelected = nextSelectedNotesPageId(pages, pageId);
  if (trashLoaded) {
    trashedPages = [trashedPage, ...trashedPages.filter((page) => page.id !== pageId)];
  }
  archivedPages = archivedPages.filter((page) => page.id !== pageId);
  applyPostMutation({ removedPageIds: [pageId], sidebarImpact: "hierarchy" });
  const nextSelected = preferredNextSelected && pages.some((page) => page.id === preferredNextSelected)
    ? preferredNextSelected
    : pages[0]?.id ?? null;
  await selectPage(nextSelected);
}

async function archivePage(pageId: string): Promise<void> {
  const archivedPage = await archiveNotesPage(pageId, true);
  if (archiveLoaded) {
    archivedPages = [archivedPage, ...archivedPages.filter((page) => page.id !== pageId)];
  }
  const nextSelected = nextSelectedNotesPageId(pages, pageId);
  applyPostMutation({ removedPageIds: [pageId], sidebarImpact: "hierarchy" });
  await selectPage(nextSelected);
}

async function unarchivePage(pageId: string): Promise<void> {
  const restoredPage = await archiveNotesPage(pageId, false);
  archivedPages = archivedPages.filter((page) => page.id !== pageId);
  pageWorkGeneration += 1;
  undoController.reset(restoredPage.id);
  pageHistoryController.resetPageState();
  openPagePanelSubsystems.clear();
  openSelectedPage(restoredPage.id, defaultNotesPageOpenMode(notesPageProjectId(restoredPage)));
  applyPostMutation({ pages: [restoredPage], sidebarImpact: "hierarchy" });
  await loadPageTree(restoredPage.id);
  recordRecentPage(restoredPage.id);
  requestPageLoadFocus();
}

async function restorePage(pageId: string): Promise<void> {
  const restoredPage = await trashNotesPage(pageId, false);
  trashedPages = trashedPages.filter((page) => page.id !== pageId);
  pageWorkGeneration += 1;
  undoController.reset(restoredPage.id);
  pageHistoryController.resetPageState();
  openPagePanelSubsystems.clear();
  openSelectedPage(restoredPage.id, defaultNotesPageOpenMode(notesPageProjectId(restoredPage)));
  applyPostMutation({ pages: [restoredPage], sidebarImpact: "hierarchy" });
  await loadPageTree(restoredPage.id);
  recordRecentPage(restoredPage.id);
  requestPageLoadFocus();
}

async function permanentlyDeletePage(pageId: string): Promise<void> {
  await flushPendingBlockSaves();
  const deletedPageIds = await permanentlyDeleteNotesPage(pageId);
  const deletedPageIdSet = new Set(deletedPageIds);
  removePagesFromActiveCollections(deletedPageIdSet);
  archivedPages = archivedPages.filter((page) => !deletedPageIdSet.has(page.id));
  trashedPages = trashedPages.filter((page) => !deletedPageIdSet.has(page.id));
  const nextFavoritePageIds = favoritePageIds.filter((id) => !deletedPageIdSet.has(id));
  const nextRecentPageIds = recentPageIds.filter((id) => !deletedPageIdSet.has(id));
  const nextExpandedPageIds = sidebarExpandedPageIds.filter((id) => !deletedPageIdSet.has(id));
  favoritePageIds = nextFavoritePageIds;
  recentPageIds = nextRecentPageIds;
  sidebarExpandedPageIds = nextExpandedPageIds;
  saveNotesFavoritePageIds(nextFavoritePageIds);
  saveNotesRecentPageIds(nextRecentPageIds);
  saveNotesSidebarExpandedPageIds(nextExpandedPageIds);
  sidebarRefreshCoordinator.schedule("hierarchy");
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

function visibleBlockIds(): string[] {
  return flatBlockItems().map((item) => item.block.id);
}

function requestPageLoadFocus(requestedBlockId: string | null = null): void {
  requestBlockFocus(planNotesPageLoadFocus(visibleBlockIds(), requestedBlockId));
}

const {
  localApplyBlockUpdate,
  markBlockLocallyChanged,
  saveBlockNow,
  scheduleBlockSave,
  flushBlockSave,
  flushPendingBlockSaves,
} = createNotesBlockPersistence({
  readBlock: (blockId) => blocksById[blockId],
  replaceBlock,
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
  flushPendingMutations: async () => {
    await Promise.all([
      flushPendingBlockSaves(),
      blockActions.flushOptimisticBlockWrites(),
    ]);
  },
  applyLocalSnapshot: applyLocalUndoSnapshot,
});

const pageHistoryController = createNotesPageHistoryController({
  readSelectedPageId: () => selectedPageId,
  applyPostMutation,
  requestPageLoadFocus,
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
  createChildPageAfterBlock,
  applyPostMutation,
  loadPageTree,
  refreshOpenLinks,
  localApplyBlockUpdate,
  localInsertBlockAfter: insertBlockAfter,
  localRemoveLeafBlock: removeLeafBlockLocally,
  saveBlockNow,
  scheduleBlockSave,
  flushBlockSave,
  flushPendingBlockSaves,
  createUndoSnapshot: undoController.snapshot,
  createUndoSnapshotForBlocks: undoController.snapshotBlocks,
  recordUndo: undoController.record,
});

const {
  updateBlockText, updateBlockRichText,
  insertPageMention,
  insertDateMention,
  insertObjectMention,
  insertInlineEquation,
  updateBlockTextLink,
  updateBlockTextAnnotations,
  updateBookmark,
  updateEmbedUrl,
  updateLinkPreviewUrl,
  updateEquationExpression,
  updateMedia,
  updateTableCell,
  updateTableCellRichText,
  addTableRow,
  removeTableRow,
  addTableColumn,
  removeTableColumn,
  addColumn,
  removeColumn,
  moveColumn,
  resizeColumn,
  moveBlockToColumn,
  updateTabLabel,
  updateTabIcon,
  addTab,
  removeTab,
  moveTab,
  moveBlockToTab,
  convertBlock,
  toggleTodo,
  updateCodeLanguage,
  updateBlockColor,
  updateToggleOpen,
  convertBlockToToggleHeading,
  createSiblingAfter,
  splitTextBlockAtSelection,
  pastePlainTextIntoBlock,
  pasteRichHtmlIntoBlock,
  pasteBlockSelection,
  deleteBlock,
  deleteBlockSelection,
  mergeBlockWithPrevious,
  nestBlock,
  outdentBlock,
  moveBlockUp,
  moveBlockDown,
  moveBlockSelection,
  dropBlockWithinSiblings,
  dropBlockOnBlock,
  moveBlockToPage,
  duplicateBlock,
  duplicateBlockSelection,
  addTemplateChild,
  useTemplateBlock,
  addButtonChild,
  updateButtonIcon,
  updateButtonInsertPosition,
  useButtonBlock,
  createLinkedDatabaseViewAfter,
  convertUnsupportedBlock,
} = blockActions;

function isOnlyBlock(blockId: string): boolean {
  return isOnlyNotesBlockInContext(blockTreeSnapshot(), blockId);
}

function focusBlock(blockId: string, selection: NotesTextSelection | null = null): void {
  requestBlockFocus(blockId, selection);
}

async function undoNotesEdit(): Promise<boolean> {
  try {
    if (!undoController.canUndo() && selectedPageId) {
      await ensureOptionalSubsystem("undo", selectedPageId);
    }
    return await undoController.undo();
  } catch (error) {
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  }
}

async function redoNotesEdit(): Promise<boolean> {
  try {
    if (!undoController.canRedo() && selectedPageId) {
      await ensureOptionalSubsystem("undo", selectedPageId);
    }
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
  if (!allPages.some((page) => page.id === target.pageId)) {
    await reloadPages(target.pageId);
  }
  if (!allPages.some((page) => page.id === target.pageId)) return false;
  if (loadedPage?.id !== target.pageId) {
    await selectPage(target.pageId, { focusBlockId: target.blockId ?? null });
  }
  if (!target.blockId) {
    requestPageLoadFocus();
    return true;
  }
  if (!blocksById[target.blockId] || !visibleBlockIds().includes(target.blockId)) return false;
  requestBlockFocus(planNotesPageLoadFocus(visibleBlockIds(), target.blockId));
  return true;
}

export function getNotes() {
  return {
    get pages(): NotesPage[] {
      return pages;
    },
    get allPages(): NotesPage[] {
      return allPages;
    },
    get folders(): NotesFolder[] {
      return folders;
    },
    get archivedPages(): NotesPage[] {
      return archivedPages;
    },
    get pageTemplates(): NotesPageTemplate[] {
      return pageTemplates;
    },
    get workspacePages(): NotesPage[] {
      return allPages.filter((page) => page.parent.type === "workspace");
    },
    get favoritePageIds(): readonly string[] {
      return favoritePageIds;
    },
    get recentPageIds(): readonly string[] {
      return recentPageIds;
    },
    get collapsedFolderIds(): readonly string[] {
      return collapsedFolderIds;
    },
    get sidebarExpandedPageIds(): readonly string[] {
      return sidebarExpandedPageIds;
    },
    get sidebarPageIdsWithChildren(): readonly string[] {
      return sidebarPageIdsWithChildren;
    },
    get sidebarMissingParentPageIds(): readonly string[] {
      return sidebarMissingParentPageIds;
    },
    get sidebarTrashedParentPageIds(): readonly string[] {
      return sidebarTrashedParentPageIds;
    },
    get trashedPages(): NotesPage[] {
      return trashedPages;
    },
    get selectedPageId(): string | null {
      return selectedPageId;
    },
    get pageOpenMode(): NotesPageOpenMode {
      return pageOpenMode;
    },
    get loadedPage(): NotesPage | null {
      return loadedPage;
    },
    get primaryContentReady(): boolean {
      return primaryContentReady;
    },
    get pageBreadcrumbItems(): NotesPageBreadcrumbItem[] {
      return pageBreadcrumbItems;
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
    get pageAliases(): NotesPageAlias[] {
      return pageAliases;
    },
    get pageAliasesLoading(): boolean {
      return pageAliasesLoading;
    },
    get pageAliasesError(): string | null {
      return pageAliasesError;
    },
    get unresolvedLinks(): NotesUnresolvedLink[] {
      return unresolvedLinks;
    },
    get unresolvedLinksLoading(): boolean {
      return unresolvedLinksLoading;
    },
    get unresolvedLinksError(): string | null {
      return unresolvedLinksError;
    },
    get linkResolutionPages(): NotesPage[] {
      return linkResolutionPages.length > 0 ? linkResolutionPages : allPages;
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
    get activeCommentAnchor(): NotesCommentAnchorCreate | null {
      return activeCommentAnchor;
    },
    get suggestions(): NotesSuggestion[] {
      return suggestions;
    },
    get suggestionsLoading(): boolean {
      return suggestionsLoading;
    },
    get suggestionsError(): string | null {
      return suggestionsError;
    },
    get suggestionsIncludeDecided(): boolean {
      return suggestionsIncludeDecided;
    },
    get activeSuggestionDraft(): NotesSuggestionDraft | null {
      return activeSuggestionDraft;
    },
    get localUser(): NotesLocalUser | null {
      return localUser;
    },
    get localUserLoading(): boolean {
      return localUserLoading;
    },
    get localUserError(): string | null {
      return localUserError;
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
    get searchIncludeResolvedComments(): boolean {
      return searchIncludeResolvedComments;
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
    get pageTemplatesLoading(): boolean {
      return pageTemplatesLoading;
    },
    get pageTemplatesError(): string | null {
      return pageTemplatesError;
    },
    get pageHistorySnapshots(): NotesPageHistorySnapshot[] {
      return pageHistoryController.snapshots;
    },
    get pageHistorySnapshotsLoading(): boolean {
      return pageHistoryController.snapshotsLoading;
    },
    get pageHistorySnapshotsError(): string | null {
      return pageHistoryController.snapshotsError;
    },
    get pageHistoryVersion(): NotesLoadedPage | null {
      return pageHistoryController.version;
    },
    get pageHistoryVersionLoading(): boolean {
      return pageHistoryController.versionLoading;
    },
    get pageHistoryVersionError(): string | null {
      return pageHistoryController.versionError;
    },
    get pageHistorySettings(): NotesPageHistorySettings | null {
      return pageHistoryController.settings;
    },
    get pageHistorySettingsLoading(): boolean {
      return pageHistoryController.settingsLoading;
    },
    get pageHistorySettingsError(): string | null {
      return pageHistoryController.settingsError;
    },
    get pageHistoryActionLoading(): boolean {
      return pageHistoryController.actionLoading;
    },
    get pageHistoryActionError(): string | null {
      return pageHistoryController.actionError;
    },
    get focusBlockId(): string | null {
      return focusRequest.blockId;
    },
    get focusRequestId(): number {
      return focusRequest.requestId;
    },
    get focusSelection(): NotesTextSelection | null {
      return focusRequest.selection;
    },
    get titleFocusPageId(): string | null {
      return titleFocusRequest.pageId;
    },
    get titleFocusRequestId(): number {
      return titleFocusRequest.requestId;
    },
    pageTitleDraftForPage,
    get canUndoNotesEdit(): boolean {
      return undoController.canUndo();
    },
    get canRedoNotesEdit(): boolean {
      return undoController.canRedo();
    },
    load,
    ensureLoaded,
    loadMoreWorkspaceWindow,
    selectPage,
    showSelectedPageAs,
    createPage,
    createSubpage,
    createFolder,
    renameFolder,
    moveFolder,
    deleteFolder,
    importHtmlPage,
    importNotionApi,
    importNotionExportFolder,
    exportHtmlArchive,
    exportJsonGraph,
    exportAgentBridge,
    createChildPageFromBlock,
    applyPageTemplate,
    createPageTemplateFromCurrentPage,
    updatePageTemplateFromCurrentPage,
    renamePageTemplate,
    duplicatePageTemplate,
    deletePageTemplate,
    reloadPageTemplates,
    ensureOptionalSubsystem,
    setPagePanelSubsystemOpen,
    loadLocalUser,
    updateLocalUserDisplayName,
    loadPageHistorySettings: pageHistoryController.loadSettings,
    updatePageHistoryRetention: pageHistoryController.updateRetention,
    reloadPageHistory: pageHistoryController.reloadSnapshots,
    loadPageHistoryVersion: pageHistoryController.loadVersion,
    restorePageHistoryVersion: pageHistoryController.restoreVersion,
    copyPageHistoryBlocks: pageHistoryController.copyBlocks,
    renamePage,
    duplicatePage,
    movePage,
    movePageToFolder,
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
    reloadPageAliases,
    reloadUnresolvedLinks,
    addPageAlias,
    deletePageAlias,
    resolveUnresolvedLink,
    reloadComments,
    reloadSuggestions,
    setCommentsIncludeResolved,
    setSuggestionsIncludeDecided,
    setActiveCommentParent,
    startBlockComment,
    startInlineComment,
    startInlineSuggestion,
    cancelSuggestionDraft,
    createSuggestion,
    acceptSuggestion,
    rejectSuggestion,
    createComment,
    replyToCommentThread,
    updateComment,
    deleteComment,
    setCommentThreadResolved,
    markCommentThreadsRead,
    markVisibleCommentThreadsRead,
    search,
    setSearchIncludeResolvedComments,
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
    setFolderCollapsed,
    setSidebarPageCollapsed,
    updatePageIcon,
    updatePageCover,
    openBlockLink,
    openNotesLink,
    undoNotesEdit,
    redoNotesEdit,
    setPageTitleDraft,
    clearPageTitleDraft,
    updateBlockText, updateBlockRichText,
    insertPageMention,
    insertDateMention,
    insertObjectMention,
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
    splitTextBlockAtSelection,
    pastePlainTextIntoBlock,
    pasteRichHtmlIntoBlock,
    pasteBlockSelection,
    deleteBlock,
    deleteBlockSelection,
    mergeBlockWithPrevious,
    nestBlock,
    outdentBlock,
    moveBlockUp,
    moveBlockDown,
    moveBlockSelection,
    dropBlockWithinSiblings,
    dropBlockOnBlock,
    moveBlockToPage,
    duplicateBlock,
    duplicateBlockSelection,
    addTemplateChild,
    useTemplateBlock,
    addButtonChild,
    updateButtonIcon,
    updateButtonInsertPosition,
    useButtonBlock,
    createLinkedDatabaseViewAfter,
    convertUnsupportedBlock,
    blockPlainText,
    updateBookmark,
    updateLinkPreviewUrl,
    updateEmbedUrl,
    updateEquationExpression,
    updateMedia,
    updateTableCell,
    updateTableCellRichText,
    addTableRow,
    removeTableRow,
    addTableColumn,
    removeTableColumn,
    addColumn,
    removeColumn,
    moveColumn,
    resizeColumn,
    moveBlockToColumn,
    updateTabLabel,
    updateTabIcon,
    addTab,
    removeTab,
    moveTab,
    moveBlockToTab,
  };
}
