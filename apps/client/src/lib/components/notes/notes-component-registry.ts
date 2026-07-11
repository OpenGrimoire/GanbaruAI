import {
  createLazyComponentLoader,
  type LazyComponentImporter,
} from "$lib/lazy-component-loader";

export type NotesSurfaceKind = "home" | "editor" | "archive" | "trash";

export type LoadedNotesSurface =
  | { kind: "home"; component: typeof import("./NotesProjectHome.svelte").default }
  | { kind: "editor"; component: typeof import("./NotesEditor.svelte").default }
  | { kind: "archive"; component: typeof import("./NotesArchiveView.svelte").default }
  | { kind: "trash"; component: typeof import("./NotesTrashView.svelte").default };

export type NotesOptionalComponentKind =
  | "project-settings"
  | "project-history"
  | "confirm-dialog"
  | "page-picker"
  | "project-navigator"
  | "destination-picker";

export type LoadedNotesOptionalComponent =
  | {
      kind: "project-settings";
      component: typeof import("./NotesProjectSettingsPanel.svelte").default;
    }
  | {
      kind: "project-history";
      component: typeof import("./NotesProjectVersionHistoryModal.svelte").default;
    }
  | {
      kind: "confirm-dialog";
      component: typeof import("$lib/components/ui/ConfirmDialog.svelte").default;
    }
  | {
      kind: "page-picker";
      component: typeof import("./NotesPagePickerPanel.svelte").default;
    }
  | {
      kind: "project-navigator";
      component: typeof import("./NotesProjectNavigator.svelte").default;
    }
  | {
      kind: "destination-picker";
      component: typeof import("./NotesDestinationPickerList.svelte").default;
    };

const SURFACE_IMPORTERS = {
  home: () => import("./NotesProjectHome.svelte")
    .then((module) => ({ default: { kind: "home" as const, component: module.default } })),
  editor: () => import("./NotesEditor.svelte")
    .then((module) => ({ default: { kind: "editor" as const, component: module.default } })),
  archive: () => import("./NotesArchiveView.svelte")
    .then((module) => ({ default: { kind: "archive" as const, component: module.default } })),
  trash: () => import("./NotesTrashView.svelte")
    .then((module) => ({ default: { kind: "trash" as const, component: module.default } })),
} satisfies Readonly<Record<NotesSurfaceKind, LazyComponentImporter<LoadedNotesSurface>>>;

const OPTIONAL_IMPORTERS = {
  "project-settings": () => import("./NotesProjectSettingsPanel.svelte")
    .then((module) => ({
      default: { kind: "project-settings" as const, component: module.default },
    })),
  "project-history": () => import("./NotesProjectVersionHistoryModal.svelte")
    .then((module) => ({
      default: { kind: "project-history" as const, component: module.default },
    })),
  "confirm-dialog": () => import("$lib/components/ui/ConfirmDialog.svelte")
    .then((module) => ({
      default: { kind: "confirm-dialog" as const, component: module.default },
    })),
  "page-picker": () => import("./NotesPagePickerPanel.svelte")
    .then((module) => ({
      default: { kind: "page-picker" as const, component: module.default },
    })),
  "project-navigator": () => import("./NotesProjectNavigator.svelte")
    .then((module) => ({
      default: { kind: "project-navigator" as const, component: module.default },
    })),
  "destination-picker": () => import("./NotesDestinationPickerList.svelte")
    .then((module) => ({
      default: { kind: "destination-picker" as const, component: module.default },
    })),
} satisfies Readonly<Record<
  NotesOptionalComponentKind,
  LazyComponentImporter<LoadedNotesOptionalComponent>
>>;

const surfaceLoader = createLazyComponentLoader<NotesSurfaceKind, LoadedNotesSurface>(
  SURFACE_IMPORTERS,
);
const optionalLoader = createLazyComponentLoader<
  NotesOptionalComponentKind,
  LoadedNotesOptionalComponent
>(OPTIONAL_IMPORTERS);

/** Loads and caches the constructor for one active Notes surface. */
export function loadNotesSurface(kind: NotesSurfaceKind): Promise<LoadedNotesSurface> {
  return surfaceLoader.load(kind);
}

/** Retries an active Notes surface import after a failure. */
export function retryNotesSurface(kind: NotesSurfaceKind): Promise<LoadedNotesSurface> {
  return surfaceLoader.retry(kind);
}

/** Reports whether one active Notes surface constructor has loaded. */
export function notesSurfaceHasLoaded(kind: NotesSurfaceKind): boolean {
  return surfaceLoader.hasLoaded(kind);
}

/** Loads and caches one trigger-owned Notes component constructor. */
export function loadNotesOptionalComponent(
  kind: NotesOptionalComponentKind,
): Promise<LoadedNotesOptionalComponent> {
  return optionalLoader.load(kind);
}

/** Retries a trigger-owned Notes component import after a failure. */
export function retryNotesOptionalComponent(
  kind: NotesOptionalComponentKind,
): Promise<LoadedNotesOptionalComponent> {
  return optionalLoader.retry(kind);
}

/** Reports whether one trigger-owned Notes constructor has loaded. */
export function notesOptionalComponentHasLoaded(kind: NotesOptionalComponentKind): boolean {
  return optionalLoader.hasLoaded(kind);
}
