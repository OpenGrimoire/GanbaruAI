<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import Check from "@lucide/svelte/icons/check";
  import Plus from "@lucide/svelte/icons/plus";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import MiniDatePicker from "$lib/components/calendar/MiniDatePicker.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import type { CalendarEvent } from "$lib/components/calendar/types";
  import {
    selectDateRangeEnd,
    selectDateRangeStart,
  } from "$lib/calendar/date-range-selection";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectCustomFieldTypeLabel,
    projectLabelColorDotStyle,
    projectLabelColorSwatchClass,
    projectPriorityBadgeClass,
    projectPriorityLabel,
    projectStatusBadgeClass,
    projectTaskArchivedBadgeClass,
    projectTaskTypeLabel,
  } from "$lib/projects/project-display";
  import {
    PROJECT_PRIORITIES,
    PROJECT_TASK_TYPES,
  } from "$lib/projects/types";
  import type {
    ProjectChecklistItem,
    ProjectCustomField,
    ProjectCustomFieldOption,
    ProjectLabel,
    ProjectLinkableEvent,
    ProjectPriority,
    ProjectStatus,
    ProjectTask,
    ProjectTaskType,
  } from "$lib/projects/types";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";
  import type { ProjectTaskModalLayout } from "$lib/projects/project-toolbar";
  import ProjectTaskDetailHistorySection from "./ProjectTaskDetailHistorySection.svelte";

  let {
    taskId,
    layout = "modal",
    showArchivedTasks,
    showInactiveSections,
    onClose,
    onOpenTask,
    onShowArchivedTasks,
  }: {
    taskId: string;
    layout?: ProjectTaskModalLayout;
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
  let detailScrollContainer: HTMLDivElement | undefined = $state();
  let discardCloseConfirmOpen = $state(false);
  let datePickerTarget: DetailDateTarget | null = $state(null);
  let customFieldDatePickerTarget = $state<string | null>(null);
  let pendingTaskOpenId = $state<string | null>(null);
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

  type DetailDateTarget = "start" | "due" | "target" | "eventStart" | "eventEnd";

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
  const detailHasUnsavedEdits = $derived.by(() => {
    if (!selectedTask) return false;
    return detailDirty
      || projectCustomFields.some((field) => customFieldValueDirty(selectedTask, field))
      || projects.checklistItemsForTask(selectedTask.id).some((item) => checklistItemDirty(item));
  });
  const todayDate = $derived(Temporal.Now.plainDateISO().toString());

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
    datePickerTarget = null;
    customFieldDatePickerTarget = null;
  }

  function closeTaskDetailImmediately(): void {
    if (selectedTask) loadTaskDetailDraft(selectedTask);
    onClose();
  }

  function requestTaskDetailClose(): void {
    if (detailHasUnsavedEdits) {
      pendingTaskOpenId = null;
      discardCloseConfirmOpen = true;
      return;
    }
    closeTaskDetailImmediately();
  }

  function confirmDiscardTaskDetail(): void {
    discardCloseConfirmOpen = false;
    const nextTaskId = pendingTaskOpenId;
    pendingTaskOpenId = null;
    if (selectedTask) loadTaskDetailDraft(selectedTask);
    if (nextTaskId) {
      onOpenTask(nextTaskId);
      return;
    }
    onClose();
  }

  function cancelDiscardTaskDetail(): void {
    discardCloseConfirmOpen = false;
    pendingTaskOpenId = null;
  }

  function openTaskDetail(task: ProjectTask): void {
    if (detailHasUnsavedEdits) {
      pendingTaskOpenId = task.id;
      discardCloseConfirmOpen = true;
      return;
    }
    onOpenTask(task.id);
  }

  function handleTaskDetailKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || discardCloseConfirmOpen) return;
    event.preventDefault();
    event.stopPropagation();
    requestTaskDetailClose();
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

  function setDetailDateValue(target: DetailDateTarget, value: string): void {
    if (target === "start") {
      detailStartDate = value;
      return;
    }
    if (target === "due") {
      detailDueDate = value;
      return;
    }
    if (target === "target") {
      detailTargetEndDate = value;
      return;
    }
    if (target === "eventStart") {
      eventLinkStartDate = value;
      return;
    }
    eventLinkEndDate = value;
  }

  function toggleDetailDatePicker(target: DetailDateTarget): void {
    datePickerTarget = datePickerTarget === target ? null : target;
    customFieldDatePickerTarget = null;
  }

  function selectDetailDate(dateStr: string): void {
    if (!datePickerTarget) return;
    if (datePickerTarget === "start") {
      const nextRange = selectDateRangeStart({
        selectedDate: dateStr,
        startDate: detailStartDate || undefined,
        endDate: detailDueDate || undefined,
      });
      detailStartDate = nextRange.startDate ?? "";
      detailDueDate = nextRange.endDate ?? "";
      datePickerTarget = null;
      return;
    }
    if (datePickerTarget === "due") {
      const nextRange = selectDateRangeEnd({
        selectedDate: dateStr,
        startDate: detailStartDate || undefined,
        endDate: detailDueDate || undefined,
      });
      detailStartDate = nextRange.startDate ?? "";
      detailDueDate = nextRange.endDate ?? "";
      datePickerTarget = null;
      return;
    }
    setDetailDateValue(datePickerTarget, dateStr);
    datePickerTarget = null;
  }

  function clearDetailDate(target: DetailDateTarget): void {
    setDetailDateValue(target, "");
    if (datePickerTarget === target) datePickerTarget = null;
  }

  function selectCustomFieldDate(dateStr: string): void {
    if (!customFieldDatePickerTarget) return;
    customFieldDateDrafts = {
      ...customFieldDateDrafts,
      [customFieldDatePickerTarget]: dateStr,
    };
    customFieldDatePickerTarget = null;
  }

  function clearCustomFieldDate(fieldId: string): void {
    customFieldDateDrafts = {
      ...customFieldDateDrafts,
      [fieldId]: "",
    };
    if (customFieldDatePickerTarget === fieldId) customFieldDatePickerTarget = null;
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

<svelte:window onkeydown={handleTaskDetailKeydown} />

{#if selectedTask}
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
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="fixed inset-0 z-70 flex items-center justify-center bg-black/30 p-3"
      onclick={requestTaskDetailClose}
    >
    <div
      class={cn(
        "flex min-h-0 flex-col overflow-hidden border border-border bg-card text-card-foreground",
        layout === "fullscreen" && "h-[calc(100dvh-1rem)] w-[calc(100vw-1rem)] rounded-md",
        layout === "sheet" && "h-[min(88dvh,48rem)] w-[calc(100vw-1rem)] max-w-3xl rounded-md",
        layout === "modal" && "h-[min(86dvh,54rem)] w-[min(56rem,calc(100vw-2rem))] rounded-md",
      )}
      role="dialog"
      aria-modal="true"
      aria-label={t("projects.detail.title")}
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
    >
      <header class="flex shrink-0 items-center gap-2 border-b border-border bg-card px-4 py-3">
        <div class="min-w-0 flex-1">
          <div class="flex min-w-0 items-center gap-2">
            <input
              bind:value={detailTitle}
              aria-label={t("projects.detail.titleLabel")}
              class="min-h-8 min-w-0 flex-1 rounded-md bg-transparent px-1 text-[1.1rem] font-semibold text-foreground outline-none focus:bg-background focus:ring-2 focus:ring-ring/30"
            />
            {#if selectedTask.archivedAt}
              <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectTaskArchivedBadgeClass(selectedTask))}>
                {t("projects.taskLifecycle.archived")}
              </span>
            {/if}
          </div>
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.detail.close")}
          title={t("projects.detail.close")}
          onclick={requestTaskDetailClose}
        >
          <X size={15} strokeWidth={1.75} />
        </button>
      </header>

      <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void saveTaskDetail(); }}>
        <div class="relative min-h-0 flex-1 bg-background/50">
          <div bind:this={detailScrollContainer} class="hide-scrollbar h-full overflow-y-auto px-4 py-4">
          <div class="mx-auto grid max-w-5xl gap-5">
            <section class="task-detail-section task-detail-section-first">
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
              <span>{t("projects.detail.description")}</span>
              <textarea
                bind:value={detailDescription}
                rows="5"
                class="min-h-28 resize-none rounded-md border border-border bg-card px-3 py-2 text-[0.833333rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
              ></textarea>
              </label>
            </section>

            <section class="task-detail-section">
              <h2 class="task-detail-section-title">{t("projects.detail.properties")}</h2>

              <div class="grid gap-1">
                <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.status")}</div>
                <div class="flex flex-wrap gap-1">
                  {#each statuses as status (status.id)}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.766667rem]",
                        detailStatusId === status.id
                          ? projectStatusBadgeClass(status)
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
                          ? projectPriorityBadgeClass(priority)
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        detailPriority = priority;
                      }}
                    >
                      {projectPriorityLabel(priority, t)}
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
                      {projectTaskTypeLabel(taskType, t)}
                    </button>
                  {/each}
                </div>
              </div>

              <label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-background px-2 text-[0.8rem]">
                <input type="checkbox" bind:checked={detailMilestone} class="h-4 w-4 accent-primary" />
                <span>{t("projects.detail.milestone")}</span>
              </label>
            </section>

            <section class="task-detail-section">
              <h2 class="task-detail-section-title">{t("projects.detail.dates")}</h2>
              <div class="grid gap-3 min-[760px]:grid-cols-2">
                <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.estimateMinutes")}</span>
                  <input
                    bind:value={detailEstimateMinutes}
                    inputmode="numeric"
                    class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
                  />
                </label>

                <div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.startDate")}</span>
                  <div class="flex gap-1">
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                        !detailStartDate && "text-muted-foreground",
                      )}
                      onclick={() => toggleDetailDatePicker("start")}
                    >
                      <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                      <span class="truncate">{detailStartDate || t("projects.detail.noDate")}</span>
                    </button>
                    {#if detailStartDate}
                      <button
                        type="button"
                        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                        aria-label={t("projects.detail.clearDate", t("projects.detail.startDate"))}
                        onclick={() => clearDetailDate("start")}
                      >
                        <X size={13} strokeWidth={1.75} />
                      </button>
                    {/if}
                  </div>
                  {#if datePickerTarget === "start"}
                    <div class="w-fit rounded-lg border border-border bg-card p-2">
                      <MiniDatePicker
                        selectedDate={detailStartDate || todayDate}
                        rangeStartDate={detailStartDate || undefined}
                        rangeEndDate={detailDueDate || undefined}
                        small
                        highlightToday={false}
                        activeHighlight="primary"
                        onselect={selectDetailDate}
                        oncancel={() => { datePickerTarget = null; }}
                      />
                    </div>
                  {/if}
                </div>

                <div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.dueDate")}</span>
                  <div class="flex gap-1">
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                        !detailDueDate && "text-muted-foreground",
                      )}
                      onclick={() => toggleDetailDatePicker("due")}
                    >
                      <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                      <span class="truncate">{detailDueDate || t("projects.detail.noDate")}</span>
                    </button>
                    {#if detailDueDate}
                      <button
                        type="button"
                        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                        aria-label={t("projects.detail.clearDate", t("projects.detail.dueDate"))}
                        onclick={() => clearDetailDate("due")}
                      >
                        <X size={13} strokeWidth={1.75} />
                      </button>
                    {/if}
                  </div>
                  {#if datePickerTarget === "due"}
                    <div class="w-fit rounded-lg border border-border bg-card p-2">
                      <MiniDatePicker
                        selectedDate={detailDueDate || todayDate}
                        rangeStartDate={detailStartDate || undefined}
                        rangeEndDate={detailDueDate || undefined}
                        small
                        highlightToday={false}
                        activeHighlight="primary"
                        onselect={selectDetailDate}
                        oncancel={() => { datePickerTarget = null; }}
                      />
                    </div>
                  {/if}
                </div>

                <div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                  <span>{t("projects.detail.targetEndDate")}</span>
                  <div class="flex gap-1">
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                        !detailTargetEndDate && "text-muted-foreground",
                      )}
                      onclick={() => toggleDetailDatePicker("target")}
                    >
                      <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                      <span class="truncate">{detailTargetEndDate || t("projects.detail.noDate")}</span>
                    </button>
                    {#if detailTargetEndDate}
                      <button
                        type="button"
                        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                        aria-label={t("projects.detail.clearDate", t("projects.detail.targetEndDate"))}
                        onclick={() => clearDetailDate("target")}
                      >
                        <X size={13} strokeWidth={1.75} />
                      </button>
                    {/if}
                  </div>
                  {#if datePickerTarget === "target"}
                    <div class="w-fit rounded-lg border border-border bg-card p-2">
                      <MiniDatePicker
                        selectedDate={detailTargetEndDate || todayDate}
                        small
                        highlightMode="none"
                        activeHighlight="primary"
                        onselect={selectDetailDate}
                        oncancel={() => { datePickerTarget = null; }}
                      />
                    </div>
                  {/if}
                </div>
              </div>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.blockerReason")}</span>
                <input
                  bind:value={detailBlockerReason}
                  class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
                />
              </label>
              <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                <span>{t("projects.detail.changeReason")}</span>
                <input
                  bind:value={detailChangeReason}
                  maxlength="1000"
                  placeholder={t("projects.detail.changeReasonPlaceholder")}
                  class="min-h-8 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground focus:ring-2 focus:ring-ring/30"
                />
              </label>
            </section>

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.labels")}</h2>
                <span class="text-[0.733333rem] text-muted-foreground">{selectedTaskLabels.length}</span>
              </div>
              {#if selectedTaskLabels.length > 0}
                <div class="flex flex-wrap gap-1">
                  {#each selectedTaskLabels as label (label.id)}
                    <span class="inline-flex min-h-7 max-w-full items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.766667rem]">
                      <span
                        class={cn("h-2 w-2 shrink-0 rounded-full border", projectLabelColorSwatchClass(label.color))}
                        style={projectLabelColorDotStyle(label.color, theme.current)}
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
                          class={cn("h-2 w-2 shrink-0 rounded-full border", projectLabelColorSwatchClass(label.color))}
                          style={projectLabelColorDotStyle(label.color, theme.current)}
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
              <section class="task-detail-section">
                <div class="flex items-center justify-between gap-2">
                  <h2 class="task-detail-section-title">{t("projects.customFields.taskValues")}</h2>
                  <span class="text-[0.733333rem] text-muted-foreground">{projectCustomFields.length}</span>
                </div>
                <div class="grid gap-2">
                  {#each projectCustomFields as field (field.id)}
                    <div class="grid gap-1 rounded-md border border-border bg-background p-2">
                      <div class="flex min-w-0 items-center justify-between gap-2">
                        <div class="min-w-0">
                          <div class="truncate text-[0.8rem] font-medium">{field.name}</div>
                          <div class="truncate text-[0.733333rem] text-muted-foreground">
                            {projectCustomFieldTypeLabel(field.fieldType, t)}
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
                        <div class="grid gap-1">
                          <div class="flex gap-1">
                            <button
                              type="button"
                              class={cn(
                                "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                                !(customFieldDateDrafts[field.id] ?? "") && "text-muted-foreground",
                              )}
                              onclick={() => {
                                customFieldDatePickerTarget = customFieldDatePickerTarget === field.id ? null : field.id;
                                datePickerTarget = null;
                              }}
                            >
                              <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                              <span class="truncate">{customFieldDateDrafts[field.id] || t("projects.detail.noDate")}</span>
                            </button>
                            {#if customFieldDateDrafts[field.id]}
                              <button
                                type="button"
                                class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                                aria-label={t("projects.detail.clearDate", field.name)}
                                onclick={() => clearCustomFieldDate(field.id)}
                              >
                                <X size={13} strokeWidth={1.75} />
                              </button>
                            {/if}
                          </div>
                          {#if customFieldDatePickerTarget === field.id}
                            <div class="w-fit rounded-lg border border-border bg-card p-2">
                              <MiniDatePicker
                                selectedDate={customFieldDateDrafts[field.id] || todayDate}
                                small
                                highlightMode="none"
                                activeHighlight="primary"
                                onselect={selectCustomFieldDate}
                                oncancel={() => { customFieldDatePickerTarget = null; }}
                              />
                            </div>
                          {/if}
                        </div>
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

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.parentTask")}</h2>
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

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.dependencies")}</h2>
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

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.checklist")}</h2>
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

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.subtasks")}</h2>
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
                    <span class={cn("rounded border px-1.5 py-0.5 text-[0.733333rem]", projectStatusBadgeClass(subtaskStatus))}>
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

            <section class="task-detail-section">
              <div class="flex items-center justify-between gap-2">
                <h2 class="task-detail-section-title">{t("projects.detail.scheduledBlocks")}</h2>
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
                <div class="grid gap-2 min-[760px]:grid-cols-2">
                  <div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkStartDate")}</span>
                    <div class="flex gap-1">
                      <button
                        type="button"
                        class={cn(
                          "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                          !eventLinkStartDate && "text-muted-foreground",
                        )}
                        onclick={() => toggleDetailDatePicker("eventStart")}
                      >
                        <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                        <span class="truncate">{eventLinkStartDate || t("projects.detail.noDate")}</span>
                      </button>
                      {#if eventLinkStartDate}
                        <button
                          type="button"
                          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                          aria-label={t("projects.detail.clearDate", t("projects.detail.eventLinkStartDate"))}
                          onclick={() => clearDetailDate("eventStart")}
                        >
                          <X size={13} strokeWidth={1.75} />
                        </button>
                      {/if}
                    </div>
                    {#if datePickerTarget === "eventStart"}
                      <div class="w-fit rounded-lg border border-border bg-card p-2">
                        <MiniDatePicker
                          selectedDate={eventLinkStartDate || todayDate}
                          small
                          highlightMode="none"
                          activeHighlight="primary"
                          onselect={selectDetailDate}
                          oncancel={() => { datePickerTarget = null; }}
                        />
                      </div>
                    {/if}
                  </div>
                  <div class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
                    <span>{t("projects.detail.eventLinkEndDate")}</span>
                    <div class="flex gap-1">
                      <button
                        type="button"
                        class={cn(
                          "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-card px-2 text-left text-[0.8rem] text-foreground hover:bg-accent",
                          !eventLinkEndDate && "text-muted-foreground",
                        )}
                        onclick={() => toggleDetailDatePicker("eventEnd")}
                      >
                        <CalendarDays size={14} strokeWidth={1.75} class="shrink-0" />
                        <span class="truncate">{eventLinkEndDate || t("projects.detail.noDate")}</span>
                      </button>
                      {#if eventLinkEndDate}
                        <button
                          type="button"
                          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground"
                          aria-label={t("projects.detail.clearDate", t("projects.detail.eventLinkEndDate"))}
                          onclick={() => clearDetailDate("eventEnd")}
                        >
                          <X size={13} strokeWidth={1.75} />
                        </button>
                      {/if}
                    </div>
                    {#if datePickerTarget === "eventEnd"}
                      <div class="w-fit rounded-lg border border-border bg-card p-2">
                        <MiniDatePicker
                          selectedDate={eventLinkEndDate || todayDate}
                          small
                          highlightMode="none"
                          activeHighlight="primary"
                          onselect={selectDetailDate}
                          oncancel={() => { datePickerTarget = null; }}
                        />
                      </div>
                    {/if}
                  </div>
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

            <ProjectTaskDetailHistorySection events={selectedTaskHistory} />

            {#if detailError}
              <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
                {detailError}
              </div>
            {/if}
          </div>
        </div>
        <CalendarScrollbar
          scrollContainer={detailScrollContainer}
          stickyTop={8}
          stickyBottom={8}
          wheelPassthrough
        />
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
            type="submit"
            class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
            disabled={detailSaving || !detailDirty}
          >
            <Save size={14} strokeWidth={1.75} />
            <span>{detailSaving ? t("common.loading") : t("projects.detail.save")}</span>
          </button>
        </footer>
      </form>
    </div>
    </div>
    {#if discardCloseConfirmOpen}
      <ConfirmDialog
        title={t("calendar.view.discardUnsavedTitle")}
        message={t("calendar.view.changesLost")}
        confirmLabel={t("calendar.view.discard")}
        cancelLabel={t("common.cancelShortcut")}
        onConfirm={confirmDiscardTaskDetail}
        onCancel={cancelDiscardTaskDetail}
      />
    {/if}
{/if}

<style>
  .task-detail-section {
    display: grid;
    gap: 0.75rem;
    padding-top: 1.1rem;
  }

  .task-detail-section + .task-detail-section {
    border-top: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
  }

  .task-detail-section-first {
    padding-top: 0;
  }

  .task-detail-section-title {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0;
  }
</style>
