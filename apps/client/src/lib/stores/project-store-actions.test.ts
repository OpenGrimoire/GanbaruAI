import { beforeEach, describe, expect, it, vi } from "vitest";
import { linkProjectTaskEvent, updateProjectTask } from "$lib/api/projects";
import { createProjectStoreActions } from "$lib/stores/project-store-actions";
import { createProjectStoreSelectors } from "$lib/stores/project-store-selectors";
import actionSource from "$lib/stores/project-store-actions.ts?raw";
import type {
  ProjectMutation,
  ProjectsSnapshot,
  ProjectTask,
  ProjectTaskUpdate,
} from "$lib/projects/types";

vi.mock("$lib/api/projects", async (importOriginal) => ({
  ...await importOriginal<typeof import("$lib/api/projects")>(),
  updateProjectTask: vi.fn(),
  linkProjectTaskEvent: vi.fn(),
}));

function emptySnapshot(): ProjectsSnapshot {
  return {
    groups: [], projects: [], sections: [], statuses: [], priorities: [], tasks: [],
    checklistItems: [], tags: [], taskTagLinks: [], customFields: [], customFieldOptions: [],
    customFieldValues: [], customFieldOptionValues: [], dependencies: [], eventLinks: [],
    taskChangeEvents: [], viewPreferences: [], customEmojis: [],
  };
}

const task: ProjectTask = {
  id: "task-1", projectId: "project-1", sectionId: "section-1", statusId: "status-1",
  title: "Before", description: "", priority: "none", taskType: "task",
  sectionSortOrder: 1000, statusSortOrder: 1000, milestone: false,
  createdAt: "created", updatedAt: "before",
};

function taskMutation(changedTask: ProjectTask): ProjectMutation {
  return {
    changed: { ...emptySnapshot(), tasks: [changedTask] },
    removals: [],
    calendarEventProjectAssignments: [],
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((next) => { resolve = next; });
  return { promise, resolve };
}

describe("createProjectStoreActions", () => {
  beforeEach(() => vi.clearAllMocks());

  it("contains no ordinary mutation path that follows success with a snapshot reload", () => {
    expect(actionSource).not.toMatch(/\breload\s*\(/u);
  });

  function setup() {
    let snapshot = { ...emptySnapshot(), tasks: [task] };
    let loadGeneration = 0;
    const reloaded = { ...emptySnapshot(), tasks: [{ ...task, title: "Forced", updatedAt: "forced" }] };
    const reload = vi.fn(async (_projectId?: string | null) => {
      loadGeneration += 1;
      snapshot = reloaded;
    });
    const applyCalendarEventProjectAssignments = vi.fn(async () => undefined);
    const actions = createProjectStoreActions({
      selectors: createProjectStoreSelectors(() => snapshot),
      readSnapshot: () => snapshot,
      readLoadGeneration: () => loadGeneration,
      updateSnapshot: (updater) => { snapshot = updater(snapshot); },
      applyCalendarEventProjectAssignments,
      readSelectedProjectId: () => "project-1",
      setSelectedProjectId: vi.fn(),
      reload,
      ensureProjectData: vi.fn(async () => undefined),
    });
    return { actions, readSnapshot: () => snapshot, reload, applyCalendarEventProjectAssignments };
  }

  it("patches an authoritative task result with no follow-up snapshot load", async () => {
    vi.mocked(updateProjectTask).mockImplementation(async (update: ProjectTaskUpdate) =>
      taskMutation({ ...task, title: update.title, updatedAt: "server" }));
    const context = setup();

    await context.actions.updateTask(task, { title: "After" });

    expect(context.readSnapshot().tasks[0]).toMatchObject({ title: "After", updatedAt: "server" });
    expect(context.reload).not.toHaveBeenCalled();
  });

  it("keeps the snapshot unchanged when the command transaction fails", async () => {
    vi.mocked(updateProjectTask).mockRejectedValue(new Error("transaction rolled back"));
    const context = setup();

    await expect(context.actions.updateTask(task, { title: "After" })).rejects.toThrow("transaction rolled back");

    expect(context.readSnapshot().tasks[0]).toEqual(task);
    expect(context.reload).not.toHaveBeenCalled();
  });

  it("ignores an older response for the same entity", async () => {
    const first = deferred<ProjectMutation>();
    const second = deferred<ProjectMutation>();
    vi.mocked(updateProjectTask)
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const context = setup();
    const oldRequest = context.actions.updateTask(task, { title: "Old response" });
    const newRequest = context.actions.updateTask(task, { title: "New response" });

    second.resolve(taskMutation({ ...task, title: "New response", updatedAt: "new" }));
    await newRequest;
    first.resolve(taskMutation({ ...task, title: "Old response", updatedAt: "old" }));
    await oldRequest;

    expect(context.readSnapshot().tasks[0]).toMatchObject({ title: "New response", updatedAt: "new" });
  });

  it("allows an explicit forced reload to reconcile a local patch", async () => {
    vi.mocked(updateProjectTask).mockResolvedValue(
      taskMutation({ ...task, title: "Local patch", updatedAt: "local" }),
    );
    const context = setup();
    await context.actions.updateTask(task, { title: "Local patch" });

    await context.reload("project-1");

    expect(context.readSnapshot().tasks[0]).toMatchObject({ title: "Forced", updatedAt: "forced" });
    expect(context.reload).toHaveBeenCalledOnce();
  });

  it("does not apply a mutation response that predates a forced reload", async () => {
    const pending = deferred<ProjectMutation>();
    vi.mocked(updateProjectTask).mockReturnValue(pending.promise);
    const context = setup();
    const request = context.actions.updateTask(task, { title: "Stale" });

    await context.reload("project-1");
    pending.resolve(taskMutation({ ...task, title: "Stale", updatedAt: "stale" }));
    await request;

    expect(context.readSnapshot().tasks[0]).toMatchObject({ title: "Forced", updatedAt: "forced" });
  });

  it("patches event links and forwards a server-derived calendar assignment", async () => {
    vi.mocked(linkProjectTaskEvent).mockResolvedValue({
      changed: {
        ...emptySnapshot(),
        eventLinks: [{ taskId: "task-1", eventId: "event-1", linkKind: "scheduled", createdAt: "server" }],
      },
      removals: [],
      calendarEventProjectAssignments: [{ eventId: "event-1", projectId: "project-1" }],
    });
    const context = setup();

    await context.actions.linkTaskEvent("task-1", "event-1");

    expect(context.readSnapshot().eventLinks).toHaveLength(1);
    expect(context.applyCalendarEventProjectAssignments).toHaveBeenCalledWith([
      { eventId: "event-1", projectId: "project-1" },
    ]);
    expect(context.reload).not.toHaveBeenCalled();
  });
});
