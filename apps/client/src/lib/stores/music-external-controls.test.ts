import { listen } from "@tauri-apps/api/event";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_PLAYBACK_SNAPSHOT } from "$lib/music/playback";
import { createMusicExternalControls } from "./music-external-controls";

function createContext() {
  const unlisten = vi.fn();
  const listenMock = vi.fn(async () => unlisten) as unknown as typeof listen;
  return {
    context: {
      currentSource: () => null,
      snapshot: () => DEFAULT_PLAYBACK_SNAPSHOT,
      title: () => "Nothing loaded",
      sourceKindLabel: () => "No source",
      artworkUrl: () => null,
      isBusy: () => false,
      canPrevious: () => false,
      canNext: () => false,
      volume: () => 1,
      muted: () => false,
      shuffleEnabled: () => true,
      play: vi.fn(async () => undefined),
      pause: vi.fn(async () => undefined),
      togglePlay: vi.fn(async () => undefined),
      stop: vi.fn(async () => undefined),
      previous: vi.fn(async () => undefined),
      next: vi.fn(async () => undefined),
      seekBy: vi.fn(async () => undefined),
      seekTo: vi.fn(async () => undefined),
      setVolume: vi.fn(async () => undefined),
      setRate: vi.fn(async () => undefined),
      toggleShuffle: vi.fn(),
      handleWindowMessage: vi.fn(),
      listen: listenMock,
    },
    listenMock,
    unlisten,
  };
}

describe("Music external controls", () => {
  it("owns one listener set and releases every listener on destroy", async () => {
    const { context, listenMock, unlisten } = createContext();
    const controls = createMusicExternalControls(context);

    controls.init();
    controls.init();
    await Promise.resolve();

    expect(listenMock).toHaveBeenCalledTimes(5);
    controls.destroy();
    expect(unlisten).toHaveBeenCalledTimes(5);
    expect(controls.isInitialized()).toBe(false);
  });
});
