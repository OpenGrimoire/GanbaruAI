<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    beginLazyComponentLoad,
    rejectLazyComponentLoad,
    resolveLazyComponentLoad,
    type LazyComponentLoadState,
  } from "$lib/lazy-component-loader";
  import { dismissOnOutside } from "$lib/utils/dismiss-on-outside";
  import {
    NOTES_BACKGROUND_COLORS,
    NOTES_TEXT_COLORS,
    notesBlockColorSwatchStyle,
  } from "$lib/notes/block-color";
  import {
    CLOSED_NOTES_BLOCK_HANDLE_MENUS,
    notesBlockHandleActionMenuStyle,
    notesBlockHandleMenuStateAfterAction,
    notesBlockHandleMenuStateAfterToggle,
  } from "$lib/notes/block-handle";
  import type {
    NotesBlockHandleAction,
    NotesBlockHandleMenuState,
  } from "$lib/notes/block-handle";
  import type {
    NotesBlockInsertCommand,
    NotesBlockInsertMenuRect,
  } from "$lib/notes/block-insertion";
  import type { NotesMoveToPageTarget } from "$lib/notes/block-move";
  import type { NotesColor } from "$lib/notes/types";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import FolderInput from "@lucide/svelte/icons/folder-input";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Pilcrow from "@lucide/svelte/icons/pilcrow";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { onDestroy } from "svelte";
  import {
    loadNotesEditorPanel,
    loadNotesTextControl,
    retryNotesEditorPanel,
    retryNotesTextControl,
    type LoadedNotesEditorPanel,
    type LoadedNotesTextControl,
  } from "./notes-editor-component-registry";

  let {
    onAddBelow,
    onTurnInto,
    canSetColor,
    currentColor,
    onColorSelect,
    onCopyLink,
    onDuplicate,
    onComment,
    commentCount = 0,
    unreadCommentCount = 0,
    onMoveUp,
    onMoveDown,
    moveTargets,
    onMoveToPage,
    onDelete,
    onDragStart,
    onDragEnd,
    visible = false,
    onMenuOpenChange,
  }: {
    onAddBelow: (command?: NotesBlockInsertCommand) => void;
    onTurnInto: () => void;
    canSetColor: boolean;
    currentColor: NotesColor;
    onColorSelect: (color: NotesColor) => void;
    onCopyLink: () => Promise<void> | void;
    onDuplicate: () => void;
    onComment: () => void;
    commentCount?: number;
    unreadCommentCount?: number;
    onMoveUp: () => void;
    onMoveDown: () => void;
    moveTargets: NotesMoveToPageTarget[];
    onMoveToPage: (pageId: string) => void;
    onDelete: () => void;
    onDragStart: (event: DragEvent) => void;
    onDragEnd: () => void;
    visible?: boolean;
    onMenuOpenChange?: (open: boolean) => void;
  } = $props();

  const { t } = getLocalization();
  let menuOpen = $state(false);
  let insertMenuOpen = $state(false);
  let moveMenuOpen = $state(false);
  let addButton: HTMLButtonElement | null = $state(null);
  let actionButton: HTMLButtonElement | null = $state(null);
  let insertMenuTriggerRect = $state<NotesBlockInsertMenuRect | null>(null);
  let actionMenuTriggerRect = $state<NotesBlockInsertMenuRect | null>(null);
  let copyLinkStatus = $state<"idle" | "copied" | "failed">("idle");
  let copyLinkTimer: ReturnType<typeof setTimeout> | null = null;
  let destinationPickerLoadState = $state<LazyComponentLoadState<
    "destination-picker",
    LoadedNotesEditorPanel
  > | null>(null);
  let insertMenuLoadState = $state<LazyComponentLoadState<
    "block-insert-menu",
    LoadedNotesTextControl
  > | null>(null);
  const visibleCommentCount = $derived(unreadCommentCount > 0 ? unreadCommentCount : commentCount);
  const visibleCommentLabel = $derived(
    unreadCommentCount > 0
      ? t("notes.blockUnreadCommentsCount", unreadCommentCount, commentCount)
      : t("notes.blockCommentsCount", commentCount),
  );
  const actionMenuStyle = $derived(
    actionMenuTriggerRect && typeof window !== "undefined"
      ? notesBlockHandleActionMenuStyle({
        triggerRect: actionMenuTriggerRect,
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
      })
      : "",
  );
  const anyMenuOpen = $derived(menuOpen || insertMenuOpen || moveMenuOpen);

  onDestroy(() => {
    if (copyLinkTimer) clearTimeout(copyLinkTimer);
  });

  $effect(() => {
    onMenuOpenChange?.(anyMenuOpen);
  });

  $effect(() => {
    if (!insertMenuOpen || typeof window === "undefined") return;
    const update = () => {
      updateInsertMenuTriggerRect();
    };
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  });

  $effect(() => {
    if (!menuOpen || typeof window === "undefined") return;
    const update = () => {
      updateActionMenuTriggerRect();
    };
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  });

  function currentMenuState(): NotesBlockHandleMenuState {
    return {
      menuOpen,
      insertMenuOpen,
      moveMenuOpen,
    };
  }

  function applyMenuState(state: NotesBlockHandleMenuState): void {
    menuOpen = state.menuOpen;
    insertMenuOpen = state.insertMenuOpen;
    moveMenuOpen = state.moveMenuOpen;
  }

  function closeMenus(): void {
    applyMenuState(CLOSED_NOTES_BLOCK_HANDLE_MENUS);
  }

  function runAction(actionType: NotesBlockHandleAction, action: () => void): void {
    applyMenuState(notesBlockHandleMenuStateAfterAction(currentMenuState(), actionType));
    action();
  }

  function insertBlock(command: NotesBlockInsertCommand): void {
    applyMenuState(CLOSED_NOTES_BLOCK_HANDLE_MENUS);
    onAddBelow(command);
  }

  function updateInsertMenuTriggerRect(): void {
    const rect = addButton?.getBoundingClientRect();
    insertMenuTriggerRect = rect
      ? {
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
        left: rect.left,
      }
      : null;
  }

  function updateActionMenuTriggerRect(): void {
    const rect = actionButton?.getBoundingClientRect();
    actionMenuTriggerRect = rect
      ? {
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
        left: rect.left,
      }
      : null;
  }

  function toggleInsertMenu(): void {
    const state = notesBlockHandleMenuStateAfterToggle(currentMenuState(), "insert");
    applyMenuState(state);
    if (state.insertMenuOpen) updateInsertMenuTriggerRect();
  }

  function toggleActionMenu(): void {
    const state = notesBlockHandleMenuStateAfterToggle(currentMenuState(), "actions");
    applyMenuState(state);
    if (state.menuOpen) updateActionMenuTriggerRect();
  }

  function toggleMoveMenu(): void {
    applyMenuState(notesBlockHandleMenuStateAfterToggle(currentMenuState(), "move"));
  }

  function moveToPage(pageId: string): void {
    runAction("move_to_page", () => onMoveToPage(pageId));
  }

  function moveToPageTarget(targetKey: string): void {
    const target = moveTargets.find((candidate) => candidate.key === targetKey);
    if (!target) return;
    moveToPage(target.id);
  }

  function selectColor(color: NotesColor): void {
    applyMenuState(notesBlockHandleMenuStateAfterAction(currentMenuState(), "color"));
    onColorSelect(color);
  }

  function resetCopyLinkStatusLater(): void {
    if (copyLinkTimer) clearTimeout(copyLinkTimer);
    copyLinkTimer = setTimeout(() => {
      copyLinkStatus = "idle";
      copyLinkTimer = null;
    }, 2_000);
  }

  async function copyLinkToBlock(): Promise<void> {
    applyMenuState(notesBlockHandleMenuStateAfterAction(currentMenuState(), "copy_link"));
    try {
      await onCopyLink();
      copyLinkStatus = "copied";
    } catch (error) {
      console.warn("copy notes block link failed", error);
      copyLinkStatus = "failed";
    }
    resetCopyLinkStatusLater();
  }

  function colorLabel(color: NotesColor): string {
    switch (color) {
      case "default":
        return t("notes.blockColor.default");
      case "gray":
        return t("notes.blockColor.gray");
      case "brown":
        return t("notes.blockColor.brown");
      case "orange":
        return t("notes.blockColor.orange");
      case "yellow":
        return t("notes.blockColor.yellow");
      case "green":
        return t("notes.blockColor.green");
      case "blue":
        return t("notes.blockColor.blue");
      case "purple":
        return t("notes.blockColor.purple");
      case "pink":
        return t("notes.blockColor.pink");
      case "red":
        return t("notes.blockColor.red");
      case "gray_background":
        return t("notes.blockColor.grayBackground");
      case "brown_background":
        return t("notes.blockColor.brownBackground");
      case "orange_background":
        return t("notes.blockColor.orangeBackground");
      case "yellow_background":
        return t("notes.blockColor.yellowBackground");
      case "green_background":
        return t("notes.blockColor.greenBackground");
      case "blue_background":
        return t("notes.blockColor.blueBackground");
      case "purple_background":
        return t("notes.blockColor.purpleBackground");
      case "pink_background":
        return t("notes.blockColor.pinkBackground");
      case "red_background":
        return t("notes.blockColor.redBackground");
    }
  }
  function requestDestinationPicker(retry = false): void {
    if (!retry && destinationPickerLoadState) return;
    const loadingState = beginLazyComponentLoad(destinationPickerLoadState, "destination-picker");
    destinationPickerLoadState = loadingState;
    const request = retry
      ? retryNotesEditorPanel("destination-picker")
      : loadNotesEditorPanel("destination-picker");
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
      console.error("load Notes block destination picker failed", error);
    });
  }

  function requestInsertMenu(retry = false): void {
    if (!retry && insertMenuLoadState) return;
    const loadingState = beginLazyComponentLoad(insertMenuLoadState, "block-insert-menu");
    insertMenuLoadState = loadingState;
    const request = retry
      ? retryNotesTextControl("block-insert-menu")
      : loadNotesTextControl("block-insert-menu");
    void request.then((component) => {
      if (!insertMenuLoadState) return;
      insertMenuLoadState = resolveLazyComponentLoad(
        insertMenuLoadState,
        "block-insert-menu",
        loadingState.requestId,
        component,
      );
    }).catch((error: unknown) => {
      if (!insertMenuLoadState) return;
      insertMenuLoadState = rejectLazyComponentLoad(
        insertMenuLoadState,
        "block-insert-menu",
        loadingState.requestId,
        error,
      );
      console.error("load Notes block insert menu failed", error);
    });
  }

  $effect(() => {
    if (moveMenuOpen) requestDestinationPicker();
    if (insertMenuOpen) requestInsertMenu();
  });
