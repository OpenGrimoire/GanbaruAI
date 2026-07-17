<script lang="ts">
  import Copy from "@lucide/svelte/icons/copy";
  import MoreHorizontal from "@lucide/svelte/icons/ellipsis";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Play from "@lucide/svelte/icons/play";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicPlaylist, MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { isSystemMusicPlaylistId, systemMusicPlaylistName } from "$lib/music/music-system-playlists";

  let {
    detail,
    summary,
    playing = false,
    onPlay,
    onEdit,
    onDuplicate,
    onDelete,
  }: {
    detail: MusicPlaylist;
    summary: MusicPlaylistSummary;
    playing?: boolean;
    onPlay: () => void;
    onEdit: () => void;
    onDuplicate: () => void;
    onDelete: () => void;
  } = $props();

  const { t } = getLocalization();
  let menuOpen = $state(false);
  const protectedIdentity = $derived(isSystemMusicPlaylistId(detail.id));
  const displayName = $derived(systemMusicPlaylistName(detail.id, detail.name, t));
</script>

<section class="shrink-0 border-b border-border/60 bg-card/35 px-3 py-3">
  <div class="flex min-w-0 items-start gap-3">
    <div class="min-w-0 flex-1">
      <div class="flex min-w-0 items-center gap-2"><h2 class="truncate text-base font-semibold">{displayName}</h2>{#if protectedIdentity}<span class="shrink-0 rounded-full bg-secondary px-2 py-0.5 text-[0.6rem] text-muted-foreground">{t("music.builder.protectedPlaylist")}</span>{/if}{#if playing}<span class="shrink-0 rounded-full bg-success/12 px-2 py-0.5 text-[0.62rem] font-medium text-success">{t("music.builder.playing")}</span>{/if}</div>
      {#if detail.description}<p class="mt-1 line-clamp-2 max-w-2xl text-[0.7rem] leading-relaxed text-muted-foreground">{detail.description}</p>{/if}
      <div class="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-[0.65rem] text-muted-foreground">
        {#if detail.intendedUses.length > 0}<span>{t("music.builder.playlistIntent", detail.intendedUses.map((use) => t(`music.builder.intendedUse.${use}`)).join(", "))}</span>{/if}
        <span>{t("music.builder.eligibleOfTotal", summary.eligibleCount, summary.totalCount)}</span>
        <span>{summary.localCount} {t("music.builder.local")}</span>
        <span>{summary.onlineCount} {t("music.builder.youtube")}</span>
        {#if summary.unavailableCount > 0}<span class="text-warning">{t("music.builder.unavailableCount", summary.unavailableCount)}</span>{/if}
        {#if summary.snoozedCount > 0}<span>{t("music.builder.snoozedCount", summary.snoozedCount)}</span>{/if}
        <span>{detail.shuffleEnabled ? t("music.builder.shuffleOnDefault") : t("music.builder.shuffleOffDefault")}</span>
        <span>{t("music.builder.repeatModeValue", detail.repeatMode)}</span>
      </div>
    </div>
    <button type="button" onclick={onPlay} class="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-lg bg-primary px-3 text-xs font-semibold text-primary-foreground"><Play size={14} fill="currentColor" />{t("music.play")}</button>
    <div class="relative shrink-0">
      <button type="button" onclick={() => menuOpen = !menuOpen} class="grid h-9 w-9 place-items-center rounded-lg bg-secondary" aria-label={t("music.builder.moreActions")} aria-expanded={menuOpen}><MoreHorizontal size={16} /></button>
      {#if menuOpen}
        <div class="absolute right-0 top-[calc(100%+0.35rem)] z-30 w-40 rounded-lg border border-border/70 bg-card p-1 shadow-xl">
          <button type="button" onclick={() => { menuOpen = false; onEdit(); }} class="playlist-menu-item"><Pencil size={13} />{t("music.builder.editPlaylist")}</button>
          <button type="button" onclick={() => { menuOpen = false; onDuplicate(); }} class="playlist-menu-item"><Copy size={13} />{t("music.builder.duplicatePlaylist")}</button>
          {#if !protectedIdentity}<button type="button" onclick={() => { menuOpen = false; onDelete(); }} class="playlist-menu-item text-destructive"><Trash2 size={13} />{t("music.builder.deletePlaylist")}</button>{/if}
        </div>
      {/if}
    </div>
  </div>
</section>

<style>
  .playlist-menu-item { display: flex; width: 100%; height: 2rem; align-items: center; gap: 0.5rem; border-radius: 0.4rem; padding-inline: 0.55rem; font-size: 0.7rem; text-align: left; }
  .playlist-menu-item:hover { background: var(--accent); color: var(--accent-foreground); }
</style>
