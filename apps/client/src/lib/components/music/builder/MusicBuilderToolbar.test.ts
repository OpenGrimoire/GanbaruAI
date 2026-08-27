// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import MusicBuilderToolbar from "./MusicBuilderToolbar.svelte";

describe("MusicBuilderToolbar", () => {
  let target: HTMLDivElement | null = null;
  let component: ReturnType<typeof mount> | null = null;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = null;
    target = null;
  });

  it("uses the compact player label without weakening its accessible name", async () => {
    const onOpenPlayer = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(MusicBuilderToolbar, {
      target,
      props: {
        status: "Ready",
        compactPlayerLabel: true,
        onOpenPlayer,
      },
    });
    await tick();

    const button = target.querySelector<HTMLButtonElement>("[data-music-focus-key='builder:back-to-player']");
    expect(button?.textContent?.trim()).toBe("Return");
    expect(button?.getAttribute("aria-label")).toBe("Return to music player");
    button?.click();
    expect(onOpenPlayer).toHaveBeenCalledOnce();
  });
});
