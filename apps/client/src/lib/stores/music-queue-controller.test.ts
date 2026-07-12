import { describe, expect, it, vi } from "vitest";
import { localFileSourceFromPath } from "$lib/music/sources";
import { createMusicQueueController, type MusicQueueState } from "./music-queue-controller";

function createState(): MusicQueueState {
  const queue = [
    localFileSourceFromPath("/music/a.mp3", "a"),
    localFileSourceFromPath("/music/b.mp3", "b"),
    localFileSourceFromPath("/music/c.mp3", "c"),
  ];
  return {
    currentSource: queue[0],
    queue,
    shuffleEnabled: false,
    shuffleExplicit: false,
    shuffleOrder: [],
    queueHistory: [],
    pendingQueueIndex: null,
  };
}

describe("Music queue controller", () => {
  it("records navigation history before loading the next source", async () => {
    const state = createState();
    const loadSource = vi.fn(async () => undefined);
    const controller = createMusicQueueController({
      state,
      isBusy: () => false,
      loadSource,
      persistSettings: vi.fn(),
      updateExternalControls: vi.fn(),
      updateTray: vi.fn(),
    });

    await controller.playNext();
    expect(state.queueHistory).toEqual([0]);
    expect(state.pendingQueueIndex).toBe(1);
    expect(loadSource).toHaveBeenCalledWith(state.queue[1]);
  });

  it("uses history before linear previous navigation", async () => {
    const state = createState();
    state.currentSource = state.queue[2];
    state.queueHistory = [0];
    const loadSource = vi.fn(async () => undefined);
    const controller = createMusicQueueController({
      state,
      isBusy: () => false,
      loadSource,
      persistSettings: vi.fn(),
      updateExternalControls: vi.fn(),
      updateTray: vi.fn(),
    });

    await controller.playPrevious();
    expect(state.queueHistory).toEqual([]);
    expect(loadSource).toHaveBeenCalledWith(state.queue[0]);
  });
});
