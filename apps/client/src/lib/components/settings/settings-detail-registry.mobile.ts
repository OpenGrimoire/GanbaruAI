import type { SettingsDetailKind } from "./types";
import type { LoadedSettingsDetail } from "./settings-detail-registry";

function unavailableDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  return Promise.reject(new Error(`Settings detail ${kind} is unavailable on mobile.`));
}

/** Reject desktop-only settings detail surfaces without importing their component graphs. */
export function loadSettingsDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  return unavailableDetail(kind);
}

/** Reject retries for settings detail surfaces that have no mobile implementation. */
export function retrySettingsDetail(kind: SettingsDetailKind): Promise<LoadedSettingsDetail> {
  return unavailableDetail(kind);
}

/** Mobile settings never cache a desktop-only detail surface. */
export function settingsDetailHasLoaded(_kind: SettingsDetailKind): boolean {
  return false;
}
