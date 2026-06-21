import { describe, expect, it } from "vitest";
import {
  projectListDropSortOrder,
  projectListSectionDropSortOrder,
  type ProjectListDragSection,
  type ProjectListDragTask,
} from "./list-drag";

function task(id: string, sectionSortOrder: number): ProjectListDragTask {
  return { id, sectionSortOrder };
}

function section(id: string, sortOrder: number): ProjectListDragSection {
  return { id, sortOrder };
}

describe("projectListDropSortOrder", () => {
  it("places a task between visual neighbors in ascending order", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000), task("c", 3000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "asc",
    })).toBe(2500);
  });

  it("places a task before the first visual task in ascending order", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000)],
      draggedTaskId: "dragged",
      overTaskId: "a",
      position: "before",
      sortDirection: "asc",
    })).toBe(500);
  });

  it("appends a task in ascending order when no target task is supplied", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000)],
      draggedTaskId: "dragged",
      sortDirection: "asc",
    })).toBe(3000);
  });

  it("places a task between visual neighbors in descending order", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("c", 3000), task("b", 2000), task("a", 1000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "desc",
    })).toBe(1500);
  });

  it("places a task before the first visual task in descending order", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("b", 2000), task("a", 1000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "before",
      sortDirection: "desc",
    })).toBe(3000);
  });

  it("ignores the dragged task when calculating a same-section insertion", () => {
    expect(projectListDropSortOrder({
      orderedTasks: [task("a", 1000), task("dragged", 2000), task("b", 3000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "asc",
    })).toBe(4000);
  });
});

describe("projectListSectionDropSortOrder", () => {
  it("places a section between visual neighbors", () => {
    expect(projectListSectionDropSortOrder({
      orderedSections: [section("a", 1000), section("b", 2000), section("c", 3000)],
      draggedSectionId: "dragged",
      overSectionId: "b",
      position: "after",
    })).toBe(2500);
  });

  it("places a section before the first section", () => {
    expect(projectListSectionDropSortOrder({
      orderedSections: [section("a", 1000), section("b", 2000)],
      draggedSectionId: "dragged",
      overSectionId: "a",
      position: "before",
    })).toBe(500);
  });

  it("appends a section when no target section is supplied", () => {
    expect(projectListSectionDropSortOrder({
      orderedSections: [section("a", 1000), section("b", 2000)],
      draggedSectionId: "dragged",
    })).toBe(3000);
  });

  it("ignores the dragged section when calculating same-list insertion", () => {
    expect(projectListSectionDropSortOrder({
      orderedSections: [section("a", 1000), section("dragged", 2000), section("b", 3000)],
      draggedSectionId: "dragged",
      overSectionId: "b",
      position: "after",
    })).toBe(4000);
  });
});
