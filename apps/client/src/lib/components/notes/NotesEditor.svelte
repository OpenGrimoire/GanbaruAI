<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import Download from "@lucide/svelte/icons/download";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import History from "@lucide/svelte/icons/history";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import Link2 from "@lucide/svelte/icons/link-2";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Pencil from "@lucide/svelte/icons/pencil";
  import PencilLine from "@lucide/svelte/icons/pencil-line";
  import SmilePlus from "@lucide/svelte/icons/smile-plus";
  import Square from "@lucide/svelte/icons/square";
  import SquareArrowLeft from "@lucide/svelte/icons/square-arrow-left";
  import SquareArrowUpRight from "@lucide/svelte/icons/square-arrow-up-right";
  import Star from "@lucide/svelte/icons/star";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import type { NotesPageOpenMode } from "$lib/notes/page-open-mode";
  import {
    notesCommentParentKey,
    openNotesCommentThreadCount,
    unreadNotesCommentThreadCount,
  } from "$lib/notes/comments";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import { addNotesWorkspaceBreadcrumb, buildNotesPageBreadcrumb } from "$lib/notes/page-breadcrumb";
  import { notesPageTitle } from "$lib/notes/page-title";
  import {
    notesPageProjectId,
    notesPagesForProject,
  } from "$lib/notes/project-membership";
  import { openNotesSuggestionCount } from "$lib/notes/suggestions";
  import { buildNotesTableOfContents } from "$lib/notes/table-of-contents";
  import type {
    NotesAgentBridgeExportRequest,
    NotesHtmlExportRequest,
    NotesPage,
    NotesPageIcon as NotesPageIconValue,
    NotesParent,
  } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { cn } from "$lib/utils";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import NotesAgentBridgeExportDialog from "./NotesAgentBridgeExportDialog.svelte";
  import NotesBacklinks from "./NotesBacklinks.svelte";
  import NotesBlockList from "./NotesBlockList.svelte";
  import NotesComments from "./NotesComments.svelte";
  import NotesDestinationPickerList from "./NotesDestinationPickerList.svelte";
  import NotesHtmlExportDialog from "./NotesHtmlExportDialog.svelte";
  import NotesPageCover from "./NotesPageCover.svelte";
  import NotesPageCoverMenu from "./NotesPageCoverMenu.svelte";
  import NotesPageHistory from "./NotesPageHistory.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import NotesPageIconMenu from "./NotesPageIconMenu.svelte";
  import NotesPageLinks from "./NotesPageLinks.svelte";
  import NotesSuggestions from "./NotesSuggestions.svelte";

  type NotesEditorPanel = "links" | "comments" | "suggestions" | "history";

  let {
    projectId = null,
    openMode = "full",
    onClose,
    onOpenModeChange,
  }: {
    projectId?: string | null;
    openMode?: NotesPageOpenMode;
    onClose?: () => void;
    onOpenModeChange?: (mode: NotesPageOpenMode) => void;
  } = $props();

  const notes = getNotes();
  const localization = getLocalization();
  const { t } = localization;

  let titleDraft = $state("");
  let titleInput: HTMLInputElement | null = $state(null);
  let iconMenuAnchor = $state<"title" | "page-icon" | null>(null);
  let coverMenuOpen = $state(false);
  let pageMenuOpen = $state(false);
  let moveMenuOpen = $state(false);
  let openModeMenuOpen = $state(false);
  let activePanel = $state<NotesEditorPanel | null>(null);
  let htmlExportOpen = $state(false);
  let agentBridgeExportOpen = $state(false);
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);
  let blockScrollViewport: HTMLDivElement | null = $state(null);
  let lastTitlePageId = "";
  let lastHandledTitleFocusRequestId = 0;

  const page = $derived(notes.loadedPage);
  const editablePageTitle = $derived(page ? notesPageTitle(page, "") : "");
  const currentPageTitle = $derived(page ? notesPageTitle(page, t("notes.untitled")) : t("notes.untitled"));
  const pageIconLabel = $derived(pageIconScreenReaderText(page?.icon ?? null));
  const effectiveProjectId = $derived(page ? notesPageProjectId(page) ?? projectId : projectId);
  const projectPages = $derived(notesPagesForProject(notes.allPages, effectiveProjectId));
  const moveTargets = $derived(page
    ? notesPageMoveTargets(
        projectPages,
        page.id,
        t("notes.workspace"),
        (candidate) => notesPageTitle(candidate, t("notes.untitled")),
        notes.recentPageIds,
      )
    : []);
  const breadcrumbItems = $derived(
    notes.pageBreadcrumbItems.length > 0
      ? addNotesWorkspaceBreadcrumb(
          notes.pageBreadcrumbItems,
          t("notes.workspace"),
          t("notes.untitled"),
          t("notes.breadcrumbMissingPage"),
        )
      : buildNotesPageBreadcrumb(page, notes.allPages, t("notes.workspace"), t("notes.untitled")),
  );
  const tableOfContentsItems = $derived(buildNotesTableOfContents(notes.flatBlocks));
  const pageFavorited = $derived(page ? notes.favoritePageIds.includes(page.id) : false);
  const editedDateLabel = $derived(
    page ? new Date(page.last_edited_time).toLocaleString(localization.locale) : "",
  );
  const openCommentCount = $derived(openNotesCommentThreadCount(notes.commentThreads));
  const unreadCommentCount = $derived(unreadNotesCommentThreadCount(notes.commentThreads));
  const openSuggestionCount = $derived(openNotesSuggestionCount(notes.suggestions));
  const linksBadgeCount = $derived(notes.backlinks.length + notes.pageAliases.length + notes.unresolvedLinks.length);
  const commentsBadgeCount = $derived(unreadCommentCount > 0 ? unreadCommentCount : openCommentCount);
  const peekMode = $derived(openMode !== "full");

  $effect(() => {
    if (!page || page.id === lastTitlePageId) return;
    if (lastTitlePageId) notes.clearPageTitleDraft(lastTitlePageId);
    lastTitlePageId = page.id;
    notes.clearPageTitleDraft(page.id);
    titleDraft = editablePageTitle;
    activePanel = null;
    pageMenuOpen = false;
    moveMenuOpen = false;
    openModeMenuOpen = false;
    iconMenuAnchor = null;
    coverMenuOpen = false;
  });

  $effect(() => {
    const titleFocusRequestId = notes.titleFocusRequestId;
    if (titleFocusRequestId === lastHandledTitleFocusRequestId) return;
    if (!page || notes.titleFocusPageId !== page.id) return;
    lastHandledTitleFocusRequestId = titleFocusRequestId;
    focusTitleInput(false);
  });

  $effect(() => {
    const _focusRequestId = notes.focusRequestId;
    const focusSelection = notes.focusSelection;
    const blockId = notes.focusBlockId;
    if (!blockId) return;
    void tick().then(() => {
      const anchor = blockScrollViewport?.querySelector<HTMLElement>(
        `#${CSS.escape(notesBlockAnchorId(blockId))}`,
      );
      anchor?.scrollIntoView({
        block: focusSelection ? "nearest" : "center",
        inline: "nearest",
      });
    });
  });

  onDestroy(() => {
    if (lastTitlePageId) notes.clearPageTitleDraft(lastTitlePageId);
  });

  async function saveTitle(): Promise<void> {
    if (!page) return;
    const title = titleDraft.trim();
    titleDraft = title;
    if (title === editablePageTitle) {
      notes.clearPageTitleDraft(page.id);
      return;
    }
    await notes.renamePage(page.id, title);
    notes.clearPageTitleDraft(page.id);
  }

  function handleTitleInput(event: Event): void {
    if (!page) return;
    const target = event.currentTarget;
    const title = target instanceof HTMLInputElement ? target.value : titleDraft;
    notes.setPageTitleDraft(page.id, title);
  }

  function handleTitleKeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      event.currentTarget instanceof HTMLInputElement && event.currentTarget.blur();
    }
  }

  function actionButtonClass(active = false): string {
    return cn(
      "relative flex h-7 min-w-7 shrink-0 items-center justify-center rounded-md px-1.5 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground",
      active && "bg-accent text-foreground",
    );
  }

  function menuItemClass(destructive = false): string {
    return cn(
      "flex w-full items-center gap-2 rounded px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent",
      destructive ? "text-destructive" : "text-popover-foreground",
    );
  }

  function openModeMenuItemClass(mode: NotesPageOpenMode): string {
    return cn(
      "flex w-full items-center gap-2 rounded px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent",
      openMode === mode ? "bg-accent text-accent-foreground" : "text-popover-foreground",
    );
  }

  function togglePanel(panel: NotesEditorPanel): void {
    activePanel = activePanel === panel ? null : panel;
    pageMenuOpen = false;
    moveMenuOpen = false;
    if (activePanel === "history") void notes.reloadPageHistory();
  }

  function openPanelFromMenu(panel: NotesEditorPanel): void {
    activePanel = panel;
    pageMenuOpen = false;
    moveMenuOpen = false;
    if (panel === "history") void notes.reloadPageHistory();
  }

  function closePageMenu(): void {
    pageMenuOpen = false;
    moveMenuOpen = false;
  }

  function closeOpenModeMenu(): void {
    openModeMenuOpen = false;
  }

  function closeActionPanel(): void {
    activePanel = null;
  }

  function toggleCoverMenu(): void {
    const nextOpen = !coverMenuOpen;
    coverMenuOpen = nextOpen;
    if (nextOpen) {
      iconMenuAnchor = null;
      closePageMenu();
      closeActionPanel();
    }
  }

  function toggleIconMenu(anchor: "title" | "page-icon"): void {
    const nextAnchor = iconMenuAnchor === anchor ? null : anchor;
    iconMenuAnchor = nextAnchor;
    if (nextAnchor) {
      coverMenuOpen = false;
      closePageMenu();
      closeActionPanel();
    }
  }

  function closeCoverMenu(): void {
    coverMenuOpen = false;
  }

  function closeIconMenu(): void {
    iconMenuAnchor = null;
  }

  function selectOpenMode(mode: NotesPageOpenMode): void {
    openModeMenuOpen = false;
    onOpenModeChange?.(mode);
  }

  function openAsFullPage(): void {
    selectOpenMode("full");
  }

  function focusTitleInput(selectTitle = true): void {
    closePageMenu();
    void tick().then(() => {
      titleInput?.focus();
      if (selectTitle) {
        titleInput?.select();
        return;
      }
      const offset = titleInput?.value.length ?? 0;
      titleInput?.setSelectionRange(offset, offset);
    });
  }

  function duplicateCurrentPage(): void {
    if (!page) return;
    void notes.duplicatePage(page.id, t("notes.duplicatePageTitle", currentPageTitle));
    closePageMenu();
  }

  function moveToTarget(targetKey: string): void {
    if (!page) return;
    const target = moveTargets.find((candidate) => candidate.key === targetKey);
    if (!target) return;
    closePageMenu();
    void notes.movePage(page.id, target.parent);
  }

  function confirmArchivePage(): void {
    const targetPage = pendingArchivePage;
    pendingArchivePage = null;
    if (targetPage) void notes.archivePage(targetPage.id);
  }

  function confirmTrashPage(): void {
    const targetPage = pendingTrashPage;
    pendingTrashPage = null;
    if (targetPage) void notes.trashPage(targetPage.id);
  }

  function archiveCurrentPage(): void {
    if (!page) return;
    pendingArchivePage = page;
    closePageMenu();
  }

  function trashCurrentPage(): void {
    if (!page) return;
    pendingTrashPage = page;
    closePageMenu();
  }

  function exportHtmlArchive(input: {
    includePageTree: boolean;
    includeComments: boolean;
    includeResolvedComments: boolean;
    includeAssets: boolean;
    includeDatabaseViews: boolean;
  }) {
    const request: Omit<NotesHtmlExportRequest, "page_id"> = {
      include_page_tree: input.includePageTree,
      include_comments: input.includeComments,
      include_resolved_comments: input.includeResolvedComments,
      include_assets: input.includeAssets,
      include_database_views: input.includeDatabaseViews,
    };
    return notes.exportHtmlArchive(request);
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
    const request: NotesAgentBridgeExportRequest = {
      include_descendants: input.includeDescendants,
      include_backlinks: input.includeBacklinks,
      include_database_views: input.includeDatabaseViews,
      include_task_context: input.includeTaskContext,
      include_page_comments: input.includePageComments,
      include_resolved_comments: input.includeResolvedComments,
      project_ids: input.projectIds,
    };
    return notes.exportAgentBridge(request);
  }

  function openPageDiscussion(): void {
    if (!page) return;
    const pageParent = { type: "page_id", page_id: page.id } satisfies NotesParent;
    if (!notes.activeCommentParent || notesCommentParentKey(notes.activeCommentParent) !== notesCommentParentKey(pageParent)) {
      notes.setActiveCommentParent(pageParent);
    }
    activePanel = "comments";
    pageMenuOpen = false;
    moveMenuOpen = false;
  }

  function pageIconScreenReaderText(icon: NotesPageIconValue | null): string {
    if (!icon) return t("notes.noPageIcon");
    if (icon.type === "emoji") return icon.emoji;
    if (icon.type === "icon") return icon.icon.name;
    if (icon.type === "custom_emoji") return icon.custom_emoji.name ?? t("notes.pageIconCustomEmoji");
    return icon.type === "external" ? t("notes.pageIconImage") : icon.file.name ?? t("notes.pageIconImage");
  }
