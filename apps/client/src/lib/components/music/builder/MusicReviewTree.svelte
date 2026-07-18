<script lang="ts">
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Check from "@lucide/svelte/icons/check";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import Search from "@lucide/svelte/icons/search";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import {
    buildMusicReviewTree,
    flattenMusicReviewTree,
    musicReviewTreeAncestorFolderIds,
    musicReviewTreeFolderIds,
    searchMusicReviewTree,
    type MusicReviewTreeNode,
  } from "$lib/music/music-review-tree";
  import { cn } from "$lib/utils";

  let {
    items,
    totalCount,
    activeItemId,
    onActivate,
    onAssign,
    canRefresh = true,
    refreshing = false,
    onRefresh = () => undefined,
  }: {
    items: MusicItemListEntry[];
    totalCount: number;
    activeItemId: string | null;
    onActivate: (itemId: string) => void;
    onAssign: (itemIds: string[]) => void;
    canRefresh?: boolean;
    refreshing?: boolean;
    onRefresh?: () => void;
  } = $props();

  const { t } = getLocalization();
  let expandedIds = $state<Set<string>>(new Set());
  let explicitlyCollapsedIds = $state<Set<string>>(new Set());
  let lastExpandedActiveItemId = $state<string | null>(null);
  let selectedIds = $state<Set<string>>(new Set());
  let selectedFolderIds = $state<Set<string>>(new Set());
  let search = $state("");
  const tree = $derived(buildMusicReviewTree(items));
  const searching = $derived(search.trim().length > 0);
  const searchResult = $derived(searchMusicReviewTree(tree, search));
  const rows = $derived(searching ? searchResult.rows : flattenMusicReviewTree(tree, expandedIds));
  const folderSelectionReady = $derived(items.length >= totalCount);

  $effect(() => {
    const validIds = new Set(items.map((item) => item.id));
    const retained = [...selectedIds].filter((itemId) => validIds.has(itemId));
    if (retained.length !== selectedIds.size) selectedIds = new Set(retained);
  });

  $effect(() => {
    const validFolderIds = musicReviewTreeFolderIds(tree);
    const retainedSelected = [...selectedFolderIds].filter((folderId) => validFolderIds.has(folderId));
    if (retainedSelected.length !== selectedFolderIds.size) selectedFolderIds = new Set(retainedSelected);
    const retainedCollapsed = new Set([...explicitlyCollapsedIds]
      .filter((folderId) => validFolderIds.has(folderId)));
    if (retainedCollapsed.size !== explicitlyCollapsedIds.size) {
      explicitlyCollapsedIds = retainedCollapsed;
    }
    const nextExpanded = new Set([...validFolderIds]
      .filter((folderId) => !retainedCollapsed.has(folderId)));
    if (nextExpanded.size !== expandedIds.size
      || [...nextExpanded].some((folderId) => !expandedIds.has(folderId))) {
      expandedIds = nextExpanded;
    }
  });

  $effect(() => {
    const itemId = activeItemId;
    if (!itemId || itemId === lastExpandedActiveItemId) return;
    lastExpandedActiveItemId = itemId;
    const ancestorIds = musicReviewTreeAncestorFolderIds(tree, itemId);
    if (ancestorIds.length === 0) return;
    const nextExpanded = new Set(expandedIds);
    const nextCollapsed = new Set(explicitlyCollapsedIds);
    for (const folderId of ancestorIds) {
      nextExpanded.add(folderId);
      nextCollapsed.delete(folderId);
    }
    expandedIds = nextExpanded;
    explicitlyCollapsedIds = nextCollapsed;
  });

  function toggleExpanded(nodeId: string): void {
    if (searching) return;
    const next = new Set(expandedIds);
    const nextCollapsed = new Set(explicitlyCollapsedIds);
    if (next.has(nodeId)) {
      next.delete(nodeId);
      nextCollapsed.add(nodeId);
    } else {
      next.add(nodeId);
      nextCollapsed.delete(nodeId);
    }
    expandedIds = next;
    explicitlyCollapsedIds = nextCollapsed;
  }

  function toggleFolder(node: MusicReviewTreeNode): void {
    const folderIds = musicReviewTreeFolderIds([node]);
    const nextItems = new Set(selectedIds);
    const nextFolders = new Set(selectedFolderIds);
    const selecting = !nextFolders.has(node.id);
    for (const itemId of node.itemIds) {
      if (selecting) nextItems.add(itemId);
      else nextItems.delete(itemId);
    }
    for (const folderId of folderIds) {
      if (selecting) nextFolders.add(folderId);
      else nextFolders.delete(folderId);
    }
    selectedIds = nextItems;
    selectedFolderIds = nextFolders;
  }

  function toggleItem(itemId: string): void {
    const next = new Set(selectedIds);
    if (next.has(itemId)) {
      next.delete(itemId);
      const nextFolders = new Set(selectedFolderIds);
      const removeContainingFolders = (nodes: readonly MusicReviewTreeNode[]): void => {
        for (const node of nodes) {
          if (node.itemIds.includes(itemId)) nextFolders.delete(node.id);
          removeContainingFolders(node.children);
        }
      };
      removeContainingFolders(tree);
      selectedFolderIds = nextFolders;
    } else next.add(itemId);
    selectedIds = next;
  }

  function handleSearchKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || !searching) return;
    event.preventDefault();
    search = "";
  }
</script>

