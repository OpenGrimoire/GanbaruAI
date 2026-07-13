import { tick } from "svelte";
import {
  loadLocalMedia,
  mediaPlayerErrorMessage,
  setLocalMuted,
  type LocalBackendKind,
  type LocalPlayerSnapshot,
} from "$lib/api/media-player";
import {
  getPlaybackState,
  pickMediaFolder,
  registerEmbeddedArtwork,
  registerMediaFile,
} from "$lib/api/music";
import {
  clampVolume,
  DEFAULT_PLAYBACK_SNAPSHOT,
  initialQueueSelection,
  normalizeLocalPlayableStartMs,
  type PersistedPlaybackState,
  type PlaybackSnapshot,
} from "$lib/music/playback";
import {
  isYouTubeSource,
  localFileSourceFromPath,
  parseMusicSourceInput,
  type LocalFileSource,
  type MusicSource,
} from "$lib/music/sources";
import type { MusicHostedMediaController } from "./music-hosted-media-controller";
import type { MusicLoadRuntime } from "./music-load-runtime";
import type { YouTubeSource } from "./music-player-youtube-host";

export interface LoadSourceOptions {
  autoplay?: boolean;
  resume?: boolean;
  preserveQueue?: boolean;
}

export interface MusicSourceState {
  sourceInput: string;
  currentSource: MusicSource | null;
  parseError: string | null;
  playerError: string | null;
  snapshot: PlaybackSnapshot;
  queue: MusicSource[];
  folderScanTruncated: boolean;
  shuffleEnabled: boolean;
  shuffleOrder: number[];
  queueHistory: number[];
  pendingQueueIndex: number | null;
  sourceActionBusy: boolean;
  muted: boolean;
  localMediaSrc: string | null;
  localHasVideo: boolean;
  localBackendKind: LocalBackendKind;
  localVideoReady: boolean;
  currentArtworkUrl: string | null;
}

interface MusicSourceControllerContext {
  state: MusicSourceState;
  loadRuntime: MusicLoadRuntime;
  hostedMedia: MusicHostedMediaController;
  destroyYouTube(): void;
  clearYouTubePlaylist(): void;
  resetLocalPlayback(): Promise<void>;
  shouldUseVideoElement(snapshot: LocalPlayerSnapshot): boolean;
  prepareWebview(resumeMs: number, playableStartMs: number): void;
  configureWebview(): void;
  playWebview(): Promise<void>;
  playNative(): Promise<void>;
  applyNativeSnapshot(snapshot: LocalPlayerSnapshot): void;
  loadYouTube(
    source: YouTubeSource,
    persisted: PersistedPlaybackState | null,
    generation: number,
    autoplay: boolean,
  ): Promise<void>;
  setLoadedPlaybackState(state: PersistedPlaybackState | null): void;
  persistPlayback(): Promise<void>;
  persistSettings(): void;
  updateExternalControls(): void;
  updateTray(): void;
}

export interface MusicSourceController {
  loadFromInput(): Promise<void>;
  loadFolder(): Promise<void>;
  loadSource(source: MusicSource, options?: LoadSourceOptions): Promise<void>;
  resetPlayer(): Promise<void>;
}

