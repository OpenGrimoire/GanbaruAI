// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getMobileBackStack } from "../../stores/mobile-back-stack.svelte";
import MobilePomodoroSheet from "./MobilePomodoroSheet.svelte";

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
  stopSession: vi.fn(async () => undefined),
}));

vi.mock("$lib/stores/pomodoro.svelte", () => ({
  getPomodoro: () => pomodoro,
}));

vi.mock("$lib/platform", () => ({
  BUILD_PLATFORM_PROFILE: {
    platform: "android",
    shell: "mobile",
    capabilities: ["system.android-back"],
  },
  platformHasCapability: (_profile: unknown, capability: string) =>
    capability === "system.android-back",
}));

async function flushFocusUpdates(): Promise<void> {
  await tick();
  await new Promise<void>((resolve) => queueMicrotask(resolve));
}

describe("MobilePomodoroSheet", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;
  let removeTestBackLayer: (() => void) | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    removeTestBackLayer?.();
    target?.remove();
    component = undefined;
    removeTestBackLayer = undefined;
    target = undefined;
    pomodoro.stopSession.mockClear();
  });

  it("anchors the sheet and confirmation to the visual viewport", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MobilePomodoroSheet, {
      target,
      props: { onClose: vi.fn(), onOpenCalendar: vi.fn() },
    });
    await tick();

    const sheet = target.querySelector<HTMLElement>("[data-mobile-pomodoro-sheet]");
    expect(sheet?.style.left).toBe("var(--visual-viewport-offset-left)");
    expect(sheet?.style.top).toBe("var(--visual-viewport-offset-top)");
    expect(sheet?.style.width).toBe("var(--visual-viewport-width)");
    expect(sheet?.style.height).toBe("var(--visual-viewport-height)");

    const stopButton = target.querySelector<HTMLButtonElement>("[data-mobile-pomodoro-stop]");
    expect(stopButton?.classList.contains("min-h-14")).toBe(true);
    stopButton?.click();
    await tick();

    const confirmation = target.querySelector<HTMLElement>(".confirm-dialog");
    const confirmationViewport = confirmation?.parentElement;
    expect(confirmationViewport?.style.left).toBe("var(--visual-viewport-offset-left)");
    expect(confirmationViewport?.style.top).toBe("var(--visual-viewport-offset-top)");
    expect(confirmationViewport?.style.width).toBe("var(--visual-viewport-width)");
    expect(confirmationViewport?.style.height).toBe("var(--visual-viewport-height)");
    expect(sheet?.inert).toBe(true);
  });

  it("uses Android Back to cancel the confirmation before closing the sheet", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const onClose = vi.fn();
    const sheetBack = vi.fn();
    removeTestBackLayer = getMobileBackStack().activate({ handle: sheetBack });
    component = mount(MobilePomodoroSheet, {
      target,
      props: { onClose, onOpenCalendar: vi.fn() },
    });
    await flushFocusUpdates();

    const stopButton = target.querySelector<HTMLButtonElement>("[data-mobile-pomodoro-stop]");
    stopButton?.focus();
    stopButton?.click();
    await flushFocusUpdates();
    expect(target.querySelector(".confirm-dialog")).not.toBeNull();

    expect(getMobileBackStack().consume()).toBe(true);
    await flushFocusUpdates();
    expect(target.querySelector(".confirm-dialog")).toBeNull();
    expect(document.activeElement).toBe(stopButton);
    expect(sheetBack).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();

    expect(getMobileBackStack().consume()).toBe(true);
    expect(sheetBack).toHaveBeenCalledOnce();
    removeTestBackLayer?.();
    removeTestBackLayer = undefined;
  });

  it("stops the active session only after confirmation", async () => {
    target = document.createElement("div");
    document.body.append(target);
    const onClose = vi.fn();
    component = mount(MobilePomodoroSheet, {
      target,
      props: { onClose, onOpenCalendar: vi.fn() },
    });
    await tick();

    target.querySelector<HTMLButtonElement>("[data-mobile-pomodoro-stop]")?.click();
    await tick();
    expect(pomodoro.stopSession).not.toHaveBeenCalled();

    const confirmationButtons = target.querySelectorAll<HTMLButtonElement>(".confirm-dialog button");
    confirmationButtons.item(1).click();
    await vi.waitFor(() => {
      expect(pomodoro.stopSession).toHaveBeenCalledOnce();
      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  it("directs an inactive session to its calendar-owned start flow", async () => {
    pomodoro.isActive = false;
    pomodoro.canPauseResume = false;
    pomodoro.canAddFocusTime = false;
    target = document.createElement("div");
    document.body.append(target);
    const onOpenCalendar = vi.fn();
    component = mount(MobilePomodoroSheet, {
      target,
      props: { onClose: vi.fn(), onOpenCalendar },
    });
    await tick();

    expect(target.querySelector("[data-mobile-pomodoro-inactive]")).not.toBeNull();
    expect(target.querySelector("[data-mobile-pomodoro-stop]")).toBeNull();
    target.querySelector<HTMLButtonElement>("[data-mobile-pomodoro-inactive] button")?.click();
    expect(onOpenCalendar).toHaveBeenCalledOnce();

    pomodoro.isActive = true;
    pomodoro.canPauseResume = true;
    pomodoro.canAddFocusTime = true;
  });
});
