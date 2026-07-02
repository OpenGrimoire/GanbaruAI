<script lang="ts">
  import { notesRichTextColorStyle } from "$lib/notes/block-color";
  import type { NotesResolvedCommentAnchor } from "$lib/notes/comments";
  import { equationPreviewText } from "$lib/notes/equation";
  import type { NotesRichText } from "$lib/notes/types";

  let {
    richText,
    commentAnchors = [],
  }: {
    richText: readonly NotesRichText[];
    commentAnchors?: readonly NotesResolvedCommentAnchor[];
  } = $props();

  const visibleRichText = $derived(richText.filter(richTextItemIsVisible));
  const visibleRuns = $derived(visibleRichTextRuns(visibleRichText));

  interface VisibleRichTextRun {
    item: NotesRichText;
    text: string;
    start: number;
    end: number;
  }

  interface VisibleRichTextSegment {
    text: string;
    start: number;
    end: number;
  }

  function richTextItemIsVisible(item: NotesRichText): boolean {
    if (item.type === "equation") return equationPreviewText(item.equation.expression).length > 0;
    return item.plain_text.length > 0;
  }

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

  function visibleText(item: NotesRichText): string {
    return item.type === "equation" ? equationPreviewText(item.equation.expression) : item.plain_text;
  }

  function visibleRichTextRuns(items: readonly NotesRichText[]): VisibleRichTextRun[] {
    let cursor = 0;
    return items.map((item) => {
      const text = visibleText(item);
      const start = cursor;
      cursor += text.length;
      return { item, text, start, end: cursor };
    });
  }

  function splitRunByAnchors(run: VisibleRichTextRun): VisibleRichTextSegment[] {
    const boundaries = new Set<number>([run.start, run.end]);
    for (const anchor of commentAnchors) {
      const start = Math.max(run.start, anchor.start);
      const end = Math.min(run.end, anchor.end);
      if (start < end) {
        boundaries.add(start);
        boundaries.add(end);
      }
    }
    const sorted = [...boundaries].sort((left, right) => left - right);
    const segments: VisibleRichTextSegment[] = [];
    for (let index = 0; index < sorted.length - 1; index += 1) {
      const start = sorted[index];
      const end = sorted[index + 1];
      if (start === end) continue;
      segments.push({
        start,
        end,
        text: run.text.slice(start - run.start, end - run.start),
      });
    }
    return segments;
  }

  function anchorsForRange(start: number, end: number): NotesResolvedCommentAnchor[] {
    return commentAnchors.filter((anchor) => anchor.start < end && anchor.end > start);
  }

  function anchorIdsForRange(start: number, end: number): string | undefined {
    const ids = anchorsForRange(start, end).map((anchor) => anchor.threadId);
    return ids.length > 0 ? ids.join(" ") : undefined;
  }

  function commentAnchorClass(start: number, end: number): string {
    const anchors = anchorsForRange(start, end);
    if (anchors.length === 0) return "";
    return anchors.some((anchor) => anchor.status === "open")
      ? " notes-rich-text-comment-anchor"
      : " notes-rich-text-comment-anchor notes-rich-text-comment-anchor-resolved";
  }

  function linkUrl(item: NotesRichText): string | null {
    return item.type === "text" ? item.text.link?.url ?? item.href : item.href;
  }
</script>

{#each visibleRuns as run}
  {#if run.item.type === "mention"}
    <span
      class={`notes-rich-text-segment inline-flex max-w-full items-center rounded bg-accent px-1 text-accent-foreground${commentAnchorClass(run.start, run.end)}`}
      style={notesRichTextColorStyle(run.item.annotations.color)}
      data-notes-bold={run.item.annotations.bold ? "true" : undefined}
      data-notes-italic={run.item.annotations.italic ? "true" : undefined}
      data-notes-underline={run.item.annotations.underline ? "true" : undefined}
      data-notes-strikethrough={run.item.annotations.strikethrough ? "true" : undefined}
      data-notes-code={run.item.annotations.code ? "true" : undefined}
      data-notes-rich-text-color={run.item.annotations.color === "default" ? undefined : run.item.annotations.color}
      data-notes-link-url={linkUrl(run.item) ?? undefined}
      data-notes-comment-anchor={anchorIdsForRange(run.start, run.end)}
    >
      {run.text}
    </span>
  {:else if run.item.type === "equation"}
    <span
      class={`${textClass(run.item)} inline-flex max-w-full items-center rounded bg-muted/70 px-1 py-0.5 font-serif text-[1.02em]${commentAnchorClass(run.start, run.end)}`}
      style={notesRichTextColorStyle(run.item.annotations.color)}
      title={run.item.equation.expression}
      data-notes-bold={run.item.annotations.bold ? "true" : undefined}
      data-notes-italic={run.item.annotations.italic ? "true" : undefined}
      data-notes-underline={run.item.annotations.underline ? "true" : undefined}
      data-notes-strikethrough={run.item.annotations.strikethrough ? "true" : undefined}
      data-notes-code={run.item.annotations.code ? "true" : undefined}
      data-notes-rich-text-color={run.item.annotations.color === "default" ? undefined : run.item.annotations.color}
      data-notes-link-url={linkUrl(run.item) ?? undefined}
      data-notes-comment-anchor={anchorIdsForRange(run.start, run.end)}
    >
      {run.text}
    </span>
  {:else}
    {#each splitRunByAnchors(run) as segment}
      <span
        class={`${textClass(run.item)}${commentAnchorClass(segment.start, segment.end)}`}
        style={notesRichTextColorStyle(run.item.annotations.color)}
        title={run.item.href ?? run.item.text.link?.url ?? undefined}
        data-notes-bold={run.item.annotations.bold ? "true" : undefined}
        data-notes-italic={run.item.annotations.italic ? "true" : undefined}
        data-notes-underline={run.item.annotations.underline ? "true" : undefined}
        data-notes-strikethrough={run.item.annotations.strikethrough ? "true" : undefined}
        data-notes-code={run.item.annotations.code ? "true" : undefined}
        data-notes-rich-text-color={run.item.annotations.color === "default" ? undefined : run.item.annotations.color}
        data-notes-link-url={linkUrl(run.item) ?? undefined}
        data-notes-comment-anchor={anchorIdsForRange(segment.start, segment.end)}
      >
        {segment.text}
      </span>
    {/each}
  {/if}
{/each}

<style>
  .notes-rich-text-segment {
    color: var(--notes-rich-text-color, inherit);
    background: var(--notes-rich-text-bg, transparent);
    box-shadow: inset 0 0 0 1px var(--notes-rich-text-border, transparent);
  }

  .notes-rich-text-comment-anchor {
    border-radius: 0.2rem;
    background: hsl(var(--primary) / 0.16);
    box-shadow: inset 0 -0.12rem 0 hsl(var(--primary) / 0.45);
  }

  .notes-rich-text-comment-anchor-resolved {
    background: hsl(var(--muted-foreground) / 0.12);
    box-shadow: inset 0 -0.12rem 0 hsl(var(--muted-foreground) / 0.35);
  }
</style>
