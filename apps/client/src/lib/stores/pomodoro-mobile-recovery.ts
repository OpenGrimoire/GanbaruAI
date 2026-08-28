import type {
  PauseInterval,
  PauseReason,
  PersistedSegment,
  SegmentPhase,
} from "$lib/components/calendar/types";
import {
  clonePomodoroConfig,
  isValidPomodoroConfig,
  rhythmPositionCount,
  type PomodoroConfig,
  type PomodoroRhythm,
  type PomodoroRhythmSource,
  type PomodoroPresetKey,
} from "$lib/pomodoro/rhythm";
import type { PomodoroRuntime } from "./pomodoro-runtime";

export type PomodoroMobileRecoveryReason =
  | "multiple_open_runs"
  | "invalid_state"
  | "run_window_expired"
  | "phase_expired";

export type PomodoroMobileRecoveryResult =
  | { kind: "none" }
  | {
      kind: "closed";
      reason: PomodoroMobileRecoveryReason;
      closedRunIds: string[];
    }
  | { kind: "resumed"; run: PomodoroRecoveredMobileRun };

export interface PomodoroRecoveredMobileRun {
  runId: string;
  blockId: string;
  eventTitle: string | null;
  eventDate: string;
  plannedEnd: string;
  startedAt: string;
  recoveredAt: string;
  rhythm: PomodoroRhythm;
  rhythmSource: PomodoroRhythmSource;
  presetKey: PomodoroPresetKey | null;
  idleTimeoutMinutes: number | null;
  segment: PersistedSegment;
  completedFocusCount: number;
  phaseElapsedSeconds: number;
  phaseWorkDurationSeconds: number;
  remainingSeconds: number;
  isRunning: boolean;
  focusExtensionUsed: boolean;
  openPauseReason: PauseReason | null;
}

interface PomodoroMobileRecoveryContext {
  runtime: PomodoroRuntime;
  nativeIdleDetectionAvailable: boolean;
  stopVisualTick(): void;
  stopPausedOpportunityCountdown(): void;
  stopOvertime(): void;
  stopIdleChecking(): void;
  initListeners(): void;
  refreshFutureSegments(blockId: string, eventDate: string): void;
  startHeartbeat(): void;
  startVisualTick(): void;
  startPausedOpportunityCountdown(): void;
  startIdleChecking(): void;
  scheduleBreakEndWarning(): void;
  updateTray(): void;
  publishWindowSnapshot(): void;
  nowMs?(): number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isTimestamp(value: unknown): value is string {
  return typeof value === "string" && Number.isFinite(Date.parse(value));
}

function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function isPositiveInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0;
}

function parsePauseReason(value: unknown): PauseReason | null {
  return value === "idle" || value === "manual" || value === "suspend"
    ? value
    : null;
}

function parsePause(value: unknown): PauseInterval | null {
  if (!isRecord(value) || !isTimestamp(value.startedAt)) return null;
  const reason = parsePauseReason(value.reason);
  if (!reason) return null;
  if (value.endedAt !== null && !isTimestamp(value.endedAt)) return null;
  return {
    startedAt: value.startedAt,
    endedAt: value.endedAt,
    reason,
  };
}

function parsePhase(value: unknown): SegmentPhase | null {
  return value === "focus" || value === "short_break" || value === "long_break"
    ? value
    : null;
}

function parseRhythm(value: unknown): PomodoroRhythm | null {
  if (!isRecord(value)) return null;
  if (value.kind === "count") {
    if (
      !isPositiveInteger(value.focusDurationMinutes)
      || !isPositiveInteger(value.shortBreakMinutes)
      || !isPositiveInteger(value.longBreakMinutes)
      || !isPositiveInteger(value.longBreakAfterFocusCount)
    ) return null;
    return {
      kind: "count",
      focusDurationMinutes: value.focusDurationMinutes,
      shortBreakMinutes: value.shortBreakMinutes,
      longBreakMinutes: value.longBreakMinutes,
      longBreakAfterFocusCount: value.longBreakAfterFocusCount,
    };
  }
  if (value.kind !== "sequence" || !Array.isArray(value.steps)) return null;
  const steps: Extract<PomodoroRhythm, { kind: "sequence" }>["steps"] = [];
  for (const step of value.steps) {
    if (
      !isRecord(step)
      || !isPositiveInteger(step.focusDurationMinutes)
      || (step.breakPhase !== "short_break" && step.breakPhase !== "long_break")
      || !isPositiveInteger(step.breakDurationMinutes)
    ) return null;
    steps.push({
      focusDurationMinutes: step.focusDurationMinutes,
      breakPhase: step.breakPhase,
      breakDurationMinutes: step.breakDurationMinutes,
    });
  }
  return { kind: "sequence", steps };
}

