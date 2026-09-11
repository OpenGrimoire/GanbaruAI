// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { attachNotesBlockSelectionDelegates } from "./notes-block-selection-controller.svelte";

describe("Notes block selection controller", () => {
  it("removes delegated listeners when the list action is destroyed", () => {
    const node = document.createElement("div");
    const pointerDown = vi.fn();
    const pointerOver = vi.fn();
    const keydown = vi.fn();
    const action = attachNotesBlockSelectionDelegates(node, { pointerDown, pointerOver, keydown });

    node.dispatchEvent(new Event("pointerdown"));
    node.dispatchEvent(new Event("pointerover"));
    node.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect([pointerDown, pointerOver, keydown].map((handler) => handler.mock.calls.length))
      .toEqual([1, 1, 1]);

    action.destroy();
    node.dispatchEvent(new Event("pointerdown"));
    node.dispatchEvent(new Event("pointerover"));
    node.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect([pointerDown, pointerOver, keydown].map((handler) => handler.mock.calls.length))
      .toEqual([1, 1, 1]);
  });
});
