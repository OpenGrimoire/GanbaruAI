<script lang="ts">
  import { tick } from "svelte";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FileText from "@lucide/svelte/icons/file-text";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { buildNotesPageTree, type NotesPageParentStatus } from "$lib/notes/page-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesPagesForProject } from "$lib/notes/project-membership";
  import type { NotesPage } from "$lib/notes/types";
  import {
    projectPickerPanelEstimatedListHeight,
    projectPickerPanelHeight,
    projectPickerScrollState,
  } from "$lib/projects/project-picker-panels";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { cn } from "$lib/utils";
  import NotesPageIcon from "./NotesPageIcon.svelte";

  type MaybePromise<T> = T | Promise<T>;

  let {
    selectedPageId,
    projectId = null,
    panelMaxHeight = null,
    visibleRows = null,
    frameStyle = null,
    className = "",
    zIndexClass = "",
    showSearch = true,
    focusSearchRequestId = 0,
    rootElement = $bindable<HTMLDivElement | undefined>(),
    onPageSelected = undefined,
    onLayoutChange = undefined,
    onPointerLeave = undefined,
  }: {
    selectedPageId: string | null;
    projectId?: string | null;
    panelMaxHeight?: number | null;
    visibleRows?: number | null;
    frameStyle?: string | null;
    className?: string;
    zIndexClass?: string;
    showSearch?: boolean;
    focusSearchRequestId?: number;
    rootElement?: HTMLDivElement | undefined;
    onPageSelected?: () => MaybePromise<void>;
    onLayoutChange?: () => void;
    onPointerLeave?: (event: PointerEvent) => void;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();
  const iconStrokeWidth = 1.6;
  const iconSize = 13;
  const panelFallbackHeaderHeight = 40;
  const panelFallbackFooterHeight = 44;
  const panelListPadding = 6;
  const panelRowHeight = 32;

  let search = $state("");
  let pageDraft = $state("");
  let createPageOpen = $state(false);
  let searchInput = $state<HTMLInputElement | undefined>();
  let panelHeaderElement = $state<HTMLDivElement | undefined>();
  let panelFooterElement = $state<HTMLDivElement | undefined>();
  let panelStyle = $state("");
  let pageScrollElement = $state<HTMLElement | undefined>();
  let pageScrollContentElement = $state<HTMLElement | undefined>();
  let pageScrollable = $state(false);
  let pageCanScrollUp = $state(false);
  let pageCanScrollDown = $state(false);
  let pageScrollStateFrame: number | null = null;

  const normalizedSearch = $derived(search.trim());
  const projectPages = $derived.by(() => notesPagesForProject(notes.allPages, projectId));
  const treeItems = $derived.by(() =>
    buildNotesPageTree(projectPages, {
      activePageId: selectedPageId,
      expandedPageIds: notes.sidebarExpandedPageIds,
      pageIdsWithChildren: notes.sidebarPageIdsWithChildren,
      missingParentPageIds: notes.sidebarMissingParentPageIds,
      trashedParentPageIds: notes.sidebarTrashedParentPageIds,
      query: normalizedSearch,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
    })
  );
  const rootStyle = $derived(frameStyle ?? panelStyle);

  function cssPixelValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function scrollAreaVerticalPadding(element: HTMLElement | undefined): number {
    if (!element) return panelListPadding;
    const style = getComputedStyle(element);
    return cssPixelValue(style.paddingTop) + cssPixelValue(style.paddingBottom);
  }

  function estimatedListHeight(): number {
    return projectPickerPanelEstimatedListHeight({
      itemCount: treeItems.length,
      visibleRows,
      listPadding: panelListPadding,
      rowHeight: panelRowHeight,
    });
  }

  function updatePanelStyle(): void {
    if (frameStyle) {
      panelStyle = "";
      return;
    }
    const headerHeight = showSearch
      ? panelHeaderElement?.offsetHeight ?? panelFallbackHeaderHeight
      : 0;
    const footerHeight = panelFooterElement?.offsetHeight ?? panelFallbackFooterHeight;
    const measuredListHeight = pageScrollContentElement
      ? pageScrollContentElement.scrollHeight + scrollAreaVerticalPadding(pageScrollElement)
      : undefined;
    const listHeight = measuredListHeight ?? estimatedListHeight();
    const panelHeight = projectPickerPanelHeight({
      headerHeight,
      footerHeight,
      listHeight,
      maxHeight: panelMaxHeight,
      visibleRows,
      listPadding: panelListPadding,
      rowHeight: panelRowHeight,
    });
    panelStyle = [
      `height: ${panelHeight}px`,
      `max-height: ${panelHeight}px`,
    ].join("; ");
  }

  function refreshPageScrollState(): void {
    pageScrollStateFrame = null;
    const element = pageScrollElement;
    if (!element) {
      pageScrollable = false;
      pageCanScrollUp = false;
      pageCanScrollDown = false;
      return;
    }
    const state = projectPickerScrollState(element);
    pageScrollable = state.scrollable;
    pageCanScrollUp = state.canScrollUp;
    pageCanScrollDown = state.canScrollDown;
  }

  function requestPageScrollStateRefresh(): void {
    if (pageScrollStateFrame !== null) cancelAnimationFrame(pageScrollStateFrame);
    pageScrollStateFrame = requestAnimationFrame(refreshPageScrollState);
  }

  function handlePageScroll(): void {
    refreshPageScrollState();
  }

  function rowTitle(page: NotesPage): string {
    return notesPageTitle(page, t("notes.untitled"));
  }

  function parentStatusLabel(status: NotesPageParentStatus): string {
    return status === "trashed" ? t("notes.parentInTrash") : t("notes.parentMissing");
  }

  function rowPaddingStyle(depth: number): string {
    const depthOffset = Math.min(Math.max(depth, 0), 8) * 0.75;
    return `padding-left: ${0.5 + depthOffset}rem`;
  }

  async function selectPage(pageId: string): Promise<void> {
    await notes.selectPage(pageId);
    await onPageSelected?.();
  }

  async function submitPage(): Promise<void> {
    const title = pageDraft.trim();
    await notes.createPage(title, { projectId });
    pageDraft = "";
    createPageOpen = false;
    await onPageSelected?.();
  }

  $effect(() => {
    const requestId = focusSearchRequestId;
    void requestId;
    if (!showSearch || requestId === 0) return;
    void tick().then(() => {
      searchInput?.focus();
    });
  });

  $effect(() => {
    const maxHeight = panelMaxHeight;
    const rowCount = treeItems.length;
    const creatingPage = createPageOpen;
    const searchVisible = showSearch;
    void maxHeight;
    void rowCount;
    void creatingPage;
    void searchVisible;
    requestAnimationFrame(() => {
      updatePanelStyle();
      requestPageScrollStateRefresh();
      onLayoutChange?.();
    });
  });

  $effect(() => {
    const scrollElement = pageScrollElement;
    if (!scrollElement) return;
    const resizeObserver = new ResizeObserver(() => {
      updatePanelStyle();
      requestPageScrollStateRefresh();
      onLayoutChange?.();
    });
    resizeObserver.observe(scrollElement);
    if (panelHeaderElement) resizeObserver.observe(panelHeaderElement);
    if (panelFooterElement) resizeObserver.observe(panelFooterElement);
    if (pageScrollContentElement) resizeObserver.observe(pageScrollContentElement);
    updatePanelStyle();
    requestPageScrollStateRefresh();
    return () => {
      resizeObserver.disconnect();
      if (pageScrollStateFrame !== null) {
        cancelAnimationFrame(pageScrollStateFrame);
        pageScrollStateFrame = null;
      }
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={rootElement}
  class={cn(
    "project-picker-panel flex min-h-0 w-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60",
    zIndexClass,
    className,
  )}
  style={rootStyle}
  onpointerleave={onPointerLeave}
>
  {#if showSearch}
    <div bind:this={panelHeaderElement} class="px-1.5 pb-0.5 pt-1.5">
      <div class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 pl-2 pr-1">
        <Search size={13} strokeWidth={iconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
        <input
          bind:this={searchInput}
          bind:value={search}
          placeholder={t("notes.searchPlaceholder")}
          aria-label={t("notes.searchLabel")}
          class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
        />
      </div>
    </div>
  {/if}

  <div class="relative min-h-0 flex-1">
    <div
      bind:this={pageScrollElement}
      class={cn(
        "project-picker-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto",
        showSearch ? "pb-1 pt-0.5" : "py-1",
        pageScrollable ? "px-1 pr-2" : "px-1",
        pageScrollable && pageCanScrollUp && pageCanScrollDown && "project-picker-scroll-both",
        pageScrollable && pageCanScrollUp && !pageCanScrollDown && "project-picker-scroll-top",
        pageScrollable && !pageCanScrollUp && pageCanScrollDown && "project-picker-scroll-bottom",
      )}
      onscroll={handlePageScroll}
    >
      <div bind:this={pageScrollContentElement}>
        {#if notes.loading && projectPages.length === 0}
          <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
            {t("notes.loading")}
          </div>
        {:else if notes.loadError}
          <div class="px-3 py-2 text-[0.8rem] text-destructive">
            {t("notes.loadFailed", notes.loadError)}
          </div>
        {:else if treeItems.length === 0}
          <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
            {normalizedSearch ? t("notes.noSearchResults") : t("notes.noPages")}
          </div>
        {:else}
          <div class="grid">
            {#each treeItems as item (item.page.id)}
              {@const title = rowTitle(item.page)}
              <button
                type="button"
                class={cn(
                  "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] transition-colors hover:bg-accent hover:text-accent-foreground",
                  item.page.id === selectedPageId ? "bg-accent text-accent-foreground" : "text-popover-foreground",
                )}
                style={rowPaddingStyle(item.depth)}
                aria-label={title}
                onclick={() => { void selectPage(item.page.id); }}
              >
                {#if item.page.icon}
                  <NotesPageIcon icon={item.page.icon} size={iconSize} class="shrink-0 text-popover-foreground/70" />
                {:else}
                  <FileText size={iconSize} strokeWidth={iconStrokeWidth} class="shrink-0 text-popover-foreground/70" />
                {/if}
                <span class="min-w-0 flex-1 truncate">{title}</span>
                {#if item.parentStatus}
                  <TriangleAlert
                    size={13}
                    strokeWidth={iconStrokeWidth}
                    class="shrink-0 text-destructive"
                    aria-label={parentStatusLabel(item.parentStatus)}
                    data-app-tooltip={parentStatusLabel(item.parentStatus)}
                  />
                {:else if item.hasChildren}
                  <ChevronRight size={13} strokeWidth={iconStrokeWidth} class="shrink-0 text-popover-foreground/45" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
    <CalendarScrollbar
      scrollContainer={pageScrollElement}
      stickyTop={4}
      stickyBottom={4}
      wheelPassthrough
    />
  </div>

  <div bind:this={panelFooterElement} class="relative z-10 shrink-0 bg-popover p-1.5">
    <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
    {#if createPageOpen}
      <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitPage(); }}>
        <input
          bind:value={pageDraft}
          placeholder={t("notes.titlePlaceholder")}
          class="min-h-7 min-w-0 flex-1 rounded border border-border bg-muted/40 px-2 text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
        />
        <button type="submit" class="min-h-7 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
          {t("common.save")}
        </button>
      </form>
    {:else}
      <button
        type="button"
        class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        onclick={() => {
          createPageOpen = true;
          void tick().then(() => {
            onLayoutChange?.();
          });
        }}
      >
        <Plus size={13} strokeWidth={iconStrokeWidth} />
        <span>{t("notes.newPage")}</span>
      </button>
    {/if}
  </div>
</div>

<style>
  .project-picker-panel {
    --cal-scrollbar-thumb: color-mix(in srgb, var(--popover-foreground) 18%, var(--popover));
    --cal-scrollbar-thumb-hover: color-mix(in srgb, var(--popover-foreground) 36%, var(--popover));
  }

  .project-picker-scroll-area {
    --project-picker-scroll-fade: 28px;

    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-picker-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--project-picker-scroll-fade), black);
    mask-image: linear-gradient(to bottom, transparent, black var(--project-picker-scroll-fade), black);
  }

  .project-picker-scroll-bottom {
    -webkit-mask-image: linear-gradient(to top, transparent, black var(--project-picker-scroll-fade), black);
    mask-image: linear-gradient(to top, transparent, black var(--project-picker-scroll-fade), black);
  }

  .project-picker-scroll-both {
    -webkit-mask-image:
      linear-gradient(
        to bottom,
        transparent,
        black var(--project-picker-scroll-fade),
        black calc(100% - var(--project-picker-scroll-fade)),
        transparent
      );
    mask-image:
      linear-gradient(
        to bottom,
        transparent,
        black var(--project-picker-scroll-fade),
        black calc(100% - var(--project-picker-scroll-fade)),
        transparent
      );
  }
</style>
