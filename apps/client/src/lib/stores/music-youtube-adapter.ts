import { tick } from "svelte";
import { getYouTubeHostUrl } from "$lib/api/music";
import {
  clampVolume,
  initialQueueSelection,
  stableStatusDuringYouTubeBuffering,
  type PersistedPlaybackState,
  type PlaybackSnapshot,
} from "$lib/music/playback";
import {
  isYouTubeSource,
  youtubeVideoSourceFromId,
  type MusicSource,
} from "$lib/music/sources";
import { youtubeErrorMessage } from "$lib/music/youtube-player";
import type { MusicLoadRuntime } from "./music-load-runtime";
import {
  buildYouTubeHostUrl,
  parseYouTubeHostMessage,
  type ResolvingYouTubePlaylist,
  type YouTubeCommandAction,
  type YouTubeHostPlaylistMessage,
  type YouTubeSource,
} from "./music-player-youtube-host";

const OPTIMISTIC_PAUSE_MS = 1_500;
const PLAYLIST_RESOLVE_TIMEOUT_MS = 18_000;

export interface MusicYouTubeState {
  currentSource: MusicSource | null;
  snapshot: PlaybackSnapshot;
  playerError: string | null;
  queue: MusicSource[];
  queueHistory: number[];
  shuffleEnabled: boolean;
  shuffleOrder: number[];
  pendingQueueIndex: number | null;
  youtubeHostUrl: string | null;
  youtubeFrame: HTMLIFrameElement | null;
  youtubeHostToken: string | null;
  youtubeHostReady: boolean;
}

interface MusicYouTubeAdapterContext {
  state: MusicYouTubeState;
  loadRuntime: MusicLoadRuntime;
  effectiveVolume(): number;
  loadSource(source: MusicSource, autoplay: boolean): Promise<void>;
  persist(force?: boolean): Promise<void>;
  updateExternalControls(): void;
  updateTray(): void;
  canPlayNext(): boolean;
  playNext(): Promise<void>;
  getHostUrl?: typeof getYouTubeHostUrl;
  now?(): number;
}

export interface MusicYouTubeAdapter {
  registerFrame(frame: HTMLIFrameElement | null): void;
  load(
    source: YouTubeSource,
    persisted: PersistedPlaybackState | null,
    generation: number,
    autoplay: boolean,
  ): Promise<void>;
  handleMessage(event: MessageEvent<unknown>): void;
  post(payload: Record<string, unknown> & { action: YouTubeCommandAction | "snapshot" }): void;
  beginOptimisticPause(): void;
  clearOptimisticPause(): void;
  destroy(): void;
  clearPlaylistResolution(): void;
}

