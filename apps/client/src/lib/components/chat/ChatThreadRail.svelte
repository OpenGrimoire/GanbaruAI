<script lang="ts">
  import { onMount, tick } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ChevronsDownUp from "@lucide/svelte/icons/chevrons-down-up";
  import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
  import ChevronsRight from "@lucide/svelte/icons/chevrons-right";
  import ChevronsUpDown from "@lucide/svelte/icons/chevrons-up-down";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Folder from "@lucide/svelte/icons/folder";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import type {
    ChatThreadShellRead,
    ProjectWorkingFolderId,
    ProjectWorkingFolderRead,
  } from "$lib/chat/contracts";
  import {
    buildChatRailModel,
    filterThreadTitles,
    partitionThreadSearchResults,
    threadStatus,
  } from "$lib/chat/shell-model";
  import * as chatApi from "$lib/api/chat";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    COMPACT_IDENTITY_ICON_SIZE,
    COMPACT_IDENTITY_ICON_STROKE_WIDTH,
  } from "$lib/icon-sizing";
  import {
    notesRowContextMenuGeometry,
    notesRowContextMenuStyle,
  } from "$lib/notes/row-context-menu";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { openDetachedViewWindow } from "$lib/windows/detached";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ChatArchiveBrowser from "./ChatArchiveBrowser.svelte";

  let {
    expanded,
    showCollapsedStrip,
    onExpand,
    onCollapse,
  }: {
    expanded: boolean;
    showCollapsedStrip: boolean;
    onExpand: () => void;
    onCollapse: () => void;
  } = $props();

  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const explorerToolbarIconSize = 16;
  const explorerRowIconSize = COMPACT_IDENTITY_ICON_SIZE;
  const explorerIconStrokeWidth = COMPACT_IDENTITY_ICON_STROKE_WIDTH;
  let search = $state("");
  let searchOpen = $state(false);
  let remoteResults = $state<ChatThreadShellRead[]>([]);
  let showArchive = $state(false);
  let deleteThread = $state<ChatThreadShellRead | null>(null);
  let renameThreadId = $state<string | null>(null);
  let renameValue = $state("");
  let menuThread = $state<ChatThreadShellRead | null>(null);
  let menuStyle = $state("");
  let railError = $state<string | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let collapsedFolderIds = $state<string[]>([]);
  const localResults = $derived(filterThreadTitles(
    [...chat.activeThreads, ...chat.archivedThreads],
    search,
  ));
  const searchResults = $derived(search.trim() ? mergeResults(localResults, remoteResults) : []);
  const partitionedSearchResults = $derived(partitionThreadSearchResults(searchResults));
  const rail = $derived(buildChatRailModel(
    chat.workingFolders,
    chat.activeThreads,
    projects.selectedProjectId,
    chat.selectedWorkingFolderId,
    chat.selectedThreadId,
  ));
  const allFoldersExpanded = $derived(rail.folders.every((entry) => (
    !collapsedFolderIds.includes(entry.workingFolder.workingFolder.id)
  )));

  onMount(() => {
    const focusSearch = () => { void openSearch(); };
    window.addEventListener("ganbaru-ai:chat-focus-search", focusSearch);
    return () => window.removeEventListener("ganbaru-ai:chat-focus-search", focusSearch);
  });

  $effect(() => {
    const selectedFolderId = chat.selectedWorkingFolderId;
    if (!selectedFolderId || !collapsedFolderIds.includes(selectedFolderId)) return;
    collapsedFolderIds = collapsedFolderIds.filter((id) => id !== selectedFolderId);
  });

  $effect(() => {
    const query = search.trim();
    if (!query) {
      remoteResults = [];
      return;
    }
    const timer = window.setTimeout(() => {
      void chatApi.searchChatThreadTitles(query, null)
        .then((results) => {
          if (search.trim() === query) remoteResults = results;
        })
        .catch((error: unknown) => {
          railError = errorMessage(error);
        });
    }, 250);
    return () => window.clearTimeout(timer);
  });

  async function openSearch(): Promise<void> {
    searchOpen = true;
    await tick();
    searchInput?.focus();
  }

  function newChat(workingFolderId?: ProjectWorkingFolderId): void {
    const requested = workingFolderId
      ? chat.workingFolders.find((entry) => entry.workingFolder.id === workingFolderId)
      : chat.selectedWorkingFolder;
    const target = requested?.bindingStatus === "available" && requested.workingFolder.archivedAt === null
      ? requested
      : workingFolderId
        ? null
        : chat.workingFolders.find((entry) => (
            entry.workingFolder.projectId === projects.selectedProjectId
              && entry.workingFolder.kind === "managed"
              && entry.bindingStatus === "available"
          ));
    if (!target) {
      railError = t("chat.explorer.noAvailableFolder");
      return;
    }
    chat.newDraft(target.workingFolder.id);
  }

  function toggleFolder(workingFolderId: string): void {
    collapsedFolderIds = collapsedFolderIds.includes(workingFolderId)
      ? collapsedFolderIds.filter((id) => id !== workingFolderId)
      : [...collapsedFolderIds, workingFolderId];
  }

  function toggleAllFolders(): void {
    collapsedFolderIds = allFoldersExpanded
      ? rail.folders.map((entry) => entry.workingFolder.workingFolder.id)
      : [];
  }

  function folderStatus(folder: ProjectWorkingFolderRead): string | null {
    if (folder.workingFolder.archivedAt) return t("chat.explorer.folderArchived");
    if (folder.bindingStatus === "available") return null;
    if (folder.bindingStatus === "repository_mismatch") return t("chat.explorer.repositoryMismatch");
    if (folder.bindingStatus === "missing") return t("chat.explorer.folderMissing");
    return t("chat.explorer.folderUnbound");
  }

  function recoverFolder(folder: ProjectWorkingFolderRead): void {
    const title = t("chat.firstUse.locateFolder");
    runRailOperation(() => {
      if (folder.workingFolder.archivedAt) {
        return chat.restoreWorkingFolder(folder.workingFolder.id);
      }
      return folder.bindingStatus === "repository_mismatch"
        ? chat.rebindWorkingFolder(folder.workingFolder.id, title)
        : chat.locateWorkingFolder(folder.workingFolder.id, title);
    });
  }

  function threadContext(thread: ChatThreadShellRead): string {
    const project = projects.projectById(thread.projectId);
    const folder = chat.workingFolders.find((entry) => (
      entry.workingFolder.id === thread.workingFolderId
    ))?.workingFolder;
    return [project?.name, folder?.displayName].filter(Boolean).join(" / ");
  }

  function selectSearchResult(thread: ChatThreadShellRead): void {
    chat.selectThreadShell(thread);
    search = "";
    searchOpen = false;
    showArchive = false;
  }

  function beginRename(thread: ChatThreadShellRead): void {
    menuThread = null;
    renameThreadId = thread.id;
    renameValue = thread.title;
  }

  function openThreadMenu(thread: ChatThreadShellRead, clientX: number, clientY: number): void {
    menuStyle = notesRowContextMenuStyle(notesRowContextMenuGeometry({
      clientX,
      clientY,
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
    }));
    menuThread = thread;
  }

  function openThreadButtonMenu(event: MouseEvent, thread: ChatThreadShellRead): void {
    event.preventDefault();
    event.stopPropagation();
    const target = event.currentTarget;
    if (!(target instanceof HTMLElement)) return;
    const rect = target.getBoundingClientRect();
    openThreadMenu(thread, rect.right, rect.bottom + 4);
  }

  function openThreadContextMenu(event: MouseEvent, thread: ChatThreadShellRead): void {
    event.preventDefault();
    event.stopPropagation();
    openThreadMenu(thread, event.clientX, event.clientY);
  }

  function closeThreadMenuFromWindow(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Element)) return;
    if (target.closest("[data-chat-thread-menu], [data-chat-thread-menu-trigger]")) return;
    menuThread = null;
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key !== "Escape" || !menuThread) return;
    event.preventDefault();
    menuThread = null;
  }

  async function commitRename(thread: ChatThreadShellRead): Promise<void> {
    try {
      const title = renameValue.trim();
      if (title && title !== thread.title) await chat.renameThread(thread, title);
      renameThreadId = null;
    } catch (error: unknown) {
      railError = errorMessage(error);
    }
  }

  function statusLabel(thread: ChatThreadShellRead): string {
    const status = threadStatus(thread);
    if (status === "waiting_answer") return t("chat.status.waitingAnswer");
    if (status === "waiting_approval") return t("chat.status.waitingApproval");
    if (status === "working") return t("chat.status.working");
    if (status === "error") return t("chat.status.error");
    if (status === "unread") return t("chat.status.unread");
    if (status === "archived") return t("chat.status.archived");
    return t("chat.status.idle");
  }

  function mergeResults(
    local: ChatThreadShellRead[],
    remote: ChatThreadShellRead[],
  ): ChatThreadShellRead[] {
    const seen = new Set<string>();
    return [...local, ...remote].filter((thread) => (
      !seen.has(thread.id) && Boolean(seen.add(thread.id))
    ));
  }

  function confirmDelete(): void {
    const thread = deleteThread;
    if (!thread) return;
    deleteThread = null;
    runRailOperation(() => chat.deleteThread(thread));
  }

  function runMenuOperation(
    thread: ChatThreadShellRead,
    action: (selected: ChatThreadShellRead) => Promise<unknown>,
  ): void {
    menuThread = null;
    runRailOperation(() => action(thread));
  }

  function runRailOperation(action: () => Promise<unknown>): void {
    railError = null;
    void action().catch((error: unknown) => {
      railError = errorMessage(error);
    });
  }

  function copyThreadId(threadId: string): void {
    runRailOperation(() => navigator.clipboard.writeText(threadId));
  }

  function detach(thread: ChatThreadShellRead): void {
    chat.selectThread(thread.id);
    runRailOperation(() => openDetachedViewWindow("chat"));
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function rowKeydown(event: KeyboardEvent, thread: ChatThreadShellRead): void {
    if (event.key === "Enter") {
      event.preventDefault();
      chat.selectThread(thread.id);
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const rows = [...document.querySelectorAll<HTMLElement>("[data-chat-thread-id]")];
    const current = rows.indexOf(event.currentTarget as HTMLElement);
    const next = event.key === "Home"
      ? 0
      : event.key === "End"
        ? rows.length - 1
        : Math.max(0, Math.min(rows.length - 1, current + (event.key === "ArrowDown" ? 1 : -1)));
    rows[next]?.focus();
  }
</script>

<svelte:window onpointerdown={closeThreadMenuFromWindow} onkeydown={handleWindowKeydown} />

<aside class="chat-project-explorer relative h-full min-h-0 overflow-hidden" class:expanded aria-label={t("chat.explorer.label")}>
  <div class="chat-explorer-collapsed-rail absolute inset-y-0 left-0 z-10 w-11" class:visible={!expanded && showCollapsedStrip} aria-hidden={expanded || !showCollapsedStrip}>
    <div class="flex h-(--cal-header-row-h) items-center justify-center">
      <button type="button" class="chat-explorer-icon-button" aria-label={t("chat.openRail")} data-app-tooltip={t("chat.openRail")} onclick={onExpand}>
        <ChevronsRight size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} />
      </button>
    </div>
  </div>

  <div class="chat-explorer-content flex h-full min-h-0 w-64 min-w-64 flex-col" inert={!expanded}>
    <div class="flex h-(--cal-header-row-h) shrink-0 items-center gap-0.5 px-2">
      <button type="button" class="chat-explorer-icon-button" aria-label={t("chat.newChat")} data-app-tooltip={t("chat.newChat")} onclick={() => newChat()}><MessageSquarePlus size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
      <button type="button" class="chat-explorer-icon-button" aria-label={t("chat.archivedChats")} data-app-tooltip={t("chat.archivedChats")} onclick={() => { showArchive = true; }}><Archive size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
      <button type="button" class="chat-explorer-icon-button" aria-label={allFoldersExpanded ? t("chat.explorer.collapseFolders") : t("chat.explorer.expandFolders")} data-app-tooltip={allFoldersExpanded ? t("chat.explorer.collapseFolders") : t("chat.explorer.expandFolders")} onclick={toggleAllFolders}>
        {#if allFoldersExpanded}<ChevronsDownUp size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} />{:else}<ChevronsUpDown size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} />{/if}
      </button>
      <button type="button" class="chat-explorer-icon-button" class:active={searchOpen} aria-label={t("chat.search")} data-app-tooltip={t("chat.search")} onclick={() => void openSearch()}><Search size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
      <button type="button" class="chat-explorer-icon-button" aria-label={t("chat.collapseRail")} data-app-tooltip={t("chat.collapseRail")} onclick={onCollapse}><ChevronsLeft size={explorerToolbarIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
    </div>

    {#if searchOpen}
      <div class="shrink-0 px-2 pb-2">
        <label class="flex items-center gap-1.5 rounded-md bg-accent/50 px-2 py-1.5">
          <Search size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} class="shrink-0 text-muted-foreground" />
          <input bind:this={searchInput} class="min-w-0 flex-1 bg-transparent text-[0.8rem] outline-none placeholder:text-muted-foreground" type="search" bind:value={search} placeholder={t("chat.search")} />
          <button type="button" class="flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground" aria-label={t("chat.clearSearch")} onclick={() => { search = ""; searchOpen = false; }}><X size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
        </label>
      </div>
    {/if}
    {#if railError}<div role="alert" class="shrink-0 px-3 pb-2 text-[0.733333rem] text-destructive">{railError}</div>{/if}

    <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
      {#if search.trim()}
        {#if searchResults.length === 0}<p class="px-1 py-2 text-[0.8rem] text-muted-foreground">{t("chat.noSearchResults")}</p>{/if}
        {#if partitionedSearchResults.active.length > 0}
          <h2 class="px-1 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">{t("chat.activeChats")}</h2>
          {#each partitionedSearchResults.active as thread (thread.id)}
            <button type="button" class="chat-search-result" onclick={() => selectSearchResult(thread)}><MessageSquare size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /><span class="min-w-0 flex-1"><strong>{thread.title}</strong><small>{threadContext(thread)}</small></span></button>
          {/each}
        {/if}
        {#if partitionedSearchResults.archived.length > 0}
          <h2 class="px-1 pb-1 pt-3 text-[0.7rem] font-medium text-muted-foreground">{t("chat.archivedChats")}</h2>
          {#each partitionedSearchResults.archived as thread (thread.id)}
            <button type="button" class="chat-search-result" onclick={() => selectSearchResult(thread)}><Archive size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /><span class="min-w-0 flex-1"><strong>{thread.title}</strong><small>{threadContext(thread)}</small></span></button>
          {/each}
        {/if}
      {:else if rail.folders.length === 0}
        <p class="px-1 py-2 text-[0.8rem] text-muted-foreground">{t("chat.explorer.noFolders")}</p>
      {:else}
        {#each rail.folders as folderModel (folderModel.workingFolder.workingFolder.id)}
          {@const folder = folderModel.workingFolder}
          {@const folderId = folder.workingFolder.id}
          {@const collapsed = collapsedFolderIds.includes(folderId)}
          {@const status = folderStatus(folder)}
          <section class="mb-1">
            <div class="chat-folder-row group/folder" class:selected={folderModel.selected}>
              <button type="button" class="chat-folder-main" aria-expanded={!collapsed} onclick={() => toggleFolder(folderId)}>
                {#if collapsed}<Folder size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} />{:else}<FolderOpen size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} />{/if}
                <span class="min-w-0 flex-1 truncate">{folder.workingFolder.displayName}</span>
                {#if status}<span class="max-w-20 truncate text-[0.633333rem] text-muted-foreground">{status}</span>{/if}
              </button>
              {#if folder.bindingStatus === "available" && !folder.workingFolder.archivedAt}
                <button type="button" class="chat-folder-action" aria-label={t("chat.explorer.newChatInFolder", folder.workingFolder.displayName)} onclick={() => newChat(folderId)}><Plus size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
              {:else}
                <button type="button" class="chat-folder-action visible" aria-label={folder.workingFolder.archivedAt ? t("chat.restore") : t("chat.firstUse.locateFolder")} onclick={() => recoverFolder(folder)}><FolderOpen size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
              {/if}
            </div>
            {#if !collapsed}
              <div class="pl-4">
                {#if folderModel.hasDraft}
                  <div class="chat-thread-row selected">
                    <button type="button" class="chat-thread-main font-semibold" aria-current="page" onclick={() => newChat(folderId)}><MessageSquarePlus size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /><span>{t("chat.newChat")}</span></button>
                  </div>
                {/if}
                {#each folderModel.threads as thread (thread.id)}
                  {@const statusKind = threadStatus(thread)}
                  <div class="chat-thread-row group/thread" class:selected={chat.selectedThreadId === thread.id} role="group" oncontextmenu={(event) => openThreadContextMenu(event, thread)}>
                    {#if renameThreadId === thread.id}
                      <input class="mx-1 h-7 min-w-0 flex-1 rounded border border-ring bg-background px-1 text-xs" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(thread); if (event.key === "Escape") renameThreadId = null; }} onblur={() => void commitRename(thread)} />
                    {:else}
                      <button type="button" class="chat-thread-main" data-chat-thread-id={thread.id} title={statusLabel(thread)} onkeydown={(event) => rowKeydown(event, thread)} onclick={() => chat.selectThread(thread.id)}>
                        {#if statusKind === "working"}<LoaderCircle size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} class="animate-spin" />{:else if statusKind === "error"}<CircleAlert size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} />{:else if statusKind === "unread" || statusKind.startsWith("waiting")}<CircleDot size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} />{:else}<MessageSquare size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} />{/if}
                        <span>{thread.title}</span>
                      </button>
                    {/if}
                    <button type="button" class="chat-thread-menu-trigger" aria-label={t("chat.moreActions")} data-chat-thread-menu-trigger onclick={(event) => openThreadButtonMenu(event, thread)}><Ellipsis size={explorerRowIconSize} strokeWidth={explorerIconStrokeWidth} /></button>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {/each}
      {/if}
    </div>
  </div>

  {#if showArchive}
    <ChatArchiveBrowser onClose={() => { showArchive = false; }} onRestored={() => { showArchive = false; }} onDelete={(thread) => { deleteThread = thread; }} />
  {/if}
</aside>

{#if menuThread}
  {@const currentMenuThread = menuThread}
  <div class="chat-menu fixed z-90 overflow-y-auto" style={menuStyle} data-chat-thread-menu role="menu">
    <button type="button" role="menuitem" onclick={() => beginRename(currentMenuThread)}>{t("chat.rename")}</button>
    <button type="button" role="menuitem" onclick={() => runMenuOperation(currentMenuThread, (thread) => chat.setThreadRead(thread, Boolean(thread.unreadAt)))}>{currentMenuThread.unreadAt ? t("chat.markRead") : t("chat.markUnread")}</button>
    <button type="button" role="menuitem" onclick={() => { menuThread = null; detach(currentMenuThread); }}>{t("chat.detach")}</button>
    <button type="button" role="menuitem" onclick={() => { menuThread = null; copyThreadId(currentMenuThread.id); }}>{t("chat.copyThreadId")}</button>
    <button type="button" role="menuitem" onclick={() => runMenuOperation(currentMenuThread, (thread) => chat.openWorkingFolder(thread.workingFolderId))}>{t("chat.openFolder")}</button>
    <button type="button" role="menuitem" onclick={() => runMenuOperation(currentMenuThread, (thread) => chat.archiveThread(thread))}>{t("chat.archive")}</button>
    <button type="button" role="menuitem" class="text-destructive" onclick={() => { deleteThread = currentMenuThread; menuThread = null; }}>{t("chat.deletePermanently")}</button>
  </div>
{/if}

{#if deleteThread}<ConfirmDialog title={t("chat.deleteTitle")} message={t("chat.deleteMessage", deleteThread.title)} confirmLabel={t("chat.deletePermanently")} cancelLabel={t("chat.cancel")} onConfirm={confirmDelete} onCancel={() => { deleteThread = null; }} />{/if}

<style>
  .chat-project-explorer { width: 2.75rem; background: var(--cal-bg); transition: width 180ms cubic-bezier(0.2, 0, 0, 1); }
  .chat-project-explorer.expanded { width: 16rem; }
  .chat-explorer-collapsed-rail { background: var(--cal-bg); opacity: 0; pointer-events: none; transition: opacity 60ms linear; }
  .chat-explorer-collapsed-rail.visible { opacity: 1; pointer-events: auto; transition-delay: 100ms; }
  .chat-explorer-icon-button { display: flex; width: 1.75rem; height: 1.75rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  .chat-explorer-icon-button:hover, .chat-explorer-icon-button.active { background: var(--accent); color: var(--foreground); }
  .chat-folder-row { display: flex; min-height: 2rem; align-items: center; border-radius: 0.375rem; color: var(--foreground); }
  .chat-folder-row:hover { background: color-mix(in srgb, var(--accent) 68%, transparent); }
  .chat-folder-row.selected { font-weight: 600; }
  .chat-folder-main { display: flex; min-width: 0; min-height: 2rem; flex: 1; align-items: center; gap: 0.5rem; padding-inline: 0.375rem 0.25rem; text-align: left; font-size: 0.8rem; }
  .chat-folder-action { display: flex; width: 1.5rem; height: 1.5rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.3rem; color: var(--muted-foreground); opacity: 0; }
  .chat-folder-action.visible, .chat-folder-row:hover .chat-folder-action, .chat-folder-action:focus-visible { opacity: 1; }
  .chat-folder-action:hover { background: var(--accent); color: var(--foreground); }
  .chat-thread-row { display: flex; min-height: 1.9rem; align-items: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  .chat-thread-row:hover { background: color-mix(in srgb, var(--accent) 60%, transparent); color: var(--foreground); }
  .chat-thread-row.selected { color: var(--foreground); font-weight: 600; }
  .chat-thread-main { display: flex; min-width: 0; min-height: 1.9rem; flex: 1; align-items: center; gap: 0.45rem; padding-inline: 0.375rem; text-align: left; }
  .chat-thread-main span { min-width: 0; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.8rem; }
  .chat-thread-menu-trigger { display: flex; width: 1.5rem; height: 1.5rem; align-items: center; justify-content: center; border-radius: 0.3rem; color: var(--muted-foreground); opacity: 0; }
  .chat-thread-row:hover .chat-thread-menu-trigger, .chat-thread-menu-trigger:focus-visible { opacity: 1; }
  .chat-thread-menu-trigger:hover { background: var(--accent); color: var(--foreground); }
  .chat-search-result { display: flex; width: 100%; min-height: 2.4rem; align-items: flex-start; gap: 0.5rem; border-radius: 0.375rem; padding: 0.375rem; text-align: left; }
  .chat-search-result:hover { background: var(--accent); }
  .chat-search-result strong, .chat-search-result small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chat-search-result strong { font-size: 0.8rem; font-weight: 500; }
  .chat-search-result small { color: var(--muted-foreground); font-size: 0.666667rem; }
  @media (hover: none) { .chat-folder-action, .chat-thread-menu-trigger { opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .chat-project-explorer, .chat-explorer-collapsed-rail { transition: none; } }
</style>
