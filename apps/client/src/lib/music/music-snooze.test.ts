import { describe, expect, it } from "vitest";
import { Temporal } from "@js-temporal/polyfill";
import { isValidFutureSnoozeDate, musicSnoozeEndsAt } from "./music-snooze";

describe("music Snooze expiry", () => {
  it("uses the next local midnight for the rest of today across DST", () => {
    const now = Temporal.ZonedDateTime.from("2026-03-07T23:30:00-05:00[America/New_York]");
    const end = musicSnoozeEndsAt("today", now.epochMilliseconds, "America/New_York");
    expect(Temporal.Instant.fromEpochMilliseconds(end!).toZonedDateTimeISO("America/New_York").toString())
      .toBe("2026-03-08T00:00:00-05:00[America/New_York]");
  });

  it("adds calendar months without turning them into a fixed number of days", () => {
    const now = Temporal.ZonedDateTime.from("2026-01-31T10:00:00-06:00[America/Monterrey]");
    const end = musicSnoozeEndsAt("month", now.epochMilliseconds, "America/Monterrey");
    expect(Temporal.Instant.fromEpochMilliseconds(end!).toZonedDateTimeISO("America/Monterrey").toPlainDateTime().toString())
      .toBe("2026-02-28T10:00:00");
  });

  it("validates custom local date-times against the current instant", () => {
    const now = Temporal.Instant.from("2026-07-15T12:00:00Z").epochMilliseconds;
    expect(isValidFutureSnoozeDate("2026-07-15T08:00", now, "America/New_York")).toBe(false);
    expect(isValidFutureSnoozeDate("2026-07-16T08:00", now, "America/New_York")).toBe(true);
    expect(isValidFutureSnoozeDate("not-a-date", now, "America/New_York")).toBe(false);
  });

  it("keeps until-resumed open ended", () => {
    expect(musicSnoozeEndsAt("until-resumed", 1, "UTC")).toBeNull();
  });
});
