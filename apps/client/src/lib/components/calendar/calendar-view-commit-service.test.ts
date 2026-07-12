import { describe, expect, it, vi } from "vitest";
import type { CalendarEvent } from "./types";
import type { EditSessionState } from "./edit-session.svelte";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";
import {
  CalendarViewCommitService,
  calendarDataOnly,
} from "./calendar-view-commit-service";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;

function event(id: string): CalendarEvent {
  return {
    id,
    title: id,
    start: "2026-07-12 09:00",
    end: "2026-07-12 10:00",
    timezone: "UTC",
    calendarId: "calendar-a",
  };
}

describe("calendarDataOnly", () => {
  it("removes project task links from calendar persistence", () => {
    expect(calendarDataOnly({
      title: "Title",
      start: "2026-07-12 09:00",
      end: "2026-07-12 10:00",
      description: "",
      linkedTaskIds: ["task-a"],
    })).not.toHaveProperty("linkedTaskIds");
  });
});

describe("CalendarViewCommitService", () => {
  it("updates a direct edit before synchronizing its active Pomodoro", async () => {
    const original = event("event-a");
    const state: EditSessionState = {
      mode: "edit",
      sessionKey: 1,
      originalEvent: original,
      instanceEvent: original,
      templateId: original.id,
      detailsLoaded: true,
      anchor: { x: 0, y: 0, width: 0, height: 0 },
    };
    const order: string[] = [];
    const updateBlock = vi.fn(async (_event: CalendarEvent) => { order.push("update"); });
    const syncSavedActivePomodoro = vi.fn(async () => { order.push("sync"); });
    const calendarStore = { updateBlock, rawBlocks: [] } as unknown as CalendarStore;
    const pomodoro = { isActive: false } as unknown as PomodoroStore;
    const service = new CalendarViewCommitService({
      calendarStore,
      pomodoro,
      getSessionState: () => state,
      getViewWindow: () => ({ start: {} as never, end: {} as never }),
      isRecurring: () => false,
      effectiveScope: () => "this",
      activeDate: () => undefined,
      syncSavedActivePomodoro,
    });

    await service.persist({
      title: "Updated",
      start: original.start,
      end: original.end,
      description: "",
      linkedTaskIds: ["task-a"],
    });

    expect(order).toEqual(["update", "sync"]);
    expect(updateBlock).toHaveBeenCalledWith(expect.objectContaining({
      id: original.id,
      title: "Updated",
    }));
    expect(updateBlock.mock.calls[0]?.[0]).not.toHaveProperty("linkedTaskIds");
  });

  it("does not perform persistence after the session closes", async () => {
    const addBlock = vi.fn();
    const updateBlock = vi.fn();
    const service = new CalendarViewCommitService({
      calendarStore: { addBlock, updateBlock } as unknown as CalendarStore,
      pomodoro: {} as PomodoroStore,
      getSessionState: () => ({ mode: "closed" }),
      getViewWindow: () => ({ start: {} as never, end: {} as never }),
      isRecurring: () => false,
      effectiveScope: () => "this",
      activeDate: () => undefined,
      syncSavedActivePomodoro: vi.fn(),
    });
    await service.persist({
      title: "Ignored",
      start: "2026-07-12 09:00",
      end: "2026-07-12 10:00",
      description: "",
    });
    expect(addBlock).not.toHaveBeenCalled();
    expect(updateBlock).not.toHaveBeenCalled();
  });
});
