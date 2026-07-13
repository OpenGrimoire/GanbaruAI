import { clonePomodoroConfig, rhythmPositionCount } from "$lib/pomodoro/rhythm";
import type { TimerSnapshot } from "./pomodoro-machine";
import type { PomodoroRuntime } from "./pomodoro-runtime";
import {
  cloneSegmentsForWindowSync,
  type PomodoroWindowSnapshot,
} from "./pomodoro-window-sync";

export interface PomodoroWindowStateController {
  buildTimerSnapshot(): TimerSnapshot;
  buildWindowSnapshot(): PomodoroWindowSnapshot;
  applyWindowSnapshot(snapshot: PomodoroWindowSnapshot): void;
}

/** Owns serialization between shared runtime state and window snapshots. */
export function createPomodoroWindowStateController(
  runtime: PomodoroRuntime,
  isCoordinator: () => boolean,
): PomodoroWindowStateController {
  function buildTimerSnapshot(): TimerSnapshot {
    return {
      phase: runtime.phase,
      remainingSeconds: runtime.remainingSeconds,
      currentRhythmPosition: runtime.currentRhythmPosition,
      config: runtime.config,
      skipNextBreak: runtime.skipNextBreak,
      notificationShown: runtime.notificationShown,
      phaseEndTime: runtime.phaseEndTime,
      activeBlockEndMs: runtime.activeBlockEndMs,
      lastTickMs: runtime.lastTickMs,
    };
  }

  function buildWindowSnapshot(): PomodoroWindowSnapshot {
    return {
      phase: runtime.phase,
      remainingSeconds: runtime.remainingSeconds,
      phaseTotalSeconds: runtime.phaseTotalSeconds,
      phaseWorkDurationSeconds: runtime.phaseWorkDurationSeconds,
      currentRhythmPosition: runtime.currentRhythmPosition,
      totalRhythmPositions: rhythmPositionCount(runtime.config),
      isRunning: runtime.isRunning,
      config: clonePomodoroConfig(runtime.config),
      completedPomodoros: runtime.completedPomodoros,
      activeBlockId: runtime.activeBlockId,
      activeRunId: runtime.activeRunId,
      activeBlockEndMs: runtime.activeBlockEndMs,
      dismissedBlockId: runtime.dismissedBlockId,
      breakOvertimeSeconds: runtime.breakOvertimeSeconds,
      segments: cloneSegmentsForWindowSync(runtime.segments),
      segmentVersion: runtime.segmentVersion,
      blockExpired: runtime.blockExpired,
      focusExtensionUsed: runtime.focusExtensionUsed,
      suspendedAway: runtime.suspendedAway ? { ...runtime.suspendedAway } : null,
      idlePaused: runtime.idlePaused ? { ...runtime.idlePaused } : null,
    };
  }

  function applyWindowSnapshot(snapshot: PomodoroWindowSnapshot): void {
    if (isCoordinator()) return;
    runtime.phase = snapshot.phase;
    runtime.remainingSeconds = snapshot.remainingSeconds;
    runtime.phaseTotalSeconds = snapshot.phaseTotalSeconds;
    runtime.phaseWorkDurationSeconds = snapshot.phaseWorkDurationSeconds;
    runtime.currentRhythmPosition = snapshot.currentRhythmPosition;
    runtime.isRunning = snapshot.isRunning;
    runtime.config = clonePomodoroConfig(snapshot.config);
    runtime.completedPomodoros = snapshot.completedPomodoros;
    runtime.activeBlockId = snapshot.activeBlockId;
    runtime.activeRunId = snapshot.activeRunId;
    runtime.activeBlockEndMs = snapshot.activeBlockEndMs;
    runtime.dismissedBlockId = snapshot.dismissedBlockId;
    runtime.breakOvertimeSeconds = snapshot.breakOvertimeSeconds;
    runtime.segments = cloneSegmentsForWindowSync(snapshot.segments);
    runtime.currentSegmentIndex = runtime.segments.findIndex(
      (segment) => segment.status === "active",
    );
    runtime.segmentVersion = snapshot.segmentVersion;
    runtime.blockExpired = snapshot.blockExpired;
    runtime.focusExtensionUsed = snapshot.focusExtensionUsed;
    runtime.suspendedAway = snapshot.suspendedAway
      ? { ...snapshot.suspendedAway }
      : null;
    runtime.idlePaused = snapshot.idlePaused ? { ...snapshot.idlePaused } : null;
  }

  return { buildTimerSnapshot, buildWindowSnapshot, applyWindowSnapshot };
}
