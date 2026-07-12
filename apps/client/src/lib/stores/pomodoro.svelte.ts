import type { PomodoroPhase } from "@ganbaru-ai/shared-types";
import type {
  PersistedSegment,
  SegmentPhase,
} from "$lib/components/calendar/types";
import {
  clonePomodoroConfig,
  focusDurationMinutesAtPosition,
  normalizeRhythmPosition,
  phaseDurationMinutesAtPosition,
  rhythmPositionCount,
} from "$lib/pomodoro/rhythm";
import {
  type PomodoroAdaptivePlannedBlockWrite,
  type PomodoroRunStartAdaptiveDecision,
} from "$lib/pomodoro/adaptive/persistence";
import {
  nowIso,
  type PomodoroBackendWriteOptions,
  type PomodoroRunClosure,
  type PomodoroRunEndReason,
  type PomodoroRunEventType,
  type PomodoroSegmentEndReason,
} from "./pomodoro-backend-writes";
import { createPomodoroRunRepository } from "./pomodoro-run-repository";
import { createPomodoroEffects } from "./pomodoro-effects.svelte";
import { createPomodoroWindowCoordinator } from "./pomodoro-window-coordinator";
import { createPomodoroSegmentController } from "./pomodoro-segment-controller";
import { createPomodoroIdleController } from "./pomodoro-idle-controller";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";
import { createPomodoroDoomscrollingController } from "./pomodoro-doomscrolling-controller";
import { createPomodoroTimerRuntime } from "./pomodoro-timer-runtime";
import { createPomodoroWindowStateController } from "./pomodoro-window-state-controller";
import { createPomodoroActiveBlockController } from "./pomodoro-active-block-controller";
import { createPomodoroSessionController } from "./pomodoro-session-controller";
import { createPomodoroCommandController } from "./pomodoro-command-controller";
import { createPomodoroSuspendController } from "./pomodoro-suspend-controller";
import { createPomodoroBlockExpiryController } from "./pomodoro-block-expiry-controller";
import { createPomodoroExtensionController } from "./pomodoro-extension-controller";
import { createPomodoroOvertimeController } from "./pomodoro-overtime-controller";
import { createPomodoroPhaseController } from "./pomodoro-phase-controller";
import { createPomodoroTickController } from "./pomodoro-tick-controller";
import { isAdaptiveCountConfig } from "./pomodoro-adaptive-decisions";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  type PomodoroConfig,
  DEFAULT_CONFIG,
  TIME_MULTIPLIER,
  BREAK_EXTENSION_SECONDS,
} from "./pomodoro-machine";
import {
  type IdlePauseState,
  type PomodoroWindowCommand,
} from "./pomodoro-window-sync";

const pomodoroCoordinator = getCurrentWindow().label === "main";

const DEFAULT_FOCUS_SECONDS = focusDurationMinutesAtPosition(DEFAULT_CONFIG, 1) * TIME_MULTIPLIER;

let phase = $state<PomodoroPhase>("focus");
let remainingSeconds = $state(DEFAULT_FOCUS_SECONDS);
let phaseTotalSeconds = $state(DEFAULT_FOCUS_SECONDS);
let phaseElapsedSeconds = 0;
let phaseWorkDurationSeconds = DEFAULT_FOCUS_SECONDS;
let currentRhythmPosition = $state(1);
let isRunning = $state(false);
let config = $state<PomodoroConfig>(clonePomodoroConfig(DEFAULT_CONFIG));
let completedPomodoros = $state(0);
let sessionStartTime: string | null = null;
let skipNextBreak = false;
let notificationShown = false;
let focusExtensionUsed = false;
let phaseEndTime: number | null = null;
let activeBlockId = $state<string | null>(null);
let activeRunId = $state<string | null>(null);
let activeBlockEndMs = $state<number | null>(null);
let dismissedBlockId = $state<string | null>(null);
let autoStartSuppressed = $state(false);
const FOCUS_EXTENSION_SECONDS = 180;

// Segment tracking state
let segments = $state<PersistedSegment[]>([]);
let currentSegmentIndex = -1;
const segmentEndReasons = new Map<string, PomodoroSegmentEndReason>();

// Bumped after DB writes to persisted segments complete, so DayColumn re-fetches.
let segmentVersion = $state(0);

// Tracks seconds elapsed after a break timer reaches 0 but before user acknowledgment.
// Reactive ($state) so the rail derived recomputes during the grace window.
let breakOvertimeSeconds = $state(0);

// Block expiry: set when the calendar event ends naturally (via tick).
// Keeps activeBlockId intact so the next checkActiveBlock can trigger
// transitionToBlock for focus inheritance to a successor block.
let blockExpired = $state(false);

// Suspend/wake detection
let lastTickMs: number | null = null;
let suspendedAway = $state<{ awaySeconds: number } | null>(null);

