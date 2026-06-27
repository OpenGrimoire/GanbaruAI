import {
  createProjectChecklistItem,
  createProjectCustomEmoji,
  createProjectCustomField,
  createProjectCustomFieldOption,
  createProjectLabel,
  createProject as createProjectBackend,
  createProjectGroup,
  createProjectSection,
  createProjectStatus,
  createProjectTaskDependency,
  createProjectTask,
  deleteProjectGroup,
  deleteProjectCustomField,
  deleteProjectCustomFieldOption,
  deleteProjectCustomEmoji,
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
  savedTaskViewPreferenceKey,
  savedTaskViewPreferenceValue,
} from "$lib/projects/saved-task-views";
import {
  TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
  taskListColumnWidthsPreferenceValue,
  type ProjectTaskListColumnWidths,
} from "$lib/projects/project-list-view";
import {
  TASK_LIST_COLUMNS_PREFERENCE_KEY,
  taskListColumnsPreferenceValue,
} from "$lib/projects/task-list-columns";
import * as projectSnapshot from "$lib/projects/project-snapshot";
import { normalizeProjectName } from "$lib/projects/project-text";
import {
  checklistItemUpdatePayload,
  customFieldOptionUpdatePayload,
  customFieldUpdatePayload,
  groupUpdatePayload,
  labelUpdatePayload,
  projectUpdatePayload,
  sectionUpdatePayload,
  taskUpdatePayload,
  type ProjectTaskUpdatePatch,
} from "$lib/projects/project-update-payloads";
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
  ProjectCreate,
  ProjectCustomEmoji,
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldOptionValue,
  ProjectCustomFieldType,
  ProjectCustomFieldValue,
  ProjectCustomFieldValueUpdate,
  ProjectGroup,
  ProjectLabel,
  ProjectLinkableEvent,
  ProjectPriority,
  ProjectSavedTaskView,
  ProjectsSnapshot,
  ProjectSection,
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
  customEmojis: [],
});
let loaded = $state(false);
let loading = $state(false);
let loadError = $state<string | null>(null);
let selectedProjectId = $state<string | null>(loadSavedActiveProjectId());
let activeView = $state<ProjectViewId>(loadSavedProjectViewId());
let loadedProjectIds = $state<string[]>([]);
let loadRequestId = 0;
let selectProjectRequestId = 0;

function activeProjects(): Project[] {
  return projectSnapshot.activeProjects(snapshot);
}

function firstProjectId(): string | null {
  return projectSnapshot.firstProjectId(snapshot);
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
  return projectSnapshot.projectById(snapshot, projectId);
}

function groupById(groupId: string | null | undefined): ProjectGroup | undefined {
  return projectSnapshot.groupById(snapshot, groupId);
}

function sectionsForProject(projectId: string | null | undefined): ProjectSection[] {
  return projectSnapshot.sectionsForProject(snapshot, projectId);
}

function sectionsForProjectIncludingInactive(projectId: string | null | undefined): ProjectSection[] {
  return projectSnapshot.sectionsForProjectIncludingInactive(snapshot, projectId);
}

function statusesForProject(projectId: string | null | undefined): ProjectStatus[] {
  return projectSnapshot.statusesForProject(snapshot, projectId);
}

function tasksForProject(projectId: string | null | undefined): ProjectTask[] {
  return projectSnapshot.tasksForProject(snapshot, projectId);
}

function tasksForProjectIncludingArchived(projectId: string | null | undefined): ProjectTask[] {
  return projectSnapshot.tasksForProjectIncludingArchived(snapshot, projectId);
}

function topLevelTasksForSection(projectId: string, sectionId: string): ProjectTask[] {
  return projectSnapshot.topLevelTasksForSection(snapshot, projectId, sectionId);
}

function topLevelTasksForStatus(projectId: string, statusId: string): ProjectTask[] {
  return projectSnapshot.topLevelTasksForStatus(snapshot, projectId, statusId);
}

function subtasksForTask(parentTaskId: string | null | undefined): ProjectTask[] {
  return projectSnapshot.subtasksForTask(snapshot, parentTaskId);
}

function subtasksForTaskIncludingArchived(parentTaskId: string | null | undefined): ProjectTask[] {
  return projectSnapshot.subtasksForTaskIncludingArchived(snapshot, parentTaskId);
}

function projectsForGroup(groupId: string): Project[] {
  return projectSnapshot.projectsForGroup(snapshot, groupId);
}

