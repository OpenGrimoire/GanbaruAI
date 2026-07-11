import {
  createProjectChecklistItem,
  createProjectCustomEmoji,
  createProjectCustomField,
  createProjectCustomFieldOption,
  createProjectTag,
  createProject as createProjectBackend,
  createProjectGroup,
  createProjectPriority,
  createProjectSection,
  createProjectStatus,
  createProjectTask,
  createProjectTaskDependency,
  deleteProjectChecklistItem,
  deleteProjectCustomEmoji,
  deleteProjectCustomField,
  deleteProjectCustomFieldOption,
  deleteProjectGroup,
  deleteProjectPriority,
  deleteProjectStatus,
  deleteProjectTag,
  deleteProjectTaskDependency,
  deleteProjectViewPreference,
  linkProjectTaskEvent,
  linkProjectTaskTag,
  searchProjectLinkableEvents,
  setProjectGroupCollapsed,
  unlinkProjectTaskEvent,
  unlinkProjectTaskTag,
  updateProject as updateProjectBackend,
  updateProjectNotesSettings,
  updateProjectChecklistItem,
  updateProjectCustomField,
  updateProjectCustomFieldOption,
  updateProjectCustomFieldValue,
  updateProjectGroup,
  updateProjectPriority,
  updateProjectSection,
  updateProjectStatus,
  updateProjectTag,
  updateProjectTask,
  upsertProjectViewPreference,
} from "$lib/api/projects";
import {
  invalidateAssetUrl,
  invalidateAssetUrlKind,
} from "$lib/api/asset-url-cache";
import type { EventColor } from "$lib/components/calendar/types";
import type { NotesPageOpenMode } from "$lib/notes/page-open-mode";
import type { NotesHistoryRetentionDays } from "$lib/notes/history-retention";
import {
  TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
  taskListColumnWidthsPreferenceValue,
  type ProjectTaskListColumnWidths,
} from "$lib/projects/project-list-view";
import { normalizeProjectName } from "$lib/projects/project-text";
import { parseProjectIcon } from "$lib/projects/project-icons";
import {
  checklistItemUpdatePayload,
  customFieldOptionUpdatePayload,
  customFieldUpdatePayload,
  groupUpdatePayload,
  projectUpdatePayload,
  sectionUpdatePayload,
  tagUpdatePayload,
  taskUpdatePayload,
  type ProjectTaskUpdatePatch,
} from "$lib/projects/project-update-payloads";
import {
  savedTaskViewPreferenceKey,
  savedTaskViewPreferenceValue,
} from "$lib/projects/saved-task-views";
import {
  TASK_LIST_COLUMNS_PREFERENCE_KEY,
  taskListColumnsPreferenceValue,
} from "$lib/projects/task-list-columns";
import {
  PROJECT_TAG_DEFAULT_COLOR,
  PROJECT_TEMPLATE_DEFAULTS,
  type Project,
  type ProjectChecklistItem,
  type ProjectCreate,
  type ProjectCustomEmoji,
  type ProjectCustomField,
  type ProjectCustomFieldOption,
  type ProjectCustomFieldType,
  type ProjectCustomFieldValueUpdate,
  type ProjectGroup,
  type ProjectLinkableEvent,
  type ProjectPriority,
  type ProjectPriorityConfig,
  type ProjectSavedTaskView,
  type ProjectsSnapshot,
  type ProjectSection,
  type ProjectStatus,
  type ProjectStatusCategory,
  type ProjectTag,
  type ProjectTask,
  type ProjectTaskEventLink,
  type ProjectTaskListColumn,
  type ProjectTaskType,
  type ProjectTemplateId,
  type ProjectUpdate,
} from "$lib/projects/types";
import type { ProjectStoreSelectors } from "$lib/stores/project-store-selectors";

