import type { CalendarEvent, RecurringScope } from "./types";
import type { EditSessionState } from "./edit-session.svelte";
import type { PanelSaveData } from "./event-panel-payloads";
import { buildRecurringCommitPlan } from "./recurrence-edit-plan";
import { executeRecurrenceCommitPlan } from "./recurrence-edit-executor";
import { computeViewWindow, formatCalendarDate, formatDatePart } from "./utils";
import type { getCalendar } from "$lib/stores/calendar.svelte";
import type { getPomodoro } from "$lib/stores/pomodoro.svelte";

type CalendarStore = ReturnType<typeof getCalendar>;
type PomodoroStore = ReturnType<typeof getPomodoro>;
type ViewWindow = ReturnType<typeof computeViewWindow>;

export interface CalendarPanelPersistResult {
  saveRefreshedVisibleWindow: boolean;
}

export function calendarDataOnly(data: PanelSaveData): PanelSaveData {
  const { linkedTaskIds: _linkedTaskIds, ...calendarData } = data;
  return calendarData;
}

export interface CalendarViewCommitServiceOptions {
  calendarStore: CalendarStore;
  pomodoro: PomodoroStore;
  getSessionState: () => EditSessionState;
  getViewWindow: () => ViewWindow;
  isRecurring: (event: CalendarEvent) => boolean;
  effectiveScope: (
    state: Extract<EditSessionState, { mode: "edit" }>,
    requested?: RecurringScope,
  ) => RecurringScope;
  activeDate: (templateId: string) => string | undefined;
  syncSavedActivePomodoro: (event: CalendarEvent) => Promise<void>;
  now?: () => Date;
}

/** Owns create, direct edit, and atomic recurrence persistence decisions. */
export class CalendarViewCommitService {
  constructor(private readonly options: CalendarViewCommitServiceOptions) {}

  async persist(
    data: PanelSaveData,
    scope?: RecurringScope,
    settings: { syncActivePomodoro?: boolean } = {},
  ): Promise<CalendarPanelPersistResult> {
    const state = this.options.getSessionState();
    const calendarData = calendarDataOnly(data);
    if (state.mode === "closed") return { saveRefreshedVisibleWindow: false };
    if (state.mode === "create") {
      await this.options.calendarStore.addBlock(calendarData);
      return { saveRefreshedVisibleWindow: false };
    }

    if (this.options.isRecurring(state.originalEvent)) {
      return this.persistRecurrence(state, calendarData, scope);
    }

    const updated: CalendarEvent = { ...state.originalEvent, ...calendarData };
    await this.options.calendarStore.updateBlock(updated);
    if (settings.syncActivePomodoro ?? true) {
      await this.options.syncSavedActivePomodoro(updated);
    }
    return { saveRefreshedVisibleWindow: false };
  }

  private async persistRecurrence(
    state: Extract<EditSessionState, { mode: "edit" }>,
    changes: PanelSaveData,
    requestedScope?: RecurringScope,
  ): Promise<CalendarPanelPersistResult> {
    const scope = this.options.effectiveScope(state, requestedScope);
    const now = (this.options.now ?? (() => new Date()))();
    const activeDate = this.options.activeDate(state.templateId);
    const plan = buildRecurringCommitPlan({
      rawBlocks: this.options.calendarStore.rawBlocks,
      templateId: state.templateId,
      instanceEvent: state.instanceEvent,
      changes,
      scope,
      activeBlockId: this.options.pomodoro.isActive && activeDate
        ? this.options.pomodoro.activeBlockId ?? undefined
        : undefined,
      activeDate,
      today: formatDatePart(now),
      currentTime: formatCalendarDate(now).split(" ")[1],
    });
    const blocking = plan.diagnostics.find((diagnostic) => diagnostic.severity === "error");
    if (blocking) throw new Error(blocking.message);
    await executeRecurrenceCommitPlan(plan, {
      calendarStore: this.options.calendarStore,
      pomodoro: this.options.pomodoro,
      window: this.options.getViewWindow(),
    });
    return { saveRefreshedVisibleWindow: plan.requiresCanonicalRefresh };
  }
}
