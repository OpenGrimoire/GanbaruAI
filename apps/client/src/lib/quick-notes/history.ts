import type {
  QuickNoteFormatting,
  QuickNoteTextRun,
} from "./types";
import type { QuickNoteSelection } from "./rich-text";

export type QuickNoteHistoryAction = "undo" | "redo";
export type QuickNoteHistoryKind = "insert" | "delete" | "break" | "format" | "paste" | "replace";

export interface QuickNoteHistorySnapshot {
  runs: QuickNoteTextRun[];
  selection: QuickNoteSelection;
  typingFormatting: QuickNoteFormatting;
}

interface QuickNoteHistoryEntry {
  before: QuickNoteHistorySnapshot;
  after: QuickNoteHistorySnapshot;
  kind: QuickNoteHistoryKind;
  updatedAt: number;
}

export interface QuickNoteHistoryState {
  undo: QuickNoteHistoryEntry[];
  redo: QuickNoteHistoryEntry[];
}

const HISTORY_LIMIT = 100;
const COALESCE_WINDOW_MS = 1_000;

function cloneSnapshot(snapshot: QuickNoteHistorySnapshot): QuickNoteHistorySnapshot {
  return {
    runs: snapshot.runs.map((run) => ({ ...run })),
    selection: { ...snapshot.selection },
    typingFormatting: { ...snapshot.typingFormatting },
  };
}

function formattingEqual(left: QuickNoteFormatting, right: QuickNoteFormatting): boolean {
  return left.bold === right.bold
    && left.italic === right.italic
    && left.underline === right.underline;
}

function runsEqual(left: readonly QuickNoteTextRun[], right: readonly QuickNoteTextRun[]): boolean {
  return left.length === right.length && left.every((run, index) => {
    const other = right[index];
    return other !== undefined
      && run.content === other.content
      && run.bold === other.bold
      && run.italic === other.italic
      && run.underline === other.underline;
  });
}

function snapshotsEqual(left: QuickNoteHistorySnapshot, right: QuickNoteHistorySnapshot): boolean {
  return runsEqual(left.runs, right.runs)
    && left.selection.start === right.selection.start
    && left.selection.end === right.selection.end
    && formattingEqual(left.typingFormatting, right.typingFormatting);
}

function contentsEqual(left: QuickNoteHistorySnapshot, right: QuickNoteHistorySnapshot): boolean {
  return runsEqual(left.runs, right.runs);
}

export function createQuickNoteHistory(): QuickNoteHistoryState {
  return { undo: [], redo: [] };
}

export function recordQuickNoteHistory(
  state: QuickNoteHistoryState,
  before: QuickNoteHistorySnapshot,
  after: QuickNoteHistorySnapshot,
  kind: QuickNoteHistoryKind,
  now = Date.now(),
): QuickNoteHistoryState {
  if (contentsEqual(before, after)) return state;
  const latest = state.undo.at(-1);
  const canCoalesce = (kind === "insert" || kind === "delete")
    && latest?.kind === kind
    && now - latest.updatedAt <= COALESCE_WINDOW_MS
    && snapshotsEqual(latest.after, before);
  if (canCoalesce && latest) {
    return {
      undo: [
        ...state.undo.slice(0, -1),
        { ...latest, after: cloneSnapshot(after), updatedAt: now },
      ],
      redo: [],
    };
  }
  return {
    undo: [
      ...state.undo.slice(-(HISTORY_LIMIT - 1)),
      {
        before: cloneSnapshot(before),
        after: cloneSnapshot(after),
        kind,
        updatedAt: now,
      },
    ],
    redo: [],
  };
}

export function stepQuickNoteHistory(
  state: QuickNoteHistoryState,
  action: QuickNoteHistoryAction,
): { state: QuickNoteHistoryState; snapshot: QuickNoteHistorySnapshot } | null {
  const source = action === "undo" ? state.undo : state.redo;
  const entry = source.at(-1);
  if (!entry) return null;
  return action === "undo"
    ? {
        state: {
          undo: state.undo.slice(0, -1),
          redo: [...state.redo, entry],
        },
        snapshot: cloneSnapshot(entry.before),
      }
    : {
        state: {
          undo: [...state.undo, entry],
          redo: state.redo.slice(0, -1),
        },
        snapshot: cloneSnapshot(entry.after),
      };
}

export function quickNoteHistoryShortcutAction(
  event: Pick<KeyboardEvent, "altKey" | "ctrlKey" | "key" | "metaKey" | "shiftKey">,
): QuickNoteHistoryAction | null {
  if (!(event.ctrlKey || event.metaKey) || event.altKey) return null;
  const key = event.key.toLowerCase();
  if (key === "z") return event.shiftKey ? "redo" : "undo";
  if (key === "y" && !event.shiftKey) return "redo";
  return null;
}
