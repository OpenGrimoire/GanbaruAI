import { externalMediaUrlIsSupported } from "./media";
import type { NotesPageCover } from "./types";

/** Create a Notion-style external page cover payload for the Tauri update boundary. */
export function createNotesExternalPageCover(url: string): NotesPageCover {
  const trimmed = url.trim();
  if (!trimmed) throw new Error("page cover URL must not be empty");
  if (!externalMediaUrlIsSupported("image", trimmed)) {
    throw new Error("page cover URL must be a supported HTTPS image URL");
  }
  return { type: "external", external: { url: trimmed } };
}

/** Return the previewable URL for cover sources that can be rendered locally. */
export function notesPageCoverUrl(cover: NotesPageCover | null): string | null {
  if (!cover) return null;
  if (cover.type === "external") return cover.external.url;
  if (cover.type === "file") return cover.file.url;
  return null;
}
