import type { PomodoroRuntime } from "./pomodoro-runtime";
import { DEFAULT_CONFIG } from "./pomodoro-machine";
import { clonePomodoroConfig } from "$lib/pomodoro/rhythm";

/** Builds isolated mutable state for Pomodoro controller tests. */
export function createPomodoroRuntimeFixture(
  overrides: Partial<PomodoroRuntime> = {},
): PomodoroRuntime {
  return {
    phase: "focus",
    remainingSeconds: 1_500,
    phaseTotalSeconds: 1_500,
    phaseElapsedSeconds: 0,
    phaseWorkDurationSeconds: 1_500,
    currentRhythmPosition: 1,
    isRunning: false,
    config: clonePomodoroConfig(DEFAULT_CONFIG),
    completedPomodoros: 0,
    sessionStartTime: null,
    skipNextBreak: false,
    notificationShown: false,
    focusExtensionUsed: false,
    phaseEndTime: null,
    activeBlockId: null,
    activeRunId: null,
    activeBlockEndMs: null,
    dismissedBlockId: null,
    autoStartSuppressed: false,
    segments: [],
    currentSegmentIndex: -1,
    segmentEndReasons: new Map(),
    segmentVersion: 0,
    breakOvertimeSeconds: 0,
    blockExpired: false,
    lastTickMs: null,
    suspendedAway: null,
    idleTimeoutMs: null,
    idlePaused: null,
    ...overrides,
  };
}
