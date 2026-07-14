// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { dismissOnOutside } from "./dismiss-on-outside";

describe("dismissOnOutside", () => {
  it("ignores pointer events inside the node", () => {
    const root = document.createElement("div");
    const button = document.createElement("button");
    root.append(button);
    document.body.append(root);
    const onDismiss = vi.fn();
    const action = dismissOnOutside(root, { onDismiss });

    button.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));

    expect(onDismiss).not.toHaveBeenCalled();
    action.destroy();
    root.remove();
  });

  it("dismisses on pointer events outside the node", () => {
    const root = document.createElement("div");
    const outside = document.createElement("button");
    document.body.append(root, outside);
    const onDismiss = vi.fn();
    const action = dismissOnOutside(root, { onDismiss });

    outside.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));

    expect(onDismiss).toHaveBeenCalledWith("outside-pointer", expect.any(PointerEvent));
    action.destroy();
    root.remove();
    outside.remove();
  });

  it("dismisses on Escape", () => {
    const root = document.createElement("div");
    document.body.append(root);
    const onDismiss = vi.fn();
    const action = dismissOnOutside(root, { onDismiss });

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

    expect(onDismiss).toHaveBeenCalledWith("escape", expect.any(KeyboardEvent));
    action.destroy();
    root.remove();
  });

  it("respects disabled updates", () => {
    const root = document.createElement("div");
    const outside = document.createElement("button");
    document.body.append(root, outside);
    const onDismiss = vi.fn();
    const action = dismissOnOutside(root, { onDismiss });

    action.update({ enabled: false, onDismiss });
    outside.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

    expect(onDismiss).not.toHaveBeenCalled();
    action.destroy();
    root.remove();
    outside.remove();
  });
});
