import { describe, expect, it } from "vitest";
import { projectKanbanDropSortOrder, type ProjectKanbanDragTask } from "./kanban-drag";

function task(id: string, statusSortOrder: number): ProjectKanbanDragTask {
  return { id, statusSortOrder };
}

describe("projectKanbanDropSortOrder", () => {
  it("places a card between visual neighbors in ascending order", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000), task("c", 3000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "asc",
    })).toBe(2500);
  });

  it("places a card before the first visual card in ascending order", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000)],
      draggedTaskId: "dragged",
      overTaskId: "a",
      position: "before",
      sortDirection: "asc",
    })).toBe(500);
  });

  it("appends a card in ascending order when no target card is supplied", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("a", 1000), task("b", 2000)],
      draggedTaskId: "dragged",
      sortDirection: "asc",
    })).toBe(3000);
  });

  it("places a card between visual neighbors in descending order", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("c", 3000), task("b", 2000), task("a", 1000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "desc",
    })).toBe(1500);
  });

  it("places a card before the first visual card in descending order", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("b", 2000), task("a", 1000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "before",
      sortDirection: "desc",
    })).toBe(3000);
  });

  it("ignores the dragged card when calculating a same-column insertion", () => {
    expect(projectKanbanDropSortOrder({
      orderedTasks: [task("a", 1000), task("dragged", 2000), task("b", 3000)],
      draggedTaskId: "dragged",
      overTaskId: "b",
      position: "after",
      sortDirection: "asc",
    })).toBe(4000);
  });
});
