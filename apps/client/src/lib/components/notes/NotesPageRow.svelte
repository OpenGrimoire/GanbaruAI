<script lang="ts">
  import { tick } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import { NOTES_PAGE_CHROME_EMOJI_SCALE } from "$lib/notes/page-icon";
  import type { NotesDestinationPickerTarget } from "$lib/notes/destination-picker";
  import type { NotesPageMoveTarget } from "$lib/notes/page-move";
  import type { NotesPageParentStatus } from "$lib/notes/page-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import type { NotesPage, NotesParent } from "$lib/notes/types";
  import Archive from "@lucide/svelte/icons/archive";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Copy from "@lucide/svelte/icons/copy";
  import FileText from "@lucide/svelte/icons/file-text";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import FolderTree from "@lucide/svelte/icons/folder-tree";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Plus from "@lucide/svelte/icons/plus";
  import Star from "@lucide/svelte/icons/star";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import {
    loadNotesOptionalComponent,
    retryNotesOptionalComponent,
    type LoadedNotesOptionalComponent,
  } from "./notes-component-registry";

  let {
    page,
    depth,
    hasChildren,
    collapsed,
    parentStatus,
    favorited,
    selected,
    onSelect,
    onRename,
    onToggleCollapsed,
    onToggleFavorite,
    onCreateChild,
    onDuplicate,
    moveTargets,
    onRequestMoveTargets = undefined,
    onMove,
    folderMoveTargets = undefined,
    onMoveToFolder = undefined,
    onArchive,
    onTrash,
    blockDropActive = false,
    onBlockDragOver,
    onBlockDragLeave,
    onBlockDrop,
    readOnly = false,
    displayTitle,
  }: {
    page: NotesPage;
    depth: number;
    hasChildren: boolean;
    collapsed: boolean;
    parentStatus: NotesPageParentStatus | null;
    favorited: boolean;
    selected: boolean;
    onSelect: () => void;
    onRename: (title: string) => void;
    onToggleCollapsed: (collapsed: boolean) => void;
    onToggleFavorite: (favorited: boolean) => void;
    onCreateChild: () => void;
    onDuplicate: () => void;
    moveTargets: NotesPageMoveTarget[];
    onRequestMoveTargets?: () => void | Promise<void>;
    onMove: (parent: NotesParent) => void;
    folderMoveTargets?: (NotesDestinationPickerTarget & { folderId: string | null })[];
    onMoveToFolder?: (folderId: string | null) => void;
    onArchive: () => void;
    onTrash: () => void;
    blockDropActive?: boolean;
    onBlockDragOver?: (pageId: string, event: DragEvent) => void;
    onBlockDragLeave?: (pageId: string, event: DragEvent) => void;
    onBlockDrop?: (pageId: string, event: DragEvent) => void;
    readOnly?: boolean;
    displayTitle?: string;
  } = $props();

  const { t } = getLocalization();
  let editing = $state(false);
  let menuOpen = $state(false);
  let moveMenuOpen = $state(false);
  let folderMoveMenuOpen = $state(false);
  let titleDraft = $state("");
  let renameInput = $state<HTMLInputElement | null>(null);
  let destinationPickerLoadState = $state<LazyComponentLoadState<
    "destination-picker",
    LoadedNotesOptionalComponent
  > | null>(null);
  const title = $derived(
    displayTitle === undefined
      ? notesPageTitle(page, t("notes.untitled"))
      : displayTitle.trim() || t("notes.untitled"),
  );
  const editableTitle = $derived(notesPageTitle(page, ""));

  $effect(() => {
    if (!editing) titleDraft = editableTitle;
  });

  $effect(() => {
    if (!menuOpen) {
      moveMenuOpen = false;
      folderMoveMenuOpen = false;
    }
  });

  $effect(() => {
    if (!editing) return;
    void tick().then(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  });

  $effect(() => {
    if (moveMenuOpen || folderMoveMenuOpen) {
      requestDestinationPicker();
      void onRequestMoveTargets?.();
    }
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
      console.error("load Notes page destination picker failed", error);
    });
  }

  function moveToTarget(targetKey: string): void {
    const target = moveTargets.find((candidate) => candidate.key === targetKey);
    if (!target) return;
    menuOpen = false;
    moveMenuOpen = false;
    onMove(target.parent);
  }

  function moveToFolderTarget(targetKey: string): void {
    const target = folderMoveTargets?.find((candidate) => candidate.key === targetKey);
    if (!target || !onMoveToFolder) return;
    menuOpen = false;
    moveMenuOpen = false;
    folderMoveMenuOpen = false;
    onMoveToFolder(target.folderId);
  }

  function closeMenu(): void {
    menuOpen = false;
    moveMenuOpen = false;
    folderMoveMenuOpen = false;
  }

  function saveRename(): void {
    const title = titleDraft.trim();
    editing = false;
    menuOpen = false;
    titleDraft = title;
    if (title === editableTitle) return;
    onRename(title);
  }

  function handleRenameKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      saveRename();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      editing = false;
      titleDraft = editableTitle;
    }
  }

  function parentStatusLabel(status: NotesPageParentStatus): string {
    return status === "trashed" ? t("notes.parentInTrash") : t("notes.parentMissing");
  }
