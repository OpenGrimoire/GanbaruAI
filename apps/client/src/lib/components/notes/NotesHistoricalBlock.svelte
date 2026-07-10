<script lang="ts">
  import CheckSquare from "@lucide/svelte/icons/square-check-big";
  import Square from "@lucide/svelte/icons/square";
  import { parseNotesRichTextArray } from "$lib/notes/block-validation";
  import type { NotesRichText } from "$lib/notes/types";
  import NotesRichTextInline from "./NotesRichTextInline.svelte";

  let {
    block,
    depth = 0,
  }: {
    block: Record<string, unknown>;
    depth?: number;
  } = $props();

  const type = $derived(typeof block.type === "string" ? block.type : "paragraph");
  const content = $derived(blockContent(block, type));
  const richText = $derived(parseRichText(content));
  const plainText = $derived(typeof block.plain_text === "string" ? block.plain_text : "");
  const checked = $derived(content.checked === true);

  function blockContent(
    row: Record<string, unknown>,
    blockType: string,
  ): Record<string, unknown> {
    const payload = typeof row.payload === "object"
      && row.payload !== null
      && !Array.isArray(row.payload)
      ? row.payload as Record<string, unknown>
      : row;
    const nested = payload[blockType];
    return typeof nested === "object" && nested !== null && !Array.isArray(nested)
      ? nested as Record<string, unknown>
      : payload;
  }

  function parseRichText(value: Record<string, unknown>): NotesRichText[] {
    try {
      return parseNotesRichTextArray(value.rich_text ?? [], "historical block rich text");
    } catch {
      return [];
    }
  }

  function contentClass(blockType: string): string {
    switch (blockType) {
      case "heading_1":
        return "mt-5 text-2xl font-semibold leading-tight";
      case "heading_2":
        return "mt-4 text-xl font-semibold leading-tight";
      case "heading_3":
      case "heading_4":
        return "mt-3 text-base font-semibold leading-snug";
      case "quote":
        return "border-l-2 border-foreground/40 pl-3 italic";
      case "code":
        return "rounded-md bg-muted px-3 py-2 font-mono text-[0.8rem] whitespace-pre-wrap";
      default:
        return "text-[0.933333rem] leading-6";
    }
  }
</script>

{#if plainText || richText.length > 0 || type === "divider"}
  <div
    class={`min-w-0 ${contentClass(type)}`}
    style={`margin-left: ${Math.min(depth, 6) * 1.1}rem`}
  >
    {#if type === "divider"}
      <div class="my-2 h-px bg-border"></div>
    {:else}
      <div class="flex min-w-0 items-start gap-2">
        {#if type === "bulleted_list_item"}
          <span class="mt-0.5 shrink-0">•</span>
        {:else if type === "numbered_list_item"}
          <span class="mt-0.5 shrink-0">1.</span>
        {:else if type === "to_do"}
          {#if checked}
            <CheckSquare class="mt-1 size-4 shrink-0 text-muted-foreground" />
          {:else}
            <Square class="mt-1 size-4 shrink-0 text-muted-foreground" />
          {/if}
        {/if}
        <div class:line-through={type === "to_do" && checked} class="min-w-0 wrap-break-word">
          {#if richText.length > 0}
            <NotesRichTextInline {richText} />
          {:else}
            {plainText}
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}
