import type { CalendarEvent } from "$lib/components/calendar/types";
import { selectFocusOwner } from "$lib/pomodoro/ownership";
import { parseCalendarDate } from "$lib/components/calendar/utils";

export interface SelectActivePomodoroBlockOptions {
  now: Date;
  activeBlockId: string | null;
}

/** Select a commitment without treating eligibility as permission to execute it. */
export function selectActivePomodoroBlock(
  events: readonly CalendarEvent[],
  options: SelectActivePomodoroBlockOptions,
): CalendarEvent | undefined {
  const candidates = events
    .filter((event) => event.pomodoroConfig && !event.allDay && event.status !== "cancelled")
    .map((event) => ({
      id: event.id,
      createdAt: event.createdAt,
      startMs: parseCalendarDate(event.start).getTime(),
      endMs: parseCalendarDate(event.end).getTime(),
      event,
    }));
  return selectFocusOwner(candidates, options.now.getTime(), options.activeBlockId)?.event;
}

/** Returns the next start or end boundary that can change Pomodoro ownership. */
export function nextPomodoroBlockBoundaryMs(
  events: readonly CalendarEvent[],
  nowMs: number,
): number | null {
  let nextBoundaryMs: number | null = null;
  for (const event of events) {
    if (!event.pomodoroConfig || event.allDay || event.status === "cancelled") continue;
    const boundaries = [
      parseCalendarDate(event.start).getTime(),
      parseCalendarDate(event.end).getTime(),
    ];
    for (const boundaryMs of boundaries) {
      if (!Number.isFinite(boundaryMs) || boundaryMs <= nowMs) continue;
      nextBoundaryMs = nextBoundaryMs === null
        ? boundaryMs
        : Math.min(nextBoundaryMs, boundaryMs);
    }
  }
  return nextBoundaryMs;
}
