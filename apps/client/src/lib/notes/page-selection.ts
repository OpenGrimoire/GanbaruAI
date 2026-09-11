const SELECTED_PAGE_CONFIG_KEY = "notes.selectedPageId";

export function notesSelectedPageConfigKey(): string {
  return SELECTED_PAGE_CONFIG_KEY;
}

export function parseStoredNotesPageId(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

/** Return the restored page id only when the page still exists. */
export function restoredNotesPageSelection(
  pageId: string | null,
  pages: readonly { id: string }[],
): string | null {
  const normalizedPageId = pageId?.trim();
  if (!normalizedPageId) return null;
  return pages.some((page) => page.id === normalizedPageId) ? normalizedPageId : null;
}

/** Pick the page that should become selected after a page leaves the visible list. */
export function nextSelectedNotesPageId(
  pages: readonly { id: string }[],
  removedPageId: string,
): string | null {
  const index = pages.findIndex((page) => page.id === removedPageId);
  const remaining = pages.filter((page) => page.id !== removedPageId);
  if (remaining.length === 0) return null;
  if (index < 0) return remaining[0]?.id ?? null;
  return remaining[Math.min(index, remaining.length - 1)]?.id ?? null;
}

export function filterNotesPagesByTitle<Page>(
  pages: readonly Page[],
  query: string,
  titleForPage: (page: Page) => string,
): Page[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) return [...pages];
  return pages.filter((page) => titleForPage(page).toLowerCase().includes(normalizedQuery));
}
