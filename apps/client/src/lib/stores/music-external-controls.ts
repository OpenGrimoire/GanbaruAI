import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { updateMediaControls } from "$lib/api/media-controls";
import {
  musicHardwareActionFromKey,
  parseMusicHardwareControlPayload,
  type MusicHardwareControlPayload,
} from "$lib/music/hardware-controls";
import type { MusicSource } from "$lib/music/sources";
import type { PlaybackSnapshot } from "$lib/music/playback";
import { getNavigation } from "$lib/stores/navigation.svelte";

interface MusicExternalControlsContext {
  currentSource(): MusicSource | null;
  snapshot(): PlaybackSnapshot;
  title(): string;
  sourceKindLabel(): string;
  artworkUrl(): string | null;
  isBusy(): boolean;
  canPrevious(): boolean;
  canNext(): boolean;
  volume(): number;
  muted(): boolean;
  shuffleEnabled(): boolean;
  play(): Promise<void>;
  pause(): Promise<void>;
  togglePlay(): Promise<void>;
  stop(): Promise<void>;
  previous(): Promise<void>;
  next(): Promise<void>;
  seekBy(deltaMs: number): Promise<void>;
  seekTo(positionMs: number): Promise<void>;
  setVolume(volume: number): Promise<void>;
  setRate(rate: number): Promise<void>;
  toggleShuffle(): void;
  handleWindowMessage(event: MessageEvent<unknown>): void;
  listen?: typeof listen;
}

export interface MusicExternalControls {
  init(): void;
  destroy(): void;
  isInitialized(): boolean;
  update(): void;
  updateBrowser(): void;
  updateNative(): void;
}

