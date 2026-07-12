import {
  appendNotesBlockChildren,
  createNotesDatabase,
  createNotesLinkedDatabaseView,
  duplicateNotesBlocks,
  moveNotesBlock,
  moveNotesBlocks,
  trashNotesBlock,
  trashNotesBlocks,
} from "$lib/api/notes";
import {
  collectLoadedBlockSubtreeIds,
} from "$lib/notes/block-duplicate";
import { cloneNotesJson } from "$lib/notes/json-clone";
import {
  notesButtonWithIcon,
  notesButtonWithPrimaryInsertPosition,
} from "$lib/notes/button-block";
import {
  unsupportedBlockJsonText,
  unsupportedBlockSummaryText,
  type NotesUnsupportedConversionTarget,
} from "$lib/notes/unsupported";
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
import { createNotesLinkedDatabaseViewRequest } from "$lib/notes/database-linked";
import {
  blockEditableRichText,
  blockConvertedToType,
  blockPlainText,
  blockWithCodeLanguage,
  blockWithHeadingToggleable,
  blockWithHeadingToggleOpen,
  blockWithRichText,
  blockWithTodoChecked,
  blockWithToggleOpen,
  blockUpdateFromBlock,
  createBlockUpdate,
  createBlockWrite,
  createColumnPayload,
  createEmptyTableRowPayload,
  DEFAULT_TABLE_ROW_COUNT,
  DEFAULT_TABLE_WIDTH,
  isTextEditableBlock,
  type NotesHeadingBlockType,
} from "$lib/notes/block-factory";
import type { NotesTextSelection } from "$lib/notes/editor-selection";
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
  parentIdForBlock,
  type NotesChildReparentPlan,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import { splitRichTextForBlock } from "$lib/notes/rich-text-split";
import {
  type NotesUndoKind,
  type NotesUndoRecordOptions,
  type NotesUndoSnapshot,
} from "$lib/notes/undo-history";
import type {
  NotesBlock,
  NotesAppendBlockChildrenRequest,
  NotesBlockTreeItem,
  NotesBlockType,
  NotesBlockUpdate,
  NotesBlockWrite,
  NotesButtonInsertPosition,
  NotesColor,
  NotesIcon,
  NotesMoveBlockRequest,
  NotesMoveBlocksRequest,
  NotesColumnBlockItems,
  NotesParent,
  NotesTabBlockItems,
  NotesTableRowBlock,
} from "$lib/notes/types";
import {
  createNotesColumnActions,
  type NotesColumnActions,
} from "$lib/stores/notes-store-column-actions";
import {
  createNotesTabActions,
  type NotesTabActions,
} from "$lib/stores/notes-store-tab-actions";
import {
  createNotesMediaBlockActions,
  invalidateReplacedNotesMediaAsset,
  type NotesMediaBlockActions,
} from "$lib/stores/notes-store-block-media-actions";
import {
  createNotesRichTextBlockActions,
  type NotesRichTextBlockActions,
} from "$lib/stores/notes-store-block-rich-text-actions";
import {
  createNotesTableBlockActions,
  type NotesTableBlockActions,
} from "$lib/stores/notes-store-block-table-actions";
import {
  notesPostAppendResult,
  notesPostMoveManyResult,
  notesPostMoveResult,
  notesPostTrashResult,
  type NotesPostMutationResult,
} from "$lib/notes/post-mutation";

const DEFAULT_DATABASE_TITLE = "Untitled database";
const START_OF_BLOCK_SELECTION: NotesTextSelection = { start: 0, end: 0 };

export interface NotesBlockReadCapabilities {
  readSelectedPageId: () => string | null;
  readBlocksById: () => Record<string, NotesBlock>;
  readChildIdsByParentId: () => Record<string, string[]>;
  treeState: () => NotesTreeState;
  outlineSubtreeIds: (rootBlockIds: readonly string[]) => string[];
  blockById: (blockId: string) => NotesBlock | undefined;
  flatBlockItemsForBlockContext: (blockId: string) => NotesBlockTreeItem[];
  tableRowsForBlock: (blockId: string) => NotesTableRowBlock[];
  columnItemsForBlock: (blockId: string) => NotesColumnBlockItems[];
  tabItemsForBlock: (blockId: string) => NotesTabBlockItems[];
}

