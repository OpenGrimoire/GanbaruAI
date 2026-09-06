import { describe, expect, it } from "vitest";
import type { PersistedSegment } from "$lib/components/calendar/types";
import {
  buildMobilePomodoroNotificationState,
  type MobilePomodoroNotificationCopy,
} from "./pomodoro-mobile-notification";
import { createCustomCountPomodoroConfig } from "$lib/pomodoro/rhythm";

const copy: MobilePomodoroNotificationCopy = {
  channelName: "Focus sessions",
  channelDescription: "Active focus session progress",
  alertsChannelName: "Focus alerts",
  alertsChannelDescription: "Focus phase boundary alerts",
  focusTitle: "Focus",
  shortBreakTitle: "Short break",
  longBreakTitle: "Long break",
  pausedText: "Paused",
  focusCompleteTitle: "Focus complete",
  breakCompleteTitle: "Break complete",
  sessionCompleteText: "Session complete",
};

const config = createCustomCountPomodoroConfig({
  focusDurationMinutes: 40,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  longBreakAfterFocusCount: 4,
});

function segment(overrides: Partial<PersistedSegment> = {}): PersistedSegment {
  return {
    id: "focus-1",
    eventId: "event-1",
    eventDate: "2026-08-28",
    runId: "run-1",
    rhythmPosition: 1,
    phase: "focus",
    plannedStart: "2026-08-28T16:00:00.000Z",
    plannedEnd: "2026-08-28T16:40:00.000Z",
    actualStart: "2026-08-28T16:00:00.000Z",
    actualEnd: null,
    pauseLog: [],
    status: "active",
    ...overrides,
  };
}

describe("Android Pomodoro notification projection", () => {
  it("publishes the accepted phase only even when future focus and breaks are planned", () => {
    const state = buildMobilePomodoroNotificationState({
      activeRunId: "run-1",
      activeBlockId: "event-1",
      activeBlockTitle: "Write project brief",
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: Date.parse("2026-08-28T16:45:00.000Z"),
      remainingSeconds: 1_500,
      totalSeconds: 2_400,
      isRunning: true,
      skipNextBreak: false,
      config,
      currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:20:00.000Z"),
      copy,
      segments: [
        segment(),
        segment({
          id: "break-1",
          phase: "short_break",
          status: "planned",
          actualStart: null,
          plannedStart: "2026-08-28T16:40:00.000Z",
          plannedEnd: "2026-08-28T16:45:00.000Z",
        }),
        segment({
          id: "focus-2",
          rhythmPosition: 2,
          status: "planned",
          actualStart: null,
          plannedStart: "2026-08-28T16:45:00.000Z",
          plannedEnd: "2026-08-28T17:25:00.000Z",
        }),
      ],
    });

    expect(state?.phases).toEqual([
      expect.objectContaining({
        id: "focus-1",
        endsAtEpochMs: Date.parse("2026-08-28T16:45:00.000Z"),
      }),
    ]);
  });

  it("keeps a paused session visible without projecting automatic phase changes", () => {
    const state = buildMobilePomodoroNotificationState({
      activeRunId: "run-1",
      activeBlockId: "event-1",
      activeBlockTitle: "Write project brief",
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: null,
      remainingSeconds: 1_200,
      totalSeconds: 2_400,
      isRunning: false,
      skipNextBreak: false,
      config,
      currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:20:00.000Z"),
      copy,
      segments: [segment()],
    });

    expect(state).toMatchObject({ isRunning: false, remainingSeconds: 1_200 });
    expect(state?.phases).toHaveLength(1);
    expect(state?.phases[0]?.endsAtEpochMs).toBe(Date.parse("2026-08-28T17:00:00.000Z"));
  });

  it("does not publish a finished break awaiting return as an accepted paused phase", () => {
    expect(buildMobilePomodoroNotificationState({
      activeRunId: "run-1", activeBlockId: "event-1", activeBlockTitle: null,
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: null, remainingSeconds: 0, totalSeconds: 300, isRunning: false,
      skipNextBreak: false, config, currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:20:00.000Z"), copy,
      segments: [{ ...segment(), phase: "short_break" }],
    })).toBeNull();
  });

  it("publishes a committed phase without requiring a full event projection", () => {
    const state = buildMobilePomodoroNotificationState({
      activeRunId: "run-1",
      activeBlockId: "event-1",
      activeBlockTitle: null,
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: Date.parse("2026-08-28T16:40:00.000Z"),
      remainingSeconds: 1_200,
      totalSeconds: 2_400,
      isRunning: true,
      skipNextBreak: false,
      config,
      currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:20:00.000Z"),
      copy,
      segments: [segment()],
    });

    expect(state?.phases).toHaveLength(1);
  });

  it("does not extend a current phase or publish later phases from a changed event plan", () => {
    const state = buildMobilePomodoroNotificationState({
      activeRunId: "run-1",
      activeBlockId: "event-1",
      activeBlockTitle: "Write project brief",
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: Date.parse("2026-08-28T16:13:43.670Z"),
      remainingSeconds: 2_000,
      totalSeconds: 2_400,
      isRunning: true,
      skipNextBreak: false,
      config,
      currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:06:00.000Z"),
      copy,
      segments: [
        segment(),
        segment({
          id: "break-1",
          phase: "short_break",
          status: "planned",
          actualStart: null,
          plannedStart: "2026-08-28T16:13:43.670Z",
          plannedEnd: "2026-08-28T16:18:43.670Z",
        }),
        segment({
          id: "focus-2",
          rhythmPosition: 2,
          status: "planned",
          actualStart: null,
          plannedStart: "2026-08-28T16:18:43.670Z",
          plannedEnd: "2026-08-28T16:58:43.670Z",
        }),
        segment({
          id: "break-2",
          phase: "short_break",
          rhythmPosition: 2,
          status: "planned",
          actualStart: null,
          plannedStart: "2026-08-28T16:58:43.670Z",
          plannedEnd: "2026-08-28T16:59:43.670Z",
        }),
      ],
    });

    expect(state?.phases.at(-1)).toMatchObject({
      id: "focus-1",
      startsAtEpochMs: Date.parse("2026-08-28T16:00:00.000Z"),
      endsAtEpochMs: Date.parse("2026-08-28T16:13:43.670Z"),
    });
  });

  it("publishes a trimmed event title for the native notification", () => {
    const state = buildMobilePomodoroNotificationState({
      activeRunId: "run-1",
      activeBlockId: "event-1",
      activeBlockTitle: "  Write project brief  ",
      activeBlockEndMs: Date.parse("2026-08-28T17:00:00.000Z"),
      phaseEndTime: null,
      remainingSeconds: 1_200,
      totalSeconds: 2_400,
      isRunning: false,
      skipNextBreak: false,
      config,
      currentSegmentIndex: 0,
      nowMs: Date.parse("2026-08-28T16:20:00.000Z"),
      copy,
      segments: [segment()],
    });

    expect(state?.eventTitle).toBe("Write project brief");
  });
});
