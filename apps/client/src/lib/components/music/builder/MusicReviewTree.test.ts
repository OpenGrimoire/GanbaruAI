// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MusicItemListEntry } from "$lib/music/library-contracts";
import MusicReviewTree from "./MusicReviewTree.svelte";

function item(id: string, title: string, relativePath: string): MusicItemListEntry {
  return {
    id, identityKey: id, sourceKind: "local-file", mediaKind: "audio", title, artist: "", album: "",
    relativePath, artworkOverride: null, durationMs: null, availability: "available", reviewState: "unreviewed",
    discoveredAt: 1, updatedAt: 1, version: 1, playlistCount: 0, activeSnoozeCount: 0,
    lastPlayedAt: null, playCount: 0, membershipId: null, membershipPosition: null,
    membershipWeight: null, membershipEnabled: null, membershipVersion: null,
  };
}

describe("MusicReviewTree", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("selects a complete folder and submits its descendants for playlist assignment", async () => {
    const onAssign = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicReviewTree, {
      target,
      props: {
        items: [item("one", "One", "Album/one.flac"), item("two", "Two", "Album/two.flac")],
        totalCount: 2,
        loading: false,
        activeItemId: "one",
        onActivate: vi.fn(),
        onAssign,
      },
    });
    await tick();

    const album = target.querySelector<HTMLInputElement>('input[aria-label="Select all 2 tracks in Album"]');
    expect(album).not.toBeNull();
    album?.click();
    await tick();
    const assign = [...target.querySelectorAll<HTMLButtonElement>("button")]
      .find((button) => button.textContent?.includes("Add 2 selected to playlists"));
    expect(assign).not.toBeNull();
    assign?.click();

    expect(onAssign).toHaveBeenCalledWith(["one", "two"]);
  });

  it("opens every nested folder when Review first appears", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicReviewTree, {
      target,
      props: {
        items: [
          item("theme", "Theme", "Games/Nier/Disc 1/theme.flac"),
          item("ending", "Ending", "Games/Nier/ending.flac"),
        ],
        totalCount: 2,
        loading: false,
        activeItemId: null,
        onActivate: vi.fn(),
        onAssign: vi.fn(),
      },
    });
    await tick();

    expect(
      [...target.querySelectorAll<HTMLButtonElement>("button[aria-expanded]")]
        .map((button) => button.getAttribute("aria-label")),
    ).toEqual([
      "Collapse Music",
      "Collapse Games",
      "Collapse Nier",
      "Collapse Disc 1",
    ]);
    expect(target.textContent).toContain("Theme");
    expect(target.textContent).toContain("Ending");
  });
});
