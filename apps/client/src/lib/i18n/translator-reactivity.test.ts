// @vitest-environment jsdom

import { mount, unmount } from "svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$lib/vault/config", () => ({
  getConfigKey: () => undefined,
  setConfigKey: () => undefined,
}));

describe("translator reactivity", () => {
  let target: HTMLDivElement | undefined;
  let component: ReturnType<typeof mount> | undefined;

  afterEach(async () => {
    if (component) await unmount(component);
    target?.remove();
    component = undefined;
    target = undefined;
  });

  it("updates mounted translations only after each catalog becomes active", async () => {
    const { getLocalization } = await import("./translator.svelte");
    const localization = getLocalization();
    await localization.setLanguagePreference("en", { persist: false });
    const { default: LocalizationTestStub } = await import(
      "./LocalizationTestStub.test.svelte"
    );
    target = document.createElement("div");
    document.body.append(target);
    component = mount(LocalizationTestStub, { target });

    expect(target.textContent?.trim()).toBe("Save");
    expect(target.querySelector("[data-localization-test-stub]")?.getAttribute("data-locale"))
      .toBe("en");

    await localization.setLanguagePreference("es", { persist: false });
    await vi.waitFor(() => {
      expect(target?.textContent?.trim()).toBe("Guardar");
      expect(target?.querySelector("[data-localization-test-stub]")?.getAttribute("data-locale"))
        .toBe("es");
    });

    await localization.setLanguagePreference("en", { persist: false });
    await vi.waitFor(() => expect(target?.textContent?.trim()).toBe("Save"));
  });
});