function parseRhythmSource(value: unknown): PomodoroRhythmSource | null {
  return value === "preset" || value === "custom" ? value : null;
}

function parsePresetKey(value: unknown): PomodoroPresetKey | null | undefined {
  if (value === null) return null;
  return value === "adaptive"
    || value === "creative"
    || value === "balanced"
    || value === "deep"
    || value === "extended"
    ? value
    : undefined;
}

function parseSegment(value: unknown): PersistedSegment | null {
  if (!isRecord(value)) return null;
  const phase = parsePhase(value.phase);
  if (!phase || value.status !== "active" || value.actualEnd !== null) return null;
  if (
    !isNonEmptyString(value.id)
    || !isNonEmptyString(value.eventId)
    || !isNonEmptyString(value.eventDate)
    || !isNonEmptyString(value.runId)
    || !isPositiveInteger(value.rhythmPosition)
    || !isTimestamp(value.plannedStart)
    || !isTimestamp(value.plannedEnd)
    || !isTimestamp(value.actualStart)
    || !Array.isArray(value.pauseLog)
  ) return null;
  const pauseLog: PauseInterval[] = [];
  for (const pauseValue of value.pauseLog) {
    const pause = parsePause(pauseValue);
    if (!pause) return null;
    pauseLog.push(pause);
  }
  return {
    id: value.id,
    eventId: value.eventId,
    eventDate: value.eventDate,
    runId: value.runId,
    rhythmPosition: value.rhythmPosition,
    phase,
    plannedStart: value.plannedStart,
    plannedEnd: value.plannedEnd,
    actualStart: value.actualStart,
    actualEnd: null,
    pauseLog,
    status: "active",
  };
}

function parseRecoveryReason(value: unknown): PomodoroMobileRecoveryReason | null {
  return value === "multiple_open_runs"
    || value === "invalid_state"
    || value === "run_window_expired"
    || value === "phase_expired"
    ? value
    : null;
}

function pausesAreChronological(
  pauses: readonly PauseInterval[],
  actualStart: string,
  recoveredAt: string,
): boolean {
  let previousEndMs = Date.parse(actualStart);
  const recoveredAtMs = Date.parse(recoveredAt);
  for (const [index, pause] of pauses.entries()) {
    const startedAtMs = Date.parse(pause.startedAt);
    if (startedAtMs < previousEndMs || startedAtMs > recoveredAtMs) return false;
    if (pause.endedAt === null) return index + 1 === pauses.length;
    const endedAtMs = Date.parse(pause.endedAt);
    if (endedAtMs < startedAtMs || endedAtMs > recoveredAtMs) return false;
    previousEndMs = endedAtMs;
  }
  return true;
}

function phaseMatchesRhythm(
  phase: SegmentPhase,
  position: number,
  rhythm: PomodoroRhythm,
): boolean {
  if (phase === "focus") return true;
  if (rhythm.kind === "count") {
    return phase === (position === rhythm.longBreakAfterFocusCount
      ? "long_break"
      : "short_break");
  }
  return rhythm.steps[position - 1]?.breakPhase === phase;
}

