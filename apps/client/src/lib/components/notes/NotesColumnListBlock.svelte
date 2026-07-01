<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import type { NotesMoveToPageTarget } from "$lib/notes/block-move";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import { notesUndoShortcutAction } from "$lib/notes/undo-history";
  import type { NotesHeadingBlockType } from "$lib/notes/block-factory";
  import type { NotesSlashAction, NotesSlashCommand } from "$lib/notes/slash-commands";
  import type {
    NotesDateMentionTarget,
    NotesPageMentionTarget,
    NotesRichTextAnnotationPatch,
  } from "$lib/notes/rich-text";
  import type { NotesSiblingDropPosition } from "$lib/notes/block-tree";
  import type {
    NotesBlockTreeItem,
    NotesBlockType,
    NotesColor,
    NotesColumnBlockItems,
    NotesPageBreadcrumbItem,
    NotesRichText,
    NotesTableOfContentsItem,
    NotesTableRowBlock,
  } from "$lib/notes/types";
  import NotesBlockHandle from "./NotesBlockHandle.svelte";
  import NotesBlockRow from "./NotesBlockRow.svelte";
  import NotesSlashMenu from "./NotesSlashMenu.svelte";

  let {
    item,
    columnItems,
    breadcrumbItems,
    tableOfContentsItems,
    tableRowsForBlock,
    previousBlockType,
    previousBlockTypeForBlock,
    isOnlyBlock,
    isOnlyBlockForBlock,
    focusBlockId,
    focusRequestId,
    mentionTargets,
    onTextInput,
    onReplaceRichText,
    onInsertPageMention,
    onInsertDateMention,
    onApplyTextLink,
    onInsertInlineEquation,
    onPastePlainText,
    onPasteRichHtml,
    onApplyTextAnnotations,
    onKeyboardAction,
    onUndo,
    onRedo,
    onAddBelow,
    onConvert,
    onConvertToToggleHeading,
    onColorChange,
    onCopyLink,
    onDuplicate,
    onUseTemplate,
    onUseButton,
    onComment,
    onMoveUp,
    onMoveDown,
    moveTargets,
    moveTargetsForBlock,
    onMoveToPage,
    onDelete,
    isDragging,
    dropPosition,
    draggingBlockId,
    dropPositionForBlock,
    onDragStart,
    onDragEnd,
    onDragOver,
    onDragLeave,
    onDrop,
    onToggleTodo,
    onToggleOpen,
    onCodeLanguageChange,
    onBookmarkChange,
    onLinkPreviewUrlChange,
    onEmbedUrlChange,
    onEquationExpressionChange,
    onMediaChange,
    onTableCellChange,
    onSelectPage,
    onFocusBlock,
  }: {
    item: NotesBlockTreeItem;
    columnItems: NotesColumnBlockItems[];
    breadcrumbItems: NotesPageBreadcrumbItem[];
    tableOfContentsItems: NotesTableOfContentsItem[];
    tableRowsForBlock: (blockId: string) => NotesTableRowBlock[];
    previousBlockType: NotesBlockType | null;
    previousBlockTypeForBlock: (blockId: string) => NotesBlockType | null;
    isOnlyBlock: boolean;
    isOnlyBlockForBlock: (blockId: string) => boolean;
    focusBlockId: string | null;
    focusRequestId: number;
    mentionTargets: NotesPageMentionTarget[];
    onTextInput: (blockId: string, text: string) => void;
    onReplaceRichText: (
      blockId: string,
      richText: readonly NotesRichText[],
    ) => Promise<void> | void;
    onInsertPageMention: (
      blockId: string,
      start: number,
      end: number,
      target: NotesPageMentionTarget,
    ) => Promise<void> | void;
    onInsertDateMention: (
      blockId: string,
      start: number,
      end: number,
      target: NotesDateMentionTarget,
    ) => Promise<void> | void;
    onApplyTextLink: (
      blockId: string,
      start: number,
      end: number,
      url: string | null,
    ) => Promise<void> | void;
    onInsertInlineEquation: (
      blockId: string,
      start: number,
      end: number,
      expression: string,
    ) => Promise<void> | void;
    onPastePlainText: (
      blockId: string,
      start: number,
      end: number,
      plainText: string,
    ) => Promise<boolean> | boolean;
    onPasteRichHtml: (
      blockId: string,
      start: number,
      end: number,
      html: string,
    ) => Promise<boolean> | boolean;
    onApplyTextAnnotations: (
      blockId: string,
      start: number,
      end: number,
      patch: NotesRichTextAnnotationPatch,
    ) => Promise<void> | void;
    onKeyboardAction: (blockId: string, action: NotesKeyboardAction) => void;
    onUndo: () => Promise<void> | void;
    onRedo: () => Promise<void> | void;
    onAddBelow: (blockId: string, type?: NotesBlockType) => void;
    onConvert: (blockId: string, type: NotesBlockType, clearText?: boolean) => void;
    onConvertToToggleHeading: (blockId: string, type: NotesHeadingBlockType) => void;
    onColorChange: (blockId: string, color: NotesColor) => void;
    onCopyLink: (blockId: string) => Promise<void> | void;
    onDuplicate: (blockId: string) => void;
    onUseTemplate: (blockId: string) => void;
    onUseButton: (blockId: string) => void;
    onComment: (blockId: string) => void;
    onMoveUp: (blockId: string) => void;
    onMoveDown: (blockId: string) => void;
    moveTargets: NotesMoveToPageTarget[];
    moveTargetsForBlock: (blockId: string) => NotesMoveToPageTarget[];
    onMoveToPage: (blockId: string, pageId: string) => void;
    onDelete: (blockId: string) => void;
    isDragging: boolean;
    dropPosition: NotesSiblingDropPosition | null;
    draggingBlockId: string | null;
    dropPositionForBlock: (blockId: string) => NotesSiblingDropPosition | null;
    onDragStart: (blockId: string, event: DragEvent) => void;
    onDragEnd: () => void;
    onDragOver: (blockId: string, event: DragEvent) => void;
    onDragLeave: (blockId: string, event: DragEvent) => void;
    onDrop: (blockId: string, event: DragEvent) => void;
    onToggleTodo: (blockId: string, checked: boolean) => void;
    onToggleOpen: (blockId: string, open: boolean) => void;
    onCodeLanguageChange: (blockId: string, language: string) => void;
    onBookmarkChange: (blockId: string, url: string, caption: string) => void;
    onLinkPreviewUrlChange: (blockId: string, url: string) => void;
    onEmbedUrlChange: (blockId: string, url: string) => void;
    onEquationExpressionChange: (blockId: string, expression: string) => void;
    onMediaChange: (blockId: string, url: string, caption: string, name?: string) => void;
    onTableCellChange: (rowBlockId: string, columnIndex: number, text: string) => void;
    onSelectPage: (pageId: string) => void;
    onFocusBlock: (blockId: string) => void;
  } = $props();

  const { t } = getLocalization();
  let focusButton: HTMLButtonElement | null = $state(null);
  let slashOpen = $state(false);
  const block = $derived(item.block);
  const columnTemplate = $derived(
    columnItems.length > 0
      ? columnItems
        .map(({ column }) => `${column.column.width_ratio ?? 1}fr`)
        .join(" ")
      : "1fr",
  );

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => {
      focusButton?.focus();
    });
  });

  function handleKeydown(event: KeyboardEvent): void {
    const undoAction = notesUndoShortcutAction(event);
    if (undoAction) {
      event.preventDefault();
      void Promise.resolve(undoAction === "undo" ? onUndo() : onRedo());
      return;
    }
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: "",
      selectionStart: 0,
      selectionEnd: 0,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (action.type === "none" || action.type === "insert_newline") return;
    if (action.type === "open_slash_menu") {
      slashOpen = true;
      return;
    }
    if (action.preventDefault) event.preventDefault();
    slashOpen = false;
    onKeyboardAction(block.id, action);
  }

  function selectSlashCommand(command: NotesSlashCommand): void {
    slashOpen = false;
    switch (command.kind) {
      case "block":
        onConvert(block.id, command.blockType, true);
        return;
      case "toggle_heading":
        onConvertToToggleHeading(block.id, command.headingType);
        return;
      case "action":
        runSlashAction(command.action);
        return;
      case "color":
        return;
    }
  }

  function runSlashAction(action: NotesSlashAction): void {
    switch (action) {
      case "copy_link":
        void Promise.resolve(onCopyLink(block.id)).catch((error) => {
          console.warn("copy notes block link failed", error);
        });
        return;
      case "duplicate":
        onDuplicate(block.id);
        return;
      case "move_up":
        onMoveUp(block.id);
        return;
      case "move_down":
        onMoveDown(block.id);
        return;
      case "delete":
        onDelete(block.id);
        return;
    }
  }

  function openTurnIntoMenu(): void {
    slashOpen = true;
  }
