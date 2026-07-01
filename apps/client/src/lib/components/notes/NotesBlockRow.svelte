<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { bookmarkCaptionPlainText, canOpenBookmarkUrl } from "$lib/notes/bookmark";
  import { canOpenEmbedUrl, embedDisplayTitle, embedUrlPlainText } from "$lib/notes/embed";
  import { equationExpressionPlainText, equationPreviewText } from "$lib/notes/equation";
  import {
    canOpenLinkPreviewUrl,
    linkPreviewDisplaySource,
    linkPreviewDisplayTitle,
    linkPreviewUrlPlainText,
  } from "$lib/notes/link-preview";
  import {
    unsupportedBlockTypeName,
    unsupportedBlockWarnings,
  } from "$lib/notes/unsupported";
  import {
    blockEditableRichText,
    blockHasVisibleRichTextFormatting,
    blockTextAnnotationsForSelection,
    blockTextLinkRangeForSelection,
    blockPlainText,
    headingIsToggleable,
    headingToggleOpen,
    isHeadingBlockType,
    isTextEditableBlock,
    tableCellPlainText,
    type NotesHeadingBlockType,
  } from "$lib/notes/block-factory";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import {
    buildDateMentionTargets,
    detectPageMentionQuery,
    filterPageMentionTargets,
    normalizeRichTextEquationExpression,
    normalizeRichTextLinkUrl,
    richTextAnnotationTogglePatch,
    richTextColorPatch,
    type NotesDateMentionTarget,
    type NotesRichTextAnnotationName,
    type NotesRichTextAnnotationPatch,
    type NotesMentionTarget,
    type NotesMentionQuery,
    type NotesPageMentionTarget,
  } from "$lib/notes/rich-text";
  import type { NotesMoveToPageTarget } from "$lib/notes/block-move";
  import {
    blockColor,
    canBlockHaveColor,
    notesBlockColorStyle,
  } from "$lib/notes/block-color";
  import { shouldHandleNotesPlainTextPaste } from "$lib/notes/block-clipboard";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import type { NotesSlashAction, NotesSlashCommand } from "$lib/notes/slash-commands";
  import type { NotesSiblingDropPosition } from "$lib/notes/block-tree";
  import type {
    NotesBlockTreeItem,
    NotesBlockType,
    NotesColor,
    NotesPageBreadcrumbItem,
    NotesTableRowBlock,
    NotesTableOfContentsItem,
  } from "$lib/notes/types";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FileText from "@lucide/svelte/icons/file-text";
  import Info from "@lucide/svelte/icons/info";
  import BookmarkIcon from "@lucide/svelte/icons/bookmark";
  import LinkIcon from "@lucide/svelte/icons/link";
  import Sigma from "@lucide/svelte/icons/sigma";
  import CircleHelp from "@lucide/svelte/icons/circle-help";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Database from "@lucide/svelte/icons/database";
  import Copy from "@lucide/svelte/icons/copy";
  import MousePointerClick from "@lucide/svelte/icons/mouse-pointer-click";
  import NotesMediaBlock from "./NotesMediaBlock.svelte";
  import NotesBlockHandle from "./NotesBlockHandle.svelte";
  import NotesInlineToolbar from "./NotesInlineToolbar.svelte";
  import NotesMentionMenu from "./NotesMentionMenu.svelte";
  import NotesRichTextInline from "./NotesRichTextInline.svelte";
  import NotesSlashMenu from "./NotesSlashMenu.svelte";

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
    onTextInput,
    onInsertPageMention,
    onInsertDateMention,
    onApplyTextLink,
    onInsertInlineEquation,
    onPastePlainText,
    onApplyTextAnnotations,
    onKeyboardAction,
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
    onTableCellChange,
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
    onTextInput: (blockId: string, text: string) => void;
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
    onApplyTextAnnotations: (
      blockId: string,
      start: number,
      end: number,
      patch: NotesRichTextAnnotationPatch,
    ) => Promise<void> | void;
    onKeyboardAction: (blockId: string, action: NotesKeyboardAction) => void;
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
    onMoveToPage: (blockId: string, pageId: string) => void;
    onDelete: (blockId: string) => void;
    isDragging: boolean;
    dropPosition: NotesSiblingDropPosition | null;
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

  const localization = getLocalization();
  const { t } = localization;
  const locale = $derived(localization.locale);
  let textarea: HTMLTextAreaElement | null = $state(null);
  let dividerButton: HTMLButtonElement | null = $state(null);
  let breadcrumbButton: HTMLButtonElement | null = $state(null);
  let tableOfContentsButton: HTMLButtonElement | null = $state(null);
  let bookmarkUrlInput: HTMLInputElement | null = $state(null);
  let linkPreviewUrlInput: HTMLInputElement | null = $state(null);
  let embedUrlInput: HTMLInputElement | null = $state(null);
  let equationInput: HTMLInputElement | null = $state(null);
  let firstTableCellInput: HTMLInputElement | null = $state(null);
  let childDatabaseButton: HTMLButtonElement | null = $state(null);
  let syncedBlockButton: HTMLButtonElement | null = $state(null);
  let unsupportedButton: HTMLButtonElement | null = $state(null);
  let bookmarkOpenError = $state<string | null>(null);
  let linkPreviewOpenError = $state<string | null>(null);
  let embedOpenError = $state<string | null>(null);
  let slashOpen = $state(false);
  let mentionQuery: NotesMentionQuery | null = $state(null);
  let mentionActiveIndex = $state(0);
  let textSelection = $state({ start: 0, end: 0 });
  let linkEditorOpen = $state(false);
  let linkRange = $state({ start: 0, end: 0, url: null as string | null });
  let linkUrlInput = $state("");
  let linkError = $state<string | null>(null);
  const dateMentionLabels = $derived({
    today: t("notes.dateMentionToday"),
    tomorrow: t("notes.dateMentionTomorrow"),
    yesterday: t("notes.dateMentionYesterday"),
    nextWeek: t("notes.dateMentionNextWeek"),
    date: t("notes.dateMention"),
    reminder: t("notes.reminderMention"),
    remindTitle: (dateLabel: string) => t("notes.remindOnDate", dateLabel),
  });
  const block = $derived(item.block);
  const text = $derived(blockPlainText(block));
  const textRows = $derived(Math.max(1, text.split("\n").length));
  const showTextEditor = $derived(isTextEditableBlock(block.type));
  const isBlockFocused = $derived(focusBlockId === block.id);
  const canUseMentions = $derived(
    showTextEditor && block.type !== "code",
  );
  const canUseLinks = $derived(canUseMentions);
  const canUseInlineFormatting = $derived(canUseMentions);
  const editableRichText = $derived(blockEditableRichText(block));
  const hasVisibleRichTextFormatting = $derived(blockHasVisibleRichTextFormatting(block));
  const showRichTextPreview = $derived(
    canUseInlineFormatting && hasVisibleRichTextFormatting && !isBlockFocused,
  );
  const currentTextAnnotationRange = $derived(
    blockTextAnnotationsForSelection(block, textSelection.start, textSelection.end),
  );
  const currentTextLinkRange = $derived(
    blockTextLinkRangeForSelection(block, textSelection.start, textSelection.end),
  );
  const canOpenInlineToolbar = $derived(
    canUseInlineFormatting && textSelection.start !== textSelection.end && !showRichTextPreview,
  );
  const canOpenLinkEditor = $derived(
    canUseLinks
      && (
        textSelection.start !== textSelection.end
        || currentTextLinkRange.url !== null
        || linkEditorOpen
      ),
  );
  function mentionTargetsForQuery(
    query: NotesMentionQuery | null,
    enabled: boolean,
  ): NotesMentionTarget[] {
    if (!query || !enabled) return [];
    const dateTargets = buildDateMentionTargets(query.query, {
      today: Temporal.Now.plainDateISO(),
      locale,
      labels: dateMentionLabels,
      limit: 4,
    });
    return [
      ...dateTargets,
      ...filterPageMentionTargets(mentionTargets, query.query),
    ].slice(0, 8);
  }

  const mentionMatches = $derived(mentionTargetsForQuery(mentionQuery, canUseMentions));
  const mentionOpen = $derived(mentionQuery !== null && canUseMentions);
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
  const bookmarkCaption = $derived(
    block.type === "bookmark" ? bookmarkCaptionPlainText(block.bookmark) : "",
  );
  const bookmarkCanOpen = $derived(
    block.type === "bookmark" && canOpenBookmarkUrl(block.bookmark.url),
  );
  const linkPreviewUrl = $derived(
    block.type === "link_preview" ? linkPreviewUrlPlainText(block.link_preview) : "",
  );
  const linkPreviewCanOpen = $derived(
    block.type === "link_preview" && canOpenLinkPreviewUrl(block.link_preview.url),
  );
  const linkPreviewTitle = $derived(linkPreviewDisplayTitle(linkPreviewUrl));
  const linkPreviewSource = $derived(linkPreviewDisplaySource(linkPreviewUrl));
  const embedUrl = $derived(block.type === "embed" ? embedUrlPlainText(block.embed) : "");
  const embedCanOpen = $derived(block.type === "embed" && canOpenEmbedUrl(block.embed.url));
  const embedTitle = $derived(embedDisplayTitle(embedUrl));
  const equationExpression = $derived(
    block.type === "equation" ? equationExpressionPlainText(block.equation) : "",
  );
  const equationPreview = $derived(equationPreviewText(equationExpression));
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
  const tableColumnIndexes = $derived(
    block.type === "table"
      ? Array.from({ length: Math.max(1, block.table.table_width) }, (_, index) => index)
      : [],
  );

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => {
      if (textarea) {
        textarea.focus();
        const length = textarea.value.length;
        textarea.setSelectionRange(length, length);
      } else {
        bookmarkUrlInput?.focus();
        linkPreviewUrlInput?.focus();
        embedUrlInput?.focus();
        equationInput?.focus();
        firstTableCellInput?.focus();
        childDatabaseButton?.focus();
        syncedBlockButton?.focus();
        dividerButton?.focus();
        breadcrumbButton?.focus();
        tableOfContentsButton?.focus();
        unsupportedButton?.focus();
      }
    });
  });

  $effect(() => {
    if (mentionActiveIndex >= mentionMatches.length) mentionActiveIndex = 0;
  });

  function textareaClass(type: NotesBlockType): string {
    const base = "min-h-8 w-full resize-none overflow-hidden bg-transparent px-1 py-1 outline-none placeholder:text-muted-foreground";
    if (type === "heading_1") return `${base} text-[1.45rem] font-semibold leading-tight`;
    if (type === "heading_2") return `${base} text-[1.2rem] font-semibold leading-tight`;
    if (type === "heading_3") return `${base} text-[1rem] font-semibold leading-tight`;
    if (type === "heading_4") return `${base} text-[0.933333rem] font-semibold leading-snug`;
    if (type === "code") return `${base} rounded-md bg-muted/60 font-mono text-[0.82rem] leading-relaxed`;
    if (type === "callout") return `${base} text-[0.933333rem] leading-relaxed`;
    if (type === "quote") return `${base} border-l-2 border-border pl-3 italic`;
    return `${base} text-[0.933333rem] leading-relaxed`;
  }

  function richTextPreviewClass(type: NotesBlockType): string {
    return `${textareaClass(type)} block cursor-text whitespace-pre-wrap text-left`;
  }

  function markerFor(type: NotesBlockType): string {
    if (type === "bulleted_list_item") return "•";
    if (type === "numbered_list_item") return "1.";
    if (type === "quote") return "";
    return "";
  }

  function toggleButtonLabel(): string {
    if (block.type === "toggle") return toggleOpen ? t("notes.closeToggle") : t("notes.openToggle");
    if (headingToggleable) return headingOpen ? t("notes.closeToggle") : t("notes.openToggle");
    return t("notes.toggleBlock");
  }

  async function applyTextAnnotationsToRange(
    start: number,
    end: number,
    patch: NotesRichTextAnnotationPatch,
  ): Promise<void> {
    if (start === end) return;
    await Promise.resolve(onApplyTextAnnotations(block.id, start, end, patch));
    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(start, end);
    textSelection = { start, end };
  }

  async function insertInlineEquationFromRange(
    start: number,
    end: number,
    expression: string,
  ): Promise<boolean> {
    const normalizedExpression = normalizeRichTextEquationExpression(expression);
    if (start === end || !normalizedExpression) return false;
    await Promise.resolve(onInsertInlineEquation(block.id, start, end, normalizedExpression));
    const cursor = start + normalizedExpression.length;
    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(cursor, cursor);
    textSelection = { start: cursor, end: cursor };
    return true;
  }

  function insertInlineEquationFromSelection(): void {
    void insertInlineEquationFromRange(
      currentTextAnnotationRange.start,
      currentTextAnnotationRange.end,
      text.slice(currentTextAnnotationRange.start, currentTextAnnotationRange.end),
    );
  }

  function insertInlineEquationFromTextarea(target: EventTarget | null): boolean {
    if (!canUseInlineFormatting || !(target instanceof HTMLTextAreaElement)) return false;
    const start = target.selectionStart;
    const end = target.selectionEnd;
    if (start === end) return false;
    const expression = target.value.slice(start, end);
    if (!normalizeRichTextEquationExpression(expression)) return false;
    void insertInlineEquationFromRange(start, end, expression);
    syncTextSelection(target);
    return true;
  }

  function toggleTextAnnotation(name: NotesRichTextAnnotationName): void {
    const patch = richTextAnnotationTogglePatch(currentTextAnnotationRange.annotations, name);
    void applyTextAnnotationsToRange(
      currentTextAnnotationRange.start,
      currentTextAnnotationRange.end,
      patch,
    );
  }

  function applyTextColor(color: NotesColor): void {
    void applyTextAnnotationsToRange(
      currentTextAnnotationRange.start,
      currentTextAnnotationRange.end,
      richTextColorPatch(color),
    );
  }

  function annotationShortcutName(event: KeyboardEvent): NotesRichTextAnnotationName | null {
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return null;
    const key = event.key.toLowerCase();
    if (!event.shiftKey && key === "b") return "bold";
    if (!event.shiftKey && key === "i") return "italic";
    if (!event.shiftKey && key === "u") return "underline";
    if (event.shiftKey && key === "s") return "strikethrough";
    if (!event.shiftKey && key === "e") return "code";
    return null;
  }

  function applyAnnotationShortcut(event: KeyboardEvent): boolean {
    const name = annotationShortcutName(event);
    if (!name || !canUseInlineFormatting) return false;
    const target = event.currentTarget;
    if (!(target instanceof HTMLTextAreaElement)) return false;
    const start = target.selectionStart;
    const end = target.selectionEnd;
    if (start === end) return false;
    const range = blockTextAnnotationsForSelection(block, start, end);
    void applyTextAnnotationsToRange(
      start,
      end,
      richTextAnnotationTogglePatch(range.annotations, name),
    );
    syncTextSelection(target);
    event.preventDefault();
    return true;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && !event.shiftKey && !event.altKey && event.key.toLowerCase() === "k") {
      if (openLinkEditorFromTextarea(event.currentTarget)) {
        event.preventDefault();
      }
      return;
    }
    if (
      (event.ctrlKey || event.metaKey)
      && event.shiftKey
      && !event.altKey
      && event.key.toLowerCase() === "e"
    ) {
      if (insertInlineEquationFromTextarea(event.currentTarget)) {
        event.preventDefault();
      }
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && headingToggleable) {
      event.preventDefault();
      onToggleOpen(block.id, !headingOpen);
      return;
    }
    if (applyAnnotationShortcut(event)) return;
    if (mentionOpen) {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        mentionActiveIndex = mentionMatches.length === 0
          ? 0
          : (mentionActiveIndex + 1) % mentionMatches.length;
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        mentionActiveIndex = mentionMatches.length === 0
          ? 0
          : (mentionActiveIndex + mentionMatches.length - 1) % mentionMatches.length;
        return;
      }
      if (event.key === "Enter" || event.key === "Tab") {
        const target = mentionMatches[mentionActiveIndex];
        if (target) {
          event.preventDefault();
          void selectMention(target);
        }
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        mentionQuery = null;
        mentionActiveIndex = 0;
        return;
      }
    }
    const target = event.currentTarget;
    const selectionStart = target instanceof HTMLTextAreaElement ? target.selectionStart : 0;
    const selectionEnd = target instanceof HTMLTextAreaElement ? target.selectionEnd : 0;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text,
      selectionStart,
      selectionEnd,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (action.type === "none" || action.type === "insert_newline") return;
    if (action.type === "open_slash_menu") {
      slashOpen = true;
      mentionQuery = null;
      return;
    }
    if (action.preventDefault) event.preventDefault();
    slashOpen = false;
    mentionQuery = null;
    onKeyboardAction(block.id, action);
  }

  function updateMentionQueryFromTextarea(target: HTMLTextAreaElement): void {
    if (!canUseMentions) {
      mentionQuery = null;
      mentionActiveIndex = 0;
      return;
    }
    mentionQuery = detectPageMentionQuery(target.value, target.selectionStart, target.selectionEnd);
    mentionActiveIndex = 0;
  }

  function syncTextSelection(target: EventTarget | null): void {
    if (!(target instanceof HTMLTextAreaElement)) return;
    textSelection = {
      start: target.selectionStart,
      end: target.selectionEnd,
    };
  }

  function openLinkEditorFromTextarea(target: EventTarget | null): boolean {
    if (!canUseLinks || !(target instanceof HTMLTextAreaElement)) return false;
    syncTextSelection(target);
    const range = blockTextLinkRangeForSelection(block, target.selectionStart, target.selectionEnd);
    if (range.start === range.end && !range.url) return false;
    linkRange = range;
    linkUrlInput = range.url ?? "";
    linkError = null;
    linkEditorOpen = true;
    slashOpen = false;
    mentionQuery = null;
    return true;
  }

  function openLinkEditorFromButton(): void {
    if (!textarea) return;
    openLinkEditorFromTextarea(textarea);
  }

  async function applyLinkFromEditor(): Promise<void> {
    const normalizedUrl = normalizeRichTextLinkUrl(linkUrlInput);
    if (!normalizedUrl) {
      linkError = t("notes.linkUrlInvalid");
      return;
    }
    linkError = null;
    await Promise.resolve(onApplyTextLink(block.id, linkRange.start, linkRange.end, normalizedUrl));
    linkEditorOpen = false;
    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(linkRange.end, linkRange.end);
  }

  async function removeLinkFromEditor(): Promise<void> {
    linkError = null;
    await Promise.resolve(onApplyTextLink(block.id, linkRange.start, linkRange.end, null));
    linkEditorOpen = false;
    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(linkRange.end, linkRange.end);
  }

  function handleInput(event: Event): void {
    const target = event.currentTarget;
    if (!(target instanceof HTMLTextAreaElement)) return;
    syncTextSelection(target);
    const value = target.value;
    slashOpen = value.startsWith("/") && !value.includes("\n");
    if (slashOpen) {
      mentionQuery = null;
    } else {
      updateMentionQueryFromTextarea(target);
    }
    onTextInput(block.id, value);
  }

  async function handlePaste(event: ClipboardEvent): Promise<void> {
    const target = event.currentTarget;
    if (!(target instanceof HTMLTextAreaElement)) return;
    const plainText = event.clipboardData?.getData("text/plain") ?? "";
    if (
      !shouldHandleNotesPlainTextPaste({
        currentBlockType: block.type,
        currentText: target.value,
        selectionStart: target.selectionStart,
        plainText,
      })
    ) {
      return;
    }
    const start = target.selectionStart;
    const end = target.selectionEnd;
    event.preventDefault();
    const handled = await Promise.resolve(onPastePlainText(block.id, start, end, plainText));
    if (handled) return;
    const nextText = `${target.value.slice(0, start)}${plainText}${target.value.slice(end)}`;
    onTextInput(block.id, nextText);
    await tick();
    const cursor = start + plainText.length;
    textarea?.focus();
    textarea?.setSelectionRange(cursor, cursor);
  }

  function handleTextareaBlur(): void {
    slashOpen = false;
    window.setTimeout(() => {
      mentionQuery = null;
      mentionActiveIndex = 0;
    }, 120);
  }

  async function selectMention(target: NotesMentionTarget): Promise<void> {
    if (!mentionQuery) return;
    const cursor = mentionQuery.start + target.title.length;
    const range = mentionQuery;
    mentionQuery = null;
    mentionActiveIndex = 0;
    if (target.kind === "page") {
      await Promise.resolve(onInsertPageMention(block.id, range.start, range.end, target));
    } else {
      await Promise.resolve(onInsertDateMention(block.id, range.start, range.end, target));
    }
    await tick();
    textarea?.focus();
    textarea?.setSelectionRange(cursor, cursor);
  }

  function handleBookmarkInput(url: string, caption: string): void {
    bookmarkOpenError = null;
    onBookmarkChange(block.id, url, caption);
  }

  function handleLinkPreviewInput(url: string): void {
    linkPreviewOpenError = null;
    onLinkPreviewUrlChange(block.id, url);
  }

  function handleEmbedInput(url: string): void {
    embedOpenError = null;
    onEmbedUrlChange(block.id, url);
  }

  function handleBookmarkKeydown(event: KeyboardEvent): void {
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement ? target : null;
    const actionText = text || (input?.value ?? "");
    const selectionStart = input?.selectionStart ?? 0;
    const selectionEnd = input?.selectionEnd ?? 0;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: actionText,
      selectionStart,
      selectionEnd,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (
      action.type === "none"
      || action.type === "insert_newline"
      || action.type === "open_slash_menu"
    ) {
      return;
    }
    if (action.preventDefault) event.preventDefault();
    onKeyboardAction(block.id, action);
  }

  function handleChildPageKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter" || event.key === " ") return;
    handleKeydown(event);
  }

  function handleEmbedKeydown(event: KeyboardEvent): void {
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement ? target : null;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: embedUrl,
      selectionStart: input?.selectionStart ?? 0,
      selectionEnd: input?.selectionEnd ?? 0,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (
      action.type === "none"
      || action.type === "insert_newline"
      || action.type === "open_slash_menu"
    ) {
      return;
    }
    if (action.preventDefault) event.preventDefault();
    onKeyboardAction(block.id, action);
  }

  function handleLinkPreviewKeydown(event: KeyboardEvent): void {
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement ? target : null;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: linkPreviewUrl,
      selectionStart: input?.selectionStart ?? 0,
      selectionEnd: input?.selectionEnd ?? 0,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (
      action.type === "none"
      || action.type === "insert_newline"
      || action.type === "open_slash_menu"
    ) {
      return;
    }
    if (action.preventDefault) event.preventDefault();
    onKeyboardAction(block.id, action);
  }

  function handleEquationKeydown(event: KeyboardEvent): void {
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement ? target : null;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: equationExpression,
      selectionStart: input?.selectionStart ?? 0,
      selectionEnd: input?.selectionEnd ?? 0,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (
      action.type === "none"
      || action.type === "insert_newline"
      || action.type === "open_slash_menu"
    ) {
      return;
    }
    if (action.preventDefault) event.preventDefault();
    onKeyboardAction(block.id, action);
  }

  function firstTableCell(node: HTMLInputElement): { destroy: () => void } {
    firstTableCellInput = node;
    return {
      destroy() {
        if (firstTableCellInput === node) firstTableCellInput = null;
      },
    };
  }

  function tableCellValue(row: NotesTableRowBlock, columnIndex: number): string {
    return tableCellPlainText(row.table_row.cells[columnIndex] ?? []);
  }

  async function openBookmark(): Promise<void> {
    if (block.type !== "bookmark" || !bookmarkCanOpen) return;
    try {
      bookmarkOpenError = null;
      await openUrl(block.bookmark.url.trim());
    } catch (error) {
      bookmarkOpenError = error instanceof Error ? error.message : String(error);
    }
  }

  async function openEmbed(): Promise<void> {
    if (block.type !== "embed" || !embedCanOpen) return;
    try {
      embedOpenError = null;
      await openUrl(block.embed.url.trim());
    } catch (error) {
      embedOpenError = error instanceof Error ? error.message : String(error);
    }
  }

  async function openLinkPreview(): Promise<void> {
    if (block.type !== "link_preview" || !linkPreviewCanOpen) return;
    try {
      linkPreviewOpenError = null;
      await openUrl(block.link_preview.url.trim());
    } catch (error) {
      linkPreviewOpenError = error instanceof Error ? error.message : String(error);
    }
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
    class:notes-callout-surface={block.type === "callout"}
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
    {:else if markerFor(block.type)}
      <div class="mt-1.5 w-5 shrink-0 text-right text-[0.866667rem] text-muted-foreground">
        {markerFor(block.type)}
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
            {#if index > 0}
              <ChevronRight class="size-3.5 shrink-0" aria-hidden="true" />
            {/if}
            {#if crumb.id && !crumb.current}
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
              <span class="min-w-0 truncate px-1 py-0.5">
                {crumb.title}
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
        <section class="my-1 min-w-0" aria-label={t("notes.blockType.table")}>
          {#if tableRows.length === 0}
            <button
              bind:this={tableOfContentsButton}
              type="button"
              class="min-h-8 w-full rounded px-1 text-left text-[0.8rem] text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
              onkeydown={handleKeydown}
            >
              {t("notes.emptyTable")}
            </button>
          {:else}
            <div class="notes-table-scroll overflow-x-auto rounded-md border border-border bg-background/70">
              <table
                class="notes-table w-max min-w-full table-fixed border-collapse text-[0.866667rem]"
                style={`--notes-table-width: ${tableColumnIndexes.length}`}
              >
                <tbody>
                  {#each tableRows as row, rowIndex (row.id)}
                    <tr>
                      {#each tableColumnIndexes as columnIndex}
                        <td
                          class:notes-table-column-header={block.table.has_column_header && rowIndex === 0}
                          class:notes-table-row-header={block.table.has_row_header && columnIndex === 0}
                        >
                          {#if rowIndex === 0 && columnIndex === 0}
                            <input
                              use:firstTableCell
                              class="notes-table-cell-input"
                              type="text"
                              value={tableCellValue(row, columnIndex)}
                              aria-label={t("notes.tableCell", rowIndex + 1, columnIndex + 1)}
                              oninput={(event) => {
                                onTableCellChange(row.id, columnIndex, event.currentTarget.value);
                              }}
                            />
                          {:else}
                            <input
                              class="notes-table-cell-input"
                              type="text"
                              value={tableCellValue(row, columnIndex)}
                              aria-label={t("notes.tableCell", rowIndex + 1, columnIndex + 1)}
                              oninput={(event) => {
                                onTableCellChange(row.id, columnIndex, event.currentTarget.value);
                              }}
                            />
                          {/if}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </section>
      {:else if block.type === "image" || block.type === "video" || block.type === "audio" || block.type === "file" || block.type === "pdf"}
        <NotesMediaBlock
          {block}
          {previousBlockType}
          {isOnlyBlock}
          {focusBlockId}
          {focusRequestId}
          {onKeyboardAction}
          {onMediaChange}
        />
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
      {:else if block.type === "bookmark"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-border bg-background/70 p-2"
          aria-label={t("notes.blockType.bookmark")}
        >
          <div
            class="mt-1 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <BookmarkIcon class="size-4" />
          </div>
          <div class="flex min-w-0 flex-1 flex-col gap-1">
            <input
              bind:this={bookmarkUrlInput}
              class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
              type="url"
              inputmode="url"
              aria-label={t("notes.bookmarkUrl")}
              value={block.bookmark.url}
              placeholder={t("notes.bookmarkUrlPlaceholder")}
              oninput={(event) => {
                handleBookmarkInput(event.currentTarget.value, bookmarkCaption);
              }}
              onkeydown={handleBookmarkKeydown}
            />
            <input
              class="min-h-7 w-full min-w-0 bg-transparent text-[0.8rem] text-muted-foreground outline-none placeholder:text-muted-foreground"
              type="text"
              aria-label={t("notes.bookmarkCaption")}
              value={bookmarkCaption}
              placeholder={t("notes.bookmarkCaptionPlaceholder")}
              oninput={(event) => {
                handleBookmarkInput(block.bookmark.url, event.currentTarget.value);
              }}
              onkeydown={handleBookmarkKeydown}
            />
            {#if bookmarkOpenError}
              <p class="text-[0.733333rem] text-destructive">
                {t("notes.openBookmarkFailed", bookmarkOpenError)}
              </p>
            {/if}
          </div>
          <button
            type="button"
            class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
            aria-label={t("notes.openBookmark", block.bookmark.url)}
            disabled={!bookmarkCanOpen}
            onclick={() => {
              void openBookmark();
            }}
          >
            <ExternalLink class="size-4" />
          </button>
        </section>
      {:else if block.type === "link_preview"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-border bg-background/70 p-2"
          aria-label={t("notes.blockType.linkPreview")}
        >
          <div
            class="mt-1 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <LinkIcon class="size-4" />
          </div>
          <div class="flex min-w-0 flex-1 flex-col gap-1.5">
            <input
              bind:this={linkPreviewUrlInput}
              class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
              type="url"
              inputmode="url"
              aria-label={t("notes.linkPreviewUrl")}
              value={linkPreviewUrl}
              placeholder={t("notes.linkPreviewUrlPlaceholder")}
              oninput={(event) => {
                handleLinkPreviewInput(event.currentTarget.value);
              }}
              onkeydown={handleLinkPreviewKeydown}
            />
            <output
              class="flex min-h-14 min-w-0 flex-col justify-center gap-0.5 rounded-md border border-border bg-muted/30 px-3 py-2 text-[0.866667rem]"
              aria-label={t("notes.linkPreviewPreview")}
            >
              <span class="min-w-0 truncate font-medium text-foreground">
                {linkPreviewTitle || t("notes.blockType.linkPreview")}
              </span>
              {#if linkPreviewSource}
                <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
                  {linkPreviewSource}
                </span>
              {/if}
            </output>
            {#if linkPreviewOpenError}
              <p class="text-[0.733333rem] text-destructive">
                {t("notes.openLinkPreviewFailed", linkPreviewOpenError)}
              </p>
            {/if}
          </div>
          <button
            type="button"
            class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
            aria-label={t("notes.openLinkPreview", linkPreviewUrl)}
            disabled={!linkPreviewCanOpen}
            onclick={() => {
              void openLinkPreview();
            }}
          >
            <ExternalLink class="size-4" />
          </button>
        </section>
      {:else if block.type === "synced_block"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-primary/35 bg-primary/5 p-2"
          aria-label={t("notes.blockType.syncedBlock")}
        >
          <div
            class="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary"
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
              {t("notes.blockType.syncedBlock")}
            </span>
            <span class="min-w-0 truncate text-[0.8rem] text-muted-foreground">
              {#if syncedBlockSourceId}
                {t("notes.syncedBlockReference", syncedBlockSourceId)}
              {:else}
                {t("notes.syncedBlockOriginal")}
              {/if}
            </span>
          </button>
        </section>
      {:else if block.type === "embed"}
        <section
          class="my-1 flex min-w-0 items-start gap-2 rounded-md border border-border bg-background/70 p-2"
          aria-label={t("notes.blockType.embed")}
        >
          <div
            class="mt-1 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <ExternalLink class="size-4" />
          </div>
          <div class="flex min-w-0 flex-1 flex-col gap-1.5">
            <input
              bind:this={embedUrlInput}
              class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
              type="url"
              inputmode="url"
              aria-label={t("notes.embedUrl")}
              value={embedUrl}
              placeholder={t("notes.embedUrlPlaceholder")}
              oninput={(event) => {
                handleEmbedInput(event.currentTarget.value);
              }}
              onkeydown={handleEmbedKeydown}
            />
            <output
              class="min-h-10 min-w-0 truncate rounded-md bg-muted/40 px-3 py-2 text-[0.866667rem] text-muted-foreground"
              aria-label={t("notes.embedPreview")}
            >
              {embedTitle || t("notes.blockType.embed")}
            </output>
            {#if embedOpenError}
              <p class="text-[0.733333rem] text-destructive">
                {t("notes.openEmbedFailed", embedOpenError)}
              </p>
            {/if}
          </div>
          <button
            type="button"
            class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
            aria-label={t("notes.openEmbed", embedUrl)}
            disabled={!embedCanOpen}
            onclick={() => {
              void openEmbed();
            }}
          >
            <ExternalLink class="size-4" />
          </button>
        </section>
      {:else if block.type === "equation"}
        <section
          class="my-1 flex min-w-0 flex-col gap-2 rounded-md border border-border bg-background/70 p-2"
          aria-label={t("notes.blockType.equation")}
        >
          <div class="flex min-w-0 items-center gap-2">
            <div
              class="flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
              aria-hidden="true"
            >
              <Sigma class="size-4" />
            </div>
            <input
              bind:this={equationInput}
              class="min-h-8 w-full min-w-0 bg-transparent font-mono text-[0.866667rem] outline-none placeholder:text-muted-foreground"
              type="text"
              inputmode="text"
              aria-label={t("notes.equationExpression")}
              value={equationExpression}
              placeholder={t("notes.equationPlaceholder")}
              spellcheck={false}
              oninput={(event) => {
                onEquationExpressionChange(block.id, event.currentTarget.value);
              }}
              onkeydown={handleEquationKeydown}
            />
          </div>
          <output
            class="min-h-10 min-w-0 overflow-x-auto rounded-md bg-muted/40 px-3 py-2 text-center font-serif text-[1.066667rem] text-foreground"
            aria-label={t("notes.equationPreview")}
          >
            {equationPreview}
          </output>
        </section>
      {:else if showTextEditor}
        {#if block.type === "code"}
          <div class="mb-1 flex justify-end">
            <select
              class="rounded border border-border bg-background px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground"
              aria-label={t("notes.codeLanguage")}
              value={block.code.language}
              onchange={(event) => onCodeLanguageChange(block.id, event.currentTarget.value)}
            >
              <option value="plain text">{t("notes.codeLanguagePlainText")}</option>
              <option value="typescript">TypeScript</option>
              <option value="rust">Rust</option>
              <option value="sql">SQL</option>
              <option value="bash">Bash</option>
              <option value="json">JSON</option>
              <option value="markdown">Markdown</option>
            </select>
          </div>
        {:else if block.type === "template"}
          <div class="mb-1 flex justify-end">
            <button
              type="button"
              class="inline-flex min-h-7 items-center gap-1.5 rounded border border-border bg-background px-2 text-[0.8rem] font-medium text-foreground hover:bg-accent disabled:cursor-not-allowed disabled:opacity-45"
              aria-label={t("notes.useTemplate", text || t("notes.blockType.template"))}
              disabled={!block.has_children}
              onclick={() => onUseTemplate(block.id)}
            >
              <Copy class="size-3.5" aria-hidden="true" />
              <span>{t("notes.useTemplateButton")}</span>
            </button>
          </div>
        {:else if block.type === "button"}
          <div class="mb-1 flex justify-end">
            <button
              type="button"
              class="inline-flex min-h-7 items-center gap-1.5 rounded border border-border bg-background px-2 text-[0.8rem] font-medium text-foreground hover:bg-accent disabled:cursor-not-allowed disabled:opacity-45"
              aria-label={t("notes.useButton", text || t("notes.blockType.button"))}
              disabled={!block.has_children}
              onclick={() => onUseButton(block.id)}
            >
              <MousePointerClick class="size-3.5" aria-hidden="true" />
              <span>{t("notes.useButtonButton")}</span>
            </button>
          </div>
        {/if}
        {#if canOpenInlineToolbar}
          <div class="mb-1 flex justify-end">
            <NotesInlineToolbar
              annotations={currentTextAnnotationRange.annotations}
              onToggleAnnotation={toggleTextAnnotation}
              onColorSelect={applyTextColor}
              onCreateEquation={insertInlineEquationFromSelection}
              onOpenLink={openLinkEditorFromButton}
            />
          </div>
        {:else if canOpenLinkEditor}
          <div class="mb-1 flex justify-end">
            <button
              type="button"
              class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
              aria-label={t("notes.openLinkEditor")}
              title={t("notes.openLinkEditor")}
              onclick={openLinkEditorFromButton}
            >
              <LinkIcon class="size-3.5" aria-hidden="true" />
            </button>
          </div>
        {/if}
        {#if showRichTextPreview}
          <button
            type="button"
            class={richTextPreviewClass(block.type)}
            aria-label={text || t("notes.blockPlaceholder")}
            onclick={() => onFocusBlock(block.id)}
          >
            <NotesRichTextInline richText={editableRichText} />
          </button>
        {:else}
          <textarea
            bind:this={textarea}
            class={textareaClass(block.type)}
            rows={textRows}
            value={text}
            placeholder={t("notes.blockPlaceholder")}
            spellcheck={block.type !== "code"}
            data-notes-block-id={block.id}
            oninput={handleInput}
            onkeydown={handleKeydown}
            onpaste={handlePaste}
            onkeyup={(event) => syncTextSelection(event.currentTarget)}
            onclick={(event) => syncTextSelection(event.currentTarget)}
            onselect={(event) => syncTextSelection(event.currentTarget)}
            onblur={handleTextareaBlur}
          ></textarea>
        {/if}
        {#if linkEditorOpen}
          <div class="mt-1 rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-sm">
            <div class="flex min-w-0 items-center gap-1.5">
              <input
                class="min-w-0 flex-1 rounded border border-input bg-background px-2 py-1 text-[0.8rem] outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={linkUrlInput}
                aria-label={t("notes.linkUrl")}
                placeholder={t("notes.linkUrlPlaceholder")}
                oninput={(event) => {
                  linkUrlInput = event.currentTarget.value;
                  linkError = null;
                }}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void applyLinkFromEditor();
                  }
                  if (event.key === "Escape") {
                    event.preventDefault();
                    linkEditorOpen = false;
                  }
                }}
              />
              <button
                type="button"
                class="rounded bg-primary px-2 py-1 text-[0.8rem] font-medium text-primary-foreground hover:bg-primary/90"
                onclick={() => {
                  void applyLinkFromEditor();
                }}
              >
                {t("notes.applyLink")}
              </button>
              {#if linkRange.url}
                <button
                  type="button"
                  class="rounded border border-border px-2 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                  onclick={() => {
                    void removeLinkFromEditor();
                  }}
                >
                  {t("notes.removeLink")}
                </button>
              {/if}
            </div>
            {#if linkError}
              <p class="mt-1 text-[0.733333rem] text-destructive">{linkError}</p>
            {/if}
          </div>
        {/if}
      {/if}

      {#if mentionOpen}
        <NotesMentionMenu
          targets={mentionMatches}
          activeIndex={mentionActiveIndex}
          onSelect={(target) => {
            void selectMention(target);
          }}
        />
      {:else if slashOpen}
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

  .notes-callout-surface {
    min-height: 2.5rem;
    padding-block: 0.35rem;
  }

  .notes-toc-item {
    padding-left: calc((var(--notes-toc-level) - 1) * 1rem);
  }

  .notes-table {
    min-width: calc(var(--notes-table-width) * 9rem);
  }

  .notes-table td {
    width: 9rem;
    min-width: 9rem;
    border-right: 1px solid hsl(var(--border));
    border-bottom: 1px solid hsl(var(--border));
  }

  .notes-table tr:last-child td {
    border-bottom: 0;
  }

  .notes-table td:last-child {
    border-right: 0;
  }

  .notes-table-cell-input {
    min-height: 2rem;
    width: 100%;
    min-width: 0;
    background: transparent;
    padding: 0.35rem 0.5rem;
    outline: none;
  }

  .notes-table-cell-input:focus {
    box-shadow: inset 0 0 0 2px hsl(var(--ring));
  }

  .notes-table-column-header,
  .notes-table-row-header {
    background: hsl(var(--muted) / 0.45);
    font-weight: 600;
  }
</style>