<section class="review-tree flex min-h-0 flex-col" aria-label={t("music.builder.reviewFolders")}>
  <div class="shrink-0 p-2">
    <div class="flex h-8 items-center gap-2 rounded-full bg-secondary/35 px-2.5 focus-within:bg-secondary/55">
      <Search size={13} class="shrink-0 text-muted-foreground" />
      <input bind:value={search} onkeydown={handleSearchKeydown} type="text" inputmode="search" enterkeyhint="search" autocomplete="off" aria-label={t("music.builder.searchReviewTree")} placeholder={t("music.builder.searchReviewTree")} class="min-w-0 flex-1 bg-transparent text-[0.7rem] outline-none placeholder:text-muted-foreground" />
      {#if searching}
        <span class="shrink-0 text-[0.6rem] tabular-nums text-muted-foreground" aria-live="polite" aria-label={t("music.builder.reviewSearchMatchCount", searchResult.matchCount)} title={t("music.builder.reviewSearchMatchCount", searchResult.matchCount)}>{searchResult.matchCount}</span>
        <button type="button" onclick={() => search = ""} class="grid h-6 w-6 shrink-0 place-items-center rounded-full text-muted-foreground hover:bg-background/60 hover:text-foreground" aria-label={t("music.builder.clearReviewSearch")}><X size={12} /></button>
      {/if}
      <button type="button" onclick={onRefresh} disabled={!canRefresh || refreshing} class="grid h-6 w-6 shrink-0 place-items-center rounded-full text-muted-foreground hover:bg-background/60 hover:text-foreground disabled:opacity-40" aria-label={refreshing ? t("music.builder.refreshing") : t("music.builder.refreshLocalFolders")} title={refreshing ? t("music.builder.refreshing") : t("music.builder.refreshLocalFolders")}><RefreshCw class={cn(refreshing && "animate-spin motion-reduce:animate-none")} size={12} /></button>
    </div>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto px-1.5 pb-2" data-music-scrollable="true">
    {#if searching && rows.length === 0}
      <p class="px-3 py-6 text-center text-[0.68rem] text-muted-foreground">{t("music.builder.noReviewSearchMatches")}</p>
    {/if}
    {#each rows as row (row.kind === "folder" ? row.node.id : row.item.id)}
      {#if row.kind === "folder"}
        {@const checked = selectedFolderIds.has(row.node.id)}
        <div class="group relative flex h-8 min-w-0 items-center rounded-lg hover:bg-accent/60" style={`padding-left: ${row.depth * 0.75}rem`}>
          <button type="button" onclick={() => toggleExpanded(row.node.id)} disabled={searching} class="absolute inset-0 rounded-lg focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring disabled:cursor-default" aria-label={(searching || expandedIds.has(row.node.id)) ? t("music.builder.collapseFolder", row.node.name) : t("music.builder.expandFolder", row.node.name)} aria-expanded={searching || expandedIds.has(row.node.id)}></button>
          <span class="pointer-events-none grid h-8 w-7 shrink-0 place-items-center text-muted-foreground">
            <ChevronRight size={14} class={cn("transition-transform motion-reduce:transition-none", (searching || expandedIds.has(row.node.id)) && "rotate-90")} />
          </span>
          <label class="relative z-10 grid h-full w-7 shrink-0 cursor-pointer place-items-center">
            <input type="checkbox" checked={checked} disabled={!folderSelectionReady} onchange={() => toggleFolder(row.node)} class="peer absolute h-4 w-4 opacity-0" aria-label={t("music.builder.selectFolder", row.node.name, row.node.itemIds.length)} />
            <span class={cn("pointer-events-none grid h-4 w-4 place-items-center rounded border peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring", checked ? "border-primary bg-primary text-primary-foreground" : "border-border/80 bg-background/70", !folderSelectionReady && "opacity-35")}>
              {#if checked}<Check size={11} strokeWidth={2.5} />{/if}
            </span>
          </label>
          <span class="pointer-events-none flex min-w-0 flex-1 items-center pr-2 text-left">
            <span class="min-w-0 flex-1 truncate text-[0.7rem] font-medium">{row.node.name}</span>
            <span class="text-[0.6rem] tabular-nums text-muted-foreground">{row.node.itemIds.length}</span>
          </span>
        </div>
      {:else}
        <div class={cn("relative flex h-8 min-w-0 items-center rounded-lg", activeItemId === row.item.id ? "bg-primary/10 text-foreground" : "hover:bg-accent/50")} style={`padding-left: ${row.depth * 0.75 + 1.75}rem`}>
          <button type="button" onclick={() => onActivate(row.item.id)} class="absolute inset-0 rounded-lg focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring" aria-label={row.item.reviewState === "reviewed" ? `${row.item.title}, ${t("music.builder.markReviewed")}` : row.item.title} aria-current={activeItemId === row.item.id ? "true" : undefined}></button>
          <label class="relative z-10 grid h-full w-7 shrink-0 cursor-pointer place-items-center">
            <input type="checkbox" checked={selectedIds.has(row.item.id)} onchange={() => toggleItem(row.item.id)} class="peer absolute h-4 w-4 opacity-0" aria-label={t("music.builder.selectTrack", row.item.title)} />
            <span class={cn("pointer-events-none grid h-4 w-4 place-items-center rounded border peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring", selectedIds.has(row.item.id) ? "border-primary bg-primary text-primary-foreground" : "border-border/80 bg-background/70")}>
              {#if selectedIds.has(row.item.id)}<Check size={11} strokeWidth={2.5} />{/if}
            </span>
          </label>
          <span class="pointer-events-none flex min-w-0 flex-1 items-center pr-2 text-left">
            <span class="min-w-0 flex-1 truncate text-[0.68rem]">{row.item.title}</span>
            {#if row.item.reviewState === "reviewed"}
              <span class="ml-2 grid h-5 w-5 shrink-0 place-items-center text-primary" aria-hidden="true"><Check size={12} strokeWidth={2.5} /></span>
            {/if}
          </span>
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
