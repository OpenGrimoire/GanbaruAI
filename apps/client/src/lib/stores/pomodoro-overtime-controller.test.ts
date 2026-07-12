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
      completeOvertimeBreak: vi.fn(async () => undefined),
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
      completeOvertimeBreak: vi.fn(async () => undefined),
    });
    controller.start();
    vi.advanceTimersByTime(2_000);
    controller.stop();
    expect(runtime.breakOvertimeSeconds).toBe(0);
    expect(controller.isActive()).toBe(false);
  });
});
