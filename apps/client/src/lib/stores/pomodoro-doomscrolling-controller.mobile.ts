import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";

interface MobileDoomscrollingControllerOptions {
  isCoordinator(): boolean;
}

/** Keep desktop process-blocking code out of the Android timer composition. */
export function createPomodoroDoomscrollingController(
  _runtime: PomodoroRuntime,
  _clock: Pick<PomodoroClockController, "sessionActive">,
  _options: MobileDoomscrollingControllerOptions,
): { writeCurrentState(force?: boolean): void } {
  return { writeCurrentState: () => {} };
}
