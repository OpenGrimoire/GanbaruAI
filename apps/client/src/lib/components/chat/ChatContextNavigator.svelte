<script lang="ts">
  import { tick } from "svelte";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import Search from "@lucide/svelte/icons/search";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import type {
    ChatThreadShellRead,
    ProjectWorkingFolderId,
    ProjectWorkingFolderRead,
  } from "$lib/chat/contracts";
  import {
    chatNavigationFolders,
    siblingChatThreads,
    threadStatus,
  } from "$lib/chat/shell-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { COMPACT_IDENTITY_ICON_STROKE_WIDTH } from "$lib/icon-sizing";
  import { isPointerAimingAtSubmenu, type MenuAimPoint } from "$lib/projects/menu-aim";
  import {
    projectPickerBridgeFrameStyle,
    projectPickerMenuAimRect,
    projectPickerPanelFrameStyle,
    projectPickerPanelHeight,
    projectPickerPointerPoint,
    projectPickerSubpanelAimOrigin,
    projectPickerSubpanelGeometry,
    projectPickerSubpanelSide,
  } from "$lib/projects/project-picker-panels";
  import { getChat } from "$lib/stores/chat.svelte";
  import { cn } from "$lib/utils";

  type NavigatorMode = "folders" | "threads";

  let {
    mode,
    projectId,
    workingFolderId,
    panelMaxHeight,
    onFolderSelected,
    onThreadSelected,
    onNewChat,
    onAddFolder,
  }: {
    mode: NavigatorMode;
    projectId: string | null;
    workingFolderId: ProjectWorkingFolderId | null;
    panelMaxHeight: number;
    onFolderSelected: (workingFolderId: ProjectWorkingFolderId) => void;
    onThreadSelected: (thread: ChatThreadShellRead) => void;
    onNewChat: (workingFolderId: ProjectWorkingFolderId) => void;
    onAddFolder: () => void;
  } = $props();

  const chat = getChat();
  const { t } = getLocalization();
  const panelGap = 4;
  const rowHeight = 32;
  const panelPadding = 8;
  const footerHeight = 44;
  const headerHeight = 40;
  const identityIconStrokeWidth = COMPACT_IDENTITY_ICON_STROKE_WIDTH;
  let search = $state("");
  let subpanelSearch = $state("");
  let rootElement = $state<HTMLDivElement | null>(null);
  let activeFolderId = $state<ProjectWorkingFolderId | null>(null);
  let activeFolderAnchor = $state<HTMLElement | null>(null);
  let subpanelElement = $state<HTMLDivElement | null>(null);
  let subpanelHeaderElement = $state<HTMLDivElement | null>(null);
  let subpanelFooterElement = $state<HTMLDivElement | null>(null);
  let mainScrollElement = $state<HTMLDivElement | null>(null);
  let subpanelScrollElement = $state<HTMLDivElement | null>(null);
  let subpanelScrollContentElement = $state<HTMLDivElement | null>(null);
  let subpanelBridgeElement = $state<HTMLDivElement | null>(null);
  let subpanelStyle = $state("");
  let subpanelBridgeStyle = $state("");
  let menuAimOrigin: MenuAimPoint | null = null;

  const normalizedSearch = $derived(search.trim().toLocaleLowerCase());
  const normalizedSubpanelSearch = $derived(subpanelSearch.trim().toLocaleLowerCase());
  const folders = $derived(chatNavigationFolders(chat.workingFolders, projectId));
  const visibleFolders = $derived(folders.filter((entry) => (
    !normalizedSearch
      || entry.workingFolder.displayName.toLocaleLowerCase().includes(normalizedSearch)
  )));
  const directThreads = $derived(filterThreads(siblingChatThreads(
    chat.activeThreads,
    projectId,
    workingFolderId,
  ), normalizedSearch));
  const selectedNavigationFolder = $derived(folders.find((entry) => (
    entry.workingFolder.id === workingFolderId
  )) ?? null);
  const activeFolder = $derived(folders.find((entry) => (
    entry.workingFolder.id === activeFolderId
  )) ?? null);
  const activeFolderThreads = $derived(filterThreads(siblingChatThreads(
    chat.activeThreads,
    projectId,
    activeFolderId,
  ), normalizedSubpanelSearch));
  const mainItemCount = $derived(mode === "folders" ? visibleFolders.length : directThreads.length);
  const mainPanelHeight = $derived(projectPickerPanelHeight({
    headerHeight,
    footerHeight,
    listHeight: panelPadding + rowHeight * Math.max(1, mainItemCount),
    maxHeight: panelMaxHeight,
    visibleRows: null,
    listPadding: panelPadding,
    rowHeight,
  }));

  function filterThreads(
    threads: readonly ChatThreadShellRead[],
    query: string,
  ): ChatThreadShellRead[] {
    if (!query) return [...threads];
    return threads.filter((thread) => thread.title.toLocaleLowerCase().includes(query));
  }

  function navigatorBounds() {
    const rect = rootElement?.closest(".chat-workspace")?.getBoundingClientRect();
    return rect
      ? { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom }
      : { left: 0, right: window.innerWidth, top: 0, bottom: window.innerHeight };
  }

  function updateSubpanelGeometry(): void {
    if (!rootElement || !activeFolderAnchor || !activeFolderId) return;
    const anchorRect = activeFolderAnchor.getBoundingClientRect();
    const panelRect = rootElement.getBoundingClientRect();
    const measuredListHeight = subpanelScrollContentElement?.scrollHeight;
    const contentListHeight = measuredListHeight && measuredListHeight > 0
      ? measuredListHeight + panelPadding
      : undefined;
    const measuredHeaderHeight = subpanelHeaderElement?.offsetHeight ?? headerHeight;
    const measuredFooterHeight = subpanelFooterElement?.offsetHeight ?? footerHeight;
    const geometry = projectPickerSubpanelGeometry({
      anchorRect,
      panelRect,
      bounds: navigatorBounds(),
      gap: panelGap,
      footerHeight: measuredHeaderHeight + measuredFooterHeight,
      projectCount: Math.max(1, activeFolderThreads.length),
      visibleRows: null,
      listHeight: contentListHeight,
      listPadding: panelPadding,
      rowHeight,
    });
    subpanelStyle = projectPickerPanelFrameStyle(geometry.panel);
    subpanelBridgeStyle = projectPickerBridgeFrameStyle(geometry.bridge);
  }

  function showFolderThreads(
    folder: ProjectWorkingFolderRead,
    anchor: HTMLElement,
  ): void {
    if (activeFolderId !== folder.workingFolder.id) subpanelSearch = "";
    activeFolderId = folder.workingFolder.id;
    activeFolderAnchor = anchor;
    menuAimOrigin = projectPickerSubpanelAimOrigin(anchor.getBoundingClientRect());
    void tick().then(updateSubpanelGeometry);
  }

  function handleFolderPointerMove(
    folder: ProjectWorkingFolderRead,
    event: PointerEvent,
  ): void {
    const anchor = event.currentTarget;
    if (!(anchor instanceof HTMLElement)) return;
    if (activeFolderId === folder.workingFolder.id) {
      menuAimOrigin = projectPickerPointerPoint(event);
      return;
    }
    if (subpanelElement && menuAimOrigin && activeFolderAnchor) {
      const aiming = isPointerAimingAtSubmenu({
        origin: menuAimOrigin,
        point: projectPickerPointerPoint(event),
        submenu: projectPickerMenuAimRect(subpanelElement.getBoundingClientRect()),
        side: projectPickerSubpanelSide(
          activeFolderAnchor.getBoundingClientRect(),
          subpanelElement.getBoundingClientRect(),
        ),
      });
      if (aiming) return;
    }
    showFolderThreads(folder, anchor);
  }

  function closeSubpanel(event?: PointerEvent): void {
    const target = event?.relatedTarget;
    if (
      target instanceof Node
      && (rootElement?.contains(target)
        || subpanelElement?.contains(target)
        || subpanelBridgeElement?.contains(target))
    ) {
      return;
    }
    activeFolderId = null;
    activeFolderAnchor = null;
    subpanelSearch = "";
    menuAimOrigin = null;
  }

  function chooseFolder(folder: ProjectWorkingFolderRead): void {
    closeSubpanel();
    onFolderSelected(folder.workingFolder.id);
  }

  function chooseThread(thread: ChatThreadShellRead): void {
    closeSubpanel();
    onThreadSelected(thread);
  }

  function folderStatus(folder: ProjectWorkingFolderRead): string | null {
    if (folder.bindingStatus === "available") return null;
    if (folder.bindingStatus === "repository_mismatch") return t("chat.explorer.repositoryMismatch");
    if (folder.bindingStatus === "missing") return t("chat.explorer.folderMissing");
    return t("chat.explorer.folderUnbound");
  }

  $effect(() => {
    const folderId = activeFolderId;
    const threadCount = activeFolderThreads.length;
    const query = subpanelSearch;
    void threadCount;
    void query;
    if (!folderId) return;
    requestAnimationFrame(updateSubpanelGeometry);
  });