/** Owns the YouTube host protocol, playlist resolution, and optimistic pause policy. */
export function createMusicYouTubeAdapter(
  context: MusicYouTubeAdapterContext,
): MusicYouTubeAdapter {
  const state = context.state;
  const getHostUrl = context.getHostUrl ?? getYouTubeHostUrl;
  const currentTime = context.now ?? Date.now;
  let hostBaseUrl: string | null = null;
  let hostLoadId: string | null = null;
  let resolvingPlaylist: ResolvingYouTubePlaylist | null = null;
  let playlistTimeoutId: number | null = null;
  let optimisticPauseUntil = 0;
  let handlingEnded = false;

  function registerFrame(frame: HTMLIFrameElement | null): void {
    state.youtubeFrame = frame;
  }

  function post(
    payload: Record<string, unknown> & { action: YouTubeCommandAction | "snapshot" },
  ): void {
    if (!state.youtubeFrame?.contentWindow || !state.youtubeHostToken) return;
    state.youtubeFrame.contentWindow.postMessage({
      type: "ganbaru-ai-youtube-command",
      token: state.youtubeHostToken,
      ...payload,
    }, state.youtubeHostUrl ? new URL(state.youtubeHostUrl).origin : "*");
  }

  function clearPlaylistTimeout(): void {
    if (playlistTimeoutId === null) return;
    if (typeof window !== "undefined") window.clearTimeout(playlistTimeoutId);
    playlistTimeoutId = null;
  }

  function clearPlaylistResolution(): void {
    resolvingPlaylist = null;
    clearPlaylistTimeout();
  }

  function failPlaylistResolution(playlistId: string): void {
    if (!resolvingPlaylist || resolvingPlaylist.playlistId !== playlistId) return;
    clearPlaylistResolution();
    state.playerError = "The YouTube playlist did not return playable videos. Check that it is public and supports embedded playback.";
    state.snapshot = {
      ...state.snapshot,
      status: "error",
      error: state.playerError,
    };
    context.updateTray();
    void context.persist();
  }

  function startPlaylistTimeout(resolving: ResolvingYouTubePlaylist): void {
    clearPlaylistTimeout();
    if (typeof window === "undefined") return;
    playlistTimeoutId = window.setTimeout(() => {
      if (
        resolvingPlaylist?.generation === resolving.generation
        && resolvingPlaylist.playlistId === resolving.playlistId
      ) failPlaylistResolution(resolving.playlistId);
    }, PLAYLIST_RESOLVE_TIMEOUT_MS);
  }

  async function ensureHostFrame(
    generation: number,
    source: YouTubeSource,
    persisted: PersistedPlaybackState | null,
    autoplay: boolean,
  ): Promise<void> {
    if (!hostBaseUrl) hostBaseUrl = await getHostUrl();
    if (!context.loadRuntime.isCurrent(generation)) return;
    const url = buildYouTubeHostUrl({
      baseUrl: hostBaseUrl,
      generation,
      source,
      persisted,
      autoplay,
      volume: context.effectiveVolume(),
      rate: state.snapshot.rate,
    });
    state.youtubeHostUrl = url.toString();
    state.youtubeHostToken = url.searchParams.get("token");
    hostLoadId = url.searchParams.get("load");
    state.youtubeHostReady = false;
    await tick();
  }

  async function load(
    source: YouTubeSource,
    persisted: PersistedPlaybackState | null,
    generation: number,
    autoplay: boolean,
  ): Promise<void> {
    resolvingPlaylist = source.kind === "youtube-playlist"
      ? {
          playlistId: source.playlistId,
          preferredVideoId: source.videoId,
          startMs: source.startMs,
          endMs: source.endMs,
          autoplay,
          generation,
        }
      : null;
    if (resolvingPlaylist) startPlaylistTimeout(resolvingPlaylist);
    else clearPlaylistTimeout();
    try {
      await ensureHostFrame(generation, source, persisted, autoplay);
    } catch (error) {
      if (!context.loadRuntime.isCurrent(generation)) return;
      clearPlaylistResolution();
      state.playerError = error instanceof Error ? error.message : String(error);
      state.snapshot = {
        ...state.snapshot,
        status: "error",
        error: state.playerError,
      };
      context.updateTray();
    }
  }

  function matchesVideo(source: MusicSource, videoId: string | null): boolean {
    if (source.kind === "youtube-video") {
      return videoId ? source.videoId === videoId : true;
    }
    if (source.kind === "youtube-playlist") {
      return source.videoId !== null && (videoId ? source.videoId === videoId : true);
    }
    return false;
  }

  function applyMetadataTitle(videoId: string | null, title: string | null): void {
    const resolvedTitle = title?.trim();
    if (!resolvedTitle) return;
    if (
      state.currentSource
      && matchesVideo(state.currentSource, videoId)
      && state.currentSource.title !== resolvedTitle
    ) state.currentSource = { ...state.currentSource, title: resolvedTitle };
    let changed = false;
    const queue = state.queue.map((source) => {
      if (!matchesVideo(source, videoId) || source.title === resolvedTitle) return source;
      changed = true;
      return { ...source, title: resolvedTitle };
    });
    if (changed) state.queue = queue;
  }

  async function applyPlaylist(message: YouTubeHostPlaylistMessage): Promise<void> {
    const resolving = resolvingPlaylist;
    if (
      !resolving
      || !context.loadRuntime.isCurrent(resolving.generation)
      || resolving.playlistId !== message.playlistId
      || message.videoIds.length === 0
    ) return;
    const preferredIndex = resolving.preferredVideoId
      ? message.videoIds.indexOf(resolving.preferredVideoId)
      : -1;
    const selection = preferredIndex >= 0
      ? { index: preferredIndex, remainingOrder: [] }
      : initialQueueSelection(message.videoIds.length, state.shuffleEnabled);
    const selectedIndex = selection.index ?? 0;
    const queue = message.videoIds.map((videoId, index) =>
      youtubeVideoSourceFromId(videoId, {
        startMs: index === selectedIndex ? resolving.startMs : null,
        endMs: index === selectedIndex ? resolving.endMs : null,
      }));
    state.queue = queue;
    state.queueHistory = [];
    state.shuffleOrder = selection.remainingOrder.filter(
      (index) => index !== selectedIndex,
    );
    state.pendingQueueIndex = selectedIndex;
    clearPlaylistResolution();
    await context.loadSource(
      queue[selectedIndex] ?? queue[0],
      resolving.autoplay,
    );
  }

  async function advanceEndedTrack(): Promise<void> {
    if (handlingEnded || !context.canPlayNext()) return;
    handlingEnded = true;
    try {
      await context.persist(true);
      await context.playNext();
    } finally {
      handlingEnded = false;
    }
  }

  function handleMessage(event: MessageEvent<unknown>): void {
    if (!state.youtubeFrame?.contentWindow || event.source !== state.youtubeFrame.contentWindow) return;
    const message = parseYouTubeHostMessage(event.data);
    if (
      !message
      || message.token !== state.youtubeHostToken
      || message.load !== hostLoadId
    ) return;
    if (message.type === "ganbaru-ai-youtube-ready") {
      state.youtubeHostReady = true;
      return;
    }
    if (message.type === "ganbaru-ai-youtube-error") {
      clearPlaylistResolution();
      state.playerError = youtubeErrorMessage(message.code);
      state.snapshot = {
        ...state.snapshot,
        status: "error",
        error: state.playerError,
      };
      void context.persist();
      context.updateTray();
      return;
    }
    if (message.type === "ganbaru-ai-youtube-playlist") {
      void applyPlaylist(message);
      return;
    }
    if (message.type === "ganbaru-ai-youtube-playlist-error") {
      failPlaylistResolution(message.playlistId);
      return;
    }
    if (state.currentSource?.kind === "youtube-playlist" && !resolvingPlaylist) return;
    applyMetadataTitle(message.videoId, message.title);
    if (message.status === "playing" && currentTime() < optimisticPauseUntil) return;
    if (message.status !== "playing") optimisticPauseUntil = 0;
    const wasEnded = state.snapshot.status === "ended";
    const status = stableStatusDuringYouTubeBuffering(
      state.snapshot.status,
      message.status,
    );
    state.snapshot = {
      ...state.snapshot,
      status,
      positionMs: message.positionMs,
      durationMs: message.durationMs,
      error: null,
    };
    context.updateExternalControls();
    void context.persist();
    context.updateTray();
    if (status === "ended" && !wasEnded) void advanceEndedTrack();
  }

  function beginOptimisticPause(): void {
    optimisticPauseUntil = currentTime() + OPTIMISTIC_PAUSE_MS;
  }

  function clearOptimisticPause(): void {
    optimisticPauseUntil = 0;
  }

  function destroy(): void {
    clearPlaylistResolution();
    if (state.currentSource && isYouTubeSource(state.currentSource)) {
      post({ action: "stop" });
    }
  }

  return {
    registerFrame,
    load,
    handleMessage,
    post,
    beginOptimisticPause,
    clearOptimisticPause,
    destroy,
    clearPlaylistResolution,
  };
}
