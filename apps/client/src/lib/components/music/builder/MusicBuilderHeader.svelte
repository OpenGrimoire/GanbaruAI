<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Menu from "@lucide/svelte/icons/menu";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    title,
    search,
    busy,
    resultCount,
    compact = false,
    primaryLabel = null,
    canUndo = false,
    onOpenPlayer,
    onSearch,
    onRefresh,
    onPrimary = () => undefined,
    onUndo = () => undefined,
    onToggleNavigation = () => undefined,
  }: {
    title: string;
    search: string;
    busy: boolean;
    resultCount: number | null;
    compact?: boolean;
    primaryLabel?: string | null;
    canUndo?: boolean;
    onOpenPlayer: () => void;
    onSearch: (search: string) => void;
    onRefresh: () => void;
    onPrimary?: () => void;
    onUndo?: () => void;
    onToggleNavigation?: () => void;
  } = $props();

  const { t } = getLocalization();
  let draft = $state("");
  let compactSearchOpen = $state(false);
  let searchInput = $state<HTMLInputElement | null>(null);
  let debounceId: number | null = null;

  $effect(() => {
    const nextSearch = search;
    if (nextSearch !== untrack(() => draft)) draft = nextSearch;
  });

  onDestroy(() => {
    if (debounceId !== null) window.clearTimeout(debounceId);
  });

  function updateSearch(value: string): void {
    draft = value;
    if (debounceId !== null) window.clearTimeout(debounceId);
    debounceId = window.setTimeout(() => {
      debounceId = null;
      onSearch(draft);
    }, 180);
  }

  function clearSearch(): void {
    if (debounceId !== null) window.clearTimeout(debounceId);
    debounceId = null;
    draft = "";
    onSearch("");
    searchInput?.focus();
  }

  async function openCompactSearch(): Promise<void> {
    compactSearchOpen = true;
    await Promise.resolve();
    searchInput?.focus();
  }
</script>

<header class="builder-header shrink-0 border-b border-border/60 bg-background/75 backdrop-blur-sm">
  <div class="flex min-h-11 items-center gap-2 px-2.5">
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

    {#if compact}
      <button
        type="button"
        onclick={onToggleNavigation}
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
        aria-label={t("music.builder.compactNavigation")}
      >
        <Menu size={17} strokeWidth={1.6} />
      </button>
    {/if}

    <div class="min-w-0 flex-1">
      <div class="flex min-w-0 items-center gap-2">
        <h1 class="truncate text-sm font-semibold tracking-[-0.01em] text-foreground">{title}</h1>
        {#if resultCount !== null}
          <span class="shrink-0 rounded-full bg-secondary/80 px-1.5 py-0.5 text-[0.65rem] tabular-nums text-muted-foreground">
            {resultCount}
          </span>
        {/if}
        {#if busy}
          <span class="builder-busy-dot h-1.5 w-1.5 shrink-0 rounded-full bg-primary" aria-label={t("music.builder.refreshing")}></span>
        {/if}
      </div>
    </div>

    {#if !compact || compactSearchOpen}
      <label class="relative min-w-28 flex-[0_1_18rem]">
        <span class="sr-only">{t("music.builder.search")}</span>
        <Search class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" size={14} strokeWidth={1.6} />
        <input
          bind:this={searchInput}
          value={draft}
          oninput={(event) => updateSearch(event.currentTarget.value)}
          onkeydown={(event) => {
            if (event.key === "Escape" && compact) {
              event.stopPropagation();
              compactSearchOpen = false;
            }
          }}
          class="h-8 w-full rounded-lg border border-border/70 bg-card/80 pl-8 pr-8 text-xs text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:border-ring/60 focus:ring-2 focus:ring-ring/25"
          placeholder={t("music.builder.search")}
        />
        {#if draft}
          <button
            type="button"
            onclick={clearSearch}
            class="absolute right-1 top-1/2 inline-flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            aria-label={t("music.builder.clearSearch")}
          >
            <X size={13} strokeWidth={1.7} />
          </button>
        {/if}
      </label>
    {:else}
      <button
        type="button"
        onclick={() => { void openCompactSearch(); }}
        class="inline-flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        aria-label={t("music.builder.search")}
      >
        <Search size={16} strokeWidth={1.6} />
      </button>
    {/if}

    {#if canUndo && !compact}
      <button
        type="button"
        onclick={onUndo}
        class="inline-flex h-8 shrink-0 items-center rounded-lg px-2.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground"
      >
        {t("music.builder.undo")}
      </button>
    {/if}

    <button
      type="button"
      onclick={onRefresh}
      disabled={busy}
      class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground disabled:opacity-45"
      aria-label={busy ? t("music.builder.refreshing") : t("music.builder.refresh")}
    >
      <RefreshCw class={busy ? "animate-spin motion-reduce:animate-none" : ""} size={15} strokeWidth={1.6} />
    </button>

    {#if primaryLabel && !compact}
      <button
        type="button"
        onclick={onPrimary}
        class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-lg bg-primary px-3 text-xs font-semibold text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
      >
        <Plus size={14} strokeWidth={1.8} />
        {primaryLabel}
      </button>
    {/if}
  </div>
</header>

<style>
  .builder-header {
    container-type: inline-size;
  }

  .builder-busy-dot {
    animation: builder-pulse 1.4s ease-in-out infinite;
  }

  @container (width < 400px) {
    .builder-player-label {
      display: none;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .builder-busy-dot {
      animation: none;
    }
  }

  @keyframes builder-pulse {
    0%, 100% { opacity: 0.35; transform: scale(0.86); }
    50% { opacity: 1; transform: scale(1); }
  }
</style>
