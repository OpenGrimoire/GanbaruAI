<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { buildNotesBlockLink, buildNotesPageLink } from "$lib/notes/block-link";
  import { notesPageIconText } from "$lib/notes/page-icon";
  import { notesPageTitle } from "$lib/notes/page-title";
  import type { NotesHeadingBlockType } from "$lib/notes/block-factory";
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
  import type { NotesSiblingDropPosition } from "$lib/notes/block-tree";
  import type {
    NotesBlock,
    NotesBlockTreeItem,
    NotesBlockType,
    NotesPageBreadcrumbItem,
    NotesTableOfContentsItem,
  } from "$lib/notes/types";
  import NotesBlockRow from "./NotesBlockRow.svelte";
  import NotesColumnListBlock from "./NotesColumnListBlock.svelte";
  import NotesTabBlock from "./NotesTabBlock.svelte";

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
  const notesBlockDragMime = "application/x-ganbaru-notes-block";
  let draggingBlockId = $state<string | null>(null);
  let dropTarget = $state<{ blockId: string; position: NotesSiblingDropPosition } | null>(null);
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

  function handleKeyboardAction(blockId: string, action: NotesKeyboardAction): void {
    if (action.type === "create_sibling") {
      void notes.createSiblingAfter(blockId);
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

  function handleConvert(blockId: string, type: NotesBlockType, clearText = false): void {
    void notes.convertBlock(blockId, type, clearText);
  }

  function convertToToggleHeading(blockId: string, type: NotesHeadingBlockType): void {
    void notes.convertBlockToToggleHeading(blockId, type);
  }

  function moveTargetsForBlock(block: NotesBlock): NotesMoveToPageTarget[] {
    return notesMoveToPageTargets(notes.pages, block, pageId, t("notes.untitled"));
  }

  function moveTargetsForBlockId(blockId: string): NotesMoveToPageTarget[] {
    const block = notes.blockById(blockId);
    return block ? moveTargetsForBlock(block) : [];
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

  function insertInlineEquation(
    blockId: string,
    start: number,
    end: number,
    expression: string,
  ): void {
    void notes.insertInlineEquation(blockId, start, end, expression);
  }

  function pastePlainText(
    blockId: string,
    start: number,
    end: number,
    plainText: string,
  ): Promise<boolean> {
    return notes.pastePlainTextIntoBlock(blockId, start, end, plainText);
  }

  function blockParentId(blockId: string): string | null {
    const block = notes.blockById(blockId);
    return block ? notes.parentIdForBlock(block) : null;
  }

  function draggedBlockIdFromEvent(event: DragEvent): string | null {
    const transferred = event.dataTransfer?.getData(notesBlockDragMime) ?? "";
    return transferred || draggingBlockId;
  }

  function canDropOnBlock(sourceBlockId: string | null, targetBlockId: string): boolean {
    if (!sourceBlockId || sourceBlockId === targetBlockId) return false;
    const sourceParentId = blockParentId(sourceBlockId);
    const targetParentId = blockParentId(targetBlockId);
    return !!sourceParentId && sourceParentId === targetParentId;
  }

  function dropPositionFromEvent(event: DragEvent): NotesSiblingDropPosition {
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) return "after";
    const rect = target.getBoundingClientRect();
    return event.clientY < rect.top + rect.height / 2 ? "before" : "after";
  }

  function handleBlockDragStart(blockId: string, event: DragEvent): void {
    draggingBlockId = blockId;
    dropTarget = null;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData(notesBlockDragMime, blockId);
      event.dataTransfer.setData("text/plain", blockId);
    }
  }

  function handleBlockDragEnd(): void {
    draggingBlockId = null;
    dropTarget = null;
  }

  function handleBlockDragOver(targetBlockId: string, event: DragEvent): void {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!canDropOnBlock(sourceBlockId, targetBlockId)) {
      if (dropTarget?.blockId === targetBlockId) dropTarget = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dropTarget = { blockId: targetBlockId, position: dropPositionFromEvent(event) };
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
    if (!canDropOnBlock(sourceBlockId, targetBlockId)) {
      handleBlockDragEnd();
      return;
    }
    event.preventDefault();
    const position = dropTarget?.blockId === targetBlockId
      ? dropTarget.position
      : dropPositionFromEvent(event);
    handleBlockDragEnd();
    if (!sourceBlockId) return;
    void notes.dropBlockWithinSiblings(sourceBlockId, targetBlockId, position);
  }

  function dropPositionForBlock(blockId: string): NotesSiblingDropPosition | null {
    return dropTarget?.blockId === blockId ? dropTarget.position : null;
  }
</script>

<div class="flex min-w-0 flex-col gap-0.5 pb-8">
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
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={(blockId, start, end, url) => {
          void notes.updateBlockTextLink(blockId, start, end, url);
        }}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onApplyTextAnnotations={applyTextAnnotations}
        onKeyboardAction={handleKeyboardAction}
        onAddBelow={(blockId, type) => {
          void notes.createSiblingAfter(blockId, type);
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
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
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
        onTableCellChange={(rowBlockId, columnIndex, text) => {
          void notes.updateTableCell(rowBlockId, columnIndex, text);
        }}
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
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={(blockId, start, end, url) => {
          void notes.updateBlockTextLink(blockId, start, end, url);
        }}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onApplyTextAnnotations={applyTextAnnotations}
        onKeyboardAction={handleKeyboardAction}
        onAddBelow={(blockId, type) => {
          void notes.createSiblingAfter(blockId, type);
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
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
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
        onTableCellChange={(rowBlockId, columnIndex, text) => {
          void notes.updateTableCell(rowBlockId, columnIndex, text);
        }}
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
        onTextInput={(blockId, text) => {
          void notes.updateBlockText(blockId, text);
        }}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onApplyTextLink={(blockId, start, end, url) => {
          void notes.updateBlockTextLink(blockId, start, end, url);
        }}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onApplyTextAnnotations={applyTextAnnotations}
        onKeyboardAction={handleKeyboardAction}
        onAddBelow={(blockId, type) => {
          void notes.createSiblingAfter(blockId, type);
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
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
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
        onTableCellChange={(rowBlockId, columnIndex, text) => {
          void notes.updateTableCell(rowBlockId, columnIndex, text);
        }}
        {onSelectPage}
        {onFocusBlock}
      />
    {/if}
  {/each}
</div>
