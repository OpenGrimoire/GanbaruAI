import { describe, expect, it } from "vitest";
import {
  projectDefaultPomodoroConfig,
  projectPomodoroSummaryLabel,
} from "./project-default-pomodoro";

describe("project default pomodoro helpers", () => {
  it("formats compact rhythm summaries", () => {
    expect(projectPomodoroSummaryLabel({
      focusDurationMinutes: 40,
      shortBreakMinutes: 5,
      longBreakMinutes: 10,
      longBreakAfterFocusCount: 4,
    })).toBe("F 40 / SB 05 / LB 10 / C 4");
  });

  it("returns no config when the project default is none", () => {
    expect(projectDefaultPomodoroConfig({
      defaultPomodoroMode: "none",
      defaultPomodoroPresetKey: "adaptive",
    })).toBeUndefined();
  });

  it("builds preset configs from project defaults", () => {
    expect(projectDefaultPomodoroConfig({
      defaultPomodoroMode: "preset",
      defaultPomodoroPresetKey: "creative",
      defaultIdleTimeoutMinutes: 5,
    })).toEqual({
      rhythm: {
        kind: "count",
        focusDurationMinutes: 25,
        shortBreakMinutes: 5,
        longBreakMinutes: 15,
        longBreakAfterFocusCount: 4,
      },
      rhythmSource: "preset",
      presetKey: "creative",
      idleTimeoutMinutes: 5,
    });
  });

  it("builds custom count configs from project defaults", () => {
    expect(projectDefaultPomodoroConfig({
      defaultPomodoroMode: "custom",
      defaultPomodoroFocusMinutes: 45,
      defaultPomodoroShortBreakMinutes: 8,
      defaultPomodoroLongBreakMinutes: 20,
      defaultPomodoroLongBreakAfterFocusCount: 3,
      defaultIdleTimeoutMinutes: undefined,
    })).toEqual({
      rhythm: {
        kind: "count",
        focusDurationMinutes: 45,
        shortBreakMinutes: 8,
        longBreakMinutes: 20,
        longBreakAfterFocusCount: 3,
      },
      rhythmSource: "custom",
      presetKey: null,
      idleTimeoutMinutes: null,
    });
  });
});
