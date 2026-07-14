<script lang="ts">
  import type { QuickNoteTextRun } from "$lib/quick-notes/types";

  let { runs }: { runs: readonly QuickNoteTextRun[] } = $props();

  type VisiblePart = QuickNoteTextRun;

  interface VisibleLine {
    parts: VisiblePart[];
  }

  function visibleLines(source: readonly QuickNoteTextRun[]): VisibleLine[] {
    const lines: VisibleLine[] = [{ parts: [] }];
    for (const run of source) {
      const parts = run.content.split("\n");
      for (let index = 0; index < parts.length; index += 1) {
        const content = parts[index] ?? "";
        if (content) lines.at(-1)?.parts.push({ ...run, content });
        if (index < parts.length - 1) lines.push({ parts: [] });
      }
    }
    return lines;
  }

  const lines = $derived(visibleLines(runs));
</script>{#each lines as line}
  <div class="quick-note-line" data-notes-editor-line="true">
    {#if line.parts.length === 0}
      <span class="quick-note-empty-line" data-notes-editor-sentinel="empty-line">{"\u200b"}</span>
    {:else}
      {#each line.parts as part}
        <span
          class:font-semibold={part.bold}
          class:italic={part.italic}
          class:underline={part.underline}
          class="wrap-break-word"
        >{part.content}</span>
      {/each}
    {/if}
  </div>
{/each}<style>
  .quick-note-line {
    display: block;
    line-height: inherit;
    min-height: 1lh;
  }

  .quick-note-empty-line {
    color: transparent;
  }
</style>
