import type { NotesPageIcon } from "./types";

export const NOTES_PAGE_EMOJI_ICON_CHOICES = [
  "📝",
  "📌",
  "📚",
  "✅",
  "💡",
  "🎯",
  "🧠",
  "🗓️",
  "🔖",
  "⭐",
  "🚧",
  "🧭",
  "🧩",
  "📊",
  "🛠️",
  "🌱",
] as const;

export type NotesPageEmojiIconChoice = (typeof NOTES_PAGE_EMOJI_ICON_CHOICES)[number];

/** Create a validated page emoji icon payload for the Tauri update boundary. */
export function createNotesEmojiPageIcon(emoji: string): NotesPageIcon {
  const trimmed = emoji.trim();
  if (!trimmed) throw new Error("page icon emoji must not be empty");
  return { type: "emoji", emoji: trimmed };
}

/** Return the text glyph used for a page icon when the current renderer supports it. */
export function notesPageIconText(icon: NotesPageIcon | null): string | null {
  return icon?.type === "emoji" ? icon.emoji : null;
}
