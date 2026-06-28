import { parseSavedTaskViewPreference } from "$lib/projects/saved-task-views";
import { normalizeProjectName } from "$lib/projects/project-text";
import type {
  Project,
  ProjectChecklistItem,
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldValue,
  ProjectGroup,
  ProjectTag,
  ProjectPriorityConfig,
  ProjectSavedTaskView,
  ProjectSection,
  ProjectsSnapshot,
  ProjectStatus,
  ProjectTask,
  ProjectTaskChangeEvent,
  ProjectTaskDependency,
  ProjectTaskEventLink,
  ProjectTaskTagLink,
} from "$lib/projects/types";

function sortByOrderAndName<T extends { sortOrder: number; name: string }>(a: T, b: T): number {
  return a.sortOrder - b.sortOrder || a.name.localeCompare(b.name);
}

function sortTasksBySectionOrder(a: ProjectTask, b: ProjectTask): number {
  return a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt);
}

function sortTasksByStatusOrder(a: ProjectTask, b: ProjectTask): number {
  return a.statusSortOrder - b.statusSortOrder || a.createdAt.localeCompare(b.createdAt);
}

export function activeProjects(source: ProjectsSnapshot): Project[] {
  return source.projects
    .filter((project) => project.status === "active")
    .sort(sortByOrderAndName);
}

export function firstProjectId(source: ProjectsSnapshot): string | null {
  return activeProjects(source)[0]?.id ?? source.projects[0]?.id ?? null;
}

export function projectById(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): Project | undefined {
  if (!projectId) return undefined;
  return source.projects.find((project) => project.id === projectId);
}

export function groupById(
  source: ProjectsSnapshot,
  groupId: string | null | undefined,
): ProjectGroup | undefined {
  if (!groupId) return undefined;
  return source.groups.find((group) => group.id === groupId);
}

export function sectionsForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectSection[] {
  if (!projectId) return [];
  return source.sections
    .filter((section) => section.projectId === projectId && !section.archivedAt && !section.hiddenAt)
    .sort(sortByOrderAndName);
}

