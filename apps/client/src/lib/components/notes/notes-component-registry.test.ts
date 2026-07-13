import { describe, expect, it } from "vitest";
import {
  loadNotesOptionalComponent,
  loadNotesSurface,
  notesOptionalComponentHasLoaded,
  notesSurfaceHasLoaded,
} from "./notes-component-registry";

describe("Notes component registry", () => {
  it("loads only the requested active surface and caches its constructor", async () => {
    const first = loadNotesSurface("archive");
    const second = loadNotesSurface("archive");

    expect(second).toBe(first);
    await expect(first).resolves.toMatchObject({ kind: "archive" });
    expect(notesSurfaceHasLoaded("archive")).toBe(true);
    expect(notesSurfaceHasLoaded("trash")).toBe(false);
  });

  it("keeps navigator and maintenance constructors separate and single-flight", async () => {
    const first = loadNotesOptionalComponent("page-picker");
    const second = loadNotesOptionalComponent("page-picker");

    expect(second).toBe(first);
    await expect(first).resolves.toMatchObject({ kind: "page-picker" });
    expect(notesOptionalComponentHasLoaded("page-picker")).toBe(true);
    expect(notesOptionalComponentHasLoaded("project-navigator")).toBe(false);
    expect(notesOptionalComponentHasLoaded("project-settings")).toBe(false);
    expect(notesOptionalComponentHasLoaded("project-history")).toBe(false);
    expect(notesOptionalComponentHasLoaded("confirm-dialog")).toBe(false);
    expect(notesOptionalComponentHasLoaded("destination-picker")).toBe(false);
  });
});