function parseRecoveredRun(value: unknown): PomodoroRecoveredMobileRun | null {
  if (!isRecord(value)) return null;
  const rhythm = parseRhythm(value.rhythm);
  const rhythmSource = parseRhythmSource(value.rhythmSource);
  const presetKey = parsePresetKey(value.presetKey);
  const segment = parseSegment(value.segment);
  const openPauseReason = value.openPauseReason === null
    ? null
    : parsePauseReason(value.openPauseReason);
  if (!rhythm || !rhythmSource || presetKey === undefined || !segment) return null;
  if (value.openPauseReason !== null && openPauseReason === null) return null;
  if (
    !isNonEmptyString(value.runId)
    || !isNonEmptyString(value.blockId)
    || (value.eventTitle !== null && typeof value.eventTitle !== "string")
    || !isNonEmptyString(value.eventDate)
    || !isTimestamp(value.plannedEnd)
    || !isTimestamp(value.startedAt)
    || !isTimestamp(value.recoveredAt)
    || (value.idleTimeoutMinutes !== null && !isPositiveInteger(value.idleTimeoutMinutes))
    || !isNonNegativeInteger(value.completedFocusCount)
    || !isNonNegativeInteger(value.phaseElapsedSeconds)
    || !isPositiveInteger(value.phaseWorkDurationSeconds)
    || !isPositiveInteger(value.remainingSeconds)
    || typeof value.isRunning !== "boolean"
    || typeof value.focusExtensionUsed !== "boolean"
  ) return null;

  const config: PomodoroConfig = {
    rhythm,
    rhythmSource,
    presetKey,
    idleTimeoutMinutes: value.idleTimeoutMinutes,
  };
  const lastPause = segment.pauseLog[segment.pauseLog.length - 1];
  const persistedOpenPauseReason = lastPause?.endedAt === null
    ? lastPause.reason
    : null;
  const recoveredAtMs = Date.parse(value.recoveredAt);
  const plannedEndMs = Date.parse(value.plannedEnd);
  const startedAtMs = Date.parse(value.startedAt);
  const segmentPlannedStartMs = Date.parse(segment.plannedStart);
  const segmentPlannedEndMs = Date.parse(segment.plannedEnd);
  const segmentActualStartMs = Date.parse(segment.actualStart ?? "");
  if (
    !isValidPomodoroConfig(config)
    || segment.runId !== value.runId
    || segment.eventId !== value.blockId
    || segment.eventDate !== value.eventDate
    || plannedEndMs <= recoveredAtMs
    || startedAtMs > recoveredAtMs
    || segmentPlannedStartMs >= segmentPlannedEndMs
    || segmentActualStartMs < startedAtMs
    || segmentActualStartMs > recoveredAtMs
    || segment.rhythmPosition > rhythmPositionCount(config)
    || !phaseMatchesRhythm(segment.phase, segment.rhythmPosition, rhythm)
    || !pausesAreChronological(segment.pauseLog, segment.actualStart ?? "", value.recoveredAt)
    || value.phaseElapsedSeconds >= value.phaseWorkDurationSeconds
    || value.phaseElapsedSeconds + value.remainingSeconds > value.phaseWorkDurationSeconds
    || value.isRunning !== (openPauseReason === null)
    || persistedOpenPauseReason !== openPauseReason
  ) return null;

  return {
    runId: value.runId,
    blockId: value.blockId,
    eventTitle: value.eventTitle,
    eventDate: value.eventDate,
    plannedEnd: value.plannedEnd,
    startedAt: value.startedAt,
    recoveredAt: value.recoveredAt,
    rhythm,
    rhythmSource,
    presetKey,
    idleTimeoutMinutes: value.idleTimeoutMinutes,
    segment,
    completedFocusCount: value.completedFocusCount,
    phaseElapsedSeconds: value.phaseElapsedSeconds,
    phaseWorkDurationSeconds: value.phaseWorkDurationSeconds,
    remainingSeconds: value.remainingSeconds,
    isRunning: value.isRunning,
    focusExtensionUsed: value.focusExtensionUsed,
    openPauseReason,
  };
}

