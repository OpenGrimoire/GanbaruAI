import type { PersistedSegment } from "$lib/components/calendar/types";
import type {
  PomodoroRunEndReason,
  PomodoroRunEventType,
  PomodoroSegmentEndReason,
} from "./pomodoro-backend-writes";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";

interface SessionSegmentCapabilities {
  appendPause(
    segment: PersistedSegment,
    startedAt: string,
    reason: "manual",
  ): void;
  closeLastPause(segment: PersistedSegment, endedAt: string): boolean;
  markSegment(
    index: number,
    status: "interrupted",
    persist: boolean,
    endIso: string,
    reason: PomodoroSegmentEndReason,
  ): Promise<void>;
  skipPlannedSegmentsAfter(index: number, errorPrefix: string): Promise<void>;
  clearSegments(): void;
}

interface PomodoroSessionContext {
  runtime: PomodoroRuntime;
  clock: Pick<
    PomodoroClockController,
    | "canPauseResume"
    | "refreshPausedOpportunityRemaining"
    | "activeBlockDeadlineReached"
    | "resetPhaseProgress"
  >;
  segments: SessionSegmentCapabilities;
  closeActiveRun(
    endedAt: string,
    endReason: PomodoroRunEndReason,
    segmentStatus: "completed" | "interrupted",
    segmentEndReason: PomodoroSegmentEndReason,
    eventType: PomodoroRunEventType,
    throwOnError?: boolean,
  ): Promise<void>;
  persistSegment(segment: PersistedSegment, errorPrefix: string): void;
  cleanupOrphans(): Promise<void>;
  closeOverlay(): void;
  clearBreakEndWarning(): void;
  clearMusicPausedByPomodoro(): void;
  resetPausedFocusNotificationState(): void;
  pauseMusicForPomodoroPause(): void;
  resumeMusicFromPomodoroPause(): void;
  scheduleBreakEndWarning(): void;
  updateTray(): void;
  stopPausedOpportunityCountdown(): void;
  startPausedOpportunityCountdown(): void;
  startVisualTick(): void;
  stopVisualTick(): void;
  stopOvertime(): void;
  startIdleChecking(): void;
  stopIdleChecking(): void;
  initListeners(): void;
  expirePausedBlockAtDeadline(): void;
  resetFocusNotificationState(): void;
  recordManualPhaseAdvanceEvent(occurredAt: string): void;
  advancePhase(): Promise<void>;
  defaultFocusSeconds: number;
  nowMs?(): number;
  nowIso?(): string;
}

export interface PomodoroSessionController {
  stopSession(): Promise<void>;
  completeActiveBlockAt(endIso?: string): Promise<void>;
  pause(): void;
  resume(): void;
  skip(): void;
  cleanupOrphans(): Promise<void>;
}

