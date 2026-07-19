<script lang="ts">
  import { onMount, tick } from "svelte";
  import AlertCircle from "@lucide/svelte/icons/alert-circle";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Play from "@lucide/svelte/icons/play";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Settings2 from "@lucide/svelte/icons/settings-2";
  import {
    getLocalRootBindings,
    getMusicPlaylistPlaybackEntries,
    getMusicPlaylistSummaries,
  } from "$lib/api/music-library";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { projectMusicPlaylistPlayback } from "$lib/music/music-playlist-playback";
  import { systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import { getMusicPlayer } from "$lib/stores/music-player.svelte";
  import { requireActiveVaultIdentity } from "$lib/vault/active-vault";
  import { cn } from "$lib/utils";

  let {
    onOpenBuilder,
    onNewPlaylist,
  }: {
    onOpenBuilder: () => void;
    onNewPlaylist: () => void;
  } = $props();

  const { t } = getLocalization();
  const player = getMusicPlayer();
  let root = $state<HTMLElement | null>(null);
  let trigger = $state<HTMLButtonElement | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let open = $state(false);
  let search = $state("");
  let playlists = $state<MusicPlaylistSummary[]>([]);
  let loading = $state(false);
  let playingId = $state<string | null>(null);
  let error = $state<string | null>(null);
  let noEligiblePlaylist = $state<MusicPlaylistSummary | null>(null);

  const recentIdSet = $derived(new Set(player.recentPlaylistIds));
  const matching = $derived.by(() => {
    const query = search.trim().toLocaleLowerCase();
    return playlists.filter((playlist) => !query
      || systemMusicPlaylistName(playlist.id, playlist.name, t).toLocaleLowerCase().includes(query));
  });
  const recent = $derived(player.recentPlaylistIds
    .map((id) => matching.find((playlist) => playlist.id === id))
    .filter((playlist): playlist is MusicPlaylistSummary => Boolean(playlist)));
  const remaining = $derived(matching.filter((playlist) => !recentIdSet.has(playlist.id)));

  onMount(() => {
    const handlePointer = (event: PointerEvent) => {
      if (open && event.target instanceof Node && root && !root.contains(event.target)) close();
    };
    const handleKey = (event: KeyboardEvent) => {
      if (open && event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        close();
        trigger?.focus();
      }
    };
    window.addEventListener("pointerdown", handlePointer);
    window.addEventListener("keydown", handleKey, true);
    return () => {
      window.removeEventListener("pointerdown", handlePointer);
      window.removeEventListener("keydown", handleKey, true);
    };
  });

  async function toggle(): Promise<void> {
    open = !open;
    if (!open) return;
    error = null;
    noEligiblePlaylist = null;
    await Promise.all([refresh(), tick()]);
    searchInput?.focus();
  }

  function close(): void {
    open = false;
    search = "";
  }

  function handleFocusOut(event: FocusEvent): void {
    if (!open || !(event.relatedTarget instanceof Node) || root?.contains(event.relatedTarget)) return;
    close();
  }

  async function refresh(): Promise<void> {
    loading = true;
    try {
      playlists = await getMusicPlaylistSummaries(Date.now(), 0, 500);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function play(playlist: MusicPlaylistSummary): Promise<void> {
    if (playingId) return;
    playingId = playlist.id;
    error = null;
    noEligiblePlaylist = null;
    try {
      const entries = await getMusicPlaylistPlaybackEntries(playlist.id, Date.now());
      const rootIds = [...new Set(entries.flatMap((entry) => entry.rootId ? [entry.rootId] : []))];
      const bindings = rootIds.length > 0
        ? await getLocalRootBindings(requireActiveVaultIdentity(), rootIds)
        : [];
      const projection = projectMusicPlaylistPlayback(entries, bindings, {
        nowMs: Date.now(),
        online: player.online,
      });
      const loaded = await player.loadSavedPlaylist(
        playlist.id,
        systemMusicPlaylistName(playlist.id, playlist.name, t),
        projection.entries,
        playlist.shuffleEnabled,
        playlist.repeatMode,
        { structuralSkipped: projection.structuralSkipped },
      );
      if (loaded) close();
      else noEligiblePlaylist = playlist;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      playingId = null;
    }
  }

  function openBuilder(): void {
    close();
    onOpenBuilder();
  }

  function newPlaylist(): void {
    close();
    onNewPlaylist();
  }
</script>

<div bind:this={root} class="relative z-20 min-w-0" onfocusout={handleFocusOut}>
  <button
    bind:this={trigger}
    type="button"
    onclick={() => { void toggle(); }}
    class={cn(
      "flex h-7 max-w-56 min-w-0 items-center gap-1.5 rounded-md bg-secondary px-2.5 text-[0.8rem] font-medium text-secondary-foreground transition-colors hover:bg-accent hover:text-accent-foreground",
      open && "bg-accent text-accent-foreground",
    )}
    aria-haspopup="dialog"
    aria-expanded={open}
    aria-label={t("music.launcher.choosePlaylist")}
    data-music-playlist-launcher
  >
    <ListMusic size={14} strokeWidth={1.5} class="shrink-0" />
    <span class="hidden min-w-0 truncate min-[620px]:inline">{player.activePlaylistName ?? t("music.launcher.playlists")}</span>
    <ChevronDown size={12} class="hidden shrink-0 min-[620px]:block" />
  </button>

  {#if open}
    <div
      role="dialog"
      aria-label={t("music.launcher.choosePlaylist")}
      tabindex="-1"
      class="playlist-launcher-popover absolute left-0 top-[calc(100%+0.4rem)] flex max-h-[min(30rem,calc(100vh-5rem))] w-[min(23rem,calc(100vw-1rem))] flex-col overflow-hidden rounded-xl border border-border/80 bg-popover text-popover-foreground shadow-2xl"
    >
      <div class="border-b border-border/60 p-2.5">
        <div class="flex items-center gap-2 rounded-lg bg-secondary/65 px-2.5">
          <Search size={13} class="shrink-0 text-muted-foreground" />
          <input bind:this={searchInput} bind:value={search} aria-label={t("music.launcher.search")} placeholder={t("music.launcher.search")} class="h-8 min-w-0 flex-1 bg-transparent text-xs outline-none placeholder:text-muted-foreground" />
        </div>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-2" data-music-scrollable="true">
        {#if loading && playlists.length === 0}
          <div class="grid min-h-32 place-items-center text-center text-xs text-muted-foreground"><div><LoaderCircle class="mx-auto mb-2 animate-spin motion-reduce:animate-none" size={18} /><p>{t("music.launcher.loading")}</p></div></div>
        {:else if error}
          <div class="rounded-lg border border-destructive/25 bg-destructive/8 p-3 text-xs"><div class="flex gap-2"><AlertCircle size={15} class="mt-0.5 shrink-0 text-destructive" /><p class="min-w-0 wrap-break-word">{error}</p></div><button type="button" onclick={() => { void refresh(); }} class="mt-2 font-medium text-primary hover:underline">{t("music.launcher.retry")}</button></div>
        {:else if noEligiblePlaylist}
          <div class="rounded-lg border border-warning/30 bg-warning/8 p-3 text-xs">
            <p class="font-semibold">{t("music.launcher.nothingPlayable")}</p>
            <p class="mt-1 leading-relaxed text-muted-foreground">{player.online ? t("music.launcher.blockedExplanation") : t("music.launcher.offlineExplanation")}</p>
            <div class="mt-2 flex flex-wrap gap-2"><button type="button" onclick={openBuilder} class="rounded-md bg-secondary px-2.5 py-1.5 font-medium hover:bg-accent">{t("music.launcher.openIssues")}</button><button type="button" onclick={() => { noEligiblePlaylist = null; }} class="rounded-md px-2.5 py-1.5 font-medium text-primary hover:bg-primary/10">{t("music.launcher.chooseAnother")}</button></div>
          </div>
        {:else if matching.length === 0}
          <div class="grid min-h-32 place-items-center px-5 text-center"><div><ListMusic class="mx-auto mb-2 text-muted-foreground" size={20} /><p class="text-xs font-medium">{playlists.length === 0 ? t("music.launcher.empty") : t("music.launcher.noMatches")}</p><p class="mt-1 text-[0.68rem] leading-relaxed text-muted-foreground">{playlists.length === 0 ? t("music.launcher.emptyHint") : t("music.launcher.noMatchesHint")}</p></div></div>
        {:else}
          {#if recent.length > 0}
            <p class="px-2 pb-1 pt-0.5 text-[0.62rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">{t("music.launcher.recent")}</p>
            {#each recent as playlist (playlist.id)}
              <button type="button" onclick={() => { void play(playlist); }} disabled={Boolean(playingId)} class="group flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left hover:bg-accent disabled:opacity-60">
                <span class={cn("grid h-7 w-7 shrink-0 place-items-center rounded-md bg-secondary text-muted-foreground", player.activePlaylistId === playlist.id && "bg-primary/15 text-primary")}>{#if playingId === playlist.id}<LoaderCircle class="animate-spin motion-reduce:animate-none" size={13} />{:else if player.activePlaylistId === playlist.id}<Check size={13} />{:else}<Play size={13} />{/if}</span>
                <span class="min-w-0 flex-1"><span class="block truncate text-xs font-medium">{systemMusicPlaylistName(playlist.id, playlist.name, t)}</span><span class="block truncate text-[0.64rem] text-muted-foreground">{t("music.launcher.playlistCounts", playlist.eligibleCount, playlist.totalCount)}{#if playlist.unavailableCount + playlist.snoozedCount > 0} · {t("music.launcher.issueCount", playlist.unavailableCount + playlist.snoozedCount)}{/if}</span></span>
              </button>
            {/each}
          {/if}
          {#if remaining.length > 0}
            <p class="px-2 pb-1 pt-2 text-[0.62rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">{t("music.launcher.allPlaylists")}</p>
            {#each remaining as playlist (playlist.id)}
              <button type="button" onclick={() => { void play(playlist); }} disabled={Boolean(playingId)} class="group flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left hover:bg-accent disabled:opacity-60">
                <span class={cn("grid h-7 w-7 shrink-0 place-items-center rounded-md bg-secondary text-muted-foreground", player.activePlaylistId === playlist.id && "bg-primary/15 text-primary")}>{#if playingId === playlist.id}<LoaderCircle class="animate-spin motion-reduce:animate-none" size={13} />{:else if player.activePlaylistId === playlist.id}<Check size={13} />{:else}<Play size={13} />{/if}</span>
                <span class="min-w-0 flex-1"><span class="block truncate text-xs font-medium">{systemMusicPlaylistName(playlist.id, playlist.name, t)}</span><span class="block truncate text-[0.64rem] text-muted-foreground">{t("music.launcher.playlistCounts", playlist.eligibleCount, playlist.totalCount)}{#if playlist.unavailableCount + playlist.snoozedCount > 0} · {t("music.launcher.issueCount", playlist.unavailableCount + playlist.snoozedCount)}{/if}</span></span>
              </button>
            {/each}
          {/if}
        {/if}
      </div>

      <div class="grid grid-cols-2 gap-1.5 border-t border-border/60 p-2">
        <button type="button" onclick={openBuilder} class="flex h-8 items-center justify-center gap-1.5 rounded-md bg-secondary text-[0.68rem] font-medium hover:bg-accent"><Settings2 size={13} />{t("music.launcher.openBuilder")}</button>
        <button type="button" onclick={newPlaylist} class="flex h-8 items-center justify-center gap-1.5 rounded-md bg-primary text-[0.68rem] font-medium text-primary-foreground hover:bg-primary/90"><Plus size={13} />{t("music.launcher.newPlaylist")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  @media (max-height: 260px) {
    .playlist-launcher-popover { position: fixed; inset: 0.5rem; width: auto; max-height: none; }
  }
</style>
