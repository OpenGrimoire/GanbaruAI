import { notesBlockInsertMenuStyle } from "./block-insertion";
import type { NotesBlockInsertMenuPlacementInput } from "./block-insertion";

export type NotesBlockHandleMenuToggle = "actions" | "insert" | "move";
export type NotesBlockHandleAction =
  | "turn_into"
  | "color"
  | "copy_link"
  | "duplicate"
  | "comment"
  | "move_up"
  | "move_down"
  | "move_to_page"
  | "delete";

export interface NotesBlockHandleMenuState {
  menuOpen: boolean;
  insertMenuOpen: boolean;
  moveMenuOpen: boolean;
}

export const CLOSED_NOTES_BLOCK_HANDLE_MENUS: NotesBlockHandleMenuState = {
  menuOpen: false,
  insertMenuOpen: false,
  moveMenuOpen: false,
};

/**
 * Plans visible block-handle menus from a trigger toggle.
 */
export function notesBlockHandleMenuStateAfterToggle(
  current: NotesBlockHandleMenuState,
  toggle: NotesBlockHandleMenuToggle,
): NotesBlockHandleMenuState {
  switch (toggle) {
    case "actions": {
      const menuOpen = !current.menuOpen;
      return {
        menuOpen,
        insertMenuOpen: false,
        moveMenuOpen: false,
      };
    }
    case "insert":
      return {
        menuOpen: false,
        insertMenuOpen: !current.insertMenuOpen,
        moveMenuOpen: false,
      };
    case "move":
      return {
        menuOpen: true,
        insertMenuOpen: false,
        moveMenuOpen: !current.moveMenuOpen,
      };
  }
}

/**
 * Plans visible block-handle menus after a menu action runs.
 */
export function notesBlockHandleMenuStateAfterAction(
  current: NotesBlockHandleMenuState,
  action: NotesBlockHandleAction,
): NotesBlockHandleMenuState {
  switch (action) {
    case "color":
    case "copy_link":
      return {
        menuOpen: current.menuOpen,
        insertMenuOpen: false,
        moveMenuOpen: false,
      };
    case "move_to_page":
    case "turn_into":
    case "duplicate":
    case "comment":
    case "move_up":
    case "move_down":
    case "delete":
      return CLOSED_NOTES_BLOCK_HANDLE_MENUS;
  }
}

/**
 * Places the block action menu with the same viewport clamp as the plus menu.
 */
export function notesBlockHandleActionMenuStyle(
  input: Omit<NotesBlockInsertMenuPlacementInput, "preferredWidth">,
): string {
  return notesBlockInsertMenuStyle({
    ...input,
    preferredWidth: 224,
  });
}
