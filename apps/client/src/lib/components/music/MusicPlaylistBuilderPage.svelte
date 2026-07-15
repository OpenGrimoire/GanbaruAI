<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import { fly } from "svelte/transition";
  import Check from "@lucide/svelte/icons/check";
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
  import { createMusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import { createMusicReviewAuditionController } from "$lib/music/music-review-audition.svelte";
  import { createMusicReviewController } from "$lib/music/music-review-controller.svelte";
  import {
    parseMusicReviewAutoplay,
    parseMusicReviewExitPreference,
    type MusicReviewExitPreference,
  } from "$lib/music/music-review";
  import type { MusicIssue, MusicSourceCollection } from "$lib/music/library-contracts";
  import type { MusicSourceRefreshPlan } from "$lib/music/music-source-refresh";
  import { restoreMusicFocus } from "$lib/music/music-focus-recovery";
  import { onActiveVaultIdentityChange, requireActiveVaultIdentity } from "$lib/vault/active-vault";
  import { getConfigKey, setConfigKey } from "$lib/vault/config";
  import MusicBuilderAsyncState from "./builder/MusicBuilderAsyncState.svelte";
  import MusicBuilderFilterBar from "./builder/MusicBuilderFilterBar.svelte";
  import MusicBuilderHeader from "./builder/MusicBuilderHeader.svelte";
  import MusicBuilderInspector from "./builder/MusicBuilderInspector.svelte";
  import MusicBuilderNavigation from "./builder/MusicBuilderNavigation.svelte";
  import MusicBuilderOverview from "./builder/MusicBuilderOverview.svelte";
  import MusicVirtualItemList from "./builder/MusicVirtualItemList.svelte";
  import MusicAddSourceDialog from "./builder/MusicAddSourceDialog.svelte";
  import MusicIssueBrowser from "./builder/MusicIssueBrowser.svelte";
  import MusicItemRepairDialog from "./builder/MusicItemRepairDialog.svelte";
  import MusicNetworkRefreshDialog from "./builder/MusicNetworkRefreshDialog.svelte";
  import MusicRelinkWizard from "./builder/MusicRelinkWizard.svelte";
  import MusicSourceRemovalDialog from "./builder/MusicSourceRemovalDialog.svelte";
  import MusicSourcesDashboard from "./builder/MusicSourcesDashboard.svelte";
  import MusicReviewWorkspace from "./builder/MusicReviewWorkspace.svelte";

  let { onBack }: { onBack: () => void } = $props();
  const { t } = getLocalization();
  const library = createMusicLibraryController();
  const inspector = createMusicBuilderInspectorController();
  const sources = createMusicSourcesController();
  const audition = createMusicReviewAuditionController();
  const review = createMusicReviewController(library, inspector);
  let root = $state<HTMLElement | null>(null);
  let width = $state(1000);
  let height = $state(680);
  let navigationOpen = $state(false);
  let history = $state<MusicBuilderHistory>(initialMusicBuilderRoute(1, null, { playlistIds: new Set() }));
  let unsubscribeVault: (() => void) | null = null;
  let sourceSurface = $state<"add" | "relink" | "remove" | "item-repair" | null>(null);
  let sourceSurfaceCollection = $state<MusicSourceCollection | null>(null);
  let repairItemId = $state<string | null>(null);
  let pendingRefreshPlan = $state<MusicSourceRefreshPlan | null>(null);
  let reviewExitOpen = $state(false);
  let rememberReviewExit = $state(false);
  let reviewAutoplay = $state(parseMusicReviewAutoplay(getConfigKey<unknown>("music.review.autoplay", undefined)));
  let reviewExitPreference = $state<MusicReviewExitPreference>(parseMusicReviewExitPreference(getConfigKey<unknown>("music.review.exitPreference", undefined)));
  let pendingReviewExit: (() => void) | null = null;
  const layout = $derived(projectMusicBuilderLayout({ width, height }));
  const destination = $derived(history.current.destination);
  const selectedItemId = $derived(history.current.inspectorItemId ?? library.currentState.selectedItemId);
  const hasList = $derived(destination.kind === "library" || destination.kind === "playlist");
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
    sources.setVault(vaultId);
    await Promise.all([library.refresh(), sources.load()]);
    const remembered = history.current.destination;
    history = initialMusicBuilderRoute(reviewCount, remembered, routeContext);
    library.navigate(history.current.destination);
    await library.refresh();
  }

  function primaryAction(): void {
    if (destination.kind === "sources" || destination.kind === "library") sourceSurface = "add";
  }

  function closeSourceSurface(): void {
    sources.cancelResolution();
    if (sourceSurface === "relink" && sources.relinkPlan?.state === "ready") {
      void sources.cancelRelink();
    }
    sourceSurface = null;
    sourceSurfaceCollection = null;
    repairItemId = null;
    sources.clearItemRepair();
  }

  function requestSourceRefresh(collectionIds?: string[]): void {
    const plan = sources.prepareRefresh(collectionIds);
    if (plan.requiresNetworkConfirmation) pendingRefreshPlan = plan;
    else void runSourceRefresh(plan, false);
  }

  async function runSourceRefresh(plan: MusicSourceRefreshPlan, allowNetwork: boolean): Promise<void> {
    pendingRefreshPlan = null;
    await sources.runRefresh(plan, allowNetwork);
    await library.refresh();
  }

  function collectionById(collectionId: string): MusicSourceCollection | null {
    return sources.collections.find((collection) => collection.id === collectionId) ?? null;
  }

  async function openRemoval(collectionId: string): Promise<void> {
    const collection = collectionById(collectionId);
    if (!collection) return;
    sourceSurfaceCollection = collection;
    await sources.inspectRemoval(collectionId);
    sourceSurface = "remove";
  }

  function openRelink(collectionId: string): void {
    const collection = collectionById(collectionId);
    if (!collection?.localRootId) return;
    sourceSurfaceCollection = collection;
    sourceSurface = "relink";
  }

  function repairIssue(issue: MusicIssue): void {
    if (issue.rootId) {
      const collection = sources.collections.find((entry) => entry.localRootId === issue.rootId);
      if (collection) openRelink(collection.id);
      return;
    }
    if (issue.itemId) { openItemRepair(issue.itemId); return; }
    if (issue.collectionId) requestSourceRefresh([issue.collectionId]);
  }

  function openItemRepair(itemId: string): void {
    repairItemId = itemId;
    sources.clearItemRepair();
    sourceSurface = "item-repair";
  }

  async function navigateNow(next: MusicBuilderDestination): Promise<void> {
    history = pushMusicBuilderRoute(history, { destination: next, inspectorItemId: null }, routeContext);
    library.navigate(next);
    navigationOpen = false;
    inspector.clear();
    await library.refresh();
  }

  function requestReviewExit(continuation: () => void): void {
    if (destination.kind !== "review" || !audition.active) { continuation(); return; }
    if (reviewExitPreference === "restore") { void audition.restore().then(continuation); return; }
    if (reviewExitPreference === "keep") { audition.keep(); continuation(); return; }
    pendingReviewExit = continuation;
    rememberReviewExit = false;
    reviewExitOpen = true;
  }

  function navigate(next: MusicBuilderDestination): void {
    requestReviewExit(() => { void navigateNow(next); });
  }

  async function resolveReviewExit(choice: "restore" | "keep"): Promise<void> {
    if (rememberReviewExit) {
      reviewExitPreference = choice;
      setConfigKey("music.review.exitPreference", choice);
    }
    if (choice === "restore") await audition.restore();
    else audition.keep();
    const continuation = pendingReviewExit;
    pendingReviewExit = null;
    reviewExitOpen = false;
    continuation?.();
  }

  function setReviewAutoplay(value: boolean): void {
    reviewAutoplay = value;
    setConfigKey("music.review.autoplay", value);
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
    if (!previous) { requestReviewExit(onBack); return; }
    if (destination.kind === "review" && previous.current.destination.kind !== "review") {
      requestReviewExit(() => {
        history = previous;
        library.navigate(history.current.destination);
        void library.refresh();
      });
      return;
    }
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
      if (reviewExitOpen) { event.preventDefault(); event.stopPropagation(); reviewExitOpen = false; pendingReviewExit = null; return; }
      if (sourceSurface || pendingRefreshPlan) { event.stopPropagation(); closeSourceSurface(); pendingRefreshPlan = null; return; }
      if (navigationOpen) { event.stopPropagation(); navigationOpen = false; return; }
      if (history.current.inspectorItemId) { event.stopPropagation(); void closeInspector(); return; }
      event.stopPropagation();
      void handleBack();
    }
    if (event.key === "/" && !(event.target instanceof HTMLInputElement) && !(event.target instanceof HTMLTextAreaElement)) {
      if (destination.kind === "review") return;
      event.preventDefault();
      root?.querySelector<HTMLInputElement>("input")?.focus();
    }
  }

  onMount(() => {
    try { void loadVault(requireActiveVaultIdentity()); }
    catch (error) { library.error = error instanceof Error ? error : new Error(String(error)); }
    unsubscribeVault = onActiveVaultIdentityChange((_previous, next) => {
      if (next) void loadVault(next);
      else { library.setVault(null); sources.setVault(null); }
    });
  });

  onDestroy(() => {
    unsubscribeVault?.();
    if (audition.active) void audition.restore();
  });
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
    onPrimary={primaryAction}
    onToggleNavigation={() => navigationOpen = !navigationOpen}
  />

  <div class="relative grid min-h-0 flex-1" class:builder-wide={layout.mode === "wide"} class:builder-medium={layout.mode === "medium"} class:builder-narrow={layout.mode === "narrow"} class:builder-review={destination.kind === "review"}>
    {#if layout.navigationVisible}
      <MusicBuilderNavigation {destination} playlists={library.playlistSummaries} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} />
    {/if}

    <main class="relative flex min-h-0 min-w-0 flex-col overflow-hidden bg-background/30">
      {#if destination.kind === "review"}
        {#if library.error && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="error" title={library.error.message} onRetry={() => { void library.refresh(); }} />
        {:else if library.busy && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="loading" />
        {:else if library.currentWindow.items.length === 0}
          <div class="grid h-full min-h-40 place-items-center p-5 text-center">
            <div class="max-w-sm rounded-2xl border border-border/60 bg-card/55 p-5 shadow-sm">
              <div class="mx-auto grid h-12 w-12 place-items-center rounded-2xl bg-success/12 text-success"><Check size={21} strokeWidth={1.8} aria-hidden="true" /></div>
              <h2 class="mt-3 text-sm font-semibold">{t("music.builder.emptyReviewTitle")}</h2>
              <p class="mt-1.5 text-xs leading-relaxed text-muted-foreground">{t("music.builder.emptyReviewDescription")}</p>
              <div class="mt-4 flex flex-wrap justify-center gap-2">
                {#if library.currentState.reviewState !== null}
                  <button type="button" onclick={() => { library.patchCurrentState({ reviewState: null }); void library.refresh(); }} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">{t("music.builder.reviewDeferred")}</button>
                {/if}
                <button type="button" onclick={() => navigate({ kind: "library" })} class="h-8 rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground">{t("music.builder.browseLibrary")}</button>
              </div>
            </div>
          </div>
        {:else}
          <MusicReviewWorkspace {library} {inspector} {sources} {audition} {review} autoplay={reviewAutoplay} onAutoplayChange={setReviewAutoplay} />
        {/if}
      {:else if hasList}
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
            title={destination.kind === "playlist" ? t("music.builder.emptyPlaylistsTitle") : t("music.builder.emptyLibraryTitle")}
            description={destination.kind === "playlist" ? t("music.builder.emptyPlaylistsDescription") : t("music.builder.emptyLibraryDescription")}
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
      {:else if destination.kind === "sources"}
        <MusicSourcesDashboard
          controller={sources}
          summaries={library.sourceSummaries}
          onAdd={() => sourceSurface = "add"}
          onRefreshAll={() => requestSourceRefresh()}
          onRefreshSource={(collectionId) => requestSourceRefresh([collectionId])}
          onRelink={openRelink}
          onRemove={(collectionId) => { void openRemoval(collectionId); }}
          onOpenIssues={() => { void navigate({ kind: "issues" }); }}
        />
      {:else if destination.kind === "issues"}
        <MusicIssueBrowser issues={library.issues} onRepair={repairIssue} onRefresh={() => requestSourceRefresh()} />
      {:else}
        <MusicBuilderOverview {destination} playlists={library.playlistSummaries} sources={library.sourceSummaries} issues={library.issues} onNavigate={(next) => { void navigate(next); }} />
      {/if}
    </main>

    {#if layout.mode === "wide" && destination.kind !== "review"}
      <MusicBuilderInspector controller={inspector} onRepair={openItemRepair} />
    {:else if layout.mode === "medium" && history.current.inspectorItemId}
      <div class="absolute inset-0 z-30 bg-background/45 backdrop-blur-[1px]">
        <button type="button" class="absolute inset-0" onclick={() => { void closeInspector(); }} aria-label={t("music.builder.closeInspector")}></button>
        <div class="relative ml-auto h-full w-[min(23rem,72%)] border-l border-border/70 shadow-2xl" transition:fly={{ x: 36, duration: 160 }}><MusicBuilderInspector controller={inspector} showClose onClose={() => { void closeInspector(); }} onRepair={openItemRepair} /></div>
      </div>
    {:else if layout.mode === "narrow" && history.current.inspectorItemId}
      <div class="absolute inset-0 z-30 bg-background" transition:fly={{ x: 28, duration: 150 }}><MusicBuilderInspector controller={inspector} showClose onClose={() => { void closeInspector(); }} onRepair={openItemRepair} /></div>
    {/if}

    {#if layout.mode === "narrow" && navigationOpen}
      <div class="absolute inset-0 z-40 bg-background/55 p-2 backdrop-blur-sm">
        <button type="button" class="absolute inset-0" onclick={() => navigationOpen = false} aria-label={t("music.builder.compactNavigation")}></button>
        <div class="relative h-full w-fit"><MusicBuilderNavigation compact {destination} playlists={library.playlistSummaries} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} /></div>
      </div>
    {/if}

    {#if sourceSurface === "add"}
      <MusicAddSourceDialog controller={sources} onClose={closeSourceSurface} onSaved={() => { closeSourceSurface(); void library.refresh(); }} />
    {:else if sourceSurface === "relink" && sourceSurfaceCollection}
      <MusicRelinkWizard controller={sources} collection={sourceSurfaceCollection} onClose={closeSourceSurface} onApplied={() => { sourceSurface = null; sourceSurfaceCollection = null; void library.refresh(); }} />
    {:else if sourceSurface === "remove" && sourceSurfaceCollection}
      <MusicSourceRemovalDialog controller={sources} collection={sourceSurfaceCollection} onClose={closeSourceSurface} onRemoved={() => { sourceSurface = null; sourceSurfaceCollection = null; void library.refresh(); }} />
    {:else if sourceSurface === "item-repair" && repairItemId}
      <MusicItemRepairDialog controller={sources} itemId={repairItemId} onClose={closeSourceSurface} onRepaired={() => { void library.refresh(); void inspector.select(repairItemId); }} />
    {/if}
    {#if pendingRefreshPlan}
      <MusicNetworkRefreshDialog onlineCount={pendingRefreshPlan.onlineCount} onClose={() => pendingRefreshPlan = null} onLocalOnly={() => { void runSourceRefresh(pendingRefreshPlan!, false); }} onContinue={() => { void runSourceRefresh(pendingRefreshPlan!, true); }} />
    {/if}
    {#if reviewExitOpen}
      <div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm" role="presentation">
        <div class="w-full max-w-sm rounded-xl border border-border/70 bg-card p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="music-review-exit-title" tabindex="-1">
          <h2 id="music-review-exit-title" class="text-sm font-semibold">{t("music.builder.reviewExitTitle")}</h2>
          <p class="mt-1 text-xs leading-relaxed text-muted-foreground">{t("music.builder.reviewExitDescription")}</p>
          <label class="mt-3 flex items-center gap-2 text-xs text-muted-foreground"><input type="checkbox" bind:checked={rememberReviewExit} class="accent-primary" />{t("music.builder.rememberExitChoice")}</label>
          <div class="mt-4 grid grid-cols-2 gap-2">
            <button type="button" onclick={() => { void resolveReviewExit("keep"); }} class="h-9 rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">{t("music.builder.keepReviewMusic")}</button>
            <button type="button" onclick={() => { void resolveReviewExit("restore"); }} class="h-9 rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground">{t("music.builder.restorePreviousMusic")}</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</section>

<style>
  .builder-root { container-type: size; }
  .builder-wide { grid-template-columns: minmax(10.5rem, 0.65fr) minmax(20rem, 1.7fr) minmax(15rem, 0.85fr); }
  .builder-medium { grid-template-columns: minmax(10rem, 0.55fr) minmax(0, 1.8fr); }
  .builder-narrow { grid-template-columns: minmax(0, 1fr); }
  .builder-wide.builder-review { grid-template-columns: minmax(10.5rem, 0.55fr) minmax(0, 2.45fr); }
  @container (height < 260px) { :global(.builder-header) { min-height: 2.25rem; } }
  @media (prefers-reduced-motion: reduce) { :global(.builder-root *) { scroll-behavior: auto; } }
</style>
