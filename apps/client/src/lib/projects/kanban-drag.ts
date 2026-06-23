export type ProjectKanbanDropPosition = "before" | "after";
export type ProjectKanbanSortDirection = "asc" | "desc";

export interface ProjectKanbanDragTask {
  id: string;
  statusSortOrder: number;
}

export interface ProjectKanbanDropInput {
  orderedTasks: readonly ProjectKanbanDragTask[];
  draggedTaskId: string;
  overTaskId?: string;
  position?: ProjectKanbanDropPosition;
  sortDirection: ProjectKanbanSortDirection;
}

const SORT_STEP = 1000;

function midpoint(a: number, b: number): number {
  return a + (b - a) / 2;
}

function orderBeforeFirst(first: ProjectKanbanDragTask, direction: ProjectKanbanSortDirection): number {
  if (direction === "desc") return first.statusSortOrder + SORT_STEP;
  return first.statusSortOrder > 0 ? first.statusSortOrder / 2 : 0;
}

function orderAfterLast(last: ProjectKanbanDragTask, direction: ProjectKanbanSortDirection): number {
  if (direction === "desc") return last.statusSortOrder > 0 ? last.statusSortOrder / 2 : 0;
  return last.statusSortOrder + SORT_STEP;
}

function orderBetween(
  before: ProjectKanbanDragTask,
  after: ProjectKanbanDragTask,
  direction: ProjectKanbanSortDirection,
): number {
  return direction === "desc"
    ? midpoint(after.statusSortOrder, before.statusSortOrder)
    : midpoint(before.statusSortOrder, after.statusSortOrder);
}

export function projectKanbanDropSortOrder(input: ProjectKanbanDropInput): number {
  const candidates = input.orderedTasks.filter((task) => task.id !== input.draggedTaskId);
  const overIndex = input.overTaskId
    ? candidates.findIndex((task) => task.id === input.overTaskId)
    : -1;
  const insertIndex = overIndex >= 0
    ? overIndex + (input.position === "after" ? 1 : 0)
    : candidates.length;

  const before = candidates[insertIndex - 1];
  const after = candidates[insertIndex];
  if (before && after) return orderBetween(before, after, input.sortDirection);
  if (after) return orderBeforeFirst(after, input.sortDirection);
  if (before) return orderAfterLast(before, input.sortDirection);
  return SORT_STEP;
}
