import { describe, expect, it } from "vitest";
import {
  createNotesExternalPageCover,
  createNotesLocalFilePageCover,
  notesPageCoverAssetPath,
  notesPageCoverPresetBackground,
  notesPageCoverUrl,
  NOTES_PAGE_COVER_PRESETS,
} from "./page-cover";

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

  it("creates local file cover payloads from managed assets", () => {
    expect(
      createNotesLocalFilePageCover({
        relativePath: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp",
        originalName: " cover.webp ",
        contentType: "image/webp",
        byteSize: 42,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      }),
    ).toEqual({
      type: "file",
      file: {
        url: "ganbaru-asset:notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp",
        name: "cover.webp",
        content_type: "image/webp",
        byte_size: 42,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ganbaru_asset_path: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp",
      },
    });
  });

  it("rejects unsafe local file cover assets", () => {
    expect(() =>
      createNotesLocalFilePageCover({
        relativePath: "notes/page-icons/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp",
        contentType: "image/webp",
        byteSize: 42,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      }),
    ).toThrow("page cover asset path must stay under notes/page-covers");
    expect(() =>
      createNotesLocalFilePageCover({
        relativePath: "notes/page-covers/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp",
        contentType: "image/svg+xml",
        byteSize: 42,
        sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      }),
    ).toThrow("page cover asset must be a PNG, JPG, or WebP image");
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
    expect(
      notesPageCoverAssetPath({
        type: "file",
        file: {
          url: "ganbaru-asset:notes/page-covers/a.png",
          ganbaru_asset_path: "notes/page-covers/a.png",
        },
      }),
    ).toBe("notes/page-covers/a.png");
  });

  it("exposes generated cover preset backgrounds", () => {
    expect(NOTES_PAGE_COVER_PRESETS.length).toBeGreaterThan(0);
    expect(notesPageCoverPresetBackground(NOTES_PAGE_COVER_PRESETS[0])).toContain("linear-gradient");
  });
});
