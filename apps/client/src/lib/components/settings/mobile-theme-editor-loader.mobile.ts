import type { MobileThemeEditorComponent } from "./mobile-theme-editor-loader-contract";

let loaded: MobileThemeEditorComponent | null = null;
let pending: Promise<MobileThemeEditorComponent> | null = null;

/** Lazily load and cache the Android full-screen theme editor host. */
export function loadMobileThemeEditor(): Promise<MobileThemeEditorComponent> {
  if (loaded) return Promise.resolve(loaded);
  pending ??= import("./mobile/MobileThemeEditor.svelte")
    .then((module) => {
      loaded = module.default;
      return loaded;
    })
    .finally(() => {
      pending = null;
    });
  return pending;
}

/** Retry a failed editor import without discarding a successful cached load. */
export function retryMobileThemeEditor(): Promise<MobileThemeEditorComponent> {
  pending = null;
  return loadMobileThemeEditor();
}
