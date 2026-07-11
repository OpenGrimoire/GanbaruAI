// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import type {
  ProjectOptionalDataKind,
  ProjectsOptionalData,
  ProjectsSnapshot,
  ProjectsWorkspaceSnapshot,
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
  };
});

function emptySnapshot(): ProjectsSnapshot {
  return {
    groups: [{
      id: "group-1",
      name: "Group",
      icon: "folder",
      sortOrder: 0,
      collapsed: false,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }],
    projects: [{
      id: "project-1",
      groupId: "group-1",
      name: "Project",
      icon: "folder",
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
    tasks: [],
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

    const firstKanban = projects.ensureProjectViewData("project-1", "kanban");
    const secondKanban = projects.ensureProjectViewData("project-1", "kanban");
    expect(backend.optionalCalls).toEqual(["project-1:relationships"]);
    backend.resolveOptional(
      "relationships",
      "project-1",
      optionalData("relationships"),
    );
    await Promise.all([firstKanban, secondKanban]);

    const list = projects.ensureProjectViewData("project-1", "list");
    expect(backend.optionalCalls).toEqual([
      "project-1:relationships",
      "project-1:custom_fields",
    ]);
    backend.resolveOptional("custom_fields", "project-1", optionalData("custom_fields"));
    await list;

    const toolbar = projects.ensureProjectToolbarData("project-1");
    expect(backend.optionalCalls.at(-1)).toBe("project-1:saved_views");
    backend.resolveOptional("saved_views", "project-1", optionalData("saved_views"));
    await toolbar;

    const detail = projects.ensureTaskDetailData("project-1");
    expect(backend.optionalCalls.slice(-2)).toEqual([
      "project-1:history",
      "project-1:checklist",
    ]);
    backend.resolveOptional("history", "project-1", optionalData("history"));
    backend.resolveOptional("checklist", "project-1", optionalData("checklist"));
    await detail;

    expect(backend.optionalCalls).toEqual([
      "project-1:relationships",
      "project-1:custom_fields",
      "project-1:saved_views",
      "project-1:history",
      "project-1:checklist",
    ]);
  });
});
