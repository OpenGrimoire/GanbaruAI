<script lang="ts">
  import { onMount, tick } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
  import Circle from "@lucide/svelte/icons/circle";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import type { ChatThreadShellRead, ChatWorkspaceId } from "$lib/chat/contracts";
  import { buildChatRailModel, filterThreadTitles, partitionThreadSearchResults, threadStatus } from "$lib/chat/shell-model";
  import * as chatApi from "$lib/api/chat";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import { openDetachedViewWindow } from "$lib/windows/detached";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ChatArchiveBrowser from "./ChatArchiveBrowser.svelte";

  let { onCollapse }: { onCollapse: () => void } = $props();
  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  let searchOpen = $state(false);
  let search = $state("");
  let remoteResults = $state<ChatThreadShellRead[]>([]);
  let showArchive = $state(false);
  let showWorkspaceChooser = $state(false);
  let deleteThread = $state<ChatThreadShellRead | null>(null);
  let renameThreadId = $state<string | null>(null);
  let renameValue = $state("");
  let railError = $state<string | null>(null);
  let searchInput: HTMLInputElement | undefined = $state();
  const localResults = $derived(filterThreadTitles([...chat.activeThreads, ...chat.archivedThreads], search));
  const searchResults = $derived(search.trim() ? mergeResults(localResults, remoteResults) : []);
  const partitionedSearchResults = $derived(partitionThreadSearchResults(searchResults));
  const activeSearchResults = $derived(partitionedSearchResults.active);
  const archivedSearchResults = $derived(partitionedSearchResults.archived);
  const rail = $derived(buildChatRailModel(projects.groups, projects.projects, chat.workspaces, chat.activeThreads, chat.draftWorkspaceId, chat.selectedThreadId));

  onMount(() => {
    const focusSearch = () => { void openSearch(); };
    window.addEventListener("ganbaru-ai:chat-focus-search", focusSearch);
    return () => window.removeEventListener("ganbaru-ai:chat-focus-search", focusSearch);
  });

  $effect(() => {
    const query = search.trim();
    if (!query) { remoteResults = []; return; }
    const timer = window.setTimeout(() => {
      void chatApi.searchChatThreadTitles(query, null)
        .then((results) => { if (search.trim() === query) remoteResults = results; })
        .catch((error: unknown) => { railError = errorMessage(error); });
    }, 250);
    return () => clearTimeout(timer);
  });

  async function openSearch(): Promise<void> {
    searchOpen = true;
    await tick();
    searchInput?.focus();
  }

  function newChat(workspaceId?: ChatWorkspaceId): void {
    const target = workspaceId ?? chat.selectedWorkspaceId;
    if (!target) { showWorkspaceChooser = true; return; }
    chat.newDraft(target);
  }

  function beginRename(thread: ChatThreadShellRead): void {
    renameThreadId = thread.id;
    renameValue = thread.title;
  }

  async function commitRename(thread: ChatThreadShellRead): Promise<void> {
    try {
      if (renameValue.trim() && renameValue.trim() !== thread.title) await chat.renameThread(thread, renameValue.trim());
      renameThreadId = null;
    } catch (error: unknown) {
      railError = errorMessage(error);
    }
  }

  async function detach(thread: ChatThreadShellRead): Promise<void> {
    try {
      chat.selectThread(thread.id);
      await openDetachedViewWindow("chat");
    } catch (error: unknown) {
      railError = errorMessage(error);
    }
  }

  function statusLabel(thread: ChatThreadShellRead): string {
    switch (threadStatus(thread)) {
      case "waiting_answer": return t("chat.status.waitingAnswer");
      case "waiting_approval": return t("chat.status.waitingApproval");
      case "working": return t("chat.status.working");
      case "error": return t("chat.status.error");
      case "unread": return t("chat.status.unread");
      case "archived": return t("chat.status.archived");
      default: return t("chat.status.idle");
    }
  }

  function selectSearchResult(thread: ChatThreadShellRead): void {
    chat.selectThread(thread.id);
    search = "";
    searchOpen = false;
    if (thread.archivedAt) showArchive = true;
  }

  function searchResultKeydown(event: KeyboardEvent): void {
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const rows = [...document.querySelectorAll<HTMLElement>("[data-chat-search-result]")];
    const current = rows.indexOf(event.currentTarget as HTMLElement);
    const next = event.key === "Home" ? 0
      : event.key === "End" ? rows.length - 1
      : Math.max(0, Math.min(rows.length - 1, current + (event.key === "ArrowDown" ? 1 : -1)));
    rows[next]?.focus();
  }

  function toggleGroup(groupId: string, collapsed: boolean): void {
    if (groupId === "ungrouped") return;
    runRailOperation(() => projects.setGroupCollapsed(groupId, !collapsed));
  }

  function mergeResults(local: ChatThreadShellRead[], remote: ChatThreadShellRead[]): ChatThreadShellRead[] {
    const seen = new Set<string>();
    return [...local, ...remote].filter((thread) => !seen.has(thread.id) && Boolean(seen.add(thread.id)));
  }

  function confirmDelete(): void {
    const thread = deleteThread;
    if (!thread) return;
    deleteThread = null;
    runRailOperation(() => chat.deleteThread(thread));
  }

  function runRailOperation(action: () => Promise<unknown>): void {
    railError = null;
    void action().catch((error: unknown) => { railError = errorMessage(error); });
  }

  function copyThreadId(threadId: string): void {
    runRailOperation(() => navigator.clipboard.writeText(threadId));
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function rowKeydown(event: KeyboardEvent, thread: ChatThreadShellRead): void {
    if (event.key === "Enter") { event.preventDefault(); chat.selectThread(thread.id); }
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const rows = [...document.querySelectorAll<HTMLElement>("[data-chat-thread-id]")];
      const current = rows.indexOf(event.currentTarget as HTMLElement);
      const next = event.key === "Home" ? 0
        : event.key === "End" ? rows.length - 1
        : Math.max(0, Math.min(rows.length - 1, current + (event.key === "ArrowDown" ? 1 : -1)));
      rows[next]?.focus();
    }
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      event.preventDefault();
      (event.currentTarget as HTMLElement).parentElement?.querySelector<HTMLDetailsElement>("details")?.setAttribute("open", "");
    }
  }
