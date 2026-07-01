<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    unsupportedBlockTypeName,
    unsupportedBlockWarnings,
  } from "$lib/notes/unsupported";
  import {
    blockPlainText,
    headingIsToggleable,
    headingToggleOpen,
    isHeadingBlockType,
    isTextEditableBlock,
    type NotesHeadingBlockType,
  } from "$lib/notes/block-factory";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import { notesSyncedBlockStatus } from "$lib/notes/synced-block";
  import type { NotesTemplateBlockStatus } from "$lib/notes/template-block";
  import {
    type NotesDateMentionTarget,
    type NotesRichTextAnnotationPatch,
    type NotesPageMentionTarget,
  } from "$lib/notes/rich-text";
  import type { NotesMoveToPageTarget } from "$lib/notes/block-move";
  import {
    blockColor,
    canBlockHaveColor,
    notesBlockColorStyle,
  } from "$lib/notes/block-color";
  import type { NotesBlockInsertRequest } from "$lib/notes/block-insertion";
  import { notesBlockMarker } from "$lib/notes/block-editor-ui";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import { notesUndoShortcutAction } from "$lib/notes/undo-history";
  import type { NotesSlashAction, NotesSlashCommand } from "$lib/notes/slash-commands";
  import type { NotesBlockDropIndicator } from "$lib/notes/block-drag";
  import type {
    NotesBlockTreeItem,
    NotesBlockType,
    NotesColor,
    NotesPageBreadcrumbItem,
    NotesRichText,
    NotesTableRowBlock,
    NotesTableOfContentsItem,
  } from "$lib/notes/types";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FileText from "@lucide/svelte/icons/file-text";
  import Info from "@lucide/svelte/icons/info";
  import CircleHelp from "@lucide/svelte/icons/circle-help";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Database from "@lucide/svelte/icons/database";
  import NotesMediaBlock from "./NotesMediaBlock.svelte";
  import NotesBlockHandle from "./NotesBlockHandle.svelte";
  import NotesCardBlock from "./NotesCardBlock.svelte";
  import NotesSlashMenu from "./NotesSlashMenu.svelte";
  import NotesTableBlock from "./NotesTableBlock.svelte";
  import NotesTextBlockEditor from "./NotesTextBlockEditor.svelte";

  let {
    item,
    breadcrumbItems,
    tableOfContentsItems,
    tableRows,
    previousBlockType,
    isOnlyBlock,
    focusBlockId,
    focusRequestId,
    mentionTargets,
    templateStatus,
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
    onAddTemplateChild,
    onUseButton,
    onComment,
    onMoveUp,
    onMoveDown,
    moveTargets,
    onMoveToPage,
    onDelete,
    isDragging,
    dropPosition,
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
    onTableCellRichTextChange,
    onAddTableRow,
    onRemoveTableRow,
    onAddTableColumn,
    onRemoveTableColumn,
    onSelectPage,
    onFocusBlock,
  }: {
    item: NotesBlockTreeItem;
    breadcrumbItems: NotesPageBreadcrumbItem[];
    tableOfContentsItems: NotesTableOfContentsItem[];
    tableRows: NotesTableRowBlock[];
    previousBlockType: NotesBlockType | null;
    isOnlyBlock: boolean;
    focusBlockId: string | null;
    focusRequestId: number;
    mentionTargets: NotesPageMentionTarget[];
    templateStatus: NotesTemplateBlockStatus;
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
    onAddBelow: (blockId: string, request?: NotesBlockInsertRequest) => void;
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
    onAddTemplateChild: (blockId: string) => void;
    onUseButton: (blockId: string) => void;
    onComment: (blockId: string) => void;
    onMoveUp: (blockId: string) => void;
    onMoveDown: (blockId: string) => void;
    moveTargets: NotesMoveToPageTarget[];
    onMoveToPage: (blockId: string, pageId: string) => void;
    onDelete: (blockId: string) => void;
    isDragging: boolean;
    dropPosition: NotesBlockDropIndicator | null;
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
    onTableCellRichTextChange: (
      rowBlockId: string,
      columnIndex: number,
      richText: readonly NotesRichText[],
    ) => Promise<void> | void;
    onAddTableRow: (tableBlockId: string, afterRowIndex: number) => Promise<void> | void;
    onRemoveTableRow: (tableBlockId: string, rowBlockId: string) => Promise<void> | void;
    onAddTableColumn: (tableBlockId: string, afterColumnIndex: number) => Promise<void> | void;
    onRemoveTableColumn: (tableBlockId: string, columnIndex: number) => Promise<void> | void;
    onSelectPage: (pageId: string) => void;
    onFocusBlock: (blockId: string) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;

  function breadcrumbStatusLabel(crumb: NotesPageBreadcrumbItem): string | null {
    switch (crumb.status) {
      case "archived":
        return t("notes.breadcrumbArchived");
      case "trashed":
        return t("notes.breadcrumbTrashed");
      case "missing":
        return t("notes.breadcrumbMissing");
      case "workspace":
      case "active":
        return null;
    }
  }

  function breadcrumbCanNavigate(crumb: NotesPageBreadcrumbItem): boolean {
    return !!crumb.id && !crumb.current && crumb.status === "active";
  }

  let dividerButton: HTMLButtonElement | null = $state(null);
  let breadcrumbButton: HTMLButtonElement | null = $state(null);
  let tableOfContentsButton: HTMLButtonElement | null = $state(null);
  let childDatabaseButton: HTMLButtonElement | null = $state(null);
  let syncedBlockButton: HTMLButtonElement | null = $state(null);
  let unsupportedButton: HTMLButtonElement | null = $state(null);
  let slashOpen = $state(false);
  const block = $derived(item.block);
  const text = $derived(blockPlainText(block));
  const showTextEditor = $derived(isTextEditableBlock(block.type));
  const currentColor = $derived(blockColor(block));
  const blockSupportsColor = $derived(canBlockHaveColor(block.type));
  const blockSurfaceStyle = $derived(notesBlockColorStyle(currentColor));
  const toggleOpen = $derived(block.type !== "toggle" || block.toggle.ganbaru_open !== false);
  const headingToggleable = $derived(isHeadingBlockType(block.type) && headingIsToggleable(block));
  const headingOpen = $derived(!isHeadingBlockType(block.type) || headingToggleOpen(block));
  const childPageTitle = $derived(
    block.type === "child_page" ? block.child_page.title.trim() : "",
  );
  const childDatabaseTitle = $derived(
    block.type === "child_database" ? block.child_database.title.trim() : "",
  );
  const unsupportedTypeName = $derived(
    block.type === "unsupported" ? unsupportedBlockTypeName(block.unsupported) : "",
  );
  const unsupportedWarnings = $derived(
    block.type === "unsupported" ? unsupportedBlockWarnings(block.unsupported) : [],
  );
  const unsupportedHasRawPayload = $derived(
    block.type === "unsupported" && block.unsupported.raw !== undefined,
  );
  const syncedBlockSourceId = $derived(
    block.type === "synced_block" ? block.synced_block.synced_from?.block_id ?? null : null,
  );
  const syncedBlockStatus = $derived(
    block.type === "synced_block" ? notesSyncedBlockStatus(block.synced_block) : null,
  );

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    if (showTextEditor) return;
    void tick().then(() => {
      focusControl(childDatabaseButton);
      focusControl(syncedBlockButton);
      focusControl(dividerButton);
      focusControl(breadcrumbButton);
      focusControl(tableOfContentsButton);
      focusControl(unsupportedButton);
    });
  });

  function focusControl(control: HTMLElement | null): void {
    control?.focus();
  }

  function toggleButtonLabel(): string {
    if (block.type === "toggle") return toggleOpen ? t("notes.closeToggle") : t("notes.openToggle");
    if (headingToggleable) return headingOpen ? t("notes.closeToggle") : t("notes.openToggle");
    return t("notes.toggleBlock");
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (handleUndoRedoKeydown(event)) return;
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && headingToggleable) {
      event.preventDefault();
      onToggleOpen(block.id, !headingOpen);
      return;
    }
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement
      ? target
      : null;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text,
      selectionStart: input?.selectionStart ?? 0,
      selectionEnd: input?.selectionEnd ?? 0,
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

  function handleUndoRedoKeydown(event: KeyboardEvent): boolean {
    const undoAction = notesUndoShortcutAction(event);
    if (!undoAction) return false;
    event.preventDefault();
    void Promise.resolve(undoAction === "undo" ? onUndo() : onRedo());
    return true;
  }

  function handleChildPageKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter" || event.key === " ") return;
    handleKeydown(event);
  }

  function clearSlashText(): void {
    if (text.startsWith("/")) onTextInput(block.id, "");
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
        if (command.action !== "delete") clearSlashText();
        runSlashAction(command.action);
        return;
      case "color":
        if (!blockSupportsColor) return;
        clearSlashText();
        onColorChange(block.id, command.color);
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
  class="notes-block-row group relative"
  role="group"
  tabindex="-1"
  data-notes-selectable-block-id={block.id}
  class:notes-block-focused={focusBlockId === block.id}
  class:notes-block-dragging={isDragging}
  class:notes-block-drop-before={dropPosition === "before"}
  class:notes-block-drop-after={dropPosition === "after"}
  class:notes-block-drop-inside={dropPosition === "inside"}
  class:notes-block-drop-outdent={dropPosition === "outdent"}
  style={`--notes-depth: ${Math.min(item.depth, 8)}`}
  ondragover={(event) => onDragOver(block.id, event)}
  ondragleave={(event) => onDragLeave(block.id, event)}
  ondrop={(event) => onDrop(block.id, event)}
>
  <div
    class="notes-block-surface flex min-w-0 items-start gap-1 rounded-md py-0.5 pr-2 hover:bg-accent/50"
    class:notes-callout-surface={block.type === "callout"}
    data-notes-block-selection-zone
    style={blockSurfaceStyle}
  >
    <div class="notes-block-indent shrink-0"></div>
    <NotesBlockHandle
      onAddBelow={(type) => onAddBelow(block.id, type)}
      onTurnInto={openTurnIntoMenu}
      canSetColor={blockSupportsColor}
      currentColor={currentColor}
      onColorSelect={(color) => onColorChange(block.id, color)}
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
    {#if block.type === "to_do"}
      <input
        class="mt-2 size-4 shrink-0 accent-primary"
        type="checkbox"
        checked={block.to_do.checked}
        aria-label={t("notes.todoChecked")}
        onchange={(event) => {
          const target = event.currentTarget;
          onToggleTodo(block.id, target.checked);
        }}
      />
    {:else if block.type === "toggle"}
      <button
        class="mt-1.5 flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label={toggleButtonLabel()}
        aria-expanded={toggleOpen}
        onclick={() => {
          onToggleOpen(block.id, !toggleOpen);
        }}
      >
        {#if toggleOpen}
          <ChevronDown class="size-4" />
        {:else}
          <ChevronRight class="size-4" />
        {/if}
      </button>
    {:else if headingToggleable}
      <button
        class="mt-1.5 flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label={toggleButtonLabel()}
        aria-expanded={headingOpen}
        onclick={() => {
          onToggleOpen(block.id, !headingOpen);
        }}
      >
        {#if headingOpen}
          <ChevronDown class="size-4" />
        {:else}
          <ChevronRight class="size-4" />
        {/if}
      </button>
    {:else if block.type === "callout"}
      <div
        class="mt-1.5 flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground"
        aria-hidden="true"
      >
        <Info class="size-4" />
      </div>
    {:else if notesBlockMarker(block.type)}
      <div class="mt-1.5 w-5 shrink-0 text-right text-[0.866667rem] text-muted-foreground">
        {notesBlockMarker(block.type)}
      </div>
    {:else}
      <div class="w-5 shrink-0"></div>
    {/if}

    <div class="relative min-w-0 flex-1">
      {#if block.type === "divider"}
        <button
          bind:this={dividerButton}
          class="my-2 h-5 w-full rounded-sm px-1 focus-visible:outline-none"
          aria-label={t("notes.blockType.divider")}
          onkeydown={handleKeydown}
          onclick={() => onConvert(block.id, "paragraph", true)}
        >
          <span class="block border-t border-border"></span>
        </button>
      {:else if block.type === "child_page"}
        <button
          type="button"
          class="my-1 flex min-h-9 w-full min-w-0 items-center gap-2 rounded-md px-1 text-left text-[0.933333rem] font-medium text-foreground outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
          aria-label={t("notes.openChildPage", childPageTitle || t("notes.untitled"))}
          onkeydown={handleChildPageKeydown}
          onclick={() => onSelectPage(block.id)}
        >
          <FileText class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          <span class="min-w-0 truncate">{childPageTitle || t("notes.untitled")}</span>
        </button>
      {:else if block.type === "child_database"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-border bg-background/70 p-2"
          aria-label={t("notes.blockType.childDatabase")}
        >
          <div
            class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <Database class="size-4" />
          </div>
          <button
            bind:this={childDatabaseButton}
            type="button"
            class="flex min-h-8 min-w-0 flex-1 flex-col gap-0.5 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onkeydown={handleKeydown}
            onclick={() => onFocusBlock(block.id)}
          >
            <span class="min-w-0 truncate text-[0.866667rem] font-medium text-foreground">
              {childDatabaseTitle || t("notes.untitled")}
            </span>
            <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
              {t("notes.childDatabasePreserved")}
            </span>
          </button>
        </section>
      {:else if block.type === "breadcrumb"}
        <nav
          class="my-1 flex min-h-8 min-w-0 items-center gap-1 rounded-md px-1 text-[0.8rem] text-muted-foreground"
          aria-label={t("notes.blockType.breadcrumb")}
        >
          {#each breadcrumbItems as crumb, index}
            {@const statusLabel = breadcrumbStatusLabel(crumb)}
            {#if index > 0}
              <ChevronRight class="size-3.5 shrink-0" aria-hidden="true" />
            {/if}
            {#if breadcrumbCanNavigate(crumb)}
              <button
                type="button"
                class="min-w-0 truncate rounded px-1 py-0.5 text-left hover:bg-accent hover:text-foreground"
                aria-label={t("notes.openBreadcrumbPage", crumb.title)}
                onclick={() => {
                  if (crumb.id) onSelectPage(crumb.id);
                }}
              >
                {crumb.title}
              </button>
            {:else if crumb.current}
              <button
                bind:this={breadcrumbButton}
                type="button"
                class="min-w-0 truncate rounded px-1 py-0.5 text-left text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                aria-current="page"
                onkeydown={handleKeydown}
              >
                {crumb.title}
              </button>
            {:else}
              <span
                class="inline-flex min-w-0 items-center gap-1 truncate px-1 py-0.5"
                aria-label={statusLabel ? t("notes.breadcrumbUnavailable", crumb.title, statusLabel) : undefined}
              >
                <span class="min-w-0 truncate">{crumb.title}</span>
                {#if statusLabel}
                  <span class="shrink-0 text-[0.7rem] text-muted-foreground">
                    {statusLabel}
                  </span>
                {/if}
              </span>
            {/if}
          {/each}
        </nav>
      {:else if block.type === "table_of_contents"}
        <nav
          class="my-1 min-w-0 rounded-md px-1 py-1 text-[0.866667rem]"
          aria-label={t("notes.blockType.tableOfContents")}
        >
          {#if tableOfContentsItems.length === 0}
            <button
              bind:this={tableOfContentsButton}
              type="button"
              class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
              onkeydown={handleKeydown}
            >
              {t("notes.noHeadingsForTableOfContents")}
            </button>
          {:else}
            <ol class="flex min-w-0 flex-col gap-0.5">
              {#each tableOfContentsItems as heading, index}
                <li
                  class="notes-toc-item min-w-0"
                  style={`--notes-toc-level: ${heading.level}`}
                >
                  {#if index === 0}
                    <button
                      bind:this={tableOfContentsButton}
                      type="button"
                      class="min-h-7 w-full truncate rounded px-1 text-left text-foreground outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
                      aria-label={t("notes.openTableOfContentsHeading", heading.title)}
                      onclick={() => onFocusBlock(heading.blockId)}
                    >
                      {heading.title}
                    </button>
                  {:else}
                    <button
                      type="button"
                      class="min-h-7 w-full truncate rounded px-1 text-left text-foreground outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
                      aria-label={t("notes.openTableOfContentsHeading", heading.title)}
                      onclick={() => onFocusBlock(heading.blockId)}
                    >
                      {heading.title}
                    </button>
                  {/if}
                </li>
              {/each}
            </ol>
          {/if}
        </nav>
      {:else if block.type === "table"}
        <NotesTableBlock
          {block}
          {tableRows}
          {previousBlockType}
          {isOnlyBlock}
          {focusBlockId}
          {focusRequestId}
          {onKeyboardAction}
          {onUndo}
          {onRedo}
          {onTableCellRichTextChange}
          {onAddTableRow}
          {onRemoveTableRow}
          {onAddTableColumn}
          {onRemoveTableColumn}
        />
      {:else if block.type === "image" || block.type === "video" || block.type === "audio" || block.type === "file" || block.type === "pdf"}
        <NotesMediaBlock
          {block}
          {previousBlockType}
          {isOnlyBlock}
          {focusBlockId}
          {focusRequestId}
          {onKeyboardAction}
          {onUndo}
          {onRedo}
          {onMediaChange}
        />
      {:else if block.type === "synced_block" && syncedBlockStatus}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-dashed border-border bg-muted/30 p-2"
          aria-label={t("notes.blockType.syncedBlock")}
        >
          <div
            class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <RefreshCw class="size-4" />
          </div>
          <button
            bind:this={syncedBlockButton}
            type="button"
            class="flex min-h-8 min-w-0 flex-1 flex-col gap-0.5 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onkeydown={handleKeydown}
            onclick={() => onFocusBlock(block.id)}
          >
            <span class="text-[0.866667rem] font-medium text-foreground">
              {#if syncedBlockStatus.role === "original"}
                {t("notes.syncedBlockOriginal")}
              {:else}
                {t("notes.syncedBlockDuplicate")}
              {/if}
            </span>
            <span class="min-w-0 break-all text-[0.8rem] text-muted-foreground">
              {#if syncedBlockSourceId}
                {t("notes.syncedBlockReference", syncedBlockSourceId)}
              {:else}
                {t("notes.syncedBlockOriginalDetail")}
              {/if}
            </span>
            <span class="min-w-0 text-[0.733333rem] text-muted-foreground">
              {#if syncedBlockStatus.role === "original"}
                {t("notes.syncedBlockOriginalPreserved")}
              {:else}
                {t("notes.syncedBlockDuplicatePreserved")}
              {/if}
            </span>
            <span class="min-w-0 text-[0.733333rem] text-muted-foreground">
              {t("notes.syncedBlockFanoutUnavailable")}
            </span>
          </button>
        </section>
      {:else if block.type === "unsupported"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-dashed border-border bg-muted/30 p-2"
          aria-label={t("notes.blockType.unsupported")}
        >
          <div
            class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <CircleHelp class="size-4" />
          </div>
          <button
            bind:this={unsupportedButton}
            type="button"
            class="flex min-h-8 min-w-0 flex-1 flex-col gap-0.5 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onkeydown={handleKeydown}
            onclick={() => onFocusBlock(block.id)}
          >
            <span class="text-[0.866667rem] font-medium text-foreground">
              {t("notes.unsupportedBlockTitle")}
            </span>
            <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
              {#if unsupportedTypeName}
                {t("notes.unsupportedBlockType", unsupportedTypeName)}
              {:else}
                {t("notes.unsupportedBlockUnknownType")}
              {/if}
            </span>
            {#if unsupportedHasRawPayload}
              <span class="min-w-0 truncate text-[0.733333rem] text-muted-foreground">
                {t("notes.unsupportedBlockPayloadPreserved")}
              </span>
            {/if}
            {#if unsupportedWarnings[0]}
              <span class="min-w-0 truncate text-[0.733333rem] text-muted-foreground">
                {unsupportedWarnings[0]}
              </span>
            {/if}
          </button>
        </section>
      {:else if block.type === "bookmark" || block.type === "link_preview" || block.type === "embed" || block.type === "equation"}
        <NotesCardBlock
          {block}
          {previousBlockType}
          {isOnlyBlock}
          {focusBlockId}
          {focusRequestId}
          {onKeyboardAction}
          {onUndo}
          {onRedo}
          {onBookmarkChange}
          {onLinkPreviewUrlChange}
          {onEmbedUrlChange}
          {onEquationExpressionChange}
        />
      {:else if showTextEditor}
        <NotesTextBlockEditor
          {block}
          {previousBlockType}
          {isOnlyBlock}
          {focusBlockId}
          {focusRequestId}
          {mentionTargets}
          {templateStatus}
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
          {onConvert}
          {onConvertToToggleHeading}
          {onColorChange}
          {onCopyLink}
          {onDuplicate}
          {onUseTemplate}
          {onAddTemplateChild}
          {onUseButton}
          {onMoveUp}
          {onMoveDown}
          {onDelete}
          {onToggleOpen}
          {onCodeLanguageChange}
          {onFocusBlock}
        />
      {/if}

      {#if slashOpen}
        <NotesSlashMenu
          query={text.startsWith("/") ? text.slice(1) : ""}
          canSetColor={blockSupportsColor}
          currentColor={currentColor}
          onSelect={selectSlashCommand}
        />
      {/if}
    </div>
  </div>
</div>

<style>
  .notes-block-indent {
    width: calc(var(--notes-depth) * 1.25rem);
  }

  .notes-block-surface {
    color: var(--notes-block-color, var(--foreground));
    background: var(--notes-block-bg, transparent);
    box-shadow: inset 0 0 0 1px var(--notes-block-border, transparent);
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

  .notes-block-drop-outdent::after {
    position: absolute;
    left: max(0.5rem, calc((var(--notes-depth) - 1) * 1.25rem + 2.75rem));
    right: 0.5rem;
    bottom: -1px;
    z-index: 5;
    height: 2px;
    border-radius: 999px;
    background: hsl(var(--primary));
    content: "";
  }

  .notes-block-drop-inside > .notes-block-surface {
    background: hsl(var(--primary) / 0.1);
    box-shadow:
      inset 0 0 0 2px hsl(var(--primary) / 0.65),
      inset 0 0 0 1px var(--notes-block-border, transparent);
  }

  .notes-callout-surface {
    min-height: 2.5rem;
    padding-block: 0.35rem;
  }

  .notes-toc-item {
    padding-left: calc((var(--notes-toc-level) - 1) * 1rem);
  }

</style>
