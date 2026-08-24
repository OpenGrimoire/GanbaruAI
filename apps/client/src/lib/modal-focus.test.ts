// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import { activateModalFocus, trapModalTabKey } from "./modal-focus";

describe("modal focus", () => {
  afterEach(() => {
    document.body.replaceChildren();
  });

  it("wraps forward and reverse Tab navigation", () => {
    const modal = document.createElement("div");
    const first = document.createElement("button");
    const last = document.createElement("button");
    modal.append(first, last);
    document.body.append(modal);

    last.focus();
    expect(trapModalTabKey(modal, new KeyboardEvent("keydown", { key: "Tab", cancelable: true })))
      .toBe(true);
    expect(document.activeElement).toBe(first);

    first.focus();
    expect(trapModalTabKey(
      modal,
      new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, cancelable: true }),
    )).toBe(true);
    expect(document.activeElement).toBe(last);
  });

  it("restores focus to the invoking control", async () => {
    const trigger = document.createElement("button");
    const modal = document.createElement("div");
    const close = document.createElement("button");
    modal.append(close);
    document.body.append(trigger, modal);
    trigger.focus();

    const deactivate = activateModalFocus(modal, close);
    await Promise.resolve();
    expect(document.activeElement).toBe(close);
    deactivate();
    await Promise.resolve();
    expect(document.activeElement).toBe(trigger);
  });

  it("does not let an inert parent modal compete with a nested modal", () => {
    const parent = document.createElement("div");
    const parentButton = document.createElement("button");
    const nested = document.createElement("div");
    const nestedButton = document.createElement("button");
    parent.append(parentButton);
    nested.append(nestedButton);
    document.body.append(parent, nested);
    parent.setAttribute("inert", "");
    nestedButton.focus();

    const event = new KeyboardEvent("keydown", { key: "Tab", cancelable: true });
    expect(trapModalTabKey(parent, event)).toBe(false);
    expect(event.defaultPrevented).toBe(false);
    expect(document.activeElement).toBe(nestedButton);
  });
});
