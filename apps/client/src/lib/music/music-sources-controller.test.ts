import { describe, expect, it, vi } from "vitest";
import type { MusicSourceRefreshController } from "$lib/music/music-source-refresh";
import {
  createMusicSourcesController,
  type MusicSourcesControllerApi,
} from "$lib/music/music-sources-controller.svelte";

function refreshStub(): MusicSourceRefreshController {
  return {
    prepare: (targets) => ({
      id: 1,
      targets,
      localCount: targets.filter((target) => target.kind === "local-root").length,
      onlineCount: targets.filter((target) => target.kind === "youtube-playlist").length,
      requiresNetworkConfirmation: targets.some((target) => target.kind === "youtube-playlist"),
    }),
    run: vi.fn(async () => []),
    cancel: vi.fn(async () => undefined),
    statuses: () => [],
  };
}

function api(overrides: Partial<MusicSourcesControllerApi> = {}): MusicSourcesControllerApi {
  return {
    roots: vi.fn(async () => []),
    collections: vi.fn(async () => []),
    bindings: vi.fn(async () => []),
    pickFolder: vi.fn(async () => null),
    pickFile: vi.fn(async () => null),
    createRoot: vi.fn(async (request) => ({ id: request.collectionId, version: 1 })),
    bindRoot: vi.fn(async (_vaultId, rootId, folderPath) => ({ rootId, folderPath, status: "available" as const })),
    clearBinding: vi.fn(async (_vaultId, rootId) => ({ rootId, folderPath: null, status: "needs-relink" as const })),
    previewItemRepair: vi.fn(async (itemId, filePath) => ({
      itemId, folderPath: filePath, relativePath: "track.flac", title: "Track", artist: "", album: "",
      durationMs: 1, fileSizeBytes: 1, lightweightFingerprint: "sample:1", strongFingerprint: "sha256:1",
      matchStrength: "exact" as const, reasons: ["match"],
    })),
    applyItemRepair: vi.fn(async (request) => ({ id: request.locationId, version: 1 })),
    undoItemRepair: vi.fn(async () => undefined),
    resolveYouTube: vi.fn(async (source) => ({
      kind: source.kind,
      videoId: source.kind === "youtube-video" ? source.videoId : source.videoId,
      playlistId: source.kind === "youtube-playlist" ? source.playlistId : null,
      title: source.title,
      channel: "Channel",
      durationMs: 120_000,
      videoIds: source.kind === "youtube-video" ? [source.videoId] : ["video-1", "video-2"],
      duplicateCount: 0,
    })),
    youtubeDuplicateCount: vi.fn(async () => 0),
    saveYouTubeVideo: vi.fn(async (request) => ({ id: request.videoId, version: 1 })),
    saveYouTubePlaylist: vi.fn(async (request) => ({
      collectionId: request.collectionId,
      canonicalItemCount: request.videoIds.length,
      newlyDiscoveredCount: request.videoIds.length,
      repeatedVideoCount: 0,
      generation: 1,
    })),
    removalImpact: vi.fn(async (collectionId) => ({
      collectionId, itemCount: 0, membershipCount: 0, sharedItemCount: 0,
      orphanedItemCount: 0, activeRefreshCount: 0,
    })),
    removeSource: vi.fn(async (request) => request.expectedImpact),
    createRelink: vi.fn(async (request) => ({
      id: request.planId, rootId: request.rootId, state: "ready" as const, exactCount: 0,
      likelyCount: 0, ambiguousCount: 0, missingCount: 0, newCount: 0,
      createdAt: request.createdAt, updatedAt: request.createdAt,
    })),
    relinkEntries: vi.fn(async (_planId, offset, limit) => ({ entries: [], totalCount: 0, offset, limit })),
    applyRelink: vi.fn(async (request) => ({
      id: request.planId, rootId: "root-1", state: "applied" as const, exactCount: 0,
      likelyCount: 0, ambiguousCount: 0, missingCount: 0, newCount: 0,
      createdAt: request.appliedAt, updatedAt: request.appliedAt,
    })),
    cancelRelink: vi.fn(async (planId, cancelledAt) => ({
      id: planId, rootId: "root-1", state: "cancelled" as const, exactCount: 0,
      likelyCount: 0, ambiguousCount: 0, missingCount: 0, newCount: 0,
      createdAt: cancelledAt, updatedAt: cancelledAt,
    })),
    ...overrides,
  };
}

