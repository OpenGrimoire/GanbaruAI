<script lang="ts">
  import { onMount, tick } from "svelte";
  import { flip } from "svelte/animate";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import { musicVirtualWindow, revealMusicVirtualIndex } from "$lib/music/music-virtual-window";
  import { isMusicReviewEditableTarget } from "$lib/music/music-review";
  import MusicBuilderItemRow from "./MusicBuilderItemRow.svelte";

  let {
    items,
    selectedItemId,
    selectedItemIds = selectedItemId ? [selectedItemId] : [],
    initialScrollTop = 0,
    playingItemId = null,
    playlistMode = false,
    reorderEnabled = false,
    onSelect,
    onSelectionChange = () => undefined,
    onPlay = () => undefined,
    onScrollTop = () => undefined,
    onReorder = () => undefined,
    hasMore = false,
    loadingMore = false,
    onLoadMore = () => undefined,
  }: {
    items: MusicItemListEntry[];
    selectedItemId: string | null;
    selectedItemIds?: string[];
    initialScrollTop?: number;
    playingItemId?: string | null;
    playlistMode?: boolean;
    reorderEnabled?: boolean;
    onSelect: (item: MusicItemListEntry) => void;
    onSelectionChange?: (itemIds: string[], activeItemId: string) => void;
    onPlay?: (item: MusicItemListEntry) => void;
    onScrollTop?: (scrollTop: number) => void;
    onReorder?: (item: MusicItemListEntry, targetIndex: number) => void;
    hasMore?: boolean;
    loadingMore?: boolean;
    onLoadMore?: () => void;
  } = $props();

  const { t } = getLocalization();
  const rowHeight = 64;
  let viewport = $state<HTMLElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);
  let rangeAnchor = $state<string | null>(null);
  let draggedItemId = $state<string | null>(null);
  let dropTargetId = $state<string | null>(null);
  let dropEdge = $state<"before" | "after">("before");
  let revealedPlayingItemId = $state<string | null>(null);
  let reducedMotion = $state(false);
  const windowed = $derived(musicVirtualWindow({ count: items.length, scrollTop, viewportHeight, rowHeight, overscan: 6 }));
  const visibleItems = $derived(items.slice(windowed.startIndex, windowed.endIndex));

  onMount(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = (): void => { reducedMotion = query.matches; };
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  });

  $effect(() => {
    if (!playingItemId || playingItemId === revealedPlayingItemId || !viewport) return;
    const index = items.findIndex((item) => item.id === playingItemId);
    if (index < 0) return;
    revealedPlayingItemId = playingItemId;
    viewport.scrollTop = revealMusicVirtualIndex(index, viewport.scrollTop, viewport.clientHeight, rowHeight);
  });

  function viewportAction(node: HTMLElement): { destroy: () => void } {
    viewport = node;
    node.scrollTop = initialScrollTop;
    const update = () => {
      scrollTop = node.scrollTop;
      viewportHeight = node.clientHeight;
      onScrollTop(scrollTop);
      if (hasMore && !loadingMore && node.scrollTop + node.clientHeight >= node.scrollHeight - rowHeight * 4) onLoadMore();
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
    if (event.isComposing || isMusicReviewEditableTarget(event.target)) return;
    if (playlistMode && reorderEnabled && event.altKey && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
      const selectedIndex = items.findIndex((item) => item.id === selectedItemId);
      if (selectedIndex < 0) return;
      event.preventDefault();
      const targetIndex = event.key === "ArrowUp"
        ? Math.max(0, selectedIndex - 1)
        : Math.min(items.length - 1, selectedIndex + 1);
      onReorder(items[selectedIndex], targetIndex);
      return;
    }
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

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (!(event.target instanceof Node) || !viewport?.contains(event.target)) return;
    handleKeydown(event);
  }

  function dragStart(item: MusicItemListEntry, event: DragEvent): void {
    draggedItemId = item.id;
    event.dataTransfer?.setData("text/plain", item.id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function dragOver(item: MusicItemListEntry, event: DragEvent): void {
    if (!draggedItemId) return;
    event.preventDefault();
    const row = event.currentTarget as HTMLElement;
    const rowBounds = row.getBoundingClientRect();
    dropTargetId = item.id;
    dropEdge = event.clientY < rowBounds.top + rowBounds.height / 2 ? "before" : "after";
    if (!viewport) return;
    const bounds = viewport.getBoundingClientRect();
    const edge = 36;
    if (event.clientY < bounds.top + edge) viewport.scrollTop -= rowHeight;
    else if (event.clientY > bounds.bottom - edge) viewport.scrollTop += rowHeight;
  }

  function drop(target: MusicItemListEntry, event: DragEvent): void {
    event.preventDefault();
    const sourceId = draggedItemId ?? event.dataTransfer?.getData("text/plain") ?? null;
    draggedItemId = null;
    dropTargetId = null;
    if (!sourceId || sourceId === target.id) return;
    const source = items.find((item) => item.id === sourceId);
    const sourceIndex = items.findIndex((item) => item.id === sourceId);
    const remaining = items.filter((item) => item.id !== sourceId);
    const remainingTargetIndex = remaining.findIndex((item) => item.id === target.id);
    const targetIndex = Math.min(items.length - 1, remainingTargetIndex + (dropEdge === "after" ? 1 : 0));
    if (source && sourceIndex >= 0 && remainingTargetIndex >= 0 && sourceIndex !== targetIndex) onReorder(source, targetIndex);
  }

  function dragEnd(): void {
    draggedItemId = null;
    dropTargetId = null;
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div
  class="music-builder-list h-full min-h-0 overflow-y-auto px-2 py-1 outline-none"
  use:viewportAction
  role="list"
  aria-busy={loadingMore}
>
  <div style={`height: ${windowed.topSpacer}px`} aria-hidden="true"></div>
  {#each visibleItems as item, visibleIndex (item.id)}
    <div animate:flip={{ duration: reducedMotion ? 0 : 150 }}>
    <MusicBuilderItemRow
      {item}
      position={windowed.startIndex + visibleIndex + 1}
      setSize={items.length}
      selected={selectedItemIds.includes(item.id)}
      playing={item.id === playingItemId}
      {playlistMode}
      {reorderEnabled}
      dropEdge={dropTargetId === item.id ? dropEdge : null}
      onSelect={(selected, event) => { void select(selected, event); }}
      {onPlay}
      onMore={(selected) => { void select(selected); }}
      onMove={(entry, direction) => {
        const index = items.findIndex((item) => item.id === entry.id);
        onReorder(entry, direction === "up" ? Math.max(0, index - 1) : Math.min(items.length - 1, index + 1));
      }}
      onMoveTo={onReorder}
      onDragStart={dragStart}
      onDragEnd={dragEnd}
      onDragOver={dragOver}
      onDrop={drop}
    />
    </div>
  {/each}
  <div style={`height: ${windowed.bottomSpacer}px`} aria-hidden="true"></div>
  {#if loadingMore}<div class="mx-2 my-1 h-10 animate-pulse rounded-lg bg-secondary motion-reduce:animate-none" aria-hidden="true"></div>{/if}
  <span class="sr-only" role="status" aria-live="polite">{loadingMore ? t("music.builder.loadingMore") : ""}</span>
</div>

<style>
  .music-builder-list { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
</style>
