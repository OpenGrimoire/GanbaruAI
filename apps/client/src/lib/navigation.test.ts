import { describe, expect, it } from "vitest";
import {
  APP_VIEWS,
  availableAppViews,
  canDetachMainView,
  isDetachableTabView,
  isView,
  isViewAvailable,
  firstMainView,
  mainTabViews,
  parseInitialViewSearch,
  viewLabel,
} from "./navigation";
import { platformProfileFor } from "./platform";

describe("navigation helpers", () => {
  it("accepts registered views only", () => {
    expect(isView("calendar")).toBe(true);
    expect(isView("chat")).toBe(true);
    expect(isView("music")).toBe(false);
    expect(isView("settings")).toBe(false);
    expect(isView(null)).toBe(false);
  });

  it("limits detached windows to title bar tabs", () => {
    expect(isDetachableTabView("calendar")).toBe(true);
    expect(isDetachableTabView("projects")).toBe(true);
    expect(isDetachableTabView("notes")).toBe(true);
    expect(isDetachableTabView("chat")).toBe(true);
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

  it("rejects unavailable Android views from direct and query navigation", () => {
    const android = platformProfileFor("android");

    expect(availableAppViews(android)).toEqual(["calendar", "projects", "notes"]);
    expect(isViewAvailable("notes", android)).toBe(true);
    expect(isViewAvailable("chat", android)).toBe(false);
    expect(parseInitialViewSearch("?view=notes", android)).toBe("notes");
    expect(parseInitialViewSearch("?view=chat", android)).toBeUndefined();
    expect(mainTabViews(new Set(), android)).toEqual(["calendar", "projects", "notes"]);
    expect(canDetachMainView(new Set(), android)).toBe(false);
  });

  it("provides user-facing labels for every view", () => {
    expect(viewLabel("calendar")).toBe("Calendar");
    expect(viewLabel("projects")).toBe("Projects");
    expect(viewLabel("notes")).toBe("Notes");
    expect(viewLabel("chat")).toBe("Chat");
  });

  it("removes detached tabs from the main tab list", () => {
    expect(mainTabViews(new Set(["calendar", "notes"]))).toEqual(["projects", "chat"]);
  });

  it("uses Calendar as the defensive fallback when no primary tab remains", () => {
    expect(firstMainView(new Set(["calendar", "projects", "notes", "chat"]))).toBe("calendar");
  });

  it("allows detaching only while another primary tab remains", () => {
    expect(canDetachMainView(new Set())).toBe(true);
    expect(canDetachMainView(new Set(["calendar"]))).toBe(true);
    expect(canDetachMainView(new Set(["calendar", "projects"]))).toBe(true);
    expect(canDetachMainView(new Set(["calendar", "projects", "notes"]))).toBe(false);
  });
});
