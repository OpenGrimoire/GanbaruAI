import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadLocalMedia } from "$lib/api/media-player";
import { getPlaybackState, pickMediaFolder } from "$lib/api/music";
import { DEFAULT_PLAYBACK_SNAPSHOT } from "$lib/music/playback";
import {
  localFileSourceFromPath,
  youtubeVideoSourceFromId,
} from "$lib/music/sources";
import { MusicLoadRuntime } from "./music-load-runtime";
import {
  createMusicSourceController,
  type MusicSourceState,
} from "./music-source-controller";

vi.mock("$lib/api/music", () => ({
  getPlaybackState: vi.fn(async () => null),
  pickMediaFolder: vi.fn(async () => null),
  registerEmbeddedArtwork: vi.fn(async () => null),
  registerMediaFile: vi.fn(async () => null),
}));

vi.mock("$lib/api/media-player", () => ({
  loadLocalMedia: vi.fn(async () => ({
    status: "ready",
    positionMs: 0,
    durationMs: 60_000,
    hasVideo: false,
    backendKind: "rodio",
    playableStartMs: 0,
    error: null,
  })),
  mediaPlayerErrorMessage: (error: unknown) => String(error),
  setLocalMuted: vi.fn(async () => ({
    status: "ready",
    positionMs: 0,
    durationMs: 60_000,
    hasVideo: false,
    backendKind: "rodio",
    playableStartMs: 0,
    error: null,
  })),
}));

function createState(): MusicSourceState {
  return {
    sourceInput: "",
    currentSource: null,
    parseError: null,
    playerError: null,
    snapshot: { ...DEFAULT_PLAYBACK_SNAPSHOT },
    queue: [],
    folderScanTruncated: false,
    shuffleEnabled: false,
    shuffleOrder: [],
    queueHistory: [],
    pendingQueueIndex: null,
    sourceActionBusy: false,
    muted: false,
    localMediaSrc: null,
    localHasVideo: false,
    localBackendKind: "none",
    localVideoReady: false,
    currentArtworkUrl: null,
    activePlaylistId: null,
    activeQueueItemIds: [],
  };
}

function createContext(state: MusicSourceState) {
  const loadRuntime = new MusicLoadRuntime();
  const hostedMedia = {
    nextGeneration: vi.fn(() => 1),
    startVisualTransition: vi.fn(() => false),
    finishVisualTransitionAfterPaint: vi.fn(),
    clearStaleVisual: vi.fn(),
    waitForStaleVisualPaint: vi.fn(async () => undefined),
    currentUrls: vi.fn(() => []),
    retainCurrent: vi.fn(async () => undefined),
    syncRetention: vi.fn(),
    unregisterGeneration: vi.fn(async () => undefined),
    releaseAll: vi.fn(),
    destroy: vi.fn(),
  };
  return {
    state,
    loadRuntime,
    hostedMedia,
    destroyYouTube: vi.fn(),
    clearYouTubePlaylist: vi.fn(),
    resetLocalPlayback: vi.fn(async (): Promise<void> => undefined),
    shouldUseVideoElement: () => false,
    prepareWebview: vi.fn(),
    configureWebview: vi.fn(),
    playWebview: vi.fn(async () => undefined),
    playNative: vi.fn(async () => undefined),
    applyNativeSnapshot: vi.fn(),
    loadYouTube: vi.fn(async () => undefined),
    setLoadedPlaybackState: vi.fn(),
    persistPlayback: vi.fn(async () => undefined),
    persistSettings: vi.fn(),
    updateExternalControls: vi.fn(),
    updateTray: vi.fn(),
  };
}

