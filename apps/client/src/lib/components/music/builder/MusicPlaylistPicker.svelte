<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Minus from "@lucide/svelte/icons/minus";
  import Search from "@lucide/svelte/icons/search";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { sortReviewPlaylists } from "$lib/music/music-review";
  import type { MusicFocusFit, MusicIntendedUse, MusicPlaylistSummary, MusicWeight } from "$lib/music/library-contracts";
  import { cn } from "$lib/utils";

  let {
    playlists,
    checkedIds,
    mixedIds = new Set<string>(),
    search,
    onSearch,
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
    search: string;
    onSearch: (value: string) => void;
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
  const visible = $derived(sortReviewPlaylists(playlists, checkedIds, search));

  function intendedUseLabel(use: MusicIntendedUse): string {
    return t(`music.builder.intendedUse.${use}`);
  }

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
  <label class="mx-2 mt-2 flex h-8 shrink-0 items-center gap-2 rounded-md bg-background px-2.5">
    <Search size={14} class="text-muted-foreground" />
    <input use:searchInputAction value={search} oninput={(event) => onSearch(event.currentTarget.value)} class="min-w-0 flex-1 bg-transparent text-xs outline-none" placeholder={t("music.builder.searchPlaylists")} />
  </label>
  <div class="min-h-0 flex-1 overflow-y-auto p-2" data-music-scrollable="true">
    {#each visible as playlist (playlist.id)}
      {@const checked = checkedIds.has(playlist.id)}
      {@const mixed = mixedIds.has(playlist.id)}
      <div class={cn("mb-1 flex min-w-0 flex-wrap items-center gap-2 rounded-lg border px-2 py-2 transition-colors", checked || mixed ? "border-primary/35 bg-primary/8" : "border-transparent hover:bg-accent/60")}>
        <label class="relative grid h-5 w-5 shrink-0 place-items-center">
          <input type="checkbox" checked={checked} use:triStateAction={mixed} onchange={() => onToggle(playlist)} aria-label={checked ? t("music.builder.removeFromPlaylist", playlist.name) : t("music.builder.addToPlaylist", playlist.name)} class="peer absolute inset-0 opacity-0" />
          <span class={cn("pointer-events-none grid h-5 w-5 place-items-center rounded border", checked || mixed ? "border-primary bg-primary text-primary-foreground" : "border-border bg-background")}>
            {#if mixed}<Minus size={12} strokeWidth={2.5} />{:else if checked}<Check size={13} strokeWidth={2.5} />{/if}
          </span>
        </label>
        <button type="button" data-review-playlist-id={playlist.id} onclick={() => onToggle(playlist)} class="min-w-0 flex-1 text-left">
          <span class="flex items-center gap-2"><strong class="min-w-0 flex-1 truncate text-xs font-medium">{playlist.name}</strong><span class="text-[0.6rem] tabular-nums text-muted-foreground">{playlist.totalCount}</span></span>
          <span class="block truncate text-[0.65rem] text-muted-foreground">{playlist.intendedUses.length > 0 ? playlist.intendedUses.map(intendedUseLabel).join(" · ") : playlist.description || t("music.builder.playlistTrackCount", playlist.totalCount)}</span>
        </button>
        {#if showIssue}<span class="shrink-0 text-[0.6rem] text-warning" title={issueLabel}>{issueLabel}</span>{/if}
        {#if weights[playlist.id]}
          <button type="button" onclick={() => onCycleWeight(playlist)} disabled={busyIds.has(playlist.id)} class="shrink-0 rounded-md bg-secondary px-2 py-1 text-[0.62rem] text-secondary-foreground disabled:opacity-40" title={t("music.builder.changeProbability")}>{t(`music.builder.weight.${weights[playlist.id]}`)}</button>
          <select value={focusFits[playlist.id] ?? "unknown"} onchange={(event) => onFocusFit(playlist, event.currentTarget.value as MusicFocusFit)} disabled={busyIds.has(playlist.id)} class="h-7 max-w-28 shrink-0 rounded-md border border-border/60 bg-secondary px-1 text-[0.6rem] text-secondary-foreground disabled:opacity-40" aria-label={t("music.builder.focusFitForPlaylist", playlist.name)}><option value="unknown">{t("music.builder.focusFitValue.unknown")}</option><option value="helpful">{t("music.builder.focusFitValue.helpful")}</option><option value="neutral">{t("music.builder.focusFitValue.neutral")}</option><option value="potentially-distracting">{t("music.builder.focusFitValue.potentially-distracting")}</option></select>
        {/if}
      </div>
      {#if advisoryPlaylistId === playlist.id}
        <div class="mb-2 ml-7 rounded-lg border border-warning/20 bg-warning/8 p-2.5 text-[0.65rem] leading-relaxed"><div class="flex items-start gap-2 text-warning"><CircleAlert class="mt-0.5 shrink-0" size={13} /><span>{advisoryText}</span></div><div class="mt-2 flex flex-wrap gap-1.5 pl-5"><button type="button" onclick={onAdvisoryKeep} class="h-7 rounded-md bg-secondary px-2 text-[0.61rem]">{t("music.builder.addAnyway")}</button><button type="button" onclick={onAdvisoryMark} class="h-7 rounded-md bg-secondary px-2 text-[0.61rem]">{t("music.builder.markPotentiallyDistracting")}</button><button type="button" onclick={onDisableGuidance} class="h-7 rounded-md px-2 text-[0.61rem] text-muted-foreground hover:bg-secondary">{t("music.builder.disableFocusGuidance")}</button></div></div>
      {/if}
      {#if errors[playlist.id]}<p class="mb-1 px-2 text-[0.62rem] text-destructive" role="alert">{errors[playlist.id]}</p>{/if}
    {:else}
      <p class="p-4 text-center text-xs text-muted-foreground">{t("music.builder.noPlaylistMatches")}</p>
    {/each}
  </div>
</div>
