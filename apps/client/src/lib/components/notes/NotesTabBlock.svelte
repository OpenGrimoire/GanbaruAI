<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import { blockPlainText, type NotesHeadingBlockType } from "$lib/notes/block-factory";
  import type { NotesMoveToPageTarget } from "$lib/notes/block-move";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import { notesUndoShortcutAction } from "$lib/notes/undo-history";
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
    NotesPageBreadcrumbItem,
    NotesRichText,
    NotesTabBlockItems,
    NotesTableOfContentsItem,
    NotesTableRowBlock,
  } from "$lib/notes/types";
  import NotesBlockHandle from "./NotesBlockHandle.svelte";
  import NotesBlockRow from "./NotesBlockRow.svelte";
  import NotesSlashMenu from "./NotesSlashMenu.svelte";

  let {
    item,
    tabItems,
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
    tabItems: NotesTabBlockItems[];
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
    onConvertToToggleHeading: (
      blockId: string,
      type: NotesHeadingBlockType,
      clearText?: boolean,
    ) => void;
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
  let activeTabId = $state<string | null>(null);
  const block = $derived(item.block);
  const activeTab = $derived(
    tabItems.find((tab) => tab.label.id === activeTabId) ?? tabItems[0] ?? null,
  );

  $effect(() => {
    if (tabItems.length === 0) {
      activeTabId = null;
      return;
    }
    if (!tabItems.some((tab) => tab.label.id === activeTabId)) {
      activeTabId = tabItems[0]?.label.id ?? null;
    }
  });

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => {
      focusButton?.focus();
    });
  });

  function tabLabel(tab: NotesTabBlockItems, index: number): string {
    return blockPlainText(tab.label).trim() || t("notes.tab", index + 1);
  }

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
  class="notes-block-row notes-tab-row group relative"
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
      <section class="notes-tab-layout my-1 min-w-0" aria-label={t("notes.blockType.tab")}>
        {#if tabItems.length === 0}
          <button
            bind:this={focusButton}
            type="button"
            class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onkeydown={handleKeydown}
          >
            {t("notes.emptyTabs")}
          </button>
        {:else}
          <div class="notes-tab-list" role="tablist" aria-label={t("notes.blockType.tab")}>
            {#each tabItems as tab, tabIndex (tab.label.id)}
              {@const selected = activeTab?.label.id === tab.label.id}
              <button
                type="button"
                role="tab"
                aria-selected={selected}
                class="notes-tab-trigger"
                class:notes-tab-trigger-active={selected}
                onclick={() => {
                  activeTabId = tab.label.id;
                  onFocusBlock(tab.label.id);
                }}
              >
                <span class="min-w-0 truncate">{tabLabel(tab, tabIndex)}</span>
              </button>
            {/each}
          </div>

          {#if activeTab}
            <div
              class="notes-tab-panel"
              role="tabpanel"
              aria-label={tabLabel(activeTab, tabItems.indexOf(activeTab))}
            >
              {#if activeTab.items.length === 0}
                <button
                  bind:this={focusButton}
                  type="button"
                  class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onkeydown={handleKeydown}
                >
                  {t("notes.emptyTabs")}
                </button>
              {:else}
                <div class="flex min-w-0 flex-col gap-0.5">
                  {#each activeTab.items as tabBlockItem (tabBlockItem.block.id)}
                    <NotesBlockRow
                      item={tabBlockItem}
                      {breadcrumbItems}
                      {tableOfContentsItems}
                      tableRows={tableRowsForBlock(tabBlockItem.block.id)}
                      previousBlockType={previousBlockTypeForBlock(tabBlockItem.block.id)}
                      isOnlyBlock={isOnlyBlockForBlock(tabBlockItem.block.id)}
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
                      moveTargets={moveTargetsForBlock(tabBlockItem.block.id)}
                      {onMoveToPage}
                      {onDelete}
                      isDragging={draggingBlockId === tabBlockItem.block.id}
                      dropPosition={dropPositionForBlock(tabBlockItem.block.id)}
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
            </div>
          {/if}
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

  .notes-tab-layout {
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

  .notes-tab-list {
    display: flex;
    min-width: 0;
    gap: 0.25rem;
    overflow-x: auto;
    border-bottom: 1px solid var(--border);
  }

  .notes-tab-trigger {
    display: inline-flex;
    max-width: 12rem;
    min-height: 1.9rem;
    min-width: 0;
    flex-shrink: 0;
    align-items: center;
    border-bottom: 2px solid transparent;
    padding: 0 0.55rem;
    color: hsl(var(--muted-foreground));
    font-size: 0.8rem;
    font-weight: 500;
    outline: none;
  }

  .notes-tab-trigger:hover,
  .notes-tab-trigger:focus-visible {
    background: hsl(var(--accent));
    color: hsl(var(--accent-foreground));
  }

  .notes-tab-trigger-active {
    border-bottom-color: hsl(var(--primary));
    color: hsl(var(--foreground));
  }

  .notes-tab-panel {
    min-width: 0;
    padding-top: 0.45rem;
  }

  @container (max-width: 28rem) {
    .notes-tab-trigger {
      max-width: 9rem;
      padding: 0 0.45rem;
    }
  }
</style>