// Idle detection
let idleTimeoutMs: number | null = null; // null = disabled
let idlePaused = $state<IdlePauseState | null>(null);

const runtime: PomodoroRuntime = {
  get phase() { return phase; },
  set phase(value) { phase = value; },
  get remainingSeconds() { return remainingSeconds; },
  set remainingSeconds(value) { remainingSeconds = value; },
  get phaseTotalSeconds() { return phaseTotalSeconds; },
  set phaseTotalSeconds(value) { phaseTotalSeconds = value; },
  get phaseElapsedSeconds() { return phaseElapsedSeconds; },
  set phaseElapsedSeconds(value) { phaseElapsedSeconds = value; },
  get phaseWorkDurationSeconds() { return phaseWorkDurationSeconds; },
  set phaseWorkDurationSeconds(value) { phaseWorkDurationSeconds = value; },
  get currentRhythmPosition() { return currentRhythmPosition; },
  set currentRhythmPosition(value) { currentRhythmPosition = value; },
  get isRunning() { return isRunning; },
  set isRunning(value) { isRunning = value; },
  get config() { return config; },
  set config(value) { config = value; },
  get completedPomodoros() { return completedPomodoros; },
  set completedPomodoros(value) { completedPomodoros = value; },
  get sessionStartTime() { return sessionStartTime; },
  set sessionStartTime(value) { sessionStartTime = value; },
  get skipNextBreak() { return skipNextBreak; },
  set skipNextBreak(value) { skipNextBreak = value; },
  get notificationShown() { return notificationShown; },
  set notificationShown(value) { notificationShown = value; },
  get focusExtensionUsed() { return focusExtensionUsed; },
  set focusExtensionUsed(value) { focusExtensionUsed = value; },
  get phaseEndTime() { return phaseEndTime; },
  set phaseEndTime(value) { phaseEndTime = value; },
  get activeBlockId() { return activeBlockId; },
  set activeBlockId(value) { activeBlockId = value; },
  get activeRunId() { return activeRunId; },
  set activeRunId(value) { activeRunId = value; },
  get activeBlockEndMs() { return activeBlockEndMs; },
  set activeBlockEndMs(value) { activeBlockEndMs = value; },
  get dismissedBlockId() { return dismissedBlockId; },
  set dismissedBlockId(value) { dismissedBlockId = value; },
  get autoStartSuppressed() { return autoStartSuppressed; },
  set autoStartSuppressed(value) { autoStartSuppressed = value; },
  get segments() { return segments; },
  set segments(value) { segments = value; },
  get currentSegmentIndex() { return currentSegmentIndex; },
  set currentSegmentIndex(value) { currentSegmentIndex = value; },
  segmentEndReasons,
  get segmentVersion() { return segmentVersion; },
  set segmentVersion(value) { segmentVersion = value; },
  get breakOvertimeSeconds() { return breakOvertimeSeconds; },
  set breakOvertimeSeconds(value) { breakOvertimeSeconds = value; },
  get blockExpired() { return blockExpired; },
  set blockExpired(value) { blockExpired = value; },
  get lastTickMs() { return lastTickMs; },
  set lastTickMs(value) { lastTickMs = value; },
  get suspendedAway() { return suspendedAway; },
  set suspendedAway(value) { suspendedAway = value; },
  get idleTimeoutMs() { return idleTimeoutMs; },
  set idleTimeoutMs(value) { idleTimeoutMs = value; },
  get idlePaused() { return idlePaused; },
  set idlePaused(value) { idlePaused = value; },
};

const clockController = createPomodoroClockController(runtime);
const doomscrollingController = createPomodoroDoomscrollingController(
  runtime,
  clockController,
  { isCoordinator: () => pomodoroCoordinator },
);
const {
  actualPhaseElapsedSeconds,
  phaseWorkRemainingSeconds,
  setPhaseRemainingSeconds,
  setVisibleRemainingForPause,
  refreshCurrentPhaseLimit,
  recordRunningPhaseProgress,
  refreshPausedOpportunityRemaining,
  activeBlockDeadlineReached,
  resetPhaseProgress,
} = clockController;

const timerRuntime = createPomodoroTimerRuntime({
  runtime,
  clock: clockController,
  tick,
  expirePausedBlockAtDeadline,
  updateTray: () => effects.updateTray(),
  sendHeartbeat,
});
const {
  startVisualTick,
  stopVisualTick,
  startPausedOpportunityCountdown,
  stopPausedOpportunityCountdown,
  startHeartbeat,
  stopHeartbeat,
} = timerRuntime;

function writeCurrentDoomscrollingRuntimeState(force = false): void {
  doomscrollingController.writeCurrentState(force);
}

const windowStateController = createPomodoroWindowStateController(
  runtime,
  () => pomodoroCoordinator,
);

