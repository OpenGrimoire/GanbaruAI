import { invoke } from "@tauri-apps/api/core";
import { ensureDbUrl } from "$lib/api/db";
import { normalizeEventColor } from "$lib/components/calendar/utils";
import { localTimezone } from "$lib/stores/calendar-event-payloads";
import { toCalendarDate } from "$lib/stores/map-row";
import {
  DEFAULT_FOCUS_IDLE_THRESHOLD_MINUTES,
  FOCUS_IDLE_THRESHOLD_MINUTES_OPTIONS,
  type FocusIdleThresholdMinutes,
} from "$lib/stores/preferences";
import type {
  Project,
  ProjectChecklistItem,
  ProjectChecklistItemCreate,
  ProjectChecklistItemUpdate,
  ProjectCreate,
  ProjectCustomEmoji,
  ProjectCustomEmojiCreate,
  ProjectCustomField,
  ProjectCustomFieldCreate,
  ProjectCustomFieldOption,
  ProjectCustomFieldOptionCreate,
  ProjectCustomFieldOptionUpdate,
  ProjectCustomFieldOptionValue,
  ProjectCustomFieldUpdate,
  ProjectCustomFieldValue,
  ProjectCustomFieldValueUpdate,
  ProjectGroup,
  ProjectGroupCreate,
  ProjectGroupUpdate,
  ProjectLabel,
  ProjectLabelCreate,
  ProjectLabelUpdate,
  ProjectLinkableEvent,
  ProjectSection,
  ProjectSectionCreate,
  ProjectSectionUpdate,
  ProjectStatus,
  ProjectStatusCreate,
  ProjectStatusUpdate,
  ProjectUpdate,
  ProjectsSnapshot,
  ProjectTask,
  ProjectTaskChangeEvent,
  ProjectTaskCreate,
  ProjectTaskDependency,
  ProjectTaskDependencyCreate,
  ProjectTaskEventLink,
  ProjectTaskEventLinkCreate,
  ProjectTaskLabelLink,
  ProjectTaskLabelLinkCreate,
  ProjectTaskUpdate,
  ProjectViewPreference,
  ProjectViewPreferenceUpsert,
} from "$lib/projects/types";

interface ProjectGroupRow {
  id: string;
  name: string;
  icon: string;
  color: number | null;
  sort_order: number;
  collapsed: number;
  hidden_at: string | null;
  archived_at: string | null;
  created_at: string;
  updated_at: string;
}

interface ProjectRow {
  id: string;
  group_id: string;
  name: string;
  icon: string;
  color: number | null;
  sort_order: number;
  status: Project["status"];
  default_event_name: string | null;
  default_event_time_mode: Project["defaultEventTimeMode"];
  default_event_duration_minutes: number | null;
  default_pomodoro_mode: Project["defaultPomodoroMode"];
  default_pomodoro_preset_key: NonNullable<Project["defaultPomodoroPresetKey"]> | null;
  default_pomodoro_focus_minutes: number | null;
  default_pomodoro_short_break_minutes: number | null;
  default_pomodoro_long_break_minutes: number | null;
  default_pomodoro_long_break_after_focus_count: number | null;
  default_idle_settings_source: string;
  default_idle_pause_enabled: number;
  default_idle_threshold_minutes: number;
  focus_playlist_id: string | null;
  break_playlist_id: string | null;
  work_environment_id: string | null;
  blocker_ruleset_id: string | null;
  created_at: string;
  updated_at: string;
}

interface ProjectSectionRow {
  id: string;
  project_id: string;
  name: string;
  sort_order: number;
  collapsed: number;
  hidden_at: string | null;
  archived_at: string | null;
  created_at: string;
  updated_at: string;
}

interface ProjectStatusRow {
  id: string;
  project_id: string;
  name: string;
  category: ProjectStatus["category"];
  sort_order: number;
  terminal: number;
  created_at: string;
  updated_at: string;
}

interface ProjectTaskRow {
  id: string;
  project_id: string;
  section_id: string;
  status_id: string;
  parent_task_id: string | null;
  title: string;
  description: string;
  priority: ProjectTask["priority"];
  task_type: ProjectTask["taskType"];
  section_sort_order: number;
  status_sort_order: number;
  estimate_minutes: number | null;
  due_date: string | null;
  due_time: string | null;
  start_date: string | null;
  start_time: string | null;
  target_end_date: string | null;
  completed_at: string | null;
  archived_at: string | null;
  blocker_reason: string | null;
  milestone: number;
  created_at: string;
  updated_at: string;
}

