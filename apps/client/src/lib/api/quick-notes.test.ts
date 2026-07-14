import { describe, expect, it } from "vitest";
import { mapQuickNote, mapQuickNoteTag } from "./quick-notes";

function validQuickNote(): Record<string, unknown> {
  return {
    id: "note-1",
    title: "Plan",
    bodyPlainText: "Start here",
    runs: [
      {
        content: "Start here",
        bold: true,
        italic: false,
        underline: false,
      },
    ],
    previewTruncated: false,
    color: 4,
    tagId: "tag-1",
    pinned: true,
    archived: false,
    trashedAt: null,
    revision: 2,
    createdAt: "2026-07-13T12:00:00.000Z",
    updatedAt: "2026-07-13T12:01:00.000Z",
  };
}

describe("Quick notes API boundary", () => {
  it("maps a valid canonical row", () => {
    expect(mapQuickNote(validQuickNote())).toMatchObject({
      id: "note-1",
      color: 4,
      revision: 2,
      pinned: true,
    });
  });

  it("rejects malformed rows and nested runs", () => {
    expect(() => mapQuickNote({ ...validQuickNote(), color: 32 })).toThrow("color is invalid");
    expect(() =>
      mapQuickNote({
        ...validQuickNote(),
        runs: [{ content: "text", bold: "yes", italic: false, underline: false }],
      }),
    ).toThrow("runs[0].bold must be a boolean");
    expect(() => mapQuickNote({ ...validQuickNote(), revision: 1.5 })).toThrow(
      "revision must be an integer",
    );
  });

  it("validates canonical tag rows", () => {
    const validTag = {
      id: "tag-1",
      name: "Work",
      sortOrder: 8,
      createdAt: "2026-07-13T12:00:00.000Z",
      updatedAt: "2026-07-13T12:00:00.000Z",
    };
    expect(mapQuickNoteTag(validTag)).toMatchObject({ name: "Work", sortOrder: 8 });
    expect(() => mapQuickNoteTag({ ...validTag, name: " Work " })).toThrow("name is invalid");
    expect(() => mapQuickNoteTag({ ...validTag, sortOrder: 9 })).toThrow("sortOrder is invalid");
  });
});