export interface NotesBlockNavigationCapabilities {
  setSidebarPageCollapsed: (pageId: string, collapsed: boolean) => void;
  requestBlockFocus: (blockId: string | null, selection?: NotesTextSelection | null) => void;
  createChildPageFromBlock: (blockId: string) => Promise<void>;
  createChildPageAfterBlock: (blockId: string) => Promise<void>;
  loadPageTree: (pageId: string) => Promise<void>;
  refreshOpenLinks: () => Promise<void>;
}

export interface NotesBlockLocalMutationCapabilities {
  applyPostMutation: (result: NotesPostMutationResult) => void;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  localInsertBlockAfter: (block: NotesBlock, afterBlockId: string | null) => void;
  localRemoveLeafBlock: (blockId: string) => boolean;
}

export interface NotesBlockPersistenceCapabilities {
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  flushBlockSave: (blockId: string) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
}

export interface NotesBlockUndoCapabilities {
  createUndoSnapshot: (
    focusBlockId: string | null,
    extraBlocks?: readonly NotesBlock[],
    focusSelection?: NotesTextSelection | null,
  ) => NotesUndoSnapshot | null;
  createUndoSnapshotForBlocks: (
    focusBlockId: string | null,
    blockIds: readonly string[],
    extraBlocks?: readonly NotesBlock[],
    focusSelection?: NotesTextSelection | null,
  ) => NotesUndoSnapshot | null;
  recordUndo: (options: Omit<NotesUndoRecordOptions, "id">) => void;
}

export interface NotesBlockActionsContext
  extends NotesBlockReadCapabilities,
    NotesBlockNavigationCapabilities,
    NotesBlockLocalMutationCapabilities,
    NotesBlockPersistenceCapabilities,
    NotesBlockUndoCapabilities {}

export interface NotesBlockActions
  extends NotesColumnActions,
    NotesTabActions,
    NotesMediaBlockActions,
    NotesRichTextBlockActions,
    NotesTableBlockActions {
  flushOptimisticBlockWrites: () => Promise<void>;
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
  addTemplateChild: (blockId: string) => Promise<void>;
  useTemplateBlock: (blockId: string) => Promise<void>;
  addButtonChild: (blockId: string) => Promise<void>;
  updateButtonIcon: (blockId: string, icon: NotesIcon | null) => Promise<void>;
  updateButtonInsertPosition: (
    blockId: string,
    position: NotesButtonInsertPosition,
  ) => Promise<void>;
  useButtonBlock: (blockId: string) => Promise<void>;
  createLinkedDatabaseViewAfter: (blockId: string) => Promise<void>;
  convertUnsupportedBlock: (
    blockId: string,
    target: NotesUnsupportedConversionTarget,
  ) => Promise<void>;
}

/**
 * Create Notes block mutation and UI action methods.
 */
