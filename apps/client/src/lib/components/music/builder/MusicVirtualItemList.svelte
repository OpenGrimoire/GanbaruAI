<script lang="ts">
  import { tick } from "svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import { musicVirtualWindow, revealMusicVirtualIndex } from "$lib/music/music-virtual-window";
  import MusicBuilderItemRow from "./MusicBuilderItemRow.svelte";

  let {
    items,
    selectedItemId,
    selectedItemIds = selectedItemId ? [selectedItemId] : [],
    initialScrollTop = 0,
    playingItemId = null,
    onSelect,
    onSelectionChange = () => undefined,
    onPlay = () => undefined,
    onScrollTop = () => undefined,
  }: {
    items: MusicItemListEntry[];
    selectedItemId: string | null;
    selectedItemIds?: string[];
    initialScrollTop?: number;
    playingItemId?: string | null;
    onSelect: (item: MusicItemListEntry) => void;
    onSelectionChange?: (itemIds: string[], activeItemId: string) => void;
    onPlay?: (item: MusicItemListEntry) => void;
    onScrollTop?: (scrollTop: number) => void;
  } = $props();

  const rowHeight = 64;
  let viewport = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);
  let rangeAnchor = $state<string | null>(null);
  const windowed = $derived(musicVirtualWindow({ count: items.length, scrollTop, viewportHeight, rowHeight, overscan: 6 }));
  const visibleItems = $derived(items.slice(windowed.startIndex, windowed.endIndex));

  function viewportAction(node: HTMLElement): { destroy: () => void } {
    viewport = node;
    node.scrollTop = initialScrollTop;
    const update = () => {
      scrollTop = node.scrollTop;
      viewportHeight = node.clientHeight;
      onScrollTop(scrollTop);
    };
    const observer = new ResizeObserver(update);
    observer.observe(node);
    node.addEventListener("scroll", update, { passive: true });
    update();
    return { destroy: () => { observer.disconnect(); node.removeEventListener("scroll", update); viewport = null; } };
  }

  async function select(item: MusicItemListEntry, event?: MouseEvent): Promise<void> {
    let nextSelection: string[];
    if (event?.shiftKey && rangeAnchor) {
      const anchorIndex = items.findIndex((entry) => entry.id === rangeAnchor);
      const targetIndex = items.findIndex((entry) => entry.id === item.id);
      const start = Math.min(anchorIndex, targetIndex);
      const end = Math.max(anchorIndex, targetIndex);
      nextSelection = anchorIndex >= 0 && targetIndex >= 0
        ? items.slice(start, end + 1).map((entry) => entry.id)
        : [item.id];
    } else if (event?.ctrlKey || event?.metaKey) {
      const next = new Set(selectedItemIds);
      if (next.has(item.id)) next.delete(item.id);
      else next.add(item.id);
      nextSelection = [...next];
      rangeAnchor = item.id;
    } else {
      nextSelection = [item.id];
      rangeAnchor = item.id;
    }
    onSelectionChange(nextSelection, item.id);
    onSelect(item);
    await tick();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp" && event.key !== "Home" && event.key !== "End") return;
    event.preventDefault();
    if (items.length === 0) return;
    const selectedIndex = items.findIndex((item) => item.id === selectedItemId);
    let nextIndex = selectedIndex < 0 ? 0 : selectedIndex;
    if (event.key === "ArrowDown") nextIndex = Math.min(items.length - 1, nextIndex + 1);
    if (event.key === "ArrowUp") nextIndex = Math.max(0, nextIndex - 1);
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;
    const nextItem = items[nextIndex];
    const anchorId = rangeAnchor ?? (selectedIndex >= 0 ? items[selectedIndex].id : nextItem.id);
    if (event.shiftKey) {
      const anchorIndex = items.findIndex((item) => item.id === anchorId);
      const start = Math.min(anchorIndex, nextIndex);
      const end = Math.max(anchorIndex, nextIndex);
      onSelectionChange(items.slice(start, end + 1).map((item) => item.id), nextItem.id);
      rangeAnchor = anchorId;
    } else {
      onSelectionChange([nextItem.id], nextItem.id);
      rangeAnchor = nextItem.id;
    }
    onSelect(nextItem);
    if (viewport) viewport.scrollTop = revealMusicVirtualIndex(nextIndex, viewport.scrollTop, viewport.clientHeight, rowHeight);
    void tick().then(() => document.querySelector<HTMLElement>(`[data-music-focus-key="item:${CSS.escape(items[nextIndex].id)}"] button`)?.focus());
  }
</script>

<div
  class="music-builder-list h-full min-h-0 overflow-y-auto px-2 py-1 outline-none"
  use:viewportAction
  role="listbox"
  aria-multiselectable="true"
  tabindex="0"
  onkeydown={handleKeydown}
>
  <div style={`height: ${windowed.topSpacer}px`} aria-hidden="true"></div>
  {#each visibleItems as item (item.id)}
    <MusicBuilderItemRow
      {item}
      selected={selectedItemIds.includes(item.id)}
      playing={item.id === playingItemId}
      onSelect={(selected, event) => { void select(selected, event); }}
      {onPlay}
    />
  {/each}
  <div style={`height: ${windowed.bottomSpacer}px`} aria-hidden="true"></div>
</div>

<style>
  .music-builder-list { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
</style>