interface ProjectChecklistItemRow {
  id: string;
  task_id: string;
  title: string;
  completed_at: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ProjectLabelRow {
  id: string;
  project_id: string;
  name: string;
  color: number | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ProjectTaskLabelLinkRow {
  task_id: string;
  label_id: string;
  created_at: string;
}

interface ProjectCustomFieldRow {
  id: string;
  project_id: string;
  name: string;
  field_type: ProjectCustomField["fieldType"];
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ProjectCustomFieldOptionRow {
  id: string;
  field_id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ProjectCustomFieldValueRow {
  task_id: string;
  field_id: string;
  text_value: string | null;
  number_value: number | null;
  date_value: string | null;
  checkbox_value: number | null;
  updated_at: string;
}

interface ProjectCustomFieldOptionValueRow {
  task_id: string;
  field_id: string;
  option_id: string;
  created_at: string;
}

interface ProjectTaskDependencyRow {
  id: string;
  blocking_task_id: string;
  blocked_task_id: string;
  dependency_type: ProjectTaskDependency["dependencyType"];
  created_at: string;
}

interface ProjectTaskEventLinkRow {
  task_id: string;
  event_id: string;
  link_kind: ProjectTaskEventLink["linkKind"];
  created_at: string;
}

interface ProjectTaskChangeEventRow {
  id: string;
  task_id: string;
  event_type: ProjectTaskChangeEvent["eventType"];
  field_name: string | null;
  old_value: string | null;
  new_value: string | null;
  reason: string | null;
  occurred_at: string;
}

interface ProjectViewPreferenceRow {
  project_id: string;
  view_id: ProjectViewPreference["viewId"];
  preference_key: string;
  preference_value: string;
  updated_at: string;
}

interface ProjectCustomEmojiRow {
  id: string;
  name: string;
  asset_path: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ProjectLinkableEventTaskRow {
  task_id: string;
  title: string;
  archived_at: string | null;
}

interface ProjectLinkableEventRow {
  id: string;
  project_id: string;
  title: string;
  start_time: string;
  end_time: string;
  timezone: string;
  calendar_id: string;
  color: number | null;
  all_day: number;
  status: string;
  linked_tasks: ProjectLinkableEventTaskRow[];
}

interface ProjectsSnapshotRows {
  groups: ProjectGroupRow[];
  projects: ProjectRow[];
  sections: ProjectSectionRow[];
  statuses: ProjectStatusRow[];
  tasks: ProjectTaskRow[];
  checklist_items: ProjectChecklistItemRow[];
  labels: ProjectLabelRow[];
  task_label_links: ProjectTaskLabelLinkRow[];
  custom_fields: ProjectCustomFieldRow[];
  custom_field_options: ProjectCustomFieldOptionRow[];
  custom_field_values: ProjectCustomFieldValueRow[];
  custom_field_option_values: ProjectCustomFieldOptionValueRow[];
  dependencies: ProjectTaskDependencyRow[];
  event_links: ProjectTaskEventLinkRow[];
  task_change_events: ProjectTaskChangeEventRow[];
  view_preferences: ProjectViewPreferenceRow[];
  custom_emojis: ProjectCustomEmojiRow[];
}

function optionalText(value: string | null): string | undefined {
  return value ?? undefined;
}

function optionalNumber(value: number | null): number | undefined {
  return value ?? undefined;
}

function mapProjectIdleSettingsSource(value: string): Project["defaultIdleSettingsSource"] {
  return value === "custom" ? "custom" : "global";
}

function isFocusIdleThresholdMinutes(value: number): value is FocusIdleThresholdMinutes {
  return FOCUS_IDLE_THRESHOLD_MINUTES_OPTIONS.some((option) => option === value);
}

function mapFocusIdleThresholdMinutes(value: number): FocusIdleThresholdMinutes {
  return isFocusIdleThresholdMinutes(value) ? value : DEFAULT_FOCUS_IDLE_THRESHOLD_MINUTES;
}

function mapGroup(row: ProjectGroupRow): ProjectGroup {
  return {
    id: row.id,
    name: row.name,
    icon: row.icon,
    color: optionalNumber(row.color),
    sortOrder: row.sort_order,
    collapsed: row.collapsed !== 0,
    hiddenAt: optionalText(row.hidden_at),
    archivedAt: optionalText(row.archived_at),
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapProject(row: ProjectRow): Project {
  return {
    id: row.id,
    groupId: row.group_id,
    name: row.name,
    icon: row.icon,
    color: optionalNumber(row.color),
    sortOrder: row.sort_order,
    status: row.status,
    defaultEventName: row.default_event_name,
    defaultEventTimeMode: row.default_event_time_mode,
    defaultEventDurationMinutes: row.default_event_duration_minutes,
    defaultPomodoroMode: row.default_pomodoro_mode,
    defaultPomodoroPresetKey: row.default_pomodoro_preset_key ?? undefined,
    defaultPomodoroFocusMinutes: optionalNumber(row.default_pomodoro_focus_minutes),
    defaultPomodoroShortBreakMinutes: optionalNumber(row.default_pomodoro_short_break_minutes),
    defaultPomodoroLongBreakMinutes: optionalNumber(row.default_pomodoro_long_break_minutes),
    defaultPomodoroLongBreakAfterFocusCount: optionalNumber(row.default_pomodoro_long_break_after_focus_count),
    defaultIdleSettingsSource: mapProjectIdleSettingsSource(row.default_idle_settings_source),
    defaultIdlePauseEnabled: row.default_idle_pause_enabled !== 0,
    defaultIdleThresholdMinutes: mapFocusIdleThresholdMinutes(row.default_idle_threshold_minutes),
    focusPlaylistId: optionalText(row.focus_playlist_id),
    breakPlaylistId: optionalText(row.break_playlist_id),
    workEnvironmentId: optionalText(row.work_environment_id),
    blockerRulesetId: optionalText(row.blocker_ruleset_id),
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapSection(row: ProjectSectionRow): ProjectSection {
  return {
    id: row.id,
    projectId: row.project_id,
    name: row.name,
    sortOrder: row.sort_order,
    collapsed: row.collapsed !== 0,
    hiddenAt: optionalText(row.hidden_at),
    archivedAt: optionalText(row.archived_at),
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapStatus(row: ProjectStatusRow): ProjectStatus {
  return {
    id: row.id,
    projectId: row.project_id,
    name: row.name,
    category: row.category,
    sortOrder: row.sort_order,
    terminal: row.terminal !== 0,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapTask(row: ProjectTaskRow): ProjectTask {
  return {
    id: row.id,
    projectId: row.project_id,
    sectionId: row.section_id,
    statusId: row.status_id,
    parentTaskId: optionalText(row.parent_task_id),
    title: row.title,
    description: row.description,
    priority: row.priority,
    taskType: row.task_type,
    sectionSortOrder: row.section_sort_order,
    statusSortOrder: row.status_sort_order,
    estimateMinutes: optionalNumber(row.estimate_minutes),
    dueDate: optionalText(row.due_date),
    dueTime: optionalText(row.due_time),
    startDate: optionalText(row.start_date),
    startTime: optionalText(row.start_time),
    targetEndDate: optionalText(row.target_end_date),
    completedAt: optionalText(row.completed_at),
    archivedAt: optionalText(row.archived_at),
    blockerReason: optionalText(row.blocker_reason),
    milestone: row.milestone !== 0,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapChecklistItem(row: ProjectChecklistItemRow): ProjectChecklistItem {
  return {
    id: row.id,
    taskId: row.task_id,
    title: row.title,
    completedAt: optionalText(row.completed_at),
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapLabel(row: ProjectLabelRow): ProjectLabel {
  return {
    id: row.id,
    projectId: row.project_id,
    name: row.name,
    color: optionalNumber(row.color),
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapTaskLabelLink(row: ProjectTaskLabelLinkRow): ProjectTaskLabelLink {
  return {
    taskId: row.task_id,
    labelId: row.label_id,
    createdAt: row.created_at,
  };
}

function mapCustomField(row: ProjectCustomFieldRow): ProjectCustomField {
  return {
    id: row.id,
    projectId: row.project_id,
    name: row.name,
    fieldType: row.field_type,
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapCustomFieldOption(row: ProjectCustomFieldOptionRow): ProjectCustomFieldOption {
  return {
    id: row.id,
    fieldId: row.field_id,
    name: row.name,
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapCustomFieldValue(row: ProjectCustomFieldValueRow): ProjectCustomFieldValue {
  return {
    taskId: row.task_id,
    fieldId: row.field_id,
    textValue: optionalText(row.text_value),
    numberValue: optionalNumber(row.number_value),
    dateValue: optionalText(row.date_value),
    checkboxValue: row.checkbox_value === null ? undefined : row.checkbox_value !== 0,
    updatedAt: row.updated_at,
  };
}

function mapCustomFieldOptionValue(row: ProjectCustomFieldOptionValueRow): ProjectCustomFieldOptionValue {
  return {
    taskId: row.task_id,
    fieldId: row.field_id,
    optionId: row.option_id,
    createdAt: row.created_at,
  };
}

function mapDependency(row: ProjectTaskDependencyRow): ProjectTaskDependency {
  return {
    id: row.id,
    blockingTaskId: row.blocking_task_id,
    blockedTaskId: row.blocked_task_id,
    dependencyType: row.dependency_type,
    createdAt: row.created_at,
  };
}

function mapEventLink(row: ProjectTaskEventLinkRow): ProjectTaskEventLink {
  return {
    taskId: row.task_id,
    eventId: row.event_id,
    linkKind: row.link_kind,
    createdAt: row.created_at,
  };
}

function mapTaskChangeEvent(row: ProjectTaskChangeEventRow): ProjectTaskChangeEvent {
  return {
    id: row.id,
    taskId: row.task_id,
    eventType: row.event_type,
    fieldName: optionalText(row.field_name),
    oldValue: optionalText(row.old_value),
    newValue: optionalText(row.new_value),
    reason: optionalText(row.reason),
    occurredAt: row.occurred_at,
  };
}

function mapViewPreference(row: ProjectViewPreferenceRow): ProjectViewPreference {
  return {
    projectId: row.project_id,
    viewId: row.view_id,
    preferenceKey: row.preference_key,
    preferenceValue: row.preference_value,
    updatedAt: row.updated_at,
  };
}

function mapCustomEmoji(row: ProjectCustomEmojiRow): ProjectCustomEmoji {
  return {
    id: row.id,
    name: row.name,
    assetPath: row.asset_path,
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function mapLinkableEvent(row: ProjectLinkableEventRow): ProjectLinkableEvent {
  const renderZone = localTimezone();
  const allDay = row.all_day !== 0;
  const color = normalizeEventColor(row.color);
  return {
    id: row.id,
    projectId: row.project_id,
    title: row.title,
    start: toCalendarDate(row.start_time, renderZone, allDay),
    end: toCalendarDate(row.end_time, renderZone, allDay),
    timezone: row.timezone,
    calendarId: row.calendar_id,
    color,
    allDay,
    status: row.status,
    linkedTasks: row.linked_tasks.map((task) => ({
      taskId: task.task_id,
      title: task.title,
      archivedAt: optionalText(task.archived_at),
    })),
  };
}

function mapSnapshot(rows: ProjectsSnapshotRows): ProjectsSnapshot {
  return {
    groups: rows.groups.map(mapGroup),
    projects: rows.projects.map(mapProject),
    sections: rows.sections.map(mapSection),
    statuses: rows.statuses.map(mapStatus),
    tasks: rows.tasks.map(mapTask),
    checklistItems: rows.checklist_items.map(mapChecklistItem),
    labels: rows.labels.map(mapLabel),
    taskLabelLinks: rows.task_label_links.map(mapTaskLabelLink),
    customFields: rows.custom_fields.map(mapCustomField),
    customFieldOptions: rows.custom_field_options.map(mapCustomFieldOption),
    customFieldValues: rows.custom_field_values.map(mapCustomFieldValue),
    customFieldOptionValues: rows.custom_field_option_values.map(mapCustomFieldOptionValue),
    dependencies: rows.dependencies.map(mapDependency),
    eventLinks: rows.event_links.map(mapEventLink),
    taskChangeEvents: rows.task_change_events.map(mapTaskChangeEvent),
    viewPreferences: rows.view_preferences.map(mapViewPreference),
    customEmojis: rows.custom_emojis.map(mapCustomEmoji),
  };
}

export async function loadProjectsSnapshot(projectId?: string | null): Promise<ProjectsSnapshot> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<ProjectsSnapshotRows>("projects_load_snapshot", { dbUrl, projectId: projectId ?? null });
  return mapSnapshot(rows);
}

export async function createProjectGroup(group: ProjectGroupCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_group", { dbUrl, group });
}

export async function updateProjectGroup(group: ProjectGroupUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_group", { dbUrl, group });
}

export async function deleteProjectGroup(groupId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_group", { dbUrl, groupId });
}

export async function setProjectGroupCollapsed(
  groupId: string,
  collapsed: boolean,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_set_group_collapsed", { dbUrl, groupId, collapsed });
}

export async function createProject(project: ProjectCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_project", { dbUrl, project });
}

export async function updateProject(project: ProjectUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_project", { dbUrl, project });
}

export async function createProjectSection(section: ProjectSectionCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_section", { dbUrl, section });
}

export async function updateProjectSection(section: ProjectSectionUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_section", { dbUrl, section });
}

export async function createProjectStatus(status: ProjectStatusCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_status", { dbUrl, status });
}

export async function updateProjectStatus(status: ProjectStatusUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_status", { dbUrl, status });
}

export async function deleteProjectStatus(statusId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_status", { dbUrl, statusId });
}

export async function createProjectTask(task: ProjectTaskCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_task", { dbUrl, task });
}

export async function createProjectChecklistItem(item: ProjectChecklistItemCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_checklist_item", { dbUrl, item });
}

export async function updateProjectChecklistItem(item: ProjectChecklistItemUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_checklist_item", { dbUrl, item });
}

export async function deleteProjectChecklistItem(itemId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_checklist_item", { dbUrl, itemId });
}

export async function createProjectLabel(label: ProjectLabelCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_label", { dbUrl, label });
}

export async function updateProjectLabel(label: ProjectLabelUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_label", { dbUrl, label });
}

export async function deleteProjectLabel(labelId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_label", { dbUrl, labelId });
}

export async function linkProjectTaskLabel(link: ProjectTaskLabelLinkCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_link_task_label", { dbUrl, link });
}

export async function unlinkProjectTaskLabel(taskId: string, labelId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_unlink_task_label", { dbUrl, taskId, labelId });
}

export async function createProjectCustomField(field: ProjectCustomFieldCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_custom_field", { dbUrl, field });
}

export async function updateProjectCustomField(field: ProjectCustomFieldUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_custom_field", { dbUrl, field });
}

export async function deleteProjectCustomField(fieldId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_custom_field", { dbUrl, fieldId });
}

export async function createProjectCustomFieldOption(option: ProjectCustomFieldOptionCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_custom_field_option", { dbUrl, option });
}

export async function updateProjectCustomFieldOption(option: ProjectCustomFieldOptionUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_custom_field_option", { dbUrl, option });
}

export async function deleteProjectCustomFieldOption(optionId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_custom_field_option", { dbUrl, optionId });
}

export async function updateProjectCustomFieldValue(value: ProjectCustomFieldValueUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_custom_field_value", { dbUrl, value });
}

export async function updateProjectTask(task: ProjectTaskUpdate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_update_task", { dbUrl, task });
}

export async function linkProjectTaskEvent(link: ProjectTaskEventLinkCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_link_task_event", { dbUrl, link });
}

export async function unlinkProjectTaskEvent(taskId: string, eventId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_unlink_task_event", { dbUrl, taskId, eventId });
}

export async function searchProjectLinkableEvents(
  projectId: string,
  taskId: string,
  query: string,
  startDate?: string,
  endDate?: string,
  limit = 12,
): Promise<ProjectLinkableEvent[]> {
  const dbUrl = await ensureDbUrl();
  const rows = await invoke<ProjectLinkableEventRow[]>("projects_search_linkable_events", {
    dbUrl,
    search: {
      projectId,
      taskId,
      query,
      startDate,
      endDate,
      limit,
    },
  });
  return rows.map(mapLinkableEvent);
}

export async function createProjectTaskDependency(
  dependency: ProjectTaskDependencyCreate,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_task_dependency", { dbUrl, dependency });
}

export async function deleteProjectTaskDependency(dependencyId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_task_dependency", { dbUrl, dependencyId });
}

export async function upsertProjectViewPreference(
  preference: ProjectViewPreferenceUpsert,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_upsert_view_preference", { dbUrl, preference });
}

export async function deleteProjectViewPreference(
  projectId: string,
  viewId: ProjectViewPreference["viewId"],
  preferenceKey: string,
): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_view_preference", { dbUrl, projectId, viewId, preferenceKey });
}

export async function createProjectCustomEmoji(emoji: ProjectCustomEmojiCreate): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_create_custom_emoji", { dbUrl, emoji });
}

export async function deleteProjectCustomEmoji(emojiId: string): Promise<void> {
  const dbUrl = await ensureDbUrl();
  await invoke("projects_delete_custom_emoji", { dbUrl, emojiId });
}
