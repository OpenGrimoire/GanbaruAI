<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { projectDefaultPomodoroConfig } from "$lib/projects/project-default-pomodoro";
  import {
    formatProjectScheduleWindowStart,
    projectDefaultScheduleStart,
    type ProjectScheduleWindow,
  } from "$lib/projects/project-scheduling";
  import { projectEffectiveDurationMinutes } from "$lib/projects/project-settings-duration";
  import type {
    Project,
    ProjectPriority,
    ProjectStatus,
    ProjectTask,
  } from "$lib/projects/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import type { CalendarEvent } from "$lib/components/calendar/types";
  import ProjectBulkActionBar from "./ProjectBulkActionBar.svelte";

  let {
    selectedProject,
    selectedTasks,
    selectableTasks,
    selectedActiveTaskCount,
    selectedArchivedTaskCount,
    terminalStatus,
    firstOpenStatus,
    selectedTaskIds = $bindable<string[]>(),
    showArchivedTasks = $bindable<boolean>(),
  }: {
    selectedProject: Project;
    selectedTasks: ProjectTask[];
    selectableTasks: ProjectTask[];
    selectedActiveTaskCount: number;
    selectedArchivedTaskCount: number;
    terminalStatus: ProjectStatus | undefined;
    firstOpenStatus: ProjectStatus | undefined;
    selectedTaskIds: string[];
    showArchivedTasks: boolean;
  } = $props();

  const projects = getProjects();
  const calendar = getCalendar();
  const { t } = getLocalization();

  let bulkTaskActionPending = $state(false);
  let bulkTaskError = $state<string | null>(null);
  let bulkScheduleOpen = $state(false);
  let bulkScheduleDate = $state("");
  let bulkScheduleStartTime = $state("");
  let bulkScheduleDurationMinutes = $state(60);

  const selectedSchedulableTasks = $derived(selectedTasks.filter((task) => !task.archivedAt));

  function selectFilteredTasks(): void {
    const nextIds = new Set(selectedTaskIds);
    for (const task of selectableTasks) {
      nextIds.add(task.id);
    }
    selectedTaskIds = Array.from(nextIds);
  }

  function clearTaskSelection(): void {
    selectedTaskIds = [];
    bulkTaskError = null;
    bulkScheduleOpen = false;
  }

  async function runBulkTaskAction(action: () => Promise<void>): Promise<void> {
    if (selectedTasks.length === 0) return;
    bulkTaskActionPending = true;
    bulkTaskError = null;
    try {
      await action();
      selectedTaskIds = [];
    } catch (error) {
      bulkTaskError = t(
        "projects.bulk.actionFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      bulkTaskActionPending = false;
    }
  }

  async function bulkSetSelectedStatus(status: ProjectStatus | undefined): Promise<void> {
    if (!status) return;
    await runBulkTaskAction(() => projects.setTasksStatus(selectedTasks, status.id));
  }

  async function bulkSetSelectedPriority(priority: ProjectPriority): Promise<void> {
    await runBulkTaskAction(() => projects.setTasksPriority(selectedTasks, priority));
  }

  async function bulkArchiveSelectedTasks(): Promise<void> {
    await runBulkTaskAction(() => projects.archiveTasks(selectedTasks));
  }

  async function bulkRestoreSelectedTasks(): Promise<void> {
    showArchivedTasks = true;
    await runBulkTaskAction(() => projects.restoreTasks(selectedTasks));
  }

  function openBulkScheduleForm(): void {
    const start = projectDefaultScheduleStart();
    bulkScheduleOpen = true;
    bulkScheduleDate = start.date;
    bulkScheduleStartTime = start.time;
    bulkScheduleDurationMinutes = projectEffectiveDurationMinutes(selectedProject.defaultEventDurationMinutes);
    bulkTaskError = null;
  }

  function closeBulkScheduleForm(): void {
    bulkScheduleOpen = false;
    bulkTaskError = null;
  }

  async function createScheduledTaskBlock(
    task: ProjectTask,
    project: Project,
    scheduledWindow: ProjectScheduleWindow,
  ): Promise<CalendarEvent> {
    let createdEventId: string | null = null;
    try {
      const event = await calendar.addBlock({
        title: task.title,
        start: scheduledWindow.start,
        end: scheduledWindow.end,
        projectId: project.id,
        color: project.color,
        pomodoroConfig: projectDefaultPomodoroConfig(project),
      });
      createdEventId = event.id;
      const scheduledDate = scheduledWindow.start.slice(0, 10);
      await projects.linkTaskEvent(task.id, event.id, "scheduled");
      await projects.updateTask(task, {
        startDate: scheduledDate,
        targetEndDate: scheduledDate,
        dueDate: task.dueDate ?? scheduledDate,
      });
      return event;
    } catch (error) {
      if (createdEventId) {
        await calendar.deleteBlock(createdEventId).catch((deleteError) => {
          console.error("delete failed scheduled event after task link error", deleteError);
        });
      }
      throw error;
    }
  }

  async function bulkScheduleSelectedTasks(): Promise<void> {
    const duration = Math.round(Number(bulkScheduleDurationMinutes));
    if (selectedSchedulableTasks.length === 0 || duration <= 0 || !bulkScheduleDate || !bulkScheduleStartTime) {
      bulkTaskError = t("projects.schedule.invalid");
      return;
    }
    let cursor: Temporal.PlainDateTime;
    try {
      cursor = Temporal.PlainDateTime.from(`${bulkScheduleDate}T${bulkScheduleStartTime}`);
    } catch {
      bulkTaskError = t("projects.schedule.invalid");
      return;
    }

    bulkTaskActionPending = true;
    bulkTaskError = null;
    let scheduledCount = 0;
    try {
      for (const task of selectedSchedulableTasks) {
        const scheduledWindow = formatProjectScheduleWindowStart(cursor, duration);
        await createScheduledTaskBlock(task, selectedProject, scheduledWindow);
        scheduledCount += 1;
        cursor = cursor.add({ minutes: duration });
      }
      bulkScheduleOpen = false;
      clearTaskSelection();
      projects.activeView = "calendar";
    } catch (error) {
      bulkTaskError = t(
        "projects.bulk.scheduleFailed",
        scheduledCount,
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      bulkTaskActionPending = false;
    }
  }
</script>

{#if selectedTasks.length > 0}
  <ProjectBulkActionBar
    selectedTaskCount={selectedTasks.length}
    selectableTaskCount={selectableTasks.length}
    {selectedActiveTaskCount}
    {selectedArchivedTaskCount}
    bulkSchedulableCount={selectedSchedulableTasks.length}
    {bulkTaskActionPending}
    {bulkScheduleOpen}
    {bulkScheduleDate}
    {bulkScheduleStartTime}
    {bulkScheduleDurationMinutes}
    {bulkTaskError}
    canMarkDone={Boolean(terminalStatus)}
    canReopen={Boolean(firstOpenStatus)}
    onSelectFiltered={selectFilteredTasks}
    onMarkDone={() => { void bulkSetSelectedStatus(terminalStatus); }}
    onReopen={() => { void bulkSetSelectedStatus(firstOpenStatus); }}
    onToggleBulkSchedule={() => {
      if (bulkScheduleOpen) {
        closeBulkScheduleForm();
      } else {
        openBulkScheduleForm();
      }
    }}
    onSetPriority={(priority) => { void bulkSetSelectedPriority(priority); }}
    onArchive={() => { void bulkArchiveSelectedTasks(); }}
    onRestore={() => { void bulkRestoreSelectedTasks(); }}
    onClear={clearTaskSelection}
    onBulkScheduleDateChange={(value) => {
      bulkScheduleDate = value;
    }}
    onBulkScheduleStartTimeChange={(value) => {
      bulkScheduleStartTime = value;
    }}
    onBulkScheduleDurationMinutesChange={(value) => {
      if (Number.isFinite(value)) bulkScheduleDurationMinutes = value;
    }}
    onBulkScheduleSubmit={() => { void bulkScheduleSelectedTasks(); }}
    onCloseBulkSchedule={closeBulkScheduleForm}
  />
{/if}