function projectsForGroupIncludingInactive(groupId: string): Project[] {
  return projectSnapshot.projectsForGroupIncludingInactive(snapshot, groupId);
}

function visibleGroups(): ProjectGroup[] {
  return projectSnapshot.visibleGroups(snapshot);
}

function defaultSection(projectId: string): ProjectSection | undefined {
  return projectSnapshot.defaultSection(snapshot, projectId);
}

function defaultStatus(projectId: string): ProjectStatus | undefined {
  return projectSnapshot.defaultStatus(snapshot, projectId);
}

function doneStatus(projectId: string): ProjectStatus | undefined {
  return projectSnapshot.doneStatus(snapshot, projectId);
}

function reopenStatus(projectId: string): ProjectStatus | undefined {
  return projectSnapshot.reopenStatus(snapshot, projectId);
}

function statusById(statusId: string): ProjectStatus | undefined {
  return projectSnapshot.statusById(snapshot, statusId);
}

function taskById(taskId: string | null | undefined): ProjectTask | undefined {
  return projectSnapshot.taskById(snapshot, taskId);
}

function eventLinksForTask(taskId: string | null | undefined): ProjectTaskEventLink[] {
  return projectSnapshot.eventLinksForTask(snapshot, taskId);
}

function eventLinksForEvent(eventId: string | null | undefined): ProjectTaskEventLink[] {
  return projectSnapshot.eventLinksForEvent(snapshot, eventId);
}

function taskChangeEventsForTask(taskId: string | null | undefined): ProjectTaskChangeEvent[] {
  return projectSnapshot.taskChangeEventsForTask(snapshot, taskId);
}

function recentTaskChangeEventsForProject(
  projectId: string | null | undefined,
  limit = 8,
): ProjectTaskChangeEvent[] {
  return projectSnapshot.recentTaskChangeEventsForProject(snapshot, projectId, limit);
}

function checklistItemsForTask(taskId: string | null | undefined): ProjectChecklistItem[] {
  return projectSnapshot.checklistItemsForTask(snapshot, taskId);
}

function labelsForProject(projectId: string | null | undefined): ProjectLabel[] {
  return projectSnapshot.labelsForProject(snapshot, projectId);
}

function labelById(labelId: string | null | undefined): ProjectLabel | undefined {
  return projectSnapshot.labelById(snapshot, labelId);
}

function taskLabelLinksForTask(taskId: string | null | undefined): ProjectTaskLabelLink[] {
  return projectSnapshot.taskLabelLinksForTask(snapshot, taskId);
}

function labelsForTask(taskId: string | null | undefined): ProjectLabel[] {
  return projectSnapshot.labelsForTask(snapshot, taskId);
}

function unlinkedLabelsForTask(task: ProjectTask | null | undefined): ProjectLabel[] {
  return projectSnapshot.unlinkedLabelsForTask(snapshot, task);
}

function projectLabelByName(projectId: string, name: string): ProjectLabel | undefined {
  return projectSnapshot.projectLabelByName(snapshot, projectId, name);
}

function customFieldsForProject(projectId: string | null | undefined): ProjectCustomField[] {
  return projectSnapshot.customFieldsForProject(snapshot, projectId);
}

function customFieldById(fieldId: string | null | undefined): ProjectCustomField | undefined {
  return projectSnapshot.customFieldById(snapshot, fieldId);
}

function customFieldByName(projectId: string, name: string): ProjectCustomField | undefined {
  return projectSnapshot.customFieldByName(snapshot, projectId, name);
}

function customFieldOptionsForField(fieldId: string | null | undefined): ProjectCustomFieldOption[] {
  return projectSnapshot.customFieldOptionsForField(snapshot, fieldId);
}

function customFieldOptionByName(fieldId: string, name: string): ProjectCustomFieldOption | undefined {
  return projectSnapshot.customFieldOptionByName(snapshot, fieldId, name);
}

