import { describe, expect, it } from "vitest";
import {
  notesRowContextMenuGeometry,
  notesRowContextMenuStyle,
} from "./row-context-menu";

describe("Notes row context menu geometry", () => {
  it("uses the pointer position when the menu fits", () => {
    expect(notesRowContextMenuGeometry({
      clientX: 120,
      clientY: 80,
      viewportWidth: 800,
      viewportHeight: 600,
    })).toEqual({ left: 120, top: 80, width: 256, maxHeight: 352 });
  });

  it("clamps the menu to the right and bottom edges", () => {
    expect(notesRowContextMenuGeometry({
      clientX: 790,
      clientY: 590,
      viewportWidth: 800,
      viewportHeight: 600,
    })).toEqual({ left: 536, top: 240, width: 256, maxHeight: 352 });
  });

  it("shrinks within a compact viewport", () => {
    const geometry = notesRowContextMenuGeometry({
      clientX: 200,
      clientY: 120,
      viewportWidth: 220,
      viewportHeight: 180,
    });

    expect(geometry).toEqual({ left: 8, top: 8, width: 204, maxHeight: 164 });
    expect(notesRowContextMenuStyle(geometry))
      .toBe("left: 8px; top: 8px; width: 204px; max-height: 164px");
  });
});
