export type ProjectBoardDropPosition = "before" | "after";
export type ProjectBoardSortDirection = "asc" | "desc";

export interface ProjectBoardDragTask {
  id: string;
  statusSortOrder: number;
}

export interface ProjectBoardDropInput {
  orderedTasks: readonly ProjectBoardDragTask[];
  draggedTaskId: string;
  overTaskId?: string;
  position?: ProjectBoardDropPosition;
  sortDirection: ProjectBoardSortDirection;
}

const SORT_STEP = 1000;

function midpoint(a: number, b: number): number {
  return a + (b - a) / 2;
}

function orderBeforeFirst(first: ProjectBoardDragTask, direction: ProjectBoardSortDirection): number {
  if (direction === "desc") return first.statusSortOrder + SORT_STEP;
  return first.statusSortOrder > 0 ? first.statusSortOrder / 2 : 0;
}

function orderAfterLast(last: ProjectBoardDragTask, direction: ProjectBoardSortDirection): number {
  if (direction === "desc") return last.statusSortOrder > 0 ? last.statusSortOrder / 2 : 0;
  return last.statusSortOrder + SORT_STEP;
}

function orderBetween(
  before: ProjectBoardDragTask,
  after: ProjectBoardDragTask,
  direction: ProjectBoardSortDirection,
): number {
  return direction === "desc"
    ? midpoint(after.statusSortOrder, before.statusSortOrder)
    : midpoint(before.statusSortOrder, after.statusSortOrder);
}

export function projectBoardDropSortOrder(input: ProjectBoardDropInput): number {
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