export function sectionsForProjectIncludingInactive(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectSection[] {
  if (!projectId) return [];
  return source.sections
    .filter((section) => section.projectId === projectId)
    .sort(sortByOrderAndName);
}

export function statusesForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectStatus[] {
  if (!projectId) return [];
  return source.statuses
    .filter((status) => status.projectId === projectId)
    .sort(sortByOrderAndName);
}

export function prioritiesForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectPriorityConfig[] {
  if (!projectId) return [];
  return source.priorities
    .filter((priority) => priority.projectId === projectId)
    .sort(sortByOrderAndName);
}

export function tasksForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectTask[] {
  if (!projectId) return [];
  return source.tasks
    .filter((task) => task.projectId === projectId && !task.archivedAt)
    .sort(sortTasksBySectionOrder);
}

export function tasksForProjectIncludingArchived(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectTask[] {
  if (!projectId) return [];
  return source.tasks
    .filter((task) => task.projectId === projectId)
    .sort(sortTasksBySectionOrder);
}

export function topLevelTasksForSection(
  source: ProjectsSnapshot,
  projectId: string,
  sectionId: string,
): ProjectTask[] {
  return source.tasks
    .filter((task) =>
      task.projectId === projectId
      && task.sectionId === sectionId
      && !task.parentTaskId
      && !task.archivedAt
    )
    .sort(sortTasksBySectionOrder);
}

export function topLevelTasksForStatus(
  source: ProjectsSnapshot,
  projectId: string,
  statusId: string,
): ProjectTask[] {
  return source.tasks
    .filter((task) =>
      task.projectId === projectId
      && task.statusId === statusId
      && !task.parentTaskId
      && !task.archivedAt
    )
    .sort(sortTasksByStatusOrder);
}

export function subtasksForTask(
  source: ProjectsSnapshot,
  parentTaskId: string | null | undefined,
): ProjectTask[] {
  if (!parentTaskId) return [];
  return source.tasks
    .filter((task) => task.parentTaskId === parentTaskId && !task.archivedAt)
    .sort(sortTasksBySectionOrder);
}

export function subtasksForTaskIncludingArchived(
  source: ProjectsSnapshot,
  parentTaskId: string | null | undefined,
): ProjectTask[] {
  if (!parentTaskId) return [];
  return source.tasks
    .filter((task) => task.parentTaskId === parentTaskId)
    .sort(sortTasksBySectionOrder);
}

export function projectsForGroup(source: ProjectsSnapshot, groupId: string): Project[] {
  return source.projects
    .filter((project) => project.groupId === groupId && project.status === "active")
    .sort(sortByOrderAndName);
}

export function projectsForGroupIncludingInactive(source: ProjectsSnapshot, groupId: string): Project[] {
  return source.projects
    .filter((project) => project.groupId === groupId)
    .sort(sortByOrderAndName);
}

export function visibleGroups(source: ProjectsSnapshot): ProjectGroup[] {
  return source.groups
    .filter((group) => !group.hiddenAt && !group.archivedAt)
    .sort(sortByOrderAndName);
}

export function defaultSection(source: ProjectsSnapshot, projectId: string): ProjectSection | undefined {
  return sectionsForProject(source, projectId)[0];
}

export function defaultStatus(source: ProjectsSnapshot, projectId: string): ProjectStatus | undefined {
  const statuses = statusesForProject(source, projectId);
  return statuses.find((status) => status.name.toLowerCase() === "to do")
    ?? statuses.find((status) => status.category === "not_started")
    ?? statuses[0];
}

export function doneStatus(source: ProjectsSnapshot, projectId: string): ProjectStatus | undefined {
  return statusesForProject(source, projectId).find((status) => status.terminal);
}

export function reopenStatus(source: ProjectsSnapshot, projectId: string): ProjectStatus | undefined {
  const statuses = statusesForProject(source, projectId);
  return statuses.find((status) => status.name.toLowerCase() === "to do")
    ?? statuses.find((status) => !status.terminal)
    ?? statuses[0];
}

export function statusById(source: ProjectsSnapshot, statusId: string): ProjectStatus | undefined {
  return source.statuses.find((status) => status.id === statusId);
}

export function taskById(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTask | undefined {
  if (!taskId) return undefined;
  return source.tasks.find((task) => task.id === taskId);
}

export function eventLinksForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTaskEventLink[] {
  if (!taskId) return [];
  return source.eventLinks.filter((link) => link.taskId === taskId);
}

export function eventLinksForEvent(
  source: ProjectsSnapshot,
  eventId: string | null | undefined,
): ProjectTaskEventLink[] {
  if (!eventId) return [];
  return source.eventLinks.filter((link) => link.eventId === eventId);
}

export function taskChangeEventsForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTaskChangeEvent[] {
  if (!taskId) return [];
  return source.taskChangeEvents
    .filter((event) => event.taskId === taskId)
    .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt));
}

export function recentTaskChangeEventsForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
  limit = 8,
): ProjectTaskChangeEvent[] {
  if (!projectId) return [];
  const projectTaskIds = new Set(
    source.tasks
      .filter((task) => task.projectId === projectId)
      .map((task) => task.id),
  );
  return source.taskChangeEvents
    .filter((event) => projectTaskIds.has(event.taskId))
    .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt))
    .slice(0, limit);
}

export function checklistItemsForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectChecklistItem[] {
  if (!taskId) return [];
  return source.checklistItems
    .filter((item) => item.taskId === taskId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.createdAt.localeCompare(b.createdAt));
}

export function tagsForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectTag[] {
  if (!projectId) return [];
  return source.tags
    .filter((tag) => tag.projectId === projectId)
    .sort(sortByOrderAndName);
}

