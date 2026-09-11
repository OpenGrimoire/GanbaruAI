import type { PomodoroRuntime } from "./pomodoro-runtime";

interface PomodoroOvertimeContext {
  runtime: PomodoroRuntime;
  publishWindowSnapshot(): void;
  playBreakFinishedAlert(): void;
  startConfiguredAlertInterval(): ReturnType<typeof setInterval> | null;
}

export interface PomodoroOvertimeController {
  isActive(): boolean;
  start(): void;
  stop(): void;
}

/** Display waiting time and reminders without authorizing another focus interval. */
export function createPomodoroOvertimeController(
  context: PomodoroOvertimeContext,
): PomodoroOvertimeController {
  let overtimeIntervalId: ReturnType<typeof setInterval> | null = null;
  let alertIntervalId: ReturnType<typeof setInterval> | null = null;
  let alertStarted = false;

  function isActive(): boolean {
    return overtimeIntervalId !== null;
  }

  function start(): void {
    if (overtimeIntervalId !== null) return;
    context.runtime.breakOvertimeSeconds = 0;
    overtimeIntervalId = setInterval(() => {
      context.runtime.breakOvertimeSeconds += 1;
      context.publishWindowSnapshot();
    }, 1_000);
    if (!alertStarted) {
      alertStarted = true;
      context.playBreakFinishedAlert();
      alertIntervalId = context.startConfiguredAlertInterval();
    }
  }

  function stop(): void {
    if (overtimeIntervalId !== null) {
      clearInterval(overtimeIntervalId);
      overtimeIntervalId = null;
    }
    if (alertIntervalId !== null) {
      clearInterval(alertIntervalId);
      alertIntervalId = null;
    }
    alertStarted = false;
    context.runtime.breakOvertimeSeconds = 0;
  }

  return { isActive, start, stop };
}