function customFieldValueForTask(
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldValue | undefined {
  return projectSnapshot.customFieldValueForTask(snapshot, taskId, fieldId);
}

function customFieldOptionValuesForTask(
  taskId: string | null | undefined,
  fieldId: string | null | undefined,
): ProjectCustomFieldOption[] {
  return projectSnapshot.customFieldOptionValuesForTask(snapshot, taskId, fieldId);
}

function dependenciesBlockingTask(taskId: string | null | undefined): ProjectTaskDependency[] {
  return projectSnapshot.dependenciesBlockingTask(snapshot, taskId);
}

function dependenciesBlockedByTask(taskId: string | null | undefined): ProjectTaskDependency[] {
  return projectSnapshot.dependenciesBlockedByTask(snapshot, taskId);
}

function savedTaskViewsForProject(projectId: string | null | undefined): ProjectSavedTaskView[] {
  return projectSnapshot.savedTaskViewsForProject(snapshot, projectId);
}

function taskClosure(tasks: ProjectTask[]): ProjectTask[] {
  return projectSnapshot.taskClosure(snapshot, tasks);
}

function nextGroupSortOrder(): number {
  return projectSnapshot.nextGroupSortOrder(snapshot);
}

function nextProjectSortOrder(groupId: string): number {
  return projectSnapshot.nextProjectSortOrder(snapshot, groupId);
}

function nextSectionSortOrder(projectId: string): number {
  return projectSnapshot.nextSectionSortOrder(snapshot, projectId);
}

function nextStatusSortOrder(projectId: string): number {
  return projectSnapshot.nextStatusSortOrder(snapshot, projectId);
}

function nextTaskSectionSortOrder(projectId: string, sectionId: string): number {
  return projectSnapshot.nextTaskSectionSortOrder(snapshot, projectId, sectionId);
}

function nextSubtaskSortOrder(parentTaskId: string): number {
  return projectSnapshot.nextSubtaskSortOrder(snapshot, parentTaskId);
}

function nextChecklistSortOrder(taskId: string): number {
  return projectSnapshot.nextChecklistSortOrder(snapshot, taskId);
}

function nextLabelSortOrder(projectId: string): number {
  return projectSnapshot.nextLabelSortOrder(snapshot, projectId);
}

function nextCustomFieldSortOrder(projectId: string): number {
  return projectSnapshot.nextCustomFieldSortOrder(snapshot, projectId);
}

function nextCustomFieldOptionSortOrder(fieldId: string): number {
  return projectSnapshot.nextCustomFieldOptionSortOrder(snapshot, fieldId);
}

function nextCustomEmojiSortOrder(): number {
  return Math.max(0, ...snapshot.customEmojis.map((emoji) => emoji.sortOrder)) + 1000;
}

function nextTaskStatusSortOrder(projectId: string, statusId: string): number {
  return projectSnapshot.nextTaskStatusSortOrder(snapshot, projectId, statusId);
}

function applyLoadedSnapshot(incoming: ProjectsSnapshot, projectId: string | null | undefined): void {
  if (!projectId) {
    snapshot = {
      ...snapshot,
      groups: incoming.groups,
      projects: incoming.projects,
      viewPreferences: incoming.viewPreferences,
      customEmojis: incoming.customEmojis,
    };
    return;
  }
  snapshot = projectSnapshot.mergeProjectSnapshot(snapshot, incoming, projectId);
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
  const requestId = ++selectProjectRequestId;
  if (!projectId || projectDataLoaded(projectId)) {
    setSelectedProjectId(projectId);
    return;
  }
  await ensureProjectData(projectId);
  if (requestId !== selectProjectRequestId) return;
  if (!projectDataLoaded(projectId)) return;
  setSelectedProjectId(projectId);
}

async function addGroup(name: string): Promise<void> {
  const displayName = normalizeProjectName(name);
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
  const nextName = normalizeProjectName(patch.name ?? group.name);
  if (!nextName) return;
  await updateProjectGroup(groupUpdatePayload(group, { ...patch, name: nextName }));
  await reload();
}

async function removeGroup(group: ProjectGroup): Promise<void> {
  await deleteProjectGroup(group.id);
  if (selectedProjectId && group.id === projectById(selectedProjectId)?.groupId) {
    setSelectedProjectId(null);
  }
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
  const displayName = normalizeProjectName(name);
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
    defaultEventName: templateDefaults.defaultEventName,
    defaultEventTimeMode: templateDefaults.defaultEventTimeMode,
    defaultEventDurationMinutes: templateDefaults.defaultEventDurationMinutes,
    defaultPomodoroMode: templateDefaults.defaultPomodoroMode,
    defaultPomodoroPresetKey: templateDefaults.defaultPomodoroPresetKey,
    defaultPomodoroFocusMinutes: templateDefaults.defaultPomodoroFocusMinutes,
    defaultPomodoroShortBreakMinutes: templateDefaults.defaultPomodoroShortBreakMinutes,
    defaultPomodoroLongBreakMinutes: templateDefaults.defaultPomodoroLongBreakMinutes,
    defaultPomodoroLongBreakAfterFocusCount: templateDefaults.defaultPomodoroLongBreakAfterFocusCount,
    defaultIdleSettingsSource: templateDefaults.defaultIdleSettingsSource,
    defaultIdlePauseEnabled: templateDefaults.defaultIdlePauseEnabled,
    defaultIdleThresholdMinutes: templateDefaults.defaultIdleThresholdMinutes,
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
  const displayName = normalizeProjectName(name);
  if (!displayName) return;
  await createProjectSection({
    id: crypto.randomUUID(),
    projectId,
    name: displayName,
    sortOrder: nextSectionSortOrder(projectId),
  });
  await reload();
}

async function updateSection(
  section: ProjectSection,
  patch: Partial<Pick<ProjectSection, "name" | "sortOrder" | "collapsed" | "hiddenAt" | "archivedAt">>,
): Promise<void> {
  const nextName = normalizeProjectName(patch.name ?? section.name);
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

async function addStatus(
  projectId: string,
  name: string,
  category: ProjectStatusCategory,
): Promise<void> {
  const displayName = normalizeProjectName(name);
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
  const displayName = normalizeProjectName(patch.name ?? status.name);
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
  sectionId?: string,
  statusId?: string,
  parentTaskId: string | null = null,
): Promise<ProjectTask | undefined> {
  const displayTitle = normalizeProjectName(title);
  if (!displayTitle) return;
  await ensureProjectData(projectId);
  const resolvedSectionId = sectionId ?? defaultSection(projectId)?.id;
  const resolvedStatusId = statusId ?? defaultStatus(projectId)?.id;
  if (!resolvedSectionId || !resolvedStatusId) {
    throw new Error("project needs at least one section and one status");
  }
  const taskId = crypto.randomUUID();
  await createProjectTask({
    id: taskId,
    projectId,
    sectionId: resolvedSectionId,
    statusId: resolvedStatusId,
    parentTaskId,
    title: displayTitle,
  });
  await reload(projectId);
  const createdTask = taskById(taskId);
  if (!createdTask) {
    throw new Error("created task was not returned by the project snapshot");
  }
  return createdTask;
}

async function addChecklistItem(taskId: string, title: string): Promise<void> {
  const displayTitle = normalizeProjectName(title);
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
  const nextTitle = normalizeProjectName(patch.title ?? item.title);
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
  const displayName = normalizeProjectName(name);
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
  const nextName = normalizeProjectName(patch.name ?? label.name);
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

async function addCustomEmoji(name: string, assetPath: string): Promise<ProjectCustomEmoji | undefined> {
  const displayName = normalizeProjectName(name);
  if (!displayName) return undefined;
  const emojiId = crypto.randomUUID();
  await createProjectCustomEmoji({
    id: emojiId,
    name: displayName,
    assetPath,
    sortOrder: nextCustomEmojiSortOrder(),
  });
  await reload();
  return snapshot.customEmojis.find((emoji) => emoji.id === emojiId);
}

async function removeCustomEmoji(emojiId: string): Promise<void> {
  await deleteProjectCustomEmoji(emojiId);
  await reload();
}

async function addCustomField(
  projectId: string,
  name: string,
  fieldType: ProjectCustomFieldType,
): Promise<ProjectCustomField | undefined> {
  const displayName = normalizeProjectName(name);
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
  const nextName = normalizeProjectName(patch.name ?? field.name);
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
  const displayName = normalizeProjectName(name);
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
  const nextName = normalizeProjectName(patch.name ?? option.name);
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
  const name = normalizeProjectName(view.name);
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

async function saveTaskListColumnWidths(
  projectId: string,
  widths: ProjectTaskListColumnWidths,
): Promise<void> {
  await upsertProjectViewPreference({
    projectId,
    viewId: "list",
    preferenceKey: TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
    preferenceValue: taskListColumnWidthsPreferenceValue(widths),
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
    get customEmojis(): ProjectCustomEmoji[] {
      return snapshot.customEmojis;
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
    removeGroup,
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
    addCustomEmoji,
    removeCustomEmoji,
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
    saveTaskListColumnWidths,
  };
}
