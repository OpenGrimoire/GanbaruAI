import type { NotesCommentParent, NotesSearchResult } from "./types";

export interface NotesSearchNavigationPlan {
  pageId: string;
  blockId: string | null;
  commentParent: NotesCommentParent | null;
}

export function notesSearchNavigationPlan(
  result: NotesSearchResult,
): NotesSearchNavigationPlan {
  const blockId = result.block_id;
  if (result.type !== "comment") {
    return {
      pageId: result.page.id,
      blockId,
      commentParent: null,
    };
  }
  return {
    pageId: result.page.id,
    blockId,
    commentParent: blockId
      ? { type: "block_id", block_id: blockId }
      : { type: "page_id", page_id: result.page.id },
  };
}
