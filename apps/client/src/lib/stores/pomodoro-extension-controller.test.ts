import { describe, expect, it, vi } from "vitest";
import { createPomodoroClockController } from "./pomodoro-clock-controller";
import { createPomodoroExtensionController } from "./pomodoro-extension-controller";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

describe("Pomodoro extension controller", () => {
  it("does not turn block-clamped visible time into work duration", () => {
    const nowMs = 10_000;
    const runtime = createPomodoroRuntimeFixture({
      isRunning: true,
      activeBlockId: "block-1",
      activeBlockEndMs: nowMs + 60_000,
      remainingSeconds: 60,
      phaseTotalSeconds: 60,
      phaseWorkDurationSeconds: 1_500,
    });
    const controller = createPomodoroExtensionController({
      runtime,
      clock: createPomodoroClockController(runtime, () => nowMs),
      hasOvertime: () => false,
      activeSegment: () => null,
      persistSegment: vi.fn(),
      recordRunEvent: vi.fn(),
      scheduleBreakEndWarning: vi.fn(),
      updateTray: vi.fn(),
      nowMs: () => nowMs,
    });

    expect(controller.canExtendFocusTime()).toBe(false);
    controller.addFocusTime();
    expect(runtime.phaseWorkDurationSeconds).toBe(1_500);
    expect(runtime.focusExtensionUsed).toBe(false);
  });

  it("keeps break overtime separate from extension eligibility", () => {
    const runtime = createPomodoroRuntimeFixture({
      phase: "short_break",
      isRunning: false,
      remainingSeconds: 0,
      phaseTotalSeconds: 300,
    });
    const controller = createPomodoroExtensionController({
      runtime,
      clock: createPomodoroClockController(runtime),
      hasOvertime: () => true,
      activeSegment: () => null,
      persistSegment: vi.fn(),
      recordRunEvent: vi.fn(),
      scheduleBreakEndWarning: vi.fn(),
      updateTray: vi.fn(),
    });
    expect(controller.canExtendBreakTime()).toBe(false);
  });
});
