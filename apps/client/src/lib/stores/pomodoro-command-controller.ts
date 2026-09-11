import type { PomodoroAdaptivePlannedBlockWrite } from "$lib/pomodoro/adaptive/persistence";
import type { PomodoroRunEventWrite } from "./pomodoro-backend-writes";
import type { PomodoroConfig } from "./pomodoro-machine";
import type { PomodoroNativeEventListener } from "./pomodoro-runtime-environment-contract";
import type { PomodoroRuntime } from "./pomodoro-runtime";
import type { PomodoroWindowCommand } from "./pomodoro-window-sync";

const PAUSED_FOCUS_NOTIFICATION_RESUME_EVENT = "pomodoro-paused-focus-resume";
const PAUSED_FOCUS_NOTIFICATION_STOP_ASKING_EVENT =
  "pomodoro-paused-focus-stop-asking";

interface PomodoroCommandContext {
  runtime: PomodoroRuntime;
  isCoordinator(): boolean;
  publishWindowSnapshot(): void;
  setDismissedBlockId(id: string | null): void;
  clearBlockExpired(): void;
  transferBlockId(newBlockId: string, newEndTime?: string): Promise<void>;
  startFromBlock(
    blockId: string,
    blockConfig: PomodoroConfig,
    eventTitle?: string | null,
    eventEnd?: string,
    eventDate?: string,
    blockIdleTimeoutMinutes?: number | null,
    syncIdleTimeoutOnExistingBlock?: boolean,
    adaptivePlannedBlocks?: readonly PomodoroAdaptivePlannedBlockWrite[],
  ): Promise<void>;
  setActiveIdleThresholdMinutes(minutes: number): void;
  dismissSuspend(resume: boolean): Promise<void>;
  dismissIdle(resume: boolean): Promise<void>;
  markIdleFocusFailed(failedAtMs: number | null): Promise<void>;
  completeActiveBlockAt(endIso: string): Promise<void>;
  stopSession(): Promise<void>;
  pause(): void;
  resume(): void;
  skip(): void;
  addFocusTime(seconds: number): void;
  addBreakTime(seconds: number): void;
  cleanupOrphans(): Promise<void>;
  canPauseResume(): boolean;
  advancePhase(): Promise<void>;
  updateTray(): void;
  recordManualPhaseAdvanceEvent(occurredAt: string): void;
  recordRunEvent(input: PomodoroRunEventWrite, errorPrefix: string): void;
  activeSegmentId(): string | null;
  markCurrentSegmentSkipped(occurredAt: string): Promise<void>;
  startFocusSession(): Promise<void>;
  completeCurrentBreak(endIso: string): Promise<void>;
  cappedActiveBreakEndIso(): string;
  pausedFocusPulseActive(): boolean;
  suppressPausedFocusNotifications(): void;
  nowIso(): string;
  nativeEventListener: PomodoroNativeEventListener | null;
}

export interface PomodoroCommandController {
  handleWindowCommand(command: PomodoroWindowCommand): void;
  initListeners(): void;
}

