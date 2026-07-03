import { describe, expect, it } from "vitest";
import {
  EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE,
  notesBlockHandleHoverStateAfterKeydown,
  notesBlockHandleHoverStateAfterPointerLeave,
  notesBlockHandleHoverStateAfterPointerMove,
} from "./block-handle-hover";

describe("notes block handle hover state", () => {
  it("shows the handle for the block that received pointer movement", () => {
    expect(
      notesBlockHandleHoverStateAfterPointerMove(
        EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE,
        "block-a",
      ),
    ).toEqual({ visibleBlockId: "block-a" });
  });

  it("moves the visible handle to the latest pointer-moved block", () => {
    const state = notesBlockHandleHoverStateAfterPointerMove(
      { visibleBlockId: "block-a" },
      "block-b",
    );

    expect(state).toEqual({ visibleBlockId: "block-b" });
  });

  it("hides the handle when the visible block receives pointer leave", () => {
    const state = notesBlockHandleHoverStateAfterPointerLeave(
      { visibleBlockId: "block-a" },
      "block-a",
    );

    expect(state).toEqual(EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE);
  });

  it("keeps the handle when another block receives pointer leave", () => {
    const state = notesBlockHandleHoverStateAfterPointerLeave(
      { visibleBlockId: "block-a" },
      "block-b",
    );

    expect(state).toEqual({ visibleBlockId: "block-a" });
  });

  it("hides the handle after text-producing keyboard input", () => {
    const state = notesBlockHandleHoverStateAfterKeydown({ visibleBlockId: "block-a" }, "E");

    expect(state).toEqual(EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE);
  });

  it("does not hide the handle for modifier-only keyboard input", () => {
    const state = notesBlockHandleHoverStateAfterKeydown({ visibleBlockId: "block-a" }, "Shift");

    expect(state).toEqual({ visibleBlockId: "block-a" });
  });
});
