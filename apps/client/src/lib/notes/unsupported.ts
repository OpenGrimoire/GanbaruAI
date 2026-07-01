import type { NotesUnsupportedBlockPayload } from "./types";

function normalizedText(value: unknown): string {
  return typeof value === "string" ? value.trim() : "";
}

/** Return the imported block kind that should be shown to users. */
export function unsupportedBlockTypeName(payload: NotesUnsupportedBlockPayload): string {
  return normalizedText(payload.block_type) || normalizedText(payload.source_type);
}

/** Return import warnings that are safe to show in the editor. */
export function unsupportedBlockWarnings(payload: NotesUnsupportedBlockPayload): string[] {
  if (!Array.isArray(payload.warnings)) return [];
  return payload.warnings.map(normalizedText).filter(Boolean);
}

/** Return searchable placeholder text for an unsupported block. */
export function unsupportedBlockPlainText(payload: NotesUnsupportedBlockPayload): string {
  return [unsupportedBlockTypeName(payload), ...unsupportedBlockWarnings(payload)]
    .map((part) => part.trim())
    .filter(Boolean)
    .join(" ");
}
