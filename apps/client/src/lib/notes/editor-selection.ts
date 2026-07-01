export interface NotesTextSelection {
  start: number;
  end: number;
}

interface SelectionControl {
  selectionStart: number | null;
  selectionEnd: number | null;
}

/**
 * Read a normalized text selection from a textarea or input-like control.
 */
export function notesTextSelectionFromControl(control: SelectionControl): NotesTextSelection {
  const start = control.selectionStart ?? 0;
  const end = control.selectionEnd ?? start;
  return {
    start: Math.min(start, end),
    end: Math.max(start, end),
  };
}

/**
 * Keep a text selection inside the current plain text bounds.
 */
export function clampNotesTextSelection(
  selection: NotesTextSelection,
  textLength: number,
): NotesTextSelection {
  const max = Math.max(0, textLength);
  const start = Math.min(Math.max(0, selection.start), max);
  const end = Math.min(Math.max(0, selection.end), max);
  return {
    start: Math.min(start, end),
    end: Math.max(start, end),
  };
}
