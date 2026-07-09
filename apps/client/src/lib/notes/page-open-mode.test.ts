import { describe, expect, it } from "vitest";
import {
  DEFAULT_NOTES_PAGE_OPEN_MODE,
  isNotesPageOpenMode,
  notesPageOpenModeForSelection,
  notesDefaultOpenModeForProject,
  type NotesPageOpenMode,
} from "./page-open-mode";

describe("notes page open mode", () => {
  it("uses a project override before the global default", () => {
    expect(notesDefaultOpenModeForProject("center", "side")).toBe("side");
    expect(notesDefaultOpenModeForProject("full", null)).toBe("full");
    expect(notesDefaultOpenModeForProject("side", undefined)).toBe("side");
  });

  it("uses center peek for fresh page selections", () => {
    expect(
      notesPageOpenModeForSelection({
        currentOpenMode: "side",
        hasOpenPage: false,
      }),
    ).toBe(DEFAULT_NOTES_PAGE_OPEN_MODE);
  });

  it("uses the configured default for fresh page selections", () => {
    expect(
      notesPageOpenModeForSelection({
        currentOpenMode: "center",
        defaultOpenMode: "side",
        hasOpenPage: false,
      }),
    ).toBe("side");
  });

  it.each<NotesPageOpenMode>(["side", "center", "full"])(
    "keeps %s mode when another page is already open",
    (currentOpenMode) => {
      expect(
        notesPageOpenModeForSelection({
          currentOpenMode,
          hasOpenPage: true,
        }),
      ).toBe(currentOpenMode);
    },
  );

  it("lets explicit open modes override the current mode", () => {
    expect(
      notesPageOpenModeForSelection({
        requestedOpenMode: "center",
        currentOpenMode: "side",
        hasOpenPage: true,
      }),
    ).toBe("center");
  });

  it("accepts only known open mode values", () => {
    expect(isNotesPageOpenMode("side")).toBe(true);
    expect(isNotesPageOpenMode("center")).toBe(true);
    expect(isNotesPageOpenMode("full")).toBe(true);
    expect(isNotesPageOpenMode("new-tab")).toBe(false);
    expect(isNotesPageOpenMode(undefined)).toBe(false);
  });
});
