<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import Archive from "@lucide/svelte/icons/archive";
  import Copy from "@lucide/svelte/icons/copy";
  import Download from "@lucide/svelte/icons/download";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import FolderTree from "@lucide/svelte/icons/folder-tree";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import History from "@lucide/svelte/icons/history";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import Link2 from "@lucide/svelte/icons/link-2";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Pencil from "@lucide/svelte/icons/pencil";
  import PencilLine from "@lucide/svelte/icons/pencil-line";
  import SmilePlus from "@lucide/svelte/icons/smile-plus";
  import Star from "@lucide/svelte/icons/star";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import {
    notesPageIconAssetUrl,
    pickNotesPageIconImageFile,
    saveNotesPageIconImageDataUrl,
  } from "$lib/api/notes-page-icons";
  import type {
    IconPickerAsset,
    IconPickerUploadAdapter,
  } from "$lib/components/icon-picker/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { blockPlainText, isTextEditableBlock } from "$lib/notes/block-factory";
  import { notesBlockAnchorId } from "$lib/notes/block-link";
  import { notesEditorScrollTopForTarget } from "$lib/notes/editor-scroll";
  import {
    notesBackgroundPointerTargetsDocumentEnd,
    notesDocumentEndFocusIsCurrent,
  } from "$lib/notes/editor-focus";
  import { notesTextSelectionFromEditableRoot } from "$lib/notes/editor-selection";
  import {
    formatNotesActivityDate,
    formatNotesActivityTime,
  } from "$lib/notes/page-activity";
  import { notesLocalUserDisplayName } from "$lib/notes/local-user";
  import type { NotesPageOpenMode } from "$lib/notes/page-open-mode";
  import {
    notesCommentParentKey,
    openNotesCommentThreadCount,
    unreadNotesCommentThreadCount,
  } from "$lib/notes/comments";
  import { notesPageMoveTargets } from "$lib/notes/page-move";
  import {
    notesPageIconFromPickerValue,
    notesPageIconPickerValue,
  } from "$lib/notes/page-icon-picker";
  import {
    createNotesExternalPageIcon,
    createNotesLocalFilePageIcon,
    type NotesPageIconAssetMetadata,
  } from "$lib/notes/page-icon";
  import {
    notesFoldersForProject,
    notesPageFolderMoveTargets,
  } from "$lib/notes/navigation-tree";
  import { addNotesWorkspaceBreadcrumb, buildNotesPageBreadcrumb } from "$lib/notes/page-breadcrumb";
  import { notesPageTitle } from "$lib/notes/page-title";
  import {
    notesPageProjectId,
    notesPagesForProject,
  } from "$lib/notes/project-membership";
  import { openNotesSuggestionCount } from "$lib/notes/suggestions";
  import { buildNotesTableOfContents } from "$lib/notes/table-of-contents";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
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
  import NotesBlockList from "./NotesBlockList.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import NotesPeekModeIcon from "./NotesPeekModeIcon.svelte";
  import {
    loadNotesEditorPanel,
    retryNotesEditorPanel,
    type LoadedNotesEditorPanel,
    type NotesEditorPanelKind,
  } from "./notes-editor-component-registry";

  type NotesEditorPanel = "links" | "comments" | "suggestions";

  const FOCUSED_BLOCK_SCROLL_PADDING_PX = 16;

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
  const preferences = getPreferences();
  const projects = getProjects();
  const localization = getLocalization();
  const { t } = localization;

  let titleDraft = $state("");
  let titleInput: HTMLInputElement | null = $state(null);
  let coverMenuOpen = $state(false);
  let pageMenuOpen = $state(false);
  let moveMenuOpen = $state(false);
  let folderMoveMenuOpen = $state(false);
  let activityPanelOpen = $state(false);
  let activePanel = $state<NotesEditorPanel | null>(null);
  let htmlExportOpen = $state(false);
  let agentBridgeExportOpen = $state(false);
  let pageHistoryModalOpen = $state(false);
  let pendingArchivePage = $state<NotesPage | null>(null);
  let pendingTrashPage = $state<NotesPage | null>(null);
  let requestedIconPicker = $state<"action" | "icon" | null>(null);
  let actionIconPickerTrigger: HTMLButtonElement | null = $state(null);
  let pageIconPickerTrigger: HTMLButtonElement | null = $state(null);
  let panelLoadStates = $state<Partial<Record<
    NotesEditorPanelKind,
    LazyComponentLoadState<NotesEditorPanelKind, LoadedNotesEditorPanel>
  >>>({});
  let blockScrollViewport: HTMLDivElement | null = $state(null);
  let activityNowMs = $state(Date.now());
  let activityPanelHideTimer: number | null = null;
  let lastTitlePageId = "";
  let lastHandledTitleFocusRequestId = 0;

  const page = $derived(notes.loadedPage);
  const editablePageTitle = $derived(page ? notesPageTitle(page, "") : "");
  const currentPageTitle = $derived(page ? notesPageTitle(page, t("notes.untitled")) : t("notes.untitled"));
  const pageIconLabel = $derived(pageIconScreenReaderText(page?.icon ?? null));
  const effectiveProjectId = $derived(page ? notesPageProjectId(page) ?? projectId : projectId);
  const projectPages = $derived(notesPagesForProject(
    [...new Map([...notes.allPages, ...notes.linkResolutionPages].map((item) => [item.id, item])).values()],
    effectiveProjectId,
  ));
  const projectFolders = $derived(notesFoldersForProject(notes.folders, effectiveProjectId));
  const moveTargets = $derived(page
    ? notesPageMoveTargets(
        projectPages,
        page.id,
        t("notes.workspace"),
        (candidate) => notesPageTitle(candidate, t("notes.untitled")),
        notes.recentPageIds,
      )
    : []);
  const folderMoveTargets = $derived(page
    ? notesPageFolderMoveTargets(
        projectPages,
        projectFolders,
        page.id,
        t("notes.projectRoot"),
        (candidate) => notesPageTitle(candidate, t("notes.untitled")),
        notes.recentPageIds,
      ).filter((target) => target.kind !== "page")
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
  const activityNow = $derived(new Date(activityNowMs));
  const activityAuthorName = $derived(notesLocalUserDisplayName(preferences.profileDisplayName));
  const activityTimeLabels = $derived({
    justNow: t("notes.metadataJustNow"),
    minutesAgo: (minutes: number) => t("notes.metadataMinutesAgo", minutes),
    hoursAgo: (hours: number) => t("notes.metadataHoursAgo", hours),
  });
  const editedDateLabel = $derived(page
    ? formatNotesActivityTime(page.last_edited_time, {
        locale: localization.locale,
        now: activityNow,
        labels: activityTimeLabels,
      })
    : "");
  const createdDateLabel = $derived(page
    ? formatNotesActivityDate(page.created_time, localization.locale, activityNow)
    : "");
  const activityPanelId = $derived(page ? `notes-activity-panel-${page.id}` : undefined);
  const editedMetadataLabel = $derived(t("notes.metadataEdited", editedDateLabel));
  const activityEditedByLabel = $derived(t("notes.activityEditedBy", activityAuthorName));
  const activityCreatedByLabel = $derived(t("notes.activityCreatedBy", activityAuthorName));
  const activityPanelLabel = $derived(
    page ? `${activityEditedByLabel}, ${editedDateLabel}` : t("notes.activity"),
  );
  const openCommentCount = $derived(openNotesCommentThreadCount(notes.commentThreads));
  const unreadCommentCount = $derived(unreadNotesCommentThreadCount(notes.commentThreads));
  const openSuggestionCount = $derived(openNotesSuggestionCount(notes.suggestions));
  const linksBadgeCount = $derived(notes.backlinks.length + notes.pageAliases.length + notes.unresolvedLinks.length);
  const commentsBadgeCount = $derived(unreadCommentCount > 0 ? unreadCommentCount : openCommentCount);
  const peekMode = $derived(openMode !== "full");

  function requestEditorPanel(kind: NotesEditorPanelKind, retry = false): void {
    const current = panelLoadStates[kind] ?? null;
    if (!retry && current?.key === kind) return;
    const loadingState = beginLazyComponentLoad(current, kind);
    panelLoadStates = { ...panelLoadStates, [kind]: loadingState };
    const request = retry ? retryNotesEditorPanel(kind) : loadNotesEditorPanel(kind);
    void request.then((component) => {
      const latest = panelLoadStates[kind];
      if (!latest) return;
      panelLoadStates = {
        ...panelLoadStates,
        [kind]: resolveLazyComponentLoad(latest, kind, loadingState.requestId, component),
      };
    }).catch((error: unknown) => {
      const latest = panelLoadStates[kind];
      if (!latest) return;
      panelLoadStates = {
        ...panelLoadStates,
        [kind]: rejectLazyComponentLoad(latest, kind, loadingState.requestId, error),
      };
      console.error(`load Notes ${kind} panel failed`, error);
    });
  }

  function openIconPicker(kind: "action" | "icon"): void {
    requestedIconPicker = kind;
    requestEditorPanel("icon-picker", panelLoadStates["icon-picker"]?.status === "failed");
  }

  $effect(() => {
    if (activePanel === "links") {
      requestEditorPanel("backlinks");
      requestEditorPanel("page-links");
    } else if (activePanel === "comments") {
      requestEditorPanel("comments");
    } else if (activePanel === "suggestions") {
      requestEditorPanel("suggestions");
    }
    if (moveMenuOpen || folderMoveMenuOpen) requestEditorPanel("destination-picker");
    if (coverMenuOpen) requestEditorPanel("cover-menu");
    if (page?.cover) requestEditorPanel("page-cover");
    if (htmlExportOpen) requestEditorPanel("html-export");
    if (agentBridgeExportOpen) requestEditorPanel("agent-export");
    if (pageHistoryModalOpen) requestEditorPanel("page-history");
    if (pendingArchivePage || pendingTrashPage) requestEditorPanel("confirm-dialog");
    if (requestedIconPicker && panelLoadStates["icon-picker"]?.status === "ready") {
      const kind = requestedIconPicker;
      requestedIconPicker = null;
      void tick().then(() => {
        (kind === "action" ? actionIconPickerTrigger : pageIconPickerTrigger)?.click();
      });
    }
  });

  $effect(() => {
    if (!page || page.id === lastTitlePageId) return;
    if (lastTitlePageId) notes.clearPageTitleDraft(lastTitlePageId);
    lastTitlePageId = page.id;
    notes.clearPageTitleDraft(page.id);
    titleDraft = editablePageTitle;
    activePanel = null;
    pageMenuOpen = false;
    moveMenuOpen = false;
    folderMoveMenuOpen = false;
    activityPanelOpen = false;
    coverMenuOpen = false;
    pageHistoryModalOpen = false;
  });

  function handleBlockViewportPointerDown(event: PointerEvent): void {
    if (event.button !== 0) return;
    const viewport = blockScrollViewport;
    const target = event.target;
    if (!viewport || !(target instanceof Element)) return;
    const rows = Array.from(
      viewport.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]"),
    );
    const lastRow = rows.at(-1);
    if (!lastRow) return;
    const clickedRow = target.closest<HTMLElement>("[data-notes-selectable-block-id]");
    if (!notesBackgroundPointerTargetsDocumentEnd({
      pointerY: event.clientY,
      lastRowBottom: lastRow.getBoundingClientRect().bottom,
      targetInsideRow: clickedRow !== null && viewport.contains(clickedRow),
    })) {
      return;
    }
    const blockId = lastRow.dataset.notesSelectableBlockId;
    if (!blockId) return;
    const block = notes.blockById(blockId);
    if (!block) return;
    event.preventDefault();
    if (isTextEditableBlock(block.type)) {
      const end = blockPlainText(block).length;
      const editor = Array.from(
        lastRow.querySelectorAll<HTMLElement>(
          "[contenteditable='true'][role='textbox'][data-notes-block-id]",
        ),
      ).find((candidate) => candidate.dataset.notesBlockId === blockId) ?? null;
      const selection = editor ? notesTextSelectionFromEditableRoot(editor) : null;
      const activeBlockId = document.activeElement instanceof HTMLElement
        ? document.activeElement.dataset.notesBlockId ?? null
        : null;
      if (notesDocumentEndFocusIsCurrent({
        activeBlockId,
        lastBlockId: blockId,
        selection,
        textLength: end,
      })) {
        return;
      }
      notes.focusBlock(blockId, { start: end, end });
      return;
    }
    notes.focusBlock(blockId);
  }

  function documentEndPointer(node: HTMLDivElement): { destroy: () => void } {
    node.addEventListener("pointerdown", handleBlockViewportPointerDown);
    return {
      destroy() {
        node.removeEventListener("pointerdown", handleBlockViewportPointerDown);
      },
    };
  }

  onMount(() => {
    activityNowMs = Date.now();
    const intervalId = window.setInterval(() => {
      activityNowMs = Date.now();
    }, 30_000);
    return () => window.clearInterval(intervalId);
  });

  $effect(() => {
    const titleFocusRequestId = notes.titleFocusRequestId;
    if (titleFocusRequestId === lastHandledTitleFocusRequestId) return;
    if (!page || notes.titleFocusPageId !== page.id) return;
    lastHandledTitleFocusRequestId = titleFocusRequestId;
    focusTitleInput(false);
  });

  $effect(() => {
    const focusRequestId = notes.focusRequestId;
    const focusSelection = notes.focusSelection;
    const blockId = notes.focusBlockId;
    if (!blockId) return;
    void tick().then(() => {
      const viewport = blockScrollViewport;
      if (!viewport || notes.focusRequestId !== focusRequestId) return;
      const anchor = viewport.querySelector<HTMLElement>(
        `#${CSS.escape(notesBlockAnchorId(blockId))}`,
      );
      if (!anchor) return;
      const viewportRect = viewport.getBoundingClientRect();
      const targetRect = anchor.getBoundingClientRect();
      const nextScrollTop = notesEditorScrollTopForTarget({
        scrollTop: viewport.scrollTop,
        maxScrollTop: viewport.scrollHeight - viewport.clientHeight,
        viewportTop: viewportRect.top,
        viewportBottom: viewportRect.bottom,
        targetTop: targetRect.top,
        targetBottom: targetRect.bottom,
        padding: FOCUSED_BLOCK_SCROLL_PADDING_PX,
        alignment: focusSelection ? "nearest" : "center",
      });
      if (nextScrollTop !== viewport.scrollTop) viewport.scrollTop = nextScrollTop;
    });
  });

  onDestroy(() => {
    clearActivityPanelHideTimer();
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

  function togglePanel(panel: NotesEditorPanel): void {
    const nextPanel = activePanel === panel ? null : panel;
    activePanel = nextPanel;
    if (nextPanel) {
      void notes.ensureOptionalSubsystem(nextPanel).catch((error) => {
        console.error(`load notes ${nextPanel} panel failed`, error);
      });
    }
    pageMenuOpen = false;
    moveMenuOpen = false;
    folderMoveMenuOpen = false;
  }

  function openPageHistory(): void {
    pageHistoryModalOpen = true;
    void notes.ensureOptionalSubsystem("page-history").catch((error) => {
      console.error("load notes page history failed", error);
    });
    activePanel = null;
    pageMenuOpen = false;
    moveMenuOpen = false;
  }

  function closePageMenu(): void {
    pageMenuOpen = false;
    moveMenuOpen = false;
    folderMoveMenuOpen = false;
  }

  function closeActionPanel(): void {
    activePanel = null;
  }

  function showActivityPanel(): void {
    clearActivityPanelHideTimer();
    activityPanelOpen = true;
  }

  function hideActivityPanel(): void {
    clearActivityPanelHideTimer();
    activityPanelOpen = false;
  }

  function clearActivityPanelHideTimer(): void {
    if (activityPanelHideTimer === null) return;
    window.clearTimeout(activityPanelHideTimer);
    activityPanelHideTimer = null;
  }

  function scheduleHideActivityPanel(): void {
    clearActivityPanelHideTimer();
    activityPanelHideTimer = window.setTimeout(() => {
      activityPanelOpen = false;
      activityPanelHideTimer = null;
    }, 180);
  }

  function handleActivityFocusOut(event: FocusEvent): void {
    const currentTarget = event.currentTarget;
    if (!(currentTarget instanceof HTMLElement)) return;
    const nextTarget = event.relatedTarget;
    if (nextTarget instanceof Node && currentTarget.contains(nextTarget)) return;
    hideActivityPanel();
  }

  function toggleCoverMenu(): void {
    const nextOpen = !coverMenuOpen;
    coverMenuOpen = nextOpen;
    if (nextOpen) {
      if (panelLoadStates["cover-menu"]?.status === "failed") {
        requestEditorPanel("cover-menu", true);
      }
      closePageMenu();
      closeActionPanel();
    }
  }

  function closeCoverMenu(): void {
    coverMenuOpen = false;
  }

  function prepareIconPicker(toggle: () => void): void {
    coverMenuOpen = false;
    closePageMenu();
    closeActionPanel();
    toggle();
  }

  function notesIconAssetMetadata(asset: IconPickerAsset): NotesPageIconAssetMetadata {
    if (!asset.contentType || asset.byteSize === undefined || !asset.sha256) {
      throw new Error(t("notes.pageIconUploadFailed"));
    }
    return {
      relativePath: asset.relativePath,
      originalName: asset.originalName,
      contentType: asset.contentType,
      byteSize: asset.byteSize,
      sha256: asset.sha256,
    };
  }

  function updatePageIconFromPicker(value: string): void {
    if (!page) return;
    const icon = notesPageIconFromPickerValue(value, projects.customEmojis);
    void notes.updatePageIcon(page.id, icon);
  }

  const notesIconUploadAdapter: IconPickerUploadAdapter = {
    pickImageFile: pickNotesPageIconImageFile,
    saveImageDataUrl: saveNotesPageIconImageDataUrl,
    assetUrl: (asset) => notesPageIconAssetUrl(asset.relativePath),
    selectAsset: (asset) => {
      if (!page) return;
      return notes.updatePageIcon(page.id, createNotesLocalFilePageIcon(notesIconAssetMetadata(asset)));
    },
    selectPickedAssetImmediately: true,
    selectExternalUrl: (url) => {
      if (!page) return;
      return notes.updatePageIcon(page.id, createNotesExternalPageIcon(url));
    },
  };

  function selectOpenMode(mode: NotesPageOpenMode): void {
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

  function moveToFolderTarget(targetKey: string): void {
    if (!page) return;
    const target = folderMoveTargets.find((candidate) => candidate.key === targetKey);
    if (!target) return;
    closePageMenu();
    void notes.movePageToFolder(page.id, target.folderId);
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
    void notes.ensureOptionalSubsystem("comments").catch((error) => {
      console.error("load notes comments failed", error);
    });
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
  <section class="flex h-full w-full min-w-0 flex-1 flex-col overflow-hidden">
    <div
      class="relative z-40 shrink-0"
      style="background-color: var(--cal-bg);"
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
              <NotesPeekModeIcon mode="full" class="size-4" />
            </button>
            <button
              type="button"
              class={actionButtonClass()}
              aria-label={openMode === "side" ? t("notes.centerPeek") : t("notes.sidePeek")}
              data-app-tooltip={openMode === "side" ? t("notes.centerPeek") : t("notes.sidePeek")}
              onclick={() => selectOpenMode(openMode === "side" ? "center" : "side")}
            >
              {#if openMode === "side"}
                <NotesPeekModeIcon mode="center" class="size-4" />
              {:else}
                <NotesPeekModeIcon mode="side" class="size-4" />
              {/if}
            </button>
          {/if}
        </div>
        <div class="min-w-0 flex-1"></div>
        <div
          class="relative hidden shrink-0 min-[560px]:block"
          role="group"
          aria-label={t("notes.activity")}
          onmouseenter={showActivityPanel}
          onmouseleave={scheduleHideActivityPanel}
          onfocusin={showActivityPanel}
          onfocusout={handleActivityFocusOut}
        >
          <button
            type="button"
            class="flex h-7 max-w-40 items-center rounded-md px-2 text-[0.8rem] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus:bg-accent focus:text-foreground"
            aria-label={activityPanelLabel}
            aria-expanded={activityPanelOpen}
            aria-controls={activityPanelId}
          >
            <span class="truncate">{editedMetadataLabel}</span>
          </button>
          {#if activityPanelOpen}
            <div class="absolute right-0 top-7 h-1 w-80" aria-hidden="true"></div>
            <div
              id={activityPanelId}
              class="absolute right-0 top-8 z-50 w-80 overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-md"
              role="dialog"
              aria-label={t("notes.activity")}
              data-app-floating-surface
            >
              <div class="border-b border-border px-4 py-3 text-[0.8rem] font-medium text-muted-foreground">
                {t("notes.activity")}
              </div>
              <div class="flex flex-col gap-3 px-4 py-3 text-[0.8rem]">
                <div class="flex min-w-0 items-center justify-between gap-4">
                  <span class="min-w-0 truncate text-foreground">{activityEditedByLabel}</span>
                  <span class="shrink-0 text-muted-foreground">{editedDateLabel}</span>
                </div>
                <div class="flex min-w-0 items-center justify-between gap-4">
                  <span class="min-w-0 truncate text-foreground">{activityCreatedByLabel}</span>
                  <span class="shrink-0 text-muted-foreground">{createdDateLabel}</span>
                </div>
              </div>
            </div>
          {/if}
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
              if (!pageMenuOpen) {
                moveMenuOpen = false;
                folderMoveMenuOpen = false;
              }
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
                  if (moveMenuOpen) {
                    void notes.ensureOptionalSubsystem("destinations").catch((error) => {
                      console.error("load notes move destinations failed", error);
                    });
                  }
                  folderMoveMenuOpen = false;
                }}
              >
                <FolderInput class="size-4" />
                <span>{t("notes.movePageTo")}</span>
              </button>
              {#if moveMenuOpen}
                <div class="my-1 max-h-72 overflow-auto border-y border-border bg-muted/25 py-1">
                  {#if panelLoadStates["destination-picker"]?.status === "ready" && panelLoadStates["destination-picker"].component.kind === "destination-picker"}
                    {@const NotesDestinationPickerList = panelLoadStates["destination-picker"].component.component}
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
                  {:else if panelLoadStates["destination-picker"]?.status === "failed"}
                    <button class="m-2 min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestEditorPanel("destination-picker", true)}>{t("common.retry")}</button>
                  {:else}
                    <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
                  {/if}
                </div>
              {/if}
              {#if folderMoveTargets.length > 0}
                <button
                  class={menuItemClass()}
                  type="button"
                  role="menuitem"
                  aria-expanded={folderMoveMenuOpen}
                  onclick={() => {
                    folderMoveMenuOpen = !folderMoveMenuOpen;
                    if (folderMoveMenuOpen) {
                      void notes.ensureOptionalSubsystem("destinations").catch((error) => {
                        console.error("load notes folder destinations failed", error);
                      });
                    }
                    moveMenuOpen = false;
                  }}
                >
                  <FolderTree class="size-4" />
                  <span>{t("notes.movePageToFolder")}</span>
                </button>
                {#if folderMoveMenuOpen}
                  <div class="my-1 max-h-72 overflow-auto border-y border-border bg-muted/25 py-1">
                    {#if panelLoadStates["destination-picker"]?.status === "ready" && panelLoadStates["destination-picker"].component.kind === "destination-picker"}
                      {@const NotesDestinationPickerList = panelLoadStates["destination-picker"].component.component}
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
                    {:else if panelLoadStates["destination-picker"]?.status === "failed"}
                      <button class="m-2 min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestEditorPanel("destination-picker", true)}>{t("common.retry")}</button>
                    {:else}
                      <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
                    {/if}
                  </div>
                {/if}
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
              <button class={menuItemClass()} type="button" role="menuitem" onclick={openPageHistory}>
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
              {#if panelLoadStates.backlinks?.status === "ready" && panelLoadStates.backlinks.component.kind === "backlinks"}
                {@const NotesBacklinks = panelLoadStates.backlinks.component.component}
                <NotesBacklinks embedded />
              {/if}
              {#if panelLoadStates["page-links"]?.status === "ready" && panelLoadStates["page-links"].component.kind === "page-links"}
                {@const NotesPageLinks = panelLoadStates["page-links"].component.component}
                <NotesPageLinks embedded />
              {/if}
              {#if panelLoadStates.backlinks?.status === "failed" || panelLoadStates["page-links"]?.status === "failed"}
                <button class="min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => { if (panelLoadStates.backlinks?.status === "failed") requestEditorPanel("backlinks", true); if (panelLoadStates["page-links"]?.status === "failed") requestEditorPanel("page-links", true); }}>{t("common.retry")}</button>
              {/if}
            </div>
          {:else if activePanel === "comments"}
            {#if panelLoadStates.comments?.status === "ready" && panelLoadStates.comments.component.kind === "comments"}
              {@const NotesComments = panelLoadStates.comments.component.component}
              <NotesComments embedded />
            {:else if panelLoadStates.comments?.status === "failed"}
              <button class="min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestEditorPanel("comments", true)}>{t("common.retry")}</button>
            {/if}
          {:else if activePanel === "suggestions"}
            {#if panelLoadStates.suggestions?.status === "ready" && panelLoadStates.suggestions.component.kind === "suggestions"}
              {@const NotesSuggestions = panelLoadStates.suggestions.component.component}
              <NotesSuggestions embedded />
            {:else if panelLoadStates.suggestions?.status === "failed"}
              <button class="min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestEditorPanel("suggestions", true)}>{t("common.retry")}</button>
            {/if}
          {/if}
        </div>
      {/if}
    </div>

    <div
      bind:this={blockScrollViewport}
      class="min-h-0 flex-1 overflow-auto"
      use:documentEndPointer
    >
      {#if page.cover}
        <div class="h-28 overflow-hidden bg-muted sm:h-44">
          {#if panelLoadStates["page-cover"]?.status === "ready" && panelLoadStates["page-cover"].component.kind === "page-cover"}
            {@const NotesPageCover = panelLoadStates["page-cover"].component.component}
            <NotesPageCover cover={page.cover} unavailableLabel={t("notes.pageCoverUnavailable")} />
          {:else if panelLoadStates["page-cover"]?.status === "failed"}
            <button class="m-2 min-h-8 rounded-md border border-border bg-popover px-2 text-[0.8rem]" type="button" onclick={() => requestEditorPanel("page-cover", true)}>{t("common.retry")}</button>
          {/if}
        </div>
      {/if}

      <div
        class={cn(
          "mx-auto flex w-full max-w-208 flex-col pb-12 pt-8",
          openMode === "side" ? "pl-16 pr-4 sm:pr-8" : "px-4 sm:px-8",
        )}
      >
        <div class="notes-page-title-surface group/title min-w-0 pb-5">
          <div class="notes-page-title-actions -ml-1.5 mb-2 flex min-h-8 flex-wrap items-center gap-1.5 opacity-0 transition-opacity group-hover/title:opacity-100 group-focus-within/title:opacity-100">
            {#if panelLoadStates["icon-picker"]?.status === "ready" && panelLoadStates["icon-picker"].component.kind === "icon-picker"}
              {@const IconPicker = panelLoadStates["icon-picker"].component.component}
              <IconPicker
                value={notesPageIconPickerValue(page.icon)}
                ariaLabel={page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
                uploadAdapter={notesIconUploadAdapter}
                onChange={updatePageIconFromPicker}
              >
                {#snippet trigger({ open, toggle })}
                  <button
                    bind:this={actionIconPickerTrigger}
                    class={`inline-flex max-w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground ${open ? "bg-accent text-foreground" : ""}`}
                    type="button"
                    aria-label={page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
                    data-notes-icon-picker-open={open ? "true" : undefined}
                    onclick={() => prepareIconPicker(toggle)}
                  >
                    <SmilePlus class="size-3.5" />
                    <span class="truncate">{page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}</span>
                  </button>
                {/snippet}
              </IconPicker>
            {:else}
              <button
                class="inline-flex max-w-full items-center gap-1.5 rounded-md px-1.5 py-1 text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                type="button"
                aria-label={page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}
                onclick={() => openIconPicker("action")}
              >
                <SmilePlus class="size-3.5" />
                <span class="truncate">{page.icon ? t("notes.changePageIcon") : t("notes.addPageIcon")}</span>
              </button>
            {/if}
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
                {#if panelLoadStates["cover-menu"]?.status === "ready" && panelLoadStates["cover-menu"].component.kind === "cover-menu"}
                  {@const NotesPageCoverMenu = panelLoadStates["cover-menu"].component.component}
                  <NotesPageCoverMenu
                  cover={page.cover}
                  onSelect={(cover) => {
                    coverMenuOpen = false;
                    void notes.updatePageCover(page.id, cover);
                  }}
                  />
                {:else if panelLoadStates["cover-menu"]?.status === "failed"}
                  <button class="min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestEditorPanel("cover-menu", true)}>{t("common.retry")}</button>
                {/if}
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
            <div class="mb-3 inline-flex">
              {#if panelLoadStates["icon-picker"]?.status === "ready" && panelLoadStates["icon-picker"].component.kind === "icon-picker"}
                {@const IconPicker = panelLoadStates["icon-picker"].component.component}
                <IconPicker
                  value={notesPageIconPickerValue(page.icon)}
                  ariaLabel={t("notes.changePageIcon")}
                  uploadAdapter={notesIconUploadAdapter}
                  onChange={updatePageIconFromPicker}
                >
                  {#snippet trigger({ open, toggle })}
                    <button
                      bind:this={pageIconPickerTrigger}
                      class={`flex size-16 items-center justify-center rounded-md text-foreground hover:bg-accent ${open ? "bg-accent" : ""}`}
                      type="button"
                      aria-label={t("notes.changePageIcon")}
                      data-app-tooltip={t("notes.changePageIcon")}
                      onclick={() => prepareIconPicker(toggle)}
                    >
                      <NotesPageIcon icon={page.icon} size={48} class="shrink-0" />
                      <span class="sr-only">{pageIconLabel}</span>
                    </button>
                  {/snippet}
                </IconPicker>
              {:else}
                <button
                  class="flex size-16 items-center justify-center rounded-md text-foreground hover:bg-accent"
                  type="button"
                  aria-label={t("notes.changePageIcon")}
                  data-app-tooltip={t("notes.changePageIcon")}
                  onclick={() => openIconPicker("icon")}
                >
                  <NotesPageIcon icon={page.icon} size={48} class="shrink-0" />
                  <span class="sr-only">{pageIconLabel}</span>
                </button>
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
    {#if panelLoadStates["html-export"]?.status === "ready" && panelLoadStates["html-export"].component.kind === "html-export"}
      {@const NotesHtmlExportDialog = panelLoadStates["html-export"].component.component}
      <NotesHtmlExportDialog
      pageTitle={currentPageTitle}
      onExport={exportHtmlArchive}
      onCancel={() => {
        htmlExportOpen = false;
      }}
      />
    {:else if panelLoadStates["html-export"]?.status === "failed"}
      <button class="fixed inset-0 z-50 m-auto h-10 rounded-md border border-border bg-popover px-3" type="button" onclick={() => requestEditorPanel("html-export", true)}>{t("common.retry")}</button>
    {/if}
  {/if}

  {#if agentBridgeExportOpen}
    {#if panelLoadStates["agent-export"]?.status === "ready" && panelLoadStates["agent-export"].component.kind === "agent-export"}
      {@const NotesAgentBridgeExportDialog = panelLoadStates["agent-export"].component.component}
      <NotesAgentBridgeExportDialog
      pageTitle={currentPageTitle}
      onExport={exportAgentBridge}
      onCancel={() => {
        agentBridgeExportOpen = false;
      }}
      />
    {:else if panelLoadStates["agent-export"]?.status === "failed"}
      <button class="fixed inset-0 z-50 m-auto h-10 rounded-md border border-border bg-popover px-3" type="button" onclick={() => requestEditorPanel("agent-export", true)}>{t("common.retry")}</button>
    {/if}
  {/if}

  {#if pageHistoryModalOpen}
    {#if panelLoadStates["page-history"]?.status === "ready" && panelLoadStates["page-history"].component.kind === "page-history"}
      {@const NotesPageVersionHistoryModal = panelLoadStates["page-history"].component.component}
      <NotesPageVersionHistoryModal
      pageId={page.id}
      onClose={() => {
        pageHistoryModalOpen = false;
      }}
      />
    {:else if panelLoadStates["page-history"]?.status === "failed"}
      <button class="fixed inset-0 z-50 m-auto h-10 rounded-md border border-border bg-popover px-3" type="button" onclick={() => requestEditorPanel("page-history", true)}>{t("common.retry")}</button>
    {/if}
  {/if}

  {#if pendingArchivePage}
    {#if panelLoadStates["confirm-dialog"]?.status === "ready" && panelLoadStates["confirm-dialog"].component.kind === "confirm-dialog"}
      {@const ConfirmDialog = panelLoadStates["confirm-dialog"].component.component}
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
    {:else if panelLoadStates["confirm-dialog"]?.status === "failed"}
      <button class="fixed inset-0 z-50 m-auto h-10 rounded-md border border-border bg-popover px-3" type="button" onclick={() => requestEditorPanel("confirm-dialog", true)}>{t("common.retry")}</button>
    {/if}
  {/if}

  {#if pendingTrashPage}
    {#if panelLoadStates["confirm-dialog"]?.status === "ready" && panelLoadStates["confirm-dialog"].component.kind === "confirm-dialog"}
      {@const ConfirmDialog = panelLoadStates["confirm-dialog"].component.component}
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
    {:else if panelLoadStates["confirm-dialog"]?.status === "failed"}
      <button class="fixed inset-0 z-50 m-auto h-10 rounded-md border border-border bg-popover px-3" type="button" onclick={() => requestEditorPanel("confirm-dialog", true)}>{t("common.retry")}</button>
    {/if}
  {/if}
{:else}
  <div class="flex min-w-0 flex-1 items-center justify-center px-4 text-center text-[0.933333rem] text-muted-foreground">
    {t("notes.emptyEditor")}
  </div>
{/if}

<style>
  .notes-page-title-actions:has([data-notes-icon-picker-open="true"]) {
    opacity: 1;
  }
</style>
