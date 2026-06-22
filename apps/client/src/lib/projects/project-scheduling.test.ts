import { Temporal } from "@js-temporal/polyfill";
import { describe, expect, it } from "vitest";
import {
  formatProjectScheduleWindowStart,
  projectDefaultScheduleStart,
  projectEventDurationMinutes,
  projectScheduleWindowFor,
} from "./project-scheduling";

describe("project scheduling helpers", () => {
  it("rounds the default start to the next quarter hour", () => {
    const start = projectDefaultScheduleStart(Temporal.PlainDateTime.from("2026-06-21T10:07"));

    expect(start).toEqual({
      date: "2026-06-21",
      time: "10:15",
    });
  });

  it("rolls the default start into the next day when rounding passes midnight", () => {
    const start = projectDefaultScheduleStart(Temporal.PlainDateTime.from("2026-06-21T23:59"));

    expect(start).toEqual({
      date: "2026-06-22",
      time: "00:00",
    });
  });

  it("formats schedule windows with date rollover", () => {
    const window = formatProjectScheduleWindowStart(
      Temporal.PlainDateTime.from("2026-06-21T23:30"),
      90,
    );

    expect(window).toEqual({
      start: "2026-06-21 23:30",
      end: "2026-06-22 01:00",
    });
  });

  it("rejects invalid schedule windows", () => {
    expect(projectScheduleWindowFor("", "10:00", 30)).toBeNull();
    expect(projectScheduleWindowFor("2026-06-21", "", 30)).toBeNull();
    expect(projectScheduleWindowFor("2026-06-21", "10:00", 0)).toBeNull();
    expect(projectScheduleWindowFor("not-a-date", "10:00", 30)).toBeNull();
    expect(projectScheduleWindowFor("2026-06-21", "not-a-time", 30)).toBeNull();
  });

  it("computes event duration safely", () => {
    expect(projectEventDurationMinutes({
      start: "2026-06-21 09:15",
      end: "2026-06-21 10:45",
    })).toBe(90);
    expect(projectEventDurationMinutes({
      start: "bad",
      end: "2026-06-21 10:45",
    })).toBe(0);
  });
});
