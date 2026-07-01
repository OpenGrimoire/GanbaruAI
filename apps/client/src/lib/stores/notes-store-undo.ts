import {
  clearNotesUndoState,
  loadNotesUndoState,
  moveNotesBlock,
  saveNotesUndoState,
  trashNotesBlock,
  updateNotesBlock,
} from "$lib/api/notes";
import { blockUpdateFromBlock } from "$lib/notes/block-factory";
import { parentIdForBlock, type NotesTreeState } from "$lib/notes/block-tree";
import {
  createNotesUndoSnapshot,
  parentIdsByDepth,
  parseNotesUndoStateJson,
  recordNotesUndoEntry,
  serializeNotesUndoState,
  type NotesUndoEntry,
  type NotesUndoRecordOptions,
  type NotesUndoSnapshot,
  type NotesUndoState,
} from "$lib/notes/undo-history";
import type { NotesBlock } from "$lib/notes/types";

const EMPTY_UNDO_STATE: NotesUndoState = { undo: [], redo: [] };

export interface NotesUndoControllerContext {
  readSelectedPageId: () => string | null;
  readTreeState: () => NotesTreeState;
  loadPageTreeForUndo: (pageId: string) => Promise<void>;
  requestBlockFocus: (blockId: string | null) => void;
  flushPendingBlockSaves: () => Promise<void>;
  setLoadError: (message: string) => void;
}

export interface NotesUndoController {
  hydrate: (pageId: string | null) => Promise<void>;
  snapshot: (
    focusBlockId: string | null,
    extraBlocks?: readonly NotesBlock[],
  ) => NotesUndoSnapshot | null;
  record: (options: Omit<NotesUndoRecordOptions, "id">) => void;
  undo: () => Promise<boolean>;
  redo: () => Promise<boolean>;
  canUndo: () => boolean;
  canRedo: () => boolean;
}

function cloneEntry(entry: NotesUndoEntry): NotesUndoEntry {
  return structuredClone(entry);
}

function entryIdsByPresence(
  target: NotesUndoSnapshot,
  source: NotesUndoSnapshot,
): { targetOnlyRoots: NotesBlock[]; sourceOnlyRoots: NotesBlock[] } {
  const targetIds = new Set(target.blocks.map((block) => block.id));
  const sourceIds = new Set(source.blocks.map((block) => block.id));
  const targetOnlyIds = new Set([...targetIds].filter((blockId) => !sourceIds.has(blockId)));
  const sourceOnlyIds = new Set([...sourceIds].filter((blockId) => !targetIds.has(blockId)));
  return {
    targetOnlyRoots: target.blocks.filter((block) => {
      if (!targetOnlyIds.has(block.id)) return false;
      return !targetOnlyIds.has(parentIdForBlock(block));
    }),
    sourceOnlyRoots: source.blocks.filter((block) => {
      if (!sourceOnlyIds.has(block.id)) return false;
      return !sourceOnlyIds.has(parentIdForBlock(block));
    }),
  };
}

function snapshotBlocksById(snapshot: NotesUndoSnapshot): Map<string, NotesBlock> {
  return new Map(snapshot.blocks.map((block) => [block.id, block]));
}

function stackWithLimit(entries: readonly NotesUndoEntry[]): NotesUndoEntry[] {
  return entries.slice(-40).map(cloneEntry);
}

async function applyUndoSnapshot(
  target: NotesUndoSnapshot,
  source: NotesUndoSnapshot,
): Promise<void> {
  const { targetOnlyRoots, sourceOnlyRoots } = entryIdsByPresence(target, source);
  for (const block of sourceOnlyRoots) {
    await trashNotesBlock(block.id, true);
  }
  for (const block of targetOnlyRoots) {
    await trashNotesBlock(block.id, false);
  }

  for (const block of target.blocks) {
    await updateNotesBlock(block.id, blockUpdateFromBlock(block));
  }

  const targetById = snapshotBlocksById(target);
  for (const parentId of parentIdsByDepth(target)) {
    let before: string | null = null;
    const childIds = [...(target.childIdsByParentId[parentId] ?? [])]
      .filter((childId) => targetById.has(childId))
      .reverse();
    for (const childId of childIds) {
      const block = targetById.get(childId);
      if (!block) continue;
      await moveNotesBlock(childId, {
        parent: block.parent,
        after: null,
        before,
      });
      before = childId;
    }
  }
}

