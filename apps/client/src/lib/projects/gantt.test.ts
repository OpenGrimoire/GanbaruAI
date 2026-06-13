import { describe, expect, it } from "vitest";
import {
  buildProjectDependencyCascadeProposal,
  buildProjectGanttDatePatch,
  buildProjectGanttTimeline,
} from "./gantt";
import type { ProjectStatus, ProjectTask, ProjectTaskDependency } from "./types";

const statuses: ProjectStatus[] = [
  status("todo", "To do", "not_started", false),
  status("blocked", "Blocked", "blocked", false),
  status("done", "Done", "done", true),
];

function status(
  id: string,
  name: string,
  category: ProjectStatus["category"],
  terminal: boolean,
): ProjectStatus {
  return {
    id,
    projectId: "project",
    name,
    category,
    sortOrder: 1000,
    terminal,
    createdAt: "2026-06-01T00:00:00Z",
    updatedAt: "2026-06-01T00:00:00Z",
  };
}

function task(input: Partial<ProjectTask> & Pick<ProjectTask, "id" | "title">): ProjectTask {
  return {
    id: input.id,
    projectId: "project",
    sectionId: input.sectionId ?? "section",
    statusId: input.statusId ?? "todo",
    parentTaskId: input.parentTaskId,
    title: input.title,
    description: input.description ?? "",
    priority: input.priority ?? "normal",
    taskType: input.taskType ?? "task",
    sectionSortOrder: input.sectionSortOrder ?? 1000,
    statusSortOrder: input.statusSortOrder ?? 1000,
    estimateMinutes: input.estimateMinutes,
    dueDate: input.dueDate,
    startDate: input.startDate,
    targetEndDate: input.targetEndDate,
    completedAt: input.completedAt,
    archivedAt: input.archivedAt,
    blockerReason: input.blockerReason,
    milestone: input.milestone ?? false,
    createdAt: input.createdAt ?? "2026-06-01T00:00:00Z",
    updatedAt: input.updatedAt ?? "2026-06-01T00:00:00Z",
  };
}

function dependency(input: Pick<ProjectTaskDependency, "blockingTaskId" | "blockedTaskId">): ProjectTaskDependency {
  return {
    id: `${input.blockingTaskId}-${input.blockedTaskId}`,
    dependencyType: "blocks",
    createdAt: "2026-06-01T00:00:00Z",
    ...input,
  };
}

