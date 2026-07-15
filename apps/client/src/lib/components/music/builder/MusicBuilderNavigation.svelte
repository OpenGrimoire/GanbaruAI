<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import BookOpenCheck from "@lucide/svelte/icons/book-open-check";
  import Library from "@lucide/svelte/icons/library";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import RadioTower from "@lucide/svelte/icons/radio-tower";
  import Search from "@lucide/svelte/icons/search";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import type { MusicBuilderDestination } from "$lib/music/music-builder-routing";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { cn } from "$lib/utils";

  let {
    destination,
    playlists,
    reviewCount,
    issueCount,
    compact = false,
    playingPlaylistId = null,
    onNavigate,
  }: {
    destination: MusicBuilderDestination;
    playlists: MusicPlaylistSummary[];
    reviewCount: number;
    issueCount: number;
    compact?: boolean;
    playingPlaylistId?: string | null;
    onNavigate: (destination: MusicBuilderDestination) => void;
  } = $props();

  const { t } = getLocalization();
  const iconSize = 15;
  let playlistSearch = $state("");
  const visiblePlaylists = $derived(playlists.filter((playlist) =>
    !playlistSearch.trim() || `${playlist.name} ${playlist.description}`.toLocaleLowerCase().includes(playlistSearch.trim().toLocaleLowerCase()),
  ));

  function active(kind: MusicBuilderDestination["kind"], playlistId?: string): boolean {
    return destination.kind === kind
      && (kind !== "playlist" || (destination.kind === "playlist" && destination.playlistId === playlistId));
  }

  function navigationKeyboard(node: HTMLElement): { destroy: () => void } {
    const items = (): HTMLButtonElement[] => [...node.querySelectorAll<HTMLButtonElement>("[data-builder-nav-item]")];
    const setCurrent = (current: HTMLButtonElement): void => {
      for (const item of items()) item.tabIndex = item === current ? 0 : -1;
    };
    const ensureCurrent = (): void => {
      const currentItems = items();
      if (!currentItems.some((item) => item.tabIndex === 0)) currentItems[0]?.setAttribute("tabindex", "0");
    };
    const navigateWithKeyboard = (event: KeyboardEvent): void => {
      if (event.target instanceof HTMLInputElement) return;
      if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
      const currentItems = items();
      if (currentItems.length === 0) return;
      event.preventDefault();
      const current = Math.max(0, currentItems.indexOf(document.activeElement as HTMLButtonElement));
      const index = event.key === "Home" ? 0
        : event.key === "End" ? currentItems.length - 1
        : event.key === "ArrowDown" ? Math.min(currentItems.length - 1, current + 1)
        : Math.max(0, current - 1);
      const next = currentItems[index];
      if (next) {
        setCurrent(next);
        next.focus();
      }
    };
    const handleFocus = (event: FocusEvent): void => {
      const target = event.target;
      if (target instanceof HTMLButtonElement && target.hasAttribute("data-builder-nav-item")) setCurrent(target);
    };
    const observer = new MutationObserver(ensureCurrent);
    observer.observe(node, { childList: true, subtree: true });
    queueMicrotask(ensureCurrent);
    node.addEventListener("keydown", navigateWithKeyboard);
    node.addEventListener("focusin", handleFocus);
    return { destroy: () => {
      observer.disconnect();
      node.removeEventListener("keydown", navigateWithKeyboard);
      node.removeEventListener("focusin", handleFocus);
    } };
  }
</script>

<nav
  class={cn(
    "builder-navigation min-h-0 bg-card/35",
    compact ? "w-[min(19rem,calc(100vw-1.5rem))] rounded-xl border border-border/70 p-2 shadow-xl" : "flex h-full flex-col border-r border-border/60",
  )}
  aria-label={t("music.builder.compactNavigation")}
  use:navigationKeyboard
