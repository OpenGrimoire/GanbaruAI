import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { updateMediaControls } from "$lib/api/media-controls";
import { createMusicBrowserControls } from "$lib/stores/music-browser-controls";
import {
  parseMusicHardwareControlPayload,
  type MusicHardwareControlPayload,
} from "$lib/music/hardware-controls";
import type {
  MusicExternalControls,
  MusicExternalControlsContext,
} from "$lib/stores/music-external-controls-contracts";

interface DesktopMusicExternalControlsContext extends MusicExternalControlsContext {
  listen?: typeof listen;
}

/** Owns app-level media listeners, Media Session handlers, and native controls. */
export function createMusicExternalControls(
  context: DesktopMusicExternalControlsContext,
): MusicExternalControls {
  const listenToEvent = context.listen ?? listen;
  const browser = createMusicBrowserControls(context);
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
    trackListener("tray-music-inspect-assignment", () => { context.inspectAssignment(); });
    trackListener("music-hardware-control", (event) => {
      const payload = parseMusicHardwareControlPayload(event.payload);
      if (payload) void handleHardwareControl(payload);
    });
  }

  function init(): void {
    if (initialized) return;
    initialized = true;
    browser.init();
    installTrayListeners();
    update();
  }

  function destroy(): void {
    browser.destroy();
    initialized = false;
    for (const unlisten of unlisteners.splice(0)) unlisten();
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
    browser.update();
    updateNativeMediaControls();
  }

  return {
    init,
    destroy,
    isInitialized: () => initialized,
    update,
    updateBrowser: browser.update,
    updateNative: updateNativeMediaControls,
  };
}
