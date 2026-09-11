import { describe, expect, it } from "vitest";
import {
  CLOSED_NOTES_BLOCK_HANDLE_MENUS,
  notesBlockHandleActionMenuStyle,
  notesBlockHandleMenuStateAfterAction,
  notesBlockHandleMenuStateAfterToggle,
} from "./block-handle";
import type { NotesBlockHandleAction } from "./block-handle";

describe("notes block handle menu routing", () => {
  it("opens one top-level handle menu at a time", () => {
    const insertOpen = notesBlockHandleMenuStateAfterToggle(
      {
        menuOpen: true,
        insertMenuOpen: false,
        moveMenuOpen: true,
      },
      "insert",
    );

    expect(insertOpen).toEqual({
      menuOpen: false,
      insertMenuOpen: true,
      moveMenuOpen: false,
    });

    expect(notesBlockHandleMenuStateAfterToggle(insertOpen, "actions")).toEqual({
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: false,
    });
  });

  it("keeps copy and color actions visible while closing nested move state", () => {
    const state = {
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: true,
    };

    expect(notesBlockHandleMenuStateAfterAction(state, "copy_link")).toEqual({
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: false,
    });
    expect(notesBlockHandleMenuStateAfterAction(state, "color")).toEqual({
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: false,
    });
  });

  it("closes all handle menus for actions that hand focus back to the editor", () => {
    const state = {
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: true,
    };
    const closingActions: NotesBlockHandleAction[] = [
      "turn_into",
      "duplicate",
      "comment",
      "move_up",
      "move_down",
      "move_to_page",
      "delete",
    ];

    for (const action of closingActions) {
      expect(notesBlockHandleMenuStateAfterAction(state, action)).toEqual(
        CLOSED_NOTES_BLOCK_HANDLE_MENUS,
      );
    }
  });

  it("toggles the move target panel inside the action menu", () => {
    const actionMenuOpen = {
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: false,
    };

    expect(notesBlockHandleMenuStateAfterToggle(actionMenuOpen, "move")).toEqual({
      menuOpen: true,
      insertMenuOpen: false,
      moveMenuOpen: true,
    });
    expect(
      notesBlockHandleMenuStateAfterToggle(
        {
          ...actionMenuOpen,
          moveMenuOpen: true,
        },
        "move",
      ),
    ).toEqual(actionMenuOpen);
  });

  it("clamps the action menu inside narrow viewports", () => {
    const style = notesBlockHandleActionMenuStyle({
      triggerRect: {
        top: 120,
        right: 284,
        bottom: 144,
        left: 260,
      },
      viewportWidth: 280,
      viewportHeight: 180,
    });

    expect(style).toContain("left:48px");
    expect(style).toContain("width:224px");
    expect(style).toContain("max-height:108px");
  });
});
