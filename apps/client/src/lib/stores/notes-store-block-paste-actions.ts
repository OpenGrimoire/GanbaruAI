import { cloneNotesJson } from "$lib/notes/json-clone";
import { planNotesPlainTextPaste } from "$lib/notes/block-clipboard";
import { planNotesRichHtmlPaste } from "$lib/notes/rich-text-paste";
import {
  blockColor,
} from "$lib/notes/block-color";
import {
  blockEditableRichText,
  blockPlainText,
  blockWithRichText,
  blockUpdateFromBlock,
} from "$lib/notes/block-factory";
import { createBlockWriteFromRichText } from "$lib/notes/block-rich-text-write";
import {
  notesEnterSiblingBlockType,
  notesEnterSplitsRichTextBlock,
} from "$lib/notes/block-enter";
import { planNotesInsertedBlockFocus } from "$lib/notes/editor-focus";
import { parentIdForBlock } from "$lib/notes/block-tree";
import { splitRichTextForBlock } from "$lib/notes/rich-text-split";
import type { NotesTextSelection } from "$lib/notes/editor-selection";
import type {
  NotesAppendBlockChildrenRequest,
  NotesBlock,
  NotesBlockUpdate,
  NotesBlockWrite,
  NotesParent,
} from "$lib/notes/types";
import type {
  NotesUndoSnapshot,
} from "$lib/notes/undo-history";

const START_OF_BLOCK_SELECTION: NotesTextSelection = { start: 0, end: 0 };

interface OptimisticPastePlan {
  currentUpdate: NotesBlockUpdate;
  appendedBlocks: NotesBlockWrite[];
  focusBlockId: string;
  focusOffset: number;
}

interface NotesBlockPasteActionsContext {
  readSelectedPageId: () => string | null;
  blockById: (blockId: string) => NotesBlock | undefined;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  localInsertBlockAfter: (block: NotesBlock, afterBlockId: string | null) => void;
  requestBlockFocus: (blockId: string | null, selection?: NotesTextSelection | null) => void;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  flushBlockSave: (blockId: string) => Promise<void>;
  loadPageTree: (pageId: string) => Promise<void>;
  appendAndApply: (request: NotesAppendBlockChildrenRequest) => Promise<NotesBlock[]>;
  pendingOptimisticWrite: (blockId: string) => Promise<void> | null;
  trackOptimisticBlockWrites: (blockIds: readonly string[], persistence: Promise<void>) => void;
  optimisticBlockFromWrite: (write: NotesBlockWrite, parent: NotesParent) => NotesBlock;
  undoSnapshotForBlocks: (
    blockIds: readonly string[],
    focusBlockId: string | null,
    focusSelection?: NotesTextSelection | null,
  ) => NotesUndoSnapshot | null;
  recordUndo: (
    kind: "create" | "paste",
    before: NotesUndoSnapshot | null,
    after: NotesUndoSnapshot | null,
    groupKey?: string | null,
  ) => void;
}

export interface NotesBlockPasteActions {
  splitTextBlockAtSelection: (
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
  ) => Promise<void>;
  pastePlainTextIntoBlock: (
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    plainText: string,
  ) => Promise<boolean>;
  pasteRichHtmlIntoBlock: (
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    html: string,
  ) => Promise<boolean>;
}

