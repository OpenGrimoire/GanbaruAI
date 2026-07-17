<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Bath from "@lucide/svelte/icons/bath";
  import Bike from "@lucide/svelte/icons/bike";
  import BookOpen from "@lucide/svelte/icons/book-open";
  import Coffee from "@lucide/svelte/icons/coffee";
  import Dumbbell from "@lucide/svelte/icons/dumbbell";
  import Flower2 from "@lucide/svelte/icons/flower-2";
  import Laptop from "@lucide/svelte/icons/laptop";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Minus from "@lucide/svelte/icons/minus";
  import Search from "@lucide/svelte/icons/search";
  import ShoppingCart from "@lucide/svelte/icons/shopping-cart";
  import Sunrise from "@lucide/svelte/icons/sunrise";
  import TreePine from "@lucide/svelte/icons/tree-pine";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { sortReviewPlaylists } from "$lib/music/music-review";
  import { partitionMusicPlaylists, systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import type { MusicFocusFit, MusicPlaylistSummary, MusicWeight } from "$lib/music/library-contracts";
  import { cn } from "$lib/utils";

  let {
    playlists,
    checkedIds,
    mixedIds = new Set<string>(),
    search = "",
    onSearch = () => undefined,
    showSearch = false,
    showSections = false,
    onToggle,
    weights = {},
    onCycleWeight = () => undefined,
    focusFits = {},
    onFocusFit = () => undefined,
    advisoryPlaylistId = null,
    advisoryText = "",
    onAdvisoryKeep = () => undefined,
    onAdvisoryMark = () => undefined,
    onDisableGuidance = () => undefined,
    busyIds = new Set<string>(),
    errors = {},
    showIssue = false,
    issueLabel = "",
    onSearchInput = () => undefined,
  }: {
    playlists: MusicPlaylistSummary[];
    checkedIds: Set<string>;
    mixedIds?: Set<string>;
    search?: string;
    onSearch?: (value: string) => void;
    showSearch?: boolean;
    showSections?: boolean;
    onToggle: (playlist: MusicPlaylistSummary) => void;
    weights?: Record<string, MusicWeight>;
    onCycleWeight?: (playlist: MusicPlaylistSummary) => void;
    focusFits?: Record<string, MusicFocusFit>;
    onFocusFit?: (playlist: MusicPlaylistSummary, focusFit: MusicFocusFit) => void;
    advisoryPlaylistId?: string | null;
    advisoryText?: string;
    onAdvisoryKeep?: () => void;
    onAdvisoryMark?: () => void;
    onDisableGuidance?: () => void;
    busyIds?: Set<string>;
    errors?: Record<string, string>;
    showIssue?: boolean;
    issueLabel?: string;
    onSearchInput?: (element: HTMLInputElement | null) => void;
  } = $props();

  const { t } = getLocalization();
  const visible = $derived(sortReviewPlaylists(playlists, "").filter((playlist) => {
    const query = search.trim().toLocaleLowerCase();
    return !query || `${systemMusicPlaylistName(playlist.id, playlist.name, t)} ${playlist.description}`.toLocaleLowerCase().includes(query);
  }));
  const sections = $derived(partitionMusicPlaylists(visible));
  const displayedSections = $derived(showSections
    ? [
        { title: t("music.builder.defaultPlaylists"), playlists: sections.defaults },
        { title: t("music.builder.customPlaylists"), playlists: sections.custom },
      ]
    : [{ title: null, playlists: visible }]);

  function triStateAction(node: HTMLInputElement, mixed: boolean): { update: (value: boolean) => void } {
    node.indeterminate = mixed;
    return { update: (value) => { node.indeterminate = value; } };
  }

  function searchInputAction(node: HTMLInputElement): { destroy: () => void } {
    onSearchInput(node);
    return { destroy: () => onSearchInput(null) };
  }
</script>

<div class="flex min-h-0 flex-1 flex-col">
  {#if showSearch}
    <label class="mx-3 mt-3 flex h-9 shrink-0 items-center gap-2 border-b border-border/55 px-1">
      <Search size={14} class="text-muted-foreground" />
      <input use:searchInputAction value={search} oninput={(event) => onSearch(event.currentTarget.value)} aria-label={t("music.builder.searchPlaylists")} class="min-w-0 flex-1 bg-transparent text-xs outline-none" placeholder={t("music.builder.searchPlaylists")} />
    </label>
  {/if}
  <div class="min-h-0 flex-1 overflow-y-auto p-3" data-music-scrollable="true">
    {#if visible.length === 0}
      <p class="p-4 text-center text-xs text-muted-foreground">{t("music.builder.noPlaylistMatches")}</p>
    {:else}
    {#each displayedSections as section (section.title ?? "all")}
      {#if section.playlists.length > 0}
        {#if section.title}<h3 class="mb-2 mt-1 text-[0.7rem] font-semibold text-muted-foreground">{section.title}</h3>{/if}
        <div class="playlist-grid mb-4 grid gap-2">
    {#each section.playlists as playlist (playlist.id)}
      {@const checked = checkedIds.has(playlist.id)}
      {@const mixed = mixedIds.has(playlist.id)}
      {@const playlistName = systemMusicPlaylistName(playlist.id, playlist.name, t)}
      {@const errorId = errors[playlist.id] ? `music-playlist-membership-error-${playlist.id}` : undefined}
      <div class={cn("playlist-card flex min-w-0 flex-wrap items-center gap-2 rounded-lg px-3 py-2.5 transition-colors", checked || mixed ? "bg-primary/10" : "bg-secondary/35 hover:bg-secondary/55")}>
        <button type="button" data-review-playlist-id={playlist.id} onclick={() => onToggle(playlist)} aria-describedby={errorId} class="flex min-w-0 flex-1 items-center gap-3 text-left">
          <span class={cn("grid h-8 w-8 shrink-0 place-items-center", checked || mixed ? "text-primary" : "text-muted-foreground")}>
            {#if playlist.id === "playlist-default-reading"}<BookOpen size={16} />
            {:else if playlist.id === "playlist-default-exercise"}<Dumbbell size={16} />
            {:else if playlist.id === "playlist-default-hygiene"}<Bath size={16} />
            {:else if playlist.id === "playlist-default-commute"}<Bike size={16} />
            {:else if playlist.id === "playlist-default-chores"}<ShoppingCart size={16} />
            {:else if playlist.id === "playlist-default-meditate"}<Flower2 size={16} />
            {:else if playlist.id === "playlist-default-start-of-day"}<Sunrise size={16} />
            {:else if playlist.id === "playlist-default-working"}<Laptop size={16} />
            {:else if playlist.id === "playlist-default-short-breaks"}<Coffee size={16} />
            {:else if playlist.id === "playlist-default-long-breaks"}<TreePine size={16} />
            {:else}<ListMusic size={16} />{/if}
          </span>
          <span class="min-w-0 flex-1"><strong class="block truncate text-xs font-medium">{playlistName}</strong><span class="block text-[0.62rem] tabular-nums text-muted-foreground">{t("music.tracks", playlist.totalCount)}</span></span>
        </button>
        <label class="relative grid h-6 w-6 shrink-0 place-items-center">
          <input type="checkbox" checked={checked} use:triStateAction={mixed} onchange={() => onToggle(playlist)} aria-label={checked ? t("music.builder.removeFromPlaylist", playlist.name) : t("music.builder.addToPlaylist", playlist.name)} aria-describedby={errorId} class="peer absolute inset-0 opacity-0" />
          <span class={cn("pointer-events-none grid h-5 w-5 place-items-center rounded border", checked || mixed ? "border-primary bg-primary text-primary-foreground" : "border-border bg-background/80")}>
            {#if mixed}<Minus size={12} strokeWidth={2.5} />{:else if checked}<Check size={13} strokeWidth={2.5} />{/if}
          </span>
        </label>
        {#if showIssue}<span class="shrink-0 text-[0.6rem] text-warning" title={issueLabel}>{issueLabel}</span>{/if}
        {#if weights[playlist.id]}
          <div class="flex w-full items-center gap-2 pl-11">
            <button type="button" onclick={() => onCycleWeight(playlist)} disabled={busyIds.has(playlist.id)} class="shrink-0 rounded-md bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground disabled:opacity-40" title={t("music.builder.changeProbability")}>{t(`music.builder.weight.${weights[playlist.id]}`)}</button>
            <select value={focusFits[playlist.id] ?? "unknown"} onchange={(event) => onFocusFit(playlist, event.currentTarget.value as MusicFocusFit)} disabled={busyIds.has(playlist.id)} class="h-7 min-w-0 flex-1 rounded-md border border-border/60 bg-secondary px-1 text-[0.6rem] text-secondary-foreground disabled:opacity-40" aria-label={t("music.builder.focusFitForPlaylist", playlistName)}><option value="unknown">{t("music.builder.focusFitValue.unknown")}</option><option value="helpful">{t("music.builder.focusFitValue.helpful")}</option><option value="neutral">{t("music.builder.focusFitValue.neutral")}</option><option value="potentially-distracting">{t("music.builder.focusFitValue.potentially-distracting")}</option></select>
          </div>
        {/if}
      </div>
      {#if advisoryPlaylistId === playlist.id}
        <div class="mb-2 ml-7 rounded-lg border border-warning/20 bg-warning/8 p-2.5 text-[0.65rem] leading-relaxed"><div class="flex items-start gap-2 text-warning"><CircleAlert class="mt-0.5 shrink-0" size={13} /><span>{advisoryText}</span></div><div class="mt-2 flex flex-wrap gap-1.5 pl-5"><button type="button" onclick={onAdvisoryKeep} class="h-7 rounded-md bg-secondary px-2 text-[0.61rem]">{t("music.builder.addAnyway")}</button><button type="button" onclick={onAdvisoryMark} class="h-7 rounded-md bg-secondary px-2 text-[0.61rem]">{t("music.builder.markPotentiallyDistracting")}</button><button type="button" onclick={onDisableGuidance} class="h-7 rounded-md px-2 text-[0.61rem] text-muted-foreground hover:bg-secondary">{t("music.builder.disableFocusGuidance")}</button></div></div>
      {/if}
      {#if errors[playlist.id]}<p id={errorId} class="mb-1 px-2 text-[0.62rem] text-destructive" role="alert">{errors[playlist.id]}</p>{/if}
    {/each}
        </div>
      {/if}
    {/each}
    {/if}
  </div>
</div>

<style>
  .playlist-grid { grid-template-columns: repeat(auto-fit, minmax(min(12rem, 100%), 1fr)); }
  .playlist-card { align-content: center; }
</style>
