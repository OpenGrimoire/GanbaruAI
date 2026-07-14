import { describe, expect, it } from "vitest";
import { quickNoteMasonryLayout } from "./masonry";

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
});
