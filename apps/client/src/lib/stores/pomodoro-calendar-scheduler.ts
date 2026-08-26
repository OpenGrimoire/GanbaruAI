import { Temporal } from "@js-temporal/polyfill";
import type { CalendarEvent } from "$lib/components/calendar/types";
import { parseCalendarDate } from "$lib/components/calendar/utils";
import { buildAdaptivePlannedBlocksForDate } from "$lib/pomodoro/adaptive/planned-blocks";
import {
  createLifecycleScheduler,
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

function nextLocalDayBoundaryMs(nowMs: number): number {
  const next = new Date(nowMs);
  next.setHours(24, 0, 0, 50);
  return next.getTime();
}

/** Coordinates calendar ownership with the shared Pomodoro runtime on every app shell. */
export function createPomodoroCalendarScheduler(
  options: PomodoroCalendarSchedulerOptions,
): PomodoroCalendarScheduler {
  const currentDate = options.now ?? (() => new Date());
  const currentDay = options.today ?? (() => Temporal.Now.plainDateISO());
  const buildPlannedBlocks = options.buildPlannedBlocks ?? buildAdaptivePlannedBlocksForDate;
  let trackedBlock: CalendarEvent | null = null;

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
    trackedBlock = null;
    options.onBeforeNaturalCompletion?.();
    await options.pomodoro.stopSession();
    await options.onNaturalCompletion?.(completedBlock);
  }

  async function run(context: SchedulerRunContext): Promise<number | null> {
    if (options.isBlocked() || options.pomodoro.autoStartSuppressed) return null;

    const { activeBlock, events, nowMs, plannedBlocks } = await findActiveBlock();
    if (!context.isCurrent()) return null;
    const nextDeadlineMs = nextPomodoroBlockBoundaryMs(events, nowMs)
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
      if (options.pomodoro.blockExpired) options.pomodoro.clearBlockExpired();
      const config = activeBlock.pomodoroConfig!;
      await options.pomodoro.startFromBlock(
        activeBlock.id,
        config,
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
