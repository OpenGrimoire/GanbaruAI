import { describe, expect, it } from "vitest";
import type { QuickNote } from "./types";
import {
  applyQuickNoteGroupOrder,
  moveQuickNoteId,
  quickNoteMasonryInsertion,
  quickNoteMasonryLayout,
} from "./masonry";

describe("Quick note masonry layout", () => {
  it("uses the shortest available column and reports the container height", () => {
    const layout = quickNoteMasonryLayout(654, [100, 200, 50, 80], 210, 12);
    expect(layout.columns).toBe(3);
    expect(layout.positions[3]?.top).toBe(62);
    expect(layout.height).toBe(200);
  });

  it("falls back to one column when the container is narrow", () => {
    const layout = quickNoteMasonryLayout(200, [40, 50], 210, 12);
    expect(layout.columns).toBe(1);
    expect(layout.positions[1]?.top).toBe(52);
    expect(layout.height).toBe(102);
  });

  it("chooses the insertion whose dragged top-left matches an uneven masonry slot", () => {
    const heights = [180, 80, 140, 60];
    const target = quickNoteMasonryLayout(432, [180, 140, 80, 60]).positions[2];
    const insertion = quickNoteMasonryInsertion(
      432,
      heights,
      1,
      target?.left ?? 0,
      target?.top ?? 0,
    );
    expect(insertion.index).toBe(2);
    expect(insertion.distanceSquared).toBe(0);
  });

  it("keeps the preferred slot when masonry candidates are equally close", () => {
    const insertion = quickNoteMasonryInsertion(0, [40, 40], 1, 0, 26, 1);
    expect(insertion.index).toBe(1);
  });
});

describe("Quick note group ordering", () => {
  const notes = ["pinned-a", "pinned-b", "other-a", "other-b"]
    .map((id) => ({ id }) as QuickNote);

  it("reorders only the requested group slots", () => {
    expect(applyQuickNoteGroupOrder(notes, ["other-b", "other-a"]).map((note) => note.id))
      .toEqual(["pinned-a", "pinned-b", "other-b", "other-a"]);
  });

  it("rejects incomplete or duplicate group identities", () => {
    expect(applyQuickNoteGroupOrder(notes, ["missing"]).map((note) => note.id))
      .toEqual(notes.map((note) => note.id));
    expect(applyQuickNoteGroupOrder(notes, ["other-a", "other-a"]).map((note) => note.id))
      .toEqual(notes.map((note) => note.id));
  });

  it("moves keyboard reorders one slot and clamps at the edges", () => {
    expect(moveQuickNoteId(["a", "b", "c"], "b", -1)).toEqual(["b", "a", "c"]);
    expect(moveQuickNoteId(["a", "b", "c"], "c", 1)).toEqual(["a", "b", "c"]);
  });
});
