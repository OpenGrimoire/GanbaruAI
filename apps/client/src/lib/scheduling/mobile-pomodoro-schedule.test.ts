import { describe, expect, it } from "vitest";
import type { CalendarEvent } from "$lib/components/calendar/types";
import type { Translate } from "$lib/i18n/translator.svelte";
import { createPresetPomodoroConfig } from "$lib/pomodoro/rhythm";
import { buildMobilePomodoroSchedule } from "./mobile-pomodoro-schedule";

const t = ((key: string) => key) as Translate;

function focusEvent(overrides: Partial<CalendarEvent> = {}): CalendarEvent {
  return {
    id: "focus-a",
    title: "Write release notes",
    start: "2026-08-28 10:00",
    end: "2026-08-28 11:00",
    timezone: "America/Monterrey",
    calendarId: "calendar-a",
    pomodoroConfig: createPresetPomodoroConfig("creative"),
    ...overrides,
  };
}

describe("mobile Pomodoro activation schedule", () => {
  it("projects a full deterministic phase plan at the calendar event boundary", () => {
    const [projection] = buildMobilePomodoroSchedule(
      [focusEvent()],
      t,
      new Date(2026, 7, 28, 9, 0).getTime(),
    );

    expect(projection).toMatchObject({
      eventId: "focus-a",
      eventTitle: "Write release notes",
      isRunning: true,
      totalSeconds: 25 * 60,
      remainingSeconds: 25 * 60,
    });
    expect(projection?.phases.map((phase) => phase.phase)).toEqual([
      "focus",
      "short_break",
      "focus",
      "short_break",
    ]);
    expect(projection?.phases[0]?.startsAtEpochMs).toBe(
      new Date(2026, 7, 28, 10, 0).getTime(),
    );
    expect(projection?.phases.at(-1)?.endsAtEpochMs).toBe(
      new Date(2026, 7, 28, 11, 0).getTime(),
    );
  });

  it("keeps an ongoing event for cold-start catch-up and excludes expired events", () => {
    const schedule = buildMobilePomodoroSchedule(
      [
        focusEvent({ id: "ongoing" }),
        focusEvent({ id: "expired", start: "2026-08-28 08:00", end: "2026-08-28 09:00" }),
      ],
      t,
      new Date(2026, 7, 28, 10, 30).getTime(),
    );

    expect(schedule.map((projection) => projection.eventId)).toEqual(["ongoing"]);
  });

  it("ignores cancelled, all-day, and ordinary calendar events", () => {
    expect(buildMobilePomodoroSchedule([
      focusEvent({ id: "cancelled", status: "cancelled" }),
      focusEvent({ id: "all-day", allDay: true }),
      focusEvent({ id: "ordinary", pomodoroConfig: undefined }),
    ], t, new Date(2026, 7, 28, 9, 0).getTime())).toEqual([]);
  });
});
