import type { CalendarEvent, EventSurfaceStatus, RecurringScope } from "./types";
import type { EditSessionState } from "./edit-session.svelte";
import type { CreatePreview } from "./edit-session.svelte";
import {
  buildCreateDisplay,
  closedDisplay,
  computeEditDisplay,
} from "./display-events";
import { getEventSurfaceStatusForIdentity } from "./utils";
import type { computeViewWindow } from "./utils";

type ViewWindow = ReturnType<typeof computeViewWindow>;

export function visibleCalendarEvents(input: {
  events: CalendarEvent[];
  visibleCalendarIds: ReadonlySet<string>;
  filter?: (event: CalendarEvent) => boolean;
}): CalendarEvent[] {
  const visible = input.events.filter((event) => input.visibleCalendarIds.has(event.calendarId));
  return input.filter ? visible.filter(input.filter) : visible;
}

export function projectCalendarDisplay(input: {
  rawBlocks: CalendarEvent[];
  storeEvents: CalendarEvent[];
  frozenEvents: CalendarEvent[] | null;
  state: EditSessionState;
  createPreview: CreatePreview | null;
  changes: Partial<CalendarEvent>;
  dirty: boolean;
  scope: RecurringScope;
  window: ViewWindow;
  suppressEditPreview: boolean;
  activeDate: string | undefined;
  currentDate: string;
  currentTime: string;
  activeBlockId: string | undefined;
}) {
  if (input.frozenEvents) return closedDisplay(input.frozenEvents);
  if (input.state.mode === "closed") return closedDisplay(input.storeEvents);
  if (input.state.mode === "create") {
    return buildCreateDisplay(
      input.storeEvents,
      input.createPreview,
      input.changes,
      input.window,
    );
  }
  if (input.suppressEditPreview) return closedDisplay(input.storeEvents);
  return computeEditDisplay(
    input.rawBlocks,
    input.storeEvents,
    {
      originalEvent: input.state.originalEvent,
      instanceEvent: input.state.instanceEvent,
      templateId: input.state.templateId,
    },
    input.dirty ? input.changes : {},
    input.scope,
    input.window,
    input.activeDate,
    input.currentDate,
    input.currentTime,
    input.activeBlockId,
  );
}

export function buildCalendarSaveFreeze(input: {
  rawBlocks: CalendarEvent[];
  storeEvents: CalendarEvent[];
  state: EditSessionState;
  createPreview: CreatePreview | null;
  changes: Partial<CalendarEvent>;
  scope: RecurringScope;
  window: ViewWindow;
  activeDate: string | undefined;
  currentDate: string;
  currentTime: string;
  activeBlockId: string | undefined;
}): CalendarEvent[] {
  return projectCalendarDisplay({
    ...input,
    frozenEvents: null,
    dirty: true,
    suppressEditPreview: false,
  }).events.map((event) => ({ ...event }));
}

export function projectCalendarSurfaceStatuses(input: {
  events: CalendarEvent[];
  identityByCalendarId: ReadonlyMap<string, string>;
  pendingStatus: EventSurfaceStatus | undefined;
  pendingEventId: string | undefined;
}): CalendarEvent[] {
  return input.events.map((event) => {
    if (input.pendingStatus && event.id === input.pendingEventId) {
      return { ...event, surfaceStatus: input.pendingStatus };
    }
    const status = getEventSurfaceStatusForIdentity(
      event,
      input.identityByCalendarId.get(event.calendarId),
    );
    return status === undefined ? event : { ...event, surfaceStatus: status };
  });
}