const windowCoordinator = createPomodoroWindowCoordinator({
  isCoordinator: () => pomodoroCoordinator,
  beforePublishSnapshot: writeCurrentDoomscrollingRuntimeState,
  buildSnapshot: windowStateController.buildWindowSnapshot,
  applySnapshot: windowStateController.applyWindowSnapshot,
  handleCommand: handleWindowCommand,
});

function publishWindowSnapshot(): void {
  windowCoordinator.publishSnapshot();
}

function forwardWindowCommand(command: PomodoroWindowCommand): boolean {
  return windowCoordinator.forwardCommand(command);
}

// Segment helpers

function segmentEndReasonFor(seg: PersistedSegment): PomodoroSegmentEndReason | null {
  const explicit = segmentEndReasons.get(seg.id);
  if (explicit) return explicit;
  if (seg.status === "completed") return "completed";
  if (seg.status === "skipped") return "skipped_by_user";
  if (seg.status === "interrupted") return "stopped";
  return null;
}

const runRepository = createPomodoroRunRepository({
  endReasonForSegment: segmentEndReasonFor,
  completeWrite: completePomodoroBackendWrite,
});

function applyRunStartAdaptiveDecision(decision: PomodoroRunStartAdaptiveDecision): void {
  if (!isAdaptiveCountConfig(config)) return;
  config = {
    ...clonePomodoroConfig(config),
    rhythm: { ...decision.selectedRhythm },
  };
  currentRhythmPosition = normalizeRhythmPosition(config, currentRhythmPosition);
  if (phase !== "focus") return;
  const startedMs = Date.parse(decision.occurredAt);
  const nowMs = Number.isFinite(startedMs) ? startedMs : Date.now();
  setPhaseRemainingSeconds(
    focusDurationMinutesAtPosition(config, currentRhythmPosition) * TIME_MULTIPLIER,
    0,
    nowMs,
  );
  if (isRunning) {
    phaseEndTime = nowMs + remainingSeconds * 1000;
  }
}

function runClosure(
  endedAt: string,
  endReason: PomodoroRunEndReason,
  segmentStatus: "completed" | "interrupted",
  segmentEndReason: PomodoroSegmentEndReason,
  eventType: PomodoroRunEventType,
): PomodoroRunClosure | null {
  if (!activeRunId) return null;
  const normalizedEndedAt = segmentController.normalizedActiveSegmentEndIso(endedAt);
  return {
    runId: activeRunId,
    endedAt: normalizedEndedAt,
    endReason,
    segmentStatus,
    segmentEndReason,
    eventType,
  };
}

async function closeActiveRun(
  endedAt: string,
  endReason: PomodoroRunEndReason,
  segmentStatus: "completed" | "interrupted",
  segmentEndReason: PomodoroSegmentEndReason,
  eventType: PomodoroRunEventType,
  throwOnError = false,
): Promise<void> {
  const closure = runClosure(endedAt, endReason, segmentStatus, segmentEndReason, eventType);
  stopHeartbeat();
  try {
    if (closure) await runRepository.closeRun(closure, "Failed to close pomodoro run:", throwOnError);
    activeRunId = null;
  } catch (error) {
    if (activeRunId) startHeartbeat();
    throw error;
  }
}

function pomodoroSessionActive(nowMs: number = Date.now()): boolean {
  return clockController.sessionActive(nowMs);
}

function pausedFocusPulseActive(): boolean {
  return (
    phase === "focus" &&
    !isRunning &&
    !suspendedAway &&
    !idlePaused &&
    !overtimeController.isActive() &&
    pomodoroSessionActive()
  );
}

const effects = createPomodoroEffects({
  isCoordinator: () => pomodoroCoordinator,
  phase: () => phase,
  remainingSeconds: () => remainingSeconds,
  totalSeconds: () => phaseTotalSeconds,
  isRunning: () => isRunning,
  phaseEndTime: () => phaseEndTime,
  isActive: () => pomodoroSessionActive(),
  canPauseResume: () => canPauseResumeSession(),
  canAddFocusTime: () => canExtendFocusTime(),
  pausedFocusPulseActive,
  notificationShown: () => notificationShown,
  setNotificationShown: (value) => {
    notificationShown = value;
  },
  publishWindowSnapshot,
  writeDoomscrollingRuntimeState: writeCurrentDoomscrollingRuntimeState,
  initListeners,
});

