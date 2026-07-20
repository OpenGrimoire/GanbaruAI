<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Disc3 from "@lucide/svelte/icons/disc-3";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Pause from "@lucide/svelte/icons/pause";
  import PanelLeft from "@lucide/svelte/icons/panel-left";
  import Play from "@lucide/svelte/icons/play";
  import Slash from "@lucide/svelte/icons/slash";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import X from "@lucide/svelte/icons/x";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import IconPicker from "$lib/components/icon-picker/IconPicker.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import type { MusicLibraryController } from "$lib/music/music-library-controller.svelte";
  import type { MusicReviewAuditionController } from "$lib/music/music-review-audition.svelte";
  import type { MusicReviewController } from "$lib/music/music-review-controller.svelte";
  import type { MusicReviewWorkspaceViewState } from "$lib/music/music-builder-view-state";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import type { MusicIssue } from "$lib/music/library-contracts";
  import {
    isMusicReviewEditableTarget,
    musicReviewArtworkDataUrl,
  } from "$lib/music/music-review";
  import {
    firstMusicReviewTreeItemId,
    musicReviewTreeItemIds,
    nextPendingMusicReviewTreeItemId,
  } from "$lib/music/music-review-tree";
  import { clampRate, formatPlaybackTime } from "$lib/music/playback";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import MusicPlaylistIcon from "./MusicPlaylistIcon.svelte";
  import MusicPlaylistManager from "./MusicPlaylistManager.svelte";
  import MusicPlaylistPicker from "./MusicPlaylistPicker.svelte";

  let {
    library,
    inspector,
    sources,
    audition,
    review,
    autoplay,
    onAutoplayChange,
    onOpenPlayer,
    showPanelButton = false,
    onOpenPanel = () => undefined,
    onEditPlaylist,
    onDeletePlaylist,
    onReorderPlaylists,
    issue = null,
    onRepairIssue = () => undefined,
    viewState,
  }: {
    library: MusicLibraryController;
    inspector: MusicBuilderInspectorController;
    sources: MusicSourcesController;
    audition: MusicReviewAuditionController;
    review: MusicReviewController;
    autoplay: boolean;
    onAutoplayChange: (value: boolean) => void;
    onOpenPlayer: () => void;
    showPanelButton?: boolean;
    onOpenPanel?: () => void;
    onEditPlaylist: (playlistId: string) => void;
    onDeletePlaylist: (playlistId: string) => void;
    onReorderPlaylists: (playlistIds: string[]) => Promise<boolean>;
    issue?: MusicIssue | null;
    onRepairIssue?: (issue: MusicIssue) => void;
    viewState: MusicReviewWorkspaceViewState;
  } = $props();

  const { t } = getLocalization();
  let surface = $state<HTMLElement | null>(null);
  let lastSelectedId = $state<string | null>(null);
  let lastAutoplayedId = $state<string | null>(null);
  let newPlaylistNameInput = $state<HTMLInputElement | null>(null);
  let checklistRoot = $state<HTMLElement | null>(null);
  let prefetchedArtworkUrls = $state<Record<string, string>>({});
  let prefetchedArtworkReadyIds = $state<Set<string>>(new Set());
  let artworkPrefetchGeneration = 0;
  let preparingNext = $state(false);
  let ignoreConfirmOpen = $state(false);
  const sessionSkippedIds = $derived(new Set(viewState.sessionSkippedIds));
  const reviewItemsFullyLoaded = $derived(
    library.currentWindow.items.length >= library.currentWindow.totalCount,
  );
  const reviewItemIds = $derived(musicReviewTreeItemIds(library.currentWindow.items));
  const initialReviewItemId = $derived(reviewItemsFullyLoaded
    ? firstMusicReviewTreeItemId(library.currentWindow.items)
    : null);
  const initialReviewItem = $derived(initialReviewItemId
    ? library.currentWindow.items.find((entry) => entry.id === initialReviewItemId) ?? null
    : null);
  const item = $derived(library.selectedItem ?? initialReviewItem);
  const detail = $derived(inspector.detail?.item.id === item?.id ? inspector.detail : null);
  const checkedIds = $derived(new Set(detail?.memberships.map((membership) => membership.playlistId) ?? []));
  const membershipSignature = $derived([...checkedIds].sort().join("\n"));
  const membershipsChanged = $derived(Boolean(item)
    && viewState.membershipBaselines[item!.id] !== undefined
    && viewState.membershipBaselines[item!.id] !== membershipSignature);
  const needsSave = $derived(Boolean(item) && (item!.reviewState !== "reviewed" || membershipsChanged));
  const reviewTreeIndex = $derived(item ? reviewItemIds.indexOf(item.id) : -1);
  const reviewedCount = $derived(library.currentWindow.items.filter((entry) => entry.reviewState === "reviewed").length);
  const player = $derived(audition.musicPlayer);
  const previewTitle = $derived(detail
    ? detail.item.titleOverride ?? detail.item.originalTitle
    : item?.title ?? "");
  const previewArtist = $derived(detail
    ? (detail.item.artistOverride ?? detail.item.originalArtist) || t("music.builder.noArtist")
    : item?.artist || t("music.builder.noArtist"));
  const reviewPlayerReady = $derived(audition.reviewItemId === item?.id);
  const prefetchedArtworkUrl = $derived(item ? prefetchedArtworkUrls[item.id] ?? null : null);
  const previewDurationMs = $derived(reviewPlayerReady
    ? player.snapshot.durationMs
    : detail?.item.durationMs ?? item?.durationMs ?? 0);
  const seekSliderProgress = $derived(reviewPlayerReady && player.progressMax > 0
    ? `${Math.min(100, Math.max(0, (player.progressValue / player.progressMax) * 100))}%`
    : "0%");

  $effect(() => {
    if (library.loadingMore || library.loadMoreError || library.currentWindow.items.length >= library.currentWindow.totalCount) return;
    void library.loadMore();
  });

  function availabilityLabel(): string {
    if (!detail) return "";
    if (detail.item.availability === "available") return t("music.builder.available");
    if (detail.item.availability === "missing") return t("music.builder.missing");
    if (detail.item.availability === "unavailable") return t("music.builder.unavailable");
    if (detail.item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
  }

  function attentionLabel(): string {
    if (detail?.item.availability !== "available") return availabilityLabel();
    return issue?.message ?? "";
  }

  $effect(() => {
    if (!surface) return;
    return player.claimSurface("playlist-builder-review", surface, 100);
  });

  $effect(() => {
    const nextId = item?.id ?? null;
    if (!nextId || nextId === lastSelectedId) return;
    lastSelectedId = nextId;
    library.selectItem(nextId);
    void inspector.select(nextId);
  });

  $effect(() => {
    if (reviewTreeIndex < 0) return;
    const nearbyIds = reviewItemIds
      .slice(Math.max(0, reviewTreeIndex - 1), reviewTreeIndex + 4)
    const generation = ++artworkPrefetchGeneration;
    const bindings = [...sources.bindings];
    void inspector.prefetch(nearbyIds).then(async (details) => {
      const entries = await Promise.all(details.map(async (entry) => {
        const url = await loadDecodedReviewArtwork(entry, bindings);
        return url ? [entry.item.id, url] as const : null;
      }));
      if (generation !== artworkPrefetchGeneration) return;
      prefetchedArtworkUrls = Object.fromEntries(entries.filter((entry) => entry !== null));
      prefetchedArtworkReadyIds = new Set(details.map((entry) => entry.item.id));
    });
  });

  $effect(() => {
    if (!detail || lastAutoplayedId === detail.item.id) return;
    lastAutoplayedId = detail.item.id;
    void audition.preview(detail, sources.bindings, autoplay);
  });

  $effect(() => {
    if (!detail || viewState.membershipBaselines[detail.item.id] !== undefined) return;
    viewState.membershipBaselines = { ...viewState.membershipBaselines, [detail.item.id]: membershipSignature };
  });

  onDestroy(() => {
    inspector.clear();
  });


  function openInlineCreate(): void {
    viewState.inlineCreateOpen = true;
    review.createError = null;
    void tick().then(() => newPlaylistNameInput?.focus());
  }

  async function createPlaylistAndAdd(): Promise<void> {
    const playlistId = await review.createPlaylistAndAdd(viewState.newPlaylistName, viewState.newPlaylistIcon);
    if (!playlistId) return;
    viewState.newPlaylistName = "";
    viewState.newPlaylistIcon = "lucide:list-music";
    viewState.inlineCreateOpen = false;
    await tick();
    checklistRoot?.querySelector<HTMLElement>(`[data-review-playlist-id="${playlistId}"]`)?.focus();
  }

  async function loadDecodedReviewArtwork(
    entry: NonNullable<typeof inspector.detail>,
    bindings = sources.bindings,
  ): Promise<string | null> {
    const url = await musicReviewArtworkDataUrl(entry, bindings);
    if (url && typeof Image !== "undefined") {
      const image = new Image();
      image.src = url;
      await image.decode().catch(() => undefined);
    }
    return url;
  }

  async function ensureReviewArtwork(itemId: string | null): Promise<void> {
    if (!itemId) return;
    const details = await inspector.prefetch([itemId]);
    if (prefetchedArtworkReadyIds.has(itemId)) return;
    const entry = details.find((candidate) => candidate.item.id === itemId);
    if (!entry) return;
    const url = await loadDecodedReviewArtwork(entry);
    if (url) prefetchedArtworkUrls = { ...prefetchedArtworkUrls, [itemId]: url };
    prefetchedArtworkReadyIds = new Set([...prefetchedArtworkReadyIds, itemId]);
  }

  async function finishReviewState(reviewState: "reviewed" | "ignored"): Promise<void> {
    if (preparingNext || review.actionBusy) return;
    const nextItemId = reviewTreeIndex >= 0 ? reviewItemIds[reviewTreeIndex + 1] ?? null : null;
    preparingNext = true;
    try {
      await ensureReviewArtwork(nextItemId);
      await review.changeReviewState(reviewState, null, nextItemId);
    } finally {
      preparingNext = false;
    }
  }

  function confirmIgnore(): void {
    ignoreConfirmOpen = false;
    void finishReviewState("ignored");
  }

  async function skipCurrentItem(): Promise<void> {
    if (!item || preparingNext || review.actionBusy) return;
    preparingNext = true;
    try {
      const skipped = new Set(sessionSkippedIds).add(item.id);
      let nextItemId = nextPendingMusicReviewTreeItemId(library.currentWindow.items, item.id, skipped);
      if (!nextItemId) {
        viewState.sessionSkippedIds = [];
        nextItemId = nextPendingMusicReviewTreeItemId(library.currentWindow.items, item.id, new Set());
      } else {
        viewState.sessionSkippedIds = [...skipped];
      }
      await ensureReviewArtwork(nextItemId);
      if (nextItemId) inspector.selectCached(nextItemId);
      library.selectItem(nextItemId);
    } finally {
      preparingNext = false;
    }
  }

  async function continueCurrentItem(): Promise<void> {
    if (!item || preparingNext || review.actionBusy) return;
    const nextItemId = reviewTreeIndex >= 0 ? reviewItemIds[reviewTreeIndex + 1] ?? null : null;
    viewState.membershipBaselines = { ...viewState.membershipBaselines, [item.id]: membershipSignature };
    preparingNext = true;
    try {
      await ensureReviewArtwork(nextItemId);
      if (nextItemId) inspector.selectCached(nextItemId);
      library.selectItem(nextItemId);
    } finally {
      preparingNext = false;
    }
  }

  async function saveAndContinue(): Promise<void> {
    if (!item || !needsSave || preparingNext || review.actionBusy) return;
    if (item.reviewState !== "reviewed") {
      viewState.membershipBaselines = { ...viewState.membershipBaselines, [item.id]: membershipSignature };
      await finishReviewState("reviewed");
      return;
    }
    await continueCurrentItem();
  }

  async function selectRelative(delta: number): Promise<void> {
    const targetId = reviewItemIds[reviewTreeIndex + delta];
    if (!targetId) return;
    library.selectItem(targetId);
    lastSelectedId = null;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.isComposing || event.altKey || isMusicReviewEditableTarget(event.target)) return;
    const modified = event.ctrlKey || event.metaKey;
    if (event.code === "Space" && !modified) {
      event.preventDefault();
      if (detail && audition.reviewItemId !== detail.item.id) void audition.preview(detail, sources.bindings, true);
      else void player.togglePlay();
    } else if (event.key === "ArrowLeft" && modified) {
      event.preventDefault(); void selectRelative(-1);
    } else if (event.key === "ArrowRight" && modified) {
      event.preventDefault(); void selectRelative(1);
    } else if (event.key === "Enter" && modified) {
      event.preventDefault();
      if (needsSave) void saveAndContinue();
      else void continueCurrentItem();
    } else if (modified) {
      return;
    } else if (event.key === "ArrowLeft") {
      event.preventDefault(); void player.seekByMs(-10_000);
    } else if (event.key === "ArrowRight") {
      event.preventDefault(); void player.seekByMs(10_000);
    } else if (event.key === "ArrowUp") {
      event.preventDefault(); void player.adjustVolume(0.05);
    } else if (event.key === "ArrowDown") {
      event.preventDefault(); void player.adjustVolume(-0.05);
    } else if (event.key.toLowerCase() === "m") {
      event.preventDefault(); void player.toggleMute();
    } else if (event.key === "+" || event.key === "=" || event.code === "NumpadAdd") {
      event.preventDefault(); void player.setRate(clampRate(player.snapshot.rate + 0.25));
    } else if (event.key === "-" || event.code === "NumpadSubtract") {
      event.preventDefault(); void player.setRate(clampRate(player.snapshot.rate - 0.25));
    } else if (!event.shiftKey && /^[07-9]$/.test(event.key) && player.snapshot.durationMs) {
      event.preventDefault(); void player.seekToMs(Math.round(player.snapshot.durationMs * Number(event.key) / 10));
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="review-main flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
  <section class="review-audition min-h-0 overflow-y-auto px-4 pb-3 pt-2" data-music-scrollable="true">
    <div class="flex items-center justify-between gap-3">
      {#if showPanelButton}<button type="button" onclick={onOpenPanel} class="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-muted-foreground hover:bg-secondary hover:text-foreground" aria-label={t("music.builder.openContextPanel")} title={t("music.builder.openContextPanel")}><PanelLeft size={14} /></button>{/if}
      <button type="button" onclick={onOpenPlayer} class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-full bg-secondary px-2.5 text-[0.7rem]" aria-label={t("music.backToPlayer")} data-music-focus-key="builder:back-to-player"><ChevronLeft size={14} />{t("music.backToPlayer")}</button>
      <p class="min-w-0 flex-1 truncate text-center text-[0.68rem] font-medium text-muted-foreground" role="status" aria-live="polite">{t("music.builder.reviewProgress", reviewedCount, library.currentWindow.totalCount)}</p>
      <div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1">
        <button type="button" onclick={() => onAutoplayChange(!autoplay)} aria-pressed={autoplay} class="inline-flex h-8 items-center gap-1.5 rounded-md px-2 text-[0.65rem] text-foreground transition-colors hover:bg-secondary">
          {#if autoplay}
            <Play size={13} />
          {:else}
            <span class="relative size-3.25 shrink-0" aria-hidden="true"><Play class="absolute inset-0" size={13} /><Slash class="absolute inset-0" size={13} /></span>
          {/if}
          {autoplay ? t("music.builder.reviewAutoplayOn") : t("music.builder.reviewAutoplayOff")}
        </button>
        <button type="button" onclick={() => ignoreConfirmOpen = true} disabled={!detail || review.actionBusy || preparingNext} class="inline-flex h-8 items-center gap-1.5 rounded-md px-2 text-[0.65rem] text-foreground hover:bg-secondary"><X size={13} />{t("music.builder.ignore")}</button>
      </div>
    </div>
    {#if library.currentState.groupBy !== "none" && library.currentWindow.groups.length > 0}
      <div class="mt-2 flex gap-1.5 overflow-x-auto pb-1" aria-label={t("music.builder.reviewGroups")}>
        {#each library.currentWindow.groups as group (group.key)}
          <span class="shrink-0 rounded-full bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground">{group.key} · {group.count}</span>
        {/each}
      </div>
    {/if}

    {#if item}
      <div class="review-player mt-4 flex min-w-0 items-center gap-4">
        <div bind:this={surface} class="review-media relative grid h-28 w-28 shrink-0 place-items-center overflow-hidden rounded-xl">
          {#if item.sourceKind === "local-file" && !player.localHasVideo}
            {#if prefetchedArtworkUrl}
              <img src={prefetchedArtworkUrl} alt="" class="absolute inset-0 h-full w-full object-contain" draggable="false" onload={() => player.handleArtworkLoaded()} />
            {:else if reviewPlayerReady && player.currentArtworkUrl}
              <img src={player.currentArtworkUrl} alt="" class="absolute inset-0 h-full w-full object-contain" draggable="false" onload={() => player.handleArtworkLoaded()} onerror={() => player.handleArtworkError()} />
            {:else}
              <div class="grid h-full w-full place-items-center rounded-xl bg-primary/10 text-primary">
                <Disc3 size={38} strokeWidth={1.3} />
              </div>
            {/if}
          {:else if !player.currentSource || !reviewPlayerReady}
            <div class="grid h-full w-full place-items-center rounded-xl bg-primary/10 text-primary">
              <Disc3 size={38} strokeWidth={1.3} />
            </div>
          {/if}
        </div>

        <div class="min-w-0 flex-1">
          <div class="min-w-0">
            <h2 class="truncate text-base font-semibold">{previewTitle}</h2>
            <p class="mt-0.5 truncate text-xs text-muted-foreground">{previewArtist}</p>
            {#if detail && (detail.item.availability !== "available" || issue)}
              <div class="mt-1 flex min-w-0 items-center gap-1.5 text-[0.65rem]">
                <TriangleAlert size={12} class="shrink-0 text-destructive" />
                <span class="min-w-0 truncate text-muted-foreground">{attentionLabel()}</span>
                {#if issue?.actionRequired}<button type="button" onclick={() => onRepairIssue(issue)} class="shrink-0 font-semibold text-primary hover:underline">{t("music.builder.repair")}</button>{/if}
              </div>
            {/if}
          </div>

          <div class="mt-4 flex items-center gap-3">
            <div class="min-w-0 flex-1 text-[0.68rem] tabular-nums text-muted-foreground">
              <input type="range" min="0" max={reviewPlayerReady ? player.progressMax : previewDurationMs} value={reviewPlayerReady ? player.progressValue : 0} disabled={!reviewPlayerReady} oninput={(event) => { void player.seekToMs(Number(event.currentTarget.value)); }} class="music-seek-slider music-seek-slider-edge-aligned block" style={`--music-seek-progress: ${seekSliderProgress}; --music-seek-thumb-size: 1rem; --music-seek-track-height: 0.3rem;`} aria-label={t("music.seek")} />
              <div class="mt-1 flex justify-between">
                <span>{formatPlaybackTime(reviewPlayerReady ? player.snapshot.positionMs : 0)}</span>
                <span>{formatPlaybackTime(previewDurationMs)}</span>
              </div>
            </div>
            <button type="button" onclick={() => { if (detail && !reviewPlayerReady) void audition.preview(detail, sources.bindings, true); else void player.togglePlay(); }} disabled={!detail || item.availability !== "available"} class="review-play shrink-0" aria-label={reviewPlayerReady && player.isPlaying ? t("music.pause") : t("music.play")} title={t("music.builder.reviewPlayTitle", formatShortcut("Space"))}>
              {#if reviewPlayerReady && player.isPlaying}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </section>

  <section class="review-classify flex min-h-0 flex-col">
    {#if viewState.managingPlaylists}
      <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-3" data-music-scrollable="true">
        <MusicPlaylistManager
          playlists={library.playlistSummaries}
          onEdit={onEditPlaylist}
          onDelete={onDeletePlaylist}
          onReorder={onReorderPlaylists}
          onDone={() => viewState.managingPlaylists = false}
        />
      </div>
    {:else}
      <div class="shrink-0 p-3">
        <h2 class="text-sm font-semibold">{t("music.builder.classifyPlaylists")}</h2>
      {#if viewState.inlineCreateOpen}
        <form class="mt-2" onsubmit={(event) => { event.preventDefault(); void createPlaylistAndAdd(); }}>
          <div class="flex items-center gap-2">
            <IconPicker value={viewState.newPlaylistIcon} onChange={(value) => viewState.newPlaylistIcon = value} ariaLabel={t("music.builder.selectPlaylistIcon")} showUpload={false}>
              {#snippet trigger({ open, toggle })}
                <button
                  type="button"
                  class={`grid h-8 w-8 shrink-0 place-items-center rounded-lg bg-secondary text-foreground transition-colors hover:bg-accent ${open ? "bg-accent" : ""}`}
                  aria-label={t("music.builder.selectPlaylistIcon")}
                  aria-expanded={open}
                  title={t("music.builder.selectPlaylistIcon")}
                  onclick={toggle}
                >
                  <MusicPlaylistIcon icon={viewState.newPlaylistIcon} size={16} strokeWidth={1.6} />
                </button>
              {/snippet}
            </IconPicker>
            <input bind:this={newPlaylistNameInput} bind:value={viewState.newPlaylistName} aria-label={t("music.builder.inlinePlaylistName")} class="h-8 min-w-0 flex-1 rounded-md border border-border/70 bg-background px-2.5 text-xs outline-none focus:border-primary" placeholder={t("music.builder.inlinePlaylistName")} />
          </div>
          {#if review.createError}<p class="mt-1.5 text-[0.65rem] text-destructive" role="alert">{review.createError}</p>{/if}
          <div class="mt-2 flex justify-end gap-2">
            <button type="button" onclick={() => { viewState.inlineCreateOpen = false; review.createError = null; }} class="h-8 rounded-md bg-secondary px-2.5 text-xs font-medium">{t("music.builder.cancel")}</button>
            <button type="submit" disabled={!viewState.newPlaylistName.trim() || review.creatingPlaylist} class="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-2.5 text-xs font-medium text-primary-foreground disabled:opacity-40"><ListPlus size={14} />{t("music.builder.createAndAdd")}</button>
          </div>
        </form>
      {:else}
        <div class="mt-2 flex flex-wrap items-center gap-2">
          <button type="button" onclick={openInlineCreate} class="inline-flex h-8 items-center gap-1.5 rounded-full bg-primary px-3 text-xs font-medium text-primary-foreground"><ListPlus size={14} />{t("music.builder.newPlaylist")}</button>
          <button type="button" onclick={() => { viewState.inlineCreateOpen = false; review.createError = null; viewState.managingPlaylists = true; }} class="inline-flex h-8 items-center gap-1.5 rounded-full bg-secondary px-3 text-xs font-medium text-foreground"><Pencil size={13} />{t("music.builder.managePlaylists")}</button>
        </div>
      {/if}
      </div>

      <div bind:this={checklistRoot} class="flex min-h-0 flex-1 flex-col">
        <MusicPlaylistPicker
          playlists={library.playlistSummaries}
          {checkedIds}
          onToggle={(playlist) => { void review.toggleMembership(playlist); }}
          errors={review.membershipErrors}
        />
      </div>
    {/if}

    <div class="review-actions grid shrink-0 grid-cols-2 gap-3 p-3">
      {#if item?.reviewState === "reviewed"}
        <button type="button" onclick={() => { void continueCurrentItem(); }} disabled={!detail || review.actionBusy || preparingNext} class="review-action h-full w-full border border-border/70 bg-background text-foreground">{t("music.builder.continue")}</button>
      {:else}
        <button type="button" onclick={() => { void skipCurrentItem(); }} disabled={!detail || review.actionBusy || preparingNext} class="review-action h-full w-full border border-border/70 bg-background text-foreground">{t("music.builder.skipTrack")}</button>
      {/if}
      <button type="button" onclick={() => { void saveAndContinue(); }} disabled={!detail || !needsSave || review.actionBusy || preparingNext} class="review-action bg-primary text-primary-foreground" title={t("music.builder.markReviewedTitle", formatShortcut("Mod + Enter"))}><Check size={14} />{t("music.builder.saveAndContinue")}<ChevronRight size={14} /></button>
    </div>
  </section>
</div>

{#if ignoreConfirmOpen}
  <ConfirmDialog
    title={t("music.builder.ignoreTrackTitle")}
    message={t("music.builder.ignoreTrackDescription")}
    confirmLabel={t("music.builder.ignoreTrackConfirm")}
    cancelLabel={t("common.cancel")}
    onConfirm={confirmIgnore}
    onCancel={() => ignoreConfirmOpen = false}
  />
{/if}

<style>
  .review-main { min-height: 0; }
  .review-audition { flex: 0 0 auto; }
  .review-classify { min-height: 14rem; flex: 1 1 0; }
  .review-play { display: grid; height: 2.5rem; width: 2.5rem; place-items: center; border-radius: 9999px; background: var(--primary); color: var(--primary-foreground); }
  .review-play:disabled { opacity: 0.4; }
  .review-action { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.375rem; border-radius: 0.5rem; padding: 0 0.5rem; font-size: 0.72rem; font-weight: 600; }
  .review-action:disabled { cursor: not-allowed; opacity: 0.4; }
  @container (width < 620px) {
    .review-main { min-height: 32rem; flex: 1 0 auto; overflow: visible; }
    .review-audition, .review-classify { min-height: auto; overflow: visible; }
    .review-audition { flex: 0 0 auto; padding: 0.625rem; }
    .review-classify { flex: 1 0 18rem; border-left: 0; }
    .review-classify > :global(div:nth-child(2)) { min-height: 9rem; }
    .review-actions { position: sticky; bottom: 0; z-index: 5; }
  }
  @container (width < 380px) {
    .review-player { align-items: flex-start; }
    .review-media { height: 5rem; width: 5rem; }
    .review-play { height: 2rem; width: 2rem; }
  }
  @container (width >= 620px) and (width < 860px) {
    .review-classify { min-height: 15rem; }
  }
  @container (height < 300px) and (width >= 620px) {
    .review-audition { padding-block: 0.5rem; }
    .review-audition > :global(.aspect-video) { max-height: 7rem; }
  }
</style>
