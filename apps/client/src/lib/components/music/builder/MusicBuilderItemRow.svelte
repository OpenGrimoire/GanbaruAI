<script lang="ts">
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import MoreHorizontal from "@lucide/svelte/icons/ellipsis";
  import Play from "@lucide/svelte/icons/play";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicItemListEntry } from "$lib/music/library-contracts";
  import {
    formatMusicDuration,
    musicAvailabilityTone,
    musicItemSecondaryText,
    musicReviewTone,
  } from "$lib/music/music-builder-presentation";
  import { cn } from "$lib/utils";
  import MusicArtworkThumbnail from "./MusicArtworkThumbnail.svelte";

  let {
    item,
    selected = false,
    playing = false,
    position = undefined,
    setSize = undefined,
    playlistMode = false,
    reorderEnabled = false,
    dropEdge = null,
    onSelect,
    onPlay = () => undefined,
    onMore = () => undefined,
    onMove = () => undefined,
    onMoveTo = () => undefined,
    onDragStart = () => undefined,
    onDragEnd = () => undefined,
    onDragOver = () => undefined,
    onDrop = () => undefined,
  }: {
    item: MusicItemListEntry;
    selected?: boolean;
    playing?: boolean;
    position?: number;
    setSize?: number;
    playlistMode?: boolean;
    reorderEnabled?: boolean;
    dropEdge?: "before" | "after" | null;
    onSelect: (item: MusicItemListEntry, event: MouseEvent) => void;
    onPlay?: (item: MusicItemListEntry) => void;
    onMore?: (item: MusicItemListEntry, anchor: HTMLElement) => void;
    onMove?: (item: MusicItemListEntry, direction: "up" | "down") => void;
    onMoveTo?: (item: MusicItemListEntry, position: number) => void;
    onDragStart?: (item: MusicItemListEntry, event: DragEvent) => void;
    onDragEnd?: () => void;
    onDragOver?: (item: MusicItemListEntry, event: DragEvent) => void;
    onDrop?: (item: MusicItemListEntry, event: DragEvent) => void;
  } = $props();

  const { t } = getLocalization();
  const secondary = $derived(musicItemSecondaryText(item, t("music.builder.noArtist"), t("music.builder.noAlbum")));
  const duration = $derived(formatMusicDuration(item.durationMs));
  const availabilityTone = $derived(musicAvailabilityTone(item.availability));
  const reviewTone = $derived(musicReviewTone(item.reviewState));
  let menuOpen = $state(false);
  let positionDraft = $state("");
  let menuTrigger = $state<HTMLButtonElement | null>(null);
  let menuNode = $state<HTMLElement | null>(null);

  $effect(() => {
    if (!menuOpen) return;
    const handlePointerDown = (event: PointerEvent): void => {
      if (!(event.target instanceof Node)) return;
      if (menuNode?.contains(event.target) || menuTrigger?.contains(event.target)) return;
      closeMenu(false);
    };
    window.addEventListener("pointerdown", handlePointerDown, true);
    return () => window.removeEventListener("pointerdown", handlePointerDown, true);
  });

  function closeMenu(restoreFocus = true): void {
    menuOpen = false;
    positionDraft = "";
    if (restoreFocus && menuTrigger?.isConnected) queueMicrotask(() => menuTrigger?.focus());
  }

  function toggleMenu(): void {
    if (menuOpen) {
      closeMenu();
      return;
    }
    menuOpen = true;
    queueMicrotask(() => (menuNode?.querySelector<HTMLElement>("button, input") ?? menuNode)?.focus());
  }

  function handleMenuFocusOut(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (!(next instanceof Node) || menuNode?.contains(next) || menuTrigger?.contains(next)) return;
    closeMenu(false);
  }

  function availabilityLabel(): string {
    if (item.availability === "available") return t("music.builder.available");
    if (item.availability === "missing") return t("music.builder.missing");
    if (item.availability === "unavailable") return t("music.builder.unavailable");
    if (item.availability === "ambiguous") return t("music.builder.ambiguous");
    return t("music.builder.unknownAvailability");
  }

  function handleMenuKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || !menuOpen) return;
    event.preventDefault();
    event.stopPropagation();
    closeMenu();
  }
</script>

<div
  class={cn("music-builder-row group", selected && "music-builder-row-selected", playing && "music-builder-row-playing", dropEdge === "before" && "music-builder-drop-before", dropEdge === "after" && "music-builder-drop-after")}
  role="listitem"
  tabindex="-1"
  aria-posinset={position}
  aria-setsize={setSize}
  data-music-focus-key={`item:${item.id}`}
  ondragover={(event) => onDragOver(item, event)}
  ondrop={(event) => onDrop(item, event)}
