import { describe, expect, it } from "vitest";
import { formatMusicTimecode, parseMusicTimecode, validateMusicSkipRanges } from "./music-membership-settings";

describe("music membership settings", () => {
  it("parses common timecodes without accepting malformed values", () => {
    expect(parseMusicTimecode("1:02")).toBe(62_000);
    expect(parseMusicTimecode("1:02:03.5")).toBe(3_723_500);
    expect(parseMusicTimecode("1:bad")).toBeNull();
    expect(formatMusicTimecode(3_723_000)).toBe("1:02:03");
  });

  it("rejects inverted and overlapping skip ranges", () => {
    expect(validateMusicSkipRanges([{ id: "a", membershipId: "m", startMs: 0, endMs: 10, sortOrder: 0 }])).toBeNull();
    expect(validateMusicSkipRanges([{ id: "a", membershipId: "m", startMs: 10, endMs: 10, sortOrder: 0 }])).toBe("invalid-range");
    expect(validateMusicSkipRanges([
      { id: "a", membershipId: "m", startMs: 0, endMs: 10, sortOrder: 0 },
      { id: "b", membershipId: "m", startMs: 5, endMs: 20, sortOrder: 1 },
    ])).toBe("overlapping-range");
  });
});
