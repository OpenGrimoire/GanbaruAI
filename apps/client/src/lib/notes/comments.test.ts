import { describe, expect, it } from "vitest";
import { parseNotesCommentThread } from "./block-validation";
import {
  notesCommentParentKey,
  notesCommentPlainText,
  notesCommentThreadSnippet,
  openNotesCommentThreadCount,
} from "./comments";
import { createRichText } from "./block-factory";

const comment = {
  object: "comment",
  id: "10101010-1010-4010-8010-101010101010",
  parent: { type: "block_id", block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa" },
  discussion_id: "90909090-9090-4090-8090-909090909090",
  created_time: "2026-06-30T00:00:00.000Z",
  last_edited_time: "2026-06-30T00:00:00.000Z",
  created_by: { object: "user", id: "local-user" },
  rich_text: [createRichText("Review this")],
  attachments: [],
  display_name: { type: "user", resolved_name: "You" },
  deleted_at: null,
};

describe("notes comments", () => {
  it("parses comment thread DTOs from the Tauri boundary", () => {
    const thread = parseNotesCommentThread({
      object: "comment_thread",
      id: "90909090-9090-4090-8090-909090909090",
      parent: { type: "block_id", block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa" },
      page_id: "11111111-1111-4111-8111-111111111111",
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      status: "open",
      resolved_at: null,
      resolved_by: null,
      created_time: "2026-06-30T00:00:00.000Z",
      last_edited_time: "2026-06-30T00:00:00.000Z",
      comments: [comment],
    });

    expect(thread.block_id).toBe("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    expect(notesCommentPlainText(thread.comments[0])).toBe("Review this");
  });

  it("rejects workspace parents for comments", () => {
    expect(() =>
      parseNotesCommentThread({
        object: "comment_thread",
        id: "90909090-9090-4090-8090-909090909090",
        parent: { type: "workspace", workspace: true },
        page_id: "11111111-1111-4111-8111-111111111111",
        block_id: null,
        status: "open",
        resolved_at: null,
        resolved_by: null,
        created_time: "2026-06-30T00:00:00.000Z",
        last_edited_time: "2026-06-30T00:00:00.000Z",
        comments: [],
      }),
    ).toThrow("comment thread.parent.type must be page_id or block_id");
  });

  it("summarizes open threads and parent keys", () => {
    const openThread = parseNotesCommentThread({
      object: "comment_thread",
      id: "90909090-9090-4090-8090-909090909090",
      parent: { type: "block_id", block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa" },
      page_id: "11111111-1111-4111-8111-111111111111",
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      status: "open",
      resolved_at: null,
      resolved_by: null,
      created_time: "2026-06-30T00:00:00.000Z",
      last_edited_time: "2026-06-30T00:00:00.000Z",
      comments: [comment],
    });
    const resolvedThread = { ...openThread, id: "80808080-8080-4080-8080-808080808080", status: "resolved" as const };

    expect(openNotesCommentThreadCount([openThread, resolvedThread])).toBe(1);
    expect(notesCommentThreadSnippet(openThread)).toBe("Review this");
    expect(notesCommentParentKey(openThread.parent)).toBe("block:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
  });
});
