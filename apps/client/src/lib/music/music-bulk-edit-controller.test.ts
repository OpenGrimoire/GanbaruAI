import { describe, expect, it, vi } from "vitest";
import type { MusicLibraryController } from "$lib/music/music-library-controller.svelte";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
import { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";

vi.mock("$lib/api/music-library", () => ({
  getMusicMembershipMatrix: vi.fn(async () => [
    { itemId: "item-1", playlistId: "focus", weight: "normal" },
    { itemId: "item-2", playlistId: "all", weight: "normal" },
    { itemId: "item-1", playlistId: "all", weight: "normal" },
  ]),
  bulkEditMusicMemberships: vi.fn(async () => ({ changedCount: 1 })),
}));

const summary = (id: string): MusicPlaylistSummary => ({
  id, name: id, description: "", shuffleEnabled: true, repeatMode: "all", intendedUses: [],
  totalCount: 0, eligibleCount: 0, unavailableCount: 0, snoozedCount: 0, localCount: 0,
  onlineCount: 0, version: 1,
});

describe("MusicBulkEditController", () => {
  it("projects checked, mixed, and unchecked playlist states without losing mixed intent", async () => {
    const library = { refresh: vi.fn(async () => true) } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("focus"), summary("empty")]);
    expect(controller.states).toEqual({ all: "checked", focus: "mixed", empty: "unchecked" });
    controller.toggle("focus");
    expect(controller.states.focus).toBe("checked");
  });
});