/** Validate the untrusted Tauri recovery response before it reaches timer state. */
export function parsePomodoroMobileRecoveryResult(
  value: unknown,
): PomodoroMobileRecoveryResult {
  if (!isRecord(value) || typeof value.kind !== "string") {
    throw new Error("Invalid mobile pomodoro recovery response");
  }
  if (value.kind === "none") return { kind: "none" };
  if (value.kind === "closed") {
    const reason = parseRecoveryReason(value.reason);
    if (
      !reason
      || !Array.isArray(value.closedRunIds)
      || value.closedRunIds.length === 0
      || !value.closedRunIds.every(isNonEmptyString)
    ) throw new Error("Invalid closed mobile pomodoro recovery response");
    return {
      kind: "closed",
      reason,
      closedRunIds: [...value.closedRunIds],
    };
  }
  if (value.kind === "resumed") {
    const run = parseRecoveredRun(value.run);
    if (!run) throw new Error("Invalid resumed mobile pomodoro recovery response");
    return { kind: "resumed", run };
  }
  throw new Error("Unknown mobile pomodoro recovery response kind");
}

/** Restore a backend-validated mobile run into the existing timer controllers. */
export function applyRecoveredMobileRun(
  recovered: PomodoroRecoveredMobileRun,
  context: PomodoroMobileRecoveryContext,
): void {
  const runtime = context.runtime;
  const nowMs = context.nowMs?.() ?? Date.now();
  const blockEndMs = Date.parse(recovered.plannedEnd);

  context.stopVisualTick();
  context.stopPausedOpportunityCountdown();
  context.stopOvertime();
  context.stopIdleChecking();
  context.initListeners();

  runtime.phase = recovered.segment.phase;
  runtime.remainingSeconds = recovered.remainingSeconds;
  runtime.phaseElapsedSeconds = recovered.phaseElapsedSeconds;
  runtime.phaseWorkDurationSeconds = recovered.phaseWorkDurationSeconds;
  runtime.phaseTotalSeconds = recovered.phaseElapsedSeconds + recovered.remainingSeconds;
  runtime.currentRhythmPosition = recovered.segment.rhythmPosition;
  runtime.isRunning = recovered.isRunning;
  runtime.config = clonePomodoroConfig({
    rhythm: recovered.rhythm,
    rhythmSource: recovered.rhythmSource,
    presetKey: recovered.presetKey,
    idleTimeoutMinutes: recovered.idleTimeoutMinutes,
  });
  runtime.completedPomodoros = recovered.completedFocusCount;
  runtime.sessionStartTime = recovered.startedAt;
  runtime.skipNextBreak = false;
  runtime.notificationShown = false;
  runtime.focusExtensionUsed = recovered.focusExtensionUsed;
  runtime.phaseEndTime = recovered.isRunning
    ? nowMs + recovered.remainingSeconds * 1_000
    : null;
  runtime.activeBlockId = recovered.blockId;
  runtime.activeBlockTitle = recovered.eventTitle;
  runtime.activeRunId = recovered.runId;
  runtime.activeBlockEndMs = blockEndMs;
  runtime.dismissedBlockId = null;
  runtime.autoStartSuppressed = false;
  runtime.segments = [{
    ...recovered.segment,
    pauseLog: recovered.segment.pauseLog.map((pause) => ({ ...pause })),
  }];
  runtime.currentSegmentIndex = 0;
  runtime.segmentEndReasons.clear();
  runtime.segmentVersion += 1;
  runtime.breakOvertimeSeconds = 0;
  runtime.blockExpired = false;
  runtime.lastTickMs = recovered.isRunning ? nowMs : null;
  runtime.suspendedAway = null;
  runtime.idlePaused = null;
  runtime.idleTimeoutMs = context.nativeIdleDetectionAvailable
    && recovered.idleTimeoutMinutes !== null
    ? recovered.idleTimeoutMinutes * 60_000
    : null;

  context.refreshFutureSegments(recovered.blockId, recovered.eventDate);
  context.startHeartbeat();
  if (recovered.isRunning) {
    context.startVisualTick();
    context.startIdleChecking();
    context.scheduleBreakEndWarning();
  } else {
    context.startPausedOpportunityCountdown();
  }
  context.publishWindowSnapshot();
  context.updateTray();
}
