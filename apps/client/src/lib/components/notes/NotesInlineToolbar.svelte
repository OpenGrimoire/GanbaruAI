<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    NOTES_BACKGROUND_COLORS,
    NOTES_TEXT_COLORS,
    notesBlockColorSwatchStyle,
  } from "$lib/notes/block-color";
  import type {
    NotesRichTextAnnotationName,
  } from "$lib/notes/rich-text";
  import type { NotesColor, NotesRichTextAnnotations } from "$lib/notes/types";
  import Bold from "@lucide/svelte/icons/bold";
  import Code from "@lucide/svelte/icons/code";
  import Italic from "@lucide/svelte/icons/italic";
  import LinkIcon from "@lucide/svelte/icons/link";
  import Palette from "@lucide/svelte/icons/palette";
  import Sigma from "@lucide/svelte/icons/sigma";
  import Strikethrough from "@lucide/svelte/icons/strikethrough";
  import Underline from "@lucide/svelte/icons/underline";

  let {
    annotations,
    onToggleAnnotation,
    onColorSelect,
    onCreateEquation,
    onOpenLink,
  }: {
    annotations: NotesRichTextAnnotations;
    onToggleAnnotation: (name: NotesRichTextAnnotationName) => void;
    onColorSelect: (color: NotesColor) => void;
    onCreateEquation: () => void;
    onOpenLink: () => void;
  } = $props();

  const { t } = getLocalization();

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
      ? "bg-accent text-accent-foreground"
      : "text-muted-foreground hover:bg-accent hover:text-accent-foreground";
  }
</script>

<div
  class="flex flex-wrap items-center justify-end gap-1 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-sm"
  role="toolbar"
  aria-label={t("notes.inlineToolbar")}
>
  <button
    type="button"
    class={`flex size-7 items-center justify-center rounded ${buttonClass("bold")}`}
    aria-label={t("notes.bold")}
    title={t("notes.bold")}
    aria-pressed={annotations.bold}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => onToggleAnnotation("bold")}
  >
    <Bold class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={`flex size-7 items-center justify-center rounded ${buttonClass("italic")}`}
    aria-label={t("notes.italic")}
    title={t("notes.italic")}
    aria-pressed={annotations.italic}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => onToggleAnnotation("italic")}
  >
    <Italic class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={`flex size-7 items-center justify-center rounded ${buttonClass("underline")}`}
    aria-label={t("notes.underline")}
    title={t("notes.underline")}
    aria-pressed={annotations.underline}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => onToggleAnnotation("underline")}
  >
    <Underline class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={`flex size-7 items-center justify-center rounded ${buttonClass("strikethrough")}`}
    aria-label={t("notes.strikethrough")}
    title={t("notes.strikethrough")}
    aria-pressed={annotations.strikethrough}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => onToggleAnnotation("strikethrough")}
  >
    <Strikethrough class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class={`flex size-7 items-center justify-center rounded ${buttonClass("code")}`}
    aria-label={t("notes.inlineCode")}
    title={t("notes.inlineCode")}
    aria-pressed={annotations.code}
    onmousedown={(event) => event.preventDefault()}
    onclick={() => onToggleAnnotation("code")}
  >
    <Code class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-accent-foreground"
    aria-label={t("notes.inlineEquation")}
    title={t("notes.inlineEquation")}
    onmousedown={(event) => event.preventDefault()}
    onclick={onCreateEquation}
  >
    <Sigma class="size-3.5" aria-hidden="true" />
  </button>
  <button
    type="button"
    class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-accent-foreground"
    aria-label={t("notes.openLinkEditor")}
    title={t("notes.openLinkEditor")}
    onmousedown={(event) => event.preventDefault()}
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
      class="max-w-36 bg-transparent text-[0.75rem] outline-none"
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
