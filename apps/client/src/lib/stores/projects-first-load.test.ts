// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import type {
  ProjectOptionalDataKind,
  ProjectsOptionalData,
  ProjectsSnapshot,
  ProjectsWorkspaceSnapshot,
  ProjectTaskDetailData,
} from "$lib/projects/types";

const backend = vi.hoisted(() => {
  let resolveWorkspace: ((value: unknown) => void) | undefined;
  const workspace = new Promise<unknown>((resolve) => {
    resolveWorkspace = resolve;
  });
  const optionalResolvers = new Map<string, (value: unknown) => void>();
  const optionalCalls: string[] = [];
  return {
    workspace,
    workspaceCalls: 0,
    optionalCalls,
    detailCalls: 0,
    resolveWorkspace(value: unknown) {
      resolveWorkspace?.(value);
    },
    optionalRequest(kind: string, projectId: string | null) {
      const key = `${projectId ?? "global"}:${kind}`;
      optionalCalls.push(key);
      return new Promise<unknown>((resolve) => {
        optionalResolvers.set(key, resolve);
      });
    },
    resolveOptional(kind: string, projectId: string | null, value: unknown) {
      optionalResolvers.get(`${projectId ?? "global"}:${kind}`)?.(value);
    },
    loadDetail() {
      this.detailCalls += 1;
      const task = emptySnapshot().tasks[0];
      return Promise.resolve({
        task,
        relatedTasks: [], checklistItems: [], tags: [], taskTagLinks: [], customFields: [],
        customFieldOptions: [], customFieldValues: [], customFieldOptionValues: [],
        dependencies: [], eventLinks: [], taskChangeEvents: [],
      } satisfies ProjectTaskDetailData);
    },
  };
});

vi.mock("$lib/api/projects", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/projects")>();
  return {
    ...actual,
    loadProjectsWorkspace: () => {
      backend.workspaceCalls += 1;
      return backend.workspace;
    },
    refreshProjectsWorkspace: vi.fn(),
    loadProjectsOptionalData: (kind: string, projectId: string | null) =>
      backend.optionalRequest(kind, projectId),
    loadProjectTaskDetail: () => backend.loadDetail(),
  };
});

function emptySnapshot(): ProjectsSnapshot {
  return {
    groups: [{
      id: "group-1",
      name: "Group",
      icon: "lucide:folder",
      sortOrder: 0,
      collapsed: false,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }],
    projects: [{
      id: "project-1",
      groupId: "group-1",
      name: "Project",
      icon: "lucide:folder",
      color: 8,
      sortOrder: 0,
      status: "active",
      defaultEventName: null,
      defaultEventTimeMode: "timed",
      defaultEventDurationMinutes: 60,
      defaultPomodoroMode: "preset",
      defaultPomodoroPresetKey: "adaptive",
      defaultIdleSettingsSource: "global",
      defaultIdlePauseEnabled: true,
      defaultIdleThresholdMinutes: 5,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }],
    sections: [],
    statuses: [],
    priorities: [],
    tasks: [{
      id: "task-1", projectId: "project-1", sectionId: "section-1", statusId: "status-1",
      title: "Task", description: "", priority: "normal", taskType: "task",
      sectionSortOrder: 0, statusSortOrder: 0, milestone: false,
      createdAt: "2026-01-01T00:00:00Z", updatedAt: "2026-01-01T00:00:00Z",
      detailLoaded: false,
    }],
    checklistItems: [],
    tags: [],
    taskTagLinks: [],
    customFields: [],
    customFieldOptions: [],
    customFieldValues: [],
    customFieldOptionValues: [],
    dependencies: [],
    eventLinks: [],
    taskChangeEvents: [],
    viewPreferences: [],
    customEmojis: [],
  };
}

function optionalData(
  kind: ProjectOptionalDataKind,
  projectId: string | null = "project-1",
): ProjectsOptionalData {
  return {
    kind,
    projectId,
    checklistItems: [],
    tags: [],
    taskTagLinks: [],
    customFields: [],
    customFieldOptions: [],
    customFieldValues: [],
    customFieldOptionValues: [],
    dependencies: [],
    eventLinks: [],
    taskChangeEvents: [],
    viewPreferences: [],
    customEmojis: [],
  };
}

describe("Projects initial loading", () => {
  it("uses one workspace request and single-flights view and panel data", async () => {
    const { getProjects } = await import("./projects.svelte");
    const projects = getProjects();
    const firstLoad = projects.ensureLoaded();
    const secondLoad = projects.ensureLoaded();

    expect(backend.workspaceCalls).toBe(1);
    backend.resolveWorkspace({
      resolvedProjectId: "project-1",
      activeView: "list",
      snapshot: emptySnapshot(),
    } satisfies ProjectsWorkspaceSnapshot);
    await Promise.all([firstLoad, secondLoad]);
    expect(projects.selectedProjectId).toBe("project-1");
    expect(projects.projectDataLoaded("project-1")).toBe(true);
    expect(backend.workspaceCalls).toBe(1);

    await Promise.all([
      projects.ensureProjectViewData("project-1", "kanban"),
      projects.ensureProjectViewData("project-1", "list"),
    ]);

    const toolbar = projects.ensureProjectToolbarData("project-1");
    expect(backend.optionalCalls).toEqual(["project-1:saved_views"]);
    backend.resolveOptional("saved_views", "project-1", optionalData("saved_views"));
    await toolbar;

    await Promise.all([
      projects.ensureTaskDetailData("project-1", "task-1"),
      projects.ensureTaskDetailData("project-1", "task-1"),
    ]);
    expect(backend.detailCalls).toBe(1);
    expect(backend.optionalCalls).toEqual(["project-1:saved_views"]);
  });
});
