<script lang="ts">
  import { tick } from "svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    listNotesDataSources,
    listNotesDataSourceRowPages,
  } from "$lib/api/notes";
  import { getCalendar } from "$lib/stores/calendar.svelte";
  import { getMusicPlayer } from "$lib/stores/music-player.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getPomodoro } from "$lib/stores/pomodoro.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import type { CalendarEvent } from "$lib/components/calendar/types";
  import { sourceDisplayLabel, type MusicSource } from "$lib/music/sources";
  import { buildNotesBlockLink, buildNotesPageLink } from "$lib/notes/block-link";
  import {
    notesBlockSelectionAfterClick,
    notesBlockSelectionAfterKeyboard,
    notesBlockSelectionContains,
    notesBlockSelectionForBlock,
    notesBlockSelectionPrunedToVisible,
    notesBlockSelectionRange,
    normalizeNotesSelectableBlockIds,
  } from "$lib/notes/block-selection";
  import type { NotesBlockSelectionState } from "$lib/notes/block-selection";
  import {
    notesSelectionPlainText,
    notesSelectionRootBlockIds,
    notesSelectionSubtreeIds,
    planNotesSelectionMoveWithinSiblings,
  } from "$lib/notes/block-selection-operations";
  import { notesLocalUserDisplayName } from "$lib/notes/local-user";
  import { notesPageIconText } from "$lib/notes/page-icon";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesTemplateBlockStatus } from "$lib/notes/template-block";
  import { notesButtonBlockStatus } from "$lib/notes/button-block";
  import type { NotesUnsupportedConversionTarget } from "$lib/notes/unsupported";
  import {
    blockPlainText,
    isTextEditableBlock,
    type NotesHeadingBlockType,
  } from "$lib/notes/block-factory";
  import {
    notesAdjacentRenderedBlockId,
    notesBoundaryRenderedBlockId,
    notesCollapsedNavigationSelection,
    type NotesBlockNavigationBoundary,
    type NotesBlockNavigationDirection,
  } from "$lib/notes/block-navigation";
  import type { NotesBlockInsertRequest } from "$lib/notes/block-insertion";
  import {
    notesEditableOffsetFromDomPoint,
    notesPlainTextFromEditableRoot,
    notesTextSelectionFromEditableRoot,
    restoreNotesEditableSelection,
    type NotesTextSelection,
  } from "$lib/notes/editor-selection";
  import type {
    NotesDateMentionTarget,
    NotesNamedMentionTarget,
    NotesObjectMentionTarget,
    NotesPageMentionTarget,
    NotesRichTextAnnotationPatch,
  } from "$lib/notes/rich-text";
  import type { NotesKeyboardAction } from "$lib/notes/block-keyboard";
  import {
    notesMoveToPageTargets,
    type NotesMoveToPageTarget,
  } from "$lib/notes/block-move";
  import {
    NOTES_BLOCK_DRAG_MIME,
    planNotesBlockDrop,
    setActiveNotesBlockDragId,
    type NotesBlockDropIndicator,
    type NotesBlockDropIntent,
  } from "$lib/notes/block-drag";
  import {
    EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE,
    notesBlockHandleHoverStateAfterKeydown,
    notesBlockHandleHoverStateAfterPointerLeave,
    notesBlockHandleHoverStateAfterPointerMove,
  } from "$lib/notes/block-handle-hover";
  import type {
    NotesBlock,
    NotesBlockTreeItem,
    NotesBlockType,
    NotesButtonInsertPosition,
    NotesDataSource,
    NotesIcon,
    NotesPage,
    NotesPageBreadcrumbItem,
    NotesRichText,
    NotesTableOfContentsItem,
  } from "$lib/notes/types";
  import NotesBlockRow from "./NotesBlockRow.svelte";
  import {
    loadNotesAdvancedBlock,
    retryNotesAdvancedBlock,
    type LoadedNotesAdvancedBlock,
    type NotesAdvancedBlockFamily,
  } from "./notes-editor-component-registry";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ClipboardPaste from "@lucide/svelte/icons/clipboard-paste";
  import Copy from "@lucide/svelte/icons/copy";
  import CopyPlus from "@lucide/svelte/icons/copy-plus";
  import Scissors from "@lucide/svelte/icons/scissors";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  type NotesBlockSelectionClipboard = {
    mode: "copy" | "cut";
    pageId: string;
    rootBlockIds: string[];
    subtreeBlockIds: string[];
    plainText: string;
  };

  interface NotesEditableVisualLine {
    top: number;
    right: number;
    bottom: number;
    left: number;
  }

  interface DocumentWithCaretPositionFromPoint {
    caretPositionFromPoint?: (
      x: number,
      y: number,
      options?: CaretPositionFromPointOptions,
    ) => CaretPosition | null;
  }

  interface DocumentWithCaretRangeFromPoint {
    caretRangeFromPoint?: (x: number, y: number) => Range | null;
  }

  let {
    items,
    pageId,
    breadcrumbItems,
    tableOfContentsItems,
    onSelectPage,
    onFocusBlock,
  }: {
    items: NotesBlockTreeItem[];
    pageId: string;
    breadcrumbItems: NotesPageBreadcrumbItem[];
    tableOfContentsItems: NotesTableOfContentsItem[];
    onSelectPage: (pageId: string) => void;
    onFocusBlock: (blockId: string) => void;
  } = $props();

  const notes = getNotes();
  const calendar = getCalendar();
  const musicPlayer = getMusicPlayer();
  const pomodoro = getPomodoro();
  const projects = getProjects();
  const { t } = getLocalization();
  let blockListElement: HTMLDivElement | null = $state(null);
  let draggingBlockId = $state<string | null>(null);
  let dropTarget = $state<{ blockId: string; intent: NotesBlockDropIndicator } | null>(null);
  let blockSelection = $state<NotesBlockSelectionState | null>(null);
  let selectionDragAnchorBlockId = $state<string | null>(null);
  let selectionDragPointerId = $state<number | null>(null);
  let selectionClipboard = $state<NotesBlockSelectionClipboard | null>(null);
  let selectionBusy = $state(false);
  let selectionActionError = $state<string | null>(null);
  let blockHandleHoverState = $state(EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE);
  let openBlockHandleMenuId = $state<string | null>(null);
  let mentionDataSources = $state<NotesDataSource[]>([]);
  let mentionDataSourceRowPages = $state<NotesPage[]>([]);
  let structuralBlockLoadStates = $state<Partial<Record<
    NotesAdvancedBlockFamily,
    LazyComponentLoadState<NotesAdvancedBlockFamily, LoadedNotesAdvancedBlock>
  >>>({});
  let mentionDataSourceRequestId = 0;
  const selectedBlockCount = $derived(blockSelection?.selectedBlockIds.length ?? 0);
  const selectedRootBlockIds = $derived(
    blockSelection
      ? notesSelectionRootBlockIds(currentTreeState(), blockSelection.selectedBlockIds)
      : [],
  );
  const canMoveSelectionUp = $derived(
    !!planNotesSelectionMoveWithinSiblings(currentTreeState(), selectedRootBlockIds, "up"),
  );
  const canMoveSelectionDown = $derived(
    !!planNotesSelectionMoveWithinSiblings(currentTreeState(), selectedRootBlockIds, "down"),
  );
  const mentionTargets: NotesNamedMentionTarget[] = $derived(buildMentionTargets());

  function requestStructuralBlock(kind: "column-list" | "tab", retry = false): void {
    const current = structuralBlockLoadStates[kind] ?? null;
    if (!retry && current?.key === kind) return;
    const loadingState = beginLazyComponentLoad(current, kind);
    structuralBlockLoadStates = { ...structuralBlockLoadStates, [kind]: loadingState };
    const request = retry ? retryNotesAdvancedBlock(kind) : loadNotesAdvancedBlock(kind);
    void request.then((component) => {
      const latest = structuralBlockLoadStates[kind];
      if (!latest) return;
      structuralBlockLoadStates = {
        ...structuralBlockLoadStates,
        [kind]: resolveLazyComponentLoad(latest, kind, loadingState.requestId, component),
      };
    }).catch((error: unknown) => {
      const latest = structuralBlockLoadStates[kind];
      if (!latest) return;
      structuralBlockLoadStates = {
        ...structuralBlockLoadStates,
        [kind]: rejectLazyComponentLoad(latest, kind, loadingState.requestId, error),
      };
      console.error(`load Notes ${kind} block failed`, error);
    });
  }

  $effect(() => {
    if (items.some((item) => item.block.type === "column_list")) {
      requestStructuralBlock("column-list");
    }
    if (items.some((item) => item.block.type === "tab")) requestStructuralBlock("tab");
  });

  $effect(() => {
    void loadMentionDataSources();
  });

  $effect(() => {
    if (!projects.loaded && !projects.loading) {
      void projects.ensureLoaded()
        .then(() => {
          if (projects.selectedProjectId) {
            return projects.ensureProjectData(projects.selectedProjectId);
          }
          return undefined;
        })
        .catch((error) => console.warn("notes mention project targets failed", error));
    }
  });

  $effect(() => {
    if (!calendar.loaded && !calendar.windowLoadBusy) {
      void calendar.load()
        .catch((error) => console.warn("notes mention calendar targets failed", error));
    }
  });

  async function loadMentionDataSources(): Promise<void> {
    const requestId = ++mentionDataSourceRequestId;
    try {
      const dataSources = await listNotesDataSources();
      if (requestId !== mentionDataSourceRequestId) return;
      mentionDataSources = dataSources;
      const rowPageGroups = await Promise.all(
        dataSources.map(async (dataSource) => {
          try {
            return await listNotesDataSourceRowPages(dataSource.id);
          } catch (error) {
            console.warn("notes mention data source row targets failed", error);
            return [];
          }
        }),
      );
      if (requestId !== mentionDataSourceRequestId) return;
      mentionDataSourceRowPages = rowPageGroups.flat();
    } catch (error) {
      if (requestId !== mentionDataSourceRequestId) return;
      console.warn("notes mention data source targets failed", error);
      mentionDataSources = [];
      mentionDataSourceRowPages = [];
    }
  }

  function dataSourceById(dataSourceId: string | null): NotesDataSource | undefined {
    if (!dataSourceId) return undefined;
    return mentionDataSources.find((dataSource) => dataSource.id === dataSourceId);
  }

  function pageMentionSubtitle(page: NotesPage): string {
    if (page.parent.type === "data_source_id") {
      const dataSource = dataSourceById(page.parent.data_source_id);
      return dataSource
        ? t("notes.mentionTargetDataSourceRowIn", dataSource.title || t("notes.untitled"))
        : t("notes.mentionTargetDataSourceRow");
    }
    const parentPageId = page.parent.type === "page_id" ? page.parent.page_id : null;
    const parentPage = parentPageId
      ? notes.allPages.find((candidate) => candidate.id === parentPageId)
      : null;
    return parentPage
      ? notesPageTitle(parentPage, t("notes.untitled"))
      : t("notes.workspace");
  }

  function pageMentionTargets(): NotesPageMentionTarget[] {
    const seen = new Set<string>();
    const targets: NotesPageMentionTarget[] = [];
    for (const page of [...notes.allPages, ...mentionDataSourceRowPages]) {
      if (seen.has(page.id)) continue;
      seen.add(page.id);
      targets.push({
        kind: "page",
        id: page.id,
        title: notesPageTitle(page, t("notes.untitled")),
        subtitle: pageMentionSubtitle(page),
        iconText: notesPageIconText(page.icon),
      });
    }
    return targets;
  }

  function localUserMentionTargets(): NotesObjectMentionTarget[] {
    if (!notes.localUser) return [];
    return [{
      kind: "user",
      id: notes.localUser.id,
      title: notesLocalUserDisplayName(notes.localUser.display_name),
      subtitle: t("notes.mentionTargetLocalUser"),
    }];
  }

  function databaseMentionTargets(): NotesObjectMentionTarget[] {
    const seen = new Set<string>();
    const targets: NotesObjectMentionTarget[] = [];
    for (const dataSource of mentionDataSources) {
      const databaseId = dataSource.parent.database_id;
      if (seen.has(databaseId)) continue;
      seen.add(databaseId);
      targets.push({
        kind: "database",
        id: databaseId,
        title: dataSource.title || t("notes.untitled"),
        subtitle: t("notes.mentionTargetDatabase"),
        iconText: notesPageIconText(dataSource.icon),
      });
    }
    return targets;
  }

  function projectMentionTargets(): NotesObjectMentionTarget[] {
    return projects.projects
      .filter((project) => project.status === "active")
      .map((project) => ({
        kind: "project",
        id: project.id,
        title: project.name,
        subtitle: t("notes.mentionTargetProject"),
      }));
  }

  function projectTaskMentionTargets(): NotesObjectMentionTarget[] {
    return projects.tasks
      .filter((task) => !task.archivedAt)
      .map((task) => {
        const project = projects.projectById(task.projectId);
        return {
          kind: "project_task",
          id: task.id,
          title: task.title || t("notes.untitled"),
          subtitle: project
            ? t("notes.mentionTargetProjectTaskIn", project.name)
            : t("notes.mentionTargetProjectTask"),
        };
      });
  }

  function calendarEventMentionTargets(
    events: readonly CalendarEvent[],
  ): NotesObjectMentionTarget[] {
    return events
      .filter((event) => event.status !== "cancelled")
      .map((event) => ({
        kind: "calendar_event",
        id: event.recurringParentId ?? event.id,
        title: event.title || t("notes.untitled"),
        subtitle: t("notes.mentionTargetCalendarEvent"),
      }));
  }

  function pomodoroMentionTargets(): NotesObjectMentionTarget[] {
    if (!pomodoro.activeRunId) return [];
    return [{
      kind: "pomodoro_run",
      id: pomodoro.activeRunId,
      title: t("notes.mentionTargetPomodoroRun"),
      subtitle: pomodoro.formattedTime,
    }];
  }

  function musicMentionTargets(): NotesObjectMentionTarget[] {
    const sources: MusicSource[] = musicPlayer.currentSource
      ? [musicPlayer.currentSource, ...musicPlayer.queue]
      : [...musicPlayer.queue];
    const seen = new Set<string>();
    const targets: NotesObjectMentionTarget[] = [];
    for (const source of sources) {
      if (seen.has(source.identity)) continue;
      seen.add(source.identity);
      targets.push({
        kind: "music_item",
        id: source.identity,
        title: sourceDisplayLabel(source),
        subtitle: t("notes.mentionTargetMusicItem"),
      });
    }
    return targets;
  }

  function buildMentionTargets(): NotesNamedMentionTarget[] {
    return [
      ...localUserMentionTargets(),
      ...pageMentionTargets(),
      ...databaseMentionTargets(),
      ...projectMentionTargets(),
      ...projectTaskMentionTargets(),
      ...calendarEventMentionTargets(calendar.rawBlocks),
      ...pomodoroMentionTargets(),
      ...musicMentionTargets(),
    ];
  }

  $effect(() => {
    if (typeof window === "undefined" || selectionDragPointerId === null) return;
    const stopSelectionDrag = () => {
      selectionDragAnchorBlockId = null;
      selectionDragPointerId = null;
    };
    window.addEventListener("pointerup", stopSelectionDrag);
    window.addEventListener("pointercancel", stopSelectionDrag);
    return () => {
      window.removeEventListener("pointerup", stopSelectionDrag);
      window.removeEventListener("pointercancel", stopSelectionDrag);
    };
  });

  $effect(() => {
    const _pageId = pageId;
    const _itemCount = items.length;
    const _selection = blockSelection;
    void tick().then(() => {
      pruneAndSyncBlockSelection();
    });
  });

  function renderedSelectableBlockIds(): string[] {
    if (!blockListElement) return items.map((item) => item.block.id);
    return normalizeNotesSelectableBlockIds(
      Array.from(blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]"))
        .map((element) => element.dataset.notesSelectableBlockId ?? ""),
    );
  }

  function currentTreeState() {
    return {
      blocksById: notes.blocksById,
      childIdsByParentId: notes.childIdsByParentId,
    };
  }

  function templateStatusForBlock(blockId: string) {
    return notesTemplateBlockStatus(currentTreeState(), blockId);
  }

  function buttonStatusForBlock(blockId: string) {
    return notesButtonBlockStatus(currentTreeState(), blockId);
  }

  function selectableBlockRowFromEvent(event: Event): HTMLElement | null {
    const target = event.target;
    if (!(target instanceof Element) || !blockListElement) return null;
    const row = target.closest<HTMLElement>("[data-notes-selectable-block-id]");
    if (!row || !blockListElement.contains(row)) return null;
    return row;
  }

  function selectableBlockIdFromEvent(event: Event): string | null {
    return selectableBlockRowFromEvent(event)?.dataset.notesSelectableBlockId ?? null;
  }

  function eventTargetIsEditable(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    return target.closest("input, textarea, select, button, a, [contenteditable='true'], [role='textbox']")
      !== null;
  }

  function eventTargetIsSelectionZone(target: EventTarget | null): boolean {
    return target instanceof Element && target.closest("[data-notes-block-selection-zone]") !== null;
  }

  function syncBlockSelectionAttributes(): void {
    if (!blockListElement) return;
    for (const row of blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]")) {
      const blockId = row.dataset.notesSelectableBlockId ?? "";
      const selected = notesBlockSelectionContains(blockSelection, blockId);
      row.toggleAttribute("data-notes-block-selected", selected);
    }
  }

  function pruneAndSyncBlockSelection(): void {
    const pruned = notesBlockSelectionPrunedToVisible(renderedSelectableBlockIds(), blockSelection);
    if (!sameBlockSelection(blockSelection, pruned)) {
      blockSelection = pruned;
      return;
    }
    syncBlockSelectionAttributes();
  }

  function sameBlockSelection(
    left: NotesBlockSelectionState | null,
    right: NotesBlockSelectionState | null,
  ): boolean {
    if (left === right) return true;
    if (!left || !right) return false;
    return (
      left.anchorBlockId === right.anchorBlockId
      && left.focusBlockId === right.focusBlockId
      && left.selectedBlockIds.length === right.selectedBlockIds.length
      && left.selectedBlockIds.every((blockId, index) => right.selectedBlockIds[index] === blockId)
    );
  }

  function setBlockSelection(selection: NotesBlockSelectionState | null): void {
    blockSelection = selection;
    void tick().then(syncBlockSelectionAttributes);
  }

  function focusSelectedBlockRow(blockId: string, preventScroll = true): void {
    void tick().then(() => {
      selectableBlockRowFromBlockId(blockId)?.focus({ preventScroll });
    });
  }

  function selectableBlockRowFromBlockId(blockId: string): HTMLElement | null {
    if (!blockListElement) return null;
    return Array.from(blockListElement.querySelectorAll<HTMLElement>("[data-notes-selectable-block-id]"))
      .find((element) => element.dataset.notesSelectableBlockId === blockId) ?? null;
  }

  function textEditorForBlock(blockId: string): HTMLElement | null {
    return Array.from(
      selectableBlockRowFromBlockId(blockId)
        ?.querySelectorAll<HTMLElement>(
          "[contenteditable='true'][role='textbox'][data-notes-block-id]",
        ) ?? [],
    ).find((editor) => editor.dataset.notesBlockId === blockId) ?? null;
  }

  function focusTextEditorForBlock(blockId: string): boolean {
    const block = notes.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type)) return false;

    const editor = textEditorForBlock(blockId);
    if (!editor) return false;

    editor.focus({ preventScroll: true });
    const textLength = notesPlainTextFromEditableRoot(editor).length;
    restoreNotesEditableSelection(editor, { start: textLength, end: textLength });
    return true;
  }

  function notesTextEditorFromEvent(
    event: Event,
    blockId: string,
  ): HTMLElement | null {
    const target = event.target;
    if (!(target instanceof Element)) return null;
    const editor = target.closest<HTMLElement>(
      "[contenteditable='true'][role='textbox'][data-notes-block-id]",
    );
    if (!editor || editor.dataset.notesBlockId !== blockId) return null;
    return editor;
  }

  function editableVisualLines(editor: HTMLElement): NotesEditableVisualLine[] {
    const range = editor.ownerDocument.createRange();
    range.selectNodeContents(editor);
    const lines: NotesEditableVisualLine[] = [];
    for (const rect of Array.from(range.getClientRects())) {
      if (rect.width <= 0 && rect.height <= 0) continue;
      const existing = lines.find((line) => Math.abs(line.top - rect.top) < 2);
      if (existing) {
        existing.top = Math.min(existing.top, rect.top);
        existing.right = Math.max(existing.right, rect.right);
        existing.bottom = Math.max(existing.bottom, rect.bottom);
        existing.left = Math.min(existing.left, rect.left);
      } else {
        lines.push({
          top: rect.top,
          right: rect.right,
          bottom: rect.bottom,
          left: rect.left,
        });
      }
    }
    return lines.sort((left, right) => left.top - right.top);
  }

  function editableVisualLineTops(editor: HTMLElement): number[] {
    return editableVisualLines(editor).map((line) => line.top);
  }

  function collapsedSelectionRect(editor: HTMLElement): DOMRect | null {
    const selection = editor.ownerDocument.getSelection();
    if (!selection || selection.rangeCount === 0 || !selection.isCollapsed) return null;
    if (!selection.focusNode || !editor.contains(selection.focusNode)) return null;
    const range = selection.getRangeAt(0).cloneRange();
    const rect = range.getBoundingClientRect();
    if (rect.width > 0 || rect.height > 0) return rect;
    return Array.from(range.getClientRects())
      .find((candidate) => candidate.width > 0 || candidate.height > 0) ?? null;
  }

  function caretIsOnBoundaryVisualLine(
    editor: HTMLElement,
    direction: NotesBlockNavigationDirection,
  ): boolean {
    const selection = notesTextSelectionFromEditableRoot(editor);
    const textLength = notesPlainTextFromEditableRoot(editor).length;
    if (selection && selection.start === selection.end) {
      if (direction === "previous" && selection.start === 0) return true;
      if (direction === "next" && selection.start === textLength) return true;
    }
    const lineTops = editableVisualLineTops(editor);
    if (lineTops.length <= 1) return true;
    const caretRect = collapsedSelectionRect(editor);
    if (!caretRect) return false;
    const boundaryTop = direction === "previous" ? lineTops[0] : lineTops.at(-1);
    return boundaryTop !== undefined && Math.abs(caretRect.top - boundaryTop) < 2;
  }

  function caretOffsetFromViewportPoint(
    editor: HTMLElement,
    x: number,
    y: number,
  ): number | null {
    const documentWithCaretPosition = editor.ownerDocument as DocumentWithCaretPositionFromPoint;
    const position = documentWithCaretPosition.caretPositionFromPoint?.(x, y) ?? null;
    if (position && editor.contains(position.offsetNode)) {
      return notesEditableOffsetFromDomPoint(editor, position.offsetNode, position.offset);
    }

    const documentWithCaretRange = editor.ownerDocument as DocumentWithCaretRangeFromPoint;
    const range = documentWithCaretRange.caretRangeFromPoint?.(x, y) ?? null;
    if (!range || !editor.contains(range.startContainer)) return null;
    return notesEditableOffsetFromDomPoint(editor, range.startContainer, range.startOffset);
  }

  function clampedNavigationPointX(editor: HTMLElement, x: number): number {
    const editorRect = editor.getBoundingClientRect();
    const left = editorRect.left + 1;
    const right = editorRect.right - 1;
    if (right <= left) return editorRect.left;
    return Math.min(Math.max(x, left), right);
  }

  function targetLineOffsetFromNavigationX(
    editor: HTMLElement,
    direction: NotesBlockNavigationDirection,
    navigationX: number,
  ): number | null {
    const lines = editableVisualLines(editor);
    const line = direction === "previous" ? lines.at(-1) : lines[0];
    if (!line) return null;
    const y = line.top + Math.max(1, (line.bottom - line.top) / 2);
    return caretOffsetFromViewportPoint(editor, clampedNavigationPointX(editor, navigationX), y);
  }

  function nativeLineNavigationStaysInsideEditor(
    editor: HTMLElement,
    direction: NotesBlockNavigationDirection,
  ): boolean {
    const selection = editor.ownerDocument.getSelection();
    if (!selection || selection.rangeCount === 0 || !selection.isCollapsed) return true;
    if (!selection.focusNode || !editor.contains(selection.focusNode)) return true;
    if (caretIsOnBoundaryVisualLine(editor, direction)) return false;
    return true;
  }

  function textSelectionForFocusedBlockOffset(
    blockId: string,
    offset: number,
  ): NotesTextSelection | null {
    const block = notes.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type)) return null;
    const textLength = blockPlainText(block).length;
    const safeOffset = Math.min(Math.max(0, offset), textLength);
    return notesCollapsedNavigationSelection(safeOffset);
  }

  function textSelectionForFocusedBlock(
    blockId: string,
    selection: NotesTextSelection | null,
  ): NotesTextSelection | null {
    const block = notes.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type) || !selection) return null;
    const textLength = blockPlainText(block).length;
    const start = Math.min(Math.max(0, selection.start), textLength);
    const end = Math.min(Math.max(0, selection.end), textLength);
    return {
      start: Math.min(start, end),
      end: Math.max(start, end),
    };
  }

  function adjacentNavigationFallbackOffset(
    blockId: string,
    direction: NotesBlockNavigationDirection,
  ): number {
    if (direction === "next") return 0;
    const block = notes.blockById(blockId);
    return block && isTextEditableBlock(block.type) ? blockPlainText(block).length : 0;
  }

  function adjacentNavigationSelection(
    blockId: string,
    direction: NotesBlockNavigationDirection,
    navigationX: number | null,
  ): NotesTextSelection | null {
    const block = notes.blockById(blockId);
    if (!block || !isTextEditableBlock(block.type)) return null;
    const editor = navigationX === null ? null : textEditorForBlock(blockId);
    const offset = editor && navigationX !== null
      ? targetLineOffsetFromNavigationX(editor, direction, navigationX)
      : null;
    return textSelectionForFocusedBlockOffset(
      blockId,
      offset ?? adjacentNavigationFallbackOffset(blockId, direction),
    );
  }

  function focusRenderedBlock(blockId: string, selection: NotesTextSelection | null): void {
    notes.focusBlock(blockId, textSelectionForFocusedBlock(blockId, selection));
  }

  function focusAdjacentRenderedBlock(
    currentBlockId: string,
    direction: NotesBlockNavigationDirection,
    navigationX: number | null,
  ): boolean {
    const targetBlockId = notesAdjacentRenderedBlockId(
      renderedSelectableBlockIds(),
      currentBlockId,
      direction,
    );
    if (!targetBlockId) return false;
    focusRenderedBlock(
      targetBlockId,
      adjacentNavigationSelection(targetBlockId, direction, navigationX),
    );
    return true;
  }

  function boundaryFocusOffset(blockId: string, boundary: NotesBlockNavigationBoundary): number {
    if (boundary === "first") return 0;
    const block = notes.blockById(blockId);
    return block && isTextEditableBlock(block.type) ? blockPlainText(block).length : 0;
  }

  function focusBoundaryRenderedBlock(boundary: NotesBlockNavigationBoundary): boolean {
    const targetBlockId = notesBoundaryRenderedBlockId(renderedSelectableBlockIds(), boundary);
    if (!targetBlockId) return false;
    focusRenderedBlock(
      targetBlockId,
      textSelectionForFocusedBlockOffset(targetBlockId, boundaryFocusOffset(targetBlockId, boundary)),
    );
    return true;
  }

  function handleDocumentNavigationKeydown(event: KeyboardEvent, blockId: string): boolean {
    if (event.altKey || event.shiftKey) return false;

    if (event.ctrlKey || event.metaKey) {
      if (event.key === "Home" || (event.metaKey && event.key === "ArrowUp")) {
        event.preventDefault();
        return focusBoundaryRenderedBlock("first");
      }
      if (event.key === "End" || (event.metaKey && event.key === "ArrowDown")) {
        event.preventDefault();
        return focusBoundaryRenderedBlock("last");
      }
      return false;
    }

    if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return false;
    const direction: NotesBlockNavigationDirection = event.key === "ArrowUp" ? "previous" : "next";
    const editor = notesTextEditorFromEvent(event, blockId);
    if (editor) {
      const selection = notesTextSelectionFromEditableRoot(editor);
      if (!selection || selection.start !== selection.end) return false;
      if (nativeLineNavigationStaysInsideEditor(editor, direction)) return false;
      const navigationX = collapsedSelectionRect(editor)?.left ?? null;
      if (!focusAdjacentRenderedBlock(blockId, direction, navigationX)) return false;
      event.preventDefault();
      return true;
    }

    if (eventTargetIsEditable(event.target)) return false;
    if (!focusAdjacentRenderedBlock(blockId, direction, null)) return false;
    event.preventDefault();
    return true;
  }

  function clearNativeSelection(): void {
    if (typeof window === "undefined") return;
    window.getSelection()?.removeAllRanges();
  }

  function selectBlockFromPointer(blockId: string, extend: boolean): void {
    const selection = notesBlockSelectionAfterClick({
      blockIds: renderedSelectableBlockIds(),
      current: blockSelection,
      blockId,
      extend,
    });
    setBlockSelection(selection);
    if (selection) focusSelectedBlockRow(selection.focusBlockId);
  }

  function handleBlockSelectionPointerDown(event: PointerEvent): void {
    if (event.button !== 0) return;
    const blockId = selectableBlockIdFromEvent(event);
    if (!blockId) {
      if (blockSelection) setBlockSelection(null);
      return;
    }
    const shouldExtend = event.shiftKey && blockSelection !== null;
    if (shouldExtend) {
      event.preventDefault();
      clearNativeSelection();
      selectBlockFromPointer(blockId, true);
      return;
    }
    if (!eventTargetIsSelectionZone(event.target)) {
      if (blockSelection) setBlockSelection(null);
      return;
    }
    if (eventTargetIsEditable(event.target)) return;
    if (focusTextEditorForBlock(blockId)) {
      event.preventDefault();
      setBlockSelection(null);
      return;
    }
    event.preventDefault();
    clearNativeSelection();
    selectionDragAnchorBlockId = blockId;
    selectionDragPointerId = event.pointerId;
    setBlockSelection(notesBlockSelectionForBlock(renderedSelectableBlockIds(), blockId));
    focusSelectedBlockRow(blockId);
  }

  function handleBlockSelectionPointerOver(event: PointerEvent): void {
    if (selectionDragPointerId === null || event.pointerId !== selectionDragPointerId) return;
    const anchorBlockId = selectionDragAnchorBlockId;
    const blockId = selectableBlockIdFromEvent(event);
    if (!anchorBlockId || !blockId) return;
    const selection = notesBlockSelectionRange(renderedSelectableBlockIds(), anchorBlockId, blockId);
    setBlockSelection(selection);
  }

  function showBlockHandleFromPointer(blockId: string): void {
    if (openBlockHandleMenuId) return;
    blockHandleHoverState = notesBlockHandleHoverStateAfterPointerMove(
      blockHandleHoverState,
      blockId,
    );
  }

  function hideBlockHandleAfterPointerLeave(blockId: string): void {
    blockHandleHoverState = notesBlockHandleHoverStateAfterPointerLeave(
      blockHandleHoverState,
      blockId,
    );
  }

  function hideBlockHandleAfterKeyboard(event: KeyboardEvent): void {
    blockHandleHoverState = notesBlockHandleHoverStateAfterKeydown(
      blockHandleHoverState,
      event.key,
    );
  }

  function updateBlockHandleMenuOpen(blockId: string, open: boolean): void {
    if (open) {
      openBlockHandleMenuId = blockId;
      blockHandleHoverState = EMPTY_NOTES_BLOCK_HANDLE_HOVER_STATE;
      return;
    }
    if (openBlockHandleMenuId === blockId) openBlockHandleMenuId = null;
  }

  function handleBlockListKeydown(event: KeyboardEvent): void {
    hideBlockHandleAfterKeyboard(event);
    if (event.defaultPrevented) return;
    const blockId = selectableBlockIdFromEvent(event);
    if (!blockId) return;
    if (handleDocumentNavigationKeydown(event, blockId)) return;
    if (handleBlockSelectionShortcut(event, blockId)) return;
    if (event.key === "Escape" && !event.ctrlKey && !event.metaKey && !event.altKey) {
      event.preventDefault();
      clearNativeSelection();
      setBlockSelection(notesBlockSelectionForBlock(renderedSelectableBlockIds(), blockId));
      focusSelectedBlockRow(blockId);
      return;
    }
    if (
      event.shiftKey
      && !event.ctrlKey
      && !event.metaKey
      && !event.altKey
      && (event.key === "ArrowDown" || event.key === "ArrowUp")
    ) {
      if (!blockSelection && eventTargetIsEditable(event.target)) return;
      event.preventDefault();
      clearNativeSelection();
      const selection = notesBlockSelectionAfterKeyboard({
        blockIds: renderedSelectableBlockIds(),
        current: blockSelection,
        focusedBlockId: blockId,
        direction: event.key === "ArrowDown" ? "next" : "previous",
      });
      setBlockSelection(selection);
      if (selection) focusSelectedBlockRow(selection.focusBlockId, false);
    }
  }

  function handleBlockSelectionShortcut(event: KeyboardEvent, blockId: string): boolean {
    const hasModifier = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    if (
      selectionClipboard
      && hasModifier
      && !event.shiftKey
      && !event.altKey
      && key === "v"
      && !eventTargetIsEditable(event.target)
    ) {
      event.preventDefault();
      void runSelectionAction(() => pasteSelectionClipboard(blockSelection?.focusBlockId ?? blockId));
      return true;
    }
    if (!blockSelection) return false;
    if (!event.altKey && (event.key === "Backspace" || event.key === "Delete")) {
      event.preventDefault();
      void runSelectionAction(deleteCurrentBlockSelection);
      return true;
    }
    if (hasModifier && event.shiftKey && !event.altKey && event.key === "ArrowUp") {
      event.preventDefault();
      void runSelectionAction(() => moveCurrentBlockSelection("up"));
      return true;
    }
    if (hasModifier && event.shiftKey && !event.altKey && event.key === "ArrowDown") {
      event.preventDefault();
      void runSelectionAction(() => moveCurrentBlockSelection("down"));
      return true;
    }
    if (!hasModifier || event.shiftKey || event.altKey) return false;
    if (key === "c") {
      event.preventDefault();
      void runSelectionAction(() => copyCurrentBlockSelection("copy"));
      return true;
    }
    if (key === "x") {
      event.preventDefault();
      void runSelectionAction(() => copyCurrentBlockSelection("cut"));
      return true;
    }
    if (key === "d") {
      event.preventDefault();
      void runSelectionAction(duplicateCurrentBlockSelection);
      return true;
    }
    return false;
  }

  async function runSelectionAction(action: () => Promise<void> | void): Promise<void> {
    if (selectionBusy) return;
    selectionBusy = true;
    selectionActionError = null;
    try {
      await action();
    } catch (error) {
      selectionActionError = error instanceof Error ? error.message : String(error);
    } finally {
      selectionBusy = false;
    }
  }

  async function copyCurrentBlockSelection(mode: "copy" | "cut"): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    const state = currentTreeState();
    const rootBlockIds = notesSelectionRootBlockIds(state, selection.selectedBlockIds);
    const subtreeBlockIds = notesSelectionSubtreeIds(state, rootBlockIds);
    if (rootBlockIds.length === 0 || subtreeBlockIds.length === 0) return;
    const plainText = notesSelectionPlainText(state, rootBlockIds);
    selectionClipboard = { mode, pageId, rootBlockIds, subtreeBlockIds, plainText };
    if (plainText && typeof navigator !== "undefined" && navigator.clipboard) {
      await navigator.clipboard.writeText(plainText).catch(() => undefined);
    }
    if (mode === "cut") {
      await notes.deleteBlockSelection(selection.selectedBlockIds);
      setBlockSelection(null);
    }
  }

  async function pasteSelectionClipboard(targetBlockId: string | null): Promise<void> {
    const clipboard = selectionClipboard;
    if (!clipboard || !targetBlockId) return;
    const focusBlockId = await notes.pasteBlockSelection(
      clipboard.rootBlockIds,
      clipboard.subtreeBlockIds,
      targetBlockId,
      clipboard.mode === "cut",
    );
    if (clipboard.mode === "cut") selectionClipboard = null;
    setBlockSelection(null);
    if (focusBlockId) focusSelectedBlockRow(focusBlockId, false);
  }

  async function duplicateCurrentBlockSelection(): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.duplicateBlockSelection(selection.selectedBlockIds);
    setBlockSelection(null);
  }

  async function moveCurrentBlockSelection(direction: "up" | "down"): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.moveBlockSelection(selection.selectedBlockIds, direction);
  }

  async function deleteCurrentBlockSelection(): Promise<void> {
    const selection = blockSelection;
    if (!selection) return;
    await notes.deleteBlockSelection(selection.selectedBlockIds);
    setBlockSelection(null);
  }

  function blockSelectionDelegation(node: HTMLDivElement): { destroy: () => void } {
    node.addEventListener("pointerdown", handleBlockSelectionPointerDown);
    node.addEventListener("pointerover", handleBlockSelectionPointerOver);
    node.addEventListener("keydown", handleBlockListKeydown);
    return {
      destroy() {
        node.removeEventListener("pointerdown", handleBlockSelectionPointerDown);
        node.removeEventListener("pointerover", handleBlockSelectionPointerOver);
        node.removeEventListener("keydown", handleBlockListKeydown);
      },
    };
  }

  function handleKeyboardAction(blockId: string, action: NotesKeyboardAction): void {
    if (action.type === "create_sibling") {
      void notes.createSiblingAfter(blockId);
      return;
    }
    if (action.type === "split_text_block") {
      void splitTextBlockFromKeyboardAction(blockId, action);
      return;
    }
    if (action.type === "convert_to_paragraph") {
      void notes.convertBlock(blockId, "paragraph", true);
      return;
    }
    if (action.type === "apply_text_shortcut") {
      void notes.convertBlock(blockId, action.blockType, true);
      return;
    }
    if (action.type === "toggle_block_open") {
      const block = notes.blockById(blockId);
      if (block?.type === "toggle") {
        void notes.updateToggleOpen(blockId, block.toggle.ganbaru_open === false);
      }
      return;
    }
    if (action.type === "delete_block") {
      void notes.deleteBlock(blockId);
      return;
    }
    if (action.type === "merge_with_previous") {
      void notes.mergeBlockWithPrevious(blockId);
      return;
    }
    if (action.type === "nest") {
      void notes.nestBlock(blockId);
      return;
    }
    if (action.type === "outdent") {
      void notes.outdentBlock(blockId);
      return;
    }
    if (action.type === "move_up") {
      void notes.moveBlockUp(blockId);
      return;
    }
    if (action.type === "move_down") {
      void notes.moveBlockDown(blockId);
    }
  }

  async function splitTextBlockFromKeyboardAction(
    blockId: string,
    action: Extract<NotesKeyboardAction, { type: "split_text_block" }>,
  ): Promise<void> {
    await notes.updateBlockText(blockId, action.text);
    await notes.splitTextBlockAtSelection(blockId, action.selectionStart, action.selectionEnd);
  }

  function handleConvert(blockId: string, type: NotesBlockType, clearText = false): void {
    void notes.convertBlock(blockId, type, clearText);
  }

  function convertToToggleHeading(blockId: string, type: NotesHeadingBlockType, clearText = false): void {
    void notes.convertBlockToToggleHeading(blockId, type, clearText);
  }

  function moveTargetsForBlock(block: NotesBlock): NotesMoveToPageTarget[] {
    return notesMoveToPageTargets(notes.allPages, block, pageId, t("notes.untitled"), {
      recentPageIds: notes.recentPageIds,
      excludedPageIds: loadedChildPageIdsInSubtree(block.id),
    });
  }

  function moveTargetsForBlockId(blockId: string): NotesMoveToPageTarget[] {
    const block = notes.blockById(blockId);
    return block ? moveTargetsForBlock(block) : [];
  }

  function loadedChildPageIdsInSubtree(blockId: string): string[] {
    const result: string[] = [];
    const seen = new Set<string>();
    const queue = [blockId];
    while (queue.length > 0) {
      const currentBlockId = queue.shift();
      if (!currentBlockId || seen.has(currentBlockId)) continue;
      seen.add(currentBlockId);
      const block = notes.blockById(currentBlockId);
      if (block?.type === "child_page") result.push(block.id);
      queue.push(...(notes.childIdsByParentId[currentBlockId] ?? []));
    }
    return result;
  }

  async function copyBlockLink(blockId: string): Promise<void> {
    const link = buildNotesBlockLink(window.location.href, { pageId, blockId });
    await navigator.clipboard.writeText(link);
  }

  async function insertPageMention(
    blockId: string,
    start: number,
    end: number,
    target: NotesPageMentionTarget,
  ): Promise<void> {
    await notes.insertPageMention(
      blockId,
      start,
      end,
      target.id,
      target.title,
      buildNotesPageLink(window.location.href, { pageId: target.id }),
    );
  }

  async function insertDateMention(
    blockId: string,
    start: number,
    end: number,
    target: NotesDateMentionTarget,
  ): Promise<void> {
    await notes.insertDateMention(blockId, start, end, target.date, target.title);
  }

  async function insertObjectMention(
    blockId: string,
    start: number,
    end: number,
    target: NotesObjectMentionTarget,
  ): Promise<void> {
    await notes.insertObjectMention(blockId, start, end, target);
  }

  async function applyTextAnnotations(
    blockId: string,
    start: number,
    end: number,
    patch: NotesRichTextAnnotationPatch,
  ): Promise<void> {
    await notes.updateBlockTextAnnotations(blockId, start, end, patch);
  }

  async function applyTextLink(
    blockId: string,
    start: number,
    end: number,
    url: string | null,
  ): Promise<void> {
    await notes.updateBlockTextLink(blockId, start, end, url);
  }

  async function insertInlineEquation(
    blockId: string,
    start: number,
    end: number,
    expression: string,
  ): Promise<void> {
    await notes.insertInlineEquation(blockId, start, end, expression);
  }

  function pastePlainText(
    blockId: string,
    start: number,
    end: number,
    plainText: string,
  ): Promise<boolean> {
    return notes.pastePlainTextIntoBlock(blockId, start, end, plainText);
  }

  function pasteRichHtml(
    blockId: string,
    start: number,
    end: number,
    html: string,
  ): Promise<boolean> {
    return notes.pasteRichHtmlIntoBlock(blockId, start, end, html);
  }

  function replaceBlockRichText(
    blockId: string,
    richText: readonly NotesRichText[],
  ): void {
    void notes.updateBlockRichText(blockId, richText);
  }

  function replaceTableCellRichText(
    rowBlockId: string,
    columnIndex: number,
    richText: readonly NotesRichText[],
  ): void {
    void notes.updateTableCellRichText(rowBlockId, columnIndex, richText);
  }

  function addTableRow(tableBlockId: string, afterRowIndex: number): Promise<void> {
    return notes.addTableRow(tableBlockId, afterRowIndex);
  }

  function removeTableRow(tableBlockId: string, rowBlockId: string): Promise<void> {
    return notes.removeTableRow(tableBlockId, rowBlockId);
  }

  function addTableColumn(tableBlockId: string, afterColumnIndex: number): Promise<void> {
    return notes.addTableColumn(tableBlockId, afterColumnIndex);
  }

  function removeTableColumn(tableBlockId: string, columnIndex: number): Promise<void> {
    return notes.removeTableColumn(tableBlockId, columnIndex);
  }

  function addColumn(columnListBlockId: string, afterColumnIndex: number): Promise<void> {
    return notes.addColumn(columnListBlockId, afterColumnIndex);
  }

  function removeColumn(columnListBlockId: string, columnBlockId: string): Promise<void> {
    return notes.removeColumn(columnListBlockId, columnBlockId);
  }

  function moveColumn(
    columnListBlockId: string,
    columnBlockId: string,
    direction: "left" | "right",
  ): Promise<void> {
    return notes.moveColumn(columnListBlockId, columnBlockId, direction);
  }

  function resizeColumn(
    columnListBlockId: string,
    columnBlockId: string,
    widthRatio: number,
  ): Promise<void> {
    return notes.resizeColumn(columnListBlockId, columnBlockId, widthRatio);
  }

  function moveBlockToColumn(blockId: string, columnBlockId: string): Promise<void> {
    return notes.moveBlockToColumn(blockId, columnBlockId);
  }

  function updateTabLabel(labelBlockId: string, label: string): Promise<void> {
    return notes.updateTabLabel(labelBlockId, label);
  }

  function updateTabIcon(labelBlockId: string, icon: NotesIcon | null): Promise<void> {
    return notes.updateTabIcon(labelBlockId, icon);
  }

  function addTab(tabBlockId: string, afterTabIndex: number): Promise<void> {
    return notes.addTab(tabBlockId, afterTabIndex);
  }

  function removeTab(tabBlockId: string, labelBlockId: string): Promise<void> {
    return notes.removeTab(tabBlockId, labelBlockId);
  }

  function moveTab(
    tabBlockId: string,
    labelBlockId: string,
    direction: "left" | "right",
  ): Promise<void> {
    return notes.moveTab(tabBlockId, labelBlockId, direction);
  }

  function moveBlockToTab(blockId: string, labelBlockId: string): Promise<void> {
    return notes.moveBlockToTab(blockId, labelBlockId);
  }

  function undoNotesEdit(): void {
    void notes.undoNotesEdit();
  }

  function redoNotesEdit(): void {
    void notes.redoNotesEdit();
  }

  function draggedBlockIdFromEvent(event: DragEvent): string | null {
    const transferred = event.dataTransfer?.getData(NOTES_BLOCK_DRAG_MIME) ?? "";
    return transferred || draggingBlockId;
  }

  function blockDropIntentFromEvent(event: DragEvent): NotesBlockDropIntent {
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) return "after";
    const rect = target.getBoundingClientRect();
    const depth = Number.parseInt(
      getComputedStyle(target).getPropertyValue("--notes-depth").trim(),
      10,
    );
    const safeDepth = Number.isFinite(depth) ? Math.max(depth, 0) : 0;
    const yRatio = rect.height > 0 ? (event.clientY - rect.top) / rect.height : 0.5;
    if (yRatio < 0.25) return "before";
    const outdentBoundary = rect.left + safeDepth * 20 + 28;
    if (safeDepth > 0 && event.clientX < outdentBoundary) return "outdent";
    if (yRatio > 0.75) return "after";
    return "inside";
  }

  function handleBlockDragStart(blockId: string, event: DragEvent): void {
    draggingBlockId = blockId;
    setActiveNotesBlockDragId(blockId);
    dropTarget = null;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData(NOTES_BLOCK_DRAG_MIME, blockId);
      event.dataTransfer.setData("text/plain", blockId);
    }
  }

  function handleBlockDragEnd(): void {
    draggingBlockId = null;
    setActiveNotesBlockDragId(null);
    dropTarget = null;
  }

  function handleBlockDragOver(targetBlockId: string, event: DragEvent): void {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!sourceBlockId) {
      if (dropTarget?.blockId === targetBlockId) dropTarget = null;
      return;
    }
    const intent = blockDropIntentFromEvent(event);
    const plan = planNotesBlockDrop(currentTreeState(), sourceBlockId, targetBlockId, intent);
    if (!plan) {
      if (dropTarget?.blockId === targetBlockId) dropTarget = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dropTarget = { blockId: targetBlockId, intent: plan.indicator };
  }

  function handleBlockDragLeave(targetBlockId: string, event: DragEvent): void {
    const target = event.currentTarget;
    const related = event.relatedTarget;
    if (
      target instanceof HTMLElement
      && related instanceof Node
      && target.contains(related)
    ) {
      return;
    }
    if (dropTarget?.blockId === targetBlockId) dropTarget = null;
  }

  function handleBlockDrop(targetBlockId: string, event: DragEvent): void {
    const sourceBlockId = draggedBlockIdFromEvent(event);
    if (!sourceBlockId) {
      handleBlockDragEnd();
      return;
    }
    const intent = dropTarget?.blockId === targetBlockId
      ? dropTarget.intent
      : blockDropIntentFromEvent(event);
    const plan = planNotesBlockDrop(currentTreeState(), sourceBlockId, targetBlockId, intent);
    if (!plan) {
      handleBlockDragEnd();
      return;
    }
    event.preventDefault();
    handleBlockDragEnd();
    void notes.dropBlockOnBlock(sourceBlockId, targetBlockId, plan.indicator);
  }

  function dropPositionForBlock(blockId: string): NotesBlockDropIndicator | null {
    return dropTarget?.blockId === blockId ? dropTarget.intent : null;
  }