export function tagById(
  source: ProjectsSnapshot,
  tagId: string | null | undefined,
): ProjectTag | undefined {
  if (!tagId) return undefined;
  return source.tags.find((tag) => tag.id === tagId);
}

export function taskTagLinksForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTaskTagLink[] {
  if (!taskId) return [];
  return source.taskTagLinks.filter((link) => link.taskId === taskId);
}

export function tagsForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTag[] {
  const tagIds = new Set(taskTagLinksForTask(source, taskId).map((link) => link.tagId));
  return source.tags
    .filter((tag) => tagIds.has(tag.id))
    .sort(sortByOrderAndName);
}

export function unlinkedTagsForTask(
  source: ProjectsSnapshot,
  task: ProjectTask | null | undefined,
): ProjectTag[] {
  if (!task) return [];
  const linkedTagIds = new Set(taskTagLinksForTask(source, task.id).map((link) => link.tagId));
  return tagsForProject(source, task.projectId).filter((tag) => !linkedTagIds.has(tag.id));
}

export function projectTagByName(
  source: ProjectsSnapshot,
  projectId: string,
  name: string,
): ProjectTag | undefined {
  const normalized = normalizeProjectName(name).toLowerCase();
  if (!normalized) return undefined;
  return source.tags.find((tag) =>
    tag.projectId === projectId && tag.name.trim().toLowerCase() === normalized
  );
}

export function customFieldsForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectCustomField[] {
  if (!projectId) return [];
  return source.customFields
    .filter((field) => field.projectId === projectId)
    .sort(sortByOrderAndName);
}

export function customFieldById(
  source: ProjectsSnapshot,
  fieldId: string | null | undefined,
): ProjectCustomField | undefined {
  if (!fieldId) return undefined;
  return source.customFields.find((field) => field.id === fieldId);
}

export function customFieldByName(
  source: ProjectsSnapshot,
  projectId: string,
  name: string,
): ProjectCustomField | undefined {
  const normalized = normalizeProjectName(name).toLowerCase();
  if (!normalized) return undefined;
  return source.customFields.find((field) =>
    field.projectId === projectId && field.name.trim().toLowerCase() === normalized
  );
}

export function customFieldOptionsForField(
  source: ProjectsSnapshot,
  fieldId: string | null | undefined,
): ProjectCustomFieldOption[] {
  if (!fieldId) return [];
  return source.customFieldOptions
    .filter((option) => option.fieldId === fieldId)
    .sort(sortByOrderAndName);
}

export function customFieldOptionByName(
  source: ProjectsSnapshot,
  fieldId: string,
  name: string,
): ProjectCustomFieldOption | undefined {
  const normalized = normalizeProjectName(name).toLowerCase();
  if (!normalized) return undefined;
  return source.customFieldOptions.find((option) =>
    option.fieldId === fieldId && option.name.trim().toLowerCase() === normalized
  );
}

export function customFieldValueForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldValue | undefined {
  if (!taskId || !fieldId) return undefined;
  return source.customFieldValues.find((value) => value.taskId === taskId && value.fieldId === fieldId);
}

export function customFieldOptionValuesForTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldOption[] {
  if (!taskId || !fieldId) return [];
  const optionIds = new Set(
    source.customFieldOptionValues
      .filter((value) => value.taskId === taskId && value.fieldId === fieldId)
      .map((value) => value.optionId),
  );
  return customFieldOptionsForField(source, fieldId).filter((option) => optionIds.has(option.id));
}

export function dependenciesBlockingTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTaskDependency[] {
  if (!taskId) return [];
  return source.dependencies.filter((dependency) => dependency.blockedTaskId === taskId);
}

export function dependenciesBlockedByTask(
  source: ProjectsSnapshot,
  taskId: string | null | undefined,
): ProjectTaskDependency[] {
  if (!taskId) return [];
  return source.dependencies.filter((dependency) => dependency.blockingTaskId === taskId);
}

