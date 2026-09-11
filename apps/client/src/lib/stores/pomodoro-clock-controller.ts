import {
  canPauseResumePomodoro,
  isPomodoroSessionActive,
  limitRemainingSecondsToBlockEnd,
} from "./pomodoro-machine";
import type { PomodoroRuntime } from "./pomodoro-runtime";

export interface PomodoroClockController {
  actualPhaseElapsedSeconds(): number;
  phaseWorkRemainingSeconds(): number;
  setPhaseRemainingSeconds(
    nextWorkRemainingSeconds: number,
    elapsedSeconds?: number,
    nowMs?: number,
  ): number;
  setVisibleRemainingForPause(
    nextVisibleRemainingSeconds: number,
    elapsedSeconds: number,
    nowMs?: number,
  ): number;
  refreshCurrentPhaseLimit(nowMs?: number): void;
  recordRunningPhaseProgress(nextVisibleRemainingSeconds: number): void;
  refreshPausedOpportunityRemaining(nowMs?: number): boolean;
  activeBlockDeadlineReached(nowMs?: number): boolean;
  sessionActive(nowMs?: number): boolean;
  canPauseResume(nowMs?: number): boolean;
  resetPhaseProgress(defaultRemainingSeconds: number): void;
}

/** Owns timer arithmetic while leaving phase-transition policy to the machine. */
export function createPomodoroClockController(
  runtime: PomodoroRuntime,
  now: () => number = Date.now,
): PomodoroClockController {
  function actualPhaseElapsedSeconds(): number {
    return runtime.phaseElapsedSeconds;
  }

  function phaseWorkRemainingSeconds(): number {
    return Math.max(
      0,
      runtime.phaseWorkDurationSeconds - runtime.phaseElapsedSeconds,
    );
  }

  function setPhaseRemainingSeconds(
    nextWorkRemainingSeconds: number,
    elapsedSeconds: number = 0,
    nowMs: number = now(),
  ): number {
    const normalizedElapsedSeconds = Math.max(0, Math.ceil(elapsedSeconds));
    const normalizedWorkRemainingSeconds = Math.max(
      0,
      Math.ceil(nextWorkRemainingSeconds),
    );
    const limitedRemaining = limitRemainingSecondsToBlockEnd(
      normalizedWorkRemainingSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    runtime.phaseElapsedSeconds = normalizedElapsedSeconds;
    runtime.phaseWorkDurationSeconds =
      normalizedElapsedSeconds + normalizedWorkRemainingSeconds;
    runtime.remainingSeconds = limitedRemaining;
    runtime.phaseTotalSeconds = normalizedElapsedSeconds + limitedRemaining;
    return limitedRemaining;
  }

  function setVisibleRemainingForPause(
    nextVisibleRemainingSeconds: number,
    elapsedSeconds: number,
    nowMs: number = now(),
  ): number {
    const normalizedElapsedSeconds = Math.max(0, Math.ceil(elapsedSeconds));
    const limitedRemaining = limitRemainingSecondsToBlockEnd(
      nextVisibleRemainingSeconds,
      runtime.activeBlockEndMs,
      nowMs,
    );
    runtime.phaseElapsedSeconds = normalizedElapsedSeconds;
    runtime.remainingSeconds = limitedRemaining;
    runtime.phaseTotalSeconds = normalizedElapsedSeconds + limitedRemaining;
    return limitedRemaining;
  }

  function refreshCurrentPhaseLimit(nowMs: number = now()): void {
    const elapsedSeconds = actualPhaseElapsedSeconds();
    const limitedRemaining = setPhaseRemainingSeconds(
      phaseWorkRemainingSeconds(),
      elapsedSeconds,
      nowMs,
    );
    if (runtime.isRunning) {
      runtime.phaseEndTime = nowMs + limitedRemaining * 1000;
    }
  }

  function recordRunningPhaseProgress(
    nextVisibleRemainingSeconds: number,
  ): void {
    const normalizedRemainingSeconds = Math.max(
      0,
      Math.ceil(nextVisibleRemainingSeconds),
    );
    const elapsedDeltaSeconds = Math.max(
      0,
      runtime.remainingSeconds - normalizedRemainingSeconds,
    );
    runtime.phaseElapsedSeconds = Math.min(
      runtime.phaseWorkDurationSeconds,
      runtime.phaseElapsedSeconds + elapsedDeltaSeconds,
    );
    runtime.remainingSeconds = normalizedRemainingSeconds;
  }

  function refreshPausedOpportunityRemaining(nowMs: number = now()): boolean {
    const nextRemainingSeconds = limitRemainingSecondsToBlockEnd(
      phaseWorkRemainingSeconds(),
      runtime.activeBlockEndMs,
      nowMs,
    );
    if (nextRemainingSeconds === runtime.remainingSeconds) return false;
    runtime.remainingSeconds = nextRemainingSeconds;
    return true;
  }

  function activeBlockDeadlineReached(nowMs: number = now()): boolean {
    return runtime.activeBlockEndMs !== null && nowMs >= runtime.activeBlockEndMs;
  }

  function sessionActive(nowMs: number = now()): boolean {
    return isPomodoroSessionActive({
      activeBlockId: runtime.activeBlockId,
      activeBlockEndMs: runtime.activeBlockEndMs,
      blockExpired: runtime.blockExpired,
      isRunning: runtime.isRunning,
      remainingSeconds: runtime.remainingSeconds,
      totalSeconds: runtime.phaseTotalSeconds,
      nowMs,
    });
  }

  function canPauseResume(nowMs: number = now()): boolean {
    return canPauseResumePomodoro({
      phase: runtime.phase,
      activeBlockId: runtime.activeBlockId,
      activeBlockEndMs: runtime.activeBlockEndMs,
      blockExpired: runtime.blockExpired,
      isRunning: runtime.isRunning,
      remainingSeconds: runtime.remainingSeconds,
      totalSeconds: runtime.phaseTotalSeconds,
      nowMs,
      suspendedAway: runtime.suspendedAway !== null,
      idlePaused: runtime.idlePaused !== null,
    });
  }

  function resetPhaseProgress(defaultRemainingSeconds: number): void {
    runtime.phaseElapsedSeconds = 0;
    runtime.phaseWorkDurationSeconds = defaultRemainingSeconds;
    runtime.remainingSeconds = defaultRemainingSeconds;
    runtime.phaseTotalSeconds = defaultRemainingSeconds;
  }

  return {
    actualPhaseElapsedSeconds,
    phaseWorkRemainingSeconds,
    setPhaseRemainingSeconds,
    setVisibleRemainingForPause,
    refreshCurrentPhaseLimit,
    recordRunningPhaseProgress,
    refreshPausedOpportunityRemaining,
    activeBlockDeadlineReached,
    sessionActive,
    canPauseResume,
    resetPhaseProgress,
  };
}
