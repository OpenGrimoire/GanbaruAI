<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Minus from "@lucide/svelte/icons/minus";
  import Search from "@lucide/svelte/icons/search";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { sortReviewPlaylists } from "$lib/music/music-review";
  import { partitionMusicPlaylists, systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { cn } from "$lib/utils";
  import MusicPlaylistIcon from "./MusicPlaylistIcon.svelte";

  let {
    playlists,
    checkedIds,
    mixedIds = new Set<string>(),
    search = "",
    onSearch = () => undefined,
    showSearch = false,
    showSections = false,
    onToggle,
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
    errors?: Record<string, string>;
    showIssue?: boolean;
    issueLabel?: string;
    onSearchInput?: (element: HTMLInputElement | null) => void;
  } = $props();

  const { t } = getLocalization();
  const visible = $derived(sortReviewPlaylists(playlists, "").filter((playlist) => {
    const query = search.trim().toLocaleLowerCase();
    return !query || systemMusicPlaylistName(playlist.id, playlist.name, t).toLocaleLowerCase().includes(query);
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
      <label class={cn("playlist-card flex min-w-0 cursor-pointer flex-wrap items-center gap-2 rounded-lg px-3 py-2.5 transition-colors", checked || mixed ? "bg-primary/10" : "bg-secondary/35")}>
        <span class="flex min-w-0 flex-1 items-center gap-3 text-left">
          <span class={cn("grid h-8 w-8 shrink-0 place-items-center", checked || mixed ? "text-primary" : "text-muted-foreground")}>
            <MusicPlaylistIcon icon={playlist.icon} size={16} />
          </span>
          <span class="min-w-0 flex-1"><strong class="block truncate text-xs font-medium">{playlistName}</strong><span class="block text-[0.62rem] tabular-nums text-muted-foreground">{t("music.tracks", playlist.totalCount)}</span></span>
        </span>
        <span class="relative grid h-6 w-6 shrink-0 place-items-center">
          <input type="checkbox" data-review-playlist-id={playlist.id} checked={checked} use:triStateAction={mixed} onchange={() => onToggle(playlist)} aria-label={checked ? t("music.builder.removeFromPlaylist", playlist.name) : t("music.builder.addToPlaylist", playlist.name)} aria-describedby={errorId} class="peer absolute inset-0 opacity-0" />
          <span class={cn("pointer-events-none grid h-5 w-5 place-items-center rounded border peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring", checked || mixed ? "border-primary bg-primary text-primary-foreground" : "border-border bg-background/80")}>
            {#if mixed}<Minus size={12} strokeWidth={2.5} />{:else if checked}<Check size={13} strokeWidth={2.5} />{/if}
          </span>
        </span>
        {#if showIssue}<span class="shrink-0 text-[0.6rem] text-warning" title={issueLabel}>{issueLabel}</span>{/if}
      </label>
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