describe("MusicSourcesController", () => {
  it("loads roots, collections, and device bindings as one vault projection", async () => {
    const controller = createMusicSourcesController(api({
      roots: vi.fn(async () => [{ id: "root-1", name: "OST", createdAt: 1, updatedAt: 1, version: 1 }]),
      bindings: vi.fn(async () => [{ rootId: "root-1", folderPath: "/music/ost", status: "available" as const }]),
    }), () => 10, () => "id", refreshStub());
    controller.setVault("vault-1");
    expect(await controller.load()).toBe(true);
    expect(controller.roots[0]?.name).toBe("OST");
    expect(controller.bindings[0]?.folderPath).toBe("/music/ost");
  });

  it("preserves folder preview and reports duplicate relationships before writing", async () => {
    const createRoot = vi.fn(async (request: Parameters<MusicSourcesControllerApi["createRoot"]>[0]) => ({ id: request.collectionId, version: 1 }));
    const controller = createMusicSourcesController(api({
      pickFolder: vi.fn(async () => ({ folderPath: "/music/ost", tracks: [], truncated: false })),
      createRoot,
    }), () => 10, () => "id", refreshStub());
    controller.setVault("vault-1");
    controller.bindings = [{ rootId: "root-1", folderPath: "/music/ost/", status: "available" }];
    const draft = await controller.chooseLocalFolder();
    expect(draft).toMatchObject({ name: "ost", relationship: "duplicate" });
    expect(createRoot).not.toHaveBeenCalled();
  });

  it("distinguishes videos from playlists and saves a resolved video", async () => {
    const saveYouTubeVideo = vi.fn(async (request: Parameters<MusicSourcesControllerApi["saveYouTubeVideo"]>[0]) => ({ id: request.videoId, version: 1 }));
    const controller = createMusicSourcesController(api({ saveYouTubeVideo }), () => 20, () => "id", refreshStub());
    expect(controller.parseYouTubeInput("https://youtu.be/abcDEF_1234", "youtube-playlist")).toMatchObject({ source: null });
    const parsed = controller.parseYouTubeInput("https://youtu.be/abcDEF_1234", "youtube-video");
    expect(parsed.source?.kind).toBe("youtube-video");
    const preview = await controller.resolveYouTube(parsed.source!);
    await controller.addYouTube(preview!, "");
    expect(saveYouTubeVideo).toHaveBeenCalledWith(expect.objectContaining({
      videoId: "abcDEF_1234",
      resolutionState: "ready",
    }));
  });

  it("cancels an obsolete YouTube resolution without exposing its result", async () => {
    let release!: () => void;
    const resolveYouTube = vi.fn(async (source: Parameters<MusicSourcesControllerApi["resolveYouTube"]>[0], signal: AbortSignal) => {
      await new Promise<void>((resolve, reject) => {
        release = resolve;
        signal.addEventListener("abort", () => reject(new DOMException("cancelled", "AbortError")), { once: true });
      });
      return { kind: source.kind, videoId: null, playlistId: null, title: "Late", channel: "", durationMs: null, videoIds: [], duplicateCount: 0 };
    });
    const controller = createMusicSourcesController(api({ resolveYouTube }), () => 20, () => "id", refreshStub());
    const parsed = controller.parseYouTubeInput("https://youtu.be/abcDEF_1234", "youtube-video");
    const pending = controller.resolveYouTube(parsed.source!);
    controller.cancelResolution();
    release();
    expect(await pending).toBeNull();
    expect(controller.youtubePreview).toBeNull();
  });

  it("applies a confirmed item repair as another location and can undo it", async () => {
    const applyItemRepair = vi.fn(async (request: Parameters<MusicSourcesControllerApi["applyItemRepair"]>[0]) => ({ id: request.locationId, version: 1 }));
    const bindRoot = vi.fn(async (_vaultId: string, rootId: string, folderPath: string) => ({ rootId, folderPath, status: "available" as const }));
    const undoItemRepair = vi.fn(async () => undefined);
    const clearBinding = vi.fn(async (_vaultId: string, rootId: string) => ({ rootId, folderPath: null, status: "needs-relink" as const }));
    const ids = ["repair-root", "repair-location"];
    const controller = createMusicSourcesController(api({ applyItemRepair, bindRoot, undoItemRepair, clearBinding }), () => 30, () => ids.shift()!, refreshStub());
    controller.setVault("vault-1");
    controller.itemRepairPreview = {
      itemId: "item-1", folderPath: "/music/recovered", relativePath: "track.flac",
      title: "Track", artist: "", album: "", durationMs: 1, fileSizeBytes: 1,
      lightweightFingerprint: "sample:1", strongFingerprint: "sha256:1",
      matchStrength: "weak", reasons: ["weak"],
    };

    await controller.applyItemRepair(true);
    expect(applyItemRepair).toHaveBeenCalledWith(expect.objectContaining({
      itemId: "item-1",
      acceptWeakMismatch: true,
    }));
    expect(bindRoot).toHaveBeenCalledWith("vault-1", "repair-root", "/music/recovered");
    await controller.undoItemRepair();
    expect(undoItemRepair).toHaveBeenCalledWith("repair-location", "repair-root");
    expect(clearBinding).toHaveBeenCalledWith("vault-1", "repair-root");
  });
});
