import { createLifecycleScheduler } from "$lib/scheduling/lifecycle-scheduler";
import type { PomodoroClockController } from "./pomodoro-clock-controller";
import type { PomodoroRuntime } from "./pomodoro-runtime";

const HEARTBEAT_INTERVAL_MS = 30_000;

interface PomodoroTimerRuntimeContext {
  runtime: PomodoroRuntime;
  clock: Pick<
    PomodoroClockController,
    "activeBlockDeadlineReached" | "refreshPausedOpportunityRemaining"
  >;
  tick(): void;
  expirePausedBlockAtDeadline(): void;
  updateTray(): void;
  sendHeartbeat(): void;
}

export interface PomodoroTimerRuntime {
  startVisualTick(): void;
  stopVisualTick(): void;
  startPausedOpportunityCountdown(): void;
  stopPausedOpportunityCountdown(): void;
  startHeartbeat(): void;
  stopHeartbeat(): void;
}

/** Owns the timer intervals that must exist once per app window. */
export function createPomodoroTimerRuntime(
  context: PomodoroTimerRuntimeContext,
): PomodoroTimerRuntime {
  let pausedOpportunityIntervalId: ReturnType<typeof setInterval> | null = null;
  let heartbeatIntervalId: ReturnType<typeof setInterval> | null = null;

  const visualTickScheduler = createLifecycleScheduler({
    initialDelayMs: 1_000,
    run: (schedulerContext) => {
      context.tick();
      return context.runtime.isRunning ? schedulerContext.now() + 1_000 : null;
    },
  });

  function startVisualTick(): void {
    visualTickScheduler.setEnabled(false);
    visualTickScheduler.setEnabled(true);
  }

  function stopVisualTick(): void {
    visualTickScheduler.setEnabled(false);
  }

  function stopPausedOpportunityCountdown(): void {
    if (pausedOpportunityIntervalId === null) return;
    clearInterval(pausedOpportunityIntervalId);
    pausedOpportunityIntervalId = null;
  }

  function startPausedOpportunityCountdown(): void {
    stopPausedOpportunityCountdown();
    if (!context.runtime.activeBlockId) return;
    if (context.clock.activeBlockDeadlineReached()) {
      context.expirePausedBlockAtDeadline();
      return;
    }
    context.clock.refreshPausedOpportunityRemaining();
    pausedOpportunityIntervalId = setInterval(() => {
      if (
        context.runtime.isRunning ||
        !context.runtime.activeBlockId ||
        context.runtime.suspendedAway ||
        context.runtime.idlePaused
      ) {
        stopPausedOpportunityCountdown();
        return;
      }
      if (context.clock.activeBlockDeadlineReached()) {
        context.expirePausedBlockAtDeadline();
        return;
      }
      if (context.clock.refreshPausedOpportunityRemaining()) {
        context.updateTray();
      }
    }, 1_000);
  }

  function stopHeartbeat(): void {
    if (heartbeatIntervalId === null) return;
    clearInterval(heartbeatIntervalId);
    heartbeatIntervalId = null;
  }

  function startHeartbeat(): void {
    stopHeartbeat();
    if (!context.runtime.activeRunId) return;
    context.sendHeartbeat();
    heartbeatIntervalId = setInterval(
      context.sendHeartbeat,
      HEARTBEAT_INTERVAL_MS,
    );
  }

  return {
    startVisualTick,
    stopVisualTick,
    startPausedOpportunityCountdown,
    stopPausedOpportunityCountdown,
    startHeartbeat,
    stopHeartbeat,
  };
}
