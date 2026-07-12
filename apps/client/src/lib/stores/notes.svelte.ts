import {
  applyNotesPageTemplate,
  archiveNotesPage,
  createNotesFolder,
  createNotesChildPageFromBlock,
  createNotesPage,
  createNotesPageTemplateFromPage,
  deleteNotesFolder,
  deleteNotesPageTemplate,
  duplicateNotesPage,
  duplicateNotesPageTemplate,
  getNotesBlockFrontier,
  getNotesBlockOutlineFrontier,
  getNotesPageBreadcrumb,
  listNotesPageTemplates,
  listNotesSidebarPages,
  loadNotesWorkspaceShell,
  loadNotesPage,
  openNotesPage,
  hydrateNotesBlocks,
  moveNotesPage,
  permanentlyDeleteNotesPage,
  trashNotesPage,
  updateNotesFolder,
  updateNotesPage,
  updateNotesPageTemplate,
} from "$lib/api/notes";
import { invalidateNotesPageCoverAssetUrl } from "$lib/api/notes-page-covers";
import { invalidateNotesPageIconAssetUrl } from "$lib/api/notes-page-icons";
import { blockPlainText } from "$lib/notes/block-factory";
import type { NotesBlockLinkTarget, NotesPageLinkTarget } from "$lib/notes/block-link";
import {
  buildNotesChildIdsByParent,
  parentIdForBlock,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import {
  flattenNotesBlockOutlines,
  notesBlockOutlineFromBlock,
  type NotesBlockOutlineItem,
} from "$lib/notes/block-outline";
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
import { createNotesArchiveController } from "./notes-store-archive.svelte";
import { createNotesSearchController } from "./notes-store-search.svelte";
import { createNotesLinksController } from "./notes-store-links.svelte";
import { createNotesCollaborationController } from "./notes-store-collaboration.svelte";
import { createNotesTransferActions } from "./notes-store-transfer-actions";
import {
  createNotesOptionalSubsystemController,
  type NotesOptionalSubsystem,
  type NotesPagePanelSubsystem,
} from "./notes-store-optional-subsystems";
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
  NotesBlockOutline,
  NotesColumnBlockItems,
  NotesFolder,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesLoadedPage,
  NotesPage,
  NotesPageBreadcrumbItem,
  NotesPageCover,
  NotesPageHistorySettings,
  NotesPageHistorySnapshot,
  NotesPageIcon,
  NotesPageTemplate,
  NotesParent,
  NotesTabBlockItems,
  NotesTableRowBlock,
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

const BLOCK_SAVE_DEBOUNCE_MS = 350;
const BLOCK_VIRTUALIZATION_THRESHOLD = 120;
const BLOCK_HYDRATION_LIMIT = 200;

let pages = $state<NotesPage[]>([]);
let allPages = $state<NotesPage[]>([]);
let folders = $state<NotesFolder[]>([]);
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
let blocksById = $state<Record<string, NotesBlock>>({});
let childIdsByParentId = $state<Record<string, string[]>>({});
let blockOutlines = $state<NotesBlockOutline[]>([]);
let flatBlockOutlines = $state<NotesBlockOutlineItem[]>([]);
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
let blockHydrationRequestId = 0;
let loadPromise: Promise<void> | null = null;
let pageTemplatesRequestId = 0;
let titleFocusRequest = $state<{ pageId: string | null; requestId: number }>({
  pageId: null,
  requestId: 0,
});
let pageTitleDraft = $state<{ pageId: string; title: string } | null>(null);
const preferences = getPreferences();
const projects = getProjects();
const archiveController = createNotesArchiveController();
const searchController = createNotesSearchController();
const linksController = createNotesLinksController({
  readSelectedPageId: () => selectedPageId,
  readSelectedProjectId: () => projects.selectedProjectId,
  readAllPages: () => allPages,
  reloadSelectedPage: (pageId) => loadPageTree(pageId),
  scheduleVisibleMetadataRefresh: () => sidebarRefreshCoordinator.schedule("visible-metadata"),
});
const {
  reloadArchivedPages,
  loadMoreArchivedPages,
  reloadTrashedPages,
  loadMoreTrashedPages,
} = archiveController;

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

function replaceBlockOutlines(outlines: readonly NotesBlockOutline[], pageId: string): void {
  blockOutlines = [...outlines];
  flatBlockOutlines = flattenNotesBlockOutlines(blockOutlines, pageId);
}

function mergeBlockOutlines(outlines: readonly NotesBlockOutline[], pageId: string): void {
  const next = new Map(blockOutlines.map((outline) => [outline.id, outline]));
  for (const outline of outlines) next.set(outline.id, outline);
  replaceBlockOutlines([...next.values()], pageId);
}

function syncHydratedBlockOutlines(pageId: string): void {
  const next = new Map(blockOutlines.map((outline) => [outline.id, outline]));
  for (const [parentId, childIds] of Object.entries(childIdsByParentId)) {
    childIds.forEach((blockId, index) => {
      const block = blocksById[blockId];
      if (!block) return;
      const outline = notesBlockOutlineFromBlock(block, pageId, (index + 1) * 1_000);
      next.set(blockId, {
        ...outline,
        parent: parentId === pageId
          ? { type: "page_id", page_id: pageId }
          : { type: "block_id", block_id: parentId },
      });
    });
  }
  replaceBlockOutlines([...next.values()], pageId);
}

async function loadOutlineDescendantFrontiers(
  pageId: string,
  requestId: number,
): Promise<void> {
  let frontier = blockOutlines
    .filter((outline) => outline.has_children && outline.type !== "child_page")
    .map((outline) => outline.id);
  const visited = new Set<string>();
  while (frontier.length > 0) {
    const parentIds = frontier.filter((id) => !visited.has(id));
    if (parentIds.length === 0) break;
    parentIds.forEach((id) => visited.add(id));
    const children = await getNotesBlockOutlineFrontier(pageId, parentIds);
    if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
    mergeBlockOutlines(children, pageId);
    frontier = children
      .filter((outline) => outline.has_children && outline.type !== "child_page")
      .map((outline) => outline.id);
  }
  if (requestId !== pageWorkGeneration || pageId !== selectedPageId) return;
  const initialIds = flatBlockOutlines.length < BLOCK_VIRTUALIZATION_THRESHOLD
    ? flatBlockOutlines.map((item) => item.outline.id)
    : flatBlockOutlines.slice(0, 80).map((item) => item.outline.id);
  await hydrateBlockRange(initialIds, requestId);
}

async function hydrateBlockRange(
  blockIds: readonly string[],
  generation: number = pageWorkGeneration,
): Promise<void> {
  const pageId = selectedPageId;
  if (!pageId || generation !== pageWorkGeneration) return;
  const outlinesById = new Map(blockOutlines.map((outline) => [outline.id, outline]));
  const retained = new Set(blockIds);
  if (focusRequest.blockId) retained.add(focusRequest.blockId);
  for (const blockId of [...retained]) {
    let parent = outlinesById.get(blockId)?.parent;
    while (parent?.type === "block_id") {
      if (retained.has(parent.block_id)) break;
      retained.add(parent.block_id);
      parent = outlinesById.get(parent.block_id)?.parent;
    }
  }
  const boundedIds = [...retained].slice(0, BLOCK_HYDRATION_LIMIT);
  const missingIds = boundedIds.filter((id) => !blocksById[id]);
  const requestId = ++blockHydrationRequestId;
  const hydrated = missingIds.length > 0
    ? await hydrateNotesBlocks({ page_id: pageId, block_ids: missingIds })
    : [];
  if (
    requestId !== blockHydrationRequestId
    || generation !== pageWorkGeneration
    || pageId !== selectedPageId
  ) return;
  const retainedIds = new Set(boundedIds);
  const nextBlocks = flatBlockOutlines.length >= BLOCK_VIRTUALIZATION_THRESHOLD
    ? Object.fromEntries(Object.entries(blocksById).filter(([id]) => retainedIds.has(id)))
    : { ...blocksById };
  for (const block of hydrated) nextBlocks[block.id] = block;
  blocksById = nextBlocks;
  childIdsByParentId = buildNotesChildIdsByParent(Object.values(blocksById));
  if (optionalSubsystemController.isPanelOpen("comments")) {
    void collaborationController.reloadComments(pageId);
  }
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
  const shell = await loadNotesWorkspaceShell({
    project_id: projects.selectedProjectId,
    expanded_page_ids: [...sidebarExpandedPageIds],
    seed_page_ids: sidebarSeedPageIds(),
    selected_page_id: selectedPageIdOverride,
  });
  const mergedPages = [...new Map([...allPages, ...shell.pages].map((page) => [page.id, page])).values()];
  const mergedFolders = [...new Map([...folders, ...shell.folders].map((folder) => [folder.id, folder])).values()];
  replacePages(mergedPages);
  replaceAllPages(mergedPages);
  replaceFolders(mergedFolders);
  sidebarPageIdsWithChildren = [...new Set([...sidebarPageIdsWithChildren, ...shell.page_ids_with_children])];
  sidebarMissingParentPageIds = [...shell.missing_parent_page_ids];
  sidebarTrashedParentPageIds = [...shell.trashed_parent_page_ids];
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
      blockOutlines = [];
      syncHydratedBlockOutlines(result.loadedPage.page.id);
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
    if (result.blocks || result.placements || result.removedBlockIds) {
      const removed = new Set(result.removedBlockIds ?? []);
      blockOutlines = blockOutlines.filter((outline) => !removed.has(outline.id));
      syncHydratedBlockOutlines(loadedPage.id);
    }
  }
  sidebarRefreshCoordinator.schedule(result.sidebarImpact ?? "none");
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
  replaceBlockOutlines(loaded.outlines, pageId);
  pageBreadcrumbItems = [...loaded.breadcrumb];
  void loadOutlineDescendantFrontiers(pageId, requestId).catch((error) => {
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

async function reloadPageBreadcrumb(pageId: string | null = selectedPageId): Promise<void> {
  if (!pageId) {
    pageBreadcrumbItems = [];
    return;
  }
  pageBreadcrumbItems = [...await getNotesPageBreadcrumb(pageId)];
}

async function load(): Promise<void> {
  const requestId = ++loadRequestId;
  pageWorkGeneration += 1;
  blockHydrationRequestId += 1;
  blockOutlines = [];
  flatBlockOutlines = [];
  optionalSubsystemController.resetAll();
  linksController.resetAll();
  collaborationController.resetPageState();
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
      page_cursor: pageCursor ?? "end",
      folder_cursor: folderCursor ?? "end",
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

async function loadOptionalSubsystem(
  subsystem: NotesOptionalSubsystem,
  pageId: string | null,
): Promise<void> {
  switch (subsystem) {
    case "templates":
      await reloadPageTemplates();
      break;
    case "local-user":
      await collaborationController.loadLocalUser();
      break;
    case "history-settings":
      await pageHistoryController.loadSettings();
      break;
    case "undo":
      await undoController.hydrate(pageId);
      break;
    case "links":
      await Promise.all([
        linksController.reloadBacklinks(pageId),
        linksController.reloadPageAliases(pageId),
        linksController.reloadUnresolvedLinks(pageId),
        linksController.reloadLinkResolutionPages(),
      ]);
      break;
    case "comments":
      await Promise.all([
        collaborationController.loadLocalUser(),
        collaborationController.reloadComments(pageId),
      ]);
      break;
    case "suggestions":
      await collaborationController.reloadSuggestions(pageId);
      break;
    case "page-history":
      if (pageId) await pageHistoryController.reloadSnapshots(pageId);
      break;
    case "destinations":
      await linksController.reloadLinkResolutionPages();
      break;
  }
}

async function ensureOptionalSubsystem(
  subsystem: NotesOptionalSubsystem,
  pageId: string | null = selectedPageId,
): Promise<void> {
  return optionalSubsystemController.ensure(subsystem, pageId);
}

function setPagePanelSubsystemOpen(
  subsystem: NotesPagePanelSubsystem,
  open: boolean,
): void {
  optionalSubsystemController.setPanelOpen(subsystem, open);
}

async function refreshOpenLinks(): Promise<void> {
  const pageId = selectedPageId;
  if (!pageId || !optionalSubsystemController.isPanelOpen("links")) return;
  const generation = pageWorkGeneration;
  await Promise.all([
    linksController.reloadBacklinks(pageId),
    linksController.reloadPageAliases(pageId),
    linksController.reloadUnresolvedLinks(pageId),
    linksController.reloadLinkResolutionPages(),
  ]);
  if (generation !== pageWorkGeneration || pageId !== selectedPageId) return;
  optionalSubsystemController.markPageSubsystemLoaded("links", pageId, generation);
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
  blockHydrationRequestId += 1;
  blockOutlines = [];
  flatBlockOutlines = [];
  optionalSubsystemController.resetPageScoped();
  undoController.reset(pageId);
  pageHistoryController.resetPageState();
  linksController.resetPageState();
  collaborationController.resetPageState();
  if (!pageId) {
    loadedPage = null;
    primaryContentReady = false;
    pageBreadcrumbItems = [];
    linksController.resetAll();
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
  blockOutlines = [];
  flatBlockOutlines = [];
  undoController.reset(loaded.page.id);
  pageHistoryController.resetPageState();
  optionalSubsystemController.resetPageScoped();
  openSelectedPage(loaded.page.id, openMode);
  recordRecentPage(loaded.page.id);
  applyPostMutation({
    loadedPage: loaded,
    pages: [loaded.page],
    sidebarImpact,
  });
  await reloadPageBreadcrumb(loaded.page.id);
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
  archiveController.prependTrashedPage(trashedPage);
  archiveController.removeArchivedPages(new Set([pageId]));
  applyPostMutation({ removedPageIds: [pageId], sidebarImpact: "hierarchy" });
  const nextSelected = preferredNextSelected && pages.some((page) => page.id === preferredNextSelected)
    ? preferredNextSelected
    : pages[0]?.id ?? null;
  await selectPage(nextSelected);
}

async function archivePage(pageId: string): Promise<void> {
  const archivedPage = await archiveNotesPage(pageId, true);
  archiveController.prependArchivedPage(archivedPage);
  const nextSelected = nextSelectedNotesPageId(pages, pageId);
  applyPostMutation({ removedPageIds: [pageId], sidebarImpact: "hierarchy" });
  await selectPage(nextSelected);
}

async function unarchivePage(pageId: string): Promise<void> {
  const restoredPage = await archiveNotesPage(pageId, false);
  archiveController.removeArchivedPages(new Set([pageId]));
  pageWorkGeneration += 1;
  undoController.reset(restoredPage.id);
  pageHistoryController.resetPageState();
  optionalSubsystemController.resetPageScoped();
  openSelectedPage(restoredPage.id, defaultNotesPageOpenMode(notesPageProjectId(restoredPage)));
  applyPostMutation({ pages: [restoredPage], sidebarImpact: "hierarchy" });
  await loadPageTree(restoredPage.id);
  recordRecentPage(restoredPage.id);
  requestPageLoadFocus();
}

async function restorePage(pageId: string): Promise<void> {
  const restoredPage = await trashNotesPage(pageId, false);
  archiveController.removeTrashedPages(new Set([pageId]));
  pageWorkGeneration += 1;
  undoController.reset(restoredPage.id);
  pageHistoryController.resetPageState();
  optionalSubsystemController.resetPageScoped();
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
  archiveController.removeArchivedPages(deletedPageIdSet);
  archiveController.removeTrashedPages(deletedPageIdSet);
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
  if (!archiveController.archiveLoaded && !archiveController.archiveLoading) {
    await reloadArchivedPages();
  }
}

function closeArchive(): void {
  viewMode = "pages";
}

async function openTrash(): Promise<void> {
  viewMode = "trash";
  if (!archiveController.trashLoaded && !archiveController.trashLoading) {
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

function outlineSubtreeIds(rootBlockIds: readonly string[]): string[] {
  const roots = new Set(rootBlockIds);
  const outlinesById = new Map(blockOutlines.map((outline) => [outline.id, outline]));
  const included = new Set<string>();
  for (const item of flatBlockOutlines) {
    let current: NotesBlockOutline | undefined = item.outline;
    while (current) {
      if (roots.has(current.id)) {
        included.add(item.outline.id);
        break;
      }
      const parentId: string | null = current.parent.type === "block_id"
        ? current.parent.block_id
        : null;
      current = parentId ? outlinesById.get(parentId) : undefined;
    }
  }
  return [...included];
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

const optionalSubsystemController = createNotesOptionalSubsystemController({
  readPageGeneration: () => pageWorkGeneration,
  readSelectedPageId: () => selectedPageId,
  readSelectedProjectId: () => projects.selectedProjectId,
  load: loadOptionalSubsystem,
});

const transferActions = createNotesTransferActions({
  readSelectedPageId: () => selectedPageId,
  activateReturnedPage: (nextLoadedPage) => activateReturnedPage(nextLoadedPage, "hierarchy"),
  upsertPage: upsertPageInActiveCollections,
  showPages: () => {
    viewMode = "pages";
  },
  scheduleHierarchyRefresh: () => sidebarRefreshCoordinator.schedule("hierarchy"),
  queueDescendantHydration: () => queueDescendantHydration(),
  requestPageLoadFocus: () => requestPageLoadFocus(),
});

const blockActions = createNotesBlockActions({
  readSelectedPageId: () => selectedPageId,
  readBlocksById: () => blocksById,
  readChildIdsByParentId: () => childIdsByParentId,
  treeState,
  outlineSubtreeIds,
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

const collaborationController = createNotesCollaborationController({
  readSelectedPageId: () => selectedPageId,
  readBlocksById: () => blocksById,
  flushBlockSave,
  requestBlockFocus: (blockId) => requestBlockFocus(blockId),
  updateBlockRichText: blockActions.updateBlockRichText,
  isPanelOpen: (panel) => optionalSubsystemController.isPanelOpen(panel),
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
      return archiveController.archivedPages;
    },
    get archiveHasMore(): boolean {
      return archiveController.archiveHasMore;
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
      return archiveController.trashedPages;
    },
    get trashHasMore(): boolean {
      return archiveController.trashHasMore;
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
    get backlinks() {
      return linksController.backlinks;
    },
    get backlinksLoading(): boolean {
      return linksController.backlinksLoading;
    },
    get backlinksError(): string | null {
      return linksController.backlinksError;
    },
    get pageAliases() {
      return linksController.aliases;
    },
    get pageAliasesLoading(): boolean {
      return linksController.aliasesLoading;
    },
    get pageAliasesError(): string | null {
      return linksController.aliasesError;
    },
    get unresolvedLinks() {
      return linksController.unresolvedLinks;
    },
    get unresolvedLinksLoading(): boolean {
      return linksController.unresolvedLoading;
    },
    get unresolvedLinksError(): string | null {
      return linksController.unresolvedError;
    },
    get linkResolutionPages(): NotesPage[] {
      return linksController.destinations;
    },
    get destinationHasMore(): boolean {
      return linksController.destinationHasMore;
    },
    get commentThreads() {
      return collaborationController.commentThreads;
    },
    get commentsLoading(): boolean {
      return collaborationController.commentsLoading;
    },
    get commentsError(): string | null {
      return collaborationController.commentsError;
    },
    get commentsIncludeResolved(): boolean {
      return collaborationController.commentsIncludeResolved;
    },
    get activeCommentParent() {
      return collaborationController.activeCommentParent;
    },
    get activeCommentAnchor() {
      return collaborationController.activeCommentAnchor;
    },
    get suggestions() {
      return collaborationController.suggestions;
    },
    get suggestionsLoading(): boolean {
      return collaborationController.suggestionsLoading;
    },
    get suggestionsError(): string | null {
      return collaborationController.suggestionsError;
    },
    get suggestionsIncludeDecided(): boolean {
      return collaborationController.suggestionsIncludeDecided;
    },
    get activeSuggestionDraft() {
      return collaborationController.activeSuggestionDraft;
    },
    get localUser() {
      return collaborationController.localUser;
    },
    get localUserLoading(): boolean {
      return collaborationController.localUserLoading;
    },
    get localUserError(): string | null {
      return collaborationController.localUserError;
    },
    get searchResults() {
      return searchController.results;
    },
    get searchHasMore(): boolean {
      return searchController.hasMore;
    },
    get searchLoading(): boolean {
      return searchController.loading;
    },
    get searchError(): string | null {
      return searchController.error;
    },
    get searchIncludeResolvedComments(): boolean {
      return searchController.includeResolvedComments;
    },
    get blocksById(): Record<string, NotesBlock> {
      return blocksById;
    },
    get childIdsByParentId(): Record<string, string[]> {
      return childIdsByParentId;
    },
    get flatBlockOutlines(): NotesBlockOutlineItem[] {
      return flatBlockOutlines;
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
      return archiveController.archiveLoaded;
    },
    get archiveLoading(): boolean {
      return archiveController.archiveLoading;
    },
    get archiveError(): string | null {
      return archiveController.archiveError;
    },
    get trashLoaded(): boolean {
      return archiveController.trashLoaded;
    },
    get trashLoading(): boolean {
      return archiveController.trashLoading;
    },
    get trashError(): string | null {
      return archiveController.trashError;
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
    importHtmlPage: transferActions.importHtmlPage,
    importNotionApi: transferActions.importNotionApi,
    importNotionExportFolder: transferActions.importNotionExportFolder,
    exportHtmlArchive: transferActions.exportHtmlArchive,
    exportJsonGraph: transferActions.exportJsonGraph,
    exportAgentBridge: transferActions.exportAgentBridge,
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
    loadLocalUser: collaborationController.loadLocalUser,
    updateLocalUserDisplayName: collaborationController.updateLocalUserDisplayName,
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
    loadMoreArchivedPages,
    openTrash,
    closeTrash,
    reloadTrashedPages,
    loadMoreTrashedPages,
    reloadBacklinks: linksController.reloadBacklinks,
    reloadPageAliases: linksController.reloadPageAliases,
    reloadUnresolvedLinks: linksController.reloadUnresolvedLinks,
    reloadLinkResolutionPages: linksController.reloadLinkResolutionPages,
    loadMoreDestinationCandidates: linksController.loadMoreDestinationCandidates,
    addPageAlias: linksController.addPageAlias,
    deletePageAlias: linksController.deletePageAlias,
    resolveUnresolvedLink: linksController.resolveUnresolvedLink,
    reloadComments: collaborationController.reloadComments,
    reloadSuggestions: collaborationController.reloadSuggestions,
    setCommentsIncludeResolved: collaborationController.setCommentsIncludeResolved,
    setSuggestionsIncludeDecided: collaborationController.setSuggestionsIncludeDecided,
    setActiveCommentParent: collaborationController.setActiveCommentParent,
    startBlockComment: collaborationController.startBlockComment,
    startInlineComment: collaborationController.startInlineComment,
    startInlineSuggestion: collaborationController.startInlineSuggestion,
    cancelSuggestionDraft: collaborationController.cancelSuggestionDraft,
    createSuggestion: collaborationController.createSuggestion,
    acceptSuggestion: collaborationController.acceptSuggestion,
    rejectSuggestion: collaborationController.rejectSuggestion,
    createComment: collaborationController.createComment,
    replyToCommentThread: collaborationController.replyToCommentThread,
    updateComment: collaborationController.updateComment,
    deleteComment: collaborationController.deleteComment,
    setCommentThreadResolved: collaborationController.setCommentThreadResolved,
    markCommentThreadsRead: collaborationController.markCommentThreadsRead,
    markVisibleCommentThreadsRead: collaborationController.markVisibleCommentThreadsRead,
    search: searchController.search,
    loadMoreSearchResults: searchController.loadMoreSearchResults,
    setSearchIncludeResolvedComments: searchController.setIncludeResolvedComments,
    notesCommentParentKey: collaborationController.notesCommentParentKey,
    blockById,
    tableRowsForBlock,
    columnItemsForBlock,
    tabItemsForBlock,
    parentIdForBlock,
    previousBlockType,
    isOnlyBlock,
    focusBlock,
    hydrateBlockRange,
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
