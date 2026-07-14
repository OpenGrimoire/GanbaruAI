import { listQuickNotes, listQuickNoteTags } from "$lib/api/quick-notes";
import type {
  QuickNotesCollection,
  QuickNotesWindow,
  QuickNoteTag,
} from "$lib/quick-notes/types";

export interface QuickNotesInitialSnapshot {
  tags: QuickNoteTag[];
  window: QuickNotesWindow;
}

let cachedTags: QuickNoteTag[] | null = null;
let cachedWindow: QuickNotesWindow | null = null;
const cachedViews = new Map<string, QuickNotesWindow>();
let preloadPromise: Promise<QuickNotesInitialSnapshot> | null = null;
let cacheEpoch = 0;

function viewKey(collection: QuickNotesCollection, tagId: string | null): string {
  return collection === "active" ? `active:${tagId ?? "all"}` : collection;
}

function copyWindow(window: QuickNotesWindow): QuickNotesWindow {
  return { notes: [...window.notes], nextCursor: window.nextCursor };
}

function snapshot(): QuickNotesInitialSnapshot | null {
  if (!cachedTags || !cachedWindow) return null;
  return {
    tags: [...cachedTags],
    window: copyWindow(cachedWindow),
  };
}

/** Returns the current resident Quick notes startup snapshot, if ready. */
export function readQuickNotesInitialSnapshot(): QuickNotesInitialSnapshot | null {
  return snapshot();
}

/** Starts one shared initial All-view read for the title-bar Quick notes panel. */
export function preloadQuickNotesInitialSnapshot(): Promise<QuickNotesInitialSnapshot> {
  const current = snapshot();
  if (current) return Promise.resolve(current);
  if (preloadPromise) return preloadPromise;
  const requestEpoch = cacheEpoch;
  const request = Promise.all([
    listQuickNoteTags(),
    listQuickNotes("active", "", null, null),
  ]).then(([tags, window]): Promise<QuickNotesInitialSnapshot> | QuickNotesInitialSnapshot => {
    if (requestEpoch !== cacheEpoch) {
      preloadPromise = null;
      return preloadQuickNotesInitialSnapshot();
    }
    cachedTags = [...tags];
    cacheQuickNotesViewWindow("active", null, window);
    return snapshot()!;
  });
  preloadPromise = request;
  void request.then(
    () => { if (preloadPromise === request) preloadPromise = null; },
    () => { if (preloadPromise === request) preloadPromise = null; },
  );
  return request;
}

/** Refreshes the cached tag portion after a canonical tag read. */
export function cacheQuickNoteTags(tags: readonly QuickNoteTag[]): void {
  cachedTags = [...tags];
}

/** Refreshes the cached All-view window after a canonical collection read. */
export function cacheQuickNotesAllWindow(window: QuickNotesWindow): void {
  cacheQuickNotesViewWindow("active", null, window);
}

/** Returns a resident non-search collection window when it has been visited. */
export function readQuickNotesViewWindow(
  collection: QuickNotesCollection,
  tagId: string | null,
): QuickNotesWindow | null {
  const window = cachedViews.get(viewKey(collection, tagId));
  return window ? copyWindow(window) : null;
}

/** Caches one bounded non-search collection window for instant view revisits. */
export function cacheQuickNotesViewWindow(
  collection: QuickNotesCollection,
  tagId: string | null,
  window: QuickNotesWindow,
): void {
  const copy = copyWindow(window);
  cachedViews.set(viewKey(collection, tagId), copy);
  if (collection === "active" && tagId === null) cachedWindow = copy;
}

/** Drops resident data after a mutation so the next read cannot reuse stale notes. */
export function invalidateQuickNotesInitialSnapshot(): void {
  cacheEpoch += 1;
  cachedTags = null;
  cachedWindow = null;
  cachedViews.clear();
}
