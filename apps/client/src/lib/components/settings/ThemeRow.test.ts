// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import { lightTheme } from "$lib/stores/themes";
import ThemeRow from "./ThemeRow.svelte";

describe("ThemeRow mobile actions", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("keeps editor and file actions available in the compact mobile row", async () => {
    const onOpen = vi.fn();
    const onDuplicate = vi.fn();
    const onExport = vi.fn();
    target = document.createElement("div");
    document.body.append(target);
    component = mount(ThemeRow, {
      target,
      props: {
        theme: lightTheme,
        isActive: false,
        isBuiltin: true,
        onApply: vi.fn(),
        onOpen,
        onDuplicate,
        onExport,
        onDelete: vi.fn(),
        mobileLayout: true,
      },
    });
    await tick();

    const duplicate = target.querySelector<HTMLButtonElement>(
      '[aria-label="Duplicate and edit theme"]',
    );
    const open = target.querySelector<HTMLButtonElement>('[aria-label="View theme"]');
    const exportButton = target.querySelector<HTMLButtonElement>(
      '[aria-label="Export theme JSON"]',
    );
    expect(duplicate).not.toBeNull();
    expect(open).not.toBeNull();
    expect(exportButton).not.toBeNull();
    expect(open?.classList.contains("size-8")).toBe(true);
    expect(open?.querySelector(".theme-action-visual")).toBeNull();

    duplicate?.click();
    open?.click();
    exportButton?.click();
    expect(onDuplicate).toHaveBeenCalledOnce();
    expect(onOpen).toHaveBeenCalledOnce();
    expect(onExport).toHaveBeenCalledOnce();
  });
});
