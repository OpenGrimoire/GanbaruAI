import type { SettingsDetailKind } from "./types";
import type { LoadedSettingsDetail } from "./settings-detail-registry";

function unavailableDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  return Promise.reject(new Error(`Settings detail ${kind} is unavailable on mobile.`));
}

let doomscrollingLimitLoaded = false;

/** Reject desktop-only settings detail surfaces without importing their component graphs. */
export function loadSettingsDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  if (kind === "doomscrolling-limit") {
    return import("./DoomscrollingLimitEditor.svelte").then((module) => ({
      kind: "doomscrolling-limit" as const,
      component: module.default,
    })).then((loaded) => {
      doomscrollingLimitLoaded = true;
      return loaded;
    });
  }
  return unavailableDetail(kind);
}

/** Reject retries for settings detail surfaces that have no mobile implementation. */
export function retrySettingsDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  return loadSettingsDetail(kind);
}

/** Mobile settings never cache a desktop-only detail surface. */
export function settingsDetailHasLoaded(kind: SettingsDetailKind): boolean {
  return kind === "doomscrolling-limit" && doomscrollingLimitLoaded;
}