const segmentController = createPomodoroSegmentController({
  get phase() {
    return phase;
  },
  set phase(value) {
    phase = value;
  },
  get remainingSeconds() {
    return remainingSeconds;
  },
  set remainingSeconds(value) {
    remainingSeconds = value;
  },
  get currentRhythmPosition() {
    return currentRhythmPosition;
  },
  set currentRhythmPosition(value) {
    currentRhythmPosition = value;
  },
  get config() {
    return config;
  },
  set config(value) {
    config = value;
  },
  get isRunning() {
    return isRunning;
  },
  set isRunning(value) {
    isRunning = value;
  },
  get activeBlockId() {
    return activeBlockId;
  },
  set activeBlockId(value) {
    activeBlockId = value;
  },
  get activeRunId() {
    return activeRunId;
  },
  set activeRunId(value) {
    activeRunId = value;
  },
  get activeBlockEndMs() {
    return activeBlockEndMs;
  },
  set activeBlockEndMs(value) {
    activeBlockEndMs = value;
  },
  get idleTimeoutMs() {
    return idleTimeoutMs;
  },
  set idleTimeoutMs(value) {
    idleTimeoutMs = value;
  },
  get segments() {
    return segments;
  },
  set segments(value) {
    segments = value;
  },
  get currentSegmentIndex() {
    return currentSegmentIndex;
  },
  set currentSegmentIndex(value) {
    currentSegmentIndex = value;
  },
  get segmentVersion() {
    return segmentVersion;
  },
  set segmentVersion(value) {
    segmentVersion = value;
  },
  segmentEndReasons,
  publishWindowSnapshot,
  refreshCurrentPhaseLimit,
  scheduleBreakEndWarning: effects.scheduleBreakEndWarning,
  updateTray: effects.updateTray,
  startHeartbeat,
  stopSession: stopSessionInternal,
  isPausedForBridgeSegment: () =>
    !isRunning && !overtimeController.isActive() && !suspendedAway && !idlePaused,
  applyRunStartAdaptiveDecision,
}, runRepository);

const idleController = createPomodoroIdleController({
  get phase() {
    return phase;
  },
  set phase(value) {
    phase = value;
  },
  get isRunning() {
    return isRunning;
  },
  set isRunning(value) {
    isRunning = value;
  },
  get suspendedAway() {
    return suspendedAway;
  },
  set suspendedAway(value) {
    suspendedAway = value;
  },
  get idlePaused() {
    return idlePaused;
  },
  set idlePaused(value) {
    idlePaused = value;
  },
  get idleTimeoutMs() {
    return idleTimeoutMs;
  },
  set idleTimeoutMs(value) {
    idleTimeoutMs = value;
  },
  get phaseEndTime() {
    return phaseEndTime;
  },
  set phaseEndTime(value) {
    phaseEndTime = value;
  },
  get activeBlockEndMs() {
    return activeBlockEndMs;
  },
  set activeBlockEndMs(value) {
    activeBlockEndMs = value;
  },
  get activeBlockId() {
    return activeBlockId;
  },
  set activeBlockId(value) {
    activeBlockId = value;
  },
  get dismissedBlockId() {
    return dismissedBlockId;
  },
  set dismissedBlockId(value) {
    dismissedBlockId = value;
  },
  get activeRunId() {
    return activeRunId;
  },
  set activeRunId(value) {
    activeRunId = value;
  },
  get config() {
    return config;
  },
  set config(value) {
    config = value;
  },
  get currentRhythmPosition() {
    return currentRhythmPosition;
  },
  set currentRhythmPosition(value) {
    currentRhythmPosition = value;
  },
  get remainingSeconds() {
    return remainingSeconds;
  },
  set remainingSeconds(value) {
    remainingSeconds = value;
  },
  get segments() {
    return segments;
  },
  set segments(value) {
    segments = value;
  },
  get currentSegmentIndex() {
    return currentSegmentIndex;
  },
  set currentSegmentIndex(value) {
    currentSegmentIndex = value;
  },
  get completedPomodoros() {
    return completedPomodoros;
  },
  set completedPomodoros(value) {
    completedPomodoros = value;
  },
  get sessionStartTime() {
    return sessionStartTime;
  },
  set sessionStartTime(value) {
    sessionStartTime = value;
  },
  get skipNextBreak() {
    return skipNextBreak;
  },
  set skipNextBreak(value) {
    skipNextBreak = value;
  },
  get lastTickMs() {
    return lastTickMs;
  },
  set lastTickMs(value) {
    lastTickMs = value;
  },
  startVisualTick,
  stopVisualTick,
  activeBlockDeadlineReached,
  expirePausedBlockAtDeadlineAndWait,
  stopPausedOpportunityCountdown,
  closePomodoroOverlay: effects.closePomodoroOverlay,
  clearMusicPausedByPomodoro: effects.clearMusicPausedByPomodoro,
  stopOvertime,
  resetFocusNotificationState,
  resetPhaseProgress,
  setPhaseRemainingSeconds,
  setVisibleRemainingForPause,
  actualPhaseElapsedSeconds,
  closeActiveRun,
  updateTray: effects.updateTray,
}, runRepository, segmentController, DEFAULT_FOCUS_SECONDS);

