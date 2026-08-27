import { afterEach, describe, expect, it, vi } from "vitest";
import {
  CALENDAR_TOUCH_LONG_PRESS_MS,
  CALENDAR_SWIPE_MAX_PREVIEW_PX,
  CalendarTouchHoldArbiter,
  calendarSwipeAxis,
  calendarSwipeDirection,
  calendarSwipeNavigation,
  calendarSwipePreviewOffset,
  calendarTouchMovedBeyondSlop,
} from "./calendar-mobile-gestures";

function pointerEvent(
  type: string,
  pointerId: number,
  clientX: number,
  clientY: number,
): PointerEvent {
  const event = new Event(type);
  Object.defineProperties(event, {
    pointerId: { value: pointerId },
    clientX: { value: clientX },
    clientY: { value: clientY },
  });
  return event as PointerEvent;
}

function installWindowEventTarget(): EventTarget {
  const eventTarget = new EventTarget();
  vi.stubGlobal("window", {
    addEventListener: eventTarget.addEventListener.bind(eventTarget),
    removeEventListener: eventTarget.removeEventListener.bind(eventTarget),
    setTimeout: globalThis.setTimeout.bind(globalThis),
    clearTimeout: globalThis.clearTimeout.bind(globalThis),
  });
  return eventTarget;
}

describe("mobile calendar gestures", () => {
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("keeps small pointer drift below touch slop", () => {
    expect(calendarTouchMovedBeyondSlop(6, 6)).toBe(false);
    expect(calendarTouchMovedBeyondSlop(11, 0)).toBe(true);
  });

  it("reserves dominant horizontal movement for view navigation", () => {
    expect(calendarSwipeAxis(30, 8)).toBe("horizontal");
    expect(calendarSwipeAxis(8, 30)).toBe("vertical");
    expect(calendarSwipeAxis(4, 2)).toBeNull();
  });

  it("commits by distance or velocity and maps finger direction", () => {
    expect(calendarSwipeDirection(-12)).toBe("forward");
    expect(calendarSwipeDirection(12)).toBe("back");
    expect(calendarSwipeDirection(0)).toBeNull();
    expect(calendarSwipeNavigation({ deltaX: -80, elapsedMs: 400, viewportWidth: 360 })).toBe("forward");
    expect(calendarSwipeNavigation({ deltaX: 60, elapsedMs: 80, viewportWidth: 360 })).toBe("back");
    expect(calendarSwipeNavigation({ deltaX: -45, elapsedMs: 300, viewportWidth: 360 })).toBe("forward");
    expect(calendarSwipeNavigation({ deltaX: 30, elapsedMs: 400, viewportWidth: 360 })).toBeNull();
  });

  it("bounds the visual preview with increasing resistance", () => {
    expect(calendarSwipePreviewOffset(0)).toBe(0);
    expect(calendarSwipePreviewOffset(-40)).toBeLessThan(0);
    expect(Math.abs(calendarSwipePreviewOffset(1000))).toBeLessThanOrEqual(CALENDAR_SWIPE_MAX_PREVIEW_PX);
  });

  it("activates a stationary hold and owns later touch movement", () => {
    vi.useFakeTimers();
    const eventTarget = installWindowEventTarget();
    const activate = vi.fn();
    const arbiter = new CalendarTouchHoldArbiter();

    arbiter.begin(pointerEvent("pointerdown", 3, 20, 30), activate);
    vi.advanceTimersByTime(CALENDAR_TOUCH_LONG_PRESS_MS - 1);
    expect(activate).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(activate).toHaveBeenCalledOnce();
    expect(arbiter.editingActive).toBe(true);

    const touchMove = new Event("touchmove", { cancelable: true });
    eventTarget.dispatchEvent(touchMove);
    expect(touchMove.defaultPrevented).toBe(true);
    arbiter.finish();
  });

  it("cancels a pending hold once movement crosses touch slop", () => {
    vi.useFakeTimers();
    const eventTarget = installWindowEventTarget();
    const activate = vi.fn();
    const arbiter = new CalendarTouchHoldArbiter();

    arbiter.begin(pointerEvent("pointerdown", 4, 20, 30), activate);
    eventTarget.dispatchEvent(pointerEvent("pointermove", 4, 31, 30));
    vi.advanceTimersByTime(CALENDAR_TOUCH_LONG_PRESS_MS);

    expect(activate).not.toHaveBeenCalled();
    expect(arbiter.editingActive).toBe(false);
  });
});
