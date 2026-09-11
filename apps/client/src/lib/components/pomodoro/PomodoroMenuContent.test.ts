// @vitest-environment jsdom

import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import PomodoroMenuContent from "./PomodoroMenuContent.svelte";

const pomodoro = vi.hoisted(() => ({
  isActive: true,
  isRunning: true,
  phase: "focus" as const,
  canPauseResume: true,
  canAddFocusTime: true,
  formattedTime: "24:30",
  pause: vi.fn(),
  start: vi.fn(),
  addFocusTime: vi.fn(),
  skip: vi.fn(),
}));

const musicPlayer = vi.hoisted(() => ({
  currentSource: null,
  loadedTitle: "",
  snapshot: { status: "idle" as const },
  isBusy: false,
  isPlaying: false,
  volumeMax: 1,
  volumeControlValue: 1,
  contextPlayback: null,
  canPlayPreviousTrack: false,
  canPlayNextTrack: false,
  setVolume: vi.fn(async () => undefined),
  togglePlay: vi.fn(async () => undefined),
  playPreviousTrack: vi.fn(async () => undefined),
  playNextTrack: vi.fn(async () => undefined),
  inspectContextAssignment: vi.fn(),
}));

vi.mock("$lib/stores/pomodoro.svelte", () => ({
  getPomodoro: () => pomodoro,
}));

vi.mock("$lib/stores/music-player.svelte", () => ({
  getMusicPlayer: () => musicPlayer,
}));

describe("PomodoroMenuContent", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    target = undefined;
    component = undefined;
    pomodoro.pause.mockClear();
  });

  it("uses the shared Pomodoro actions in touch presentation", () => {
    target = document.createElement("div");
    document.body.append(target);
    const onDismiss = vi.fn();
    component = mount(PomodoroMenuContent, {
      target,
      props: {
        includeMusic: false,
        touch: true,
        onDismiss,
        onOpenMusic: vi.fn(),
      },
    });

    const actions = target.querySelectorAll<HTMLButtonElement>("button");
    expect(actions).toHaveLength(3);
    expect(actions.item(0).classList.contains("min-h-12")).toBe(true);
    actions.item(0).click();
    expect(pomodoro.pause).toHaveBeenCalledOnce();
    expect(onDismiss).toHaveBeenCalledOnce();
  });
});
