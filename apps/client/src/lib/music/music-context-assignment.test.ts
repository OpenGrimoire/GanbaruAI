import { describe, expect, it } from "vitest";
import {
  resolveMusicContextAssignment,
  parseMusicContextAssignments,
  type MusicActivityPhase,
  type MusicAssignmentBehavior,
  type MusicAssignmentOwnerKind,
  type MusicContextAssignment,
} from "./music-context-assignment";

function assignment(
  ownerKind: MusicAssignmentOwnerKind,
  behavior: MusicAssignmentBehavior,
  phase: MusicActivityPhase = "focus",
  playlistId: string | null = "playlist-1",
): MusicContextAssignment {
  return {
    ownerKind,
    ownerId: `${ownerKind}-1`,
    phase,
    behavior,
    playlistId,
    soundscapeId: null,
    provenanceKind: ownerKind === "event-snapshot" ? "copied-project" : ownerKind === "work-environment" ? "work-environment" : "explicit",
    provenanceId: null,
    updatedAt: 1,
    version: 1,
  };
}

const base = {
  phase: "focus" as const,
  eventOverride: null,
  environmentAssignment: null,
  projectSnapshot: null,
  availablePlaylistIds: new Set(["playlist-1"]),
  timedEvent: true,
  pomodoroEnabled: true,
  activation: "boundary" as const,
  consecutiveEvent: false,
};

describe("music context assignment resolver", () => {
  it.each((["focus", "short-break", "long-break"] as const).flatMap((phase) => [
    [phase, "event-override" as const],
    [phase, "work-environment" as const],
    [phase, "project-snapshot" as const],
  ]))("resolves %s with %s precedence", (phase, expectedSource) => {
    const eventOverride = expectedSource === "event-override"
      ? assignment("event-override", "play-automatically", phase)
      : assignment("event-override", "inherit", phase);
    const environmentAssignment = expectedSource === "work-environment"
      ? assignment("work-environment", "prepare-silently", phase)
      : assignment("work-environment", "inherit", phase);
    const projectSnapshot = assignment("event-snapshot", "pause-music", phase);
    expect(resolveMusicContextAssignment({
      ...base,
      phase,
      eventOverride,
      environmentAssignment,
      projectSnapshot,
    }).source).toBe(expectedSource);
  });

  it.each([
    ["event override", assignment("event-override", "play-automatically"), assignment("work-environment", "pause-music"), assignment("event-snapshot", "keep-current-music"), "event-override"],
    ["environment", assignment("event-override", "inherit"), assignment("work-environment", "prepare-silently"), assignment("event-snapshot", "pause-music"), "work-environment"],
    ["project snapshot", null, null, assignment("event-snapshot", "pause-music"), "project-snapshot"],
  ] as const)("uses %s precedence", (_label, eventOverride, environmentAssignment, projectSnapshot, source) => {
    expect(resolveMusicContextAssignment({ ...base, eventOverride, environmentAssignment, projectSnapshot }).source).toBe(source);
  });

  it.each([
    ["play-automatically", true],
    ["prepare-silently", true],
    ["pause-music", false],
    ["keep-current-music", false],
  ] as const)("resolves %s with fresh-track behavior %s", (behavior, startFreshTrack) => {
    const resolved = resolveMusicContextAssignment({
      ...base,
      eventOverride: assignment("event-override", behavior),
    });
    expect(resolved).toMatchObject({ availability: "ready", startFreshTrack });
  });

  it("distinguishes missing playlists from no assignment", () => {
    expect(resolveMusicContextAssignment({
      ...base,
      eventOverride: assignment("event-override", "play-automatically", "focus", "deleted"),
    }).availability).toBe("missing-playlist");
    expect(resolveMusicContextAssignment(base).availability).toBe("no-assignment");
  });

  it("reports a deleted soundscape independently from a playable playlist", () => {
    const withSoundscape = {
      ...assignment("event-override", "play-automatically"),
      soundscapeId: "deleted-rain",
    };
    expect(resolveMusicContextAssignment({
      ...base,
      eventOverride: withSoundscape,
      availableSoundscapeIds: new Set(),
    })).toMatchObject({ availability: "ready", soundscapeAvailability: "missing-soundscape" });
    expect(resolveMusicContextAssignment({
      ...base,
      eventOverride: withSoundscape,
      availableSoundscapeIds: new Set(["deleted-rain"]),
    }).soundscapeAvailability).toBe("ready");
  });

  it("applies only focus to a timed event without Pomodoro", () => {
    expect(resolveMusicContextAssignment({
      ...base,
      pomodoroEnabled: false,
      eventOverride: assignment("event-override", "play-automatically"),
    }).availability).toBe("ready");
    expect(resolveMusicContextAssignment({
      ...base,
      phase: "short-break",
      pomodoroEnabled: false,
      eventOverride: assignment("event-override", "play-automatically", "short-break"),
    }).availability).toBe("not-applicable");
  });

  it("retains catch-up and consecutive-boundary context while still starting fresh", () => {
    expect(resolveMusicContextAssignment({
      ...base,
      activation: "catch-up",
      consecutiveEvent: true,
      eventOverride: assignment("event-override", "prepare-silently"),
    })).toMatchObject({ catchUp: true, consecutiveEvent: true, startFreshTrack: true });
  });

  it("does not apply phase music to all-day events", () => {
    expect(resolveMusicContextAssignment({
      ...base,
      timedEvent: false,
      eventOverride: assignment("event-override", "pause-music"),
    }).availability).toBe("not-applicable");
  });
});

describe("music context assignment boundary", () => {
  it("parses a complete persisted assignment", () => {
    expect(parseMusicContextAssignments([{
      ownerKind: "project-default",
      ownerId: "project-1",
      phase: "focus",
      behavior: "play-automatically",
      playlistId: "playlist-1",
      soundscapeId: null,
      provenanceKind: "explicit",
      provenanceId: null,
      updatedAt: 1_700_000_000_000,
      version: 2,
    }])).toEqual([expect.objectContaining({
      ownerId: "project-1",
      phase: "focus",
      version: 2,
    })]);
  });

  it("rejects unknown enum values and unsafe versions", () => {
    const valid = {
      ownerKind: "project-default",
      ownerId: "project-1",
      phase: "focus",
      behavior: "play-automatically",
      playlistId: null,
      soundscapeId: null,
      provenanceKind: "explicit",
      provenanceId: null,
      updatedAt: 1,
      version: 1,
    };
    expect(() => parseMusicContextAssignments([{ ...valid, behavior: "surprise-me" }])).toThrow("behavior");
    expect(() => parseMusicContextAssignments([{ ...valid, version: Number.MAX_SAFE_INTEGER + 1 }])).toThrow("version");
  });
});
