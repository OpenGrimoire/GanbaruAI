import { Temporal } from "@js-temporal/polyfill";
import type { CalendarEvent } from "$lib/components/calendar/types";
import { parseCalendarDate } from "$lib/components/calendar/utils";
import { buildAdaptivePlannedBlocksForDate } from "$lib/pomodoro/adaptive/planned-blocks";
import {
  createLifecycleScheduler,
  systemSchedulerClock,
  type LifecycleScheduler,
  type SchedulerRunContext,
} from "$lib/scheduling/lifecycle-scheduler";
import {
  nextPomodoroBlockBoundaryMs,
  selectActivePomodoroBlock,
} from "$lib/stores/pomodoro-scheduler";

type PomodoroConfig = NonNullable<CalendarEvent["pomodoroConfig"]>;
type AdaptivePlannedBlocks = ReturnType<typeof buildAdaptivePlannedBlocksForDate>;

interface PomodoroCalendarSource {
  loadPomodoroSchedulerEvents(
    start: Temporal.PlainDate,
    end: Temporal.PlainDate,
  ): Promise<CalendarEvent[]>;
}

interface PomodoroCalendarRuntime {
  activeBlockId: string | null;
  dismissedBlockId: string | null;
  readonly blockExpired: boolean;
  readonly autoStartSuppressed: boolean;
  clearBlockExpired(): void;
  startFromBlock(
    blockId: string,
    config: PomodoroConfig,
    eventTitle: string | null,
    eventEnd: string,
    eventDate: string,
    idleTimeoutMinutes: number | null,
    syncIdleTimeoutOnExistingBlock: boolean,
    plannedBlocks: AdaptivePlannedBlocks,
  ): Promise<void>;
  stopSession(): Promise<void>;
}

export interface PomodoroCalendarScheduler {
  setEnabled(enabled: boolean): void;
  invalidate(): void;
  resume(): void;
  dispose(): void;
  isEnabled(): boolean;
  clearTrackedBlock(): void;
}

interface PomodoroCalendarSchedulerOptions {
  calendar: PomodoroCalendarSource;
  pomodoro: PomodoroCalendarRuntime;
  isBlocked: () => boolean;
  /** Fresh local activity may admit desktop execution. Omission requires an explicit start. */
  canStartAutomatically?: (boundaryEpochMs: number) => Promise<boolean>;
  onNaturalCompletion?: (block: CalendarEvent | null) => void | Promise<void>;
  onBeforeNaturalCompletion?: () => void;
  onError?: (error: unknown) => void;
  now?: () => Date;
  today?: () => Temporal.PlainDate;
  buildPlannedBlocks?: (
    events: readonly CalendarEvent[],
    eventDate: string,
  ) => AdaptivePlannedBlocks;
}

const ACTIVITY_RECHECK_MS = 15_000;

function nextLocalDayBoundaryMs(nowMs: number): number {
  const next = new Date(nowMs);
  next.setHours(24, 0, 0, 50);
  return next.getTime();
}

/** Accept the current scheduled commitment through an explicit local user action. */
export async function startScheduledPomodoro(
  calendar: PomodoroCalendarSource,
  pomodoro: PomodoroCalendarRuntime,
  now: () => Date = () => new Date(),
): Promise<boolean> {
  if (pomodoro.activeBlockId || pomodoro.autoStartSuppressed) return false;
  const requestedAt = now();
  const today = Temporal.PlainDate.from({
    year: requestedAt.getFullYear(),
    month: requestedAt.getMonth() + 1,
    day: requestedAt.getDate(),
  });
  const events = await calendar.loadPomodoroSchedulerEvents(
    today.subtract({ days: 1 }), today.add({ days: 1 }),
  );
  if (pomodoro.activeBlockId || pomodoro.autoStartSuppressed) return false;
  const event = selectActivePomodoroBlock(events, { now: now(), activeBlockId: null });
  if (!event?.pomodoroConfig) return false;
  const eventDate = event.start.split(" ")[0];
  await pomodoro.startFromBlock(
    event.id, event.pomodoroConfig, event.title, event.end, eventDate,
    event.pomodoroConfig.idleTimeoutMinutes, true,
    buildAdaptivePlannedBlocksForDate(events, eventDate),
  );
  pomodoro.dismissedBlockId = null;
  return true;
}

