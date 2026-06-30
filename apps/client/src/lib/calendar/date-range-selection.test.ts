import { describe, expect, it } from "vitest";
import {
  normalizedDateRange,
  selectDateRangeEnd,
  selectDateRangeStart,
} from "./date-range-selection";

describe("date range selection", () => {
  it("keeps the end date when a selected start stays before it", () => {
    expect(selectDateRangeStart({
      selectedDate: "2026-06-26",
      startDate: "2026-06-24",
      endDate: "2026-06-30",
    })).toEqual({
      startDate: "2026-06-26",
      endDate: "2026-06-30",
    });
  });

  it("moves the end date when a selected start crosses it", () => {
    expect(selectDateRangeStart({
      selectedDate: "2026-07-01",
      startDate: "2026-06-24",
      endDate: "2026-06-30",
    })).toEqual({
      startDate: "2026-07-01",
      endDate: "2026-07-01",
    });
  });

  it("keeps the start date when a selected end stays after it", () => {
    expect(selectDateRangeEnd({
      selectedDate: "2026-07-01",
      startDate: "2026-06-24",
      endDate: "2026-06-30",
    })).toEqual({
      startDate: "2026-06-24",
      endDate: "2026-07-01",
    });
  });

  it("moves the start date when a selected end crosses it", () => {
    expect(selectDateRangeEnd({
      selectedDate: "2026-06-21",
      startDate: "2026-06-24",
      endDate: "2026-06-30",
    })).toEqual({
      startDate: "2026-06-21",
      endDate: "2026-06-21",
    });
  });

  it("can fill missing endpoints for required calendar ranges", () => {
    expect(selectDateRangeStart({
      selectedDate: "2026-06-24",
      fillMissingEndDate: true,
    })).toEqual({
      startDate: "2026-06-24",
      endDate: "2026-06-24",
    });
    expect(selectDateRangeEnd({
      selectedDate: "2026-06-24",
      fillMissingStartDate: true,
    })).toEqual({
      startDate: "2026-06-24",
      endDate: "2026-06-24",
    });
  });

  it("normalizes display ranges and drops missing or malformed endpoints", () => {
    expect(normalizedDateRange("2026-07-01", "2026-06-30")).toEqual({
      startDate: "2026-06-30",
      endDate: "2026-07-01",
    });
    expect(normalizedDateRange("2026-07-01", undefined)).toBeUndefined();
    expect(normalizedDateRange("2026-7-01", "2026-07-02")).toBeUndefined();
  });
});
