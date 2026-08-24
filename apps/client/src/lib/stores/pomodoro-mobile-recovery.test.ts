import { describe, expect, it, vi } from "vitest";
import { createPomodoroRuntimeFixture } from "./pomodoro-runtime.test-helpers";
import {
  applyRecoveredMobileRun,
  parsePomodoroMobileRecoveryResult,
  type PomodoroRecoveredMobileRun,
} from "./pomodoro-mobile-recovery";

function recoveredRun(overrides: Partial<PomodoroRecoveredMobileRun> = {}) {
  const run: PomodoroRecoveredMobileRun = {
    runId: "run-1",
    blockId: "event-1::2026-05-29",
    eventDate: "2026-05-29",
    plannedEnd: "2026-05-29T11:00:00.000Z",
    startedAt: "2026-05-29T10:00:00.000Z",
    recoveredAt: "2026-05-29T10:10:00.000Z",
    rhythm: {
      kind: "count",
      focusDurationMinutes: 40,
      shortBreakMinutes: 5,
      longBreakMinutes: 10,
      longBreakAfterFocusCount: 4,
    },
    rhythmSource: "custom",
    presetKey: null,
    idleTimeoutMinutes: 3,
    segment: {
      id: "segment-1",
      eventId: "event-1::2026-05-29",
      eventDate: "2026-05-29",
      runId: "run-1",
      rhythmPosition: 1,
      phase: "focus",
      plannedStart: "2026-05-29T10:00:00.000Z",
      plannedEnd: "2026-05-29T10:40:00.000Z",
      actualStart: "2026-05-29T10:00:00.000Z",
      actualEnd: null,
      pauseLog: [],
      status: "active",
    },
    completedFocusCount: 0,
    phaseElapsedSeconds: 600,
    phaseWorkDurationSeconds: 2_400,
    remainingSeconds: 1_800,
    isRunning: true,
    focusExtensionUsed: false,
    openPauseReason: null,
  };
  return { ...run, ...overrides };
}

function recoveryContext(run = recoveredRun()) {
  const runtime = createPomodoroRuntimeFixture();
  const callbacks = {
    stopVisualTick: vi.fn(),
    stopPausedOpportunityCountdown: vi.fn(),
    stopOvertime: vi.fn(),
    stopIdleChecking: vi.fn(),
    initListeners: vi.fn(),
    refreshFutureSegments: vi.fn(),
    startHeartbeat: vi.fn(),
    startVisualTick: vi.fn(),
    startPausedOpportunityCountdown: vi.fn(),
    startIdleChecking: vi.fn(),
    scheduleBreakEndWarning: vi.fn(),
    updateTray: vi.fn(),
    publishWindowSnapshot: vi.fn(),
  };
  return {
    run,
    runtime,
    callbacks,
    context: {
      runtime,
      nativeIdleDetectionAvailable: false,
      ...callbacks,
      nowMs: () => Date.parse("2026-05-29T10:10:00.000Z"),
    },
  };
}

describe("mobile pomodoro recovery", () => {
  it("validates and maps a resumed backend response", () => {
    const run = recoveredRun();

    expect(parsePomodoroMobileRecoveryResult({ kind: "resumed", run })).toEqual({
      kind: "resumed",
      run,
    });
    expect(parsePomodoroMobileRecoveryResult({
      kind: "closed",
      reason: "phase_expired",
      closedRunIds: ["run-1"],
    })).toEqual({
      kind: "closed",
      reason: "phase_expired",
      closedRunIds: ["run-1"],
    });
  });

  it("rejects a response whose pause ownership contradicts running state", () => {
    const run = recoveredRun({
      isRunning: false,
      openPauseReason: "manual",
    });

    expect(() => parsePomodoroMobileRecoveryResult({ kind: "resumed", run }))
      .toThrow("Invalid resumed mobile pomodoro recovery response");
  });

  it("rejects a break phase that contradicts the persisted rhythm position", () => {
    const base = recoveredRun();
    const run = recoveredRun({
      segment: { ...base.segment, phase: "long_break" },
    });

    expect(() => parsePomodoroMobileRecoveryResult({ kind: "resumed", run }))
      .toThrow("Invalid resumed mobile pomodoro recovery response");
  });

  it("restores a running phase and restarts only its required controllers", () => {
    const { run, runtime, callbacks, context } = recoveryContext();

    applyRecoveredMobileRun(run, context);

    expect(runtime.activeRunId).toBe("run-1");
    expect(runtime.activeBlockId).toBe("event-1::2026-05-29");
    expect(runtime.activeBlockEndMs).toBe(Date.parse(run.plannedEnd));
    expect(runtime.phase).toBe("focus");
    expect(runtime.remainingSeconds).toBe(1_800);
    expect(runtime.phaseElapsedSeconds).toBe(600);
    expect(runtime.phaseWorkDurationSeconds).toBe(2_400);
    expect(runtime.phaseEndTime).toBe(
      Date.parse("2026-05-29T10:40:00.000Z"),
    );
    expect(runtime.segments).toEqual([run.segment]);
    expect(runtime.currentSegmentIndex).toBe(0);
    expect(runtime.idleTimeoutMs).toBeNull();
    expect(callbacks.refreshFutureSegments).toHaveBeenCalledWith(
      run.blockId,
      run.eventDate,
    );
    expect(callbacks.startHeartbeat).toHaveBeenCalledOnce();
    expect(callbacks.startVisualTick).toHaveBeenCalledOnce();
    expect(callbacks.startPausedOpportunityCountdown).not.toHaveBeenCalled();
    expect(callbacks.publishWindowSnapshot).toHaveBeenCalledOnce();
  });

  it("restores an open persisted pause without starting the visual timer", () => {
    const pause = {
      startedAt: "2026-05-29T10:08:00.000Z",
      endedAt: null,
      reason: "manual" as const,
    };
    const base = recoveredRun();
    const run = recoveredRun({
      isRunning: false,
      openPauseReason: "manual",
      segment: { ...base.segment, pauseLog: [pause] },
    });
    const { runtime, callbacks, context } = recoveryContext(run);

    applyRecoveredMobileRun(run, context);

    expect(runtime.isRunning).toBe(false);
    expect(runtime.phaseEndTime).toBeNull();
    expect(callbacks.startHeartbeat).toHaveBeenCalledOnce();
    expect(callbacks.startVisualTick).not.toHaveBeenCalled();
    expect(callbacks.startPausedOpportunityCountdown).toHaveBeenCalledOnce();
  });
});
