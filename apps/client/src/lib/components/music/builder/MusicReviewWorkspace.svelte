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
  import type {
    MusicGroupBy,
    MusicWeight,
  } from "$lib/music/library-contracts";
  import { formatMusicDuration } from "$lib/music/music-builder-presentation";
  import { clampRate, formatPlaybackTime } from "$lib/music/playback";
  import { cn } from "$lib/utils";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import { getMusicFocusAdvisory } from "$lib/music/music-focus-guidance";
  import MusicPlaylistPicker from "./MusicPlaylistPicker.svelte";

  let {
    library,
    inspector,
    sources,
    audition,
    review,
    autoplay,
    onAutoplayChange,
  }: {
    library: MusicLibraryController;
    inspector: MusicBuilderInspectorController;
    sources: MusicSourcesController;
    audition: MusicReviewAuditionController;
    review: MusicReviewController;
    autoplay: boolean;
    onAutoplayChange: (value: boolean) => void;
  } = $props();

  const { t } = getLocalization();
  let surface = $state<HTMLElement | null>(null);
  let playlistSearch = $state("");
  let newPlaylistName = $state("");
  let newPlaylistDescription = $state("");
  let inlineCreateOpen = $state(false);
  let laterMenuOpen = $state(false);
  let laterButton = $state<HTMLButtonElement | null>(null);
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
  const focusAdvisory = $derived(detail && guidanceEnabled && dismissedAdvisoryItemId !== detail.item.id
    ? getMusicFocusAdvisory(detail.signals, checkedIds, library.playlistSummaries)
    : null);

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

  function setGrouping(groupBy: MusicGroupBy): void {
    library.patchCurrentState({ groupBy, offset: 0, selectedItemId: null });
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
    laterMenuOpen = false;
    void finishReviewState("deferred", Number.isFinite(deferredUntil) ? deferredUntil : null);
  }

  function closeLaterMenu(restoreFocus = true): void {
    laterMenuOpen = false;
    if (restoreFocus && laterButton?.isConnected) queueMicrotask(() => laterButton?.focus());
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
    } else if (!event.shiftKey && /^[0-9]$/.test(event.key) && player.snapshot.durationMs) {
      event.preventDefault(); void player.seekToMs(Math.round(player.snapshot.durationMs * Number(event.key) / 10));
    } else if (event.key === "/") {
      event.preventDefault(); playlistSearchInput?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="review-workspace grid min-h-0 flex-1 overflow-hidden">
  <section class="review-audition min-h-0 overflow-y-auto px-4 py-3" data-music-scrollable="true">
    <div class="flex items-center justify-between gap-3">
      <p class="text-[0.7rem] font-medium text-muted-foreground" role="status" aria-live="polite">{t("music.builder.reviewProgress", progressCurrent, sessionTotal)}</p>
      <div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1">
        {#if audition.active}
          <button type="button" onclick={() => { void audition.restore(); }} class="h-7 rounded-md bg-secondary px-2 text-[0.68rem] font-medium text-secondary-foreground">{t("music.builder.returnPreviousPlayback")}</button>
        {/if}
        <label class="flex items-center gap-1.5 text-[0.7rem] text-muted-foreground">
          <span>{t("music.builder.reviewGroup")}</span>
          <select value={library.currentState.groupBy} onchange={(event) => setGrouping(event.currentTarget.value as MusicGroupBy)} class="h-7 rounded-md border border-border/70 bg-card px-1.5 text-[0.68rem] text-foreground outline-none">
            <option value="none">{t("music.builder.groupNone")}</option>
            <option value="album">{t("music.builder.groupAlbum")}</option>
            <option value="folder">{t("music.builder.groupFolder")}</option>
            <option value="source-collection">{t("music.builder.groupCollection")}</option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-[0.7rem] text-muted-foreground">
          <input type="checkbox" checked={library.currentState.reviewState === null} onchange={(event) => setIncludeLater(event.currentTarget.checked)} class="accent-primary" />
          {t("music.builder.includeLater")}
        </label>
        <label class="flex items-center gap-2 text-[0.7rem] text-muted-foreground">
          <input type="checkbox" checked={autoplay} onchange={(event) => onAutoplayChange(event.currentTarget.checked)} class="accent-primary" />
          {t("music.builder.reviewAutoplay")}
        </label>
        {#if !guidanceEnabled}<button type="button" onclick={enableFocusGuidance} class="h-7 rounded-md bg-secondary px-2 text-[0.65rem] text-secondary-foreground">{t("music.builder.enableFocusGuidance")}</button>{/if}
      </div>
    </div>
    {#if library.currentState.groupBy !== "none" && library.currentWindow.groups.length > 0}
      <div class="mt-2 flex gap-1.5 overflow-x-auto pb-1" aria-label={t("music.builder.reviewGroups")}>
        {#each library.currentWindow.groups as group (group.key)}
          <span class="shrink-0 rounded-full bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground">{group.key} · {group.count}</span>
        {/each}
      </div>
    {/if}

    <div bind:this={surface} class="mt-3 aspect-video w-full overflow-hidden rounded-xl border border-border/70 bg-card shadow-inner">
      {#if detail && audition.reviewItemId === detail.item.id && !player.localHasVideo && detail.item.sourceKind === "local-file"}
        <div class="relative grid h-full place-items-center overflow-hidden bg-secondary/35">
          {#if player.currentArtworkUrl}
            <img src={player.currentArtworkUrl} alt="" class="absolute inset-0 h-full w-full object-contain" draggable="false" />
          {:else}
            <div class="grid h-16 w-16 place-items-center rounded-2xl bg-card text-muted-foreground shadow-sm"><Disc3 size={28} strokeWidth={1.3} /></div>
          {/if}
        </div>
      {:else if !detail || !player.currentSource || audition.reviewItemId !== detail.item.id}
        <div class="grid h-full place-items-center p-5 text-center">
          <button type="button" onclick={() => { if (detail) void audition.preview(detail, sources.bindings, true); }} disabled={!detail || detail.item.availability !== "available"} class="grid h-14 w-14 place-items-center rounded-full bg-primary text-primary-foreground shadow-lg transition-transform hover:scale-105 active:scale-95 disabled:opacity-40 motion-reduce:transform-none" aria-label={t("music.builder.previewTrack")}>
            <Play size={22} fill="currentColor" />
          </button>
        </div>
      {/if}
    </div>

    {#if inspector.busy && !detail}
      <div class="mt-4 h-20 animate-pulse rounded-lg bg-card motion-reduce:animate-none"></div>
    {:else if detail}
      <div class="mt-4 min-w-0">
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1">
            <h2 class="truncate text-base font-semibold">{detail.item.titleOverride ?? detail.item.originalTitle}</h2>
            <p class="mt-0.5 truncate text-xs text-muted-foreground">{(detail.item.artistOverride ?? detail.item.originalArtist) || t("music.builder.noArtist")}</p>
          </div>
          <span class={cn("rounded-full px-2 py-1 text-[0.65rem] font-medium", detail.item.availability === "available" ? "bg-success/12 text-success" : "bg-destructive/12 text-destructive")}>{availabilityLabel()}</span>
        </div>
        <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[0.7rem] text-muted-foreground">
          <span>{(detail.item.albumOverride ?? detail.item.originalAlbum) || t("music.builder.noAlbum")}</span>
          <span>{detail.item.sourceKind === "local-file" ? t("music.builder.local") : t("music.builder.youtube")}</span>
          <span>{formatMusicDuration(detail.item.durationMs)}</span>
        </div>
      </div>

      <div class="mt-4 rounded-xl border border-border/70 bg-card/70 p-3">
        <input type="range" min="0" max={player.progressMax} value={player.progressValue} disabled={audition.reviewItemId !== detail.item.id} oninput={(event) => { void player.seekToMs(Number(event.currentTarget.value)); }} class="w-full accent-primary" aria-label={t("music.seek")} />
        <div class="mt-1 flex justify-between text-[0.65rem] tabular-nums text-muted-foreground"><span>{formatPlaybackTime(player.snapshot.positionMs)}</span><span>{formatPlaybackTime(player.snapshot.durationMs)}</span></div>
        <div class="mt-2 flex items-center justify-center gap-3">
          <button type="button" onclick={() => { void selectRelative(-1); }} disabled={currentIndex <= 0} class="review-control" aria-label={t("music.builder.previousReviewItem")} title={t("music.builder.previousReviewItemTitle", formatShortcut("Mod + ←"))}><ChevronLeft size={17} /></button>
          <button type="button" onclick={() => { if (audition.reviewItemId !== detail.item.id) void audition.preview(detail, sources.bindings, true); else void player.togglePlay(); }} class="review-play" aria-label={player.isPlaying ? t("music.pause") : t("music.play")} title={t("music.builder.reviewPlayTitle", formatShortcut("Space"))}>
            {#if player.isPlaying && audition.reviewItemId === detail.item.id}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
          </button>
          <button type="button" onclick={() => { void selectRelative(1); }} disabled={currentIndex >= library.currentWindow.items.length - 1} class="review-control" aria-label={t("music.builder.nextReviewItem")} title={t("music.builder.nextReviewItemTitle", formatShortcut("Mod + →"))}><ChevronRight size={17} /></button>
        </div>
      </div>
    {/if}
  </section>

  <section class="review-classify flex min-h-0 flex-col border-l border-border/70 bg-card/35">
    <div class="shrink-0 border-b border-border/60 p-3">
      <div class="flex items-center justify-between gap-2">
        <div><h2 class="text-sm font-semibold">{t("music.builder.classifyPlaylists")}</h2><p class="text-[0.68rem] text-muted-foreground">{t("music.builder.classifyHint")}</p></div>
        {#if checkedIds.size > 0}<button type="button" onclick={() => { void review.clearMemberships(); }} class="text-[0.68rem] text-muted-foreground hover:text-foreground">{t("music.builder.clearMemberships")}</button>{/if}
      </div>
      {#if inlineCreateOpen}
        <form class="mt-2 rounded-lg border border-border/70 bg-background/75 p-2" onsubmit={(event) => { event.preventDefault(); void createPlaylistAndAdd(); }}>
          <input bind:this={newPlaylistNameInput} bind:value={newPlaylistName} class="h-8 w-full rounded-md border border-border/70 bg-background px-2.5 text-xs outline-none focus:border-primary" placeholder={t("music.builder.inlinePlaylistName")} />
          <input bind:value={newPlaylistDescription} class="mt-2 h-8 w-full rounded-md border border-border/70 bg-background px-2.5 text-xs outline-none focus:border-primary" placeholder={t("music.builder.inlinePlaylistDescription")} />
          {#if review.createError}<p class="mt-1.5 text-[0.65rem] text-destructive" role="alert">{review.createError}</p>{/if}
          <div class="mt-2 flex justify-end gap-2">
            <button type="button" onclick={() => { inlineCreateOpen = false; review.createError = null; }} class="h-8 rounded-md bg-secondary px-2.5 text-xs font-medium">{t("music.builder.cancel")}</button>
            <button type="submit" disabled={!newPlaylistName.trim() || review.creatingPlaylist} class="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-2.5 text-xs font-medium text-primary-foreground disabled:opacity-40"><ListPlus size={14} />{t("music.builder.createAndAdd")}</button>
          </div>
        </form>
      {:else}
        <button type="button" onclick={openInlineCreate} class="mt-2 inline-flex h-8 items-center gap-1.5 rounded-md bg-secondary px-2.5 text-xs font-medium"><ListPlus size={14} />{t("music.builder.newPlaylist")}</button>
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

    <div class="review-actions grid shrink-0 grid-cols-3 gap-2 border-t border-border/70 bg-card p-3">
      <button type="button" onclick={() => { void finishReviewState("ignored"); }} disabled={!detail || review.actionBusy} class="review-action bg-secondary text-secondary-foreground">{t("music.builder.ignore")}</button>
      <div class="relative">
        <button bind:this={laterButton} type="button" aria-haspopup="dialog" aria-expanded={laterMenuOpen} onclick={() => laterMenuOpen ? closeLaterMenu(false) : laterMenuOpen = true} disabled={!detail || review.actionBusy} class="review-action h-full w-full bg-secondary text-secondary-foreground">{t("music.builder.later")}</button>
        {#if laterMenuOpen}
          <div role="dialog" aria-label={t("music.builder.returnDate")} tabindex="-1" class="later-popover absolute bottom-[calc(100%+0.5rem)] left-1/2 z-20 w-56 -translate-x-1/2 rounded-xl border border-border/70 bg-card p-3 text-left shadow-xl">
            <label for="music-review-return-date" class="block text-[0.68rem] font-medium">{t("music.builder.returnDate")}</label>
            <input id="music-review-return-date" type="date" bind:value={laterDate} min={new Date(Date.now() + 86_400_000).toISOString().slice(0, 10)} class="mt-1.5 h-8 w-full rounded-md border border-border/70 bg-background px-2 text-xs outline-none" />
            <p class="mt-1.5 text-[0.62rem] leading-relaxed text-muted-foreground">{t("music.builder.returnDateHint")}</p>
            <div class="mt-2 flex justify-end gap-2">
              <button type="button" onclick={() => closeLaterMenu()} class="h-7 rounded-md bg-secondary px-2 text-[0.68rem]">{t("music.builder.cancel")}</button>
              <button type="button" onclick={deferCurrentItem} class="h-7 rounded-md bg-primary px-2 text-[0.68rem] font-medium text-primary-foreground">{t("music.builder.confirmLater")}</button>
            </div>
          </div>
        {/if}
      </div>
      <button type="button" onclick={() => { void finishReviewState("reviewed"); }} disabled={!detail || review.actionBusy} class="review-action bg-primary text-primary-foreground" title={t("music.builder.markReviewedTitle", formatShortcut("Mod + Enter"))}><Check size={14} />{t("music.builder.markReviewed")}</button>
    </div>
  </section>
</div>

<style>
  .review-workspace { grid-template-columns: minmax(18rem, 1.05fr) minmax(18rem, 0.95fr); }
  .review-control { display: grid; height: 2rem; width: 2rem; place-items: center; border-radius: 9999px; background: var(--secondary); color: var(--secondary-foreground); }
  .review-control:disabled { opacity: 0.35; }
  .review-play { display: grid; height: 2.5rem; width: 2.5rem; place-items: center; border-radius: 9999px; background: var(--primary); color: var(--primary-foreground); }
  .review-action { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.375rem; border-radius: 0.5rem; padding: 0 0.5rem; font-size: 0.72rem; font-weight: 600; }
  .review-action:disabled { opacity: 0.4; }
  @container (width < 620px) {
    .review-workspace { display: flex; flex-direction: column; overflow-y: auto; }
    .review-audition, .review-classify { min-height: auto; overflow: visible; }
    .review-audition { flex: 0 0 min(44%, 18rem); padding: 0.625rem; }
    .review-classify { flex: 1 0 18rem; border-left: 0; border-top: 1px solid var(--border); }
    .review-classify > :global(div:nth-child(2)) { min-height: 9rem; }
    .review-actions { position: sticky; bottom: 0; z-index: 5; }
  }
  @container (height < 300px) and (width >= 620px) {
    .review-audition { padding-block: 0.5rem; }
    .review-audition > :global(.aspect-video) { max-height: 7rem; }
  }
  @container (height < 260px) {
    .later-popover { position: fixed; inset: 0.5rem; width: auto; max-height: calc(100vh - 1rem); overflow-y: auto; transform: none; }
  }
</style>
