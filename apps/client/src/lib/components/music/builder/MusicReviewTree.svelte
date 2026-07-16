<script lang="ts">
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Check from "@lucide/svelte/icons/check";
  import Folder from "@lucide/svelte/icons/folder";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import Music2 from "@lucide/svelte/icons/music-2";
  import Minus from "@lucide/svelte/icons/minus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import {
    buildMusicReviewTree,
    flattenMusicReviewTree,
    musicReviewTreeFolderIds,
    toggleMusicReviewTreeSelection,
  } from "$lib/music/music-review-tree";
  import { cn } from "$lib/utils";

  let {
    items,
    totalCount,
    loading,
    activeItemId,
    onActivate,
    onAssign,
  }: {
    items: MusicItemListEntry[];
    totalCount: number;
    loading: boolean;
    activeItemId: string | null;
    onActivate: (itemId: string) => void;
    onAssign: (itemIds: string[]) => void;
  } = $props();

  const { t } = getLocalization();
  let expandedIds = $state<Set<string>>(new Set());
  let expansionInitialized = $state(false);
  let selectedIds = $state<Set<string>>(new Set());
  const tree = $derived(buildMusicReviewTree(items));
  const rows = $derived(flattenMusicReviewTree(tree, expandedIds));
  const folderSelectionReady = $derived(items.length >= totalCount);

  $effect(() => {
    const validIds = new Set(items.map((item) => item.id));
    const retained = [...selectedIds].filter((itemId) => validIds.has(itemId));
    if (retained.length !== selectedIds.size) selectedIds = new Set(retained);
  });

  $effect(() => {
    if (expansionInitialized || tree.length === 0) return;
    expandedIds = musicReviewTreeFolderIds(tree);
    expansionInitialized = true;
  });

  function toggleExpanded(nodeId: string): void {
    const next = new Set(expandedIds);
    if (next.has(nodeId)) next.delete(nodeId);
    else next.add(nodeId);
    expandedIds = next;
  }

  function toggleFolder(itemIds: string[]): void {
    selectedIds = toggleMusicReviewTreeSelection(selectedIds, itemIds);
  }

  function toggleItem(itemId: string): void {
    const next = new Set(selectedIds);
    if (next.has(itemId)) next.delete(itemId);
    else next.add(itemId);
    selectedIds = next;
  }

  function indeterminate(node: HTMLInputElement, value: boolean): { update: (next: boolean) => void } {
    node.indeterminate = value;
    return { update: (next) => { node.indeterminate = next; } };
  }
</script>

<section class="review-tree flex min-h-0 flex-col bg-background/25" aria-label={t("music.builder.reviewFolders")}>
  <div class="flex min-h-10 shrink-0 items-center gap-2 px-3">
    <Folder size={15} class="text-primary" strokeWidth={1.7} />
    <h2 class="min-w-0 flex-1 truncate text-xs font-semibold">{t("music.builder.reviewFolders")}</h2>
    <span class="text-[0.65rem] tabular-nums text-muted-foreground">{#if loading || !folderSelectionReady}{items.length}/{totalCount}{:else}{items.length}{/if}</span>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto px-1.5 pb-2" data-music-scrollable="true">
    {#each rows as row (row.kind === "folder" ? row.node.id : row.item.id)}
      {#if row.kind === "folder"}
        {@const selectedCount = row.node.itemIds.filter((itemId) => selectedIds.has(itemId)).length}
        {@const checked = row.node.itemIds.length > 0 && selectedCount === row.node.itemIds.length}
        <div class="group flex h-8 min-w-0 items-center rounded-lg hover:bg-accent/60" style={`padding-left: ${row.depth * 0.75}rem`}>
          <button type="button" onclick={() => toggleExpanded(row.node.id)} class="grid h-7 w-7 shrink-0 place-items-center rounded-md text-muted-foreground" aria-label={expandedIds.has(row.node.id) ? t("music.builder.collapseFolder", row.node.name) : t("music.builder.expandFolder", row.node.name)} aria-expanded={expandedIds.has(row.node.id)}>
            <ChevronRight size={14} class={cn("transition-transform motion-reduce:transition-none", expandedIds.has(row.node.id) && "rotate-90")} />
          </button>
          <label class="relative grid h-7 w-7 shrink-0 cursor-pointer place-items-center">
            <input type="checkbox" checked={checked} disabled={!folderSelectionReady} use:indeterminate={selectedCount > 0 && !checked} onchange={() => toggleFolder(row.node.itemIds)} class="peer absolute h-4 w-4 opacity-0" aria-label={t("music.builder.selectFolder", row.node.name, row.node.itemIds.length)} />
            <span class={cn("pointer-events-none grid h-4 w-4 place-items-center rounded border", selectedCount > 0 ? "border-primary bg-primary text-primary-foreground" : "border-border/80 bg-background/70", !folderSelectionReady && "opacity-35")}>
              {#if checked}<Check size={11} strokeWidth={2.5} />{:else if selectedCount > 0}<Minus size={10} strokeWidth={2.5} />{/if}
            </span>
          </label>
          <button type="button" onclick={() => toggleExpanded(row.node.id)} class="flex min-w-0 flex-1 items-center gap-2 pr-2 text-left">
            <Folder size={14} class="shrink-0 text-primary/80" />
            <span class="min-w-0 flex-1 truncate text-[0.7rem] font-medium">{row.node.name}</span>
            <span class="text-[0.6rem] tabular-nums text-muted-foreground">{row.node.itemIds.length}</span>
          </button>
        </div>
      {:else}
        <div class={cn("flex h-8 min-w-0 items-center rounded-lg", activeItemId === row.item.id ? "bg-primary/10 text-foreground" : "hover:bg-accent/50")} style={`padding-left: ${row.depth * 0.75 + 1.75}rem`}>
          <label class="relative grid h-7 w-7 shrink-0 cursor-pointer place-items-center">
            <input type="checkbox" checked={selectedIds.has(row.item.id)} onchange={() => toggleItem(row.item.id)} class="peer absolute h-4 w-4 opacity-0" aria-label={t("music.builder.selectTrack", row.item.title)} />
            <span class={cn("pointer-events-none grid h-4 w-4 place-items-center rounded border", selectedIds.has(row.item.id) ? "border-primary bg-primary text-primary-foreground" : "border-border/80 bg-background/70")}>
              {#if selectedIds.has(row.item.id)}<Check size={11} strokeWidth={2.5} />{/if}
            </span>
          </label>
          <button type="button" onclick={() => onActivate(row.item.id)} class="flex min-w-0 flex-1 items-center gap-2 pr-2 text-left" aria-current={activeItemId === row.item.id ? "true" : undefined}>
            <Music2 size={13} class="shrink-0 text-muted-foreground" />
            <span class="min-w-0 flex-1 truncate text-[0.68rem]">{row.item.title}</span>
          </button>
        </div>
      {/if}
    {/each}
  </div>

  {#if selectedIds.size > 0}
    <div class="shrink-0 p-2">
      <button type="button" onclick={() => onAssign([...selectedIds])} class="flex min-h-9 w-full items-center justify-center gap-2 rounded-lg bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm hover:bg-primary/90">
        <ListPlus size={14} />
        {t("music.builder.assignSelected", selectedIds.size)}
      </button>
    </div>
  {/if}
</section>
