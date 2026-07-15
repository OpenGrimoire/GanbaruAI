import { listen } from "@tauri-apps/api/event";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_PLAYBACK_SNAPSHOT } from "$lib/music/playback";
import { localFileSourceFromPath, youtubeVideoSourceFromId, type MusicSource } from "$lib/music/sources";
import { updateMediaControls } from "$lib/api/media-controls";
import { createMusicExternalControls } from "./music-external-controls";

vi.mock("$lib/api/media-controls", () => ({ updateMediaControls: vi.fn(async () => undefined) }));

function createContext() {
  const unlisten = vi.fn();
  const listenMock = vi.fn(async () => unlisten) as unknown as typeof listen;
  const context: Parameters<typeof createMusicExternalControls>[0] = {
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
  };
  return {
    context,
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

    expect(listenMock).toHaveBeenCalledTimes(4);
    controls.destroy();
    expect(unlisten).toHaveBeenCalledTimes(4);
    expect(controls.isInitialized()).toBe(false);
  });

  it("publishes queue availability for local and YouTube saved items", async () => {
    const { context } = createContext();
    let source: MusicSource = localFileSourceFromPath("/music/local.flac", "Local");
    let canPrevious = false;
    let canNext = true;
    context.currentSource = () => source;
    context.canPrevious = () => canPrevious;
    context.canNext = () => canNext;
    const controls = createMusicExternalControls(context);

    controls.updateNative();
    await vi.waitFor(() => expect(updateMediaControls).toHaveBeenLastCalledWith(
      expect.objectContaining({ title: "Nothing loaded", canPrevious: false, canNext: true }),
    ));
    source = youtubeVideoSourceFromId("dQw4w9WgXcQ");
    canPrevious = true;
    canNext = false;
    controls.updateNative();
    await vi.waitFor(() => expect(updateMediaControls).toHaveBeenLastCalledWith(
      expect.objectContaining({ canPrevious: true, canNext: false }),
    ));
  });

  it("disables native navigation when unavailable or no item is eligible", async () => {
    const { context } = createContext();
    context.currentSource = () => localFileSourceFromPath("/music/only.flac", "Only");
    const controls = createMusicExternalControls(context);
    controls.updateNative();
    await vi.waitFor(() => expect(updateMediaControls).toHaveBeenLastCalledWith(
      expect.objectContaining({ canPrevious: false, canNext: false }),
    ));

    context.currentSource = () => null;
    controls.updateNative();
    await vi.waitFor(() => expect(updateMediaControls).toHaveBeenLastCalledWith(
      expect.objectContaining({ title: null, canPlayPause: false, canPrevious: false, canNext: false }),
    ));
  });
});
