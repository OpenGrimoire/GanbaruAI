import type {
  Project,
  ProjectChecklistItem,
  ProjectChecklistItemUpdate,
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldOptionUpdate,
  ProjectCustomFieldUpdate,
  ProjectGroup,
  ProjectGroupUpdate,
  ProjectLabel,
  ProjectLabelUpdate,
  ProjectSection,
  ProjectSectionUpdate,
  ProjectTask,
  ProjectTaskUpdate,
  ProjectUpdate,
} from "$lib/projects/types";

export type ProjectTaskUpdatePatch = Partial<ProjectTask> & { changeReason?: string | null };

export function taskUpdatePayload(task: ProjectTask, patch: ProjectTaskUpdatePatch): ProjectTaskUpdate {
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

export function checklistItemUpdatePayload(
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

export function labelUpdatePayload(
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

export function customFieldUpdatePayload(
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

export function customFieldOptionUpdatePayload(
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

export function groupUpdatePayload(
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

export function projectUpdatePayload(project: Project, patch: Partial<Project>): ProjectUpdate {
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

export function sectionUpdatePayload(
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
