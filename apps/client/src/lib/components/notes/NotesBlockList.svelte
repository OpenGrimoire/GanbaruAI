<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { buildNotesBlockLink, buildNotesPageLink } from "$lib/notes/block-link";
  import {
    notesBlockSelectionAfterClick,
    notesBlockSelectionAfterKeyboard,
    notesBlockSelectionContains,
    notesBlockSelectionForBlock,
    notesBlockSelectionPrunedToVisible,
    notesBlockSelectionRange,
    normalizeNotesSelectableBlockIds,
  } from "$lib/notes/block-selection";
  import type { NotesBlockSelectionState } from "$lib/notes/block-selection";
  import {
    notesSelectionPlainText,
    notesSelectionRootBlockIds,
    notesSelectionSubtreeIds,
    planNotesSelectionMoveWithinSiblings,
  } from "$lib/notes/block-selection-operations";
  import { notesPageIconText } from "$lib/notes/page-icon";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesTemplateBlockStatus } from "$lib/notes/template-block";
  import { notesButtonBlockStatus } from "$lib/notes/button-block";
  import type { NotesUnsupportedConversionTarget } from "$lib/notes/unsupported";
  import {
    isTextEditableBlock,
    type NotesHeadingBlockType,
  } from "$lib/notes/block-factory";
  import type { NotesBlockInsertRequest } from "$lib/notes/block-insertion";
  import {
    notesPlainTextFromEditableRoot,
    restoreNotesEditableSelection,
  } from "$lib/notes/editor-selection";
  import type {
    NotesDateMentionTarget,
    NotesPageMentionTarget,
    NotesRichTextAnnotationPatch,
  } from "$lib/notes/rich-text";
  import type { NotesKeyboardAction } from "$lib/notes/block-keyboard";
  import {
    notesMoveToPageTargets,
    type NotesMoveToPageTarget,
  } from "$lib/notes/block-move";
  import {
    NOTES_BLOCK_DRAG_MIME,
    planNotesBlockDrop,
    setActiveNotesBlockDragId,
    type NotesBlockDropIndicator,
    type NotesBlockDropIntent,
  } from "$lib/notes/block-drag";
  import type {
    NotesBlock,
    NotesBlockTreeItem,
    NotesBlockType,
    NotesButtonInsertPosition,
    NotesIcon,
    NotesPageBreadcrumbItem,
    NotesRichText,
    NotesTableOfContentsItem,
  } from "$lib/notes/types";
  import NotesBlockRow from "./NotesBlockRow.svelte";
  import NotesColumnListBlock from "./NotesColumnListBlock.svelte";
  import NotesTabBlock from "./NotesTabBlock.svelte";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ClipboardPaste from "@lucide/svelte/icons/clipboard-paste";
  import Copy from "@lucide/svelte/icons/copy";
  import CopyPlus from "@lucide/svelte/icons/copy-plus";
  import Scissors from "@lucide/svelte/icons/scissors";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  type NotesBlockSelectionClipboard = {
    mode: "copy" | "cut";
    pageId: string;
    rootBlockIds: string[];
    subtreeBlockIds: string[];
    plainText: string;
  };

  let {
    items,
    pageId,
    breadcrumbItems,
    tableOfContentsItems,
    onSelectPage,
    onFocusBlock,
  }: {
    items: NotesBlockTreeItem[];
    pageId: string;
    breadcrumbItems: NotesPageBreadcrumbItem[];
    tableOfContentsItems: NotesTableOfContentsItem[];
    onSelectPage: (pageId: string) => void;
    onFocusBlock: (blockId: string) => void;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();
  let blockListElement: HTMLDivElement | null = $state(null);
  let draggingBlockId = $state<string | null>(null);
  let dropTarget = $state<{ blockId: string; intent: NotesBlockDropIndicator } | null>(null);
  let blockSelection = $state<NotesBlockSelectionState | null>(null);
  let selectionDragAnchorBlockId = $state<string | null>(null);
  let selectionDragPointerId = $state<number | null>(null);
  let selectionClipboard = $state<NotesBlockSelectionClipboard | null>(null);
  let selectionBusy = $state(false);
  let selectionActionError = $state<string | null>(null);
  const selectedBlockCount = $derived(blockSelection?.selectedBlockIds.length ?? 0);
  const selectedRootBlockIds = $derived(
    blockSelection
      ? notesSelectionRootBlockIds(currentTreeState(), blockSelection.selectedBlockIds)
      : [],
  );
  const canMoveSelectionUp = $derived(
    !!planNotesSelectionMoveWithinSiblings(currentTreeState(), selectedRootBlockIds, "up"),
  );
  const canMoveSelectionDown = $derived(
    !!planNotesSelectionMoveWithinSiblings(currentTreeState(), selectedRootBlockIds, "down"),
  );
  const mentionTargets: NotesPageMentionTarget[] = $derived(
    notes.pages.map((page) => {
      const parentPageId = page.parent.type === "page_id" ? page.parent.page_id : null;
      const parentPage = parentPageId
        ? notes.pages.find((candidate) => candidate.id === parentPageId)
        : null;
      const parentTitle = parentPage
        ? notesPageTitle(parentPage, t("notes.untitled"))
        : t("notes.workspace");
      return {
        kind: "page",
        id: page.id,
        title: notesPageTitle(page, t("notes.untitled")),
        subtitle: parentTitle,
        iconText: notesPageIconText(page.icon),
      };
    }),
  );

  $effect(() => {
    if (typeof window === "undefined" || selectionDragPointerId === null) return;
    const stopSelectionDrag = () => {
      selectionDragAnchorBlockId = null;
      selectionDragPointerId = null;
    };
    window.addEventListener("pointerup", stopSelectionDrag);
    window.addEventListener("pointercancel", stopSelectionDrag);
    return () => {
      window.removeEventListener("pointerup", stopSelectionDrag);
      window.removeEventListener("pointercancel", stopSelectionDrag);
    };
  });

  $effect(() => {
    const _pageId = pageId;
    const _itemCount = items.length;
    const _selection = blockSelection;
    void tick().then(() => {
      pruneAndSyncBlockSelection();
    });
  });

  function renderedSelectableBlockIds(): string[] {
    if (!blockListElement) return items.map((item) => item.block.id);
    return normalizeNotesSelectableBlockIds(
      Array.from(blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]"))
        .map((element) => element.dataset.notesSelectableBlockId ?? ""),
    );
  }

  function currentTreeState() {
    return {
      blocksById: notes.blocksById,
      childIdsByParentId: notes.childIdsByParentId,
    };
  }

  function templateStatusForBlock(blockId: string) {
    return notesTemplateBlockStatus(currentTreeState(), blockId);
  }

  function buttonStatusForBlock(blockId: string) {
    return notesButtonBlockStatus(currentTreeState(), blockId);
  }

  function selectableBlockRowFromEvent(event: Event): HTMLElement | null {
    const target = event.target;
    if (!(target instanceof Element) || !blockListElement) return null;
    const row = target.closest<HTMLElement>("[data-notes-selectable-block-id]");
    if (!row || !blockListElement.contains(row)) return null;
    return row;
  }

  function selectableBlockIdFromEvent(event: Event): string | null {
    return selectableBlockRowFromEvent(event)?.dataset.notesSelectableBlockId ?? null;
  }

  function eventTargetIsEditable(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    return target.closest("input, textarea, select, button, a, [contenteditable='true'], [role='textbox']")
      !== null;
  }

  function eventTargetIsSelectionZone(target: EventTarget | null): boolean {
    return target instanceof Element && target.closest("[data-notes-block-selection-zone]") !== null;
  }

  function syncBlockSelectionAttributes(): void {
    if (!blockListElement) return;
    for (const row of blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]")) {
      const blockId = row.dataset.notesSelectableBlockId ?? "";
      const selected = notesBlockSelectionContains(blockSelection, blockId);
      row.toggleAttribute("data-notes-block-selected", selected);
    }
  }

  function pruneAndSyncBlockSelection(): void {
    const pruned = notesBlockSelectionPrunedToVisible(renderedSelectableBlockIds(), blockSelection);
    if (!sameBlockSelection(blockSelection, pruned)) {
      blockSelection = pruned;
      return;
    }
    syncBlockSelectionAttributes();
  }

  function sameBlockSelection(
    left: NotesBlockSelectionState | null,
    right: NotesBlockSelectionState | null,
  ): boolean {
    if (left === right) return true;
    if (!left || !right) return false;
    return (
      left.anchorBlockId === right.anchorBlockId
      && left.focusBlockId === right.focusBlockId
      && left.selectedBlockIds.length === right.selectedBlockIds.length
      && left.selectedBlockIds.every((blockId, index) => right.selectedBlockIds[index] === blockId)
    );
  }

  function setBlockSelection(selection: NotesBlockSelectionState | null): void {
    blockSelection = selection;
    void tick().then(syncBlockSelectionAttributes);
  }

  function focusSelectedBlockRow(blockId: string, preventScroll = true): void {
    void tick().then(() => {
      selectableBlockRowFromBlockId(blockId)?.focus({ preventScroll });
    });
  }

  function selectableBlockRowFromBlockId(blockId: string): HTMLElement | null {
    if (!blockListElement) return null;
    return Array.from(blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]"))
      .find((element) => element.dataset.notesSelectableBlockId === blockId) ?? null;
  }

  function focusTextEditorForBlock(blockId: string): boolean {
    const block = notes.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type)) return false;

    const editor = selectableBlockRowFromBlockId(blockId)
      ?.querySelector<HTMLElement>("[contenteditable='true'][role='textbox']") ?? null;
    if (!editor) return false;

    editor.focus({ preventScroll: true });
    const textLength = notesPlainTextFromEditableRoot(editor).length;
    restoreNotesEditableSelection(editor, { start: textLength, end: textLength });
    return true;
  }

  function clearNativeSelection(): void {
    if (typeof window === "undefined") return;
    window.getSelection()?.removeAllRanges();
  }

  function selectBlockFromPointer(blockId: string, extend: boolean): void {
    const selection = notesBlockSelectionAfterClick({
      blockIds: renderedSelectableBlockIds(),
      current: blockSelection,
      blockId,
      extend,
    });
    setBlockSelection(selection);
    if (selection) focusSelectedBlockRow(selection.focusBlockId);
  }

  function handleBlockSelectionPointerDown(event: PointerEvent): void {
    if (event.button !== 0) return;
    const blockId = selectableBlockIdFromEvent(event);
    if (!blockId) {
      if (blockSelection) setBlockSelection(null);
      return;
    }
    const shouldExtend = event.shiftKey && blockSelection !== null;
    if (shouldExtend) {
      event.preventDefault();
      clearNativeSelection();
      selectBlockFromPointer(blockId, true);
      return;
    }
    if (!eventTargetIsSelectionZone(event.target)) {
      if (blockSelection) setBlockSelection(null);
      return;
    }
    if (eventTargetIsEditable(event.target)) return;
    if (focusTextEditorForBlock(blockId)) {
      event.preventDefault();
      setBlockSelection(null);
      return;
    }
    event.preventDefault();
    clearNativeSelection();
    selectionDragAnchorBlockId = blockId;
    selectionDragPointerId = event.pointerId;
    setBlockSelection(notesBlockSelectionForBlock(renderedSelectableBlockIds(), blockId));
    focusSelectedBlockRow(blockId);
  }

  function handleBlockSelectionPointerOver(event: PointerEvent): void {
    if (selectionDragPointerId === null || event.pointerId !== selectionDragPointerId) return;
    const anchorBlockId = selectionDragAnchorBlockId;
    const blockId = selectableBlockIdFromEvent(event);
    if (!anchorBlockId || !blockId) return;
    const selection = notesBlockSelectionRange(renderedSelectableBlockIds(), anchorBlockId, blockId);
    setBlockSelection(selection);
  }

  function handleBlockListKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    const blockId = selectableBlockIdFromEvent(event);
    if (!blockId) return;
    if (handleBlockSelectionShortcut(event, blockId)) return;
    if (event.key === "Escape" && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      clearNativeSelection();
      setBlockSelection(notesBlockSelectionForBlock(renderedSelectableBlockIds(), blockId));
      focusSelectedBlockRow(blockId);
      return;
    }
    if (
      event.shiftKey
      && !event.ctrlKey
      && !event.metaKey
      && !event.altKey
      && (event.key === "ArrowDown" || event.key === "ArrowUp")
    ) {
      if (!blockSelection && eventTargetIsEditable(event.target)) return;
      event.preventDefault();
      clearNativeSelection();
      const selection = notesBlockSelectionAfterKeyboard({
        blockIds: renderedSelectableBlockIds(),
        current: blockSelection,
        focusedBlockId: blockId,
        direction: event.key === "ArrowDown" ? "next" : "previous",
      });
      setBlockSelection(selection);
      if (selection) focusSelectedBlockRow(selection.focusBlockId, false);
    }
  }

  function handleBlockSelectionShortcut(event: KeyboardEvent, blockId: string): boolean {
    const hasModifier = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    if (
      selectionClipboard
      && hasModifier
      && !event.shiftKey
      && !event.altKey
      && key === "v"
      && !eventTargetIsEditable(event.target)
    ) {
      event.preventDefault();
      void runSelectionAction(() => pasteSelectionClipboard(blockSelection?.focusBlockId ?? blockId));
      return true;
    }
    if (!blockSelection) return false;
    if (!event.altKey && (event.key === "Backspace" || event.key === "Delete")) {
      event.preventDefault();
      void runSelectionAction(deleteCurrentBlockSelection);
      return true;
    }
    if (hasModifier && event.shiftKey && !event.altKey && event.key === "ArrowUp") {
      event.preventDefault();
      void runSelectionAction(() => moveCurrentBlockSelection("up"));
      return true;
    }
    if (hasModifier && event.shiftKey && !event.altKey && event.key === "ArrowDown") {
      event.preventDefault();
      void runSelectionAction(() => moveCurrentBlockSelection("down"));
      return true;
    }
    if (!hasModifier || event.shiftKey || event.altKey) return false;
    if (key === "c") {
      event.preventDefault();
      void runSelectionAction(() => copyCurrentBlockSelection("copy"));
      return true;
    }
    if (key === "x") {
      event.preventDefault();
      void runSelectionAction(() => copyCurrentBlockSelection("cut"));
      return true;
    }
    if (key === "d") {
      event.preventDefault();
      void runSelectionAction(duplicateCurrentBlockSelection);
      return true;
    }
    return false;
  }

  async function runSelectionAction(action: () => Promise<void> | void): Promise<void> {
    if (selectionBusy) return;
    selectionBusy = true;
    selectionActionError = null;
    try {
      await action();
    } catch (error) {
      selectionActionError = error instanceof Error ? error.message : String(error);
    } finally {
      selectionBusy = false;
    }
  }

  async function copyCurrentBlockSelection(mode: "copy" | "cut"): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    const state = currentTreeState();
    const rootBlockIds = notesSelectionRootBlockIds(state, selection.selectedBlockIds);
    const subtreeBlockIds = notesSelectionSubtreeIds(state, rootBlockIds);
    if (rootBlockIds.length === 0 || subtreeBlockIds.length === 0) return;
    const plainText = notesSelectionPlainText(state, rootBlockIds);
    selectionClipboard = { mode, pageId, rootBlockIds, subtreeBlockIds, plainText };
    if (plainText && typeof navigator !== "undefined" && navigator.clipboard) {
      await navigator.clipboard.writeText(plainText).catch(() => undefined);
    }
    if (mode === "cut") {
      await notes.deleteBlockSelection(selection.selectedBlockIds);
      setBlockSelection(null);
    }
  }

  async function pasteSelectionClipboard(targetBlockId: string | null): Promise<void> {
    const clipboard = selectionClipboard;
    if (!clipboard || !targetBlockId) return;
    const focusBlockId = await notes.pasteBlockSelection(
      clipboard.rootBlockIds,
      clipboard.subtreeBlockIds,
      targetBlockId,
      clipboard.mode === "cut",
    );
    if (clipboard.mode === "cut") selectionClipboard = null;
    setBlockSelection(null);
    if (focusBlockId) focusSelectedBlockRow(focusBlockId, false);
  }

  async function duplicateCurrentBlockSelection(): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.duplicateBlockSelection(selection.selectedBlockIds);
    setBlockSelection(null);
  }

  async function moveCurrentBlockSelection(direction: "up" | "down"): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.moveBlockSelection(selection.selectedBlockIds, direction);
  }

  async function deleteCurrentBlockSelection(): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.deleteBlockSelection(selection.selectedBlockIds);
    setBlockSelection(null);
  }

  function blockSelectionDelegation(node: HTMLDivElement): { destroy: () => void } {
    node.addEventListener("pointerdown", handleBlockSelectionPointerDown);
    node.addEventListener("pointerover", handleBlockSelectionPointerOver);
    node.addEventListener("keydown", handleBlockListKeydown);
    return {
      destroy() {
        node.removeEventListener("pointerdown", handleBlockSelectionPointerDown);
        node.removeEventListener("pointerover", handleBlockSelectionPointerOver);
        node.removeEventListener("keydown", handleBlockListKeydown);
      },
    };
  }

  function handleKeyboardAction(blockId: string, action: NotesKeyboardAction): void {
    if (action.type === "create_sibling") {
      void notes.createSiblingAfter(blockId);
      return;
    }
    if (action.type === "split_text_block") {
      void splitTextBlockFromKeyboardAction(blockId, action);
      return;
    }
    if (action.type === "convert_to_paragraph") {
      void notes.convertBlock(blockId, "paragraph", true);
      return;
    }
    if (action.type === "apply_text_shortcut") {
      void notes.convertBlock(blockId, action.blockType, true);
      return;
    }
    if (action.type === "toggle_block_open") {
      const block = notes.blockById(blockId);
      if (block?.type === "toggle") {
        void notes.updateToggleOpen(blockId, block.toggle.ganbaru_open === false);
      }
      return;
    }
    if (action.type === "delete_block") {
      void notes.deleteBlock(blockId);
      return;
    }
    if (action.type === "merge_with_previous") {
      void notes.mergeBlockWithPrevious(blockId);
      return;
    }
    if (action.type === "nest") {
      void notes.nestBlock(blockId);
      return;
    }
    if (action.type === "outdent") {
      void notes.outdentBlock(blockId);
      return;
    }
    if (action.type === "move_up") {
      void notes.moveBlockUp(blockId);
      return;
    }
    if (action.type === "move_down") {
      void notes.moveBlockDown(blockId);
    }
  }

  async function splitTextBlockFromKeyboardAction(
    blockId: string,
    action: Extract<NotesKeyboardAction, { type: "split_text_block" }>,
  ): Promise<void> {
    await notes.updateBlockText(blockId, action.text);
    await notes.splitTextBlockAtSelection(blockId, action.selectionStart, action.selectionEnd);
  }

  function handleConvert(blockId: string, type: NotesBlockType, clearText = false): void {
    void notes.convertBlock(blockId, type, clearText);
  }

  function convertToToggleHeading(blockId: string, type: NotesHeadingBlockType, clearText = false): void {
    void notes.convertBlockToToggleHeading(blockId, type, clearText);
  }

  function moveTargetsForBlock(block: NotesBlock): NotesMoveToPageTarget[] {
    return notesMoveToPageTargets(notes.pages, block, pageId, t("notes.untitled"), {
      recentPageIds: notes.recentPageIds,
      excludedPageIds: loadedChildPageIdsInSubtree(block.id),
    });
  }

  function moveTargetsForBlockId(blockId: string): NotesMoveToPageTarget[] {
    const block = notes.blockById(blockId);
    return block ? moveTargetsForBlock(block) : [];
  }

  function loadedChildPageIdsInSubtree(blockId: string): string[] {
    const result: string[] = [];
    const seen = new Set<string>();
    const queue = [blockId];
    while (queue.length > 0) {
      const currentBlockId = queue.shift();
      if (!currentBlockId || seen.has(currentBlockId)) continue;
      seen.add(currentBlockId);
      const block = notes.blockById(currentBlockId);
      if (block?.type === "child_page") result.push(block.id);
      queue.push(...(notes.childIdsByParentId[currentBlockId] ?? []));
    }
    return result;
  }

  async function copyBlockLink(blockId: string): Promise<void> {
    const link = buildNotesBlockLink(window.location.href, { pageId, blockId });
    await navigator.clipboard.writeText(link);
  }

  async function insertPageMention(
    blockId: string,
    start: number,
    end: number,
    target: NotesPageMentionTarget,
  ): Promise<void> {
    await notes.insertPageMention(
      blockId,
      start,
      end,
      target.id,
      target.title,
      buildNotesPageLink(window.location.href, { pageId: target.id }),
    );
  }

  async function insertDateMention(
    blockId: string,
    start: number,
    end: number,
    target: NotesDateMentionTarget,
  ): Promise<void> {
    await notes.insertDateMention(blockId, start, end, target.date, target.title);
  }

  async function applyTextAnnotations(
    blockId: string,
    start: number,
    end: number,
    patch: NotesRichTextAnnotationPatch,
  ): Promise<void> {
    await notes.updateBlockTextAnnotations(blockId, start, end, patch);
  }

  async function applyTextLink(
    blockId: string,
    start: number,
    end: number,
    url: string | null,
  ): Promise<void> {
    await notes.updateBlockTextLink(blockId, start, end, url);
  }

  async function insertInlineEquation(
    blockId: string,
    start: number,
    end: number,
    expression: string,
  ): Promise<void> {
    await notes.insertInlineEquation(blockId, start, end, expression);
  }

  function pastePlainText(
    blockId: string,
    start: number,
    end: number,
    plainText: string,
  ): Promise<boolean> {
    return notes.pastePlainTextIntoBlock(blockId, start, end, plainText);
  }

  function pasteRichHtml(
    blockId: string,
    start: number,
    end: number,
    html: string,
  ): Promise<boolean> {
    return notes.pasteRichHtmlIntoBlock(blockId, start, end, html);
  }

  function replaceBlockRichText(
    blockId: string,
    richText: readonly NotesRichText[],
  ): void {
    void notes.updateBlockRichText(blockId, richText);
  }

  function replaceTableCellRichText(
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ): void {
    void notes.updateTableCellRichText(rowBlockId, columnIndex, richText);
  }

  function addTableRow(tableBlockId: string, afterRowIndex: number): Promise<void> {
    return notes.addTableRow(tableBlockId, afterRowIndex);
  }

  function removeTableRow(tableBlockId: string, rowBlockId: string): Promise<void> {
    return notes.removeTableRow(tableBlockId, rowBlockId);
  }

  function addTableColumn(tableBlockId: string, afterColumnIndex: number): Promise<void> {
    return notes.addTableColumn(tableBlockId, afterColumnIndex);
  }

  function removeTableColumn(tableBlockId: string, columnIndex: number): Promise<void> {
    return notes.removeTableColumn(tableBlockId, columnIndex);
  }

  function addColumn(columnListBlockId: string, afterColumnIndex: number): Promise<void> {
    return notes.addColumn(columnListBlockId, afterColumnIndex);
  }

  function removeColumn(columnListBlockId: string, columnBlockId: string): Promise<void> {
    return notes.removeColumn(columnListBlockId, columnBlockId);
  }

  function moveColumn(
    columnListBlockId: string,
    columnBlockId: string,
    direction: "left" | "right",
  ): Promise<void> {
    return notes.moveColumn(columnListBlockId, columnBlockId, direction);
  }

  function resizeColumn(
    columnListBlockId: string,
    columnBlockId: string,
    widthRatio: number,
  ): Promise<void> {
    return notes.resizeColumn(columnListBlockId, columnBlockId, widthRatio);
  }

  function moveBlockToColumn(blockId: string, columnBlockId: string): Promise<void> {
    return notes.moveBlockToColumn(blockId, columnBlockId);
  }

  function updateTabLabel(labelBlockId: string, label: string): Promise<void> {
    return notes.updateTabLabel(labelBlockId, label);
  }

  function updateTabIcon(labelBlockId: string, icon: NotesIcon | null): Promise<void> {
    return notes.updateTabIcon(labelBlockId, icon);
  }

  function addTab(tabBlockId: string, afterTabIndex: number): Promise<void> {
    return notes.addTab(tabBlockId, afterTabIndex);
  }

  function removeTab(tabBlockId: string, labelBlockId: string): Promise<void> {
    return notes.removeTab(tabBlockId, labelBlockId);
  }

  function moveTab(
    tabBlockId: string,
    labelBlockId: string,
    direction: "left" | "right",
  ): Promise<void> {
    return notes.moveTab(tabBlockId, labelBlockId, direction);
  }

  function moveBlockToTab(blockId: string, labelBlockId: string): Promise<void> {
    return notes.moveBlockToTab(blockId, labelBlockId);
  }

  function undoNotesEdit(): void {
    void notes.undoNotesEdit();
  }

  function redoNotesEdit(): void {
    void notes.redoNotesEdit();
  }

  function draggedBlockIdFromEvent(event: DragEvent): string | null {
    const transferred = event.dataTransfer?.getData(NOTES_BLOCK_DRAG_MIME) ?? "";
    return transferred || draggingBlockId;
  }

  function blockDropIntentFromEvent(event: DragEvent): NotesBlockDropIntent {
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) return "after";
    const rect = target.getBoundingClientRect();
    const depth = Number.parseInt(
      getComputedStyle(target).getPropertyValue("--notes-depth").trim(),
      10,
    );
    const safeDepth = Number.isFinite(depth) ? Math.max(depth, 0) : 0;
    const yRatio = rect.height > 0 ? (event.clientY - rect.top) / rect.height : 0.5;
    if (yRatio < 0.25) return "before";
    const outdentBoundary = rect.left + safeDepth * 20 + 28;
    if (safeDepth > 0 && event.clientX < outdentBoundary) return "outdent";
    if (yRatio > 0.75) return "after";
    return "inside";
  }

  function handleBlockDragStart(blockId: string, event: DragEvent): void {
    draggingBlockId = blockId;
    setActiveNotesBlockDragId(blockId);
    dropTarget = null;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData(NOTES_BLOCK_DRAG_MIME, blockId);
      event.dataTransfer.setData("text/plain", blockId);
    }
  }

  function handleBlockDragEnd(): void {
    draggingBlockId = null;
    setActiveNotesBlockDragId(null);
    dropTarget = null;
  }

  function handleBlockDragOver(targetBlockId: string, event: DragEvent): void {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!sourceBlockId) {
      if (dropTarget?.blockId === targetBlockId) dropTarget = null;
      return;
    }
    const intent = blockDropIntentFromEvent(event);
    const plan = planNotesBlockDrop(currentTreeState(), sourceBlockId, targetBlockId, intent);
    if (!plan) {
      if (dropTarget?.blockId === targetBlockId) dropTarget = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dropTarget = { blockId: targetBlockId, intent: plan.indicator };
  }

  function handleBlockDragLeave(targetBlockId: string, event: DragEvent): void {
    const target = event.currentTarget;
    const related = event.relatedTarget;
    if (
      target instanceof HTMLElement
      && related instanceof Node
      && target.contains(related)
    ) {
      return;
    }
    if (dropTarget?.blockId === targetBlockId) dropTarget = null;
  }

  function handleBlockDrop(targetBlockId: string, event: DragEvent): void {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!sourceBlockId) {
      handleBlockDragEnd();
      return;
    }
    const intent = dropTarget?.blockId === targetBlockId
      ? dropTarget.intent
      : blockDropIntentFromEvent(event);
    const plan = planNotesBlockDrop(currentTreeState(), sourceBlockId, targetBlockId, intent);
    if (!plan) {
      handleBlockDragEnd();
      return;
    }
    event.preventDefault();
    handleBlockDragEnd();
    void notes.dropBlockOnBlock(sourceBlockId, targetBlockId, plan.indicator);
  }

  function dropPositionForBlock(blockId: string): NotesBlockDropIndicator | null {
    return dropTarget?.blockId === blockId ? dropTarget.intent : null;
  }
