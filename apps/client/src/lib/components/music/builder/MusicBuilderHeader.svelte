<script lang="ts">
  import { onDestroy, tick, untrack } from "svelte";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    MUSIC_BUILDER_PRIMARY_DESTINATIONS,
    musicBuilderDestinationShortcut,
    type MusicBuilderDestination,
    type MusicBuilderPrimaryDestinationKind,
  } from "$lib/music/music-builder-routing";

  let {
    destination,
    search,
    searchAvailable,
    busy,
    reviewCount,
    issueCount,
    primaryLabel = null,
    canUndo = false,
    onOpenPlayer,
    onNavigate,
    onSearch,
    onRefresh,
    onPrimary = () => undefined,
    onUndo = () => undefined,
  }: {
    destination: MusicBuilderDestination;
    search: string;
    searchAvailable: boolean;
    busy: boolean;
    reviewCount: number;
    issueCount: number;
    primaryLabel?: string | null;
    canUndo?: boolean;
    onOpenPlayer: () => void;
    onNavigate: (destination: MusicBuilderDestination) => void;
    onSearch: (search: string) => void;
    onRefresh: () => void;
    onPrimary?: () => void;
    onUndo?: () => void;
  } = $props();

  const { t } = getLocalization();
  let draft = $state("");
  let searchOpen = $state(false);
  let searchInput = $state<HTMLInputElement | null>(null);
  let debounceId: number | null = null;

  $effect(() => {
    const nextSearch = search;
    if (nextSearch !== untrack(() => draft)) draft = nextSearch;
  });

  $effect(() => {
    if (!searchAvailable) searchOpen = false;
    else if (search.trim()) searchOpen = true;
  });

  onDestroy(() => {
    if (debounceId !== null) window.clearTimeout(debounceId);
  });

  function destinationLabel(kind: MusicBuilderPrimaryDestinationKind): string {
    if (kind === "review") return t("music.builder.review");
    if (kind === "playlists") return t("music.builder.playlists");
    if (kind === "library") return t("music.builder.library");
    if (kind === "sources") return t("music.builder.sources");
    if (kind === "issues") return t("music.builder.issues");
    return t("music.builder.soundscapes");
  }

  function active(kind: MusicBuilderPrimaryDestinationKind): boolean {
    return destination.kind === kind || (kind === "playlists" && destination.kind === "playlist");
  }

  function count(kind: MusicBuilderPrimaryDestinationKind): number | null {
    if (kind === "review") return reviewCount > 0 ? reviewCount : null;
    if (kind === "issues") return issueCount > 0 ? issueCount : null;
    return null;
  }

  function updateSearch(value: string): void {
    draft = value;
    if (debounceId !== null) window.clearTimeout(debounceId);
    debounceId = window.setTimeout(() => {
      debounceId = null;
      onSearch(draft);
    }, 180);
  }

  function closeSearch(): void {
    if (debounceId !== null) window.clearTimeout(debounceId);
    debounceId = null;
    searchOpen = false;
    draft = "";
    onSearch("");
  }

  async function openSearch(): Promise<void> {
    searchOpen = true;
    await tick();
    searchInput?.focus();
  }

  function navigationKeyboard(node: HTMLElement): { destroy: () => void } {
    const items = (): HTMLButtonElement[] => [...node.querySelectorAll<HTMLButtonElement>("[data-builder-nav-item]")];
    const setCurrent = (current: HTMLButtonElement): void => {
      for (const item of items()) item.tabIndex = item === current ? 0 : -1;
    };
    const handleKeydown = (event: KeyboardEvent): void => {
      if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
      const currentItems = items();
      if (currentItems.length === 0) return;
      event.preventDefault();
      const current = Math.max(0, currentItems.indexOf(document.activeElement as HTMLButtonElement));
      const index = event.key === "Home" ? 0
        : event.key === "End" ? currentItems.length - 1
        : event.key === "ArrowRight" ? (current + 1) % currentItems.length
        : (current - 1 + currentItems.length) % currentItems.length;
      const next = currentItems[index];
      if (next) {
        setCurrent(next);
        next.focus();
      }
    };
    const handleFocus = (event: FocusEvent): void => {
      if (event.target instanceof HTMLButtonElement && event.target.hasAttribute("data-builder-nav-item")) setCurrent(event.target);
    };
    node.addEventListener("keydown", handleKeydown);
    node.addEventListener("focusin", handleFocus);
    return { destroy: () => {
      node.removeEventListener("keydown", handleKeydown);
      node.removeEventListener("focusin", handleFocus);
    } };
  }
