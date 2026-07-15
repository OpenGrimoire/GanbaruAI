<script lang="ts">
  import AlertTriangle from "@lucide/svelte/icons/triangle-alert";
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import BookOpenCheck from "@lucide/svelte/icons/book-open-check";
  import Library from "@lucide/svelte/icons/library";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import RadioTower from "@lucide/svelte/icons/radio-tower";
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
    onNavigate,
  }: {
    destination: MusicBuilderDestination;
    playlists: MusicPlaylistSummary[];
    reviewCount: number;
    issueCount: number;
    compact?: boolean;
    onNavigate: (destination: MusicBuilderDestination) => void;
  } = $props();

  const { t } = getLocalization();
  const iconSize = 15;

  function active(kind: MusicBuilderDestination["kind"], playlistId?: string): boolean {
    return destination.kind === kind
      && (kind !== "playlist" || (destination.kind === "playlist" && destination.playlistId === playlistId));
  }
</script>

<nav
  class={cn(
    "builder-navigation min-h-0 bg-card/35",
    compact ? "w-[min(19rem,calc(100vw-1.5rem))] rounded-xl border border-border/70 p-2 shadow-xl" : "flex h-full flex-col border-r border-border/60",
  )}
  aria-label={t("music.builder.compactNavigation")}
>
  <div class="builder-nav-scroll min-h-0 flex-1 overflow-y-auto p-2">
    <div class="space-y-0.5">
      <button
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
        type="button"
        onclick={() => onNavigate({ kind: "playlists" })}
        class={cn("builder-nav-item", active("playlists") && "builder-nav-item-active")}
        aria-current={active("playlists") ? "page" : undefined}
      >
        <ListMusic size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.playlists")}</span>
      </button>
      <button
        type="button"
        onclick={() => onNavigate({ kind: "library" })}
        class={cn("builder-nav-item", active("library") && "builder-nav-item-active")}
        aria-current={active("library") ? "page" : undefined}
      >
        <Library size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.library")}</span>
      </button>
      <button
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
        type="button"
        onclick={() => onNavigate({ kind: "issues" })}
        class={cn("builder-nav-item", active("issues") && "builder-nav-item-active")}
        aria-current={active("issues") ? "page" : undefined}
      >
        <AlertTriangle size={iconSize} strokeWidth={1.6} />
        <span class="min-w-0 flex-1 truncate">{t("music.builder.issues")}</span>
      </button>
      <button
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
      <div class="space-y-0.5">
        {#each playlists as playlist (playlist.id)}
          <button
            type="button"
            onclick={() => onNavigate({ kind: "playlist", playlistId: playlist.id })}
            class={cn("builder-nav-item", active("playlist", playlist.id) && "builder-nav-item-active")}
            aria-current={active("playlist", playlist.id) ? "page" : undefined}
            title={playlist.name}
          >
            <span class="h-1.5 w-1.5 shrink-0 rounded-full bg-primary/70"></span>
            <span class="min-w-0 flex-1 truncate">{playlist.name}</span>
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
