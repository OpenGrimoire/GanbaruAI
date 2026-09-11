import { afterEach, describe, expect, it, vi } from "vitest";
import { createPomodoroOvertimeController } from "./pomodoro-overtime-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

afterEach(() => vi.useRealTimers());

describe("Pomodoro overtime controller", () => {
  it("owns one counter and alert interval across repeated starts", () => {
    vi.useFakeTimers();
    const runtime = createPomodoroRuntimeFixture();
    const publishWindowSnapshot = vi.fn();
    const playBreakFinishedAlert = vi.fn();
    const controller = createPomodoroOvertimeController({
      runtime,
      publishWindowSnapshot,
      playBreakFinishedAlert,
      startConfiguredAlertInterval: () => null,
    });

    controller.start();
    controller.start();
    vi.advanceTimersByTime(1_000);

    expect(runtime.breakOvertimeSeconds).toBe(1);
    expect(publishWindowSnapshot).toHaveBeenCalledOnce();
    expect(playBreakFinishedAlert).toHaveBeenCalledOnce();
  });

  it("clears visible overtime when stopped", () => {
    vi.useFakeTimers();
    const runtime = createPomodoroRuntimeFixture();
    const controller = createPomodoroOvertimeController({
      runtime,
      publishWindowSnapshot: vi.fn(),
      playBreakFinishedAlert: vi.fn(),
      startConfiguredAlertInterval: () => null,
    });
    controller.start();
    vi.advanceTimersByTime(2_000);
    controller.stop();
    expect(runtime.breakOvertimeSeconds).toBe(0);
    expect(controller.isActive()).toBe(false);
  });

  it("keeps a missed return awaiting acceptance without creating focus", () => {
    vi.useFakeTimers();
    const runtime = createPomodoroRuntimeFixture({ phase: "short_break", isRunning: false });
    const publishWindowSnapshot = vi.fn();
    const controller = createPomodoroOvertimeController({
      runtime,
      publishWindowSnapshot,
      playBreakFinishedAlert: vi.fn(),
      startConfiguredAlertInterval: () => null,
    });
    controller.start();
    vi.advanceTimersByTime(60 * 60 * 1_000);

    expect(runtime.phase).toBe("short_break");
    expect(runtime.isRunning).toBe(false);
    expect(runtime.activeRunId).toBeNull();
    expect(runtime.segments).toEqual([]);
    expect(runtime.breakOvertimeSeconds).toBe(3_600);
    controller.stop();
  });
});
