import {
  getNotesPageBreadcrumb,
  listNotesSidebarPages,
  loadNotesPage,
  openNotesPage,
} from "$lib/api/notes";
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
import {
  notesPageOpenModeForSelection,
  notesDefaultOpenModeForProject,
  type NotesPageOpenMode,
} from "$lib/notes/page-open-mode";
import {
  notesPageProjectId,
} from "$lib/notes/project-membership";
import {
  nextNotesFocusRequest,
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
import { createNotesSidebarController } from "./notes-store-sidebar.svelte";
import { createNotesPageTemplatesController } from "./notes-store-page-templates.svelte";
import { createNotesFoldersController } from "./notes-store-folders.svelte";
import { createNotesPageActions } from "./notes-store-page-actions";
import { createNotesHydrationController } from "./notes-store-hydration";
import { createNotesWorkspaceController } from "./notes-store-workspace.svelte";
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
  initialNotesSelectedPageId,
  saveNotesSelectedPageId,
} from "./notes-store-page-state";
import { createNotesBlockPersistence } from "./notes-store-persistence";
import { invalidateNotesNotificationSchedule } from "$lib/notes/notification-schedule.svelte";
import type {
  NotesBlock,
  NotesBlockOutline,
  NotesColumnBlockItems,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesLoadedPage,
  NotesPage,
  NotesPageBreadcrumbItem,
  NotesPageHistorySettings,
  NotesPageHistorySnapshot,
  NotesTabBlockItems,
  NotesTableRowBlock,
  NotesWorkspaceShell,
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

let pages = $state<NotesPage[]>([]);
let allPages = $state<NotesPage[]>([]);
let selectedPageId = $state<string | null>(initialNotesSelectedPageId());
let pageOpenMode = $state<NotesPageOpenMode>("full");
let loadedPage = $state<NotesPage | null>(null);
let pageBreadcrumbItems = $state<NotesPageBreadcrumbItem[]>([]);
let blocksById = $state<Record<string, NotesBlock>>({});
let childIdsByParentId = $state<Record<string, string[]>>({});
let blockOutlines = $state<NotesBlockOutline[]>([]);
let flatBlockOutlines = $state<NotesBlockOutlineItem[]>([]);
let primaryContentReady = $state(false);
let viewMode = $state<NotesViewMode>("pages");
let focusRequest = $state<NotesFocusRequest>({
  blockId: null,
  requestId: 0,
  selection: null,
});
const START_OF_NOTES_BLOCK_SELECTION: NotesTextSelection = { start: 0, end: 0 };
let pageWorkGeneration = 0;
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
const sidebarController = createNotesSidebarController({
  reloadPages: () => reloadPages(),
});
const foldersController = createNotesFoldersController({
  setFolderCollapsed: (folderId, collapsed) => sidebarController.setFolderCollapsed(folderId, collapsed),
  scheduleHierarchyRefresh: () => sidebarRefreshCoordinator.schedule("hierarchy"),
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
  sidebarController.recordRecentPage(pageId);
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
  return sidebarController.seedPageIds();
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

async function hydrateBlockRange(
  blockIds: readonly string[],
  generation = pageWorkGeneration,
): Promise<void> {
  await hydrationController.hydrateBlockRange(blockIds, generation);
}

function queueDescendantHydration(
  pageId: string = selectedPageId ?? "",
  generation: number = pageWorkGeneration,
): void {
  hydrationController.queueDescendantHydration(pageId, generation);
}

async function reloadPages(selectedPageIdOverride: string | null = selectedPageId): Promise<void> {
  await workspaceController.reloadPages(selectedPageIdOverride);
}

function mergeReloadedWorkspaceShell(shell: NotesWorkspaceShell): void {
  const mergedPages = [...new Map([...allPages, ...shell.pages].map((page) => [page.id, page])).values()];
  replacePages(mergedPages);
  replaceAllPages(mergedPages);
  foldersController.merge(shell.folders);
  sidebarController.mergeMetadata({
    pageIdsWithChildren: shell.page_ids_with_children,
    missingParentPageIds: shell.missing_parent_page_ids,
    trashedParentPageIds: shell.trashed_parent_page_ids,
  });
}

function prepareWorkspaceLoad(): void {
  pageWorkGeneration += 1;
  hydrationController.invalidate();
  blockOutlines = [];
  flatBlockOutlines = [];
  optionalSubsystemController.resetAll();
  linksController.resetAll();
  collaborationController.resetPageState();
  pageHistoryController.resetPageState();
  sidebarRefreshCoordinator.cancel();
}

function applyInitialWorkspaceShell(
  shell: NotesWorkspaceShell,
  requestedSelection: string | null,
): string | null {
  replacePages(shell.pages);
  replaceAllPages(shell.pages);
  foldersController.replace(shell.folders);
  sidebarController.replaceMetadata({
    pageIdsWithChildren: shell.page_ids_with_children,
    missingParentPageIds: shell.missing_parent_page_ids,
    trashedParentPageIds: shell.trashed_parent_page_ids,
  });
  const selectionUnchanged = selectedPageId === requestedSelection;
  const nextSelected = selectionUnchanged ? shell.resolved_selected_page_id : selectedPageId;
  if (selectionUnchanged) saveSelectedPageId(nextSelected);
  return nextSelected;
}

function applyAdditionalWorkspaceShell(shell: NotesWorkspaceShell): void {
  const mergedPages = [...new Map([...allPages, ...shell.pages].map((page) => [page.id, page])).values()];
  replaceAllPages(mergedPages);
  replacePages(mergedPages);
  foldersController.merge(shell.folders);
  sidebarController.mergeMetadata({
    pageIdsWithChildren: shell.page_ids_with_children,
    missingParentPageIds: shell.missing_parent_page_ids,
    trashedParentPageIds: shell.trashed_parent_page_ids,
  });
}

function clearSelectedPageState(): void {
  loadedPage = null;
  pageBreadcrumbItems = [];
  pageHistoryController.resetPageState();
  blocksById = {};
  childIdsByParentId = {};
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
  void hydrationController.loadOutlineDescendantFrontiers(pageId, requestId).catch((error) => {
    if (requestId === pageWorkGeneration && pageId === selectedPageId) {
      workspaceController.setError(error instanceof Error ? error.message : String(error));
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

async function loadOptionalSubsystem(
  subsystem: NotesOptionalSubsystem,
  pageId: string | null,
): Promise<void> {
  switch (subsystem) {
    case "templates":
      await pageTemplatesController.reloadPageTemplates();
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
  hydrationController.invalidate();
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
  workspaceController.setLoading(true);
  workspaceController.setError(null);
  try {
    await loadPageTree(pageId, {
      focusOnLoad: true,
      focusBlockId: options.focusBlockId ?? null,
    });
    recordRecentPage(pageId);
  } catch (error) {
    workspaceController.setError(error instanceof Error ? error.message : String(error));
    throw error;
  } finally {
    workspaceController.setLoading(false);
  }
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

async function activateRestoredPage(page: NotesPage): Promise<void> {
  pageWorkGeneration += 1;
  undoController.reset(page.id);
  pageHistoryController.resetPageState();
  optionalSubsystemController.resetPageScoped();
  openSelectedPage(page.id, defaultNotesPageOpenMode(notesPageProjectId(page)));
  applyPostMutation({ pages: [page], sidebarImpact: "hierarchy" });
  await loadPageTree(page.id);
  recordRecentPage(page.id);
  requestPageLoadFocus();
}

function setFolderCollapsed(folderId: string, collapsed: boolean): void {
  sidebarController.setFolderCollapsed(folderId, collapsed);
}

function setSidebarPageCollapsed(pageId: string, collapsed: boolean): void {
  sidebarController.setPageCollapsed(pageId, collapsed);
}

function setPageFavorited(pageId: string, favorited: boolean): void {
  sidebarController.setPageFavorited(pageId, favorited);
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
    workspaceController.setError(message);
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
    workspaceController.setError(message);
  },
});

const optionalSubsystemController = createNotesOptionalSubsystemController({
  readPageGeneration: () => pageWorkGeneration,
  readSelectedPageId: () => selectedPageId,
  readSelectedProjectId: () => projects.selectedProjectId,
  load: loadOptionalSubsystem,
});

const workspaceController = createNotesWorkspaceController({
  readRequestState: () => ({
    projectId: projects.selectedProjectId,
    expandedPageIds: [...sidebarController.expandedPageIds],
    seedPageIds: sidebarSeedPageIds(),
    selectedPageId,
  }),
  prepareLoad: prepareWorkspaceLoad,
  applyInitialShell: applyInitialWorkspaceShell,
  applyAdditionalShell: applyAdditionalWorkspaceShell,
  mergeReloadedShell: mergeReloadedWorkspaceShell,
  loadSelectedPage: (pageId) => loadPageTree(pageId, { focusOnLoad: true }),
  clearSelectedPageState,
  readSelectedPageId: () => selectedPageId,
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

const pageTemplatesController = createNotesPageTemplatesController({
  readLoadedPage: () => loadedPage,
  flushPendingBlockSaves,
  activateReturnedPage: (nextLoadedPage) => activateReturnedPage(nextLoadedPage, "hierarchy"),
  queueDescendantHydration: () => queueDescendantHydration(),
  requestPageLoadFocus: () => requestPageLoadFocus(),
});

const pageActions = createNotesPageActions({
  readSelectedPageId: () => selectedPageId,
  readPages: () => pages,
  readAllPages: () => allPages,
  readLoadedPage: () => loadedPage,
  readFolders: () => foldersController.folders,
  readBlocksById: () => blocksById,
  defaultOpenMode: defaultNotesPageOpenMode,
  activateReturnedPage,
  activateRestoredPage,
  applyPostMutation,
  selectPage: (pageId) => selectPage(pageId),
  reloadPageBreadcrumb: (pageId) => reloadPageBreadcrumb(pageId),
  flushBlockSave,
  flushPendingBlockSaves,
  requestBlockFocus,
  requestTitleFocus,
  requestPageLoadFocus: () => requestPageLoadFocus(),
  queueDescendantHydration: () => queueDescendantHydration(),
  setFolderCollapsed,
  setSidebarPageCollapsed,
  prependArchivedPage: archiveController.prependArchivedPage,
  prependTrashedPage: archiveController.prependTrashedPage,
  removeArchivedPages: archiveController.removeArchivedPages,
  removeTrashedPages: archiveController.removeTrashedPages,
  removePagesFromActiveCollections,
  removeSidebarPageIds: sidebarController.removePageIds,
  scheduleHierarchyRefresh: () => sidebarRefreshCoordinator.schedule("hierarchy"),
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
  createChildPageFromBlock: pageActions.createChildPageFromBlock,
  createChildPageAfterBlock: pageActions.createChildPageAfterBlock,
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

const hydrationController = createNotesHydrationController({
  readPageGeneration: () => pageWorkGeneration,
  readSelectedPageId: () => selectedPageId,
  readBlockOutlines: () => blockOutlines,
  readFlatBlockOutlines: () => flatBlockOutlines,
  readBlocksById: () => blocksById,
  readFocusRequest: () => focusRequest,
  mergeBlockOutlines,
  replaceHydratedBlocks: (nextBlocksById, nextChildIdsByParentId) => {
    blocksById = nextBlocksById;
    childIdsByParentId = nextChildIdsByParentId;
  },
  setLoadError: (message) => {
    workspaceController.setError(message);
  },
  reloadOpenComments: (pageId) => {
    if (optionalSubsystemController.isPanelOpen("comments")) {
      void collaborationController.reloadComments(pageId);
    }
  },
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
    workspaceController.setError(error instanceof Error ? error.message : String(error));
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
    workspaceController.setError(error instanceof Error ? error.message : String(error));
    throw error;
  }
}

async function openBlockLink(target: NotesBlockLinkTarget): Promise<boolean> {
  return openNotesLink(target);
}

async function openNotesLink(target: NotesPageLinkTarget): Promise<boolean> {
  viewMode = "pages";
  await workspaceController.ensureLoaded();
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
    get folders() {
      return foldersController.folders;
    },
    get archivedPages(): NotesPage[] {
      return archiveController.archivedPages;
    },
    get archiveHasMore(): boolean {
      return archiveController.archiveHasMore;
    },
    get pageTemplates() {
      return pageTemplatesController.templates;
    },
    get workspacePages(): NotesPage[] {
      return allPages.filter((page) => page.parent.type === "workspace");
    },
    get favoritePageIds(): readonly string[] {
      return sidebarController.favoritePageIds;
    },
    get recentPageIds(): readonly string[] {
      return sidebarController.recentPageIds;
    },
    get collapsedFolderIds(): readonly string[] {
      return sidebarController.collapsedFolderIds;
    },
    get sidebarExpandedPageIds(): readonly string[] {
      return sidebarController.expandedPageIds;
    },
    get sidebarPageIdsWithChildren(): readonly string[] {
      return sidebarController.pageIdsWithChildren;
    },
    get sidebarMissingParentPageIds(): readonly string[] {
      return sidebarController.missingParentPageIds;
    },
    get sidebarTrashedParentPageIds(): readonly string[] {
      return sidebarController.trashedParentPageIds;
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
      return workspaceController.loaded;
    },
    get loading(): boolean {
      return workspaceController.loading;
    },
    get loadError(): string | null {
      return workspaceController.error;
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
      return pageTemplatesController.loading;
    },
    get pageTemplatesError(): string | null {
      return pageTemplatesController.error;
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
    load: workspaceController.load,
    ensureLoaded: workspaceController.ensureLoaded,
    loadMoreWorkspaceWindow: workspaceController.loadMoreWorkspaceWindow,
    selectPage,
    showSelectedPageAs,
    createPage: pageActions.createPage,
    createSubpage: pageActions.createSubpage,
    createFolder: foldersController.createFolder,
    renameFolder: foldersController.renameFolder,
    moveFolder: foldersController.moveFolder,
    deleteFolder: foldersController.deleteFolder,
    importHtmlPage: transferActions.importHtmlPage,
    importNotionApi: transferActions.importNotionApi,
    importNotionExportFolder: transferActions.importNotionExportFolder,
    exportHtmlArchive: transferActions.exportHtmlArchive,
    exportJsonGraph: transferActions.exportJsonGraph,
    exportAgentBridge: transferActions.exportAgentBridge,
    createChildPageFromBlock: pageActions.createChildPageFromBlock,
    applyPageTemplate: pageTemplatesController.applyPageTemplate,
    createPageTemplateFromCurrentPage: pageTemplatesController.createPageTemplateFromCurrentPage,
    updatePageTemplateFromCurrentPage: pageTemplatesController.updatePageTemplateFromCurrentPage,
    renamePageTemplate: pageTemplatesController.renamePageTemplate,
    duplicatePageTemplate: pageTemplatesController.duplicatePageTemplate,
    deletePageTemplate: pageTemplatesController.deletePageTemplate,
    reloadPageTemplates: pageTemplatesController.reloadPageTemplates,
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
    renamePage: pageActions.renamePage,
    duplicatePage: pageActions.duplicatePage,
    movePage: pageActions.movePage,
    movePageToFolder: pageActions.movePageToFolder,
    archivePage: pageActions.archivePage,
    unarchivePage: pageActions.unarchivePage,
    trashPage: pageActions.trashPage,
    restorePage: pageActions.restorePage,
    permanentlyDeletePage: pageActions.permanentlyDeletePage,
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
    updatePageIcon: pageActions.updatePageIcon,
    updatePageCover: pageActions.updatePageCover,
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
