import {
  appendNotesBlockChildren,
  duplicateNotesBlock,
  duplicateNotesBlocks,
  moveNotesBlock,
  moveNotesBlocks,
  trashNotesBlock,
  trashNotesBlocks,
} from "$lib/api/notes";
import {
  collectLoadedBlockSubtreeIds,
  createDuplicateBlockRequest,
} from "$lib/notes/block-duplicate";
import { planNotesPlainTextPaste } from "$lib/notes/block-clipboard";
import { planNotesRichHtmlPaste } from "$lib/notes/rich-text-paste";
import {
  notesSelectionRootBlockIds,
  notesSelectionSubtreeIds,
  planNotesSelectionMoveWithinSiblings,
  type NotesSelectionMoveDirection,
} from "$lib/notes/block-selection-operations";
import { blockColor, blockWithColor, canBlockHaveColor } from "$lib/notes/block-color";
import {
  createBlockWriteFromInsertCommand,
  normalizeNotesBlockInsertCommand,
  type NotesBlockInsertRequest,
} from "$lib/notes/block-insertion";
import {
  blockEditableRichText,
  blockConvertedToType,
  blockPlainText,
  blockWithBookmark,
  blockWithCodeLanguage,
  blockWithDateMention,
  blockWithEmbedUrl,
  blockWithEquationExpression,
  blockWithHeadingToggleable,
  blockWithHeadingToggleOpen,
  blockWithInlineEquation,
  blockWithLinkPreviewUrl,
  blockWithMedia,
  blockWithPageMention,
  blockWithRichText,
  blockWithText,
  blockWithTextAnnotations,
  blockWithTextLink,
  blockWithTodoChecked,
  blockWithToggleOpen,
  createBlockUpdate,
  createBlockWrite,
  createColumnPayload,
  createEmptyTableRowPayload,
  DEFAULT_TABLE_ROW_COUNT,
  DEFAULT_TABLE_WIDTH,
  type NotesHeadingBlockType,
} from "$lib/notes/block-factory";
import { createBlockWriteFromRichText } from "$lib/notes/block-rich-text-write";
import {
  notesEnterSiblingBlockType,
  notesEnterSplitsRichTextBlock,
} from "$lib/notes/block-enter";
import {
  planNotesBlockDrop,
  type NotesBlockDropIntent,
} from "$lib/notes/block-drag";
import {
  planNotesDeletedBlockFocus,
  planNotesInsertedBlockFocus,
} from "$lib/notes/editor-focus";
import {
  planDeleteBlock,
  planMergeWithPrevious,
  planMoveBlockWithinSiblings,
  planNestBlock,
  planOutdentBlock,
  planReparentChildrenAfterMerge,
  planReparentChildrenBeforeDelete,
  type NotesChildReparentPlan,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import type { NotesRichTextAnnotationPatch } from "$lib/notes/rich-text";
import { splitRichTextForBlock } from "$lib/notes/rich-text-split";
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
  NotesUndoKind,
  NotesUndoRecordOptions,
  NotesUndoSnapshot,
} from "$lib/notes/undo-history";
import type {
  NotesBlock,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesBlockUpdate,
  NotesBlockWrite,
  NotesButtonInsertPosition,
  NotesColor,
  NotesColumnBlockItems,
  NotesDateMentionValue,
  NotesParent,
  NotesRichText,
  NotesTabBlockItems,
  NotesTableRowBlock,
} from "$lib/notes/types";
import {
  createNotesColumnActions,
  type NotesColumnActions,
} from "$lib/stores/notes-store-column-actions";

export interface NotesBlockActionsContext {
  readSelectedPageId: () => string | null;
  readBlocksById: () => Record<string, NotesBlock>;
  readChildIdsByParentId: () => Record<string, string[]>;
  treeState: () => NotesTreeState;
  blockById: (blockId: string) => NotesBlock | undefined;
  flatBlockItemsForBlockContext: (blockId: string) => NotesBlockTreeItem[];
  tableRowsForBlock: (blockId: string) => NotesTableRowBlock[];
  columnItemsForBlock: (blockId: string) => NotesColumnBlockItems[];
  tabItemsForBlock: (blockId: string) => NotesTabBlockItems[];
  setSidebarPageCollapsed: (pageId: string, collapsed: boolean) => void;
  requestBlockFocus: (blockId: string | null) => void;
  createChildPageFromBlock: (blockId: string) => Promise<void>;
  createChildPageAfterBlock: (blockId: string) => Promise<void>;
  loadPageTree: (pageId: string) => Promise<void>;
  reloadPages: () => Promise<void>;
  reloadBacklinks: () => Promise<void>;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  flushBlockSave: (blockId: string) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
  createUndoSnapshot: (
    focusBlockId: string | null,
    extraBlocks?: readonly NotesBlock[],
  ) => NotesUndoSnapshot | null;
  recordUndo: (options: Omit<NotesUndoRecordOptions, "id">) => void;
}

