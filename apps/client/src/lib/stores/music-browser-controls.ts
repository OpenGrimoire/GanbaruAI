import { musicHardwareActionFromKey } from "$lib/music/hardware-controls";
import type { PlaybackSnapshot } from "$lib/music/playback";
import type {
  MusicExternalControlsContext,
} from "$lib/stores/music-external-controls-contracts";

export interface MusicBrowserControls {
  init(): void;
  destroy(): void;
  update(): void;
}

/** Owns WebView and browser media controls shared by desktop and mobile. */
export function createMusicBrowserControls(
  context: MusicExternalControlsContext,
): MusicBrowserControls {
  let initialized = false;

  const handleHardwareKeydown = (event: KeyboardEvent): void => {
    const action = musicHardwareActionFromKey(event);
    if (!action) return;
    event.preventDefault();
    event.stopPropagation();
    if (action === "play") void context.play();
    else if (action === "pause") void context.pause();
    else if (action === "playPause") void context.togglePlay();
    else if (action === "stop") void context.stop();
    else if (action === "previousTrack") void context.previous();
    else if (action === "nextTrack") void context.next();
  };

  function init(): void {
    if (initialized) return;
    initialized = true;
    if (typeof window !== "undefined") {
      window.addEventListener("message", context.handleWindowMessage);
      window.addEventListener("keydown", handleHardwareKeydown, { capture: true });
    }
    update();
  }

  function destroy(): void {
    if (typeof window !== "undefined") {
      window.removeEventListener("message", context.handleWindowMessage);
      window.removeEventListener("keydown", handleHardwareKeydown, { capture: true });
    }
    initialized = false;
    if (typeof navigator !== "undefined" && "mediaSession" in navigator) {
      navigator.mediaSession.metadata = null;
      navigator.mediaSession.playbackState = "none";
      for (const action of ["play", "pause", "previoustrack", "nexttrack", "stop"] as const) {
        navigator.mediaSession.setActionHandler(action, null);
      }
    }
  }

  function updatePosition(snapshot: PlaybackSnapshot): void {
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
        position: Math.max(0, Math.min(snapshot.positionMs / 1_000, durationSeconds)),
      });
    } catch {
      // Invalid position state must not break normal playback controls.
    }
  }

  function update(): void {
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
    updatePosition(snapshot);
    navigator.mediaSession.setActionHandler("play", () => { void context.play(); });
    navigator.mediaSession.setActionHandler("pause", () => { void context.pause(); });
    navigator.mediaSession.setActionHandler(
      "previoustrack",
      context.canPrevious() ? () => { void context.previous(); } : null,
    );
    navigator.mediaSession.setActionHandler(
      "nexttrack",
      context.canNext() ? () => { void context.next(); } : null,
    );
    navigator.mediaSession.setActionHandler("stop", () => { void context.stop(); });
  }

  return { init, destroy, update };
}