</script>

<header class="builder-header shrink-0 border-b border-border/60 bg-background/75 backdrop-blur-sm">
  <div class="flex min-h-11 min-w-0 items-center gap-1.5 px-2.5">
    <button
      type="button"
      onclick={onOpenPlayer}
      class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-secondary px-2.5 text-xs font-medium text-secondary-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
      aria-label={t("music.backToPlayer")}
      data-music-focus-key="builder:back-to-player"
    >
      <ArrowLeft size={15} strokeWidth={1.6} />
      <span class="builder-player-label">{t("music.mediaPlayer")}</span>
    </button>

    <div class="builder-destinations flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto" role="group" aria-label={t("music.builder.compactNavigation")} use:navigationKeyboard>
      {#each MUSIC_BUILDER_PRIMARY_DESTINATIONS as kind}
        {@const shortcut = musicBuilderDestinationShortcut(kind)}
        {@const badge = count(kind)}
        <button
          type="button"
          data-builder-nav-item
          aria-pressed={active(kind)}
          tabindex={active(kind) ? 0 : -1}
          onclick={() => onNavigate({ kind })}
          class:builder-destination-active={active(kind)}
          class="builder-destination inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
          title={`${destinationLabel(kind)} (${t("calendar.toolbar.shortcutKey", shortcut ?? "")})`}
        >
          <span>{destinationLabel(kind)}</span>
          {#if badge !== null}<span class:builder-warning-count={kind === "issues"} class="builder-destination-count rounded-full bg-secondary px-1.5 py-0.5 text-[0.58rem] tabular-nums">{badge > 999 ? "999+" : badge}</span>{/if}
        </button>
      {/each}
    </div>

    <div class="ml-auto flex shrink-0 items-center gap-1">
      {#if searchAvailable && searchOpen}
        <div class="relative w-28 sm:w-44">
          <Search class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" size={14} strokeWidth={1.6} />
          <input
            bind:this={searchInput}
            type="search"
            value={draft}
            oninput={(event) => updateSearch(event.currentTarget.value)}
            onkeydown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                event.stopPropagation();
                closeSearch();
              }
            }}
            class="h-8 w-full rounded-lg border border-border/70 bg-card/80 pl-8 pr-7 text-xs text-foreground outline-none placeholder:text-muted-foreground focus:border-ring/60 focus:ring-2 focus:ring-ring/25"
            placeholder={t("music.builder.search")}
            aria-label={t("music.builder.search")}
          />
          <button type="button" onclick={closeSearch} class="absolute right-1 top-1/2 inline-flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground" aria-label={t("music.builder.clearSearch")}><X size={13} strokeWidth={1.7} /></button>
        </div>
      {:else if searchAvailable}
        <button type="button" data-builder-search-trigger onclick={() => { void openSearch(); }} class="inline-flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-accent-foreground" aria-label={t("music.builder.search")} title={t("music.builder.search")}><Search size={16} strokeWidth={1.6} /></button>
      {/if}

      {#if canUndo}
        <button type="button" onclick={onUndo} class="builder-undo inline-flex h-8 shrink-0 items-center rounded-lg px-2.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground">{t("music.builder.undo")}</button>
      {/if}

      <button type="button" onclick={onRefresh} disabled={busy} class="relative inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground disabled:opacity-45" aria-label={busy ? t("music.builder.refreshing") : t("music.builder.refresh")}>
        <RefreshCw class={busy ? "animate-spin motion-reduce:animate-none" : ""} size={15} strokeWidth={1.6} />
      </button>

      {#if primaryLabel}
        <button type="button" onclick={onPrimary} class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-primary px-2.5 text-xs font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-primary/90" aria-label={primaryLabel} title={primaryLabel}>
          <Plus size={14} strokeWidth={1.8} />
          <span class="builder-primary-label">{primaryLabel}</span>
        </button>
      {/if}
    </div>
  </div>
</header>

<style>
  .builder-header { container-type: inline-size; }
  .builder-destinations { scrollbar-width: none; }
  .builder-destinations::-webkit-scrollbar { display: none; }
  .builder-destination-active { background: color-mix(in srgb, var(--accent) 78%, transparent); color: var(--foreground); }
  .builder-warning-count { color: var(--destructive); }
  @container (width < 760px) { .builder-primary-label, .builder-undo { display: none; } }
  @container (width < 460px) { .builder-player-label { display: none; } }
  @media (prefers-reduced-motion: reduce) { .builder-destination { transition: none; } }
</style>
