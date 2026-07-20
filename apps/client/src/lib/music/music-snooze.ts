import { Temporal } from "@js-temporal/polyfill";

export type MusicSnoozeDuration = "today" | "day" | "week" | "month" | "custom" | "until-resumed";

/** Resolves a user-facing Snooze duration to one stable instant. */
export function musicSnoozeEndsAt(
  duration: MusicSnoozeDuration,
  nowMs: number,
  timeZone: string,
  customLocal = "",
): number | null {
  if (duration === "until-resumed") return null;
  const now = Temporal.Instant.fromEpochMilliseconds(nowMs).toZonedDateTimeISO(timeZone);
  if (duration === "today") return now.add({ days: 1 }).startOfDay().epochMilliseconds;
  if (duration === "day") return now.add({ days: 1 }).epochMilliseconds;
  if (duration === "week") return now.add({ weeks: 1 }).epochMilliseconds;
  if (duration === "month") return now.add({ months: 1 }).epochMilliseconds;
  const local = Temporal.PlainDateTime.from(customLocal);
  return local.toZonedDateTime(timeZone, { disambiguation: "compatible" }).epochMilliseconds;
}

export function isValidFutureSnoozeDate(value: string, nowMs: number, timeZone: string): boolean {
  try {
    const end = musicSnoozeEndsAt("custom", nowMs, timeZone, value);
    return end !== null && end > nowMs;
  } catch {
    return false;
  }
}
