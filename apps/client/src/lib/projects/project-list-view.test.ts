import { describe, expect, it } from "vitest";
import type { Translate } from "$lib/i18n/translator.svelte";
import type { ProjectCustomField, ProjectStatus, ProjectTask } from "./types";
import {
  PROJECT_TASK_FILTER_DEFAULTS,
  projectTaskActiveFilterChips,
  projectTaskDataFiltersActive,
  projectTaskFiltersActive,
  parseProjectTaskListColumnWidths,
  projectTaskListDoubleClickColumnWidthRem,
  projectTaskListColumnWidthRem,
  projectTaskListColumnTrack,
  projectTaskListGridMinWidth,
  projectTaskListGridTemplate,
  selectedProjectTaskIdsInView,
  taskListColumnWidthsForProject,
  taskListColumnWidthsPreferenceValue,
  TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
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

function status(name: string): ProjectStatus {
  return {
    id: `status-${name}`,
    projectId: "project-a",
    name,
    category: "active",
    color: 0,
    sortOrder: 1000,
    terminal: false,
    createdAt: "2026-06-21T00:00:00.000Z",
    updatedAt: "2026-06-21T00:00:00.000Z",
  };
}

function customField(id: string, name: string): ProjectCustomField {
  return {
    id,
    projectId: "project-a",
    name,
    fieldType: "text",
    sortOrder: 1000,
    createdAt: "2026-06-21T00:00:00.000Z",
    updatedAt: "2026-06-21T00:00:00.000Z",
  };
}

const t = ((key: string, ...args: unknown[]) => [key, ...args].join("|")) as Translate;

describe("project list view helpers", () => {
  it("builds compact fixed grid tracks for core and custom columns", () => {
    expect(projectTaskListColumnTrack("priority")).toBe("5.2rem");
    expect(projectTaskListColumnTrack("start")).toBe("8.07rem");
    expect(projectTaskListColumnTrack("assignee")).toBe("5.2rem");
    expect(projectTaskListColumnTrack("status")).toBe("7.5rem");
    expect(projectTaskListColumnTrack("custom:field-a")).toBe("7.63rem");
    expect(projectTaskListGridTemplate(["status", "custom:field-a"])).toBe(
      "1.5rem 1.75rem 24rem 7.5rem 7.63rem 2.25rem",
    );
  });

  it("sizes tracks from visible content without using flexible ratios", () => {
    const shortTask = task("task-a");
    const longTask = {
      ...task("task-b"),
      title: "Review launch checklist and copy",
      startDate: "2026-06-21",
      startTime: "12:30",
    };

    expect(projectTaskListColumnWidthRem("status", {
      columns: ["status"],
      statuses: [status("Needs outside review before release")],
    })).toBe(12);
    expect(projectTaskListGridTemplate({
      columns: ["status", "start"],
      tasks: [shortTask, longTask],
      statuses: [status("In progress")],
      nameLabel: "Name",
      columnLabel: (column) => column === "start" ? "Start date" : "Status",
    })).toBe("1.5rem 1.75rem 24rem 7.52rem 8.07rem 2.25rem");
  });

  it("lets custom column headers set the first automatic width up to a cap", () => {
    expect(projectTaskListColumnWidthRem("custom:field-a", {
      columns: ["custom:field-a"],
      customFields: [customField("field-a", "Customer confirmation status")],
      columnLabel: () => "Customer confirmation status",
    })).toBe(13.51);
    expect(projectTaskListColumnWidthRem("custom:field-b", {
      columns: ["custom:field-b"],
      customFields: [customField("field-b", "Extremely long custom field header that should still have a limit")],
      columnLabel: () => "Extremely long custom field header that should still have a limit",
    })).toBe(16);
  });

  it("keeps the list width to the content tracks instead of the viewport", () => {
    expect(projectTaskListGridMinWidth([])).toBe("29.5rem");
    expect(projectTaskListGridMinWidth(["status", "priority", "estimate"])).toBe("47.4rem");
    expect(projectTaskListGridMinWidth(["status", "start", "due", "priority", "assignee", "reviewer"])).toBe("68.74rem");
  });

  it("applies manual column widths over automatic list sizing", () => {
    expect(projectTaskListGridTemplate({
      columns: ["status", "due"],
      columnWidths: {
        name: 18,
        status: 9,
        due: 14,
      },
    })).toBe("1.5rem 1.75rem 18rem 9rem 14rem 2.25rem");
  });

  it("uses double-click sizing to reset or fit hidden content", () => {
    expect(projectTaskListDoubleClickColumnWidthRem("priority", {
      columns: ["priority"],
      columnWidths: { priority: 4 },
    })).toBeUndefined();
    expect(projectTaskListDoubleClickColumnWidthRem("status", {
      columns: ["status"],
      statuses: [status("Needs outside review before release")],
    })).toBeCloseTo(17.6);
  });

  it("round-trips list column width preferences with validation", () => {
    const value = taskListColumnWidthsPreferenceValue({
      name: 11,
      status: 9,
      due: 80,
      "custom:field-a": 12,
    });
    expect(parseProjectTaskListColumnWidths(value, new Set(["field-a"]))).toEqual({
      "custom:field-a": 12,
      due: 64,
      name: 12,
      status: 9,
    });
    expect(parseProjectTaskListColumnWidths(value, new Set(["other"]))).toEqual({
      due: 64,
      name: 12,
      status: 9,
    });
  });

  it("loads list column widths from the matching project preference", () => {
    expect(taskListColumnWidthsForProject([
      {
        projectId: "project-a",
        viewId: "list",
        preferenceKey: TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
        preferenceValue: taskListColumnWidthsPreferenceValue({ name: 20, priority: 7 }),
        updatedAt: "2026-06-21T00:00:00.000Z",
      },
      {
        projectId: "project-b",
        viewId: "list",
        preferenceKey: TASK_LIST_COLUMN_WIDTHS_PREFERENCE_KEY,
        preferenceValue: taskListColumnWidthsPreferenceValue({ status: 10 }),
        updatedAt: "2026-06-21T00:00:00.000Z",
      },
    ], "project-a")).toEqual({ name: 20, priority: 7 });
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
      tagFilter: "tag-a",
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
      tags: [{
        id: "tag-a",
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
