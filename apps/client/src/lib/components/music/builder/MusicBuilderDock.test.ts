// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MusicBuilderDock from "./MusicBuilderDock.svelte";

describe("MusicBuilderDock", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("renders six destinations with one active label and useful badges", async () => {
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderDock, {
      target,
      props: {
        destination: { kind: "review" },
        reviewCount: 12,
        issueCount: 3,
        onNavigate: vi.fn(),
      },
    });
    await tick();

    const items = [...target.querySelectorAll<HTMLButtonElement>("[data-builder-dock-item]")];
    expect(items).toHaveLength(6);
    expect(items[0]?.getAttribute("aria-current")).toBe("page");
    expect(items[0]?.textContent).toContain("Review");
    expect(items[1]?.textContent).not.toContain("Playlists");
    expect(target.textContent).toContain("12");
    expect(target.textContent).toContain("3");
  });

  it("moves focus with arrows and activates the selected destination", async () => {
    const onNavigate = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderDock, {
      target,
      props: {
        destination: { kind: "review" },
        reviewCount: 0,
        issueCount: 0,
        onNavigate,
      },
    });
    await tick();

    const items = [...target.querySelectorAll<HTMLButtonElement>("[data-builder-dock-item]")];
    items[0]?.focus();
    items[0]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }));
    expect(document.activeElement).toBe(items[1]);
    items[1]?.click();
    expect(onNavigate).toHaveBeenCalledWith({ kind: "playlists" });
  });
});
