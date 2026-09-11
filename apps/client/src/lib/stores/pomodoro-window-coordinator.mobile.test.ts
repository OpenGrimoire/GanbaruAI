import { describe, expect, it, vi } from "vitest";
import {
  createPomodoroWindowCoordinator,
} from "./pomodoro-window-coordinator.mobile";
import type { PomodoroWindowCoordinatorContext } from "./pomodoro-window-coordinator-contract";
import { getPomodoroRuntimeEnvironment } from "./pomodoro-runtime-environment.mobile";

function createContext(): PomodoroWindowCoordinatorContext {
  return {
    isCoordinator: vi.fn(() => true),
    beforePublishSnapshot: vi.fn(),
    buildSnapshot: vi.fn(() => {
      throw new Error("mobile coordinator must not build desktop snapshots");
    }),
    applySnapshot: vi.fn(),
    handleCommand: vi.fn(),
  };
}

describe("mobile Pomodoro window coordinator", () => {
  it("owns the only runtime without desktop synchronization work", () => {
    const context = createContext();
    const coordinator = createPomodoroWindowCoordinator(context);

    expect(getPomodoroRuntimeEnvironment()).toEqual({
      isCoordinator: true,
      nativeEventListener: null,
    });
    expect(coordinator.forwardCommand({ kind: "request-snapshot" })).toBe(false);
    coordinator.sendCommand({ kind: "request-snapshot" });
    coordinator.publishSnapshot();
    coordinator.init();

    expect(context.isCoordinator).not.toHaveBeenCalled();
    expect(context.beforePublishSnapshot).not.toHaveBeenCalled();
    expect(context.buildSnapshot).not.toHaveBeenCalled();
    expect(context.applySnapshot).not.toHaveBeenCalled();
    expect(context.handleCommand).not.toHaveBeenCalled();
  });
});