>
  <div class="builder-nav-scroll min-h-0 flex-1 overflow-y-auto p-2">
    <div class="space-y-0.5">
      <button
        data-builder-nav-item
        tabindex={active("review") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "review" })}
        class={cn("builder-nav-item", active("review") && "builder-nav-item-active")}
        aria-current={active("review") ? "page" : undefined}
      >
        <BookOpenCheck size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.review")}</span>
        {#if reviewCount > 0}<span class="builder-nav-count">{reviewCount > 999 ? "999+" : reviewCount}</span>{/if}
      </button>
      <button
        data-builder-nav-item
        tabindex={active("playlists") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "playlists" })}
        class={cn("builder-nav-item", active("playlists") && "builder-nav-item-active")}
        aria-current={active("playlists") ? "page" : undefined}
      >
        <ListMusic size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.playlists")}</span>
      </button>
      <button
        data-builder-nav-item
        tabindex={active("library") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "library" })}
        class={cn("builder-nav-item", active("library") && "builder-nav-item-active")}
        aria-current={active("library") ? "page" : undefined}
      >
        <Library size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.library")}</span>
      </button>
      <button
        data-builder-nav-item
        tabindex={active("sources") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "sources" })}
        class={cn("builder-nav-item", active("sources") && "builder-nav-item-active")}
        aria-current={active("sources") ? "page" : undefined}
      >
        <RadioTower size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.sources")}</span>
        {#if issueCount > 0}<span class="builder-nav-count builder-nav-count-warning">{issueCount > 999 ? "999+" : issueCount}</span>{/if}
      </button>
      <button
        data-builder-nav-item
        tabindex={active("issues") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "issues" })}
        class={cn("builder-nav-item", active("issues") && "builder-nav-item-active")}
        aria-current={active("issues") ? "page" : undefined}
      >
        <AlertTriangle size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.issues")}</span>
      </button>
      <button
        data-builder-nav-item
        tabindex={active("soundscapes") ? 0 : -1}
        type="button"
        onclick={() => onNavigate({ kind: "soundscapes" })}
        class={cn("builder-nav-item", active("soundscapes") && "builder-nav-item-active")}
        aria-current={active("soundscapes") ? "page" : undefined}
      >
        <AudioLines size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.soundscapes")}</span>
      </button>
    </div>

    {#if playlists.length > 0}
      <div class="mx-2 my-2.5 h-px bg-border/60"></div>
      {#if playlists.length > 5}
        <label class="mx-1 mb-1.5 flex h-7 items-center gap-1.5 rounded-md bg-background/70 px-2"><Search size={12} class="text-muted-foreground" /><input bind:value={playlistSearch} aria-label={t("music.builder.searchPlaylists")} class="min-w-0 flex-1 bg-transparent text-[0.68rem] outline-none" placeholder={t("music.builder.searchPlaylists")} /></label>
      {/if}
      <div class="space-y-0.5">
        {#each visiblePlaylists as playlist (playlist.id)}
          <button
            data-builder-nav-item
            tabindex={active("playlist", playlist.id) ? 0 : -1}
            type="button"
            onclick={() => onNavigate({ kind: "playlist", playlistId: playlist.id })}
            class={cn("builder-nav-item", active("playlist", playlist.id) && "builder-nav-item-active")}
            aria-current={active("playlist", playlist.id) ? "page" : undefined}
            title={playlist.name}
          >
            <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-primary/70"></span>
            <span class="min-w-0 flex-1 truncate">{playlist.name}</span>
            {#if playingPlaylistId === playlist.id}<span class="shrink-0 text-[0.58rem] font-semibold text-foreground">{t("music.builder.nowPlayingCompact")}</span>{/if}
            {#if playlist.unavailableCount > 0}<span class="builder-nav-count builder-nav-count-warning" title={t("music.builder.unavailableCount", playlist.unavailableCount)}>{playlist.unavailableCount > 99 ? "99+" : playlist.unavailableCount}</span>{/if}
            <span class="builder-nav-count">{playlist.totalCount > 999 ? "999+" : playlist.totalCount}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</nav>

<style>
  .builder-nav-scroll {
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent;
  }

  .builder-nav-item {
    display: flex;
    width: 100%;
    min-width: 0;
    height: 2rem;
    align-items: center;
    gap: 0.55rem;
    border-radius: 0.55rem;
    padding-inline: 0.6rem;
    color: var(--muted-foreground);
    font-size: 0.75rem;
    text-align: left;
    transition: color 120ms ease, background-color 120ms ease;
  }

  .builder-nav-item:hover {
    color: var(--accent-foreground);
    background: color-mix(in srgb, var(--accent) 72%, transparent);
  }

  .builder-nav-item:focus-visible {
    outline: 2px solid color-mix(in srgb, var(--ring) 65%, transparent);
    outline-offset: -2px;
  }

  .builder-nav-item-active {
    color: var(--foreground);
    background: color-mix(in srgb, var(--primary) 12%, var(--card));
    font-weight: 600;
  }

  .builder-nav-count {
    min-width: 1.25rem;
    border-radius: 999px;
    padding: 0.1rem 0.35rem;
    background: color-mix(in srgb, var(--secondary) 82%, transparent);
    color: var(--muted-foreground);
    font-size: 0.62rem;
    line-height: 1rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .builder-nav-count-warning {
    background: color-mix(in srgb, var(--destructive) 14%, var(--secondary));
    color: var(--destructive);
  }

  @media (prefers-reduced-motion: reduce) {
    .builder-nav-item { transition: none; }
  }
</style>
