const MAX_TIMEOUT_MS = 2_147_000_000;

export type SchedulerTimeout = ReturnType<typeof setTimeout>;

export interface SchedulerClock {
  now(): number;
  setTimeout(callback: () => void, delayMs: number): SchedulerTimeout;
  clearTimeout(timeout: SchedulerTimeout): void;
}

export const systemSchedulerClock: SchedulerClock = {
  now: () => Date.now(),
  setTimeout: (callback, delayMs) => setTimeout(callback, delayMs),
  clearTimeout: (timeout) => clearTimeout(timeout),
};

export interface SchedulerRunContext {
  readonly startedAtMs: number;
  now(): number;
  isCurrent(): boolean;
}

export interface LifecycleScheduler {
  setEnabled(enabled: boolean): void;
  invalidate(): void;
  resume(): void;
  scheduleAt(deadlineMs: number): void;
  dispose(): void;
  isEnabled(): boolean;
  hasScheduledDeadline(): boolean;
}

export interface LifecycleSchedulerOptions {
  readonly clock?: SchedulerClock;
  readonly run: (context: SchedulerRunContext) => number | null | Promise<number | null>;
  readonly initialDelayMs?: number;
  readonly errorRetryMs?: number;
  readonly onError?: (error: unknown) => void;
}

/**
 * Runs one deadline-producing task at a time. Invalidations during a run are
 * coalesced into one queued rerun, and stale run results never schedule work.
 */
export function createLifecycleScheduler(
  options: LifecycleSchedulerOptions,
): LifecycleScheduler {
  const clock = options.clock ?? systemSchedulerClock;
  let enabled = false;
  let disposed = false;
  let timeout: SchedulerTimeout | null = null;
  let running = false;
  let queued = false;
  let generation = 0;

  function clearScheduledDeadline(): void {
    if (timeout === null) return;
    clock.clearTimeout(timeout);
    timeout = null;
  }

  function scheduleAt(deadlineMs: number, expectedGeneration = generation): void {
    clearScheduledDeadline();
    if (!enabled || disposed || expectedGeneration !== generation) return;
    const scheduleChunk = () => {
      if (!enabled || disposed || expectedGeneration !== generation) return;
      const remainingMs = deadlineMs - clock.now();
      const delayMs = Math.max(0, Math.min(MAX_TIMEOUT_MS, remainingMs));
      timeout = clock.setTimeout(() => {
        timeout = null;
        if (!enabled || disposed || expectedGeneration !== generation) return;
        if (deadlineMs > clock.now()) {
          scheduleChunk();
          return;
        }
        requestRun();
      }, delayMs);
    };
    scheduleChunk();
  }

  function requestRun(): void {
    if (!enabled || disposed) return;
    clearScheduledDeadline();
    if (running) {
      queued = true;
      return;
    }

    running = true;
    const runGeneration = generation;
    const context: SchedulerRunContext = {
      startedAtMs: clock.now(),
      now: () => clock.now(),
      isCurrent: () => enabled && !disposed && generation === runGeneration,
    };
    void Promise.resolve()
      .then(() => options.run(context))
      .then((nextDeadlineMs) => {
        if (!context.isCurrent() || nextDeadlineMs === null) return;
        scheduleAt(nextDeadlineMs, runGeneration);
      })
      .catch((error: unknown) => {
        if (!context.isCurrent()) return;
        options.onError?.(error);
        const retryMs = options.errorRetryMs ?? 0;
        if (retryMs > 0) scheduleAt(clock.now() + retryMs, runGeneration);
      })
      .finally(() => {
        running = false;
        if (!enabled || disposed) {
          queued = false;
          return;
        }
        if (queued || runGeneration !== generation) {
          queued = false;
          requestRun();
        }
      });
  }

  function setEnabled(nextEnabled: boolean): void {
    if (disposed || enabled === nextEnabled) return;
    enabled = nextEnabled;
    generation += 1;
    clearScheduledDeadline();
    queued = false;
    if (!enabled) return;
    const initialDelayMs = Math.max(0, options.initialDelayMs ?? 0);
    if (initialDelayMs > 0) {
      scheduleAt(clock.now() + initialDelayMs);
    } else {
      requestRun();
    }
  }

  function invalidate(): void {
    if (!enabled || disposed) return;
    generation += 1;
    clearScheduledDeadline();
    if (running) {
      queued = true;
    } else {
      requestRun();
    }
  }

  function dispose(): void {
    if (disposed) return;
    enabled = false;
    disposed = true;
    generation += 1;
    queued = false;
    clearScheduledDeadline();
  }

  return {
    setEnabled,
    invalidate,
    resume: invalidate,
    scheduleAt: (deadlineMs) => {
      if (!enabled || disposed || !Number.isFinite(deadlineMs)) return;
      generation += 1;
      queued = false;
      scheduleAt(deadlineMs);
    },
    dispose,
    isEnabled: () => enabled,
    hasScheduledDeadline: () => timeout !== null,
  };
}