export function createNotesBlockActions(context: NotesBlockActionsContext): NotesBlockActions {
  const blocksById = context.readBlocksById;
  const childIdsByParentId = context.readChildIdsByParentId;
  const pendingOptimisticBlockWrites = new Map<string, Promise<void>>();

  function trackOptimisticBlockWrites(
    blockIds: readonly string[],
    persistence: Promise<void>,
  ): void {
    for (const blockId of blockIds) {
      pendingOptimisticBlockWrites.set(blockId, persistence);
    }
    void persistence.finally(() => {
      for (const blockId of blockIds) {
        if (pendingOptimisticBlockWrites.get(blockId) === persistence) {
          pendingOptimisticBlockWrites.delete(blockId);
        }
      }
    });
  }

  async function flushOptimisticBlockWrites(): Promise<void> {
    await Promise.all([...new Set(pendingOptimisticBlockWrites.values())]);
  }

  function optimisticBlockFromWrite(write: NotesBlockWrite, parent: NotesParent): NotesBlock {
    const now = new Date().toISOString();
    const plainWrite = cloneNotesJson(write);
    const plainParent = cloneNotesJson(parent);
    return {
      object: "block",
      parent: plainParent,
      created_time: now,
      last_edited_time: now,
      has_children: false,
      in_trash: false,
      source_provider: null,
      source_object_id: null,
      source_last_edited_time: null,
      ...plainWrite,
    } as NotesBlock;
  }

  function undoSnapshot(
    focusBlockId: string | null,
    extraBlocks: readonly NotesBlock[] = [],
  ): NotesUndoSnapshot | null {
    return context.createUndoSnapshot(focusBlockId, extraBlocks);
  }

  function undoSnapshotForBlocks(
    blockIds: readonly string[],
    focusBlockId: string | null,
    focusSelection: NotesTextSelection | null = null,
    extraBlocks: readonly NotesBlock[] = [],
  ): NotesUndoSnapshot | null {
    return context.createUndoSnapshotForBlocks(
      focusBlockId,
      blockIds,
      extraBlocks,
      focusSelection,
    );
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

  function selectionAtBlockEnd(blockId: string): NotesTextSelection | null {
    const block = context.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type)) return null;
    const end = blockPlainText(block).length;
    return { start: end, end };
  }

  const columnActions = createNotesColumnActions(context);
  const tabActions = createNotesTabActions(context);
  const mediaActions = createNotesMediaBlockActions(context);
  const richTextActions = createNotesRichTextBlockActions({
    ...context,
    hasPendingOptimisticWrite: (blockId) => pendingOptimisticBlockWrites.has(blockId),
  });
  const tableActions = createNotesTableBlockActions({
    ...context,
    appendAndApply,
    trashAndApply,
    replaceBlockWithUpdate,
    undoSnapshot,
    recordUndoAfter,
  });

  async function replaceBlockWithUpdate(blockId: string, update: NotesBlockUpdate): Promise<void> {
    const previous = context.blockById(blockId);
    await context.flushBlockSave(blockId);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    invalidateReplacedNotesMediaAsset(previous, update);
  }

  async function appendAndApply(request: NotesAppendBlockChildrenRequest): Promise<NotesBlock[]> {
    const response = await appendNotesBlockChildren(request);
    context.applyPostMutation(notesPostAppendResult(request, response));
    return response.results;
  }

  async function moveAndApply(
    blockId: string,
    request: NotesMoveBlockRequest,
  ): Promise<NotesBlock> {
    const hierarchyChanged = context.blockById(blockId)?.type === "child_page";
    const block = await moveNotesBlock(blockId, request);
    context.applyPostMutation({
      ...notesPostMoveResult(block, request),
      sidebarImpact: hierarchyChanged ? "hierarchy" : "none",
    });
    return block;
  }

  async function moveManyAndApply(request: NotesMoveBlocksRequest): Promise<NotesBlock[]> {
    const hierarchyChanged = request.block_ids.some(
      (blockId) => context.blockById(blockId)?.type === "child_page",
    );
    const response = await moveNotesBlocks(request);
    context.applyPostMutation({
      ...notesPostMoveManyResult(request, response),
      sidebarImpact: hierarchyChanged ? "hierarchy" : "none",
    });
    return response.results;
  }

  async function trashAndApply(rootBlockIds: readonly string[]): Promise<void> {
    const before = context.treeState();
    const hierarchyChanged = rootBlockIds.some((blockId) => (
      collectLoadedBlockSubtreeIds(before, blockId).some(
        (subtreeId) => before.blocksById[subtreeId]?.type === "child_page",
      )
    ));
    if (rootBlockIds.length === 1) {
      const blockId = rootBlockIds[0];
      if (!blockId) return;
      await trashNotesBlock(blockId, true);
    } else {
      await trashNotesBlocks({ block_ids: [...rootBlockIds], in_trash: true });
    }
    context.applyPostMutation({
      ...notesPostTrashResult(before, rootBlockIds),
      sidebarImpact: hierarchyChanged ? "hierarchy" : "none",
    });
  }

  async function duplicateSubtreesAndApply(input: {
    rootBlockIds: readonly string[];
    sourceSubtreeBlockIds: readonly string[];
    parent: NotesParent;
    after: string | null;
    before?: string | null;
    includeTrashedSources?: boolean;
  }): Promise<NotesBlock[]> {
    const request = {
      block_ids: [...input.rootBlockIds],
      duplicated_block_ids: input.sourceSubtreeBlockIds.map((sourceId) => ({
        source_id: sourceId,
        duplicate_id: crypto.randomUUID(),
      })),
      parent: input.parent,
      after: input.after,
      before: input.before ?? null,
      include_trashed_sources: input.includeTrashedSources ?? false,
    };
    const response = await duplicateNotesBlocks(request);
    context.applyPostMutation(notesPostMoveManyResult(request, response));
    return response.results;
  }

  async function convertBlock(blockId: string, type: NotesBlockType, clearText = false): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (type === "child_page") {
      await context.createChildPageFromBlock(blockId);
      return;
    }
    if (type === "child_database") {
      await createDatabaseFromBlock(blockId);
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

  async function convertUnsupportedBlock(
    blockId: string,
    target: NotesUnsupportedConversionTarget,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "unsupported") return;
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const text = target === "code"
      ? unsupportedBlockJsonText(block.unsupported)
      : unsupportedBlockSummaryText(block.unsupported);
    const update = createBlockUpdate(target === "code" ? "code" : "paragraph", text);
    await replaceBlockWithUpdate(blockId, update);
    context.requestBlockFocus(blockId);
    recordUndoAfter("convert", before, blockId);
  }

  async function createDatabaseFromBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    const block = context.blockById(blockId);
    if (!selectedPageId || !block || !canConvertBlockToDatabase(block)) return;
    await context.flushBlockSave(blockId);
    const currentBlock = context.blockById(blockId) ?? block;
    if (!canConvertBlockToDatabase(currentBlock)) return;
    const before = undoSnapshot(blockId);
    const title = blockPlainText(currentBlock).trim() || DEFAULT_DATABASE_TITLE;
    const created = await createNotesDatabase({
      id: blockId,
      data_source_id: crypto.randomUUID(),
      view_id: crypto.randomUUID(),
      title,
      replace_block_id: blockId,
    });
    context.applyPostMutation({ blocks: [created.block] });
    context.requestBlockFocus(created.block.id);
    recordUndoAfter("convert", before, created.block.id);
  }

  function canConvertBlockToDatabase(block: NotesBlock): boolean {
    if (
      block.type === "child_page"
      || block.type === "table"
      || block.type === "table_row"
      || block.type === "column_list"
      || block.type === "column"
      || block.type === "tab"
    ) {
      return false;
    }
    return block.type !== "child_database" || block.child_database.database_id === undefined;
  }

  async function createTableFromBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const block = context.blockById(blockId);
    if (!block || block.type === "child_page" || block.type === "table_row") return;
    if ((childIdsByParentId()[blockId] ?? []).length > 0 && block.type !== "table") return;
    await replaceBlockWithUpdate(blockId, createBlockUpdate("table", ""));
    if (context.tableRowsForBlock(blockId).length === 0) {
    await appendAndApply({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: Array.from({ length: DEFAULT_TABLE_ROW_COUNT }, () => ({
          id: crypto.randomUUID(),
          type: "table_row" as const,
          table_row: createEmptyTableRowPayload(DEFAULT_TABLE_WIDTH),
        })),
      });
    }
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
    await appendAndApply({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: columns,
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: leftColumnId },
        after: null,
        children: [createBlockWrite(leftBlockId, "paragraph")],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: rightColumnId },
        after: null,
        children: [createBlockWrite(rightBlockId, "paragraph")],
      });
      context.requestBlockFocus(leftBlockId);
      return;
    }
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
    await appendAndApply({
        parent: { type: "block_id", block_id: blockId },
        after: null,
        children: [
          createBlockWrite(firstLabelId, "paragraph", firstLabel),
          createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
        ],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: firstLabelId },
        after: null,
        children: [createBlockWrite(firstContentId, "paragraph")],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: secondLabelId },
        after: null,
        children: [createBlockWrite(secondContentId, "paragraph")],
      });
      context.requestBlockFocus(firstContentId);
      return;
    }
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
    if (command.kind === "block" && command.blockType === "child_database") {
      await createDatabaseAfterBlock(blockId);
      return;
    }
    await context.flushBlockSave(blockId);
    const before = undoSnapshot(blockId);
    const newBlockId = crypto.randomUUID();
    await appendAndApply({
      parent: block.parent,
      after: blockId,
      children: [createBlockWriteFromInsertCommand(newBlockId, command)],
    });
    if (command.kind === "block" && command.blockType === "table") {
    await appendAndApply({
        parent: { type: "block_id", block_id: newBlockId },
        after: null,
        children: Array.from({ length: DEFAULT_TABLE_ROW_COUNT }, () => ({
          id: crypto.randomUUID(),
          type: "table_row" as const,
          table_row: createEmptyTableRowPayload(DEFAULT_TABLE_WIDTH),
        })),
      });
      const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId);
      context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    if (command.kind === "block" && command.blockType === "column_list") {
      const leftColumnId = crypto.randomUUID();
      const rightColumnId = crypto.randomUUID();
      const leftBlockId = crypto.randomUUID();
      const rightBlockId = crypto.randomUUID();
    await appendAndApply({
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
    await appendAndApply({
        parent: { type: "block_id", block_id: leftColumnId },
        after: null,
        children: [createBlockWrite(leftBlockId, "paragraph")],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: rightColumnId },
        after: null,
        children: [createBlockWrite(rightBlockId, "paragraph")],
      });
      const focusBlockId = planNotesInsertedBlockFocus([leftBlockId], newBlockId);
      context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    if (command.kind === "block" && command.blockType === "tab") {
      const firstLabelId = crypto.randomUUID();
      const secondLabelId = crypto.randomUUID();
      const firstContentId = crypto.randomUUID();
      const secondContentId = crypto.randomUUID();
    await appendAndApply({
        parent: { type: "block_id", block_id: newBlockId },
        after: null,
        children: [
          createBlockWrite(firstLabelId, "paragraph", "Tab 1"),
          createBlockWrite(secondLabelId, "paragraph", "Tab 2"),
        ],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: firstLabelId },
        after: null,
        children: [createBlockWrite(firstContentId, "paragraph")],
      });
    await appendAndApply({
        parent: { type: "block_id", block_id: secondLabelId },
        after: null,
        children: [createBlockWrite(secondContentId, "paragraph")],
      });
      const focusBlockId = planNotesInsertedBlockFocus([firstContentId], newBlockId);
      context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
      recordUndoAfter("create", before, focusBlockId);
      return;
    }
    const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId) ?? newBlockId;
    context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
    recordUndoAfter("create", before, focusBlockId);
  }

  async function createDatabaseAfterBlock(blockId: string): Promise<void> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId) return;
    await context.flushBlockSave(blockId);
    const currentBlock = context.blockById(blockId) ?? block;
    const before = undoSnapshot(blockId);
    const created = await createNotesDatabase({
      id: crypto.randomUUID(),
      data_source_id: crypto.randomUUID(),
      view_id: crypto.randomUUID(),
      title: DEFAULT_DATABASE_TITLE,
      parent: currentBlock.parent,
      after_block_id: blockId,
    });
    context.applyPostMutation({
      blocks: [created.block],
      placements: [{
        blockId: created.block.id,
        parent: currentBlock.parent,
        after: blockId,
      }],
    });
    context.requestBlockFocus(created.block.id);
    recordUndoAfter("create", before, created.block.id);
  }

  async function createLinkedDatabaseViewAfter(blockId: string): Promise<void> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || block.type !== "child_database" || !selectedPageId) return;
    await context.flushBlockSave(blockId);
    const currentBlock = context.blockById(blockId) ?? block;
    if (currentBlock.type !== "child_database") return;
    const before = undoSnapshot(blockId);
    const created = await createNotesLinkedDatabaseView(
      createNotesLinkedDatabaseViewRequest(
        currentBlock,
        {
          databaseId: crypto.randomUUID(),
          viewId: crypto.randomUUID(),
        },
        currentBlock.child_database.title || DEFAULT_DATABASE_TITLE,
      ),
    );
    context.applyPostMutation({
      blocks: [created.block],
      placements: [{
        blockId: created.block.id,
        parent: currentBlock.parent,
        after: blockId,
      }],
    });
    context.requestBlockFocus(created.block.id);
    recordUndoAfter("create", before, created.block.id);
  }

  async function splitTextBlockAtSelection(
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
  ): Promise<void> {
    const prerequisite = pendingOptimisticBlockWrites.get(blockId) ?? null;
    const currentBlock = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!currentBlock || !notesEnterSplitsRichTextBlock(currentBlock.type)) return;
    if (!selectedPageId) return;
    const beforeSelection = {
      start: Math.min(selectionStart, selectionEnd),
      end: Math.max(selectionStart, selectionEnd),
    };
    const before = undoSnapshotForBlocks([blockId], blockId, beforeSelection);
    const split = splitRichTextForBlock(
      blockEditableRichText(currentBlock),
      selectionStart,
      selectionEnd,
    );
    const siblingType = notesEnterSiblingBlockType(currentBlock.type);
    const newBlockId = crypto.randomUUID();
    const currentUpdate = cloneNotesJson(blockWithRichText(currentBlock, split.before));
    const nextBlockWrite = cloneNotesJson(
      createBlockWriteFromRichText(
        newBlockId,
        siblingType,
        split.after,
        blockColor(currentBlock),
      ),
    );
    const parent = cloneNotesJson(currentBlock.parent);
    const nextBlock = optimisticBlockFromWrite(nextBlockWrite, parent);
    const focusBlockId = planNotesInsertedBlockFocus([newBlockId], blockId) ?? newBlockId;

    context.localApplyBlockUpdate(blockId, currentUpdate);
    if (!prerequisite) context.scheduleBlockSave(blockId, currentUpdate);
    context.localInsertBlockAfter(nextBlock, blockId);
    context.requestBlockFocus(focusBlockId, START_OF_BLOCK_SELECTION);
    recordUndo(
      "create",
      before,
      undoSnapshotForBlocks(
        [blockId, newBlockId],
        focusBlockId,
        START_OF_BLOCK_SELECTION,
      ),
      `create:enter:${parentIdForBlock(currentBlock)}`,
    );
    const createPromise = persistSplitTextBlock(
      selectedPageId,
      blockId,
      currentUpdate,
      parent,
      nextBlockWrite,
      prerequisite,
    );
    trackOptimisticBlockWrites(
      prerequisite ? [blockId, newBlockId] : [newBlockId],
      createPromise,
    );
    void createPromise;
  }

  async function persistSplitTextBlock(
    selectedPageId: string,
    blockId: string,
    currentUpdate: NotesBlockUpdate,
    parent: NotesParent,
    nextBlockWrite: NotesBlockWrite,
    prerequisite: Promise<void> | null,
  ): Promise<void> {
    try {
      await prerequisite;
      if (prerequisite) {
        await context.saveBlockNow(blockId, cloneNotesJson(currentUpdate));
      } else {
        await context.flushBlockSave(blockId);
      }
    await appendAndApply({
        parent: cloneNotesJson(parent),
        after: blockId,
        children: [cloneNotesJson(nextBlockWrite)],
      });
      const nextBlock = context.blockById(nextBlockWrite.id);
      if (nextBlock) {
        await context.saveBlockNow(
          nextBlockWrite.id,
          cloneNotesJson(blockWithRichText(nextBlock, blockEditableRichText(nextBlock))),
        );
      }
    } catch (error) {
      console.warn("notes split block persistence failed", error);
      await context.loadPageTree(selectedPageId);
    }
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
    applyOptimisticPaste(selectedPageId, block, plan, {
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
    applyOptimisticPaste(selectedPageId, block, plan, {
      start: Math.min(selectionStart, selectionEnd),
      end: Math.max(selectionStart, selectionEnd),
    });
    return true;
  }

  interface OptimisticPastePlan {
    currentUpdate: NotesBlockUpdate;
    appendedBlocks: NotesBlockWrite[];
    focusBlockId: string;
    focusOffset: number;
  }

  function applyOptimisticPaste(
    selectedPageId: string,
    currentBlock: NotesBlock,
    plan: OptimisticPastePlan,
    beforeFocusSelection: NotesTextSelection,
  ): void {
    const affectedBlockIds = [
      currentBlock.id,
      ...plan.appendedBlocks.map((write) => write.id),
    ];
    const before = undoSnapshotForBlocks(
      affectedBlockIds,
      currentBlock.id,
      beforeFocusSelection,
    );
    const currentUpdate = cloneNotesJson(plan.currentUpdate);
    const appendedWrites = plan.appendedBlocks.map((write) => cloneNotesJson(write));
    const parent = cloneNotesJson(currentBlock.parent);
    const focusSelection = { start: plan.focusOffset, end: plan.focusOffset };

    context.localApplyBlockUpdate(currentBlock.id, currentUpdate);
    context.scheduleBlockSave(currentBlock.id, currentUpdate);

    let afterBlockId = currentBlock.id;
    for (const write of appendedWrites) {
      context.localInsertBlockAfter(optimisticBlockFromWrite(write, parent), afterBlockId);
      afterBlockId = write.id;
    }

    context.requestBlockFocus(plan.focusBlockId, focusSelection);
    recordUndo(
      "paste",
      before,
      undoSnapshotForBlocks(affectedBlockIds, plan.focusBlockId, focusSelection),
    );

    if (appendedWrites.length === 0) return;
    const persistence = persistOptimisticPaste(
      selectedPageId,
      currentBlock.id,
      parent,
      appendedWrites,
    );
    trackOptimisticBlockWrites(appendedWrites.map((write) => write.id), persistence);
    void persistence;
  }

  async function persistOptimisticPaste(
    selectedPageId: string,
    currentBlockId: string,
    parent: NotesParent,
    appendedWrites: readonly NotesBlockWrite[],
  ): Promise<void> {
    try {
      await context.flushBlockSave(currentBlockId);
    await appendAndApply({
        parent: cloneNotesJson(parent),
        after: currentBlockId,
        children: cloneNotesJson([...appendedWrites]),
      });
      for (const write of appendedWrites) {
        const latestBlock = context.blockById(write.id);
        if (latestBlock) {
          await context.saveBlockNow(write.id, blockUpdateFromBlock(latestBlock));
        }
      }
    } catch (error) {
      console.warn("notes paste persistence failed", error);
      await context.loadPageTree(selectedPageId);
    }
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
    const duplicates = await duplicateSubtreesAndApply({
      rootBlockIds: sourceRootBlockIds,
      sourceSubtreeBlockIds,
      parent: target.parent,
      after: targetBlockId,
      before: null,
      includeTrashedSources,
    });
    const focusBlockId = planNotesInsertedBlockFocus([duplicates[0]?.id], targetBlockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("paste", before, focusBlockId);
    return focusBlockId;
  }

  async function deleteBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const plan = planDeleteBlock(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    const before = undoSnapshot(blockId);
    if (plan.keepOnlyBlockAsParagraph) {
      await context.flushBlockSave(blockId);
      await replaceBlockWithUpdate(blockId, createBlockUpdate("paragraph", ""));
      context.requestBlockFocus(blockId);
      recordUndoAfter("delete", before, blockId);
      return;
    }
    if (plan.deleteBlockId) {
      const childPlan = planReparentChildrenBeforeDelete(context.treeState(), plan.deleteBlockId);
      if (!childPlan) return;
      const deletedBlock = context.blockById(plan.deleteBlockId);
      const canDeleteOptimistically = childPlan.childIds.length === 0
        && deletedBlock !== undefined
        && isTextEditableBlock(deletedBlock.type)
        && blockPlainText(deletedBlock).length === 0;
      if (canDeleteOptimistically) {
        const prerequisite = pendingOptimisticBlockWrites.get(plan.deleteBlockId) ?? null;
        const focusSelection = selectionAtBlockEnd(plan.focusBlockId);
        if (!context.localRemoveLeafBlock(plan.deleteBlockId)) return;
        context.requestBlockFocus(plan.focusBlockId, focusSelection);
        recordUndo(
          "delete",
          before,
          context.createUndoSnapshot(plan.focusBlockId, [], focusSelection),
        );
        const persistence = persistOptimisticLeafDelete(
          selectedPageId,
          plan.deleteBlockId,
          prerequisite,
        );
        trackOptimisticBlockWrites([plan.deleteBlockId], persistence);
        void persistence;
        return;
      }
      await context.flushBlockSave(blockId);
      if (!(await moveReparentedChildren(childPlan))) return;
      await trashAndApply([plan.deleteBlockId]);
    }
    context.requestBlockFocus(plan.focusBlockId);
    recordUndoAfter("delete", before, plan.focusBlockId);
  }

  async function persistOptimisticLeafDelete(
    selectedPageId: string,
    blockId: string,
    prerequisite: Promise<void> | null,
  ): Promise<void> {
    try {
      await prerequisite;
      await context.flushBlockSave(blockId);
      await trashNotesBlock(blockId, true);
    } catch (error) {
      console.warn("notes leaf block delete persistence failed", error);
      await context.loadPageTree(selectedPageId);
    }
  }

  async function deleteBlockSelection(blockIds: readonly string[]): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const rootBlockIds = notesSelectionRootBlockIds(context.treeState(), blockIds);
    if (rootBlockIds.length === 0) return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(rootBlockIds[0] ?? null);
    const focusBlockId = focusAfterDeletingSelection(rootBlockIds);
    await trashAndApply(rootBlockIds);
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
    await trashAndApply([plan.sourceBlockId]);
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
    await moveAndApply(blockId, {
      parent: { type: "block_id", block_id: plan.parentId },
      after: plan.after,
    });
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
    await moveAndApply(blockId, { parent, after: plan.after });
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
      await moveAndApply(childId, { parent, after, before: null });
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
    await moveAndApply(blockId, { parent, after: plan.after, before: plan.before });
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
    await moveManyAndApply({
      block_ids: plan.blockIds,
      parent,
      after: plan.after,
      before: plan.before,
    });
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
    await moveAndApply(sourceBlockId, { parent, after: plan.after, before: plan.before });
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
    const moved = await moveAndApply(blockId, {
      parent: { type: "page_id", page_id: pageId },
      after: null,
      before: null,
    });
    context.applyPostMutation({ sidebarImpact: "hierarchy" });
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
    const subtreeIds = context.outlineSubtreeIds([blockId]);
    if (subtreeIds.length === 0) return;
    const block = context.blockById(blockId);
    if (!block) return;
    const duplicates = await duplicateSubtreesAndApply({
      rootBlockIds: [blockId],
      sourceSubtreeBlockIds: subtreeIds,
      parent: block.parent,
      after: blockId,
    });
    const focusBlockId = planNotesInsertedBlockFocus([duplicates[0]?.id], blockId);
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
    const sourceSubtreeBlockIds = context.outlineSubtreeIds(rootBlockIds);
    if (sourceSubtreeBlockIds.length === 0) return null;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(rootBlockIds[0] ?? null);
    const duplicates = await duplicateSubtreesAndApply({
      rootBlockIds,
      sourceSubtreeBlockIds,
      parent,
      after: lastRootId,
      before: null,
    });
    const focusBlockId = planNotesInsertedBlockFocus(
      [duplicates[0]?.id],
      rootBlockIds[0] ?? null,
    );
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
      const sourceSubtreeIds = collectLoadedBlockSubtreeIds(state, childId);
      if (sourceSubtreeIds.length === 0) continue;
      const duplicates = await duplicateSubtreesAndApply({
        rootBlockIds: [childId],
        sourceSubtreeBlockIds: sourceSubtreeIds,
        parent: template.parent,
        after,
        before: null,
      });
      const duplicateId = duplicates[0]?.id;
      if (!duplicateId) continue;
      after = duplicateId;
      firstDuplicateId ??= duplicateId;
    }

    const focusBlockId = planNotesInsertedBlockFocus([firstDuplicateId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("template", before, focusBlockId);
  }

  async function addTemplateChild(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const template = context.blockById(blockId);
    if (!template || template.type !== "template") return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    const childIds = activeChildIdsForBlock(blockId);
    const newBlockId = crypto.randomUUID();
    await appendAndApply({
      parent: { type: "block_id", block_id: blockId },
      after: childIds.at(-1) ?? null,
      children: [createBlockWrite(newBlockId, "paragraph", "")],
    });
    context.requestBlockFocus(newBlockId);
    recordUndoAfter("create", before, newBlockId);
  }

  async function addButtonChild(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    const button = context.blockById(blockId);
    if (!button || button.type !== "button") return;
    await context.flushPendingBlockSaves();
    const before = undoSnapshot(blockId);
    const childIds = activeChildIdsForBlock(blockId);
    const newBlockId = crypto.randomUUID();
    await appendAndApply({
      parent: { type: "block_id", block_id: blockId },
      after: childIds.at(-1) ?? null,
      children: [createBlockWrite(newBlockId, "paragraph", "")],
    });
    context.requestBlockFocus(newBlockId);
    recordUndoAfter("create", before, newBlockId);
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
      const sourceSubtreeIds = collectLoadedBlockSubtreeIds(state, childId);
      if (sourceSubtreeIds.length === 0) continue;
      const duplicates = await duplicateSubtreesAndApply({
        rootBlockIds: [childId],
        sourceSubtreeBlockIds: sourceSubtreeIds,
        parent: target.parent,
        after,
        before,
      });
      const duplicateId = duplicates[0]?.id;
      if (!duplicateId) continue;
      after = duplicateId;
      before = null;
      firstDuplicateId ??= duplicateId;
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
    const focusBlockId = planNotesInsertedBlockFocus([firstDuplicateId], blockId);
    context.requestBlockFocus(focusBlockId);
    recordUndoAfter("button", before, focusBlockId);
  }

  async function updateButtonIcon(blockId: string, icon: NotesIcon | null): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "button") return;
    const before = undoSnapshot(blockId);
    const update = { type: "button" as const, button: notesButtonWithIcon(block.button, icon) };
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  async function updateButtonInsertPosition(
    blockId: string,
    position: NotesButtonInsertPosition,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "button") return;
    const before = undoSnapshot(blockId);
    const update = {
      type: "button" as const,
      button: notesButtonWithPrimaryInsertPosition(block.button, position),
    };
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
    recordUndoAfter("update", before, blockId, `update:${blockId}`);
  }

  return {
    flushOptimisticBlockWrites,
    ...richTextActions,
    ...mediaActions,
    ...tableActions,
    ...columnActions,
    ...tabActions,
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
    addTemplateChild,
    useTemplateBlock,
    addButtonChild,
    updateButtonIcon,
    updateButtonInsertPosition,
    useButtonBlock,
    createLinkedDatabaseViewAfter,
    convertUnsupportedBlock,
  };
}
