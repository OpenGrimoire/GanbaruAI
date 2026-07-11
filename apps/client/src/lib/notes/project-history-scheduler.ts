import {
  flushDueNotesProjectHistory,
  type NotesProjectHistorySchedule,
} from "$lib/api/notes-project-history";
import {
  createLifecycleScheduler,
  systemSchedulerClock,
  type LifecycleScheduler,
  type SchedulerClock,
} from "$lib/scheduling/lifecycle-scheduler";

const ACTIVE_CHECKPOINT_MS = 10 * 60 * 1_000;
const IDLE_CHECKPOINT_MS = 2 * 60 * 1_000;
const ERROR_RETRY_MS = 60_000;

export interface NotesProjectHistoryScheduler {
  setEnabled(enabled: boolean): void;
  noteMutation(forceCheckpoint?: boolean): void;
  applyMutationDeadline(deadline: string | null): void;
  resume(): void;
  switchVault(): void;
  shutdown(): Promise<void>;
  dispose(): void;
  hasScheduledDeadline(): boolean;
}

export interface NotesProjectHistorySchedulerOptions {
  clock?: SchedulerClock;
  flush?: () => Promise<NotesProjectHistorySchedule>;
  onError?: (error: unknown) => void;
}

function deadlineMs(value: string | null): number | null {
  if (!value) return null;
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function nextScheduleDeadline(schedule: NotesProjectHistorySchedule): number | null {
  const checkpoint = deadlineMs(schedule.nextCheckpointAt);
  const maintenance = deadlineMs(schedule.nextMaintenanceAt);
  if (checkpoint === null) return maintenance;
  if (maintenance === null) return checkpoint;
  return Math.min(checkpoint, maintenance);
}

/** Schedule Notes history only at dirty or bounded maintenance deadlines. */
export function createNotesProjectHistoryScheduler(
  options: NotesProjectHistorySchedulerOptions = {},
): NotesProjectHistoryScheduler {
  const clock = options.clock ?? systemSchedulerClock;
  const flush = options.flush ?? flushDueNotesProjectHistory;
  let firstDirtyAtMs: number | null = null;
  let lastDirtyAtMs: number | null = null;
  let lifecycle: LifecycleScheduler;

  lifecycle = createLifecycleScheduler({
    clock,
    errorRetryMs: ERROR_RETRY_MS,
    onError: options.onError,
    run: async () => {
      const schedule = await flush();
      firstDirtyAtMs = null;
      lastDirtyAtMs = null;
      return nextScheduleDeadline(schedule);
    },
  });

  function noteMutation(forceCheckpoint = false): void {
    if (!lifecycle.isEnabled()) return;
    const now = clock.now();
    firstDirtyAtMs ??= now;
    lastDirtyAtMs = now;
    const deadline = forceCheckpoint
      ? now
      : Math.min(firstDirtyAtMs + ACTIVE_CHECKPOINT_MS, lastDirtyAtMs + IDLE_CHECKPOINT_MS);
    lifecycle.scheduleAt(deadline);
  }

  function applyMutationDeadline(deadline: string | null): void {
    const authoritativeDeadline = deadlineMs(deadline);
    if (authoritativeDeadline === null) {
      return;
    }
    firstDirtyAtMs ??= clock.now();
    lastDirtyAtMs = clock.now();
    lifecycle.scheduleAt(authoritativeDeadline);
  }

  function switchVault(): void {
    firstDirtyAtMs = null;
    lastDirtyAtMs = null;
    lifecycle.resume();
  }

  async function shutdown(): Promise<void> {
    if (!lifecycle.isEnabled()) return;
    lifecycle.dispose();
    await flush();
  }

  return {
    setEnabled: lifecycle.setEnabled,
    noteMutation,
    applyMutationDeadline,
    resume: lifecycle.resume,
    switchVault,
    shutdown,
    dispose: lifecycle.dispose,
    hasScheduledDeadline: lifecycle.hasScheduledDeadline,
  };
}

const sharedNotesProjectHistoryScheduler = createNotesProjectHistoryScheduler({
  onError: (error) => console.error("Notes project history scheduler failed", error),
});

export function getNotesProjectHistoryScheduler(): NotesProjectHistoryScheduler {
  return sharedNotesProjectHistoryScheduler;
}

export function notifyNotesProjectHistoryMutation(forceCheckpoint = false): void {
  sharedNotesProjectHistoryScheduler.noteMutation(forceCheckpoint);
}

export function applyNotesProjectHistoryMutationDeadline(
  deadline: string | null,
): void {
  sharedNotesProjectHistoryScheduler.applyMutationDeadline(deadline);
}