</script>

<div
  id={notesBlockAnchorId(block.id)}
  class="notes-block-row notes-column-list-row group relative"
  role="group"
  class:notes-block-focused={focusBlockId === block.id}
  class:notes-block-dragging={isDragging}
  class:notes-block-drop-before={dropPosition === "before"}
  class:notes-block-drop-after={dropPosition === "after"}
  style={`--notes-depth: ${Math.min(item.depth, 8)}`}
  ondragover={(event) => onDragOver(block.id, event)}
  ondragleave={(event) => onDragLeave(block.id, event)}
  ondrop={(event) => onDrop(block.id, event)}
>
  <div
    class="notes-block-surface flex min-w-0 items-start gap-1 rounded-md py-0.5 pr-2 hover:bg-accent/50"
  >
    <div class="notes-block-indent shrink-0"></div>
    <NotesBlockHandle
      onAddBelow={(type) => onAddBelow(block.id, type)}
      onTurnInto={openTurnIntoMenu}
      canSetColor={false}
      currentColor="default"
      onColorSelect={() => undefined}
      onCopyLink={() => onCopyLink(block.id)}
      onDuplicate={() => onDuplicate(block.id)}
      onComment={() => onComment(block.id)}
      onMoveUp={() => onMoveUp(block.id)}
      onMoveDown={() => onMoveDown(block.id)}
      {moveTargets}
      onMoveToPage={(pageId) => onMoveToPage(block.id, pageId)}
      onDelete={() => onDelete(block.id)}
      onDragStart={(event) => onDragStart(block.id, event)}
      onDragEnd={onDragEnd}
    />
    <div class="w-5 shrink-0"></div>

    <div class="relative min-w-0 flex-1">
      <section class="notes-column-layout my-1 min-w-0" aria-label={t("notes.blockType.columns")}>
        {#if columnItems.length === 0}
          <button
            bind:this={focusButton}
            type="button"
            class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onkeydown={handleKeydown}
          >
            {t("notes.emptyColumns")}
          </button>
        {:else}
          <div class="notes-column-grid" style={`--notes-column-template: ${columnTemplate}`}>
            {#each columnItems as columnItem, columnIndex (columnItem.column.id)}
              <section
                class="notes-column min-w-0"
                aria-label={t("notes.column", columnIndex + 1)}
              >
                {#if columnItem.items.length === 0}
                  <button
                    bind:this={focusButton}
                    type="button"
                    class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    onkeydown={handleKeydown}
                  >
                    {t("notes.emptyColumns")}
                  </button>
                {:else}
                  <div class="flex min-w-0 flex-col gap-0.5">
                    {#each columnItem.items as columnBlockItem (columnBlockItem.block.id)}
                      <NotesBlockRow
                        item={columnBlockItem}
                        {breadcrumbItems}
                        {tableOfContentsItems}
                        tableRows={tableRowsForBlock(columnBlockItem.block.id)}
                        previousBlockType={previousBlockTypeForBlock(columnBlockItem.block.id)}
                        isOnlyBlock={isOnlyBlockForBlock(columnBlockItem.block.id)}
                        {focusBlockId}
                        {focusRequestId}
                        {mentionTargets}
                        {onTextInput}
                        {onReplaceRichText}
                        {onInsertPageMention}
                        {onInsertDateMention}
                        {onApplyTextLink}
                        {onInsertInlineEquation}
                        {onPastePlainText}
                        {onPasteRichHtml}
                        {onApplyTextAnnotations}
                        {onKeyboardAction}
                        {onUndo}
                        {onRedo}
                        {onAddBelow}
                        {onConvert}
                        {onConvertToToggleHeading}
                        {onColorChange}
                        {onCopyLink}
                        {onDuplicate}
                        {onUseTemplate}
                        {onUseButton}
                        {onComment}
                        {onMoveUp}
                        {onMoveDown}
                        moveTargets={moveTargetsForBlock(columnBlockItem.block.id)}
                        {onMoveToPage}
                        {onDelete}
                        isDragging={draggingBlockId === columnBlockItem.block.id}
                        dropPosition={dropPositionForBlock(columnBlockItem.block.id)}
                        {onDragStart}
                        {onDragEnd}
                        {onDragOver}
                        {onDragLeave}
                        {onDrop}
                        {onToggleTodo}
                        {onToggleOpen}
                        {onCodeLanguageChange}
                        {onBookmarkChange}
                        {onLinkPreviewUrlChange}
                        {onEmbedUrlChange}
                        {onEquationExpressionChange}
                        {onMediaChange}
                        {onTableCellChange}
                        {onSelectPage}
                        {onFocusBlock}
                      />
                    {/each}
                  </div>
                {/if}
              </section>
            {/each}
          </div>
        {/if}
      </section>

      {#if slashOpen}
        <NotesSlashMenu canSetColor={false} onSelect={selectSlashCommand} />
      {/if}
    </div>
  </div>
</div>

<style>
  .notes-block-indent {
    width: calc(var(--notes-depth) * 1.25rem);
  }

  .notes-column-layout {
    container-type: inline-size;
  }

  .notes-block-focused > .notes-block-surface {
    outline: 1px solid hsl(var(--ring) / 0.55);
    outline-offset: 1px;
  }

  .notes-block-dragging {
    opacity: 0.45;
  }

  .notes-block-drop-before::before,
  .notes-block-drop-after::after {
    position: absolute;
    left: calc(var(--notes-depth) * 1.25rem + 2.75rem);
    right: 0.5rem;
    z-index: 5;
    height: 2px;
    border-radius: 999px;
    background: hsl(var(--primary));
    content: "";
  }

  .notes-block-drop-before::before {
    top: -1px;
  }

  .notes-block-drop-after::after {
    bottom: -1px;
  }

  .notes-column-grid {
    display: grid;
    grid-template-columns: var(--notes-column-template);
    gap: 0.75rem;
  }

  .notes-column {
    border-left: 1px solid var(--border);
    padding-left: 0.5rem;
  }

  @container (max-width: 30rem) {
    .notes-column-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
