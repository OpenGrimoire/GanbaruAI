<script lang="ts">
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { flip } from "svelte/animate";
  import { onDestroy, untrack } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { moveMusicPlaylistOrder } from "$lib/music/music-playlist-order";
  import { isSystemMusicPlaylistId, orderMusicPlaylists, systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import MusicPlaylistIcon from "./MusicPlaylistIcon.svelte";

  let {
    playlists,
    onEdit,
    onDelete,
    onReorder,
    onDone,
  }: {
    playlists: MusicPlaylistSummary[];
    onEdit: (playlistId: string) => void;
    onDelete: (playlistId: string) => void;
    onReorder: (playlistIds: string[]) => Promise<boolean>;
    onDone: () => void;
  } = $props();

  type PointerDrag = {
    playlistId: string;
    pointerId: number;
    startX: number;
    startY: number;
    originIds: string[];
    moved: boolean;
  };

  const { t } = getLocalization();
  let visualPlaylists = $state<MusicPlaylistSummary[]>(untrack(() => orderMusicPlaylists(playlists)));
  let pointerDrag = $state<PointerDrag | null>(null);
  let keyboardPlaylistId = $state<string | null>(null);
  let keyboardOriginIds = $state<string[]>([]);
  let saving = $state(false);
  let reorderError = $state(false);
  let announcement = $state("");
  let scrollFrame: number | null = null;
  let scrollVelocity = 0;
  let managerRoot = $state<HTMLElement | null>(null);
  let previousBodyUserSelect = "";
  let bodySelectionLocked = false;

  $effect(() => {
    const ordered = orderMusicPlaylists(playlists);
    if (!pointerDrag && !keyboardPlaylistId && !saving) visualPlaylists = ordered;
  });

  function ids(): string[] {
    return visualPlaylists.map((playlist) => playlist.id);
  }

  function movePlaylist(playlistId: string, targetIndex: number): void {
    const sourceIndex = visualPlaylists.findIndex((playlist) => playlist.id === playlistId);
    if (sourceIndex < 0) return;
    const boundedTarget = Math.max(0, Math.min(targetIndex, visualPlaylists.length - 1));
    if (sourceIndex === boundedTarget) return;
    const moved = visualPlaylists[sourceIndex];
    if (!moved) return;
    const next = moveMusicPlaylistOrder(visualPlaylists, playlistId, boundedTarget);
    visualPlaylists = next;
    announcement = t("music.builder.playlistMoved", systemMusicPlaylistName(moved.id, moved.name, t), boundedTarget + 1, next.length);
  }

  function restoreOrder(originIds: readonly string[]): void {
    const byId = new Map(visualPlaylists.map((playlist) => [playlist.id, playlist]));
    visualPlaylists = originIds.flatMap((id) => {
      const playlist = byId.get(id);
      return playlist ? [playlist] : [];
    });
  }

  async function saveOrder(originIds: string[]): Promise<boolean> {
    const nextIds = ids();
    if (nextIds.every((id, index) => id === originIds[index])) return true;
    saving = true;
    reorderError = false;
    const saved = await onReorder(nextIds);
    if (!saved) {
      restoreOrder(originIds);
      reorderError = true;
      announcement = t("music.builder.playlistReorderFailed");
    }
    saving = false;
    return saved;
  }

  async function finishManaging(): Promise<void> {
    if (saving || pointerDrag) return;
    if (keyboardPlaylistId) {
      const origin = keyboardOriginIds;
      keyboardPlaylistId = null;
      keyboardOriginIds = [];
      if (!await saveOrder(origin)) return;
    }
    onDone();
  }

  function stopAutoScroll(): void {
    scrollVelocity = 0;
    if (scrollFrame !== null) cancelAnimationFrame(scrollFrame);
    scrollFrame = null;
  }

  function runAutoScroll(): void {
    const drag = pointerDrag;
    if (!drag || scrollVelocity === 0) {
      scrollFrame = null;
      return;
    }
    const scrollable = managerRoot?.closest<HTMLElement>("[data-music-scrollable='true']");
    if (scrollable) scrollable.scrollTop += scrollVelocity;
    scrollFrame = requestAnimationFrame(runAutoScroll);
  }

  function updateAutoScroll(clientY: number): void {
    const scrollable = managerRoot?.closest<HTMLElement>("[data-music-scrollable='true']");
    if (!scrollable) return;
    const rect = scrollable.getBoundingClientRect();
    const edge = Math.min(64, rect.height * 0.2);
    if (clientY < rect.top + edge) scrollVelocity = -Math.max(2, (rect.top + edge - clientY) / 5);
    else if (clientY > rect.bottom - edge) scrollVelocity = Math.max(2, (clientY - (rect.bottom - edge)) / 5);
    else scrollVelocity = 0;
    if (scrollVelocity !== 0 && scrollFrame === null) scrollFrame = requestAnimationFrame(runAutoScroll);
    if (scrollVelocity === 0) stopAutoScroll();
  }

  function startPointerDrag(event: PointerEvent, playlistId: string): void {
    if (saving || event.button !== 0 || !event.isPrimary) return;
    event.preventDefault();
    event.currentTarget instanceof HTMLElement && event.currentTarget.setPointerCapture(event.pointerId);
    pointerDrag = {
      playlistId,
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      originIds: ids(),
      moved: false,
    };
    previousBodyUserSelect = document.body.style.userSelect;
    document.body.style.userSelect = "none";
    bodySelectionLocked = true;
  }

  function movePointerDrag(event: PointerEvent): void {
    const drag = pointerDrag;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const moved = drag.moved || Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) >= 4;
    pointerDrag = { ...drag, moved };
    if (!moved) return;
    event.preventDefault();
    updateAutoScroll(event.clientY);
    const target = document.elementsFromPoint(event.clientX, event.clientY)
      .map((element) => element.closest<HTMLElement>("[data-playlist-manager-id]"))
      .find((element): element is HTMLElement => Boolean(element));
    const targetId = target?.dataset.playlistManagerId;
    if (!targetId || targetId === drag.playlistId) return;
    const targetIndex = visualPlaylists.findIndex((playlist) => playlist.id === targetId);
    if (targetIndex >= 0) movePlaylist(drag.playlistId, targetIndex);
  }

  function finishPointerDrag(event: PointerEvent): void {
    const drag = pointerDrag;
    if (!drag || drag.pointerId !== event.pointerId) return;
    pointerDrag = null;
    document.body.style.userSelect = previousBodyUserSelect;
    bodySelectionLocked = false;
    stopAutoScroll();
    if (drag.moved) void saveOrder(drag.originIds);
  }

  function cancelActiveReorder(): void {
    if (pointerDrag) {
      restoreOrder(pointerDrag.originIds);
      pointerDrag = null;
      document.body.style.userSelect = previousBodyUserSelect;
      bodySelectionLocked = false;
      stopAutoScroll();
      announcement = t("music.builder.playlistReorderCancelled");
    }
    if (keyboardPlaylistId) {
      restoreOrder(keyboardOriginIds);
      keyboardPlaylistId = null;
      keyboardOriginIds = [];
      announcement = t("music.builder.playlistReorderCancelled");
    }
  }

  function handleKeyboard(event: KeyboardEvent, playlist: MusicPlaylistSummary): void {
    if (saving) return;
    const active = keyboardPlaylistId === playlist.id;
    if (event.key === " " || event.key === "Enter") {
      event.preventDefault();
      if (!keyboardPlaylistId) {
        keyboardPlaylistId = playlist.id;
        keyboardOriginIds = ids();
        announcement = t("music.builder.playlistReorderPickedUp", systemMusicPlaylistName(playlist.id, playlist.name, t));
      } else if (active) {
        const origin = keyboardOriginIds;
        keyboardPlaylistId = null;
        keyboardOriginIds = [];
        void saveOrder(origin);
      }
      return;
    }
    if (!active) return;
    const index = visualPlaylists.findIndex((entry) => entry.id === playlist.id);
    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      movePlaylist(playlist.id, index - 1);
    } else if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      movePlaylist(playlist.id, index + 1);
    } else if (event.key === "Home") {
      event.preventDefault();
      movePlaylist(playlist.id, 0);
    } else if (event.key === "End") {
      event.preventDefault();
      movePlaylist(playlist.id, visualPlaylists.length - 1);
    } else if (event.key === "Escape") {
      event.preventDefault();
      cancelActiveReorder();
    }
  }

  onDestroy(() => {
    stopAutoScroll();
    if (bodySelectionLocked) document.body.style.userSelect = previousBodyUserSelect;
  });
