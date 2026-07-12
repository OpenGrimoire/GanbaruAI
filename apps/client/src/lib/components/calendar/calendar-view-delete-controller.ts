import type { CalendarEvent } from "./types";
import type {
  CalendarDeleteArchiveOutcome,
  CalendarDeleteArchivePlan,
  CalendarDeleteArchiveRestoreSnapshot,
} from "./delete-archive-plan";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";
import type { createCalendarViewToastController } from "./calendar-view-toasts.svelte";
import type { computeViewWindow } from "./utils";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;
type ToastController = ReturnType<typeof createCalendarViewToastController>;
type ViewWindow = ReturnType<typeof computeViewWindow>;

export interface CalendarViewDeleteControllerOptions {
  calendarStore: CalendarStore;
  pomodoro: PomodoroStore;
  toasts: ToastController;
  getWindow: () => ViewWindow;
  setCommitState: (state: {
    hidden: boolean;
    suppressPreview: boolean;
    frozenEvents: CalendarEvent[] | null;
  }) => void;
  closeSession: () => void;
  pendingLabel: (outcome: CalendarDeleteArchiveOutcome) => string;
  outcomeLabel: (outcome: CalendarDeleteArchiveOutcome) => string;
}

/** Owns delete/archive execution, hydrated undo snapshots, and rollback UI cleanup. */
export class CalendarViewDeleteController {
  constructor(private readonly options: CalendarViewDeleteControllerOptions) {}

  async execute(plan: CalendarDeleteArchivePlan, stopActiveSession: boolean): Promise<void> {
    const activeBlockId = this.options.pomodoro.isActive
      ? this.options.pomodoro.activeBlockId
      : null;
    this.options.setCommitState({
      hidden: true,
      suppressPreview: true,
      frozenEvents: plan.finalVisibleEvents.map((event) => ({ ...event })),
    });
    const toastId = this.options.toasts.showDeletePendingToast(
      this.options.pendingLabel(plan.outcome),
    );
    try {
      const restore = await this.buildRestore(plan);
      if (stopActiveSession && activeBlockId) {
        this.options.pomodoro.dismissedBlockId = activeBlockId;
        await this.options.pomodoro.stopSession();
      }
      await this.options.calendarStore.applyDeleteArchivePlan(plan.operations);
      await this.refreshWindow();
      this.options.closeSession();
      this.options.toasts.showDeleteUndoToast(
        toastId,
        this.options.outcomeLabel(plan.outcome),
        restore,
      );
    } finally {
      if (this.options.toasts.deleteUndoToast?.id === toastId
        && this.options.toasts.deleteUndoToast.pending) {
        this.options.toasts.dismissDeleteToastIfCurrent(toastId);
      }
      this.options.setCommitState({
        hidden: false,
        suppressPreview: false,
        frozenEvents: null,
      });
    }
  }

  async undoCurrent(): Promise<void> {
    const toast = this.options.toasts.deleteUndoToast;
    if (!toast?.restore) return;
    this.options.toasts.dismissDeleteUndoToast();
    try {
      await toast.restore();
    } catch (error) {
      console.error("[calendar] failed to restore deleted event:", error);
    }
  }

  private async buildRestore(
    plan: CalendarDeleteArchivePlan,
  ): Promise<(() => Promise<void>) | undefined> {
    const snapshots = await Promise.all(plan.restore.snapshots.map((snapshot) =>
      this.hydrateSnapshot(snapshot)));
    if (plan.restore.archivedEvents.length === 0 && snapshots.length === 0) return undefined;
    return async () => {
      for (const event of plan.restore.archivedEvents) {
        await this.options.calendarStore.restoreArchivedBlock(event);
      }
      for (const snapshot of snapshots) {
        if (snapshot.restoreMode === "insert") await this.restoreDeletedBlock(snapshot.event);
        else await this.options.calendarStore.updateBlock(snapshot.event);
      }
      await this.refreshWindow();
    };
  }

  private async hydrateSnapshot(
    snapshot: CalendarDeleteArchiveRestoreSnapshot,
  ): Promise<CalendarDeleteArchiveRestoreSnapshot> {
    const full = await this.options.calendarStore.loadFullEvent(snapshot.event.id);
    if (!full) return snapshot;
    return {
      ...snapshot,
      event: {
        ...full,
        exceptions: [...(snapshot.event.exceptions ?? full.exceptions ?? [])],
      },
    };
  }

  private async restoreDeletedBlock(event: CalendarEvent): Promise<void> {
    await this.options.calendarStore.addBlock({
      id: event.id,
      title: event.title,
      start: event.start,
      end: event.end,
      timezone: event.timezone,
      calendarId: event.calendarId,
      projectId: event.projectId,
      color: event.color,
      environmentId: event.environmentId,
      playlistId: event.playlistId,
      description: event.description,
      recurrence: event.recurrence,
      notifications: event.notifications,
      exceptions: event.exceptions,
      pomodoroConfig: event.pomodoroConfig,
      allDay: event.allDay,
      location: event.location,
      url: event.url,
      meetingEnabled: event.meetingEnabled,
      transparency: event.transparency,
      status: event.status,
      sourceUid: event.sourceUid,
      visibility: event.visibility,
      priority: event.priority,
      categories: event.categories,
      geo: event.geo,
      sequence: event.sequence,
      rdate: event.rdate,
      extendedProperties: event.extendedProperties,
      organizer: event.organizer,
      attendees: event.attendees,
      localParticipationStatus: event.localParticipationStatus,
      guestPermissions: event.guestPermissions,
    });
  }

  private async refreshWindow(): Promise<void> {
    const window = this.options.getWindow();
    await this.options.calendarStore.refreshWindow(window.start, window.end);
  }
}
