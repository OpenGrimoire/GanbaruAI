import { invoke } from "@tauri-apps/api/core";
import type { CalendarEvent } from "$lib/components/calendar/types";
import { parseCalendarDate } from "$lib/components/calendar/utils";
import type { Translate } from "$lib/i18n/translator.svelte";
import type {
  MobilePomodoroNotificationCopy,
  MobilePomodoroNotificationState,
} from "$lib/stores/pomodoro-mobile-notification";
import { computePlannedSegments } from "$lib/utils/pomodoro-segments";
import { loadNotificationSchedulerEvents } from "./mobile-calendar-notifications";

const MAX_NATIVE_ACTIVATIONS = 128;
const MAX_EVENT_TITLE_LENGTH = 160;

function stableHash(value: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function boundedTitle(value: string): string | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) return null;
  let result = "";
  for (const character of trimmed) {
    if (result.length + character.length > MAX_EVENT_TITLE_LENGTH) break;
    result += character;
  }
  return result;
}

function scheduleCopy(t: Translate): MobilePomodoroNotificationCopy {
  return {
    channelName: t("pomodoroNotification.channelName"),
    channelDescription: t("pomodoroNotification.channelDescription"),
    alertsChannelName: t("pomodoroNotification.alertsChannelName"),
    alertsChannelDescription: t("pomodoroNotification.alertsChannelDescription"),
    focusTitle: t("pomodoroNotification.focusTitle"),
    shortBreakTitle: t("pomodoroNotification.shortBreakTitle"),
    longBreakTitle: t("pomodoroNotification.longBreakTitle"),
    pausedText: t("pomodoroNotification.pausedText"),
    focusCompleteTitle: t("pomodoroNotification.focusCompleteTitle"),
    breakCompleteTitle: t("pomodoroNotification.breakCompleteTitle"),
    sessionCompleteText: t("pomodoroNotification.sessionCompleteText"),
  };
}

/** Build durable Android activation projections for future and currently active Focus events. */
export function buildMobilePomodoroSchedule(
  events: readonly CalendarEvent[],
  t: Translate,
  nowMs: number = Date.now(),
): MobilePomodoroNotificationState[] {
  const copy = scheduleCopy(t);
  return events
    .filter((event) => event.pomodoroConfig && !event.allDay && event.status !== "cancelled")
    .map((event): MobilePomodoroNotificationState | null => {
      if (event.id.length === 0 || event.id.length > 256) return null;
      const startsAtEpochMs = parseCalendarDate(event.start).getTime();
      const eventEndsAtEpochMs = parseCalendarDate(event.end).getTime();
      if (
        !Number.isFinite(startsAtEpochMs)
        || !Number.isFinite(eventEndsAtEpochMs)
        || eventEndsAtEpochMs <= Math.max(startsAtEpochMs, nowMs)
      ) return null;
      const durationMinutes = (eventEndsAtEpochMs - startsAtEpochMs) / 60_000;
      const planned = computePlannedSegments(event.pomodoroConfig!, durationMinutes);
      if (planned.length === 0 || planned.length > 128) return null;
      const occurrenceKey = `${event.id}\u0000${event.start}\u0000${event.end}`;
      const runId = `scheduled-${stableHash(occurrenceKey)}`;
      const phases = planned.map((segment, index) => ({
        id: `${runId}-${index + 1}`,
        phase: segment.phase,
        rhythmPosition: segment.rhythmPosition,
        startsAtEpochMs: startsAtEpochMs + segment.startOffsetMinutes * 60_000,
        endsAtEpochMs: startsAtEpochMs + segment.endOffsetMinutes * 60_000,
      }));
      const first = phases[0];
      const firstDurationSeconds = Math.max(
        1,
        Math.round((first.endsAtEpochMs - first.startsAtEpochMs) / 1_000),
      );
      return {
        runId,
        eventId: event.id,
        eventTitle: boundedTitle(event.title),
        eventDate: event.start.split(" ")[0],
        eventEndsAtEpochMs,
        generatedAtEpochMs: Math.min(nowMs, eventEndsAtEpochMs - 1),
        isRunning: true,
        remainingSeconds: firstDurationSeconds,
        totalSeconds: firstDurationSeconds,
        configJson: JSON.stringify(event.pomodoroConfig),
        phases,
        copy,
      };
    })
    .filter((projection): projection is MobilePomodoroNotificationState => projection !== null)
    .sort((left, right) => {
      return left.phases[0].startsAtEpochMs - right.phases[0].startsAtEpochMs
        || left.eventEndsAtEpochMs - right.eventEndsAtEpochMs
        || left.runId.localeCompare(right.runId);
    })
    .slice(0, MAX_NATIVE_ACTIVATIONS);
}

/** Reconcile Android's durable Focus activation alarms with Calendar. */
export async function reconcileMobilePomodoroSchedule(t: Translate): Promise<void> {
  const events = await loadNotificationSchedulerEvents();
  const schedule = buildMobilePomodoroSchedule(events, t);
  await invoke("plugin:ganbaru-mobile-notifications|reconcilePomodoroSchedule", { schedule });
}

/** Serialize Android Focus schedule reconciliation across rapid Calendar mutations. */
export class MobilePomodoroScheduleScheduler {
  private active: Promise<void> | null = null;
  private rerunRequested = false;

  constructor(private readonly t: Translate) {}

  reconcile(): Promise<void> {
    if (this.active) {
      this.rerunRequested = true;
      return this.active;
    }
    this.active = reconcileMobilePomodoroSchedule(this.t).finally(() => {
      this.active = null;
      if (this.rerunRequested) {
        this.rerunRequested = false;
        void this.reconcile();
      }
    });
    return this.active;
  }
}
