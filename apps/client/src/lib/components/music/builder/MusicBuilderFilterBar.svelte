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
  } from "$lib/music/library-contracts";
  import { cn } from "$lib/utils";

  type MenuKind = "source" | "availability" | "review" | "sort" | "group";
  interface Option<T extends string | null> { value: T; label: string }

  let {
    sourceKind,
    availability,
    reviewState,
    sort,
    direction,
    groupBy,
    resultCount,
    onChange,
  }: {
    sourceKind: MusicLibrarySourceKind | null;
    availability: MusicItemAvailability | null;
    reviewState: MusicReviewState | null;
    sort: MusicItemSort;
    direction: MusicSortDirection;
    groupBy: MusicGroupBy;
    resultCount: number;
    onChange: (patch: {
      sourceKind?: MusicLibrarySourceKind | null;
      availability?: MusicItemAvailability | null;
      reviewState?: MusicReviewState | null;
      sort?: MusicItemSort;
      direction?: MusicSortDirection;
      groupBy?: MusicGroupBy;
    }) => void;
  } = $props();

  const { t } = getLocalization();
  let openMenu = $state<MenuKind | null>(null);
  const filterCount = $derived(Number(sourceKind !== null) + Number(availability !== null) + Number(reviewState !== null));
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
  const sortOptions = $derived<Option<MusicItemSort>[]>([
    { value: "title", label: t("music.builder.sortTitle") },
    { value: "artist", label: t("music.builder.sortArtist") },
    { value: "album", label: t("music.builder.sortAlbum") },
    { value: "discovered-at", label: t("music.builder.sortDiscovered") },
    { value: "last-played-at", label: t("music.builder.sortLastPlayed") },
    { value: "play-count", label: t("music.builder.sortPlayCount") },
  ]);
  const groupOptions = $derived<Option<MusicGroupBy>[]>([
    { value: "none", label: t("music.builder.groupNone") },
    { value: "source-kind", label: t("music.builder.groupSource") },
    { value: "review-state", label: t("music.builder.groupReview") },
    { value: "availability", label: t("music.builder.groupAvailability") },
    { value: "album", label: t("music.builder.groupAlbum") },
  ]);

  function labelFor<T extends string | null>(options: Option<T>[], value: T): string {
    return options.find((option) => option.value === value)?.label ?? "";
  }

  function closeAfter(action: () => void): void { action(); openMenu = null; }
</script>

<svelte:window onkeydown={(event) => { if (event.key === "Escape" && openMenu) { event.stopPropagation(); openMenu = null; } }} />

<div class="flex min-h-10 shrink-0 items-center gap-1.5 overflow-x-auto border-b border-border/45 px-2 py-1.5" aria-label={t("music.builder.filters")}>
  <span class="mr-0.5 inline-flex shrink-0 items-center gap-1 text-[0.66rem] font-medium text-muted-foreground">
    <Filter size={12} strokeWidth={1.7} />
    {filterCount > 0 ? filterCount : ""}
  </span>

  <div class="relative shrink-0">
    <button type="button" class={cn("filter-pill", sourceKind && "filter-pill-active")} onclick={() => openMenu = openMenu === "source" ? null : "source"}>
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

  <div class="relative shrink-0">
    <button type="button" class={cn("filter-pill", availability && "filter-pill-active")} onclick={() => openMenu = openMenu === "availability" ? null : "availability"}>
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
    <button type="button" class={cn("filter-pill", reviewState && "filter-pill-active")} onclick={() => openMenu = openMenu === "review" ? null : "review"}>
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
    <button type="button" class="filter-pill" onclick={() => openMenu = openMenu === "sort" ? null : "sort"}>
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
    <button type="button" class="filter-pill" onclick={() => openMenu = openMenu === "group" ? null : "group"}>
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
    <button type="button" class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground" onclick={() => onChange({ sourceKind: null, availability: null, reviewState: null })} aria-label={t("music.builder.clearFilters")}><X size={12} /></button>
  {/if}
  <span class="shrink-0 px-1 text-[0.64rem] tabular-nums text-muted-foreground">{t("music.builder.resultCount", resultCount)}</span>
</div>

<style>
  .filter-pill { display: inline-flex; height: 1.75rem; align-items: center; gap: 0.25rem; border: 1px solid color-mix(in srgb, var(--border) 75%, transparent); border-radius: 999px; background: color-mix(in srgb, var(--card) 78%, transparent); padding-inline: 0.6rem; color: var(--muted-foreground); font-size: 0.65rem; white-space: nowrap; }
  .filter-pill:hover, .filter-pill-active { border-color: color-mix(in srgb, var(--primary) 35%, var(--border)); color: var(--foreground); }
  .filter-pill-active { background: color-mix(in srgb, var(--primary) 9%, var(--card)); }
  .filter-menu { position: absolute; top: calc(100% + 0.3rem); z-index: 50; min-width: 10.5rem; max-height: min(18rem, 55vh); overflow-y: auto; border: 1px solid color-mix(in srgb, var(--border) 85%, transparent); border-radius: 0.7rem; background: var(--popover); padding: 0.3rem; box-shadow: 0 12px 32px color-mix(in srgb, black 20%, transparent); }
  .filter-menu-right { right: 0; }
  .filter-menu button { display: flex; width: 100%; min-height: 1.8rem; align-items: center; justify-content: space-between; gap: 0.75rem; border-radius: 0.45rem; padding-inline: 0.55rem; color: var(--popover-foreground); font-size: 0.68rem; text-align: left; }
  .filter-menu button:hover, .filter-menu button:focus-visible { background: var(--accent); outline: none; }
</style>
