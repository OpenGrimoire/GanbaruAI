import { render } from "svelte/server";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  beginLazyComponentLoad,
  rejectLazyComponentLoad,
  resolveLazyComponentLoad,
} from "$lib/lazy-component-loader";
import {
  loadSettingsSection,
  settingsSectionHasLoaded,
  type LoadedSettingsSection,
} from "./settings-section-registry";
import { SETTINGS_SECTION_IDS, type SectionId } from "./types";

vi.mock("@tauri-apps/api/window", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@tauri-apps/api/window")>();
  return {
    ...actual,
    getCurrentWindow: () => ({ label: "main" }),
  };
});

function renderSettingsSection(loaded: LoadedSettingsSection): void {
  switch (loaded.section) {
    case "notes":
      render(loaded.component, { props: { onOpenTransferPanel: () => undefined } });
      return;
    case "doomscrolling":
      render(loaded.component, { props: { onOpenLimitEditor: () => undefined } });
      return;
    case "appearance":
      render(loaded.component, { props: {} });
      return;
    case "profile":
      render(loaded.component, { props: {} });
      return;
    case "calendars":
      render(loaded.component, { props: {} });
      return;
    case "projects":
      render(loaded.component, { props: {} });
      return;
    case "focus":
      render(loaded.component, { props: {} });
      return;
    case "music":
      render(loaded.component, { props: {} });
      return;
    case "data":
      render(loaded.component, { props: {} });
      return;
    case "updates":
      render(loaded.component, { props: {} });
      return;
    case "shortcuts":
      render(loaded.component, { props: {} });
      return;
    case "about":
      render(loaded.component, { props: {} });
  }
}

describe("Settings section registry", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("loads only the requested constructor and caches successful modules", async () => {
    expect(settingsSectionHasLoaded("appearance")).toBe(false);
    const first = await loadSettingsSection("appearance");
    const second = await loadSettingsSection("appearance");

    expect(first.section).toBe("appearance");
    expect(second).toBe(first);
    expect(settingsSectionHasLoaded("appearance")).toBe(true);
    for (const section of SETTINGS_SECTION_IDS) {
      if (section !== "appearance") expect(settingsSectionHasLoaded(section)).toBe(false);
    }
  });

  it("renders every registered Settings route", async () => {
    vi.stubGlobal("__GANBARU_AI_BUILD_REF__", "0.0.0+test");
    vi.stubGlobal("__GANBARU_AI_GITHUB_REPOSITORY__", "opengrimoire/ganbaru-ai");
    for (const section of SETTINGS_SECTION_IDS) {
      const loaded = await loadSettingsSection(section);
      expect(loaded.section).toBe(section);
      expect(() => renderSettingsSection(loaded)).not.toThrow();
    }
  });

  it("does not let a slow previous section replace the current route", () => {
    const appearance = beginLazyComponentLoad<SectionId, string>(null, "appearance");
    const updates = beginLazyComponentLoad<SectionId, string>(appearance, "updates");

    expect(
      resolveLazyComponentLoad(
        updates,
        "appearance",
        appearance.requestId,
        "appearance component",
      ),
    ).toBe(updates);
    expect(
      rejectLazyComponentLoad(
        updates,
        "appearance",
        appearance.requestId,
        new Error("stale"),
      ),
    ).toBe(updates);
    expect(
      resolveLazyComponentLoad(updates, "updates", updates.requestId, "updates component"),
    ).toEqual({
      status: "ready",
      key: "updates",
      requestId: updates.requestId,
      component: "updates component",
    });
  });
});