/** Create optimistic split, plain-text paste, and rich-HTML paste actions. */
export function createNotesBlockPasteActions(
  context: NotesBlockPasteActionsContext,
): NotesBlockPasteActions {
  async function persistSplitTextBlock(
    pageId: string,
    blockId: string,
    currentUpdate: NotesBlockUpdate,
    parent: NotesParent,
    nextWrite: NotesBlockWrite,
    prerequisite: Promise<void> | null,
  ): Promise<void> {
    try {
      await prerequisite;
      if (prerequisite) await context.saveBlockNow(blockId, cloneNotesJson(currentUpdate));
      else await context.flushBlockSave(blockId);
      await context.appendAndApply({
        parent: cloneNotesJson(parent),
        after: blockId,
        children: [cloneNotesJson(nextWrite)],
      });
      const nextBlock = context.blockById(nextWrite.id);
      if (nextBlock) {
        await context.saveBlockNow(
          nextWrite.id,
          cloneNotesJson(blockWithRichText(nextBlock, blockEditableRichText(nextBlock))),
        );
      }
    } catch (error) {
      console.warn("notes split block persistence failed", error);
      await context.loadPageTree(pageId);
    }
  }

  async function splitTextBlockAtSelection(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
  ): Promise<void> {
    const prerequisite = context.pendingOptimisticWrite(blockId);
    const block = context.blockById(blockId);
    const pageId = context.readSelectedPageId();
    if (!block || !notesEnterSplitsRichTextBlock(block.type) || !pageId) return;
    const beforeSelection = {
      start: Math.min(selectionStart, selectionEnd),
      end: Math.max(selectionStart, selectionEnd),
    };
    const before = context.undoSnapshotForBlocks([blockId], blockId, beforeSelection);
    const split = splitRichTextForBlock(blockEditableRichText(block), selectionStart, selectionEnd);
    const newBlockId = crypto.randomUUID();
    const currentUpdate = cloneNotesJson(blockWithRichText(block, split.before));
    const nextWrite = cloneNotesJson(createBlockWriteFromRichText(
      newBlockId,
      notesEnterSiblingBlockType(block.type),
      split.after,
      blockColor(block),
    ));
    const parent = cloneNotesJson(block.parent);
    const nextBlock = context.optimisticBlockFromWrite(nextWrite, parent);
    const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId) ?? newBlockId;

    context.localApplyBlockUpdate(blockId, currentUpdate);
    if (!prerequisite) context.scheduleBlockSave(blockId, currentUpdate);
    context.localInsertBlockAfter(nextBlock, blockId);
    context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
    context.recordUndo(
      "create",
      before,
      context.undoSnapshotForBlocks(
        [blockId, newBlockId],
        focusBlockId,
        START_OF_BLOCK_SELECTION,
      ),
      `create:enter:${parentIdForBlock(block)}`,
    );
    const persistence = persistSplitTextBlock(
      pageId,
      blockId,
      currentUpdate,
      parent,
      nextWrite,
      prerequisite,
    );
    context.trackOptimisticBlockWrites(
      prerequisite ? [blockId, newBlockId] : [newBlockId],
      persistence,
    );
    void persistence;
  }

  async function persistOptimisticPaste(
    pageId: string,
    currentBlockId: string,
    parent: NotesParent,
    appendedWrites: readonly NotesBlockWrite[],
  ): Promise<void> {
    try {
      await context.flushBlockSave(currentBlockId);
      await context.appendAndApply({
        parent: cloneNotesJson(parent),
        after: currentBlockId,
        children: cloneNotesJson([...appendedWrites]),
      });
      for (const write of appendedWrites) {
        const latestBlock = context.blockById(write.id);
        if (latestBlock) await context.saveBlockNow(write.id, blockUpdateFromBlock(latestBlock));
      }
    } catch (error) {
      console.warn("notes paste persistence failed", error);
      await context.loadPageTree(pageId);
    }
  }

  function applyOptimisticPaste(
    pageId: string,
    currentBlock: NotesBlock,
    plan: OptimisticPastePlan,
    beforeFocusSelection: NotesTextSelection,
  ): void {
    const affectedIds = [currentBlock.id, ...plan.appendedBlocks.map((write) => write.id)];
    const before = context.undoSnapshotForBlocks(
      affectedIds,
      currentBlock.id,
      beforeFocusSelection,
    );
    const currentUpdate = cloneNotesJson(plan.currentUpdate);
    const writes = plan.appendedBlocks.map((write) => cloneNotesJson(write));
    const parent = cloneNotesJson(currentBlock.parent);
    const focusSelection = { start: plan.focusOffset, end: plan.focusOffset };
    context.localApplyBlockUpdate(currentBlock.id, currentUpdate);
    context.scheduleBlockSave(currentBlock.id, currentUpdate);
    let after = currentBlock.id;
    for (const write of writes) {
      context.localInsertBlockAfter(context.optimisticBlockFromWrite(write, parent), after);
      after = write.id;
    }
    context.requestBlockFocus(plan.focusBlockId, focusSelection);
    context.recordUndo(
      "paste",
      before,
      context.undoSnapshotForBlocks(affectedIds, plan.focusBlockId, focusSelection),
    );
    if (writes.length === 0) return;
    const persistence = persistOptimisticPaste(pageId, currentBlock.id, parent, writes);
    context.trackOptimisticBlockWrites(writes.map((write) => write.id), persistence);
    void persistence;
  }

  async function pastePlainTextIntoBlock(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    plainText: string,
  ): Promise<boolean> {
    const block = context.blockById(blockId);
    const pageId = context.readSelectedPageId();
    if (!block || !pageId) return false;
    const plan = planNotesPlainTextPaste({
      currentBlockId: blockId,
      currentBlockType: block.type,
      currentText: blockPlainText(block),
      selectionStart,
      selectionEnd,
      plainText,
      createId: () => crypto.randomUUID(),
    });
    if (!plan) return false;
    applyOptimisticPaste(pageId, block, plan, {
      start: Math.min(selectionStart, selectionEnd),
      end: Math.max(selectionStart, selectionEnd),
    });
    return true;
  }

  async function pasteRichHtmlIntoBlock(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    html: string,
  ): Promise<boolean> {
    const block = context.blockById(blockId);
    const pageId = context.readSelectedPageId();
    if (!block || !pageId) return false;
    const plan = planNotesRichHtmlPaste({
      currentBlock: block,
      selectionStart,
      selectionEnd,
      html,
      createId: () => crypto.randomUUID(),
    });
    if (!plan) return false;
    applyOptimisticPaste(pageId, block, plan, {
      start: Math.min(selectionStart, selectionEnd),
      end: Math.max(selectionStart, selectionEnd),
    });
    return true;
  }

  return { splitTextBlockAtSelection, pastePlainTextIntoBlock, pasteRichHtmlIntoBlock };
}