</script>

<div
  use:blockSelectionDelegation
  bind:this={blockListElement}
  class="flex min-w-0 flex-col gap-0.5 pb-8"
  role="group"
  aria-label={t("notes.blockList")}
>
  {#if blockSelection}
    <div
      class="sticky top-2 z-20 mb-2 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-popover/95 px-2 py-1.5 text-xs text-popover-foreground shadow-sm backdrop-blur"
      role="toolbar"
      aria-label={t("notes.selectionActions")}
    >
      <span class="mr-1 shrink-0 font-medium text-muted-foreground">
        {t("notes.selectedBlocks", selectedBlockCount)}
      </span>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.copySelection")}
        title={t("notes.copySelection")}
        onclick={() => {
          void runSelectionAction(() => copyCurrentBlockSelection("copy"));
        }}
      >
        <Copy size={14} aria-hidden="true" />
        <span>{t("notes.copySelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.cutSelection")}
        title={t("notes.cutSelection")}
        onclick={() => {
          void runSelectionAction(() => copyCurrentBlockSelection("cut"));
        }}
      >
        <Scissors size={14} aria-hidden="true" />
        <span>{t("notes.cutSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !selectionClipboard}
        aria-label={t("notes.pasteSelection")}
        title={t("notes.pasteSelection")}
        onclick={() => {
          void runSelectionAction(() => pasteSelectionClipboard(blockSelection?.focusBlockId ?? notes.focusBlockId));
        }}
      >
        <ClipboardPaste size={14} aria-hidden="true" />
        <span>{t("notes.pasteSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.duplicateSelection")}
        title={t("notes.duplicateSelection")}
        onclick={() => {
          void runSelectionAction(duplicateCurrentBlockSelection);
        }}
      >
        <CopyPlus size={14} aria-hidden="true" />
        <span>{t("notes.duplicateSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center justify-center rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !canMoveSelectionUp}
        aria-label={t("notes.moveSelectionUp")}
        title={t("notes.moveSelectionUp")}
        onclick={() => {
          void runSelectionAction(() => moveCurrentBlockSelection("up"));
        }}
      >
        <ArrowUp size={15} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center justify-center rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !canMoveSelectionDown}
        aria-label={t("notes.moveSelectionDown")}
        title={t("notes.moveSelectionDown")}
        onclick={() => {
          void runSelectionAction(() => moveCurrentBlockSelection("down"));
        }}
      >
        <ArrowDown size={15} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.deleteSelection")}
        title={t("notes.deleteSelection")}
        onclick={() => {
          void runSelectionAction(deleteCurrentBlockSelection);
        }}
      >
        <Trash2 size={14} aria-hidden="true" />
        <span>{t("notes.deleteSelection")}</span>
      </button>
      {#if selectionActionError}
        <span class="min-w-0 flex-1 truncate text-destructive" role="status">
          {t("notes.selectionActionFailed")} {selectionActionError}
        </span>
      {/if}
    </div>
  {/if}
  {#each items as item (item.block.id)}
    {#if item.block.type === "column_list"}
      <NotesColumnListBlock
        {item}
        columnItems={notes.columnItemsForBlock(item.block.id)}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRowsForBlock={notes.tableRowsForBlock}
        previousBlockType={notes.previousBlockType(item.block.id)}
        previousBlockTypeForBlock={notes.previousBlockType}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        isOnlyBlockForBlock={notes.isOnlyBlock}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        {mentionTargets}
        {templateStatusForBlock}
        {buttonStatusForBlock}
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
        onCreateInlineComment={(blockId, start, end) => {
          void notes.startInlineComment(blockId, start, end);
        }}
        onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        moveTargetsForBlock={moveTargetsForBlockId}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        {draggingBlockId}
        {dropPositionForBlock}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name) => {
          void notes.updateMedia(blockId, url, caption, name);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        onAddColumn={addColumn}
        onRemoveColumn={removeColumn}
        onMoveColumn={moveColumn}
        onResizeColumn={resizeColumn}
        onMoveBlockToColumn={moveBlockToColumn}
        {onSelectPage}
        {onFocusBlock}
      />
    {:else if item.block.type === "tab"}
      <NotesTabBlock
        {item}
        tabItems={notes.tabItemsForBlock(item.block.id)}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRowsForBlock={notes.tableRowsForBlock}
        previousBlockType={notes.previousBlockType(item.block.id)}
        previousBlockTypeForBlock={notes.previousBlockType}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        isOnlyBlockForBlock={notes.isOnlyBlock}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        {mentionTargets}
        {templateStatusForBlock}
        {buttonStatusForBlock}
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
        onCreateInlineComment={(blockId, start, end) => {
          void notes.startInlineComment(blockId, start, end);
        }}
        onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        moveTargetsForBlock={moveTargetsForBlockId}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        {draggingBlockId}
        {dropPositionForBlock}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name) => {
          void notes.updateMedia(blockId, url, caption, name);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        onUpdateTabLabel={updateTabLabel}
        onUpdateTabIcon={updateTabIcon}
        onAddTab={addTab}
        onRemoveTab={removeTab}
        onMoveTab={moveTab}
        onMoveBlockToTab={moveBlockToTab}
        {onSelectPage}
        {onFocusBlock}
      />
    {:else}
      <NotesBlockRow
        {item}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRows={notes.tableRowsForBlock(item.block.id)}
        previousBlockType={notes.previousBlockType(item.block.id)}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        {mentionTargets}
        templateStatus={templateStatusForBlock(item.block.id)}
        buttonStatus={buttonStatusForBlock(item.block.id)}
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
        onCreateInlineComment={(blockId, start, end) => {
          void notes.startInlineComment(blockId, start, end);
        }}
        onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name) => {
          void notes.updateMedia(blockId, url, caption, name);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        {onSelectPage}
        {onFocusBlock}
      />
    {/if}
  {/each}
</div>

<style>
  :global(.notes-block-row[data-notes-block-selected="true"] > .notes-block-surface) {
    background: hsl(var(--primary) / 0.12);
    box-shadow: inset 0 0 0 1px hsl(var(--primary) / 0.42);
  }

  :global(.notes-block-row[data-notes-block-selected="true"]:focus-visible > .notes-block-surface) {
    outline: 2px solid hsl(var(--ring));
    outline-offset: 1px;
  }
</style>