</script>

{#if page}
  <section class={cn("flex min-w-0 flex-1 flex-col overflow-hidden", peekMode && "h-full w-full bg-background")}>
    <div
      class={cn("relative z-40 shrink-0", peekMode && "bg-background")}
      style={peekMode ? "" : "background-color: var(--cal-bg);"}
      use:dismissOnOutside={{ enabled: !!activePanel, onDismiss: closeActionPanel }}
    >
      <div class="flex items-center gap-1 px-3" style="height: var(--cal-header-row-h);">
        <div class="group/open-mode flex shrink-0 items-center gap-0.5">
          {#if openMode === "side"}
            <button
              type="button"
              class={actionButtonClass()}
              aria-label={t("notes.closePeek")}
              data-app-tooltip={t("notes.closePeek")}
              onclick={() => onClose?.()}
            >
              <X class="size-4" />
            </button>
          {/if}
          {#if peekMode}
            <button
              type="button"
              class={actionButtonClass()}
              aria-label={t("notes.expandNote")}
              data-app-tooltip={t("notes.expandNote")}
              onclick={openAsFullPage}
            >
              <SquareArrowUpRight class="size-4" />
            </button>
          {/if}
          <div
            class={cn(
              "relative transition-opacity",
              openModeMenuOpen ? "opacity-100" : "opacity-0 group-hover/open-mode:opacity-100 group-focus-within/open-mode:opacity-100",
            )}
            use:dismissOnOutside={{ enabled: openModeMenuOpen, onDismiss: closeOpenModeMenu }}
          >
            <button
              type="button"
              class={actionButtonClass(openModeMenuOpen)}
              aria-label={t("notes.noteOpenMode")}
              data-app-tooltip={t("notes.noteOpenMode")}
              aria-expanded={openModeMenuOpen}
              onclick={() => {
                openModeMenuOpen = !openModeMenuOpen;
                closePageMenu();
              }}
            >
              {#if openMode === "side"}
                <SquareArrowLeft class="size-4" />
              {:else}
                <Square class="size-4" />
              {/if}
            </button>
            {#if openModeMenuOpen}
              <div
                class="absolute left-0 top-8 z-50 w-44 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg"
                role="menu"
                data-app-floating-surface
              >
                <button
                  class={openModeMenuItemClass("side")}
                  type="button"
                  role="menuitemradio"
                  aria-checked={openMode === "side"}
                  onclick={() => selectOpenMode("side")}
                >
                  <SquareArrowLeft class="size-4" />
                  <span class="min-w-0 flex-1 truncate">{t("notes.sidePeek")}</span>
                  {#if openMode === "side"}
                    <Check class="size-4 shrink-0" />
                  {/if}
                </button>
                <button
                  class={openModeMenuItemClass("center")}
                  type="button"
                  role="menuitemradio"
                  aria-checked={openMode === "center"}
                  onclick={() => selectOpenMode("center")}
                >
                  <Square class="size-4" />
                  <span class="min-w-0 flex-1 truncate">{t("notes.centerPeek")}</span>
                  {#if openMode === "center"}
                    <Check class="size-4 shrink-0" />
                  {/if}
                </button>
                <button
                  class={openModeMenuItemClass("full")}
                  type="button"
                  role="menuitemradio"
                  aria-checked={openMode === "full"}
                  onclick={() => selectOpenMode("full")}
                >
                  <SquareArrowUpRight class="size-4" />
                  <span class="min-w-0 flex-1 truncate">{t("notes.fullPage")}</span>
                  {#if openMode === "full"}
                    <Check class="size-4 shrink-0" />
                  {/if}
                </button>
              </div>
            {/if}
          </div>
        </div>
        <div class="min-w-0 flex-1"></div>
        <div class="hidden shrink-0 truncate px-2 text-[0.8rem] text-muted-foreground min-[560px]:block">
          {t("notes.metadataEdited", editedDateLabel)}
        </div>
        <button
          type="button"
          class={actionButtonClass(pageFavorited)}
          aria-label={pageFavorited ? t("notes.removeFromFavorites") : t("notes.addToFavorites")}
          data-app-tooltip={pageFavorited ? t("notes.removeFromFavorites") : t("notes.addToFavorites")}
          onclick={() => {
            notes.setPageFavorited(page.id, !pageFavorited);
          }}
        >
          <Star class={`size-4 ${pageFavorited ? "fill-current" : ""}`} />
        </button>
        <button
          type="button"
          class={actionButtonClass(activePanel === "links")}
          aria-label={t("notes.noteLinks")}
          data-app-tooltip={t("notes.noteLinks")}
          onclick={() => togglePanel("links")}
        >
          <Link2 class="size-4" />
          {#if linksBadgeCount > 0}
            <span class="ml-1 text-[0.7rem] leading-none">{linksBadgeCount}</span>
          {/if}
        </button>
        <button
          type="button"
          class={actionButtonClass(activePanel === "comments")}
          aria-label={t("notes.commentsCount", openCommentCount)}
          data-app-tooltip={t("notes.commentsCount", openCommentCount)}
          onclick={() => togglePanel("comments")}
        >
          <MessageSquare class="size-4" />
          {#if commentsBadgeCount > 0}
            <span class={cn("ml-1 text-[0.7rem] leading-none", unreadCommentCount > 0 && "font-semibold text-primary")}>
              {commentsBadgeCount}
            </span>
          {/if}
        </button>
        <button
          type="button"
          class={actionButtonClass(activePanel === "suggestions")}
          aria-label={t("notes.suggestionsCount", openSuggestionCount)}
          data-app-tooltip={t("notes.suggestionsCount", openSuggestionCount)}
          onclick={() => togglePanel("suggestions")}
        >
          <PencilLine class="size-4" />
          {#if openSuggestionCount > 0}
            <span class="ml-1 text-[0.7rem] leading-none">{openSuggestionCount}</span>
          {/if}
        </button>
        <div
          class="relative"
          use:dismissOnOutside={{ enabled: pageMenuOpen, onDismiss: closePageMenu }}
        >
          <button
            type="button"
            class={actionButtonClass(pageMenuOpen)}
            aria-label={t("notes.pageActions")}
            data-app-tooltip={t("notes.pageActions")}
            aria-expanded={pageMenuOpen}
            onclick={() => {
              pageMenuOpen = !pageMenuOpen;
              activePanel = null;
              if (!pageMenuOpen) moveMenuOpen = false;
            }}
          >
            <MoreHorizontal class="size-4" />
          </button>
          {#if pageMenuOpen}
            <div
              class="absolute right-0 top-8 z-50 w-60 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-lg"
              role="menu"
              data-app-floating-surface
            >
              <button class={menuItemClass()} type="button" role="menuitem" onclick={() => focusTitleInput()}>
                <Pencil class="size-4" />
                <span>{t("notes.renamePage")}</span>
              </button>
              <button class={menuItemClass()} type="button" role="menuitem" onclick={duplicateCurrentPage}>
                <Copy class="size-4" />
                <span>{t("notes.duplicatePage")}</span>
              </button>
              <button
                class={menuItemClass()}
                type="button"
                role="menuitem"
                aria-expanded={moveMenuOpen}
                onclick={() => {
                  moveMenuOpen = !moveMenuOpen;
                }}
              >
                <FolderInput class="size-4" />
                <span>{t("notes.movePageTo")}</span>
              </button>
              {#if moveMenuOpen}
                <div class="my-1 max-h-72 overflow-auto border-y border-border bg-muted/25 py-1">
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
                </div>
              {/if}
              <button
                class={menuItemClass()}
                type="button"
                role="menuitem"
                onclick={() => {
                  htmlExportOpen = true;
                  closePageMenu();
                }}
              >
                <Download class="size-4" />
                <span>{t("notes.htmlExportOpen")}</span>
              </button>
              <button
                class={menuItemClass()}
                type="button"
                role="menuitem"
                onclick={() => {
                  agentBridgeExportOpen = true;
                  closePageMenu();
                }}
              >
                <GitBranch class="size-4" />
                <span>{t("notes.agentBridgeExportOpen")}</span>
              </button>
              <button class={menuItemClass()} type="button" role="menuitem" onclick={() => openPanelFromMenu("history")}>
                <History class="size-4" />
                <span>{t("notes.pageHistory")}</span>
              </button>
              <button class={menuItemClass()} type="button" role="menuitem" onclick={archiveCurrentPage}>
                <Archive class="size-4" />
                <span>{t("notes.archivePage")}</span>
              </button>
              <button class={menuItemClass(true)} type="button" role="menuitem" onclick={trashCurrentPage}>
                <Trash2 class="size-4" />
                <span>{t("notes.moveToTrash")}</span>
              </button>
            </div>
          {/if}
        </div>
      </div>

      {#if activePanel}
        <div
          class="absolute right-3 top-full z-50 w-[min(44rem,calc(100vw-1.5rem))] overflow-auto rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
          style="max-height: min(32rem, calc(100dvh - 7rem));"
          data-app-floating-surface
        >
          {#if activePanel === "links"}
            <div class="flex min-w-0 flex-col gap-2">
              <NotesBacklinks embedded />
              <NotesPageLinks embedded />
            </div>
          {:else if activePanel === "comments"}
            <NotesComments embedded />
          {:else if activePanel === "suggestions"}
            <NotesSuggestions embedded />
          {:else if activePanel === "history"}
            <NotesPageHistory embedded />
          {/if}
        </div>
      {/if}
    </div>

    <div bind:this={blockScrollViewport} class="min-h-0 flex-1 overflow-auto">
      {#if page.cover}
        <div class="h-28 overflow-hidden bg-muted sm:h-44">
          <NotesPageCover cover={page.cover} unavailableLabel={t("notes.pageCoverUnavailable")} />
        </div>
      {/if}

      <div class="mx-auto flex w-full max-w-208 flex-col px-4 pb-12 pt-8 sm:px-8">
        <div class="notes-page-title-surface group/title min-w-0 pb-5">
          <div class="-ml-1.5 mb-2 flex min-h-8 flex-wrap items-center gap-1.5 opacity-0 transition-opacity group-hover/title:opacity-100 group-focus-within/title:opacity-100">
            <div
              class="relative"
              use:dismissOnOutside={{ enabled: iconMenuAnchor === "title", onDismiss: closeIconMenu }}
            >
              <button
                class="inline-flex max-w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                type="button"
                aria-label={page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
                onclick={() => toggleIconMenu("title")}
              >
                <SmilePlus class="size-3.5" />
                <span class="truncate">{page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}</span>
              </button>
              {#if iconMenuAnchor === "title"}
                <NotesPageIconMenu
                  icon={page.icon}
                  onSelect={(icon) => {
                    iconMenuAnchor = null;
                    void notes.updatePageIcon(page.id, icon);
                  }}
                />
              {/if}
            </div>
            <div
              class="relative"
              use:dismissOnOutside={{ enabled: coverMenuOpen, onDismiss: closeCoverMenu }}
            >
              <button
                class="inline-flex max-w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                type="button"
                aria-label={page.cover ? t("notes.changePageCover") : t("notes.addPageCover")}
                onclick={toggleCoverMenu}
              >
                <ImagePlus class="size-3.5" />
                <span class="truncate">{page.cover ? t("notes.changePageCover") : t("notes.addPageCover")}</span>
              </button>
              {#if coverMenuOpen}
                <NotesPageCoverMenu
                  cover={page.cover}
                  onSelect={(cover) => {
                    coverMenuOpen = false;
                    void notes.updatePageCover(page.id, cover);
                  }}
                />
              {/if}
            </div>
            <button
              class="inline-flex max-w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
              type="button"
              aria-label={t("notes.addComment")}
              onclick={openPageDiscussion}
            >
              <MessageSquare class="size-3.5" />
              <span class="truncate">{t("notes.addComment")}</span>
            </button>
          </div>

          {#if page.icon}
            <div
              class="relative mb-3 inline-flex"
              use:dismissOnOutside={{ enabled: iconMenuAnchor === "page-icon", onDismiss: closeIconMenu }}
            >
              <button
                class="flex size-16 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                type="button"
                aria-label={t("notes.changePageIcon")}
                data-app-tooltip={t("notes.changePageIcon")}
                onclick={() => toggleIconMenu("page-icon")}
              >
                <NotesPageIcon icon={page.icon} size={48} class="shrink-0" />
                <span class="sr-only">{pageIconLabel}</span>
              </button>
              {#if iconMenuAnchor === "page-icon"}
                <NotesPageIconMenu
                  icon={page.icon}
                  onSelect={(icon) => {
                    iconMenuAnchor = null;
                    void notes.updatePageIcon(page.id, icon);
                  }}
                />
              {/if}
            </div>
          {/if}

          <input
            bind:this={titleInput}
            class="block w-full min-w-0 bg-transparent text-[2.5rem] font-bold leading-[1.2] text-foreground outline-none placeholder:text-muted-foreground"
            aria-label={t("notes.titleInput")}
            bind:value={titleDraft}
            placeholder={t("notes.titlePlaceholder")}
            oninput={handleTitleInput}
            onkeydown={handleTitleKeydown}
            onblur={() => {
              void saveTitle();
            }}
          />
        </div>

        <NotesBlockList
          items={notes.flatBlocks}
          pageId={page.id}
          {breadcrumbItems}
          {tableOfContentsItems}
          onSelectPage={(pageId) => {
            void notes.selectPage(pageId);
          }}
          onFocusBlock={(blockId) => {
            notes.focusBlock(blockId);
          }}
        />
      </div>
    </div>
  </section>

  {#if htmlExportOpen}
    <NotesHtmlExportDialog
      pageTitle={currentPageTitle}
      onExport={exportHtmlArchive}
      onCancel={() => {
        htmlExportOpen = false;
      }}
    />
  {/if}

  {#if agentBridgeExportOpen}
    <NotesAgentBridgeExportDialog
      pageTitle={currentPageTitle}
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
{:else}
  <div class="flex min-w-0 flex-1 items-center justify-center px-4 text-center text-[0.933333rem] text-muted-foreground">
    {t("notes.emptyEditor")}
  </div>
{/if}