</script>

<div
  class="notes-block-handle relative mt-1.5 flex shrink-0 items-center justify-end gap-0.5"
  class:notes-block-handle-visible={visible}
  class:notes-block-handle-has-comments={commentCount > 0}
  class:notes-block-handle-menu-open={anyMenuOpen}
  role="toolbar"
  aria-label={t("notes.blockActions")}
  data-notes-block-selection-zone
  use:dismissOnOutside={{ enabled: anyMenuOpen, onDismiss: closeMenus }}
>
  <button
    bind:this={addButton}
    class="notes-block-handle-button flex items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
    type="button"
    aria-label={t("notes.addBlockBelow")}
    data-app-tooltip={t("notes.addBlockBelow")}
    aria-expanded={insertMenuOpen}
    onclick={toggleInsertMenu}
  >
    <Plus class="size-3.5" />
  </button>
  <button
    bind:this={actionButton}
    class="notes-block-handle-button flex items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
    type="button"
    aria-label={t("notes.blockActions")}
    data-app-tooltip={t("notes.blockActions")}
    draggable="true"
    ondragstart={(event) => {
      applyMenuState(CLOSED_NOTES_BLOCK_HANDLE_MENUS);
      onDragStart(event);
    }}
    ondragend={onDragEnd}
    aria-expanded={menuOpen}
    onclick={toggleActionMenu}
  >
    <GripVertical class="size-3.5" />
  </button>
  {#if commentCount > 0}
    <button
      class={`notes-block-handle-comment-button flex items-center justify-center gap-0.5 rounded text-muted-foreground hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
        unreadCommentCount > 0 ? "bg-primary/10 text-primary" : ""
      }`}
      type="button"
      aria-label={visibleCommentLabel}
      data-app-tooltip={visibleCommentLabel}
      onclick={() => runAction("comment", onComment)}
    >
      <MessageSquare class="size-3.5 shrink-0" />
      <span class="min-w-0 text-[0.65rem] font-medium leading-none">{visibleCommentCount}</span>
    </button>
  {/if}

  {#if insertMenuOpen}
    {#if insertMenuLoadState?.status === "ready" && insertMenuLoadState.component.kind === "block-insert-menu"}
      {@const NotesBlockInsertMenu = insertMenuLoadState.component.component}
      <NotesBlockInsertMenu triggerRect={insertMenuTriggerRect} onSelect={insertBlock} />
    {:else if insertMenuLoadState?.status === "failed"}
      <button class="min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestInsertMenu(true)}>{t("common.retry")}</button>
    {/if}
  {/if}

  {#if menuOpen}
    <div
      class="z-50 overflow-auto rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg"
      style={actionMenuStyle}
      role="menu"
      tabindex="-1"
      data-app-floating-surface
      onmousedown={(event) => {
        const target = event.target;
        if (!(target instanceof HTMLInputElement)) event.preventDefault();
      }}
    >
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => runAction("turn_into", onTurnInto)}
      >
        <Pilcrow class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.turnInto")}</span>
      </button>
      {#if canSetColor}
        <div class="my-1 border-t border-border"></div>
        <div class="px-2.5 pb-1 pt-1 text-[0.7rem] font-medium text-muted-foreground">
          {t("notes.color")}
        </div>
        {#each NOTES_TEXT_COLORS as color}
          <button
            class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
            type="button"
            role="menuitemradio"
            aria-checked={currentColor === color}
            onclick={() => selectColor(color)}
          >
            <span
              class="notes-color-swatch"
              style={notesBlockColorSwatchStyle(color)}
              aria-hidden="true"
            >
              A
            </span>
            <span class="min-w-0 flex-1 truncate">{colorLabel(color)}</span>
            {#if currentColor === color}
              <Check class="size-3.5 shrink-0" />
            {/if}
          </button>
        {/each}
        <div class="px-2.5 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
          {t("notes.backgroundColor")}
        </div>
        {#each NOTES_BACKGROUND_COLORS as color}
          <button
            class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
            type="button"
            role="menuitemradio"
            aria-checked={currentColor === color}
            onclick={() => selectColor(color)}
          >
            <span
              class="notes-color-swatch"
              style={notesBlockColorSwatchStyle(color)}
              aria-hidden="true"
            >
              A
            </span>
            <span class="min-w-0 flex-1 truncate">{colorLabel(color)}</span>
            {#if currentColor === color}
              <Check class="size-3.5 shrink-0" />
            {/if}
          </button>
        {/each}
        <div class="my-1 border-t border-border"></div>
      {/if}
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => {
          void copyLinkToBlock();
        }}
      >
        {#if copyLinkStatus === "copied"}
          <Check class="size-4 shrink-0" />
        {:else}
          <LinkIcon class="size-4 shrink-0" />
        {/if}
        <span class="min-w-0 truncate">
          {#if copyLinkStatus === "copied"}
            {t("notes.blockLinkCopied")}
          {:else if copyLinkStatus === "failed"}
            {t("notes.copyBlockLinkFailed")}
          {:else}
            {t("notes.copyBlockLink")}
          {/if}
        </span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => runAction("duplicate", onDuplicate)}
      >
        <Copy class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.duplicateBlock")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => runAction("comment", onComment)}
      >
        <MessageSquare class="size-4 shrink-0" />
        <span class="min-w-0 flex-1 truncate">{t("notes.commentBlock")}</span>
        {#if commentCount > 0}
          <span class="shrink-0 text-[0.733333rem] text-muted-foreground">
            {#if unreadCommentCount > 0}
              {t("notes.unreadCommentShortCount", unreadCommentCount)}
            {:else}
              {commentCount}
            {/if}
          </span>
        {/if}
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => runAction("move_up", onMoveUp)}
      >
        <ArrowUp class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.moveBlockUp")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        onclick={() => runAction("move_down", onMoveDown)}
      >
        <ArrowDown class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.moveBlockDown")}</span>
      </button>
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
        type="button"
        role="menuitem"
        aria-expanded={moveMenuOpen}
        onclick={toggleMoveMenu}
      >
        <FolderInput class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.moveBlockToPage")}</span>
      </button>
      {#if moveMenuOpen}
        <div class="border-y border-border bg-muted/25 py-1" role="group" aria-label={t("notes.moveBlockToPage")}>
          {#if destinationPickerLoadState?.status === "ready" && destinationPickerLoadState.component.kind === "destination-picker"}
            {@const NotesDestinationPickerList = destinationPickerLoadState.component.component}
            <NotesDestinationPickerList
              targets={moveTargets}
              searchLabel={t("notes.moveDestinationSearch")}
              searchPlaceholder={t("notes.moveDestinationSearchPlaceholder")}
              recentLabel={t("notes.recentDestinations")}
              pagesLabel={t("notes.allPages")}
              emptyLabel={t("notes.noMoveTargets")}
              optionLabel={(target) => t("notes.moveBlockToPageTarget", target.title)}
              onSelect={moveToPageTarget}
              onClose={() => {
                moveMenuOpen = false;
              }}
            />
          {:else if destinationPickerLoadState?.status === "failed"}
            <button class="m-2 min-h-8 rounded-md border border-border px-2 text-[0.8rem] hover:bg-accent" type="button" onclick={() => requestDestinationPicker(true)}>{t("common.retry")}</button>
          {:else}
            <div class="p-2 text-[0.8rem] text-muted-foreground" aria-busy="true">{t("common.loading")}</div>
          {/if}
        </div>
      {/if}
      <button
        class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] text-destructive hover:bg-accent"
        type="button"
        role="menuitem"
        onclick={() => runAction("delete", onDelete)}
      >
        <Trash2 class="size-4 shrink-0" />
        <span class="min-w-0 truncate">{t("notes.deleteBlock")}</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .notes-block-handle {
    inline-size: 2.5rem;
    z-index: 10;
  }

  .notes-block-handle-menu-open {
    z-index: 50;
  }

  .notes-block-handle-has-comments {
    inline-size: 4.5rem;
  }

  .notes-block-handle-button {
    block-size: 1.25rem;
    inline-size: 1.25rem;
    opacity: 0;
    pointer-events: none;
    visibility: hidden;
  }

  .notes-block-handle-visible .notes-block-handle-button,
  .notes-block-handle-menu-open .notes-block-handle-button {
    opacity: 1;
    pointer-events: auto;
    visibility: visible;
  }

  .notes-block-handle-comment-button {
    block-size: 1.25rem;
    min-inline-size: 1.75rem;
    padding-inline: 0.2rem;
  }

  .notes-color-swatch {
    display: inline-flex;
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--notes-color-swatch-border);
    border-radius: 0.25rem;
    background: var(--notes-color-swatch-bg);
    color: var(--notes-color-swatch-fg);
    font-size: 0.65rem;
    font-weight: 600;
    line-height: 1;
  }

  @media (any-pointer: coarse), (max-width: 420px) {
    .notes-block-handle {
      inline-size: 3.25rem;
    }

    .notes-block-handle-has-comments {
      inline-size: 5.25rem;
    }

    .notes-block-handle-button {
      block-size: 1.5rem;
      inline-size: 1.5rem;
    }

    .notes-block-handle-comment-button {
      block-size: 1.5rem;
      min-inline-size: 2rem;
    }
  }
</style>
