export type NotesBlockHandleHoverState = {
  visibleBlockId: string | null;
};

const KEYBOARD_ONLY_MODIFIER_KEYS = new Set(["Alt", "AltGraph", "Control", "Meta", "Shift"]);

export const EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE: NotesBlockHandleHoverState = {
  visibleBlockId: null,
};

export function notesBlockHandleHoverStateAfterPointerMove(
  state: NotesBlockHandleHoverState,
  blockId: string,
): NotesBlockHandleHoverState {
  if (state.visibleBlockId === blockId) return state;
  return { visibleBlockId: blockId };
}

export function notesBlockHandleHoverStateAfterPointerLeave(
  state: NotesBlockHandleHoverState,
  blockId: string,
): NotesBlockHandleHoverState {
  if (state.visibleBlockId !== blockId) return state;
  return EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE;
}

export function notesBlockHandleHoverStateAfterKeydown(
  state: NotesBlockHandleHoverState,
  key: string,
): NotesBlockHandleHoverState {
  if (KEYBOARD_ONLY_MODIFIER_KEYS.has(key) || state.visibleBlockId === null) return state;
  return EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE;
}
