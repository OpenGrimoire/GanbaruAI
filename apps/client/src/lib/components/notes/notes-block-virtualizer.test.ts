import { describe, expect, it } from "vitest";
import { notesFocusedBlockEstimatedOffset } from "./notes-block-virtualizer.svelte";

describe("Notes block virtual focus recovery", () => {
  it("uses measured predecessors before retained-height estimates", () => {
    const items = [
      { id: "first", estimatedHeight: 36 },
      { id: "second", estimatedHeight: 48 },
      { id: "target", estimatedHeight: 36 },
    ];
    expect(notesFocusedBlockEstimatedOffset(items, new Map([["first", 52]]), "target"))
      .toBe(100);
    expect(notesFocusedBlockEstimatedOffset(items, new Map(), "missing")).toBeNull();
  });
});
