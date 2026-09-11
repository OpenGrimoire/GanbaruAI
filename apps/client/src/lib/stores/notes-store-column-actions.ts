import {
  appendNotesBlockChildren,
  moveNotesBlock,
  moveNotesBlocks,
  trashNotesBlock,
} from "$lib/api/notes";
import { collectLoadedBlockSubtreeIds } from "$lib/notes/block-duplicate";
import { createBlockWrite } from "$lib/notes/block-factory";
import {
  createNotesColumnWrite,
  notesColumnCanAdd,
  notesColumnCanMove,
  notesColumnResizeWidths,
  notesColumnWithWidthRatio,
  planNotesColumnInsertion,
  planNotesColumnRemoval,
  type NotesColumnMoveDirection,
} from "$lib/notes/column";
import type {
  NotesBlock,
  NotesAppendBlockChildrenRequest,
  NotesBlockUpdate,
  NotesColumnBlock,
  NotesColumnBlockItems,
} from "$lib/notes/types";
import type {
  NotesUndoKind,
  NotesUndoRecordOptions,
  NotesUndoSnapshot,
} from "$lib/notes/undo-history";
import type { NotesTreeState } from "$lib/notes/block-tree";
import {
  notesPostAppendResult,
  notesPostMoveManyResult,
  notesPostMoveResult,
  notesPostTrashResult,
  type NotesPostMutationResult,
} from "$lib/notes/post-mutation";

export interface NotesColumnActionsContext {
  readSelectedPageId: () => string | null;
  readChildIdsByParentId: () => Record<string, string[]>;
  treeState: () => NotesTreeState;
  blockById: (blockId: string) => NotesBlock | undefined;
  columnItemsForBlock: (blockId: string) => NotesColumnBlockItems[];
  requestBlockFocus: (blockId: string | null) => void;
  applyPostMutation: (result: NotesPostMutationResult) => void;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
  createUndoSnapshot: (
    focusBlockId: string | null,
    extraBlocks?: readonly NotesBlock[],
  ) => NotesUndoSnapshot | null;
  recordUndo: (options: Omit<NotesUndoRecordOptions, "id">) => void;
}

export interface NotesColumnActions {
  addColumn: (columnListBlockId: string, afterColumnIndex: number) => Promise<void>;
  removeColumn: (columnListBlockId: string, columnBlockId: string) => Promise<void>;
  moveColumn: (
    columnListBlockId: string,
    columnBlockId: string,
    direction: NotesColumnMoveDirection,
  ) => Promise<void>;
  resizeColumn: (
    columnListBlockId: string,
    columnBlockId: string,
    widthRatio: number,
  ) => Promise<void>;
  moveBlockToColumn: (blockId: string, columnBlockId: string) => Promise<void>;
}

/**
 * Create Notes column layout mutation and UI action methods.
 */
