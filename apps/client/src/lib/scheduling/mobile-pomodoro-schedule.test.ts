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

describe("mobile focus commitment reminders", () => {
  it("schedules a reminder without run or phase state", () => {
    const [projection] = buildMobilePomodoroSchedule(
      [focusEvent()],
      t,
      new Date(2026, 7, 28, 9, 0).getTime(),
    );

    expect(projection).toEqual({
      id: expect.stringMatching(/^reminder-/),
      eventId: "focus-a",
      title: "Write release notes",
      body: "pomodoroNotification.commitmentDueText",
      channelName: "pomodoroNotification.alertsChannelName",
      channelDescription: "pomodoroNotification.alertsChannelDescription",
      startsAtEpochMs: new Date(2026, 7, 28, 10, 0).getTime(),
      endsAtEpochMs: new Date(2026, 7, 28, 11, 0).getTime(),
    });
  });

  it("keeps an ongoing commitment for a due reminder and excludes expired events", () => {
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

it("keeps a reminder identity through end edits and deduplicates repeated occurrences", () => {
  const now = new Date(2026, 7, 28, 9, 0).getTime();
  const original = focusEvent();
  const changed = focusEvent({ end: "2026-08-28 12:00" });
  expect(buildMobilePomodoroSchedule([original], t, now)[0]?.id)
    .toBe(buildMobilePomodoroSchedule([changed], t, now)[0]?.id);
  expect(buildMobilePomodoroSchedule([original, original], t, now)).toHaveLength(1);
  expect(buildMobilePomodoroSchedule([original], t, Number.NaN)).toEqual([]);
});