export interface NotesBlockActions extends NotesColumnActions {
  updateBlockText: (blockId: string, text: string) => Promise<void>;
  updateBlockRichText: (
    blockId: string,
    richText: readonly NotesRichText[],
  ) => Promise<void>;
  insertPageMention: (
    blockId: string,
    start: number,
    end: number,
    pageId: string,
    title: string,
    href: string | null,
  ) => Promise<void>;
  insertDateMention: (
    blockId: string,
    start: number,
    end: number,
    date: NotesDateMentionValue,
    title: string,
  ) => Promise<void>;
  insertInlineEquation: (
    blockId: string,
    start: number,
    end: number,
    expression: string,
  ) => Promise<void>;
  updateBlockTextLink: (
    blockId: string,
    start: number,
    end: number,
    url: string | null,
  ) => Promise<void>;
  updateBlockTextAnnotations: (
    blockId: string,
    start: number,
    end: number,
    patch: NotesRichTextAnnotationPatch,
  ) => Promise<void>;
  updateBookmark: (blockId: string, url: string, caption: string) => Promise<void>;
  updateEmbedUrl: (blockId: string, url: string) => Promise<void>;
  updateLinkPreviewUrl: (blockId: string, url: string) => Promise<void>;
  updateEquationExpression: (blockId: string, expression: string) => Promise<void>;
  updateMedia: (
    blockId: string,
    url: string,
    caption: string,
    name?: string,
  ) => Promise<void>;
  updateTableCell: (
    rowBlockId: string,
    columnIndex: number,
    text: string,
  ) => Promise<void>;
  updateTableCellRichText: (
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ) => Promise<void>;
  addTableRow: (tableBlockId: string, afterRowIndex: number) => Promise<void>;
  removeTableRow: (tableBlockId: string, rowBlockId: string) => Promise<void>;
  addTableColumn: (tableBlockId: string, afterColumnIndex: number) => Promise<void>;
  removeTableColumn: (tableBlockId: string, columnIndex: number) => Promise<void>;
  convertBlock: (blockId: string, type: NotesBlockType, clearText?: boolean) => Promise<void>;
  toggleTodo: (blockId: string, checked: boolean) => Promise<void>;
  updateCodeLanguage: (blockId: string, language: string) => Promise<void>;
  updateBlockColor: (blockId: string, color: NotesColor) => Promise<void>;
  updateToggleOpen: (blockId: string, open: boolean) => Promise<void>;
  convertBlockToToggleHeading: (
    blockId: string,
    headingType: NotesHeadingBlockType,
    clearText?: boolean,
  ) => Promise<void>;
  createSiblingAfter: (blockId: string, request?: NotesBlockInsertRequest) => Promise<void>;
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
  pasteBlockSelection: (
    sourceRootBlockIds: readonly string[],
    sourceSubtreeBlockIds: readonly string[],
    targetBlockId: string,
    includeTrashedSources?: boolean,
  ) => Promise<string | null>;
  deleteBlock: (blockId: string) => Promise<void>;
  deleteBlockSelection: (blockIds: readonly string[]) => Promise<void>;
  mergeBlockWithPrevious: (blockId: string) => Promise<void>;
  nestBlock: (blockId: string) => Promise<void>;
  outdentBlock: (blockId: string) => Promise<void>;
  moveBlockUp: (blockId: string) => Promise<void>;
  moveBlockDown: (blockId: string) => Promise<void>;
  moveBlockSelection: (
    blockIds: readonly string[],
    direction: NotesSelectionMoveDirection,
  ) => Promise<void>;
  dropBlockWithinSiblings: (
    sourceBlockId: string,
    targetBlockId: string,
    position: "before" | "after",
  ) => Promise<void>;
  dropBlockOnBlock: (
    sourceBlockId: string,
    targetBlockId: string,
    intent: NotesBlockDropIntent,
  ) => Promise<void>;
  moveBlockToPage: (blockId: string, pageId: string) => Promise<void>;
  duplicateBlock: (blockId: string) => Promise<void>;
  duplicateBlockSelection: (blockIds: readonly string[]) => Promise<string | null>;
  useTemplateBlock: (blockId: string) => Promise<void>;
  useButtonBlock: (blockId: string) => Promise<void>;
}

/**
 * Create Notes block mutation and UI action methods.
 */
