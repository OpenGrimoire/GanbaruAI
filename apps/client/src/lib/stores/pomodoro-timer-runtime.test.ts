import { afterEach, describe, expect, it, vi } from "vitest";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import { createPomodoroTimerRuntime } from "./pomodoro-timer-runtime";

afterEach(() => {
  vi.useRealTimers();
});

describe("Pomodoro timer runtime", () => {
  it("replaces heartbeat ownership instead of stacking intervals", () => {
    vi.useFakeTimers();
    const runtime = createPomodoroRuntimeFixture({ activeRunId: "run-1" });
    const sendHeartbeat = vi.fn();
    const timer = createPomodoroTimerRuntime({
      runtime,
      clock: createPomodoroClockController(runtime),
      tick: vi.fn(),
      expirePausedBlockAtDeadline: vi.fn(),
      updateTray: vi.fn(),
      sendHeartbeat,
    });

    timer.startHeartbeat();
    timer.startHeartbeat();
    expect(sendHeartbeat).toHaveBeenCalledTimes(2);
    vi.advanceTimersByTime(30_000);
    expect(sendHeartbeat).toHaveBeenCalledTimes(3);
  });

  it("expires a paused block once its deadline is reached", () => {
    vi.useFakeTimers();
    vi.setSystemTime(10_000);
    const runtime = createPomodoroRuntimeFixture({
      activeBlockId: "block-1",
      activeBlockEndMs: 11_000,
      remainingSeconds: 10,
      phaseWorkDurationSeconds: 10,
    });
    const expirePausedBlockAtDeadline = vi.fn();
    const timer = createPomodoroTimerRuntime({
      runtime,
      clock: createPomodoroClockController(runtime),
      tick: vi.fn(),
      expirePausedBlockAtDeadline,
      updateTray: vi.fn(),
      sendHeartbeat: vi.fn(),
    });

    timer.startPausedOpportunityCountdown();
    expect(runtime.remainingSeconds).toBe(1);
    vi.advanceTimersByTime(1_000);
    expect(expirePausedBlockAtDeadline).toHaveBeenCalledOnce();
  });

  it("keeps one visual scheduler after repeated starts", async () => {
    vi.useFakeTimers();
    const runtime = createPomodoroRuntimeFixture({ isRunning: true });
    const tick = vi.fn();
    const timer = createPomodoroTimerRuntime({
      runtime,
      clock: createPomodoroClockController(runtime),
      tick,
      expirePausedBlockAtDeadline: vi.fn(),
      updateTray: vi.fn(),
      sendHeartbeat: vi.fn(),
    });

    timer.startVisualTick();
    timer.startVisualTick();
    await vi.advanceTimersByTimeAsync(1_000);
    expect(tick).toHaveBeenCalledOnce();
  });
});
