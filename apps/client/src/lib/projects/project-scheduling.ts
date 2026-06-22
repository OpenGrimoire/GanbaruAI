import { Temporal } from "@js-temporal/polyfill";
import type { CalendarEvent } from "$lib/components/calendar/types";

export interface ProjectScheduleStart {
  date: string;
  time: string;
}

export interface ProjectScheduleWindow {
  start: string;
  end: string;
}

export function projectDefaultScheduleStart(
  now: Temporal.PlainDateTime = Temporal.Now.plainDateTimeISO(),
  stepMinutes = 15,
): ProjectScheduleStart {
  const safeStep = Number.isFinite(stepMinutes) && stepMinutes > 0 ? Math.round(stepMinutes) : 15;
  const dayStart = now.with({
    hour: 0,
    minute: 0,
    second: 0,
    millisecond: 0,
    microsecond: 0,
    nanosecond: 0,
  });
  const totalMinutes = now.hour * 60 + now.minute;
  const roundedMinutes = Math.ceil(totalMinutes / safeStep) * safeStep;
  const start = dayStart.add({ minutes: roundedMinutes });
  return {
    date: start.toPlainDate().toString(),
    time: projectScheduleTimeLabel(start),
  };
}

export function projectScheduleTimeLabel(value: Temporal.PlainDateTime): string {
  return `${String(value.hour).padStart(2, "0")}:${String(value.minute).padStart(2, "0")}`;
}

export function formatProjectScheduleWindowStart(
  start: Temporal.PlainDateTime,
  durationMinutes: number,
): ProjectScheduleWindow {
  const end = start.add({ minutes: durationMinutes });
  return {
    start: `${start.toPlainDate().toString()} ${projectScheduleTimeLabel(start)}`,
    end: `${end.toPlainDate().toString()} ${projectScheduleTimeLabel(end)}`,
  };
}

export function projectScheduleWindowFor(
  date: string,
  startTime: string,
  durationMinutes: number,
): ProjectScheduleWindow | null {
  const duration = Math.round(Number(durationMinutes));
  if (duration <= 0 || !date || !startTime) return null;
  try {
    return formatProjectScheduleWindowStart(Temporal.PlainDateTime.from(`${date}T${startTime}`), duration);
  } catch {
    return null;
  }
}

export function projectEventDurationMinutes(event: Pick<CalendarEvent, "start" | "end">): number {
  try {
    const start = Temporal.PlainDateTime.from(event.start.replace(" ", "T"));
    const end = Temporal.PlainDateTime.from(event.end.replace(" ", "T"));
    return Math.max(0, Math.round(start.until(end).total({ unit: "minutes" })));
  } catch {
    return 0;
  }
}
