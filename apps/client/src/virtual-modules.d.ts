declare module "virtual:ganbaru-ai-platform-entry" {
  const appPromise: Promise<unknown>;

  export default appPromise;
}

declare module "$lib/components/settings/doomscrolling-desktop-selector" {
  import type {
    DoomscrollingAppSelection,
    DoomscrollingAppSelectorMode,
  } from "$lib/components/settings/DoomscrollingAppSelector.svelte";

  const component: typeof import("$lib/components/settings/DoomscrollingAppSelector.svelte").default;
  export { type DoomscrollingAppSelection, type DoomscrollingAppSelectorMode };
  export default component;
}

declare module "$lib/components/settings/doomscrolling-browser-connection" {
  const component: typeof import("$lib/components/settings/DoomscrollingBrowserConnectionStatus.svelte").default;
  export default component;
}
