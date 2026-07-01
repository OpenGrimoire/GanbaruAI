import { updateNotesBlock } from "$lib/api/notes";
import { applyBlockUpdate } from "$lib/notes/block-factory";
import type { NotesBlock, NotesBlockUpdate } from "$lib/notes/types";

interface PendingBlockSave {
  timer: ReturnType<typeof setTimeout>;
  update: NotesBlockUpdate;
}

export interface NotesBlockPersistenceContext {
  readBlock: (blockId: string) => NotesBlock | undefined;
  replaceBlock: (block: NotesBlock) => void;
  readSelectedPageId: () => string | null;
  loadPageTree: (pageId: string) => Promise<void>;
  setLoadError: (message: string) => void;
  debounceMs: number;
}

export interface NotesBlockPersistence {
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  flushBlockSave: (blockId: string) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
}

/**
 * Create debounced block persistence helpers for the Notes store.
 */
export function createNotesBlockPersistence(
  context: NotesBlockPersistenceContext,
): NotesBlockPersistence {
  const pendingBlockSaves = new Map<string, PendingBlockSave>();

  function localApplyBlockUpdate(blockId: string, update: NotesBlockUpdate): void {
    const block = context.readBlock(blockId);
    if (!block) return;
    context.replaceBlock(applyBlockUpdate(block, update));
  }

  async function saveBlockNow(blockId: string, update: NotesBlockUpdate): Promise<void> {
    const saved = await updateNotesBlock(blockId, update);
    context.replaceBlock(saved);
  }

  function scheduleBlockSave(blockId: string, update: NotesBlockUpdate): void {
    const pending = pendingBlockSaves.get(blockId);
    if (pending) clearTimeout(pending.timer);
    const timer = setTimeout(() => {
      pendingBlockSaves.delete(blockId);
      saveBlockNow(blockId, update).catch((error) => {
        context.setLoadError(error instanceof Error ? error.message : String(error));
        const selectedPageId = context.readSelectedPageId();
        if (selectedPageId) void context.loadPageTree(selectedPageId);
      });
    }, context.debounceMs);
    pendingBlockSaves.set(blockId, { timer, update });
  }

  async function flushBlockSave(blockId: string): Promise<void> {
    const pending = pendingBlockSaves.get(blockId);
    if (!pending) return;
    clearTimeout(pending.timer);
    pendingBlockSaves.delete(blockId);
    await saveBlockNow(blockId, pending.update);
  }

  async function flushPendingBlockSaves(): Promise<void> {
    const blockIds = [...pendingBlockSaves.keys()];
    await Promise.all(blockIds.map((blockId) => flushBlockSave(blockId)));
  }

  return {
    localApplyBlockUpdate,
    saveBlockNow,
    scheduleBlockSave,
    flushBlockSave,
    flushPendingBlockSaves,
  };
}
