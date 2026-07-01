import { describe, expect, it } from "vitest";
import { createNotesExternalPageCover, notesPageCoverUrl } from "./page-cover";

describe("notes page covers", () => {
  it("creates an external image cover payload", () => {
    expect(createNotesExternalPageCover(" https://example.com/cover.png ")).toEqual({
      type: "external",
      external: { url: "https://example.com/cover.png" },
    });
  });

  it("rejects blank and non-image cover URLs", () => {
    expect(() => createNotesExternalPageCover(" ")).toThrow("page cover URL must not be empty");
    expect(() => createNotesExternalPageCover("https://example.com/file.pdf")).toThrow(
      "page cover URL must be a supported HTTPS image URL",
    );
  });

  it("returns preview URLs only for renderable cover sources", () => {
    expect(notesPageCoverUrl({ type: "external", external: { url: "https://example.com/cover.jpg" } })).toBe(
      "https://example.com/cover.jpg",
    );
    expect(
      notesPageCoverUrl({
        type: "file",
        file: { url: "https://example.com/cover.jpg", expiry_time: "2026-06-30T12:00:00.000Z" },
      }),
    ).toBe("https://example.com/cover.jpg");
    expect(notesPageCoverUrl({ type: "file_upload", file_upload: { id: "11111111-1111-4111-8111-111111111111" } })).toBeNull();
  });
});
