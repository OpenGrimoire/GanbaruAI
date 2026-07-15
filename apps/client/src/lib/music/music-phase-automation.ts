import type { CalendarEvent } from "$lib/components/calendar/types";
import { parseCalendarDate } from "$lib/components/calendar/utils";
import type { MusicActivityPhase } from "./music-context-assignment";

export interface MusicPhaseObservation {
  eventId: string;
  phase: MusicActivityPhase;
  phaseToken: string;
  suspended: boolean;
}

export interface MusicPhaseActivation {
  key: string;
  eventId: string;
  phase: MusicActivityPhase;
  activation: "boundary" | "catch-up";
  consecutiveEvent: boolean;
}

/** Plans exactly one soundtrack action for each event and phase boundary. */
export class MusicPhaseAutomationPlanner {
  private currentKey: string | null = null;
  private currentEventId: string | null = null;
  private initialized = false;
  private pendingSuspendedKey: string | null = null;

  observe(observation: MusicPhaseObservation | null): MusicPhaseActivation | null {
    if (!observation) {
      this.currentKey = null;
      this.currentEventId = null;
      return null;
    }
    const key = musicPhaseActivationKey(observation);
    if (observation.suspended) {
      if (key !== this.currentKey) this.pendingSuspendedKey = key;
      return null;
    }
    if (key === this.currentKey) return null;
    const previousEventId = this.currentEventId;
    const caughtUpAfterSuspend = this.pendingSuspendedKey === key;
    this.pendingSuspendedKey = null;
    this.currentKey = key;
    this.currentEventId = observation.eventId;
    const activation = !this.initialized || caughtUpAfterSuspend ? "catch-up" : "boundary";
    this.initialized = true;
    return {
      key,
      eventId: observation.eventId,
      phase: observation.phase,
      activation,
      consecutiveEvent: previousEventId !== null && previousEventId !== observation.eventId,
    };
  }

  isCurrent(key: string): boolean {
    return this.currentKey === key;
  }

  retry(key: string): MusicPhaseActivation | null {
    if (this.currentKey !== key || !this.currentEventId) return null;
    const phase = phaseFromActivationKey(key);
    if (!phase) return null;
    return {
      key,
      eventId: this.currentEventId,
      phase,
      activation: "catch-up",
      consecutiveEvent: false,
    };
  }
}

export function musicPhaseActivationKey(observation: MusicPhaseObservation): string {
  return `${observation.eventId}|${observation.phase}|${observation.phaseToken}`;
}

function phaseFromActivationKey(key: string): MusicActivityPhase | null {
  const phase = key.split("|")[1];
  return phase === "focus" || phase === "short-break" || phase === "long-break" ? phase : null;
}

export interface ActiveMusicEventOptions {
  nowMs: number;
  activeBlockId: string | null;
}

/** Selects the same authoritative active block first, then a deterministic timed-event winner. */
export function selectActiveMusicEvent(
  events: readonly CalendarEvent[],
  options: ActiveMusicEventOptions,
): CalendarEvent | null {
  const active = options.activeBlockId
    ? events.find((event) => event.id === options.activeBlockId)
      ?? events.find((event) => event.recurringParentId === options.activeBlockId)
    : undefined;
  if (active && !active.allDay) return active;

  const candidates = events.flatMap((event) => {
    if (event.allDay) return [];
    const startMs = parseCalendarDate(event.start).getTime();
    const endMs = parseCalendarDate(event.end).getTime();
    if (!Number.isFinite(startMs) || !Number.isFinite(endMs)) return [];
    if (options.nowMs < startMs || options.nowMs >= endMs) return [];
    return [{ event, endMs }];
  });
  candidates.sort((left, right) =>
    left.endMs - right.endMs
    || (left.event.createdAt ?? "").localeCompare(right.event.createdAt ?? "")
    || left.event.id.localeCompare(right.event.id));
  return candidates[0]?.event ?? null;
}

export function nextMusicEventBoundaryMs(events: readonly CalendarEvent[], nowMs: number): number | null {
  let boundary: number | null = null;
  for (const event of events) {
    if (event.allDay) continue;
    for (const value of [event.start, event.end]) {
      const candidate = parseCalendarDate(value).getTime();
      if (!Number.isFinite(candidate) || candidate <= nowMs) continue;
      boundary = boundary === null ? candidate : Math.min(boundary, candidate);
    }
  }
  return boundary;
}
