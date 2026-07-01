import { orderedNotesPagesById } from "./page-navigation";
import { buildNotesPageTree, type NotesPageTreeItem } from "./page-tree";
import type { NotesPage } from "./types";

export const NOTES_SIDEBAR_RECENT_LIMIT = 5;

export interface NotesSidebarNavigationPlan {
  searchQuery: string;
  showNavigationSections: boolean;
  favoritePages: NotesPage[];
  recentPages: NotesPage[];
  treeItems: NotesPageTreeItem[];
  showPagesHeading: boolean;
}

export interface NotesSidebarNavigationInput {
  pages: readonly NotesPage[];
  favoritePageIds: readonly string[];
  recentPageIds: readonly string[];
  collapsedPageIds: readonly string[];
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
    };
  }

  const favoritePages = orderedNotesPagesById(input.pages, input.favoritePageIds);
  const favoritePageIdSet = new Set(favoritePages.map((page) => page.id));
  const recentPages = orderedNotesPagesById(input.pages, input.recentPageIds)
    .filter((page) => !favoritePageIdSet.has(page.id))
    .slice(0, input.recentLimit ?? NOTES_SIDEBAR_RECENT_LIMIT);
  const treeItems = buildNotesPageTree(input.pages, {
    activePageId: input.activePageId,
    collapsedPageIds: input.collapsedPageIds,
    titleForPage: input.titleForPage,
  });

  return {
    searchQuery,
    showNavigationSections,
    favoritePages,
    recentPages,
    treeItems,
    showPagesHeading: favoritePages.length > 0 || recentPages.length > 0,
  };
}
