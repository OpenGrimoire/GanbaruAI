import type { PomodoroPhase } from "@ganbaru-ai/shared-types";

export interface PomodoroNativeTrayState {
  phase: PomodoroPhase;
  remainingSeconds: number;
  totalSeconds: number;
  isRunning: boolean;
  isActive: boolean;
  canPauseResume: boolean;
  canAddFocusTime: boolean;
  pausedPulseFrame: number | null;
}

function progressStep(state: PomodoroNativeTrayState): number {
  if (!state.isActive || state.totalSeconds <= 0) return 100;
  const progress = 1 - state.remainingSeconds / state.totalSeconds;
  return Math.floor(Math.max(0, Math.min(1, progress)) * 100);
}

function structuralKey(state: PomodoroNativeTrayState): string {
  return [
    state.phase,
    state.totalSeconds,
    state.isRunning ? 1 : 0,
    state.isActive ? 1 : 0,
    state.canPauseResume ? 1 : 0,
    state.canAddFocusTime ? 1 : 0,
    state.pausedPulseFrame ?? "",
  ].join("|");
}

/** Coalesce native tray work to visible state transitions and percent steps. */
export function createPomodoroNativeTrayPolicy(): {
  shouldSend(state: PomodoroNativeTrayState): boolean;
  reset(): void;
} {
  let previousStructuralKey: string | undefined;
  let previousProgressStep: number | undefined;
  return {
    shouldSend(state) {
      const nextStructuralKey = structuralKey(state);
      const nextProgressStep = progressStep(state);
      if (
        nextStructuralKey === previousStructuralKey
        && nextProgressStep === previousProgressStep
      ) return false;
      previousStructuralKey = nextStructuralKey;
      previousProgressStep = nextProgressStep;
      return true;
    },
    reset() {
      previousStructuralKey = undefined;
      previousProgressStep = undefined;
    },
  };
}
