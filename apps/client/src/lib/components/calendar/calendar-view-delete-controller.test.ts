import { describe, expect, it, vi } from "vitest";
import type { CalendarEvent } from "./types";
import type { CalendarDeleteArchivePlan } from "./delete-archive-plan";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";
import type { createCalendarViewToastController } from "./calendar-view-toasts.svelte";
import { CalendarViewDeleteController } from "./calendar-view-delete-controller";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;
type ToastController = ReturnType<typeof createCalendarViewToastController>;

function event(id: string, overrides: Partial<CalendarEvent> = {}): CalendarEvent {
  return {
    id,
    title: id,
    start: "2026-07-12 09:00",
    end: "2026-07-12 10:00",
    timezone: "UTC",
    calendarId: "calendar-a",
    ...overrides,
  };
}

function plan(snapshot: CalendarEvent): CalendarDeleteArchivePlan {
  return {
    affectedVisibleIds: new Set([snapshot.id]),
    finalVisibleEvents: [],
    outcome: "delete",
    requiresActiveStop: true,
    operations: [],
    restore: {
      archivedEvents: [],
      snapshots: [{ event: snapshot, restoreMode: "insert" }],
    },
  };
}

describe("CalendarViewDeleteController", () => {
  it("hydrates restore data before stopping the active session and committing", async () => {
    const order: string[] = [];
    const snapshot = event("event-a", { exceptions: ["2026-07-14"] });
    const loadFullEvent = vi.fn(async () => {
      order.push("hydrate");
      return event("event-a", { description: "full", exceptions: ["2026-07-15"] });
    });
    const addBlock = vi.fn(async (restored: Partial<CalendarEvent>) => {
      expect(restored.description).toBe("full");
      expect(restored.exceptions).toEqual(["2026-07-14"]);
    });
    const calendarStore = {
      loadFullEvent,
      applyDeleteArchivePlan: vi.fn(async () => { order.push("apply"); }),
      refreshWindow: vi.fn(async () => { order.push("refresh"); }),
      restoreArchivedBlock: vi.fn(),
      updateBlock: vi.fn(),
      addBlock,
    } as unknown as CalendarStore;
    const pomodoro = {
      isActive: true,
      activeBlockId: "event-a",
      dismissedBlockId: null,
      stopSession: vi.fn(async () => { order.push("stop"); }),
    } as unknown as PomodoroStore;
    let toast: { id: string; pending: boolean; restore?: () => Promise<void>; label: string } | null = null;
    const showDeleteUndoToast = vi.fn(
      (_id: string, label: string, restore?: () => Promise<void>) => {
        toast = { id: "toast-a", pending: false, label, restore };
      },
    );
    const toasts = {
      get deleteUndoToast() { return toast; },
      showDeletePendingToast: vi.fn(() => {
        toast = { id: "toast-a", pending: true, label: "pending" };
        return "toast-a";
      }),
      showDeleteUndoToast,
      dismissDeleteToastIfCurrent: vi.fn(),
      dismissDeleteUndoToast: vi.fn(),
    } as unknown as ToastController;
    const setCommitState = vi.fn();
    const closeSession = vi.fn(() => { order.push("close"); });
    const controller = new CalendarViewDeleteController({
      calendarStore,
      pomodoro,
      toasts,
      getWindow: () => ({ start: {} as never, end: {} as never }),
      setCommitState,
      closeSession,
      pendingLabel: () => "pending",
      outcomeLabel: () => "deleted",
    });

    await controller.execute(plan(snapshot), true);
    expect(order).toEqual(["hydrate", "stop", "apply", "refresh", "close"]);
    await showDeleteUndoToast.mock.calls[0]?.[2]?.();
    expect(addBlock).toHaveBeenCalledOnce();
    expect(setCommitState).toHaveBeenLastCalledWith({
      hidden: false,
      suppressPreview: false,
      frozenEvents: null,
    });
  });

  it("does not stop or mutate when restore hydration fails", async () => {
    const stopSession = vi.fn();
    const applyDeleteArchivePlan = vi.fn();
    const controller = new CalendarViewDeleteController({
      calendarStore: {
        loadFullEvent: vi.fn(async () => { throw new Error("hydrate failed"); }),
        applyDeleteArchivePlan,
      } as unknown as CalendarStore,
      pomodoro: {
        isActive: true,
        activeBlockId: "event-a",
        stopSession,
      } as unknown as PomodoroStore,
      toasts: {
        deleteUndoToast: { id: "toast-a", pending: true, label: "pending" },
        showDeletePendingToast: () => "toast-a",
        dismissDeleteToastIfCurrent: vi.fn(),
      } as unknown as ToastController,
      getWindow: () => ({ start: {} as never, end: {} as never }),
      setCommitState: vi.fn(),
      closeSession: vi.fn(),
      pendingLabel: () => "pending",
      outcomeLabel: () => "deleted",
    });
    await expect(controller.execute(plan(event("event-a")), true)).rejects.toThrow("hydrate failed");
    expect(stopSession).not.toHaveBeenCalled();
    expect(applyDeleteArchivePlan).not.toHaveBeenCalled();
  });
});
