<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    NOTES_BACKGROUND_COLORS,
    NOTES_TEXT_COLORS,
    notesBlockColorSwatchStyle,
  } from "$lib/notes/block-color";
  import { shouldPreventInlineToolbarPointerDefault } from "$lib/notes/inline-toolbar";
  import type {
    NotesRichTextAnnotationName,
  } from "$lib/notes/rich-text";
  import type { NotesColor, NotesRichTextAnnotations } from "$lib/notes/types";
  import Bold from "@lucide/svelte/icons/bold";
  import Code from "@lucide/svelte/icons/code";
  import Italic from "@lucide/svelte/icons/italic";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Palette from "@lucide/svelte/icons/palette";
  import PencilLine from "@lucide/svelte/icons/pencil-line";
  import Sigma from "@lucide/svelte/icons/sigma";
  import Strikethrough from "@lucide/svelte/icons/strikethrough";
  import Underline from "@lucide/svelte/icons/underline";

  let {
    annotations,
    onToggleAnnotation,
    onColorSelect,
    onCreateEquation,
    onCreateComment,
    onCreateSuggestion,
    onOpenLink,
  }: {
    annotations: NotesRichTextAnnotations;
    onToggleAnnotation: (name: NotesRichTextAnnotationName) => void;
    onColorSelect: (color: NotesColor) => void;
    onCreateEquation: () => void;
    onCreateComment: () => void;
    onCreateSuggestion: () => void;
    onOpenLink: () => void;
  } = $props();

  const { t } = getLocalization();
  const toolbarButtonBase =
    "flex size-7 items-center justify-center rounded outline-none focus-visible:ring-2 focus-visible:ring-ring";

  function annotationActive(name: NotesRichTextAnnotationName): boolean {
    switch (name) {
      case "bold":
        return annotations.bold;
      case "italic":
        return annotations.italic;
      case "strikethrough":
        return annotations.strikethrough;
      case "underline":
        return annotations.underline;
      case "code":
        return annotations.code;
    }
  }

  function colorLabel(color: NotesColor): string {
    switch (color) {
      case "default":
        return t("notes.blockColor.default");
      case "gray":
        return t("notes.blockColor.gray");
      case "brown":
        return t("notes.blockColor.brown");
      case "orange":
        return t("notes.blockColor.orange");
      case "yellow":
        return t("notes.blockColor.yellow");
      case "green":
        return t("notes.blockColor.green");
      case "blue":
        return t("notes.blockColor.blue");
      case "purple":
        return t("notes.blockColor.purple");
      case "pink":
        return t("notes.blockColor.pink");
      case "red":
        return t("notes.blockColor.red");
      case "gray_background":
        return t("notes.blockColor.grayBackground");
      case "brown_background":
        return t("notes.blockColor.brownBackground");
      case "orange_background":
        return t("notes.blockColor.orangeBackground");
      case "yellow_background":
        return t("notes.blockColor.yellowBackground");
      case "green_background":
        return t("notes.blockColor.greenBackground");
      case "blue_background":
        return t("notes.blockColor.blueBackground");
      case "purple_background":
        return t("notes.blockColor.purpleBackground");
      case "pink_background":
        return t("notes.blockColor.pinkBackground");
      case "red_background":
        return t("notes.blockColor.redBackground");
    }
  }

  function buttonClass(name: NotesRichTextAnnotationName): string {
    return annotationActive(name)
      ? `${toolbarButtonBase} bg-accent text-accent-foreground`
      : `${toolbarButtonBase} text-muted-foreground hover:bg-accent hover:text-accent-foreground`;
  }

  function plainButtonClass(): string {
    return `${toolbarButtonBase} text-muted-foreground hover:bg-accent hover:text-accent-foreground`;
  }

  function preserveMouseSelection(event: MouseEvent): void {
    event.preventDefault();
  }

  function preserveTouchSelection(event: PointerEvent): void {
    if (shouldPreventInlineToolbarPointerDefault(event.pointerType)) event.preventDefault();
  }
