<script lang="ts">
  import { tick } from "svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FilePlus2 from "@lucide/svelte/icons/file-plus-2";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { NotesDestinationPickerTarget } from "$lib/notes/destination-picker";
  import type { NotesFolder } from "$lib/notes/types";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import {
    loadNotesOptionalComponent,
    retryNotesOptionalComponent,
    type LoadedNotesOptionalComponent,
  } from "./notes-component-registry";

  type NotesFolderRowMoveTarget = NotesDestinationPickerTarget & {
    folderId: string | null;
  };

  let {
    folder,
    depth,
    hasChildren,
    collapsed,
    renameRequestId = 0,
    moveTargets,
    onToggleCollapsed,
    onCreatePage,
    onCreateFolder,
    onRename,
    onMove,
    onDelete,
  }: {
    folder: NotesFolder;
    depth: number;
    hasChildren: boolean;
    collapsed: boolean;
    renameRequestId?: number;
    moveTargets: NotesFolderRowMoveTarget[];
    onToggleCollapsed: (collapsed: boolean) => void;
    onCreatePage: () => void;
    onCreateFolder: () => void;
    onRename: (name: string) => void;
    onMove: (parentFolderId: string | null) => void;
    onDelete: () => void;
  } = $props();

  const { t } = getLocalization();
  let editing = $state(false);
  let menuOpen = $state(false);
  let moveMenuOpen = $state(false);
  let nameDraft = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);
  let handledRenameRequestId = 0;
  let destinationPickerLoadState = $state<LazyComponentLoadState<
    "destination-picker",
    LoadedNotesOptionalComponent
  > | null>(null);

  $effect(() => {
    if (!editing) nameDraft = folder.name;
  });

  $effect(() => {
    if (!menuOpen) moveMenuOpen = false;
  });

  $effect(() => {
    if (renameRequestId <= 0 || renameRequestId === handledRenameRequestId) return;
    handledRenameRequestId = renameRequestId;
    editing = true;
  });

  $effect(() => {
    if (!editing) return;
    void tick().then(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  });

  $effect(() => {
    if (moveMenuOpen) requestDestinationPicker();
  });

  function requestDestinationPicker(retry = false): void {
    if (!retry && destinationPickerLoadState?.key === "destination-picker") return;
    const loadingState = beginLazyComponentLoad(destinationPickerLoadState, "destination-picker");
    destinationPickerLoadState = loadingState;
    const request = retry
      ? retryNotesOptionalComponent("destination-picker")
      : loadNotesOptionalComponent("destination-picker");
    void request.then((component) => {
      if (!destinationPickerLoadState) return;
      destinationPickerLoadState = resolveLazyComponentLoad(
        destinationPickerLoadState,
        "destination-picker",
        loadingState.requestId,
        component,
      );
    }).catch((error: unknown) => {
      if (!destinationPickerLoadState) return;
      destinationPickerLoadState = rejectLazyComponentLoad(
        destinationPickerLoadState,
        "destination-picker",
        loadingState.requestId,
        error,
      );
      console.error("load Notes folder destination picker failed", error);
    });
  }

  function closeMenu(): void {
    menuOpen = false;
    moveMenuOpen = false;
  }

  function saveRename(): void {
    const name = nameDraft.trim();
    editing = false;
    menuOpen = false;
    nameDraft = name || folder.name;
    if (!name || name === folder.name) return;
    onRename(name);
  }

  function handleRenameKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      saveRename();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      editing = false;
      nameDraft = folder.name;
    }
  }

  function moveToTarget(targetKey: string): void {
    const target = moveTargets.find((candidate) => candidate.key === targetKey);
    if (!target) return;
    closeMenu();
    onMove(target.folderId);
  }
</script>

<div
  class="notes-folder-row group relative"
  role="group"
  aria-label={folder.name}
  style={`--notes-folder-depth: ${Math.min(depth, 10)}`}
  use:dismissOnOutside={{ enabled: menuOpen, onDismiss: closeMenu }}
