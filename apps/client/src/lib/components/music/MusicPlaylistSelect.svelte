<script lang="ts">
  import { tick } from "svelte";
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Search from "@lucide/svelte/icons/search";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import { systemMusicPlaylistName } from "$lib/music/music-system-playlists";
  import { cn } from "$lib/utils";
  import { portal } from "$lib/utils/portal";
  import {
    pickSelectPopoverGeometry,
    type SelectPopoverGeometry,
  } from "$lib/components/settings/customSelectPosition";

  let {
    value,
    playlists,
    onChange,
    label,
    disabled = false,
    loading = false,
    class: className = "",
  }: {
    value: string | null;
    playlists: readonly MusicPlaylistSummary[];
    onChange: (playlistId: string | null) => void;
    label: string;
    disabled?: boolean;
    loading?: boolean;
    class?: string;
  } = $props();

  const { t } = getLocalization();
  let open = $state(false);
  let search = $state("");
  let trigger = $state<HTMLButtonElement | null>(null);
  let popover = $state<HTMLDivElement | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let geometry = $state<SelectPopoverGeometry | null>(null);

  const selected = $derived(playlists.find((playlist) => playlist.id === value) ?? null);
  const missing = $derived(Boolean(value && !selected && !loading));
  const matching = $derived.by(() => {
    const query = search.trim().toLocaleLowerCase();
    return playlists.filter((playlist) => !query
      || systemMusicPlaylistName(playlist.id, playlist.name, t).toLocaleLowerCase().includes(query));
  });

  async function toggle(): Promise<void> {
    if (disabled) return;
    open = !open;
    if (!open) return;
    geometry = null;
    await tick();
    position();
    searchInput?.focus();
  }

  function close(restoreFocus = false): void {
    open = false;
    search = "";
    if (restoreFocus && trigger?.isConnected) queueMicrotask(() => trigger?.focus());
  }

  function choose(playlistId: string | null): void {
    onChange(playlistId);
    close(true);
  }

  function position(): void {
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    geometry = pickSelectPopoverGeometry({
      triggerRect: rect,
      boundaryRect: {
        top: 8,
        left: 8,
        right: window.innerWidth - 8,
        bottom: window.innerHeight - 8,
        width: window.innerWidth - 16,
        height: window.innerHeight - 16,
      },
      contentHeight: popover?.scrollHeight ?? 320,
      contentWidth: Math.max(rect.width, 288),
      horizontalAlign: "start",
    });
  }

  function popoverStyle(): string {
    if (!geometry) return "visibility:hidden;top:0;left:0";
    return `top:${geometry.top}px;left:${geometry.left}px;width:${geometry.width ?? geometry.minWidth}px;max-width:${geometry.maxWidth}px;max-height:${geometry.maxHeight}px`;
  }

  function handlePopoverFocusOut(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (!(next instanceof Node) || popover?.contains(next) || trigger?.contains(next)) return;
    close();
  }

  $effect(() => {
    if (!open) return;
    const pointer = (event: PointerEvent) => {
      if (!(event.target instanceof Node)) return;
      if (!trigger?.contains(event.target) && !popover?.contains(event.target)) close();
    };
    const keydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.stopPropagation();
      close(true);
    };
    window.addEventListener("pointerdown", pointer, true);
    window.addEventListener("keydown", keydown, true);
    window.addEventListener("resize", position);
    return () => {
      window.removeEventListener("pointerdown", pointer, true);
      window.removeEventListener("keydown", keydown, true);
      window.removeEventListener("resize", position);
    };
  });
</script>

<div class={cn("min-w-0", className)}>
  <button
    bind:this={trigger}
    type="button"
    {disabled}
    onclick={() => { void toggle(); }}
    aria-haspopup="dialog"
    aria-expanded={open}
    aria-label={label}
    class={cn(
      "flex h-8 w-full min-w-0 items-center gap-2 rounded-lg border bg-background px-2.5 text-left text-xs transition-colors hover:bg-accent disabled:cursor-not-allowed disabled:opacity-55",
      missing ? "border-warning/55 text-warning-foreground" : "border-border/75",
    )}
  >
    {#if missing}<AlertTriangle size={13} class="shrink-0 text-warning" />{:else}<ListMusic size={13} class="shrink-0 text-muted-foreground" />{/if}
    <span class="min-w-0 flex-1 truncate">{loading ? t("music.assignment.loadingPlaylists") : missing ? t("music.assignment.missingPlaylist") : selected?.name ?? t("music.assignment.noPlaylist")}</span>
    <ChevronDown size={12} class="shrink-0 text-muted-foreground" />
  </button>
</div>

{#if open}
  <div
    use:portal
    bind:this={popover}
    role="dialog"
    aria-label={label}
    onfocusout={handlePopoverFocusOut}
    style={popoverStyle()}
    class="fixed z-120 flex min-h-0 flex-col overflow-hidden rounded-xl border border-border/80 bg-popover text-popover-foreground shadow-2xl"
  >
    <label class="m-2 mb-1 flex h-8 shrink-0 items-center gap-2 rounded-lg bg-secondary/70 px-2.5">
      <Search size={13} class="text-muted-foreground" />
      <input bind:this={searchInput} bind:value={search} aria-label={t("music.assignment.searchPlaylists")} class="min-w-0 flex-1 bg-transparent text-xs outline-none" placeholder={t("music.assignment.searchPlaylists")} />
    </label>
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <button type="button" onclick={() => choose(null)} class="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left text-xs hover:bg-accent">
        <span class="grid h-6 w-6 shrink-0 place-items-center rounded-md bg-secondary">{#if value === null}<Check size={12} />{/if}</span>
        <span class="font-medium">{t("music.assignment.noPlaylist")}</span>
      </button>
      {#each matching as playlist (playlist.id)}
        <button type="button" onclick={() => choose(playlist.id)} class="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left hover:bg-accent">
          <span class={cn("grid h-6 w-6 shrink-0 place-items-center rounded-md bg-secondary", value === playlist.id && "bg-primary/15 text-primary")}>{#if value === playlist.id}<Check size={12} />{:else}<ListMusic size={12} />{/if}</span>
          <span class="min-w-0 flex-1"><strong class="block truncate text-xs font-medium">{systemMusicPlaylistName(playlist.id, playlist.name, t)}</strong><span class="block truncate text-[0.65rem] text-muted-foreground">{t("music.launcher.playlistCounts", playlist.eligibleCount, playlist.totalCount)}</span></span>
        </button>
      {:else}
        <p class="px-3 py-6 text-center text-xs text-muted-foreground">{t("music.assignment.noPlaylistMatches")}</p>
      {/each}
    </div>
  </div>
{/if}
