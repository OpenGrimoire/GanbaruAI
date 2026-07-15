<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { fly } from "svelte/transition";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { createMusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import { projectMusicBuilderLayout } from "$lib/music/music-builder-layout";
  import {
    backMusicBuilderRoute,
    initialMusicBuilderRoute,
    pushMusicBuilderRoute,
    type MusicBuilderDestination,
    type MusicBuilderHistory,
  } from "$lib/music/music-builder-routing";
  import { createMusicLibraryController, type MusicDestinationState } from "$lib/music/music-library-controller.svelte";
  import { restoreMusicFocus } from "$lib/music/music-focus-recovery";
  import { onActiveVaultIdentityChange, requireActiveVaultIdentity } from "$lib/vault/active-vault";
  import MusicBuilderAsyncState from "./builder/MusicBuilderAsyncState.svelte";
  import MusicBuilderFilterBar from "./builder/MusicBuilderFilterBar.svelte";
  import MusicBuilderHeader from "./builder/MusicBuilderHeader.svelte";
  import MusicBuilderInspector from "./builder/MusicBuilderInspector.svelte";
  import MusicBuilderNavigation from "./builder/MusicBuilderNavigation.svelte";
  import MusicBuilderOverview from "./builder/MusicBuilderOverview.svelte";
  import MusicVirtualItemList from "./builder/MusicVirtualItemList.svelte";

  let { onBack }: { onBack: () => void } = $props();
  const { t } = getLocalization();
  const library = createMusicLibraryController();
  const inspector = createMusicBuilderInspectorController();
  let root = $state<HTMLElement | null>(null);
  let width = $state(1000);
  let height = $state(680);
  let navigationOpen = $state(false);
  let history = $state<MusicBuilderHistory>(initialMusicBuilderRoute(1, null, { playlistIds: new Set() }));
  let unsubscribeVault: (() => void) | null = null;
  const layout = $derived(projectMusicBuilderLayout({ width, height }));
  const destination = $derived(history.current.destination);
  const selectedItemId = $derived(history.current.inspectorItemId ?? library.currentState.selectedItemId);
  const hasList = $derived(destination.kind === "review" || destination.kind === "library" || destination.kind === "playlist");
  const issueCount = $derived(library.sourceSummaries.reduce((total, source) => total + source.openIssueCount, 0));
  const reviewCount = $derived(destination.kind === "review" ? library.currentWindow.totalCount : library.sourceSummaries.reduce((total, source) => total + source.unreviewedCount, 0));
  const routeContext = $derived({ playlistIds: new Set(library.playlistSummaries.map((playlist) => playlist.id)), itemIds: new Set(library.currentWindow.items.map((item) => item.id)) });

  function destinationTitle(): string {
    if (destination.kind === "review") return t("music.builder.review");
    if (destination.kind === "playlists") return t("music.builder.playlists");
    if (destination.kind === "playlist") return library.playlistSummaries.find((playlist) => playlist.id === destination.playlistId)?.name ?? t("music.builder.playlists");
    if (destination.kind === "library") return t("music.builder.library");
    if (destination.kind === "sources") return t("music.builder.sources");
    if (destination.kind === "issues") return t("music.builder.issues");
    return t("music.builder.soundscapes");
  }

  function primaryLabel(): string | null {
    if (destination.kind === "playlists") return t("music.builder.newPlaylist");
    if (destination.kind === "sources" || destination.kind === "library") return t("music.builder.addMusic");
    return null;
  }

  function observeRoot(node: HTMLElement): { destroy: () => void } {
    root = node;
    const update = () => { width = node.clientWidth; height = node.clientHeight; };
    const observer = new ResizeObserver(update);
    observer.observe(node);
    update();
    return { destroy: () => { observer.disconnect(); root = null; } };
  }

  async function loadVault(vaultId: string): Promise<void> {
    library.setVault(vaultId);
    await library.refresh();
    const remembered = history.current.destination;
    history = initialMusicBuilderRoute(reviewCount, remembered, routeContext);
    library.navigate(history.current.destination);
    await library.refresh();
  }

  async function navigate(next: MusicBuilderDestination): Promise<void> {
    history = pushMusicBuilderRoute(history, { destination: next, inspectorItemId: null }, routeContext);
    library.navigate(next);
    navigationOpen = false;
    inspector.clear();
    await library.refresh();
  }

  async function selectItem(itemId: string): Promise<void> {
    library.patchCurrentState({ selectedItemId: itemId });
    history = pushMusicBuilderRoute(history, { destination, inspectorItemId: itemId }, routeContext);
    await inspector.select(itemId);
  }

  async function closeInspector(): Promise<void> {
    const focusKey = inspector.itemId ? `item:${inspector.itemId}` : null;
    history = { ...history, current: { ...history.current, inspectorItemId: null } };
    inspector.clear();
    library.selectItem(null);
    await tick();
    if (focusKey) restoreMusicFocus(focusKey);
  }

  async function handleBack(): Promise<void> {
    if (history.current.inspectorItemId) { await closeInspector(); return; }
    const previous = backMusicBuilderRoute(history, routeContext);
    if (!previous) { onBack(); return; }
    history = previous;
    library.navigate(history.current.destination);
    await library.refresh();
    if (history.current.inspectorItemId) await inspector.select(history.current.inspectorItemId);
  }

  function patchFilters(patch: Partial<MusicDestinationState>): void {
    library.patchCurrentState({ ...patch, offset: 0 });
    void library.refresh();
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      if (navigationOpen) { event.stopPropagation(); navigationOpen = false; return; }
      if (history.current.inspectorItemId) { event.stopPropagation(); void closeInspector(); }
    }
    if (event.key === "/" && !(event.target instanceof HTMLInputElement) && !(event.target instanceof HTMLTextAreaElement)) {
      event.preventDefault();
      root?.querySelector<HTMLInputElement>("input")?.focus();
    }
  }

  onMount(() => {
    try { void loadVault(requireActiveVaultIdentity()); }
    catch (error) { library.error = error instanceof Error ? error : new Error(String(error)); }
    unsubscribeVault = onActiveVaultIdentityChange((_previous, next) => {
      if (next) void loadVault(next);
      else library.setVault(null);
    });
  });

  onDestroy(() => unsubscribeVault?.());
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<section bind:this={root} use:observeRoot class="builder-root flex h-full min-h-0 flex-col overflow-hidden text-foreground" style="background-color: var(--cal-bg);">
  <MusicBuilderHeader
    title={destinationTitle()}
    search={library.currentState.search}
    busy={library.busy}
    resultCount={hasList ? library.currentWindow.totalCount : null}
    compact={layout.mode === "narrow"}
    primaryLabel={primaryLabel()}
    canUndo={library.undoCount > 0}
    onBack={() => { void handleBack(); }}
    onSearch={(search) => patchFilters({ search })}
    onRefresh={() => { void library.refresh(); }}
    onUndo={() => { void library.undoLast(); }}
    onToggleNavigation={() => navigationOpen = !navigationOpen}
  />

  <div class="relative grid min-h-0 flex-1" class:builder-wide={layout.mode === "wide"} class:builder-medium={layout.mode === "medium"} class:builder-narrow={layout.mode === "narrow"}>
    {#if layout.navigationVisible}
      <MusicBuilderNavigation {destination} playlists={library.playlistSummaries} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} />
    {/if}

    <main class="relative flex min-h-0 min-w-0 flex-col overflow-hidden bg-background/30">
      {#if hasList}
        <MusicBuilderFilterBar
          sourceKind={library.currentState.sourceKind}
          availability={library.currentState.availability}
          reviewState={library.currentState.reviewState}
          sort={library.currentState.sort}
          direction={library.currentState.direction}
          groupBy={library.currentState.groupBy}
          resultCount={library.currentWindow.totalCount}
          onChange={patchFilters}
        />
        {#if library.error && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="error" title={library.error.message} onRetry={() => { void library.refresh(); }} />
        {:else if library.busy && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="loading" />
        {:else if library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState
            kind="empty"
            title={destination.kind === "review" ? t("music.builder.emptyReviewTitle") : t("music.builder.emptyLibraryTitle")}
            description={destination.kind === "review" ? t("music.builder.emptyReviewDescription") : t("music.builder.emptyLibraryDescription")}
          />
        {:else}
          {#if library.busy}<div class="absolute inset-x-0 top-10 z-10 bg-secondary/90 px-3 py-1 text-center text-[0.62rem] text-muted-foreground backdrop-blur-sm">{t("music.builder.staleData")}</div>{/if}
          <MusicVirtualItemList
            items={library.currentWindow.items}
            {selectedItemId}
            selectedItemIds={library.currentState.selectedItemIds}
            initialScrollTop={library.currentState.scrollTop}
            onSelect={(item) => { void selectItem(item.id); }}
            onSelectionChange={(itemIds, activeItemId) => library.setItemSelection(itemIds, activeItemId)}
            onScrollTop={(scrollTop) => library.setScrollTop(scrollTop)}
          />
        {/if}
      {:else}
        <MusicBuilderOverview {destination} playlists={library.playlistSummaries} sources={library.sourceSummaries} issues={library.issues} onNavigate={(next) => { void navigate(next); }} />
      {/if}
    </main>

    {#if layout.mode === "wide"}
      <MusicBuilderInspector controller={inspector} />
    {:else if layout.mode === "medium" && history.current.inspectorItemId}
      <div class="absolute inset-0 z-30 bg-background/45 backdrop-blur-[1px]">
        <button type="button" class="absolute inset-0" onclick={() => { void closeInspector(); }} aria-label={t("music.builder.closeInspector")}></button>
        <div class="relative ml-auto h-full w-[min(23rem,72%)] border-l border-border/70 shadow-2xl" transition:fly={{ x: 36, duration: 160 }}><MusicBuilderInspector controller={inspector} showClose onClose={() => { void closeInspector(); }} /></div>
      </div>
    {:else if layout.mode === "narrow" && history.current.inspectorItemId}
      <div class="absolute inset-0 z-30 bg-background" transition:fly={{ x: 28, duration: 150 }}><MusicBuilderInspector controller={inspector} showClose onClose={() => { void closeInspector(); }} /></div>
    {/if}

    {#if layout.mode === "narrow" && navigationOpen}
      <div class="absolute inset-0 z-40 bg-background/55 p-2 backdrop-blur-sm">
        <button type="button" class="absolute inset-0" onclick={() => navigationOpen = false} aria-label={t("music.builder.compactNavigation")}></button>
        <div class="relative h-full w-fit"><MusicBuilderNavigation compact {destination} playlists={library.playlistSummaries} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} /></div>
      </div>
    {/if}
  </div>
</section>

<style>
  .builder-root { container-type: size; }
  .builder-wide { grid-template-columns: minmax(10.5rem, 0.65fr) minmax(20rem, 1.7fr) minmax(15rem, 0.85fr); }
  .builder-medium { grid-template-columns: minmax(10rem, 0.55fr) minmax(0, 1.8fr); }
  .builder-narrow { grid-template-columns: minmax(0, 1fr); }
  @container (height < 260px) { :global(.builder-header) { min-height: 2.25rem; } }
  @media (prefers-reduced-motion: reduce) { :global(.builder-root *) { scroll-behavior: auto; } }
</style>
