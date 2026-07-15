import { describe, expect, it } from "vitest";
import {
  parseBindingResult,
  parseInspectorDetail,
  parseItemWindow,
  parsePlaylistSummaries,
  parseWriteReceipt,
} from "./library-contracts";
import { normalizeMusicLibraryError } from "$lib/api/music-library";

const item = {
  id: "item-1",
  identityKey: "local:item-1",
  sourceKind: "local-file",
  mediaKind: "audio",
  title: "Focus",
  artist: "Composer",
  album: "Soundtrack",
  durationMs: 120_000,
  availability: "available",
  reviewState: "unreviewed",
  discoveredAt: 1,
  updatedAt: 1,
  version: 1,
  playlistCount: 0,
  activeSnoozeCount: 0,
  lastPlayedAt: null,
  playCount: 0,
  membershipId: null,
  membershipPosition: null,
  membershipWeight: null,
  membershipEnabled: null,
  membershipVersion: null,
};

describe("music library contracts", () => {
  it("accepts a complete bounded item window", () => {
    expect(parseItemWindow({
      items: [item],
      groups: [{ key: "local-file", count: 1 }],
      totalCount: 1,
      offset: 0,
      limit: 50,
    }).items[0]?.title).toBe("Focus");
  });

  it("rejects malformed enums and numeric fields", () => {
    expect(() => parseItemWindow({
      items: [{ ...item, sourceKind: "streaming-service" }],
      groups: [], totalCount: 1, offset: 0, limit: 50,
    })).toThrow("sourceKind is not supported");
    expect(() => parseWriteReceipt({ id: "playlist-1", version: "1" })).toThrow("version must be a safe integer");
  });

  it("rejects partially shaped summaries instead of applying defaults", () => {
    expect(() => parsePlaylistSummaries([{
      id: "playlist-1",
      name: "Focus",
      description: "",
      shuffleEnabled: false,
      repeatMode: "all",
      totalCount: 1,
    }])).toThrow("eligibleCount must be a safe integer");
  });

  it("validates nested inspector rows", () => {
    expect(() => parseInspectorDetail({
      item: {
        id: "item-1", identityKey: "local:item-1", sourceKind: "local-file", mediaKind: "audio",
        youtubeVideoId: null, originalTitle: "Focus", originalArtist: "", originalAlbum: "",
        titleOverride: null, artistOverride: null, albumOverride: null, artworkOverride: null,
        durationMs: null, availability: "available", reviewState: "reviewed", reviewChangedAt: null,
        discoveredAt: 1, updatedAt: 1, version: 1,
      },
      locations: [], memberships: [], snoozes: [], signals: ["invented"],
      statistics: null, sourceCollectionIds: [],
    })).toThrow("signals[0] is not supported");
  });

  it("validates device binding state", () => {
    expect(parseBindingResult({ rootId: "root-1", folderPath: null, status: "needs-relink" }))
      .toEqual({ rootId: "root-1", folderPath: null, status: "needs-relink" });
    expect(() => parseBindingResult({ rootId: "root-1", folderPath: null, status: "lost" }))
      .toThrow("status is not supported");
  });

  it("normalizes structured backend errors without trusting arbitrary fields", () => {
    const structured = normalizeMusicLibraryError({
      code: "stale-write",
      message: "Playlist changed",
      field: "expectedVersion",
    });
    expect(structured).toMatchObject({
      code: "stale-write",
      message: "Playlist changed",
      field: "expectedVersion",
    });
    expect(normalizeMusicLibraryError({ code: "root", message: 42 }).code).toBe("unknown");
  });
});