</script>

<div
  class="notes-page-row group relative"
  class:notes-page-block-drop-target={blockDropActive}
  role="group"
  aria-label={title}
  style={`--notes-page-depth: ${Math.min(depth, 10)}`}
  data-app-tooltip={blockDropActive ? t("notes.dropBlockOnPage", title) : undefined}
  ondragover={(event) => onBlockDragOver?.(page.id, event)}
  ondragleave={(event) => onBlockDragLeave?.(page.id, event)}
  ondrop={(event) => onBlockDrop?.(page.id, event)}
  use:dismissOnOutside={{ enabled: menuOpen, onDismiss: closeMenu }}
>
  {#if editing}
    <input
      bind:this={renameInput}
      class="notes-page-row-content w-full rounded-md border border-border bg-background px-2 py-1.5 text-[0.866667rem] text-foreground outline-none"
      aria-label={t("notes.renamePage")}
      bind:value={titleDraft}
      placeholder={t("notes.titlePlaceholder")}
      onkeydown={handleRenameKeydown}
      onblur={saveRename}
    />
  {:else}
    <div
      class={`notes-page-row-content flex min-w-0 items-center rounded-md ${readOnly ? "pr-1" : "pr-6"} ${
        selected ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/70"
      }`}
    >
      <button
        class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-background/80 hover:text-foreground disabled:pointer-events-none disabled:opacity-0"
        type="button"
        aria-label={collapsed ? t("notes.expandPage") : t("notes.collapsePage")}
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
        class="flex min-w-0 flex-1 items-center gap-1.5 py-1.5 pr-1 text-left text-[0.866667rem]"
        type="button"
        onclick={onSelect}
      >
        {#if page.icon}
          <NotesPageIcon
            icon={page.icon}
            size={14}
            emojiScale={NOTES_PAGE_CHROME_EMOJI_SCALE}
            class="shrink-0"
          />
        {:else}
          <FileText class="size-3.5 shrink-0 text-muted-foreground" />
        {/if}
        <span class="min-w-0 flex-1 truncate">{title}</span>
        {#if parentStatus}
          <TriangleAlert
            class="size-3.5 shrink-0 text-destructive"
            aria-label={parentStatusLabel(parentStatus)}
            data-app-tooltip={parentStatusLabel(parentStatus)}
          />
        {/if}
        {#if favorited}
          <Star class="size-3.5 shrink-0 fill-current text-primary" aria-hidden="true" />
        {/if}
      </button>
      {#if !readOnly}
        <button
          class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground opacity-100 hover:bg-background/80 hover:text-foreground"
          type="button"
          aria-label={t("notes.newSubpage")}
          data-app-tooltip={t("notes.newSubpage")}
          onclick={(event) => {
            event.stopPropagation();
            onCreateChild();
          }}
        >
          <Plus class="size-3.5" />
        </button>
        <button
          class="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground opacity-100 hover:bg-background/80 hover:text-foreground"
          type="button"
          aria-label={t("notes.pageActions")}
          onclick={(event) => {
            event.stopPropagation();
            menuOpen = !menuOpen;
          }}
        >
          <MoreHorizontal class="size-4" />
        </button>
      {/if}
    </div>
  {/if}

  {#if menuOpen}
    <div
      class="notes-page-action-menu absolute right-1 top-8 z-20 min-w-36 rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg"
      data-app-floating-surface
    >
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          onToggleFavorite(!favorited);
          menuOpen = false;
        }}
      >
        <Star class={`size-4 ${favorited ? "fill-current text-primary" : ""}`} />
        <span>{favorited ? t("notes.removeFromFavorites") : t("notes.addToFavorites")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          editing = true;
          menuOpen = false;
        }}
      >
        <Pencil class="size-4" />
        <span>{t("notes.renamePage")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onDuplicate();
        }}
      >
        <Copy class="size-4" />
        <span>{t("notes.duplicatePage")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        aria-expanded={moveMenuOpen}
        onclick={() => {
          moveMenuOpen = !moveMenuOpen;
          folderMoveMenuOpen = false;
        }}
      >
        <FolderInput class="size-4" />
        <span>{t("notes.movePageTo")}</span>
      </button>
      {#if moveMenuOpen}
        <div
          class="notes-page-move-menu border-y border-border bg-muted/25 py-1"
          aria-label={t("notes.movePageTo")}
        >
          {#if destinationPickerLoadState?.status === "ready" && destinationPickerLoadState.component.kind === "destination-picker"}
            {@const NotesDestinationPickerList = destinationPickerLoadState.component.component}
            <NotesDestinationPickerList
              targets={moveTargets}
              searchLabel={t("notes.moveDestinationSearch")}
              searchPlaceholder={t("notes.moveDestinationSearchPlaceholder")}
              recentLabel={t("notes.recentDestinations")}
              pagesLabel={t("notes.allPages")}
              emptyLabel={t("notes.noPageMoveTargets")}
              optionLabel={(target) => t("notes.movePageToTarget", target.title)}
              onSelect={moveToTarget}
              onClose={() => {
                moveMenuOpen = false;
              }}
            />
          {:else if destinationPickerLoadState?.status === "failed"}
            <div class="p-2 text-[0.8rem] text-destructive" role="alert">
              <p>{t("common.viewLoadFailed", t("notes.movePageTo"))}</p>
              <button class="mt-2 min-h-8 rounded-md border border-border px-2 text-foreground hover:bg-accent" type="button" onclick={() => requestDestinationPicker(true)}>{t("common.retry")}</button>
            </div>
          {:else}
            <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
          {/if}
        </div>
      {/if}
      {#if folderMoveTargets && folderMoveTargets.length > 0 && onMoveToFolder}
        <button
          class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
          type="button"
          aria-expanded={folderMoveMenuOpen}
          onclick={() => {
            folderMoveMenuOpen = !folderMoveMenuOpen;
            moveMenuOpen = false;
          }}
        >
          <FolderTree class="size-4" />
          <span>{t("notes.movePageToFolder")}</span>
        </button>
        {#if folderMoveMenuOpen}
          <div
            class="notes-page-move-menu border-y border-border bg-muted/25 py-1"
            aria-label={t("notes.movePageToFolder")}
          >
            {#if destinationPickerLoadState?.status === "ready" && destinationPickerLoadState.component.kind === "destination-picker"}
              {@const NotesDestinationPickerList = destinationPickerLoadState.component.component}
              <NotesDestinationPickerList
                targets={folderMoveTargets}
                searchLabel={t("notes.moveDestinationSearch")}
                searchPlaceholder={t("notes.moveDestinationSearchPlaceholder")}
                recentLabel={t("notes.recentDestinations")}
                pagesLabel={t("notes.folders")}
                emptyLabel={t("notes.noFolderMoveTargets")}
                optionLabel={(target) => t("notes.movePageToFolderTarget", target.title)}
                onSelect={moveToFolderTarget}
                onClose={() => {
                  folderMoveMenuOpen = false;
                }}
              />
            {:else if destinationPickerLoadState?.status === "failed"}
              <div class="p-2 text-[0.8rem] text-destructive" role="alert">
                <p>{t("common.viewLoadFailed", t("notes.movePageToFolder"))}</p>
                <button class="mt-2 min-h-8 rounded-md border border-border px-2 text-foreground hover:bg-accent" type="button" onclick={() => requestDestinationPicker(true)}>{t("common.retry")}</button>
              </div>
            {:else}
              <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
            {/if}
          </div>
        {/if}
      {/if}
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onArchive();
        }}
      >
        <Archive class="size-4" />
        <span>{t("notes.archivePage")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] text-destructive hover:bg-accent"
        type="button"
        onclick={() => {
          menuOpen = false;
          onTrash();
        }}
      >
        <Trash2 class="size-4" />
        <span>{t("notes.moveToTrash")}</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .notes-page-row-content {
    margin-left: calc(var(--notes-page-depth) * 0.875rem);
  }

  .notes-page-block-drop-target .notes-page-row-content {
    background: hsl(var(--primary) / 0.12);
    box-shadow: inset 0 0 0 1px hsl(var(--primary) / 0.55);
  }

  .notes-page-action-menu {
    width: min(16rem, calc(100vw - 1rem));
    max-height: min(22rem, calc(100vh - 4rem));
    overflow-y: auto;
  }

  .notes-page-move-menu {
    max-height: 12rem;
    overflow-y: auto;
  }
</style>
