import { describe, expect, it } from "vitest";
import {
  createQuickNoteHistory,
  quickNoteHistoryShortcutAction,
  recordQuickNoteHistory,
  stepQuickNoteHistory,
  type QuickNoteHistorySnapshot,
} from "./history";

const formatting = { bold: false, italic: false, underline: false };

function snapshot(text: string, caret: number): QuickNoteHistorySnapshot {
  return {
    runs: text ? [{ content: text, ...formatting }] : [],
    selection: { start: caret, end: caret },
    typingFormatting: formatting,
  };
}

describe("Quick note editor history", () => {
  it("coalesces uninterrupted typing and restores the exact caret", () => {
    let state = createQuickNoteHistory();
    state = recordQuickNoteHistory(state, snapshot("", 0), snapshot("a", 1), "insert", 1);
    state = recordQuickNoteHistory(state, snapshot("a", 1), snapshot("ab", 2), "insert", 2);
    expect(state.undo).toHaveLength(1);

    const undone = stepQuickNoteHistory(state, "undo");
    expect(undone?.snapshot).toEqual(snapshot("", 0));
    const redone = undone && stepQuickNoteHistory(undone.state, "redo");
    expect(redone?.snapshot).toEqual(snapshot("ab", 2));
  });

  it("keeps line breaks separate and does not coalesce after a caret move", () => {
    let state = createQuickNoteHistory();
    state = recordQuickNoteHistory(state, snapshot("a", 1), snapshot("a\n", 2), "break", 1);
    state = recordQuickNoteHistory(state, snapshot("a\n", 0), snapshot("xa\n", 1), "insert", 2);
    state = recordQuickNoteHistory(state, snapshot("xa\n", 2), snapshot("xya\n", 3), "insert", 3);
    expect(state.undo).toHaveLength(3);

    const undone = stepQuickNoteHistory(state, "undo");
    expect(undone?.snapshot.selection).toEqual({ start: 2, end: 2 });
  });

  it("supports platform undo and redo shortcuts", () => {
    const base = { altKey: false, ctrlKey: true, metaKey: false, shiftKey: false };
    expect(quickNoteHistoryShortcutAction({ ...base, key: "z" })).toBe("undo");
    expect(quickNoteHistoryShortcutAction({ ...base, key: "y" })).toBe("redo");
    expect(quickNoteHistoryShortcutAction({ ...base, key: "z", shiftKey: true })).toBe("redo");
    expect(quickNoteHistoryShortcutAction({ ...base, key: "a" })).toBeNull();
  });
});
