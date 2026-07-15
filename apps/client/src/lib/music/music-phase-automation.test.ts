import { describe, expect, it, vi } from "vitest";
import type { CalendarEvent } from "$lib/components/calendar/types";
import {
  MusicPhaseAutomationPlanner,
  nextMusicEventBoundaryMs,
  selectActiveMusicEvent,
} from "./music-phase-automation";

function event(id: string, start: string, end: string, extra: Partial<CalendarEvent> = {}): CalendarEvent {
  return { id, title: id, start, end, timezone: "UTC", calendarId: "calendar", ...extra };
}

describe("MusicPhaseAutomationPlanner", () => {
  it("catches up once at launch and runs once at every later phase boundary", () => {
    const planner = new MusicPhaseAutomationPlanner();
    const focus = { eventId: "event-1", phase: "focus" as const, phaseToken: "segment-1", suspended: false };
    expect(planner.observe(focus)).toMatchObject({ activation: "catch-up", phase: "focus" });
    expect(planner.observe(focus)).toBeNull();
    expect(planner.observe({ ...focus, phase: "short-break", phaseToken: "segment-2" })).toMatchObject({
      activation: "boundary",
      phase: "short-break",
    });
    expect(planner.observe({ ...focus, phase: "long-break", phaseToken: "segment-3" })).toMatchObject({
      activation: "boundary",
      phase: "long-break",
    });
    expect(planner.observe({ ...focus, phaseToken: "segment-4" })).toMatchObject({
      activation: "boundary",
      phase: "focus",
    });
  });

  it("does not re-enforce an action across pause and resume observations", () => {
    const planner = new MusicPhaseAutomationPlanner();
    const observation = { eventId: "event-1", phase: "focus" as const, phaseToken: "segment-1", suspended: false };
    const activation = planner.observe(observation);
    expect(activation).not.toBeNull();
    expect(planner.observe(observation)).toBeNull();
    expect(planner.observe(observation)).toBeNull();
  });

  it("defers a suspended transition and catches it up only once after resume", () => {
    const planner = new MusicPhaseAutomationPlanner();
    planner.observe({ eventId: "event-1", phase: "focus", phaseToken: "segment-1", suspended: false });
    const breakObservation = { eventId: "event-1", phase: "short-break" as const, phaseToken: "segment-2", suspended: true };
    expect(planner.observe(breakObservation)).toBeNull();
    expect(planner.observe({ ...breakObservation, suspended: false })).toMatchObject({ activation: "catch-up" });
    expect(planner.observe({ ...breakObservation, suspended: false })).toBeNull();
  });

  it("marks a consecutive event and rejects stale retries", () => {
    const planner = new MusicPhaseAutomationPlanner();
    const first = planner.observe({ eventId: "event-1", phase: "focus", phaseToken: "one", suspended: false });
    const second = planner.observe({ eventId: "event-2", phase: "focus", phaseToken: "two", suspended: false });
    expect(second).toMatchObject({ consecutiveEvent: true, activation: "boundary" });
    expect(planner.retry(first?.key ?? "")).toBeNull();
    expect(planner.retry(second?.key ?? "")).toMatchObject({ activation: "catch-up" });
  });
});

describe("active soundtrack event selection", () => {
  it("uses the authoritative Pomodoro block during a calendar conflict", () => {
    const now = new Date(2026, 6, 15, 10, 30).getTime();
    const long = event("long", "2026-07-15 10:00", "2026-07-15 12:00");
    const short = event("short", "2026-07-15 10:15", "2026-07-15 11:00");
    expect(selectActiveMusicEvent([short, long], { nowMs: now, activeBlockId: "long" })?.id).toBe("long");
    expect(selectActiveMusicEvent([long, short], { nowMs: now, activeBlockId: null })?.id).toBe("short");
  });

  it("ignores all-day events and schedules the next exact boundary", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 6, 15, 9, 0));
    const allDay = event("all-day", "2026-07-15 00:00", "2026-07-16 00:00", { allDay: true });
    const later = event("later", "2026-07-15 10:00", "2026-07-15 11:00");
    expect(selectActiveMusicEvent([allDay], { nowMs: Date.now(), activeBlockId: null })).toBeNull();
    expect(nextMusicEventBoundaryMs([allDay, later], Date.now())).toBe(new Date(2026, 6, 15, 10, 0).getTime());
    vi.useRealTimers();
  });
});
