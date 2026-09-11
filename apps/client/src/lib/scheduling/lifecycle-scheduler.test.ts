import { describe, expect, it, vi } from "vitest";
import {
  createLifecycleScheduler,
  type SchedulerClock,
  type SchedulerTimeout,
} from "./lifecycle-scheduler";

class FakeSchedulerClock implements SchedulerClock {
  private nextId = 1;
  private timers = new Map<number, { at: number; callback: () => void }>();

  constructor(private nowMs = 0) {}

  now(): number {
    return this.nowMs;
  }

  setTimeout(callback: () => void, delayMs: number): SchedulerTimeout {
    const id = this.nextId++;
    this.timers.set(id, { at: this.nowMs + Math.max(0, delayMs), callback });
    return id as unknown as SchedulerTimeout;
  }

  clearTimeout(timeout: SchedulerTimeout): void {
    this.timers.delete(timeout as unknown as number);
  }

  pendingCount(): number {
    return this.timers.size;
  }

  jumpTo(nowMs: number): void {
    this.nowMs = nowMs;
  }

  async advanceTo(nowMs: number): Promise<void> {
    while (true) {
      const next = [...this.timers.entries()]
        .filter(([, timer]) => timer.at <= nowMs)
        .sort((left, right) => left[1].at - right[1].at || left[0] - right[0])[0];
      if (!next) break;
      const [id, timer] = next;
      this.timers.delete(id);
      this.nowMs = timer.at;
      timer.callback();
      await settleScheduler();
    }
    this.nowMs = nowMs;
    await settleScheduler();
  }
}

async function settleScheduler(): Promise<void> {
  for (let index = 0; index < 8; index += 1) await Promise.resolve();
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

describe("lifecycle scheduler", () => {
  it("enables, follows deadlines, disables, and cleans up", async () => {
    const clock = new FakeSchedulerClock(100);
    const run = vi.fn(() => clock.now() + 1_000);
    const scheduler = createLifecycleScheduler({ clock, run });

    scheduler.setEnabled(true);
    await settleScheduler();
    expect(run).toHaveBeenCalledOnce();
    expect(clock.pendingCount()).toBe(1);

    await clock.advanceTo(1_100);
    expect(run).toHaveBeenCalledTimes(2);
    expect(clock.pendingCount()).toBe(1);

    scheduler.setEnabled(false);
    expect(clock.pendingCount()).toBe(0);
    await clock.advanceTo(10_000);
    expect(run).toHaveBeenCalledTimes(2);

    scheduler.dispose();
    scheduler.setEnabled(true);
    expect(clock.pendingCount()).toBe(0);
  });

  it("coalesces invalidations and ignores stale async deadlines", async () => {
    const clock = new FakeSchedulerClock();
    const first = deferred<number | null>();
    const run = vi.fn()
      .mockImplementationOnce(() => first.promise)
      .mockResolvedValue(null);
    const scheduler = createLifecycleScheduler({ clock, run });

    scheduler.setEnabled(true);
    await settleScheduler();
    scheduler.invalidate();
    scheduler.invalidate();
    scheduler.invalidate();
    first.resolve(clock.now() + 50_000);
    await settleScheduler();

    expect(run).toHaveBeenCalledTimes(2);
    expect(clock.pendingCount()).toBe(0);
  });

  it("performs one catch-up run after a suspended clock jump", async () => {
    const clock = new FakeSchedulerClock();
    const run = vi.fn(() => clock.now() + 1_000);
    const scheduler = createLifecycleScheduler({ clock, run });
    scheduler.setEnabled(true);
    await settleScheduler();

    clock.jumpTo(30_000);
    scheduler.resume();
    await settleScheduler();

    expect(run).toHaveBeenCalledTimes(2);
    expect(clock.pendingCount()).toBe(1);
  });

  it("does not schedule a late result after disable", async () => {
    const clock = new FakeSchedulerClock();
    const pending = deferred<number | null>();
    const scheduler = createLifecycleScheduler({ clock, run: () => pending.promise });
    scheduler.setEnabled(true);
    await settleScheduler();

    scheduler.setEnabled(false);
    pending.resolve(10_000);
    await settleScheduler();

    expect(clock.pendingCount()).toBe(0);
  });
});
