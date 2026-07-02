import { describe, expect, it, vi } from "vitest";
import {
  createNotesDataSourcePropertyDraft,
  notesDataSourceSchemaDraftFromDto,
  notesDataSourceSchemaUpdateFromDraft,
  type NotesDataSourceSchemaPropertyDraft,
} from "./data-source-schema";
import type { NotesDataSource, NotesDatabaseView } from "./types";

vi.stubGlobal("crypto", {
  randomUUID: vi.fn(() => "99999999-9999-4999-8999-999999999999"),
});

const dataSource: NotesDataSource = {
  object: "data_source",
  id: "11111111-1111-4111-8111-111111111111",
  parent: {
    type: "database_id",
    database_id: "22222222-2222-4222-8222-222222222222",
  },
  database_parent: {
    type: "page_id",
    page_id: "33333333-3333-4333-8333-333333333333",
  },
  title: "Tasks",
  title_rich_text: [],
  description: [],
  icon: null,
  properties: {
    Name: {
      id: "title",
      name: "Name",
      description: "",
      type: "title",
      title: {},
    },
    Priority: {
      id: "priority",
      name: "Priority",
      description: "Task importance",
      type: "select",
      select: {
        options: [
          { id: "low", name: "Low", color: "blue" },
          { id: "high", name: "High", color: "red" },
        ],
      },
    },
    Estimate: {
      id: "estimate",
      name: "Estimate",
      description: "",
      type: "number",
      number: { format: "percent" },
    },
    Project: {
      id: "project_relation",
      name: "Project",
      description: "",
      type: "relation",
      relation: {
        data_source_id: "66666666-6666-4666-8666-666666666666",
        dual_property: {
          synced_property_id: "tasks_relation",
          synced_property_name: "Tasks",
        },
      },
    },
    "Project budget": {
      id: "project_budget",
      name: "Project budget",
      description: "",
      type: "rollup",
      rollup: {
        relation_property_id: "project_relation",
        relation_property_name: "Project",
        rollup_property_id: "budget",
        rollup_property_name: "Budget",
        function: "sum",
      },
    },
  },
  in_trash: false,
  source_provider: null,
  source_object_id: null,
  source_workspace_id: null,
  source_last_edited_time: null,
  created_time: "2026-07-01T00:00:00.000Z",
  last_edited_time: "2026-07-01T00:00:00.000Z",
};

const view: NotesDatabaseView = {
  object: "view",
  id: "44444444-4444-4444-8444-444444444444",
  parent: {
    type: "database_id",
    database_id: "22222222-2222-4222-8222-222222222222",
  },
  data_source_id: dataSource.id,
  name: "Table",
  type: "table",
  filter: null,
  sorts: [],
  configuration: {
    type: "table",
    table: {
      property_order: ["title", "estimate", "priority", "project_relation", "project_budget"],
      hidden_property_ids: ["priority"],
    },
  },
  url: null,
  source_provider: null,
  source_object_id: null,
  source_workspace_id: null,
  source_last_edited_time: null,
  created_time: "2026-07-01T00:00:00.000Z",
  last_edited_time: "2026-07-01T00:00:00.000Z",
};

describe("data source schema helpers", () => {
  it("loads schema draft properties in table order with visibility state", () => {
    const draft = notesDataSourceSchemaDraftFromDto(dataSource, view);

    expect(draft.map((property) => property.id)).toEqual([
      "title",
      "estimate",
      "priority",
      "project_relation",
      "project_budget",
    ]);
    expect(draft[1]?.numberFormat).toBe("percent");
    expect(draft[2]?.hidden).toBe(true);
    expect(draft[2]?.options[1]?.name).toBe("High");
    expect(draft[3]?.relationDataSourceId).toBe("66666666-6666-4666-8666-666666666666");
    expect(draft[3]?.relationSyncedPropertyId).toBe("tasks_relation");
    expect(draft[4]?.rollupRelationPropertyId).toBe("project_relation");
    expect(draft[4]?.rollupPropertyId).toBe("budget");
    expect(draft[4]?.rollupFunction).toBe("sum");
  });

  it("serializes renamed, hidden, and configured properties for Tauri", () => {
    const draft: NotesDataSourceSchemaPropertyDraft[] = [
      ...notesDataSourceSchemaDraftFromDto(dataSource, view),
      {
        ...createNotesDataSourcePropertyDraft("status", "Status", "status"),
        options: [
          { id: "todo", name: "Todo", color: "default", group: "To-do" },
          { id: "doing", name: "Doing", color: "blue", group: "In progress" },
        ],
      },
      {
        ...createNotesDataSourcePropertyDraft("unique_id", "Task ID", "task_id"),
        uniquePrefix: "TASK",
      },
    ];
    draft[1] = { ...draft[1]!, name: "Effort", hidden: true };

    const update = notesDataSourceSchemaUpdateFromDraft(draft);

    expect(update.property_order).toEqual([
      "title",
      "estimate",
      "priority",
      "project_relation",
      "project_budget",
      "status",
      "task_id",
    ]);
    expect(update.hidden_property_ids).toEqual(["estimate", "priority"]);
    expect(update.properties.Effort).toMatchObject({
      id: "estimate",
      name: "Effort",
      type: "number",
      number: { format: "percent" },
    });
    expect(update.properties.Status).toMatchObject({
      id: "status",
      type: "status",
      status: {
        groups: [
          { id: "To-do", name: "To-do", color: "gray", option_ids: ["todo"] },
          {
            id: "In progress",
            name: "In progress",
            color: "blue",
            option_ids: ["doing"],
          },
          { id: "Complete", name: "Complete", color: "green", option_ids: [] },
        ],
      },
    });
    expect(update.properties["Task ID"]).toMatchObject({
      unique_id: { prefix: "TASK" },
    });
    expect(update.properties.Project).toMatchObject({
      id: "project_relation",
      type: "relation",
      relation: {
        data_source_id: "66666666-6666-4666-8666-666666666666",
        dual_property: {
          synced_property_id: "tasks_relation",
          synced_property_name: "Tasks",
        },
      },
    });
    expect(update.properties["Project budget"]).toMatchObject({
      id: "project_budget",
      type: "rollup",
      rollup: {
        relation_property_id: "project_relation",
        relation_property_name: "Project",
        rollup_property_id: "budget",
        rollup_property_name: "Budget",
        function: "sum",
      },
    });
  });

  it("rejects duplicate property names before persistence", () => {
    const draft = notesDataSourceSchemaDraftFromDto(dataSource, view);
    draft[1] = { ...draft[1]!, name: "Name" };

    expect(() => notesDataSourceSchemaUpdateFromDraft(draft)).toThrow(
      "property names must be unique",
    );
  });
});
