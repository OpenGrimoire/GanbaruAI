import { describe, expect, it } from "vitest";
import { mapPageSummary } from "./workspace-parsing";

function pageSummary(icon: string | null): Record<string, unknown> {
  return {
    id: "page-1",
    created_time: "2026-08-30T12:00:00Z",
    last_edited_time: "2026-08-30T12:00:00Z",
    parent_type: "workspace",
    parent_page_id: null,
    parent_block_id: null,
    parent_data_source_id: null,
    folder_id: null,
    title: "Page",
    project_id: null,
    icon,
  };
}

describe("Notes workspace parsing", () => {
  it("reports stored icon JSON failures with field context", () => {
    expect(() => mapPageSummary(pageSummary("{{"))).toThrow(
      'page.icon for page "page-1" must contain valid JSON',
    );
  });
});
