import { describe, expect, it } from "vitest";
import type { CalendarEvent } from "./types";
import {
  projectCalendarSurfaceStatuses,
  visibleCalendarEvents,
} from "./calendar-view-display-projection";

function event(id: string, calendarId = "calendar-a"): CalendarEvent {
  return {
    id,
    title: id,
    start: "2026-07-12 09:00",
    end: "2026-07-12 10:00",
    timezone: "UTC",
    calendarId,
  };
}

describe("visibleCalendarEvents", () => {
  it("applies calendar visibility before the route filter", () => {
    const events = [event("keep"), event("filtered"), event("hidden", "calendar-b")];
    expect(visibleCalendarEvents({
      events,
      visibleCalendarIds: new Set(["calendar-a"]),
      filter: (candidate) => candidate.id !== "filtered",
    }).map((candidate) => candidate.id)).toEqual(["keep"]);
  });
});

describe("projectCalendarSurfaceStatuses", () => {
  it("gives an in-panel RSVP preview priority over stored attendee status", () => {
    const selected = event("event-a");
    selected.attendees = [{
      id: "attendee-a",
      email: "me@example.com",
      role: "req-participant",
      status: "accepted",
      rsvp: true,
    }];
    const [projected] = projectCalendarSurfaceStatuses({
      events: [selected],
      identityByCalendarId: new Map([["calendar-a", "me@example.com"]]),
      pendingStatus: "declined",
      pendingEventId: selected.id,
    });
    expect(projected.surfaceStatus).toBe("declined");
  });
});
