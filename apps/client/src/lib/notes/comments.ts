import type { NotesComment, NotesCommentParent, NotesCommentThread } from "./types";
import { richTextPlainText } from "./rich-text";

export function notesCommentPlainText(comment: NotesComment): string {
  return richTextPlainText(comment.rich_text);
}

export function notesCommentThreadSnippet(thread: NotesCommentThread): string {
  return thread.comments
    .map(notesCommentPlainText)
    .map((text) => text.trim())
    .find((text) => text.length > 0) ?? "";
}

export function openNotesCommentThreadCount(
  threads: readonly NotesCommentThread[],
): number {
  return threads.filter((thread) => thread.status === "open").length;
}

export function notesCommentParentKey(parent: NotesCommentParent): string {
  if (parent.type === "page_id") return `page:${parent.page_id}`;
  return `block:${parent.block_id}`;
}
