import type { PomodoroPhase } from "@ganbaru-ai/shared-types";
import type { PersistedSegment, SegmentPhase } from "$lib/components/calendar/types";
import {
  BREAK_EXTENSION_SECONDS,
  BREAK_OVERTIME_RAIL_GRACE_SECONDS,
  limitRemainingSecondsToBlockEnd,
  MAX_BREAK_EXTENSION_SECONDS,
  phaseDurationSeconds,
} from "./pomodoro-machine";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRunEventWrite } from "./pomodoro-backend-writes";
import type { PomodoroRuntime } from "./pomodoro-runtime";

const FOCUS_EXTENSION_SECONDS = 180;

interface PomodoroExtensionContext {
  runtime: PomodoroRuntime;
  clock: Pick<
    PomodoroClockController,
    | "actualPhaseElapsedSeconds"
    | "phaseWorkRemainingSeconds"
    | "setPhaseRemainingSeconds"
    | "sessionActive"
  >;
  hasOvertime(): boolean;
  activeSegment(): PersistedSegment | null;
  persistSegment(
    segment: PersistedSegment,
    errorPrefix: string,
    publishSnapshot: boolean,
    occurredAt?: string,
  ): void;
  recordRunEvent(event: PomodoroRunEventWrite, errorPrefix: string): void;
  scheduleBreakEndWarning(): void;
  updateTray(): void;
  nowMs?(): number;
  nowIso?(): string;
}

export interface PomodoroExtensionController {
  resetFocusNotificationState(): void;
  isBreakPhase(value: PomodoroPhase | SegmentPhase): value is "short_break" | "long_break";
  canExtendFocusTime(addSeconds?: number): boolean;
  canExtendBreakTime(addSeconds?: number): boolean;
  addFocusTime(seconds?: number): void;
  addBreakTime(seconds?: number): void;
  cappedActiveBreakEndIso(actualEndMs?: number): string;
  recordManualPhaseAdvanceEvent(occurredAt: string): void;
}

