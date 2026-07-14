import type { SegmentPhase, PersistedSegment } from "$lib/components/calendar/types";
import type { PomodoroBoundaryAdaptiveDecision } from "$lib/pomodoro/adaptive/persistence";
import {
  breakAfterFocusPosition,
  clonePomodoroConfig,
  focusDurationMinutesAtPosition,
  nextRhythmPosition,
  normalizeRhythmPosition,
} from "$lib/pomodoro/rhythm";
import { decideBoundaryAdaptiveForState, isAdaptiveCountConfig } from "./pomodoro-adaptive-decisions";
import type { PomodoroRunEventWrite } from "./pomodoro-backend-writes";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroExtensionController } from "./pomodoro-extension-controller";
import {
  decideAdvancePhase,
  decideReconfigure,
  decideTransition,
  TIME_MULTIPLIER,
  type PomodoroConfig,
} from "./pomodoro-machine";
import type { PomodoroRuntime } from "./pomodoro-runtime";
import type { PomodoroWindowStateController } from "./pomodoro-window-state-controller";

interface RebuildOptions {
  runEndReason: "reconfigured" | "block_transition";
  segmentStatus: "completed" | "interrupted";
  segmentEndReason: "reconfigured" | "block_transition";
  eventType: "reconfigure" | "block_transition";
  startTrigger: "reconfigure" | "block_transition";
  inheritedFocusSeconds: number;
  inheritedRhythmPosition: number;
}

interface PhaseSegmentCapabilities {
  activeSegment(): PersistedSegment | null;
  rebuildSegments(
    blockId: string,
    eventEnd: string,
    eventDate: string,
    options: RebuildOptions,
  ): Promise<void>;
  boundarySegment(
    phase: SegmentPhase,
    rhythmPosition: number,
    startedAt: string,
  ): PersistedSegment | null;
  activateBoundarySegment(
    segment: PersistedSegment,
    decision: PomodoroBoundaryAdaptiveDecision,
  ): Promise<void>;
  activateSegment(index: number): Promise<void>;
  markSegment(
    index: number,
    status: "completed",
    persist: boolean,
    endIso: string,
    reason?: "completed",
  ): Promise<void>;
}

interface PomodoroPhaseContext {
  runtime: PomodoroRuntime;
  clock: Pick<
    PomodoroClockController,
    | "actualPhaseElapsedSeconds"
    | "setPhaseRemainingSeconds"
  >;
  extensions: Pick<
    PomodoroExtensionController,
    "isBreakPhase" | "resetFocusNotificationState" | "cappedActiveBreakEndIso"
  >;
  windowState: Pick<PomodoroWindowStateController, "buildTimerSnapshot">;
  segments: PhaseSegmentCapabilities;
  stopPausedOpportunityCountdown(): void;
  stopVisualTick(): void;
  startVisualTick(): void;
  stopOvertime(): void;
  isOvertimeActive(): boolean;
  initListeners(): void;
  startIdleChecking(): void;
  closeOverlay(): void;
  clearBreakEndWarning(): void;
  clearMusicPausedByPomodoro(): void;
  resetPausedFocusNotificationState(): void;
  scheduleBreakEndWarning(): void;
  showBreakOverlay(seconds: number): void;
  updateTray(): void;
  recordRunEvent(event: PomodoroRunEventWrite, errorPrefix: string): void;
  nowMs?(): number;
  nowIso?(): string;
}

export interface PomodoroPhaseController {
  startFocusSession(): Promise<void>;
  reconfigureSession(
    blockId: string,
    config: PomodoroConfig,
    eventEnd: string,
    eventDate: string,
  ): Promise<void>;
  transitionToBlock(
    blockId: string,
    config: PomodoroConfig,
    eventEnd: string,
    eventDate: string,
  ): Promise<void>;
  advancePhase(): Promise<void>;
}

