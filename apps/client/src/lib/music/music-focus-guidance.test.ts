import { describe, expect, it } from "vitest";
import { getMusicFocusAdvisory } from "./music-focus-guidance";
import type { MusicPlaylistSummary } from "./library-contracts";

function playlist(id: string, intendedUses: MusicPlaylistSummary["intendedUses"]): MusicPlaylistSummary {
  return {
    id, name: id, description: "", shuffleEnabled: true, repeatMode: "all", intendedUses,
    totalCount: 0, eligibleCount: 0, unavailableCount: 0, snoozedCount: 0,
    localCount: 0, onlineCount: 0, version: 1,
  };
}

describe("getMusicFocusAdvisory", () => {
  it("only advises when a disruptive signal intersects a selected focus context", () => {
    const playlists = [playlist("focus", ["focus"]), playlist("exercise", ["energizing"])];
    expect(getMusicFocusAdvisory(["lyrics", "calm"], new Set(["focus"]), playlists)).toEqual({
      playlistIds: ["focus"], signals: ["lyrics"],
    });
    expect(getMusicFocusAdvisory(["lyrics"], new Set(["exercise"]), playlists)).toBeNull();
    expect(getMusicFocusAdvisory(["calm"], new Set(["focus"]), playlists)).toBeNull();
  });

  it("treats reading as focus-oriented and preserves deterministic playlist order", () => {
    const playlists = [playlist("reading", ["reading"]), playlist("focus", ["focus", "general"])];
    expect(getMusicFocusAdvisory(["sudden-changes", "high-intensity"], new Set(["focus", "reading"]), playlists)?.playlistIds)
      .toEqual(["reading", "focus"]);
  });
});