export function createNotesBlockActions(context: NotesBlockActionsContext): NotesBlockActions {
  const blocksById = context.readBlocksById;
  const childIdsByParentId = context.readChildIdsByParentId;

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

  const columnActions = createNotesColumnActions(context);

  async function replaceBlockWithUpdate(blockId: string, update: NotesBlockUpdate): Promise<void> {
    await context.flushBlockSave(blockId);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
  }

  async function updateBlockText(blockId: string, text: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    const update = blockWithText(block, text);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("typing", before, blockId, `typing:${blockId}`);
  }

  async function updateBlockRichText(
    blockId: string,
    richText: readonly NotesRichText[],
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    const update = blockWithRichText(block, richText);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("typing", before, blockId, `typing:${blockId}`);
  }

  async function insertPageMention(
    blockId: string,
    start: number,
    end: number,
    pageId: string,
    title: string,
    href: string | null,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const update = blockWithPageMention(block, start, end, pageId, title, href);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    await context.reloadBacklinks();
    recordUndoAfter("mention", before, blockId);
  }

  async function insertDateMention(
    blockId: string,
    start: number,
    end: number,
    date: NotesDateMentionValue,
    title: string,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const update = blockWithDateMention(block, start, end, date, title);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    recordUndoAfter("mention", before, blockId);
  }

  async function insertInlineEquation(
    blockId: string,
    start: number,
    end: number,
    expression: string,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const update = blockWithInlineEquation(block, start, end, expression);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    recordUndoAfter("equation", before, blockId);
  }

  async function updateBlockTextLink(
    blockId: string,
    start: number,
    end: number,
    url: string | null,
  ): Promise<void> {
    await context.flushBlockSave(blockId);
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    const update = blockWithTextLink(block, start, end, url);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    await context.reloadBacklinks();
    recordUndoAfter("link", before, blockId);
  }

  async function updateBlockTextAnnotations(
    blockId: string,
    start: number,
    end: number,
    patch: NotesRichTextAnnotationPatch,
  ): Promise<void> {
    await context.flushBlockSave(blockId);
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    const update = blockWithTextAnnotations(block, start, end, patch);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    recordUndoAfter("formatting", before, blockId);
  }

  async function updateBookmark(blockId: string, url: string, caption: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "bookmark") return;
    const before = undoSnapshot(blockId);
    const update = blockWithBookmark(block, url, caption);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateEmbedUrl(blockId: string, url: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "embed") return;
    const before = undoSnapshot(blockId);
    const update = blockWithEmbedUrl(block, url);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateLinkPreviewUrl(blockId: string, url: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "link_preview") return;
    const before = undoSnapshot(blockId);
    const update = blockWithLinkPreviewUrl(block, url);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateEquationExpression(blockId: string, expression: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "equation") return;
    const before = undoSnapshot(blockId);
    const update = blockWithEquationExpression(block, expression);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateMedia(
    blockId: string,
    url: string,
    caption: string,
    name?: string,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (
      !block
      || !["image", "video", "audio", "file", "pdf"].includes(block.type)
    ) {
      return;
    }
    const before = undoSnapshot(blockId);
    const update = blockWithMedia(block, url, caption, name);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateTableCell(
    rowBlockId: string,
    columnIndex: number,
    text: string,
  ): Promise<void> {
    const block = context.blockById(rowBlockId);
    if (!block || block.type !== "table_row") return;
    const before = undoSnapshot(rowBlockId);
    const update = notesTableRowWithCellText(block, columnIndex, text);
    context.localApplyBlockUpdate(rowBlockId, update);
    context.scheduleBlockSave(rowBlockId, update);
    recordUndoAfter("typing", before, rowBlockId, `typing:${rowBlockId}:${columnIndex}`);
  }

  async function updateTableCellRichText(
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ): Promise<void> {
    const block = context.blockById(rowBlockId);
    if (!block || block.type !== "table_row") return;
    const before = undoSnapshot(rowBlockId);
    const update = notesTableRowWithCellRichText(block, columnIndex, richText);
    context.localApplyBlockUpdate(rowBlockId, update);
    context.scheduleBlockSave(rowBlockId, update);
    recordUndoAfter("typing", before, rowBlockId, `typing:${rowBlockId}:${columnIndex}`);
  }

  function tableInsertIndex(index: number, width: number): number {
    if (!Number.isFinite(index)) return width;
    return Math.max(0, Math.min(width, Math.trunc(index)));
  }

  function tableRemoveIndex(index: number, width: number): number {
    if (!Number.isFinite(index)) return Math.max(0, width - 1);
    return Math.max(0, Math.min(Math.max(0, width - 1), Math.trunc(index)));
  }

  async function addTableRow(tableBlockId: string, afterRowIndex: number): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const table = context.blockById(tableBlockId);
    if (!selectedPageId || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    if (!notesTableCanAddRow(rows.length)) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(tableBlockId);
    const rowIndex = tableRemoveIndex(afterRowIndex, Math.max(1, rows.length));
    const after = rows[rowIndex]?.id ?? rows.at(-1)?.id ?? null;
    await appendNotesBlockChildren({
      parent: { type: "block_id", block_id: tableBlockId },
      after,
      children: [
        createNotesTableRowWrite(
          crypto.randomUUID(),
          notesTableVisibleWidth(table, rows),
        ),
      ],
    });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(tableBlockId);
    recordUndoAfter("create", before, tableBlockId);
  }

  async function removeTableRow(tableBlockId: string, rowBlockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const table = context.blockById(tableBlockId);
    if (!selectedPageId || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    if (!notesTableCanRemoveRow(rows.length)) return;
    if (!rows.some((row) => row.id === rowBlockId)) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(tableBlockId);
    await trashNotesBlock(rowBlockId, true);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(tableBlockId);
    recordUndoAfter("delete", before, tableBlockId);
  }

  async function addTableColumn(tableBlockId: string, afterColumnIndex: number): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const table = context.blockById(tableBlockId);
    if (!selectedPageId || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    const width = notesTableVisibleWidth(table, rows);
    if (!notesTableCanAddColumn(width)) return;
    const insertIndex = tableInsertIndex(afterColumnIndex + 1, width);
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(tableBlockId);
    await replaceBlockWithUpdate(tableBlockId, notesTableWithWidth(table, width + 1));
    for (const row of rows) {
      const update = notesTableRowWithInsertedColumn(row, insertIndex, width);
      context.localApplyBlockUpdate(row.id, update);
      await context.saveBlockNow(row.id, update);
    }
    if (rows.length === 0) {
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: tableBlockId },
        after: null,
        children: [createNotesTableRowWrite(crypto.randomUUID(), width + 1)],
      });
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(tableBlockId);
    recordUndoAfter("update", before, tableBlockId);
  }

  async function removeTableColumn(tableBlockId: string, columnIndex: number): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const table = context.blockById(tableBlockId);
    if (!selectedPageId || !table || table.type !== "table") return;
    const rows = context.tableRowsForBlock(tableBlockId);
    const width = notesTableVisibleWidth(table, rows);
    if (!notesTableCanRemoveColumn(width)) return;
    const removeIndex = tableRemoveIndex(columnIndex, width);
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(tableBlockId);
    await replaceBlockWithUpdate(tableBlockId, notesTableWithWidth(table, width - 1));
    for (const row of rows) {
      const update = notesTableRowWithRemovedColumn(row, removeIndex, width);
      context.localApplyBlockUpdate(row.id, update);
      await context.saveBlockNow(row.id, update);
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(tableBlockId);
    recordUndoAfter("update", before, tableBlockId);
  }

  async function convertBlock(blockId: string, type: NotesBlockType, clearText = false): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (type === "child_page") {
      await context.createChildPageFromBlock(blockId);
      return;
    }
    const before = undoSnapshot(blockId);
    if (type === "table") {
      await createTableFromBlock(blockId);
      recordUndoAfter("convert", before, blockId);
      return;
    }
    if (type === "column_list") {
      await createColumnListFromBlock(blockId);
      recordUndoAfter("convert", before, blockId);
      return;
    }
    if (type === "tab") {
      await createTabFromBlock(blockId);
      recordUndoAfter("convert", before, blockId);
      return;
    }
    if (block.type === "child_page") return;
    if (block.type === "table" || block.type === "table_row") return;
    if (block.type === "column_list" || block.type === "column") return;
    if (block.type === "tab") return;
    const update = clearText ? createBlockUpdate(type, "") : blockConvertedToType(block, type);
    await replaceBlockWithUpdate(blockId, update);
    context.requestBlockFocus(blockId);
    recordUndoAfter("convert", before, blockId);
  }

  async function createTableFromBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const block = context.blockById(blockId);
    if (!block || block.type === "child_page" || block.type === "table_row") return;
    if ((childIdsByParentId()[blockId] ?? []).length > 0 && block.type !== "table") return;
    await replaceBlockWithUpdate(blockId, createBlockUpdate("table", ""));
    if (context.tableRowsForBlock(blockId).length === 0) {
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: Array.from({ length: DEFAULT_TABLE_ROW_COUNT }, () => ({
          id: crypto.randomUUID(),
          type: "table_row" as const,
          table_row: createEmptyTableRowPayload(DEFAULT_TABLE_WIDTH),
        })),
      });
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
  }

  async function createColumnListFromBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const block = context.blockById(blockId);
    if (
      !block
      || block.type === "child_page"
      || block.type === "table_row"
      || block.type === "column"
    ) {
      return;
    }
    if ((childIdsByParentId()[blockId] ?? []).length > 0 && block.type !== "column_list") return;
    await replaceBlockWithUpdate(blockId, createBlockUpdate("column_list", ""));
    if (context.columnItemsForBlock(blockId).length === 0) {
      const leftColumnId = crypto.randomUUID();
      const rightColumnId = crypto.randomUUID();
      const leftBlockId = crypto.randomUUID();
      const rightBlockId = crypto.randomUUID();
      const columns: NotesBlockWrite[] = [
        {
          id: leftColumnId,
          type: "column",
          column: createColumnPayload(0.5),
        },
        {
          id: rightColumnId,
          type: "column",
          column: createColumnPayload(0.5),
        },
      ];
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: columns,
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: leftColumnId },
        after: null,
        children: [createBlockWrite(leftBlockId, "paragraph")],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: rightColumnId },
        after: null,
        children: [createBlockWrite(rightBlockId, "paragraph")],
      });
      await context.loadPageTree(selectedPageId);
      context.requestBlockFocus(leftBlockId);
      return;
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
  }

  async function createTabFromBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const block = context.blockById(blockId);
    if (
      !block
      || block.type === "child_page"
      || block.type === "table_row"
      || block.type === "column"
    ) {
      return;
    }
    if ((childIdsByParentId()[blockId] ?? []).length > 0 && block.type !== "tab") return;
    const firstLabel = blockPlainText(block).trim() || "Tab 1";
    await replaceBlockWithUpdate(blockId, createBlockUpdate("tab", ""));
    if (context.tabItemsForBlock(blockId).length === 0) {
      const firstLabelId = crypto.randomUUID();
      const secondLabelId = crypto.randomUUID();
      const firstContentId = crypto.randomUUID();
      const secondContentId = crypto.randomUUID();
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: [
          createBlockWrite(firstLabelId, "paragraph", firstLabel),
          createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
        ],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: firstLabelId },
        after: null,
        children: [createBlockWrite(firstContentId, "paragraph")],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: secondLabelId },
        after: null,
        children: [createBlockWrite(secondContentId, "paragraph")],
      });
      await context.loadPageTree(selectedPageId);
      context.requestBlockFocus(firstContentId);
      return;
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
  }

  async function toggleTodo(blockId: string, checked: boolean): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    await replaceBlockWithUpdate(blockId, blockWithTodoChecked(block, checked));
    recordUndoAfter("update", before, blockId);
  }

  async function updateCodeLanguage(blockId: string, language: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    await replaceBlockWithUpdate(blockId, blockWithCodeLanguage(block, language));
    recordUndoAfter("update", before, blockId);
  }

  async function updateBlockColor(blockId: string, color: NotesColor): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    await replaceBlockWithUpdate(blockId, blockWithColor(block, color));
    recordUndoAfter("formatting", before, blockId);
  }

  async function updateToggleOpen(blockId: string, open: boolean): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const before = undoSnapshot(blockId);
    if (block.type === "toggle") {
      await replaceBlockWithUpdate(blockId, blockWithToggleOpen(block, open));
      recordUndoAfter("update", before, blockId);
      return;
    }
    if (
      (block.type === "heading_1" && block.heading_1.is_toggleable === true)
      || (block.type === "heading_2" && block.heading_2.is_toggleable === true)
      || (block.type === "heading_3" && block.heading_3.is_toggleable === true)
      || (block.type === "heading_4" && block.heading_4.is_toggleable === true)
    ) {
      await replaceBlockWithUpdate(blockId, blockWithHeadingToggleOpen(block, open));
      recordUndoAfter("update", before, blockId);
    }
  }

  async function convertBlockToToggleHeading(
    blockId: string,
    headingType: NotesHeadingBlockType,
    clearText = false,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (block.type === "child_page") return;
    if (block.type === "table" || block.type === "table_row") return;
    if (block.type === "column_list" || block.type === "column") return;
    const before = undoSnapshot(blockId);
    const update = clearText
      ? createBlockUpdate(
        headingType,
        "",
        canBlockHaveColor(block.type) ? blockColor(block) : "default",
        {
          isToggleable: true,
          open: true,
        },
      )
      : blockWithHeadingToggleable(block, headingType, true);
    await replaceBlockWithUpdate(blockId, update);
    context.requestBlockFocus(blockId);
    recordUndoAfter("convert", before, blockId);
  }

  async function createSiblingAfter(
    blockId: string,
    request?: NotesBlockInsertRequest,
  ): Promise<void> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId) return;
    const command = normalizeNotesBlockInsertCommand(request);
    if (command.kind === "block" && command.blockType === "child_page") {
      await context.createChildPageAfterBlock(blockId);
      return;
    }
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const newBlockId = crypto.randomUUID();
    await appendNotesBlockChildren({
      parent: block.parent,
      after: blockId,
      children: [createBlockWriteFromInsertCommand(newBlockId, command)],
    });
    if (command.kind === "block" && command.blockType === "table") {
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: newBlockId },
        after: null,
        children: Array.from({ length: DEFAULT_TABLE_ROW_COUNT }, () => ({
          id: crypto.randomUUID(),
          type: "table_row" as const,
          table_row: createEmptyTableRowPayload(DEFAULT_TABLE_WIDTH),
        })),
      });
      await context.loadPageTree(selectedPageId);
      const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId);
      context.requestBlockFocus(focusBlockId);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    if (command.kind === "block" && command.blockType === "column_list") {
      const leftColumnId = crypto.randomUUID();
      const rightColumnId = crypto.randomUUID();
      const leftBlockId = crypto.randomUUID();
      const rightBlockId = crypto.randomUUID();
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: newBlockId },
        after: null,
        children: [
          {
            id: leftColumnId,
            type: "column",
            column: createColumnPayload(0.5),
          },
          {
            id: rightColumnId,
            type: "column",
            column: createColumnPayload(0.5),
          },
        ],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: leftColumnId },
        after: null,
        children: [createBlockWrite(leftBlockId, "paragraph")],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: rightColumnId },
        after: null,
        children: [createBlockWrite(rightBlockId, "paragraph")],
      });
      await context.loadPageTree(selectedPageId);
      const focusBlockId = planNotesInsertedBlockFocus([leftBlockId], newBlockId);
      context.requestBlockFocus(focusBlockId);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    if (command.kind === "block" && command.blockType === "tab") {
      const firstLabelId = crypto.randomUUID();
      const secondLabelId = crypto.randomUUID();
      const firstContentId = crypto.randomUUID();
      const secondContentId = crypto.randomUUID();
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: newBlockId },
        after: null,
        children: [
          createBlockWrite(firstLabelId, "paragraph", "Tab 1"),
          createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
        ],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: firstLabelId },
        after: null,
        children: [createBlockWrite(firstContentId, "paragraph")],
      });
      await appendNotesBlockChildren({
        parent: { type: "block_id", block_id: secondLabelId },
        after: null,
        children: [createBlockWrite(secondContentId, "paragraph")],
      });
      await context.loadPageTree(selectedPageId);
      const focusBlockId = planNotesInsertedBlockFocus([firstContentId], newBlockId);
      context.requestBlockFocus(focusBlockId);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    await context.loadPageTree(selectedPageId);
    const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("create", before, focusBlockId);
  }

  async function splitTextBlockAtSelection(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
  ): Promise<void> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId || !notesEnterSplitsRichTextBlock(block.type)) return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const split = splitRichTextForBlock(blockEditableRichText(block), selectionStart, selectionEnd);
    const siblingType = notesEnterSiblingBlockType(block.type);
    const newBlockId = crypto.randomUUID();
    const currentUpdate = blockWithRichText(block, split.before);
    context.localApplyBlockUpdate(blockId, currentUpdate);
    await context.saveBlockNow(blockId, currentUpdate);
    await appendNotesBlockChildren({
      parent: block.parent,
      after: blockId,
      children: [
        createBlockWriteFromRichText(newBlockId, siblingType, split.after, blockColor(block)),
      ],
    });
    await context.loadPageTree(selectedPageId);
    const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("create", before, focusBlockId);
  }

  async function pastePlainTextIntoBlock(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    plainText: string,
  ): Promise<boolean> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId) return false;
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
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    context.localApplyBlockUpdate(blockId, plan.currentUpdate);
    await context.saveBlockNow(blockId, plan.currentUpdate);
    if (plan.appendedBlocks.length > 0) {
      await appendNotesBlockChildren({
        parent: block.parent,
        after: blockId,
        children: plan.appendedBlocks,
      });
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(plan.focusBlockId);
    recordUndoAfter("paste", before, plan.focusBlockId);
    return true;
  }

  async function pasteRichHtmlIntoBlock(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    html: string,
  ): Promise<boolean> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId) return false;
    const plan = planNotesRichHtmlPaste({
      currentBlock: block,
      selectionStart,
      selectionEnd,
      html,
      createId: () => crypto.randomUUID(),
    });
    if (!plan) return false;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    context.localApplyBlockUpdate(blockId, plan.currentUpdate);
    await context.saveBlockNow(blockId, plan.currentUpdate);
    if (plan.appendedBlocks.length > 0) {
      await appendNotesBlockChildren({
        parent: block.parent,
        after: blockId,
        children: plan.appendedBlocks,
      });
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(plan.focusBlockId);
    recordUndoAfter("paste", before, plan.focusBlockId);
    return true;
  }

  async function pasteBlockSelection(
    sourceRootBlockIds: readonly string[],
    sourceSubtreeBlockIds: readonly string[],
    targetBlockId: string,
    includeTrashedSources = false,
  ): Promise<string | null> {
    const selectedPageId = context.readSelectedPageId();
    const target = context.blockById(targetBlockId);
    if (!selectedPageId || !target || sourceRootBlockIds.length === 0) return null;
    if (sourceSubtreeBlockIds.length === 0) return null;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(targetBlockId);
    const duplicates = await duplicateNotesBlocks({
      block_ids: [...sourceRootBlockIds],
      duplicated_block_ids: sourceSubtreeBlockIds.map((sourceId) => ({
        source_id: sourceId,
        duplicate_id: crypto.randomUUID(),
      })),
      parent: target.parent,
      after: targetBlockId,
      before: null,
      include_trashed_sources: includeTrashedSources,
    });
    const focusBlockId = planNotesInsertedBlockFocus([duplicates.results[0]?.id], targetBlockId);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("paste", before, focusBlockId);
    return focusBlockId;
  }

  async function deleteBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planDeleteBlock(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    const before = undoSnapshot(blockId);
    if (plan.keepOnlyBlockAsParagraph) {
      await replaceBlockWithUpdate(blockId, createBlockUpdate("paragraph", ""));
      context.requestBlockFocus(blockId);
      recordUndoAfter("delete", before, blockId);
      return;
    }
    if (plan.deleteBlockId) {
      const childPlan = planReparentChildrenBeforeDelete(context.treeState(), plan.deleteBlockId);
      if (!childPlan || !(await moveReparentedChildren(childPlan))) return;
      await trashNotesBlock(plan.deleteBlockId, true);
      await context.loadPageTree(selectedPageId);
    }
    context.requestBlockFocus(plan.focusBlockId);
    recordUndoAfter("delete", before, plan.focusBlockId);
  }

  async function deleteBlockSelection(blockIds: readonly string[]): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const rootBlockIds = notesSelectionRootBlockIds(context.treeState(), blockIds);
    if (rootBlockIds.length === 0) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(rootBlockIds[0] ?? null);
    const focusBlockId = focusAfterDeletingSelection(rootBlockIds);
    await trashNotesBlocks({ block_ids: rootBlockIds, in_trash: true });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("delete", before, focusBlockId);
  }

  async function mergeBlockWithPrevious(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    let plan = planMergeWithPrevious(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    await context.flushBlockSave(plan.targetBlockId);
    plan = planMergeWithPrevious(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    const target = context.blockById(plan.targetBlockId);
    if (!target) return;
    const childPlan = planReparentChildrenAfterMerge(
      context.treeState(),
      plan.sourceBlockId,
      plan.targetBlockId,
    );
    if (!childPlan) return;
    const before = undoSnapshot(blockId);
    await replaceBlockWithUpdate(target.id, blockWithRichText(target, plan.mergedRichText));
    if (!(await moveReparentedChildren(childPlan))) return;
    await trashNotesBlock(plan.sourceBlockId, true);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(target.id);
    recordUndoAfter("delete", before, target.id);
  }

  async function nestBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planNestBlock(context.treeState(), blockId);
    if (!plan) return;
    const before = undoSnapshot(blockId);
    await moveNotesBlock(blockId, {
      parent: { type: "block_id", block_id: plan.parentId },
      after: plan.after,
    });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
    recordUndoAfter("move", before, blockId);
  }

  async function outdentBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planOutdentBlock(context.treeState(), blockId);
    if (!plan) return;
    const parent: NotesParent = plan.parentId === selectedPageId
      ? { type: "page_id", page_id: selectedPageId }
      : { type: "block_id", block_id: plan.parentId };
    const before = undoSnapshot(blockId);
    await moveNotesBlock(blockId, { parent, after: plan.after });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
    recordUndoAfter("move", before, blockId);
  }

  function parentFromMoveParentId(parentId: string): NotesParent | null {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return null;
    return parentId === selectedPageId
      ? { type: "page_id", page_id: selectedPageId }
      : { type: "block_id", block_id: parentId };
  }

  function rootSelectionParent(rootBlockIds: readonly string[]): NotesParent | null {
    const first = rootBlockIds[0] ? context.blockById(rootBlockIds[0]) : undefined;
    if (!first) return null;
    const parentKey = notesParentStorageKey(first.parent);
    const sameParent = rootBlockIds.every((blockId) => {
      const block = context.blockById(blockId);
      return block !== undefined && notesParentStorageKey(block.parent) === parentKey;
    });
    return sameParent ? first.parent : null;
  }

  function notesParentStorageKey(parent: NotesParent): string {
    if (parent.type === "page_id") return `page:${parent.page_id}`;
    if (parent.type === "block_id") return `block:${parent.block_id}`;
    return "workspace";
  }

  function focusAfterDeletingSelection(rootBlockIds: readonly string[]): string | null {
    const firstRoot = rootBlockIds[0];
    if (!firstRoot) return null;
    const state = context.treeState();
    return planNotesDeletedBlockFocus({
      visibleBlockIds: context.flatBlockItemsForBlockContext(firstRoot).map(
        (item) => item.block.id,
      ),
      removedBlockIds: notesSelectionSubtreeIds(state, rootBlockIds),
      firstRemovedBlockId: firstRoot,
    });
  }

  async function moveReparentedChildren(plan: NotesChildReparentPlan): Promise<boolean> {
    if (plan.childIds.length === 0) return true;
    const parent = parentFromMoveParentId(plan.parentId);
    if (!parent) return false;
    let after = plan.after;
    for (const childId of plan.childIds) {
      await context.flushBlockSave(childId);
      await moveNotesBlock(childId, { parent, after, before: null });
      after = childId;
    }
    return true;
  }

  async function moveBlockWithinSiblings(
    blockId: string,
    direction: "up" | "down",
  ): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planMoveBlockWithinSiblings(context.treeState(), blockId, direction);
    if (!plan) return;
    const parent = parentFromMoveParentId(plan.parentId);
    if (!parent) return;
    const before = undoSnapshot(blockId);
    await moveNotesBlock(blockId, { parent, after: plan.after, before: plan.before });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
    recordUndoAfter("move", before, blockId);
  }

  async function moveBlockSelection(
    blockIds: readonly string[],
    direction: NotesSelectionMoveDirection,
  ): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const rootBlockIds = notesSelectionRootBlockIds(context.treeState(), blockIds);
    const plan = planNotesSelectionMoveWithinSiblings(context.treeState(), rootBlockIds, direction);
    if (!plan) return;
    const parent = parentFromMoveParentId(plan.parentId);
    if (!parent) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(plan.focusBlockId);
    await moveNotesBlocks({
      block_ids: plan.blockIds,
      parent,
      after: plan.after,
      before: plan.before,
    });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(plan.focusBlockId);
    recordUndoAfter("move", before, plan.focusBlockId);
  }

  async function dropBlockWithinSiblings(
    sourceBlockId: string,
    targetBlockId: string,
    position: "before" | "after",
  ): Promise<void> {
    await dropBlockOnBlock(sourceBlockId, targetBlockId, position);
  }

  async function dropBlockOnBlock(
    sourceBlockId: string,
    targetBlockId: string,
    intent: NotesBlockDropIntent,
  ): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushPendingBlockSaves();
    const plan = planNotesBlockDrop(context.treeState(), sourceBlockId, targetBlockId, intent);
    if (!plan) return;
    const parent = parentFromMoveParentId(plan.parentId);
    if (!parent) return;
    const before = undoSnapshot(sourceBlockId);
    await moveNotesBlock(sourceBlockId, { parent, after: plan.after, before: plan.before });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(sourceBlockId);
    recordUndoAfter("move", before, sourceBlockId);
  }

  async function moveBlockToPage(blockId: string, pageId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId || pageId === selectedPageId) return;
    const block = context.blockById(blockId);
    if (!block || (block.type === "child_page" && block.id === pageId)) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    const state = context.treeState();
    const movedSubtreeIds = collectLoadedBlockSubtreeIds(state, blockId);
    const focusBlockId = planNotesDeletedBlockFocus({
      visibleBlockIds: context.flatBlockItemsForBlockContext(blockId).map(
        (item) => item.block.id,
      ),
      removedBlockIds: movedSubtreeIds,
      firstRemovedBlockId: blockId,
    });
    const movedSubtree = movedSubtreeIds
      .map((subtreeId) => context.blockById(subtreeId))
      .filter((candidate): candidate is NotesBlock => candidate !== undefined);
    const moved = await moveNotesBlock(blockId, {
      parent: { type: "page_id", page_id: pageId },
      after: null,
      before: null,
    });
    await context.reloadPages();
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(focusBlockId);
    const afterSubtree = movedSubtree.map((subtreeBlock) =>
      subtreeBlock.id === moved.id ? moved : subtreeBlock
    );
    recordUndoAfter("move", before, focusBlockId, null, afterSubtree);
  }

  async function duplicateBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    if (context.blockById(blockId)?.type === "child_page") return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const request = createDuplicateBlockRequest(
      context.treeState(),
      blockId,
      () => crypto.randomUUID(),
    );
    if (request.duplicated_block_ids.length === 0) return;
    const duplicate = await duplicateNotesBlock(blockId, request);
    const focusBlockId = planNotesInsertedBlockFocus([duplicate.id], blockId);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("duplicate", before, focusBlockId);
  }

  async function duplicateBlockSelection(blockIds: readonly string[]): Promise<string | null> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return null;
    const state = context.treeState();
    const rootBlockIds = notesSelectionRootBlockIds(state, blockIds);
    if (rootBlockIds.length === 0) return null;
    if (rootBlockIds.some((blockId) => context.blockById(blockId)?.type === "child_page")) {
      return null;
    }
    const parent = rootSelectionParent(rootBlockIds);
    if (!parent) return null;
    const lastRootId = rootBlockIds[rootBlockIds.length - 1];
    if (!lastRootId) return null;
    const sourceSubtreeBlockIds = notesSelectionSubtreeIds(state, rootBlockIds);
    if (sourceSubtreeBlockIds.length === 0) return null;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(rootBlockIds[0] ?? null);
    const duplicates = await duplicateNotesBlocks({
      block_ids: rootBlockIds,
      duplicated_block_ids: sourceSubtreeBlockIds.map((sourceId) => ({
        source_id: sourceId,
        duplicate_id: crypto.randomUUID(),
      })),
      parent,
      after: lastRootId,
      before: null,
      include_trashed_sources: false,
    });
    const focusBlockId = planNotesInsertedBlockFocus(
      [duplicates.results[0]?.id],
      rootBlockIds[0] ?? null,
    );
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("duplicate", before, focusBlockId);
    return focusBlockId;
  }

  async function useTemplateBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const template = context.blockById(blockId);
    if (!template || template.type !== "template") return;
    const childIds = (childIdsByParentId()[blockId] ?? [])
      .filter((childId) => {
        const child = blocksById()[childId];
        return child !== undefined && !child.in_trash;
      });
    if (childIds.length === 0) return;
    const state = context.treeState();
    const templateContainsChildPage = childIds.some((childId) =>
      collectLoadedBlockSubtreeIds(state, childId).some((subtreeId) =>
        blocksById()[subtreeId]?.type === "child_page"
      )
    );
    if (templateContainsChildPage) return;

    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    let after = blockId;
    let firstDuplicateId: string | null = null;
    for (const childId of childIds) {
      const request = createDuplicateBlockRequest(state, childId, () => crypto.randomUUID());
      if (request.duplicated_block_ids.length === 0) continue;
      const duplicate = await duplicateNotesBlock(childId, request);
      await moveNotesBlock(duplicate.id, {
        parent: template.parent,
        after,
        before: null,
      });
      after = duplicate.id;
      firstDuplicateId ??= duplicate.id;
    }

    await context.loadPageTree(selectedPageId);
    const focusBlockId = planNotesInsertedBlockFocus([firstDuplicateId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("template", before, focusBlockId);
  }

  function activeChildIdsForBlock(blockId: string): string[] {
    return (childIdsByParentId()[blockId] ?? []).filter((childId) => {
      const child = blocksById()[childId];
      return child !== undefined && !child.in_trash;
    });
  }

  function loadedSubtreesContainChildPage(state: NotesTreeState, childIds: readonly string[]): boolean {
    return childIds.some((childId) =>
      collectLoadedBlockSubtreeIds(state, childId).some(
        (subtreeId) => blocksById()[subtreeId]?.type === "child_page",
      ),
    );
  }

  function firstActiveSiblingId(parentId: string): string | null {
    return (
      (childIdsByParentId()[parentId] ?? []).find((childId) => {
        const child = blocksById()[childId];
        return child !== undefined && !child.in_trash;
      }) ?? null
    );
  }

  function buttonInsertTarget(
    blockId: string,
    position: NotesButtonInsertPosition,
  ): { parent: NotesParent; after: string | null; before: string | null } | null {
    const button = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId || !button || button.type !== "button") return null;
    if (position === "below_button") {
      return { parent: button.parent, after: blockId, before: null };
    }
    if (position === "above_button") {
      return { parent: button.parent, after: null, before: blockId };
    }
    const pageParent = { type: "page_id", page_id: selectedPageId } as const satisfies NotesParent;
    if (position === "bottom_of_page") {
      return { parent: pageParent, after: null, before: null };
    }
    return {
      parent: pageParent,
      after: null,
      before: firstActiveSiblingId(selectedPageId),
    };
  }

  async function insertLoadedChildSubtrees(
    childIds: readonly string[],
    target: { parent: NotesParent; after: string | null; before: string | null },
  ): Promise<string | null> {
    const state = context.treeState();
    let after = target.after;
    let before = target.before;
    let firstDuplicateId: string | null = null;
    for (const childId of childIds) {
      const request = createDuplicateBlockRequest(state, childId, () => crypto.randomUUID());
      if (request.duplicated_block_ids.length === 0) continue;
      const duplicate = await duplicateNotesBlock(childId, request);
      await moveNotesBlock(duplicate.id, {
        parent: target.parent,
        after,
        before,
      });
      after = duplicate.id;
      before = null;
      firstDuplicateId ??= duplicate.id;
    }
    return firstDuplicateId;
  }

  async function useButtonBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const button = context.blockById(blockId);
    if (!button || button.type !== "button") return;
    const action = button.button.actions.find((candidate) => candidate.type === "insert_blocks");
    if (!action) return;
    const childIds = activeChildIdsForBlock(blockId);
    if (childIds.length === 0) return;
    const state = context.treeState();
    if (loadedSubtreesContainChildPage(state, childIds)) return;
    const target = buttonInsertTarget(blockId, action.position);
    if (!target) return;

    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    const firstDuplicateId = await insertLoadedChildSubtrees(childIds, target);
    await context.loadPageTree(selectedPageId);
    const focusBlockId = planNotesInsertedBlockFocus([firstDuplicateId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("button", before, focusBlockId);
  }

  return {
    updateBlockText,
    updateBlockRichText,
    insertPageMention,
    insertDateMention,
    insertInlineEquation,
    updateBlockTextLink,
    updateBlockTextAnnotations,
    updateBookmark,
    updateEmbedUrl,
    updateLinkPreviewUrl,
    updateEquationExpression,
    updateMedia,
    updateTableCell,
    updateTableCellRichText,
    addTableRow,
    removeTableRow,
    addTableColumn,
    removeTableColumn,
    ...columnActions,
    convertBlock,
    toggleTodo,
    updateCodeLanguage,
    updateBlockColor,
    updateToggleOpen,
    convertBlockToToggleHeading,
    createSiblingAfter,
    splitTextBlockAtSelection,
    pastePlainTextIntoBlock,
    pasteRichHtmlIntoBlock,
    pasteBlockSelection,
    deleteBlock,
    deleteBlockSelection,
    mergeBlockWithPrevious,
    nestBlock,
    outdentBlock,
    moveBlockUp: (blockId: string) => moveBlockWithinSiblings(blockId, "up"),
    moveBlockDown: (blockId: string) => moveBlockWithinSiblings(blockId, "down"),
    moveBlockSelection,
    dropBlockWithinSiblings,
    dropBlockOnBlock,
    moveBlockToPage,
    duplicateBlock,
    duplicateBlockSelection,
    useTemplateBlock,
    useButtonBlock,
  };
}
