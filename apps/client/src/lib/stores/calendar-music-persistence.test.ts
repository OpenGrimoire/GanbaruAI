import { describe, expect, it } from "vitest";
import { calendarDataOnly } from "$lib/components/calendar/calendar-view-commit-service";
import type { CalendarEvent } from "$lib/components/calendar/types";
import type { MusicContextAssignmentDraft } from "$lib/music/music-context-assignment";
import { prepareUpdateBlockPayload } from "./calendar-event-payloads";

const snapshot: MusicContextAssignmentDraft = {
  phase: "focus",
  behavior: "play-automatically",
  playlistId: "playlist-1",
  soundscapeId: null,
  provenanceKind: "copied-project",
  provenanceId: "project-1",
};
const override: MusicContextAssignmentDraft = {
  phase: "short-break",
  behavior: "pause-music",
  playlistId: null,
  soundscapeId: null,
  provenanceKind: "explicit",
  provenanceId: null,
};
const event: CalendarEvent = {
  id: "event-1",
  title: "Focus",
  start: "2026-07-15 09:00",
  end: "2026-07-15 10:00",
  timezone: "America/Monterrey",
  calendarId: "local",
};

describe("calendar music persistence", () => {
  it("encodes snapshots and overrides as transactional event update fields", () => {
    const result = prepareUpdateBlockPayload({
      id: event.id,
      musicSnapshotAssignments: [snapshot],
      musicOverrideAssignments: [override],
    }, [event]);
    expect(result.payload.fields).toEqual(expect.arrayContaining([
      { field: "musicSnapshotAssignments", value: [snapshot] },
      { field: "musicOverrideAssignments", value: [override] },
    ]));
  });

  it("keeps assignments when calendar-only data drops project task links", () => {
    expect(calendarDataOnly({
      title: "Focus",
      start: event.start,
      end: event.end,
      description: "",
      linkedTaskIds: ["task-1"],
      musicSnapshotAssignments: [snapshot],
      musicOverrideAssignments: [override],
    })).toMatchObject({
      musicSnapshotAssignments: [snapshot],
      musicOverrideAssignments: [override],
    });
  });
});