/** Owns phase boundaries, adaptive ordering, and block transition continuity. */
export function createPomodoroPhaseController(
  context: PomodoroPhaseContext,
): PomodoroPhaseController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;
  const currentIso = context.nowIso ?? (() => new Date().toISOString());
  let advanceInFlight = false;

  function applyBoundaryAdaptiveDecision(
    decision: PomodoroBoundaryAdaptiveDecision | null,
  ): void {
    if (!decision || !isAdaptiveCountConfig(runtime.config)) return;
    runtime.config = {
      ...clonePomodoroConfig(runtime.config),
      rhythm: { ...decision.selectedRhythm },
    };
    runtime.currentRhythmPosition = normalizeRhythmPosition(
      runtime.config,
      runtime.currentRhythmPosition,
    );
  }

  async function boundaryDecision(
    opportunityKind: "focus_start" | "break_start",
    occurredAt: string,
  ): Promise<PomodoroBoundaryAdaptiveDecision | null> {
    return decideBoundaryAdaptiveForState({
      config: runtime.config,
      idleDetectionEnabled: runtime.idleTimeoutMs !== null,
      activeBlockEndMs: runtime.activeBlockEndMs,
      activeSegmentPlannedEnd: context.segments.activeSegment()?.plannedEnd ?? null,
      opportunityKind,
      occurredAt,
    });
  }

  async function activateNextPlannedFocus(): Promise<void> {
    const nextFocus = runtime.segments.findIndex(
      (segment, index) =>
        index > runtime.currentSegmentIndex
        && segment.phase === "focus"
        && segment.status === "planned",
    );
    if (nextFocus !== -1) await context.segments.activateSegment(nextFocus);
  }

  async function activateBoundaryOrFallback(
    phase: SegmentPhase,
    startedAt: string,
    decision: PomodoroBoundaryAdaptiveDecision | null,
  ): Promise<boolean> {
    const segment = decision
      ? context.segments.boundarySegment(
          phase,
          runtime.currentRhythmPosition,
          startedAt,
        )
      : null;
    if (!segment || !decision) return false;
    await context.segments.activateBoundarySegment(segment, decision);
    return true;
  }

  async function startFocusSession(): Promise<void> {
    context.closeOverlay();
    context.clearBreakEndWarning();
    context.clearMusicPausedByPomodoro();
    context.resetPausedFocusNotificationState();
    context.stopPausedOpportunityCountdown();
    context.stopVisualTick();
    context.stopOvertime();
    const boundaryOccurredAt = currentIso();
    const adaptiveDecision = await boundaryDecision(
      "focus_start",
      boundaryOccurredAt,
    );
    applyBoundaryAdaptiveDecision(adaptiveDecision);
    const nextPosition = context.extensions.isBreakPhase(runtime.phase)
      ? nextRhythmPosition(runtime.config, runtime.currentRhythmPosition)
      : runtime.currentRhythmPosition;
    runtime.phase = "focus";
    runtime.currentRhythmPosition = nextPosition;
    context.clock.setPhaseRemainingSeconds(
      focusDurationMinutesAtPosition(
        runtime.config,
        runtime.currentRhythmPosition,
      ) * TIME_MULTIPLIER,
    );
    context.extensions.resetFocusNotificationState();
    runtime.isRunning = true;
    const startedAt = currentIso();
    const nowMs = currentMs();
    runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
    runtime.sessionStartTime = startedAt;
    context.startVisualTick();
    runtime.lastTickMs = nowMs;
    if (!await activateBoundaryOrFallback("focus", startedAt, adaptiveDecision)) {
      await activateNextPlannedFocus();
    }
    context.updateTray();
  }

  async function reconfigureSession(
    blockId: string,
    newConfig: PomodoroConfig,
    eventEnd: string,
    eventDate: string,
  ): Promise<void> {
    context.stopPausedOpportunityCountdown();
    runtime.activeBlockEndMs = new Date(eventEnd.replace(" ", "T")).getTime();
    const elapsedSeconds = context.clock.actualPhaseElapsedSeconds();
    const result = decideReconfigure({
      phase: runtime.phase,
      remainingSeconds: runtime.remainingSeconds,
      elapsedSeconds,
      currentConfig: runtime.config,
      newConfig,
      currentRhythmPosition: runtime.currentRhythmPosition,
      hasOvertimeInterval: context.isOvertimeActive(),
    });
    runtime.config = clonePomodoroConfig(newConfig);
    runtime.currentRhythmPosition = normalizeRhythmPosition(
      runtime.config,
      runtime.currentRhythmPosition,
    );
    const nowMs = currentMs();
    context.clock.setPhaseRemainingSeconds(
      result.newRemainingSeconds,
      elapsedSeconds,
      nowMs,
    );
    if (runtime.isRunning) {
      runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
    } else if (result.exitOvertime) {
      context.stopOvertime();
      runtime.isRunning = true;
      runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
      context.startVisualTick();
      runtime.lastTickMs = nowMs;
    }
    context.scheduleBreakEndWarning();
    if (result.resetNotification) runtime.notificationShown = false;
    await context.segments.rebuildSegments(blockId, eventEnd, eventDate, {
      runEndReason: "reconfigured",
      segmentStatus: "completed",
      segmentEndReason: "reconfigured",
      eventType: "reconfigure",
      startTrigger: "reconfigure",
      inheritedFocusSeconds: runtime.phase === "focus" ? elapsedSeconds : 0,
      inheritedRhythmPosition: runtime.currentRhythmPosition,
    });
    context.updateTray();
  }

  async function transitionToBlock(
    blockId: string,
    newConfig: PomodoroConfig,
    eventEnd: string,
    eventDate: string,
  ): Promise<void> {
    context.stopPausedOpportunityCountdown();
    const previousConfig = runtime.config;
    const elapsedSeconds = context.clock.actualPhaseElapsedSeconds();
    const inheritedFocusSeconds = runtime.phase === "focus" ? elapsedSeconds : 0;
    const inheritedRhythmPosition = runtime.currentRhythmPosition;
    runtime.activeBlockId = blockId;
    runtime.activeBlockEndMs = new Date(eventEnd.replace(" ", "T")).getTime();
    context.initListeners();
    const result = decideTransition({
      previousConfig,
      newConfig,
      phase: runtime.phase,
      remainingSeconds: runtime.remainingSeconds,
      elapsedFocusSeconds: runtime.phase === "focus" ? elapsedSeconds : undefined,
      currentRhythmPosition: runtime.currentRhythmPosition,
      blockExpired: runtime.blockExpired,
    });
    runtime.config = clonePomodoroConfig(newConfig);
    runtime.currentRhythmPosition = normalizeRhythmPosition(
      runtime.config,
      runtime.currentRhythmPosition,
    );
    const nowMs = currentMs();
    switch (result.kind) {
      case "trigger_break":
        runtime.completedPomodoros += 1;
        runtime.sessionStartTime = null;
        runtime.notificationShown = false;
        runtime.phase = result.breakPhase;
        context.clock.setPhaseRemainingSeconds(result.breakDurationSeconds);
        runtime.currentRhythmPosition = result.rhythmPosition;
        runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
        if (!runtime.isRunning) {
          runtime.isRunning = true;
          context.startVisualTick();
          runtime.lastTickMs = nowMs;
        }
        context.showBreakOverlay(runtime.remainingSeconds);
        break;
      case "continue_focus":
        context.clock.setPhaseRemainingSeconds(result.remainingSeconds, elapsedSeconds);
        runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
        if (!runtime.isRunning) {
          runtime.isRunning = true;
          if (!runtime.sessionStartTime) runtime.sessionStartTime = currentIso();
          context.startVisualTick();
          runtime.lastTickMs = nowMs;
          context.startIdleChecking();
        }
        if (result.resetNotification) runtime.notificationShown = false;
        break;
      case "fresh_start":
        context.stopOvertime();
        runtime.phase = "focus";
        context.clock.setPhaseRemainingSeconds(result.remainingSeconds);
        runtime.currentRhythmPosition = 1;
        runtime.completedPomodoros = 0;
        context.extensions.resetFocusNotificationState();
        runtime.isRunning = true;
        runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
        runtime.sessionStartTime = currentIso();
        context.startVisualTick();
        runtime.lastTickMs = nowMs;
        context.startIdleChecking();
        break;
      case "keep_break":
        break;
    }
    runtime.blockExpired = false;
    await context.segments.rebuildSegments(blockId, eventEnd, eventDate, {
      runEndReason: "block_transition",
      segmentStatus: "interrupted",
      segmentEndReason: "block_transition",
      eventType: "block_transition",
      startTrigger: "block_transition",
      inheritedFocusSeconds,
      inheritedRhythmPosition,
    });
    context.updateTray();
  }

  async function advancePhase(): Promise<void> {
    if (advanceInFlight) return;
    advanceInFlight = true;
    try {
      const boundaryOpportunity =
        runtime.phase === "focus" && !runtime.skipNextBreak
          ? "break_start"
          : "focus_start";
      const skippedBreakPhase = runtime.phase === "focus" && runtime.skipNextBreak
        ? breakAfterFocusPosition(
            runtime.config,
            runtime.currentRhythmPosition,
          ).phase
        : "short_break";
      let boundaryOccurredAt = currentIso();
      if (runtime.phase === "focus") {
        context.clearMusicPausedByPomodoro();
        runtime.completedPomodoros += 1;
        runtime.sessionStartTime = null;
        runtime.notificationShown = false;
        await context.segments.markSegment(
          runtime.currentSegmentIndex,
          "completed",
          true,
          boundaryOccurredAt,
          "completed",
        );
      } else {
        boundaryOccurredAt = context.extensions.cappedActiveBreakEndIso();
        await context.segments.markSegment(
          runtime.currentSegmentIndex,
          "completed",
          true,
          boundaryOccurredAt,
        );
      }
      const adaptiveDecision = await boundaryDecision(
        boundaryOpportunity,
        boundaryOccurredAt,
      );
      applyBoundaryAdaptiveDecision(adaptiveDecision);
      const result = decideAdvancePhase(context.windowState.buildTimerSnapshot());
      const nowMs = currentMs();
      switch (result.kind) {
        case "skip_break_to_focus": {
          runtime.skipNextBreak = false;
          runtime.phase = "focus";
          context.clock.setPhaseRemainingSeconds(result.remainingSeconds);
          context.extensions.resetFocusNotificationState();
          runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
          runtime.currentRhythmPosition = result.nextRhythmPosition;
          const occurredAt = currentIso();
          if (runtime.activeRunId) {
            context.recordRunEvent({
              runId: runtime.activeRunId,
              segmentId: null,
              eventType: "skip_break",
              occurredAt,
              phase: skippedBreakPhase,
              reason: "skip_next_break",
              durationSeconds: null,
            }, "Failed to record skipped break:");
          }
          if (!await activateBoundaryOrFallback("focus", occurredAt, adaptiveDecision)) {
            const breakIndex = runtime.currentSegmentIndex + 1;
            const breakSegment = runtime.segments[breakIndex];
            if (breakSegment && breakSegment.phase !== "focus") {
              breakSegment.status = "skipped";
              breakSegment.actualStart = occurredAt;
              breakSegment.actualEnd = occurredAt;
              await activateNextPlannedFocus();
            }
          }
          break;
        }
        case "focus_to_long_break":
        case "focus_to_short_break": {
          runtime.phase = result.kind === "focus_to_long_break"
            ? "long_break"
            : "short_break";
          context.clock.setPhaseRemainingSeconds(result.remainingSeconds);
          runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
          runtime.currentRhythmPosition = result.rhythmPosition;
          const occurredAt = currentIso();
          if (!await activateBoundaryOrFallback(
            runtime.phase,
            occurredAt,
            adaptiveDecision,
          )) {
            const breakIndex = runtime.currentSegmentIndex + 1;
            if (runtime.segments[breakIndex]?.phase === runtime.phase) {
              await context.segments.activateSegment(breakIndex);
            }
          }
          context.showBreakOverlay(runtime.remainingSeconds);
          break;
        }
        case "break_to_focus": {
          runtime.phase = "focus";
          runtime.currentRhythmPosition = result.nextRhythmPosition;
          context.clock.setPhaseRemainingSeconds(result.remainingSeconds);
          context.extensions.resetFocusNotificationState();
          runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
          const occurredAt = currentIso();
          if (!await activateBoundaryOrFallback("focus", occurredAt, adaptiveDecision)) {
            await activateNextPlannedFocus();
          }
          break;
        }
      }
      context.updateTray();
    } finally {
      advanceInFlight = false;
    }
  }

  return { startFocusSession, reconfigureSession, transitionToBlock, advancePhase };
}
