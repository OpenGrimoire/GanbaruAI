import { describe, expect, it, vi } from "vitest";
import type { MusicLibraryController } from "$lib/music/music-library-controller.svelte";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
import { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";

const { applyMusicReviewSelection, bulkEditMusicMemberships, bulkSetMusicReviewState, getMusicMembershipMatrix } = vi.hoisted(() => ({
  applyMusicReviewSelection: vi.fn(async () => ({
    membershipChangedCount: 2,
    reviewChangedCount: 2,
    items: [{ id: "item-1", version: 2 }, { id: "item-2", version: 3 }],
  })),
  bulkEditMusicMemberships: vi.fn(async () => ({ changedCount: 1 })),
  bulkSetMusicReviewState: vi.fn(async () => ({ changedCount: 2 })),
  getMusicMembershipMatrix: vi.fn(async () => [
    { itemId: "item-1", playlistId: "focus", weight: "normal" },
    { itemId: "item-2", playlistId: "all", weight: "normal" },
    { itemId: "item-1", playlistId: "all", weight: "normal" },
  ]),
}));

vi.mock("$lib/api/music-library", () => ({
  applyMusicReviewSelection,
  getMusicMembershipMatrix,
  bulkEditMusicMemberships,
  bulkSetMusicReviewState,
  createMusicPlaylist: vi.fn(async () => ({ id: "created", version: 1 })),
}));

const summary = (id: string): MusicPlaylistSummary => ({
  id, sortOrder: 0, name: id, icon: "lucide:list-music", shuffleEnabled: true, repeatMode: "all", intendedUses: [],
  totalCount: 0, eligibleCount: 0, unavailableCount: 0, snoozedCount: 0, localCount: 0,
  onlineCount: 0, version: 1,
});

describe("MusicBulkEditController", () => {
  it("projects checked, mixed, and unchecked playlist states without losing mixed intent", async () => {
    const library = { refreshAfterMutation: vi.fn(async () => true) } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("focus"), summary("empty")]);
    expect(controller.states).toEqual({ all: "checked", focus: "mixed", empty: "unchecked" });
    expect(controller.hasExistingMemberships).toBe(true);
    controller.toggle("focus");
    expect(controller.states.focus).toBe("checked");
    expect(controller.membershipsChanged).toBe(true);
    controller.toggle("focus");
    expect(controller.states.focus).toBe("unchecked");
    controller.toggle("focus");
    expect(controller.states.focus).toBe("mixed");
    expect(controller.membershipsChanged).toBe(false);
    expect(controller.touchedPlaylistIds).not.toContain("focus");
  });

  it("applies memberships and marks every selected track reviewed without replacing the item window", async () => {
    const library = {
      currentWindow: { items: [
        { id: "item-1", reviewState: "unreviewed", updatedAt: 1, version: 1 },
        { id: "item-2", reviewState: "deferred", updatedAt: 1, version: 2 },
      ] },
      refreshAfterMutation: vi.fn(async () => true),
      refreshSummariesAfterMutation: vi.fn(async () => true),
    } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("focus")]);
    controller.toggle("focus");

    expect(await controller.saveReviewSelection()).toBe(true);
    expect(applyMusicReviewSelection).toHaveBeenCalledWith(expect.objectContaining({
      items: [{ itemId: "item-1", expectedVersion: 1 }, { itemId: "item-2", expectedVersion: 2 }],
      reviewState: "reviewed",
      addPlaylistIds: ["focus"],
    }));
    expect(library.currentWindow.items).toEqual([
      { id: "item-1", reviewState: "reviewed", updatedAt: 100, version: 2 },
      { id: "item-2", reviewState: "reviewed", updatedAt: 100, version: 3 },
    ]);
    expect(library.refreshAfterMutation).not.toHaveBeenCalled();
    expect(library.refreshSummariesAfterMutation).toHaveBeenCalledOnce();
  });

  it("ignores an unassigned selection and removes it from the visible Review window", async () => {
    getMusicMembershipMatrix.mockResolvedValueOnce([]);
    const library = {
      currentWindow: {
        items: [
          { id: "item-1", reviewState: "unreviewed", updatedAt: 1, version: 1 },
          { id: "item-2", reviewState: "deferred", updatedAt: 1, version: 2 },
        ],
        totalCount: 4,
      },
      refreshAfterMutation: vi.fn(async () => true),
      refreshSummariesAfterMutation: vi.fn(async () => true),
    } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("focus")]);

    expect(controller.hasExistingMemberships).toBe(false);
    expect(await controller.ignoreReviewSelection()).toBe(true);
    expect(applyMusicReviewSelection).toHaveBeenCalledWith({
      actionId: "action",
      items: [{ itemId: "item-1", expectedVersion: 1 }, { itemId: "item-2", expectedVersion: 2 }],
      reviewState: "ignored",
      addPlaylistIds: [],
      removePlaylistIds: [],
      updatedAt: 100,
    });
    expect(library.currentWindow.items).toEqual([]);
    expect(library.currentWindow.totalCount).toBe(2);
    expect(library.refreshAfterMutation).not.toHaveBeenCalled();
    expect(library.refreshSummariesAfterMutation).toHaveBeenCalledOnce();
  });

  it("does not ignore a selection that already belongs to a playlist", async () => {
    const library = {
      currentWindow: { items: [
        { id: "item-1", reviewState: "unreviewed", updatedAt: 1, version: 1 },
        { id: "item-2", reviewState: "unreviewed", updatedAt: 1, version: 1 },
      ] },
    } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("focus")]);
    const callCount = applyMusicReviewSelection.mock.calls.length;

    expect(await controller.ignoreReviewSelection()).toBe(false);
    expect(applyMusicReviewSelection).toHaveBeenCalledTimes(callCount);
  });

  it("keeps explicit playlist choices while the inline tree selection changes", async () => {
    const library = { refreshAfterMutation: vi.fn(async () => true) } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1"], [summary("all"), summary("empty")]);
    controller.toggle("empty");
    await controller.open(["item-1", "item-2"], [summary("all"), summary("empty")], true);

    expect(controller.states.empty).toBe("checked");
    expect(controller.touchedPlaylistIds).toEqual(new Set(["empty"]));
  });

  it("keeps the current playlist projection visible while an expanded selection loads", async () => {
    const library = { refreshAfterMutation: vi.fn(async () => true) } as unknown as MusicLibraryController;
    const controller = new MusicBulkEditController(library, () => 100, () => "action");
    await controller.open(["item-1"], [summary("all"), summary("empty")]);
    controller.toggle("empty");
    let resolveMatrix!: (value: Array<{ itemId: string; playlistId: string; weight: "normal" }>) => void;
    getMusicMembershipMatrix.mockImplementationOnce(() => new Promise((resolve) => { resolveMatrix = resolve; }));

    const loading = controller.open(["item-1", "item-2"], [summary("all"), summary("empty")], true);

    expect(controller.loading).toBe(true);
    expect(controller.states.empty).toBe("checked");
    resolveMatrix([{ itemId: "item-1", playlistId: "all", weight: "normal" }]);
    expect(await loading).toBe(true);
    expect(controller.states.empty).toBe("checked");
  });
});
