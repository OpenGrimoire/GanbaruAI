import { writeDoomscrollingRuntimeState } from "$lib/api/doomscrolling";
import { nowIso } from "./pomodoro-backend-writes";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";

type DoomscrollingPauseReason = "manual" | "idle" | "suspend";

export interface PomodoroDoomscrollingController {
  writeCurrentState(force?: boolean): void;
}

interface PomodoroDoomscrollingControllerOptions {
  isCoordinator(): boolean;
  writeState?: typeof writeDoomscrollingRuntimeState;
  now?: () => string;
}

/** Derives and deduplicates the blocker runtime projection. */
export function createPomodoroDoomscrollingController(
  runtime: PomodoroRuntime,
  clock: Pick<PomodoroClockController, "sessionActive">,
  options: PomodoroDoomscrollingControllerOptions,
): PomodoroDoomscrollingController {
  const writeState = options.writeState ?? writeDoomscrollingRuntimeState;
  const currentIso = options.now ?? nowIso;
  let lastStateKey = "";

  function pauseReason(paused: boolean): DoomscrollingPauseReason | null {
    if (!paused) return null;
    if (runtime.idlePaused) return "idle";
    if (runtime.suspendedAway) return "suspend";
    return "manual";
  }

  function writeCurrentState(force = false): void {
    if (!options.isCoordinator()) return;
    const minuteBucket = Math.ceil(runtime.remainingSeconds / 60);
    const active = clock.sessionActive();
    const paused = active && !runtime.isRunning;
    const currentPauseReason = pauseReason(paused);
    const stateKey = [
      active ? "1" : "0",
      paused ? "1" : "0",
      currentPauseReason ?? "",
      active ? runtime.phase : "inactive",
      runtime.activeRunId ?? "",
      runtime.activeBlockId ?? "",
      String(minuteBucket),
    ].join("|");
    if (!force && stateKey === lastStateKey) return;
    lastStateKey = stateKey;

    writeState({
      active,
      paused,
      pauseReason: currentPauseReason,
      phase: active ? runtime.phase : "inactive",
      activeRunId: runtime.activeRunId,
      activeBlockId: runtime.activeBlockId,
      remainingSeconds: active ? runtime.remainingSeconds : null,
      updatedAt: currentIso(),
    }).catch((error: unknown) => {
      console.warn("doomscrolling state write failed", error);
    });
  }

  return { writeCurrentState };
}