</script>

<div
  use:blockSelectionDelegation
  bind:this={blockListElement}
  class="notes-block-list flex min-w-0 flex-col gap-0.5 pb-8"
  role="group"
  aria-label={t("notes.blockList")}
>
  {#if blockSelection}
    <div
      class="sticky top-2 z-20 mb-2 flex min-w-0 flex-wrap items-center gap-1 rounded-md border border-border bg-popover/95 px-2 py-1.5 text-xs text-popover-foreground shadow-sm backdrop-blur"
      role="toolbar"
      aria-label={t("notes.selectionActions")}
    >
      <span class="mr-1 shrink-0 font-medium text-muted-foreground">
        {t("notes.selectedBlocks", selectedBlockCount)}
      </span>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.copySelection")}
        title={t("notes.copySelection")}
        onclick={() => {
          void runSelectionAction(() => copyCurrentBlockSelection("copy"));
        }}
      >
        <Copy size={14} aria-hidden="true" />
        <span>{t("notes.copySelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.cutSelection")}
        title={t("notes.cutSelection")}
        onclick={() => {
          void runSelectionAction(() => copyCurrentBlockSelection("cut"));
        }}
      >
        <Scissors size={14} aria-hidden="true" />
        <span>{t("notes.cutSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !selectionClipboard}
        aria-label={t("notes.pasteSelection")}
        title={t("notes.pasteSelection")}
        onclick={() => {
          void runSelectionAction(() => pasteSelectionClipboard(blockSelection?.focusBlockId ?? notes.focusBlockId));
        }}
      >
        <ClipboardPaste size={14} aria-hidden="true" />
        <span>{t("notes.pasteSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.duplicateSelection")}
        title={t("notes.duplicateSelection")}
        onclick={() => {
          void runSelectionAction(duplicateCurrentBlockSelection);
        }}
      >
        <CopyPlus size={14} aria-hidden="true" />
        <span>{t("notes.duplicateSelection")}</span>
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center justify-center rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !canMoveSelectionUp}
        aria-label={t("notes.moveSelectionUp")}
        title={t("notes.moveSelectionUp")}
        onclick={() => {
          void runSelectionAction(() => moveCurrentBlockSelection("up"));
        }}
      >
        <ArrowUp size={15} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center justify-center rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || !canMoveSelectionDown}
        aria-label={t("notes.moveSelectionDown")}
        title={t("notes.moveSelectionDown")}
        onclick={() => {
          void runSelectionAction(() => moveCurrentBlockSelection("down"));
        }}
      >
        <ArrowDown size={15} aria-hidden="true" />
      </button>
      <button
        type="button"
        class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
        disabled={selectionBusy || selectedRootBlockIds.length === 0}
        aria-label={t("notes.deleteSelection")}
        title={t("notes.deleteSelection")}
        onclick={() => {
          void runSelectionAction(deleteCurrentBlockSelection);
        }}
      >
        <Trash2 size={14} aria-hidden="true" />
        <span>{t("notes.deleteSelection")}</span>
      </button>
      {#if selectionActionError}
        <span class="min-w-0 flex-1 truncate text-destructive" role="status">
          {t("notes.selectionActionFailed")} {selectionActionError}
        </span>
      {/if}
    </div>
  {/if}
  {#each items as item (item.block.id)}
    {#if item.block.type === "column_list"}
      {#if structuralBlockLoadStates["column-list"]?.status === "ready" && structuralBlockLoadStates["column-list"].component.kind === "column-list"}
        {@const NotesColumnListBlock = structuralBlockLoadStates["column-list"].component.component}
        <NotesColumnListBlock
        {item}
        columnItems={notes.columnItemsForBlock(item.block.id)}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRowsForBlock={notes.tableRowsForBlock}
        previousBlockType={notes.previousBlockType(item.block.id)}
        previousBlockTypeForBlock={notes.previousBlockType}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        isOnlyBlockForBlock={notes.isOnlyBlock}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        focusSelection={notes.focusSelection}
        handleVisibleBlockId={blockHandleHoverState.visibleBlockId}
        {mentionTargets}
        {templateStatusForBlock}
        {buttonStatusForBlock}
        onTextInput={(blockId, text, selection) => {
          void notes.updateBlockText(blockId, text, selection);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onInsertObjectMention={insertObjectMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
        onCreateInlineComment={(blockId, start, end) => {
          void notes.startInlineComment(blockId, start, end);
        }}
        onCreateInlineSuggestion={(blockId, start, end) => {
          void notes.startInlineSuggestion(blockId, start, end);
        }}
        onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        moveTargetsForBlock={moveTargetsForBlockId}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        {draggingBlockId}
        {dropPositionForBlock}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name, assetChange) => {
          void notes.updateMedia(blockId, url, caption, name, assetChange);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        onAddColumn={addColumn}
        onRemoveColumn={removeColumn}
        onMoveColumn={moveColumn}
        onResizeColumn={resizeColumn}
        onMoveBlockToColumn={moveBlockToColumn}
        {onSelectPage}
        {onFocusBlock}
        onHandlePointerMove={showBlockHandleFromPointer}
        onHandlePointerLeave={hideBlockHandleAfterPointerLeave}
        onHandleMenuOpenChange={updateBlockHandleMenuOpen}
        />
      {:else if structuralBlockLoadStates["column-list"]?.status === "failed"}
        <div class="my-1 rounded-md border border-destructive/40 p-2 text-[0.8rem] text-destructive" role="alert">
          <p>{t("common.viewLoadFailed", t("notes.blockType.columns"))}</p>
          <button class="mt-2 min-h-8 rounded-md border border-border px-2 text-foreground hover:bg-accent" type="button" onclick={() => requestStructuralBlock("column-list", true)}>{t("common.retry")}</button>
        </div>
      {:else}
        <div class="my-1 min-h-8 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
      {/if}
    {:else if item.block.type === "tab"}
      {#if structuralBlockLoadStates.tab?.status === "ready" && structuralBlockLoadStates.tab.component.kind === "tab"}
        {@const NotesTabBlock = structuralBlockLoadStates.tab.component.component}
        <NotesTabBlock
        {item}
        tabItems={notes.tabItemsForBlock(item.block.id)}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRowsForBlock={notes.tableRowsForBlock}
        previousBlockType={notes.previousBlockType(item.block.id)}
        previousBlockTypeForBlock={notes.previousBlockType}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        isOnlyBlockForBlock={notes.isOnlyBlock}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        focusSelection={notes.focusSelection}
        handleVisibleBlockId={blockHandleHoverState.visibleBlockId}
        {mentionTargets}
        {templateStatusForBlock}
        {buttonStatusForBlock}
        onTextInput={(blockId, text, selection) => {
          void notes.updateBlockText(blockId, text, selection);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onInsertObjectMention={insertObjectMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
        onCreateInlineComment={(blockId, start, end) => {
          void notes.startInlineComment(blockId, start, end);
        }}
        onCreateInlineSuggestion={(blockId, start, end) => {
          void notes.startInlineSuggestion(blockId, start, end);
        }}
        onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        moveTargetsForBlock={moveTargetsForBlockId}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        {draggingBlockId}
        {dropPositionForBlock}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name, assetChange) => {
          void notes.updateMedia(blockId, url, caption, name, assetChange);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        onUpdateTabLabel={updateTabLabel}
        onUpdateTabIcon={updateTabIcon}
        onAddTab={addTab}
        onRemoveTab={removeTab}
        onMoveTab={moveTab}
        onMoveBlockToTab={moveBlockToTab}
        {onSelectPage}
        {onFocusBlock}
        onHandlePointerMove={showBlockHandleFromPointer}
        onHandlePointerLeave={hideBlockHandleAfterPointerLeave}
        onHandleMenuOpenChange={updateBlockHandleMenuOpen}
        />
      {:else if structuralBlockLoadStates.tab?.status === "failed"}
        <div class="my-1 rounded-md border border-destructive/40 p-2 text-[0.8rem] text-destructive" role="alert">
          <p>{t("common.viewLoadFailed", t("notes.blockType.tab"))}</p>
          <button class="mt-2 min-h-8 rounded-md border border-border px-2 text-foreground hover:bg-accent" type="button" onclick={() => requestStructuralBlock("tab", true)}>{t("common.retry")}</button>
        </div>
      {:else}
        <div class="my-1 min-h-8 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
      {/if}
    {:else}
      <NotesBlockRow
        {item}
        {breadcrumbItems}
        {tableOfContentsItems}
        tableRows={notes.tableRowsForBlock(item.block.id)}
        previousBlockType={notes.previousBlockType(item.block.id)}
        isOnlyBlock={notes.isOnlyBlock(item.block.id)}
        focusBlockId={notes.focusBlockId}
        focusRequestId={notes.focusRequestId}
        focusSelection={notes.focusSelection}
        handleVisibleBlockId={blockHandleHoverState.visibleBlockId}
        {mentionTargets}
        templateStatus={templateStatusForBlock(item.block.id)}
        buttonStatus={buttonStatusForBlock(item.block.id)}
        onTextInput={(blockId, text, selection) => {
          void notes.updateBlockText(blockId, text, selection);
        }}
        onReplaceRichText={replaceBlockRichText}
        onInsertPageMention={insertPageMention}
        onInsertDateMention={insertDateMention}
        onInsertObjectMention={insertObjectMention}
        onApplyTextLink={applyTextLink}
        onInsertInlineEquation={insertInlineEquation}
        onPastePlainText={pastePlainText}
        onPasteRichHtml={pasteRichHtml}
        onApplyTextAnnotations={applyTextAnnotations}
      onCreateInlineComment={(blockId, start, end) => {
        void notes.startInlineComment(blockId, start, end);
      }}
      onCreateInlineSuggestion={(blockId, start, end) => {
        void notes.startInlineSuggestion(blockId, start, end);
      }}
      onKeyboardAction={handleKeyboardAction}
        onUndo={undoNotesEdit}
        onRedo={redoNotesEdit}
        onAddBelow={(blockId, request?: NotesBlockInsertRequest) => {
          void notes.createSiblingAfter(blockId, request);
        }}
        onConvert={handleConvert}
        onConvertToToggleHeading={convertToToggleHeading}
        onColorChange={(blockId, color) => {
          void notes.updateBlockColor(blockId, color);
        }}
        onCopyLink={copyBlockLink}
        onDuplicate={(blockId) => {
          void notes.duplicateBlock(blockId);
        }}
        onUseTemplate={(blockId) => {
          void notes.useTemplateBlock(blockId);
        }}
        onAddTemplateChild={(blockId) => {
          void notes.addTemplateChild(blockId);
        }}
        onUseButton={(blockId) => {
          void notes.useButtonBlock(blockId);
        }}
        onAddButtonChild={(blockId) => {
          void notes.addButtonChild(blockId);
        }}
        onButtonIconChange={(blockId, icon: NotesIcon | null) => {
          void notes.updateButtonIcon(blockId, icon);
        }}
        onButtonInsertPositionChange={(blockId, position: NotesButtonInsertPosition) => {
          void notes.updateButtonInsertPosition(blockId, position);
        }}
        onCreateLinkedDatabaseView={(blockId) => {
          void notes.createLinkedDatabaseViewAfter(blockId);
        }}
        onConvertUnsupported={(blockId, target: NotesUnsupportedConversionTarget) => {
          void notes.convertUnsupportedBlock(blockId, target);
        }}
        onComment={(blockId) => {
          void notes.startBlockComment(blockId);
        }}
        onMoveUp={(blockId) => {
          void notes.moveBlockUp(blockId);
        }}
        onMoveDown={(blockId) => {
          void notes.moveBlockDown(blockId);
        }}
        moveTargets={moveTargetsForBlock(item.block)}
        onMoveToPage={(blockId, targetPageId) => {
          void notes.moveBlockToPage(blockId, targetPageId);
        }}
        onDelete={(blockId) => {
          void notes.deleteBlock(blockId);
        }}
        isDragging={draggingBlockId === item.block.id}
        dropPosition={dropPositionForBlock(item.block.id)}
        onDragStart={handleBlockDragStart}
        onDragEnd={handleBlockDragEnd}
        onDragOver={handleBlockDragOver}
        onDragLeave={handleBlockDragLeave}
        onDrop={handleBlockDrop}
        onToggleTodo={(blockId, checked) => {
          void notes.toggleTodo(blockId, checked);
        }}
        onToggleOpen={(blockId, open) => {
          void notes.updateToggleOpen(blockId, open);
        }}
        onCodeLanguageChange={(blockId, language) => {
          void notes.updateCodeLanguage(blockId, language);
        }}
        onBookmarkChange={(blockId, url, caption) => {
          void notes.updateBookmark(blockId, url, caption);
        }}
        onLinkPreviewUrlChange={(blockId, url) => {
          void notes.updateLinkPreviewUrl(blockId, url);
        }}
        onEmbedUrlChange={(blockId, url) => {
          void notes.updateEmbedUrl(blockId, url);
        }}
        onEquationExpressionChange={(blockId, expression) => {
          void notes.updateEquationExpression(blockId, expression);
        }}
        onMediaChange={(blockId, url, caption, name, assetChange) => {
          void notes.updateMedia(blockId, url, caption, name, assetChange);
        }}
        onTableCellRichTextChange={replaceTableCellRichText}
        onAddTableRow={addTableRow}
        onRemoveTableRow={removeTableRow}
        onAddTableColumn={addTableColumn}
        onRemoveTableColumn={removeTableColumn}
        {onSelectPage}
        {onFocusBlock}
        onHandlePointerMove={showBlockHandleFromPointer}
        onHandlePointerLeave={hideBlockHandleAfterPointerLeave}
        onHandleMenuOpenChange={updateBlockHandleMenuOpen}
      />
    {/if}
  {/each}
</div>

<style>
  :global(.notes-block-row[data-notes-block-selected="true"] > .notes-block-surface) {
    background: hsl(var(--primary) / 0.12);
    box-shadow: inset 0 0 0 1px hsl(var(--primary) / 0.42);
  }

  :global(.notes-block-row[data-notes-block-selected="true"]:focus-visible > .notes-block-surface) {
    outline: 2px solid hsl(var(--ring));
    outline-offset: 1px;
  }

</style>