/** Owns manual session commands and their persistence ordering. */
export function createPomodoroSessionController(
  context: PomodoroSessionContext,
): PomodoroSessionController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;
  const currentIso = context.nowIso ?? (() => new Date().toISOString());

  function resetStoppedState(clearPauseState: boolean): void {
    context.stopVisualTick();
    context.stopOvertime();
    context.stopIdleChecking();
    runtime.isRunning = false;
    runtime.lastTickMs = null;
    runtime.phaseEndTime = null;
    if (!runtime.dismissedBlockId && runtime.activeBlockId) {
      runtime.dismissedBlockId = runtime.activeBlockId;
    }
    runtime.activeBlockId = null;
    runtime.activeRunId = null;
    runtime.activeBlockEndMs = null;
    runtime.idleTimeoutMs = null;
    runtime.blockExpired = false;
    if (clearPauseState) {
      runtime.suspendedAway = null;
      runtime.idlePaused = null;
    }
    runtime.phase = "focus";
    context.clock.resetPhaseProgress(context.defaultFocusSeconds);
    runtime.currentRhythmPosition = 1;
    runtime.completedPomodoros = 0;
    runtime.sessionStartTime = null;
    runtime.skipNextBreak = false;
    context.resetFocusNotificationState();
  }

  async function stopSession(): Promise<void> {
    context.closeOverlay();
    context.clearBreakEndWarning();
    context.clearMusicPausedByPomodoro();
    context.resetPausedFocusNotificationState();
    context.stopPausedOpportunityCountdown();
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      const segment = runtime.segments[runtime.currentSegmentIndex];
      if (segment.status === "active") {
        const endIso = currentIso();
        await context.segments.markSegment(
          runtime.currentSegmentIndex,
          "interrupted",
          true,
          endIso,
          "stopped",
        );
        await context.closeActiveRun(
          endIso,
          "stopped",
          "interrupted",
          "stopped",
          "stop",
        );
      }
      await context.segments.skipPlannedSegmentsAfter(
        runtime.currentSegmentIndex,
        "Failed to skip segment:",
      );
    } else if (runtime.activeRunId) {
      await context.closeActiveRun(
        currentIso(),
        "stopped",
        "interrupted",
        "stopped",
        "stop",
      );
    }
    resetStoppedState(false);
    context.segments.clearSegments();
    context.updateTray();
  }

  async function completeActiveBlockAt(endIso: string = currentIso()): Promise<void> {
    context.closeOverlay();
    context.clearBreakEndWarning();
    context.resetPausedFocusNotificationState();
    const parsedEndMs = Date.parse(endIso);
    const normalizedEndIso = Number.isFinite(parsedEndMs)
      ? new Date(parsedEndMs).toISOString()
      : currentIso();
    context.clearMusicPausedByPomodoro();
    context.stopPausedOpportunityCountdown();
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      await context.segments.markSegment(
        runtime.currentSegmentIndex,
        "interrupted",
        true,
        normalizedEndIso,
        "event_expired",
      );
      await context.segments.skipPlannedSegmentsAfter(
        runtime.currentSegmentIndex,
        "Failed to skip segment:",
      );
    } else if (runtime.activeRunId) {
      await context.closeActiveRun(
        normalizedEndIso,
        "completed",
        "interrupted",
        "event_expired",
        "complete",
        true,
      );
    }
    if (runtime.activeRunId) {
      await context.closeActiveRun(
        normalizedEndIso,
        "completed",
        "interrupted",
        "event_expired",
        "complete",
        true,
      );
    }
    resetStoppedState(true);
    context.updateTray();
  }

  function pause(): void {
    if (!runtime.isRunning || !context.clock.canPauseResume()) return;
    context.clearBreakEndWarning();
    context.resetPausedFocusNotificationState();
    runtime.isRunning = false;
    runtime.phaseEndTime = null;
    runtime.lastTickMs = null;
    context.stopVisualTick();
    context.stopIdleChecking();
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      const segment = runtime.segments[runtime.currentSegmentIndex];
      context.segments.appendPause(segment, currentIso(), "manual");
      context.persistSegment(segment, "Failed to save pause:");
    }
    context.startPausedOpportunityCountdown();
    context.pauseMusicForPomodoroPause();
    context.updateTray();
  }

  function resume(): void {
    if (runtime.isRunning) return;
    context.initListeners();
    context.clock.refreshPausedOpportunityRemaining();
    if (context.clock.activeBlockDeadlineReached()) {
      context.expirePausedBlockAtDeadline();
      return;
    }
    if (!context.clock.canPauseResume()) return;
    context.resetPausedFocusNotificationState();
    context.stopPausedOpportunityCountdown();
    runtime.isRunning = true;
    const nowMs = currentMs();
    runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
    if (!runtime.sessionStartTime) runtime.sessionStartTime = currentIso();
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      const segment = runtime.segments[runtime.currentSegmentIndex];
      if (context.segments.closeLastPause(segment, currentIso())) {
        context.persistSegment(segment, "Failed to save resume:");
      }
    }
    context.startVisualTick();
    runtime.lastTickMs = nowMs;
    context.scheduleBreakEndWarning();
    context.startIdleChecking();
    context.resumeMusicFromPomodoroPause();
    context.updateTray();
  }

  function skip(): void {
    context.recordManualPhaseAdvanceEvent(currentIso());
    void context.advancePhase();
  }

  async function cleanupOrphans(): Promise<void> {
    await context.cleanupOrphans().catch((error: unknown) => {
      console.warn("Failed to recover open pomodoro runs:", error);
    });
  }

  return { stopSession, completeActiveBlockAt, pause, resume, skip, cleanupOrphans };
}