/** Owns app-level media listeners, Media Session handlers, and native controls. */
export function createMusicExternalControls(
  context: MusicExternalControlsContext,
): MusicExternalControls {
  const listenToEvent = context.listen ?? listen;
  const unlisteners: UnlistenFn[] = [];
  let initialized = false;

  async function handleHardwareControl(
    payload: MusicHardwareControlPayload,
  ): Promise<void> {
    switch (payload.action) {
      case "play":
        await context.play();
        return;
      case "pause":
        await context.pause();
        return;
      case "playPause":
        await context.togglePlay();
        return;
      case "stop":
        await context.stop();
        return;
      case "previousTrack":
        await context.previous();
        return;
      case "nextTrack":
        await context.next();
        return;
      case "seekBy":
        if (payload.deltaMs !== undefined) await context.seekBy(payload.deltaMs);
        return;
      case "seekTo":
        if (payload.positionMs !== undefined) await context.seekTo(payload.positionMs);
        return;
      case "setVolume":
        if (payload.volume !== undefined) await context.setVolume(payload.volume);
        return;
      case "setRate":
        if (payload.rate !== undefined) await context.setRate(payload.rate);
        return;
      case "setShuffle":
        if (
          payload.shuffleEnabled !== undefined
          && payload.shuffleEnabled !== context.shuffleEnabled()
        ) context.toggleShuffle();
    }
  }

  const handleHardwareKeydown = (event: KeyboardEvent): void => {
    const action = musicHardwareActionFromKey(event);
    if (!action) return;
    event.preventDefault();
    event.stopPropagation();
    void handleHardwareControl({ action });
  };

  function trackListener(
    eventName: string,
    handler: Parameters<typeof listen>[1],
  ): void {
    listenToEvent(eventName, handler).then((unlisten) => {
      if (initialized) unlisteners.push(unlisten);
      else unlisten();
    }).catch((error: unknown) => {
      console.warn(`Failed to listen for ${eventName}:`, error);
    });
  }

  function installTrayListeners(): void {
    trackListener("tray-music-play-pause", () => { void context.togglePlay(); });
    trackListener("tray-music-previous", () => { void context.previous(); });
    trackListener("tray-music-next", () => { void context.next(); });
    trackListener("tray-music-open", () => { getNavigation().navigate("music"); });
    trackListener("music-hardware-control", (event) => {
      const payload = parseMusicHardwareControlPayload(event.payload);
      if (payload) void handleHardwareControl(payload);
    });
  }

  function init(): void {
    if (initialized) return;
    initialized = true;
    if (typeof window !== "undefined") {
      window.addEventListener("message", context.handleWindowMessage);
      window.addEventListener("keydown", handleHardwareKeydown, { capture: true });
    }
    installTrayListeners();
    update();
  }

  function destroy(): void {
    if (typeof window !== "undefined") {
      window.removeEventListener("message", context.handleWindowMessage);
      window.removeEventListener("keydown", handleHardwareKeydown, { capture: true });
    }
    initialized = false;
    for (const unlisten of unlisteners.splice(0)) unlisten();
    if (typeof navigator !== "undefined" && "mediaSession" in navigator) {
      navigator.mediaSession.metadata = null;
      navigator.mediaSession.playbackState = "none";
      for (const action of ["play", "pause", "previoustrack", "nexttrack", "stop"] as const) {
        navigator.mediaSession.setActionHandler(action, null);
      }
    }
  }

  function updateBrowserPositionState(snapshot: PlaybackSnapshot): void {
    if (typeof navigator === "undefined" || !("mediaSession" in navigator)) return;
    if (!("setPositionState" in navigator.mediaSession)) return;
    const durationSeconds = snapshot.durationMs === null
      ? null
      : snapshot.durationMs / 1_000;
    if (durationSeconds === null || durationSeconds <= 0) return;
    try {
      navigator.mediaSession.setPositionState({
        duration: durationSeconds,
        playbackRate: snapshot.rate,
        position: Math.max(
          0,
          Math.min(snapshot.positionMs / 1_000, durationSeconds),
        ),
      });
    } catch {
      // Invalid position state must not break normal playback controls.
    }
  }

  function updateBrowserMediaSession(): void {
    if (typeof navigator === "undefined" || !("mediaSession" in navigator)) return;
    const source = context.currentSource();
    const snapshot = context.snapshot();
    if (!source) {
      navigator.mediaSession.playbackState = "none";
      return;
    }
    if (typeof MediaMetadata !== "undefined") {
      const artworkUrl = context.artworkUrl();
      navigator.mediaSession.metadata = new MediaMetadata({
        title: context.title(),
        artist: context.sourceKindLabel(),
        artwork: artworkUrl ? [{ src: artworkUrl }] : [],
      });
    }
    navigator.mediaSession.playbackState = snapshot.status === "playing"
      ? "playing"
      : snapshot.status === "paused"
        ? "paused"
        : "none";
    updateBrowserPositionState(snapshot);
    navigator.mediaSession.setActionHandler("play", () => { void context.play(); });
    navigator.mediaSession.setActionHandler("pause", () => { void context.pause(); });
    navigator.mediaSession.setActionHandler("previoustrack", () => { void context.previous(); });
    navigator.mediaSession.setActionHandler("nexttrack", () => { void context.next(); });
    navigator.mediaSession.setActionHandler("stop", () => { void context.stop(); });
  }

  function updateNativeMediaControls(): void {
    const source = context.currentSource();
    const snapshot = context.snapshot();
    void updateMediaControls({
      status: snapshot.status,
      title: source ? context.title() : null,
      sourceKindLabel: source ? context.sourceKindLabel() : null,
      artworkUrl: context.artworkUrl(),
      canPlayPause: Boolean(source) && !context.isBusy(),
      canPrevious: context.canPrevious(),
      canNext: context.canNext(),
      canSeek: Boolean(source) && (snapshot.durationMs ?? 0) > 0,
      positionMs: Math.max(0, Math.round(snapshot.positionMs)),
      durationMs: snapshot.durationMs === null
        ? null
        : Math.max(0, Math.round(snapshot.durationMs)),
      volume: context.volume(),
      muted: context.muted(),
      rate: snapshot.rate,
      shuffleEnabled: context.shuffleEnabled(),
    }).catch(() => null);
  }

  function update(): void {
    updateBrowserMediaSession();
    updateNativeMediaControls();
  }

  return {
    init,
    destroy,
    isInitialized: () => initialized,
    update,
    updateBrowser: updateBrowserMediaSession,
    updateNative: updateNativeMediaControls,
  };
}