/** Owns source parsing, folder construction, replacement, and backend selection. */
export function createMusicSourceController(
  context: MusicSourceControllerContext,
): MusicSourceController {
  const state = context.state;

  async function loadFromInput(): Promise<void> {
    const result = parseMusicSourceInput(state.sourceInput);
    state.parseError = result.error;
    state.playerError = null;
    if (!result.source) return;
    state.sourceActionBusy = true;
    try {
      await loadSource(result.source, { autoplay: true });
    } finally {
      state.sourceActionBusy = false;
    }
  }

  async function loadFolder(): Promise<void> {
    state.parseError = null;
    state.playerError = null;
    state.sourceActionBusy = true;
    try {
      const selection = await pickMediaFolder();
      if (!selection) return;
      state.folderScanTruncated = selection.truncated;
      state.queue = selection.tracks.map((track) =>
        localFileSourceFromPath(track.path, track.title, track.artworkPath));
      state.shuffleOrder = [];
      state.queueHistory = [];
      state.pendingQueueIndex = null;
      if (state.queue.length === 0) {
        state.playerError = "No supported audio or video files were found in that folder.";
        context.updateTray();
        return;
      }
      const queueSelection = initialQueueSelection(
        state.queue.length,
        state.shuffleEnabled,
      );
      const initialIndex = queueSelection.index ?? 0;
      state.shuffleOrder = queueSelection.remainingOrder;
      state.pendingQueueIndex = initialIndex;
      await loadSource(state.queue[initialIndex] ?? state.queue[0], {
        autoplay: true,
        resume: false,
        preserveQueue: true,
      });
    } catch (error) {
      state.playerError = error instanceof Error ? error.message : String(error);
      context.updateTray();
    } finally {
      state.sourceActionBusy = false;
    }
  }

  async function loadLocalSource(
    source: LocalFileSource,
    persisted: PersistedPlaybackState | null,
    generation: number,
    hostedGeneration: number,
    options: LoadSourceOptions,
  ): Promise<void> {
    const registeredUrls: string[] = [];
    try {
      const localSnapshot = await loadLocalMedia({
        source: {
          kind: "local-file",
          path: source.path,
          identity: source.identity,
          title: source.title,
        },
        startMs: persisted?.positionMs ?? source.startMs ?? 0,
        volume: clampVolume(state.snapshot.volume),
        rate: state.snapshot.rate,
      });
      if (!context.loadRuntime.isCurrent(generation)) return;
      await setLocalMuted(state.muted);
      if (!context.loadRuntime.isCurrent(generation)) return;
      const useVideoElement = context.shouldUseVideoElement(localSnapshot);
      const mediaUrl = useVideoElement
        ? await registerMediaFile(source.path, hostedGeneration)
        : null;
      if (mediaUrl) registeredUrls.push(mediaUrl);
      if (!context.loadRuntime.isCurrent(generation)) {
        await context.hostedMedia.unregisterGeneration(registeredUrls, hostedGeneration);
        return;
      }
      let artworkUrl = source.artworkPath
        ? await registerMediaFile(source.artworkPath, hostedGeneration).catch(() => null)
        : null;
      if (artworkUrl) registeredUrls.push(artworkUrl);
      if (!artworkUrl) {
        artworkUrl = await registerEmbeddedArtwork(source.path, hostedGeneration)
          .catch(() => null);
        if (artworkUrl) registeredUrls.push(artworkUrl);
      }
      if (!context.loadRuntime.isCurrent(generation)) {
        await context.hostedMedia.unregisterGeneration(registeredUrls, hostedGeneration);
        return;
      }
      context.prepareWebview(
        persisted?.positionMs ?? source.startMs ?? 0,
        useVideoElement
          ? normalizeLocalPlayableStartMs(localSnapshot.playableStartMs)
          : 0,
      );
      if (useVideoElement) context.hostedMedia.clearStaleVisual();
      state.localHasVideo = localSnapshot.hasVideo;
      state.localBackendKind = localSnapshot.backendKind;
      state.localVideoReady = !useVideoElement;
      state.localMediaSrc = mediaUrl;
      state.currentArtworkUrl = artworkUrl;
      await context.hostedMedia.retainCurrent(hostedGeneration);
      if (!context.loadRuntime.isCurrent(generation)) {
        await context.hostedMedia.unregisterGeneration(registeredUrls, hostedGeneration);
        return;
      }
      context.applyNativeSnapshot(localSnapshot);
      await tick();
      if (!context.loadRuntime.isCurrent(generation)) return;
      if (options.autoplay) {
        if (useVideoElement) {
          context.configureWebview();
          await context.playWebview();
        } else {
          await context.playNative();
        }
      } else if (useVideoElement) {
        context.configureWebview();
      }
      context.updateExternalControls();
      await context.persistPlayback();
      context.updateTray();
    } catch (error) {
      await context.hostedMedia.unregisterGeneration(registeredUrls, hostedGeneration);
      if (!context.loadRuntime.isCurrent(generation)) return;
      await context.hostedMedia.retainCurrent(hostedGeneration).catch(() => null);
      state.playerError = mediaPlayerErrorMessage(error);
      state.snapshot = {
        ...state.snapshot,
        status: "error",
        error: state.playerError,
      };
      context.updateTray();
    }
  }

  async function loadSource(
    source: MusicSource,
    options: LoadSourceOptions = {},
  ): Promise<void> {
    const generation = context.loadRuntime.begin();
    const transitionStarted = source.kind === "local-file"
      ? context.hostedMedia.startVisualTransition()
      : false;
    if (source.kind !== "local-file") context.hostedMedia.clearStaleVisual();
    if (transitionStarted) {
      await context.hostedMedia.waitForStaleVisualPaint();
      if (!context.loadRuntime.isCurrent(generation)) return;
    }
    const hostedGeneration = context.hostedMedia.nextGeneration();
    const nextVolume = clampVolume(state.snapshot.volume);
    context.destroyYouTube();
    await context.resetLocalPlayback();
    if (!context.loadRuntime.isCurrent(generation)) return;
    state.currentArtworkUrl = null;
    if (source.kind !== "local-file") {
      await context.hostedMedia.retainCurrent(hostedGeneration).catch(() => null);
      if (!context.loadRuntime.isCurrent(generation)) return;
    }
    if (!options.preserveQueue) {
      state.queue = [source];
      state.shuffleOrder = [];
      state.queueHistory = [];
      state.pendingQueueIndex = 0;
    }
    state.currentSource = source;
    state.pendingQueueIndex = null;
    state.sourceInput = "";
    state.parseError = null;
    state.playerError = null;
    if (!isYouTubeSource(source)) context.clearYouTubePlaylist();
    state.snapshot = {
      ...DEFAULT_PLAYBACK_SNAPSHOT,
      status: "loading",
      volume: nextVolume,
      rate: state.snapshot.rate,
    };
    context.persistSettings();
    context.updateTray();
    const persisted = options.resume === false
      ? null
      : await getPlaybackState(source.identity).catch(() => null);
    if (!context.loadRuntime.isCurrent(generation)) return;
    context.setLoadedPlaybackState(persisted);
    if (source.kind === "local-file") {
      await loadLocalSource(
        source,
        persisted,
        generation,
        hostedGeneration,
        options,
      );
    } else {
      await context.loadYouTube(
        source,
        persisted,
        generation,
        options.autoplay === true,
      );
    }
  }

  async function resetPlayer(): Promise<void> {
    const generation = context.loadRuntime.begin();
    context.hostedMedia.clearStaleVisual();
    const hostedGeneration = context.hostedMedia.nextGeneration();
    context.destroyYouTube();
    await context.resetLocalPlayback();
    if (!context.loadRuntime.isCurrent(generation)) return;
    state.currentSource = null;
    state.parseError = null;
    state.playerError = null;
    context.setLoadedPlaybackState(null);
    state.snapshot = {
      ...DEFAULT_PLAYBACK_SNAPSHOT,
      volume: state.snapshot.volume,
      rate: state.snapshot.rate,
    };
    state.sourceInput = "";
    context.clearYouTubePlaylist();
    state.queue = [];
    state.folderScanTruncated = false;
    state.shuffleOrder = [];
    state.queueHistory = [];
    state.pendingQueueIndex = null;
    state.currentArtworkUrl = null;
    await context.hostedMedia.retainCurrent(hostedGeneration).catch(() => null);
    context.updateExternalControls();
    context.updateTray();
  }

  return { loadFromInput, loadFolder, loadSource, resetPlayer };
}