>
  {#if playlistMode && reorderEnabled}
    <button type="button" draggable="true" ondragstart={(event) => onDragStart(item, event)} ondragend={onDragEnd} class="-mr-1 grid h-8 w-5 shrink-0 cursor-grab place-items-center rounded text-muted-foreground hover:bg-accent active:cursor-grabbing" aria-label={t("music.builder.dragToReorder", item.title)} title={t("music.builder.dragToReorder", item.title)}><GripVertical size={13} /></button>
  {/if}
  <button
    type="button"
    class="music-builder-artwork"
    onclick={(event) => {
      event.stopPropagation();
      onPlay(item);
    }}
    aria-label={t("music.play")}
  >
    <MusicArtworkThumbnail path={item.artworkOverride} sourceKind={item.sourceKind} version={item.updatedAt} />
    <span class="music-builder-play-overlay"><Play size={14} fill="currentColor" strokeWidth={1.5} /></span>
  </button>

  <button
    type="button"
    class="grid min-w-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(5rem,0.65fr)] items-center gap-3 text-left max-[470px]:grid-cols-1 max-[470px]:gap-0.5"
    onclick={(event) => onSelect(item, event)}
    aria-label={t("music.builder.selectTrack", item.title)}
    aria-pressed={selected}
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
      <span class="mt-0.5 flex min-w-0 items-center gap-2 truncate text-[0.62rem] text-muted-foreground/75">
        {#if playlistMode && item.membershipPosition !== null}
          <span>{t("music.builder.playlistPosition", item.membershipPosition + 1)}</span>
          {#if item.membershipWeight}<span>{t(`music.builder.weight.${item.membershipWeight}`)}</span>{/if}
          {#if item.membershipEnabled === false}<span class="text-warning">{t("music.builder.membershipDisabled")}</span>{/if}
        {:else}
          <span>{t("music.builder.playlistsMembership", item.playlistCount)}</span>
        {/if}
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
      <span class="row-status" title={t("music.builder.snoozed")}><Clock3 size={12} strokeWidth={1.7} /><span class="sr-only">{t("music.builder.snoozed")}</span></span>
    {/if}
    {#if reviewTone === "accent"}
      <span class="h-1.5 w-1.5 rounded-full bg-primary" title={t("music.builder.unreviewed")}><span class="sr-only">{t("music.builder.unreviewed")}</span></span>
    {/if}
    {#if duration}<span class="w-10 text-right text-[0.65rem] tabular-nums text-muted-foreground">{duration}</span>{/if}
    <div class="relative">
    <button
      bind:this={menuTrigger}
      data-music-row-menu={item.id}
      type="button"
      class="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground opacity-70 transition hover:bg-accent hover:text-accent-foreground group-hover:opacity-100 focus:opacity-100"
      onclick={(event) => { if (playlistMode) toggleMenu(); else onMore(item, event.currentTarget); }}
      onkeydown={handleMenuKeydown}
      aria-label={t("music.builder.moreActions")}
      aria-haspopup={playlistMode ? "dialog" : undefined}
      aria-expanded={playlistMode ? menuOpen : undefined}
    ><MoreHorizontal size={15} strokeWidth={1.7} /></button>
    {#if playlistMode && menuOpen}
      <div bind:this={menuNode} role="dialog" aria-label={t("music.builder.moreActions")} tabindex="-1" onkeydown={handleMenuKeydown} onfocusout={handleMenuFocusOut} class="absolute bottom-[calc(100%+0.25rem)] right-0 z-20 w-44 rounded-lg border border-border/70 bg-card p-1 shadow-xl">
        {#if reorderEnabled}
        <button type="button" onclick={() => { closeMenu(); onMove(item, "up"); }} class="row-menu-item"><ArrowUp size={12} />{t("music.builder.moveUp")}</button>
        <button type="button" onclick={() => { closeMenu(); onMove(item, "down"); }} class="row-menu-item"><ArrowDown size={12} />{t("music.builder.moveDown")}</button>
        <form class="mt-1 flex gap-1 border-t border-border/60 pt-1" onsubmit={(event) => { event.preventDefault(); const position = Number(positionDraft); if (Number.isInteger(position) && position > 0) { closeMenu(); onMoveTo(item, position - 1); } }}>
          <input bind:value={positionDraft} inputmode="numeric" aria-label={t("music.builder.moveToPosition")} placeholder="#" class="h-7 min-w-0 flex-1 rounded bg-background px-2 text-[0.68rem] outline-none focus:ring-1 focus:ring-primary" />
          <button type="submit" class="h-7 rounded bg-secondary px-2 text-[0.65rem] font-medium">{t("music.builder.move")}</button>
        </form>
        {:else}<p class="p-2 text-[0.64rem] leading-relaxed text-muted-foreground">{t("music.builder.reorderManualSortOnly")}</p>{/if}
      </div>
    {/if}
    </div>
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
  .music-builder-drop-before { box-shadow: inset 0 2px var(--primary); }
  .music-builder-drop-after { box-shadow: inset 0 -2px var(--primary); }
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
  .row-menu-item { display: flex; height: 1.8rem; width: 100%; align-items: center; gap: 0.45rem; border-radius: 0.35rem; padding-inline: 0.45rem; font-size: 0.68rem; }
  .row-menu-item:hover { background: var(--accent); color: var(--accent-foreground); }
  @media (prefers-reduced-motion: reduce) { .music-builder-row, .music-builder-play-overlay { transition: none; } .playing-bars i { animation: none; } }
  @keyframes playing-wave { to { height: 25%; } }
</style>
