// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { localFileSourceFromPath } from "$lib/music/sources";
import { getMusicPlayer } from "$lib/stores/music-player.svelte";

class ResizeObserverStub {
  observe(): void {}
  disconnect(): void {}
}

describe("MusicPanel", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    vi.unstubAllGlobals();
  });

  it("mounts an interactive dialog above its persistent media layer and closes with Escape", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverStub);
    target = document.createElement("div");
    document.body.append(target);
    const onclose = vi.fn();
    const { default: MusicPanel } = await import("./MusicPanel.svelte");

    component = mount(MusicPanel, { target, props: { onclose } });
    await tick();

    const dialog = target.querySelector<HTMLElement>("[role='dialog']");
    expect(dialog).not.toBeNull();
    expect(dialog?.classList.contains("z-70")).toBe(true);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(onclose).toHaveBeenCalledOnce();
  });

  it("does not reset persistent playback state when the panel closes", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverStub);
    target = document.createElement("div");
    document.body.append(target);
    const player = getMusicPlayer();
    const source = localFileSourceFromPath("/music/persistent.flac", "Persistent");
    player.currentSource = source;
    player.queue = [source];
    const { default: MusicPanel } = await import("./MusicPanel.svelte");

    component = mount(MusicPanel, { target, props: { onclose: vi.fn() } });
    await tick();
    await unmount(component);
    component = undefined;

    expect(getMusicPlayer()).toBe(player);
    expect(player.currentSource).toEqual(source);
    expect(player.queue).toEqual([source]);

    player.currentSource = null;
    player.queue = [];
  });

  it("keeps the player DOM mounted while opening and reopening the lazy builder", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverStub);
    target = document.createElement("div");
    document.body.append(target);
    const { default: MusicPanel } = await import("./MusicPanel.svelte");

    component = mount(MusicPanel, { target, props: { onclose: vi.fn() } });
    await tick();
    const playerPage = target.querySelector<HTMLElement>("[data-music-player-page]");
    expect(playerPage).not.toBeNull();

    target.querySelector<HTMLButtonElement>("[data-music-playlist-launcher]")?.click();
    await tick();
    const firstOpenBuilder = [...target.querySelectorAll<HTMLButtonElement>("button")]
      .find((button) => button.textContent?.includes("Open builder"));
    firstOpenBuilder?.click();
    await vi.waitFor(() => {
      expect(target?.querySelector(".builder-root"), target?.textContent ?? "").not.toBeNull();
    }, { timeout: 5_000 });
    expect(target.querySelector("[data-music-player-page]")).toBe(playerPage);
    expect(playerPage?.classList.contains("hidden")).toBe(true);

    target.querySelector<HTMLButtonElement>(`[data-music-focus-key="builder:back-to-player"]`)?.click();
    await tick();
    expect(target.querySelector("[data-music-player-page]")).toBe(playerPage);
    expect(playerPage?.classList.contains("hidden")).toBe(false);

    target.querySelector<HTMLButtonElement>("[data-music-playlist-launcher]")?.click();
    await tick();
    const openBuilder = [...target.querySelectorAll<HTMLButtonElement>("button")]
      .find((button) => button.textContent?.includes("Open builder"));
    openBuilder?.click();
    await tick();
    expect(target.querySelector(".builder-root")).not.toBeNull();
    expect(target.querySelector("[data-music-player-page]")).toBe(playerPage);
  });
});
