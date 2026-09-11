import { describe, expect, it } from "vitest";
import type { CalendarEvent, PomodoroConfig } from "$lib/components/calendar/types";
import { createPresetPomodoroConfig } from "$lib/pomodoro/rhythm";
import {
  nextPomodoroBlockBoundaryMs,
  selectActivePomodoroBlock,
} from "./pomodoro-scheduler";

const config: PomodoroConfig = createPresetPomodoroConfig("adaptive");

function event(overrides: Partial<CalendarEvent> & Pick<CalendarEvent, "id" | "start" | "end">): CalendarEvent {
  return {
    title: overrides.id,
    timezone: "America/Monterrey",
    calendarId: "local",
    pomodoroConfig: config,
    ...overrides,
  };
}

describe("selectActivePomodoroBlock", () => {
  const now = new Date(2026, 4, 25, 10, 15);

  it("returns undefined when no pomodoro event overlaps now", () => {
    expect(
      selectActivePomodoroBlock(
        [
          event({ id: "past", start: "2026-05-25 09:00", end: "2026-05-25 10:00" }),
          event({ id: "plain", start: "2026-05-25 10:00", end: "2026-05-25 11:00", pomodoroConfig: undefined }),
        ],
        { now, activeBlockId: null },
      ),
    ).toBeUndefined();
  });

  it("treats second-precision event ends as expired immediately after the cut second", () => {
    expect(
      selectActivePomodoroBlock(
        [
          event({ id: "cut", start: "2026-05-25 10:00", end: "2026-05-25 10:15:30" }),
        ],
        { now: new Date(2026, 4, 25, 10, 15, 31), activeBlockId: null },
      ),
    ).toBeUndefined();
  });

  it("keeps the current active block when it is still a candidate", () => {
    const long = event({ id: "long", start: "2026-05-25 09:00", end: "2026-05-25 12:00" });
    const short = event({ id: "short", start: "2026-05-25 10:00", end: "2026-05-25 10:30" });

    expect(
      selectActivePomodoroBlock([short, long], { now, activeBlockId: "long" }),
    ).toBe(long);
  });

  it("picks the candidate with the shortest remaining duration", () => {
    const long = event({ id: "long", start: "2026-05-25 09:00", end: "2026-05-25 12:00" });
    const short = event({ id: "short", start: "2026-05-25 10:00", end: "2026-05-25 10:30" });

    expect(
      selectActivePomodoroBlock([long, short], { now, activeBlockId: null }),
    ).toBe(short);
  });

  it("uses creation time as the deterministic tie-breaker", () => {
    const newer = event({
      id: "newer",
      start: "2026-05-25 10:00",
      end: "2026-05-25 11:00",
      createdAt: "2026-05-20T10:00:00Z",
    });
    const older = event({
      id: "older",
      start: "2026-05-25 10:00",
      end: "2026-05-25 11:00",
      createdAt: "2026-05-19T10:00:00Z",
    });

    expect(
      selectActivePomodoroBlock([newer, older], { now, activeBlockId: null }),
    ).toBe(older);
  });

  it("orders occurrence identities consistently without locale collation", () => {
    const upper = event({ id: "Z", start: "2026-05-25 10:00", end: "2026-05-25 11:00" });
    const lower = event({ id: "a", start: "2026-05-25 10:00", end: "2026-05-25 11:00" });
    for (const events of [[upper, lower], [lower, upper]]) {
      expect(selectActivePomodoroBlock(events, { now, activeBlockId: null })).toBe(upper);
    }
  });

  it("excludes cancelled and all-day commitments even when they were the current owner", () => {
    const cancelled = event({ id: "cancelled", start: "2026-05-25 10:00", end: "2026-05-25 11:00", status: "cancelled" });
    const allDay = event({ id: "all-day", start: "2026-05-25 00:00", end: "2026-05-26 00:00", allDay: true });
    expect(selectActivePomodoroBlock([cancelled, allDay], { now, activeBlockId: cancelled.id })).toBeUndefined();
    expect(nextPomodoroBlockBoundaryMs([cancelled, allDay], now.getTime())).toBeNull();
  });

  it("returns the next Pomodoro start or end boundary without polling", () => {
    const active = event({
      id: "active",
      start: "2026-05-25 10:00",
      end: "2026-05-25 10:30",
    });
    const future = event({
      id: "future",
      start: "2026-05-25 10:20",
      end: "2026-05-25 11:00",
    });

    expect(nextPomodoroBlockBoundaryMs([active, future], now.getTime())).toBe(
      new Date(2026, 4, 25, 10, 20).getTime(),
    );
    expect(nextPomodoroBlockBoundaryMs([active], new Date(2026, 4, 25, 10, 30).getTime()))
      .toBeNull();
  });
});
