import { describe, expect, it } from "vitest";
import type { MusicPlaylistPlaybackEntry } from "$lib/music/library-contracts";
import { projectMusicPlaylistPlayback } from "$lib/music/music-playlist-playback";

const entry = (patch: Partial<MusicPlaylistPlaybackEntry> = {}): MusicPlaylistPlaybackEntry => ({
  membershipId: "membership", itemId: "item", identityKey: "local:item", sourceKind: "local-file",
  youtubeVideoId: null, title: "Track", availability: "available", rootId: "root", relativePath: "album/track.flac",
  position: 0, weight: "normal", enabled: true, startMs: null, endMs: null, volume: null, rate: null,
  snoozed: false, ...patch,
});

describe("projectMusicPlaylistPlayback", () => {
  it("builds a mixed queue and reports every skipped eligibility reason", () => {
    const projection = projectMusicPlaylistPlayback([
      entry(),
      entry({ itemId: "youtube", sourceKind: "youtube-video", youtubeVideoId: "video123", rootId: null, relativePath: null }),
      entry({ itemId: "disabled", enabled: false }),
      entry({ itemId: "snoozed", snoozed: true }),
      entry({ itemId: "missing", availability: "missing" }),
    ], [{ rootId: "root", folderPath: "/music", status: "available" }]);
    expect(projection.sources).toHaveLength(2);
    expect(projection.itemIds).toEqual(["item", "youtube"]);
    expect(projection.skipped).toMatchObject({ disabled: 1, snoozed: 1, unavailable: 1 });
  });
});
