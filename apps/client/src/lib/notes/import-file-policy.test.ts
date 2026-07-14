import { describe, expect, it } from "vitest";
import {
  classifyNotesImportFileReference,
  notesImportFileChoicesForReference,
} from "./import-file-policy";

describe("notes import file policy helpers", () => {
  it("classifies external, managed, local, and empty references", () => {
    expect(classifyNotesImportFileReference("")).toBe("empty");
    expect(classifyNotesImportFileReference(" https://example.com/file.pdf ")).toBe(
      "external_url",
    );
    expect(classifyNotesImportFileReference("http://example.com/file.pdf")).toBe("external_url");
    expect(classifyNotesImportFileReference("ganbaru-asset:notes/files/a.pdf")).toBe(
      "managed_asset",
    );
    expect(classifyNotesImportFileReference("assets/image.png")).toBe("local_file");
  });

  it("offers only explicit safe choices for each reference kind", () => {
    expect(notesImportFileChoicesForReference("", true)).toEqual(["skip"]);
    expect(notesImportFileChoicesForReference("ganbaru-asset:notes/files/a.pdf", true)).toEqual([
      "skip",
    ]);
    expect(notesImportFileChoicesForReference("https://example.com/file.pdf", true)).toEqual([
      "keep_external_reference",
      "skip",
    ]);
    expect(notesImportFileChoicesForReference("files/image.png", false)).toEqual(["skip"]);
    expect(notesImportFileChoicesForReference("files/image.png", true)).toEqual([
      "copy_local_file",
      "skip",
    ]);
  });
});
