import { MAX_BREAK_OVERTIME_SECONDS } from "./pomodoro-machine";
import type { PomodoroRuntime } from "./pomodoro-runtime";

interface PomodoroOvertimeContext {
  runtime: PomodoroRuntime;
  publishWindowSnapshot(): void;
  playBreakFinishedAlert(): void;
  startConfiguredAlertInterval(): ReturnType<typeof setInterval> | null;
  completeOvertimeBreak(): Promise<void>;
}

export interface PomodoroOvertimeController {
  isActive(): boolean;
  start(): void;
  stop(): void;
}

/** Owns break overtime counting and alert intervals. */
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
      if (
        context.runtime.breakOvertimeSeconds >= MAX_BREAK_OVERTIME_SECONDS
      ) {
        void context.completeOvertimeBreak();
      }
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
