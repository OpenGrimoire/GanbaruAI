import {
  createProjectChecklistItem,
  createProjectCustomField,
  createProjectCustomFieldOption,
  createProjectLabel,
  createProject as createProjectBackend,
  createProjectGroup,
  createProjectSection,
  createProjectStatus,
  createProjectTaskDependency,
  createProjectTask,
  deleteProjectCustomField,
  deleteProjectCustomFieldOption,
  deleteProjectChecklistItem,
  deleteProjectLabel,
  deleteProjectViewPreference,
  deleteProjectTaskDependency,
  loadProjectsSnapshot,
  linkProjectTaskLabel,
  linkProjectTaskEvent,
  searchProjectLinkableEvents,
  setProjectGroupCollapsed,
  unlinkProjectTaskLabel,
  unlinkProjectTaskEvent,
  updateProjectChecklistItem,
  updateProjectCustomField,
  updateProjectCustomFieldOption,
  updateProjectCustomFieldValue,
  updateProjectGroup,
  updateProjectLabel,
  updateProject as updateProjectBackend,
  updateProjectSection,
  updateProjectStatus,
  updateProjectTask,
  upsertProjectViewPreference,
} from "$lib/api/projects";
import {
  parseSavedTaskViewPreference,
  savedTaskViewPreferenceKey,
  savedTaskViewPreferenceValue,
} from "$lib/projects/saved-task-views";
import {
  TASK_LIST_COLUMNS_PREFERENCE_KEY,
  taskListColumnsPreferenceValue,
} from "$lib/projects/task-list-columns";
import { PROJECT_TEMPLATE_DEFAULTS } from "$lib/projects/types";
import {
  loadSavedActiveProjectId,
  loadSavedProjectViewId,
  saveActiveProjectId,
  saveProjectViewId,
} from "$lib/projects/project-ui-preferences";
import type {
  Project,
  ProjectChecklistItem,
  ProjectChecklistItemUpdate,
  ProjectCreate,
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldOptionValue,
  ProjectCustomFieldOptionUpdate,
  ProjectCustomFieldType,
  ProjectCustomFieldUpdate,
  ProjectCustomFieldValue,
  ProjectCustomFieldValueUpdate,
  ProjectGroup,
  ProjectGroupUpdate,
  ProjectLabel,
  ProjectLabelUpdate,
  ProjectLinkableEvent,
  ProjectPriority,
  ProjectSavedTaskView,
  ProjectsSnapshot,
  ProjectSection,
  ProjectSectionUpdate,
  ProjectStatus,
  ProjectStatusCategory,
  ProjectTemplateId,
  ProjectUpdate,
  ProjectTask,
  ProjectTaskChangeEvent,
  ProjectTaskDependency,
  ProjectTaskEventLink,
  ProjectTaskLabelLink,
  ProjectTaskListColumn,
  ProjectTaskType,
  ProjectTaskUpdate,
  ProjectViewPreference,
  ProjectViewId,
} from "$lib/projects/types";

let snapshot = $state<ProjectsSnapshot>({
  groups: [],
  projects: [],
  sections: [],
  statuses: [],
  tasks: [],
  checklistItems: [],
  labels: [],
  taskLabelLinks: [],
  customFields: [],
  customFieldOptions: [],
  customFieldValues: [],
  customFieldOptionValues: [],
  dependencies: [],
  eventLinks: [],
  taskChangeEvents: [],
  viewPreferences: [],
});
let loaded = $state(false);
let loading = $state(false);
let loadError = $state<string | null>(null);
let selectedProjectId = $state<string | null>(loadSavedActiveProjectId());
let activeView = $state<ProjectViewId>(loadSavedProjectViewId());
let loadedProjectIds = $state<string[]>([]);
let loadRequestId = 0;

