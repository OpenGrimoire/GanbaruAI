import {
  createNotesTableRowWrite,
  notesTableCanAddColumn,
  notesTableCanAddRow,
  notesTableCanRemoveColumn,
  notesTableCanRemoveRow,
  notesTableRowWithCellRichText,
  notesTableRowWithCellText,
  notesTableRowWithInsertedColumn,
  notesTableRowWithRemovedColumn,
  notesTableVisibleWidth,
  notesTableWithWidth,
} from "$lib/notes/table";
import type {
  NotesAppendBlockChildrenRequest,
  NotesBlock,
  NotesBlockUpdate,
  NotesRichText,
  NotesTableRowBlock,
} from "$lib/notes/types";
import type { NotesUndoSnapshot } from "$lib/notes/undo-history";

interface NotesTableBlockActionsContext {
  readSelectedPageId: () => string | null;
  blockById: (blockId: string) => NotesBlock | undefined;
  tableRowsForBlock: (blockId: string) => NotesTableRowBlock[];
  requestBlockFocus: (blockId: string | null) => void;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
  appendAndApply: (request: NotesAppendBlockChildrenRequest) => Promise<NotesBlock[]>;
  trashAndApply: (rootBlockIds: readonly string[]) => Promise<void>;
  replaceBlockWithUpdate: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  undoSnapshot: (focusBlockId: string | null) => NotesUndoSnapshot | null;
  recordUndoAfter: (
    kind: "create" | "delete" | "typing" | "update",
    before: NotesUndoSnapshot | null,
    focusBlockId: string | null,
    groupKey?: string | null,
  ) => void;
}

export interface NotesTableBlockActions {
  updateTableCell: (rowBlockId: string, columnIndex: number, text: string) => Promise<void>;
  updateTableCellRichText: (
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ) => Promise<void>;
  addTableRow: (tableBlockId: string, afterRowIndex: number) => Promise<void>;
  removeTableRow: (tableBlockId: string, rowBlockId: string) => Promise<void>;
  addTableColumn: (tableBlockId: string, afterColumnIndex: number) => Promise<void>;
  removeTableColumn: (tableBlockId: string, columnIndex: number) => Promise<void>;
}

function tableInsertIndex(index: number, width: number): number {
  if (!Number.isFinite(index)) return width;
  return Math.max(0, Math.min(width, Math.trunc(index)));
}

function tableRemoveIndex(index: number, width: number): number {
  if (!Number.isFinite(index)) return Math.max(0, width - 1);
  return Math.max(0, Math.min(Math.max(0, width - 1), Math.trunc(index)));
}

/** Create database-table row, cell, and column mutations. */
export function createNotesTableBlockActions(
  context: NotesTableBlockActionsContext,
): NotesTableBlockActions {
  async function updateTableCell(
    rowBlockId: string,
    columnIndex: number,
    text: string,
  ): Promise<void> {
    const block = context.blockById(rowBlockId);
    if (!block || block.type !== "table_row") return;
    const before = context.undoSnapshot(rowBlockId);
    const update = notesTableRowWithCellText(block, columnIndex, text);
    context.localApplyBlockUpdate(rowBlockId, update);
    context.scheduleBlockSave(rowBlockId, update);
    context.recordUndoAfter("typing", before, rowBlockId, `typing:${rowBlockId}:${columnIndex}`);
  }

  async function updateTableCellRichText(
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ): Promise<void> {
    const block = context.blockById(rowBlockId);
    if (!block || block.type !== "table_row") return;
    const before = context.undoSnapshot(rowBlockId);
    const update = notesTableRowWithCellRichText(block, columnIndex, richText);
    context.localApplyBlockUpdate(rowBlockId, update);
    context.scheduleBlockSave(rowBlockId, update);
    context.recordUndoAfter("typing", before, rowBlockId, `typing:${rowBlockId}:${columnIndex}`);
  }

  async function addTableRow(tableBlockId: string, afterRowIndex: number): Promise<void> {
    const table = context.blockById(tableBlockId);
    if (!context.readSelectedPageId() || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    if (!notesTableCanAddRow(rows.length)) return;
    await context.flushPendingBlockSaves();
    const before = context.undoSnapshot(tableBlockId);
    const rowIndex = tableRemoveIndex(afterRowIndex, Math.max(1, rows.length));
    const after = rows[rowIndex]?.id ?? rows.at(-1)?.id ?? null;
    await context.appendAndApply({
      parent: { type: "block_id", block_id: tableBlockId },
      after,
      children: [createNotesTableRowWrite(crypto.randomUUID(), notesTableVisibleWidth(table, rows))],
    });
    context.requestBlockFocus(tableBlockId);
    context.recordUndoAfter("create", before, tableBlockId);
  }

  async function removeTableRow(tableBlockId: string, rowBlockId: string): Promise<void> {
    const table = context.blockById(tableBlockId);
    if (!context.readSelectedPageId() || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    if (!notesTableCanRemoveRow(rows.length) || !rows.some((row) => row.id === rowBlockId)) return;
    await context.flushPendingBlockSaves();
    const before = context.undoSnapshot(tableBlockId);
    await context.trashAndApply([rowBlockId]);
    context.requestBlockFocus(tableBlockId);
    context.recordUndoAfter("delete", before, tableBlockId);
  }

  async function addTableColumn(tableBlockId: string, afterColumnIndex: number): Promise<void> {
    const table = context.blockById(tableBlockId);
    if (!context.readSelectedPageId() || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    const width = notesTableVisibleWidth(table, rows);
    if (!notesTableCanAddColumn(width)) return;
    const insertIndex = tableInsertIndex(afterColumnIndex + 1, width);
    await context.flushPendingBlockSaves();
    const before = context.undoSnapshot(tableBlockId);
    await context.replaceBlockWithUpdate(tableBlockId, notesTableWithWidth(table, width + 1));
    for (const row of rows) {
      const update = notesTableRowWithInsertedColumn(row, insertIndex, width);
      context.localApplyBlockUpdate(row.id, update);
      await context.saveBlockNow(row.id, update);
    }
    if (rows.length === 0) {
      await context.appendAndApply({
        parent: { type: "block_id", block_id: tableBlockId },
        after: null,
        children: [createNotesTableRowWrite(crypto.randomUUID(), width + 1)],
      });
    }
    context.requestBlockFocus(tableBlockId);
    context.recordUndoAfter("update", before, tableBlockId);
  }

  async function removeTableColumn(tableBlockId: string, columnIndex: number): Promise<void> {
    const table = context.blockById(tableBlockId);
    if (!context.readSelectedPageId() || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    const width = notesTableVisibleWidth(table, rows);
    if (!notesTableCanRemoveColumn(width)) return;
    const removeIndex = tableRemoveIndex(columnIndex, width);
    await context.flushPendingBlockSaves();
    const before = context.undoSnapshot(tableBlockId);
    await context.replaceBlockWithUpdate(tableBlockId, notesTableWithWidth(table, width - 1));
    for (const row of rows) {
      const update = notesTableRowWithRemovedColumn(row, removeIndex, width);
      context.localApplyBlockUpdate(row.id, update);
      await context.saveBlockNow(row.id, update);
    }
    context.requestBlockFocus(tableBlockId);
    context.recordUndoAfter("update", before, tableBlockId);
  }

  return {
    updateTableCell,
    updateTableCellRichText,
    addTableRow,
    removeTableRow,
    addTableColumn,
    removeTableColumn,
  };
}