/** Owns focus and break extension limits, persistence, and manual boundary events. */
export function createPomodoroExtensionController(
  context: PomodoroExtensionContext,
): PomodoroExtensionController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;
  const currentIso = context.nowIso ?? (() => new Date().toISOString());

  function resetFocusNotificationState(): void {
    runtime.notificationShown = false;
    runtime.focusExtensionUsed = false;
  }

  function isBreakPhase(
    value: PomodoroPhase | SegmentPhase,
  ): value is "short_break" | "long_break" {
    return value === "short_break" || value === "long_break";
  }

  function canExtendFocusTime(addSeconds = FOCUS_EXTENSION_SECONDS): boolean {
    if (
      runtime.phase !== "focus"
      || runtime.focusExtensionUsed
      || !context.clock.sessionActive()
      || addSeconds <= 0
    ) return false;
    const nowMs = currentMs();
    const currentWorkRemainingSeconds = context.clock.phaseWorkRemainingSeconds();
    const currentVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    const extendedVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds + addSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    return extendedVisibleRemainingSeconds > currentVisibleRemainingSeconds;
  }

  function usedBreakExtensionSeconds(): number {
    if (!isBreakPhase(runtime.phase)) return 0;
    const configuredBreakSeconds = phaseDurationSeconds(
      runtime.phase,
      runtime.config,
      runtime.currentRhythmPosition,
    );
    return Math.max(
      0,
      runtime.phaseWorkDurationSeconds - configuredBreakSeconds,
    );
  }

  function canExtendBreakTime(addSeconds = BREAK_EXTENSION_SECONDS): boolean {
    if (
      !isBreakPhase(runtime.phase)
      || !context.clock.sessionActive()
      || context.hasOvertime()
      || addSeconds <= 0
    ) return false;
    const remainingExtensionSeconds =
      MAX_BREAK_EXTENSION_SECONDS - usedBreakExtensionSeconds();
    if (remainingExtensionSeconds <= 0) return false;
    const nowMs = currentMs();
    const currentWorkRemainingSeconds = context.clock.phaseWorkRemainingSeconds();
    const currentVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    const extendedVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds
        + Math.min(Math.ceil(addSeconds), remainingExtensionSeconds),
      runtime.activeBlockEndMs,
      nowMs,
    );
    return extendedVisibleRemainingSeconds > currentVisibleRemainingSeconds;
  }

  function addFocusTime(seconds = FOCUS_EXTENSION_SECONDS): void {
    if (!canExtendFocusTime(seconds)) return;
    const elapsedSeconds = context.clock.actualPhaseElapsedSeconds();
    const nextWorkRemainingSeconds =
      context.clock.phaseWorkRemainingSeconds() + seconds;
    const occurredAt = currentIso();
    runtime.focusExtensionUsed = true;
    runtime.notificationShown = false;
    context.clock.setPhaseRemainingSeconds(
      nextWorkRemainingSeconds,
      elapsedSeconds,
    );
    if (runtime.phaseEndTime !== null) {
      runtime.phaseEndTime = currentMs() + runtime.remainingSeconds * 1000;
    }
    const segment = context.activeSegment();
    if (segment?.phase === "focus" && segment.status === "active") {
      segment.plannedEnd = new Date(
        new Date(segment.plannedEnd).getTime() + seconds * 1000,
      ).toISOString();
      context.persistSegment(
        segment,
        "Failed to save focus extension:",
        true,
        occurredAt,
      );
    }
    context.updateTray();
  }

  function addBreakTime(seconds = BREAK_EXTENSION_SECONDS): void {
    if (!canExtendBreakTime(seconds)) return;
    const requestedSeconds = Math.max(0, Math.ceil(seconds));
    const remainingExtensionSeconds =
      MAX_BREAK_EXTENSION_SECONDS - usedBreakExtensionSeconds();
    const allowedSeconds = Math.min(requestedSeconds, remainingExtensionSeconds);
    const nowMs = currentMs();
    const elapsedSeconds = context.clock.actualPhaseElapsedSeconds();
    const currentWorkRemainingSeconds = context.clock.phaseWorkRemainingSeconds();
    const currentVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    const extendedVisibleRemainingSeconds = limitRemainingSecondsToBlockEnd(
      currentWorkRemainingSeconds + allowedSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    const addedVisibleSeconds =
      extendedVisibleRemainingSeconds - currentVisibleRemainingSeconds;
    if (addedVisibleSeconds <= 0) return;
    context.clock.setPhaseRemainingSeconds(
      currentWorkRemainingSeconds + addedVisibleSeconds,
      elapsedSeconds,
      nowMs,
    );
    if (runtime.phaseEndTime !== null) {
      runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
    }
    context.scheduleBreakEndWarning();
    const segment = context.activeSegment();
    if (segment && isBreakPhase(segment.phase) && segment.status === "active") {
      segment.plannedEnd = new Date(
        new Date(segment.plannedEnd).getTime() + addedVisibleSeconds * 1000,
      ).toISOString();
      context.persistSegment(segment, "Failed to save break extension:", true);
    }
    context.updateTray();
  }

  function cappedActiveBreakEndIso(actualEndMs = currentMs()): string {
    const segment = context.activeSegment();
    if (!segment || !isBreakPhase(segment.phase)) {
      return new Date(actualEndMs).toISOString();
    }
    const plannedEndMs = new Date(segment.plannedEnd).getTime();
    return new Date(Math.min(
      actualEndMs,
      plannedEndMs + BREAK_OVERTIME_RAIL_GRACE_SECONDS * 1000,
    )).toISOString();
  }

  function recordManualPhaseAdvanceEvent(occurredAt: string): void {
    if (!runtime.activeRunId) return;
    const segment = context.activeSegment();
    if (runtime.phase === "focus" && segment?.status === "active") {
      context.recordRunEvent({
        runId: runtime.activeRunId,
        segmentId: segment.id,
        eventType: "go_to_break_now",
        occurredAt,
        phase: "focus",
        reason: "manual",
        durationSeconds: null,
      }, "Failed to record early focus end:");
      return;
    }
    if (
      isBreakPhase(runtime.phase)
      && segment?.status === "active"
      && context.clock.phaseWorkRemainingSeconds() > 0
    ) {
      context.recordRunEvent({
        runId: runtime.activeRunId,
        segmentId: segment.id,
        eventType: "start_focus_now",
        occurredAt,
        phase: runtime.phase,
        reason: "manual",
        durationSeconds: null,
      }, "Failed to record early break end:");
    }
  }

  return {
    resetFocusNotificationState,
    isBreakPhase,
    canExtendFocusTime,
    canExtendBreakTime,
    addFocusTime,
    addBreakTime,
    cappedActiveBreakEndIso,
    recordManualPhaseAdvanceEvent,
  };
}