/**
 * Create per-page Notes undo and redo history with SQLite-backed recovery state.
 */
export function createNotesUndoController(
  context: NotesUndoControllerContext,
): NotesUndoController {
  let state: NotesUndoState = EMPTY_UNDO_STATE;
  let hydratedPageId: string | null = null;
  let hydrateRequestId = 0;
  let persistTimer: ReturnType<typeof setTimeout> | null = null;

  function persistPageId(): string | null {
    return state.undo.at(-1)?.after.pageId
      ?? state.redo.at(-1)?.before.pageId
      ?? hydratedPageId;
  }

  async function persistNow(): Promise<void> {
    if (persistTimer) {
      clearTimeout(persistTimer);
      persistTimer = null;
    }
    const pageId = persistPageId();
    if (!pageId) return;
    try {
      if (state.undo.length === 0 && state.redo.length === 0) {
        await clearNotesUndoState(pageId);
        return;
      }
      await saveNotesUndoState(pageId, serializeNotesUndoState(state));
    } catch (error) {
      context.setLoadError(error instanceof Error ? error.message : String(error));
    }
  }

  function schedulePersist(): void {
    if (persistTimer) clearTimeout(persistTimer);
    persistTimer = setTimeout(() => {
      void persistNow();
    }, 250);
  }

  async function hydrate(pageId: string | null): Promise<void> {
    const requestId = ++hydrateRequestId;
    hydratedPageId = pageId;
    if (!pageId) {
      state = EMPTY_UNDO_STATE;
      return;
    }
    try {
      const stateJson = await loadNotesUndoState(pageId);
      if (requestId !== hydrateRequestId) return;
      state = stateJson ? parseNotesUndoStateJson(stateJson) : EMPTY_UNDO_STATE;
    } catch (error) {
      if (requestId !== hydrateRequestId) return;
      state = EMPTY_UNDO_STATE;
      await clearNotesUndoState(pageId).catch(() => undefined);
      context.setLoadError(error instanceof Error ? error.message : String(error));
    }
  }

  function snapshot(
    focusBlockId: string | null,
    extraBlocks: readonly NotesBlock[] = [],
  ): NotesUndoSnapshot | null {
    return createNotesUndoSnapshot(
      context.readSelectedPageId(),
      context.readTreeState(),
      focusBlockId,
      extraBlocks,
    );
  }

  function record(options: Omit<NotesUndoRecordOptions, "id">): void {
    state = recordNotesUndoEntry(state, {
      ...options,
      id: crypto.randomUUID(),
    });
    schedulePersist();
  }

  async function applyEntry(entry: NotesUndoEntry, direction: "undo" | "redo"): Promise<boolean> {
    await context.flushPendingBlockSaves();
    const target = direction === "undo" ? entry.before : entry.after;
    const source = direction === "undo" ? entry.after : entry.before;
    await applyUndoSnapshot(target, source);
    await context.loadPageTreeForUndo(target.pageId);
    context.requestBlockFocus(target.focusBlockId);
    return true;
  }

  async function undo(): Promise<boolean> {
    const entry = state.undo.at(-1);
    if (!entry) return false;
    await applyEntry(entry, "undo");
    state = {
      undo: state.undo.slice(0, -1).map(cloneEntry),
      redo: stackWithLimit([...state.redo, entry]),
    };
    await persistNow();
    return true;
  }

  async function redo(): Promise<boolean> {
    const entry = state.redo.at(-1);
    if (!entry) return false;
    await applyEntry(entry, "redo");
    state = {
      undo: stackWithLimit([...state.undo, entry]),
      redo: state.redo.slice(0, -1).map(cloneEntry),
    };
    await persistNow();
    return true;
  }

  return {
    hydrate,
    snapshot,
    record,
    undo,
    redo,
    canUndo: () => state.undo.length > 0,
    canRedo: () => state.redo.length > 0,
  };
}
