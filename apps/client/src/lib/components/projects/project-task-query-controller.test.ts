import { describe, expect, it, vi } from "vitest";
import type { Translate } from "$lib/i18n/translator.svelte";
import type { ProjectTaskViewRequest } from "$lib/projects/types";
import {
  ProjectTaskQueryController,
  normalizeProjectFilterDate,
  repairProjectTaskQueryReferences,
} from "./project-task-query-controller.svelte";

type ControllerInput = ConstructorParameters<typeof ProjectTaskQueryController>[0];

function createController(activeView: "list" | "kanban" = "list") {
  const requests: Array<{ request: ProjectTaskViewRequest; append: boolean }> = [];
  const projects = {
    selectedProject: { id: "project-1" },
    activeView,
    taskViewLoading: false,
    taskViewPage: activeView === "list"
      ? { nextCursor: "list-cursor", columnCounts: [] }
      : { columnCounts: [{ statusId: "status-1", nextCursor: "column-cursor" }] },
    sectionsForProject: () => [{ id: "section-1" }],
    sectionsForProjectIncludingInactive: () => [{ id: "section-1" }],
    loadTaskView: async (request: ProjectTaskViewRequest, append: boolean) => {
      requests.push({ request, append });
    },
  } as unknown as ControllerInput["projects"];
  const controller = new ProjectTaskQueryController({
    projects,
    calendar: { rawBlocks: [] } as unknown as ControllerInput["calendar"],
    translate: ((key: string) => key) as Translate,
  });
  return { controller, requests };
}

describe("Project task query controller policy", () => {
  it("normalizes valid dates and rejects incomplete dates", () => {
    expect(normalizeProjectFilterDate(" 2026-07-12 ")).toBe("2026-07-12");
    expect(normalizeProjectFilterDate("2026-02-30")).toBeUndefined();
    expect(normalizeProjectFilterDate(" ")).toBeUndefined();
  });

  it("repairs every reference whose project collection disappeared", () => {
    expect(repairProjectTaskQueryReferences({
      sectionFilter: "removed-section",
      tagFilter: "removed-tag",
      customFieldFilters: [
        { fieldId: "kept-field", mode: "filled" },
        { fieldId: "removed-field", mode: "empty" },
      ],
      sortMode: "custom:removed-field",
      sortDirection: "desc",
      sectionIds: new Set(["section-1"]),
      tagIds: new Set(["tag-1"]),
      fieldIds: new Set(["kept-field"]),
      optionIds: new Set(),
    })).toEqual({
      sectionFilter: "all",
      tagFilter: "all",
      customFieldFilters: [{ fieldId: "kept-field", mode: "filled" }],
      sortMode: "manual",
      sortDirection: "asc",
    });
  });

  it("keeps special no-tag filters and live custom sorting", () => {
    expect(repairProjectTaskQueryReferences({
      sectionFilter: "section-1",
      tagFilter: "none",
      customFieldFilters: [{ fieldId: "field-1", mode: "option", optionId: "option-1" }],
      sortMode: "custom:field-1",
      sortDirection: "desc",
      sectionIds: new Set(["section-1"]),
      tagIds: new Set(),
      fieldIds: new Set(["field-1"]),
      optionIds: new Set(["option-1"]),
    })).toMatchObject({
      sectionFilter: "section-1",
      tagFilter: "none",
      sortMode: "custom:field-1",
      sortDirection: "desc",
    });
  });

  it("builds the backend request from canonical state without local-only grouping", () => {
    const { controller } = createController();
    controller.groupBy = "priority";
    controller.dueRangeStart = " 2026-07-01 ";

    const request = controller.request();
    expect(request).toMatchObject({
      projectId: "project-1",
      view: "list",
      pageSize: 100,
      dueRangeStart: "2026-07-01",
      visibleSectionIds: ["section-1"],
    });
    expect(request).not.toHaveProperty("groupBy");
  });

  it("keeps list and Kanban continuation cursors in their distinct request fields", async () => {
    const list = createController("list");
    list.controller.loadNextList(["selected-task"]);
    await vi.waitFor(() => expect(list.requests).toHaveLength(1));
    expect(list.requests[0]).toMatchObject({
      append: true,
      request: { cursor: "list-cursor", columnCursors: {} },
    });

    const kanban = createController("kanban");
    kanban.controller.loadNextKanban(["selected-task"]);
    await vi.waitFor(() => expect(kanban.requests).toHaveLength(1));
    expect(kanban.requests[0]).toMatchObject({
      append: true,
      request: { columnCursors: { "status-1": "column-cursor" } },
    });
    expect(kanban.requests[0].request).not.toHaveProperty("cursor");
  });
});
