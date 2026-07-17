<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Disc3 from "@lucide/svelte/icons/disc-3";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicBuilderInspectorController } from "$lib/music/music-builder-inspector.svelte";
  import type { MusicLibraryController } from "$lib/music/music-library-controller.svelte";
  import {
    MUSIC_CONTEXT_BOUNDARY_EVENT,
    type MusicReviewAuditionController,
  } from "$lib/music/music-review-audition.svelte";
  import type { MusicReviewController } from "$lib/music/music-review-controller.svelte";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import {
    isMusicReviewEditableTarget,
  } from "$lib/music/music-review";
  import type { MusicWeight } from "$lib/music/library-contracts";
  import { clampRate, formatPlaybackTime } from "$lib/music/playback";
  import { cn } from "$lib/utils";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import { getMusicFocusAdvisory } from "$lib/music/music-focus-guidance";
  import MusicPlaylistPicker from "./MusicPlaylistPicker.svelte";
  import MusicReviewTree from "./MusicReviewTree.svelte";

  let {
    library,
    inspector,
    sources,
    audition,
    review,
    autoplay,
    onAutoplayChange,
    onAssignSelection,
    onOpenPlayer,
  }: {
    library: MusicLibraryController;
    inspector: MusicBuilderInspectorController;
    sources: MusicSourcesController;
    audition: MusicReviewAuditionController;
    review: MusicReviewController;
    autoplay: boolean;
    onAutoplayChange: (value: boolean) => void;
    onAssignSelection: (itemIds: string[]) => void;
    onOpenPlayer: () => void;
  } = $props();

  const { t } = getLocalization();
  let surface = $state<HTMLElement | null>(null);
  let playlistSearch = $state("");
  let newPlaylistName = $state("");
  let newPlaylistDescription = $state("");
  let inlineCreateOpen = $state(false);
  let laterMenuOpen = $state(false);
  let laterButton = $state<HTMLButtonElement | null>(null);
  let laterPopover = $state<HTMLElement | null>(null);
  let laterDateInput = $state<HTMLInputElement | null>(null);
  let laterDate = $state("");
  let lastSelectedId = $state<string | null>(null);
  let lastAutoplayedId = $state<string | null>(null);
  let sessionTotal = $state(0);
  let playlistSearchInput = $state<HTMLInputElement | null>(null);
  let newPlaylistNameInput = $state<HTMLInputElement | null>(null);
  let checklistRoot = $state<HTMLElement | null>(null);
  let dismissedAdvisoryItemId = $state<string | null>(null);
  let guidanceEnabled = $state(true);
  const focusGuidanceStorageKey = "ganbaru.music.focus-guidance.enabled";
  const item = $derived(library.selectedItem ?? library.currentWindow.items[0] ?? null);
  const detail = $derived(inspector.detail?.item.id === item?.id ? inspector.detail : null);
  const checkedIds = $derived(new Set(detail?.memberships.map((membership) => membership.playlistId) ?? []));
  const membershipWeights = $derived(Object.fromEntries(
    (detail?.memberships ?? []).map((membership) => [membership.playlistId, membership.weight]),
  ) as Record<string, MusicWeight>);
  const membershipFocusFits = $derived(Object.fromEntries(
    (detail?.memberships ?? []).map((membership) => [membership.playlistId, membership.focusFit]),
  ) as Record<string, import("$lib/music/library-contracts").MusicFocusFit>);
  const currentIndex = $derived(item ? library.currentWindow.items.findIndex((entry) => entry.id === item.id) : -1);
  const progressCurrent = $derived(Math.max(1, sessionTotal - library.currentWindow.totalCount + currentIndex + 1));
  const player = $derived(audition.musicPlayer);
  const seekSliderProgress = $derived(player.progressMax > 0
    ? `${Math.min(100, Math.max(0, (player.progressValue / player.progressMax) * 100))}%`
    : "0%");
  const focusAdvisory = $derived(detail && guidanceEnabled && dismissedAdvisoryItemId !== detail.item.id
    ? getMusicFocusAdvisory(detail.signals, checkedIds, library.playlistSummaries)
    : null);

  $effect(() => {
    if (library.loadingMore || library.loadMoreError || library.currentWindow.items.length >= library.currentWindow.totalCount) return;
    void library.loadMore();
  });

  $effect(() => {
    if (!laterMenuOpen) return;
    const handlePointerDown = (event: PointerEvent): void => {
      if (!(event.target instanceof Node)) return;
      if (laterPopover?.contains(event.target) || laterButton?.contains(event.target)) return;
      closeLaterMenu(false);
    };
    window.addEventListener("pointerdown", handlePointerDown, true);
    return () => window.removeEventListener("pointerdown", handlePointerDown, true);
  });

  $effect(() => {
    if (sessionTotal === 0 && library.currentWindow.totalCount > 0) {
      sessionTotal = library.currentWindow.totalCount;
    }
  });

  function availabilityLabel(): string {
    if (!detail) return "";
    if (detail.item.availability === "available") return t("music.builder.available");
    if (detail.item.availability === "missing") return t("music.builder.missing");
    if (detail.item.availability === "unavailable") return t("music.builder.unavailable");
    if (detail.item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
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
    if (!detail || lastAutoplayedId === detail.item.id) return;
    lastAutoplayedId = detail.item.id;
    void audition.preview(detail, sources.bindings, autoplay);
  });

  onDestroy(() => {
    inspector.clear();
  });

  onMount(() => {
    guidanceEnabled = localStorage.getItem(focusGuidanceStorageKey) !== "false";
    const handleBoundary = (event: Event) => {
      if (!(event instanceof CustomEvent)) return;
      const owner = event.detail?.owner;
      if (owner === "calendar-event" || owner === "pomodoro") audition.supersedeForBoundary(owner);
    };
    window.addEventListener(MUSIC_CONTEXT_BOUNDARY_EVENT, handleBoundary);
    return () => window.removeEventListener(MUSIC_CONTEXT_BOUNDARY_EVENT, handleBoundary);
  });

  function disableFocusGuidance(): void {
    guidanceEnabled = false;
    localStorage.setItem(focusGuidanceStorageKey, "false");
  }

  function enableFocusGuidance(): void {
    guidanceEnabled = true;
    localStorage.setItem(focusGuidanceStorageKey, "true");
  }

  function openInlineCreate(): void {
    inlineCreateOpen = true;
    review.createError = null;
    void tick().then(() => newPlaylistNameInput?.focus());
  }

  function setIncludeLater(include: boolean): void {
    sessionTotal = 0;
    library.patchCurrentState({ reviewState: include ? null : "unreviewed", offset: 0, selectedItemId: null });
    inspector.clear();
    lastSelectedId = null;
    void library.refresh();
  }

  async function createPlaylistAndAdd(): Promise<void> {
    const playlistId = await review.createPlaylistAndAdd(newPlaylistName, newPlaylistDescription);
    if (!playlistId) return;
    newPlaylistName = "";
    newPlaylistDescription = "";
    inlineCreateOpen = false;
    await tick();
    checklistRoot?.querySelector<HTMLElement>(`[data-review-playlist-id="${playlistId}"]`)?.focus();
  }

  async function finishReviewState(reviewState: "reviewed" | "deferred" | "ignored", deferredUntil: number | null = null): Promise<void> {
    if (await review.changeReviewState(reviewState, deferredUntil)) {
      lastSelectedId = null;
      lastAutoplayedId = null;
    }
  }

  function deferCurrentItem(): void {
    const deferredUntil = laterDate ? new Date(`${laterDate}T09:00:00`).getTime() : null;
    closeLaterMenu();
    void finishReviewState("deferred", Number.isFinite(deferredUntil) ? deferredUntil : null);
  }

  async function openLaterMenu(): Promise<void> {
    laterMenuOpen = true;
    await tick();
    laterDateInput?.focus();
  }

  function closeLaterMenu(restoreFocus = true): void {
    laterMenuOpen = false;
    if (restoreFocus && laterButton?.isConnected) queueMicrotask(() => laterButton?.focus());
  }

  function handleLaterFocusOut(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (!(next instanceof Node) || laterPopover?.contains(next) || laterButton?.contains(next)) return;
    closeLaterMenu(false);
  }

  async function selectRelative(delta: number): Promise<void> {
    const target = library.currentWindow.items[currentIndex + delta];
    if (!target) return;
    library.selectItem(target.id);
    lastSelectedId = null;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape" && laterMenuOpen) {
      event.preventDefault();
      event.stopPropagation();
      closeLaterMenu();
      return;
    }
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
      event.preventDefault(); void finishReviewState("reviewed");
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
    } else if (event.key === "/") {
      event.preventDefault(); playlistSearchInput?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="review-workspace grid min-h-0 flex-1 overflow-hidden">
  <MusicReviewTree
    items={library.currentWindow.items}
    totalCount={library.currentWindow.totalCount}
    loading={library.loadingMore}
    activeItemId={item?.id ?? null}
    onActivate={(itemId) => library.selectItem(itemId)}
    onAssign={onAssignSelection}
  />
  <div class="review-main flex min-h-0 min-w-0 flex-col overflow-hidden">
  <section class="review-audition min-h-0 overflow-y-auto px-4 py-3" data-music-scrollable="true">
    <div class="flex items-center justify-between gap-3">
      <div class="flex min-w-0 items-center gap-2">
        <button type="button" onclick={onOpenPlayer} class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-secondary px-2.5 text-[0.7rem] font-medium" aria-label={t("music.backToPlayer")}><ChevronLeft size={14} />{t("music.mediaPlayer")}</button>
        <p class="truncate text-[0.68rem] font-medium text-muted-foreground" role="status" aria-live="polite">{t("music.builder.reviewProgress", progressCurrent, sessionTotal)}</p>
      </div>
      <div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1">
        {#if audition.active}
          <button type="button" onclick={() => { void audition.restore(); }} class="h-7 rounded-md bg-secondary px-2 text-[0.68rem] font-medium text-secondary-foreground">{t("music.builder.returnPreviousPlayback")}</button>
        {/if}
        <button type="button" onclick={() => setIncludeLater(library.currentState.reviewState !== null)} aria-pressed={library.currentState.reviewState === null} class={cn("h-7 rounded-full px-2.5 text-[0.65rem] font-medium", library.currentState.reviewState === null ? "bg-primary/12 text-primary" : "bg-secondary/70 text-muted-foreground")}>{t("music.builder.includeLater")}</button>
        <button type="button" onclick={() => onAutoplayChange(!autoplay)} aria-pressed={autoplay} class={cn("h-7 rounded-full px-2.5 text-[0.65rem] font-medium", autoplay ? "bg-primary/12 text-primary" : "bg-secondary/70 text-muted-foreground")}>{t("music.builder.reviewAutoplay")}</button>
        {#if !guidanceEnabled}<button type="button" onclick={enableFocusGuidance} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem] text-secondary-foreground">{t("music.builder.enableFocusGuidance")}</button>{/if}
        <button type="button" onclick={() => { void finishReviewState("ignored"); }} disabled={!detail || review.actionBusy} class="h-7 rounded-md px-2 text-[0.65rem] text-muted-foreground hover:bg-secondary disabled:opacity-40">{t("music.builder.ignore")}</button>
      </div>
    </div>
    {#if library.currentState.groupBy !== "none" && library.currentWindow.groups.length > 0}
      <div class="mt-2 flex gap-1.5 overflow-x-auto pb-1" aria-label={t("music.builder.reviewGroups")}>
        {#each library.currentWindow.groups as group (group.key)}
          <span class="shrink-0 rounded-full bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground">{group.key} · {group.count}</span>
        {/each}
      </div>
    {/if}

    {#if inspector.busy && !detail}
      <div class="mt-3 h-20 animate-pulse rounded-lg bg-card motion-reduce:animate-none"></div>
    {:else if detail}
      <div class="review-player mt-4 flex min-w-0 items-center gap-4">
        <div bind:this={surface} class="review-media relative grid h-28 w-28 shrink-0 place-items-center overflow-hidden rounded-xl">
          {#if audition.reviewItemId === detail.item.id && !player.localHasVideo && detail.item.sourceKind === "local-file"}
            {#if player.currentArtworkUrl}
              <img src={player.currentArtworkUrl} alt="" class="absolute inset-0 h-full w-full object-contain" draggable="false" />
            {:else}
              <Disc3 class="text-muted-foreground" size={38} strokeWidth={1.3} />
            {/if}
          {:else if !player.currentSource || audition.reviewItemId !== detail.item.id}
            <Disc3 class="text-muted-foreground" size={38} strokeWidth={1.3} />
          {/if}
        </div>

        <div class="min-w-0 flex-1">
          <div class="min-w-0">
            <h2 class="truncate text-base font-semibold">{detail.item.titleOverride ?? detail.item.originalTitle}</h2>
            <p class="mt-0.5 truncate text-xs text-muted-foreground">{(detail.item.artistOverride ?? detail.item.originalArtist) || t("music.builder.noArtist")}</p>
          </div>

          <div class="mt-4 flex items-center gap-3">
            <div class="min-w-0 flex-1 text-[0.68rem] tabular-nums text-muted-foreground">
              <input type="range" min="0" max={player.progressMax} value={player.progressValue} disabled={audition.reviewItemId !== detail.item.id} oninput={(event) => { void player.seekToMs(Number(event.currentTarget.value)); }} class="music-seek-slider music-seek-slider-edge-aligned block disabled:opacity-40" style={`--music-seek-progress: ${seekSliderProgress}; --music-seek-thumb-size: 1rem; --music-seek-track-height: 0.3rem;`} aria-label={t("music.seek")} />
              <div class="mt-1 flex justify-between">
                <span>{formatPlaybackTime(player.snapshot.positionMs)}</span>
                <span>{formatPlaybackTime(player.snapshot.durationMs)}</span>
              </div>
            </div>
            <button type="button" onclick={() => { if (audition.reviewItemId !== detail.item.id) void audition.preview(detail, sources.bindings, true); else void player.togglePlay(); }} disabled={detail.item.availability !== "available"} class="review-play shrink-0" aria-label={player.isPlaying ? t("music.pause") : t("music.play")} title={t("music.builder.reviewPlayTitle", formatShortcut("Space"))}>
              {#if player.isPlaying && audition.reviewItemId === detail.item.id}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </section>

  <section class="review-classify flex min-h-0 flex-col">
    <div class="shrink-0 p-3">
      <div class="flex items-center justify-between gap-2">
        <div><h2 class="text-sm font-semibold">{t("music.builder.classifyPlaylists")}</h2><p class="text-[0.68rem] text-muted-foreground">{t("music.builder.classifyHint")}</p></div>
        {#if checkedIds.size > 0}<button type="button" onclick={() => { void review.clearMemberships(); }} class="text-[0.68rem] text-muted-foreground hover:text-foreground">{t("music.builder.clearMemberships")}</button>{/if}
      </div>
      {#if inlineCreateOpen}
        <form class="mt-2 rounded-lg border border-border/70 bg-background/75 p-2" onsubmit={(event) => { event.preventDefault(); void createPlaylistAndAdd(); }}>
          <input bind:this={newPlaylistNameInput} bind:value={newPlaylistName} aria-label={t("music.builder.inlinePlaylistName")} class="h-8 w-full rounded-md border border-border/70 bg-background px-2.5 text-xs outline-none focus:border-primary" placeholder={t("music.builder.inlinePlaylistName")} />
          <input bind:value={newPlaylistDescription} aria-label={t("music.builder.inlinePlaylistDescription")} class="mt-2 h-8 w-full rounded-md border border-border/70 bg-background px-2.5 text-xs outline-none focus:border-primary" placeholder={t("music.builder.inlinePlaylistDescription")} />
          {#if review.createError}<p class="mt-1.5 text-[0.65rem] text-destructive" role="alert">{review.createError}</p>{/if}
          <div class="mt-2 flex justify-end gap-2">
            <button type="button" onclick={() => { inlineCreateOpen = false; review.createError = null; }} class="h-8 rounded-md bg-secondary px-2.5 text-xs font-medium">{t("music.builder.cancel")}</button>
            <button type="submit" disabled={!newPlaylistName.trim() || review.creatingPlaylist} class="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-2.5 text-xs font-medium text-primary-foreground disabled:opacity-40"><ListPlus size={14} />{t("music.builder.createAndAdd")}</button>
          </div>
        </form>
      {:else}
        <button type="button" onclick={openInlineCreate} class="mt-2 inline-flex h-8 items-center gap-1.5 rounded-full bg-primary px-3 text-xs font-medium text-primary-foreground"><ListPlus size={14} />{t("music.builder.newPlaylist")}</button>
      {/if}
    </div>

    <div bind:this={checklistRoot} class="flex min-h-0 flex-1 flex-col">
      <MusicPlaylistPicker
        playlists={library.playlistSummaries}
        {checkedIds}
        search={playlistSearch}
        onSearch={(value) => playlistSearch = value}
        onSearchInput={(element) => playlistSearchInput = element}
        onToggle={(playlist) => { void review.toggleMembership(playlist); }}
        weights={membershipWeights}
        focusFits={membershipFocusFits}
        onCycleWeight={(playlist) => { const membership = review.membershipFor(playlist.id); if (membership) void review.cycleMembershipWeight(membership); }}
        onFocusFit={(playlist, focusFit) => { const membership = review.membershipFor(playlist.id); if (membership) void review.setMembershipFocusFit(membership, focusFit); }}
        advisoryPlaylistId={focusAdvisory?.playlistIds[0] ?? null}
        advisoryText={focusAdvisory ? t("music.builder.focusAdvisory", focusAdvisory.signals.map((signal) => t(`music.builder.signal.${signal}`)).join(", ")) : ""}
        onAdvisoryKeep={() => { if (detail) dismissedAdvisoryItemId = detail.item.id; }}
        onAdvisoryMark={() => { if (!focusAdvisory || !detail) return; for (const playlistId of focusAdvisory.playlistIds) { const membership = review.membershipFor(playlistId); if (membership) void review.setMembershipFocusFit(membership, "potentially-distracting"); } dismissedAdvisoryItemId = detail.item.id; }}
        onDisableGuidance={disableFocusGuidance}
        busyIds={review.membershipBusy}
        errors={review.membershipErrors}
        showIssue={detail?.item.availability !== "available"}
        issueLabel={availabilityLabel()}
      />
    </div>

    <div class="review-actions grid shrink-0 grid-cols-2 gap-3 p-3">
      <div class="relative">
        <button bind:this={laterButton} type="button" aria-haspopup="dialog" aria-expanded={laterMenuOpen} onclick={() => { if (laterMenuOpen) closeLaterMenu(false); else void openLaterMenu(); }} disabled={!detail || review.actionBusy} class="review-action h-full w-full border border-border/70 bg-background text-foreground">{t("music.builder.skipTrack")}</button>
        {#if laterMenuOpen}
          <div bind:this={laterPopover} role="dialog" aria-label={t("music.builder.returnDate")} tabindex="-1" onfocusout={handleLaterFocusOut} class="later-popover absolute bottom-[calc(100%+0.5rem)] left-1/2 z-20 w-56 -translate-x-1/2 rounded-xl border border-border/70 bg-card p-3 text-left shadow-xl">
            <label for="music-review-return-date" class="block text-[0.68rem] font-medium">{t("music.builder.returnDate")}</label>
            <input bind:this={laterDateInput} id="music-review-return-date" type="date" bind:value={laterDate} min={new Date(Date.now() + 86_400_000).toISOString().slice(0, 10)} class="mt-1.5 h-8 w-full rounded-md border border-border/70 bg-background px-2 text-xs outline-none" />
            <p class="mt-1.5 text-[0.62rem] leading-relaxed text-muted-foreground">{t("music.builder.returnDateHint")}</p>
            <div class="mt-2 flex justify-end gap-2">
              <button type="button" onclick={() => closeLaterMenu()} class="h-7 rounded-md bg-secondary px-2 text-[0.68rem]">{t("music.builder.cancel")}</button>
              <button type="button" onclick={deferCurrentItem} class="h-7 rounded-md bg-primary px-2 text-[0.68rem] font-medium text-primary-foreground">{t("music.builder.confirmLater")}</button>
            </div>
          </div>
        {/if}
      </div>
      <button type="button" onclick={() => { void finishReviewState("reviewed"); }} disabled={!detail || review.actionBusy} class="review-action bg-primary text-primary-foreground" title={t("music.builder.markReviewedTitle", formatShortcut("Mod + Enter"))}><Check size={14} />{t("music.builder.saveAndNext")}<ChevronRight size={14} /></button>
    </div>
  </section>
  </div>
</div>

<style>
  .review-workspace { grid-template-columns: minmax(13rem, 0.72fr) minmax(22rem, 2fr); }
  .review-tree { grid-column: 1; min-height: 0; border-right: 1px solid color-mix(in srgb, var(--border) 46%, transparent); }
  .review-main { grid-column: 2; min-height: 0; }
  .review-audition { flex: 0 0 auto; }
  .review-classify { min-height: 14rem; flex: 1 1 0; }
  .review-play { display: grid; height: 2.5rem; width: 2.5rem; place-items: center; border-radius: 9999px; background: var(--primary); color: var(--primary-foreground); }
  .review-play:disabled { opacity: 0.4; }
  .review-action { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.375rem; border-radius: 0.5rem; padding: 0 0.5rem; font-size: 0.72rem; font-weight: 600; }
  .review-action:disabled { opacity: 0.4; }
  @container (width < 620px) {
    .review-workspace { display: flex; flex-direction: column; overflow-y: auto; }
    .review-tree { min-height: 12rem; flex: 0 0 42%; border-right: 0; border-bottom: 1px solid color-mix(in srgb, var(--border) 46%, transparent); }
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
    .review-workspace { grid-template-columns: minmax(11rem, 0.72fr) minmax(16rem, 1.15fr); }
    .review-classify { min-height: 15rem; }
  }
  @container (height < 300px) and (width >= 620px) {
    .review-audition { padding-block: 0.5rem; }
    .review-audition > :global(.aspect-video) { max-height: 7rem; }
  }
  @container (height < 260px) {
    .later-popover { position: fixed; inset: 0.5rem; width: auto; max-height: calc(100vh - 1rem); overflow-y: auto; transform: none; }
  }
</style>