const overtimeController = createPomodoroOvertimeController({
  runtime,
  publishWindowSnapshot,
  playBreakFinishedAlert: effects.playBreakFinishedAlert,
  startConfiguredAlertInterval:
    effects.startConfiguredBreakFinishedAlertInterval,
  completeOvertimeBreak: async () => {
    await segmentController.markSegment(
      currentSegmentIndex,
      "completed",
      true,
      cappedActiveBreakEndIso(),
    );
    await startFocusSession();
  },
});

const extensionController = createPomodoroExtensionController({
  runtime,
  clock: clockController,
  hasOvertime: overtimeController.isActive,
  activeSegment: segmentController.activeSegment,
  persistSegment: runRepository.persistSegment,
  recordRunEvent: runRepository.recordRunEvent,
  scheduleBreakEndWarning: effects.scheduleBreakEndWarning,
  updateTray: effects.updateTray,
});

const phaseController = createPomodoroPhaseController({
  runtime,
  clock: clockController,
  extensions: extensionController,
  windowState: windowStateController,
  segments: segmentController,
  stopPausedOpportunityCountdown,
  stopVisualTick,
  startVisualTick,
  stopOvertime,
  isOvertimeActive: overtimeController.isActive,
  initListeners,
  startIdleChecking: idleController.startChecking,
  closeOverlay: effects.closePomodoroOverlay,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  clearMusicPausedByPomodoro: effects.clearMusicPausedByPomodoro,
  resetPausedFocusNotificationState: effects.resetPausedFocusNotificationState,
  scheduleBreakEndWarning: effects.scheduleBreakEndWarning,
  showBreakOverlay: effects.showBreakOverlay,
  updateTray: effects.updateTray,
  recordRunEvent: runRepository.recordRunEvent,
});

const tickController = createPomodoroTickController({
  runtime,
  clock: clockController,
  windowState: windowStateController,
  closeOverlay: effects.closePomodoroOverlay,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  appendSuspendPause: (segment, startedAt, endedAt) =>
    segmentController.appendPause(segment, startedAt, "suspend", endedAt),
  persistSegment: (segment) =>
    runRepository.persistSegment(
      segment,
      "Failed to save suspend pause:",
      false,
    ),
  timestampMs: segmentController.timestampMs,
  interruptCurrentSegment: (endIso) => {
    void segmentController.markSegment(
      currentSegmentIndex,
      "interrupted",
      true,
      endIso,
      "event_expired",
    );
  },
  skipRemainingSegments: () => {
    void segmentController.skipPlannedSegmentsAfter(
      currentSegmentIndex,
      "Failed to skip segment:",
    );
  },
  closeExpiredRun: (endIso) => {
    void closeActiveRun(
      endIso,
      "completed",
      "interrupted",
      "event_expired",
      "complete",
    );
  },
  stopVisualTick,
  stopOvertime,
  stopIdleChecking: idleController.stopChecking,
  startOvertime: overtimeController.start,
  advancePhase,
  showNotification: effects.showNotification,
  updateTray: effects.updateTray,
});

const blockExpiryController = createPomodoroBlockExpiryController({
  runtime,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  clearMusicPausedByPomodoro: effects.clearMusicPausedByPomodoro,
  resetPausedFocusNotificationState: effects.resetPausedFocusNotificationState,
  stopPausedOpportunityCountdown,
  refreshPausedOpportunityRemaining,
  interruptCurrentSegment: (endIso) =>
    segmentController.markSegment(
      currentSegmentIndex,
      "interrupted",
      true,
      endIso,
      "event_expired",
    ),
  skipRemainingSegments: () =>
    segmentController.skipPlannedSegmentsAfter(
      currentSegmentIndex,
      "Failed to skip segment:",
    ),
  closeActiveRun: (endIso) =>
    closeActiveRun(
      endIso,
      "completed",
      "interrupted",
      "event_expired",
      "complete",
    ),
  stopVisualTick,
  stopOvertime,
  stopIdleChecking: idleController.stopChecking,
  updateTray: effects.updateTray,
});

const activeBlockController = createPomodoroActiveBlockController({
  runtime,
  segments: segmentController,
  transferActiveEventReference: (input) =>
    runRepository.transferActiveEventReference(input),
  setActiveTimeoutMs: idleController.setActiveTimeoutMs,
  startIdleChecking: idleController.startChecking,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  clearMusicPausedByPomodoro: effects.clearMusicPausedByPomodoro,
  stopPausedOpportunityCountdown,
  stopVisualTick,
  startVisualTick,
  initListeners,
  setPhaseRemainingSeconds,
  resetFocusNotificationState,
  updateTray: effects.updateTray,
  publishWindowSnapshot,
  hasOvertime: overtimeController.isActive,
  reconfigureSession,
  transitionToBlock,
});
const {
  setDismissedBlockId,
  clearBlockExpired: clearBlockExpiredInternal,
  transferBlockId: transferBlockIdInternal,
  adoptTransferredBlockId: adoptTransferredBlockIdInternal,
  startFromBlock: startFromBlockInternal,
} = activeBlockController;

