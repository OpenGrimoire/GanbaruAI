import { describe, expect, it } from "vitest";
import {
  DEFAULT_PROJECT_VIEW_ID,
  isProjectViewId,
  parseStoredProjectId,
  parseStoredProjectViewId,
} from "./project-ui-preferences";

describe("project ui preferences", () => {
  it("accepts supported project view ids only", () => {
    expect(isProjectViewId("list")).toBe(true);
    expect(isProjectViewId("board")).toBe(true);
    expect(isProjectViewId("calendar")).toBe(true);
    expect(isProjectViewId("gantt")).toBe(true);
    expect(isProjectViewId("summary")).toBe(true);
    expect(isProjectViewId("timeline")).toBe(false);
    expect(isProjectViewId(undefined)).toBe(false);
  });

  it("falls back to the list view for invalid stored view ids", () => {
    expect(parseStoredProjectViewId("board")).toBe("board");
    expect(parseStoredProjectViewId("timeline")).toBe(DEFAULT_PROJECT_VIEW_ID);
    expect(parseStoredProjectViewId(null)).toBe(DEFAULT_PROJECT_VIEW_ID);
  });

  it("trims saved project ids and rejects blank values", () => {
    expect(parseStoredProjectId(" project-a ")).toBe("project-a");
    expect(parseStoredProjectId("   ")).toBeNull();
    expect(parseStoredProjectId(null)).toBeNull();
  });
});
