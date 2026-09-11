import type { PomodoroAdaptivePlannedBlockWrite } from "$lib/pomodoro/adaptive/persistence";
import {
  clonePomodoroConfig,
  focusDurationMinutesAtPosition,
} from "$lib/pomodoro/rhythm";
import type { PersistedSegment } from "$lib/components/calendar/types";
import {
  decideStartFromBlock,
  TIME_MULTIPLIER,
  type PomodoroConfig,
} from "./pomodoro-machine";
import type { PomodoroRuntime } from "./pomodoro-runtime";

interface ActiveBlockSegmentCapabilities {
  activeSegment(): PersistedSegment | null;
  eventDateFromBlockId(blockId: string): string | null;
  applyActiveBlockWindowChange(
    blockId: string,
    endMs: number,
    eventDate?: string,
  ): void;
  createSegments(
    blockId: string,
    eventEnd: string,
    eventDate: string,
    adaptivePlannedBlocks: readonly PomodoroAdaptivePlannedBlockWrite[],
  ): Promise<void>;
}

interface PomodoroActiveBlockContext {
  runtime: PomodoroRuntime;
  segments: ActiveBlockSegmentCapabilities;
  transferActiveEventReference(input: {
    newEventId: string;
    newEventDate: string | null;
    plannedEnd: string | null;
  }): Promise<void>;
  setActiveTimeoutMs(timeoutMs: number | null): void;
  startIdleChecking(): void;
  clearBreakEndWarning(): void;
  clearMusicPausedByPomodoro(): void;
  stopPausedOpportunityCountdown(): void;
  stopVisualTick(): void;
  startVisualTick(): void;
  initListeners(): void;
  setPhaseRemainingSeconds(seconds: number): number;
  resetFocusNotificationState(): void;
  updateTray(): void;
  publishWindowSnapshot(): void;
  hasOvertime(): boolean;
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
  nowMs?(): number;
  nowIso?(): string;
}

export interface PomodoroActiveBlockController {
  setDismissedBlockId(id: string | null): void;
  clearBlockExpired(): void;
  transferBlockId(newBlockId: string, newEndTime?: string): Promise<void>;
  adoptTransferredBlockId(
    newBlockId: string,
    newEndTime?: string,
    eventDate?: string | null,
  ): void;
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
}

