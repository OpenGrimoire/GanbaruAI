import { describe, expect, it } from "vitest";
import { projectVisibleRange } from "./visible-range";

describe("projectVisibleRange", () => {
  it.each([0, 1, 100, 10_000])("bounds a %i item collection", (count) => {
    const range = projectVisibleRange(count, 40_000, 400, 40, 5);
    expect(range.start).toBeGreaterThanOrEqual(0);
    expect(range.end).toBeLessThanOrEqual(count);
    expect(range.end - range.start).toBeLessThanOrEqual(20);
    expect(range.beforePx + range.afterPx + (range.end - range.start) * 40).toBe(count * 40);
  });

  it("includes overscan around the visible rows", () => {
    expect(projectVisibleRange(100, 400, 400, 40, 5)).toEqual({
      start: 5, end: 25, beforePx: 200, afterPx: 3_000,
    });
  });
});
