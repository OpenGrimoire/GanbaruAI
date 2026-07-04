<script lang="ts">
  import { tick } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import FileText from "@lucide/svelte/icons/file-text";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { planNotesSidebarNavigation } from "$lib/notes/page-sidebar";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesSearchNavigationPlan } from "$lib/notes/search";
  import type { NotesPage, NotesPageTemplate, NotesSearchResult } from "$lib/notes/types";
  import NotesPageRow from "./NotesPageRow.svelte";
  import NotesPageTemplateRow from "./NotesPageTemplateRow.svelte";

  let {
    focusSearchRequestId = 0,
  }: {
    focusSearchRequestId?: number;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();
  let search = $state("");
  let searchInput = $state<HTMLInputElement | null>(null);
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);
  let pendingDeleteTemplate = $state<NotesPageTemplate | null>(null);

  const sidebarPlan = $derived.by(() =>
    planNotesSidebarNavigation({
      pages: notes.pages,
      favoritePageIds: notes.favoritePageIds,
      recentPageIds: notes.recentPageIds,
      expandedPageIds: notes.sidebarExpandedPageIds,
      pageIdsWithChildren: notes.sidebarPageIdsWithChildren,
      missingParentPageIds: notes.sidebarMissingParentPageIds,
      trashedParentPageIds: notes.sidebarTrashedParentPageIds,
      activePageId: notes.selectedPageId,
      search,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
    })
  );
  const searchQuery = $derived(sidebarPlan.searchQuery);

  $effect(() => {
    const requestId = focusSearchRequestId;
    void requestId;
    if (requestId === 0) return;
    void tick().then(() => {
      searchInput?.focus();
    });
  });

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

  function createPage(): void {
    void notes.createPage("");
  }

  function createSubpage(parentPageId: string): void {
    void notes.createSubpage(parentPageId, "");
  }

  function createTemplateFromCurrentPage(): void {
    const page = notes.loadedPage;
    if (!page) return;
    const title = notesPageTitle(page, t("notes.untitled"));
    void notes.createPageTemplateFromCurrentPage(t("notes.pageTemplateName", title));
  }

  function applyTemplate(template: NotesPageTemplate): void {
    void notes.applyPageTemplate(template.id, template.name);
  }

  function duplicateTemplate(template: NotesPageTemplate): void {
    void notes.duplicatePageTemplate(
      template.id,
      t("notes.duplicatePageTemplateName", template.name),
    );
  }

  function duplicatePage(page: NotesPage): void {
    const title = notesPageTitle(page, t("notes.untitled"));
    void notes.duplicatePage(page.id, t("notes.duplicatePageTitle", title));
  }

  function moveTargets(page: NotesPage) {
    return notesPageMoveTargets(
      notes.pages,
      page.id,
      t("notes.workspace"),
      (candidate) => notesPageTitle(candidate, t("notes.untitled")),
      notes.recentPageIds,
    );
  }

  function confirmArchivePage(): void {
    const page = pendingArchivePage;
    pendingArchivePage = null;
    if (page) void notes.archivePage(page.id);
  }

  function confirmTrashPage(): void {
    const page = pendingTrashPage;
    pendingTrashPage = null;
    if (page) void notes.trashPage(page.id);
  }

  function confirmDeleteTemplate(): void {
    const template = pendingDeleteTemplate;
    pendingDeleteTemplate = null;
    if (template) void notes.deletePageTemplate(template.id);
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

  function openSearchResult(result: NotesSearchResult): void {
    const plan = notesSearchNavigationPlan(result);
    if (plan.blockId) {
      void notes.openNotesLink({ pageId: plan.pageId, blockId: plan.blockId }).then((opened) => {
        if (opened && plan.commentParent) notes.setActiveCommentParent(plan.commentParent);
      });
      return;
    }
    void notes.selectPage(plan.pageId).then(() => {
      if (plan.commentParent) notes.setActiveCommentParent(plan.commentParent);
    });
  }
</script>

<div class="h-full min-h-0 overflow-auto">
  <div class="mx-auto flex w-full max-w-6xl flex-col gap-5 px-4 py-4">
    <div class="flex flex-wrap items-center gap-2">
      <label class="flex min-w-64 flex-1 items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5">
        <Search class="size-4 shrink-0 text-muted-foreground" />
        <input
          bind:this={searchInput}
          class="min-w-0 flex-1 bg-transparent text-[0.866667rem] text-foreground outline-none placeholder:text-muted-foreground"
          bind:value={search}
          placeholder={t("notes.searchPlaceholder")}
          aria-label={t("notes.searchLabel")}
        />
      </label>
      <button
        type="button"
        class="flex h-8 shrink-0 items-center gap-1.5 rounded-md bg-primary px-2.5 text-[0.8rem] font-medium text-primary-foreground hover:bg-primary/90"
        onclick={createPage}
      >
        <Plus class="size-4" />
        <span>{t("notes.newPage")}</span>
      </button>
    </div>

    {#if notes.loading && notes.pages.length === 0}
      <div class="text-[0.866667rem] text-muted-foreground">{t("notes.loading")}</div>
    {:else if notes.loadError}
      <div class="text-[0.866667rem] text-destructive">
        {t("notes.loadFailed", notes.loadError)}
      </div>
    {:else if searchQuery}
      <section class="flex min-w-0 flex-col gap-2">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <h2 class="text-[0.8rem] font-semibold tracking-normal text-muted-foreground">
            {notes.searchLoading ? t("notes.searching") : t("notes.searchResults")}
          </h2>
          <label class="flex items-center gap-1.5 text-[0.733333rem] text-muted-foreground">
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
          <div class="text-[0.866667rem] text-destructive">
            {t("notes.searchFailed", notes.searchError)}
          </div>
        {:else if notes.searchLoading && notes.searchResults.length === 0}
          <div class="text-[0.866667rem] text-muted-foreground">{t("notes.searching")}</div>
        {:else if notes.searchResults.length === 0}
          <div class="text-[0.866667rem] text-muted-foreground">{t("notes.noSearchResults")}</div>
        {:else}
          <div class="grid gap-1 min-[720px]:grid-cols-2">
            {#each notes.searchResults as result (result.id)}
              {@const title = notesPageTitle(result.page, t("notes.untitled"))}
              <button
                type="button"
                class={`flex min-w-0 items-start gap-2 rounded-md px-2 py-1.5 text-left text-[0.866667rem] hover:bg-accent/70 ${
                  result.page.id === notes.selectedPageId ? "bg-accent text-accent-foreground" : "text-foreground"
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
                  {#if result.type === "comment" && result.comment_anchor}
                    <span class="block truncate text-[0.733333rem] text-muted-foreground">
                      {t("notes.searchResultCommentAnchor", result.comment_anchor.text)}
                    </span>
                  {/if}
                  {#if result.snippet}
                    <span class="mt-0.5 line-clamp-2 block text-[0.733333rem] leading-snug text-muted-foreground">
                      {result.snippet}
                    </span>
                  {/if}
                </span>
              </button>
            {/each}
          </div>
        {/if}
      </section>
    {:else}
      <div class="grid min-h-0 gap-5 min-[860px]:grid-cols-[minmax(0,1fr)_minmax(18rem,22rem)]">
        <section class="flex min-w-0 flex-col gap-4">
          {#if sidebarPlan.treeItems.length === 0}
            <div class="text-[0.866667rem] text-muted-foreground">{t("notes.noPages")}</div>
          {:else}
            {#if sidebarPlan.showNavigationSections && sidebarPlan.favoritePages.length > 0}
              <div class="flex min-w-0 flex-col gap-1">
                <h2 class="px-2 text-[0.733333rem] font-semibold tracking-normal text-muted-foreground">
                  {t("notes.favorites")}
                </h2>
                {#each sidebarPlan.favoritePages as page (page.id)}
                  <NotesPageRow
                    {page}
                    depth={0}
                    hasChildren={false}
                    collapsed={false}
                    parentStatus={sidebarPlan.parentStatusByPageId[page.id] ?? null}
                    favorited={notes.favoritePageIds.includes(page.id)}
                    selected={page.id === notes.selectedPageId}
                    onSelect={() => {
                      void notes.selectPage(page.id);
                    }}
                    onRename={(title) => {
                      void notes.renamePage(page.id, title);
                    }}
                    onToggleCollapsed={() => undefined}
                    onToggleFavorite={(favorited) => {
                      notes.setPageFavorited(page.id, favorited);
                    }}
                    onCreateChild={() => {
                      createSubpage(page.id);
                    }}
                    onDuplicate={() => {
                      duplicatePage(page);
                    }}
                    moveTargets={moveTargets(page)}
                    onMove={(parent) => {
                      void notes.movePage(page.id, parent);
                    }}
                    onArchive={() => {
                      pendingArchivePage = page;
                    }}
                    onTrash={() => {
                      pendingTrashPage = page;
                    }}
                  />
                {/each}
              </div>
            {/if}
            {#if sidebarPlan.showNavigationSections && sidebarPlan.recentPages.length > 0}
              <div class="flex min-w-0 flex-col gap-1">
                <h2 class="px-2 text-[0.733333rem] font-semibold tracking-normal text-muted-foreground">
                  {t("notes.recents")}
                </h2>
                {#each sidebarPlan.recentPages as page (page.id)}
                  <NotesPageRow
                    {page}
                    depth={0}
                    hasChildren={false}
                    collapsed={false}
                    parentStatus={sidebarPlan.parentStatusByPageId[page.id] ?? null}
                    favorited={notes.favoritePageIds.includes(page.id)}
                    selected={page.id === notes.selectedPageId}
                    onSelect={() => {
                      void notes.selectPage(page.id);
                    }}
                    onRename={(title) => {
                      void notes.renamePage(page.id, title);
                    }}
                    onToggleCollapsed={() => undefined}
                    onToggleFavorite={(favorited) => {
                      notes.setPageFavorited(page.id, favorited);
                    }}
                    onCreateChild={() => {
                      createSubpage(page.id);
                    }}
                    onDuplicate={() => {
                      duplicatePage(page);
                    }}
                    moveTargets={moveTargets(page)}
                    onMove={(parent) => {
                      void notes.movePage(page.id, parent);
                    }}
                    onArchive={() => {
                      pendingArchivePage = page;
                    }}
                    onTrash={() => {
                      pendingTrashPage = page;
                    }}
                  />
                {/each}
              </div>
            {/if}
            <div class="flex min-w-0 flex-col gap-1">
              <h2 class="px-2 text-[0.733333rem] font-semibold tracking-normal text-muted-foreground">
                {t("notes.pages")}
              </h2>
              {#each sidebarPlan.treeItems as item (item.page.id)}
                <NotesPageRow
                  page={item.page}
                  depth={item.depth}
                  hasChildren={item.hasChildren}
                  collapsed={item.collapsed}
                  parentStatus={item.parentStatus}
                  favorited={notes.favoritePageIds.includes(item.page.id)}
                  selected={item.page.id === notes.selectedPageId}
                  onSelect={() => {
                    void notes.selectPage(item.page.id);
                  }}
                  onRename={(title) => {
                    void notes.renamePage(item.page.id, title);
                  }}
                  onToggleCollapsed={(collapsed) => {
                    notes.setSidebarPageCollapsed(item.page.id, collapsed);
                  }}
                  onToggleFavorite={(favorited) => {
                    notes.setPageFavorited(item.page.id, favorited);
                  }}
                  onCreateChild={() => {
                    createSubpage(item.page.id);
                  }}
                  onDuplicate={() => {
                    duplicatePage(item.page);
                  }}
                  moveTargets={moveTargets(item.page)}
                  onMove={(parent) => {
                    void notes.movePage(item.page.id, parent);
                  }}
                  onArchive={() => {
                    pendingArchivePage = item.page;
                  }}
                  onTrash={() => {
                    pendingTrashPage = item.page;
                  }}
                />
              {/each}
            </div>
          {/if}
        </section>

        <aside class="flex min-w-0 flex-col gap-2">
          <div class="flex items-center justify-between gap-2">
            <h2 class="text-[0.733333rem] font-semibold tracking-normal text-muted-foreground">
              {t("notes.pageTemplates")}
            </h2>
            <button
              class="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-45"
              type="button"
              aria-label={t("notes.createPageTemplate")}
              data-app-tooltip={notes.loadedPage ? t("notes.createPageTemplate") : t("notes.createPageTemplateUnavailable")}
              disabled={!notes.loadedPage}
              onclick={createTemplateFromCurrentPage}
            >
              <Plus class="size-4" />
            </button>
          </div>
          {#if notes.pageTemplatesError}
            <div class="text-[0.8rem] text-destructive">
              {t("notes.loadPageTemplatesFailed", notes.pageTemplatesError)}
            </div>
          {:else if notes.pageTemplatesLoading && notes.pageTemplates.length === 0}
            <div class="text-[0.8rem] text-muted-foreground">{t("notes.loadingPageTemplates")}</div>
          {:else if notes.pageTemplates.length === 0}
            <div class="text-[0.8rem] text-muted-foreground">{t("notes.noPageTemplates")}</div>
          {:else}
            <div class="flex flex-col gap-1">
              {#each notes.pageTemplates as template (template.id)}
                <NotesPageTemplateRow
                  {template}
                  canUpdateFromCurrentPage={Boolean(notes.loadedPage)}
                  onApply={() => {
                    applyTemplate(template);
                  }}
                  onRename={(name) => {
                    void notes.renamePageTemplate(template.id, name);
                  }}
                  onUpdateFromCurrentPage={() => {
                    void notes.updatePageTemplateFromCurrentPage(template.id);
                  }}
                  onDuplicate={() => {
                    duplicateTemplate(template);
                  }}
                  onDelete={() => {
                    pendingDeleteTemplate = template;
                  }}
                />
              {/each}
            </div>
          {/if}

          <div class="mt-3 grid gap-2 border-t border-border pt-3">
            <button
              class={`flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-[0.866667rem] ${
                notes.viewMode === "archive"
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/70 hover:text-foreground"
              }`}
              aria-current={notes.viewMode === "archive" ? "page" : undefined}
              onclick={() => {
                void notes.openArchive();
              }}
            >
              <Archive class="size-4 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{t("notes.archive")}</span>
              {#if notes.archivedPages.length > 0}
                <span class="rounded bg-muted px-1.5 text-[0.733333rem] text-muted-foreground">
                  {notes.archivedPages.length}
                </span>
              {/if}
            </button>
            <button
              class={`flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-[0.866667rem] ${
                notes.viewMode === "trash"
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/70 hover:text-foreground"
              }`}
              aria-current={notes.viewMode === "trash" ? "page" : undefined}
              onclick={() => {
                void notes.openTrash();
              }}
            >
              <Trash2 class="size-4 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{t("notes.trash")}</span>
              {#if notes.trashedPages.length > 0}
                <span class="rounded bg-muted px-1.5 text-[0.733333rem] text-muted-foreground">
                  {notes.trashedPages.length}
                </span>
              {/if}
            </button>
          </div>
        </aside>
      </div>
    {/if}
  </div>
</div>

{#if pendingArchivePage}
  <ConfirmDialog
    title={t("notes.archiveConfirmTitle", notesPageTitle(pendingArchivePage, t("notes.untitled")))}
    message={t("notes.archiveConfirmMessage")}
    confirmLabel={t("notes.archiveConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={confirmArchivePage}
    onCancel={() => {
      pendingArchivePage = null;
    }}
  />
{/if}

{#if pendingTrashPage}
  <ConfirmDialog
    title={t("notes.trashConfirmTitle", notesPageTitle(pendingTrashPage, t("notes.untitled")))}
    message={t("notes.trashConfirmMessage")}
    confirmLabel={t("notes.trashConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={confirmTrashPage}
    onCancel={() => {
      pendingTrashPage = null;
    }}
  />
{/if}

{#if pendingDeleteTemplate}
  <ConfirmDialog
    title={t("notes.deletePageTemplateConfirmTitle", pendingDeleteTemplate.name)}
    message={t("notes.deletePageTemplateConfirmMessage")}
    confirmLabel={t("notes.deletePageTemplateConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={confirmDeleteTemplate}
    onCancel={() => {
      pendingDeleteTemplate = null;
    }}
  />
{/if}
