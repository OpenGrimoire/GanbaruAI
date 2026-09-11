const QUICK_NOTE_VIEW_SHORTCUT_COUNT = 10;

/** Returns the visible numeric shortcut for a Quick notes view position. */
export function quickNoteViewShortcut(index: number): string | null {
  if (!Number.isInteger(index) || index < 0 || index >= QUICK_NOTE_VIEW_SHORTCUT_COUNT) return null;
  return index === QUICK_NOTE_VIEW_SHORTCUT_COUNT - 1 ? "0" : String(index + 1);
}

/** Resolves a numeric key to its zero-based Quick notes view position. */
export function quickNoteViewIndexForKey(key: string): number | null {
  if (key === "0") return QUICK_NOTE_VIEW_SHORTCUT_COUNT - 1;
  if (!/^[1-9]$/u.test(key)) return null;
  return Number(key) - 1;
}
