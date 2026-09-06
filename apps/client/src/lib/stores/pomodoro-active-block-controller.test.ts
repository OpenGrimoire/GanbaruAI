import { describe, expect, it, vi } from "vitest";
import { createPomodoroActiveBlockController } from "./pomodoro-active-block-controller";
import { DEFAULT_CONFIG } from "./pomodoro-machine";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";

function createContext(
  runtime: ReturnType<typeof createPomodoroRuntimeFixture>,
  overrides: Record<string, unknown> = {},
) {
  return {
    runtime,
    segments: {
      activeSegment: () => runtime.segments[runtime.currentSegmentIndex] ?? null,
      eventDateFromBlockId: () => null,
      applyActiveBlockWindowChange: vi.fn(),
      createSegments: vi.fn(async (): Promise<void> => undefined),
    },
    transferActiveEventReference: vi.fn(async () => undefined),
    setActiveTimeoutMs: vi.fn(),
    startIdleChecking: vi.fn(),
    clearBreakEndWarning: vi.fn(),
    clearMusicPausedByPomodoro: vi.fn(),
    stopPausedOpportunityCountdown: vi.fn(),
    stopVisualTick: vi.fn(),
    startVisualTick: vi.fn(),
    initListeners: vi.fn(),
    setPhaseRemainingSeconds: vi.fn((seconds: number) => {
      runtime.remainingSeconds = seconds;
      return seconds;
    }),
    resetFocusNotificationState: vi.fn(),
    updateTray: vi.fn(),
    publishWindowSnapshot: vi.fn(),
    hasOvertime: () => false,
    reconfigureSession: vi.fn(async () => undefined),
    transitionToBlock: vi.fn(async () => undefined),
    nowMs: () => 1_000,
    nowIso: () => "2026-07-12T00:00:01.000Z",
    ...overrides,
  };
}

describe("Pomodoro active block controller", () => {
  it("does not start countdown or native effects before the run commits", async () => {
    const runtime = createPomodoroRuntimeFixture();
    const context = createContext(runtime);
    let commit!: () => void;
    context.segments.createSegments.mockImplementation(() => new Promise<void>((resolve) => { commit = resolve; }));
    const pending = createPomodoroActiveBlockController(context).startFromBlock(
      "block-1", DEFAULT_CONFIG, "Write", "2026-07-12 00:30:00", "2026-07-12",
    );
    expect(runtime.isRunning).toBe(false);
    expect(context.startVisualTick).not.toHaveBeenCalled();
    expect(context.startIdleChecking).not.toHaveBeenCalled();
    expect(context.updateTray).not.toHaveBeenCalled();
    commit();
    await pending;
    expect(runtime.isRunning).toBe(true);
  });

  it("returns a persistence failure without leaving an executing session", async () => {
    const runtime = createPomodoroRuntimeFixture();
    const context = createContext(runtime);
    context.segments.createSegments.mockRejectedValue(new Error("disk full"));
    await expect(createPomodoroActiveBlockController(context).startFromBlock(
      "block-1", DEFAULT_CONFIG, "Write", "2026-07-12 00:30:00", "2026-07-12",
    )).rejects.toThrow("disk full");
    expect(runtime.isRunning).toBe(false);
    expect(runtime.activeBlockId).toBeNull();
    expect(runtime.activeRunId).toBeNull();
    expect(runtime.phaseEndTime).toBeNull();
    expect(context.startVisualTick).not.toHaveBeenCalled();
    expect(context.updateTray).not.toHaveBeenCalled();
  });

  it("persists a transfer before adopting the successor id", async () => {
    const runtime = createPomodoroRuntimeFixture({
      activeBlockId: "old-block",
      segments: [
        {
          id: "segment-1",
          eventId: "old-block",
          eventDate: "2026-07-12",
          runId: "run-1",
          rhythmPosition: 1,
          phase: "focus",
          plannedStart: "2026-07-12T00:00:00.000Z",
          plannedEnd: "2026-07-12T00:25:00.000Z",
          actualStart: null,
          actualEnd: null,
          pauseLog: [],
          status: "active",
        },
      ],
      currentSegmentIndex: 0,
    });
    let resolveTransfer!: () => void;
    const transferActiveEventReference = vi.fn(
      () => new Promise<void>((resolve) => { resolveTransfer = resolve; }),
    );
    const context = createContext(runtime, { transferActiveEventReference });
    const transfer = createPomodoroActiveBlockController(context)
      .transferBlockId("new-block");

    expect(runtime.activeBlockId).toBe("old-block");
    resolveTransfer();
    await transfer;
    expect(runtime.activeBlockId).toBe("new-block");
    expect(runtime.segments[0]?.eventId).toBe("new-block");
  });

  it("initializes a new block once and keeps timer state in the shared runtime", async () => {
    const runtime = createPomodoroRuntimeFixture();
    const context = createContext(runtime);
    await createPomodoroActiveBlockController(context).startFromBlock(
      "block-1",
      DEFAULT_CONFIG,
      "Write project brief",
      "2026-07-12 00:30:00",
      "2026-07-12",
    );

    expect(runtime.activeBlockId).toBe("block-1");
    expect(runtime.activeBlockTitle).toBe("Write project brief");
    expect(runtime.isRunning).toBe(true);
    expect(runtime.phaseEndTime).toBe(1_000 + runtime.remainingSeconds * 1_000);
    expect(context.startVisualTick).toHaveBeenCalledOnce();
    expect(context.segments.createSegments).toHaveBeenCalledOnce();
    expect(context.updateTray).toHaveBeenCalledOnce();
  });
});
