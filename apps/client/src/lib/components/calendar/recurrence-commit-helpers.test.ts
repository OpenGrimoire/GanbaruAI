import { describe, expect, it } from "vitest";
import { patchForTemplateAnchor } from "./recurrence-commit-helpers";
import type { CalendarEvent } from "./types";

function makeTemplate(overrides: Partial<CalendarEvent> = {}): CalendarEvent {
  return {
    id: "template-a",
    title: "Focus",
    start: "2026-06-12 09:00",
    end: "2026-06-12 10:00",
    timezone: "America/Monterrey",
    calendarId: "local",
    projectId: "project-a",
    environmentId: "environment-a",
    playlistId: "playlist-a",
    recurrence: { frequency: "daily", interval: 1, end: { type: "never" } },
    ...overrides,
  };
}

describe("patchForTemplateAnchor", () => {
  it("preserves project automation identifiers when updating a recurring template", () => {
    const template = makeTemplate();
    const occurrence: CalendarEvent = {
      ...template,
      id: "template-a::2026-06-14",
      start: "2026-06-14 09:00",
      end: "2026-06-14 10:00",
      recurringParentId: template.id,
    };

    const result = patchForTemplateAnchor(template, occurrence, {
      title: "Updated focus",
    });

    expect(result.projectId).toBe("project-a");
    expect(result.environmentId).toBe("environment-a");
    expect(result.playlistId).toBe("playlist-a");
  });
});
