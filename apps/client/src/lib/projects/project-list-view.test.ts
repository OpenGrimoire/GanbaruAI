import { describe, expect, it } from "vitest";
import type { Translate } from "$lib/i18n/translator.svelte";
import type { ProjectTask } from "./types";
import {
  PROJECT_TASK_FILTER_DEFAULTS,
  projectTaskActiveFilterChips,
  projectTaskDataFiltersActive,
  projectTaskFiltersActive,
  projectTaskListColumnTrack,
  projectTaskListGridMinWidth,
  projectTaskListGridTemplate,
  selectedProjectTaskIdsInView,
} from "./project-list-view";

function task(id: string): ProjectTask {
  return {
    id,
    projectId: "project-a",
    sectionId: "section-a",
    statusId: "status-a",
    title: id,
    description: "",
    priority: "normal",
    taskType: "task",
    sectionSortOrder: 1000,
    statusSortOrder: 1000,
    milestone: false,
    createdAt: "2026-06-21T00:00:00.000Z",
    updatedAt: "2026-06-21T00:00:00.000Z",
  };
}

const t = ((key: string, ...args: unknown[]) => [key, ...args].join("|")) as Translate;

describe("project list view helpers", () => {
  it("builds stable grid tracks for core and custom columns", () => {
    expect(projectTaskListColumnTrack("priority")).toBe("minmax(7rem, 0.7fr)");
    expect(projectTaskListColumnTrack("start")).toBe("minmax(7rem, 0.7fr)");
    expect(projectTaskListColumnTrack("assignee")).toBe("minmax(6rem, 0.55fr)");
    expect(projectTaskListColumnTrack("status")).toBe("minmax(8rem, 0.8fr)");
    expect(projectTaskListColumnTrack("custom:field-a")).toBe("minmax(9rem, 0.85fr)");
    expect(projectTaskListGridTemplate(["status", "custom:field-a"])).toBe(
      "1.5rem 1.75rem minmax(16rem, 2fr) minmax(8rem, 0.8fr) minmax(9rem, 0.85fr)",
    );
  });

  it("keeps the list width stable as columns are added", () => {
    expect(projectTaskListGridMinWidth([])).toBe("47rem");
    expect(projectTaskListGridMinWidth(["status", "priority", "estimate"])).toBe("47rem");
    expect(projectTaskListGridMinWidth(["status", "start", "due", "priority", "assignee", "reviewer"])).toBe("66rem");
  });

  it("removes selected tasks that are no longer visible", () => {
    expect(selectedProjectTaskIdsInView(["task-a", "task-c"], [task("task-a"), task("task-b")])).toEqual(["task-a"]);
  });

  it("separates data filters from presentation filters", () => {
    expect(projectTaskDataFiltersActive(PROJECT_TASK_FILTER_DEFAULTS)).toBe(false);
    expect(projectTaskFiltersActive(PROJECT_TASK_FILTER_DEFAULTS)).toBe(false);
    expect(projectTaskDataFiltersActive({
      ...PROJECT_TASK_FILTER_DEFAULTS,
      search: "release",
    })).toBe(true);
    expect(projectTaskDataFiltersActive({
      ...PROJECT_TASK_FILTER_DEFAULTS,
      groupBy: "status",
    })).toBe(false);
    expect(projectTaskFiltersActive({
      ...PROJECT_TASK_FILTER_DEFAULTS,
      groupBy: "status",
    })).toBe(true);
  });

  it("builds active filter chips with lookup labels", () => {
    expect(projectTaskActiveFilterChips({
      ...PROJECT_TASK_FILTER_DEFAULTS,
      search: "release",
      sectionFilter: "section-a",
      priorityFilter: "urgent",
      dueFilter: "range",
      dueRangeStart: "invalid",
      dueRangeEnd: "invalid",
      labelFilter: "label-a",
      customFieldFilters: [{ fieldId: "field-a", mode: "option", optionId: "option-a" }],
      sections: [{
        id: "section-a",
        projectId: "project-a",
        name: "Backlog",
        sortOrder: 1000,
        collapsed: false,
        createdAt: "2026-06-21T00:00:00.000Z",
        updatedAt: "2026-06-21T00:00:00.000Z",
      }],
      labels: [{
        id: "label-a",
        projectId: "project-a",
        name: "Design",
        color: 2,
        sortOrder: 1000,
        createdAt: "2026-06-21T00:00:00.000Z",
        updatedAt: "2026-06-21T00:00:00.000Z",
      }],
      customFields: [{
        id: "field-a",
        projectId: "project-a",
        name: "Stage",
        fieldType: "select",
        sortOrder: 1000,
        createdAt: "2026-06-21T00:00:00.000Z",
        updatedAt: "2026-06-21T00:00:00.000Z",
      }],
      customFieldOptionsForField: () => [{
        id: "option-a",
        fieldId: "field-a",
        name: "Discovery",
        sortOrder: 1000,
        createdAt: "2026-06-21T00:00:00.000Z",
        updatedAt: "2026-06-21T00:00:00.000Z",
      }],
      normalizedDueRangeStart: "2026-06-21",
      normalizedDueRangeEnd: "2026-06-22",
      t,
    }).map((chip) => chip.label)).toEqual([
      "release",
      "Backlog",
      "projects.priority.urgent",
      "projects.filters.dueRangeChip|2026-06-21|2026-06-22",
      "Design",
      "projects.filters.customFieldChip|Stage|Discovery",
    ]);
  });
});
