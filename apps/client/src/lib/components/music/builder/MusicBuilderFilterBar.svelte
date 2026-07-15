<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Filter from "@lucide/svelte/icons/list-filter";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type {
    MusicGroupBy,
    MusicItemAvailability,
    MusicItemSort,
    MusicLibrarySourceKind,
    MusicReviewState,
    MusicSortDirection,
    MusicSourceSummary,
  } from "$lib/music/library-contracts";
  import { cn } from "$lib/utils";

  type MenuKind = "source" | "collection" | "membership" | "availability" | "review" | "snooze" | "sort" | "group";
  interface Option<T extends string | null> { value: T; label: string }

  let {
    sourceKind,
    availability,
    reviewState,
    sort,
    direction,
    groupBy,
    resultCount,
    sourceCollectionId,
    membershipPlaylistId,
    snoozed,
    sources = [],
    playlists = [],
    playlistMode = false,
    onChange,
  }: {
    sourceKind: MusicLibrarySourceKind | null;
    availability: MusicItemAvailability | null;
    reviewState: MusicReviewState | null;
    sort: MusicItemSort;
    direction: MusicSortDirection;
    groupBy: MusicGroupBy;
    resultCount: number;
    sourceCollectionId: string | null;
    membershipPlaylistId: string | null;
    snoozed: boolean | null;
    sources?: MusicSourceSummary[];
    playlists?: import("$lib/music/library-contracts").MusicPlaylistSummary[];
    playlistMode?: boolean;
    onChange: (patch: {
      sourceKind?: MusicLibrarySourceKind | null;
      availability?: MusicItemAvailability | null;
      reviewState?: MusicReviewState | null;
      sourceCollectionId?: string | null;
      membershipPlaylistId?: string | null;
      snoozed?: boolean | null;
      sort?: MusicItemSort;
      direction?: MusicSortDirection;
      groupBy?: MusicGroupBy;
    }) => void;
  } = $props();

  const { t } = getLocalization();
  let openMenu = $state<MenuKind | null>(null);
  let menuTop = $state(0);
  let menuLeft = $state(0);
  let menuAnchor: HTMLElement | null = null;
  const filterCount = $derived(Number(sourceKind !== null) + Number(sourceCollectionId !== null) + Number(membershipPlaylistId !== null) + Number(availability !== null) + Number(reviewState !== null) + Number(snoozed !== null));
  const snoozeValue = $derived<"snoozed" | "active" | null>(snoozed === null ? null : snoozed ? "snoozed" : "active");
  const sourceOptions = $derived<Option<MusicLibrarySourceKind | null>[]>([
    { value: null, label: t("music.builder.allSources") },
    { value: "local-file", label: t("music.builder.local") },
    { value: "youtube-video", label: t("music.builder.youtube") },
  ]);
  const availabilityOptions = $derived<Option<MusicItemAvailability | null>[]>([
    { value: null, label: t("music.builder.allAvailability") },
    { value: "available", label: t("music.builder.available") },
    { value: "missing", label: t("music.builder.missing") },
    { value: "unavailable", label: t("music.builder.unavailable") },
    { value: "ambiguous", label: t("music.builder.ambiguous") },
    { value: "unknown", label: t("music.builder.unknownAvailability") },
  ]);
  const reviewOptions = $derived<Option<MusicReviewState | null>[]>([
    { value: null, label: t("music.builder.allReviewStates") },
    { value: "unreviewed", label: t("music.builder.unreviewed") },
    { value: "reviewed", label: t("music.builder.reviewed") },
    { value: "deferred", label: t("music.builder.deferred") },
    { value: "ignored", label: t("music.builder.ignored") },
  ]);
  const collectionOptions = $derived<Option<string | null>[]>([
    { value: null, label: t("music.builder.allCollections") },
    ...sources.map((source) => ({ value: source.id, label: source.name })),
  ]);
  const membershipOptions = $derived<Option<string | null>[]>([
    { value: null, label: t("music.builder.anyPlaylistMembership") },
    ...playlists.map((playlist) => ({ value: playlist.id, label: playlist.name })),
  ]);
  const snoozeOptions = $derived<Option<"snoozed" | "active" | null>[]>([
    { value: null, label: t("music.builder.allSnoozeStates") },
    { value: "snoozed", label: t("music.builder.snoozed") },
    { value: "active", label: t("music.builder.notSnoozed") },
  ]);
  const sortOptions = $derived<Option<MusicItemSort>[]>([
    { value: "title", label: t("music.builder.sortTitle") },
    { value: "artist", label: t("music.builder.sortArtist") },
    { value: "album", label: t("music.builder.sortAlbum") },
    { value: "source-order", label: t("music.builder.sortSourceOrder") },
    { value: "discovered-at", label: t("music.builder.sortDiscovered") },
    ...(playlistMode ? [{ value: "added-to-playlist" as const, label: t("music.builder.sortAddedToPlaylist") }] : []),
    { value: "last-played-at", label: t("music.builder.sortLastPlayed") },
    { value: "play-count", label: t("music.builder.sortPlayCount") },
    ...(playlistMode ? [{ value: "manual-position" as const, label: t("music.builder.sortManual") }] : []),
  ]);
  const groupOptions = $derived<Option<MusicGroupBy>[]>([
    { value: "none", label: t("music.builder.groupNone") },
    { value: "source-kind", label: t("music.builder.groupSource") },
    { value: "review-state", label: t("music.builder.groupReview") },
    { value: "availability", label: t("music.builder.groupAvailability") },
    { value: "album", label: t("music.builder.groupAlbum") },
    { value: "folder", label: t("music.builder.groupFolder") },
    { value: "source-collection", label: t("music.builder.groupCollection") },
  ]);

  function labelFor<T extends string | null>(options: Option<T>[], value: T): string {
    return options.find((option) => option.value === value)?.label ?? "";
  }

  function closeMenu(restoreFocus: boolean): void {
    const anchor = menuAnchor;
    openMenu = null;
    menuAnchor = null;
    if (restoreFocus && anchor?.isConnected) queueMicrotask(() => anchor.focus());
  }
  function closeAfter(action: () => void): void { action(); closeMenu(true); }
  function toggleMenu(kind: MenuKind, anchor: HTMLElement): void {
    if (openMenu === kind) { closeMenu(true); return; }
    const bounds = anchor.getBoundingClientRect();
    menuTop = Math.max(4, Math.min(bounds.bottom + 5, window.innerHeight - 284));
    menuLeft = Math.max(4, Math.min(bounds.left, window.innerWidth - 180));
    menuAnchor = anchor;
    openMenu = kind;
  }
