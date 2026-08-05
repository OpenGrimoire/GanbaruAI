// @vitest-environment jsdom

import { mount, tick, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import SettingsModal from "./SettingsModal.svelte";

vi.mock("@tauri-apps/api/window", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/window")>();
  return {
    ...actual,
    getCurrentWindow: () => ({ label: "main" }),
  };
});

vi.mock("$lib/stores/themeEditor.svelte", () => ({
  getThemeEditor: () => ({ editingId: null }),
}));

vi.mock("$lib/stores/viewport.svelte", () => ({
  getViewport: () => ({ below: () => false }),
}));

vi.mock("$lib/buildInfo", () => ({
  BUILD_REF: "0.0.0+test",
  GITHUB_REPOSITORY: "opengrimoire/ganbaru-ai",
}));

vi.mock("$lib/components/settings/settings-detail-registry", async () => {
  const { default: Stub } = await import("./SettingsSectionTestStub.test.svelte");
  const load = async (kind: string) => ({ kind, component: Stub });
  return {
    loadSettingsDetail: load,
    retrySettingsDetail: load,
  };
});

class ResizeObserverStub {
  observe(): void {}
  disconnect(): void {}
}

describe("SettingsModal remount state", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
    vi.unstubAllGlobals();
  });

  it("does not retain the previous active section across modal instances", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverStub);
    target = document.createElement("div");
    document.body.append(target);

    component = mount(SettingsModal, {
      target,
      props: { initialSection: "about", onClose: () => undefined },
    });
    await tick();
    expect(target.querySelector("[data-settings-modal-panel]")?.getAttribute("data-settings-section"))
      .toBe("about");
    expect(target.textContent).not.toContain("Loading");
    await unmount(component);
    component = mount(SettingsModal, {
      target,
      props: { onClose: () => undefined },
    });
    await tick();
    expect(target.querySelector("[data-settings-modal-panel]")?.getAttribute("data-settings-section"))
      .toBe("appearance");
    expect(target.textContent).not.toContain("Loading");
  });
});
