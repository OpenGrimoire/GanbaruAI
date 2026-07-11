<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { projectPriorityDisplayLabel } from "$lib/projects/project-display";
  import {
    projectCustomFieldDisplayText,
    projectCustomFieldUsesOptions,
  } from "$lib/projects/custom-fields";
  import {
    doubleClickProjectTaskListColumnResizeWidths,
    keyboardProjectTaskListColumnResizeWidths,
    moveProjectTaskListColumnResize,
    projectTaskListResizableColumnWidthRem,
    projectTaskListGridMinWidth,
    projectTaskListGridTemplate,
    startProjectTaskListColumnResize,
    type ProjectTaskListColumnResizeGesture,
    type ProjectTaskListColumnWidths,
    type ProjectTaskListResizableColumn,
  } from "$lib/projects/project-list-view";
  import {
    projectListDropSortOrder,
    projectListDropPositionFromPoint,
    projectListPointerDragGestureReady,
    projectListSectionDropSortOrder,
    projectListSectionDragAllowed,
    projectListSectionDropAllowed,
    projectListTaskDragAllowed,
    projectListTaskDropAllowed,
    type ProjectListDropPosition,
    type ProjectListPointerDragGesture,
  } from "$lib/projects/list-drag";
  import {
    projectListDueDateEditPlan,
    projectListStartDateEditPlan,
  } from "$lib/projects/project-list-date-edit";
  import {
    projectListGroupQuickAddPlan,
    projectListGroupTaskCreateTarget,
    projectListGroupTaskDraftKey,
    projectListSectionTaskCreateTarget,
    type ProjectListGroupQuickAddPlan,
    type ProjectListTaskCreateTarget,
  } from "$lib/projects/project-list-quick-add";
  import {
    type ProjectCustomField,
    type ProjectCustomFieldOption,
    type ProjectCustomFieldValue,
    type ProjectCustomFieldValueUpdate,
    type ProjectTag,
    type ProjectPriority,
    type ProjectPriorityConfig,
    type ProjectSection,
    type ProjectStatus,
    type ProjectTask,
    type ProjectTaskGroupMode,
    type ProjectTaskListColumn,
    type ProjectTaskSortDirection,
    type ProjectTaskSortMode,
  } from "$lib/projects/types";
  import type { ProjectTaskListGroup } from "$lib/projects/task-view";
  import ProjectListColumnHeaders from "./ProjectListColumnHeaders.svelte";
  import ProjectListGroupHeader from "./ProjectListGroupHeader.svelte";
  import ProjectListScrollbars from "./ProjectListScrollbars.svelte";
  import ProjectListSectionAddRow from "./ProjectListSectionAddRow.svelte";
  import ProjectListSectionBlock from "./ProjectListSectionBlock.svelte";
  import ProjectListTaskAddRow from "./ProjectListTaskAddRow.svelte";
  import ProjectListTaskRows from "./ProjectListTaskRows.svelte";

  type ProjectListTaskMenu = "status" | "priority" | "start" | "due";
  let {
    selectedProjectId,
    sections,
    statuses,
    priorities,
    tasks,
    allProjectTasks,
    listTaskGroups,
    taskGroupBy,
    taskSortMode,
    taskSortDirection,
    taskListColumns,
    taskListColumnWidths,
    projectCustomFields,
    selectedTaskId,
    selectedTaskIds,
    showArchivedTasks,
    onOpenTask,
    onSelectedTaskIdsChange,
    onRevealTask,
    onTaskListColumnWidthsChange,
    onNeedMore,
  }: {
    selectedProjectId: string | null;
    sections: ProjectSection[];
    statuses: ProjectStatus[];
    priorities: ProjectPriorityConfig[];
    tasks: ProjectTask[];
    allProjectTasks: ProjectTask[];
    listTaskGroups: ProjectTaskListGroup[];
    taskGroupBy: ProjectTaskGroupMode;
    taskSortMode: ProjectTaskSortMode;
    taskSortDirection: ProjectTaskSortDirection;
    taskListColumns: ProjectTaskListColumn[];
    taskListColumnWidths: ProjectTaskListColumnWidths;
    projectCustomFields: ProjectCustomField[];
    selectedTaskId: string | null;
    selectedTaskIds: string[];
    showArchivedTasks: boolean;
    onOpenTask: (task: ProjectTask) => void;
    onSelectedTaskIdsChange: (taskIds: string[]) => void;
    onRevealTask: (task: ProjectTask | undefined) => void;
    onTaskListColumnWidthsChange: (widths: ProjectTaskListColumnWidths, options?: { persist?: boolean }) => void;
    onNeedMore: () => void;
  } = $props();

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_LIST_DRAG_MIME = "application/x-ganbaru-project-list-task";
  const PROJECT_LIST_SECTION_DRAG_MIME = "application/x-ganbaru-project-list-section";
  const PROJECT_LIST_KEYBOARD_SCROLL_PX = 48;

  let taskCreatePendingTarget = $state<ProjectListTaskCreateTarget | null>(null);
  let taskCreateErrorTarget = $state<ProjectListTaskCreateTarget | null>(null);
  let taskCreateErrorMessage = $state<string | null>(null);
  let sectionDraft = $state("");
  let sectionTaskDrafts = $state<Record<string, string>>({});
  let groupTaskDrafts = $state<Record<string, string>>({});
  let activeSectionTaskDraftInputId = $state<string | null>(null);
  let activeGroupTaskDraftInputId = $state<string | null>(null);
  let sectionDraftInputActive = $state(false);
  let listDraggingTaskId = $state<string | null>(null);
  let listDragOverSectionId = $state<string | null>(null);
  let listDragOverTaskId = $state<string | null>(null);
  let listDragOverPosition = $state<ProjectListDropPosition | "section" | null>(null);
  let listDropPendingTaskId = $state<string | null>(null);
  let listRowDragGesture = $state<ProjectListPointerDragGesture | null>(null);
  let listDraggingSectionId = $state<string | null>(null);
  let listSectionDragOverId = $state<string | null>(null);
  let listSectionDragOverPosition = $state<ProjectListDropPosition | null>(null);
  let listSectionDropPendingId = $state<string | null>(null);
  let listSectionDragGesture = $state<ProjectListPointerDragGesture | null>(null);
  let suppressedTaskOpenTaskId = $state<string | null>(null);
  let sectionNameDrafts = $state<Record<string, string>>({});
  let sectionOptionsMenuId = $state<string | null>(null);
  let statusMenuTaskId = $state<string | null>(null);
  let priorityMenuTaskId = $state<string | null>(null);
  let startDateMenuTaskId = $state<string | null>(null);
  let dueDateMenuTaskId = $state<string | null>(null);
  let projectViewScrollContainer = $state<HTMLDivElement | null>(null);
  let listColumnResizeGesture = $state<ProjectTaskListColumnResizeGesture | null>(null);

  const selectedTaskIdSet = $derived.by(() => new Set(selectedTaskIds));
  const effectiveTaskListColumnWidths = $derived(listColumnResizeGesture?.draftWidths ?? taskListColumnWidths);
  const taskListGridInput = $derived({
    columns: taskListColumns,
    columnWidths: effectiveTaskListColumnWidths,
    tasks,
    statuses,
    customFields: projectCustomFields,
    nameLabel: t("projects.list.name"),
    sectionLabels: sections.map((section) => section.name),
    groupLabels: listTaskGroups.map((group) => taskListGroupTitle(group.value)),
    columnLabel: taskListColumnLabel,
    priorityLabel: (priority: ProjectPriority) => projectPriorityDisplayLabel(priority, priorities, t),
    estimateLabel,
    customFieldDisplayValue,
    scheduledLabel,
  });
  const taskListGridTemplate = $derived(projectTaskListGridTemplate(taskListGridInput));
  const taskListGridMinWidth = $derived(projectTaskListGridMinWidth(taskListGridInput));
  const listRangeDateColumnsVisible = $derived(taskListColumns.includes("start") && taskListColumns.includes("due"));

  function cssPixelValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function projectListMaxHorizontalScrollLeft(): number {
    const el = projectViewScrollContainer;
    if (!el) return 0;

    const content = el.firstElementChild;
    if (!(content instanceof HTMLElement)) {
      return Math.max(0, el.scrollWidth - el.clientWidth);
    }

    const contentStyle = getComputedStyle(content);
    const paddingRight = cssPixelValue(contentStyle.paddingRight);
    let contentRight = 0;

    for (const child of Array.from(content.children)) {
      if (!(child instanceof HTMLElement)) continue;
      contentRight = Math.max(contentRight, child.offsetLeft + child.offsetWidth);
    }

    if (contentRight <= 0) return Math.max(0, el.scrollWidth - el.clientWidth);
    return Math.max(0, contentRight + paddingRight - el.clientWidth);
  }

  function setProjectListCounterScroll(scrollLeft: number): number {
    const el = projectViewScrollContainer;
    if (!el) return scrollLeft;
    const maxScrollLeft = projectListMaxHorizontalScrollLeft();
    const nextScrollLeft = Math.max(0, Math.min(maxScrollLeft, scrollLeft));
    el.style.setProperty("--project-list-scroll-left", `${nextScrollLeft}px`);
    el.style.setProperty("--project-list-scroll-left-negative", `${-nextScrollLeft}px`);
    return nextScrollLeft;
  }

  function setProjectListHorizontalScroll(scrollLeft: number): number {
    const el = projectViewScrollContainer;
    const nextScrollLeft = setProjectListCounterScroll(scrollLeft);
    if (el && el.scrollLeft !== nextScrollLeft) {
      el.scrollLeft = nextScrollLeft;
    }
    return nextScrollLeft;
  }

  function syncProjectListCounterScroll(): void {
    const el = projectViewScrollContainer;
    if (!el) return;
    setProjectListHorizontalScroll(el.scrollLeft);
  }

  function projectListRootFontSizePx(): number {
    const parsed = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : 16;
  }

  function startProjectListColumnResize(
    event: PointerEvent,
    column: ProjectTaskListResizableColumn,
  ): void {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget;
    if (!(trigger instanceof HTMLElement)) return;
    const rootFontSizePx = projectListRootFontSizePx();
    const headerCell = trigger.closest(".project-list-header-cell");
    const startWidthRem = headerCell instanceof HTMLElement
      ? headerCell.getBoundingClientRect().width / rootFontSizePx
      : projectTaskListResizableColumnWidthRem(column, taskListGridInput);
    const widthsAtStart = { ...effectiveTaskListColumnWidths };
    listColumnResizeGesture = startProjectTaskListColumnResize({
      column,
      pointerId: event.pointerId,
      startClientX: event.clientX,
      startWidthRem,
      rootFontSizePx,
      widthsAtStart,
    });
    trigger.setPointerCapture(event.pointerId);
  }

  function handleProjectListColumnResizePointerMove(event: PointerEvent): void {
    const gesture = listColumnResizeGesture;
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    event.preventDefault();
    listColumnResizeGesture = moveProjectTaskListColumnResize(gesture, event.clientX);
    void tick().then(syncProjectListCounterScroll);
  }

  function finishProjectListColumnResize(event: PointerEvent, persist: boolean): void {
    const gesture = listColumnResizeGesture;
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    if (persist && gesture.moved) {
      onTaskListColumnWidthsChange(gesture.draftWidths, { persist: true });
    }
    listColumnResizeGesture = null;
    void tick().then(syncProjectListCounterScroll);
  }

  function handleProjectListColumnResizeDoubleClick(
    event: MouseEvent,
    column: ProjectTaskListResizableColumn,
  ): void {
    event.preventDefault();
    event.stopPropagation();
    const nextWidths = doubleClickProjectTaskListColumnResizeWidths({
      column,
      widths: effectiveTaskListColumnWidths,
      gridInput: taskListGridInput,
    });
    onTaskListColumnWidthsChange(nextWidths, { persist: true });
    void tick().then(syncProjectListCounterScroll);
  }

  function handleProjectListColumnResizeKeydown(
    event: KeyboardEvent,
    column: ProjectTaskListResizableColumn,
  ): void {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    event.stopPropagation();
    const direction = event.key === "ArrowRight" ? 1 : -1;
    onTaskListColumnWidthsChange(keyboardProjectTaskListColumnResizeWidths({
      column,
      direction,
      wideStep: event.shiftKey,
      widths: effectiveTaskListColumnWidths,
      gridInput: taskListGridInput,
    }), { persist: true });
    void tick().then(syncProjectListCounterScroll);
  }

  $effect(() => {
    const el = projectViewScrollContainer;
    if (!el) return;
    setProjectListCounterScroll(el.scrollLeft);
  });

  function projectListKeyboardScrollAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    return !target.closest("input, textarea, select, [contenteditable='true'], [role='textbox']");
  }

  function handleProjectListHorizontalKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape" && projectListAddDraftActive()) {
      event.preventDefault();
      event.stopPropagation();
      cancelActiveProjectListAddDrafts();
      projectListBlurTarget(event.target);
      return;
    }

    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (!projectListKeyboardScrollAllowed(event.target)) return;
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;

    const el = projectViewScrollContainer;
    if (!el) return;
    const maxScrollLeft = projectListMaxHorizontalScrollLeft();
    if (maxScrollLeft <= 0) return;

    const delta = event.key === "ArrowRight"
      ? PROJECT_LIST_KEYBOARD_SCROLL_PX
      : -PROJECT_LIST_KEYBOARD_SCROLL_PX;
    const nextScrollLeft = Math.max(0, Math.min(maxScrollLeft, el.scrollLeft + delta));
    if (nextScrollLeft === el.scrollLeft) return;

    event.preventDefault();
    setProjectListHorizontalScroll(nextScrollLeft);
  }

  function projectListEventTargetElement(target: EventTarget | null): Element | null {
    if (target instanceof Element) return target;
    if (target instanceof Node) return target.parentElement;
    return null;
  }

  function projectListBlurTarget(target: EventTarget | null): void {
    const targetElement = projectListEventTargetElement(target);
    if (targetElement instanceof HTMLElement) {
      targetElement.blur();
      return;
    }
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }

  function projectListAddDraftActive(): boolean {
    return activeSectionTaskDraftInputId !== null
      || activeGroupTaskDraftInputId !== null
      || sectionDraftInputActive
      || sectionDraft.trim().length > 0
      || Object.values(groupTaskDrafts).some((draft) => draft.trim().length > 0);
  }

  function cancelActiveProjectListAddDrafts(): void {
    if (activeSectionTaskDraftInputId) {
      const targetId = activeSectionTaskDraftInputId;
      sectionTaskDrafts = { ...sectionTaskDrafts, [targetId]: "" };
      activeSectionTaskDraftInputId = null;
      clearTaskCreateError(projectListSectionTaskCreateTarget(targetId));
    }

    if (activeGroupTaskDraftInputId) {
      const targetId = activeGroupTaskDraftInputId;
      groupTaskDrafts = { ...groupTaskDrafts, [targetId]: "" };
      activeGroupTaskDraftInputId = null;
      clearTaskCreateError(projectListGroupTaskCreateTarget(targetId));
    }

    if (sectionDraftInputActive || sectionDraft.trim()) {
      sectionDraft = "";
      sectionDraftInputActive = false;
    }
  }

  function cancelProjectListAddDraftsForOutsideTarget(target: Element): void {
    if (activeSectionTaskDraftInputId) {
      const taskAddRow = target.closest("[data-section-task-add-row]");
      const taskAddRowSectionId = taskAddRow?.getAttribute("data-section-task-add-row");
      if (taskAddRowSectionId !== activeSectionTaskDraftInputId) {
        const targetId = activeSectionTaskDraftInputId;
        sectionTaskDrafts = { ...sectionTaskDrafts, [targetId]: "" };
        activeSectionTaskDraftInputId = null;
        clearTaskCreateError(projectListSectionTaskCreateTarget(targetId));
      }
    }

    if (activeGroupTaskDraftInputId) {
      const taskAddRow = target.closest("[data-group-task-add-row]");
      const taskAddRowGroupId = taskAddRow?.getAttribute("data-group-task-add-row");
      if (taskAddRowGroupId !== activeGroupTaskDraftInputId) {
        const targetId = activeGroupTaskDraftInputId;
        groupTaskDrafts = { ...groupTaskDrafts, [targetId]: "" };
        activeGroupTaskDraftInputId = null;
        clearTaskCreateError(projectListGroupTaskCreateTarget(targetId));
      }
    }

    if ((sectionDraftInputActive || sectionDraft.trim()) && !target.closest("[data-add-section-row='true']")) {
      sectionDraft = "";
      sectionDraftInputActive = false;
    }
  }

  function handleProjectWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    const targetElement = projectListEventTargetElement(target);
    if (targetElement) {
      cancelProjectListAddDraftsForOutsideTarget(targetElement);
    }
    if (
      sectionOptionsMenuId
      && targetElement
      && !targetElement.closest("[data-section-options-root='true']")
    ) {
      sectionOptionsMenuId = null;
    }
    if (
      statusMenuTaskId
      && targetElement
      && !targetElement.closest("[data-list-status-menu-root='true']")
    ) {
      statusMenuTaskId = null;
    }
    if (
      priorityMenuTaskId
      && targetElement
      && !targetElement.closest("[data-list-priority-menu-root='true']")
    ) {
      priorityMenuTaskId = null;
    }
    if (
      (startDateMenuTaskId || dueDateMenuTaskId)
      && targetElement
      && !targetElement.closest("[data-list-date-menu-root='true']")
    ) {
      startDateMenuTaskId = null;
      dueDateMenuTaskId = null;
    }
  }

  function taskListGroupTitle(value: string): string {
    if (taskGroupBy === "status") {
      return statuses.find((status) => status.id === value)?.name ?? t("projects.grouping.missingStatus");
    }
    if (taskGroupBy === "priority") {
      return projectPriorityDisplayLabel(value, priorities, t);
    }
    if (taskGroupBy === "due") {
      if (value === "overdue") return t("projects.filters.overdue");
      if (value === "today") return t("projects.filters.today");
      if (value === "week") return t("projects.filters.thisWeek");
      if (value === "later") return t("projects.grouping.laterDue");
      if (value === "earlier") return t("projects.grouping.earlierDue");
      return t("projects.filters.noDueDate");
    }
    if (value === "scheduled") return t("projects.filters.scheduled");
    if (value === "unscheduled") return t("projects.filters.unscheduled");
    return value;
  }

  function taskListColumnLabel(column: ProjectTaskListColumn): string {
    const customFieldId = column.startsWith("custom:") ? column.slice("custom:".length) : undefined;
    if (customFieldId) {
      return projectCustomFields.find((field) => field.id === customFieldId)?.name
        ?? t("projects.columns.customField");
    }
    if (column === "priority") return t("projects.columns.priority");
    if (column === "estimate") return t("projects.columns.estimate");
    if (column === "start") return t("projects.columns.start");
    if (column === "due") return t("projects.columns.due");
    if (column === "scheduled") return t("projects.columns.scheduled");
    if (column === "dependencies") return t("projects.columns.dependencies");
    if (column === "assignee") return t("projects.columns.assignee");
    if (column === "reviewer") return t("projects.columns.reviewer");
    return t("projects.columns.status");
  }

  function tasksForSection(section: ProjectSection): ProjectTask[] {
    return tasks.filter((task) => task.sectionId === section.id && !task.parentTaskId);
  }

  function listDragEnabled(): boolean {
    return taskGroupBy === "section" && taskSortMode === "manual";
  }

  function listSectionDragEnabled(): boolean {
    return taskGroupBy === "section";
  }

  function subtasksForTask(parent: ProjectTask): ProjectTask[] {
    return showArchivedTasks
      ? projects.subtasksForTaskIncludingArchived(parent.id)
      : projects.subtasksForTask(parent.id);
  }

  function taskById(taskId: string): ProjectTask | undefined {
    return allProjectTasks.find((task) => task.id === taskId);
  }

  function sectionById(sectionId: string): ProjectSection | undefined {
    return sections.find((section) => section.id === sectionId);
  }

  function canStartListTaskDrag(task: ProjectTask): boolean {
    return projectListTaskDragAllowed({
      dragEnabled: listDragEnabled(),
      task,
      dropPendingTaskId: listDropPendingTaskId,
    });
  }

  function canStartListSectionDrag(section: ProjectSection): boolean {
    return projectListSectionDragAllowed({
      dragEnabled: listSectionDragEnabled(),
      section,
      dropPendingSectionId: listSectionDropPendingId,
    });
  }

  function listRowDragTargetAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    if (target.closest("[data-list-row-drag-source='true']")) return true;
    return !target.closest("button, input, textarea, select, a, [role='button']");
  }

  function listSectionDragTargetAllowed(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    if (target.closest("[data-list-section-drag-source='true']")) return true;
    return !target.closest("button, textarea, select, a, [role='button']");
  }

  function handleListRowPointerDown(event: PointerEvent, task: ProjectTask): void {
    if (event.button !== 0 || !canStartListTaskDrag(task) || !listRowDragTargetAllowed(event.target)) {
      listRowDragGesture = null;
      return;
    }
    listRowDragGesture = {
      itemId: task.id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      startedAt: Date.now(),
    };
  }

  function clearListRowDragGesture(event?: PointerEvent): void {
    if (event && listRowDragGesture && event.pointerId !== listRowDragGesture.pointerId) return;
    listRowDragGesture = null;
  }

  function handleListSectionPointerDown(event: PointerEvent, section: ProjectSection): void {
    if (
      event.button !== 0
      || !canStartListSectionDrag(section)
      || !listSectionDragTargetAllowed(event.target)
    ) {
      listSectionDragGesture = null;
      return;
    }
    listSectionDragGesture = {
      itemId: section.id,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      startedAt: Date.now(),
    };
  }

  function clearListSectionDragGesture(event?: PointerEvent): void {
    if (event && listSectionDragGesture && event.pointerId !== listSectionDragGesture.pointerId) return;
    listSectionDragGesture = null;
  }

  function listRowDragGestureReady(event: DragEvent, task: ProjectTask): boolean {
    return projectListPointerDragGestureReady({
      gesture: listRowDragGesture,
      itemId: task.id,
      clientX: event.clientX,
      clientY: event.clientY,
      now: Date.now(),
    });
  }

  function listSectionDragGestureReady(event: DragEvent, section: ProjectSection): boolean {
    return projectListPointerDragGestureReady({
      gesture: listSectionDragGesture,
      itemId: section.id,
      clientX: event.clientX,
      clientY: event.clientY,
      now: Date.now(),
    });
  }

  function suppressNextTaskOpen(taskId: string): void {
    suppressedTaskOpenTaskId = taskId;
    window.setTimeout(() => {
      if (suppressedTaskOpenTaskId === taskId) suppressedTaskOpenTaskId = null;
    }, 0);
  }

  function resetListDragTarget(): void {
    listDragOverSectionId = null;
    listDragOverTaskId = null;
    listDragOverPosition = null;
  }

  function resetListSectionDragTarget(): void {
    listSectionDragOverId = null;
    listSectionDragOverPosition = null;
  }

  function listDragTaskId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_LIST_DRAG_MIME) || listDraggingTaskId;
  }

  function listDragSectionId(event: DragEvent): string | null {
    return event.dataTransfer?.getData(PROJECT_LIST_SECTION_DRAG_MIME) || listDraggingSectionId;
  }

  function canDropListTask(task: ProjectTask | undefined, section: ProjectSection): boolean {
    return projectListTaskDropAllowed({
      dragEnabled: listDragEnabled(),
      task,
      targetSection: section,
    });
  }

  function canDropListSection(
    draggedSection: ProjectSection | undefined,
    targetSection: ProjectSection,
  ): boolean {
    return projectListSectionDropAllowed({
      dragEnabled: listSectionDragEnabled(),
      draggedSection,
      targetSection,
    });
  }

  function handleListTaskDragStart(event: DragEvent, task: ProjectTask): void {
    if (!canStartListTaskDrag(task) || !listRowDragGestureReady(event, task)) {
      event.preventDefault();
      return;
    }
    event.stopPropagation();
    listDraggingTaskId = task.id;
    listRowDragGesture = null;
    suppressNextTaskOpen(task.id);
    event.dataTransfer?.setData(PROJECT_LIST_DRAG_MIME, task.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleListSectionDragStart(event: DragEvent, section: ProjectSection): void {
    if (!canStartListSectionDrag(section) || !listSectionDragGestureReady(event, section)) {
      event.preventDefault();
      return;
    }
    event.stopPropagation();
    listDraggingSectionId = section.id;
    listSectionDragGesture = null;
    resetListDragTarget();
    event.dataTransfer?.setData(PROJECT_LIST_SECTION_DRAG_MIME, section.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function handleListTaskDragEnd(): void {
    listDraggingTaskId = null;
    listDropPendingTaskId = null;
    listRowDragGesture = null;
    resetListDragTarget();
  }

  function handleListSectionDragEnd(): void {
    listDraggingSectionId = null;
    listSectionDropPendingId = null;
    listSectionDragGesture = null;
    resetListSectionDragTarget();
  }

  function listRowDropPosition(event: DragEvent): ProjectListDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return projectListDropPositionFromPoint(event.clientY, rect);
  }

  function listSectionDropPosition(event: DragEvent): ProjectListDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return projectListDropPositionFromPoint(event.clientY, rect);
  }

  function handleListRowDragOver(event: DragEvent, section: ProjectSection, task: ProjectTask): void {
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!dragged || !canDropListTask(dragged, section) || dragged.id === task.id) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    listDragOverSectionId = section.id;
    listDragOverTaskId = task.id;
    listDragOverPosition = listRowDropPosition(event);
  }

  function handleListSectionDragOver(event: DragEvent, section: ProjectSection): void {
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!canDropListTask(dragged, section)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    listDragOverSectionId = section.id;
    listDragOverTaskId = null;
    listDragOverPosition = "section";
  }

  function handleListSectionGroupDragOver(event: DragEvent, section: ProjectSection): void {
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (draggedSection && canDropListSection(draggedSection, section) && draggedSection.id !== section.id) {
      event.preventDefault();
      event.stopPropagation();
      if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
      resetListDragTarget();
      listSectionDragOverId = section.id;
      listSectionDragOverPosition = listSectionDropPosition(event);
      return;
    }
    handleListSectionDragOver(event, section);
  }

  async function dropListTask(
    event: DragEvent,
    section: ProjectSection,
    targetTask?: ProjectTask,
    position?: ProjectListDropPosition,
  ): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!dragged || !canDropListTask(dragged, section) || dragged.id === targetTask?.id) {
      resetListDragTarget();
      return;
    }

    const orderedTasks = tasksForSection(section);
    const nextSectionSortOrder = projectListDropSortOrder({
      orderedTasks,
      draggedTaskId: dragged.id,
      overTaskId: targetTask?.id,
      position,
      sortDirection: taskSortDirection,
    });

    if (
      dragged.sectionId === section.id
      && dragged.sectionSortOrder === nextSectionSortOrder
    ) {
      resetListDragTarget();
      return;
    }

    listDropPendingTaskId = dragged.id;
    resetListDragTarget();
    try {
      await projects.updateTask(dragged, {
        sectionId: section.id,
        sectionSortOrder: nextSectionSortOrder,
      });
    } finally {
      listDropPendingTaskId = null;
      listDraggingTaskId = null;
    }
  }

  async function dropListSection(event: DragEvent, targetSection: ProjectSection): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (
      !draggedSection
      || !canDropListSection(draggedSection, targetSection)
      || draggedSection.id === targetSection.id
    ) {
      resetListSectionDragTarget();
      return;
    }

    const orderedSections = sections.filter((section) => !section.archivedAt && !section.hiddenAt);
    const nextSortOrder = projectListSectionDropSortOrder({
      orderedSections,
      draggedSectionId: draggedSection.id,
      overSectionId: targetSection.id,
      position: listSectionDropPosition(event),
    });

    if (draggedSection.sortOrder === nextSortOrder) {
      resetListSectionDragTarget();
      return;
    }

    listSectionDropPendingId = draggedSection.id;
    resetListSectionDragTarget();
    try {
      await projects.updateSection(draggedSection, { sortOrder: nextSortOrder });
    } finally {
      listSectionDropPendingId = null;
      listDraggingSectionId = null;
    }
  }

  async function dropListSectionOrTask(event: DragEvent, section: ProjectSection): Promise<void> {
    const draggedSection = sectionById(listDragSectionId(event) ?? "");
    if (draggedSection && canDropListSection(draggedSection, section)) {
      await dropListSection(event, section);
      return;
    }
    await dropListTask(event, section);
  }

  function listDropMarkerVisible(
    section: ProjectSection,
    task: ProjectTask,
    position: ProjectListDropPosition,
  ): boolean {
    return listDragOverSectionId === section.id
      && listDragOverTaskId === task.id
      && listDragOverPosition === position;
  }

  function listSectionDropMarkerVisible(
    section: ProjectSection,
    position: ProjectListDropPosition,
  ): boolean {
    return listSectionDragOverId === section.id
      && listSectionDragOverPosition === position;
  }

  function statusForTask(task: ProjectTask): ProjectStatus | undefined {
    return projects.statusById(task.statusId);
  }

  function taskSelected(task: ProjectTask): boolean {
    return selectedTaskIdSet.has(task.id);
  }

  function allTasksSelected(groupTasks: ProjectTask[]): boolean {
    return groupTasks.length > 0 && groupTasks.every((task) => selectedTaskIdSet.has(task.id));
  }

  function someTasksSelected(groupTasks: ProjectTask[]): boolean {
    return groupTasks.some((task) => selectedTaskIdSet.has(task.id));
  }

  function toggleTaskSelection(task: ProjectTask): void {
    onSelectedTaskIdsChange(
      taskSelected(task)
        ? selectedTaskIds.filter((taskId) => taskId !== task.id)
        : [...selectedTaskIds, task.id],
    );
  }

  function toggleTaskGroupSelection(groupTasks: ProjectTask[]): void {
    if (groupTasks.length === 0) return;
    const nextIds = new Set(selectedTaskIds);
    if (allTasksSelected(groupTasks)) {
      for (const task of groupTasks) nextIds.delete(task.id);
    } else {
      for (const task of groupTasks) nextIds.add(task.id);
    }
    onSelectedTaskIdsChange(Array.from(nextIds));
  }

  function blockedByDependencies(task: ProjectTask) {
    return projects.dependenciesBlockingTask(task.id);
  }

  function blocksDependencies(task: ProjectTask) {
    return projects.dependenciesBlockedByTask(task.id);
  }

  function tagsForTask(task: ProjectTask): ProjectTag[] {
    return projects.tagsForTask(task.id);
  }

  function visibleTaskTags(task: ProjectTask): ProjectTag[] {
    return tagsForTask(task).slice(0, 3);
  }

  function hiddenTaskTagCount(task: ProjectTask): number {
    return Math.max(0, tagsForTask(task).length - visibleTaskTags(task).length);
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldDisplayValue(task: ProjectTask, field: ProjectCustomField): string | undefined {
    const value = projects.customFieldValueForTask(task.id, field.id);
    const displayText = projectCustomFieldDisplayText(value, field);
    if (displayText !== undefined && field.fieldType !== "checkbox") return displayText;
    if (field.fieldType === "checkbox") {
      if (value?.checkboxValue === undefined) return undefined;
      return value.checkboxValue
        ? t("projects.customFields.checked")
        : t("projects.customFields.unchecked");
    }
    if (!projectCustomFieldUsesOptions(field.fieldType)) return undefined;
    const optionNames = projects.customFieldOptionValuesForTask(task.id, field.id)
      .map((option) => option.name);
    return optionNames.length > 0 ? optionNames.join(", ") : undefined;
  }

  function customFieldValue(task: ProjectTask, field: ProjectCustomField): ProjectCustomFieldValue | undefined {
    return projects.customFieldValueForTask(task.id, field.id);
  }

  function customFieldOptionValues(task: ProjectTask, field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionValuesForTask(task.id, field.id);
  }

  async function saveTaskCustomFieldValueFromList(
    task: ProjectTask,
    field: ProjectCustomField,
    value: Omit<ProjectCustomFieldValueUpdate, "taskId" | "fieldId">,
  ): Promise<void> {
    if (task.archivedAt) return;
    await projects.saveCustomFieldValue({
      taskId: task.id,
      fieldId: field.id,
      ...value,
    });
  }

  function sectionForTask(task: ProjectTask): ProjectSection | undefined {
    return sections.find((section) => section.id === task.sectionId);
  }

  async function setTaskStatusFromList(task: ProjectTask, status: ProjectStatus): Promise<void> {
    if (task.archivedAt || task.statusId === status.id) {
      statusMenuTaskId = null;
      return;
    }
    await projects.setTasksStatus([task], status.id);
    statusMenuTaskId = null;
  }

  async function setTaskPriorityFromList(task: ProjectTask, priority: ProjectPriority): Promise<void> {
    if (task.archivedAt || task.priority === priority) {
      priorityMenuTaskId = null;
      return;
    }
    await projects.setTaskPriority(task, priority);
    priorityMenuTaskId = null;
  }

  async function setTaskStartDateFromList(task: ProjectTask, startDate: string | undefined): Promise<void> {
    const plan = projectListStartDateEditPlan({
      task,
      selectedDate: startDate,
      rangeDateColumnsVisible: listRangeDateColumnsVisible,
    });
    if (plan.patch) {
      await projects.updateTask(task, plan.patch);
    }
    startDateMenuTaskId = null;
    dueDateMenuTaskId = plan.nextOpenMenu === "due" ? task.id : null;
  }

  async function setTaskDueDateFromList(task: ProjectTask, dueDate: string | undefined): Promise<void> {
    const plan = projectListDueDateEditPlan({
      task,
      selectedDate: dueDate,
      rangeDateColumnsVisible: listRangeDateColumnsVisible,
    });
    if (plan.patch) {
      await projects.updateTask(task, plan.patch);
    }
    dueDateMenuTaskId = null;
    startDateMenuTaskId = plan.nextOpenMenu === "start" ? task.id : null;
  }

  async function setTaskStartTimeFromList(task: ProjectTask, startTime: string | undefined): Promise<void> {
    if (task.archivedAt) {
      startDateMenuTaskId = null;
      return;
    }
    if (!task.startDate || task.startTime === startTime) return;
    await projects.updateTask(task, { startTime });
  }

  async function setTaskDueTimeFromList(task: ProjectTask, dueTime: string | undefined): Promise<void> {
    if (task.archivedAt) {
      dueDateMenuTaskId = null;
      return;
    }
    if (!task.dueDate || task.dueTime === dueTime) return;
    await projects.updateTask(task, { dueTime });
  }

  function closeProjectListTaskMenu(menu: ProjectListTaskMenu): void {
    if (menu === "status") statusMenuTaskId = null;
    if (menu === "priority") priorityMenuTaskId = null;
    if (menu === "start") startDateMenuTaskId = null;
    if (menu === "due") dueDateMenuTaskId = null;
  }

  function closeOtherProjectListTaskMenus(menu: ProjectListTaskMenu): void {
    if (menu !== "status") statusMenuTaskId = null;
    if (menu !== "priority") priorityMenuTaskId = null;
    if (menu !== "start") startDateMenuTaskId = null;
    if (menu !== "due") dueDateMenuTaskId = null;
  }

  function projectListTaskMenuTaskId(menu: ProjectListTaskMenu): string | null {
    if (menu === "status") return statusMenuTaskId;
    if (menu === "priority") return priorityMenuTaskId;
    if (menu === "start") return startDateMenuTaskId;
    return dueDateMenuTaskId;
  }

  function setProjectListTaskMenuTaskId(menu: ProjectListTaskMenu, taskId: string | null): void {
    if (menu === "status") statusMenuTaskId = taskId;
    if (menu === "priority") priorityMenuTaskId = taskId;
    if (menu === "start") startDateMenuTaskId = taskId;
    if (menu === "due") dueDateMenuTaskId = taskId;
  }

  function toggleProjectListTaskMenu(menu: ProjectListTaskMenu, taskId: string): void {
    const nextTaskId = projectListTaskMenuTaskId(menu) === taskId ? null : taskId;
    setProjectListTaskMenuTaskId(menu, nextTaskId);
    if (nextTaskId) closeOtherProjectListTaskMenus(menu);
  }

  function groupTaskQuickAddPlan(group: ProjectTaskListGroup): ProjectListGroupQuickAddPlan {
    return projectListGroupQuickAddPlan({
      groupBy: taskGroupBy,
      group,
      statuses,
      priorities,
    });
  }

  function clearTaskCreateError(target: ProjectListTaskCreateTarget): void {
    if (taskCreateErrorTarget !== target) return;
    taskCreateErrorTarget = null;
    taskCreateErrorMessage = null;
  }

  function setTaskCreateError(target: ProjectListTaskCreateTarget, message: string): void {
    taskCreateErrorTarget = target;
    taskCreateErrorMessage = message;
  }

  function taskCreateErrorFor(target: ProjectListTaskCreateTarget): string | null {
    return taskCreateErrorTarget === target ? taskCreateErrorMessage : null;
  }

  function taskCreateFailedMessage(error: unknown): string {
    const message = error instanceof Error ? error.message : String(error);
    return t("projects.tasks.createFailed", message);
  }

  async function createTaskFromDraft(
    target: ProjectListTaskCreateTarget,
    title: string,
    options: {
      sectionId?: string;
      statusId?: string;
      patch?: Partial<Pick<ProjectTask, "priority" | "dueDate">>;
    } = {},
  ): Promise<ProjectTask | undefined> {
    const projectId = selectedProjectId;
    if (!projectId) {
      setTaskCreateError(target, t("projects.tasks.selectProjectFirst"));
      return undefined;
    }
    const displayTitle = title.trim();
    if (!displayTitle) return undefined;
    taskCreatePendingTarget = target;
    clearTaskCreateError(target);
    try {
      const createdTask = await projects.addTask(projectId, displayTitle, options.sectionId, options.statusId);
      if (!createdTask || !options.patch || Object.keys(options.patch).length === 0) return createdTask;
      await projects.updateTask(createdTask, options.patch);
      return projects.taskById(createdTask.id) ?? createdTask;
    } catch (error) {
      console.error("create project task failed", error);
      setTaskCreateError(target, taskCreateFailedMessage(error));
      return undefined;
    } finally {
      if (taskCreatePendingTarget === target) {
        taskCreatePendingTarget = null;
      }
    }
  }

  async function submitSectionTask(sectionId: string): Promise<void> {
    const target = projectListSectionTaskCreateTarget(sectionId);
    const title = (sectionTaskDrafts[sectionId] ?? "").trim();
    if (!title) return;
    const createdTask = await createTaskFromDraft(target, title, { sectionId });
    if (!createdTask) return;
    sectionTaskDrafts = { ...sectionTaskDrafts, [sectionId]: "" };
    onRevealTask(createdTask);
  }

  async function submitGroupTask(group: ProjectTaskListGroup): Promise<void> {
    const quickAddPlan = groupTaskQuickAddPlan(group);
    if (!quickAddPlan.enabled) return;
    const groupKey = projectListGroupTaskDraftKey(taskGroupBy, group);
    const target = projectListGroupTaskCreateTarget(groupKey);
    const title = (groupTaskDrafts[groupKey] ?? "").trim();
    if (!title) return;
    const createdTask = await createTaskFromDraft(target, title, {
      statusId: quickAddPlan.statusId,
      patch: quickAddPlan.patch,
    });
    if (!createdTask) return;
    groupTaskDrafts = { ...groupTaskDrafts, [groupKey]: "" };
    onRevealTask(createdTask);
  }

  async function submitSection(): Promise<void> {
    const name = sectionDraft.trim();
    if (!selectedProjectId || !name) return;
    await projects.addSection(selectedProjectId, name);
    sectionDraft = "";
  }

  function sectionNameDraft(section: ProjectSection): string {
    return sectionNameDrafts[section.id] ?? section.name;
  }

  function sectionDraftDirty(section: ProjectSection): boolean {
    return sectionNameDraft(section) !== section.name;
  }

  function sectionDraftSaveable(section: ProjectSection): boolean {
    return sectionDraftDirty(section)
      && sectionNameDraft(section).trim().length > 0
      && !section.hiddenAt
      && !section.archivedAt;
  }

  async function saveSection(section: ProjectSection): Promise<void> {
    const name = sectionNameDraft(section).trim();
    if (!name) return;
    await projects.updateSection(section, { name });
    sectionNameDrafts = {
      ...sectionNameDrafts,
      [section.id]: name,
    };
  }

  async function hideSection(section: ProjectSection): Promise<void> {
    await projects.hideSection(section);
  }

  async function archiveSection(section: ProjectSection): Promise<void> {
    await projects.archiveSection(section);
  }

  async function restoreSection(section: ProjectSection): Promise<void> {
    await projects.restoreSection(section);
  }

  async function toggleSectionCollapsed(section: ProjectSection): Promise<void> {
    await projects.updateSection(section, { collapsed: !section.collapsed });
  }

  function openTaskDetail(task: ProjectTask): void {
    if (suppressedTaskOpenTaskId === task.id) {
      suppressedTaskOpenTaskId = null;
      return;
    }
    onOpenTask(task);
  }

  function estimateLabel(minutes: number): string {
    return t("projects.list.estimateMinutes", minutes);
  }

  function scheduledLinksForTask(taskId: string) {
    return projects.eventLinksForTask(taskId).filter((link) => link.linkKind === "scheduled");
  }

  function scheduledLabel(taskId: string): string | null {
    const count = scheduledLinksForTask(taskId).length;
    const starts = projects.eventLinksForTask(taskId)
      .filter((link) => link.linkKind === "scheduled")
      .map((link) => calendar.rawBlocks.find((event) => event.id === link.eventId)?.start)
      .filter((start): start is string => Boolean(start))
      .sort((a, b) => a.localeCompare(b));
    const now = new Date();
    const date = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")} ${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`;
    const nextStart = starts.find((start) => start >= date) ?? starts[0];
    if (nextStart) return t("projects.schedule.nextScheduled", nextStart.slice(0, 16), count);
    return count > 0 ? t("projects.schedule.scheduledCount", count) : null;
  }

  function handleProjectListScroll(event: Event): void {
    syncProjectListCounterScroll();
    const target = event.currentTarget as HTMLElement;
    if (target.scrollHeight - target.scrollTop - target.clientHeight < 600) onNeedMore();
  }
</script>

<svelte:window
  onkeydown={handleProjectListHorizontalKeydown}
  onpointerdown={handleProjectWindowPointerDown}
  onpointermove={handleProjectListColumnResizePointerMove}
  onpointerup={(event) => finishProjectListColumnResize(event, true)}
  onpointercancel={(event) => finishProjectListColumnResize(event, false)}
/>

<div
  bind:this={projectViewScrollContainer}
  class="project-list-scroll h-full min-h-0 overflow-auto"
  onscroll={handleProjectListScroll}
>
  <div class="flex min-h-full flex-col gap-5 p-3">
    {#if taskGroupBy === "section"}
      {#each sections as section (section.id)}
        {@const sectionTasks = tasksForSection(section)}
        <ProjectListSectionBlock
            {section}
            {sectionTasks}
            gridTemplate={taskListGridTemplate}
            gridMinWidth={taskListGridMinWidth}
            {taskListColumns}
            {taskListColumnLabel}
            {statuses}
            {priorities}
            {projectCustomFields}
            {selectedTaskId}
            {statusMenuTaskId}
            {priorityMenuTaskId}
            {startDateMenuTaskId}
            {dueDateMenuTaskId}
            draggingTaskId={listDraggingTaskId}
            dropPendingTaskId={listDropPendingTaskId}
            sectionDragOver={listDragOverSectionId === section.id}
            sectionDragging={listDraggingSectionId === section.id}
            sectionDropPending={listSectionDropPendingId === section.id}
            sectionDropMarkerBefore={listSectionDropMarkerVisible(section, "before")}
            sectionDropMarkerAfter={listSectionDropMarkerVisible(section, "after")}
            taskSectionDropMarkerVisible={listDragOverSectionId === section.id && listDragOverPosition === "section"}
            theme={theme.current}
            allTasksSelected={allTasksSelected(sectionTasks)}
            partiallySelected={someTasksSelected(sectionTasks) && !allTasksSelected(sectionTasks)}
            canDragSection={canStartListSectionDrag(section)}
            sectionDraft={sectionNameDraft(section)}
            sectionDraftDirty={sectionDraftDirty(section)}
            sectionDraftSaveable={sectionDraftSaveable(section)}
            sectionOptionsMenuOpen={sectionOptionsMenuId === section.id}
            taskDraft={sectionTaskDrafts[section.id] ?? ""}
            taskAddLabel={t("projects.list.addTaskInSection", section.name)}
            taskAddActive={activeSectionTaskDraftInputId === section.id}
            taskAddPending={taskCreatePendingTarget !== null}
            taskAddError={taskCreateErrorFor(projectListSectionTaskCreateTarget(section.id))}
            {statusForTask}
            {subtasksForTask}
            {scheduledLabel}
            {visibleTaskTags}
            {hiddenTaskTagCount}
            {blockedByDependencies}
            {blocksDependencies}
            {taskSelected}
            {estimateLabel}
            {customFieldDisplayValue}
            {customFieldOptions}
            {customFieldValue}
            {customFieldOptionValues}
            canStartTaskDrag={canStartListTaskDrag}
            taskDropMarkerVisible={(task, position) => listDropMarkerVisible(section, task, position)}
            onSectionDragOver={(event) => handleListSectionGroupDragOver(event, section)}
            onSectionDrop={(event) => { void dropListSectionOrTask(event, section); }}
            onToggleSectionSelection={() => toggleTaskGroupSelection(sectionTasks)}
            onToggleSectionCollapsed={() => toggleSectionCollapsed(section)}
            onSectionDraftChange={(value) => {
              sectionNameDrafts = {
                ...sectionNameDrafts,
                [section.id]: value,
              };
            }}
            onSaveSection={() => saveSection(section)}
            onToggleSectionOptionsMenu={() => {
              sectionOptionsMenuId = sectionOptionsMenuId === section.id ? null : section.id;
            }}
            onRestoreSection={() => {
              sectionOptionsMenuId = null;
              return restoreSection(section);
            }}
            onHideSection={() => {
              sectionOptionsMenuId = null;
              return hideSection(section);
            }}
            onArchiveSection={() => {
              sectionOptionsMenuId = null;
              return archiveSection(section);
            }}
            onSectionPointerDown={(event) => handleListSectionPointerDown(event, section)}
            onSectionPointerUp={clearListSectionDragGesture}
            onSectionPointerCancel={clearListSectionDragGesture}
            onSectionDragStart={(event) => handleListSectionDragStart(event, section)}
            onSectionDragEnd={handleListSectionDragEnd}
            onResizePointerDown={startProjectListColumnResize}
            onResizeDoubleClick={handleProjectListColumnResizeDoubleClick}
            onResizeKeydown={handleProjectListColumnResizeKeydown}
            onSaveCustomFieldValue={saveTaskCustomFieldValueFromList}
            onToggleTaskSelection={toggleTaskSelection}
            onOpenTask={openTaskDetail}
            onTaskPointerDown={(event, task) => handleListRowPointerDown(event, task)}
            onTaskPointerUp={clearListRowDragGesture}
            onTaskPointerCancel={clearListRowDragGesture}
            onTaskDragStart={(event, task) => handleListTaskDragStart(event, task)}
            onTaskDragEnd={handleListTaskDragEnd}
            onTaskDragOver={(event, task) => handleListRowDragOver(event, section, task)}
            onTaskDrop={(event, task) => { void dropListTask(event, section, task, listRowDropPosition(event)); }}
            onToggleStatusMenu={(task) => toggleProjectListTaskMenu("status", task.id)}
            onSetStatus={(task, nextStatus) => { void setTaskStatusFromList(task, nextStatus); }}
            onTogglePriorityMenu={(task) => toggleProjectListTaskMenu("priority", task.id)}
            onSetPriority={(task, priority) => { void setTaskPriorityFromList(task, priority); }}
            onToggleStartDateMenu={(task) => toggleProjectListTaskMenu("start", task.id)}
            onCloseStartDateMenu={() => closeProjectListTaskMenu("start")}
            onSetStartDate={(task, startDate) => { void setTaskStartDateFromList(task, startDate); }}
            onClearStartDate={(task) => { void setTaskStartDateFromList(task, undefined); }}
            onSetStartTime={(task, startTime) => { void setTaskStartTimeFromList(task, startTime); }}
            onClearStartTime={(task) => { void setTaskStartTimeFromList(task, undefined); }}
            onToggleDueDateMenu={(task) => toggleProjectListTaskMenu("due", task.id)}
            onCloseDueDateMenu={() => closeProjectListTaskMenu("due")}
            onSetDueDate={(task, dueDate) => { void setTaskDueDateFromList(task, dueDate); }}
            onClearDueDate={(task) => { void setTaskDueDateFromList(task, undefined); }}
            onSetDueTime={(task, dueTime) => { void setTaskDueTimeFromList(task, dueTime); }}
            onClearDueTime={(task) => { void setTaskDueTimeFromList(task, undefined); }}
            onToggleSubtaskDone={(subtask) => { void projects.toggleTaskDone(subtask); }}
            onTaskDraftChange={(value) => {
                sectionTaskDrafts = {
                  ...sectionTaskDrafts,
                  [section.id]: value,
                };
            }}
            onTaskAddActiveChange={(active) => {
                if (active) {
                  activeSectionTaskDraftInputId = section.id;
                } else if (activeSectionTaskDraftInputId === section.id) {
                  activeSectionTaskDraftInputId = null;
                }
            }}
            onSubmitTask={() => submitSectionTask(section.id)}
        />
      {/each}
      <ProjectListSectionAddRow
        gridTemplate={taskListGridTemplate}
        gridMinWidth={taskListGridMinWidth}
        label={t("projects.header.addSection")}
        draft={sectionDraft}
        active={sectionDraftInputActive}
        onDraftChange={(value) => {
          sectionDraft = value;
        }}
        onActiveChange={(active) => {
          sectionDraftInputActive = active;
        }}
        onSubmit={submitSection}
      />
    {:else}
      {#each listTaskGroups as group (group.id)}
        {@const groupKey = projectListGroupTaskDraftKey(taskGroupBy, group)}
        {@const groupTitle = taskListGroupTitle(group.value)}
        {@const groupQuickAddPlan = groupTaskQuickAddPlan(group)}
        {@const groupCreateTarget = projectListGroupTaskCreateTarget(groupKey)}
        <section
          class="flex flex-col gap-0"
          style={`min-width: max(100%, ${taskListGridMinWidth});`}
        >
          <ProjectListGroupHeader
            title={groupTitle}
            gridTemplate={taskListGridTemplate}
            gridMinWidth={taskListGridMinWidth}
            taskCount={group.tasks.length}
            allSelected={allTasksSelected(group.tasks)}
            partiallySelected={someTasksSelected(group.tasks) && !allTasksSelected(group.tasks)}
            onToggleSelection={() => toggleTaskGroupSelection(group.tasks)}
          />
          <ProjectListColumnHeaders
            mode="group"
            gridTemplate={taskListGridTemplate}
            gridMinWidth={taskListGridMinWidth}
            {taskListColumns}
            {taskListColumnLabel}
            onResizePointerDown={startProjectListColumnResize}
            onResizeDoubleClick={handleProjectListColumnResizeDoubleClick}
            onResizeKeydown={handleProjectListColumnResizeKeydown}
          />
          <ProjectListTaskRows
            tasks={group.tasks}
            {statuses}
            {priorities}
            {taskListColumns}
            {projectCustomFields}
            {selectedTaskId}
            {statusMenuTaskId}
            {priorityMenuTaskId}
            {startDateMenuTaskId}
            {dueDateMenuTaskId}
            gridTemplate={taskListGridTemplate}
            gridMinWidth={taskListGridMinWidth}
            theme={theme.current}
            {statusForTask}
            {subtasksForTask}
            {scheduledLabel}
            {visibleTaskTags}
            {hiddenTaskTagCount}
            {blockedByDependencies}
            {blocksDependencies}
            {taskSelected}
            sectionNameForTask={(task) => sectionForTask(task)?.name}
            {estimateLabel}
            {customFieldDisplayValue}
            {customFieldOptions}
            {customFieldValue}
            {customFieldOptionValues}
            onSaveCustomFieldValue={saveTaskCustomFieldValueFromList}
            onToggleTaskSelection={toggleTaskSelection}
            onOpenTask={openTaskDetail}
            onToggleStatusMenu={(task) => toggleProjectListTaskMenu("status", task.id)}
            onSetStatus={(task, nextStatus) => { void setTaskStatusFromList(task, nextStatus); }}
            onTogglePriorityMenu={(task) => toggleProjectListTaskMenu("priority", task.id)}
            onSetPriority={(task, priority) => { void setTaskPriorityFromList(task, priority); }}
            onToggleStartDateMenu={(task) => toggleProjectListTaskMenu("start", task.id)}
            onCloseStartDateMenu={() => closeProjectListTaskMenu("start")}
            onSetStartDate={(task, startDate) => { void setTaskStartDateFromList(task, startDate); }}
            onClearStartDate={(task) => { void setTaskStartDateFromList(task, undefined); }}
            onSetStartTime={(task, startTime) => { void setTaskStartTimeFromList(task, startTime); }}
            onClearStartTime={(task) => { void setTaskStartTimeFromList(task, undefined); }}
            onToggleDueDateMenu={(task) => toggleProjectListTaskMenu("due", task.id)}
            onCloseDueDateMenu={() => closeProjectListTaskMenu("due")}
            onSetDueDate={(task, dueDate) => { void setTaskDueDateFromList(task, dueDate); }}
            onClearDueDate={(task) => { void setTaskDueDateFromList(task, undefined); }}
            onSetDueTime={(task, dueTime) => { void setTaskDueTimeFromList(task, dueTime); }}
            onClearDueTime={(task) => { void setTaskDueTimeFromList(task, undefined); }}
            onToggleSubtaskDone={(subtask) => { void projects.toggleTaskDone(subtask); }}
          />
            {#if groupQuickAddPlan.enabled}
              <ProjectListTaskAddRow
                mode="group"
                rowId={groupKey}
                gridTemplate={taskListGridTemplate}
                gridMinWidth={taskListGridMinWidth}
                label={t("projects.list.addTaskInSection", groupTitle)}
                draft={groupTaskDrafts[groupKey] ?? ""}
                active={activeGroupTaskDraftInputId === groupKey}
                pending={taskCreatePendingTarget !== null}
                error={taskCreateErrorFor(groupCreateTarget)}
                onDraftChange={(value) => {
                  groupTaskDrafts = {
                    ...groupTaskDrafts,
                    [groupKey]: value,
                  };
                }}
                onActiveChange={(active) => {
                  if (active) {
                    activeGroupTaskDraftInputId = groupKey;
                  } else if (activeGroupTaskDraftInputId === groupKey) {
                    activeGroupTaskDraftInputId = null;
                  }
                }}
                onSubmit={() => submitGroupTask(group)}
              />
            {/if}
        </section>
      {/each}
    {/if}
    {#if tasks.length === 0 && allProjectTasks.length > 0}
      <div class="rounded-md border border-dashed border-border px-2 py-3 text-[0.8rem] text-muted-foreground">
        {t("projects.filters.noMatchingTasks")}
      </div>
    {/if}
  </div>
</div>
<ProjectListScrollbars
  scrollContainer={projectViewScrollContainer}
  getMaxScrollLeft={projectListMaxHorizontalScrollLeft}
  onScrollPositionChange={setProjectListHorizontalScroll}
/>

<style>
  :global(.project-list-scroll .project-list-divider) {
    position: relative;
    --project-list-divider-left: 3.25rem;
    --project-list-divider-right: 0.25rem;
  }

  :global(.project-list-scroll .project-list-divider)::after {
    position: absolute;
    right: var(--project-list-divider-right);
    bottom: 0;
    left: var(--project-list-divider-left);
    height: 0;
    border-bottom: 1px solid var(--cal-gridline);
    content: "";
    pointer-events: none;
  }

  :global(.project-list-scroll .project-list-sticky-row.project-list-divider)::after {
    transform: translateX(var(--project-list-scroll-left-negative, 0px));
    will-change: transform;
  }

  :global(.project-list-scroll .project-list-inline-divider) {
    position: relative;
    --project-list-divider-left: 3.25rem;
    --project-list-divider-right: 0.25rem;
  }

  :global(.project-list-scroll .project-list-inline-divider)::before {
    position: absolute;
    top: 0;
    right: var(--project-list-divider-right);
    left: var(--project-list-divider-left);
    height: 0;
    border-bottom: 1px solid var(--cal-gridline);
    content: "";
    pointer-events: none;
  }

  :global(.project-list-scroll .project-list-sticky-row) {
    position: relative;
    z-index: 1;
    transform: translateX(var(--project-list-scroll-left, 0px));
    background-color: var(--cal-bg);
    will-change: transform;
  }

  :global(.project-list-scroll .project-list-add-row-caret) {
    animation: project-list-caret-blink 1s step-end infinite;
  }

  @keyframes project-list-caret-blink {
    0%,
    49% {
      opacity: 1;
    }

    50%,
    100% {
      opacity: 0;
    }
  }

  .project-list-scroll {
    overflow-x: hidden;
    overflow-y: auto;
    padding-right: 0.5rem;
    padding-bottom: 0.5rem;
    scrollbar-width: none;
  }

  .project-list-scroll::-webkit-scrollbar {
    display: none;
  }
</style>
