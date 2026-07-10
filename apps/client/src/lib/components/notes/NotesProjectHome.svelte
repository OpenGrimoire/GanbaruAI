<script lang="ts">
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    buildNotesNavigationTree,
    notesFolderMoveTargets,
    notesFoldersForProject,
    notesPageFolderMoveTargets,
  } from "$lib/notes/navigation-tree";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesPagesForProject } from "$lib/notes/project-membership";
  import type { NotesFolder, NotesPage } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import NotesFolderRow from "./NotesFolderRow.svelte";
  import NotesPageRow from "./NotesPageRow.svelte";

  let {
    projectId = null,
  }: {
    projectId?: string | null;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();

  let search = $state("");
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);
  let pendingDeleteFolder = $state<NotesFolder | null>(null);
  let folderRenameTargetId = $state<string | null>(null);
  let folderRenameRequestId = $state(0);
  let folderActionError = $state<string | null>(null);

  const projectPages = $derived.by(() => notesPagesForProject(notes.allPages, projectId));
  const projectFolders = $derived.by(() => notesFoldersForProject(notes.folders, projectId));
  const treeItems = $derived.by(() =>
    buildNotesNavigationTree(projectPages, projectFolders, {
      activePageId: notes.selectedPageId,
      expandedPageIds: notes.sidebarExpandedPageIds,
      collapsedFolderIds: notes.collapsedFolderIds,
      pageIdsWithChildren: notes.sidebarPageIdsWithChildren,
      missingParentPageIds: notes.sidebarMissingParentPageIds,
      trashedParentPageIds: notes.sidebarTrashedParentPageIds,
      query: search,
      titleForPage: (page) => notesPageTitle(page, t("notes.untitled")),
    })
  );

  function createPage(folderId: string | null = null): void {
    if (folderId) notes.setFolderCollapsed(folderId, false);
    void notes.createPage("", { projectId, folderId });
  }

  function createSubpage(parentPageId: string): void {
    void notes.createSubpage(parentPageId, "");
  }

  function nextFolderName(parentFolderId: string | null): string {
    const baseName = t("notes.defaultFolderName");
    const siblingNames = new Set(
      projectFolders
        .filter((folder) => folder.parent_folder_id === parentFolderId)
        .map((folder) => folder.name.trim().toLocaleLowerCase()),
    );
    if (!siblingNames.has(baseName.toLocaleLowerCase())) return baseName;
    let suffix = 2;
    while (siblingNames.has(`${baseName} ${suffix}`.toLocaleLowerCase())) suffix += 1;
    return `${baseName} ${suffix}`;
  }

  async function createFolder(parentFolderId: string | null = null): Promise<void> {
    if (!projectId) return;
    folderActionError = null;
    try {
      const folder = await notes.createFolder(
        projectId,
        nextFolderName(parentFolderId),
        parentFolderId,
      );
      if (parentFolderId) notes.setFolderCollapsed(parentFolderId, false);
      folderRenameTargetId = folder.id;
      folderRenameRequestId += 1;
    } catch (error) {
      folderActionError = error instanceof Error ? error.message : String(error);
    }
  }

  async function renameFolder(folderId: string, name: string): Promise<void> {
    folderActionError = null;
    try {
      await notes.renameFolder(folderId, name);
    } catch (error) {
      folderActionError = error instanceof Error ? error.message : String(error);
    }
  }

  async function moveFolder(folderId: string, parentFolderId: string | null): Promise<void> {
    folderActionError = null;
    try {
      await notes.moveFolder(folderId, parentFolderId);
      if (parentFolderId) notes.setFolderCollapsed(parentFolderId, false);
    } catch (error) {
      folderActionError = error instanceof Error ? error.message : String(error);
    }
  }

  function duplicatePage(page: NotesPage): void {
    const title = notesPageTitle(page, t("notes.untitled"));
    void notes.duplicatePage(page.id, t("notes.duplicatePageTitle", title));
  }

  function pageMoveTargets(page: NotesPage) {
    return notesPageMoveTargets(
      projectPages,
      page.id,
      t("notes.workspace"),
      (candidate) => notesPageTitle(candidate, t("notes.untitled")),
      notes.recentPageIds,
    );
  }

  function pageFolderMoveTargets(page: NotesPage) {
    return notesPageFolderMoveTargets(
      projectPages,
      projectFolders,
      page.id,
      t("notes.projectRoot"),
      (candidate) => notesPageTitle(candidate, t("notes.untitled")),
      notes.recentPageIds,
    ).filter((target) => target.kind !== "page");
  }

  function folderMoveTargets(folder: NotesFolder) {
    return notesFolderMoveTargets(projectFolders, folder.id, t("notes.projectRoot"));
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

  async function confirmDeleteFolder(): Promise<void> {
    const folder = pendingDeleteFolder;
    pendingDeleteFolder = null;
    if (!folder) return;
    folderActionError = null;
    try {
      await notes.deleteFolder(folder.id);
    } catch (error) {
      folderActionError = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col overflow-auto px-4 py-4">
  <div class="mx-auto flex w-full max-w-208 shrink-0 flex-wrap items-center gap-2">
    <label class="flex min-w-48 flex-1 items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5">
      <Search class="size-4 shrink-0 text-muted-foreground" />
      <input
        class="min-w-0 flex-1 bg-transparent text-[0.866667rem] text-foreground outline-none placeholder:text-muted-foreground"
        bind:value={search}
        placeholder={t("notes.searchPlaceholder")}
        aria-label={t("notes.searchLabel")}
      />
    </label>
    <button
      type="button"
      class="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-background px-2.5 text-[0.8rem] font-medium text-foreground hover:bg-accent disabled:pointer-events-none disabled:opacity-45"
      disabled={!projectId}
      onclick={() => {
        void createFolder();
      }}
    >
      <FolderPlus class="size-4" />
      <span>{t("notes.newFolder")}</span>
    </button>
    <button
      type="button"
      class="flex h-8 shrink-0 items-center gap-1.5 rounded-md bg-primary px-2.5 text-[0.8rem] font-medium text-primary-foreground hover:bg-primary/90"
      onclick={() => createPage()}
    >
      <Plus class="size-4" />
      <span>{t("notes.newPage")}</span>
    </button>
  </div>

  {#if folderActionError}
    <div class="mx-auto mt-2 w-full max-w-208 text-[0.8rem] text-destructive" role="alert">
      {t("notes.folderActionFailed", folderActionError)}
    </div>
  {/if}

  {#if notes.loading && projectPages.length === 0 && projectFolders.length === 0}
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
      {#each treeItems as item (item.key)}
        {#if item.kind === "folder"}
          <NotesFolderRow
            folder={item.folder}
            depth={item.depth}
            hasChildren={item.hasChildren}
            collapsed={item.collapsed}
            renameRequestId={folderRenameTargetId === item.folder.id ? folderRenameRequestId : 0}
            moveTargets={folderMoveTargets(item.folder)}
            onToggleCollapsed={(collapsed) => {
              notes.setFolderCollapsed(item.folder.id, collapsed);
            }}
            onCreatePage={() => createPage(item.folder.id)}
            onCreateFolder={() => {
              void createFolder(item.folder.id);
            }}
            onRename={(name) => {
              void renameFolder(item.folder.id, name);
            }}
            onMove={(parentFolderId) => {
              void moveFolder(item.folder.id, parentFolderId);
            }}
            onDelete={() => {
              pendingDeleteFolder = item.folder;
            }}
          />
        {:else}
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
            moveTargets={pageMoveTargets(item.page)}
            onMove={(parent) => {
              void notes.movePage(item.page.id, parent);
            }}
            folderMoveTargets={pageFolderMoveTargets(item.page)}
            onMoveToFolder={(folderId) => {
              void notes.movePageToFolder(item.page.id, folderId);
            }}
            onArchive={() => {
              pendingArchivePage = item.page;
            }}
            onTrash={() => {
              pendingTrashPage = item.page;
            }}
          />
        {/if}
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

{#if pendingDeleteFolder}
  <ConfirmDialog
    title={t("notes.deleteFolderConfirmTitle", pendingDeleteFolder.name)}
    message={t("notes.deleteFolderConfirmMessage")}
    confirmLabel={t("notes.deleteFolderConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => {
      void confirmDeleteFolder();
    }}
    onCancel={() => {
      pendingDeleteFolder = null;
    }}
  />
{/if}