const sessionController = createPomodoroSessionController({
  runtime,
  clock: clockController,
  segments: segmentController,
  closeActiveRun,
  persistSegment: (segment, errorPrefix) =>
    runRepository.persistSegment(segment, errorPrefix, false),
  cleanupOrphans: runRepository.cleanupOrphans,
  closeOverlay: effects.closePomodoroOverlay,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  clearMusicPausedByPomodoro: effects.clearMusicPausedByPomodoro,
  resetPausedFocusNotificationState: effects.resetPausedFocusNotificationState,
  pauseMusicForPomodoroPause: effects.pauseMusicForPomodoroPause,
  resumeMusicFromPomodoroPause: effects.resumeMusicFromPomodoroPause,
  scheduleBreakEndWarning: effects.scheduleBreakEndWarning,
  updateTray: effects.updateTray,
  stopPausedOpportunityCountdown,
  startPausedOpportunityCountdown,
  startVisualTick,
  stopVisualTick,
  stopOvertime,
  startIdleChecking: idleController.startChecking,
  stopIdleChecking: idleController.stopChecking,
  initListeners,
  expirePausedBlockAtDeadline,
  resetFocusNotificationState,
  recordManualPhaseAdvanceEvent,
  advancePhase,
  defaultFocusSeconds: DEFAULT_FOCUS_SECONDS,
});

const suspendController = createPomodoroSuspendController({
  runtime,
  stopPausedOpportunityCountdown,
  startVisualTick,
  scheduleBreakEndWarning: effects.scheduleBreakEndWarning,
  startIdleChecking: idleController.startChecking,
  stopIdleChecking: idleController.stopChecking,
  updateTray: effects.updateTray,
  clearBreakEndWarning: effects.clearBreakEndWarning,
  stopOvertime,
  resetPhaseProgress,
  resetFocusNotificationState,
  clearSegments: segmentController.clearSegments,
  activeSegmentPauseStart: () => {
    const segment = segmentController.activeSegment();
    return segment?.pauseLog[segment.pauseLog.length - 1]?.startedAt ?? null;
  },
  interruptCurrentSegment: (endIso) =>
    segmentController.markSegment(
      currentSegmentIndex,
      "interrupted",
      true,
      endIso,
      "stopped",
    ),
  skipRemainingSegments: () =>
    segmentController.skipPlannedSegmentsAfter(
      currentSegmentIndex,
      "Failed to skip segment:",
    ),
  closeActiveRun,
  defaultFocusSeconds: DEFAULT_FOCUS_SECONDS,
});

const commandController = createPomodoroCommandController({
  runtime,
  isCoordinator: () => pomodoroCoordinator,
  publishWindowSnapshot,
  setDismissedBlockId,
  clearBlockExpired: clearBlockExpiredInternal,
  transferBlockId: transferBlockIdInternal,
  startFromBlock: startFromBlockInternal,
  setActiveIdleThresholdMinutes: idleController.setActiveThresholdMinutes,
  dismissSuspend,
  dismissIdle: idleController.dismiss,
  markIdleFocusFailed: idleController.markFocusFailed,
  completeActiveBlockAt: completeActiveBlockAtInternal,
  stopSession: stopSessionInternal,
  pause: pauseSession,
  resume: resumeSession,
  skip: skipSession,
  addFocusTime: addFocusTimeInternal,
  addBreakTime: addBreakTimeInternal,
  cleanupOrphans: cleanupOrphansInternal,
  canPauseResume: canPauseResumeSession,
  advancePhase,
  updateTray: effects.updateTray,
  recordManualPhaseAdvanceEvent,
  recordRunEvent: runRepository.recordRunEvent,
  activeSegmentId: () => segmentController.activeSegment()?.id ?? null,
  markCurrentSegmentSkipped: (occurredAt) =>
    segmentController.markSegment(
      currentSegmentIndex,
      "interrupted",
      true,
      occurredAt,
      "skipped_by_user",
    ),
  startFocusSession,
  completeCurrentBreak: (endIso) =>
    segmentController.markSegment(
      currentSegmentIndex,
      "completed",
      true,
      endIso,
    ),
  cappedActiveBreakEndIso,
  pausedFocusPulseActive,
  suppressPausedFocusNotifications:
    effects.suppressPausedFocusNotificationsForCurrentPause,
  nowIso,
});

function expirePausedBlockAtDeadline(): void {
  blockExpiryController.expirePausedBlock();
}

async function expirePausedBlockAtDeadlineAndWait(): Promise<void> {
  await blockExpiryController.expirePausedBlockAndWait();
}

