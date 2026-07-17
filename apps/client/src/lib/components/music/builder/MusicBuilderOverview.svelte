<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Plus from "@lucide/svelte/icons/plus";
  import Download from "@lucide/svelte/icons/download";
  import Upload from "@lucide/svelte/icons/upload";
  import RadioTower from "@lucide/svelte/icons/radio-tower";
  import { flip } from "svelte/animate";
  import { onMount } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicIssue, MusicPlaylistSummary, MusicSourceSummary } from "$lib/music/library-contracts";
  import type { MusicBuilderDestination } from "$lib/music/music-builder-routing";
  import { partitionMusicPlaylists, systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import MusicBuilderAsyncState from "./MusicBuilderAsyncState.svelte";
  import MusicSoundscapeBuilder from "../MusicSoundscapeBuilder.svelte";

  let {
    destination,
    search = "",
    playlists,
    sources,
    issues,
    onNavigate,
    onPrimary = () => undefined,
    onImport = () => undefined,
    onExport = () => undefined,
  }: {
    destination: MusicBuilderDestination;
    search?: string;
    playlists: MusicPlaylistSummary[];
    sources: MusicSourceSummary[];
    issues: MusicIssue[];
    onNavigate: (destination: MusicBuilderDestination) => void;
    onPrimary?: () => void;
    onImport?: () => void;
    onExport?: () => void;
  } = $props();

  const { t } = getLocalization();
  let reducedMotion = $state(false);
  const visiblePlaylists = $derived(playlists.filter((playlist) => {
    const query = search.trim().toLocaleLowerCase();
    return !query || `${systemMusicPlaylistName(playlist.id, playlist.name, t)} ${playlist.description}`.toLocaleLowerCase().includes(query);
  }));
  const playlistSections = $derived(partitionMusicPlaylists(visiblePlaylists));

  onMount(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => { reducedMotion = query.matches; };
    update();
    query.addEventListener("change", update);
    return () => query.removeEventListener("change", update);
  });
</script>

