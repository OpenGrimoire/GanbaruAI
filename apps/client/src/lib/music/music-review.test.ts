import { describe, expect, it } from "vitest";
import {
  musicReviewSource,
  nextMusicWeight,
  parseMusicReviewAutoplay,
  sortReviewPlaylists,
} from "./music-review";
import type { MusicInspectorDetail, MusicPlaylistSummary } from "./library-contracts";

const playlist = (id: string, name: string): MusicPlaylistSummary => ({
  id, sortOrder: 0, name, icon: "lucide:list-music", shuffleEnabled: true, repeatMode: "all", intendedUses: [], totalCount: 0,
  eligibleCount: 0, unavailableCount: 0, snoozedCount: 0, localCount: 0, onlineCount: 0, version: 1,
});

function detail(sourceKind: "local-file" | "youtube-video"): MusicInspectorDetail {
  return {
    item: {
      id: "item", identityKey: "item", sourceKind, mediaKind: "audio",
      youtubeVideoId: sourceKind === "youtube-video" ? "abc12345" : null,
      originalTitle: "Title", originalArtist: "Artist", originalAlbum: "Album",
      originalTrackNumber: null, originalArtworkIdentity: null, youtubeResolutionState: null,
      titleOverride: null, artistOverride: null, albumOverride: null, artworkOverride: null,
      durationMs: 1000, availability: "available", reviewState: "unreviewed",
      reviewChangedAt: null, reviewDeferredUntil: null, discoveredAt: 1, updatedAt: 1, version: 1,
    },
    locations: sourceKind === "local-file" ? [{
      id: "location", itemId: "item", rootId: "root", relativePath: "album/song.flac",
      fileSizeBytes: 1, modifiedAtMs: 1, lightweightFingerprint: "a", strongFingerprint: "b",
      availability: "available", lastSeenGeneration: 1, firstSeenAt: 1, updatedAt: 1,
    }] : [],
    memberships: [], membershipSkipRanges: [], snoozes: [], signals: [], statistics: null, sourceCollectionIds: [],
  };
}

describe("music review helpers", () => {
  it("defaults unsafe stored autoplay values", () => {
    expect(parseMusicReviewAutoplay("true")).toBe(false);
  });

  it("resolves local and YouTube canonical items into playable sources", () => {
    expect(musicReviewSource(detail("local-file"), [{ rootId: "root", folderPath: "/Music", status: "available" }]))
      .toMatchObject({ kind: "local-file", path: "/Music/album/song.flac" });
    expect(musicReviewSource(detail("youtube-video"), []))
      .toMatchObject({ kind: "youtube-video", videoId: "abc12345" });
  });

  it("resolves scanner-detected sidecar artwork for review playback", () => {
    const localDetail = detail("local-file");
    localDetail.item.originalArtworkIdentity = "sidecar:album/cover.jpg";

    expect(musicReviewSource(localDetail, [{ rootId: "root", folderPath: "/Music", status: "available" }]))
      .toMatchObject({
        kind: "local-file",
        path: "/Music/album/song.flac",
        artworkPath: "/Music/album/cover.jpg",
      });
  });

  it("keeps playlist ordering independent from selection while filtering", () => {
    expect(sortReviewPlaylists([playlist("a", "Work"), playlist("b", "Reading")], "")
      .map((entry) => entry.id)).toEqual(["b", "a"]);
    expect(sortReviewPlaylists([playlist("a", "Work"), playlist("b", "Reading")], "read")
      .map((entry) => entry.id)).toEqual(["b"]);
  });

  it("cycles through all probability weights", () => {
    expect(nextMusicWeight("much-more-often")).toBe("rarely");
  });
});
