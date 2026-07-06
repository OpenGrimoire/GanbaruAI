<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { planNotesSidebarNavigation } from "$lib/notes/page-sidebar";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesSearchNavigationPlan } from "$lib/notes/search";
  import {
    getActiveNotesBlockDragId,
    NOTES_BLOCK_DRAG_MIME,
    planNotesBlockPageDrop,
  } from "$lib/notes/block-drag";
  import type { NotesPage, NotesPageTemplate, NotesSearchResult } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import Download from "@lucide/svelte/icons/download";
  import FileText from "@lucide/svelte/icons/file-text";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import NotesHtmlExportDialog from "./NotesHtmlExportDialog.svelte";
  import NotesAgentBridgeExportDialog from "./NotesAgentBridgeExportDialog.svelte";
  import NotesPageRow from "./NotesPageRow.svelte";
  import NotesPageTemplateRow from "./NotesPageTemplateRow.svelte";

  const notes = getNotes();
  const { t } = getLocalization();
  let search = $state("");
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);
  let pendingDeleteTemplate = $state<NotesPageTemplate | null>(null);
  let htmlExportOpen = $state(false);
  let agentBridgeExportOpen = $state(false);
  let blockDropTargetPageId = $state<string | null>(null);
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

  function exportHtmlArchive(input: {
    includePageTree: boolean;
    includeComments: boolean;
    includeResolvedComments: boolean;
    includeAssets: boolean;
    includeDatabaseViews: boolean;
  }) {
    return notes.exportHtmlArchive({
      include_page_tree: input.includePageTree,
      include_comments: input.includeComments,
      include_resolved_comments: input.includeResolvedComments,
      include_assets: input.includeAssets,
      include_database_views: input.includeDatabaseViews,
    });
  }

  function exportAgentBridge(input: {
    includeDescendants: boolean;
    includeBacklinks: boolean;
    includeDatabaseViews: boolean;
    includeTaskContext: boolean;
    includePageComments: boolean;
    includeResolvedComments: boolean;
    projectIds: string[];
  }) {
    return notes.exportAgentBridge({
      include_descendants: input.includeDescendants,
      include_backlinks: input.includeBacklinks,
      include_database_views: input.includeDatabaseViews,
      include_task_context: input.includeTaskContext,
      include_page_comments: input.includePageComments,
      include_resolved_comments: input.includeResolvedComments,
      project_ids: input.projectIds,
    });
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

  function requestArchivePage(page: NotesPage): void {
    pendingArchivePage = page;
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

  function currentTreeState() {
    return {
      blocksById: notes.blocksById,
      childIdsByParentId: notes.childIdsByParentId,
    };
  }

  function draggedBlockIdFromEvent(event: DragEvent): string | null {
    const transferred = event.dataTransfer?.getData(NOTES_BLOCK_DRAG_MIME) ?? "";
    return transferred || getActiveNotesBlockDragId();
  }

  function pageBlockDropPlan(pageId: string, event: DragEvent) {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!sourceBlockId) return null;
    return planNotesBlockPageDrop(
      currentTreeState(),
      sourceBlockId,
      pageId,
      notes.selectedPageId,
    );
  }

  function handlePageBlockDragOver(pageId: string, event: DragEvent): void {
    const plan = pageBlockDropPlan(pageId, event);
    if (!plan) {
      if (blockDropTargetPageId === pageId) blockDropTargetPageId = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    blockDropTargetPageId = pageId;
  }

  function handlePageBlockDragLeave(pageId: string, event: DragEvent): void {
    const target = event.currentTarget;
    const related = event.relatedTarget;
    if (
      target instanceof HTMLElement
      && related instanceof Node
      && target.contains(related)
    ) {
      return;
    }
    if (blockDropTargetPageId === pageId) blockDropTargetPageId = null;
  }

  function handlePageBlockDrop(pageId: string, event: DragEvent): void {
    const plan = pageBlockDropPlan(pageId, event);
    blockDropTargetPageId = null;
    if (!plan) return;
    event.preventDefault();
    void notes.moveBlockToPage(plan.blockId, plan.pageId);
  }
</script>

<aside class="notes-sidebar flex w-64 shrink-0 flex-col border-r border-border bg-muted/40">
  <div class="flex shrink-0 items-center justify-between gap-2 border-b border-border px-3 py-2">
    <div class="min-w-0 truncate text-[0.933333rem] font-semibold text-foreground">
      {t("notes.title")}
    </div>
    <div class="flex shrink-0 items-center gap-1">
      <button
        class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-45"
        aria-label={t("notes.htmlExportOpen")}
        data-app-tooltip={notes.loadedPage ? t("notes.htmlExportOpen") : t("notes.htmlExportUnavailable")}
        disabled={!notes.loadedPage}
        onclick={() => {
          htmlExportOpen = true;
        }}
      >
        <Download class="size-4" />
      </button>
      <button
        class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-45"
        aria-label={t("notes.agentBridgeExportOpen")}
        data-app-tooltip={notes.loadedPage ? t("notes.agentBridgeExportOpen") : t("notes.agentBridgeExportUnavailable")}
        disabled={!notes.loadedPage}
        onclick={() => {
          agentBridgeExportOpen = true;
        }}
      >
        <GitBranch class="size-4" />
      </button>
      <button
        class="rounded-md bg-primary p-1.5 text-primary-foreground hover:bg-primary/90"
        aria-label={t("notes.newPage")}
        onclick={createPage}
      >
        <Plus class="size-4" />
      </button>
    </div>
  </div>

  <div class="shrink-0 px-3 py-2">
    <label class="flex items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5">
      <Search class="size-4 shrink-0 text-muted-foreground" />
      <input
        class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground"
        bind:value={search}
        placeholder={t("notes.searchPlaceholder")}
        aria-label={t("notes.searchLabel")}
      />
    </label>
  </div>

  <div class="min-h-0 flex-1 overflow-auto px-2 pb-2">
    {#if notes.loading && notes.pages.length === 0}
      <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.loading")}</div>
    {:else if notes.loadError}
      <div class="px-2 py-2 text-[0.8rem] text-destructive">
        {t("notes.loadFailed", notes.loadError)}
      </div>
    {:else if searchQuery}
      <div class="flex flex-col gap-1">
        <div class="flex flex-wrap items-center justify-between gap-2 px-2 pb-1 pt-2">
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
          <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
            {t("notes.searching")}
          </div>
        {:else if notes.searchResults.length === 0}
          <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
            {t("notes.noSearchResults")}
          </div>
        {:else}
          {#each notes.searchResults as result (result.id)}
            {@const title = notesPageTitle(result.page, t("notes.untitled"))}
            <button
              type="button"
              class={`flex w-full min-w-0 items-start gap-2 rounded-md px-2 py-1.5 text-left text-[0.8rem] hover:bg-accent/70 ${
                result.page.id === notes.selectedPageId
                  ? "bg-accent text-accent-foreground"
                  : "text-foreground"
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
        {/if}
      </div>
    {:else if sidebarPlan.treeItems.length === 0}
      <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("notes.noPages")}
      </div>
    {:else}
      <div class="flex flex-col gap-1">
        {#if sidebarPlan.showNavigationSections && sidebarPlan.favoritePages.length > 0}
          <div class="px-2 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.favorites")}
          </div>
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
                requestArchivePage(page);
              }}
              onTrash={() => {
                pendingTrashPage = page;
              }}
              blockDropActive={blockDropTargetPageId === page.id}
              onBlockDragOver={handlePageBlockDragOver}
              onBlockDragLeave={handlePageBlockDragLeave}
              onBlockDrop={handlePageBlockDrop}
            />
          {/each}
        {/if}
        {#if sidebarPlan.showNavigationSections && sidebarPlan.recentPages.length > 0}
          <div class="px-2 pb-1 pt-3 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.recents")}
          </div>
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
                requestArchivePage(page);
              }}
              onTrash={() => {
                pendingTrashPage = page;
              }}
              blockDropActive={blockDropTargetPageId === page.id}
              onBlockDragOver={handlePageBlockDragOver}
              onBlockDragLeave={handlePageBlockDragLeave}
              onBlockDrop={handlePageBlockDrop}
            />
          {/each}
        {/if}
        {#if sidebarPlan.showPagesHeading}
          <div class="px-2 pb-1 pt-3 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.pages")}
          </div>
        {/if}
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
              requestArchivePage(item.page);
            }}
            onTrash={() => {
              pendingTrashPage = item.page;
            }}
            blockDropActive={blockDropTargetPageId === item.page.id}
            onBlockDragOver={handlePageBlockDragOver}
            onBlockDragLeave={handlePageBlockDragLeave}
            onBlockDrop={handlePageBlockDrop}
          />
        {/each}
      </div>
    {/if}
    {#if !searchQuery && !notes.loadError}
      <div class="mt-3 border-t border-border pt-2">
        <div class="flex items-center justify-between gap-2 px-2 pb-1 text-[0.7rem] font-medium text-muted-foreground">
          <span class="min-w-0 truncate">{t("notes.pageTemplates")}</span>
          <button
            class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-45"
            type="button"
            aria-label={t("notes.createPageTemplate")}
            data-app-tooltip={notes.loadedPage ? t("notes.createPageTemplate") : t("notes.createPageTemplateUnavailable")}
            disabled={!notes.loadedPage}
            onclick={createTemplateFromCurrentPage}
          >
            <Plus class="size-3.5" />
          </button>
        </div>
        {#if notes.pageTemplatesError}
          <div class="px-2 py-2 text-[0.8rem] text-destructive">
            {t("notes.loadPageTemplatesFailed", notes.pageTemplatesError)}
          </div>
        {:else if notes.pageTemplatesLoading && notes.pageTemplates.length === 0}
          <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
            {t("notes.loadingPageTemplates")}
          </div>
        {:else if notes.pageTemplates.length === 0}
          <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
            {t("notes.noPageTemplates")}
          </div>
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
      </div>
    {/if}
  </div>

  <div class="shrink-0 border-t border-border p-2">
    <button
      class={`mb-1 flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[0.866667rem] ${
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
      class={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[0.866667rem] ${
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

{#if htmlExportOpen && notes.loadedPage}
  <NotesHtmlExportDialog
    pageTitle={notesPageTitle(notes.loadedPage, t("notes.untitled"))}
    onExport={exportHtmlArchive}
    onCancel={() => {
      htmlExportOpen = false;
    }}
  />
{/if}

{#if agentBridgeExportOpen && notes.loadedPage}
  <NotesAgentBridgeExportDialog
    pageTitle={notesPageTitle(notes.loadedPage, t("notes.untitled"))}
    onExport={exportAgentBridge}
    onCancel={() => {
      agentBridgeExportOpen = false;
    }}
  />
{/if}

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

<style>
  @container notes-view (max-width: 520px) {
    .notes-sidebar {
      width: 100%;
      max-height: 42%;
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }
  }
</style>
