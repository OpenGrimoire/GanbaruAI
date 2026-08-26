import {
  getLocalSnapshot,
  mediaPlayerErrorMessage,
  pauseLocalMedia,
  playLocalMedia,
  seekLocalMedia,
  setLocalMuted,
  setLocalRate,
  setLocalVolume,
  stopLocalMedia,
  type LocalBackendKind,
  type LocalPlayerSnapshot,
} from "$lib/api/media-player";
import type { PlaybackSnapshot } from "$lib/music/playback";
import type { MusicSource } from "$lib/music/sources";
import type { SchedulerRunContext } from "$lib/scheduling/lifecycle-scheduler";

export interface MusicNativeLocalState {
  currentSource: MusicSource | null;
  snapshot: PlaybackSnapshot;
  playerError: string | null;
  muted: boolean;
  localBackendKind: LocalBackendKind;
}

interface MusicNativeLocalAdapterContext {
  state: MusicNativeLocalState;
  updateExternalControls(): void;
  updateTray(): void;
  persist(force?: boolean): Promise<void>;
  playNext(): Promise<void>;
  handlePosition(positionMs: number): void;
}

export interface MusicNativeLocalAdapter {
  isActive(): boolean;
  applySnapshot(snapshot: LocalPlayerSnapshot): void;
  play(): Promise<void>;
  pause(): Promise<void>;
  stop(): Promise<void>;
  seek(positionMs: number): Promise<void>;
  applyVolume(clearMute: boolean): Promise<void>;
  applyMute(): Promise<void>;
  applyRate(rate: number): Promise<void>;
  refresh(context?: SchedulerRunContext): Promise<void>;
  reset(): Promise<void>;
}

/** Owns Rust-backed local audio commands, recovery, and ended-track handling. */
export function createMusicNativeLocalAdapter(
  context: MusicNativeLocalAdapterContext,
): MusicNativeLocalAdapter {
  const state = context.state;
  let handlingEnded = false;

  function isActive(): boolean {
    return state.currentSource?.kind === "local-file"
      && (state.localBackendKind === "rodio" || state.localBackendKind === "media3");
  }

  function applySnapshot(localSnapshot: LocalPlayerSnapshot): void {
    state.snapshot = {
      status: localSnapshot.status,
      positionMs: localSnapshot.positionMs,
      durationMs: localSnapshot.durationMs,
      volume: state.snapshot.volume,
      rate: state.snapshot.rate,
      error: localSnapshot.error,
    };
    state.playerError = localSnapshot.error;
    context.handlePosition(localSnapshot.positionMs);
  }

  function applyError(error: unknown): void {
    state.playerError = mediaPlayerErrorMessage(error);
    state.snapshot = {
      ...state.snapshot,
      status: "error",
      error: state.playerError,
    };
  }

  async function play(): Promise<void> {
    try {
      applySnapshot(await playLocalMedia());
      state.playerError = null;
      context.updateExternalControls();
    } catch (error) {
      applyError(error);
    }
    context.updateTray();
  }

  async function pause(): Promise<void> {
    state.snapshot = { ...state.snapshot, status: "paused" };
    context.updateExternalControls();
    try {
      applySnapshot(await pauseLocalMedia());
      context.updateExternalControls();
    } catch (error) {
      applyError(error);
    }
  }

  async function stop(): Promise<void> {
    try {
      applySnapshot(await stopLocalMedia());
    } catch {
      // Stopping is best effort. Local UI state still resets.
    }
    state.snapshot = { ...state.snapshot, status: "idle", positionMs: 0 };
    context.updateExternalControls();
  }

  async function seek(positionMs: number): Promise<void> {
    try {
      applySnapshot(await seekLocalMedia(positionMs));
    } catch (error) {
      applyError(error);
    }
  }

  async function applyVolume(clearMute: boolean): Promise<void> {
    try {
      applySnapshot(await setLocalVolume(state.snapshot.volume));
      if (clearMute) applySnapshot(await setLocalMuted(false));
    } catch (error) {
      applyError(error);
    }
  }

  async function applyMute(): Promise<void> {
    try {
      applySnapshot(await setLocalMuted(state.muted));
    } catch (error) {
      applyError(error);
    }
  }

  async function applyRate(rate: number): Promise<void> {
    try {
      applySnapshot(await setLocalRate(rate));
    } catch (error) {
      applyError(error);
    }
  }

  async function refresh(schedulerContext?: SchedulerRunContext): Promise<void> {
    if (handlingEnded) return;
    const wasEnded = state.snapshot.status === "ended";
    try {
      const localSnapshot = await getLocalSnapshot();
      if (!isActive() || (schedulerContext && !schedulerContext.isCurrent())) return;
      applySnapshot(localSnapshot);
      if (localSnapshot.status === "ended" && !wasEnded) {
        handlingEnded = true;
        context.updateExternalControls();
        await context.persist(true);
        context.updateTray();
        try {
          await context.playNext();
        } finally {
          handlingEnded = false;
        }
        return;
      }
      context.updateExternalControls();
      void context.persist();
      context.updateTray();
    } catch {
      handlingEnded = false;
    }
  }

  async function reset(): Promise<void> {
    await stopLocalMedia().catch(() => null);
  }

  return {
    isActive,
    applySnapshot,
    play,
    pause,
    stop,
    seek,
    applyVolume,
    applyMute,
    applyRate,
    refresh,
    reset,
  };
}