</script>

<aside class="relative flex h-full min-h-0 flex-col border-r border-border bg-sidebar/50" aria-label={t("chat.title")}>
  <header class="flex h-11 shrink-0 items-center gap-1 border-b border-border/70 px-2">
    {#if searchOpen}
      <label class="relative min-w-0 flex-1"><Search size={13} class="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground" /><input bind:this={searchInput} class="h-8 w-full rounded-md border border-border bg-background pl-7 pr-7 text-xs" type="search" bind:value={search} placeholder={t("chat.search")} /><button type="button" class="absolute right-1 top-1/2 -translate-y-1/2" aria-label={t("chat.clearSearch")} onclick={() => { search = ""; searchOpen = false; }}><X size={13} /></button></label>
    {:else}<strong class="min-w-0 flex-1 truncate px-1 text-sm">{t("chat.title")}</strong><button type="button" class="chat-icon-button" aria-label={t("chat.newChat")} onclick={() => newChat()}><MessageSquarePlus size={15} /></button><button type="button" class="chat-icon-button" aria-label={t("chat.search")} onclick={() => void openSearch()}><Search size={15} /></button>{/if}
    <details class="relative"><summary class="chat-icon-button list-none" aria-label={t("chat.moreActions")}><Ellipsis size={15} /></summary><div class="chat-menu right-0"><button type="button" onclick={() => { showArchive = true; }}>{t("chat.archivedChats")}</button><button type="button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.manageWorkspaces")}</button><button type="button" onclick={() => settings.open("chat")}>{t("chat.settings")}</button></div></details>
    <button type="button" class="chat-icon-button" aria-label={t("chat.collapseRail")} onclick={onCollapse}><ChevronsLeft size={15} /></button>
  </header>
  {#if railError}<div role="alert" class="border-b border-destructive/30 px-3 py-2 text-xs text-destructive">{railError}</div>{/if}

  {#if search.trim()}
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      {#if searchResults.length === 0}<p class="p-3 text-xs text-muted-foreground">{t("chat.noSearchResults")}</p>{/if}
      {#if activeSearchResults.length > 0}<h2 class="px-2 pb-1 pt-2 text-[0.666667rem] font-semibold uppercase text-muted-foreground">{t("chat.activeChats")}</h2>{#each activeSearchResults as thread}<button type="button" data-chat-search-result class="w-full rounded-md p-2 text-left hover:bg-accent" onkeydown={searchResultKeydown} onclick={() => selectSearchResult(thread)}><span class="block truncate text-xs font-medium">{thread.title}</span><span class="block truncate text-[0.666667rem] text-muted-foreground">{chat.workspaces.find((entry) => entry.workspace.id === thread.workspaceId)?.workspace.displayName}</span></button>{/each}{/if}
      {#if archivedSearchResults.length > 0}<h2 class="px-2 pb-1 pt-3 text-[0.666667rem] font-semibold uppercase text-muted-foreground">{t("chat.archivedChats")}</h2>{#each archivedSearchResults as thread}<button type="button" data-chat-search-result class="w-full rounded-md p-2 text-left hover:bg-accent" onkeydown={searchResultKeydown} onclick={() => selectSearchResult(thread)}><span class="block truncate text-xs font-medium">{thread.title}</span><span class="block truncate text-[0.666667rem] text-muted-foreground">{chat.workspaces.find((entry) => entry.workspace.id === thread.workspaceId)?.workspace.displayName}</span></button>{/each}{/if}
    </div>
  {:else}
    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      {#each rail.groups as group}
        <section class="mb-2">
          <button type="button" class="flex w-full items-center gap-1 rounded px-1 py-1 text-left text-[0.666667rem] font-semibold uppercase tracking-wide text-muted-foreground hover:bg-accent" disabled={group.id === "ungrouped"} onclick={() => toggleGroup(group.id, group.collapsed)}>{#if group.collapsed}<ChevronRight size={12} />{:else}<ChevronDown size={12} />{/if}<span class="truncate">{group.label}</span>{#if group.hidden}<span>{t("chat.hiddenGroup")}</span>{/if}{#if group.archived}<span>{t("chat.status.archived")}</span>{/if}</button>
          {#each group.projects as projectModel}
            {@const selectedInProject = projectModel.workspaces.some((workspace) => workspace.threads.some((thread) => thread.id === chat.selectedThreadId))}
            {#if !group.collapsed || selectedInProject}
              <div class="mt-1 border-l-2" style={`border-left-color:${projectModel.project.color ?? "transparent"}`}><div class="flex items-center gap-1.5 px-2 py-1 text-xs font-medium"><ProjectIcon name={projectModel.project.icon} size={13} /><span class="truncate">{projectModel.project.name}</span><button type="button" class="ml-auto chat-icon-button size-6" aria-label={t("chat.newChat")} onclick={() => newChat(projectModel.workspaces[0]?.workspace.workspace.id)}><MessageSquarePlus size={12} /></button></div>
                {#each projectModel.workspaces as workspaceModel}
                  {#if workspaceModel.showSubdivision}<div class="px-3 py-1 text-[0.666667rem] text-muted-foreground">{workspaceModel.workspace.workspace.displayName}</div>{/if}
                  {#if workspaceModel.hasDraft}<button type="button" class="chat-thread-row chat-thread-row-main" class:active={chat.selectedThreadId === null && chat.draftWorkspaceId === workspaceModel.workspace.workspace.id} onclick={() => chat.selectWorkspace(workspaceModel.workspace.workspace.id)}><CircleDot size={12} /><span class="truncate">{t("chat.draft")}</span></button>{/if}
                  {#each workspaceModel.threads as thread}
                    <div class="chat-thread-row group" class:active={chat.selectedThreadId === thread.id}>
                      {#if renameThreadId === thread.id}<input class="mx-2 h-7 min-w-0 flex-1 rounded border border-ring bg-background px-1 text-xs" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(thread); if (event.key === "Escape") renameThreadId = null; }} onblur={() => void commitRename(thread)} />{:else}<button type="button" class="chat-thread-row-main" data-chat-thread-id={thread.id} onkeydown={(event) => rowKeydown(event, thread)} onclick={() => chat.selectThread(thread.id)}>{#if threadStatus(thread) === "working"}<LoaderCircle size={12} class="animate-spin" />{:else if threadStatus(thread) === "error"}<CircleAlert size={12} />{:else if threadStatus(thread) === "unread" || threadStatus(thread).startsWith("waiting")}<CircleDot size={12} />{:else}<Circle size={12} />{/if}<span class="min-w-0 flex-1"><span class="block truncate text-xs">{thread.title}</span><span class="block truncate text-[0.6rem] text-muted-foreground">{statusLabel(thread)}</span></span></button>{/if}
                      <details class="relative"><summary class="chat-icon-button size-6 list-none opacity-100 @min-[500px]:opacity-0 @min-[500px]:group-hover:opacity-100" aria-label={t("chat.moreActions")}><Ellipsis size={12} /></summary><div class="chat-menu right-0 top-6"><button type="button" onclick={() => beginRename(thread)}>{t("chat.rename")}</button><button type="button" onclick={() => runRailOperation(() => chat.setThreadRead(thread, Boolean(thread.unreadAt)))}>{thread.unreadAt ? t("chat.markRead") : t("chat.markUnread")}</button><button type="button" onclick={() => void detach(thread)}>{t("chat.detach")}</button><button type="button" onclick={() => copyThreadId(thread.id)}>{t("chat.copyThreadId")}</button><button type="button" onclick={() => runRailOperation(() => chat.openWorkspaceFolder(thread.workspaceId))}>{t("chat.openFolder")}</button><button type="button" onclick={() => runRailOperation(() => chat.archiveThread(thread))}>{t("chat.archive")}</button><button type="button" class="text-destructive" onclick={() => { deleteThread = thread; }}>{t("chat.deletePermanently")}</button></div></details>
                    </div>
                  {/each}
                {/each}
              </div>
            {/if}
          {/each}
        </section>
      {/each}
      {#if rail.standalone.length > 0}<section><div class="px-2 py-1 text-[0.666667rem] font-semibold uppercase text-muted-foreground">{t("chat.standalone")}</div>{#each rail.standalone as workspaceModel}<div class="flex items-center px-3 py-1 text-[0.666667rem] text-muted-foreground"><span class="min-w-0 flex-1 truncate">{workspaceModel.workspace.workspace.displayName}</span><button type="button" class="chat-icon-button size-6" aria-label={t("chat.newChat")} onclick={() => newChat(workspaceModel.workspace.workspace.id)}><MessageSquarePlus size={12} /></button></div>{#if workspaceModel.hasDraft}<button type="button" class="chat-thread-row chat-thread-row-main" class:active={chat.selectedThreadId === null && chat.draftWorkspaceId === workspaceModel.workspace.workspace.id} onclick={() => chat.selectWorkspace(workspaceModel.workspace.workspace.id)}><CircleDot size={12} /><span class="truncate">{t("chat.draft")}</span></button>{/if}{#each workspaceModel.threads as thread}<div class="chat-thread-row group" class:active={chat.selectedThreadId === thread.id}>{#if renameThreadId === thread.id}<input class="mx-2 h-7 min-w-0 flex-1 rounded border border-ring bg-background px-1 text-xs" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(thread); if (event.key === "Escape") renameThreadId = null; }} onblur={() => void commitRename(thread)} />{:else}<button type="button" class="chat-thread-row-main" data-chat-thread-id={thread.id} onkeydown={(event) => rowKeydown(event, thread)} onclick={() => chat.selectThread(thread.id)}>{#if threadStatus(thread) === "working"}<LoaderCircle size={12} class="animate-spin" />{:else if threadStatus(thread) === "error"}<CircleAlert size={12} />{:else if threadStatus(thread) === "unread" || threadStatus(thread).startsWith("waiting")}<CircleDot size={12} />{:else}<Circle size={12} />{/if}<span class="min-w-0 flex-1"><span class="block truncate text-xs">{thread.title}</span><span class="block truncate text-[0.6rem] text-muted-foreground">{statusLabel(thread)}</span></span></button>{/if}<details class="relative"><summary class="chat-icon-button size-6 list-none opacity-100 @min-[500px]:opacity-0 @min-[500px]:group-hover:opacity-100" aria-label={t("chat.moreActions")}><Ellipsis size={12} /></summary><div class="chat-menu right-0 top-6"><button type="button" onclick={() => beginRename(thread)}>{t("chat.rename")}</button><button type="button" onclick={() => runRailOperation(() => chat.setThreadRead(thread, Boolean(thread.unreadAt)))}>{thread.unreadAt ? t("chat.markRead") : t("chat.markUnread")}</button><button type="button" onclick={() => void detach(thread)}>{t("chat.detach")}</button><button type="button" onclick={() => copyThreadId(thread.id)}>{t("chat.copyThreadId")}</button><button type="button" onclick={() => runRailOperation(() => chat.openWorkspaceFolder(thread.workspaceId))}>{t("chat.openFolder")}</button><button type="button" onclick={() => runRailOperation(() => chat.archiveThread(thread))}>{t("chat.archive")}</button><button type="button" class="text-destructive" onclick={() => { deleteThread = thread; }}>{t("chat.deletePermanently")}</button></div></details></div>{/each}{/each}</section>{/if}
    </div>
  {/if}

  {#if showArchive}<ChatArchiveBrowser onClose={() => { showArchive = false; }} onDelete={(thread) => { deleteThread = thread; }} />{/if}
  {#if showWorkspaceChooser}<div class="absolute inset-x-2 top-12 z-30 rounded-lg border border-border bg-popover p-2 shadow-xl"><div class="mb-2 flex items-center"><strong class="flex-1 px-1 text-xs">{t("chat.firstUse.selectWorkspaceTitle")}</strong><button type="button" class="chat-icon-button" onclick={() => { showWorkspaceChooser = false; }}><X size={13} /></button></div>{#each chat.workspaces.filter((entry) => entry.workspace.archivedAt === null) as workspace}<button type="button" class="w-full rounded p-2 text-left text-xs hover:bg-accent" onclick={() => { newChat(workspace.workspace.id); showWorkspaceChooser = false; }}>{workspace.workspace.displayName}</button>{/each}</div>{/if}
</aside>

{#if deleteThread}<ConfirmDialog title={t("chat.deleteTitle")} message={t("chat.deleteMessage", deleteThread.title)} confirmLabel={t("chat.deletePermanently")} cancelLabel={t("chat.cancel")} onConfirm={confirmDelete} onCancel={() => { deleteThread = null; }} />{/if}

<style>
  :global(.chat-icon-button) { display: inline-flex; width: 2rem; height: 2rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  :global(.chat-icon-button:hover) { background: var(--accent); color: var(--foreground); }
  :global(.chat-small-button) { min-height: 1.75rem; border-radius: 0.375rem; border: 1px solid var(--border); padding: 0.2rem 0.5rem; font-size: 0.666667rem; }
  :global(.chat-menu) { position: absolute; z-index: 50; display: flex; width: max-content; min-width: 10rem; flex-direction: column; border: 1px solid var(--border); border-radius: 0.375rem; background: var(--popover); padding: 0.25rem; box-shadow: 0 8px 24px rgb(0 0 0 / 0.18); }
  :global(.chat-menu button) { min-height: 1.875rem; border-radius: 0.25rem; padding: 0.25rem 0.5rem; text-align: left; font-size: 0.733333rem; }
  :global(.chat-menu button:hover) { background: var(--accent); }
  .chat-thread-row { display: flex; width: 100%; min-height: 2.25rem; align-items: center; border-radius: 0.375rem; color: var(--muted-foreground); text-align: left; }
  .chat-thread-row-main { display: flex; min-width: 0; flex: 1; align-items: center; gap: 0.5rem; padding: 0.25rem 0.25rem 0.25rem 0.75rem; text-align: left; }
  .chat-thread-row:hover, .chat-thread-row.active { background: var(--accent); color: var(--foreground); }
</style>
