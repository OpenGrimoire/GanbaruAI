// @vitest-environment jsdom

import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import PomodoroProgressRing from "./PomodoroProgressRing.svelte";

describe("PomodoroProgressRing", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("shows the remaining portion of an active phase", () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(PomodoroProgressRing, {
      target,
      props: {
        active: true,
        remainingSeconds: 600,
        totalSeconds: 1_200,
      },
    });

    const arc = target.querySelector<SVGCircleElement>("[data-pomodoro-progress-arc]");
    const dashLength = Number.parseFloat(arc?.getAttribute("stroke-dasharray") ?? "");
    expect(dashLength).toBeCloseTo(Math.PI * 8, 5);
  });

  it("keeps only the subdued track when there is no active session", () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(PomodoroProgressRing, {
      target,
      props: {
        active: false,
        remainingSeconds: 0,
        totalSeconds: 0,
      },
    });

    expect(target.querySelector("[data-pomodoro-progress-ring]")).not.toBeNull();
    expect(target.querySelector("[data-pomodoro-progress-arc]")).toBeNull();
  });

  it("uses the shared paused pulse frame", () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(PomodoroProgressRing, {
      target,
      props: {
        active: true,
        remainingSeconds: 900,
        totalSeconds: 1_200,
        paused: true,
        pausedPulseAmount: 0.42,
      },
    });

    const arc = target.querySelector<SVGCircleElement>("[data-pomodoro-progress-arc]");
    expect(arc?.classList.contains("pomodoro-ring-paused-pulse")).toBe(true);
    expect(arc?.getAttribute("style")).toContain("42%");
  });
});
