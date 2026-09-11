import type { CalendarEvent, RecurringScope } from "./types";
import type { EditSessionState } from "./edit-session.svelte";
import type { PanelSaveData } from "./event-panel-payloads";
import type { CalendarViewCommitService } from "./calendar-view-commit-service";
import { calendarDataOnly } from "./calendar-view-commit-service";
import { formatCalendarDateWithSeconds } from "./utils";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";
import type { createCalendarViewToastController } from "./calendar-view-toasts.svelte";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;
type ToastController = ReturnType<typeof createCalendarViewToastController>;

export interface CalendarViewSaveControllerOptions {
  calendarStore: CalendarStore;
  pomodoro: PomodoroStore;
  toasts: ToastController;
  commitService: CalendarViewCommitService;
  getSessionState: () => EditSessionState;
  canEnablePomodoro: (state: Extract<EditSessionState, { mode: "edit" }>) => boolean;
  isSelectedEndable: (state: Extract<EditSessionState, { mode: "edit" }>) => boolean;
  isSelectedActivePomodoro: (state: Extract<EditSessionState, { mode: "edit" }>) => boolean;
  isRecurring: (event: CalendarEvent) => boolean;
  wouldSaveStopSession: (data: PanelSaveData, scope?: RecurringScope) => boolean;
  endWouldStopProductivity: (state: Extract<EditSessionState, { mode: "edit" }>) => boolean;
  confirmSaveStop: (action: () => Promise<void>) => void;
  confirmEndStop: (action: () => Promise<void>) => void;
  buildFreeze: (data: PanelSaveData, scope?: RecurringScope) => CalendarEvent[];
  setDisplayState: (state: {
    suppressGlow: boolean;
    suppressPreview: boolean;
    frozenEvents: CalendarEvent[] | null;
  }) => void;
  refreshWindow: () => Promise<void>;
  closeSession: () => void;
  afterRender: () => Promise<void>;
  savePendingLabel: () => string;
  saveSuccessLabel: () => string;
  saveErrorLabel: (error: unknown) => string;
  logError: (
    context: "enable-active-pomodoro" | "panel-save",
    error: unknown,
    data: PanelSaveData,
    scope?: RecurringScope,
  ) => void;
  now?: () => Date;
}

/** Owns panel save, active-event ending, confirmation, and display freeze ordering. */
export class CalendarViewSaveController {
  endingActiveEvent = $state(false);
  private sessionStopPending = false;

  constructor(private readonly options: CalendarViewSaveControllerOptions) {}

  async save(data: PanelSaveData, scope?: RecurringScope): Promise<void> {
    if (!this.sessionStopPending && this.options.wouldSaveStopSession(data, scope)) {
      this.options.confirmSaveStop(async () => {
        this.sessionStopPending = true;
        await this.save(data, scope);
      });
      return;
    }
    if (this.shouldEnablePomodoro(data)) {
      await this.enablePomodoro(data);
      return;
    }

    const state = this.options.getSessionState();
    const suppressAutoStart = state.mode === "edit"
      && this.options.pomodoro.isActive
      && !!this.options.pomodoro.activeBlockId;
    this.options.setDisplayState({
      suppressGlow: true,
      suppressPreview: true,
      frozenEvents: this.options.buildFreeze(data, scope),
    });
    if (suppressAutoStart) this.options.pomodoro.autoStartSuppressed = true;
    const toastId = this.options.toasts.showSavePendingToast(this.options.savePendingLabel());
    try {
      const result = await this.options.commitService.persist(data, scope);
      if (this.sessionStopPending) {
        await this.options.pomodoro.stopSession();
        this.sessionStopPending = false;
      }
      if (!result.saveRefreshedVisibleWindow) await this.options.refreshWindow();
      this.options.closeSession();
      this.options.toasts.showSaveSuccessToast(toastId, this.options.saveSuccessLabel());
      await this.options.afterRender();
    } catch (error) {
      this.sessionStopPending = false;
      this.options.logError("panel-save", error, data, scope);
      this.options.toasts.showSaveErrorToast(toastId, this.options.saveErrorLabel(error));
    } finally {
      this.dismissPendingSaveToast(toastId);
      if (suppressAutoStart) this.options.pomodoro.autoStartSuppressed = false;
      this.clearDisplayState();
    }
  }

