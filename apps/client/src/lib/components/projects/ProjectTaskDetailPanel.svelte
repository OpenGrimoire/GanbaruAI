<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Check from "@lucide/svelte/icons/check";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import { getEventColor } from "$lib/components/calendar/utils";
  import type { CalendarEvent, EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    PROJECT_PRIORITIES,
    PROJECT_TASK_TYPES,
  } from "$lib/projects/types";
  import type {
    ProjectChecklistItem,
    ProjectCustomField,
    ProjectCustomFieldOption,
    ProjectCustomFieldType,
    ProjectLabel,
    ProjectLinkableEvent,
    ProjectPriority,
    ProjectSection,
    ProjectStatus,
    ProjectTask,
    ProjectTaskChangeEvent,
    ProjectTaskType,
  } from "$lib/projects/types";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";

  let {
    taskId,
    showArchivedTasks,
    showInactiveSections,
    onClose,
    onOpenTask,
    onShowArchivedTasks,
  }: {
    taskId: string;
    showArchivedTasks: boolean;
    showInactiveSections: boolean;
    onClose: () => void;
    onOpenTask: (taskId: string) => void;
    onShowArchivedTasks: () => void;
  } = $props();

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const { t } = getLocalization();

  let detailDraftTaskId = $state<string | null>(null);
  let detailDraftUpdatedAt = $state<string | null>(null);
  let detailTitle = $state("");
  let detailDescription = $state("");
  let detailSectionId = $state("");
  let detailStatusId = $state("");
  let detailPriority = $state<ProjectPriority>("normal");
  let detailTaskType = $state<ProjectTaskType>("task");
  let detailEstimateMinutes = $state("");
  let detailStartDate = $state("");
  let detailDueDate = $state("");
  let detailTargetEndDate = $state("");
  let detailBlockerReason = $state("");
  let detailChangeReason = $state("");
  let detailMilestone = $state(false);
  let detailSaving = $state(false);
  let detailError = $state<string | null>(null);
  let subtaskDraft = $state("");
  let checklistDraft = $state("");
  let labelDraft = $state("");
  let checklistTitleDrafts = $state<Record<string, string>>({});
  let customFieldTextDrafts = $state<Record<string, string>>({});
  let customFieldNumberDrafts = $state<Record<string, string>>({});
  let customFieldDateDrafts = $state<Record<string, string>>({});
  let customFieldCheckboxDrafts = $state<Record<string, boolean>>({});
  let customFieldSelectDrafts = $state<Record<string, string>>({});
  let customFieldMultiDrafts = $state<Record<string, string[]>>({});
  let dependencySearch = $state("");
  let parentTaskSearch = $state("");
  let eventLinkSearch = $state("");
  let eventLinkStartDate = $state("");
  let eventLinkEndDate = $state("");
  let eventLinkResults = $state<ProjectLinkableEvent[]>([]);
  let eventLinkSearchPending = $state(false);
  let eventLinkSearchError = $state<string | null>(null);
  let eventLinkSearchRunId = 0;

  const selectedTask = $derived(projects.taskById(taskId));
  const selectedProjectId = $derived(selectedTask?.projectId ?? null);
  const allProjectSections = $derived(projects.sectionsForProjectIncludingInactive(selectedProjectId));
  const sections = $derived.by(() =>
    showInactiveSections ? allProjectSections : projects.sectionsForProject(selectedProjectId)
  );
  const visibleSectionIds = $derived.by(() => new Set(sections.map((section) => section.id)));
  const statuses = $derived(projects.statusesForProject(selectedProjectId));
  const activeProjectTasks = $derived.by(() =>
    projects.tasksForProject(selectedProjectId).filter((task) => visibleSectionIds.has(task.sectionId))
  );
  const allProjectTasksWithArchived = $derived.by(() =>
    projects.tasksForProjectIncludingArchived(selectedProjectId)
      .filter((task) => visibleSectionIds.has(task.sectionId))
  );
  const allProjectTasks = $derived(showArchivedTasks ? allProjectTasksWithArchived : activeProjectTasks);
  const projectCustomFields = $derived(projects.customFieldsForProject(selectedProjectId));
  const allProjectEvents = $derived.by(() => {
    if (!selectedProjectId) return [];
    return calendar.rawBlocks
      .filter((event) => event.projectId === selectedProjectId)
      .sort((a, b) => a.start.localeCompare(b.start));
  });
  const detailDirty = $derived.by(() => {
    if (!selectedTask) return false;
    return detailTitle !== selectedTask.title
      || detailDescription !== selectedTask.description
      || detailSectionId !== selectedTask.sectionId
      || detailStatusId !== selectedTask.statusId
      || detailPriority !== selectedTask.priority
      || detailTaskType !== selectedTask.taskType
      || detailEstimateMinutes !== String(selectedTask.estimateMinutes ?? "")
      || detailStartDate !== (selectedTask.startDate ?? "")
      || detailDueDate !== (selectedTask.dueDate ?? "")
      || detailTargetEndDate !== (selectedTask.targetEndDate ?? "")
      || detailBlockerReason !== (selectedTask.blockerReason ?? "")
      || detailMilestone !== selectedTask.milestone;
  });

  $effect(() => {
    if (!selectedTask) return;
    if (
      detailDraftTaskId !== selectedTask.id
      || (!detailDirty && detailDraftUpdatedAt !== selectedTask.updatedAt)
    ) {
      loadTaskDetailDraft(selectedTask);
    }
  });

  $effect(() => {
    const task = selectedTask;
    const projectId = selectedProjectId;
    const query = eventLinkSearch;
    const startDateDraft = eventLinkStartDate;
    const endDateDraft = eventLinkEndDate;
    if (!task || !projectId) {
      eventLinkResults = [];
      eventLinkSearchPending = false;
      eventLinkSearchError = null;
      return;
    }

    let startDate: string | undefined;
    let endDate: string | undefined;
    try {
      startDate = normalizeOptionalDate(startDateDraft);
      endDate = normalizeOptionalDate(endDateDraft);
      if (startDate && endDate && startDate > endDate) {
        throw new Error(t("projects.detail.eventLinkInvalidDateRange"));
      }
    } catch (error) {
      eventLinkSearchRunId++;
      eventLinkSearchPending = false;
      eventLinkSearchError = error instanceof Error ? error.message : String(error);
      eventLinkResults = [];
      return;
    }

    const runId = ++eventLinkSearchRunId;
    eventLinkSearchPending = true;
    eventLinkSearchError = null;
    const timeoutId = setTimeout(() => {
      void projects.searchLinkableEvents(projectId, task.id, query, startDate, endDate, 12)
        .then((results) => {
          if (runId !== eventLinkSearchRunId) return;
          eventLinkResults = results;
        })
        .catch((error) => {
          if (runId !== eventLinkSearchRunId) return;
          eventLinkSearchError = t(
            "projects.detail.eventLinkSearchFailed",
            error instanceof Error ? error.message : String(error),
          );
          eventLinkResults = [];
        })
        .finally(() => {
          if (runId === eventLinkSearchRunId) {
            eventLinkSearchPending = false;
          }
        });
    }, query.trim() ? 150 : 0);

    return () => {
      clearTimeout(timeoutId);
    };
  });

  function statusForTask(task: ProjectTask): ProjectStatus | undefined {
    return projects.statusById(task.statusId);
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return allProjectTasks.find((task) => task.id === taskId);
  }

  function blockedByDependencies(task: ProjectTask) {
    return projects.dependenciesBlockingTask(task.id);
  }

  function blocksDependencies(task: ProjectTask) {
    return projects.dependenciesBlockedByTask(task.id);
  }

  function dependencyCandidateTasks(task: ProjectTask): ProjectTask[] {
    const existingBlockingTaskIds = new Set(
      blockedByDependencies(task).map((dependency) => dependency.blockingTaskId),
    );
    const query = dependencySearch.trim().toLowerCase();
    return allProjectTasks
      .filter((candidate) =>
        candidate.id !== task.id
        && !existingBlockingTaskIds.has(candidate.id)
        && (!query
          || candidate.title.toLowerCase().includes(query)
          || candidate.description.toLowerCase().includes(query))
      )
      .slice(0, 8);
  }

  function taskHasAnySubtasks(task: ProjectTask): boolean {
    return projects.subtasksForTaskIncludingArchived(task.id).length > 0;
  }

  function parentTaskCandidateTasks(task: ProjectTask): ProjectTask[] {
    const query = parentTaskSearch.trim().toLowerCase();
    return activeProjectTasks
      .filter((candidate) =>
        candidate.projectId === task.projectId
        && !candidate.parentTaskId
        && candidate.id !== task.id
        && (!query || candidate.title.toLowerCase().includes(query))
      )
      .sort((a, b) => a.sectionSortOrder - b.sectionSortOrder || a.title.localeCompare(b.title))
      .slice(0, 8);
  }

  function eventLinkCandidateEvents(task: ProjectTask): ProjectLinkableEvent[] {
    const linkedEventIds = new Set(projects.eventLinksForTask(task.id).map((link) => link.eventId));
    return eventLinkResults
      .filter((event) => !linkedEventIds.has(event.id))
      .slice(0, 8);
  }

  function labelsForTask(task: ProjectTask): ProjectLabel[] {
    return projects.labelsForTask(task.id);
  }

  function labelCandidateLabels(task: ProjectTask): ProjectLabel[] {
    const query = labelDraft.trim().toLowerCase();
    return projects.unlinkedLabelsForTask(task)
      .filter((label) => !query || label.name.toLowerCase().includes(query))
      .slice(0, 8);
  }

  function canCreateLabel(task: ProjectTask): boolean {
    const name = labelDraft.trim();
    if (!name) return false;
    return !projects.labelsForProject(task.projectId)
      .some((label) => label.name.trim().toLowerCase() === name.toLowerCase());
  }

  function priorityLabel(priority: ProjectPriority): string {
    if (priority === "low") return t("projects.priority.low");
    if (priority === "high") return t("projects.priority.high");
    if (priority === "urgent") return t("projects.priority.urgent");
    return t("projects.priority.normal");
  }

  function taskTypeLabel(taskType: ProjectTaskType): string {
    if (taskType === "bug") return t("projects.taskType.bug");
    if (taskType === "habit") return t("projects.taskType.habit");
    if (taskType === "milestone") return t("projects.taskType.milestone");
    return t("projects.taskType.task");
  }

  function taskArchivedBadgeClass(task: ProjectTask): string {
    return task.archivedAt
      ? "border-muted-foreground/30 bg-muted/50 text-muted-foreground"
      : "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
  }

  function statusBadgeClass(status: ProjectStatus | undefined): string {
    if (status?.category === "done") return "border-emerald-500/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300";
    if (status?.category === "blocked") return "border-destructive/40 bg-destructive/10 text-destructive";
    if (status?.category === "active") return "border-sky-500/40 bg-sky-500/10 text-sky-700 dark:text-sky-300";
    return "border-border bg-muted/50 text-muted-foreground";
  }

  function priorityClass(priority: ProjectPriority): string {
    if (priority === "urgent") return "border-destructive/40 bg-destructive/10 text-destructive";
    if (priority === "high") return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
    if (priority === "low") return "border-muted-foreground/20 bg-muted/30 text-muted-foreground";
    return "border-border bg-background/70 text-foreground";
  }

  function customFieldTypeLabel(fieldType: ProjectCustomFieldType): string {
    if (fieldType === "number") return t("projects.customFields.typeNumber");
    if (fieldType === "date") return t("projects.customFields.typeDate");
    if (fieldType === "select") return t("projects.customFields.typeSelect");
    if (fieldType === "multi_select") return t("projects.customFields.typeMultiSelect");
    if (fieldType === "checkbox") return t("projects.customFields.typeCheckbox");
    if (fieldType === "url") return t("projects.customFields.typeUrl");
    return t("projects.customFields.typeText");
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldValueDirty(task: ProjectTask, field: ProjectCustomField): boolean {
    const value = projects.customFieldValueForTask(task.id, field.id);
    const optionValues = projects.customFieldOptionValuesForTask(task.id, field.id);
    if (field.fieldType === "text" || field.fieldType === "url") {
      return (customFieldTextDrafts[field.id] ?? "") !== (value?.textValue ?? "");
    }
    if (field.fieldType === "number") {
      return (customFieldNumberDrafts[field.id] ?? "") !== String(value?.numberValue ?? "");
    }
    if (field.fieldType === "date") {
      return (customFieldDateDrafts[field.id] ?? "") !== (value?.dateValue ?? "");
    }
    if (field.fieldType === "checkbox") {
      return (customFieldCheckboxDrafts[field.id] ?? false) !== (value?.checkboxValue ?? false);
    }
    if (field.fieldType === "select") {
      return (customFieldSelectDrafts[field.id] ?? "none") !== (optionValues[0]?.id ?? "none");
    }
    const currentIds = optionValues.map((option) => option.id).sort();
    const draftIds = [...(customFieldMultiDrafts[field.id] ?? [])].sort();
    return currentIds.join("\u0000") !== draftIds.join("\u0000");
  }

  function labelColorDotStyle(color: EventColor | undefined): string {
    if (color === undefined) return "";
    return `background-color: ${getEventColor(color, theme.current).bg};`;
  }

  function labelColorSwatchClass(color: EventColor | undefined): string {
    return color === undefined ? "border-border bg-muted/50" : "border-transparent";
  }

  function linkedEventRowsForTask(task: ProjectTask): ProjectLinkableEvent[] {
    const linkedEventIds = new Set(projects.eventLinksForTask(task.id).map((link) => link.eventId));
    const rowsById = new Map(eventLinkResults.map((event) => [event.id, event]));
    for (const event of allProjectEvents) {
      if (linkedEventIds.has(event.id) && !rowsById.has(event.id)) {
        rowsById.set(event.id, calendarEventToLinkableEvent(event));
      }
    }
    return Array.from(rowsById.values())
      .filter((event) => linkedEventIds.has(event.id))
      .sort((a, b) => a.start.localeCompare(b.start));
  }

  function calendarEventToLinkableEvent(event: CalendarEvent): ProjectLinkableEvent {
    return {
      id: event.id,
      projectId: event.projectId ?? "",
      title: event.title,
      start: event.start,
      end: event.end,
      timezone: event.timezone,
      calendarId: event.calendarId,
      color: event.color,
      allDay: event.allDay === true,
      status: event.status ?? "confirmed",
      linkedTasks: [],
    };
  }

  function otherLinkedTaskText(event: ProjectLinkableEvent, task: ProjectTask): string {
    return event.linkedTasks
      .filter((linkedTask) => linkedTask.taskId !== task.id)
      .map((linkedTask) => linkedTask.title)
      .join(", ");
  }

  function historyEventLabel(event: ProjectTaskChangeEvent): string {
    const field = event.fieldName ? historyFieldLabel(event.fieldName) : "";
    const oldValue = event.oldValue ?? t("projects.history.emptyValue");
    const newValue = event.newValue ?? t("projects.history.emptyValue");
    if (event.eventType === "created") return t("projects.history.created");
    if (event.eventType === "scheduled") return t("projects.history.scheduled");
    if (event.eventType === "event_unlinked") return t("projects.history.eventUnlinked");
    if (event.eventType === "dependency_added") return t("projects.history.dependencyAdded", newValue);
    if (event.eventType === "dependency_removed") return t("projects.history.dependencyRemoved", oldValue);
    if (event.eventType === "completed") return t("projects.history.completed", oldValue, newValue);
    if (event.eventType === "reopened") return t("projects.history.reopened", oldValue, newValue);
    if (event.eventType === "archived") return t("projects.history.archived");
    if (event.fieldName) return t("projects.history.fieldChanged", field, oldValue, newValue);
    return t("projects.history.updated");
  }

  function historyFieldLabel(fieldName: string): string {
    if (fieldName.startsWith("custom_field:")) return fieldName.slice("custom_field:".length);
    if (fieldName === "title") return t("projects.history.fields.title");
    if (fieldName === "description") return t("projects.history.fields.description");
    if (fieldName === "status") return t("projects.history.fields.status");
    if (fieldName === "section") return t("projects.history.fields.section");
    if (fieldName === "parent") return t("projects.history.fields.parent");
    if (fieldName === "priority") return t("projects.history.fields.priority");
    if (fieldName === "type") return t("projects.history.fields.type");
    if (fieldName === "estimate") return t("projects.history.fields.estimate");
    if (fieldName === "due_date") return t("projects.history.fields.dueDate");
    if (fieldName === "start_date") return t("projects.history.fields.startDate");
    if (fieldName === "target_date") return t("projects.history.fields.targetDate");
    if (fieldName === "archived_at") return t("projects.history.fields.archiveState");
    if (fieldName === "blocker_reason") return t("projects.history.fields.blockerReason");
    if (fieldName === "milestone") return t("projects.history.fields.milestone");
    if (fieldName === "checklist") return t("projects.history.fields.checklist");
    if (fieldName === "event_id") return t("projects.history.fields.event");
    if (fieldName === "blocking_task_id") return t("projects.history.fields.dependency");
    return fieldName;
  }

  function checklistItemDraftTitle(item: ProjectChecklistItem): string {
    return checklistTitleDrafts[item.id] ?? item.title;
  }

  function checklistItemDirty(item: ProjectChecklistItem): boolean {
    return checklistItemDraftTitle(item) !== item.title;
  }

  function adjacentChecklistItem(item: ProjectChecklistItem, direction: -1 | 1): ProjectChecklistItem | undefined {
    const ordered = projects.checklistItemsForTask(item.taskId);
    const index = ordered.findIndex((entry) => entry.id === item.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function subtasksForTask(parent: ProjectTask): ProjectTask[] {
    return showArchivedTasks
      ? projects.subtasksForTaskIncludingArchived(parent.id)
      : projects.subtasksForTask(parent.id);
  }

  function adjacentSubtask(task: ProjectTask, direction: -1 | 1): ProjectTask | undefined {
    if (!task.parentTaskId) return undefined;
    const ordered = projects.subtasksForTask(task.parentTaskId);
    const index = ordered.findIndex((entry) => entry.id === task.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function sectionForTask(task: ProjectTask): ProjectSection | undefined {
    return sections.find((section) => section.id === task.sectionId);
  }

  function loadTaskDetailDraft(task: ProjectTask): void {
    detailDraftTaskId = task.id;
    detailDraftUpdatedAt = task.updatedAt;
    detailTitle = task.title;
    detailDescription = task.description;
    detailSectionId = task.sectionId;
    detailStatusId = task.statusId;
    detailPriority = task.priority;
    detailTaskType = task.taskType;
    detailEstimateMinutes = String(task.estimateMinutes ?? "");
    detailStartDate = task.startDate ?? "";
    detailDueDate = task.dueDate ?? "";
    detailTargetEndDate = task.targetEndDate ?? "";
    detailBlockerReason = task.blockerReason ?? "";
    detailChangeReason = "";
    detailMilestone = task.milestone;
    detailError = null;
    subtaskDraft = "";
    checklistDraft = "";
    labelDraft = "";
    checklistTitleDrafts = Object.fromEntries(
      projects.checklistItemsForTask(task.id).map((item) => [item.id, item.title]),
    );
    const textDrafts: Record<string, string> = {};
    const numberDrafts: Record<string, string> = {};
    const dateDrafts: Record<string, string> = {};
    const checkboxDrafts: Record<string, boolean> = {};
    const selectDrafts: Record<string, string> = {};
    const multiDrafts: Record<string, string[]> = {};
    for (const field of projects.customFieldsForProject(task.projectId)) {
      const value = projects.customFieldValueForTask(task.id, field.id);
      const optionValues = projects.customFieldOptionValuesForTask(task.id, field.id);
      textDrafts[field.id] = value?.textValue ?? "";
      numberDrafts[field.id] = String(value?.numberValue ?? "");
      dateDrafts[field.id] = value?.dateValue ?? "";
      checkboxDrafts[field.id] = value?.checkboxValue ?? false;
      selectDrafts[field.id] = optionValues[0]?.id ?? "none";
      multiDrafts[field.id] = optionValues.map((option) => option.id);
    }
    customFieldTextDrafts = textDrafts;
    customFieldNumberDrafts = numberDrafts;
    customFieldDateDrafts = dateDrafts;
    customFieldCheckboxDrafts = checkboxDrafts;
    customFieldSelectDrafts = selectDrafts;
    customFieldMultiDrafts = multiDrafts;
    dependencySearch = "";
    parentTaskSearch = "";
    eventLinkSearch = "";
    eventLinkStartDate = "";
    eventLinkEndDate = "";
    eventLinkResults = [];
    eventLinkSearchError = null;
  }

  function closeTaskDetail(): void {
    if (selectedTask) loadTaskDetailDraft(selectedTask);
    onClose();
  }

  function openTaskDetail(task: ProjectTask): void {
    onOpenTask(task.id);
  }

  function normalizeOptionalDate(value: string): string | undefined {
    const trimmed = value.trim();
    if (!trimmed) return undefined;
    try {
      return Temporal.PlainDate.from(trimmed).toString();
    } catch {
      throw new Error(t("projects.detail.invalidDate"));
    }
  }

  function normalizeOptionalPositiveInteger(value: string): number | undefined {
    const trimmed = value.trim();
    if (!trimmed) return undefined;
    const parsed = Number(trimmed);
    if (!Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(t("projects.detail.invalidEstimate"));
    }
    return parsed;
  }

  function toggleCustomFieldMultiOption(field: ProjectCustomField, option: ProjectCustomFieldOption): void {
    const current = customFieldMultiDrafts[field.id] ?? [];
    customFieldMultiDrafts = {
      ...customFieldMultiDrafts,
      [field.id]: current.includes(option.id)
        ? current.filter((optionId) => optionId !== option.id)
        : [...current, option.id],
    };
  }

  async function saveTaskCustomField(task: ProjectTask, field: ProjectCustomField): Promise<void> {
    detailError = null;
    try {
      let textValue: string | null = null;
      let numberValue: number | null = null;
      let dateValue: string | null = null;
      let checkboxValue: boolean | null = null;
      let optionIds: string[] = [];
      if (field.fieldType === "text" || field.fieldType === "url") {
        const text = (customFieldTextDrafts[field.id] ?? "").trim();
        textValue = text || null;
      } else if (field.fieldType === "number") {
        const numberText = (customFieldNumberDrafts[field.id] ?? "").trim();
        if (numberText) {
          const parsed = Number(numberText);
          if (!Number.isFinite(parsed)) {
            detailError = t("projects.customFields.invalidNumber");
            return;
          }
          numberValue = parsed;
        }
      } else if (field.fieldType === "date") {
        const dateText = (customFieldDateDrafts[field.id] ?? "").trim();
        if (dateText) {
          try {
            dateValue = Temporal.PlainDate.from(dateText).toString();
          } catch {
            detailError = t("projects.detail.invalidDate");
            return;
          }
        }
      } else if (field.fieldType === "checkbox") {
        checkboxValue = customFieldCheckboxDrafts[field.id] ?? false;
      } else if (field.fieldType === "select") {
        const optionId = customFieldSelectDrafts[field.id] ?? "none";
        optionIds = optionId === "none" ? [] : [optionId];
      } else {
        optionIds = customFieldMultiDrafts[field.id] ?? [];
      }
      await projects.saveCustomFieldValue({
        taskId: task.id,
        fieldId: field.id,
        textValue,
        numberValue,
        dateValue,
        checkboxValue,
        optionIds,
      });
      loadTaskDetailDraft(task);
    } catch (error) {
      detailError = t(
        "projects.customFields.valueSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitSubtask(parent: ProjectTask): Promise<void> {
    const statusId = projects.defaultStatus(parent.projectId)?.id ?? parent.statusId;
    await projects.addTask(parent.projectId, subtaskDraft, parent.sectionId, statusId, parent.id);
    subtaskDraft = "";
  }

  async function submitChecklistItem(task: ProjectTask): Promise<void> {
    await projects.addChecklistItem(task.id, checklistDraft);
    checklistDraft = "";
  }

  async function saveChecklistItem(item: ProjectChecklistItem): Promise<void> {
    const title = checklistItemDraftTitle(item).trim();
    if (!title) {
      detailError = t("projects.detail.checklistItemTitleRequired");
      return;
    }
    detailError = null;
    await projects.updateChecklistItem(item, { title });
    checklistTitleDrafts = { ...checklistTitleDrafts, [item.id]: title };
  }

  async function attachExistingLabel(task: ProjectTask, label: ProjectLabel): Promise<void> {
    detailError = null;
    try {
      await projects.linkTaskLabel(task.id, label.id);
      labelDraft = "";
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitTaskLabel(task: ProjectTask): Promise<void> {
    const name = labelDraft.trim();
    if (!name) {
      detailError = t("projects.detail.labelNameRequired");
      return;
    }
    detailError = null;
    try {
      await projects.addAndLinkTaskLabel(task, name);
      labelDraft = "";
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function detachTaskLabel(task: ProjectTask, label: ProjectLabel): Promise<void> {
    detailError = null;
    try {
      await projects.unlinkTaskLabel(task.id, label.id);
    } catch (error) {
      detailError = t(
        "projects.detail.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveChecklistItemInDetail(item: ProjectChecklistItem, direction: -1 | 1): Promise<void> {
    detailError = null;
    await projects.moveChecklistItem(item, direction);
  }

  async function moveSubtaskInDetail(task: ProjectTask, direction: -1 | 1): Promise<void> {
    detailError = null;
    await projects.moveSubtask(task, direction);
  }

  async function promoteSubtaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.promoteSubtask(task);
    } catch (error) {
      detailError = t(
        "projects.detail.promoteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function demoteTaskFromDetail(task: ProjectTask, parentTask: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.demoteTaskToSubtask(task, parentTask);
      parentTaskSearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.demoteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function addBlockingDependency(blockingTask: ProjectTask, blockedTask: ProjectTask): Promise<void> {
    detailError = null;
    try {
      await projects.addTaskDependency(blockingTask.id, blockedTask.id);
      dependencySearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.dependencySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function removeDependency(dependencyId: string): Promise<void> {
    detailError = null;
    try {
      await projects.removeTaskDependency(dependencyId);
    } catch (error) {
      detailError = t(
        "projects.detail.dependencySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function linkExistingEvent(task: ProjectTask, event: ProjectLinkableEvent): Promise<void> {
    detailError = null;
    try {
      await projects.linkTaskEvent(task.id, event.id, "scheduled");
      eventLinkSearch = "";
    } catch (error) {
      detailError = t(
        "projects.detail.eventLinkSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function unlinkExistingEvent(task: ProjectTask, event: ProjectLinkableEvent): Promise<void> {
    detailError = null;
    try {
      await projects.unlinkTaskEvent(task.id, event.id);
    } catch (error) {
      detailError = t(
        "projects.detail.eventLinkSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function archiveTaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    detailSaving = true;
    try {
      await projects.archiveTasks([task]);
    } catch (error) {
      detailError = t("projects.detail.archiveFailed", error instanceof Error ? error.message : String(error));
    } finally {
      detailSaving = false;
    }
  }

  async function restoreTaskFromDetail(task: ProjectTask): Promise<void> {
    detailError = null;
    detailSaving = true;
    onShowArchivedTasks();
    try {
      await projects.restoreTasks([task]);
    } catch (error) {
      detailError = t("projects.detail.restoreFailed", error instanceof Error ? error.message : String(error));
    } finally {
      detailSaving = false;
    }
  }

  async function saveTaskDetail(): Promise<void> {
    if (!selectedTask) return;
    const title = detailTitle.trim();
    if (!title) {
      detailError = t("projects.detail.titleRequired");
      return;
    }
    if (!detailSectionId || !detailStatusId) return;
    detailSaving = true;
    detailError = null;
    try {
      const estimateMinutes = normalizeOptionalPositiveInteger(detailEstimateMinutes);
      const startDate = normalizeOptionalDate(detailStartDate);
      const dueDate = normalizeOptionalDate(detailDueDate);
      const targetEndDate = normalizeOptionalDate(detailTargetEndDate);
      await projects.updateTask(selectedTask, {
        title,
        description: detailDescription.trim(),
        sectionId: detailSectionId,
        statusId: detailStatusId,
        priority: detailPriority,
        taskType: detailTaskType,
        estimateMinutes,
        startDate,
        dueDate,
        targetEndDate,
        blockerReason: detailBlockerReason.trim() || undefined,
        milestone: detailMilestone,
        changeReason: detailChangeReason.trim() || null,
      });
      detailChangeReason = "";
      detailDraftUpdatedAt = selectedTask.updatedAt;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      detailError = message === t("projects.detail.invalidEstimate")
        || message === t("projects.detail.invalidDate")
        ? message
        : t("projects.detail.saveFailed", message);
    } finally {
      detailSaving = false;
    }
  }
</script>

{#if selectedTask}
    {@const selectedTaskStatus = statusForTask(selectedTask)}
    {@const selectedTaskSection = sectionForTask(selectedTask)}
    {@const selectedTaskEvents = linkedEventRowsForTask(selectedTask)}
    {@const selectedTaskHistory = projects.taskChangeEventsForTask(selectedTask.id).slice(0, 8)}
    {@const selectedTaskChecklist = projects.checklistItemsForTask(selectedTask.id)}
    {@const selectedTaskLabels = labelsForTask(selectedTask)}
    {@const selectedTaskLabelCandidates = labelCandidateLabels(selectedTask)}
    {@const selectedTaskSubtasks = subtasksForTask(selectedTask)}
    {@const selectedTaskBlockedBy = blockedByDependencies(selectedTask)}
    {@const selectedTaskBlocks = blocksDependencies(selectedTask)}
    {@const selectedTaskParent = selectedTask.parentTaskId ? taskById(selectedTask.parentTaskId) : undefined}
    {@const selectedTaskDependencyCandidates = dependencyCandidateTasks(selectedTask)}
    {@const selectedTaskEventCandidates = eventLinkCandidateEvents(selectedTask)}
    <aside class="flex min-h-0 w-[min(23rem,42vw)] min-w-64 shrink-0 flex-col border-l border-border bg-card max-[760px]:fixed max-[760px]:inset-2 max-[760px]:z-30 max-[760px]:w-auto max-[760px]:rounded-md max-[760px]:border">
      <header class="flex shrink-0 items-center gap-2 border-b border-border px-3 py-2">
        <div class="min-w-0 flex-1">
          <div class="truncate text-[0.933333rem] font-semibold">{t("projects.detail.title")}</div>
          <div class="flex min-w-0 items-center gap-1.5">
            <span class="truncate text-[0.733333rem] text-muted-foreground">
              {selectedTaskSection?.name ?? t("projects.list.general")} / {selectedTaskStatus?.name ?? t("projects.list.status")}
            </span>
            {#if selectedTask.archivedAt}
              <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", taskArchivedBadgeClass(selectedTask))}>
                {t("projects.taskLifecycle.archived")}
              </span>
            {/if}
          </div>
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.detail.discard")}
          title={t("projects.detail.discard")}
          disabled={!detailDirty}
          onclick={() => loadTaskDetailDraft(selectedTask)}
        >
          <RotateCcw size={14} strokeWidth={1.75} />
        </button>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.detail.close")}
          title={t("projects.detail.close")}
          onclick={closeTaskDetail}
        >
          <X size={15} strokeWidth={1.75} />
        </button>
      </header>

      <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void saveTaskDetail(); }}>
        <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
          <div class="grid gap-3">
            <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.detail.titleLabel")}</span>
              <input
                bind:value={detailTitle}
                class="min-h-9 rounded-md border border-border bg-background px-2 text-[0.9rem] font-medium text-foreground"
              />
            </label>

            <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.detail.description")}</span>
              <textarea
                bind:value={detailDescription}
                rows="5"
                class="min-h-28 resize-none rounded-md border border-border bg-background px-2 py-2 text-[0.833333rem] text-foreground"
              ></textarea>
            </label>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.properties")}</h2>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.status")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each statuses as status (status.id)}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailStatusId === status.id
                          ? statusBadgeClass(status)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailStatusId = status.id;
                      }}
                    >
                      {status.name}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.section")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each sections as section (section.id)}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailSectionId === section.id
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailSectionId = section.id;
                      }}
                    >
                      {section.name}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.priority")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_PRIORITIES as priority}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailPriority === priority
                          ? priorityClass(priority)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailPriority = priority;
                      }}
                    >
                      {priorityLabel(priority)}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.type")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_TASK_TYPES as taskType}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailTaskType === taskType
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailTaskType = taskType;
                      }}
                    >
                      {taskTypeLabel(taskType)}
                    </button>
                  {/each}
                </div>
              </div>

              <label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-background px-2 text-[0.8rem]">
                <input type="checkbox" bind:checked={detailMilestone} class="h-4 w-4 accent-primary" />
                <span>{t("projects.detail.milestone")}</span>
              </label>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.labels")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskLabels.length}</span>
              </div>
              {#if selectedTaskLabels.length > 0}
                <div class="flex flex-wrap gap-1">
                  {#each selectedTaskLabels as label (label.id)}
                    <span class="inline-flex min-h-7 max-w-full items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.766667rem]">
                      <span
                        class={cn("h-2 w-2 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                        style={labelColorDotStyle(label.color)}
                      ></span>
                      <span class="truncate">{label.name}</span>
                      <button
                        type="button"
                        class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        aria-label={t("projects.actions.removeLabel", label.name)}
                        title={t("projects.actions.removeLabel", label.name)}
                        onclick={() => { void detachTaskLabel(selectedTask, label); }}
                      >
                        <X size={12} strokeWidth={1.75} />
                      </button>
                    </span>
                  {/each}
                </div>
              {:else}
                <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.detail.noLabels")}
                </div>
              {/if}
              <div class="grid gap-1">
                <div class="flex gap-1">
                  <input
                    bind:value={labelDraft}
                    placeholder={t("projects.detail.addLabelPlaceholder")}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitTaskLabel(selectedTask);
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                    aria-label={t("projects.detail.addLabel")}
                    onclick={() => { void submitTaskLabel(selectedTask); }}
                  >
                    <Plus size={14} strokeWidth={1.75} />
                  </button>
                </div>
                <div class="grid gap-1">
                  {#each selectedTaskLabelCandidates as label (label.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                      onclick={() => { void attachExistingLabel(selectedTask, label); }}
                    >
                      <span class="flex min-w-0 items-center gap-2">
                        <span
                          class={cn("h-2 w-2 shrink-0 rounded-full border", labelColorSwatchClass(label.color))}
                          style={labelColorDotStyle(label.color)}
                        ></span>
                        <span class="truncate text-[0.8rem]">{label.name}</span>
                      </span>
                      <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                    </button>
                  {:else}
                    {#if canCreateLabel(selectedTask)}
                      <button
                        type="button"
                        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                        onclick={() => { void submitTaskLabel(selectedTask); }}
                      >
                        <span class="truncate text-[0.8rem]">{t("projects.detail.createLabel", labelDraft.trim())}</span>
                        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                      </button>
                    {:else}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.detail.noLabelCandidates")}
                      </div>
                    {/if}
                  {/each}
                </div>
              </div>
            </section>

            {#if projectCustomFields.length > 0}
              <section class="grid gap-2 border-t border-border/70 pt-3">
                <div class="flex items-center justify-between gap-2">
                  <h2 class="text-[0.8rem] font-semibold">{t("projects.customFields.taskValues")}</h2>
                  <span class="text-[0.733333rem] text-muted-foreground">{projectCustomFields.length}</span>
                </div>
                <div class="grid gap-2">
                  {#each projectCustomFields as field (field.id)}
                    <div class="grid gap-1 rounded-md border border-border bg-background p-2">
                      <div class="flex min-w-0 items-center justify-between gap-2">
                        <div class="min-w-0">
                          <div class="truncate text-[0.8rem] font-medium">{field.name}</div>
                          <div class="truncate text-[0.733333rem] text-muted-foreground">
                            {customFieldTypeLabel(field.fieldType)}
                          </div>
                        </div>
                        <button
                          type="button"
                          class="flex min-h-7 shrink-0 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.733333rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                          disabled={!customFieldValueDirty(selectedTask, field)}
                          onclick={() => { void saveTaskCustomField(selectedTask, field); }}
                        >
                          <Save size={13} strokeWidth={1.75} />
                          <span>{t("projects.customFields.saveValue")}</span>
                        </button>
                      </div>

                      {#if field.fieldType === "text" || field.fieldType === "url"}
                        <input
                          value={customFieldTextDrafts[field.id] ?? ""}
                          inputmode={field.fieldType === "url" ? "url" : "text"}
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          placeholder={t("projects.customFields.emptyValue")}
                          oninput={(event) => {
                            customFieldTextDrafts = {
                              ...customFieldTextDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "number"}
                        <input
                          value={customFieldNumberDrafts[field.id] ?? ""}
                          inputmode="decimal"
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          placeholder={t("projects.customFields.emptyValue")}
                          oninput={(event) => {
                            customFieldNumberDrafts = {
                              ...customFieldNumberDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "date"}
                        <input
                          value={customFieldDateDrafts[field.id] ?? ""}
                          placeholder="YYYY-MM-DD"
                          class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
                          oninput={(event) => {
                            customFieldDateDrafts = {
                              ...customFieldDateDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void saveTaskCustomField(selectedTask, field);
                            }
                          }}
                        />
                      {:else if field.fieldType === "checkbox"}
                        <label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-card px-2 text-[0.8rem]">
                          <input
                            type="checkbox"
                            checked={customFieldCheckboxDrafts[field.id] ?? false}
                            class="h-4 w-4 accent-primary"
                            onchange={(event) => {
                              customFieldCheckboxDrafts = {
                                ...customFieldCheckboxDrafts,
                                [field.id]: event.currentTarget.checked,
                              };
                            }}
                          />
                          <span>{field.name}</span>
                        </label>
                      {:else if field.fieldType === "select"}
                        <div class="flex flex-wrap gap-1">
                          <button
                            type="button"
                            class={cn(
                              "rounded-md border px-2 py-1 text-[0.733333rem]",
                              (customFieldSelectDrafts[field.id] ?? "none") === "none"
                                ? "border-primary/50 bg-primary/10 text-primary"
                                : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                            )}
                            onclick={() => {
                              customFieldSelectDrafts = {
                                ...customFieldSelectDrafts,
                                [field.id]: "none",
                              };
                            }}
                          >
                            {t("projects.customFields.selectNone")}
                          </button>
                          {#each customFieldOptions(field) as option (option.id)}
                            <button
                              type="button"
                              class={cn(
                                "rounded-md border px-2 py-1 text-[0.733333rem]",
                                customFieldSelectDrafts[field.id] === option.id
                                  ? "border-primary/50 bg-primary/10 text-primary"
                                  : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                              )}
                              onclick={() => {
                                customFieldSelectDrafts = {
                                  ...customFieldSelectDrafts,
                                  [field.id]: option.id,
                                };
                              }}
                            >
                              {option.name}
                            </button>
                          {:else}
                            <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                              {t("projects.customFields.noOptions")}
                            </div>
                          {/each}
                        </div>
                      {:else}
                        <div class="flex flex-wrap gap-1">
                          {#each customFieldOptions(field) as option (option.id)}
                            {@const optionSelected = (customFieldMultiDrafts[field.id] ?? []).includes(option.id)}
                            <button
                              type="button"
                              class={cn(
                                "flex min-h-7 items-center gap-1 rounded-md border px-2 text-[0.733333rem]",
                                optionSelected
                                  ? "border-primary/50 bg-primary/10 text-primary"
                                  : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                              )}
                              onclick={() => toggleCustomFieldMultiOption(field, option)}
                            >
                              {#if optionSelected}
                                <Check size={12} strokeWidth={1.75} />
                              {/if}
                              <span>{option.name}</span>
                            </button>
                          {:else}
                            <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                              {t("projects.customFields.noOptions")}
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>
                  {/each}
                </div>
              </section>
            {/if}

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.dates")}</h2>
              <div class="grid gap-2 min-[980px]:grid-cols-2">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.estimateMinutes")}</span>
                  <input
                    bind:value={detailEstimateMinutes}
                    inputmode="numeric"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.startDate")}</span>
                  <input
                    bind:value={detailStartDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.dueDate")}</span>
                  <input
                    bind:value={detailDueDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.targetEndDate")}</span>
                  <input
                    bind:value={detailTargetEndDate}
                    placeholder="YYYY-MM-DD"
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
              </div>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.blockerReason")}</span>
                <input
                  bind:value={detailBlockerReason}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                />
              </label>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.changeReason")}</span>
                <input
                  bind:value={detailChangeReason}
                  maxlength="1000"
                  placeholder={t("projects.detail.changeReasonPlaceholder")}
                  class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                />
              </label>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.parentTask")}</h2>
                {#if selectedTask.parentTaskId}
                  <button
                    type="button"
                    class="flex min-h-7 items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                    onclick={() => { void promoteSubtaskFromDetail(selectedTask); }}
                  >
                    <ArrowLeft size={13} strokeWidth={1.75} />
                    <span>{t("projects.detail.promoteSubtask")}</span>
                  </button>
                {/if}
              </div>
              {#if selectedTask.parentTaskId}
                <div class="rounded-md border border-border bg-background px-2 py-1.5 text-[0.8rem]">
                  {selectedTaskParent?.title ?? t("projects.detail.missingDependencyTask")}
                </div>
              {:else if taskHasAnySubtasks(selectedTask)}
                <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.detail.demoteBlockedBySubtasks")}
                </div>
              {:else}
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.demoteToParent")}</span>
                  <input
                    bind:value={parentTaskSearch}
                    placeholder={t("projects.detail.demoteToParentPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                  />
                </label>
                <div class="grid gap-1">
                  {#each parentTaskCandidateTasks(selectedTask) as candidate (candidate.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left text-muted-foreground hover:bg-accent hover:text-foreground"
                      onclick={() => { void demoteTaskFromDetail(selectedTask, candidate); }}
                    >
                      <span class="truncate text-[0.8rem] text-foreground">{candidate.title}</span>
                      <ArrowRight size={13} strokeWidth={1.75} />
                    </button>
                  {:else}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.noParentCandidates")}
                    </div>
                  {/each}
                </div>
              {/if}
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.dependencies")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">
                  {selectedTaskBlockedBy.length + selectedTaskBlocks.length}
                </span>
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blockedBy")}</div>
                {#each selectedTaskBlockedBy as dependency (dependency.id)}
                  {@const blockingTask = taskById(dependency.blockingTaskId)}
                  <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
                    <span class="truncate text-[0.8rem]">
                      {blockingTask?.title ?? t("projects.detail.missingDependencyTask")}
                    </span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteDependency")}
                      title={t("projects.actions.deleteDependency")}
                      onclick={() => { void removeDependency(dependency.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noBlockedBy")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blocks")}</div>
                {#each selectedTaskBlocks as dependency (dependency.id)}
                  {@const blockedTask = taskById(dependency.blockedTaskId)}
                  <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
                    <span class="truncate text-[0.8rem]">
                      {blockedTask?.title ?? t("projects.detail.missingDependencyTask")}
                    </span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteDependency")}
                      title={t("projects.actions.deleteDependency")}
                      onclick={() => { void removeDependency(dependency.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noBlocks")}
                  </div>
                {/each}
              </div>

              <div class="grid gap-1">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.addBlockedBy")}</span>
                  <input
                    bind:value={dependencySearch}
                    placeholder={t("projects.detail.addBlockedByPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <div class="grid gap-1">
                  {#each selectedTaskDependencyCandidates as candidate (candidate.id)}
                    <button
                      type="button"
                      class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                      onclick={() => { void addBlockingDependency(candidate, selectedTask); }}
                    >
                      <span class="truncate text-[0.8rem]">{candidate.title}</span>
                      <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                    </button>
                  {:else}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.noDependencyCandidates")}
                    </div>
                  {/each}
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.checklist")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskChecklist.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskChecklist as item (item.id)}
                  {@const previousChecklistItem = adjacentChecklistItem(item, -1)}
                  {@const nextChecklistItem = adjacentChecklistItem(item, 1)}
                  <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-background px-2">
                    <button
                      type="button"
                      class={cn(
                        "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                        item.completedAt ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                      )}
                      aria-label={t("projects.actions.toggleChecklistItem")}
                      onclick={() => { void projects.setChecklistItemCompleted(item, !item.completedAt); }}
                    >
                      {#if item.completedAt}
                        <Check size={13} strokeWidth={2} />
                      {/if}
                    </button>
                    <input
                      value={checklistItemDraftTitle(item)}
                      class={cn(
                        "min-h-7 min-w-0 bg-transparent px-1 text-[0.8rem]",
                        item.completedAt ? "text-muted-foreground line-through" : "text-foreground",
                      )}
                      aria-label={t("projects.detail.checklistItemTitle")}
                      oninput={(event) => {
                        checklistTitleDrafts = {
                          ...checklistTitleDrafts,
                          [item.id]: event.currentTarget.value,
                        };
                      }}
                      onkeydown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          void saveChecklistItem(item);
                        }
                      }}
                    />
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!previousChecklistItem}
                      aria-label={previousChecklistItem ? t("projects.actions.moveChecklistItemUp", item.title) : t("projects.actions.noPreviousChecklistItem")}
                      title={previousChecklistItem ? t("projects.actions.moveChecklistItemUp", item.title) : t("projects.actions.noPreviousChecklistItem")}
                      onclick={() => { void moveChecklistItemInDetail(item, -1); }}
                    >
                      <ArrowUp size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!nextChecklistItem}
                      aria-label={nextChecklistItem ? t("projects.actions.moveChecklistItemDown", item.title) : t("projects.actions.noNextChecklistItem")}
                      title={nextChecklistItem ? t("projects.actions.moveChecklistItemDown", item.title) : t("projects.actions.noNextChecklistItem")}
                      onclick={() => { void moveChecklistItemInDetail(item, 1); }}
                    >
                      <ArrowDown size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!checklistItemDirty(item)}
                      aria-label={t("projects.actions.saveChecklistItem", item.title)}
                      title={t("projects.actions.saveChecklistItem", item.title)}
                      onclick={() => { void saveChecklistItem(item); }}
                    >
                      <Save size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.deleteChecklistItem", item.title)}
                      onclick={() => { void projects.removeChecklistItem(item.id); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noChecklistItems")}
                  </div>
                {/each}
              </div>
              <div class="flex gap-1">
                <input
                  bind:value={checklistDraft}
                  placeholder={t("projects.detail.addChecklistItemPlaceholder")}
                  class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitChecklistItem(selectedTask);
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                  aria-label={t("projects.detail.addChecklistItem")}
                  onclick={() => { void submitChecklistItem(selectedTask); }}
                >
                  <Plus size={14} strokeWidth={1.75} />
                </button>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.subtasks")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskSubtasks.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskSubtasks as subtask (subtask.id)}
                  {@const subtaskStatus = statusForTask(subtask)}
                  {@const previousSubtask = adjacentSubtask(subtask, -1)}
                  {@const nextSubtask = adjacentSubtask(subtask, 1)}
                  <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-background px-2">
                    <button
                      type="button"
                      class={cn(
                        "flex h-5 w-5 shrink-0 items-center justify-center rounded border",
                        subtaskStatus?.terminal ? "border-emerald-500 bg-emerald-500 text-white" : "border-border hover:bg-accent",
                      )}
                      aria-label={t("projects.actions.toggleComplete")}
                      onclick={() => { void projects.toggleTaskDone(subtask); }}
                    >
                      {#if subtaskStatus?.terminal}
                        <Check size={13} strokeWidth={2} />
                      {/if}
                    </button>
                    <button
                      type="button"
                      class="min-w-0 text-left"
                      aria-label={t("projects.actions.openTaskDetails", subtask.title)}
                      onclick={() => openTaskDetail(subtask)}
                    >
                      <span class="block truncate text-[0.8rem]">{subtask.title}</span>
                    </button>
                    <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", statusBadgeClass(subtaskStatus))}>
                      {subtaskStatus?.name ?? t("projects.list.status")}
                    </span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={Boolean(subtask.archivedAt)}
                      aria-label={t("projects.actions.promoteSubtask", subtask.title)}
                      title={t("projects.actions.promoteSubtask", subtask.title)}
                      onclick={() => { void promoteSubtaskFromDetail(subtask); }}
                    >
                      <ArrowLeft size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!previousSubtask}
                      aria-label={previousSubtask ? t("projects.actions.moveSubtaskUp", subtask.title) : t("projects.actions.noPreviousSubtask")}
                      title={previousSubtask ? t("projects.actions.moveSubtaskUp", subtask.title) : t("projects.actions.noPreviousSubtask")}
                      onclick={() => { void moveSubtaskInDetail(subtask, -1); }}
                    >
                      <ArrowUp size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                      disabled={!nextSubtask}
                      aria-label={nextSubtask ? t("projects.actions.moveSubtaskDown", subtask.title) : t("projects.actions.noNextSubtask")}
                      title={nextSubtask ? t("projects.actions.moveSubtaskDown", subtask.title) : t("projects.actions.noNextSubtask")}
                      onclick={() => { void moveSubtaskInDetail(subtask, 1); }}
                    >
                      <ArrowDown size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noSubtasks")}
                  </div>
                {/each}
              </div>
              <div class="flex gap-1">
                <input
                  bind:value={subtaskDraft}
                  placeholder={t("projects.detail.addSubtaskPlaceholder")}
                  class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem]"
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitSubtask(selectedTask);
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex min-h-8 items-center justify-center rounded-md border border-border bg-background px-2 text-[0.8rem] hover:bg-accent"
                  aria-label={t("projects.detail.addSubtask")}
                  onclick={() => { void submitSubtask(selectedTask); }}
                >
                  <Plus size={14} strokeWidth={1.75} />
                </button>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.scheduledBlocks")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskEvents.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskEvents as event (event.id)}
                  <div class="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-2 rounded-md border border-border bg-background px-2 py-1.5">
                    <div class="min-w-0">
                      <span class="block truncate text-[0.8rem]">{event.title || t("calendar.event.noTitle")}</span>
                      <span class="block truncate text-[0.733333rem] text-muted-foreground">{event.start}</span>
                    </div>
                    <span class="text-[0.733333rem] text-muted-foreground">{event.end}</span>
                    <button
                      type="button"
                      class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                      aria-label={t("projects.actions.unlinkScheduledBlock")}
                      title={t("projects.actions.unlinkScheduledBlock")}
                      onclick={() => { void unlinkExistingEvent(selectedTask, event); }}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noScheduledBlocks")}
                  </div>
                {/each}
              </div>
              {#if eventLinkSearchError}
                <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                  {eventLinkSearchError}
                </div>
              {/if}
              <div class="grid gap-1">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.linkExistingBlock")}</span>
                  <input
                    bind:value={eventLinkSearch}
                    placeholder={t("projects.detail.linkExistingBlockPlaceholder")}
                    class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                  />
                </label>
                <div class="grid gap-2 min-[980px]:grid-cols-2">
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkStartDate")}</span>
                    <input
                      bind:value={eventLinkStartDate}
                      placeholder="YYYY-MM-DD"
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkEndDate")}</span>
                    <input
                      bind:value={eventLinkEndDate}
                      placeholder="YYYY-MM-DD"
                      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
                    />
                  </label>
                </div>
                <div class="grid gap-1">
                  {#if eventLinkSearchPending}
                    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                      {t("projects.detail.searchingBlocks")}
                    </div>
                  {:else}
                    {#each selectedTaskEventCandidates as event (event.id)}
                      {@const otherLinkedTasks = otherLinkedTaskText(event, selectedTask)}
                      <button
                        type="button"
                        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
                        onclick={() => { void linkExistingEvent(selectedTask, event); }}
                      >
                        <span class="min-w-0">
                          <span class="block truncate text-[0.8rem]">{event.title || t("calendar.event.noTitle")}</span>
                          <span class="block truncate text-[0.733333rem] text-muted-foreground">{event.start}</span>
                          {#if otherLinkedTasks}
                            <span class="block truncate text-[0.7rem] text-muted-foreground">
                              {t("projects.detail.linkedToTasks", otherLinkedTasks)}
                            </span>
                          {/if}
                        </span>
                        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
                      </button>
                    {:else}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.detail.noLinkableBlocks")}
                      </div>
                    {/each}
                  {/if}
                </div>
              </div>
            </section>

            <section class="grid gap-2 border-t border-border/70 pt-3">
              <div class="flex items-center justify-between gap-2">
                <h2 class="text-[0.8rem] font-semibold">{t("projects.detail.history")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskHistory.length}</span>
              </div>
              <div class="grid gap-1">
                {#each selectedTaskHistory as event (event.id)}
                  <div class="rounded-md border border-border bg-background px-2 py-1.5">
                    <div class="truncate text-[0.8rem]">{historyEventLabel(event)}</div>
                    {#if event.reason}
                      <div class="truncate text-[0.733333rem] text-muted-foreground">
                        {t("projects.history.reason", event.reason)}
                      </div>
                    {/if}
                    <div class="truncate text-[0.733333rem] text-muted-foreground">{event.occurredAt}</div>
                  </div>
                {:else}
                  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                    {t("projects.detail.noHistory")}
                  </div>
                {/each}
              </div>
            </section>

            {#if detailError}
              <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                {detailError}
              </div>
            {/if}
          </div>
        </div>

        <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border px-3 py-2">
          {#if selectedTask.archivedAt}
            <button
              type="button"
              class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
              disabled={detailSaving}
              onclick={() => { void restoreTaskFromDetail(selectedTask); }}
            >
              <ArchiveRestore size={14} strokeWidth={1.75} />
              <span>{t("projects.detail.restore")}</span>
            </button>
          {:else}
            <button
              type="button"
              class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-destructive/30 bg-destructive/10 px-2 text-[0.8rem] text-destructive hover:bg-destructive/15 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={detailSaving}
              onclick={() => { void archiveTaskFromDetail(selectedTask); }}
            >
              <Archive size={14} strokeWidth={1.75} />
              <span>{t("projects.detail.archive")}</span>
            </button>
          {/if}
          <button
            type="button"
            class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
            disabled={!detailDirty}
            onclick={() => loadTaskDetailDraft(selectedTask)}
          >
            <RotateCcw size={14} strokeWidth={1.75} />
            <span>{t("projects.detail.discard")}</span>
          </button>
          <button
            type="submit"
            class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
            disabled={detailSaving || !detailDirty}
          >
            <Save size={14} strokeWidth={1.75} />
            <span>{detailSaving ? t("common.loading") : t("projects.detail.save")}</span>
          </button>
        </footer>
      </form>
    </aside>
{/if}
