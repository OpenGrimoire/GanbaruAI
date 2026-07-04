<script lang="ts">
  import { tick } from "svelte";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { buildNotesPageTree } from "$lib/notes/page-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesPagesForProject } from "$lib/notes/project-membership";
  import type { NotesPage } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import NotesPageRow from "./NotesPageRow.svelte";

  let {
    focusSearchRequestId = 0,
    projectId = null,
  }: {
    focusSearchRequestId?: number;
    projectId?: string | null;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();

  let search = $state("");
  let searchInput = $state<HTMLInputElement | null>(null);
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);

  const projectPages = $derived.by(() => notesPagesForProject(notes.allPages, projectId));
  const treeItems = $derived.by(() =>
    buildNotesPageTree(projectPages, {
      activePageId: notes.selectedPageId,
      expandedPageIds: notes.sidebarExpandedPageIds,
      pageIdsWithChildren: notes.sidebarPageIdsWithChildren,
      missingParentPageIds: notes.sidebarMissingParentPageIds,
      trashedParentPageIds: notes.sidebarTrashedParentPageIds,
      query: search,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
    })
  );

  $effect(() => {
    const requestId = focusSearchRequestId;
    void requestId;
    if (requestId === 0) return;
    void tick().then(() => {
      searchInput?.focus();
    });
  });

  function createPage(): void {
    void notes.createPage("", { projectId });
  }

  function createSubpage(parentPageId: string): void {
    void notes.createSubpage(parentPageId, "");
  }

  function duplicatePage(page: NotesPage): void {
    const title = notesPageTitle(page, t("notes.untitled"));
    void notes.duplicatePage(page.id, t("notes.duplicatePageTitle", title));
  }

  function moveTargets(page: NotesPage) {
    return notesPageMoveTargets(
      projectPages,
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
</script>

<div class="flex h-full min-h-0 flex-col overflow-auto px-4 py-4">
  <div class="mx-auto flex w-full max-w-208 shrink-0 flex-wrap items-center gap-2">
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

  {#if notes.loading && projectPages.length === 0}
    <div class="mx-auto mt-4 w-full max-w-208 text-[0.866667rem] text-muted-foreground">{t("notes.loading")}</div>
  {:else if notes.loadError}
    <div class="mx-auto mt-4 w-full max-w-208 text-[0.866667rem] text-destructive">
      {t("notes.loadFailed", notes.loadError)}
    </div>
  {:else if treeItems.length === 0}
    <div class="mx-auto flex min-h-0 w-full max-w-208 flex-1 items-center justify-center text-center text-[0.866667rem] text-muted-foreground">
      {search.trim() ? t("notes.noSearchResults") : t("notes.noPages")}
    </div>
  {:else}
    <div class="mx-auto mt-4 flex w-full max-w-208 min-w-0 flex-col">
      {#each treeItems as item (item.page.id)}
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
