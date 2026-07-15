<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { registerMediaFile, revealLocalFile } from "$lib/api/music";
  import { invalidateMusicArtwork } from "$lib/music/music-artwork-cache";
  import { resetMusicStatistics } from "$lib/api/music-library";
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
  import { createMusicPlaylistController } from "$lib/music/music-playlist-controller.svelte";
  import { createMusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";
  import {
    parseMusicReviewAutoplay,
    parseMusicReviewExitPreference,
    type MusicReviewExitPreference,
  } from "$lib/music/music-review";
  import { musicReviewSource } from "$lib/music/music-review";
  import type { MusicPlaylistMembership } from "$lib/music/library-contracts";
  import type { MusicIssue, MusicSourceCollection } from "$lib/music/library-contracts";
  import type { MusicSourceRefreshPlan } from "$lib/music/music-source-refresh";
  import { restoreMusicFocus } from "$lib/music/music-focus-recovery";
  import { onMusicLibraryChanged } from "$lib/music/music-library-events";
  import { onActiveVaultIdentityChange, requireActiveVaultIdentity } from "$lib/vault/active-vault";
  import { getConfigKey, setConfigKey } from "$lib/vault/config";
  import MusicBuilderAsyncState from "./builder/MusicBuilderAsyncState.svelte";
  import MusicBuilderFilterBar from "./builder/MusicBuilderFilterBar.svelte";
  import MusicBuilderHeader from "./builder/MusicBuilderHeader.svelte";
  import MusicBuilderInspectorSurface from "./builder/MusicBuilderInspectorSurface.svelte";
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
  import MusicPlaylistDialog from "./builder/MusicPlaylistDialog.svelte";
  import MusicPlaylistHeader from "./builder/MusicPlaylistHeader.svelte";
  import MusicBulkActionBar from "./builder/MusicBulkActionBar.svelte";
  import MusicBulkMembershipDialog from "./builder/MusicBulkMembershipDialog.svelte";
  import MusicBulkWeightDialog from "./builder/MusicBulkWeightDialog.svelte";
  import MusicBulkStatusDialog from "./builder/MusicBulkStatusDialog.svelte";
  import type { MusicBuilderInitialAction } from "$lib/music/music-builder-loader";

  let {
    onBack,
    initialAction = null,
    onInitialActionHandled = () => undefined,
  }: {
    onBack: () => void;
    initialAction?: MusicBuilderInitialAction | null;
    onInitialActionHandled?: () => void;
  } = $props();
  const { t } = getLocalization();
  const library = createMusicLibraryController();
  const inspector = createMusicBuilderInspectorController();
  const sources = createMusicSourcesController();
  const audition = createMusicReviewAuditionController();
  const review = createMusicReviewController(library, inspector);
  const playlist = createMusicPlaylistController(library);
  const bulk = createMusicBulkEditController(library);
  const actionableItemIds = $derived(library.currentState.selectedItemIds.filter((itemId) =>
    library.currentWindow.items.some((item) => item.id === itemId),
  ));
  let root = $state<HTMLElement | null>(null);
  let width = $state(1000);
  let height = $state(680);
  let navigationOpen = $state(false);
  let history = $state<MusicBuilderHistory>(initialMusicBuilderRoute(1, null, { playlistIds: new Set() }));
  let unsubscribeVault: (() => void) | null = null;
  let unsubscribeLibraryChanges: (() => void) | null = null;
  let sourceSurface = $state<"add" | "relink" | "remove" | "item-repair" | null>(null);
  let sourceSurfaceCollection = $state<MusicSourceCollection | null>(null);
  let repairItemId = $state<string | null>(null);
  let pendingRefreshPlan = $state<MusicSourceRefreshPlan | null>(null);
  let reviewExitOpen = $state(false);
  let playlistSurface = $state<"create" | "edit" | "duplicate" | "delete" | null>(null);
  let bulkSurface = $state<"memberships" | "weight" | "review" | "snooze" | null>(null);
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
  const playingItemId = $derived(audition.musicPlayer.activeQueueItemIds[audition.musicPlayer.currentQueueIndex] ?? null);
  const playlistNames = $derived(Object.fromEntries(library.playlistSummaries.map((entry) => [entry.id, entry.name])));
  const sourceNames = $derived(Object.fromEntries(library.sourceSummaries.map((entry) => [entry.id, entry.name])));

  $effect(() => {
    const action = initialAction;
    if (!action || !library.vaultId) return;
    if (action === "new-playlist") playlistSurface = "create";
    else if (action.kind === "open-soundscapes") void navigateNow({ kind: "soundscapes" });
    else {
      void navigateNow({ kind: "library" }).then(() => selectItem(action.itemId));
    }
    onInitialActionHandled();
  });

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
    if (history.current.destination.kind === "playlist") await playlist.load(history.current.destination.playlistId);
  }

  function primaryAction(): void {
    if (destination.kind === "playlists") { playlist.clear(); playlistSurface = "create"; return; }
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
    if (next.kind === "playlist") await playlist.load(next.playlistId);
    else playlist.clear();
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
    if (history.current.destination.kind === "playlist") await playlist.load(history.current.destination.playlistId);
    else playlist.clear();
    if (history.current.inspectorItemId) await inspector.select(history.current.inspectorItemId);
  }

  function patchFilters(patch: Partial<MusicDestinationState>): void {
    library.patchCurrentState({ ...patch, offset: 0 });
    void library.refresh();
  }

  async function syncInspectorMetadata(): Promise<void> {
    const item = inspector.detail?.item;
    if (!item) return;
    const listItem = library.currentWindow.items.find((entry) => entry.id === item.id);
    if (listItem) {
      listItem.title = item.titleOverride?.trim() || item.originalTitle;
      listItem.artist = item.artistOverride?.trim() || item.originalArtist;
      listItem.album = item.albumOverride?.trim() || item.originalAlbum;
      listItem.artworkOverride = item.artworkOverride;
      listItem.updatedAt = item.updatedAt;
    }
    if (item.artworkOverride) invalidateMusicArtwork(item.artworkOverride);
    const artworkUrl = item.artworkOverride ? await registerMediaFile(item.artworkOverride, Date.now()).catch(() => null) : null;
    audition.musicPlayer.applyLibraryMetadata(item.id, item.identityKey, item.titleOverride?.trim() || item.originalTitle, artworkUrl);
  }

  async function previewMembership(membership: MusicPlaylistMembership): Promise<void> {
    const detail = inspector.detail;
    if (!detail) return;
    const source = musicReviewSource(detail, sources.bindings);
    if (!source) return;
    await audition.musicPlayer.loadSource({
      ...source,
      startMs: membership.startMs,
      endMs: membership.endMs,
    }, { autoplay: true, resume: false, preserveQueue: true });
    if (membership.volume !== null) await audition.musicPlayer.setVolume(membership.volume);
    if (membership.rate !== null) await audition.musicPlayer.setRate(membership.rate);
  }

  async function playLibraryItem(itemId: string): Promise<void> {
    await inspector.select(itemId);
    if (!inspector.detail) return;
    const source = musicReviewSource(inspector.detail, sources.bindings);
    if (!source) { openItemRepair(itemId); return; }
    audition.musicPlayer.clearContextPlayback();
    await audition.musicPlayer.loadSource(source, { autoplay: true, resume: false });
    audition.musicPlayer.activeQueueItemIds = [itemId];
  }

  async function openBulkMemberships(removeCurrent = false): Promise<void> {
    if (actionableItemIds.length === 0) return;
    bulkSurface = "memberships";
    const loaded = await bulk.open(actionableItemIds, library.playlistSummaries);
    if (!loaded) return;
    if (removeCurrent && destination.kind === "playlist") bulk.toggle(destination.playlistId);
  }

  async function openItemMembership(itemId: string): Promise<void> {
    library.setItemSelection([itemId], itemId);
    bulkSurface = "memberships";
    await bulk.open([itemId], library.playlistSummaries);
  }

  async function openBulkWeight(): Promise<void> {
    if (destination.kind !== "playlist" || actionableItemIds.length === 0) return;
    bulkSurface = "weight";
    await bulk.open(actionableItemIds, library.playlistSummaries);
  }

  function closeBulkSurface(): void {
    bulkSurface = null;
    bulk.clear();
  }

  function openBulkStatus(surface: "review" | "snooze"): void {
    bulk.useSelection(actionableItemIds);
    bulkSurface = surface;
  }

  function openItemStatus(itemId: string, surface: "review" | "snooze"): void {
    library.setItemSelection([itemId], itemId);
    bulk.useSelection([itemId]);
    bulkSurface = surface;
  }

  async function showInspectorFile(itemId: string): Promise<void> {
    if (inspector.itemId !== itemId) await inspector.select(itemId);
    const location = inspector.detail?.locations.find((entry) => entry.availability === "available");
    if (!location) { openItemRepair(itemId); return; }
    const folder = sources.bindings.find((binding) => binding.rootId === location.rootId)?.folderPath;
    if (!folder) { openItemRepair(itemId); return; }
    const separator = folder.includes("\\") && !folder.includes("/") ? "\\" : "/";
    const path = `${folder.replace(/[\\/]+$/, "")}${separator}${location.relativePath.replace(/[\\/]+/g, separator)}`;
    await revealLocalFile(path);
  }

  async function resetInspectorStatistics(itemId: string): Promise<void> {
    await resetMusicStatistics({ itemIds: [itemId], resetAggregates: true, resetRecentSelections: true });
    await inspector.select(null);
    await inspector.select(itemId);
    await library.refresh();
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      if (reviewExitOpen) { event.preventDefault(); event.stopPropagation(); reviewExitOpen = false; pendingReviewExit = null; return; }
      if (bulkSurface) { event.stopPropagation(); closeBulkSurface(); return; }
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
    unsubscribeLibraryChanges = onMusicLibraryChanged(() => { void library.refresh(); });
  });

  onDestroy(() => {
    unsubscribeVault?.();
    unsubscribeLibraryChanges?.();
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
      <MusicBuilderNavigation {destination} playlists={library.playlistSummaries} playingPlaylistId={audition.musicPlayer.activePlaylistId} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} />
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
        {#if destination.kind === "playlist" && playlist.detail}
          {@const summary = library.playlistSummaries.find((entry) => entry.id === destination.playlistId)}
          {#if summary}
            <MusicPlaylistHeader
              detail={playlist.detail}
              {summary}
              playing={audition.musicPlayer.activePlaylistId === destination.playlistId}
              onPlay={() => { void playlist.play(sources.bindings); }}
              onEdit={() => playlistSurface = "edit"}
              onDuplicate={() => playlistSurface = "duplicate"}
              onDelete={() => { void playlist.inspectDelete().then((loaded) => { if (loaded) playlistSurface = "delete"; }); }}
            />
            {#if playlist.playbackIssue === "no-eligible-items"}
              <div class="mx-3 mt-2 flex flex-wrap items-center gap-2 rounded-lg border border-warning/30 bg-warning/8 px-3 py-2 text-[0.68rem] text-warning" role="status">
                <span class="min-w-0 flex-1">{t("music.builder.noEligiblePlaylistItems")}</span>
                <button type="button" onclick={() => { void navigate({ kind: "issues" }); }} class="h-7 rounded-md bg-secondary px-2.5 font-medium text-secondary-foreground">{t("music.builder.openIssues")}</button>
              </div>
            {/if}
          {/if}
        {/if}
        <MusicBuilderFilterBar
          sourceKind={library.currentState.sourceKind}
          availability={library.currentState.availability}
          reviewState={library.currentState.reviewState}
          sort={library.currentState.sort}
          direction={library.currentState.direction}
          groupBy={library.currentState.groupBy}
          resultCount={library.currentWindow.totalCount}
          sourceCollectionId={library.currentState.sourceCollectionId}
          membershipPlaylistId={library.currentState.membershipPlaylistId}
          snoozed={library.currentState.snoozed}
          sources={library.sourceSummaries}
          playlists={library.playlistSummaries}
          playlistMode={destination.kind === "playlist"}
          onChange={patchFilters}
        />
        {#if library.currentState.groupBy !== "none" && library.currentWindow.groups.length > 0}
          <div class="flex shrink-0 gap-1.5 overflow-x-auto border-b border-border/40 px-3 py-1.5" aria-label={t("music.builder.reviewGroups")}>
            {#each library.currentWindow.groups as group (group.key)}<span class="shrink-0 rounded-full bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground">{group.key} · {group.count}</span>{/each}
          </div>
        {/if}
        {#if library.error && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="error" title={library.error.message} onRetry={() => { void library.refresh(); }} />
        {:else if library.busy && library.currentWindow.items.length === 0}
          <MusicBuilderAsyncState kind="loading" />
        {:else if library.currentWindow.items.length === 0}
          {#if destination.kind === "playlist" && (library.playlistSummaries.find((entry) => entry.id === destination.playlistId)?.totalCount ?? 0) === 0}
            <div class="grid min-h-0 flex-1 place-items-center p-4 text-center">
              <div class="max-w-sm rounded-2xl border border-border/60 bg-card/55 p-5 shadow-sm">
                <h2 class="text-sm font-semibold">{t("music.builder.emptyPlaylistActionTitle")}</h2>
                <p class="mt-1.5 text-xs leading-relaxed text-muted-foreground">{t("music.builder.emptyPlaylistActionDescription")}</p>
                <div class="mt-4 flex flex-wrap justify-center gap-2"><button type="button" onclick={() => { void navigate({ kind: "library" }); }} class="h-8 rounded-md bg-primary px-3 text-xs font-semibold text-primary-foreground">{t("music.builder.addFromLibrary")}</button><button type="button" onclick={() => { void navigate({ kind: "review" }); }} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium">{t("music.builder.startReview")}</button></div>
              </div>
            </div>
          {:else}
            <MusicBuilderAsyncState
              kind="empty"
              title={destination.kind === "playlist" ? t("music.builder.noPlaylistFilterResults") : t("music.builder.emptyLibraryTitle")}
              description={destination.kind === "playlist" ? t("music.builder.adjustPlaylistFilters") : t("music.builder.emptyLibraryDescription")}
            />
          {/if}
        {:else}
          {#if library.busy}<div class="absolute inset-x-0 top-10 z-10 bg-secondary/90 px-3 py-1 text-center text-[0.62rem] text-muted-foreground backdrop-blur-sm">{t("music.builder.staleData")}</div>{/if}
          <MusicVirtualItemList
            items={library.currentWindow.items}
            {selectedItemId}
            selectedItemIds={library.currentState.selectedItemIds}
            playlistMode={destination.kind === "playlist"}
            reorderEnabled={destination.kind === "playlist" && library.currentState.sort === "manual-position" && library.currentState.groupBy === "none"}
            {playingItemId}
            initialScrollTop={library.currentState.scrollTop}
            onSelect={(item) => { void selectItem(item.id); }}
            onSelectionChange={(itemIds, activeItemId) => library.setItemSelection(itemIds, activeItemId)}
            onScrollTop={(scrollTop) => library.setScrollTop(scrollTop)}
            onReorder={(item, targetIndex) => { void playlist.reorder(item.id, targetIndex, sources.bindings); }}
            onPlay={(item) => {
              const activeIndex = audition.musicPlayer.activeQueueItemIds.indexOf(item.id);
              if (audition.musicPlayer.activePlaylistId === (destination.kind === "playlist" ? destination.playlistId : null) && activeIndex >= 0) void audition.musicPlayer.playQueueItem(activeIndex);
              else if (destination.kind === "playlist") void playlist.play(sources.bindings, item.id);
              else void playLibraryItem(item.id);
            }}
            hasMore={library.currentWindow.items.length < library.currentWindow.totalCount}
            loadingMore={library.loadingMore}
            onLoadMore={() => { void library.loadMore(); }}
          />
          {#if actionableItemIds.length > 1}
            <MusicBulkActionBar
              selectedCount={actionableItemIds.length}
              allVisibleSelected={library.currentWindow.items.length > 0 && library.currentWindow.items.every((item) => library.currentState.selectedItemIds.includes(item.id))}
              playlistMode={destination.kind === "playlist"}
              onSelectVisible={() => library.setItemSelection(library.currentWindow.items.map((item) => item.id), library.currentState.selectedItemId)}
              onClear={() => library.setItemSelection([], null)}
              onAddToPlaylists={() => { void openBulkMemberships(); }}
              onRemove={() => { void openBulkMemberships(true); }}
              onWeight={() => { void openBulkWeight(); }}
              onSnooze={() => openBulkStatus("snooze")}
              onReviewState={() => openBulkStatus("review")}
              onAvailability={actionableItemIds.some((itemId) => library.currentWindow.items.find((item) => item.id === itemId)?.availability !== "available") ? () => { void navigate({ kind: "issues" }); } : undefined}
            />
          {/if}
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
        <MusicBuilderOverview {destination} playlists={library.playlistSummaries} sources={library.sourceSummaries} issues={library.issues} onNavigate={(next) => { void navigate(next); }} onPrimary={primaryAction} />
      {/if}
    </main>

    {#if destination.kind !== "review"}
      <MusicBuilderInspectorSurface
        mode={layout.mode}
        open={Boolean(history.current.inspectorItemId)}
        controller={inspector}
        activePlaylistId={destination.kind === "playlist" ? destination.playlistId : null}
        {playlistNames}
        {sourceNames}
        closeLabel={t("music.builder.closeInspector")}
        onClose={() => { void closeInspector(); }}
        onPlay={(itemId) => { if (destination.kind === "playlist") void playlist.play(sources.bindings, itemId); else void playLibraryItem(itemId); }}
        onShowFile={(itemId) => { void showInspectorFile(itemId); }}
        onReviewState={(itemId) => openItemStatus(itemId, "review")}
        onSnooze={(itemId) => openItemStatus(itemId, "snooze")}
        onEditMembership={(itemId) => { void openItemMembership(itemId); }}
        onResetStatistics={(itemId) => { void resetInspectorStatistics(itemId); }}
        onRepair={openItemRepair}
        onMetadataSaved={syncInspectorMetadata}
        onPreviewMembership={(membership) => { void previewMembership(membership); }}
        onOpenSource={() => { void navigate({ kind: "sources" }); }}
      />
    {/if}

    {#if layout.mode === "narrow" && navigationOpen}
      <div class="absolute inset-0 z-40 bg-background/55 p-2 backdrop-blur-sm">
        <button type="button" class="absolute inset-0" onclick={() => navigationOpen = false} aria-label={t("music.builder.compactNavigation")}></button>
        <div class="relative h-full w-fit"><MusicBuilderNavigation compact {destination} playlists={library.playlistSummaries} playingPlaylistId={audition.musicPlayer.activePlaylistId} {reviewCount} {issueCount} onNavigate={(next) => { void navigate(next); }} /></div>
      </div>
    {/if}

    {#if sourceSurface === "add"}
      <MusicAddSourceDialog controller={sources} onClose={closeSourceSurface} onSaved={() => { closeSourceSurface(); void library.refresh(); }} />
    {:else if sourceSurface === "relink" && sourceSurfaceCollection}
      <MusicRelinkWizard controller={sources} collection={sourceSurfaceCollection} onClose={closeSourceSurface} onApplied={() => { sourceSurface = null; sourceSurfaceCollection = null; void library.refresh(); }} />
    {:else if sourceSurface === "remove" && sourceSurfaceCollection}
      <MusicSourceRemovalDialog controller={sources} collection={sourceSurfaceCollection} onClose={closeSourceSurface} onRemoved={() => { sourceSurface = null; sourceSurfaceCollection = null; void library.refresh(); }} />
    {:else if sourceSurface === "item-repair" && repairItemId}
      <MusicItemRepairDialog controller={sources} itemId={repairItemId} onClose={closeSourceSurface} onRepaired={() => { void library.refresh(); void inspector.select(repairItemId); void playlist.refreshActivePlayback(sources.bindings); }} />
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
    {#if playlistSurface}
      <MusicPlaylistDialog
        controller={playlist}
        mode={playlistSurface}
        playlists={library.playlistSummaries}
        activeInPlayer={Boolean(playlist.detail && audition.musicPlayer.activePlaylistId === playlist.detail.id)}
        onClose={() => playlistSurface = null}
        onSaved={(playlistId) => { playlistSurface = null; void navigateNow({ kind: "playlist", playlistId }); }}
        onDeleted={(replacementPlaylistId) => {
          const deletedPlaylistId = destination.kind === "playlist" ? destination.playlistId : null;
          playlistSurface = null;
          if (deletedPlaylistId && audition.musicPlayer.activePlaylistId === deletedPlaylistId) {
            audition.musicPlayer.detachDeletedPlaylist(deletedPlaylistId);
            if (replacementPlaylistId) {
              void navigateNow({ kind: "playlist", playlistId: replacementPlaylistId }).then(() => playlist.play(sources.bindings));
              return;
            }
          }
          const neighboringPlaylistId = replacementPlaylistId ?? library.playlistSummaries[0]?.id ?? null;
          if (neighboringPlaylistId) void navigateNow({ kind: "playlist", playlistId: neighboringPlaylistId });
          else void navigateNow({ kind: "playlists" });
        }}
      />
    {/if}
    {#if bulkSurface === "memberships"}
      <MusicBulkMembershipDialog
        controller={bulk}
        playlists={library.playlistSummaries}
        onClose={closeBulkSurface}
        onSaved={() => { closeBulkSurface(); library.setItemSelection([], null); void playlist.refreshActivePlayback(sources.bindings); }}
      />
    {:else if bulkSurface === "weight" && destination.kind === "playlist"}
      <MusicBulkWeightDialog
        controller={bulk}
        playlistId={destination.playlistId}
        onClose={closeBulkSurface}
        onSaved={() => { closeBulkSurface(); void playlist.refreshActivePlayback(sources.bindings); }}
      />
    {:else if bulkSurface === "review" || bulkSurface === "snooze"}
      <MusicBulkStatusDialog
        controller={bulk}
        mode={bulkSurface}
        playlistId={destination.kind === "playlist" ? destination.playlistId : null}
        onClose={closeBulkSurface}
        onSaved={() => { closeBulkSurface(); library.setItemSelection([], null); void playlist.refreshActivePlayback(sources.bindings); }}
      />
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
