import {
  createLazyComponentLoader,
  type LazyComponentImporter,
} from "$lib/lazy-component-loader";
import { importNotesProjectVersionHistoryModal } from "$lib/components/notes/notes-project-platform-importers";

export type NotesSurfaceKind = "archive" | "trash";

export type LoadedNotesSurface =
  | { kind: "archive"; component: typeof import("./NotesArchiveView.svelte").default }
  | { kind: "trash"; component: typeof import("./NotesTrashView.svelte").default };

export type NotesOptionalComponentKind =
  | "project-history"
  | "confirm-dialog"
  | "destination-picker";

export type LoadedNotesOptionalComponent =
  | {
      kind: "project-history";
      component: typeof import("./NotesProjectVersionHistoryModal.svelte").default;
    }
  | {
      kind: "confirm-dialog";
      component: typeof import("$lib/components/ui/ConfirmDialog.svelte").default;
    }
  | {
      kind: "destination-picker";
      component: typeof import("./NotesDestinationPickerList.svelte").default;
    };

const SURFACE_IMPORTERS = {
  archive: () => import("./NotesArchiveView.svelte")
    .then((module) => ({ default: { kind: "archive" as const, component: module.default } })),
  trash: () => import("./NotesTrashView.svelte")
    .then((module) => ({ default: { kind: "trash" as const, component: module.default } })),
} satisfies Readonly<Record<NotesSurfaceKind, LazyComponentImporter<LoadedNotesSurface>>>;

const OPTIONAL_IMPORTERS = {
  "project-history": () => importNotesProjectVersionHistoryModal()
    .then((module) => ({
      default: { kind: "project-history" as const, component: module.default },
    })),
  "confirm-dialog": () => import("$lib/components/ui/ConfirmDialog.svelte")
    .then((module) => ({
      default: { kind: "confirm-dialog" as const, component: module.default },
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