</script>

<svelte:window onkeydown={(event) => { if (event.key === "Escape") cancelActiveReorder(); }} />

<div class="sticky top-0 z-20 -mx-1 mb-3 flex min-w-0 items-start justify-between gap-3 px-1 pb-2" style="background-color: var(--cal-bg);">
  <div class="min-w-0"><h2 class="text-sm font-semibold">{t("music.builder.managePlaylists")}</h2><p class="mt-0.5 text-[0.68rem] leading-relaxed text-muted-foreground">{t("music.builder.managePlaylistsHint")}</p></div>
  <button type="button" onclick={() => { void finishManaging(); }} disabled={saving || Boolean(pointerDrag)} class="h-8 shrink-0 rounded-lg bg-primary px-3 text-xs font-semibold text-primary-foreground disabled:opacity-50">{t("music.builder.doneManaging")}</button>
</div>
<div bind:this={managerRoot} class="grid grid-cols-[repeat(auto-fill,minmax(min(17rem,100%),1fr))] gap-2" aria-busy={saving}>
  {#each visualPlaylists as playlist, index (playlist.id)}
    {@const protectedPlaylist = isSystemMusicPlaylistId(playlist.id)}
    {@const playlistName = systemMusicPlaylistName(playlist.id, playlist.name, t)}
    <div
      class={`playlist-manager-row flex min-w-0 items-center gap-2 rounded-xl px-2 py-2 ${pointerDrag?.playlistId === playlist.id || keyboardPlaylistId === playlist.id ? "is-reordering" : ""}`}
      data-playlist-manager-id={playlist.id}
      animate:flip={{ duration: 160 }}
    >
      <button
        type="button"
        class={`grid h-9 w-7 shrink-0 touch-none place-items-center rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60 ${pointerDrag?.playlistId === playlist.id ? "cursor-grabbing" : "cursor-grab"}`}
        aria-label={t("music.builder.reorderPlaylist", playlistName, index + 1, visualPlaylists.length)}
        aria-pressed={keyboardPlaylistId === playlist.id}
        title={t("music.builder.reorderPlaylist", playlistName, index + 1, visualPlaylists.length)}
        disabled={saving}
        onpointerdown={(event) => startPointerDrag(event, playlist.id)}
        onpointermove={movePointerDrag}
        onpointerup={finishPointerDrag}
        onpointercancel={cancelActiveReorder}
        onkeydown={(event) => handleKeyboard(event, playlist)}
      ><GripVertical size={16} /></button>
      <span class="grid h-9 w-9 shrink-0 place-items-center rounded-lg bg-secondary text-foreground"><MusicPlaylistIcon icon={playlist.icon} size={17} /></span>
      <span class="min-w-0 flex-1">
        <strong class="block truncate text-xs font-semibold">{playlistName}</strong>
        <span class="mt-0.5 block text-[0.64rem] tabular-nums text-muted-foreground">{t("music.tracks", playlist.totalCount)}</span>
      </span>
      {#if protectedPlaylist}
        <button type="button" class="grid h-8 w-8 shrink-0 cursor-not-allowed place-items-center rounded-lg text-muted-foreground opacity-35" aria-disabled="true" aria-label={t("music.builder.defaultPlaylistEditProtected")} title={t("music.builder.defaultPlaylistEditProtected")}><Pencil size={14} /></button>
        <button type="button" class="grid h-8 w-8 shrink-0 cursor-not-allowed place-items-center rounded-lg text-muted-foreground opacity-35" aria-disabled="true" aria-label={t("music.builder.defaultPlaylistDeleteProtected")} title={t("music.builder.defaultPlaylistDeleteProtected")}><Trash2 size={14} /></button>
      {:else}
        <button type="button" class="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground" onclick={() => onEdit(playlist.id)} aria-label={t("music.builder.editNamedPlaylist", playlistName)} title={t("music.builder.editNamedPlaylist", playlistName)} disabled={saving}><Pencil size={14} /></button>
        <button type="button" class="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-muted-foreground hover:bg-destructive/10 hover:text-destructive" onclick={() => onDelete(playlist.id)} aria-label={t("music.builder.deleteNamedPlaylist", playlistName)} title={t("music.builder.deleteNamedPlaylist", playlistName)} disabled={saving}><Trash2 size={14} /></button>
      {/if}
    </div>
  {/each}
</div>
{#if reorderError}<p class="mt-2 text-xs text-destructive" role="alert">{t("music.builder.playlistReorderFailed")}</p>{/if}
<p class="sr-only" aria-live="polite">{announcement}</p>

<style>
  .playlist-manager-row { border: 1px solid color-mix(in srgb, var(--border) 58%, transparent); background: color-mix(in srgb, var(--card) 68%, transparent); transition: background-color 140ms ease, border-color 140ms ease, box-shadow 140ms ease; }
  .playlist-manager-row.is-reordering { border-color: color-mix(in srgb, var(--primary) 42%, var(--border)); background: color-mix(in srgb, var(--primary) 9%, var(--card)); box-shadow: 0 8px 24px color-mix(in srgb, black 12%, transparent); }
  @media (prefers-reduced-motion: reduce) { .playlist-manager-row { transition: none; } }
</style>