export interface ProjectStoreActionContext {
  selectors: ProjectStoreSelectors;
  readSnapshot: () => ProjectsSnapshot;
  updateSnapshot: (updater: (snapshot: ProjectsSnapshot) => ProjectsSnapshot) => void;
  readSelectedProjectId: () => string | null;
  setSelectedProjectId: (projectId: string | null) => void;
  reload: (projectId?: string | null) => Promise<void>;
  ensureProjectData: (projectId: string | null | undefined) => Promise<void>;
}

/**
 * Creates the write side of the project store around state hooks owned by the Svelte store.
 */
export function createProjectStoreActions(context: ProjectStoreActionContext) {
  const {
    ensureProjectData,
    readSelectedProjectId,
    readSnapshot,
    reload,
    selectors,
    setSelectedProjectId,
    updateSnapshot,
  } = context;

  function projectAssetPath(icon: string): string | null {
    const parsed = parseProjectIcon(icon);
    return parsed.kind === "asset" ? parsed.relativePath : null;
  }

  function invalidateReplacedProjectIcon(previousIcon: string, nextIcon: string): void {
    const previousPath = projectAssetPath(previousIcon);
    if (!previousPath || previousPath === projectAssetPath(nextIcon)) return;
    invalidateAssetUrl("project-icon", previousPath);
  }

  async function addGroup(name: string): Promise<void> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return;
    await createProjectGroup({
      id: crypto.randomUUID(),
      name: displayName,
      icon: "folder",
      color: null,
      sortOrder: selectors.nextGroupSortOrder(),
    });
    await reload();
  }

  async function setGroupCollapsed(groupId: string, collapsed: boolean): Promise<void> {
    await setProjectGroupCollapsed(groupId, collapsed);
    updateSnapshot((current) => ({
      ...current,
      groups: current.groups.map((group) =>
        group.id === groupId ? { ...group, collapsed } : group
      ),
    }));
  }

  async function updateGroup(
    group: ProjectGroup,
    patch: Partial<Pick<ProjectGroup, "name" | "icon" | "color" | "sortOrder" | "collapsed">>,
  ): Promise<void> {
    const nextName = normalizeProjectName(patch.name ?? group.name);
    if (!nextName) return;
    const update = groupUpdatePayload(group, { ...patch, name: nextName });
    await updateProjectGroup(update);
    invalidateReplacedProjectIcon(group.icon, update.icon);
    await reload();
  }

  async function removeGroup(group: ProjectGroup): Promise<void> {
    await deleteProjectGroup(group.id);
    invalidateAssetUrlKind("project-icon");
    const selectedProjectId = readSelectedProjectId();
    if (selectedProjectId && group.id === selectors.projectById(selectedProjectId)?.groupId) {
      setSelectedProjectId(null);
    }
    await reload();
  }

  async function moveGroup(group: ProjectGroup, direction: -1 | 1): Promise<void> {
    const ordered = selectors.visibleGroups();
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
      sortOrder: selectors.nextProjectSortOrder(groupId),
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
    const previousIcon = selectors.projectById(project.id)?.icon ?? project.icon;
    await updateProjectBackend(project);
    invalidateReplacedProjectIcon(previousIcon, project.icon);
    await reload();
  }

  async function setNotesSettings(
    projectId: string,
    openMode: NotesPageOpenMode | null,
    historyRetentionDays: NotesHistoryRetentionDays | null,
  ): Promise<void> {
    await updateProjectNotesSettings(projectId, openMode, historyRetentionDays);
    await reload(projectId);
  }

  async function moveProject(project: Project, direction: -1 | 1, includeInactive = false): Promise<void> {
    const ordered = includeInactive
      ? selectors.projectsForGroupIncludingInactive(project.groupId)
      : selectors.projectsForGroup(project.groupId);
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
      sortOrder: selectors.nextSectionSortOrder(projectId),
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
    const updates = selectors.sectionsForProject(projectId)
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
    color: EventColor,
  ): Promise<void> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return;
    await createProjectStatus({
      id: crypto.randomUUID(),
      projectId,
      name: displayName,
      category,
      color,
      sortOrder: selectors.nextStatusSortOrder(projectId),
      terminal: category === "done",
    });
    await reload();
  }

  async function updateStatus(
    status: ProjectStatus,
    patch: Partial<Pick<ProjectStatus, "name" | "category" | "color" | "sortOrder">>,
  ): Promise<void> {
    const displayName = normalizeProjectName(patch.name ?? status.name);
    if (!displayName) return;
    const category = patch.category ?? status.category;
    await updateProjectStatus({
      id: status.id,
      name: displayName,
      category,
      color: patch.color ?? status.color,
      sortOrder: patch.sortOrder ?? status.sortOrder,
      terminal: category === "done",
    });
    await reload();
  }

  async function moveStatus(status: ProjectStatus, direction: -1 | 1): Promise<void> {
    const ordered = selectors.statusesForProject(status.projectId);
    const index = ordered.findIndex((entry) => entry.id === status.id);
    const target = ordered[index + direction];
    if (index < 0 || !target) return;
    await updateProjectStatus({
      id: status.id,
      name: status.name,
      category: status.category,
      color: status.color,
      sortOrder: target.sortOrder,
      terminal: status.category === "done",
    });
    await updateProjectStatus({
      id: target.id,
      name: target.name,
      category: target.category,
      color: target.color,
      sortOrder: status.sortOrder,
      terminal: target.category === "done",
    });
    await reload();
  }

  async function removeStatus(statusId: string): Promise<void> {
    await deleteProjectStatus(statusId);
    await reload();
  }

  async function addPriority(
    projectId: string,
    name: string,
    color: EventColor,
  ): Promise<void> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return;
    await createProjectPriority({
      id: crypto.randomUUID(),
      projectId,
      name: displayName,
      color,
      sortOrder: selectors.nextPrioritySortOrder(projectId),
    });
    await reload();
  }

  async function updatePriority(
    priority: ProjectPriorityConfig,
    patch: Partial<Pick<ProjectPriorityConfig, "name" | "color" | "sortOrder">>,
  ): Promise<void> {
    const displayName = normalizeProjectName(patch.name ?? priority.name);
    if (!displayName) return;
    await updateProjectPriority({
      id: priority.id,
      projectId: priority.projectId,
      name: displayName,
      color: patch.color ?? priority.color,
      sortOrder: patch.sortOrder ?? priority.sortOrder,
    });
    await reload();
  }

  async function movePriority(priority: ProjectPriorityConfig, direction: -1 | 1): Promise<void> {
    const ordered = selectors.prioritiesForProject(priority.projectId);
    const index = ordered.findIndex((entry) => entry.id === priority.id);
    const target = ordered[index + direction];
    if (index < 0 || !target) return;
    await updateProjectPriority({
      id: priority.id,
      projectId: priority.projectId,
      name: priority.name,
      color: priority.color,
      sortOrder: target.sortOrder,
    });
    await updateProjectPriority({
      id: target.id,
      projectId: target.projectId,
      name: target.name,
      color: target.color,
      sortOrder: priority.sortOrder,
    });
    await reload();
  }

  async function removePriority(priority: ProjectPriorityConfig): Promise<void> {
    await deleteProjectPriority(priority.projectId, priority.id);
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
    const resolvedSectionId = sectionId ?? selectors.defaultSection(projectId)?.id;
    const resolvedStatusId = statusId ?? selectors.defaultStatus(projectId)?.id;
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
    const createdTask = selectors.taskById(taskId);
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
      sortOrder: selectors.nextChecklistSortOrder(taskId),
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
    const ordered = selectors.checklistItemsForTask(item.taskId);
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

  async function addTag(
    projectId: string,
    name: string,
    color: ProjectTag["color"] | null = PROJECT_TAG_DEFAULT_COLOR,
  ): Promise<ProjectTag | undefined> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return undefined;
    const existingTag = selectors.projectTagByName(projectId, displayName);
    if (existingTag) return existingTag;
    const tagId = crypto.randomUUID();
    await createProjectTag({
      id: tagId,
      projectId,
      name: displayName,
      color: color ?? PROJECT_TAG_DEFAULT_COLOR,
      sortOrder: selectors.nextTagSortOrder(projectId),
    });
    await reload();
    return selectors.tagById(tagId);
  }

  async function updateTag(
    tag: ProjectTag,
    patch: Partial<Pick<ProjectTag, "name" | "color" | "sortOrder">>,
  ): Promise<void> {
    const nextName = normalizeProjectName(patch.name ?? tag.name);
    if (!nextName) return;
    await updateProjectTag(tagUpdatePayload(tag, { ...patch, name: nextName }));
    await reload();
  }

  async function moveTag(tag: ProjectTag, direction: -1 | 1): Promise<void> {
    const ordered = selectors.tagsForProject(tag.projectId);
    const index = ordered.findIndex((entry) => entry.id === tag.id);
    const target = ordered[index + direction];
    if (index < 0 || !target) return;
    await updateProjectTag(tagUpdatePayload(tag, { sortOrder: target.sortOrder }));
    await updateProjectTag(tagUpdatePayload(target, { sortOrder: tag.sortOrder }));
    await reload();
  }

  async function removeTag(tagId: string): Promise<void> {
    await deleteProjectTag(tagId);
    await reload();
  }

  async function linkTaskTag(taskId: string, tagId: string): Promise<void> {
    await linkProjectTaskTag({ taskId, tagId });
    await reload();
  }

  async function unlinkTaskTag(taskId: string, tagId: string): Promise<void> {
    await unlinkProjectTaskTag(taskId, tagId);
    await reload();
  }

  async function addAndLinkTaskTag(task: ProjectTask, name: string): Promise<void> {
    const tag = await addTag(task.projectId, name);
    if (!tag) return;
    await linkTaskTag(task.id, tag.id);
  }

  async function addCustomEmoji(name: string, assetPath: string): Promise<ProjectCustomEmoji | undefined> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return undefined;
    const emojiId = crypto.randomUUID();
    await createProjectCustomEmoji({
      id: emojiId,
      name: displayName,
      assetPath,
      sortOrder: selectors.nextCustomEmojiSortOrder(),
    });
    await reload();
    return readSnapshot().customEmojis.find((emoji) => emoji.id === emojiId);
  }

  async function removeCustomEmoji(emojiId: string): Promise<void> {
    const assetPath = readSnapshot().customEmojis.find((emoji) => emoji.id === emojiId)?.assetPath;
    await deleteProjectCustomEmoji(emojiId);
    if (assetPath) invalidateAssetUrl("project-icon", assetPath);
    else invalidateAssetUrlKind("project-icon");
    await reload();
  }

  async function addCustomField(
    projectId: string,
    name: string,
    fieldType: ProjectCustomFieldType,
  ): Promise<ProjectCustomField | undefined> {
    const displayName = normalizeProjectName(name);
    if (!displayName) return undefined;
    const existingField = selectors.customFieldByName(projectId, displayName);
    if (existingField) return existingField;
    const fieldId = crypto.randomUUID();
    await createProjectCustomField({
      id: fieldId,
      projectId,
      name: displayName,
      fieldType,
      sortOrder: selectors.nextCustomFieldSortOrder(projectId),
    });
    await reload();
    return selectors.customFieldById(fieldId);
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
    const ordered = selectors.customFieldsForProject(field.projectId);
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
    const existingOption = selectors.customFieldOptionByName(fieldId, displayName);
    if (existingOption) return existingOption;
    const optionId = crypto.randomUUID();
    await createProjectCustomFieldOption({
      id: optionId,
      fieldId,
      name: displayName,
      sortOrder: selectors.nextCustomFieldOptionSortOrder(fieldId),
    });
    await reload();
    return readSnapshot().customFieldOptions.find((option) => option.id === optionId);
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
    const ordered = selectors.customFieldOptionsForField(option.fieldId);
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
    await updateTask(task, {
      statusId,
      statusSortOrder: selectors.nextTaskStatusSortOrder(task.projectId, statusId),
    });
  }

  async function setTasksStatus(tasks: ProjectTask[], statusId: string): Promise<void> {
    const firstTask = tasks[0];
    if (!firstTask) return;
    const startSortOrder = selectors.nextTaskStatusSortOrder(firstTask.projectId, statusId);
    await updateTasks(tasks, (_task, index) => ({
      statusId,
      statusSortOrder: startSortOrder + index * 1000,
    }));
  }

  async function toggleTaskDone(task: ProjectTask): Promise<void> {
    const currentStatus = selectors.statusById(task.statusId);
    const target = currentStatus?.terminal
      ? selectors.reopenStatus(task.projectId)
      : selectors.doneStatus(task.projectId);
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
    await updateTasks(selectors.taskClosure(tasks), () => ({ archivedAt }));
  }

  async function restoreTasks(tasks: ProjectTask[]): Promise<void> {
    await updateTasks(selectors.taskClosure(tasks), () => ({ archivedAt: undefined }));
  }

  async function setTaskType(task: ProjectTask, taskType: ProjectTaskType): Promise<void> {
    await updateTask(task, { taskType });
  }

  async function moveTaskInSection(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (task.parentTaskId) return;
    const ordered = selectors.topLevelTasksForSection(task.projectId, task.sectionId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    const target = ordered[index + direction];
    if (index < 0 || !target) return;
    await updateProjectTask(taskUpdatePayload(task, { sectionSortOrder: target.sectionSortOrder }));
    await updateProjectTask(taskUpdatePayload(target, { sectionSortOrder: task.sectionSortOrder }));
    await reload();
  }

  async function moveTaskInStatus(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (task.parentTaskId) return;
    const ordered = selectors.topLevelTasksForStatus(task.projectId, task.statusId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    const target = ordered[index + direction];
    if (index < 0 || !target) return;
    await updateProjectTask(taskUpdatePayload(task, { statusSortOrder: target.statusSortOrder }));
    await updateProjectTask(taskUpdatePayload(target, { statusSortOrder: task.statusSortOrder }));
    await reload();
  }

  async function moveSubtask(task: ProjectTask, direction: -1 | 1): Promise<void> {
    if (!task.parentTaskId) return;
    const ordered = selectors.subtasksForTask(task.parentTaskId);
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
      sectionSortOrder: selectors.nextTaskSectionSortOrder(task.projectId, task.sectionId),
      statusSortOrder: selectors.nextTaskStatusSortOrder(task.projectId, task.statusId),
    });
  }

  async function demoteTaskToSubtask(task: ProjectTask, parentTask: ProjectTask): Promise<void> {
    if (task.parentTaskId || parentTask.parentTaskId || task.id === parentTask.id) return;
    if (task.projectId !== parentTask.projectId) return;
    if (selectors.subtasksForTaskIncludingArchived(task.id).length > 0) return;
    await updateTask(task, {
      parentTaskId: parentTask.id,
      sectionId: parentTask.sectionId,
      sectionSortOrder: selectors.nextSubtaskSortOrder(parentTask.id),
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
    const existing = selectors.eventLinksForEvent(eventId);
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

  return {
    addGroup,
    setGroupCollapsed,
    updateGroup,
    removeGroup,
    moveGroup,
    addProject,
    updateProject,
    setNotesSettings,
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
    removeStatus,
    addPriority,
    updatePriority,
    movePriority,
    removePriority,
    addTask,
    addChecklistItem,
    setChecklistItemCompleted,
    updateChecklistItem,
    moveChecklistItem,
    removeChecklistItem,
    addTag,
    updateTag,
    moveTag,
    removeTag,
    linkTaskTag,
    unlinkTaskTag,
    addAndLinkTaskTag,
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

export type ProjectStoreActions = ReturnType<typeof createProjectStoreActions>;