function activeProjects(): Project[] {
  return snapshot.projects
    .filter((project) => project.status === "active")
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function firstProjectId(): string | null {
  return activeProjects()[0]?.id ?? snapshot.projects[0]?.id ?? null;
}

function ensureSelectedProject(): void {
  if (snapshot.projects.length === 0) return;
  if (selectedProjectId && snapshot.projects.some((project) => project.id === selectedProjectId)) {
    return;
  }
  setSelectedProjectId(firstProjectId());
}

function setSelectedProjectId(projectId: string | null): void {
  if (selectedProjectId === projectId) return;
  selectedProjectId = projectId;
  saveActiveProjectId(projectId);
}

function projectDataLoaded(projectId: string | null | undefined): boolean {
  return Boolean(projectId && loadedProjectIds.includes(projectId));
}

function markProjectDataLoaded(projectId: string): void {
  if (loadedProjectIds.includes(projectId)) return;
  loadedProjectIds = [...loadedProjectIds, projectId];
}

function setActiveView(view: ProjectViewId): void {
  if (activeView === view) return;
  activeView = view;
  saveProjectViewId(view);
}

function projectById(projectId: string | null | undefined): Project | undefined {
  if (!projectId) return undefined;
  return snapshot.projects.find((project) => project.id === projectId);
}

function groupById(groupId: string | null | undefined): ProjectGroup | undefined {
  if (!groupId) return undefined;
  return snapshot.groups.find((group) => group.id === groupId);
}

function sectionsForProject(projectId: string | null | undefined): ProjectSection[] {
  if (!projectId) return [];
  return snapshot.sections
    .filter((section) => section.projectId === projectId && !section.archivedAt && !section.hiddenAt)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function sectionsForProjectIncludingInactive(projectId: string | null | undefined): ProjectSection[] {
  if (!projectId) return [];
  return snapshot.sections
    .filter((section) => section.projectId === projectId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function statusesForProject(projectId: string | null | undefined): ProjectStatus[] {
  if (!projectId) return [];
  return snapshot.statuses
    .filter((status) => status.projectId === projectId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function tasksForProject(projectId: string | null | undefined): ProjectTask[] {
  if (!projectId) return [];
  return snapshot.tasks
    .filter((task) => task.projectId === projectId && !task.archivedAt)
    .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function tasksForProjectIncludingArchived(projectId: string | null | undefined): ProjectTask[] {
  if (!projectId) return [];
  return snapshot.tasks
    .filter((task) => task.projectId === projectId)
    .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function topLevelTasksForSection(projectId: string, sectionId: string): ProjectTask[] {
  return snapshot.tasks
    .filter((task) =>
      task.projectId === projectId
      && task.sectionId === sectionId
      && !task.parentTaskId
      && !task.archivedAt
    )
    .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function topLevelTasksForStatus(projectId: string, statusId: string): ProjectTask[] {
  return snapshot.tasks
    .filter((task) =>
      task.projectId === projectId
      && task.statusId === statusId
      && !task.parentTaskId
      && !task.archivedAt
    )
    .sort((a, b) => a.statusSortOrder - b.statusSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function subtasksForTask(parentTaskId: string | null | undefined): ProjectTask[] {
  if (!parentTaskId) return [];
  return snapshot.tasks
    .filter((task) => task.parentTaskId === parentTaskId && !task.archivedAt)
    .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function subtasksForTaskIncludingArchived(parentTaskId: string | null | undefined): ProjectTask[] {
  if (!parentTaskId) return [];
  return snapshot.tasks
    .filter((task) => task.parentTaskId === parentTaskId)
    .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.createdAt.localeCompare(b.createdAt));
}

function projectsForGroup(groupId: string): Project[] {
  return snapshot.projects
    .filter((project) => project.groupId === groupId && project.status === "active")
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function projectsForGroupIncludingInactive(groupId: string): Project[] {
  return snapshot.projects
    .filter((project) => project.groupId === groupId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function visibleGroups(): ProjectGroup[] {
  return snapshot.groups
    .filter((group) => !group.hiddenAt && !group.archivedAt)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function defaultSection(projectId: string): ProjectSection | undefined {
  return sectionsForProject(projectId)[0];
}

function defaultStatus(projectId: string): ProjectStatus | undefined {
  const statuses = statusesForProject(projectId);
  return statuses.find((status) => status.name.toLowerCase() === "to do")
    ?? statuses.find((status) => status.category === "not_started")
    ?? statuses[0];
}

function doneStatus(projectId: string): ProjectStatus | undefined {
  return statusesForProject(projectId).find((status) => status.terminal);
}

function reopenStatus(projectId: string): ProjectStatus | undefined {
  const statuses = statusesForProject(projectId);
  return statuses.find((status) => status.name.toLowerCase() === "to do")
    ?? statuses.find((status) => !status.terminal)
    ?? statuses[0];
}

function statusById(statusId: string): ProjectStatus | undefined {
  return snapshot.statuses.find((status) => status.id === statusId);
}

function taskById(taskId: string | null | undefined): ProjectTask | undefined {
  if (!taskId) return undefined;
  return snapshot.tasks.find((task) => task.id === taskId);
}

function eventLinksForTask(taskId: string | null | undefined): ProjectTaskEventLink[] {
  if (!taskId) return [];
  return snapshot.eventLinks.filter((link) => link.taskId === taskId);
}

function eventLinksForEvent(eventId: string | null | undefined): ProjectTaskEventLink[] {
  if (!eventId) return [];
  return snapshot.eventLinks.filter((link) => link.eventId === eventId);
}

function taskChangeEventsForTask(taskId: string | null | undefined): ProjectTaskChangeEvent[] {
  if (!taskId) return [];
  return snapshot.taskChangeEvents
    .filter((event) => event.taskId === taskId)
    .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt));
}

function recentTaskChangeEventsForProject(
  projectId: string | null | undefined,
  limit = 8,
): ProjectTaskChangeEvent[] {
  if (!projectId) return [];
  const projectTaskIds = new Set(
    snapshot.tasks
      .filter((task) => task.projectId === projectId)
      .map((task) => task.id),
  );
  return snapshot.taskChangeEvents
    .filter((event) => projectTaskIds.has(event.taskId))
    .sort((a, b) => b.occurredAt.localeCompare(a.occurredAt))
    .slice(0, limit);
}

function checklistItemsForTask(taskId: string | null | undefined): ProjectChecklistItem[] {
  if (!taskId) return [];
  return snapshot.checklistItems
    .filter((item) => item.taskId === taskId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.createdAt.localeCompare(b.createdAt));
}

function labelsForProject(projectId: string | null | undefined): ProjectLabel[] {
  if (!projectId) return [];
  return snapshot.labels
    .filter((label) => label.projectId === projectId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function labelById(labelId: string | null | undefined): ProjectLabel | undefined {
  if (!labelId) return undefined;
  return snapshot.labels.find((label) => label.id === labelId);
}

function taskLabelLinksForTask(taskId: string | null | undefined): ProjectTaskLabelLink[] {
  if (!taskId) return [];
  return snapshot.taskLabelLinks.filter((link) => link.taskId === taskId);
}

function labelsForTask(taskId: string | null | undefined): ProjectLabel[] {
  const labelIds = new Set(taskLabelLinksForTask(taskId).map((link) => link.labelId));
  return snapshot.labels
    .filter((label) => labelIds.has(label.id))
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function unlinkedLabelsForTask(task: ProjectTask | null | undefined): ProjectLabel[] {
  if (!task) return [];
  const linkedLabelIds = new Set(taskLabelLinksForTask(task.id).map((link) => link.labelId));
  return labelsForProject(task.projectId).filter((label) => !linkedLabelIds.has(label.id));
}

function projectLabelByName(projectId: string, name: string): ProjectLabel | undefined {
  const normalized = normalizedName(name).toLowerCase();
  if (!normalized) return undefined;
  return snapshot.labels.find((label) =>
    label.projectId === projectId && label.name.trim().toLowerCase() === normalized
  );
}

function customFieldsForProject(projectId: string | null | undefined): ProjectCustomField[] {
  if (!projectId) return [];
  return snapshot.customFields
    .filter((field) => field.projectId === projectId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function customFieldById(fieldId: string | null | undefined): ProjectCustomField | undefined {
  if (!fieldId) return undefined;
  return snapshot.customFields.find((field) => field.id === fieldId);
}

function customFieldByName(projectId: string, name: string): ProjectCustomField | undefined {
  const normalized = normalizedName(name).toLowerCase();
  if (!normalized) return undefined;
  return snapshot.customFields.find((field) =>
    field.projectId === projectId && field.name.trim().toLowerCase() === normalized
  );
}

function customFieldOptionsForField(fieldId: string | null | undefined): ProjectCustomFieldOption[] {
  if (!fieldId) return [];
  return snapshot.customFieldOptions
    .filter((option) => option.fieldId === fieldId)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.name.localeCompare(b.name));
}

function customFieldOptionByName(fieldId: string, name: string): ProjectCustomFieldOption | undefined {
  const normalized = normalizedName(name).toLowerCase();
  if (!normalized) return undefined;
  return snapshot.customFieldOptions.find((option) =>
    option.fieldId === fieldId && option.name.trim().toLowerCase() === normalized
  );
}

function customFieldValueForTask(
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldValue | undefined {
  if (!taskId || !fieldId) return undefined;
  return snapshot.customFieldValues.find((value) => value.taskId === taskId && value.fieldId === fieldId);
}

function customFieldOptionValuesForTask(
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldOption[] {
  if (!taskId || !fieldId) return [];
  const optionIds = new Set(
    snapshot.customFieldOptionValues
      .filter((value) => value.taskId === taskId && value.fieldId === fieldId)
      .map((value) => value.optionId),
  );
  return customFieldOptionsForField(fieldId).filter((option) => optionIds.has(option.id));
}

function dependenciesBlockingTask(taskId: string | null | undefined): ProjectTaskDependency[] {
  if (!taskId) return [];
  return snapshot.dependencies.filter((dependency) => dependency.blockedTaskId === taskId);
}

function dependenciesBlockedByTask(taskId: string | null | undefined): ProjectTaskDependency[] {
  if (!taskId) return [];
  return snapshot.dependencies.filter((dependency) => dependency.blockingTaskId === taskId);
}

function savedTaskViewsForProject(projectId: string | null | undefined): ProjectSavedTaskView[] {
  if (!projectId) return [];
  const customFields = customFieldsForProject(projectId);
  const customFieldIds = new Set(customFields.map((field) => field.id));
  const customFieldOptionIds = new Set(
    customFields.flatMap((field) => customFieldOptionsForField(field.id).map((option) => option.id)),
  );
  return snapshot.viewPreferences
    .filter((preference) => preference.projectId === projectId)
    .map((preference) => parseSavedTaskViewPreference(preference, customFieldIds, customFieldOptionIds))
    .filter((view): view is ProjectSavedTaskView => view !== undefined)
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt) || a.name.localeCompare(b.name));
}

function taskClosure(tasks: ProjectTask[]): ProjectTask[] {
  const taskMap = new Map(snapshot.tasks.map((task) => [task.id, task]));
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
    for (const child of snapshot.tasks) {
      if (child.parentTaskId === task.id) pending.push(child.id);
    }
  }
  return result;
}

function nextGroupSortOrder(): number {
  return Math.max(0, ...snapshot.groups.map((group) => group.sortOrder)) + 1000;
}

function nextProjectSortOrder(groupId: string): number {
  return Math.max(0, ...projectsForGroupIncludingInactive(groupId).map((project) => project.sortOrder)) + 1000;
}

function nextSectionSortOrder(projectId: string): number {
  return Math.max(0, ...sectionsForProject(projectId).map((section) => section.sortOrder)) + 1000;
}

function nextStatusSortOrder(projectId: string): number {
  return Math.max(0, ...statusesForProject(projectId).map((status) => status.sortOrder)) + 1000;
}

function nextTaskSectionSortOrder(projectId: string, sectionId: string): number {
  return Math.max(
    0,
    ...snapshot.tasks
      .filter((task) =>
        task.projectId === projectId
        && task.sectionId === sectionId
        && !task.parentTaskId
        && !task.archivedAt
      )
      .map((task) => task.sectionSortOrder),
  ) + 1000;
}

function nextSubtaskSortOrder(parentTaskId: string): number {
  return Math.max(
    0,
    ...subtasksForTaskIncludingArchived(parentTaskId).map((task) => task.sectionSortOrder),
  ) + 1000;
}

function nextChecklistSortOrder(taskId: string): number {
  return Math.max(0, ...checklistItemsForTask(taskId).map((item) => item.sortOrder)) + 1000;
}

function nextLabelSortOrder(projectId: string): number {
  return Math.max(0, ...labelsForProject(projectId).map((label) => label.sortOrder)) + 1000;
}

function nextCustomFieldSortOrder(projectId: string): number {
  return Math.max(0, ...customFieldsForProject(projectId).map((field) => field.sortOrder)) + 1000;
}

function nextCustomFieldOptionSortOrder(fieldId: string): number {
  return Math.max(0, ...customFieldOptionsForField(fieldId).map((option) => option.sortOrder)) + 1000;
}

function nextTaskStatusSortOrder(projectId: string, statusId: string): number {
  return Math.max(
    0,
    ...snapshot.tasks
      .filter((task) =>
        task.projectId === projectId
        && task.statusId === statusId
        && !task.parentTaskId
        && !task.archivedAt
      )
      .map((task) => task.statusSortOrder),
  ) + 1000;
}

type ProjectTaskUpdatePatch = Partial<ProjectTask> & { changeReason?: string | null };

function taskUpdatePayload(task: ProjectTask, patch: ProjectTaskUpdatePatch): ProjectTaskUpdate {
  const merged = { ...task, ...patch };
  return {
    id: merged.id,
    sectionId: merged.sectionId,
    statusId: merged.statusId,
    parentTaskId: merged.parentTaskId ?? null,
    title: merged.title,
    description: merged.description,
    priority: merged.priority,
    taskType: merged.taskType,
    sectionSortOrder: merged.sectionSortOrder,
    statusSortOrder: merged.statusSortOrder,
    estimateMinutes: merged.estimateMinutes ?? null,
    dueDate: merged.dueDate ?? null,
    startDate: merged.startDate ?? null,
    targetEndDate: merged.targetEndDate ?? null,
    archivedAt: merged.archivedAt ?? null,
    blockerReason: merged.blockerReason ?? null,
    milestone: merged.milestone,
    changeReason: patch.changeReason ?? null,
  };
}

function checklistItemUpdatePayload(
  item: ProjectChecklistItem,
  patch: Partial<Pick<ProjectChecklistItem, "title" | "completedAt" | "sortOrder">>,
): ProjectChecklistItemUpdate {
  const merged = { ...item, ...patch };
  return {
    id: merged.id,
    title: merged.title,
    completed: Boolean(merged.completedAt),
    sortOrder: merged.sortOrder,
  };
}

function labelUpdatePayload(
  label: ProjectLabel,
  patch: Partial<Pick<ProjectLabel, "name" | "color" | "sortOrder">>,
): ProjectLabelUpdate {
  const merged = { ...label, ...patch };
  return {
    id: merged.id,
    name: merged.name,
    color: merged.color ?? null,
    sortOrder: merged.sortOrder,
  };
}

function customFieldUpdatePayload(
  field: ProjectCustomField,
  patch: Partial<Pick<ProjectCustomField, "name" | "sortOrder">>,
): ProjectCustomFieldUpdate {
  const merged = { ...field, ...patch };
  return {
    id: merged.id,
    name: merged.name,
    sortOrder: merged.sortOrder,
  };
}

function customFieldOptionUpdatePayload(
  option: ProjectCustomFieldOption,
  patch: Partial<Pick<ProjectCustomFieldOption, "name" | "sortOrder">>,
): ProjectCustomFieldOptionUpdate {
  const merged = { ...option, ...patch };
  return {
    id: merged.id,
    name: merged.name,
    sortOrder: merged.sortOrder,
  };
}

function groupUpdatePayload(
  group: ProjectGroup,
  patch: Partial<Pick<ProjectGroup, "name" | "icon" | "color" | "sortOrder" | "collapsed">>,
): ProjectGroupUpdate {
  const merged = { ...group, ...patch };
  return {
    id: merged.id,
    name: merged.name,
    icon: merged.icon,
    color: merged.color ?? null,
    sortOrder: merged.sortOrder,
    collapsed: merged.collapsed,
  };
}

function projectUpdatePayload(project: Project, patch: Partial<Project>): ProjectUpdate {
  const merged = { ...project, ...patch };
  return {
    id: merged.id,
    groupId: merged.groupId,
    name: merged.name,
    icon: merged.icon,
    color: merged.color ?? null,
    sortOrder: merged.sortOrder,
    status: merged.status,
    defaultEventDurationMinutes: merged.defaultEventDurationMinutes,
    defaultPomodoroPresetKey: merged.defaultPomodoroPresetKey ?? null,
    defaultIdleTimeoutMinutes: merged.defaultIdleTimeoutMinutes ?? null,
    focusPlaylistId: merged.focusPlaylistId ?? null,
    breakPlaylistId: merged.breakPlaylistId ?? null,
    workEnvironmentId: merged.workEnvironmentId ?? null,
    blockerRulesetId: merged.blockerRulesetId ?? null,
  };
}

function normalizedName(value: string): string {
  return value.trim();
}

function taskIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.tasks.filter((task) => task.projectId === projectId).map((task) => task.id));
}

function fieldIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.customFields.filter((field) => field.projectId === projectId).map((field) => field.id));
}

function labelIdsForProject(source: ProjectsSnapshot, projectId: string): Set<string> {
  return new Set(source.labels.filter((label) => label.projectId === projectId).map((label) => label.id));
}

function snapshotWithoutProjectData(source: ProjectsSnapshot, projectId: string): ProjectsSnapshot {
  const taskIds = taskIdsForProject(source, projectId);
  const fieldIds = fieldIdsForProject(source, projectId);
  const labelIds = labelIdsForProject(source, projectId);
  return {
    ...source,
    sections: source.sections.filter((section) => section.projectId !== projectId),
    statuses: source.statuses.filter((status) => status.projectId !== projectId),
    tasks: source.tasks.filter((task) => task.projectId !== projectId),
    checklistItems: source.checklistItems.filter((item) => !taskIds.has(item.taskId)),
    labels: source.labels.filter((label) => label.projectId !== projectId),
    taskLabelLinks: source.taskLabelLinks.filter((link) => !taskIds.has(link.taskId) && !labelIds.has(link.labelId)),
    customFields: source.customFields.filter((field) => field.projectId !== projectId),
    customFieldOptions: source.customFieldOptions.filter((option) => !fieldIds.has(option.fieldId)),
    customFieldValues: source.customFieldValues.filter((value) => !taskIds.has(value.taskId) && !fieldIds.has(value.fieldId)),
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

function mergeProjectSnapshot(current: ProjectsSnapshot, incoming: ProjectsSnapshot, projectId: string): ProjectsSnapshot {
  const base = snapshotWithoutProjectData(current, projectId);
  return {
    ...base,
    groups: incoming.groups,
    projects: incoming.projects,
    viewPreferences: incoming.viewPreferences,
    sections: [...base.sections, ...incoming.sections],
    statuses: [...base.statuses, ...incoming.statuses],
    tasks: [...base.tasks, ...incoming.tasks],
    checklistItems: [...base.checklistItems, ...incoming.checklistItems],
    labels: [...base.labels, ...incoming.labels],
    taskLabelLinks: [...base.taskLabelLinks, ...incoming.taskLabelLinks],
    customFields: [...base.customFields, ...incoming.customFields],
    customFieldOptions: [...base.customFieldOptions, ...incoming.customFieldOptions],
    customFieldValues: [...base.customFieldValues, ...incoming.customFieldValues],
    customFieldOptionValues: [...base.customFieldOptionValues, ...incoming.customFieldOptionValues],
    dependencies: [...base.dependencies, ...incoming.dependencies],
    eventLinks: [...base.eventLinks, ...incoming.eventLinks],
    taskChangeEvents: [...base.taskChangeEvents, ...incoming.taskChangeEvents],
  };
}

function applyLoadedSnapshot(incoming: ProjectsSnapshot, projectId: string | null | undefined): void {
  if (!projectId) {
    snapshot = {
      ...snapshot,
      groups: incoming.groups,
      projects: incoming.projects,
      viewPreferences: incoming.viewPreferences,
    };
    return;
  }
  snapshot = mergeProjectSnapshot(snapshot, incoming, projectId);
  if (incoming.projects.some((project) => project.id === projectId)) {
    markProjectDataLoaded(projectId);
  }
}

async function reload(projectId: string | null = selectedProjectId): Promise<void> {
  const requestId = ++loadRequestId;
  loading = true;
  loadError = null;
  try {
    const incoming = await loadProjectsSnapshot(projectId);
    if (requestId !== loadRequestId) return;
    applyLoadedSnapshot(incoming, projectId);
    loaded = true;
    const requestedProjectId = projectId;
    ensureSelectedProject();
    if (selectedProjectId && selectedProjectId !== requestedProjectId && !projectDataLoaded(selectedProjectId)) {
      const selectedSnapshot = await loadProjectsSnapshot(selectedProjectId);
      if (requestId !== loadRequestId) return;
      applyLoadedSnapshot(selectedSnapshot, selectedProjectId);
    }
  } catch (error) {
    if (requestId !== loadRequestId) return;
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === loadRequestId) loading = false;
  }
}

async function ensureLoaded(): Promise<void> {
  if (loaded || loading) return;
  await reload();
}

async function ensureProjectData(projectId: string | null | undefined): Promise<void> {
  if (!projectId) return;
  if (!loaded) await ensureLoaded();
  if (projectDataLoaded(projectId)) return;
  await reload(projectId);
}

async function selectProject(projectId: string | null): Promise<void> {
  setSelectedProjectId(projectId);
  await ensureProjectData(projectId);
}

async function addGroup(name: string): Promise<void> {
  const displayName = normalizedName(name);
  if (!displayName) return;
  await createProjectGroup({
    id: crypto.randomUUID(),
    name: displayName,
    icon: "folder",
    color: null,
    sortOrder: nextGroupSortOrder(),
  });
  await reload();
}

async function setGroupCollapsed(groupId: string, collapsed: boolean): Promise<void> {
  await setProjectGroupCollapsed(groupId, collapsed);
  snapshot = {
    ...snapshot,
    groups: snapshot.groups.map((group) =>
      group.id === groupId ? { ...group, collapsed } : group
    ),
  };
}

async function updateGroup(
  group: ProjectGroup,
  patch: Partial<Pick<ProjectGroup, "name" | "icon" | "color" | "sortOrder" | "collapsed">>,
): Promise<void> {
  const nextName = normalizedName(patch.name ?? group.name);
  if (!nextName) return;
  await updateProjectGroup(groupUpdatePayload(group, { ...patch, name: nextName }));
  await reload();
}

async function moveGroup(group: ProjectGroup, direction: -1 | 1): Promise<void> {
  const ordered = visibleGroups();
  const index = ordered.findIndex((entry) => entry.id === group.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectGroup(groupUpdatePayload(group, { sortOrder: target.sortOrder }));
  await updateProjectGroup(groupUpdatePayload(target, { sortOrder: group.sortOrder }));
  await reload();
}

async function addProject(groupId: string, name: string, templateId: ProjectTemplateId = "blank"): Promise<void> {
  const displayName = normalizedName(name);
  if (!displayName) return;
  const templateDefaults = PROJECT_TEMPLATE_DEFAULTS[templateId];
  const project: ProjectCreate = {
    id: crypto.randomUUID(),
    groupId,
    templateId,
    name: displayName,
    icon: templateDefaults.icon,
    color: templateDefaults.color,
    sortOrder: nextProjectSortOrder(groupId),
    defaultEventDurationMinutes: templateDefaults.defaultEventDurationMinutes,
    defaultPomodoroPresetKey: templateDefaults.defaultPomodoroPresetKey,
    defaultIdleTimeoutMinutes: templateDefaults.defaultIdleTimeoutMinutes,
  };
  await createProjectBackend(project);
  setSelectedProjectId(project.id);
  await reload();
}

async function updateProject(project: ProjectUpdate): Promise<void> {
  await updateProjectBackend(project);
  await reload();
}

async function moveProject(project: Project, direction: -1 | 1, includeInactive = false): Promise<void> {
  const ordered = includeInactive ? projectsForGroupIncludingInactive(project.groupId) : projectsForGroup(project.groupId);
  const index = ordered.findIndex((entry) => entry.id === project.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectBackend(projectUpdatePayload(project, { sortOrder: target.sortOrder }));
  await updateProjectBackend(projectUpdatePayload(target, { sortOrder: project.sortOrder }));
  await reload();
}

async function addSection(projectId: string, name: string): Promise<void> {
  const displayName = normalizedName(name);
  if (!displayName) return;
  await createProjectSection({
    id: crypto.randomUUID(),
    projectId,
    name: displayName,
    sortOrder: nextSectionSortOrder(projectId),
  });
  await reload();
}

function sectionUpdatePayload(
  section: ProjectSection,
  patch: Partial<Pick<ProjectSection, "name" | "sortOrder" | "collapsed" | "hiddenAt" | "archivedAt">>,
): ProjectSectionUpdate {
  const merged = { ...section, ...patch };
  return {
    id: merged.id,
    name: merged.name,
    sortOrder: merged.sortOrder,
    collapsed: merged.collapsed,
    hiddenAt: merged.hiddenAt ?? null,
    archivedAt: merged.archivedAt ?? null,
  };
}

async function updateSection(
  section: ProjectSection,
  patch: Partial<Pick<ProjectSection, "name" | "sortOrder" | "collapsed" | "hiddenAt" | "archivedAt">>,
): Promise<void> {
  const nextName = normalizedName(patch.name ?? section.name);
  if (!nextName) return;
  await updateProjectSection(sectionUpdatePayload(section, { ...patch, name: nextName }));
  await reload();
}

async function hideSection(section: ProjectSection): Promise<void> {
  await updateSection(section, {
    hiddenAt: new Date().toISOString(),
    archivedAt: undefined,
  });
}

async function archiveSection(section: ProjectSection): Promise<void> {
  await updateSection(section, {
    hiddenAt: undefined,
    archivedAt: new Date().toISOString(),
  });
}

async function restoreSection(section: ProjectSection): Promise<void> {
  await updateSection(section, {
    hiddenAt: undefined,
    archivedAt: undefined,
  });
}

async function applySectionCollapseState(projectId: string, collapsedSectionIds: readonly string[]): Promise<void> {
  const collapsedIds = new Set(collapsedSectionIds);
  const updates = sectionsForProject(projectId)
    .filter((section) => section.collapsed !== collapsedIds.has(section.id));
  for (const section of updates) {
    await updateProjectSection(sectionUpdatePayload(section, {
      collapsed: collapsedIds.has(section.id),
    }));
  }
  if (updates.length > 0) await reload();
}

async function moveSection(section: ProjectSection, direction: -1 | 1): Promise<void> {
  const ordered = sectionsForProject(section.projectId);
  const index = ordered.findIndex((entry) => entry.id === section.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectSection(sectionUpdatePayload(section, { sortOrder: target.sortOrder }));
  await updateProjectSection(sectionUpdatePayload(target, { sortOrder: section.sortOrder }));
  await reload();
}

async function addStatus(
  projectId: string,
  name: string,
  category: ProjectStatusCategory,
): Promise<void> {
  const displayName = normalizedName(name);
  if (!displayName) return;
  await createProjectStatus({
    id: crypto.randomUUID(),
    projectId,
    name: displayName,
    category,
    sortOrder: nextStatusSortOrder(projectId),
    terminal: category === "done",
  });
  await reload();
}

async function updateStatus(
  status: ProjectStatus,
  patch: Partial<Pick<ProjectStatus, "name" | "category" | "sortOrder">>,
): Promise<void> {
  const displayName = normalizedName(patch.name ?? status.name);
  if (!displayName) return;
  const category = patch.category ?? status.category;
  await updateProjectStatus({
    id: status.id,
    name: displayName,
    category,
    sortOrder: patch.sortOrder ?? status.sortOrder,
    terminal: category === "done",
  });
  await reload();
}

async function moveStatus(status: ProjectStatus, direction: -1 | 1): Promise<void> {
  const ordered = statusesForProject(status.projectId);
  const index = ordered.findIndex((entry) => entry.id === status.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectStatus({
    id: status.id,
    name: status.name,
    category: status.category,
    sortOrder: target.sortOrder,
    terminal: status.category === "done",
  });
  await updateProjectStatus({
    id: target.id,
    name: target.name,
    category: target.category,
    sortOrder: status.sortOrder,
    terminal: target.category === "done",
  });
  await reload();
}

async function addTask(
  projectId: string,
  title: string,
  sectionId = defaultSection(projectId)?.id,
  statusId = defaultStatus(projectId)?.id,
  parentTaskId: string | null = null,
): Promise<void> {
  const displayTitle = normalizedName(title);
  if (!displayTitle || !sectionId || !statusId) return;
  await createProjectTask({
    id: crypto.randomUUID(),
    projectId,
    sectionId,
    statusId,
    parentTaskId,
    title: displayTitle,
  });
  await reload();
}

async function addChecklistItem(taskId: string, title: string): Promise<void> {
  const displayTitle = normalizedName(title);
  if (!displayTitle) return;
  await createProjectChecklistItem({
    id: crypto.randomUUID(),
    taskId,
    title: displayTitle,
    sortOrder: nextChecklistSortOrder(taskId),
  });
  await reload();
}

async function setChecklistItemCompleted(
  item: ProjectChecklistItem,
  completed: boolean,
): Promise<void> {
  await updateProjectChecklistItem(checklistItemUpdatePayload(item, {
    completedAt: completed ? new Date().toISOString() : undefined,
  }));
  await reload();
}

async function updateChecklistItem(
  item: ProjectChecklistItem,
  patch: Partial<Pick<ProjectChecklistItem, "title" | "sortOrder">>,
): Promise<void> {
  const nextTitle = normalizedName(patch.title ?? item.title);
  if (!nextTitle) return;
  await updateProjectChecklistItem(checklistItemUpdatePayload(item, { ...patch, title: nextTitle }));
  await reload();
}

async function moveChecklistItem(item: ProjectChecklistItem, direction: -1 | 1): Promise<void> {
  const ordered = checklistItemsForTask(item.taskId);
  const index = ordered.findIndex((entry) => entry.id === item.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectChecklistItem(checklistItemUpdatePayload(item, { sortOrder: target.sortOrder }));
  await updateProjectChecklistItem(checklistItemUpdatePayload(target, { sortOrder: item.sortOrder }));
  await reload();
}

async function removeChecklistItem(itemId: string): Promise<void> {
  await deleteProjectChecklistItem(itemId);
  await reload();
}

async function addLabel(
  projectId: string,
  name: string,
  color: ProjectLabel["color"] | null = null,
): Promise<ProjectLabel | undefined> {
  const displayName = normalizedName(name);
  if (!displayName) return undefined;
  const existingLabel = projectLabelByName(projectId, displayName);
  if (existingLabel) return existingLabel;
  const labelId = crypto.randomUUID();
  await createProjectLabel({
    id: labelId,
    projectId,
    name: displayName,
    color: color ?? null,
    sortOrder: nextLabelSortOrder(projectId),
  });
  await reload();
  return labelById(labelId);
}

async function updateLabel(
  label: ProjectLabel,
  patch: Partial<Pick<ProjectLabel, "name" | "color" | "sortOrder">>,
): Promise<void> {
  const nextName = normalizedName(patch.name ?? label.name);
  if (!nextName) return;
  await updateProjectLabel(labelUpdatePayload(label, { ...patch, name: nextName }));
  await reload();
}

async function moveLabel(label: ProjectLabel, direction: -1 | 1): Promise<void> {
  const ordered = labelsForProject(label.projectId);
  const index = ordered.findIndex((entry) => entry.id === label.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectLabel(labelUpdatePayload(label, { sortOrder: target.sortOrder }));
  await updateProjectLabel(labelUpdatePayload(target, { sortOrder: label.sortOrder }));
  await reload();
}

async function removeLabel(labelId: string): Promise<void> {
  await deleteProjectLabel(labelId);
  await reload();
}

async function linkTaskLabel(taskId: string, labelId: string): Promise<void> {
  await linkProjectTaskLabel({ taskId, labelId });
  await reload();
}

async function unlinkTaskLabel(taskId: string, labelId: string): Promise<void> {
  await unlinkProjectTaskLabel(taskId, labelId);
  await reload();
}

async function addAndLinkTaskLabel(task: ProjectTask, name: string): Promise<void> {
  const label = await addLabel(task.projectId, name);
  if (!label) return;
  await linkTaskLabel(task.id, label.id);
}

async function addCustomField(
  projectId: string,
  name: string,
  fieldType: ProjectCustomFieldType,
): Promise<ProjectCustomField | undefined> {
  const displayName = normalizedName(name);
  if (!displayName) return undefined;
  const existingField = customFieldByName(projectId, displayName);
  if (existingField) return existingField;
  const fieldId = crypto.randomUUID();
  await createProjectCustomField({
    id: fieldId,
    projectId,
    name: displayName,
    fieldType,
    sortOrder: nextCustomFieldSortOrder(projectId),
  });
  await reload();
  return customFieldById(fieldId);
}

async function updateCustomField(
  field: ProjectCustomField,
  patch: Partial<Pick<ProjectCustomField, "name" | "sortOrder">>,
): Promise<void> {
  const nextName = normalizedName(patch.name ?? field.name);
  if (!nextName) return;
  await updateProjectCustomField(customFieldUpdatePayload(field, { ...patch, name: nextName }));
  await reload();
}

async function moveCustomField(field: ProjectCustomField, direction: -1 | 1): Promise<void> {
  const ordered = customFieldsForProject(field.projectId);
  const index = ordered.findIndex((entry) => entry.id === field.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectCustomField(customFieldUpdatePayload(field, { sortOrder: target.sortOrder }));
  await updateProjectCustomField(customFieldUpdatePayload(target, { sortOrder: field.sortOrder }));
  await reload();
}

async function removeCustomField(fieldId: string): Promise<void> {
  await deleteProjectCustomField(fieldId);
  await reload();
}

async function addCustomFieldOption(
  fieldId: string,
  name: string,
): Promise<ProjectCustomFieldOption | undefined> {
  const displayName = normalizedName(name);
  if (!displayName) return undefined;
  const existingOption = customFieldOptionByName(fieldId, displayName);
  if (existingOption) return existingOption;
  const optionId = crypto.randomUUID();
  await createProjectCustomFieldOption({
    id: optionId,
    fieldId,
    name: displayName,
    sortOrder: nextCustomFieldOptionSortOrder(fieldId),
  });
  await reload();
  return snapshot.customFieldOptions.find((option) => option.id === optionId);
}

async function updateCustomFieldOption(
  option: ProjectCustomFieldOption,
  patch: Partial<Pick<ProjectCustomFieldOption, "name" | "sortOrder">>,
): Promise<void> {
  const nextName = normalizedName(patch.name ?? option.name);
  if (!nextName) return;
  await updateProjectCustomFieldOption(customFieldOptionUpdatePayload(option, { ...patch, name: nextName }));
  await reload();
}

async function moveCustomFieldOption(option: ProjectCustomFieldOption, direction: -1 | 1): Promise<void> {
  const ordered = customFieldOptionsForField(option.fieldId);
  const index = ordered.findIndex((entry) => entry.id === option.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectCustomFieldOption(customFieldOptionUpdatePayload(option, { sortOrder: target.sortOrder }));
  await updateProjectCustomFieldOption(customFieldOptionUpdatePayload(target, { sortOrder: option.sortOrder }));
  await reload();
}

async function removeCustomFieldOption(optionId: string): Promise<void> {
  await deleteProjectCustomFieldOption(optionId);
  await reload();
}

async function saveCustomFieldValue(value: ProjectCustomFieldValueUpdate): Promise<void> {
  await updateProjectCustomFieldValue(value);
  await reload();
}

async function updateTask(task: ProjectTask, patch: ProjectTaskUpdatePatch): Promise<void> {
  await updateProjectTask(taskUpdatePayload(task, patch));
  await reload();
}

async function updateTasks(
  tasks: ProjectTask[],
  patchForTask: (task: ProjectTask, index: number) => ProjectTaskUpdatePatch,
): Promise<void> {
  for (const [index, task] of tasks.entries()) {
    await updateProjectTask(taskUpdatePayload(task, patchForTask(task, index)));
  }
  await reload();
}

async function setTaskStatus(task: ProjectTask, statusId: string): Promise<void> {
  await updateTask(task, { statusId, statusSortOrder: nextTaskStatusSortOrder(task.projectId, statusId) });
}

async function setTasksStatus(tasks: ProjectTask[], statusId: string): Promise<void> {
  const firstTask = tasks[0];
  if (!firstTask) return;
  const startSortOrder = nextTaskStatusSortOrder(firstTask.projectId, statusId);
  await updateTasks(tasks, (_task, index) => ({
    statusId,
    statusSortOrder: startSortOrder + index * 1000,
  }));
}

async function toggleTaskDone(task: ProjectTask): Promise<void> {
  const currentStatus = statusById(task.statusId);
  const target = currentStatus?.terminal ? reopenStatus(task.projectId) : doneStatus(task.projectId);
  if (!target) return;
  await setTaskStatus(task, target.id);
}

async function setTaskPriority(task: ProjectTask, priority: ProjectPriority): Promise<void> {
  await updateTask(task, { priority });
}

async function setTasksPriority(tasks: ProjectTask[], priority: ProjectPriority): Promise<void> {
  await updateTasks(tasks, () => ({ priority }));
}

async function archiveTasks(tasks: ProjectTask[]): Promise<void> {
  const archivedAt = new Date().toISOString();
  await updateTasks(taskClosure(tasks), () => ({ archivedAt }));
}

async function restoreTasks(tasks: ProjectTask[]): Promise<void> {
  await updateTasks(taskClosure(tasks), () => ({ archivedAt: undefined }));
}

async function setTaskType(task: ProjectTask, taskType: ProjectTaskType): Promise<void> {
  await updateTask(task, { taskType });
}

async function moveTaskInSection(task: ProjectTask, direction: -1 | 1): Promise<void> {
  if (task.parentTaskId) return;
  const ordered = topLevelTasksForSection(task.projectId, task.sectionId);
  const index = ordered.findIndex((entry) => entry.id === task.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectTask(taskUpdatePayload(task, { sectionSortOrder: target.sectionSortOrder }));
  await updateProjectTask(taskUpdatePayload(target, { sectionSortOrder: task.sectionSortOrder }));
  await reload();
}

async function moveTaskInStatus(task: ProjectTask, direction: -1 | 1): Promise<void> {
  if (task.parentTaskId) return;
  const ordered = topLevelTasksForStatus(task.projectId, task.statusId);
  const index = ordered.findIndex((entry) => entry.id === task.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectTask(taskUpdatePayload(task, { statusSortOrder: target.statusSortOrder }));
  await updateProjectTask(taskUpdatePayload(target, { statusSortOrder: task.statusSortOrder }));
  await reload();
}

async function moveSubtask(task: ProjectTask, direction: -1 | 1): Promise<void> {
  if (!task.parentTaskId) return;
  const ordered = subtasksForTask(task.parentTaskId);
  const index = ordered.findIndex((entry) => entry.id === task.id);
  const target = ordered[index + direction];
  if (index < 0 || !target) return;
  await updateProjectTask(taskUpdatePayload(task, { sectionSortOrder: target.sectionSortOrder }));
  await updateProjectTask(taskUpdatePayload(target, { sectionSortOrder: task.sectionSortOrder }));
  await reload();
}

async function promoteSubtask(task: ProjectTask): Promise<void> {
  if (!task.parentTaskId) return;
  await updateTask(task, {
    parentTaskId: undefined,
    sectionSortOrder: nextTaskSectionSortOrder(task.projectId, task.sectionId),
    statusSortOrder: nextTaskStatusSortOrder(task.projectId, task.statusId),
  });
}

async function demoteTaskToSubtask(task: ProjectTask, parentTask: ProjectTask): Promise<void> {
  if (task.parentTaskId || parentTask.parentTaskId || task.id === parentTask.id) return;
  if (task.projectId !== parentTask.projectId) return;
  if (subtasksForTaskIncludingArchived(task.id).length > 0) return;
  await updateTask(task, {
    parentTaskId: parentTask.id,
    sectionId: parentTask.sectionId,
    sectionSortOrder: nextSubtaskSortOrder(parentTask.id),
  });
}

async function linkTaskEvent(
  taskId: string,
  eventId: string,
  linkKind: ProjectTaskEventLink["linkKind"] = "scheduled",
): Promise<void> {
  await linkProjectTaskEvent({ taskId, eventId, linkKind });
  await reload();
}

async function unlinkTaskEvent(taskId: string, eventId: string): Promise<void> {
  await unlinkProjectTaskEvent(taskId, eventId);
  await reload();
}

async function setEventTaskLinks(eventId: string, taskIds: readonly string[]): Promise<void> {
  const desired = new Set(taskIds.filter((taskId) => taskId.trim().length > 0));
  const existing = eventLinksForEvent(eventId);
  const existingTaskIds = new Set(existing.map((link) => link.taskId));
  let changed = false;

  for (const link of existing) {
    if (desired.has(link.taskId)) continue;
    await unlinkProjectTaskEvent(link.taskId, eventId);
    changed = true;
  }

  for (const taskId of desired) {
    if (existingTaskIds.has(taskId)) continue;
    await linkProjectTaskEvent({ taskId, eventId, linkKind: "scheduled" });
    changed = true;
  }

  if (changed) await reload();
}

async function searchLinkableEvents(
  projectId: string,
  taskId: string,
  query: string,
  startDate?: string,
  endDate?: string,
  limit = 12,
): Promise<ProjectLinkableEvent[]> {
  return searchProjectLinkableEvents(projectId, taskId, query, startDate, endDate, limit);
}

async function addTaskDependency(blockingTaskId: string, blockedTaskId: string): Promise<void> {
  if (!blockingTaskId || !blockedTaskId || blockingTaskId === blockedTaskId) return;
  await createProjectTaskDependency({
    id: crypto.randomUUID(),
    blockingTaskId,
    blockedTaskId,
    dependencyType: "blocks",
  });
  await reload();
}

async function removeTaskDependency(dependencyId: string): Promise<void> {
  await deleteProjectTaskDependency(dependencyId);
  await reload();
}

async function saveTaskView(view: ProjectSavedTaskView): Promise<void> {
  const name = normalizedName(view.name);
  if (!name) return;
  await upsertProjectViewPreference({
    projectId: view.projectId,
    viewId: view.viewId,
    preferenceKey: savedTaskViewPreferenceKey(view.id),
    preferenceValue: savedTaskViewPreferenceValue({ ...view, name }),
  });
  await reload();
}

async function deleteTaskView(view: ProjectSavedTaskView): Promise<void> {
  await deleteProjectViewPreference(
    view.projectId,
    view.viewId,
    savedTaskViewPreferenceKey(view.id),
  );
  await reload();
}

async function saveTaskListColumns(
  projectId: string,
  columns: readonly ProjectTaskListColumn[],
): Promise<void> {
  await upsertProjectViewPreference({
    projectId,
    viewId: "list",
    preferenceKey: TASK_LIST_COLUMNS_PREFERENCE_KEY,
    preferenceValue: taskListColumnsPreferenceValue(columns),
  });
  await reload();
}

export function getProjects() {
  return {
    get snapshot(): ProjectsSnapshot {
      return snapshot;
    },
    get groups(): ProjectGroup[] {
      return snapshot.groups;
    },
    get projects(): Project[] {
      return snapshot.projects;
    },
    get sections(): ProjectSection[] {
      return snapshot.sections;
    },
    get statuses(): ProjectStatus[] {
      return snapshot.statuses;
    },
    get tasks(): ProjectTask[] {
      return snapshot.tasks;
    },
    get checklistItems(): ProjectChecklistItem[] {
      return snapshot.checklistItems;
    },
    get labels(): ProjectLabel[] {
      return snapshot.labels;
    },
    get taskLabelLinks(): ProjectTaskLabelLink[] {
      return snapshot.taskLabelLinks;
    },
    get customFields(): ProjectCustomField[] {
      return snapshot.customFields;
    },
    get customFieldOptions(): ProjectCustomFieldOption[] {
      return snapshot.customFieldOptions;
    },
    get customFieldValues(): ProjectCustomFieldValue[] {
      return snapshot.customFieldValues;
    },
    get customFieldOptionValues(): ProjectCustomFieldOptionValue[] {
      return snapshot.customFieldOptionValues;
    },
    get dependencies(): ProjectTaskDependency[] {
      return snapshot.dependencies;
    },
    get eventLinks(): ProjectTaskEventLink[] {
      return snapshot.eventLinks;
    },
    get taskChangeEvents(): ProjectTaskChangeEvent[] {
      return snapshot.taskChangeEvents;
    },
    get viewPreferences(): ProjectViewPreference[] {
      return snapshot.viewPreferences;
    },
    get selectedProjectId(): string | null {
      ensureSelectedProject();
      return selectedProjectId;
    },
    set selectedProjectId(projectId: string | null) {
      setSelectedProjectId(projectId);
    },
    get selectedProject(): Project | undefined {
      ensureSelectedProject();
      return projectById(selectedProjectId);
    },
    get selectedGroup(): ProjectGroup | undefined {
      const selectedProject = projectById(selectedProjectId);
      return groupById(selectedProject?.groupId);
    },
    get activeView(): ProjectViewId {
      return activeView;
    },
    set activeView(view: ProjectViewId) {
      setActiveView(view);
    },
    get loaded(): boolean {
      return loaded;
    },
    get loading(): boolean {
      return loading;
    },
    get loadError(): string | null {
      return loadError;
    },
    load: reload,
    ensureLoaded,
    ensureProjectData,
    projectDataLoaded,
    selectProject,
    projectById,
    groupById,
    projectsForGroup,
    projectsForGroupIncludingInactive,
    visibleGroups,
    sectionsForProject,
    sectionsForProjectIncludingInactive,
    statusesForProject,
    tasksForProject,
    tasksForProjectIncludingArchived,
    topLevelTasksForSection,
    topLevelTasksForStatus,
    subtasksForTask,
    subtasksForTaskIncludingArchived,
    defaultSection,
    defaultStatus,
    statusById,
    taskById,
    eventLinksForTask,
    eventLinksForEvent,
    taskChangeEventsForTask,
    recentTaskChangeEventsForProject,
    checklistItemsForTask,
    labelsForProject,
    labelById,
    taskLabelLinksForTask,
    labelsForTask,
    unlinkedLabelsForTask,
    customFieldsForProject,
    customFieldById,
    customFieldOptionsForField,
    customFieldValueForTask,
    customFieldOptionValuesForTask,
    dependenciesBlockingTask,
    dependenciesBlockedByTask,
    savedTaskViewsForProject,
    addGroup,
    setGroupCollapsed,
    updateGroup,
    moveGroup,
    addProject,
    updateProject,
    moveProject,
    addSection,
    updateSection,
    hideSection,
    archiveSection,
    restoreSection,
    applySectionCollapseState,
    moveSection,
    addStatus,
    updateStatus,
    moveStatus,
    addTask,
    addChecklistItem,
    setChecklistItemCompleted,
    updateChecklistItem,
    moveChecklistItem,
    removeChecklistItem,
    addLabel,
    updateLabel,
    moveLabel,
    removeLabel,
    linkTaskLabel,
    unlinkTaskLabel,
    addAndLinkTaskLabel,
    addCustomField,
    updateCustomField,
    moveCustomField,
    removeCustomField,
    addCustomFieldOption,
    updateCustomFieldOption,
    moveCustomFieldOption,
    removeCustomFieldOption,
    saveCustomFieldValue,
    updateTask,
    updateTasks,
    setTaskStatus,
    setTasksStatus,
    toggleTaskDone,
    setTaskPriority,
    setTasksPriority,
    setTaskType,
    archiveTasks,
    restoreTasks,
    moveTaskInSection,
    moveTaskInStatus,
    moveSubtask,
    promoteSubtask,
    demoteTaskToSubtask,
    linkTaskEvent,
    unlinkTaskEvent,
    setEventTaskLinks,
    searchLinkableEvents,
    addTaskDependency,
    removeTaskDependency,
    saveTaskView,
    deleteTaskView,
    saveTaskListColumns,
  };
}
