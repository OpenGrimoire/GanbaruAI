import { describe, expect, it } from "vitest";
import type { MusicItemListEntry } from "$lib/music/library-contracts";
import {
  buildMusicReviewTree,
  flattenMusicReviewTree,
  musicReviewTreeFolderIds,
  toggleMusicReviewTreeSelection,
} from "$lib/music/music-review-tree";

function item(id: string, relativePath: string | null, sourceKind: "local-file" | "youtube-video" = "local-file"): MusicItemListEntry {
  return {
    id, identityKey: id, sourceKind, mediaKind: "audio", title: id, artist: "", album: "",
    relativePath, artworkOverride: null, durationMs: null, availability: "available", reviewState: "unreviewed",
    discoveredAt: 1, updatedAt: 1, version: 1, playlistCount: 0, activeSnoozeCount: 0,
    lastPlayedAt: null, playCount: 0, membershipId: null, membershipPosition: null,
    membershipWeight: null, membershipEnabled: null, membershipVersion: null,
  };
}

describe("music review tree", () => {
  it("preserves nested folders and keeps online items visible", () => {
    const tree = buildMusicReviewTree([
      item("theme", "Games/Nier/Disc 1/theme.flac"),
      item("ending", "Games/Nier/ending.flac"),
      item("root", "loose.mp3"),
      item("video", null, "youtube-video"),
    ]);

    expect(tree.map((node) => node.name)).toEqual(["Music", "Online"]);
    expect(tree[0]?.itemIds).toEqual(["root", "ending", "theme"]);
    expect(tree[0]?.children[0]?.children[0]?.children[0]?.path).toBe("Games/Nier/Disc 1");
    expect(tree[1]?.itemIds).toEqual(["video"]);
  });

  it("flattens only expanded branches and toggles complete descendants", () => {
    const tree = buildMusicReviewTree([item("one", "Album/one.flac"), item("two", "Album/two.flac")]);
    const root = tree[0]!;
    const album = root.children[0]!;
    const rows = flattenMusicReviewTree(tree, new Set([root.id, album.id]));
    expect(rows.map((row) => row.kind === "folder" ? row.node.name : row.item.id)).toEqual(["Music", "Album", "one", "two"]);
    expect([...toggleMusicReviewTreeSelection(new Set(["other"]), album.itemIds)]).toEqual(["other", "one", "two"]);
    expect([...toggleMusicReviewTreeSelection(new Set(["one", "two", "other"]), album.itemIds)]).toEqual(["other"]);
  });

  it("collects every nested folder for the initial expanded state", () => {
    const tree = buildMusicReviewTree([
      item("theme", "Games/Nier/Disc 1/theme.flac"),
      item("video", null, "youtube-video"),
    ]);

    expect([...musicReviewTreeFolderIds(tree)]).toEqual([
      "review-root:local",
      "review-folder:Games",
      "review-folder:Games/Nier",
      "review-folder:Games/Nier/Disc 1",
      "review-root:online",
    ]);
    expect(
      flattenMusicReviewTree(tree, musicReviewTreeFolderIds(tree))
        .map((row) => row.kind === "folder" ? row.node.name : row.item.id),
    ).toEqual(["Music", "Games", "Nier", "Disc 1", "theme", "Online", "video"]);
  });
});