</script>

<div
  class="flex max-w-full flex-wrap items-center justify-end gap-1 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-sm"
  role="toolbar"
  aria-label={t("notes.inlineToolbar")}
  aria-orientation="horizontal"
>
  <button
    type="button"
    class={buttonClass("bold")}
    aria-label={t("notes.bold")}
    title={t("notes.bold")}
    aria-pressed={annotations.bold}
    aria-keyshortcuts="Control+B Meta+B"
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={() => onToggleAnnotation("bold")}
  >
    <Bold class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={buttonClass("italic")}
    aria-label={t("notes.italic")}
    title={t("notes.italic")}
    aria-pressed={annotations.italic}
    aria-keyshortcuts="Control+I Meta+I"
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={() => onToggleAnnotation("italic")}
  >
    <Italic class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={buttonClass("underline")}
    aria-label={t("notes.underline")}
    title={t("notes.underline")}
    aria-pressed={annotations.underline}
    aria-keyshortcuts="Control+U Meta+U"
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={() => onToggleAnnotation("underline")}
  >
    <Underline class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={buttonClass("strikethrough")}
    aria-label={t("notes.strikethrough")}
    title={t("notes.strikethrough")}
    aria-pressed={annotations.strikethrough}
    aria-keyshortcuts="Control+Shift+S Meta+Shift+S"
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={() => onToggleAnnotation("strikethrough")}
  >
    <Strikethrough class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={buttonClass("code")}
    aria-label={t("notes.inlineCode")}
    title={t("notes.inlineCode")}
    aria-pressed={annotations.code}
    aria-keyshortcuts="Control+E Meta+E"
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={() => onToggleAnnotation("code")}
  >
    <Code class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={plainButtonClass()}
    aria-label={t("notes.inlineEquation")}
    title={t("notes.inlineEquation")}
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={onCreateEquation}
  >
    <Sigma class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={plainButtonClass()}
    aria-label={t("notes.inlineComment")}
    title={t("notes.inlineComment")}
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={onCreateComment}
  >
    <MessageSquare class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={plainButtonClass()}
    aria-label={t("notes.inlineSuggestion")}
    title={t("notes.inlineSuggestion")}
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={onCreateSuggestion}
  >
    <PencilLine class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={plainButtonClass()}
    aria-label={t("notes.openLinkEditor")}
    title={t("notes.openLinkEditor")}
    onmousedown={preserveMouseSelection}
    onpointerdown={preserveTouchSelection}
    onclick={onOpenLink}
  >
    <LinkIcon class="size-3.5" aria-hidden="true" />
  </button>
  <label
    class="flex min-h-7 min-w-0 items-center gap-1 rounded px-1 text-[0.75rem] text-muted-foreground"
  >
    <Palette class="size-3.5 shrink-0" aria-hidden="true" />
    <span class="sr-only">{t("notes.textColor")}</span>
    <select
      class="max-w-36 rounded bg-transparent text-[0.75rem] outline-none focus-visible:ring-2 focus-visible:ring-ring"
      aria-label={t("notes.textColor")}
      value={annotations.color}
      onchange={(event) => onColorSelect(event.currentTarget.value as NotesColor)}
    >
      <option value="default">{colorLabel("default")}</option>
      <optgroup label={t("notes.textColors")}>
        {#each NOTES_TEXT_COLORS.filter((color) => color !== "default") as color}
          <option value={color}>{colorLabel(color)}</option>
        {/each}
      </optgroup>
      <optgroup label={t("notes.backgroundColors")}>
        {#each NOTES_BACKGROUND_COLORS as color}
          <option value={color}>{colorLabel(color)}</option>
        {/each}
      </optgroup>
    </select>
  </label>
  <span
    class="size-4 rounded border"
    aria-hidden="true"
    style={notesBlockColorSwatchStyle(annotations.color)}
  ></span>
</div>
