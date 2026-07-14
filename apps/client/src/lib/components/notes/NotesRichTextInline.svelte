<script lang="ts">
  import { notesRichTextColorStyle } from "$lib/notes/block-color";
  import type { NotesResolvedCommentAnchor } from "$lib/notes/comments";
  import { equationPreviewText } from "$lib/notes/equation";
  import type { NotesResolvedSuggestionAnchor } from "$lib/notes/suggestions";
  import type { NotesRichText } from "$lib/notes/types";

  let {
    richText,
    commentAnchors = [],
    suggestionAnchors = [],
  }: {
    richText: readonly NotesRichText[];
    commentAnchors?: readonly NotesResolvedCommentAnchor[];
    suggestionAnchors?: readonly NotesResolvedSuggestionAnchor[];
  } = $props();

  const visibleRichText = $derived(richText.filter(richTextItemIsVisible));
  const visibleRuns = $derived(visibleRichTextRuns(visibleRichText));
  const visibleLines = $derived(visibleRichTextLines(visibleRuns));

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

  interface VisibleRichTextLine {
    parts: VisibleRichTextLinePart[];
  }

  interface VisibleRichTextLinePart {
    item: NotesRichText;
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
    for (const anchor of suggestionAnchors) {
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

  function suggestionAnchorsForRange(start: number, end: number): NotesResolvedSuggestionAnchor[] {
    return suggestionAnchors.filter((anchor) => anchor.start < end && anchor.end > start);
  }

  function suggestionAnchorIdsForRange(start: number, end: number): string | undefined {
    const ids = suggestionAnchorsForRange(start, end).map((anchor) => anchor.suggestionId);
    return ids.length > 0 ? ids.join(" ") : undefined;
  }

  function suggestionAnchorClass(start: number, end: number): string {
    const anchors = suggestionAnchorsForRange(start, end);
    if (anchors.length === 0) return "";
    if (anchors.some((anchor) => anchor.status === "open")) return " notes-rich-text-suggestion-anchor";
    if (anchors.some((anchor) => anchor.status === "accepted")) {
      return " notes-rich-text-suggestion-anchor notes-rich-text-suggestion-anchor-accepted";
    }
    return " notes-rich-text-suggestion-anchor notes-rich-text-suggestion-anchor-rejected";
  }

  function linkUrl(item: NotesRichText): string | null {
    return item.type === "text" ? item.text.link?.url ?? item.href : item.href;
  }

  function appendLinePart(
    line: VisibleRichTextLine,
    item: NotesRichText,
    text: string,
    start: number,
  ): void {
    if (text.length === 0) return;
    line.parts.push({
      item,
      text,
      start,
      end: start + text.length,
    });
  }

  function visibleRichTextLines(runs: readonly VisibleRichTextRun[]): VisibleRichTextLine[] {
    if (runs.length === 0) return [];
    const lines: VisibleRichTextLine[] = [{ parts: [] }];

    for (const run of runs) {
      for (const segment of splitRunByAnchors(run)) {
        const parts = segment.text.split("\n");
        let cursor = segment.start;
        for (let index = 0; index < parts.length; index += 1) {
          const part = parts[index] ?? "";
          appendLinePart(lines.at(-1) ?? lines[0], run.item, part, cursor);
          cursor += part.length;
          if (index < parts.length - 1) {
            lines.push({ parts: [] });
            cursor += 1;
          }
        }
      }
    }

    return lines;
  }
</script>{#each visibleLines as line}
  <div class="notes-rich-text-line" data-notes-editor-line="true">
    {#if line.parts.length === 0}
      <span class="notes-rich-text-empty-line-sentinel" data-notes-editor-sentinel="empty-line">{"\u200b"}</span>
    {:else}
      {#each line.parts as part}
        {#if part.item.type === "mention"}
          <span
            class={`notes-rich-text-segment inline-flex max-w-full items-center rounded bg-accent px-1 text-accent-foreground${commentAnchorClass(part.start, part.end)}${suggestionAnchorClass(part.start, part.end)}`}
            style={notesRichTextColorStyle(part.item.annotations.color)}
            data-notes-bold={part.item.annotations.bold ? "true" : undefined}
            data-notes-italic={part.item.annotations.italic ? "true" : undefined}
            data-notes-underline={part.item.annotations.underline ? "true" : undefined}
            data-notes-strikethrough={part.item.annotations.strikethrough ? "true" : undefined}
            data-notes-code={part.item.annotations.code ? "true" : undefined}
            data-notes-rich-text-color={part.item.annotations.color === "default" ? undefined : part.item.annotations.color}
            data-notes-link-url={linkUrl(part.item) ?? undefined}
            data-notes-comment-anchor={anchorIdsForRange(part.start, part.end)}
            data-notes-suggestion-anchor={suggestionAnchorIdsForRange(part.start, part.end)}
          >
            {part.text}
          </span>
        {:else if part.item.type === "equation"}
          <span
            class={`${textClass(part.item)} inline-flex max-w-full items-center rounded bg-muted/70 px-1 py-0.5 font-serif text-[1.02em]${commentAnchorClass(part.start, part.end)}${suggestionAnchorClass(part.start, part.end)}`}
            style={notesRichTextColorStyle(part.item.annotations.color)}
            title={part.item.equation.expression}
            data-notes-bold={part.item.annotations.bold ? "true" : undefined}
            data-notes-italic={part.item.annotations.italic ? "true" : undefined}
            data-notes-underline={part.item.annotations.underline ? "true" : undefined}
            data-notes-strikethrough={part.item.annotations.strikethrough ? "true" : undefined}
            data-notes-code={part.item.annotations.code ? "true" : undefined}
            data-notes-rich-text-color={part.item.annotations.color === "default" ? undefined : part.item.annotations.color}
            data-notes-link-url={linkUrl(part.item) ?? undefined}
            data-notes-comment-anchor={anchorIdsForRange(part.start, part.end)}
            data-notes-suggestion-anchor={suggestionAnchorIdsForRange(part.start, part.end)}
          >
            {part.text}
          </span>
        {:else}
          <span
            class={`${textClass(part.item)}${commentAnchorClass(part.start, part.end)}${suggestionAnchorClass(part.start, part.end)}`}
            style={notesRichTextColorStyle(part.item.annotations.color)}
            title={part.item.href ?? part.item.text.link?.url ?? undefined}
            data-notes-bold={part.item.annotations.bold ? "true" : undefined}
            data-notes-italic={part.item.annotations.italic ? "true" : undefined}
            data-notes-underline={part.item.annotations.underline ? "true" : undefined}
            data-notes-strikethrough={part.item.annotations.strikethrough ? "true" : undefined}
            data-notes-code={part.item.annotations.code ? "true" : undefined}
            data-notes-rich-text-color={part.item.annotations.color === "default" ? undefined : part.item.annotations.color}
            data-notes-link-url={linkUrl(part.item) ?? undefined}
            data-notes-comment-anchor={anchorIdsForRange(part.start, part.end)}
            data-notes-suggestion-anchor={suggestionAnchorIdsForRange(part.start, part.end)}
          >
            {part.text}
          </span>
        {/if}
      {/each}
    {/if}
  </div>
{/each}<style>
  .notes-rich-text-line {
    display: block;
    line-height: inherit;
  }

  .notes-rich-text-segment {
    color: var(--notes-rich-text-color, inherit);
    background: var(--notes-rich-text-bg, transparent);
    box-shadow: inset 0 0 0 1px var(--notes-rich-text-border, transparent);
  }

  .notes-rich-text-empty-line-sentinel {
    color: transparent;
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

  .notes-rich-text-suggestion-anchor {
    border-radius: 0.2rem;
    background: hsl(var(--secondary) / 0.38);
    box-shadow: inset 0 -0.12rem 0 hsl(var(--primary) / 0.7);
  }

  .notes-rich-text-suggestion-anchor-accepted {
    background: hsl(var(--primary) / 0.12);
    box-shadow: inset 0 -0.12rem 0 hsl(var(--primary) / 0.45);
  }

  .notes-rich-text-suggestion-anchor-rejected {
    background: hsl(var(--destructive) / 0.1);
    box-shadow: inset 0 -0.12rem 0 hsl(var(--destructive) / 0.35);
  }
</style>