export function savedTaskViewsForProject(
  source: ProjectsSnapshot,
  projectId: string | null | undefined,
): ProjectSavedTaskView[] {
  if (!projectId) return [];
  const customFields = customFieldsForProject(source, projectId);
  const customFieldIds = new Set(customFields.map((field) => field.id));
  const customFieldOptionIds = new Set(
    customFields.flatMap((field) => customFieldOptionsForField(source, field.id).map((option) => option.id)),
  );
  return source.viewPreferences
    .filter((preference) => preference.projectId === projectId)
    .map((preference) => parseSavedTaskViewPreference(preference, customFieldIds, customFieldOptionIds))
    .filter((view): view is ProjectSavedTaskView => view !== undefined)
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt) || a.name.localeCompare(b.name));
}

export function taskClosure(source: ProjectsSnapshot, tasks: ProjectTask[]): ProjectTask[] {
  const taskMap = new Map(source.tasks.map((task) => [task.id, task]));
  const visited = new Set<string>();
  const pending = tasks.map((task) => task.id);
  const result: ProjectTask[] = [];
  while (pending.length > 0) {
    const taskId = pending.pop();
    if (!taskId || visited.has(taskId)) continue;
    visited.add(taskId);
    const task = taskMap.get(taskId);
    if (!task) continue;
    result.push(task);
    for (const child of source.tasks) {
      if (child.parentTaskId === task.id) pending.push(child.id);
    }
  }
  return result;
}

export function nextGroupSortOrder(source: ProjectsSnapshot): number {
  return Math.max(0, ...source.groups.map((group) => group.sortOrder)) + 1000;
}

export function nextProjectSortOrder(source: ProjectsSnapshot, groupId: string): number {
  return Math.max(0, ...projectsForGroupIncludingInactive(source, groupId).map((project) => project.sortOrder)) + 1000;
}

export function nextSectionSortOrder(source: ProjectsSnapshot, projectId: string): number {
  return Math.max(0, ...sectionsForProject(source, projectId).map((section) => section.sortOrder)) + 1000;
}

export function nextStatusSortOrder(source: ProjectsSnapshot, projectId: string): number {
  return Math.max(0, ...statusesForProject(source, projectId).map((status) => status.sortOrder)) + 1000;
}

export function nextPrioritySortOrder(source: ProjectsSnapshot, projectId: string): number {
  return Math.max(0, ...prioritiesForProject(source, projectId).map((priority) => priority.sortOrder)) + 1000;
}

export function nextTaskSectionSortOrder(
  source: ProjectsSnapshot,
  projectId: string,
  sectionId: string,
): number {
  return Math.max(
    0,
    ...source.tasks
      .filter((task) =>
        task.projectId === projectId
        && task.sectionId === sectionId
        && !task.parentTaskId
        && !task.archivedAt
      )
      .map((task) => task.sectionSortOrder),
  ) + 1000;
}

export function nextSubtaskSortOrder(source: ProjectsSnapshot, parentTaskId: string): number {
  return Math.max(
    0,
    ...subtasksForTaskIncludingArchived(source, parentTaskId).map((task) => task.sectionSortOrder),
  ) + 1000;
}

export function nextChecklistSortOrder(source: ProjectsSnapshot, taskId: string): number {
  return Math.max(0, ...checklistItemsForTask(source, taskId).map((item) => item.sortOrder)) + 1000;
}

export function nextTagSortOrder(source: ProjectsSnapshot, projectId: string): number {
  return Math.max(0, ...tagsForProject(source, projectId).map((tag) => tag.sortOrder)) + 1000;
}

export function nextCustomFieldSortOrder(source: ProjectsSnapshot, projectId: string): number {
  return Math.max(0, ...customFieldsForProject(source, projectId).map((field) => field.sortOrder)) + 1000;
}

export function nextCustomFieldOptionSortOrder(source: ProjectsSnapshot, fieldId: string): number {
  return Math.max(0, ...customFieldOptionsForField(source, fieldId).map((option) => option.sortOrder)) + 1000;
}