/** Owns active calendar block adoption, transfer, and session entry. */
export function createPomodoroActiveBlockController(
  context: PomodoroActiveBlockContext,
): PomodoroActiveBlockController {
  const runtime = context.runtime;
  const currentMs = context.nowMs ?? Date.now;
  const currentIso = context.nowIso ?? (() => new Date().toISOString());

  function setDismissedBlockId(id: string | null): void {
    runtime.dismissedBlockId = id;
    context.publishWindowSnapshot();
  }

  function clearBlockExpired(): void {
    runtime.blockExpired = false;
    context.publishWindowSnapshot();
  }

  async function transferBlockId(
    newBlockId: string,
    newEndTime?: string,
  ): Promise<void> {
    if (!runtime.activeBlockId) return;
    const newEndMs = newEndTime
      ? new Date(newEndTime.replace(" ", "T")).getTime()
      : null;
    const plannedEnd = newEndMs !== null && Number.isFinite(newEndMs)
      ? new Date(newEndMs).toISOString()
      : null;
    const newEventDate = context.segments.eventDateFromBlockId(newBlockId)
      ?? context.segments.activeSegment()?.eventDate
      ?? null;
    await context.transferActiveEventReference({
      newEventId: newBlockId,
      newEventDate,
      plannedEnd,
    });
    adoptTransferredBlockId(newBlockId, newEndTime, newEventDate);
  }

  function adoptTransferredBlockId(
    newBlockId: string,
    newEndTime?: string,
    eventDate?: string | null,
  ): void {
    if (!runtime.activeBlockId) return;
    const newEndMs = newEndTime
      ? new Date(newEndTime.replace(" ", "T")).getTime()
      : null;
    const newEventDate = context.segments.eventDateFromBlockId(newBlockId)
      ?? eventDate
      ?? context.segments.activeSegment()?.eventDate
      ?? null;
    runtime.activeBlockId = newBlockId;
    for (const segment of runtime.segments) {
      segment.eventId = newBlockId;
      if (newEventDate) segment.eventDate = newEventDate;
    }
    if (newEndMs !== null && Number.isFinite(newEndMs)) {
      context.segments.applyActiveBlockWindowChange(
        newBlockId,
        newEndMs,
        newEventDate ?? undefined,
      );
      return;
    }
    context.publishWindowSnapshot();
  }

  async function startFromBlock(
    blockId: string,
    blockConfig: PomodoroConfig,
    eventTitle?: string | null,
    eventEnd?: string,
    eventDate?: string,
    blockIdleTimeoutMinutes?: number | null,
    syncIdleTimeoutOnExistingBlock = true,
    adaptivePlannedBlocks: readonly PomodoroAdaptivePlannedBlockWrite[] = [],
  ): Promise<void> {
    if (eventTitle !== undefined) {
      const normalizedTitle = eventTitle?.trim() ?? "";
      runtime.activeBlockTitle = normalizedTitle.length > 0 ? normalizedTitle : null;
    }
    const newConfig = clonePomodoroConfig(blockConfig);
    const idleMinutes = blockIdleTimeoutMinutes ?? newConfig.idleTimeoutMinutes;
    const newIdleMs = idleMinutes !== null && idleMinutes > 0
      ? idleMinutes * 60_000
      : null;
    const incomingEndMs = eventEnd && eventDate
      ? new Date(eventEnd.replace(" ", "T")).getTime()
      : null;
    const decision = decideStartFromBlock({
      currentBlockId: runtime.activeBlockId,
      incomingBlockId: blockId,
      incomingConfig: newConfig,
      currentConfig: runtime.config,
      currentEndMs: runtime.activeBlockEndMs,
      incomingEndMs,
      hasOvertimeInterval: context.hasOvertime(),
    });
    const syncIdleTimeout = syncIdleTimeoutOnExistingBlock
      || decision.kind === "new_session"
      || decision.kind === "transition"
      || decision.kind === "reconfigure";
    if (syncIdleTimeout) context.setActiveTimeoutMs(newIdleMs);

    switch (decision.kind) {
      case "noop":
        return;
      case "update_end_only":
      case "rebuild_segments":
        context.segments.applyActiveBlockWindowChange(
          blockId,
          decision.newEndMs,
          eventDate,
        );
        return;
      case "reconfigure":
        await context.reconfigureSession(
          blockId,
          decision.newConfig,
          eventEnd!,
          eventDate!,
        );
        return;
      case "transition":
        await context.transitionToBlock(
          blockId,
          decision.newConfig,
          eventEnd!,
          eventDate!,
        );
        return;
      case "new_session": {
        context.clearBreakEndWarning();
        context.clearMusicPausedByPomodoro();
        context.stopPausedOpportunityCountdown();
        context.initListeners();
        context.stopVisualTick();
        runtime.activeBlockId = blockId;
        runtime.activeBlockEndMs = decision.newEndMs;
        runtime.config = clonePomodoroConfig(decision.newConfig);
        runtime.phase = "focus";
        runtime.currentRhythmPosition = 1;
        context.setPhaseRemainingSeconds(
          focusDurationMinutesAtPosition(
            runtime.config,
            runtime.currentRhythmPosition,
          ) * TIME_MULTIPLIER,
        );
        runtime.completedPomodoros = 0;
        runtime.skipNextBreak = false;
        context.resetFocusNotificationState();
        runtime.isRunning = false;
        const nowMs = currentMs();
        runtime.phaseEndTime = nowMs + runtime.remainingSeconds * 1000;
        runtime.sessionStartTime = currentIso();
        try {
          if (eventEnd && eventDate) {
            await context.segments.createSegments(
              blockId,
              eventEnd,
              eventDate,
              adaptivePlannedBlocks,
            );
          }
        } catch (error) {
          runtime.activeBlockId = null;
          runtime.activeBlockTitle = null;
          runtime.activeBlockEndMs = null;
          runtime.activeRunId = null;
          runtime.phaseEndTime = null;
          runtime.sessionStartTime = null;
          runtime.lastTickMs = null;
          context.setActiveTimeoutMs(null);
          context.publishWindowSnapshot();
          throw error;
        }
        runtime.isRunning = true;
        context.startVisualTick();
        runtime.lastTickMs = currentMs();
        context.startIdleChecking();
        context.updateTray();
      }
    }
  }

  return {
    setDismissedBlockId,
    clearBlockExpired,
    transferBlockId,
    adoptTransferredBlockId,
    startFromBlock,
  };
}
