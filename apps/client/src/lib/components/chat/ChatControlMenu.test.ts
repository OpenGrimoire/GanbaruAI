// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ChatControlMenu, { type ChatControlOption } from "./ChatControlMenu.svelte";

const options: ChatControlOption[] = [
  { value: "ask_for_approval", label: "Ask for approval", icon: "shield-question-mark" },
  { value: "full_access", label: "Full access", icon: "shield-alert" },
];

describe("ChatControlMenu", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("keeps its custom list open until an option is selected", async () => {
    const onChange = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatControlMenu, {
      target,
      props: {
        value: "ask_for_approval",
        options,
        ariaLabel: "Safety",
        onChange,
      },
    });

    const trigger = target.querySelector<HTMLButtonElement>("button[aria-haspopup='listbox']");
    trigger?.click();
    await tick();

    const listbox = document.body.querySelector<HTMLElement>("[role='listbox'][aria-label='Safety']");
    expect(trigger?.getAttribute("aria-expanded")).toBe("true");
    expect(listbox).not.toBeNull();
    const fullAccess = [...(listbox?.querySelectorAll<HTMLButtonElement>("button") ?? [])]
      .find((button) => button.textContent?.includes("Full access"));
    expect([...(listbox?.querySelectorAll<HTMLElement>(".option-icon") ?? [])]
      .every((icon) => icon.classList.contains("option-icon") && !icon.classList.contains("warning"))).toBe(true);
    fullAccess?.click();
    await tick();

    expect(onChange).toHaveBeenCalledWith("full_access");
    expect(trigger?.getAttribute("aria-expanded")).toBe("false");
  });

  it("closes when focus moves to an outside pointer target", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ChatControlMenu, {
      target,
      props: {
        value: "ask_for_approval",
        options,
        ariaLabel: "Safety",
        onChange: vi.fn(),
      },
    });

    const trigger = target.querySelector<HTMLButtonElement>("button[aria-haspopup='listbox']");
    trigger?.click();
    await tick();
    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await tick();

    expect(trigger?.getAttribute("aria-expanded")).toBe("false");
  });
});