>
  {#if editing}
    <input
      bind:this={renameInput}
      class="notes-folder-row-content rounded-md border border-border bg-background px-2 py-1.5 text-[0.866667rem] text-foreground outline-none"
      style="width: calc(100% - var(--notes-folder-indent));"
      aria-label={t("notes.renameFolder")}
      bind:value={nameDraft}
      placeholder={t("notes.folderNamePlaceholder")}
      onkeydown={handleRenameKeydown}
      onblur={saveRename}
    />
  {:else}
    <div class="notes-folder-row-content flex min-w-0 items-center rounded-md pr-1 text-foreground hover:bg-accent/70">
      <button
        class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground"
        type="button"
        aria-label={collapsed ? t("notes.expandFolder") : t("notes.collapseFolder")}
        disabled={!hasChildren}
        onclick={(event) => {
          event.stopPropagation();
          onToggleCollapsed(!collapsed);
        }}
      >
        {#if hasChildren && collapsed}
          <ChevronRight class="size-4" />
        {:else if hasChildren}
          <ChevronDown class="size-4" />
        {/if}
      </button>
      <button
        class="flex min-w-0 flex-1 items-center gap-1.5 py-1.5 pr-1 text-left text-[0.866667rem] font-medium"
        type="button"
        aria-expanded={hasChildren ? !collapsed : undefined}
        onclick={() => {
          if (hasChildren) onToggleCollapsed(!collapsed);
        }}
      >
        {#if collapsed || !hasChildren}
          <Folder class="size-3.5 shrink-0 text-muted-foreground" />
        {:else}
          <FolderOpen class="size-3.5 shrink-0 text-muted-foreground" />
        {/if}
        <span class="min-w-0 flex-1 truncate">{folder.name}</span>
      </button>
      <button
        class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground"
        type="button"
        aria-label={t("notes.newNoteInFolder")}
        data-app-tooltip={t("notes.newNoteInFolder")}
        onclick={(event) => {
          event.stopPropagation();
          onCreatePage();
        }}
      >
        <FilePlus2 class="size-3.5" />
      </button>
      <button
        class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground"
        type="button"
        aria-label={t("notes.folderActions")}
        onclick={(event) => {
          event.stopPropagation();
          menuOpen = !menuOpen;
        }}
      >
        <MoreHorizontal class="size-4" />
      </button>
    </div>
  {/if}

  {#if menuOpen}
    <div
      class="notes-folder-action-menu absolute right-1 top-8 z-20 min-w-40 rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg"
      data-app-floating-surface
    >
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          closeMenu();
          onCreatePage();
        }}
      >
        <FilePlus2 class="size-4" />
        <span>{t("notes.newNoteInFolder")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          closeMenu();
          onCreateFolder();
        }}
      >
        <FolderPlus class="size-4" />
        <span>{t("notes.newSubfolder")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          editing = true;
          closeMenu();
        }}
      >
        <Pencil class="size-4" />
        <span>{t("notes.renameFolder")}</span>
      </button>
      {#if moveTargets.length > 0}
        <button
          class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
          type="button"
          aria-expanded={moveMenuOpen}
          onclick={() => {
            moveMenuOpen = !moveMenuOpen;
          }}
        >
          <FolderInput class="size-4" />
          <span>{t("notes.moveFolderTo")}</span>
        </button>
        {#if moveMenuOpen}
          <div class="notes-folder-move-menu border-y border-border bg-muted/25 py-1">
            {#if destinationPickerLoadState?.status === "ready" && destinationPickerLoadState.component.kind === "destination-picker"}
              {@const NotesDestinationPickerList = destinationPickerLoadState.component.component}
              <NotesDestinationPickerList
                targets={moveTargets}
                searchLabel={t("notes.moveDestinationSearch")}
                searchPlaceholder={t("notes.moveDestinationSearchPlaceholder")}
                recentLabel={t("notes.recentDestinations")}
                pagesLabel={t("notes.folders")}
                emptyLabel={t("notes.noFolderMoveTargets")}
                optionLabel={(target) => t("notes.moveFolderToTarget", target.title)}
                onSelect={moveToTarget}
                onClose={() => {
                  moveMenuOpen = false;
                }}
              />
            {:else if destinationPickerLoadState?.status === "failed"}
              <div class="p-2 text-[0.8rem] text-destructive" role="alert">
                <p>{t("common.viewLoadFailed", t("notes.moveFolderTo"))}</p>
                <button class="mt-2 min-h-8 rounded-md border border-border px-2 text-foreground hover:bg-accent" type="button" onclick={() => requestDestinationPicker(true)}>{t("common.retry")}</button>
              </div>
            {:else}
              <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
            {/if}
          </div>
        {/if}
      {/if}
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] text-destructive hover:bg-accent"
        type="button"
        onclick={() => {
          closeMenu();
          onDelete();
        }}
      >
        <Trash2 class="size-4" />
        <span>{t("notes.deleteFolder")}</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .notes-folder-row {
    --notes-folder-indent: calc(var(--notes-folder-depth) * 0.875rem);
  }

  .notes-folder-row-content {
    margin-left: var(--notes-folder-indent);
  }

  .notes-folder-action-menu {
    width: min(16rem, calc(100vw - 1rem));
    max-height: min(22rem, calc(100vh - 4rem));
    overflow-y: auto;
  }

  .notes-folder-move-menu {
    max-height: 12rem;
    overflow-y: auto;
  }
</style>
