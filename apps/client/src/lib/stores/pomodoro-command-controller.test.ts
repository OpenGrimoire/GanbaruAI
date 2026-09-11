import { describe, expect, it, vi } from "vitest";
import { createPomodoroCommandController } from "./pomodoro-command-controller";
import type { PomodoroNativeEventListener } from "./pomodoro-runtime-environment-contract";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

function createContext(isCoordinator = true) {
  const runtime = createPomodoroRuntimeFixture();
  const listenMock = vi.fn(() => Promise.resolve(() => undefined)) as unknown as
    PomodoroNativeEventListener;
  return {
    runtime,
    isCoordinator: () => isCoordinator,
    publishWindowSnapshot: vi.fn(),
    setDismissedBlockId: vi.fn(),
    clearBlockExpired: vi.fn(),
    transferBlockId: vi.fn(async () => undefined),
    startFromBlock: vi.fn(async () => undefined),
    setActiveIdleThresholdMinutes: vi.fn(),
    dismissSuspend: vi.fn(async () => undefined),
    dismissIdle: vi.fn(async () => undefined),
    markIdleFocusFailed: vi.fn(async () => undefined),
    completeActiveBlockAt: vi.fn(async () => undefined),
    stopSession: vi.fn(async () => undefined),
    pause: vi.fn(),
    resume: vi.fn(),
    skip: vi.fn(),
    addFocusTime: vi.fn(),
    addBreakTime: vi.fn(),
    cleanupOrphans: vi.fn(async () => undefined),
    canPauseResume: () => true,
    advancePhase: vi.fn(async () => undefined),
    updateTray: vi.fn(),
    recordManualPhaseAdvanceEvent: vi.fn(),
    recordRunEvent: vi.fn(),
    activeSegmentId: () => null,
    markCurrentSegmentSkipped: vi.fn(async () => undefined),
    startFocusSession: vi.fn(async () => undefined),
    completeCurrentBreak: vi.fn(async () => undefined),
    cappedActiveBreakEndIso: () => "2026-07-12T00:00:00.000Z",
    pausedFocusPulseActive: () => false,
    suppressPausedFocusNotifications: vi.fn(),
    nowIso: () => "2026-07-12T00:00:00.000Z",
    nativeEventListener: listenMock,
  };
}

describe("Pomodoro command controller", () => {
  it("installs one listener set across repeated accessor initialization", () => {
    const context = createContext();
    const controller = createPomodoroCommandController(context);

    controller.initListeners();
    controller.initListeners();

    expect(context.nativeEventListener).toHaveBeenCalledTimes(10);
  });

  it("ignores commands and listener initialization in secondary windows", () => {
    const context = createContext(false);
    const controller = createPomodoroCommandController(context);

    controller.handleWindowCommand({ kind: "pause" });
    controller.initListeners();

    expect(context.pause).not.toHaveBeenCalled();
    expect(context.nativeEventListener).not.toHaveBeenCalled();
  });

  it("routes coordinator commands without changing their payload", () => {
    const context = createContext();
    const controller = createPomodoroCommandController(context);

    controller.handleWindowCommand({
      kind: "transfer-block-id",
      newBlockId: "successor",
      newEndTime: "2026-07-12 00:30:00",
    });

    expect(context.transferBlockId).toHaveBeenCalledWith(
      "successor",
      "2026-07-12 00:30:00",
    );
  });
});
