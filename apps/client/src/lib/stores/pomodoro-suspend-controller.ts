import type { PomodoroRunEndReason, PomodoroRunEventType, PomodoroSegmentEndReason } from "./pomodoro-backend-writes";
import type { PomodoroRuntime } from "./pomodoro-runtime";

interface PomodoroSuspendContext {
  runtime: PomodoroRuntime;
  stopPausedOpportunityCountdown(): void;
  startVisualTick(): void;
  scheduleBreakEndWarning(): void;
  startIdleChecking(): void;
  stopIdleChecking(): void;
  updateTray(): void;
  clearBreakEndWarning(): void;
  stopOvertime(): void;
  resetPhaseProgress(seconds: number): void;
  resetFocusNotificationState(): void;
  clearSegments(): void;
  activeSegmentPauseStart(): string | null;
  interruptCurrentSegment(endIso: string): Promise<void>;
  skipRemainingSegments(): Promise<void>;
  closeActiveRun(
    endedAt: string,
    endReason: PomodoroRunEndReason,
    segmentStatus: "completed" | "interrupted",
    segmentEndReason: PomodoroSegmentEndReason,
    eventType: PomodoroRunEventType,
  ): Promise<void>;
  defaultFocusSeconds: number;
  nowMs?(): number;
  nowIso?(): string;
}

export interface PomodoroSuspendController {
  dismiss(resume: boolean): Promise<void>;
}

/** Bridges suspend decisions into timer, persistence, and idle ownership. */
export function createPomodoroSuspendController(
  context: PomodoroSuspendContext,
): PomodoroSuspendController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;
  const currentIso = context.nowIso ?? (() => new Date().toISOString());

  async function dismiss(resume: boolean): Promise<void> {
    context.stopPausedOpportunityCountdown();
    if (resume) {
      runtime.suspendedAway = null;
      runtime.activeBlockEndMs = null;
      runtime.isRunning = true;
      const nowMs = currentMs();
      runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
      runtime.lastTickMs = nowMs;
      context.startVisualTick();
      context.scheduleBreakEndWarning();
      context.startIdleChecking();
      context.updateTray();
      return;
    }

    context.clearBreakEndWarning();
    const endIso = context.activeSegmentPauseStart() ?? currentIso();
    if (
      runtime.currentSegmentIndex >= 0
      && runtime.currentSegmentIndex < runtime.segments.length
    ) {
      await context.interruptCurrentSegment(endIso);
      await context.skipRemainingSegments();
    }
    await context.closeActiveRun(
      endIso,
      "stopped",
      "interrupted",
      "stopped",
      "stop",
    );
    runtime.suspendedAway = null;
    context.stopOvertime();
    context.stopIdleChecking();
    runtime.isRunning = false;
    runtime.phaseEndTime = null;
    runtime.dismissedBlockId = runtime.activeBlockId;
    runtime.activeBlockId = null;
    runtime.activeBlockTitle = null;
    runtime.activeBlockEndMs = null;
    runtime.idleTimeoutMs = null;
    runtime.lastTickMs = null;
    runtime.phase = "focus";
    context.resetPhaseProgress(context.defaultFocusSeconds);
    runtime.currentRhythmPosition = 1;
    runtime.completedPomodoros = 0;
    runtime.sessionStartTime = null;
    runtime.skipNextBreak = false;
    context.resetFocusNotificationState();
    context.clearSegments();
    context.updateTray();
  }

  return { dismiss };
}