</script>

<div
  bind:this={rootElement}
  class="chat-context-navigator flex min-h-0 w-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
  style={`height:${mainPanelHeight}px;max-height:${mainPanelHeight}px`}
  role="menu"
  tabindex="-1"
  data-chat-context-navigator={mode}
  onpointerleave={closeSubpanel}
>
  <div class="shrink-0 px-1.5 pb-0.5 pt-1.5">
    <label class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 px-2">
      <Search size={13} strokeWidth={identityIconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
      <input
        bind:value={search}
        class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground outline-none placeholder:text-popover-foreground/45"
        placeholder={mode === "folders" ? t("chat.explorer.searchFolders") : t("chat.search")}
      />
    </label>
  </div>
  <div class="relative min-h-0 flex-1">
    <div bind:this={mainScrollElement} class="h-full min-h-0 overflow-y-auto px-1 py-0.5" data-chat-navigator-list>
      {#if mode === "folders"}
        {#if visibleFolders.length === 0}
          <p class="px-2 py-2 text-[0.8rem] text-popover-foreground/60">{t("chat.explorer.noFolders")}</p>
        {:else}
          {#each visibleFolders as folder (folder.workingFolder.id)}
            {@const status = folderStatus(folder)}
            <button
              type="button"
              class={cn(
                "grid min-h-8 w-full grid-cols-[1rem_minmax(0,1fr)_auto] items-center gap-2 rounded-md px-2 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground",
                activeFolderId === folder.workingFolder.id && "bg-accent text-accent-foreground",
              )}
              onpointerenter={(event) => showFolderThreads(folder, event.currentTarget)}
              onpointermove={(event) => handleFolderPointerMove(folder, event)}
              onfocus={(event) => showFolderThreads(folder, event.currentTarget)}
              onclick={() => chooseFolder(folder)}
              data-chat-folder-id={folder.workingFolder.id}
            >
              {#if folder.workingFolder.id === workingFolderId}
                <FolderOpen size={14} strokeWidth={identityIconStrokeWidth} />
              {:else}
                <Folder size={14} strokeWidth={identityIconStrokeWidth} />
              {/if}
              <span class="min-w-0 truncate">{folder.workingFolder.displayName}</span>
              <span class="flex min-w-0 items-center gap-1 text-[0.666667rem] text-popover-foreground/55">
                {#if status}<span class="max-w-24 truncate">{status}</span>{/if}
                <ChevronRight size={13} strokeWidth={identityIconStrokeWidth} />
              </span>
            </button>
          {/each}
        {/if}
      {:else if directThreads.length === 0}
        <p class="px-2 py-2 text-[0.8rem] text-popover-foreground/60">{t("chat.explorer.noChatsInFolder")}</p>
      {:else}
        {#each directThreads as thread (thread.id)}
          <button
            type="button"
            class={cn(
              "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground",
              thread.id === chat.selectedThreadId && "font-semibold text-foreground",
            )}
            onclick={() => chooseThread(thread)}
            data-chat-thread-id={thread.id}
          >
            <MessageSquare size={13} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
            <span class="min-w-0 flex-1 truncate">{thread.title}</span>
          </button>
        {/each}
      {/if}
    </div>
    <CalendarScrollbar scrollContainer={mainScrollElement ?? undefined} stickyTop={4} stickyBottom={4} wheelPassthrough />
  </div>
  {#if mode === "folders" || workingFolderId}
    <div class="relative shrink-0 bg-popover p-1.5">
      <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
      {#if mode === "threads" && workingFolderId}
      <button type="button" class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] hover:bg-accent hover:text-accent-foreground disabled:cursor-not-allowed disabled:opacity-50" disabled={selectedNavigationFolder?.bindingStatus !== "available"} onclick={() => onNewChat(workingFolderId)} data-chat-new-folder-id={workingFolderId}>
        <MessageSquarePlus size={13} strokeWidth={identityIconStrokeWidth} />
        <span>{t("chat.newChat")}</span>
      </button>
      {:else if mode === "folders"}
        <button type="button" class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] hover:bg-accent hover:text-accent-foreground" onclick={onAddFolder} data-chat-add-folder>
          <FolderPlus size={13} strokeWidth={identityIconStrokeWidth} />
          <span>{t("projects.settings.workingFolders.addExisting")}</span>
        </button>
      {/if}
    </div>
  {/if}
</div>

{#if mode === "folders" && activeFolder && activeFolderAnchor}
  <div
    bind:this={subpanelBridgeElement}
    class="fixed z-81 bg-transparent"
    style={subpanelBridgeStyle}
    aria-hidden="true"
    onpointerleave={closeSubpanel}
  ></div>
  <div
    bind:this={subpanelElement}
    class="chat-context-subpanel fixed z-81 flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
    style={subpanelStyle}
    role="menu"
    tabindex="-1"
    data-chat-folder-thread-panel={activeFolder.workingFolder.id}
    onpointerleave={closeSubpanel}
  >
    <div bind:this={subpanelHeaderElement} class="shrink-0 px-1.5 pb-0.5 pt-1.5">
      <label class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 px-2">
        <Search size={13} strokeWidth={identityIconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
        <input bind:value={subpanelSearch} class="min-w-0 flex-1 bg-transparent text-[0.8rem] outline-none placeholder:text-popover-foreground/45" placeholder={t("chat.search")} />
      </label>
    </div>
    <div class="relative min-h-0 flex-1">
      <div bind:this={subpanelScrollElement} class="h-full min-h-0 overflow-y-auto px-1 py-0.5" data-chat-navigator-list>
        <div bind:this={subpanelScrollContentElement}>
          {#if activeFolderThreads.length === 0}
            <p class="px-2 py-2 text-[0.8rem] text-popover-foreground/60">{t("chat.explorer.noChatsInFolder")}</p>
          {:else}
            {#each activeFolderThreads as thread (thread.id)}
              {@const status = threadStatus(thread)}
              <button type="button" class={cn("flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground", thread.id === chat.selectedThreadId && "font-semibold text-foreground")} onclick={() => chooseThread(thread)} data-chat-thread-id={thread.id}>
                {#if status === "working"}
                  <LoaderCircle size={12} strokeWidth={identityIconStrokeWidth} class="shrink-0 animate-spin" />
                {:else if status === "error"}
                  <CircleAlert size={12} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
                {:else if status === "unread" || status.startsWith("waiting")}
                  <CircleDot size={12} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
                {:else}
                  <MessageSquare size={12} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
                {/if}
                <span class="min-w-0 flex-1 truncate">{thread.title}</span>
              </button>
            {/each}
          {/if}
        </div>
      </div>
      <CalendarScrollbar scrollContainer={subpanelScrollElement ?? undefined} stickyTop={4} stickyBottom={4} wheelPassthrough />
    </div>
    <div bind:this={subpanelFooterElement} class="relative shrink-0 bg-popover p-1.5">
      <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
      <button type="button" class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] hover:bg-accent hover:text-accent-foreground disabled:cursor-not-allowed disabled:opacity-50" disabled={activeFolder.bindingStatus !== "available"} onclick={() => onNewChat(activeFolder.workingFolder.id)} data-chat-new-folder-id={activeFolder.workingFolder.id}>
        <MessageSquarePlus size={13} strokeWidth={identityIconStrokeWidth} />
        <span>{t("chat.newChat")}</span>
      </button>
    </div>
  </div>
{/if}
