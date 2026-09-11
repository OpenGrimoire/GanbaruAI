import type { PomodoroWindowCommand } from "./pomodoro-window-sync";
import type {
  PomodoroWindowCoordinator,
  PomodoroWindowCoordinatorContext,
} from "./pomodoro-window-coordinator-contract";

export type {
  PomodoroWindowCoordinator,
  PomodoroWindowCoordinatorContext,
} from "./pomodoro-window-coordinator-contract";

/** Avoid desktop window synchronization listeners and emissions on the one-webview mobile shell. */
export function createPomodoroWindowCoordinator(
  context: PomodoroWindowCoordinatorContext,
): PomodoroWindowCoordinator {
  void context;
  return {
    publishSnapshot(): void {},
    sendCommand(_command: PomodoroWindowCommand): void {},
    forwardCommand(_command: PomodoroWindowCommand): boolean {
      return false;
    },
    init(): void {},
  };
}
