<script lang="ts">
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import MoreHorizontal from "@lucide/svelte/icons/ellipsis";
  import Music2 from "@lucide/svelte/icons/music-2";
  import Play from "@lucide/svelte/icons/play";
  import Youtube from "@lucide/svelte/icons/youtube";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import {
    formatMusicDuration,
    musicAvailabilityTone,
    musicItemSecondaryText,
    musicReviewTone,
  } from "$lib/music/music-builder-presentation";
  import { cn } from "$lib/utils";

  let {
    item,
    selected = false,
    playing = false,
    onSelect,
    onPlay = () => undefined,
    onMore = () => undefined,
  }: {
    item: MusicItemListEntry;
    selected?: boolean;
    playing?: boolean;
    onSelect: (item: MusicItemListEntry, event: MouseEvent) => void;
    onPlay?: (item: MusicItemListEntry) => void;
    onMore?: (item: MusicItemListEntry, anchor: HTMLElement) => void;
  } = $props();

  const { t } = getLocalization();
  const secondary = $derived(musicItemSecondaryText(item, t("music.builder.noArtist"), t("music.builder.noAlbum")));
  const duration = $derived(formatMusicDuration(item.durationMs));
  const availabilityTone = $derived(musicAvailabilityTone(item.availability));
  const reviewTone = $derived(musicReviewTone(item.reviewState));

  function availabilityLabel(): string {
    if (item.availability === "available") return t("music.builder.available");
    if (item.availability === "missing") return t("music.builder.missing");
    if (item.availability === "unavailable") return t("music.builder.unavailable");
    if (item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
  }
</script>

<div
  class={cn("music-builder-row group", selected && "music-builder-row-selected", playing && "music-builder-row-playing")}
  role="option"
  aria-selected={selected}
  data-music-focus-key={`item:${item.id}`}
>
  <button
    type="button"
    class="music-builder-artwork"
    onclick={(event) => {
      event.stopPropagation();
      onPlay(item);
    }}
    aria-label={t("music.play")}
  >
    {#if item.sourceKind === "youtube-video"}
      <Youtube size={17} strokeWidth={1.5} />
    {:else}
      <Music2 size={16} strokeWidth={1.5} />
    {/if}
    <span class="music-builder-play-overlay"><Play size={14} fill="currentColor" strokeWidth={1.5} /></span>
  </button>

  <button
    type="button"
    class="grid min-w-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(5rem,0.65fr)] items-center gap-3 text-left max-[470px]:grid-cols-1 max-[470px]:gap-0.5"
    onclick={(event) => onSelect(item, event)}
    aria-label={t("music.builder.selectTrack", item.title)}
  >
    <span class="min-w-0">
      <span class="flex min-w-0 items-center gap-1.5">
        {#if playing}<span class="playing-bars" aria-hidden="true"><i></i><i></i><i></i></span>{/if}
        <span class="block truncate text-xs font-semibold text-foreground">{item.title}</span>
      </span>
      <span class="mt-0.5 block truncate text-[0.68rem] text-muted-foreground">{secondary.primary}</span>
    </span>
    <span class="min-w-0 max-[470px]:hidden">
      <span class="block truncate text-[0.7rem] text-muted-foreground">{secondary.secondary}</span>
      <span class="mt-0.5 block truncate text-[0.62rem] text-muted-foreground/75">
        {t("music.builder.playlistsMembership", item.playlistCount)}
      </span>
    </span>
  </button>

  <div class="flex shrink-0 items-center gap-1.5">
    {#if item.availability !== "available"}
      <span
        class={cn("row-status", availabilityTone === "danger" ? "row-status-danger" : "row-status-warning")}
        title={availabilityLabel()}
      ><CircleAlert size={12} strokeWidth={1.8} /><span class="sr-only">{availabilityLabel()}</span></span>
    {/if}
    {#if item.activeSnoozeCount > 0}
      <span class="row-status" title={t("music.builder.snoozed")}><Clock3 size={12} strokeWidth={1.7} /></span>
    {/if}
    {#if reviewTone === "accent"}
      <span class="h-1.5 w-1.5 rounded-full bg-primary" title={t("music.builder.unreviewed")}></span>
    {/if}
    {#if duration}<span class="w-10 text-right text-[0.65rem] tabular-nums text-muted-foreground">{duration}</span>{/if}
    <button
      type="button"
      class="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground opacity-70 transition hover:bg-accent hover:text-accent-foreground group-hover:opacity-100 focus:opacity-100"
      onclick={(event) => onMore(item, event.currentTarget)}
      aria-label={t("music.builder.moreActions")}
    ><MoreHorizontal size={15} strokeWidth={1.7} /></button>
  </div>
</div>

<style>
  .music-builder-row {
    display: flex;
    height: 4rem;
    min-width: 0;
    align-items: center;
    gap: 0.65rem;
    border: 1px solid transparent;
    border-radius: 0.75rem;
    padding: 0.35rem 0.45rem;
    transition: background-color 120ms ease, border-color 120ms ease, transform 120ms ease;
  }
  .music-builder-row:hover { background: color-mix(in srgb, var(--accent) 55%, transparent); }
  .music-builder-row-selected { border-color: color-mix(in srgb, var(--primary) 36%, transparent); background: color-mix(in srgb, var(--primary) 8%, var(--card)); }
  .music-builder-row-playing { box-shadow: inset 3px 0 color-mix(in srgb, var(--primary) 82%, transparent); }
  .music-builder-artwork { position: relative; display: grid; height: 2.75rem; width: 2.75rem; flex: none; place-items: center; overflow: hidden; border-radius: 0.65rem; background: linear-gradient(145deg, color-mix(in srgb, var(--primary) 13%, var(--secondary)), var(--secondary)); color: var(--muted-foreground); }
  .music-builder-play-overlay { position: absolute; inset: 0; display: grid; place-items: center; background: color-mix(in srgb, var(--background) 55%, transparent); color: var(--foreground); opacity: 0; transition: opacity 120ms ease; }
  .music-builder-artwork:hover .music-builder-play-overlay, .music-builder-artwork:focus-visible .music-builder-play-overlay { opacity: 1; }
  .row-status { display: inline-flex; color: var(--muted-foreground); }
  .row-status-danger { color: var(--destructive); }
  .row-status-warning { color: color-mix(in srgb, var(--destructive) 65%, var(--foreground)); }
  .playing-bars { display: inline-flex; height: 0.8rem; align-items: end; gap: 1px; color: var(--primary); }
  .playing-bars i { width: 2px; border-radius: 1px; background: currentColor; animation: playing-wave 0.8s ease-in-out infinite alternate; }
  .playing-bars i:nth-child(1) { height: 45%; }
  .playing-bars i:nth-child(2) { height: 90%; animation-delay: 160ms; }
  .playing-bars i:nth-child(3) { height: 60%; animation-delay: 320ms; }
  @media (prefers-reduced-motion: reduce) { .music-builder-row, .music-builder-play-overlay { transition: none; } .playing-bars i { animation: none; } }
  @keyframes playing-wave { to { height: 25%; } }
</style>
