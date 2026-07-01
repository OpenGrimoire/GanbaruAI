import { describe, expect, it } from "vitest";
import {
  clampNotesTextSelection,
  notesTextSelectionFromControl,
} from "./editor-selection";

describe("notes editor selection helpers", () => {
  it("normalizes reversed control selections", () => {
    expect(notesTextSelectionFromControl({ selectionStart: 8, selectionEnd: 3 })).toEqual({
      start: 3,
      end: 8,
    });
  });

  it("treats missing selection ends as a collapsed selection", () => {
    expect(notesTextSelectionFromControl({ selectionStart: 4, selectionEnd: null })).toEqual({
      start: 4,
      end: 4,
    });
  });

  it("clamps selections inside the current text length", () => {
    expect(clampNotesTextSelection({ start: -2, end: 12 }, 5)).toEqual({
      start: 0,
      end: 5,
    });
  });
});