describe("Music source controller", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads a selected local folder into one deterministic queue", async () => {
    vi.mocked(pickMediaFolder).mockResolvedValueOnce({
      folderPath: "/music/soundtracks",
      tracks: [
        { path: "/music/soundtracks/01-intro.flac", title: "Intro", artworkPath: null },
        { path: "/music/soundtracks/02-focus.mp3", title: "Focus", artworkPath: "/music/cover.jpg" },
      ],
      truncated: false,
    });
    const state = createState();
    const context = createContext(state);
    const controller = createMusicSourceController(context);

    await controller.loadFolder();

    expect(state.queue.map((source) => source.identity)).toEqual([
      "local:/music/soundtracks/01-intro.flac",
      "local:/music/soundtracks/02-focus.mp3",
    ]);
    expect(state.currentSource?.identity).toBe("local:/music/soundtracks/01-intro.flac");
    expect(state.folderScanTruncated).toBe(false);
    expect(loadLocalMedia).toHaveBeenCalledWith(expect.objectContaining({
      source: expect.objectContaining({ path: "/music/soundtracks/01-intro.flac" }),
      startMs: 0,
    }));
    expect(context.playNative).toHaveBeenCalledOnce();
  });

  it("restores saved playback when loading a source normally", async () => {
    vi.mocked(getPlaybackState).mockResolvedValueOnce({
      sourceIdentity: "local:/music/focus.flac",
      sourceKind: "local-file",
      positionMs: 42_000,
      durationMs: 180_000,
      status: "paused",
      updatedAt: 1_700_000_000_000,
    });
    const state = createState();
    const context = createContext(state);
    const controller = createMusicSourceController(context);
    const source = localFileSourceFromPath("/music/focus.flac", "Focus");

    await controller.loadSource(source);

    expect(context.setLoadedPlaybackState).toHaveBeenCalledWith(expect.objectContaining({
      positionMs: 42_000,
      status: "paused",
    }));
    expect(loadLocalMedia).toHaveBeenCalledWith(expect.objectContaining({ startMs: 42_000 }));
  });

  it("switches from local playback to YouTube through the shared source owner", async () => {
    const state = createState();
    const context = createContext(state);
    const controller = createMusicSourceController(context);
    const local = localFileSourceFromPath("/music/focus.flac", "Focus");
    const youtube = youtubeVideoSourceFromId("video-2");

    await controller.loadSource(local, { autoplay: false });
    await controller.loadSource(youtube, { autoplay: true });

    expect(state.currentSource).toEqual(youtube);
    expect(state.queue).toEqual([youtube]);
    expect(context.resetLocalPlayback).toHaveBeenCalledTimes(2);
    expect(context.destroyYouTube).toHaveBeenCalledTimes(2);
    expect(context.loadYouTube).toHaveBeenCalledWith(youtube, null, expect.any(Number), true);
  });

  it("prevents a slower local load from replacing a newer YouTube source", async () => {
    const state = createState();
    const context = createContext(state);
    let releaseFirstReset!: () => void;
    context.resetLocalPlayback
      .mockImplementationOnce(() => new Promise<void>((resolve) => {
        releaseFirstReset = resolve;
      }))
      .mockResolvedValue(undefined);
    const controller = createMusicSourceController(context);
    const local = localFileSourceFromPath("/music/first.mp3", "first");
    const youtube = youtubeVideoSourceFromId("video-2");

    const firstLoad = controller.loadSource(local);
    const secondLoad = controller.loadSource(youtube);
    await secondLoad;
    releaseFirstReset();
    await firstLoad;

    expect(state.currentSource?.identity).toBe(youtube.identity);
    expect(context.loadYouTube).toHaveBeenCalledOnce();
  });

  it("keeps the newest local source during rapid local replacement", async () => {
    const state = createState();
    const context = createContext(state);
    let releaseFirstReset!: () => void;
    context.resetLocalPlayback
      .mockImplementationOnce(() => new Promise<void>((resolve) => {
        releaseFirstReset = resolve;
      }))
      .mockResolvedValue(undefined);
    const controller = createMusicSourceController(context);
    const first = localFileSourceFromPath("/music/first.mp3", "first");
    const second = localFileSourceFromPath("/music/second.mp3", "second");

    const firstLoad = controller.loadSource(first);
    const secondLoad = controller.loadSource(second);
    await secondLoad;
    releaseFirstReset();
    await firstLoad;

    expect(state.currentSource?.identity).toBe(second.identity);
  });
});
