<script lang="ts">
  import { Temporal } from "@js-temporal/polyfill";
  import { tick } from "svelte";
  import Copy from "@lucide/svelte/icons/copy";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MousePointerClick from "@lucide/svelte/icons/mouse-pointer-click";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    blockEditableRichText,
    blockHasVisibleRichTextFormatting,
    blockPlainText,
    blockTextAnnotationsForSelection,
    blockTextLinkRangeForSelection,
    headingIsToggleable,
    headingToggleOpen,
    isHeadingBlockType,
    type NotesHeadingBlockType,
  } from "$lib/notes/block-factory";
  import { blockColor, canBlockHaveColor } from "$lib/notes/block-color";
  import { shouldHandleNotesPlainTextPaste } from "$lib/notes/block-clipboard";
  import {
    notesRichTextPreviewClass,
    notesTextareaClass,
  } from "$lib/notes/block-editor-ui";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import { notesTextSelectionFromControl } from "$lib/notes/editor-selection";
  import {
    buildDateMentionTargets,
    detectPageMentionQuery,
    filterPageMentionTargets,
    normalizeRichTextEquationExpression,
    normalizeRichTextLinkUrl,
    richTextAnnotationTogglePatch,
    richTextColorPatch,
    type NotesDateMentionTarget,
    type NotesMentionQuery,
    type NotesMentionTarget,
    type NotesPageMentionTarget,
    type NotesRichTextAnnotationName,
    type NotesRichTextAnnotationPatch,
  } from "$lib/notes/rich-text";
  import type { NotesSlashAction, NotesSlashCommand } from "$lib/notes/slash-commands";
  import type {
    NotesBlock,
    NotesBlockType,
    NotesColor,
  } from "$lib/notes/types";
  import NotesInlineToolbar from "./NotesInlineToolbar.svelte";
  import NotesMentionMenu from "./NotesMentionMenu.svelte";
  import NotesRichTextInline from "./NotesRichTextInline.svelte";
  import NotesSlashMenu from "./NotesSlashMenu.svelte";

  let {
    block,
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
    onConvert,
    onConvertToToggleHeading,
    onColorChange,
    onCopyLink,
    onDuplicate,
    onUseTemplate,
    onUseButton,
    onMoveUp,
    onMoveDown,
    onDelete,
    onToggleOpen,
    onCodeLanguageChange,
    onFocusBlock,
  }: {
    block: NotesBlock;
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
    onConvert: (blockId: string, type: NotesBlockType, clearText?: boolean) => void;
    onConvertToToggleHeading: (blockId: string, type: NotesHeadingBlockType) => void;
    onColorChange: (blockId: string, color: NotesColor) => void;
    onCopyLink: (blockId: string) => Promise<void> | void;
    onDuplicate: (blockId: string) => void;
    onUseTemplate: (blockId: string) => void;
    onUseButton: (blockId: string) => void;
    onMoveUp: (blockId: string) => void;
    onMoveDown: (blockId: string) => void;
    onDelete: (blockId: string) => void;
    onToggleOpen: (blockId: string, open: boolean) => void;
    onCodeLanguageChange: (blockId: string, language: string) => void;
    onFocusBlock: (blockId: string) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const locale = $derived(localization.locale);
  let textarea: HTMLTextAreaElement | null = $state(null);
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
  const text = $derived(blockPlainText(block));
  const textRows = $derived(Math.max(1, text.split("\n").length));
  const isBlockFocused = $derived(focusBlockId === block.id);
  const canUseMentions = $derived(block.type !== "code");
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
  const currentColor = $derived(blockColor(block));
  const blockSupportsColor = $derived(canBlockHaveColor(block.type));
  const headingToggleable = $derived(isHeadingBlockType(block.type) && headingIsToggleable(block));
  const headingOpen = $derived(!isHeadingBlockType(block.type) || headingToggleOpen(block));

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

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => {
      if (!textarea) return;
      textarea.focus();
      const length = textarea.value.length;
      textarea.setSelectionRange(length, length);
    });
  });

  $effect(() => {
    if (mentionActiveIndex >= mentionMatches.length) mentionActiveIndex = 0;
  });

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
    textSelection = notesTextSelectionFromControl(target);
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
</script>

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
    class={notesRichTextPreviewClass(block.type)}
    aria-label={text || t("notes.blockPlaceholder")}
    onclick={() => onFocusBlock(block.id)}
  >
    <NotesRichTextInline richText={editableRichText} />
  </button>
{:else}
  <textarea
    bind:this={textarea}
    class={notesTextareaClass(block.type)}
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
