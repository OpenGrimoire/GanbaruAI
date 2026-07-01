import { describe, expect, it } from "vitest";
import {
  isNotesInsertableBlockType,
  notesInsertableBlockTypes,
} from "./block-insertion";

describe("notes block insertion catalog", () => {
  it("includes user-created block types", () => {
    expect(notesInsertableBlockTypes()).toContain("paragraph");
    expect(notesInsertableBlockTypes()).toContain("heading_4");
    expect(notesInsertableBlockTypes()).toContain("to_do");
    expect(notesInsertableBlockTypes()).toContain("column_list");
    expect(notesInsertableBlockTypes()).toContain("tab");
    expect(notesInsertableBlockTypes()).toContain("pdf");
    expect(notesInsertableBlockTypes()).toContain("link_preview");
    expect(notesInsertableBlockTypes()).toContain("template");
    expect(notesInsertableBlockTypes()).toContain("button");
  });

  it("excludes internal structural and preservation block types", () => {
    expect(notesInsertableBlockTypes()).not.toContain("column");
    expect(notesInsertableBlockTypes()).not.toContain("table_row");
    expect(notesInsertableBlockTypes()).not.toContain("child_database");
    expect(notesInsertableBlockTypes()).not.toContain("synced_block");
    expect(notesInsertableBlockTypes()).not.toContain("unsupported");
  });

  it("validates insertable block types", () => {
    expect(isNotesInsertableBlockType("bookmark")).toBe(true);
    expect(isNotesInsertableBlockType("link_preview")).toBe(true);
    expect(isNotesInsertableBlockType("child_database")).toBe(false);
    expect(isNotesInsertableBlockType("synced_block")).toBe(false);
    expect(isNotesInsertableBlockType("table_row")).toBe(false);
    expect(isNotesInsertableBlockType("unknown")).toBe(false);
  });
});
