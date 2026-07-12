import { savePlaybackState } from "$lib/api/music";
import {
  shouldPersistPlaybackState,
  type PersistedPlaybackState,
  type PlaybackSnapshot,
} from "$lib/music/playback";
import { isYouTubeSource, type MusicSource } from "$lib/music/sources";
import {
  createLifecycleScheduler,
  type SchedulerRunContext,
} from "$lib/scheduling/lifecycle-scheduler";
import { createMusicPlaybackWriter } from "./music-playback-writer";
import {
  activeMusicSnapshotBackend,
  type MusicSnapshotBackend,
} from "./music-snapshot-policy";
import type { MusicLoadRuntime } from "./music-load-runtime";

const YOUTUBE_PLAYING_SNAPSHOT_MS = 1_000;
const LOCAL_PLAYING_SNAPSHOT_MS = 500;
const PAUSED_SNAPSHOT_MS = 5_000;

interface MusicPlaybackRuntimeContext {
  loadRuntime: MusicLoadRuntime;
  currentSource(): MusicSource | null;
  snapshot(): PlaybackSnapshot;
  isInitialized(): boolean;
  usesNativeLocalBackend(): boolean;
  postYouTubeSnapshot(): void;
  refreshNativeLocalSnapshot(context: SchedulerRunContext): Promise<void>;
  saveState?: typeof savePlaybackState;
  now?(): number;
}

export interface MusicPlaybackRuntime {
  setLoadedState(state: PersistedPlaybackState | null): void;
  persist(force?: boolean): Promise<void>;
  syncScheduler(): void;
  resumeScheduler(): void;
  stopScheduler(): void;
}

/** Owns playback persistence generation and backend-specific snapshot cadence. */
export function createMusicPlaybackRuntime(
  context: MusicPlaybackRuntimeContext,
): MusicPlaybackRuntime {
  const writer = createMusicPlaybackWriter(context.saveState ?? savePlaybackState);
  const currentTime = context.now ?? Date.now;
  let lastPersisted: PersistedPlaybackState | null = null;
  let snapshotBackend: MusicSnapshotBackend | null = null;

  const scheduler = createLifecycleScheduler({
    run: runSnapshotRefresh,
    errorRetryMs: 15_000,
    onError: (error) => {
      console.warn("Failed to refresh music playback snapshot:", error);
    },
  });

  function setLoadedState(state: PersistedPlaybackState | null): void {
    lastPersisted = state;
  }

  async function persist(force = false): Promise<void> {
    const source = context.currentSource();
    if (!source) return;
    const snapshot = context.snapshot();
    const next: PersistedPlaybackState = {
      sourceIdentity: source.identity,
      sourceKind: source.kind,
      positionMs: Math.max(0, Math.round(snapshot.positionMs)),
      durationMs: snapshot.durationMs === null
        ? null
        : Math.max(0, Math.round(snapshot.durationMs)),
      status: snapshot.status,
      updatedAt: currentTime(),
    };
    if (!force && !shouldPersistPlaybackState({
      ...next,
      nowMs: next.updatedAt,
    }, lastPersisted)) return;
    const generation = context.loadRuntime.current();
    const saved = await writer.save(
      next,
      generation,
      (candidateGeneration, sourceIdentity) =>
        context.loadRuntime.isCurrent(candidateGeneration)
        && sourceIdentity === context.currentSource()?.identity,
    );
    if (saved) lastPersisted = next;
  }

  async function runSnapshotRefresh(
    schedulerContext: SchedulerRunContext,
  ): Promise<number | null> {
    const source = context.currentSource();
    const snapshot = context.snapshot();
    if (!source || (snapshot.status !== "playing" && snapshot.status !== "paused")) {
      return null;
    }
    if (isYouTubeSource(source)) {
      context.postYouTubeSnapshot();
      await persist();
    } else if (context.usesNativeLocalBackend()) {
      await context.refreshNativeLocalSnapshot(schedulerContext);
    } else {
      return null;
    }
    if (!schedulerContext.isCurrent()) return null;
    const currentSnapshot = context.snapshot();
    const cadenceMs = currentSnapshot.status === "paused"
      ? PAUSED_SNAPSHOT_MS
      : isYouTubeSource(source)
        ? YOUTUBE_PLAYING_SNAPSHOT_MS
        : LOCAL_PLAYING_SNAPSHOT_MS;
    return schedulerContext.now() + cadenceMs;
  }

  function syncScheduler(): void {
    const nextBackend = context.isInitialized()
      ? activeMusicSnapshotBackend(
          context.currentSource(),
          context.snapshot().status,
          context.usesNativeLocalBackend(),
        )
      : null;
    if (nextBackend !== snapshotBackend) {
      scheduler.setEnabled(false);
      snapshotBackend = nextBackend;
    }
    scheduler.setEnabled(nextBackend !== null);
  }

  return {
    setLoadedState,
    persist,
    syncScheduler,
    resumeScheduler: scheduler.resume,
    stopScheduler: () => scheduler.setEnabled(false),
  };
}
