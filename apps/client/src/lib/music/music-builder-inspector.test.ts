import { describe, expect, it } from "vitest";
import type { MusicInspectorDetail } from "./library-contracts";
import { MusicBuilderInspectorController } from "./music-builder-inspector.svelte";

function detail(id: string): MusicInspectorDetail {
  return {
    item: {
      id, identityKey: `local:${id}`, sourceKind: "local-file", mediaKind: "audio",
      youtubeVideoId: null, originalTitle: id, originalArtist: "", originalAlbum: "",
      originalTrackNumber: null, originalArtworkIdentity: null, youtubeResolutionState: null,
      titleOverride: null, artistOverride: null, albumOverride: null, artworkOverride: null,
      durationMs: null, availability: "available", reviewState: "unreviewed",
      reviewChangedAt: null, reviewDeferredUntil: null, discoveredAt: 1, updatedAt: 1, version: 1,
    },
    locations: [], memberships: [], membershipSkipRanges: [], snoozes: [], signals: [], statistics: null,
    sourceCollectionIds: [],
  };
}

describe("music builder inspector controller", () => {
  it("rejects stale details after selection changes", async () => {
    let resolveOld!: (detail: MusicInspectorDetail) => void;
    const old = new Promise<MusicInspectorDetail>((resolve) => { resolveOld = resolve; });
    const controller = new MusicBuilderInspectorController({
      detail: (id) => id === "old" ? old : Promise.resolve(detail(id)),
      setSignals: async () => 2,
    });
    const oldSelection = controller.select("old");
    expect(await controller.select("new")).toBe(true);
    resolveOld(detail("old"));
    expect(await oldSelection).toBe(false);
    expect(controller.detail?.item.id).toBe("new");
  });

  it("saves canonical signals optimistically and restores them on failure", async () => {
    let shouldFail = false;
    const controller = new MusicBuilderInspectorController({
      detail: async (id) => detail(id),
      setSignals: async () => {
        if (shouldFail) throw new Error("save failed");
        return 2;
      },
    });
    await controller.select("item-1");
    expect(await controller.saveSignals(["lyrics", "calm"])).toBe(true);
    expect(controller.detail?.signals).toEqual(["lyrics", "calm"]);
    expect(controller.detail?.item.version).toBe(2);

    expect(await controller.undoSignals()).toBe(true);
    expect(controller.detail?.signals).toEqual([]);

    shouldFail = true;
    expect(await controller.saveSignals(["energizing"])).toBe(false);
    expect(controller.detail?.signals).toEqual([]);
  });

  it("preserves section expansion while inspector presentation changes", () => {
    const controller = new MusicBuilderInspectorController();
    controller.toggleSection("provenance");
    expect(controller.expandedSections.has("provenance")).toBe(true);
    controller.toggleSection("provenance");
    expect(controller.expandedSections.has("provenance")).toBe(false);
  });
});
