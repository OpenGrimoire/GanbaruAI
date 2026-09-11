import { Temporal } from "@js-temporal/polyfill";
import { describe, expect, it, vi } from "vitest";
import type { CalendarEvent } from "$lib/components/calendar/types";
import { DEFAULT_CONFIG } from "$lib/stores/pomodoro-machine";
import { createPomodoroCalendarScheduler, startScheduledPomodoro } from "./pomodoro-calendar-scheduler";

function activeEvent(): CalendarEvent {
  return {
    id: "focus-event",
    title: "Focus",
    start: "2026-08-26 14:00:00",
    end: "2026-08-26 15:00:00",
    timezone: "America/Monterrey",
    calendarId: "calendar-1",
    createdAt: "2026-08-26T12:00:00.000Z",
    pomodoroConfig: { ...DEFAULT_CONFIG },
  };
}

describe("Pomodoro calendar scheduler", () => {
  it("starts a desktop commitment after a fresh activity observation", async () => {
    const event = activeEvent();
    const startFromBlock = vi.fn(async () => undefined);
    const pomodoro = {
      activeBlockId: null as string | null,
      dismissedBlockId: null as string | null,
      blockExpired: false,
      autoStartSuppressed: false,
      clearBlockExpired: vi.fn(),
      startFromBlock,
      stopSession: vi.fn(async () => undefined),
    };
    const scheduler = createPomodoroCalendarScheduler({
      calendar: {
        loadPomodoroSchedulerEvents: vi.fn(async () => [event]),
      },
      pomodoro,
      isBlocked: () => false,
      canStartAutomatically: vi.fn(async () => true),
      now: () => new Date(2026, 7, 26, 14, 30),
      today: () => Temporal.PlainDate.from("2026-08-26"),
      buildPlannedBlocks: () => [],
    });

    scheduler.setEnabled(true);

    await vi.waitFor(() => {
      expect(startFromBlock).toHaveBeenCalledWith(
        event.id,
        event.pomodoroConfig,
        event.title,
        event.end,
        "2026-08-26",
        event.pomodoroConfig?.idleTimeoutMinutes,
        false,
        [],
      );
    });
    scheduler.dispose();
  });

  it("does not start a dismissed active event", async () => {
    const event = activeEvent();
    const startFromBlock = vi.fn(async () => undefined);
    const scheduler = createPomodoroCalendarScheduler({
      calendar: {
        loadPomodoroSchedulerEvents: vi.fn(async () => [event]),
      },
      pomodoro: {
        activeBlockId: null,
        dismissedBlockId: event.id,
        blockExpired: false,
        autoStartSuppressed: false,
        clearBlockExpired: vi.fn(),
        startFromBlock,
        stopSession: vi.fn(async () => undefined),
      },
      isBlocked: () => false,
      now: () => new Date(2026, 7, 26, 14, 30),
      today: () => Temporal.PlainDate.from("2026-08-26"),
      buildPlannedBlocks: () => [],
    });

    scheduler.setEnabled(true);
    await new Promise<void>((resolve) => setTimeout(resolve, 0));

    expect(startFromBlock).not.toHaveBeenCalled();
    scheduler.dispose();
  });
});

it("keeps mobile commitments as reminders when no automatic admission source exists", async () => {
  const startFromBlock = vi.fn(async () => undefined);
  const loadEvents = vi.fn(async () => [activeEvent()]);
  const scheduler = createPomodoroCalendarScheduler({
    calendar: { loadPomodoroSchedulerEvents: loadEvents },
    pomodoro: {
      activeBlockId: null,
      dismissedBlockId: null,
      blockExpired: false,
      autoStartSuppressed: false,
      clearBlockExpired: vi.fn(),
      startFromBlock,
      stopSession: vi.fn(async () => undefined),
    },
    isBlocked: () => false,
    now: () => new Date(2026, 7, 26, 14, 30),
    today: () => Temporal.PlainDate.from("2026-08-26"),
    buildPlannedBlocks: () => [],
  });
  scheduler.setEnabled(true);
  await vi.waitFor(() => expect(loadEvents).toHaveBeenCalledOnce());
  expect(startFromBlock).not.toHaveBeenCalled();
  scheduler.dispose();
});

it("accepts a due commitment explicitly without requiring an activity source", async () => {
  const event = activeEvent();
  const runtime = {
    activeBlockId: null,
    dismissedBlockId: event.id,
    blockExpired: false,
    autoStartSuppressed: false,
    clearBlockExpired: vi.fn(),
    startFromBlock: vi.fn(async () => undefined),
    stopSession: vi.fn(async () => undefined),
  };
  expect(await startScheduledPomodoro(
    { loadPomodoroSchedulerEvents: async () => [event] }, runtime,
    () => new Date(2026, 7, 26, 14, 30),
  )).toBe(true);
  expect(runtime.startFromBlock).toHaveBeenCalledOnce();
  expect(runtime.startFromBlock.mock.calls[0]).toEqual([
    event.id, event.pomodoroConfig, event.title, event.end, "2026-08-26",
    event.pomodoroConfig?.idleTimeoutMinutes, true, expect.any(Array),
  ]);
  expect(runtime.dismissedBlockId).toBeNull();
});

it("rechecks the deadline after loading an explicit start and propagates failed persistence", async () => {
  const runtime = {
    activeBlockId: null,
    dismissedBlockId: null,
    blockExpired: false,
    autoStartSuppressed: false,
    clearBlockExpired: vi.fn(),
    startFromBlock: vi.fn(async () => { throw new Error("disk full"); }),
    stopSession: vi.fn(async () => undefined),
  };
  let now = new Date(2026, 7, 26, 14, 59);
  expect(await startScheduledPomodoro({
    loadPomodoroSchedulerEvents: async () => {
      now = new Date(2026, 7, 26, 15, 1);
      return [activeEvent()];
    },
  }, runtime, () => now)).toBe(false);
  expect(runtime.startFromBlock).not.toHaveBeenCalled();
  await expect(startScheduledPomodoro(
    { loadPomodoroSchedulerEvents: async () => [activeEvent()] }, runtime,
    () => new Date(2026, 7, 26, 14, 30),
  )).rejects.toThrow("disk full");
});
