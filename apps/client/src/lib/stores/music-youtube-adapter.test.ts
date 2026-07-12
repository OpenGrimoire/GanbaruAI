import { describe, expect, it, vi } from "vitest";
import { DEFAULT_PLAYBACK_SNAPSHOT } from "$lib/music/playback";
import {
  parseMusicSourceInput,
  youtubeVideoSourceFromId,
} from "$lib/music/sources";
import { MusicLoadRuntime } from "./music-load-runtime";
import {
  createMusicYouTubeAdapter,
  type MusicYouTubeState,
} from "./music-youtube-adapter";

describe("Music YouTube adapter", () => {
  it("ignores stale playing snapshots during the optimistic pause window", async () => {
    let nowMs = 100;
    const source = youtubeVideoSourceFromId("video-1");
    const contentWindow = { postMessage: vi.fn() };
    const state: MusicYouTubeState = {
      currentSource: source,
      snapshot: { ...DEFAULT_PLAYBACK_SNAPSHOT, status: "paused" },
      playerError: null,
      queue: [source],
      queueHistory: [],
      shuffleEnabled: false,
      shuffleOrder: [],
      pendingQueueIndex: null,
      youtubeHostUrl: null,
      youtubeFrame: { contentWindow } as unknown as HTMLIFrameElement,
      youtubeHostToken: null,
      youtubeHostReady: false,
    };
    const loadRuntime = new MusicLoadRuntime();
    const generation = loadRuntime.begin();
    const adapter = createMusicYouTubeAdapter({
      state,
      loadRuntime,
      effectiveVolume: () => 1,
      loadSource: vi.fn(async () => undefined),
      persist: vi.fn(async () => undefined),
      updateExternalControls: vi.fn(),
      updateTray: vi.fn(),
      canPlayNext: () => false,
      playNext: vi.fn(async () => undefined),
      getHostUrl: vi.fn(async () => "http://127.0.0.1:1234/player?token=test-token"),
      now: () => nowMs,
    });
    await adapter.load(source, null, generation, false);
    adapter.beginOptimisticPause();
    const message = {
      token: "test-token",
      load: String(generation),
      type: "ganbaru-ai-youtube-state",
      status: "playing",
      positionMs: 500,
      durationMs: 10_000,
      videoId: "video-1",
      title: "Video 1",
    };

    adapter.handleMessage({
      source: contentWindow,
      data: message,
    } as unknown as MessageEvent<unknown>);
    expect(state.snapshot.status).toBe("paused");

    nowMs = 2_000;
    adapter.handleMessage({
      source: contentWindow,
      data: message,
    } as unknown as MessageEvent<unknown>);
    expect(state.snapshot.status).toBe("playing");
  });

  it("rejects a playlist result after a newer source generation starts", async () => {
    const parsed = parseMusicSourceInput(
      "https://www.youtube.com/playlist?list=PL1234567890",
    );
    if (!parsed.source || parsed.source.kind !== "youtube-playlist") {
      throw new Error("Expected a YouTube playlist fixture");
    }
    const contentWindow = { postMessage: vi.fn() };
    const state: MusicYouTubeState = {
      currentSource: parsed.source,
      snapshot: { ...DEFAULT_PLAYBACK_SNAPSHOT, status: "loading" },
      playerError: null,
      queue: [parsed.source],
      queueHistory: [],
      shuffleEnabled: false,
      shuffleOrder: [],
      pendingQueueIndex: null,
      youtubeHostUrl: null,
      youtubeFrame: { contentWindow } as unknown as HTMLIFrameElement,
      youtubeHostToken: null,
      youtubeHostReady: false,
    };
    const loadRuntime = new MusicLoadRuntime();
    const playlistGeneration = loadRuntime.begin();
    const loadSource = vi.fn(async () => undefined);
    const adapter = createMusicYouTubeAdapter({
      state,
      loadRuntime,
      effectiveVolume: () => 1,
      loadSource,
      persist: vi.fn(async () => undefined),
      updateExternalControls: vi.fn(),
      updateTray: vi.fn(),
      canPlayNext: () => false,
      playNext: vi.fn(async () => undefined),
      getHostUrl: vi.fn(async () => "http://127.0.0.1:1234/player?token=test-token"),
    });
    await adapter.load(parsed.source, null, playlistGeneration, false);
    loadRuntime.begin();

    adapter.handleMessage({
      source: contentWindow,
      data: {
        token: "test-token",
        load: String(playlistGeneration),
        type: "ganbaru-ai-youtube-playlist",
        playlistId: parsed.source.playlistId,
        videoIds: ["video-a", "video-b"],
        index: 0,
      },
    } as unknown as MessageEvent<unknown>);
    await Promise.resolve();

    expect(loadSource).not.toHaveBeenCalled();
    expect(state.queue).toEqual([parsed.source]);
  });
});
