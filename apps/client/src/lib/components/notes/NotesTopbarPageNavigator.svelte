<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FileText from "@lucide/svelte/icons/file-text";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesPageParentStatus, NotesPageTreeItem } from "$lib/notes/page-tree";
  import { planNotesSidebarNavigation } from "$lib/notes/page-sidebar";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesSearchNavigationPlan } from "$lib/notes/search";
  import type { NotesPage, NotesSearchResult } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";

  let {
    selectedPageId,
    panelMaxHeight,
    onPageSelected,
    onCreatePage,
  }: {
    selectedPageId: string | null;
    panelMaxHeight: number | null;
    onPageSelected: () => void;
    onCreatePage: () => void;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();
  let search = $state("");

  const sidebarPlan = $derived.by(() =>
    planNotesSidebarNavigation({
      pages: notes.pages,
      favoritePageIds: notes.favoritePageIds,
      recentPageIds: notes.recentPageIds,
      expandedPageIds: notes.sidebarExpandedPageIds,
      pageIdsWithChildren: notes.sidebarPageIdsWithChildren,
      missingParentPageIds: notes.sidebarMissingParentPageIds,
      trashedParentPageIds: notes.sidebarTrashedParentPageIds,
      activePageId: selectedPageId,
      search,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
      recentLimit: 6,
    })
  );
  const searchQuery = $derived(sidebarPlan.searchQuery);

  $effect(() => {
    const query = searchQuery;
    const includeResolvedComments = notes.searchIncludeResolvedComments;
    const timer = window.setTimeout(() => {
      void notes.search(query, 20, includeResolvedComments);
    }, query ? 180 : 0);
    return () => {
      window.clearTimeout(timer);
    };
  });

  function rowTitle(page: NotesPage): string {
    return notesPageTitle(page, t("notes.untitled"));
  }

  function parentStatusLabel(status: NotesPageParentStatus): string {
    return status === "trashed" ? t("notes.parentInTrash") : t("notes.parentMissing");
  }

  function searchResultTypeLabel(result: NotesSearchResult): string {
    if (result.type === "page") return t("notes.searchResultPage");
    if (result.type === "block") return t("notes.searchResultBlock");
    return t("notes.searchResultComment");
  }

  function commentSearchTargetLabel(result: NotesSearchResult): string {
    if (result.comment_anchor) return t("notes.searchResultInlineComment");
    if (result.block_id) return t("notes.searchResultBlockComment");
    return t("notes.searchResultPageDiscussion");
  }

  function searchResultDetail(result: NotesSearchResult): string {
    if (result.type !== "comment") return searchResultTypeLabel(result);
    const target = commentSearchTargetLabel(result);
    const status = result.comment_status === "resolved"
      ? t("notes.searchResultResolved")
      : t("notes.searchResultOpen");
    if (result.comment_author) {
      return t("notes.searchResultCommentBy", target, result.comment_author.resolved_name, status);
    }
    return t("notes.searchResultCommentStatus", target, status);
  }

  function openPage(pageId: string): void {
    void notes.selectPage(pageId).then(onPageSelected);
  }

  function openSearchResult(result: NotesSearchResult): void {
    const plan = notesSearchNavigationPlan(result);
    if (plan.blockId) {
      void notes.openNotesLink({ pageId: plan.pageId, blockId: plan.blockId }).then((opened) => {
        if (opened && plan.commentParent) notes.setActiveCommentParent(plan.commentParent);
        if (opened) onPageSelected();
      });
      return;
    }
    void notes.selectPage(plan.pageId).then(() => {
      if (plan.commentParent) notes.setActiveCommentParent(plan.commentParent);
      onPageSelected();
    });
  }

  function toggleItem(item: NotesPageTreeItem): void {
    notes.setSidebarPageCollapsed(item.page.id, !item.collapsed);
  }
</script>

<div
  class="overflow-auto rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
  style={panelMaxHeight ? `max-height: ${panelMaxHeight}px;` : undefined}
>
  <div class="flex items-center gap-2 px-1 pb-2">
    <label class="flex min-w-0 flex-1 items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5">
      <Search class="size-4 shrink-0 text-muted-foreground" />
      <input
        class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground"
        bind:value={search}
        placeholder={t("notes.searchPlaceholder")}
        aria-label={t("notes.searchLabel")}
      />
    </label>
    <button
      type="button"
      class="flex size-8 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground hover:bg-primary/90"
      aria-label={t("notes.newPage")}
      title={t("notes.newPage")}
      onclick={onCreatePage}
    >
      <Plus class="size-4" />
    </button>
  </div>

  {#if notes.loading && notes.pages.length === 0}
    <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.loading")}</div>
  {:else if notes.loadError}
    <div class="px-2 py-2 text-[0.8rem] text-destructive">
      {t("notes.loadFailed", notes.loadError)}
    </div>
  {:else if searchQuery}
    <div class="flex flex-col gap-1">
      <div class="flex flex-wrap items-center justify-between gap-2 px-2 pb-1 pt-1">
        <div class="text-[0.7rem] font-medium text-muted-foreground">
          {notes.searchLoading ? t("notes.searching") : t("notes.searchResults")}
        </div>
        <label class="flex items-center gap-1.5 text-[0.7rem] text-muted-foreground">
          <input
            class="size-3 accent-primary"
            type="checkbox"
            checked={notes.searchIncludeResolvedComments}
            onchange={(event) => {
              notes.setSearchIncludeResolvedComments(event.currentTarget.checked);
            }}
          />
          <span>{t("notes.searchIncludeResolvedComments")}</span>
        </label>
      </div>
      {#if notes.searchError}
        <div class="px-2 py-2 text-[0.8rem] text-destructive">
          {t("notes.searchFailed", notes.searchError)}
        </div>
      {:else if notes.searchLoading && notes.searchResults.length === 0}
        <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.searching")}</div>
      {:else if notes.searchResults.length === 0}
        <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.noSearchResults")}</div>
      {:else}
        {#each notes.searchResults as result (result.id)}
          {@const title = rowTitle(result.page)}
          <button
            type="button"
            class={`flex w-full min-w-0 items-start gap-2 rounded-md px-2 py-1.5 text-left text-[0.8rem] hover:bg-accent/70 ${
              result.page.id === selectedPageId ? "bg-accent text-accent-foreground" : "text-foreground"
            }`}
            aria-label={t("notes.openSearchResult", title)}
            onclick={() => {
              openSearchResult(result);
            }}
          >
            <span class="mt-0.5 flex size-4 shrink-0 items-center justify-center text-muted-foreground">
              {#if result.type === "comment"}
                <MessageSquare class="size-3.5" />
              {:else}
                <FileText class="size-3.5" />
              {/if}
            </span>
            <span class="min-w-0 flex-1">
              <span class="block truncate font-medium">{title}</span>
              <span class="block truncate text-[0.733333rem] text-muted-foreground">
                {searchResultDetail(result)}
              </span>
              {#if result.snippet}
                <span class="mt-0.5 line-clamp-2 block text-[0.733333rem] leading-snug text-muted-foreground">
                  {result.snippet}
                </span>
              {/if}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  {:else if sidebarPlan.treeItems.length === 0}
    <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.noPages")}</div>
  {:else}
    <div class="flex flex-col gap-1">
      {#if sidebarPlan.showNavigationSections && sidebarPlan.favoritePages.length > 0}
        <div class="px-2 pb-1 pt-1 text-[0.7rem] font-medium text-muted-foreground">
          {t("notes.favorites")}
        </div>
        {#each sidebarPlan.favoritePages as page (page.id)}
          {@const title = rowTitle(page)}
          <button
            type="button"
            class={`flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-[0.8rem] hover:bg-accent/70 ${
              page.id === selectedPageId ? "bg-accent text-accent-foreground" : "text-foreground"
            }`}
            onclick={() => {
              openPage(page.id);
            }}
          >
            {#if page.icon}
              <NotesPageIcon icon={page.icon} size={14} class="shrink-0 text-muted-foreground" />
            {:else}
              <FileText class="size-3.5 shrink-0 text-muted-foreground" />
            {/if}
            <span class="min-w-0 flex-1 truncate">{title}</span>
            <Star class="size-3.5 shrink-0 fill-current text-primary" aria-hidden="true" />
          </button>
        {/each}
      {/if}

      {#if sidebarPlan.showNavigationSections && sidebarPlan.recentPages.length > 0}
        <div class="px-2 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
          {t("notes.recents")}
        </div>
        {#each sidebarPlan.recentPages as page (page.id)}
          {@const title = rowTitle(page)}
          <button
            type="button"
            class={`flex min-w-0 items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-[0.8rem] hover:bg-accent/70 ${
              page.id === selectedPageId ? "bg-accent text-accent-foreground" : "text-foreground"
            }`}
            onclick={() => {
              openPage(page.id);
            }}
          >
            {#if page.icon}
              <NotesPageIcon icon={page.icon} size={14} class="shrink-0 text-muted-foreground" />
            {:else}
              <FileText class="size-3.5 shrink-0 text-muted-foreground" />
            {/if}
            <span class="min-w-0 flex-1 truncate">{title}</span>
          </button>
        {/each}
      {/if}

      {#if sidebarPlan.showPagesHeading}
        <div class="px-2 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
          {t("notes.pages")}
        </div>
      {/if}
      {#each sidebarPlan.treeItems as item (item.page.id)}
        {@const title = rowTitle(item.page)}
        <div
          class="flex min-w-0 items-center rounded-md hover:bg-accent/70"
          style={`padding-left: ${Math.min(item.depth, 8) * 0.75}rem;`}
        >
          <button
            type="button"
            class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground disabled:pointer-events-none disabled:opacity-0"
            aria-label={item.collapsed ? t("notes.expandPage") : t("notes.collapsePage")}
            disabled={!item.hasChildren}
            onclick={() => {
              toggleItem(item);
            }}
          >
            {#if item.hasChildren && item.collapsed}
              <ChevronRight class="size-4" />
            {:else if item.hasChildren}
              <ChevronDown class="size-4" />
            {/if}
          </button>
          <button
            type="button"
            class={`flex min-w-0 flex-1 items-center gap-1.5 rounded-md py-1.5 pr-2 text-left text-[0.8rem] ${
              item.page.id === selectedPageId ? "text-foreground" : "text-foreground"
            }`}
            onclick={() => {
              openPage(item.page.id);
            }}
          >
            {#if item.page.icon}
              <NotesPageIcon icon={item.page.icon} size={14} class="shrink-0 text-muted-foreground" />
            {:else}
              <FileText class="size-3.5 shrink-0 text-muted-foreground" />
            {/if}
            <span class="min-w-0 flex-1 truncate">{title}</span>
            {#if item.parentStatus}
              <TriangleAlert
                class="size-3.5 shrink-0 text-destructive"
                aria-label={parentStatusLabel(item.parentStatus)}
                data-app-tooltip={parentStatusLabel(item.parentStatus)}
              />
            {/if}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>
