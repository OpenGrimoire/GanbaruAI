import { describe, expect, it } from "vitest";
import { lightTheme } from "$lib/stores/themes";
import { translateFromPartialCatalog, type Translate } from "$lib/i18n/translator.svelte";
import {
  projectLabelColorDotStyle,
  projectLabelColorSwatchClass,
  projectLifecycleBadgeClass,
  projectPersonInitials,
  projectPriorityBadgeClass,
  projectPriorityLabel,
  projectStatusBadgeClass,
  projectTaskArchivedBadgeClass,
} from "./project-display";

const t = ((key, ...args) => translateFromPartialCatalog({}, key, ...args)) as Translate;

describe("project display helpers", () => {
  it("returns localized priority labels", () => {
    expect(projectPriorityLabel("urgent", t)).toBe("Urgent");
    expect(projectPriorityLabel("normal", t)).toBe("Normal");
  });

  it("returns lifecycle and task badge classes", () => {
    expect(projectLifecycleBadgeClass("hidden")).toContain("amber");
    expect(projectLifecycleBadgeClass("archived")).toContain("muted");
    expect(projectTaskArchivedBadgeClass({ archivedAt: "2026-06-21T00:00:00.000Z" })).toContain("muted");
    expect(projectTaskArchivedBadgeClass({ archivedAt: undefined })).toContain("emerald");
  });

  it("returns status and priority tone classes", () => {
    expect(projectStatusBadgeClass(undefined)).toContain("muted");
    expect(projectStatusBadgeClass({
      id: "status-a",
      projectId: "project-a",
      name: "Blocked",
      category: "blocked",
      sortOrder: 1000,
      terminal: false,
      createdAt: "2026-06-21T00:00:00.000Z",
      updatedAt: "2026-06-21T00:00:00.000Z",
    })).toContain("destructive");
    expect(projectPriorityBadgeClass("high")).toContain("amber");
  });

  it("returns label swatch styling only when a palette color exists", () => {
    expect(projectLabelColorSwatchClass(undefined)).toContain("muted");
    expect(projectLabelColorDotStyle(undefined, lightTheme)).toBe("");
    expect(projectLabelColorDotStyle(2, lightTheme)).toMatch(/^background-color: #[0-9a-f]{6};$/i);
  });

  it("returns compact initials for task people", () => {
    expect(projectPersonInitials("You")).toBe("Y");
    expect(projectPersonInitials("Victor Rivera")).toBe("VR");
    expect(projectPersonInitials("  Ana Maria Lopez  ")).toBe("AM");
    expect(projectPersonInitials("")).toBe("?");
  });
});
