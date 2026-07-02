import { describe, expect, it } from "vitest";
import { notesSearchNavigationPlan } from "./search";
import type { NotesPage, NotesSearchResult } from "./types";

const page: NotesPage = {
  object: "page",
  id: "11111111-1111-4111-8111-111111111111",
  created_time: "2026-06-30T12:00:00.000Z",
  last_edited_time: "2026-06-30T12:00:00.000Z",
  parent: { type: "workspace", workspace: true },
  in_trash: false,
  archived: false,
  icon: null,
  cover: null,
  properties: {},
  url: null,
  public_url: null,
  source_provider: null,
  source_object_id: null,
  source_workspace_id: null,
  source_last_edited_time: null,
};

function searchResult(overrides: Partial<NotesSearchResult>): NotesSearchResult {
  return {
    object: "search_result",
    id: "page:11111111-1111-4111-8111-111111111111",
    type: "page",
    page,
    block_id: null,
    block_type: null,
    comment_id: null,
    discussion_id: null,
    comment_status: null,
    comment_author: null,
    comment_anchor: null,
    snippet: "",
    last_edited_time: "2026-06-30T12:00:00.000Z",
    ...overrides,
  };
}

describe("notes search navigation", () => {
  it("opens block results at their target block without activating comments", () => {
    expect(notesSearchNavigationPlan(searchResult({
      id: "block:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      type: "block",
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      block_type: "paragraph",
    }))).toEqual({
      pageId: page.id,
      blockId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      commentParent: null,
    });
  });

  it("opens page discussion comment results at the page comments", () => {
    expect(notesSearchNavigationPlan(searchResult({
      id: "comment:10101010-1010-4010-8010-101010101010",
      type: "comment",
      comment_id: "10101010-1010-4010-8010-101010101010",
      discussion_id: "90909090-9090-4090-8090-909090909090",
      comment_status: "open",
      comment_author: { type: "user", resolved_name: "Reviewer" },
    }))).toEqual({
      pageId: page.id,
      blockId: null,
      commentParent: { type: "page_id", page_id: page.id },
    });
  });

  it("opens inline comment results at the anchored block comments", () => {
    expect(notesSearchNavigationPlan(searchResult({
      id: "comment:10101010-1010-4010-8010-101010101010",
      type: "comment",
      block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      comment_id: "10101010-1010-4010-8010-101010101010",
      discussion_id: "90909090-9090-4090-8090-909090909090",
      comment_status: "resolved",
      comment_author: { type: "user", resolved_name: "Reviewer" },
      comment_anchor: {
        object: "comment_anchor",
        type: "text_range",
        block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        start: 0,
        end: 6,
        text: "Anchor",
        prefix: "",
        suffix: " text",
        created_time: "2026-06-30T12:00:00.000Z",
        last_edited_time: "2026-06-30T12:00:00.000Z",
      },
    }))).toEqual({
      pageId: page.id,
      blockId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      commentParent: {
        type: "block_id",
        block_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      },
    });
  });
});
