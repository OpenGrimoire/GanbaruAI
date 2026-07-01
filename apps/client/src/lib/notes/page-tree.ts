import { notesPageTitle } from "./page-title";
import type { NotesPage } from "./types";

const SIDEBAR_COLLAPSED_PAGE_IDS_CONFIG_KEY = "notes.sidebarCollapsedPageIds";

export interface NotesPageTreeItem {
  page: NotesPage;
  depth: number;
  hasChildren: boolean;
  collapsed: boolean;
  matchesQuery: boolean;
  descendantMatchesQuery: boolean;
}

export interface NotesPageTreeOptions {
  collapsedPageIds?: readonly string[];
  activePageId?: string | null;
  query?: string;
  untitledTitle?: string;
  titleForPage?: (page: NotesPage) => string;
}

/** Return the config key that stores collapsed sidebar page ids. */
export function notesSidebarCollapsedPageIdsConfigKey(): string {
  return SIDEBAR_COLLAPSED_PAGE_IDS_CONFIG_KEY;
}

/** Parse persisted collapsed sidebar page ids from config defensively. */
export function parseStoredNotesSidebarCollapsedPageIds(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return [...new Set(value.filter((item): item is string => typeof item === "string"))];
}

/** Build the flat visible tree for the Notes sidebar. */
export function buildNotesPageTree(
  pages: readonly NotesPage[],
  options: NotesPageTreeOptions = {},
): NotesPageTreeItem[] {
  const titleForPage = options.titleForPage
    ?? ((page: NotesPage) => notesPageTitle(page, options.untitledTitle ?? "Untitled"));
  const normalizedQuery = normalizeSearchText(options.query ?? "");
  const pageById = new Map(pages.map((page) => [page.id, page]));
  const childrenByParentId = new Map<string | null, NotesPage[]>();
  for (const page of pages) {
    const parentId = sidebarParentPageId(page, pageById);
    const siblings = childrenByParentId.get(parentId) ?? [];
    siblings.push(page);
    childrenByParentId.set(parentId, siblings);
  }

  const collapsedPageIds = new Set(options.collapsedPageIds ?? []);
  const activeAncestorIds = activePageAncestorIds(pageById, options.activePageId ?? null);
  const result: NotesPageTreeItem[] = [];
  appendTreeItems({
    parentId: null,
    depth: 0,
    childrenByParentId,
    collapsedPageIds,
    activeAncestorIds,
    normalizedQuery,
    titleForPage,
    result,
  });
  return result;
}

interface AppendTreeItemsOptions {
  parentId: string | null;
  depth: number;
  childrenByParentId: ReadonlyMap<string | null, readonly NotesPage[]>;
  collapsedPageIds: ReadonlySet<string>;
  activeAncestorIds: ReadonlySet<string>;
  normalizedQuery: string;
  titleForPage: (page: NotesPage) => string;
  result: NotesPageTreeItem[];
}

function appendTreeItems(options: AppendTreeItemsOptions): void {
  const children = options.childrenByParentId.get(options.parentId) ?? [];
  for (const page of children) {
    const childPages = options.childrenByParentId.get(page.id) ?? [];
    const hasChildren = childPages.length > 0;
    const matchesQuery = pageMatchesQuery(page, options.normalizedQuery, options.titleForPage);
    const descendantMatchesQuery = hasChildren
      && childPages.some((child) =>
        pageOrDescendantMatchesQuery(
          child,
          options.childrenByParentId,
          options.normalizedQuery,
          options.titleForPage,
        )
      );
    if (options.normalizedQuery && !matchesQuery && !descendantMatchesQuery) continue;
    const collapsed = hasChildren
      && options.collapsedPageIds.has(page.id)
      && !options.normalizedQuery
      && !options.activeAncestorIds.has(page.id);
    options.result.push({
      page,
      depth: options.depth,
      hasChildren,
      collapsed,
      matchesQuery,
      descendantMatchesQuery,
    });
    if (hasChildren && !collapsed) {
      appendTreeItems({
        ...options,
        parentId: page.id,
        depth: options.depth + 1,
      });
    }
  }
}

function pageOrDescendantMatchesQuery(
  page: NotesPage,
  childrenByParentId: ReadonlyMap<string | null, readonly NotesPage[]>,
  normalizedQuery: string,
  titleForPage: (page: NotesPage) => string,
): boolean {
  if (!normalizedQuery) return true;
  if (pageMatchesQuery(page, normalizedQuery, titleForPage)) return true;
  return (childrenByParentId.get(page.id) ?? []).some((child) =>
    pageOrDescendantMatchesQuery(child, childrenByParentId, normalizedQuery, titleForPage)
  );
}

function pageMatchesQuery(
  page: NotesPage,
  normalizedQuery: string,
  titleForPage: (page: NotesPage) => string,
): boolean {
  if (!normalizedQuery) return true;
  return normalizeSearchText(titleForPage(page)).includes(normalizedQuery);
}

function sidebarParentPageId(
  page: NotesPage,
  pageById: ReadonlyMap<string, NotesPage>,
): string | null {
  if (page.parent.type !== "page_id") return null;
  const parentId = page.parent.page_id;
  if (!pageById.has(parentId) || parentId === page.id) return null;
  if (pageHasParentCycle(page, pageById)) return null;
  return parentId;
}

function pageHasParentCycle(
  page: NotesPage,
  pageById: ReadonlyMap<string, NotesPage>,
): boolean {
  const seen = new Set([page.id]);
  let cursor: NotesPage | undefined = page;
  while (cursor?.parent.type === "page_id") {
    const parentId = cursor.parent.page_id;
    if (seen.has(parentId)) return true;
    seen.add(parentId);
    cursor = pageById.get(parentId);
  }
  return false;
}

function activePageAncestorIds(
  pageById: ReadonlyMap<string, NotesPage>,
  activePageId: string | null,
): Set<string> {
  const result = new Set<string>();
  if (!activePageId) return result;
  const activePage = pageById.get(activePageId);
  if (!activePage) return result;
  const seen = new Set([activePage.id]);
  let cursor: NotesPage | undefined = activePage;
  while (cursor?.parent.type === "page_id") {
    const parentId = cursor.parent.page_id;
    if (seen.has(parentId)) return result;
    const parent = pageById.get(parentId);
    if (!parent) return result;
    result.add(parent.id);
    seen.add(parent.id);
    cursor = parent;
  }
  return result;
}

function normalizeSearchText(value: string): string {
  return value.trim().toLowerCase();
}
