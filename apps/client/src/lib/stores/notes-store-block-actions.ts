import {
  appendNotesBlockChildren,
  duplicateNotesBlock,
  moveNotesBlock,
  trashNotesBlock,
} from "$lib/api/notes";
import {
  collectLoadedBlockSubtreeIds,
  createDuplicateBlockRequest,
} from "$lib/notes/block-duplicate";
import { planNotesPlainTextPaste } from "$lib/notes/block-clipboard";
import { blockWithColor } from "$lib/notes/block-color";
import {
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
  blockWithTableCell,
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
import {
  planDeleteBlock,
  planDropBlockWithinSiblings,
  planMergeWithPrevious,
  planMoveBlockWithinSiblings,
  planNestBlock,
  planOutdentBlock,
  type NotesTreeState,
} from "$lib/notes/block-tree";
import type { NotesRichTextAnnotationPatch } from "$lib/notes/rich-text";
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
  NotesTabBlockItems,
  NotesTableRowBlock,
} from "$lib/notes/types";

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
  loadPageTree: (pageId: string) => Promise<void>;
  reloadPages: () => Promise<void>;
  reloadBacklinks: () => Promise<void>;
  localApplyBlockUpdate: (blockId: string, update: NotesBlockUpdate) => void;
  saveBlockNow: (blockId: string, update: NotesBlockUpdate) => Promise<void>;
  scheduleBlockSave: (blockId: string, update: NotesBlockUpdate) => void;
  flushBlockSave: (blockId: string) => Promise<void>;
  flushPendingBlockSaves: () => Promise<void>;
}

export interface NotesBlockActions {
  updateBlockText: (blockId: string, text: string) => Promise<void>;
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
  convertBlock: (blockId: string, type: NotesBlockType, clearText?: boolean) => Promise<void>;
  toggleTodo: (blockId: string, checked: boolean) => Promise<void>;
  updateCodeLanguage: (blockId: string, language: string) => Promise<void>;
  updateBlockColor: (blockId: string, color: NotesColor) => Promise<void>;
  updateToggleOpen: (blockId: string, open: boolean) => Promise<void>;
  convertBlockToToggleHeading: (
    blockId: string,
    headingType: NotesHeadingBlockType,
  ) => Promise<void>;
  createSiblingAfter: (blockId: string, type?: NotesBlockType) => Promise<void>;
  pastePlainTextIntoBlock: (
    blockId: string,
    selectionStart: number,
    selectionEnd: number,
    plainText: string,
  ) => Promise<boolean>;
  deleteBlock: (blockId: string) => Promise<void>;
  mergeBlockWithPrevious: (blockId: string) => Promise<void>;
  nestBlock: (blockId: string) => Promise<void>;
  outdentBlock: (blockId: string) => Promise<void>;
  moveBlockUp: (blockId: string) => Promise<void>;
  moveBlockDown: (blockId: string) => Promise<void>;
  dropBlockWithinSiblings: (
    sourceBlockId: string,
    targetBlockId: string,
    position: "before" | "after",
  ) => Promise<void>;
  moveBlockToPage: (blockId: string, pageId: string) => Promise<void>;
  duplicateBlock: (blockId: string) => Promise<void>;
  useTemplateBlock: (blockId: string) => Promise<void>;
  useButtonBlock: (blockId: string) => Promise<void>;
}

/**
 * Create Notes block mutation and UI action methods.
 */
