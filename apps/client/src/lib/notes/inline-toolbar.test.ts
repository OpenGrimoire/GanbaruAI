import { describe, expect, it } from "vitest";
import {
  planNotesInlineToolbarPlacement,
  shouldPreventInlineToolbarPointerDefault,
} from "./inline-toolbar";

const selectionRect = {
  top: 120,
  right: 260,
  bottom: 140,
  left: 180,
  width: 80,
  height: 20,
};

describe("notes inline toolbar placement", () => {
  it("floats above the selected text when there is room", () => {
    expect(
      planNotesInlineToolbarPlacement({
        selectionRect,
        toolbarWidth: 240,
        toolbarHeight: 40,
        viewport: { width: 800, height: 600 },
      }),
    ).toEqual({
      mode: "floating",
      top: 74,
      left: 100,
      maxWidth: 784,
    });
  });

  it("floats below the selected text when the selection is near the top", () => {
    expect(
      planNotesInlineToolbarPlacement({
        selectionRect: { ...selectionRect, top: 20, bottom: 40 },
        toolbarWidth: 240,
        toolbarHeight: 40,
        viewport: { width: 800, height: 600 },
      }),
    ).toMatchObject({
      mode: "floating",
      top: 46,
      left: 100,
    });
  });

  it("clamps the toolbar inside the viewport horizontally", () => {
    expect(
      planNotesInlineToolbarPlacement({
        selectionRect: { ...selectionRect, left: 760, right: 790 },
        toolbarWidth: 240,
        toolbarHeight: 40,
        viewport: { width: 800, height: 600 },
      }),
    ).toMatchObject({
      mode: "floating",
      left: 552,
    });
  });

  it("docks instead of overlapping content on narrow viewports", () => {
    expect(
      planNotesInlineToolbarPlacement({
        selectionRect,
        toolbarWidth: 240,
        toolbarHeight: 70,
        viewport: { width: 320, height: 480 },
      }),
    ).toEqual({
      mode: "docked",
      maxWidth: 304,
    });
  });

  it("preserves selection for touch and pen toolbar activation", () => {
    expect(shouldPreventInlineToolbarPointerDefault("touch")).toBe(true);
    expect(shouldPreventInlineToolbarPointerDefault("pen")).toBe(true);
    expect(shouldPreventInlineToolbarPointerDefault("mouse")).toBe(false);
  });
});
