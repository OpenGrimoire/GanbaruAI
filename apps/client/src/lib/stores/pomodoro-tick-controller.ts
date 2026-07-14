import type { PersistedSegment } from "$lib/components/calendar/types";
import { decideTick } from "./pomodoro-machine";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";
import type { PomodoroWindowStateController } from "./pomodoro-window-state-controller";

interface PomodoroTickContext {
  runtime: PomodoroRuntime;
  clock: Pick<
    PomodoroClockController,
    | "actualPhaseElapsedSeconds"
    | "setVisibleRemainingForPause"
    | "recordRunningPhaseProgress"
  >;
  windowState: Pick<PomodoroWindowStateController, "buildTimerSnapshot">;
  closeOverlay(): void;
  clearBreakEndWarning(): void;
  appendSuspendPause(
    segment: PersistedSegment,
    startedAt: string,
    endedAt: string,
  ): void;
  persistSegment(segment: PersistedSegment): void;
  timestampMs(value: string): number;
  interruptCurrentSegment(endIso: string): void;
  skipRemainingSegments(): void;
  closeExpiredRun(endIso: string): void;
  stopVisualTick(): void;
  stopOvertime(): void;
  stopIdleChecking(): void;
  startOvertime(): void;
  advancePhase(): Promise<void>;
  showNotification(): void;
  updateTray(): void;
  nowMs?(): number;
}

export interface PomodoroTickController {
  tick(): void;
}

/** Applies pure tick decisions to runtime and persistence side effects. */
export function createPomodoroTickController(
  context: PomodoroTickContext,
): PomodoroTickController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;

  function markCurrentAndSkipRemaining(endIso: string): void {
    if (
      runtime.currentSegmentIndex < 0
      || runtime.currentSegmentIndex >= runtime.segments.length
    ) return;
    context.interruptCurrentSegment(endIso);
    context.skipRemainingSegments();
  }

  function recordSuspendPause(startedAt: string, endedAt: string): void {
    if (
      runtime.currentSegmentIndex < 0
      || runtime.currentSegmentIndex >= runtime.segments.length
    ) return;
    const segment = runtime.segments[runtime.currentSegmentIndex];
    context.appendSuspendPause(segment, startedAt, endedAt);
    context.persistSegment(segment);
  }

  function tick(): void {
    const nowMs = currentMs();
    const result = decideTick(context.windowState.buildTimerSnapshot(), nowMs);
    switch (result.kind) {
      case "suspend_and_block_expired":
        context.closeOverlay();
        context.clearBreakEndWarning();
        recordSuspendPause(result.suspendStartIso, result.suspendEndIso);
        context.clock.setVisibleRemainingForPause(
          result.preSuspendRemainingSeconds,
          context.clock.actualPhaseElapsedSeconds(),
          context.timestampMs(result.suspendStartIso),
        );
        context.stopVisualTick();
        runtime.isRunning = false;
        runtime.lastTickMs = null;
        markCurrentAndSkipRemaining(result.suspendStartIso);
        context.closeExpiredRun(result.suspendStartIso);
        runtime.sessionStartTime = null;
        context.stopOvertime();
        context.stopIdleChecking();
        runtime.phaseEndTime = null;
        runtime.blockExpired = true;
        context.updateTray();
        return;
      case "suspend_block_active":
        context.clearBreakEndWarning();
        recordSuspendPause(result.suspendStartIso, result.suspendEndIso);
        context.clock.setVisibleRemainingForPause(
          result.preSuspendRemainingSeconds,
          context.clock.actualPhaseElapsedSeconds(),
          context.timestampMs(result.suspendStartIso),
        );
        runtime.phaseEndTime = result.newPhaseEndTime;
        context.stopVisualTick();
        runtime.isRunning = false;
        runtime.lastTickMs = null;
        runtime.suspendedAway = { awaySeconds: result.awaySeconds };
        context.updateTray();
        return;
      case "block_expired": {
        context.closeOverlay();
        context.clearBreakEndWarning();
        runtime.lastTickMs = nowMs;
        context.clock.recordRunningPhaseProgress(0);
        const endIso = new Date(runtime.activeBlockEndMs!).toISOString();
        markCurrentAndSkipRemaining(endIso);
        context.closeExpiredRun(endIso);
        runtime.sessionStartTime = null;
        context.stopVisualTick();
        context.stopOvertime();
        context.stopIdleChecking();
        runtime.isRunning = false;
        runtime.lastTickMs = null;
        runtime.phaseEndTime = null;
        runtime.blockExpired = true;
        context.updateTray();
        return;
      }
      case "break_finished":
        context.clearBreakEndWarning();
        runtime.lastTickMs = nowMs;
        context.clock.recordRunningPhaseProgress(0);
        runtime.remainingSeconds = 0;
        context.stopVisualTick();
        runtime.isRunning = false;
        context.startOvertime();
        return;
      case "focus_finished":
        runtime.lastTickMs = nowMs;
        context.clock.recordRunningPhaseProgress(0);
        void context.advancePhase();
        return;
      case "countdown_with_notification":
        runtime.lastTickMs = nowMs;
        context.clock.recordRunningPhaseProgress(result.remainingSeconds);
        context.showNotification();
        context.updateTray();
        return;
      case "countdown":
        runtime.lastTickMs = nowMs;
        context.clock.recordRunningPhaseProgress(result.remainingSeconds);
        context.updateTray();
    }
  }

  return { tick };
}
