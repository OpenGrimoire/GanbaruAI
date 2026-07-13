import { describe, expect, it } from "vitest";
import { applyProjectMutation } from "$lib/projects/project-snapshot-mutations";
import type { ProjectMutation, ProjectsSnapshot } from "$lib/projects/types";

function emptySnapshot(): ProjectsSnapshot {
  return {
    groups: [], projects: [], sections: [], statuses: [], priorities: [], tasks: [],
    checklistItems: [], tags: [], taskTagLinks: [], customFields: [], customFieldOptions: [],
    customFieldValues: [], customFieldOptionValues: [], dependencies: [], eventLinks: [],
    taskChangeEvents: [], viewPreferences: [], customEmojis: [],
  };
}

function mutation(changed: Partial<ProjectsSnapshot>, removals: ProjectMutation["removals"] = []): ProjectMutation {
  return {
    changed: { ...emptySnapshot(), ...changed },
    removals,
    calendarEventProjectAssignments: [],
  };
}

describe("applyProjectMutation", () => {
  it("immutably upserts authoritative rows for simple and composite collections", () => {
    const initial = emptySnapshot();
    const result = applyProjectMutation(initial, mutation({
      groups: [{
        id: "group-1", name: "Work", icon: "lucide:folder", sortOrder: 1000, collapsed: false,
        createdAt: "created", updatedAt: "updated",
      }],
      taskTagLinks: [{ taskId: "task-1", tagId: "tag-1", createdAt: "created" }],
      customFieldValues: [{ taskId: "task-1", fieldId: "field-1", textValue: "value", updatedAt: "updated" }],
      eventLinks: [{ taskId: "task-1", eventId: "event-1", linkKind: "scheduled", createdAt: "created" }],
    }));

    expect(result.groups[0]?.name).toBe("Work");
    expect(result.taskTagLinks).toHaveLength(1);
    expect(result.customFieldValues[0]?.textValue).toBe("value");
    expect(result.eventLinks[0]?.eventId).toBe("event-1");
    expect(initial).toEqual(emptySnapshot());
  });

  it("applies removals and their local cascade before replacement rows", () => {
    const initial = mutation({
      groups: [{ id: "group-1", name: "Work", icon: "lucide:folder", sortOrder: 1000, collapsed: false, createdAt: "c", updatedAt: "u" }],
      projects: [{
        id: "project-1", groupId: "group-1", name: "Project", icon: "lucide:folder", sortOrder: 1000,
        status: "active", defaultEventName: null, defaultEventTimeMode: "timed",
        defaultEventDurationMinutes: null, defaultPomodoroMode: "preset", defaultIdleSettingsSource: "global",
        defaultIdlePauseEnabled: true, defaultIdleThresholdMinutes: 5, createdAt: "c", updatedAt: "u",
      }],
      tasks: [{
        id: "task-1", projectId: "project-1", sectionId: "section-1", statusId: "status-1",
        title: "Task", description: "", priority: "none", taskType: "task", sectionSortOrder: 1000,
        statusSortOrder: 1000, milestone: false, createdAt: "c", updatedAt: "u",
      }],
      tags: [{ id: "tag-1", projectId: "project-1", name: "Tag", sortOrder: 1000, createdAt: "c", updatedAt: "u" }],
      taskTagLinks: [{ taskId: "task-1", tagId: "tag-1", createdAt: "c" }],
      checklistItems: [{ id: "item-1", taskId: "task-1", title: "Item", sortOrder: 1000, createdAt: "c", updatedAt: "u" }],
    }).changed;

    const result = applyProjectMutation(initial, mutation({}, [{ kind: "group", id: "group-1" }]));

    expect(result.groups).toEqual([]);
    expect(result.projects).toEqual([]);
    expect(result.tasks).toEqual([]);
    expect(result.tags).toEqual([]);
    expect(result.taskTagLinks).toEqual([]);
    expect(result.checklistItems).toEqual([]);
  });

  it("clears scalar and option values before applying the authoritative replacement", () => {
    const initial = mutation({
      customFieldValues: [{ taskId: "task-1", fieldId: "field-1", textValue: "old", updatedAt: "old" }],
      customFieldOptionValues: [{ taskId: "task-1", fieldId: "field-1", optionId: "option-old", createdAt: "old" }],
    }).changed;
    const result = applyProjectMutation(initial, mutation({
      customFieldOptionValues: [{ taskId: "task-1", fieldId: "field-1", optionId: "option-new", createdAt: "new" }],
    }, [{ kind: "custom_field_value", taskId: "task-1", fieldId: "field-1" }]));

    expect(result.customFieldValues).toEqual([]);
    expect(result.customFieldOptionValues.map((value) => value.optionId)).toEqual(["option-new"]);
  });
});