</script>

<svelte:window onkeydown={(event) => { if (event.key === "Escape" && openMenu) { event.stopPropagation(); closeMenu(true); } }} />

<div class="flex min-h-10 shrink-0 items-center gap-1.5 overflow-x-auto border-b border-border/45 px-2 py-1.5" style={`--filter-menu-top:${menuTop}px;--filter-menu-left:${menuLeft}px`} aria-label={t("music.builder.filters")}>
  <span class="mr-0.5 inline-flex shrink-0 items-center gap-1 text-[0.66rem] font-medium text-muted-foreground">
    <Filter size={12} strokeWidth={1.7} />
    {filterCount > 0 ? filterCount : ""}
  </span>

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "source"} class={cn("filter-pill", sourceKind && "filter-pill-active")} onclick={(event) => toggleMenu("source", event.currentTarget)}>
      {labelFor(sourceOptions, sourceKind)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "source"}
      <div class="filter-menu">
        {#each sourceOptions as option (option.value)}
          <button type="button" onclick={() => closeAfter(() => onChange({ sourceKind: option.value }))}>{option.label}{#if option.value === sourceKind}<Check size={12} />{/if}</button>
        {/each}
      </div>
    {/if}
  </div>

  {#if playlists.length > 0}
    <div class="relative shrink-0">
      <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "membership"} class={cn("filter-pill", membershipPlaylistId && "filter-pill-active")} onclick={(event) => toggleMenu("membership", event.currentTarget)}>
        {labelFor(membershipOptions, membershipPlaylistId)} <ChevronDown size={11} />
      </button>
      {#if openMenu === "membership"}<div class="filter-menu">{#each membershipOptions as option (option.value)}<button type="button" onclick={() => closeAfter(() => onChange({ membershipPlaylistId: option.value }))}>{option.label}{#if option.value === membershipPlaylistId}<Check size={12} />{/if}</button>{/each}</div>{/if}
    </div>
  {/if}

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "collection"} class={cn("filter-pill", sourceCollectionId && "filter-pill-active")} onclick={(event) => toggleMenu("collection", event.currentTarget)}>
      {labelFor(collectionOptions, sourceCollectionId)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "collection"}<div class="filter-menu">{#each collectionOptions as option (option.value)}<button type="button" onclick={() => closeAfter(() => onChange({ sourceCollectionId: option.value }))}>{option.label}{#if option.value === sourceCollectionId}<Check size={12} />{/if}</button>{/each}</div>{/if}
  </div>

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "availability"} class={cn("filter-pill", availability && "filter-pill-active")} onclick={(event) => toggleMenu("availability", event.currentTarget)}>
      {labelFor(availabilityOptions, availability)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "availability"}
      <div class="filter-menu">
        {#each availabilityOptions as option (option.value)}
          <button type="button" onclick={() => closeAfter(() => onChange({ availability: option.value }))}>{option.label}{#if option.value === availability}<Check size={12} />{/if}</button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "snooze"} class={cn("filter-pill", snoozed !== null && "filter-pill-active")} onclick={(event) => toggleMenu("snooze", event.currentTarget)}>{labelFor(snoozeOptions, snoozeValue)} <ChevronDown size={11} /></button>
    {#if openMenu === "snooze"}<div class="filter-menu">{#each snoozeOptions as option (option.value)}<button type="button" onclick={() => closeAfter(() => onChange({ snoozed: option.value === null ? null : option.value === "snoozed" }))}>{option.label}{#if option.value === snoozeValue}<Check size={12} />{/if}</button>{/each}</div>{/if}
  </div>

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "review"} class={cn("filter-pill", reviewState && "filter-pill-active")} onclick={(event) => toggleMenu("review", event.currentTarget)}>
      {labelFor(reviewOptions, reviewState)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "review"}
      <div class="filter-menu">
        {#each reviewOptions as option (option.value)}
          <button type="button" onclick={() => closeAfter(() => onChange({ reviewState: option.value }))}>{option.label}{#if option.value === reviewState}<Check size={12} />{/if}</button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="relative ml-auto shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "sort"} class="filter-pill" onclick={(event) => toggleMenu("sort", event.currentTarget)}>
      {labelFor(sortOptions, sort)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "sort"}
      <div class="filter-menu filter-menu-right">
        {#each sortOptions as option (option.value)}
          <button type="button" onclick={() => closeAfter(() => onChange({ sort: option.value }))}>{option.label}{#if option.value === sort}<Check size={12} />{/if}</button>
        {/each}
        <div class="my-1 h-px bg-border/60"></div>
        <button type="button" onclick={() => closeAfter(() => onChange({ direction: direction === "ascending" ? "descending" : "ascending" }))}>
          {direction === "ascending" ? t("music.builder.ascending") : t("music.builder.descending")}
        </button>
      </div>
    {/if}
  </div>

  <div class="relative shrink-0">
    <button type="button" aria-haspopup="menu" aria-expanded={openMenu === "group"} class="filter-pill" onclick={(event) => toggleMenu("group", event.currentTarget)}>
      {labelFor(groupOptions, groupBy)} <ChevronDown size={11} />
    </button>
    {#if openMenu === "group"}
      <div class="filter-menu filter-menu-right">
        {#each groupOptions as option (option.value)}
          <button type="button" onclick={() => closeAfter(() => onChange({ groupBy: option.value }))}>{option.label}{#if option.value === groupBy}<Check size={12} />{/if}</button>
        {/each}
      </div>
    {/if}
  </div>

  {#if filterCount > 0}
    <button type="button" class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground" onclick={() => onChange({ sourceKind: null, sourceCollectionId: null, membershipPlaylistId: null, availability: null, reviewState: null, snoozed: null })} aria-label={t("music.builder.clearFilters")}><X size={12} /></button>
  {/if}
  <span class="shrink-0 px-1 text-[0.64rem] tabular-nums text-muted-foreground">{t("music.builder.resultCount", resultCount)}</span>
</div>

<style>
  .filter-pill { display: inline-flex; height: 1.75rem; align-items: center; gap: 0.25rem; border: 1px solid color-mix(in srgb, var(--border) 75%, transparent); border-radius: 999px; background: color-mix(in srgb, var(--card) 78%, transparent); padding-inline: 0.6rem; color: var(--muted-foreground); font-size: 0.65rem; white-space: nowrap; }
  .filter-pill:hover, .filter-pill-active { border-color: color-mix(in srgb, var(--primary) 35%, var(--border)); color: var(--foreground); }
  .filter-pill-active { background: color-mix(in srgb, var(--primary) 9%, var(--card)); }
  .filter-menu { position: fixed; top: var(--filter-menu-top); left: var(--filter-menu-left); z-index: 70; min-width: 10.5rem; max-height: min(18rem, 55vh); overflow-y: auto; border: 1px solid color-mix(in srgb, var(--border) 85%, transparent); border-radius: 0.7rem; background: var(--popover); padding: 0.3rem; box-shadow: 0 12px 32px color-mix(in srgb, black 20%, transparent); }
  .filter-menu button { display: flex; width: 100%; min-height: 1.8rem; align-items: center; justify-content: space-between; gap: 0.75rem; border-radius: 0.45rem; padding-inline: 0.55rem; color: var(--popover-foreground); font-size: 0.68rem; text-align: left; }
  .filter-menu button:hover, .filter-menu button:focus-visible { background: var(--accent); outline: none; }
</style>
