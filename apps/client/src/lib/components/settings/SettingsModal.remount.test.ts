// @vitest-environment jsdom

import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$lib/stores/themeEditor.svelte", () => ({
  getThemeEditor: () => ({ editingId: null }),
}));

vi.mock("$lib/stores/viewport.svelte", () => ({
  getViewport: () => ({ below: () => false }),
}));

vi.mock("$lib/components/settings/settings-section-registry", async () => {
  const { default: Stub } = await import("./SettingsSectionTestStub.test.svelte");
  const load = async (section: string) => ({ section, component: Stub });
  return {
    loadSettingsSection: load,
    retrySettingsSection: load,
  };
});

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
    const { default: SettingsModal } = await import("./SettingsModal.svelte");

    component = mount(SettingsModal, {
      target,
      props: { initialSection: "updates", onClose: () => undefined },
    });
    await vi.waitFor(() => {
      expect(target?.querySelector("[data-settings-modal-panel]")?.getAttribute("data-settings-section"))
        .toBe("updates");
      expect(target?.querySelector("[data-settings-section-test-stub]")).not.toBeNull();
    });

    await unmount(component);
    component = mount(SettingsModal, {
      target,
      props: { onClose: () => undefined },
    });
    await vi.waitFor(() => {
      expect(target?.querySelector("[data-settings-modal-panel]")?.getAttribute("data-settings-section"))
        .toBe("appearance");
      expect(target?.querySelector("[data-settings-section-test-stub]")).not.toBeNull();
    });
  });
});
