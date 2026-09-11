import { describe, expect, it, vi } from "vitest";
import type { CalendarEvent } from "./types";
import type { EditSessionState } from "./edit-session.svelte";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";
import type { createCalendarViewToastController } from "./calendar-view-toasts.svelte";
import type { CalendarViewCommitService } from "./calendar-view-commit-service";
import { CalendarViewSaveController } from "./calendar-view-save-controller.svelte";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;
type ToastController = ReturnType<typeof createCalendarViewToastController>;

function event(): CalendarEvent {
  return {
    id: "event-a",
    title: "Event",
    start: "2026-07-12 09:00",
    end: "2026-07-12 10:00",
    timezone: "UTC",
    calendarId: "calendar-a",
  };
}

describe("CalendarViewSaveController", () => {
  it("persists before stopping the confirmed active session and refreshing", async () => {
    const selected = event();
    const state: EditSessionState = {
      mode: "edit",
      sessionKey: 1,
      originalEvent: selected,
      instanceEvent: selected,
      templateId: selected.id,
      detailsLoaded: true,
      anchor: { x: 0, y: 0, width: 0, height: 0 },
    };
    const order: string[] = [];
    const persist = vi.fn(async () => {
      order.push("persist");
      return { saveRefreshedVisibleWindow: false };
    });
    const pomodoro = {
      isActive: true,
      activeBlockId: selected.id,
      autoStartSuppressed: false,
      stopSession: vi.fn(async () => { order.push("stop"); }),
    } as unknown as PomodoroStore;
    const toasts = {
      saveSuccessToast: null,
      showSavePendingToast: vi.fn(() => "toast-a"),
      showSaveSuccessToast: vi.fn(),
      showSaveErrorToast: vi.fn(),
      dismissSaveToastIfCurrent: vi.fn(),
    } as unknown as ToastController;
    let confirmed: (() => Promise<void>) | undefined;
    const controller = new CalendarViewSaveController({
      calendarStore: {} as CalendarStore,
      pomodoro,
      toasts,
      commitService: { persist } as unknown as CalendarViewCommitService,
      getSessionState: () => state,
      canEnablePomodoro: () => false,
      isSelectedEndable: () => true,
      isSelectedActivePomodoro: () => true,
      isRecurring: () => false,
      wouldSaveStopSession: () => true,
      endWouldStopProductivity: () => false,
      confirmSaveStop: (action) => { confirmed = action; },
      confirmEndStop: vi.fn(),
      buildFreeze: () => [selected],
      setDisplayState: vi.fn(),
      refreshWindow: async () => { order.push("refresh"); },
      closeSession: () => { order.push("close"); },
      afterRender: async () => undefined,
      savePendingLabel: () => "saving",
      saveSuccessLabel: () => "saved",
      saveErrorLabel: () => "failed",
      logError: vi.fn(),
    });
    const data = { ...selected, description: "" };

    await controller.save(data);
    expect(persist).not.toHaveBeenCalled();
    if (!confirmed) throw new Error("Expected save confirmation");
    await confirmed();

    expect(order).toEqual(["persist", "stop", "refresh", "close"]);
    expect(pomodoro.autoStartSuppressed).toBe(false);
  });
});