/** Owns idempotent native event listeners and window command dispatch. */
export function createPomodoroCommandController(
  context: PomodoroCommandContext,
): PomodoroCommandController {
  let initialized = false;

  function handleWindowCommand(command: PomodoroWindowCommand): void {
    if (!context.isCoordinator()) return;
    switch (command.kind) {
      case "request-snapshot":
        context.publishWindowSnapshot();
        return;
      case "set-dismissed-block-id":
        context.setDismissedBlockId(command.id);
        return;
      case "clear-block-expired":
        context.clearBlockExpired();
        return;
      case "transfer-block-id":
        void context.transferBlockId(command.newBlockId, command.newEndTime);
        return;
      case "start-from-block":
        void context.startFromBlock(
          command.blockId,
          command.blockConfig,
          command.eventTitle,
          command.eventEnd,
          command.eventDate,
          command.blockIdleTimeoutMinutes,
          command.syncIdleTimeoutOnExistingBlock,
          command.adaptivePlannedBlocks,
        );
        return;
      case "set-active-idle-threshold-minutes":
        context.setActiveIdleThresholdMinutes(command.minutes);
        return;
      case "dismiss-suspend":
        void context.dismissSuspend(command.resume);
        return;
      case "dismiss-idle":
        void context.dismissIdle(command.resume);
        return;
      case "mark-idle-focus-failed":
        void context.markIdleFocusFailed(command.failedAtMs ?? null);
        return;
      case "complete-active-block-at":
        void context.completeActiveBlockAt(command.endIso);
        return;
      case "stop-session":
        void context.stopSession();
        return;
      case "pause":
        context.pause();
        return;
      case "start":
        context.resume();
        return;
      case "skip":
        context.skip();
        return;
      case "add-focus-time":
        context.addFocusTime(command.seconds);
        return;
      case "cleanup-orphans":
        void context.cleanupOrphans();
    }
  }

  function warnListenerFailure(label: string) {
    return (error: unknown): void => console.warn(label, error);
  }

  function initListeners(): void {
    if (!context.isCoordinator() || initialized || !context.nativeEventListener) return;
    initialized = true;
    const listenToEvent = context.nativeEventListener;

    listenToEvent("pomodoro-skip-break", () => {
      document.dispatchEvent(new Event("ganbaru-ai-clear-snap"));
      if (context.runtime.phase === "short_break" || context.runtime.phase === "long_break") {
        const occurredAt = context.nowIso();
        context.recordManualPhaseAdvanceEvent(occurredAt);
        if (context.runtime.activeRunId) {
          context.recordRunEvent({
            runId: context.runtime.activeRunId,
            segmentId: context.activeSegmentId(),
            eventType: "skip_break",
            occurredAt,
            phase: context.runtime.phase,
            reason: "skipped_by_user",
            durationSeconds: null,
          }, "Failed to record skipped break:");
        }
        void context.markCurrentSegmentSkipped(occurredAt)
          .then(context.startFocusSession);
      } else {
        context.runtime.skipNextBreak = true;
      }
    }).catch(warnListenerFailure("Failed to listen for pomodoro-skip-break:"));

    listenToEvent("pomodoro-break-acknowledged", () => {
      document.dispatchEvent(new Event("ganbaru-ai-clear-snap"));
      if (context.runtime.phase === "short_break" || context.runtime.phase === "long_break") {
        void context.completeCurrentBreak(context.cappedActiveBreakEndIso())
          .then(context.startFocusSession);
      }
    }).catch(warnListenerFailure("Failed to listen for pomodoro-break-acknowledged:"));

    listenToEvent<{ seconds: number }>("pomodoro-break-extended", (event) => {
      context.addBreakTime(event.payload.seconds);
    }).catch(warnListenerFailure("Failed to listen for pomodoro-break-extended:"));

    listenToEvent("idle-overlay-resume", () => {
      if (context.runtime.idlePaused) void context.dismissIdle(true);
    }).catch(warnListenerFailure("Failed to listen for idle-overlay-resume:"));

    listenToEvent<{ failedAtMs?: number }>("idle-overlay-focus-failed", (event) => {
      if (!context.runtime.idlePaused) return;
      const failedAtMs = event.payload.failedAtMs;
      void context.markIdleFocusFailed(
        typeof failedAtMs === "number" && Number.isFinite(failedAtMs)
          ? failedAtMs
          : null,
      );
    }).catch(warnListenerFailure("Failed to listen for idle-overlay-focus-failed:"));

    listenToEvent("tray-pause-resume", () => {
      if (!context.canPauseResume()) return;
      if (context.runtime.isRunning) context.pause();
      else context.resume();
    }).catch(warnListenerFailure("Failed to listen for tray-pause-resume:"));

    listenToEvent("tray-skip", () => {
      if (context.runtime.suspendedAway || context.runtime.idlePaused) return;
      void context.advancePhase();
      context.updateTray();
    }).catch(warnListenerFailure("Failed to listen for tray-skip:"));

    listenToEvent<{ seconds: number }>("pomodoro-add-time", (event) => {
      context.addFocusTime(event.payload.seconds);
    }).catch(warnListenerFailure("Failed to listen for pomodoro-add-time:"));

    listenToEvent(PAUSED_FOCUS_NOTIFICATION_RESUME_EVENT, () => {
      if (
        context.runtime.suspendedAway
        || context.runtime.idlePaused
        || context.runtime.isRunning
        || !context.pausedFocusPulseActive()
      ) return;
      context.resume();
    }).catch(warnListenerFailure(
      "Failed to listen for paused focus notification resume:",
    ));

    listenToEvent(PAUSED_FOCUS_NOTIFICATION_STOP_ASKING_EVENT, () => {
      if (!context.pausedFocusPulseActive()) return;
      context.suppressPausedFocusNotifications();
    }).catch(warnListenerFailure(
      "Failed to listen for paused focus notification stop asking:",
    ));
  }

  return { handleWindowCommand, initListeners };
}
