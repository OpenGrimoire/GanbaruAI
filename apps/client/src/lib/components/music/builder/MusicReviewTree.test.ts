// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MusicItemListEntry } from "$lib/music/library-contracts";
import MusicReviewTree from "./MusicReviewTree.svelte";

function item(id: string, title: string, relativePath: string): MusicItemListEntry {
  return {
    id, identityKey: id, sourceKind: "local-file", mediaKind: "audio", title, artist: "", album: "",
    localRootId: "root-1", relativePath, originalArtworkIdentity: null, artworkOverride: null, durationMs: null, availability: "available", reviewState: "unreviewed",
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

  it("propagates folder selection downward without marking ancestors", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicReviewTree, {
      target,
      props: {
        items: [item("one", "One", "Album/one.flac"), item("two", "Two", "Album/two.flac")],
        totalCount: 2,
        activeItemId: null,
        onActivate: vi.fn(),
        onAssign: vi.fn(),
      },
    });
    await tick();

    const music = target.querySelector<HTMLInputElement>('input[aria-label="Select all 2 tracks in Music"]');
    const album = target.querySelector<HTMLInputElement>('input[aria-label="Select all 2 tracks in Album"]');
    const one = target.querySelector<HTMLInputElement>('input[aria-label="Select One"]');
    expect(music).not.toBeNull();
    expect(album).not.toBeNull();
    expect(one).not.toBeNull();

    one?.click();
    await tick();
    expect(one?.checked).toBe(true);
    expect(album?.checked).toBe(false);
    expect(album?.indeterminate).toBe(false);
    expect(music?.checked).toBe(false);
    expect(music?.indeterminate).toBe(false);

    album?.click();
    await tick();
    expect(album?.checked).toBe(true);
    expect(music?.checked).toBe(false);
    expect(music?.indeterminate).toBe(false);
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
        activeItemId: null,
        onActivate: vi.fn(),
        onAssign: vi.fn(),
      },
    });
    expect(target.textContent).toContain("Theme");
    expect(target.textContent).toContain("Ending");
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
  });

  it("shows every search match with folder context and clears with Escape", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicReviewTree, {
      target,
      props: {
        items: [
          item("morning", "Morning Theme", "Albums/Morning/theme.flac"),
          item("night", "Night Theme", "Albums/Night/theme.flac"),
          item("ending", "Ending", "Albums/Ending/ending.flac"),
        ],
        totalCount: 3,
        activeItemId: null,
        onActivate: vi.fn(),
        onAssign: vi.fn(),
      },
    });
    await tick();

    const search = target.querySelector<HTMLInputElement>('input[aria-label="Search folders and tracks"]');
    expect(search).not.toBeNull();
    if (!search) return;
    search.value = "theme";
    search.dispatchEvent(new InputEvent("input", { bubbles: true }));
    await tick();

    expect(target.textContent).toContain("Morning Theme");
    expect(target.textContent).toContain("Night Theme");
    expect(target.textContent).not.toContain("Ending");
    expect(target.querySelector('[aria-label="2 matches"]')).not.toBeNull();

    search.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await tick();
    expect(search.value).toBe("");
    expect(target.textContent).toContain("Ending");
  });

  it("requests a local-folder refresh from the search toolbar", async () => {
    const onRefresh = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicReviewTree, {
      target,
      props: {
        items: [item("one", "One", "Album/one.flac")],
        totalCount: 1,
        activeItemId: null,
        onActivate: vi.fn(),
        onAssign: vi.fn(),
        onRefresh,
      },
    });
    await tick();

    target.querySelector<HTMLButtonElement>('button[aria-label="Refresh local folders"]')?.click();
    expect(onRefresh).toHaveBeenCalledOnce();
  });
});
