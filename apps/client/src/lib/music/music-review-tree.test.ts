import { describe, expect, it } from "vitest";
import type { MusicItemListEntry } from "$lib/music/library-contracts";
import {
  buildMusicReviewTree,
  firstMusicReviewTreeItemId,
  flattenMusicReviewTree,
  musicReviewTreeAncestorFolderIds,
  musicReviewTreeItemIds,
  musicReviewTreeFolderIds,
  nextPendingMusicReviewTreeItemId,
  searchMusicReviewTree,
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

  it("chooses the first pending item in visible tree order instead of input order", () => {
    const deepItem = item("dogfight", "Soundtracks/Anime/Album/dogfight.flac");
    const rootItem = item("brown-noise", "brown_noise.flac");
    const reviewedRootItem = item("alpha-reviewed", "alpha_reviewed.flac");
    reviewedRootItem.reviewState = "reviewed";
    const items = [deepItem, rootItem, reviewedRootItem];

    expect(musicReviewTreeItemIds(items)).toEqual([
      "alpha-reviewed",
      "brown-noise",
      "dogfight",
    ]);
    expect(firstMusicReviewTreeItemId(items)).toBe("brown-noise");
  });

  it("finds the next pending item while wrapping around reviewed and session-skipped tracks", () => {
    const first = item("first", "first.flac");
    const second = item("second", "second.flac");
    const third = item("third", "third.flac");
    second.reviewState = "reviewed";

    expect(nextPendingMusicReviewTreeItemId([first, second, third], "first", new Set(["third"]))).toBe("first");
    expect(nextPendingMusicReviewTreeItemId([first, second, third], "first", new Set(["first", "third"]))).toBeNull();
    expect(nextPendingMusicReviewTreeItemId([first, second, third], "third", new Set())).toBe("first");
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

  it("returns every ancestor needed to reveal an active track", () => {
    const tree = buildMusicReviewTree([
      item("theme", "Games/Nier/Disc 1/theme.flac"),
      item("root", "loose.mp3"),
    ]);

    expect(musicReviewTreeAncestorFolderIds(tree, "theme")).toEqual([
      "review-root:local",
      "review-folder:Games",
      "review-folder:Games/Nier",
      "review-folder:Games/Nier/Disc 1",
    ]);
    expect(musicReviewTreeAncestorFolderIds(tree, "root")).toEqual(["review-root:local"]);
  });

  it("returns every match with ancestor folders while expanding matching folders", () => {
    const tree = buildMusicReviewTree([
      item("theme", "Games/Nier/Disc 1/theme.flac"),
      item("ending", "Games/Nier/ending.flac"),
      item("other", "Games/Other/other.flac"),
    ]);

    const trackResult = searchMusicReviewTree(tree, "theme");
    expect(trackResult.matchCount).toBe(1);
    expect(trackResult.rows.map((row) => row.kind === "folder" ? row.node.name : row.item.id))
      .toEqual(["Music", "Games", "Nier", "Disc 1", "theme"]);

    const folderResult = searchMusicReviewTree(tree, "nier");
    expect(folderResult.matchCount).toBe(3);
    expect(folderResult.rows.map((row) => row.kind === "folder" ? row.node.name : row.item.id))
      .toEqual(["Music", "Games", "Nier", "ending", "Disc 1", "theme"]);
    expect(searchMusicReviewTree(tree, "missing")).toEqual({ rows: [], matchCount: 0 });
  });
});
