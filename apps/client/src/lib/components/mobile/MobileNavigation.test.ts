// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MobileNavigation from "./MobileNavigation.svelte";

describe("MobileNavigation", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("marks only the current top destination and navigates from its compact control", async () => {
    const onNavigate = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MobileNavigation, {
      target,
      props: {
        current: "projects",
        presentation: "top",
        onNavigate,
      },
    });
    await tick();

    const buttons = [...target.querySelectorAll<HTMLButtonElement>("button")];
    expect(buttons).toHaveLength(4);
    expect(buttons.filter((button) => button.getAttribute("aria-current") === "page")).toEqual([buttons[1]]);
    expect(target.querySelectorAll("[data-mobile-navigation-indicator]")).toHaveLength(1);
    expect(target.textContent?.trim()).toBe("");

    buttons[2]?.click();
    expect(onNavigate).toHaveBeenCalledWith("notes");
  });

  it("keeps destination labels in the larger-window rail", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MobileNavigation, {
      target,
      props: {
        current: "calendar",
        presentation: "rail",
        onNavigate: vi.fn(),
      },
    });
    await tick();

    expect(target.textContent).toContain("Calendar");
    expect(target.textContent).toContain("Projects");
    expect(target.textContent).toContain("Notes");
    expect(target.textContent).toContain("Chat");
    expect(target.querySelector("[data-mobile-navigation-indicator]")).toBeNull();
  });
});