export function createNotesColumnActions(
  context: NotesColumnActionsContext,
): NotesColumnActions {
  function undoSnapshot(
    focusBlockId: string | null,
    extraBlocks: readonly NotesBlock[] = [],
  ): NotesUndoSnapshot | null {
    return context.createUndoSnapshot(focusBlockId, extraBlocks);
  }

  function recordUndo(
    kind: NotesUndoKind,
    before: NotesUndoSnapshot | null,
    after: NotesUndoSnapshot | null,
    groupKey: string | null = null,
  ): void {
    context.recordUndo({ kind, before, after, groupKey });
  }

  function recordUndoAfter(
    kind: NotesUndoKind,
    before: NotesUndoSnapshot | null,
    focusBlockId: string | null,
    groupKey: string | null = null,
    extraBlocks: readonly NotesBlock[] = [],
  ): void {
    recordUndo(kind, before, undoSnapshot(focusBlockId, extraBlocks), groupKey);
  }

  function columnBlocksForList(columnListBlockId: string): NotesColumnBlock[] {
    return context.columnItemsForBlock(columnListBlockId).map((item) => item.column);
  }

  function activeChildBlockIds(parentBlockId: string): string[] {
    return (context.readChildIdsByParentId()[parentBlockId] ?? []).filter((blockId) => {
      const block = context.blockById(blockId);
      return block !== undefined && !block.in_trash;
    });
  }

  async function saveColumnWidths(
    columns: readonly NotesColumnBlock[],
    widths: readonly number[],
  ): Promise<void> {
    for (const [index, column] of columns.entries()) {
      const width = widths[index];
      if (width === undefined) continue;
      const update = notesColumnWithWidthRatio(width);
      context.localApplyBlockUpdate(column.id, update);
      await context.saveBlockNow(column.id, update);
    }
  }

  async function addColumn(columnListBlockId: string, afterColumnIndex: number): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const columnList = context.blockById(columnListBlockId);
    if (!selectedPageId || !columnList || columnList.type !== "column_list") return;
    const columns = columnBlocksForList(columnListBlockId);
    if (!notesColumnCanAdd(columns.length)) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(columnListBlockId);

    if (columns.length === 0) {
      const leftColumnId = crypto.randomUUID();
      const rightColumnId = crypto.randomUUID();
      const leftBlockId = crypto.randomUUID();
      const rightBlockId = crypto.randomUUID();
      const columnsRequest = {
        parent: { type: "block_id", block_id: columnListBlockId },
        after: null,
        children: [
          createNotesColumnWrite(leftColumnId, 0.5),
          createNotesColumnWrite(rightColumnId, 0.5),
        ],
      } satisfies NotesAppendBlockChildrenRequest;
      context.applyPostMutation(notesPostAppendResult(
        columnsRequest,
        await appendNotesBlockChildren(columnsRequest),
      ));
      const leftBlockRequest = {
        parent: { type: "block_id", block_id: leftColumnId },
        after: null,
        children: [createBlockWrite(leftBlockId, "paragraph")],
      } satisfies NotesAppendBlockChildrenRequest;
      context.applyPostMutation(notesPostAppendResult(
        leftBlockRequest,
        await appendNotesBlockChildren(leftBlockRequest),
      ));
      const rightBlockRequest = {
        parent: { type: "block_id", block_id: rightColumnId },
        after: null,
        children: [createBlockWrite(rightBlockId, "paragraph")],
      } satisfies NotesAppendBlockChildrenRequest;
      context.applyPostMutation(notesPostAppendResult(
        rightBlockRequest,
        await appendNotesBlockChildren(rightBlockRequest),
      ));
      context.requestBlockFocus(leftBlockId);
      recordUndoAfter("create", before, leftBlockId);
      return;
    }

    const plan = planNotesColumnInsertion(columns, afterColumnIndex);
    if (!plan) return;
    const newColumnId = crypto.randomUUID();
    const newBlockId = crypto.randomUUID();
    const existingWidths = columns.map((_, index) => {
      const plannedIndex = index < plan.insertIndex ? index : index + 1;
      return plan.widths[plannedIndex] ?? 1 / (columns.length + 1);
    });
    await saveColumnWidths(columns, existingWidths);
    const columnRequest = {
      parent: { type: "block_id", block_id: columnListBlockId },
      after: columns[Math.max(0, plan.insertIndex - 1)]?.id ?? columns.at(-1)?.id ?? null,
      children: [
        createNotesColumnWrite(newColumnId, plan.widths[plan.insertIndex] ?? 1),
      ],
    } satisfies NotesAppendBlockChildrenRequest;
    context.applyPostMutation(notesPostAppendResult(
      columnRequest,
      await appendNotesBlockChildren(columnRequest),
    ));
    const blockRequest = {
      parent: { type: "block_id", block_id: newColumnId },
      after: null,
      children: [createBlockWrite(newBlockId, "paragraph")],
    } satisfies NotesAppendBlockChildrenRequest;
    context.applyPostMutation(notesPostAppendResult(
      blockRequest,
      await appendNotesBlockChildren(blockRequest),
    ));
    context.requestBlockFocus(newBlockId);
    recordUndoAfter("create", before, newBlockId);
  }

  async function removeColumn(columnListBlockId: string, columnBlockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const columnList = context.blockById(columnListBlockId);
    if (!selectedPageId || !columnList || columnList.type !== "column_list") return;
    const columns = columnBlocksForList(columnListBlockId);
    const plan = planNotesColumnRemoval(columns, columnBlockId);
    if (!plan) return;
    const removedColumn = columns[plan.removeIndex];
    const targetColumn = columns[plan.targetIndex];
    if (!removedColumn || !targetColumn) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(columnListBlockId);
    const movedChildIds = activeChildBlockIds(removedColumn.id);
    const targetChildIds = activeChildBlockIds(targetColumn.id);
    if (movedChildIds.length > 0) {
      const moveRequest = {
        block_ids: movedChildIds,
        parent: { type: "block_id", block_id: targetColumn.id },
        after: targetChildIds.at(-1) ?? null,
      } as const;
      context.applyPostMutation(notesPostMoveManyResult(
        moveRequest,
        await moveNotesBlocks(moveRequest),
      ));
    }
    await trashNotesBlock(removedColumn.id, true);
    context.applyPostMutation(notesPostTrashResult(context.treeState(), [removedColumn.id]));
    const remainingColumns = columns.filter((column) => column.id !== removedColumn.id);
    await saveColumnWidths(remainingColumns, plan.widths);
    const focusBlockId = movedChildIds[0] ?? targetChildIds.at(-1) ?? columnListBlockId;
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("delete", before, focusBlockId);
  }

  async function moveColumn(
    columnListBlockId: string,
    columnBlockId: string,
    direction: NotesColumnMoveDirection,
  ): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const columnList = context.blockById(columnListBlockId);
    if (!selectedPageId || !columnList || columnList.type !== "column_list") return;
    const columns = columnBlocksForList(columnListBlockId);
    if (!notesColumnCanMove(columns, columnBlockId, direction)) return;
    const columnIndex = columns.findIndex((column) => column.id === columnBlockId);
    const targetColumn = columns[direction === "left" ? columnIndex - 1 : columnIndex + 1];
    if (!targetColumn) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(columnListBlockId);
    const moveRequest = {
      parent: { type: "block_id", block_id: columnListBlockId },
      after: direction === "right" ? targetColumn.id : null,
      before: direction === "left" ? targetColumn.id : null,
    } as const;
    context.applyPostMutation(notesPostMoveResult(
      await moveNotesBlock(columnBlockId, moveRequest),
      moveRequest,
    ));
    context.requestBlockFocus(columnListBlockId);
    recordUndoAfter("move", before, columnListBlockId);
  }

  async function resizeColumn(
    columnListBlockId: string,
    columnBlockId: string,
    widthRatio: number,
  ): Promise<void> {
    const columnList = context.blockById(columnListBlockId);
    if (!columnList || columnList.type !== "column_list") return;
    const columns = columnBlocksForList(columnListBlockId);
    const widths = notesColumnResizeWidths(columns, columnBlockId, widthRatio);
    if (!widths) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(columnListBlockId);
    await saveColumnWidths(columns, widths);
    context.requestBlockFocus(columnListBlockId);
    recordUndoAfter("update", before, columnListBlockId, `update:${columnListBlockId}:columns`);
  }

  async function moveBlockToColumn(blockId: string, columnBlockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const block = context.blockById(blockId);
    const column = context.blockById(columnBlockId);
    if (!selectedPageId || !block || !column || column.type !== "column") return;
    if (block.type === "column" || block.type === "table_row") return;
    if (collectLoadedBlockSubtreeIds(context.treeState(), blockId).includes(columnBlockId)) return;
    const originalChildIds = activeChildBlockIds(columnBlockId);
    if (
      block.parent.type === "block_id"
      && block.parent.block_id === columnBlockId
      && originalChildIds.at(-1) === blockId
    ) {
      return;
    }
    const childIds = originalChildIds.filter((childId) => childId !== blockId);
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    const moveRequest = {
      parent: { type: "block_id", block_id: columnBlockId },
      after: childIds.at(-1) ?? null,
      before: null,
    } as const;
    context.applyPostMutation(notesPostMoveResult(
      await moveNotesBlock(blockId, moveRequest),
      moveRequest,
    ));
    context.requestBlockFocus(blockId);
    recordUndoAfter("move", before, blockId);
  }

  return {
    addColumn,
    removeColumn,
    moveColumn,
    resizeColumn,
    moveBlockToColumn,
  };
}