  async end(data: PanelSaveData, scope?: RecurringScope): Promise<void> {
    if (this.endingActiveEvent) return;
    const state = this.options.getSessionState();
    if (state.mode !== "edit" || !this.options.isSelectedEndable(state)) return;
    if (this.options.endWouldStopProductivity(state)) {
      this.options.confirmEndStop(() => this.executeEnd(data, scope));
      return;
    }
    await this.executeEnd(data, scope);
  }

  private shouldEnablePomodoro(data: PanelSaveData): boolean {
    const state = this.options.getSessionState();
    return state.mode === "edit" && this.options.canEnablePomodoro(state) && !!data.pomodoroConfig;
  }

  private async enablePomodoro(data: PanelSaveData): Promise<void> {
    const state = this.options.getSessionState();
    const config = data.pomodoroConfig;
    if (state.mode !== "edit" || !config || !this.options.canEnablePomodoro(state)) return;
    this.options.setDisplayState({
      suppressGlow: true,
      suppressPreview: true,
      frozenEvents: this.options.buildFreeze(data),
    });
    const toastId = this.options.toasts.showSavePendingToast(this.options.savePendingLabel());
    try {
      const calendarData = calendarDataOnly(data);
      await this.options.calendarStore.updateBlock({ ...state.originalEvent, ...calendarData });
      await this.options.pomodoro.startFromBlock(
        state.originalEvent.id,
        config,
        calendarData.title,
        calendarData.end,
        calendarData.start.split(" ")[0],
        config.idleTimeoutMinutes,
      );
      await this.options.refreshWindow();
      this.options.closeSession();
      this.options.toasts.showSaveSuccessToast(toastId, this.options.saveSuccessLabel());
      await this.options.afterRender();
    } catch (error) {
      this.options.logError("enable-active-pomodoro", error, data);
      this.options.toasts.showSaveErrorToast(toastId, this.options.saveErrorLabel(error));
    } finally {
      this.dismissPendingSaveToast(toastId);
      this.clearDisplayState();
    }
  }

  private async executeEnd(data: PanelSaveData, scope?: RecurringScope): Promise<void> {
    const state = this.options.getSessionState();
    if (state.mode !== "edit" || !this.options.isSelectedEndable(state)) return;
    const actualEnd = (this.options.now ?? (() => new Date()))();
    const endedData = { ...data, end: formatCalendarDateWithSeconds(actualEnd) };
    const completesPomodoro = this.options.isSelectedActivePomodoro(state);
    const suppressAutoStart = this.options.pomodoro.isActive && !!this.options.pomodoro.activeBlockId;
    this.endingActiveEvent = true;
    this.options.setDisplayState({
      suppressGlow: true,
      suppressPreview: true,
      frozenEvents: this.options.buildFreeze(endedData, scope),
    });
    if (suppressAutoStart) this.options.pomodoro.autoStartSuppressed = true;
    try {
      let refreshed = false;
      if (!completesPomodoro && !this.options.isRecurring(state.originalEvent)) {
        await this.options.calendarStore.updateBlock({ id: state.originalEvent.id, end: endedData.end });
      } else {
        const result = await this.options.commitService.persist(endedData, scope, {
          syncActivePomodoro: false,
        });
        refreshed = result.saveRefreshedVisibleWindow;
      }
      if (completesPomodoro) {
        await this.options.pomodoro.completeActiveBlockAt(actualEnd.toISOString());
      }
      if (!refreshed) await this.options.refreshWindow();
      this.options.closeSession();
      await this.options.afterRender();
    } finally {
      if (suppressAutoStart) this.options.pomodoro.autoStartSuppressed = false;
      this.endingActiveEvent = false;
      this.clearDisplayState();
    }
  }

  private dismissPendingSaveToast(toastId: string): void {
    if (this.options.toasts.saveSuccessToast?.id === toastId
      && this.options.toasts.saveSuccessToast.pending) {
      this.options.toasts.dismissSaveToastIfCurrent(toastId);
    }
  }

  private clearDisplayState(): void {
    this.options.setDisplayState({
      suppressGlow: false,
      suppressPreview: false,
      frozenEvents: null,
    });
  }
}
