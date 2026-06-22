<script lang="ts">
  import { tick } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CirclePlus from "@lucide/svelte/icons/circle-plus";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Plus from "@lucide/svelte/icons/plus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    selectDateRangeEnd,
    selectDateRangeStart,
  } from "$lib/calendar/date-range-selection";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";
  import { projectPriorityLabel } from "$lib/projects/project-display";
  import {
    clampProjectTaskListManualColumnWidth,
    projectTaskListDoubleClickColumnWidthRem,
    projectTaskListResizableColumnWidthRem,
    projectTaskListGridMinWidth,
    projectTaskListGridTemplate,
    type ProjectTaskListColumnWidths,
    type ProjectTaskListResizableColumn,
  } from "$lib/projects/project-list-view";
  import {
    projectListDropSortOrder,
    projectListSectionDropSortOrder,
    type ProjectListDropPosition,
  } from "$lib/projects/list-drag";
  import {
    PROJECT_PRIORITIES,
    type ProjectCustomField,
    type ProjectCustomFieldOption,
    type ProjectLabel,
    type ProjectPriority,
    type ProjectSection,
    type ProjectStatus,
    type ProjectTask,
    type ProjectTaskGroupMode,
    type ProjectTaskListColumn,
    type ProjectTaskSortDirection,
    type ProjectTaskSortMode,
  } from "$lib/projects/types";
  import type { ProjectTaskListGroup } from "$lib/projects/task-view";
  import ProjectListScrollbars from "./ProjectListScrollbars.svelte";
  import ProjectListTaskRow from "./ProjectListTaskRow.svelte";

  type ProjectListAddRowInputFocusOptions = {
    selector: string;
    beforeFocus?: () => void;
  };
  type TaskCreateTarget = `section:${string}`;
  interface ListRowDragGesture {
    taskId: string;
    pointerId: number;
    startX: number;
    startY: number;
    startedAt: number;
  }
  interface ListSectionDragGesture {
    sectionId: string;
    pointerId: number;
    startX: number;
    startY: number;
    startedAt: number;
  }
  interface ListColumnResizeGesture {
    column: ProjectTaskListResizableColumn;
    pointerId: number;
    startClientX: number;
    startWidthRem: number;
    rootFontSizePx: number;
    widthsAtStart: ProjectTaskListColumnWidths;
    draftWidths: ProjectTaskListColumnWidths;
    moved: boolean;
  }

  let {
    selectedProjectId,
    sections,
    statuses,
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
  }: {
    selectedProjectId: string | null;
    sections: ProjectSection[];
    statuses: ProjectStatus[];
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
  } = $props();

  const projects = getProjects();
  const calendar = getCalendar();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_LIST_DRAG_MIME = "application/x-ganbaru-project-list-task";
  const PROJECT_LIST_SECTION_DRAG_MIME = "application/x-ganbaru-project-list-section";
  const LIST_ROW_DRAG_THRESHOLD_PX = 4;
  const LIST_ROW_DRAG_HOLD_MS = 120;
  const PROJECT_LIST_KEYBOARD_SCROLL_PX = 48;

  let taskCreatePendingTarget = $state<TaskCreateTarget | null>(null);
  let taskCreateErrorTarget = $state<TaskCreateTarget | null>(null);
  let taskCreateErrorMessage = $state<string | null>(null);
  let sectionDraft = $state("");
  let sectionTaskDrafts = $state<Record<string, string>>({});
  let activeSectionTaskDraftInputId = $state<string | null>(null);
  let sectionDraftInputActive = $state(false);
  let listDraggingTaskId = $state<string | null>(null);
  let listDragOverSectionId = $state<string | null>(null);
  let listDragOverTaskId = $state<string | null>(null);
  let listDragOverPosition = $state<ProjectListDropPosition | "section" | null>(null);
  let listDropPendingTaskId = $state<string | null>(null);
  let listRowDragGesture = $state<ListRowDragGesture | null>(null);
  let listDraggingSectionId = $state<string | null>(null);
  let listSectionDragOverId = $state<string | null>(null);
  let listSectionDragOverPosition = $state<ProjectListDropPosition | null>(null);
  let listSectionDropPendingId = $state<string | null>(null);
  let listSectionDragGesture = $state<ListSectionDragGesture | null>(null);
  let suppressedTaskOpenTaskId = $state<string | null>(null);
  let sectionNameDrafts = $state<Record<string, string>>({});
  let sectionOptionsMenuId = $state<string | null>(null);
  let statusMenuTaskId = $state<string | null>(null);
  let priorityMenuTaskId = $state<string | null>(null);
  let startDateMenuTaskId = $state<string | null>(null);
  let dueDateMenuTaskId = $state<string | null>(null);
  let projectViewScrollContainer = $state<HTMLDivElement | null>(null);
  let listColumnResizeGesture = $state<ListColumnResizeGesture | null>(null);

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
    priorityLabel: (priority: ProjectPriority) => projectPriorityLabel(priority, t),
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

  function roundProjectListColumnWidthRem(width: number): number {
    return Math.round(width * 100) / 100;
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
    const draftWidths = {
      ...widthsAtStart,
      [column]: roundProjectListColumnWidthRem(clampProjectTaskListManualColumnWidth(column, startWidthRem)),
    };
    listColumnResizeGesture = {
      column,
      pointerId: event.pointerId,
      startClientX: event.clientX,
      startWidthRem,
      rootFontSizePx,
      widthsAtStart,
      draftWidths,
      moved: false,
    };
    trigger.setPointerCapture(event.pointerId);
  }

  function handleProjectListColumnResizePointerMove(event: PointerEvent): void {
    const gesture = listColumnResizeGesture;
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    event.preventDefault();
    const deltaRem = (event.clientX - gesture.startClientX) / gesture.rootFontSizePx;
    const nextWidth = roundProjectListColumnWidthRem(
      clampProjectTaskListManualColumnWidth(gesture.column, gesture.startWidthRem + deltaRem),
    );
    const draftWidths = {
      ...gesture.widthsAtStart,
      [gesture.column]: nextWidth,
    };
    listColumnResizeGesture = {
      ...gesture,
      draftWidths,
      moved: gesture.moved || Math.abs(event.clientX - gesture.startClientX) >= 1,
    };
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
    const nextWidths = { ...effectiveTaskListColumnWidths };
    const nextWidth = projectTaskListDoubleClickColumnWidthRem(column, taskListGridInput);
    if (nextWidth === undefined) {
      delete nextWidths[column];
    } else {
      nextWidths[column] = roundProjectListColumnWidthRem(nextWidth);
    }
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
    const step = event.shiftKey ? 2 : 0.5;
    const currentWidth = projectTaskListResizableColumnWidthRem(column, taskListGridInput);
    const nextWidth = roundProjectListColumnWidthRem(
      clampProjectTaskListManualColumnWidth(column, currentWidth + direction * step),
    );
    onTaskListColumnWidthsChange({
      ...effectiveTaskListColumnWidths,
      [column]: nextWidth,
    }, { persist: true });
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
      || sectionDraftInputActive
      || sectionDraft.trim().length > 0;
  }

  function cancelActiveProjectListAddDrafts(): void {
    if (activeSectionTaskDraftInputId) {
      const targetId = activeSectionTaskDraftInputId;
      sectionTaskDrafts = { ...sectionTaskDrafts, [targetId]: "" };
      activeSectionTaskDraftInputId = null;
      clearTaskCreateError(sectionTaskCreateTarget(targetId));
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
        clearTaskCreateError(sectionTaskCreateTarget(targetId));
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

  function projectListAddRowClickShouldFocus(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return true;
    return !target.closest("input, textarea, select, button, a, [contenteditable='true'], [role='textbox']");
  }

  function focusProjectListTextInput(input: HTMLInputElement): void {
    input.focus({ preventScroll: true });
    const caretPosition = input.value.length;
    input.setSelectionRange(caretPosition, caretPosition);
    requestAnimationFrame(() => {
      input.focus({ preventScroll: true });
      input.setSelectionRange(caretPosition, caretPosition);
    });
  }

  function focusProjectListInputFromRow(node: HTMLElement, options: ProjectListAddRowInputFocusOptions, event: MouseEvent): void {
    if (event.button !== 0) return;
    const target = event.target;
    if (!(target instanceof Element)) return;
    if (!projectListAddRowClickShouldFocus(target)) return;

    options.beforeFocus?.();
    void tick().then(() => {
      const input = node.querySelector<HTMLInputElement>(options.selector);
      if (!input) return;
      focusProjectListTextInput(input);
    });
  }

  function projectListAddRowInputFocus(node: HTMLElement, options: ProjectListAddRowInputFocusOptions): {
    update: (nextOptions: ProjectListAddRowInputFocusOptions) => void;
    destroy: () => void;
  } {
    let currentOptions = options;
    const handleClick = (event: MouseEvent): void => {
      focusProjectListInputFromRow(node, currentOptions, event);
    };

    node.addEventListener("click", handleClick);

    return {
      update(nextOptions: ProjectListAddRowInputFocusOptions) {
        currentOptions = nextOptions;
      },
      destroy() {
        node.removeEventListener("click", handleClick);
      },
    };
  }

  function taskListGroupTitle(value: string): string {
    if (taskGroupBy === "status") {
      return statuses.find((status) => status.id === value)?.name ?? t("projects.grouping.missingStatus");
    }
    if (taskGroupBy === "priority" && PROJECT_PRIORITIES.includes(value as ProjectPriority)) {
      return projectPriorityLabel(value as ProjectPriority, t);
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
    return listDragEnabled()
      && !task.archivedAt
      && !task.parentTaskId
      && listDropPendingTaskId === null;
  }

  function canStartListSectionDrag(section: ProjectSection): boolean {
    return listSectionDragEnabled()
      && !section.archivedAt
      && !section.hiddenAt
      && listSectionDropPendingId === null;
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
      taskId: task.id,
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
      sectionId: section.id,
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
    const gesture = listRowDragGesture;
    if (!gesture || gesture.taskId !== task.id) return false;
    const distance = Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY);
    const elapsed = Date.now() - gesture.startedAt;
    return distance >= LIST_ROW_DRAG_THRESHOLD_PX
      || (elapsed >= LIST_ROW_DRAG_HOLD_MS && distance >= 1);
  }

  function listSectionDragGestureReady(event: DragEvent, section: ProjectSection): boolean {
    const gesture = listSectionDragGesture;
    if (!gesture || gesture.sectionId !== section.id) return false;
    const distance = Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY);
    const elapsed = Date.now() - gesture.startedAt;
    return distance >= LIST_ROW_DRAG_THRESHOLD_PX
      || (elapsed >= LIST_ROW_DRAG_HOLD_MS && distance >= 1);
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

  function canDropListTask(task: ProjectTask | undefined, section: ProjectSection): task is ProjectTask {
    return listDragEnabled()
      && !!task
      && !task.archivedAt
      && !task.parentTaskId
      && !section.archivedAt
      && !section.hiddenAt
      && task.projectId === section.projectId;
  }

  function canDropListSection(
    draggedSection: ProjectSection | undefined,
    targetSection: ProjectSection,
  ): draggedSection is ProjectSection {
    return listSectionDragEnabled()
      && !!draggedSection
      && !draggedSection.archivedAt
      && !draggedSection.hiddenAt
      && !targetSection.archivedAt
      && !targetSection.hiddenAt
      && draggedSection.projectId === targetSection.projectId;
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
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function listSectionDropPosition(event: DragEvent): ProjectListDropPosition {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    return event.clientY >= rect.top + rect.height / 2 ? "after" : "before";
  }

  function handleListRowDragOver(event: DragEvent, section: ProjectSection, task: ProjectTask): void {
    const dragged = taskById(listDragTaskId(event) ?? "");
    if (!canDropListTask(dragged, section) || dragged.id === task.id) return;
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
    if (canDropListSection(draggedSection, section) && draggedSection.id !== section.id) {
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
    if (!canDropListTask(dragged, section) || dragged.id === targetTask?.id) {
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
    if (!canDropListSection(draggedSection, targetSection) || draggedSection.id === targetSection.id) {
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
    if (canDropListSection(draggedSection, section)) {
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

  function labelsForTask(task: ProjectTask): ProjectLabel[] {
    return projects.labelsForTask(task.id);
  }

  function visibleTaskLabels(task: ProjectTask): ProjectLabel[] {
    return labelsForTask(task).slice(0, 3);
  }

  function hiddenTaskLabelCount(task: ProjectTask): number {
    return Math.max(0, labelsForTask(task).length - visibleTaskLabels(task).length);
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldDisplayValue(task: ProjectTask, field: ProjectCustomField): string | undefined {
    const value = projects.customFieldValueForTask(task.id, field.id);
    if (field.fieldType === "text" || field.fieldType === "url") {
      const text = value?.textValue?.trim();
      return text || undefined;
    }
    if (field.fieldType === "number") {
      const number = value?.numberValue;
      if (number === undefined) return undefined;
      return Number.isInteger(number) ? number.toFixed(0) : number.toString();
    }
    if (field.fieldType === "date") {
      return value?.dateValue || undefined;
    }
    if (field.fieldType === "checkbox") {
      if (value?.checkboxValue === undefined) return undefined;
      return value.checkboxValue
        ? t("projects.customFields.checked")
        : t("projects.customFields.unchecked");
    }
    const optionNames = projects.customFieldOptionValuesForTask(task.id, field.id)
      .map((option) => option.name);
    return optionNames.length > 0 ? optionNames.join(", ") : undefined;
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
    if (task.archivedAt) {
      startDateMenuTaskId = null;
      return;
    }
    if (!startDate) {
      if (task.startDate !== undefined) {
        await projects.updateTask(task, { startDate });
      }
      startDateMenuTaskId = null;
      return;
    }
    const nextRange = selectDateRangeStart({
      selectedDate: startDate,
      startDate: task.startDate,
      endDate: task.dueDate,
    });
    if (task.startDate === nextRange.startDate && task.dueDate === nextRange.endDate) {
      startDateMenuTaskId = null;
      return;
    }
    const shouldPromptForDueDate = listRangeDateColumnsVisible && !task.dueDate && !nextRange.endDate;
    await projects.updateTask(task, {
      startDate: nextRange.startDate,
      dueDate: nextRange.endDate,
    });
    startDateMenuTaskId = null;
    dueDateMenuTaskId = shouldPromptForDueDate ? task.id : null;
  }

  async function setTaskDueDateFromList(task: ProjectTask, dueDate: string | undefined): Promise<void> {
    if (task.archivedAt) {
      dueDateMenuTaskId = null;
      return;
    }
    if (!dueDate) {
      if (task.dueDate !== undefined) {
        await projects.updateTask(task, { dueDate });
      }
      dueDateMenuTaskId = null;
      return;
    }
    const nextRange = selectDateRangeEnd({
      selectedDate: dueDate,
      startDate: task.startDate,
      endDate: task.dueDate,
    });
    if (task.startDate === nextRange.startDate && task.dueDate === nextRange.endDate) {
      dueDateMenuTaskId = null;
      return;
    }
    const shouldPromptForStartDate = listRangeDateColumnsVisible && !task.startDate && !nextRange.startDate;
    await projects.updateTask(task, {
      startDate: nextRange.startDate,
      dueDate: nextRange.endDate,
    });
    dueDateMenuTaskId = null;
    startDateMenuTaskId = shouldPromptForStartDate ? task.id : null;
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

  function sectionTaskCreateTarget(sectionId: string): TaskCreateTarget {
    return `section:${sectionId}`;
  }

  function clearTaskCreateError(target: TaskCreateTarget): void {
    if (taskCreateErrorTarget !== target) return;
    taskCreateErrorTarget = null;
    taskCreateErrorMessage = null;
  }

  function setTaskCreateError(target: TaskCreateTarget, message: string): void {
    taskCreateErrorTarget = target;
    taskCreateErrorMessage = message;
  }

  function taskCreateErrorFor(target: TaskCreateTarget): string | null {
    return taskCreateErrorTarget === target ? taskCreateErrorMessage : null;
  }

  function taskCreateFailedMessage(error: unknown): string {
    const message = error instanceof Error ? error.message : String(error);
    return t("projects.tasks.createFailed", message);
  }

  async function createTaskFromDraft(target: TaskCreateTarget, title: string, sectionId?: string): Promise<ProjectTask | undefined> {
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
      return await projects.addTask(projectId, displayTitle, sectionId);
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
    const target = sectionTaskCreateTarget(sectionId);
    const title = (sectionTaskDrafts[sectionId] ?? "").trim();
    if (!title) return;
    const createdTask = await createTaskFromDraft(target, title, sectionId);
    if (!createdTask) return;
    sectionTaskDrafts = { ...sectionTaskDrafts, [sectionId]: "" };
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
</script>

{#snippet listColumnHeaderCell(label: string, column: ProjectTaskListResizableColumn)}
  <div class="project-list-header-cell relative flex min-h-11 min-w-0 items-center self-stretch rounded-md px-2 py-1 hover:bg-accent/20">
    <span class="relative z-10 truncate">{label}</span>
    <button
      type="button"
      class="project-list-column-resize-hit"
      aria-label={t("projects.columns.resizeColumn", label)}
      data-app-tooltip-disabled="true"
      onpointerdown={(event) => startProjectListColumnResize(event, column)}
      ondblclick={(event) => handleProjectListColumnResizeDoubleClick(event, column)}
      onkeydown={(event) => handleProjectListColumnResizeKeydown(event, column)}
    ></button>
  </div>
{/snippet}

{#snippet listAddColumnHeaderCell()}
  <div
    class="flex min-h-11 min-w-0 items-center justify-center self-stretch rounded-md text-muted-foreground"
    aria-hidden="true"
  >
    <CirclePlus size={15} strokeWidth={1.75} />
  </div>
{/snippet}

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
  onscroll={syncProjectListCounterScroll}
>
  <div class="flex min-h-full flex-col gap-5 p-3">
    {#if taskGroupBy === "section"}
      {#each sections as section (section.id)}
        {@const sectionTasks = tasksForSection(section)}
        {#if listSectionDropMarkerVisible(section, "before")}
          <div class="h-1 rounded-full bg-primary"></div>
        {/if}
        <section
          class={cn(
            "flex flex-col gap-0 border border-transparent",
            listDragOverSectionId === section.id && "border-primary/40 bg-primary/5",
            listDraggingSectionId === section.id && "opacity-50",
            listSectionDropPendingId === section.id && "opacity-60",
          )}
          style={`min-width: max(100%, ${taskListGridMinWidth});`}
          role="list"
          aria-label={section.name}
          ondragover={(event) => handleListSectionGroupDragOver(event, section)}
          ondrop={(event) => { void dropListSectionOrTask(event, section); }}
        >
          <div
            class={cn(
              "project-list-divider project-list-sticky-row group/section-header grid min-h-11 items-center px-1",
              canStartListSectionDrag(section) && "cursor-grab active:cursor-grabbing",
            )}
            style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
            role="group"
            aria-label={section.name}
            draggable={canStartListSectionDrag(section)}
            onpointerdown={(event) => handleListSectionPointerDown(event, section)}
            onpointerup={clearListSectionDragGesture}
            onpointercancel={clearListSectionDragGesture}
            ondragstart={(event) => handleListSectionDragStart(event, section)}
            ondragend={handleListSectionDragEnd}
          >
            <div class="flex h-7 items-center justify-center">
              <button
                type="button"
                class={cn(
                  "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                  allTasksSelected(sectionTasks)
                    ? "border-primary bg-primary text-primary-foreground opacity-100"
                    : "border-border bg-background opacity-0 hover:bg-accent group-hover/section-header:opacity-100 group-focus-within/section-header:opacity-100",
                  someTasksSelected(sectionTasks) && !allTasksSelected(sectionTasks) && "border-primary/70 bg-primary/10 text-primary opacity-100",
                )}
                aria-label={allTasksSelected(sectionTasks) ? t("projects.actions.unselectTaskGroup", section.name) : t("projects.actions.selectTaskGroup", section.name)}
                disabled={sectionTasks.length === 0}
                onclick={() => toggleTaskGroupSelection(sectionTasks)}
              >
                {#if allTasksSelected(sectionTasks)}
                  <Check size={13} strokeWidth={2} />
                {:else if someTasksSelected(sectionTasks)}
                  <span class="h-0.5 w-2.5 rounded-full bg-current"></span>
                {/if}
              </button>
            </div>
            <button
              type="button"
              class="flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
              aria-label={section.collapsed ? t("projects.actions.expandSection", section.name) : t("projects.actions.collapseSection", section.name)}
              title={section.collapsed ? t("projects.actions.expandSection", section.name) : t("projects.actions.collapseSection", section.name)}
              onclick={() => { void toggleSectionCollapsed(section); }}
            >
              {#if section.collapsed}
                <ChevronRight size={14} strokeWidth={1.75} />
              {:else}
                <ChevronDown size={14} strokeWidth={1.75} />
              {/if}
            </button>
            <div
              class={cn(
                "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded-md px-2 transition-colors focus-within:bg-card focus-within:ring-1 focus-within:ring-inset focus-within:ring-foreground/50",
                sectionDraftDirty(section)
                  ? "bg-card shadow-sm ring-1 ring-inset ring-border"
                  : "bg-transparent ring-0 hover:bg-card/60",
              )}
            >
              <input
                value={sectionNameDraft(section)}
                data-list-section-drag-source="true"
                class="min-h-7 min-w-0 flex-1 bg-transparent text-[0.866667rem] font-semibold disabled:text-muted-foreground"
                aria-label={t("projects.list.sectionName")}
                disabled={Boolean(section.hiddenAt || section.archivedAt)}
                oninput={(event) => {
                  sectionNameDrafts = {
                    ...sectionNameDrafts,
                    [section.id]: event.currentTarget.value,
                  };
                }}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void saveSection(section);
                  }
                }}
              />
              {#if sectionDraftSaveable(section)}
                <button
                  type="button"
                  class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                  onclick={() => { void saveSection(section); }}
                >
                  {t("common.save")}
                </button>
              {/if}
              <div class="relative" data-section-options-root="true">
                <button
                  type="button"
                  class={cn(
                    "flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground opacity-0 transition-opacity hover:bg-accent hover:text-foreground group-hover/section-header:opacity-100 group-focus-within/section-header:opacity-100",
                    sectionOptionsMenuId === section.id && "bg-accent text-foreground opacity-100",
                  )}
                  aria-label={t("projects.actions.sectionOptions", section.name)}
                  title={t("projects.actions.sectionOptions", section.name)}
                  aria-expanded={sectionOptionsMenuId === section.id}
                  onclick={() => {
                    sectionOptionsMenuId = sectionOptionsMenuId === section.id ? null : section.id;
                  }}
                >
                  <MoreHorizontal size={14} strokeWidth={1.75} />
                </button>
                {#if sectionOptionsMenuId === section.id}
                  <div
                    class="absolute right-0 top-8 z-30 w-52 rounded-lg border border-border bg-popover p-1 text-[0.866667rem] text-popover-foreground shadow-sm"
                    role="menu"
                    aria-label={t("projects.actions.sectionOptions", section.name)}
                  >
                    {#if section.hiddenAt || section.archivedAt}
                      <button
                        type="button"
                        class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                        role="menuitem"
                        onclick={() => {
                          sectionOptionsMenuId = null;
                          void restoreSection(section);
                        }}
                      >
                        <ArchiveRestore size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                        <span>{t("projects.actions.restoreSection", section.name)}</span>
                      </button>
                    {:else}
                      <button
                        type="button"
                        class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                        role="menuitem"
                        onclick={() => {
                          sectionOptionsMenuId = null;
                          void hideSection(section);
                        }}
                      >
                        <EyeOff size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                        <span>{t("projects.actions.hideSection", section.name)}</span>
                      </button>
                      <button
                        type="button"
                        class="flex min-h-9 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accent hover:text-foreground"
                        role="menuitem"
                        onclick={() => {
                          sectionOptionsMenuId = null;
                          void archiveSection(section);
                        }}
                      >
                        <Archive size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                        <span>{t("projects.actions.archiveSection", section.name)}</span>
                      </button>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          </div>
          {#if !section.collapsed && !section.archivedAt && !section.hiddenAt}
            <div
              class="project-list-divider group/column-header grid min-h-11 items-center px-1 text-[0.866667rem] font-semibold text-foreground"
              style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
            >
              <div></div>
              <div></div>
              {@render listColumnHeaderCell(t("projects.list.name"), "name")}
              {#each taskListColumns as column (column)}
                {@render listColumnHeaderCell(taskListColumnLabel(column), column)}
              {/each}
              {@render listAddColumnHeaderCell()}
            </div>
            <div class="grid">
              {#each sectionTasks as task (task.id)}
                {@const status = statusForTask(task)}
                {@const subtasks = subtasksForTask(task)}
                {#if listDropMarkerVisible(section, task, "before")}
                  <div class="h-1 rounded-full bg-primary"></div>
                {/if}
                <ProjectListTaskRow
                  {task}
                  {status}
                  {statuses}
                  {subtasks}
                  scheduled={scheduledLabel(task.id)}
                  taskLabels={visibleTaskLabels(task)}
                  hiddenLabels={hiddenTaskLabelCount(task)}
                  blockedByCount={blockedByDependencies(task).length}
                  blocksCount={blocksDependencies(task).length}
                  {taskListColumns}
                  {projectCustomFields}
                  {selectedTaskId}
                  taskSelected={taskSelected(task)}
                  gridTemplate={taskListGridTemplate}
                  gridMinWidth={taskListGridMinWidth}
                  draggable={canStartListTaskDrag(task)}
                  dragging={listDraggingTaskId === task.id}
                  dropPending={listDropPendingTaskId === task.id}
                  statusMenuOpen={statusMenuTaskId === task.id}
                  priorityMenuOpen={priorityMenuTaskId === task.id}
                  startDateMenuOpen={startDateMenuTaskId === task.id}
                  dueDateMenuOpen={dueDateMenuTaskId === task.id}
                  theme={theme.current}
                  {estimateLabel}
                  {customFieldDisplayValue}
                  {statusForTask}
                  onToggleTaskSelection={toggleTaskSelection}
                  onOpenTask={openTaskDetail}
                  onPointerDown={(event) => handleListRowPointerDown(event, task)}
                  onPointerUp={clearListRowDragGesture}
                  onPointerCancel={clearListRowDragGesture}
                  onDragStart={(event) => handleListTaskDragStart(event, task)}
                  onDragEnd={handleListTaskDragEnd}
                  onDragOver={(event) => handleListRowDragOver(event, section, task)}
                  onDrop={(event) => { void dropListTask(event, section, task, listRowDropPosition(event)); }}
                  onToggleStatusMenu={() => {
                    const nextTaskId = statusMenuTaskId === task.id ? null : task.id;
                    statusMenuTaskId = nextTaskId;
                    if (nextTaskId) {
                      priorityMenuTaskId = null;
                      startDateMenuTaskId = null;
                      dueDateMenuTaskId = null;
                    }
                  }}
                  onSetStatus={(nextStatus) => { void setTaskStatusFromList(task, nextStatus); }}
                  onTogglePriorityMenu={() => {
                    const nextTaskId = priorityMenuTaskId === task.id ? null : task.id;
                    priorityMenuTaskId = nextTaskId;
                    if (nextTaskId) {
                      statusMenuTaskId = null;
                      startDateMenuTaskId = null;
                      dueDateMenuTaskId = null;
                    }
                  }}
                  onSetPriority={(priority) => { void setTaskPriorityFromList(task, priority); }}
                  onToggleStartDateMenu={() => {
                    const nextTaskId = startDateMenuTaskId === task.id ? null : task.id;
                    startDateMenuTaskId = nextTaskId;
                    if (nextTaskId) {
                      statusMenuTaskId = null;
                      priorityMenuTaskId = null;
                      dueDateMenuTaskId = null;
                    }
                  }}
                  onCloseStartDateMenu={() => { startDateMenuTaskId = null; }}
                  onSetStartDate={(startDate) => { void setTaskStartDateFromList(task, startDate); }}
                  onClearStartDate={() => { void setTaskStartDateFromList(task, undefined); }}
                  onSetStartTime={(startTime) => { void setTaskStartTimeFromList(task, startTime); }}
                  onClearStartTime={() => { void setTaskStartTimeFromList(task, undefined); }}
                  onToggleDueDateMenu={() => {
                    const nextTaskId = dueDateMenuTaskId === task.id ? null : task.id;
                    dueDateMenuTaskId = nextTaskId;
                    if (nextTaskId) {
                      statusMenuTaskId = null;
                      priorityMenuTaskId = null;
                      startDateMenuTaskId = null;
                    }
                  }}
                  onCloseDueDateMenu={() => { dueDateMenuTaskId = null; }}
                  onSetDueDate={(dueDate) => { void setTaskDueDateFromList(task, dueDate); }}
                  onClearDueDate={() => { void setTaskDueDateFromList(task, undefined); }}
                  onSetDueTime={(dueTime) => { void setTaskDueTimeFromList(task, dueTime); }}
                  onClearDueTime={() => { void setTaskDueTimeFromList(task, undefined); }}
                  onToggleSubtaskDone={(subtask) => { void projects.toggleTaskDone(subtask); }}
                />
                {#if listDropMarkerVisible(section, task, "after")}
                  <div class="h-1 rounded-full bg-primary"></div>
                {/if}
              {/each}
              {#if listDragOverSectionId === section.id && listDragOverPosition === "section"}
                <div class="h-1 rounded-full bg-primary"></div>
              {/if}
            </div>
            <div
              class="project-list-divider grid cursor-text items-center px-1 py-1.5"
              data-section-task-add-row={section.id}
              style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
              use:projectListAddRowInputFocus={{ selector: "[data-section-task-input]" }}
            >
              <div class="absolute inset-0 z-0 cursor-text" aria-hidden="true"></div>
              <div class="relative z-10"></div>
              <div class="relative z-10"></div>
              <form
                class="contents"
                onsubmit={(event) => { event.preventDefault(); void submitSectionTask(section.id); }}
              >
                <div class="relative z-10 min-w-0 px-2" style="grid-column: 3;">
                  {#if !(sectionTaskDrafts[section.id] ?? "").trim() && activeSectionTaskDraftInputId !== section.id}
                    <div
                      class="pointer-events-none absolute inset-y-0 left-2 flex items-center gap-2 text-muted-foreground"
                      aria-hidden="true"
                    >
                      <span class="flex h-5 w-5 shrink-0 items-center justify-center">
                        <Plus size={15} strokeWidth={1.75} />
                      </span>
                      <span class="text-[0.866667rem]">{t("projects.list.addTaskInSection", section.name)}</span>
                    </div>
                  {/if}
                  <input
                    data-section-task-input={section.id}
                    aria-label={t("projects.list.addTaskInSection", section.name)}
                    value={sectionTaskDrafts[section.id] ?? ""}
                    onfocus={() => {
                      activeSectionTaskDraftInputId = section.id;
                    }}
                    onblur={() => {
                      if (activeSectionTaskDraftInputId === section.id) {
                        activeSectionTaskDraftInputId = null;
                      }
                    }}
                    oninput={(event) => {
                      sectionTaskDrafts = {
                        ...sectionTaskDrafts,
                        [section.id]: event.currentTarget.value,
                      };
                    }}
                    class="min-h-8 w-full min-w-0 bg-transparent text-[0.866667rem] text-foreground"
                  />
                  {#if activeSectionTaskDraftInputId === section.id && !(sectionTaskDrafts[section.id] ?? "").trim()}
                    <span
                      class="project-list-add-row-caret pointer-events-none absolute left-2 top-1/2 h-4 w-px -translate-y-1/2 bg-foreground"
                      aria-hidden="true"
                    ></span>
                  {/if}
                </div>
                {#if (sectionTaskDrafts[section.id] ?? "").trim()}
                  <div class="relative z-10 flex min-w-0 items-center px-2" style="grid-column: 4;">
                    <button
                      type="submit"
                      disabled={taskCreatePendingTarget !== null}
                      class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      {t("projects.list.saveWithEnter")}
                    </button>
                  </div>
                {/if}
              </form>
              {#if taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                <div class="col-span-full rounded border border-destructive/30 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                  {taskCreateErrorFor(sectionTaskCreateTarget(section.id))}
                </div>
              {/if}
            </div>
          {/if}
        </section>
        {#if listSectionDropMarkerVisible(section, "after")}
          <div class="h-1 rounded-full bg-primary"></div>
        {/if}
      {/each}
      <div
        class="project-list-sticky-row grid cursor-text items-center px-1 py-1.5"
        data-add-section-row="true"
        style={`grid-template-columns: ${taskListGridTemplate}; min-width: max(100%, ${taskListGridMinWidth});`}
        use:projectListAddRowInputFocus={{
          selector: "[data-add-section-input='true']",
          beforeFocus: () => {
            sectionDraftInputActive = true;
          },
        }}
      >
        <div class="absolute inset-0 z-0 cursor-text" aria-hidden="true"></div>
        <div class="relative z-10"></div>
        <div class="relative z-10"></div>
        <form
          class="contents"
          onsubmit={(event) => { event.preventDefault(); void submitSection(); }}
        >
          <div class="relative z-10 min-w-0 px-2" style="grid-column: 3;">
            {#if !sectionDraft.trim() && !sectionDraftInputActive}
              <div
                class="pointer-events-none absolute inset-y-0 left-2 flex items-center gap-2 text-muted-foreground"
                aria-hidden="true"
              >
                <span class="flex h-5 w-5 shrink-0 items-center justify-center">
                  <Plus size={15} strokeWidth={1.75} />
                </span>
                <span class="text-[0.866667rem]">{t("projects.header.addSection")}</span>
              </div>
            {/if}
            <input
              data-add-section-input="true"
              aria-label={t("projects.header.addSection")}
              bind:value={sectionDraft}
              onfocus={() => {
                sectionDraftInputActive = true;
              }}
              onblur={() => {
                sectionDraftInputActive = false;
              }}
              class="min-h-8 w-full min-w-0 bg-transparent text-[0.866667rem] text-foreground"
            />
            {#if sectionDraftInputActive && !sectionDraft.trim()}
              <span
                class="project-list-add-row-caret pointer-events-none absolute left-2 top-1/2 h-4 w-px -translate-y-1/2 bg-foreground"
                aria-hidden="true"
              ></span>
            {/if}
          </div>
          {#if sectionDraft.trim()}
            <div class="relative z-10 flex min-w-0 items-center px-2" style="grid-column: 4;">
              <button
                type="submit"
                class="flex h-7 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border bg-card px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                {t("projects.list.saveWithEnter")}
              </button>
            </div>
          {/if}
        </form>
      </div>
    {:else}
      {#each listTaskGroups as group (group.id)}
        <section
          class="flex flex-col gap-0"
          style={`min-width: max(100%, ${taskListGridMinWidth});`}
        >
          <div
            class="project-list-divider project-list-sticky-row group/list-group-header grid min-h-11 items-center px-1"
            style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
          >
            <div class="flex h-7 items-center justify-center">
              <button
                type="button"
                class={cn(
                  "flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded border transition-opacity",
                  allTasksSelected(group.tasks)
                    ? "border-primary bg-primary text-primary-foreground opacity-100"
                    : "border-border bg-background opacity-0 hover:bg-accent group-hover/list-group-header:opacity-100 group-focus-within/list-group-header:opacity-100",
                  someTasksSelected(group.tasks) && !allTasksSelected(group.tasks) && "border-primary/70 bg-primary/10 text-primary opacity-100",
                )}
                aria-label={allTasksSelected(group.tasks) ? t("projects.actions.unselectTaskGroup", taskListGroupTitle(group.value)) : t("projects.actions.selectTaskGroup", taskListGroupTitle(group.value))}
                disabled={group.tasks.length === 0}
                onclick={() => toggleTaskGroupSelection(group.tasks)}
              >
                {#if allTasksSelected(group.tasks)}
                  <Check size={13} strokeWidth={2} />
                {:else if someTasksSelected(group.tasks)}
                  <span class="h-0.5 w-2.5 rounded-full bg-current"></span>
                {/if}
              </button>
            </div>
            <div></div>
            <span class="min-w-0 truncate px-2 text-[0.866667rem] font-semibold">
              {taskListGroupTitle(group.value)}
            </span>
          </div>
          <div
            class="project-list-divider group/list-column-header grid min-h-11 items-center px-1 text-[0.866667rem] font-semibold text-foreground"
            style={`grid-template-columns: ${taskListGridTemplate}; min-width: ${taskListGridMinWidth};`}
          >
            <div></div>
            <div></div>
            {@render listColumnHeaderCell(t("projects.list.name"), "name")}
            {#each taskListColumns as column (column)}
              {@render listColumnHeaderCell(taskListColumnLabel(column), column)}
            {/each}
            {@render listAddColumnHeaderCell()}
          </div>
          <div class="grid">
            {#each group.tasks as task (task.id)}
              {@const status = statusForTask(task)}
              {@const section = sectionForTask(task)}
              <ProjectListTaskRow
                {task}
                {status}
                {statuses}
                subtasks={subtasksForTask(task)}
                scheduled={scheduledLabel(task.id)}
                taskLabels={visibleTaskLabels(task)}
                hiddenLabels={hiddenTaskLabelCount(task)}
                blockedByCount={blockedByDependencies(task).length}
                blocksCount={blocksDependencies(task).length}
                {taskListColumns}
                {projectCustomFields}
                {selectedTaskId}
                taskSelected={taskSelected(task)}
                gridTemplate={taskListGridTemplate}
                gridMinWidth={taskListGridMinWidth}
                sectionName={section?.name}
                draggable={false}
                dragging={false}
                dropPending={false}
                statusMenuOpen={statusMenuTaskId === task.id}
                priorityMenuOpen={priorityMenuTaskId === task.id}
                startDateMenuOpen={startDateMenuTaskId === task.id}
                dueDateMenuOpen={dueDateMenuTaskId === task.id}
                theme={theme.current}
                {estimateLabel}
                {customFieldDisplayValue}
                {statusForTask}
                onToggleTaskSelection={toggleTaskSelection}
                onOpenTask={openTaskDetail}
                onToggleStatusMenu={() => {
                  const nextTaskId = statusMenuTaskId === task.id ? null : task.id;
                  statusMenuTaskId = nextTaskId;
                  if (nextTaskId) {
                    priorityMenuTaskId = null;
                    startDateMenuTaskId = null;
                    dueDateMenuTaskId = null;
                  }
                }}
                onSetStatus={(nextStatus) => { void setTaskStatusFromList(task, nextStatus); }}
                onTogglePriorityMenu={() => {
                  const nextTaskId = priorityMenuTaskId === task.id ? null : task.id;
                  priorityMenuTaskId = nextTaskId;
                  if (nextTaskId) {
                    statusMenuTaskId = null;
                    startDateMenuTaskId = null;
                    dueDateMenuTaskId = null;
                  }
                }}
                onSetPriority={(priority) => { void setTaskPriorityFromList(task, priority); }}
                onToggleStartDateMenu={() => {
                  const nextTaskId = startDateMenuTaskId === task.id ? null : task.id;
                  startDateMenuTaskId = nextTaskId;
                  if (nextTaskId) {
                    statusMenuTaskId = null;
                    priorityMenuTaskId = null;
                    dueDateMenuTaskId = null;
                  }
                }}
                onCloseStartDateMenu={() => { startDateMenuTaskId = null; }}
                onSetStartDate={(startDate) => { void setTaskStartDateFromList(task, startDate); }}
                onClearStartDate={() => { void setTaskStartDateFromList(task, undefined); }}
                onSetStartTime={(startTime) => { void setTaskStartTimeFromList(task, startTime); }}
                onClearStartTime={() => { void setTaskStartTimeFromList(task, undefined); }}
                onToggleDueDateMenu={() => {
                  const nextTaskId = dueDateMenuTaskId === task.id ? null : task.id;
                  dueDateMenuTaskId = nextTaskId;
                  if (nextTaskId) {
                    statusMenuTaskId = null;
                    priorityMenuTaskId = null;
                    startDateMenuTaskId = null;
                  }
                }}
                onCloseDueDateMenu={() => { dueDateMenuTaskId = null; }}
                onSetDueDate={(dueDate) => { void setTaskDueDateFromList(task, dueDate); }}
                onClearDueDate={() => { void setTaskDueDateFromList(task, undefined); }}
                onSetDueTime={(dueTime) => { void setTaskDueTimeFromList(task, dueTime); }}
                onClearDueTime={() => { void setTaskDueTimeFromList(task, undefined); }}
                onToggleSubtaskDone={(subtask) => { void projects.toggleTaskDone(subtask); }}
              />
            {/each}
          </div>
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

  .project-list-header-cell::before {
    position: absolute;
    inset: 0;
    z-index: 1;
    border: 1px solid transparent;
    border-radius: 0.375rem;
    content: "";
    pointer-events: none;
  }

  .project-list-header-cell:hover::before {
    border-color: color-mix(in srgb, var(--foreground) 25%, transparent);
  }

  .project-list-column-resize-hit {
    position: absolute;
    top: 0.375rem;
    right: -0.375rem;
    bottom: 0.375rem;
    z-index: 20;
    width: 0.75rem;
    border: 0;
    background: transparent;
    cursor: col-resize;
    padding: 0;
  }

  .project-list-column-resize-hit::after {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 0.25rem;
    transform: translateX(-50%);
    border-radius: 9999px;
    background: var(--primary);
    content: "";
    opacity: 0;
  }

  .project-list-column-resize-hit:hover::after,
  .project-list-column-resize-hit:focus-visible::after,
  .project-list-column-resize-hit:active::after {
    opacity: 1;
  }

  .project-list-add-row-caret {
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
