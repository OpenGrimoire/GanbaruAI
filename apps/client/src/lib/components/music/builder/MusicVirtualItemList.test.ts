// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { MusicItemListEntry } from "$lib/music/library-contracts";
import MusicVirtualItemList from "./MusicVirtualItemList.svelte";

const item: MusicItemListEntry = {
  id: "item-1",
  identityKey: "local:item-1",
  sourceKind: "local-file",
  mediaKind: "audio",
  title: "Quiet morning",
  artist: "Composer",
  album: "Soundtrack",
  relativePath: "Soundtrack/Quiet morning.flac",
  artworkOverride: null,
  durationMs: 120_000,
  availability: "available",
  reviewState: "reviewed",
  discoveredAt: 1,
  updatedAt: 1,
  version: 1,
  playlistCount: 1,
  activeSnoozeCount: 0,
  lastPlayedAt: null,
  playCount: 0,
  membershipId: "membership-1",
  membershipPosition: 0,
  membershipWeight: "normal",
  membershipEnabled: true,
  membershipVersion: 1,
};

describe("MusicVirtualItemList", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  beforeEach(() => {
    vi.stubGlobal("ResizeObserver", class {
      observe(): void {}
      disconnect(): void {}
    });
    vi.stubGlobal("matchMedia", vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })));
  });

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    vi.unstubAllGlobals();
    component = null;
    target = null;
  });

  it("exposes interactive rows as positioned list items and ignores editor arrows", async () => {
    const onSelect = vi.fn();
    const onReorder = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicVirtualItemList, {
      target,
      props: {
        items: [item],
        selectedItemId: item.id,
        selectedItemIds: [item.id],
        playlistMode: true,
        reorderEnabled: true,
        onSelect,
        onReorder,
      },
    });
    await tick();

    expect(target.querySelector('[role="list"]')).not.toBeNull();
    const row = target.querySelector<HTMLElement>('[role="listitem"]');
    expect(row?.getAttribute("aria-posinset")).toBe("1");
    expect(row?.getAttribute("aria-setsize")).toBe("1");
    expect(target.querySelector('[role="option"]')).toBeNull();
    expect(target.querySelector<HTMLButtonElement>('button[aria-pressed="true"]')).not.toBeNull();

    target.querySelector<HTMLButtonElement>('button[aria-haspopup="dialog"]')?.click();
    await tick();
    const positionInput = target.querySelector<HTMLInputElement>('input[inputmode="numeric"]');
    positionInput?.focus();
    positionInput?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }));
    expect(onSelect).not.toHaveBeenCalled();
    expect(onReorder).not.toHaveBeenCalled();
  });
});
