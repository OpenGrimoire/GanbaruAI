<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { orderedNotesPagesById } from "$lib/notes/page-navigation";
  import { buildNotesPageTree } from "$lib/notes/page-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import type { NotesPage, NotesSearchResult } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import FileText from "@lucide/svelte/icons/file-text";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import NotesPageRow from "./NotesPageRow.svelte";

  const notes = getNotes();
  const { t } = getLocalization();
  let search = $state("");
  let pendingTrashPage = $state<NotesPage | null>(null);
  const favoritePages = $derived(orderedNotesPagesById(notes.pages, notes.favoritePageIds));
  const recentPages = $derived(
    orderedNotesPagesById(notes.pages, notes.recentPageIds)
      .filter((page) => !notes.favoritePageIds.includes(page.id))
      .slice(0, 5),
  );
  const showNavigationSections = $derived(!search.trim());
  const searchQuery = $derived(search.trim());
  const treeItems = $derived(
    buildNotesPageTree(notes.pages, {
      activePageId: notes.selectedPageId,
      collapsedPageIds: notes.sidebarCollapsedPageIds,
      query: showNavigationSections ? "" : search,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
    }),
  );

  $effect(() => {
    const query = searchQuery;
    const timer = window.setTimeout(() => {
      void notes.search(query);
    }, query ? 180 : 0);
    return () => {
      window.clearTimeout(timer);
    };
  });

  function createPage(): void {
    void notes.createPage(t("notes.defaultPageTitle"));
  }

  function createSubpage(parentPageId: string): void {
    void notes.createSubpage(parentPageId, t("notes.defaultPageTitle"));
  }

  function duplicatePage(page: NotesPage): void {
    const title = notesPageTitle(page, t("notes.untitled"));
    void notes.duplicatePage(page.id, t("notes.duplicatePageTitle", title));
  }

  function moveTargets(page: NotesPage) {
    return notesPageMoveTargets(notes.pages, page.id, t("notes.workspace"), (candidate) =>
      notesPageTitle(candidate, t("notes.untitled"))
    );
  }

  function archivePage(page: NotesPage): void {
    void notes.archivePage(page.id);
  }

  function confirmTrashPage(): void {
    const page = pendingTrashPage;
    pendingTrashPage = null;
    if (page) void notes.trashPage(page.id);
  }

  function searchResultTypeLabel(result: NotesSearchResult): string {
    if (result.type === "page") return t("notes.searchResultPage");
    if (result.type === "block") return t("notes.searchResultBlock");
    return t("notes.searchResultComment");
  }

  function openSearchResult(result: NotesSearchResult): void {
    if (result.type === "block" && result.block_id) {
      void notes.openNotesLink({ pageId: result.page.id, blockId: result.block_id });
      return;
    }
    void notes.selectPage(result.page.id);
  }
</script>

<aside class="notes-sidebar flex w-64 shrink-0 flex-col border-r border-border bg-muted/40">
  <div class="flex shrink-0 items-center justify-between gap-2 border-b border-border px-3 py-2">
    <div class="min-w-0 truncate text-[0.933333rem] font-semibold text-foreground">
      {t("notes.title")}
    </div>
    <button
      class="rounded-md bg-primary p-1.5 text-primary-foreground hover:bg-primary/90"
      aria-label={t("notes.newPage")}
      onclick={createPage}
    >
      <Plus class="size-4" />
    </button>
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
        <div class="px-2 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
          {notes.searchLoading ? t("notes.searching") : t("notes.searchResults")}
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
                  {searchResultTypeLabel(result)}
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
    {:else if treeItems.length === 0}
      <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("notes.noPages")}
      </div>
    {:else}
      <div class="flex flex-col gap-1">
        {#if showNavigationSections && favoritePages.length > 0}
          <div class="px-2 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.favorites")}
          </div>
          {#each favoritePages as page (page.id)}
            <NotesPageRow
              {page}
              depth={0}
              hasChildren={false}
              collapsed={false}
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
                archivePage(page);
              }}
              onTrash={() => {
                pendingTrashPage = page;
              }}
            />
          {/each}
        {/if}
        {#if showNavigationSections && recentPages.length > 0}
          <div class="px-2 pb-1 pt-3 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.recents")}
          </div>
          {#each recentPages as page (page.id)}
            <NotesPageRow
              {page}
              depth={0}
              hasChildren={false}
              collapsed={false}
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
                archivePage(page);
              }}
              onTrash={() => {
                pendingTrashPage = page;
              }}
            />
          {/each}
        {/if}
        {#if showNavigationSections && (favoritePages.length > 0 || recentPages.length > 0)}
          <div class="px-2 pb-1 pt-3 text-[0.7rem] font-medium text-muted-foreground">
            {t("notes.pages")}
          </div>
        {/if}
        {#each treeItems as item (item.page.id)}
          <NotesPageRow
            page={item.page}
            depth={item.depth}
            hasChildren={item.hasChildren}
            collapsed={item.collapsed}
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
              archivePage(item.page);
            }}
            onTrash={() => {
              pendingTrashPage = item.page;
            }}
          />
        {/each}
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
