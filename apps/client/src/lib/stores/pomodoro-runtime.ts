import type { PomodoroPhase } from "@ganbaru-ai/shared-types";
import type { PersistedSegment } from "$lib/components/calendar/types";
import type { PomodoroSegmentEndReason } from "./pomodoro-backend-writes";
import type { PomodoroConfig } from "./pomodoro-machine";
import type { IdlePauseState } from "./pomodoro-window-sync";

/** Shared mutable timer state exposed to lifecycle controllers. */
export interface PomodoroRuntime {
  phase: PomodoroPhase;
  remainingSeconds: number;
  phaseTotalSeconds: number;
  phaseElapsedSeconds: number;
  phaseWorkDurationSeconds: number;
  currentRhythmPosition: number;
  isRunning: boolean;
  config: PomodoroConfig;
  completedPomodoros: number;
  sessionStartTime: string | null;
  skipNextBreak: boolean;
  notificationShown: boolean;
  focusExtensionUsed: boolean;
  phaseEndTime: number | null;
  activeBlockId: string | null;
  activeBlockTitle: string | null;
  activeRunId: string | null;
  activeBlockEndMs: number | null;
  dismissedBlockId: string | null;
  autoStartSuppressed: boolean;
  segments: PersistedSegment[];
  currentSegmentIndex: number;
  segmentEndReasons: Map<string, PomodoroSegmentEndReason>;
  segmentVersion: number;
  breakOvertimeSeconds: number;
  blockExpired: boolean;
  lastTickMs: number | null;
  suspendedAway: { awaySeconds: number } | null;
  idleTimeoutMs: number | null;
  idlePaused: IdlePauseState | null;
}