<div class="overview-scroll h-full min-h-0 overflow-y-auto overscroll-contain p-3" data-music-scrollable="true">
  {#if destination.kind === "playlists"}
    <div class="mb-3 flex flex-wrap items-center justify-end gap-2"><button type="button" onclick={onImport} class="inline-flex h-8 items-center gap-1.5 rounded-lg bg-secondary px-3 text-xs font-medium"><Upload size={13} />{t("music.builder.importPlaylists")}</button><button type="button" onclick={onExport} disabled={playlists.length === 0} class="inline-flex h-8 items-center gap-1.5 rounded-lg bg-secondary px-3 text-xs font-medium disabled:opacity-40"><Download size={13} />{t("music.builder.exportPlaylists")}</button></div>
    {#if playlists.length === 0}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptyPlaylistsTitle")} description={t("music.builder.emptyPlaylistsDescription")} actionLabel={t("music.builder.newPlaylist")} onAction={onPrimary} />
    {:else if visiblePlaylists.length === 0}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.noPlaylistFilterResults")} description={t("music.builder.adjustPlaylistFilters")} />
    {:else}
      {#each [{ title: t("music.builder.defaultPlaylists"), playlists: playlistSections.defaults }, { title: t("music.builder.customPlaylists"), playlists: playlistSections.custom }] as section (section.title)}
        {#if section.playlists.length > 0}<h2 class="mb-2 mt-4 text-xs font-semibold text-muted-foreground first:mt-0">{section.title}</h2><div class="grid grid-cols-[repeat(auto-fill,minmax(min(14rem,100%),1fr))] gap-2.5">
        {#each section.playlists as playlist (playlist.id)}
          <button type="button" class="overview-card group" animate:flip={{ duration: reducedMotion ? 0 : 140 }} onclick={() => onNavigate({ kind: "playlist", playlistId: playlist.id })}>
            <span class="overview-icon"><ListMusic size={18} strokeWidth={1.45} /></span>
            <span class="min-w-0 flex-1 text-left">
              <strong class="block truncate text-xs font-semibold text-foreground">{systemMusicPlaylistName(playlist.id, playlist.name, t)}</strong>
              <span class="mt-1 block line-clamp-2 min-h-7 text-[0.68rem] leading-relaxed text-muted-foreground">{playlist.description || t("music.tracks", playlist.totalCount)}</span>
              <span class="mt-2 flex flex-wrap gap-1.5 text-[0.62rem] text-muted-foreground">
                <span>{t("music.tracks", playlist.totalCount)}</span><span>·</span><span>{playlist.localCount} {t("music.builder.local")}</span><span>·</span><span>{playlist.onlineCount} {t("music.builder.youtube")}</span>
              </span>
            </span>
          </button>
        {/each}
        </div>{/if}
      {/each}
      <button type="button" class="overview-card overview-add mt-2.5 w-full" onclick={onPrimary}><span class="overview-icon"><Plus size={18} /></span><span class="text-xs font-semibold">{t("music.builder.newPlaylist")}</span></button>
    {/if}
  {:else if destination.kind === "sources"}
    {#if sources.length === 0}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptySourcesTitle")} description={t("music.builder.emptySourcesDescription")} actionLabel={t("music.builder.addMusic")} onAction={onPrimary} />
    {:else}
      <div class="grid grid-cols-[repeat(auto-fill,minmax(min(15rem,100%),1fr))] gap-2.5">
        {#each sources as source (source.id)}
          <article class="overview-card">
            <span class="overview-icon"><RadioTower size={18} strokeWidth={1.45} /></span>
            <span class="min-w-0 flex-1">
              <span class="flex items-center gap-2"><strong class="min-w-0 flex-1 truncate text-xs font-semibold">{source.name}</strong><i class:source-warning={source.health === "issues"} class="source-health" title={source.health}></i></span>
              <span class="mt-1.5 block text-[0.68rem] text-muted-foreground">{t("music.tracks", source.itemCount)}</span>
              <span class="mt-2 flex flex-wrap gap-1.5 text-[0.62rem] text-muted-foreground"><span>{source.newCount} {t("music.builder.unreviewed")}</span>{#if source.openIssueCount > 0}<span>·</span><button type="button" class="text-destructive hover:underline" onclick={() => onNavigate({ kind: "issues" })}>{t("music.builder.issueCount", source.openIssueCount)}</button>{/if}</span>
            </span>
          </article>
        {/each}
      </div>
    {/if}
  {:else if destination.kind === "issues"}
    {#if issues.length === 0}
      <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptyIssuesTitle")} description={t("music.builder.emptyIssuesDescription")} />
    {:else}
      <div class="space-y-2">
        {#each issues as issue (issue.id)}
          <button type="button" class="overview-card w-full" animate:flip={{ duration: reducedMotion ? 0 : 140 }} onclick={() => issue.itemId && onNavigate({ kind: "library" })}>
            <span class="overview-icon overview-icon-warning"><AlertTriangle size={17} strokeWidth={1.5} /></span>
            <span class="min-w-0 flex-1 text-left"><strong class="block text-xs font-semibold text-foreground">{issue.issueKind}</strong><span class="mt-1 block text-[0.68rem] leading-relaxed text-muted-foreground">{issue.message}</span></span>
          </button>
        {/each}
      </div>
    {/if}
  {:else if destination.kind === "soundscapes"}
    <MusicSoundscapeBuilder />
  {:else}
    <MusicBuilderAsyncState kind="empty" title={t("music.builder.emptyLibraryTitle")} description={t("music.builder.emptyLibraryDescription")} actionLabel={t("music.builder.addMusic")} onAction={onPrimary} />
  {/if}
</div>

<style>
  .overview-scroll { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .overview-card { display: flex; min-width: 0; align-items: flex-start; gap: 0.75rem; overflow: hidden; border: 1px solid color-mix(in srgb, var(--border) 62%, transparent); border-radius: 0.85rem; background: color-mix(in srgb, var(--card) 75%, transparent); padding: 0.75rem; color: var(--foreground); box-shadow: 0 1px 0 color-mix(in srgb, white 3%, transparent); transition: border-color 130ms ease, background-color 130ms ease, transform 130ms ease; }
  button.overview-card:hover { border-color: color-mix(in srgb, var(--primary) 30%, var(--border)); background: color-mix(in srgb, var(--accent) 45%, var(--card)); transform: translateY(-1px); }
  .overview-icon { display: grid; height: 2.35rem; width: 2.35rem; flex: none; place-items: center; border-radius: 0.7rem; background: color-mix(in srgb, var(--primary) 10%, var(--secondary)); color: var(--muted-foreground); }
  .overview-icon-warning { color: var(--destructive); }
  .overview-add { min-height: 5.5rem; align-items: center; justify-content: center; border-style: dashed; color: var(--muted-foreground); }
  .source-health { height: 0.45rem; width: 0.45rem; flex: none; border-radius: 999px; background: color-mix(in srgb, var(--primary) 70%, var(--muted)); }
  .source-warning { background: var(--destructive); }
  @media (prefers-reduced-motion: reduce) { .overview-card, button.overview-card:hover { transition: none; transform: none; } }
</style>
