import { orderedNotesPagesById } from "./page-navigation";
import {
  buildNotesPageTree,
  type NotesPageParentStatus,
  type NotesPageTreeItem,
} from "./page-tree";
import type { NotesPage, NotesParent } from "./types";

export const NOTES_SIDEBAR_RECENT_LIMIT = 5;

export interface NotesSidebarNavigationPlan {
  searchQuery: string;
  showNavigationSections: boolean;
  favoritePages: NotesPage[];
  recentPages: NotesPage[];
  treeItems: NotesPageTreeItem[];
  showPagesHeading: boolean;
  parentStatusByPageId: Record<string, NotesPageParentStatus>;
}

export interface NotesSidebarNavigationInput {
  pages: readonly NotesPage[];
  favoritePageIds: readonly string[];
  recentPageIds: readonly string[];
  expandedPageIds: readonly string[];
  pageIdsWithChildren: readonly string[];
  missingParentPageIds: readonly string[];
  trashedParentPageIds: readonly string[];
  activePageId: string | null;
  search: string;
  titleForPage: (page: NotesPage) => string;
  recentLimit?: number;
}

/**
 * Plan the visible sidebar sections from durable navigation metadata.
 */
export function planNotesSidebarNavigation(
  input: NotesSidebarNavigationInput,
): NotesSidebarNavigationPlan {
  const searchQuery = input.search.trim();
  const showNavigationSections = searchQuery.length === 0;
  if (!showNavigationSections) {
    return {
      searchQuery,
      showNavigationSections,
      favoritePages: [],
      recentPages: [],
      treeItems: [],
      showPagesHeading: false,
      parentStatusByPageId: {},
    };
  }

  const favoritePages = orderedNotesPagesById(input.pages, input.favoritePageIds);
  const favoritePageIdSet = new Set(favoritePages.map((page) => page.id));
  const recentPages = orderedNotesPagesById(input.pages, input.recentPageIds)
    .filter((page) => !favoritePageIdSet.has(page.id))
    .slice(0, input.recentLimit ?? NOTES_SIDEBAR_RECENT_LIMIT);
  const treeItems = buildNotesPageTree(input.pages, {
    activePageId: input.activePageId,
    expandedPageIds: input.expandedPageIds,
    pageIdsWithChildren: input.pageIdsWithChildren,
    missingParentPageIds: input.missingParentPageIds,
    trashedParentPageIds: input.trashedParentPageIds,
    titleForPage: input.titleForPage,
  });
  const parentStatusEntries: Array<[string, NotesPageParentStatus]> = input.pages.flatMap(
    (page) => {
      const parentStatus = notesPageParentStatus(page.parent, {
        missingParentPageIds: input.missingParentPageIds,
        trashedParentPageIds: input.trashedParentPageIds,
      });
      return parentStatus ? [[page.id, parentStatus]] : [];
    },
  );
  const parentStatusByPageId: Record<string, NotesPageParentStatus> =
    Object.fromEntries(parentStatusEntries);

  return {
    searchQuery,
    showNavigationSections,
    favoritePages,
    recentPages,
    treeItems,
    showPagesHeading: favoritePages.length > 0 || recentPages.length > 0,
    parentStatusByPageId,
  };
}

interface NotesPageParentStatusInput {
  missingParentPageIds: readonly string[];
  trashedParentPageIds: readonly string[];
}

function notesPageParentStatus(
  parent: NotesParent,
  input: NotesPageParentStatusInput,
): NotesPageParentStatus | null {
  if (parent.type !== "page_id") return null;
  if (input.trashedParentPageIds.includes(parent.page_id)) return "trashed";
  if (input.missingParentPageIds.includes(parent.page_id)) return "missing";
  return null;
}
