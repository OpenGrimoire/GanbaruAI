import { describe, expect, it } from "vitest";
import {
  APP_VIEWS,
  canDetachMainView,
  isDetachableTabView,
  isView,
  firstMainView,
  mainTabViews,
  parseInitialViewSearch,
  viewLabel,
} from "./navigation";

describe("navigation helpers", () => {
  it("accepts registered views only", () => {
    expect(isView("calendar")).toBe(true);
    expect(isView("music")).toBe(false);
    expect(isView("settings")).toBe(false);
    expect(isView(null)).toBe(false);
  });

  it("limits detached windows to title bar tabs", () => {
    expect(isDetachableTabView("calendar")).toBe(true);
    expect(isDetachableTabView("projects")).toBe(true);
    expect(isDetachableTabView("notes")).toBe(true);
    expect(isDetachableTabView("music")).toBe(false);
  });

  it("parses an initial view from the window search string", () => {
    for (const view of APP_VIEWS) {
      expect(parseInitialViewSearch(`?view=${view}`)).toBe(view);
    }
    expect(parseInitialViewSearch("?view=music")).toBeUndefined();
    expect(parseInitialViewSearch("?view=settings")).toBeUndefined();
    expect(parseInitialViewSearch("")).toBeUndefined();
  });

  it("provides user-facing labels for every view", () => {
    expect(viewLabel("calendar")).toBe("Calendar");
    expect(viewLabel("projects")).toBe("Projects");
    expect(viewLabel("notes")).toBe("Notes");
  });

  it("removes detached tabs from the main tab list", () => {
    expect(mainTabViews(new Set(["calendar", "notes"]))).toEqual(["projects"]);
  });

  it("uses Calendar as the defensive fallback when no primary tab remains", () => {
    expect(firstMainView(new Set(["calendar", "projects", "notes"]))).toBe("calendar");
  });

  it("allows detaching only while another primary tab remains", () => {
    expect(canDetachMainView(new Set())).toBe(true);
    expect(canDetachMainView(new Set(["calendar"]))).toBe(true);
    expect(canDetachMainView(new Set(["calendar", "projects"]))).toBe(false);
  });
});
