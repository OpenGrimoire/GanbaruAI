import {
  EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE,
  notesBlockHandleHoverStateAfterKeydown,
  notesBlockHandleHoverStateAfterPointerLeave,
  notesBlockHandleHoverStateAfterPointerMove,
} from "$lib/notes/block-handle-hover";

/** Coordinate the one visible Notes block handle and open handle menu. */
export function createNotesBlockHandleController() {
  let hoverState = $state(EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE);
  let openMenuBlockId = $state<string | null>(null);

  return {
    get visibleBlockId() {
      return hoverState.visibleBlockId;
    },
    get openMenuBlockId() {
      return openMenuBlockId;
    },
    pointerMove(blockId: string): void {
      if (openMenuBlockId) return;
      hoverState = notesBlockHandleHoverStateAfterPointerMove(hoverState, blockId);
    },
    pointerLeave(blockId: string): void {
      hoverState = notesBlockHandleHoverStateAfterPointerLeave(hoverState, blockId);
    },
    keydown(event: KeyboardEvent): void {
      hoverState = notesBlockHandleHoverStateAfterKeydown(hoverState, event.key);
    },
    setMenuOpen(blockId: string, open: boolean): void {
      if (open) {
        openMenuBlockId = blockId;
        hoverState = EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE;
        return;
      }
      if (openMenuBlockId === blockId) openMenuBlockId = null;
    },
  };
}
