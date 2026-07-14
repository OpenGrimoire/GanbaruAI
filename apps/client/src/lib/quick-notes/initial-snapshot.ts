import { listQuickNotes, listQuickNoteTags } from "$lib/api/quick-notes";
import type { QuickNotesWindow, QuickNoteTag } from "$lib/quick-notes/types";

export interface QuickNotesInitialSnapshot {
  tags: QuickNoteTag[];
  window: QuickNotesWindow;
}

let cachedTags: QuickNoteTag[] | null = null;
let cachedWindow: QuickNotesWindow | null = null;
let preloadPromise: Promise<QuickNotesInitialSnapshot> | null = null;
let cacheEpoch = 0;

function snapshot(): QuickNotesInitialSnapshot | null {
  if (!cachedTags || !cachedWindow) return null;
  return {
    tags: [...cachedTags],
    window: { notes: [...cachedWindow.notes], nextCursor: cachedWindow.nextCursor },
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
    cachedWindow = { notes: [...window.notes], nextCursor: window.nextCursor };
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
  cachedWindow = { notes: [...window.notes], nextCursor: window.nextCursor };
}

/** Drops resident data after a mutation so the next read cannot reuse stale notes. */
export function invalidateQuickNotesInitialSnapshot(): void {
  cacheEpoch += 1;
  cachedTags = null;
  cachedWindow = null;
}
