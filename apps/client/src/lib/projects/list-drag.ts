export type ProjectListDropPosition = "before" | "after";
export type ProjectListSortDirection = "asc" | "desc";

export interface ProjectListDragTask {
  id: string;
  sectionSortOrder: number;
}

export interface ProjectListDragSection {
  id: string;
  sortOrder: number;
}

export interface ProjectListDropInput {
  orderedTasks: readonly ProjectListDragTask[];
  draggedTaskId: string;
  overTaskId?: string;
  position?: ProjectListDropPosition;
  sortDirection: ProjectListSortDirection;
}

export interface ProjectListSectionDropInput {
  orderedSections: readonly ProjectListDragSection[];
  draggedSectionId: string;
  overSectionId?: string;
  position?: ProjectListDropPosition;
}

const SORT_STEP = 1000;

function midpoint(a: number, b: number): number {
  return a + (b - a) / 2;
}

function orderBeforeFirst(first: ProjectListDragTask, direction: ProjectListSortDirection): number {
  if (direction === "desc") return first.sectionSortOrder + SORT_STEP;
  return first.sectionSortOrder > 0 ? first.sectionSortOrder / 2 : 0;
}

function orderAfterLast(last: ProjectListDragTask, direction: ProjectListSortDirection): number {
  if (direction === "desc") return last.sectionSortOrder > 0 ? last.sectionSortOrder / 2 : 0;
  return last.sectionSortOrder + SORT_STEP;
}

function orderBetween(
  before: ProjectListDragTask,
  after: ProjectListDragTask,
  direction: ProjectListSortDirection,
): number {
  return direction === "desc"
    ? midpoint(after.sectionSortOrder, before.sectionSortOrder)
    : midpoint(before.sectionSortOrder, after.sectionSortOrder);
}

export function projectListDropSortOrder(input: ProjectListDropInput): number {
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

export function projectListSectionDropSortOrder(input: ProjectListSectionDropInput): number {
  const candidates = input.orderedSections.filter((section) => section.id !== input.draggedSectionId);
  const overIndex = input.overSectionId
    ? candidates.findIndex((section) => section.id === input.overSectionId)
    : -1;
  const insertIndex = overIndex >= 0
    ? overIndex + (input.position === "after" ? 1 : 0)
    : candidates.length;

  const before = candidates[insertIndex - 1];
  const after = candidates[insertIndex];
  if (before && after) return midpoint(before.sortOrder, after.sortOrder);
  if (after) return after.sortOrder > 0 ? after.sortOrder / 2 : 0;
  if (before) return before.sortOrder + SORT_STEP;
  return SORT_STEP;
}
