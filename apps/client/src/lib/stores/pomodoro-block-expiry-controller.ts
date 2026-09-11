import type { PomodoroRuntime } from "./pomodoro-runtime";

interface PomodoroBlockExpiryContext {
  runtime: PomodoroRuntime;
  clearBreakEndWarning(): void;
  clearMusicPausedByPomodoro(): void;
  resetPausedFocusNotificationState(): void;
  stopPausedOpportunityCountdown(): void;
  refreshPausedOpportunityRemaining(nowMs: number): boolean;
  interruptCurrentSegment(endIso: string): Promise<void>;
  skipRemainingSegments(): Promise<void>;
  closeActiveRun(endIso: string): Promise<void>;
  stopVisualTick(): void;
  stopOvertime(): void;
  stopIdleChecking(): void;
  updateTray(): void;
}

export interface PomodoroBlockExpiryController {
  expirePausedBlock(): void;
  expirePausedBlockAndWait(): Promise<void>;
}

/** Owns paused active-block deadline completion and cleanup ordering. */
export function createPomodoroBlockExpiryController(
  context: PomodoroBlockExpiryContext,
): PomodoroBlockExpiryController {
  const runtime = context.runtime;

  function prepareExpiry(): string | null {
    if (runtime.activeBlockEndMs === null) return null;
    context.clearBreakEndWarning();
    context.clearMusicPausedByPomodoro();
    context.resetPausedFocusNotificationState();
    const endIso = new Date(runtime.activeBlockEndMs).toISOString();
    context.stopPausedOpportunityCountdown();
    context.refreshPausedOpportunityRemaining(runtime.activeBlockEndMs);
    return endIso;
  }

  function finishExpiry(): void {
    runtime.sessionStartTime = null;
    context.stopVisualTick();
    context.stopOvertime();
    context.stopIdleChecking();
    runtime.isRunning = false;
    runtime.lastTickMs = null;
    runtime.phaseEndTime = null;
    runtime.remainingSeconds = 0;
    runtime.blockExpired = true;
    context.updateTray();
  }

  function expirePausedBlock(): void {
    const endIso = prepareExpiry();
    if (!endIso) return;
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      const segment = runtime.segments[runtime.currentSegmentIndex];
      if (segment.status === "active") {
        void context.interruptCurrentSegment(endIso);
      }
      void context.skipRemainingSegments();
    }
    void context.closeActiveRun(endIso);
    finishExpiry();
  }

  async function expirePausedBlockAndWait(): Promise<void> {
    const endIso = prepareExpiry();
    if (!endIso) return;
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      const segment = runtime.segments[runtime.currentSegmentIndex];
      if (segment.status === "active") {
        await context.interruptCurrentSegment(endIso);
      }
      await context.skipRemainingSegments();
    }
    await context.closeActiveRun(endIso);
    finishExpiry();
  }

  return { expirePausedBlock, expirePausedBlockAndWait };
}