function resetFocusNotificationState(): void {
  extensionController.resetFocusNotificationState();
}

function isBreakPhase(value: PomodoroPhase | SegmentPhase): value is "short_break" | "long_break" {
  return extensionController.isBreakPhase(value);
}

function canExtendFocusTime(addSeconds: number = FOCUS_EXTENSION_SECONDS): boolean {
  return extensionController.canExtendFocusTime(addSeconds);
}

function canPauseResumeSession(nowMs: number = Date.now()): boolean {
  return clockController.canPauseResume(nowMs);
}

function canExtendBreakTime(addSeconds: number = BREAK_EXTENSION_SECONDS): boolean {
  return extensionController.canExtendBreakTime(addSeconds);
}

function addFocusTimeInternal(seconds: number = FOCUS_EXTENSION_SECONDS): void {
  extensionController.addFocusTime(seconds);
}

function addBreakTimeInternal(seconds: number = BREAK_EXTENSION_SECONDS): void {
  extensionController.addBreakTime(seconds);
}

function cappedActiveBreakEndIso(actualEndMs: number = Date.now()): string {
  return extensionController.cappedActiveBreakEndIso(actualEndMs);
}

function completePomodoroBackendWrite(options: PomodoroBackendWriteOptions = {}): void {
  if (options.bumpSegmentVersion !== false) segmentVersion++;
  if (options.publishSnapshot !== false) publishWindowSnapshot();
}

function recordManualPhaseAdvanceEvent(occurredAt: string): void {
  extensionController.recordManualPhaseAdvanceEvent(occurredAt);
}

function sendHeartbeat(): void {
  if (!activeRunId) return;
  writeCurrentDoomscrollingRuntimeState(true);
  runRepository.sendHeartbeat(activeRunId, nowIso());
}

/**
 * User explicitly changed the pomodoro config on the active block.
 * Adjusts the current phase timer to the new duration and rebuilds segments.
 */
async function reconfigureSession(
  blockId: string,
  newConfig: PomodoroConfig,
  eventEnd: string,
  eventDate: string,
): Promise<void> {
  await phaseController.reconfigureSession(blockId, newConfig, eventEnd, eventDate);
}

/**
 * Seamless transition between adjacent/overlapping blocks.
 * Preserves focus continuity via inheritance: accumulated focus time is
 * compared to the new block's focus threshold. If exceeded, a break triggers.
 * If not, focus continues with adjusted remaining time.
 */
async function transitionToBlock(
  blockId: string,
  newConfig: PomodoroConfig,
  eventEnd: string,
  eventDate: string,
): Promise<void> {
  await phaseController.transitionToBlock(blockId, newConfig, eventEnd, eventDate);
}

// Listeners

function handleWindowCommand(command: PomodoroWindowCommand): void {
  commandController.handleWindowCommand(command);
}

function initListeners(): void {
  commandController.initListeners();
}

// Session control

function stopOvertime() {
  overtimeController.stop();
}

async function startFocusSession(): Promise<void> {
  await phaseController.startFocusSession();
}

async function dismissSuspend(resume: boolean): Promise<void> {
  await suspendController.dismiss(resume);
}

// Timer

function tick() {
  tickController.tick();
}

async function advancePhase() {
  await phaseController.advancePhase();
}

async function stopSessionInternal(): Promise<void> {
  await sessionController.stopSession();
}

async function completeActiveBlockAtInternal(endIso: string = nowIso()): Promise<void> {
  await sessionController.completeActiveBlockAt(endIso);
}

function pauseSession(): void {
  sessionController.pause();
}

function resumeSession(): void {
  sessionController.resume();
}

function skipSession(): void {
  sessionController.skip();
}

async function cleanupOrphansInternal(): Promise<void> {
  await sessionController.cleanupOrphans();
}

// Public API