export function createNotesBlockActions(context: NotesBlockActionsContext): NotesBlockActions {
  const blocksById = context.readBlocksById;
  const childIdsByParentId = context.readChildIdsByParentId;

  async function replaceBlockWithUpdate(blockId: string, update: NotesBlockUpdate): Promise<void> {
    await context.flushBlockSave(blockId);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
  }

  async function updateBlockText(blockId: string, text: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    const update = blockWithText(block, text);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
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
    const update = blockWithPageMention(block, start, end, pageId, title, href);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    await context.reloadBacklinks();
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
    const update = blockWithDateMention(block, start, end, date, title);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
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
    const update = blockWithInlineEquation(block, start, end, expression);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
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
    const update = blockWithTextLink(block, start, end, url);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
    await context.reloadBacklinks();
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
    const update = blockWithTextAnnotations(block, start, end, patch);
    context.localApplyBlockUpdate(blockId, update);
    await context.saveBlockNow(blockId, update);
  }

  async function updateBookmark(blockId: string, url: string, caption: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "bookmark") return;
    const update = blockWithBookmark(block, url, caption);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
  }

  async function updateEmbedUrl(blockId: string, url: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "embed") return;
    const update = blockWithEmbedUrl(block, url);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
  }

  async function updateLinkPreviewUrl(blockId: string, url: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "link_preview") return;
    const update = blockWithLinkPreviewUrl(block, url);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
  }

  async function updateEquationExpression(blockId: string, expression: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block || block.type !== "equation") return;
    const update = blockWithEquationExpression(block, expression);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
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
    const update = blockWithMedia(block, url, caption, name);
    context.localApplyBlockUpdate(blockId, update);
    context.scheduleBlockSave(blockId, update);
  }

  async function updateTableCell(
    rowBlockId: string,
    columnIndex: number,
    text: string,
  ): Promise<void> {
    const block = context.blockById(rowBlockId);
    if (!block || block.type !== "table_row") return;
    const update = blockWithTableCell(block, columnIndex, text);
    context.localApplyBlockUpdate(rowBlockId, update);
    context.scheduleBlockSave(rowBlockId, update);
  }

  async function convertBlock(blockId: string, type: NotesBlockType, clearText = false): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (type === "child_page") {
      await context.createChildPageFromBlock(blockId);
      return;
    }
    if (type === "table") {
      await createTableFromBlock(blockId);
      return;
    }
    if (type === "column_list") {
      await createColumnListFromBlock(blockId);
      return;
    }
    if (type === "tab") {
      await createTabFromBlock(blockId);
      return;
    }
    if (block.type === "child_page") return;
    if (block.type === "table" || block.type === "table_row") return;
    if (block.type === "column_list" || block.type === "column") return;
    if (block.type === "tab") return;
    const update = clearText ? createBlockUpdate(type, "") : blockConvertedToType(block, type);
    await replaceBlockWithUpdate(blockId, update);
    context.requestBlockFocus(type === "divider" ? null : blockId);
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
    await replaceBlockWithUpdate(blockId, blockWithTodoChecked(block, checked));
  }

  async function updateCodeLanguage(blockId: string, language: string): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    await replaceBlockWithUpdate(blockId, blockWithCodeLanguage(block, language));
  }

  async function updateBlockColor(blockId: string, color: NotesColor): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    await replaceBlockWithUpdate(blockId, blockWithColor(block, color));
  }

  async function updateToggleOpen(blockId: string, open: boolean): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (block.type === "toggle") {
      await replaceBlockWithUpdate(blockId, blockWithToggleOpen(block, open));
      return;
    }
    if (
      (block.type === "heading_1" && block.heading_1.is_toggleable === true)
      || (block.type === "heading_2" && block.heading_2.is_toggleable === true)
      || (block.type === "heading_3" && block.heading_3.is_toggleable === true)
      || (block.type === "heading_4" && block.heading_4.is_toggleable === true)
    ) {
      await replaceBlockWithUpdate(blockId, blockWithHeadingToggleOpen(block, open));
    }
  }

  async function convertBlockToToggleHeading(
    blockId: string,
    headingType: NotesHeadingBlockType,
  ): Promise<void> {
    const block = context.blockById(blockId);
    if (!block) return;
    if (block.type === "child_page") return;
    if (block.type === "table" || block.type === "table_row") return;
    if (block.type === "column_list" || block.type === "column") return;
    await replaceBlockWithUpdate(blockId, blockWithHeadingToggleable(block, headingType, true));
    context.requestBlockFocus(blockId);
  }

  async function createSiblingAfter(blockId: string, type: NotesBlockType = "paragraph"): Promise<void> {
    const block = context.blockById(blockId);
    const selectedPageId = context.readSelectedPageId();
    if (!block || !selectedPageId) return;
    await context.flushBlockSave(blockId);
    const newBlockId = crypto.randomUUID();
    await appendNotesBlockChildren({
      parent: block.parent,
      after: blockId,
      children: [createBlockWrite(newBlockId, type)],
    });
    if (type === "tab") {
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
      context.requestBlockFocus(firstContentId);
      return;
    }
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(type === "divider" ? null : newBlockId);
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
    return true;
  }

  async function deleteBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planDeleteBlock(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    if (plan.keepOnlyBlockAsParagraph) {
      await replaceBlockWithUpdate(blockId, createBlockUpdate("paragraph", ""));
      context.requestBlockFocus(blockId);
      return;
    }
    if (plan.deleteBlockId) {
      await trashNotesBlock(plan.deleteBlockId, true);
      await context.loadPageTree(selectedPageId);
    }
    context.requestBlockFocus(plan.focusBlockId);
  }

  async function mergeBlockWithPrevious(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planMergeWithPrevious(context.flatBlockItemsForBlockContext(blockId), blockId);
    if (!plan) return;
    const target = context.blockById(plan.targetBlockId);
    if (!target) return;
    await replaceBlockWithUpdate(target.id, blockWithText(target, plan.mergedText));
    await trashNotesBlock(plan.sourceBlockId, true);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(target.id);
  }

  async function nestBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(blockId);
    const plan = planNestBlock(context.treeState(), blockId);
    if (!plan) return;
    await moveNotesBlock(blockId, {
      parent: { type: "block_id", block_id: plan.parentId },
      after: plan.after,
    });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
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
    await moveNotesBlock(blockId, { parent, after: plan.after });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
  }

  function parentFromMoveParentId(parentId: string): NotesParent | null {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return null;
    return parentId === selectedPageId
      ? { type: "page_id", page_id: selectedPageId }
      : { type: "block_id", block_id: parentId };
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
    await moveNotesBlock(blockId, { parent, after: plan.after, before: plan.before });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(blockId);
  }

  async function dropBlockWithinSiblings(
    sourceBlockId: string,
    targetBlockId: string,
    position: "before" | "after",
  ): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    await context.flushBlockSave(sourceBlockId);
    const plan = planDropBlockWithinSiblings(context.treeState(), sourceBlockId, targetBlockId, position);
    if (!plan) return;
    const parent = parentFromMoveParentId(plan.parentId);
    if (!parent) return;
    await moveNotesBlock(sourceBlockId, { parent, after: plan.after, before: plan.before });
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(sourceBlockId);
  }

  async function moveBlockToPage(blockId: string, pageId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId || pageId === selectedPageId) return;
    const block = context.blockById(blockId);
    if (!block || (block.type === "child_page" && block.id === pageId)) return;
    await context.flushBlockSave(blockId);
    await moveNotesBlock(blockId, {
      parent: { type: "page_id", page_id: pageId },
      after: null,
      before: null,
    });
    await context.reloadPages();
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(null);
  }

  async function duplicateBlock(blockId: string): Promise<void> {
    const selectedPageId = context.readSelectedPageId();
    if (!selectedPageId) return;
    if (context.blockById(blockId)?.type === "child_page") return;
    await context.flushBlockSave(blockId);
    const request = createDuplicateBlockRequest(context.treeState(), blockId, () => crypto.randomUUID());
    if (request.duplicated_block_ids.length === 0) return;
    const duplicate = await duplicateNotesBlock(blockId, request);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(duplicate.id);
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
    context.requestBlockFocus(firstDuplicateId);
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
    const firstDuplicateId = await insertLoadedChildSubtrees(childIds, target);
    await context.loadPageTree(selectedPageId);
    context.requestBlockFocus(firstDuplicateId);
  }

  return {
    updateBlockText,
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
    convertBlock,
    toggleTodo,
    updateCodeLanguage,
    updateBlockColor,
    updateToggleOpen,
    convertBlockToToggleHeading,
    createSiblingAfter,
    pastePlainTextIntoBlock,
    deleteBlock,
    mergeBlockWithPrevious,
    nestBlock,
    outdentBlock,
    moveBlockUp: (blockId: string) => moveBlockWithinSiblings(blockId, "up"),
    moveBlockDown: (blockId: string) => moveBlockWithinSiblings(blockId, "down"),
    dropBlockWithinSiblings,
    moveBlockToPage,
    duplicateBlock,
    useTemplateBlock,
    useButtonBlock,
  };
}
