import { describe, expect, it } from "vitest";
import { createPomodoroNativeTrayPolicy, type PomodoroNativeTrayState } from "./pomodoro-native-update-policy";

function runningState(remainingSeconds: number): PomodoroNativeTrayState {
  return {
    phase: "focus",
    remainingSeconds,
    totalSeconds: 600,
    isRunning: true,
    isActive: true,
    canPauseResume: true,
    canAddFocusTime: true,
    pausedPulseFrame: null,
  };
}

describe("Pomodoro native update policy", () => {
  it("does not send one tray IPC call per visual tick for ten uninterrupted minutes", () => {
    const policy = createPomodoroNativeTrayPolicy();
    let calls = 0;
    for (let remaining = 600; remaining >= 0; remaining -= 1) {
      if (policy.shouldSend(runningState(remaining))) calls += 1;
    }
    expect(calls).toBeLessThanOrEqual(101);
    expect(calls).toBeGreaterThan(90);
  });

  it("sends pause and resume transitions immediately within the same progress step", () => {
    const policy = createPomodoroNativeTrayPolicy();
    expect(policy.shouldSend(runningState(599))).toBe(true);
    expect(policy.shouldSend(runningState(598))).toBe(false);
    expect(policy.shouldSend({ ...runningState(598), isRunning: false })).toBe(true);
    expect(policy.shouldSend(runningState(598))).toBe(true);
  });

  it("deduplicates identical paused pulse frames", () => {
    const policy = createPomodoroNativeTrayPolicy();
    const paused = { ...runningState(300), isRunning: false, pausedPulseFrame: 4 };
    expect(policy.shouldSend(paused)).toBe(true);
    expect(policy.shouldSend(paused)).toBe(false);
    expect(policy.shouldSend({ ...paused, pausedPulseFrame: 5 })).toBe(true);
  });
});