export function getPomodoro() {
  windowCoordinator.init();
  initListeners();
  return {
    get phase() {
      return phase;
    },
    get remainingSeconds() {
      return remainingSeconds;
    },
    get phaseElapsedSeconds() {
      return phaseElapsedSeconds;
    },
    get phaseWorkDurationSeconds() {
      return phaseWorkDurationSeconds;
    },
    get currentConfig() {
      return config;
    },
    get currentRhythmPosition() {
      return currentRhythmPosition;
    },
    get totalRhythmPositions() {
      return rhythmPositionCount(config);
    },
    get currentCycle() {
      return currentRhythmPosition;
    },
    get totalCycles() {
      return rhythmPositionCount(config);
    },
    get isRunning() {
      return isRunning;
    },
    get isActive() {
      return pomodoroSessionActive();
    },
    get completedPomodoros() {
      return completedPomodoros;
    },
    get totalSecondsForPhase() {
      return phaseTotalSeconds;
    },
    get canAddFocusTime() {
      return canExtendFocusTime();
    },
    get canPauseResume() {
      return canPauseResumeSession();
    },
    get pausedPulseFrame() {
      return effects.currentPausedTrayPulseFrame();
    },
    get pausedPulseAmount() {
      return effects.currentPausedPulseAmount();
    },
    get formattedTime() {
      const mins = Math.floor(remainingSeconds / 60);
      const secs = remainingSeconds % 60;
      return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
    },
    get activeBlockId() {
      return activeBlockId;
    },
    get activeRunId() {
      return activeRunId;
    },
    get dismissedBlockId() {
      return dismissedBlockId;
    },
    set dismissedBlockId(id: string | null) {
      if (forwardWindowCommand({ kind: "set-dismissed-block-id", id })) return;
      setDismissedBlockId(id);
    },
    get autoStartSuppressed() {
      return autoStartSuppressed;
    },
    set autoStartSuppressed(value: boolean) {
      autoStartSuppressed = value;
    },
    get breakOvertimeSeconds() {
      return breakOvertimeSeconds;
    },
    get segments() {
      return segments;
    },
    get segmentVersion() {
      return segmentVersion;
    },
    get blockExpired() {
      return blockExpired;
    },
    clearBlockExpired() {
      if (forwardWindowCommand({ kind: "clear-block-expired" })) return;
      clearBlockExpiredInternal();
    },
    /** Transfer session to a new block ID without resetting state (after detachInstance creates a standalone). */
    async transferBlockId(newBlockId: string, newEndTime?: string) {
      if (forwardWindowCommand({ kind: "transfer-block-id", newBlockId, newEndTime })) return;
      await transferBlockIdInternal(newBlockId, newEndTime);
    },
    adoptTransferredBlockId(newBlockId: string, newEndTime?: string) {
      adoptTransferredBlockIdInternal(newBlockId, newEndTime);
    },
    get suspendedAway() {
      return suspendedAway;
    },
    async dismissSuspend(resume: boolean) {
      if (forwardWindowCommand({ kind: "dismiss-suspend", resume })) return;
      await dismissSuspend(resume);
    },
    get idlePaused() {
      return idlePaused;
    },
    async dismissIdle(resume: boolean) {
      if (forwardWindowCommand({ kind: "dismiss-idle", resume })) return;
      await idleController.dismiss(resume);
    },
    async markIdleFocusFailed(failedAtMs: number | null = null) {
      if (forwardWindowCommand({
        kind: "mark-idle-focus-failed",
        failedAtMs: failedAtMs ?? undefined,
      })) return;
      await idleController.markFocusFailed(failedAtMs);
    },
    async startFromBlock(
      blockId: string,
      blockConfig: PomodoroConfig,
      eventEnd?: string,
      eventDate?: string,
      blockIdleTimeoutMinutes?: number | null,
      syncIdleTimeoutOnExistingBlock?: boolean,
      adaptivePlannedBlocks?: PomodoroAdaptivePlannedBlockWrite[],
    ) {
      if (forwardWindowCommand({
        kind: "start-from-block",
        blockId,
        blockConfig,
        eventEnd,
        eventDate,
        blockIdleTimeoutMinutes,
        syncIdleTimeoutOnExistingBlock,
        adaptivePlannedBlocks,
      })) return;
      await startFromBlockInternal(
        blockId,
        blockConfig,
        eventEnd,
        eventDate,
        blockIdleTimeoutMinutes,
        syncIdleTimeoutOnExistingBlock,
        adaptivePlannedBlocks,
      );
    },
    setActiveIdleThresholdMinutes(minutes: number) {
      if (forwardWindowCommand({ kind: "set-active-idle-threshold-minutes", minutes })) return;
      idleController.setActiveThresholdMinutes(minutes);
    },
    async stopSession() {
      if (forwardWindowCommand({ kind: "stop-session" })) return;
      await stopSessionInternal();
    },
    async completeActiveBlockAt(endIso: string) {
      if (forwardWindowCommand({ kind: "complete-active-block-at", endIso })) return;
      await completeActiveBlockAtInternal(endIso);
    },
    pause() {
      if (forwardWindowCommand({ kind: "pause" })) return;
      pauseSession();
    },
    start() {
      if (forwardWindowCommand({ kind: "start" })) return;
      resumeSession();
    },
    skip() {
      if (forwardWindowCommand({ kind: "skip" })) return;
      skipSession();
    },
    addFocusTime(seconds: number = FOCUS_EXTENSION_SECONDS) {
      if (forwardWindowCommand({ kind: "add-focus-time", seconds })) return;
      addFocusTimeInternal(seconds);
    },
    /** Clean up orphaned segments from previous app sessions. Call once on startup. */
    async cleanupOrphans() {
      if (forwardWindowCommand({ kind: "cleanup-orphans" })) return;
      await cleanupOrphansInternal();
    },
  };
}
