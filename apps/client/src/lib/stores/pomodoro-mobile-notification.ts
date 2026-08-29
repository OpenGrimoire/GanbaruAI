import type { PomodoroPhase } from "@ganbaru-ai/shared-types";
import type { PersistedSegment } from "$lib/components/calendar/types";
import {
  phaseDurationMinutesAtPosition,
  type PomodoroConfig,
} from "$lib/pomodoro/rhythm";

const MAX_NATIVE_PHASES = 128;
const MAX_NATIVE_EVENT_TITLE_LENGTH = 160;

export interface MobilePomodoroNotificationPhase {
  id: string;
  phase: PomodoroPhase;
  rhythmPosition: number;
  startsAtEpochMs: number;
  endsAtEpochMs: number;
}

export interface MobilePomodoroNotificationCopy {
  channelName: string;
  channelDescription: string;
  alertsChannelName: string;
  alertsChannelDescription: string;
  focusTitle: string;
  shortBreakTitle: string;
  longBreakTitle: string;
  pausedText: string;
  focusCompleteTitle: string;
  breakCompleteTitle: string;
  sessionCompleteText: string;
}

export interface MobilePomodoroNotificationState {
  runId: string;
  eventId: string;
  eventTitle: string | null;
  eventDate: string;
  eventEndsAtEpochMs: number;
  generatedAtEpochMs: number;
  isRunning: boolean;
  remainingSeconds: number;
  totalSeconds: number;
  configJson: string;
  phases: MobilePomodoroNotificationPhase[];
  copy: MobilePomodoroNotificationCopy;
}

interface BuildMobilePomodoroNotificationStateInput {
  activeRunId: string | null;
  activeBlockId: string | null;
  activeBlockTitle: string | null;
  activeBlockEndMs: number | null;
  phaseEndTime: number | null;
  remainingSeconds: number;
  totalSeconds: number;
  isRunning: boolean;
  skipNextBreak: boolean;
  config: PomodoroConfig;
  segments: readonly PersistedSegment[];
  currentSegmentIndex: number;
  copy: MobilePomodoroNotificationCopy;
  nowMs?: number;
}

function timestamp(value: string): number | null {
  const result = Date.parse(value);
  return Number.isFinite(result) ? result : null;
}

function notificationEventTitle(value: string | null): string | null {
  const trimmed = value?.trim() ?? "";
  if (trimmed.length === 0) return null;
  let result = "";
  for (const character of trimmed) {
    if (result.length + character.length > MAX_NATIVE_EVENT_TITLE_LENGTH) break;
    result += character;
  }
  return result;
}

/** Build the bounded native phase projection used by Android alarms and cold recovery. */
export function buildMobilePomodoroNotificationState(
  input: BuildMobilePomodoroNotificationStateInput,
): MobilePomodoroNotificationState | null {
  const nowMs = input.nowMs ?? Date.now();
  const eventTitle = notificationEventTitle(input.activeBlockTitle);
  const current = input.segments[input.currentSegmentIndex];
  if (
    !input.activeRunId
    || !input.activeBlockId
    || !current
    || current.runId !== input.activeRunId
    || current.status !== "active"
    || input.activeBlockEndMs === null
    || !Number.isFinite(input.activeBlockEndMs)
    || input.activeBlockEndMs <= nowMs
    || !Number.isInteger(input.remainingSeconds)
    || !Number.isInteger(input.totalSeconds)
    || input.remainingSeconds < 0
    || input.totalSeconds <= 0
    || input.remainingSeconds > input.totalSeconds
    || input.skipNextBreak
  ) return null;

  const activeStartMs = timestamp(current.actualStart ?? "");
  if (activeStartMs === null) return null;

  if (!input.isRunning || input.phaseEndTime === null) {
    return {
      runId: input.activeRunId,
      eventId: input.activeBlockId,
      eventTitle,
      eventDate: current.eventDate,
      eventEndsAtEpochMs: input.activeBlockEndMs,
      generatedAtEpochMs: nowMs,
      isRunning: false,
      remainingSeconds: input.remainingSeconds,
      totalSeconds: input.totalSeconds,
      configJson: JSON.stringify(input.config),
      phases: [{
        id: current.id,
        phase: current.phase,
        rhythmPosition: current.rhythmPosition,
        startsAtEpochMs: activeStartMs,
        endsAtEpochMs: input.activeBlockEndMs,
      }],
      copy: input.copy,
    };
  }

  const currentEndMs = Math.min(input.phaseEndTime, input.activeBlockEndMs);
  if (!Number.isFinite(currentEndMs) || currentEndMs <= nowMs) return null;
  const phases: MobilePomodoroNotificationPhase[] = [{
    id: current.id,
    phase: current.phase,
    rhythmPosition: current.rhythmPosition,
    startsAtEpochMs: activeStartMs,
    endsAtEpochMs: currentEndMs,
  }];
  let nextStartMs = currentEndMs;

  for (
    let index = input.currentSegmentIndex + 1;
    index < input.segments.length && phases.length < MAX_NATIVE_PHASES;
    index += 1
  ) {
    const segment = input.segments[index];
    if (segment.status !== "planned" || segment.runId !== input.activeRunId) continue;
    const durationMs = phaseDurationMinutesAtPosition(
      segment.phase,
      input.config,
      segment.rhythmPosition,
    ) * 60_000;
    if (durationMs <= 0) return null;
    const nextEndMs = Math.min(nextStartMs + durationMs, input.activeBlockEndMs);
    if (nextEndMs <= nextStartMs) break;
    phases.push({
      id: segment.id,
      phase: segment.phase,
      rhythmPosition: segment.rhythmPosition,
      startsAtEpochMs: nextStartMs,
      endsAtEpochMs: nextEndMs,
    });
    nextStartMs = nextEndMs;
    if (nextStartMs >= input.activeBlockEndMs) break;
  }

  if (phases.at(-1)?.endsAtEpochMs !== input.activeBlockEndMs) return null;
  return {
    runId: input.activeRunId,
    eventId: input.activeBlockId,
    eventTitle,
    eventDate: current.eventDate,
    eventEndsAtEpochMs: input.activeBlockEndMs,
    generatedAtEpochMs: nowMs,
    isRunning: true,
    remainingSeconds: input.remainingSeconds,
    totalSeconds: input.totalSeconds,
    configJson: JSON.stringify(input.config),
    phases,
    copy: input.copy,
  };
}
