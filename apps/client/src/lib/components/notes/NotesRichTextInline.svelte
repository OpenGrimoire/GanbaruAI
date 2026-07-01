<script lang="ts">
  import { notesRichTextColorStyle } from "$lib/notes/block-color";
  import { equationPreviewText } from "$lib/notes/equation";
  import type { NotesRichText } from "$lib/notes/types";

  let {
    richText,
  }: {
    richText: readonly NotesRichText[];
  } = $props();

  function textClass(item: NotesRichText): string {
    const classes = ["notes-rich-text-segment"];
    if (item.annotations.bold) classes.push("font-semibold");
    if (item.annotations.italic) classes.push("italic");
    if (item.annotations.underline) classes.push("underline");
    if (item.annotations.strikethrough) classes.push("line-through");
    if (item.annotations.code) {
      classes.push("rounded bg-muted/70 px-1 py-0.5 font-mono text-[0.9em]");
    }
    if (item.type === "text" && (item.href || item.text.link)) {
      classes.push("text-primary underline underline-offset-2");
    }
    return classes.join(" ");
  }
</script>

{#each richText as item}
  {#if item.type === "mention"}
    <span
      class="notes-rich-text-segment inline-flex max-w-full items-center rounded bg-accent px-1 text-accent-foreground"
      style={notesRichTextColorStyle(item.annotations.color)}
    >
      {item.plain_text}
    </span>
  {:else if item.type === "equation"}
    <span
      class={`${textClass(item)} inline-flex max-w-full items-center rounded bg-muted/70 px-1 py-0.5 font-serif text-[1.02em]`}
      style={notesRichTextColorStyle(item.annotations.color)}
      title={item.equation.expression}
    >
      {equationPreviewText(item.equation.expression)}
    </span>
  {:else}
    <span
      class={textClass(item)}
      style={notesRichTextColorStyle(item.annotations.color)}
      title={item.href ?? item.text.link?.url ?? undefined}
    >
      {item.plain_text}
    </span>
  {/if}
{/each}

<style>
  .notes-rich-text-segment {
    color: var(--notes-rich-text-color, inherit);
    background: var(--notes-rich-text-bg, transparent);
    box-shadow: inset 0 0 0 1px var(--notes-rich-text-border, transparent);
  }
</style>