export function nextTaskStatusSortOrder(
  source: ProjectsSnapshot,
  projectId: string,
  statusId: string,
): number {
  return Math.max(
    0,
    ...source.tasks
      .filter((task) =>
        task.projectId === projectId
        && task.statusId === statusId
        && !task.parentTaskId
        && !task.archivedAt
      )
      .map((task) => task.statusSortOrder),
  ) + 1000;
}

function taskIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.tasks.filter((task) => task.projectId === projectId).map((task) => task.id));
}

function fieldIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.customFields.filter((field) => field.projectId === projectId).map((field) => field.id));
}

function tagIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.tags.filter((tag) => tag.projectId === projectId).map((tag) => tag.id));
}

function snapshotWithoutProjectData(source: ProjectsSnapshot, projectId: string): ProjectsSnapshot {
  const taskIds = taskIdsForProject(source, projectId);
  const fieldIds = fieldIdsForProject(source, projectId);
  const tagIds = tagIdsForProject(source, projectId);
  return {
    ...source,
    sections: source.sections.filter((section) => section.projectId !== projectId),
    statuses: source.statuses.filter((status) => status.projectId !== projectId),
    priorities: source.priorities.filter((priority) => priority.projectId !== projectId),
    tasks: source.tasks.filter((task) => task.projectId !== projectId),
    checklistItems: source.checklistItems.filter((item) => !taskIds.has(item.taskId)),
    tags: source.tags.filter((tag) => tag.projectId !== projectId),
    taskTagLinks: source.taskTagLinks.filter((link) => !taskIds.has(link.taskId) && !tagIds.has(link.tagId)),
    customFields: source.customFields.filter((field) => field.projectId !== projectId),
    customFieldOptions: source.customFieldOptions.filter((option) => !fieldIds.has(option.fieldId)),
    customFieldValues: source.customFieldValues.filter((value) =>
      !taskIds.has(value.taskId) && !fieldIds.has(value.fieldId)
    ),
    customFieldOptionValues: source.customFieldOptionValues.filter((value) =>
      !taskIds.has(value.taskId) && !fieldIds.has(value.fieldId)
    ),
    dependencies: source.dependencies.filter((dependency) =>
      !taskIds.has(dependency.blockingTaskId) && !taskIds.has(dependency.blockedTaskId)
    ),
    eventLinks: source.eventLinks.filter((link) => !taskIds.has(link.taskId)),
    taskChangeEvents: source.taskChangeEvents.filter((event) => !taskIds.has(event.taskId)),
  };
}

export function mergeProjectSnapshot(
  current: ProjectsSnapshot,
  incoming: ProjectsSnapshot,
  projectId: string,
): ProjectsSnapshot {
  const base = snapshotWithoutProjectData(current, projectId);
  return {
    ...base,
    groups: incoming.groups,
    projects: incoming.projects,
    viewPreferences: incoming.viewPreferences,
    customEmojis: incoming.customEmojis,
    sections: [...base.sections, ...incoming.sections],
    statuses: [...base.statuses, ...incoming.statuses],
    priorities: [...base.priorities, ...incoming.priorities],
    tasks: [...base.tasks, ...incoming.tasks],
    checklistItems: [...base.checklistItems, ...incoming.checklistItems],
    tags: [...base.tags, ...incoming.tags],
    taskTagLinks: [...base.taskTagLinks, ...incoming.taskTagLinks],
    customFields: [...base.customFields, ...incoming.customFields],
    customFieldOptions: [...base.customFieldOptions, ...incoming.customFieldOptions],
    customFieldValues: [...base.customFieldValues, ...incoming.customFieldValues],
    customFieldOptionValues: [...base.customFieldOptionValues, ...incoming.customFieldOptionValues],
    dependencies: [...base.dependencies, ...incoming.dependencies],
    eventLinks: [...base.eventLinks, ...incoming.eventLinks],
    taskChangeEvents: [...base.taskChangeEvents, ...incoming.taskChangeEvents],
  };
}
