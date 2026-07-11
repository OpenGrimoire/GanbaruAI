import {
  createLazyComponentLoader,
  type LazyComponentImporter,
} from "$lib/lazy-component-loader";
import type { SectionId } from "./types";

export type LoadedSettingsSection =
  | { section: "appearance"; component: typeof import("./AppearanceSection.svelte").default }
  | { section: "profile"; component: typeof import("./ProfileSection.svelte").default }
  | { section: "calendars"; component: typeof import("./CalendarsSection.svelte").default }
  | { section: "projects"; component: typeof import("./ProjectsSection.svelte").default }
  | { section: "notes"; component: typeof import("./NotesSection.svelte").default }
  | { section: "focus"; component: typeof import("./FocusSection.svelte").default }
  | { section: "music"; component: typeof import("./MusicSection.svelte").default }
  | { section: "doomscrolling"; component: typeof import("./DoomscrollingSection.svelte").default }
  | { section: "data"; component: typeof import("./DataSection.svelte").default }
  | { section: "updates"; component: typeof import("./UpdatesSection.svelte").default }
  | { section: "shortcuts"; component: typeof import("./ShortcutsSection.svelte").default }
  | { section: "about"; component: typeof import("./AboutSection.svelte").default };

const SECTION_IMPORTERS = {
  appearance: () => import("./AppearanceSection.svelte")
    .then((module) => ({ default: { section: "appearance" as const, component: module.default } })),
  profile: () => import("./ProfileSection.svelte")
    .then((module) => ({ default: { section: "profile" as const, component: module.default } })),
  calendars: () => import("./CalendarsSection.svelte")
    .then((module) => ({ default: { section: "calendars" as const, component: module.default } })),
  projects: () => import("./ProjectsSection.svelte")
    .then((module) => ({ default: { section: "projects" as const, component: module.default } })),
  notes: () => import("./NotesSection.svelte")
    .then((module) => ({ default: { section: "notes" as const, component: module.default } })),
  focus: () => import("./FocusSection.svelte")
    .then((module) => ({ default: { section: "focus" as const, component: module.default } })),
  music: () => import("./MusicSection.svelte")
    .then((module) => ({ default: { section: "music" as const, component: module.default } })),
  doomscrolling: () => import("./DoomscrollingSection.svelte")
    .then((module) => ({ default: { section: "doomscrolling" as const, component: module.default } })),
  data: () => import("./DataSection.svelte")
    .then((module) => ({ default: { section: "data" as const, component: module.default } })),
  updates: () => import("./UpdatesSection.svelte")
    .then((module) => ({ default: { section: "updates" as const, component: module.default } })),
  shortcuts: () => import("./ShortcutsSection.svelte")
    .then((module) => ({ default: { section: "shortcuts" as const, component: module.default } })),
  about: () => import("./AboutSection.svelte")
    .then((module) => ({ default: { section: "about" as const, component: module.default } })),
} satisfies Readonly<Record<SectionId, LazyComponentImporter<LoadedSettingsSection>>>;

const loader = createLazyComponentLoader<SectionId, LoadedSettingsSection>(SECTION_IMPORTERS);

/** Loads and caches the constructor for one Settings section. */
export function loadSettingsSection(section: SectionId): Promise<LoadedSettingsSection> {
  return loader.load(section);
}

/** Retries a failed Settings section import. */
export function retrySettingsSection(section: SectionId): Promise<LoadedSettingsSection> {
  return loader.retry(section);
}

/** Reports whether a section constructor is already cached. */
export function settingsSectionHasLoaded(section: SectionId): boolean {
  return loader.hasLoaded(section);
}
