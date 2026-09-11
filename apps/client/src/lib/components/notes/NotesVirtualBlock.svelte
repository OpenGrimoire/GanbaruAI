<script lang="ts">
  import type { Snippet } from "svelte";
  import type { NotesBlockTreeItem } from "$lib/notes/types";

  let {
    item,
    blockId,
    retainedHeight,
    measure,
    children,
  }: {
    item: NotesBlockTreeItem | undefined;
    blockId: string;
    retainedHeight: number;
    measure: (
      node: HTMLElement,
      blockId: string,
    ) => { update(nextBlockId: string): void; destroy(): void };
    children: Snippet<[NotesBlockTreeItem]>;
  } = $props();
</script>

{#if item}
  <div use:measure={item.block.id} data-notes-virtual-block={item.block.id}>
    {@render children(item)}
  </div>
{:else}
  <div
    class="rounded-sm bg-muted/20"
    style:height={`${retainedHeight}px`}
    data-notes-block-placeholder={blockId}
    aria-hidden="true"
  ></div>
{/if}
