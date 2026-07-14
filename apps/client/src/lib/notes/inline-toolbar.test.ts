import { describe, expect, it } from "vitest";
import {
  notesInlineToolbarWrapperClass,
  notesInlineToolbarWrapperStyle,
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

  it("hides the toolbar while its first placement is being measured", () => {
    expect(notesInlineToolbarWrapperClass(null)).toBe(
      "pointer-events-none fixed z-40 flex justify-center",
    );
    expect(notesInlineToolbarWrapperStyle(null)).toBe(
      "visibility: hidden; left: 0px; top: 0px;",
    );
  });

  it("uses fixed coordinates for floating toolbar placement", () => {
    const placement = planNotesInlineToolbarPlacement({
      selectionRect,
      toolbarWidth: 240,
      toolbarHeight: 40,
      viewport: { width: 800, height: 600 },
    });

    expect(notesInlineToolbarWrapperClass(placement)).toBe("fixed z-40 flex justify-center");
    expect(notesInlineToolbarWrapperStyle(placement)).toBe(
      "max-width: 784px; left: 100px; top: 74px;",
    );
  });
});