/** Coordinates calendar ownership with the shared Pomodoro runtime on every app shell. */
export function createPomodoroCalendarScheduler(
  options: PomodoroCalendarSchedulerOptions,
): PomodoroCalendarScheduler {
  const currentDate = options.now ?? (() => new Date());
  const currentDay = options.today ?? (() => Temporal.Now.plainDateISO());
  const buildPlannedBlocks = options.buildPlannedBlocks ?? buildAdaptivePlannedBlocksForDate;
  let trackedBlock: CalendarEvent | null = null;
  let previousExecutionBoundaryMs = 0;

  async function findActiveBlock(): Promise<{
    activeBlock: CalendarEvent | undefined;
    events: readonly CalendarEvent[];
    nowMs: number;
    plannedBlocks: AdaptivePlannedBlocks;
  }> {
    const now = currentDate();
    const today = currentDay();
    const events = await options.calendar.loadPomodoroSchedulerEvents(
      today.subtract({ days: 1 }),
      today.add({ days: 1 }),
    );
    const activeBlock = selectActivePomodoroBlock(events, {
      now,
      activeBlockId: options.pomodoro.activeBlockId,
    });
    const eventDate = activeBlock?.start.split(" ")[0] ?? null;
    return {
      activeBlock,
      events,
      nowMs: now.getTime(),
      plannedBlocks: eventDate ? buildPlannedBlocks(events, eventDate) : [],
    };
  }

  async function completeTrackedBlock(): Promise<void> {
    const completedBlock = trackedBlock;
    if (completedBlock) {
      previousExecutionBoundaryMs = parseCalendarDate(completedBlock.end).getTime();
    }
    trackedBlock = null;
    options.onBeforeNaturalCompletion?.();
    await options.pomodoro.stopSession();
    await options.onNaturalCompletion?.(completedBlock);
  }

  async function run(context: SchedulerRunContext): Promise<number | null> {
    if (options.isBlocked() || options.pomodoro.autoStartSuppressed) return null;

    const { activeBlock, events, nowMs, plannedBlocks } = await findActiveBlock();
    if (!context.isCurrent()) return null;
    let nextDeadlineMs = nextPomodoroBlockBoundaryMs(events, nowMs)
      ?? nextLocalDayBoundaryMs(nowMs);

    if (
      options.pomodoro.dismissedBlockId
      && activeBlock?.id !== options.pomodoro.dismissedBlockId
    ) {
      options.pomodoro.dismissedBlockId = null;
    }

    if (activeBlock && activeBlock.id === options.pomodoro.dismissedBlockId) {
      return nextDeadlineMs;
    }

    if (activeBlock) {
      if (options.pomodoro.activeBlockId !== activeBlock.id) {
        const boundaryMs = Math.max(
          parseCalendarDate(activeBlock.start).getTime(),
          trackedBlock ? parseCalendarDate(trackedBlock.end).getTime() : 0,
          previousExecutionBoundaryMs,
        );
        const admitted = await options.canStartAutomatically?.(boundaryMs) ?? false;
        if (!context.isCurrent() || options.isBlocked() || options.pomodoro.autoStartSuppressed) {
          return null;
        }
        if (!admitted) {
          if (options.canStartAutomatically) {
            nextDeadlineMs = Math.min(nextDeadlineMs, currentDate().getTime() + ACTIVITY_RECHECK_MS);
          }
          if (trackedBlock && parseCalendarDate(trackedBlock.end).getTime() <= currentDate().getTime()) {
            await completeTrackedBlock();
          }
          return nextDeadlineMs;
        }
        // A local command or a deadline may have changed ownership during the observation.
        const current = selectActivePomodoroBlock(events, {
          now: currentDate(),
          activeBlockId: options.pomodoro.activeBlockId,
        });
        if (current?.id !== activeBlock.id) return currentDate().getTime();
      }
      if (options.pomodoro.blockExpired) options.pomodoro.clearBlockExpired();
      const config = activeBlock.pomodoroConfig!;
      await options.pomodoro.startFromBlock(
        activeBlock.id,
        config,
        activeBlock.title,
        activeBlock.end,
        activeBlock.start.split(" ")[0],
        config.idleTimeoutMinutes,
        false,
        plannedBlocks,
      );
      if (!context.isCurrent()) return null;
      trackedBlock = { ...activeBlock };
    } else if (options.pomodoro.activeBlockId && options.pomodoro.blockExpired) {
      options.pomodoro.clearBlockExpired();
      await completeTrackedBlock();
    } else if (
      options.pomodoro.activeBlockId
      && trackedBlock
      && parseCalendarDate(trackedBlock.end).getTime() <= currentDate().getTime()
    ) {
      await completeTrackedBlock();
    } else if (options.pomodoro.activeBlockId && trackedBlock) {
      return nextDeadlineMs;
    }

    return nextDeadlineMs;
  }

  const scheduler: LifecycleScheduler = createLifecycleScheduler({
    clock: { ...systemSchedulerClock, now: () => currentDate().getTime() },
    run,
    errorRetryMs: 60_000,
    onError: options.onError,
  });

  return {
    setEnabled: scheduler.setEnabled,
    invalidate: scheduler.invalidate,
    resume: scheduler.resume,
    dispose: scheduler.dispose,
    isEnabled: scheduler.isEnabled,
    clearTrackedBlock: () => {
      trackedBlock = null;
    },
  };
}