describe("buildProjectGanttTimeline", () => {
  it("builds proportional task bars from dated tasks", () => {
    const timeline = buildProjectGanttTimeline({
      tasks: [
        task({ id: "a", title: "A", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
        task({ id: "b", title: "B", startDate: "2026-06-13", targetEndDate: "2026-06-14" }),
      ],
      statuses,
      dependencies: [],
      dependencyBlockedTaskIds: new Set(),
      today: "2026-06-11",
    });

    expect(timeline.startDate).toBe("2026-06-10");
    expect(timeline.endDate).toBe("2026-06-14");
    expect(timeline.totalDays).toBe(5);
    expect(timeline.rows.map((row) => row.task.id)).toEqual(["a", "b"]);
    expect(timeline.rows[0].leftPercent).toBe(0);
    expect(timeline.rows[0].widthPercent).toBe(60);
    expect(timeline.todayPercent).toBe(30);
  });

  it("renders milestones as fixed date markers", () => {
    const timeline = buildProjectGanttTimeline({
      tasks: [
        task({ id: "range", title: "Range", startDate: "2026-06-10", targetEndDate: "2026-06-14" }),
        task({ id: "launch", title: "Launch", taskType: "milestone", dueDate: "2026-06-12" }),
      ],
      statuses,
      dependencies: [],
      dependencyBlockedTaskIds: new Set(),
      today: "2026-06-10",
    });

    const milestone = timeline.rows.find((row) => row.task.id === "launch");

    expect(milestone?.milestone).toBe(true);
    expect(milestone?.startDate).toBe("2026-06-12");
    expect(milestone?.endDate).toBe("2026-06-12");
    expect(milestone?.markerPercent).toBe(50);
  });

  it("marks overdue, blocked, done, and dependency counts", () => {
    const timeline = buildProjectGanttTimeline({
      tasks: [
        task({ id: "late", title: "Late", dueDate: "2026-06-10" }),
        task({ id: "blocked", title: "Blocked", statusId: "blocked", dueDate: "2026-06-13" }),
        task({ id: "done", title: "Done", statusId: "done", dueDate: "2026-06-09" }),
      ],
      statuses,
      dependencies: [
        dependency({ blockingTaskId: "blocked", blockedTaskId: "late" }),
        dependency({ blockingTaskId: "done", blockedTaskId: "late" }),
      ],
      dependencyBlockedTaskIds: new Set(["late"]),
      today: "2026-06-12",
    });

    const late = timeline.rows.find((row) => row.task.id === "late");
    const blocked = timeline.rows.find((row) => row.task.id === "blocked");
    const done = timeline.rows.find((row) => row.task.id === "done");

    expect(late?.overdue).toBe(true);
    expect(late?.blocked).toBe(true);
    expect(late?.blockedByCount).toBe(2);
    expect(blocked?.blocked).toBe(true);
    expect(blocked?.blocksCount).toBe(1);
    expect(done?.overdue).toBe(false);
    expect(done?.done).toBe(true);
    expect(timeline.dependencyEdges).toHaveLength(2);
  });

  it("builds finish-to-start dependency edges and detects conflicts", () => {
    const timeline = buildProjectGanttTimeline({
      tasks: [
        task({ id: "blocking", title: "Blocking", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
        task({ id: "blocked", title: "Blocked", startDate: "2026-06-13", targetEndDate: "2026-06-14" }),
        task({ id: "conflict", title: "Conflict", startDate: "2026-06-11", targetEndDate: "2026-06-15" }),
        task({ id: "undated", title: "Undated" }),
      ],
      statuses,
      dependencies: [
        dependency({ blockingTaskId: "blocking", blockedTaskId: "blocked" }),
        dependency({ blockingTaskId: "blocking", blockedTaskId: "conflict" }),
        dependency({ blockingTaskId: "blocking", blockedTaskId: "undated" }),
      ],
      dependencyBlockedTaskIds: new Set(),
      today: "2026-06-10",
    });

    expect(timeline.dependencyEdges.map((edge) => edge.blockedTaskId)).toEqual(["blocked", "conflict"]);
    expect(timeline.dependencyEdges[0]).toMatchObject({
      blockingTaskId: "blocking",
      blockedTaskId: "blocked",
      lagDays: 1,
      violated: false,
    });
    expect(timeline.dependencyEdges[1]).toMatchObject({
      blockingTaskId: "blocking",
      blockedTaskId: "conflict",
      lagDays: -1,
      violated: true,
    });
  });

  it("returns an empty timeline when no tasks have dates", () => {
    const timeline = buildProjectGanttTimeline({
      tasks: [task({ id: "undated", title: "Undated" })],
      statuses,
      dependencies: [],
      dependencyBlockedTaskIds: new Set(),
      today: "2026-06-12",
    });

    expect(timeline.rows).toEqual([]);
    expect(timeline.ticks).toEqual([]);
    expect(timeline.dependencyEdges).toEqual([]);
  });
});

describe("buildProjectDependencyCascadeProposal", () => {
  it("proposes shifting a blocked task after the blocking task finishes", () => {
    const proposal = buildProjectDependencyCascadeProposal({
      tasks: [
        task({ id: "blocking", title: "Blocking", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
        task({ id: "blocked", title: "Blocked", startDate: "2026-06-11", targetEndDate: "2026-06-13" }),
      ],
      dependencies: [
        dependency({ blockingTaskId: "blocking", blockedTaskId: "blocked" }),
      ],
    });

    expect(proposal.unresolvedDependencyIds).toEqual([]);
    expect(proposal.items).toHaveLength(1);
    expect(proposal.items[0]).toMatchObject({
      taskId: "blocked",
      shiftDays: 2,
      originalRangeStart: "2026-06-11",
      originalRangeEnd: "2026-06-13",
      nextStartDate: "2026-06-13",
      nextTargetEndDate: "2026-06-15",
      nextRangeStart: "2026-06-13",
      nextRangeEnd: "2026-06-15",
    });
    expect(proposal.items[0].reasons.map((reason) => reason.blockingTaskId)).toEqual(["blocking"]);
  });

  it("propagates shifts through chained dependencies", () => {
    const proposal = buildProjectDependencyCascadeProposal({
      tasks: [
        task({ id: "a", title: "A", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
        task({ id: "b", title: "B", startDate: "2026-06-11", targetEndDate: "2026-06-13" }),
        task({ id: "c", title: "C", startDate: "2026-06-14", targetEndDate: "2026-06-15" }),
      ],
      dependencies: [
        dependency({ blockingTaskId: "a", blockedTaskId: "b" }),
        dependency({ blockingTaskId: "b", blockedTaskId: "c" }),
      ],
    });

    expect(proposal.items.map((item) => [item.taskId, item.nextRangeStart, item.nextRangeEnd])).toEqual([
      ["b", "2026-06-13", "2026-06-15"],
      ["c", "2026-06-16", "2026-06-17"],
    ]);
  });

  it("shifts only date fields the task already uses", () => {
    const proposal = buildProjectDependencyCascadeProposal({
      tasks: [
        task({ id: "blocking", title: "Blocking", dueDate: "2026-06-12" }),
        task({ id: "blocked", title: "Blocked", dueDate: "2026-06-12" }),
      ],
      dependencies: [
        dependency({ blockingTaskId: "blocking", blockedTaskId: "blocked" }),
      ],
    });

    expect(proposal.items[0]).toMatchObject({
      taskId: "blocked",
      nextStartDate: undefined,
      nextDueDate: "2026-06-13",
      nextTargetEndDate: undefined,
      nextRangeStart: "2026-06-13",
      nextRangeEnd: "2026-06-13",
    });
  });

  it("does not propose dates for undated blocked tasks", () => {
    const proposal = buildProjectDependencyCascadeProposal({
      tasks: [
        task({ id: "blocking", title: "Blocking", dueDate: "2026-06-12" }),
        task({ id: "blocked", title: "Blocked" }),
      ],
      dependencies: [
        dependency({ blockingTaskId: "blocking", blockedTaskId: "blocked" }),
      ],
    });

    expect(proposal.items).toEqual([]);
    expect(proposal.unresolvedDependencyIds).toEqual([]);
  });
});

describe("buildProjectGanttDatePatch", () => {
  it("moves every existing date field by the same day delta", () => {
    const patch = buildProjectGanttDatePatch(
      task({
        id: "task",
        title: "Task",
        startDate: "2026-06-10",
        dueDate: "2026-06-12",
        targetEndDate: "2026-06-14",
      }),
      "move",
      2,
    );

    expect(patch).toEqual({
      startDate: "2026-06-12",
      dueDate: "2026-06-14",
      targetEndDate: "2026-06-16",
    });
  });

  it("resizes the start edge without changing due or target end dates", () => {
    const patch = buildProjectGanttDatePatch(
      task({
        id: "task",
        title: "Task",
        startDate: "2026-06-10",
        dueDate: "2026-06-13",
        targetEndDate: "2026-06-14",
      }),
      "resize-start",
      1,
    );

    expect(patch).toEqual({
      startDate: "2026-06-11",
      dueDate: "2026-06-13",
      targetEndDate: "2026-06-14",
    });
  });

  it("resizes the end edge through target end when it exists", () => {
    const patch = buildProjectGanttDatePatch(
      task({
        id: "task",
        title: "Task",
        startDate: "2026-06-10",
        dueDate: "2026-06-13",
        targetEndDate: "2026-06-14",
      }),
      "resize-end",
      -2,
    );

    expect(patch).toEqual({
      startDate: "2026-06-10",
      dueDate: "2026-06-13",
      targetEndDate: "2026-06-12",
    });
  });

  it("creates a start date when a due-only task is resized from the start edge", () => {
    const patch = buildProjectGanttDatePatch(
      task({ id: "task", title: "Task", dueDate: "2026-06-12" }),
      "resize-start",
      -3,
    );

    expect(patch).toEqual({
      startDate: "2026-06-09",
      dueDate: "2026-06-12",
      targetEndDate: undefined,
    });
  });

  it("creates a target end date when a start-only task is resized from the end edge", () => {
    const patch = buildProjectGanttDatePatch(
      task({ id: "task", title: "Task", startDate: "2026-06-12" }),
      "resize-end",
      4,
    );

    expect(patch).toEqual({
      startDate: "2026-06-12",
      dueDate: undefined,
      targetEndDate: "2026-06-16",
    });
  });

  it("moves milestones instead of resizing them", () => {
    const patch = buildProjectGanttDatePatch(
      task({ id: "milestone", title: "Milestone", taskType: "milestone", dueDate: "2026-06-12" }),
      "resize-end",
      5,
    );

    expect(patch).toEqual({
      startDate: undefined,
      dueDate: "2026-06-17",
      targetEndDate: undefined,
    });
  });

  it("clamps resized ranges so start and end do not cross", () => {
    const startPatch = buildProjectGanttDatePatch(
      task({ id: "task", title: "Task", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
      "resize-start",
      10,
    );
    const endPatch = buildProjectGanttDatePatch(
      task({ id: "task", title: "Task", startDate: "2026-06-10", targetEndDate: "2026-06-12" }),
      "resize-end",
      -10,
    );

    expect(startPatch?.startDate).toBe("2026-06-12");
    expect(endPatch?.targetEndDate).toBe("2026-06-10");
  });
});
