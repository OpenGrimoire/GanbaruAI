import { describe, expect, it } from "vitest";
import { mapMusicWithConcurrency } from "./music-bounded-work";

describe("bounded Music work", () => {
  it("preserves order and never exceeds the accepted concurrency", async () => {
    let active = 0;
    let maximumActive = 0;
    const values = Array.from({ length: 37 }, (_, index) => index);
    const results = await mapMusicWithConcurrency(values, 4, async (value) => {
      active += 1;
      maximumActive = Math.max(maximumActive, active);
      await Promise.resolve();
      active -= 1;
      return value * 2;
    });
    expect(results).toEqual(values.map((value) => value * 2));
    expect(maximumActive).toBe(4);
  });

  it("handles empty work and clamps invalid limits", async () => {
    expect(await mapMusicWithConcurrency([], 8, async (value: number) => value)).toEqual([]);
    expect(await mapMusicWithConcurrency([1, 2], 0, async (value) => value)).toEqual([1, 2]);
  });
});
