<script lang="ts">
  import { onMount, tick } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import Search from "@lucide/svelte/icons/search";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import X from "@lucide/svelte/icons/x";
  import type { ChatThreadShellRead, ChatWorkspaceId } from "$lib/chat/contracts";
  import { buildChatRailModel, filterThreadTitles, partitionThreadSearchResults, threadStatus } from "$lib/chat/shell-model";
  import * as chatApi from "$lib/api/chat";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { formatRelativeMinutes } from "$lib/i18n/formatters";
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
  let search = $state("");
  let remoteResults = $state<ChatThreadShellRead[]>([]);
  let showArchive = $state(false);
  let showWorkspaceChooser = $state(false);
  let deleteThread = $state<ChatThreadShellRead | null>(null);
  let renameThreadId = $state<string | null>(null);
  let renameValue = $state("");
  let railError = $state<string | null>(null);
  let searchInput: HTMLInputElement | undefined = $state();
  let now = $state(Date.now());
  const localResults = $derived(filterThreadTitles([...chat.activeThreads, ...chat.archivedThreads], search));
  const searchResults = $derived(search.trim() ? mergeResults(localResults, remoteResults) : []);
  const partitionedSearchResults = $derived(partitionThreadSearchResults(searchResults));
  const activeSearchResults = $derived(partitionedSearchResults.active);
  const archivedSearchResults = $derived(partitionedSearchResults.archived);
  const rail = $derived(buildChatRailModel(projects.groups, projects.projects, chat.workspaces, chat.activeThreads, chat.draftWorkspaceId, chat.selectedThreadId));

  onMount(() => {
    const focusSearch = () => { void openSearch(); };
    window.addEventListener("ganbaru-ai:chat-focus-search", focusSearch);
    const timer = window.setInterval(() => { now = Date.now(); }, 60_000);
    return () => {
      window.removeEventListener("ganbaru-ai:chat-focus-search", focusSearch);
      window.clearInterval(timer);
    };
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

  function activityTimeLabel(thread: ChatThreadShellRead): string {
    const timestamp = Date.parse(thread.lastActivityAt);
    if (!Number.isFinite(timestamp)) return "";
    return formatRelativeMinutes(t, (timestamp - now) / 60_000);
  }

  function selectSearchResult(thread: ChatThreadShellRead): void {
    chat.selectThread(thread.id);
    search = "";
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

<aside class="chat-rail" aria-label={t("chat.title")}>
  <header class="chat-rail-header">
    <strong class="min-w-0 flex-1 truncate px-1 text-sm font-medium">{t("chat.title")}</strong>
    <button type="button" class="chat-icon-button" aria-label={t("chat.newChat")} onclick={() => newChat()}><MessageSquarePlus size={15} /></button>
    <details class="relative"><summary class="chat-icon-button list-none" aria-label={t("chat.moreActions")}><Ellipsis size={15} /></summary><div class="chat-menu right-0"><button type="button" onclick={() => { showArchive = true; }}>{t("chat.archivedChats")}</button><button type="button" onclick={() => settings.open("chat", { chatSubsection: "workspaces" })}>{t("chat.manageWorkspaces")}</button><button type="button" onclick={() => settings.open("chat")}>{t("chat.settings")}</button></div></details>
    <button type="button" class="chat-icon-button" aria-label={t("chat.collapseRail")} onclick={onCollapse}><ChevronsLeft size={15} /></button>
  </header>
  <div class="chat-rail-search"><label><Search size={14} /><input bind:this={searchInput} type="search" bind:value={search} placeholder={t("chat.search")} />{#if search}<button type="button" aria-label={t("chat.clearSearch")} onclick={() => { search = ""; }}><X size={13} /></button>{/if}</label></div>
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
              <div class="chat-project"><div class="chat-project-header"><ProjectIcon name={projectModel.project.icon} size={14} /><span class="truncate">{projectModel.project.name}</span><button type="button" class="ml-auto chat-icon-button size-6" aria-label={t("chat.newChat")} onclick={() => newChat(projectModel.workspaces[0]?.workspace.workspace.id)}><MessageSquarePlus size={12} /></button></div>
                {#each projectModel.workspaces as workspaceModel}
                  {#if workspaceModel.showSubdivision}<div class="px-3 py-1 text-[0.666667rem] text-muted-foreground">{workspaceModel.workspace.workspace.displayName}</div>{/if}
                  {#if workspaceModel.hasDraft}<button type="button" class="chat-thread-row chat-thread-row-main" class:active={chat.selectedThreadId === null && chat.draftWorkspaceId === workspaceModel.workspace.workspace.id} onclick={() => chat.selectWorkspace(workspaceModel.workspace.workspace.id)}><CircleDot size={12} /><span class="truncate">{t("chat.draft")}</span></button>{/if}
                  {#each workspaceModel.threads as thread}
                    <div class="chat-thread-row group" class:active={chat.selectedThreadId === thread.id}>
                      {#if renameThreadId === thread.id}<input class="mx-2 h-7 min-w-0 flex-1 rounded border border-ring bg-background px-1 text-xs" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(thread); if (event.key === "Escape") renameThreadId = null; }} onblur={() => void commitRename(thread)} />{:else}<button type="button" class="chat-thread-row-main" data-chat-thread-id={thread.id} title={statusLabel(thread)} onkeydown={(event) => rowKeydown(event, thread)} onclick={() => chat.selectThread(thread.id)}>{#if threadStatus(thread) === "working"}<LoaderCircle size={12} class="animate-spin" />{:else if threadStatus(thread) === "error"}<CircleAlert size={12} />{:else if threadStatus(thread) === "unread" || threadStatus(thread).startsWith("waiting")}<CircleDot size={12} />{/if}<span class="min-w-0 flex-1 truncate text-xs">{thread.title}</span><span class="thread-time">{activityTimeLabel(thread)}</span></button>{/if}
                      <details class="relative"><summary class="chat-icon-button size-6 list-none opacity-100 @min-[500px]:opacity-0 @min-[500px]:group-hover:opacity-100" aria-label={t("chat.moreActions")}><Ellipsis size={12} /></summary><div class="chat-menu right-0 top-6"><button type="button" onclick={() => beginRename(thread)}>{t("chat.rename")}</button><button type="button" onclick={() => runRailOperation(() => chat.setThreadRead(thread, Boolean(thread.unreadAt)))}>{thread.unreadAt ? t("chat.markRead") : t("chat.markUnread")}</button><button type="button" onclick={() => void detach(thread)}>{t("chat.detach")}</button><button type="button" onclick={() => copyThreadId(thread.id)}>{t("chat.copyThreadId")}</button><button type="button" onclick={() => runRailOperation(() => chat.openWorkspaceFolder(thread.workspaceId))}>{t("chat.openFolder")}</button><button type="button" onclick={() => runRailOperation(() => chat.archiveThread(thread))}>{t("chat.archive")}</button><button type="button" class="text-destructive" onclick={() => { deleteThread = thread; }}>{t("chat.deletePermanently")}</button></div></details>
                    </div>
                  {/each}
                {/each}
              </div>
            {/if}
          {/each}
        </section>
      {/each}
      {#if rail.standalone.length > 0}<section><div class="px-2 py-1 text-[0.666667rem] font-semibold uppercase text-muted-foreground">{t("chat.standalone")}</div>{#each rail.standalone as workspaceModel}<div class="flex items-center px-3 py-1 text-[0.666667rem] text-muted-foreground"><span class="min-w-0 flex-1 truncate">{workspaceModel.workspace.workspace.displayName}</span><button type="button" class="chat-icon-button size-6" aria-label={t("chat.newChat")} onclick={() => newChat(workspaceModel.workspace.workspace.id)}><MessageSquarePlus size={12} /></button></div>{#if workspaceModel.hasDraft}<button type="button" class="chat-thread-row chat-thread-row-main" class:active={chat.selectedThreadId === null && chat.draftWorkspaceId === workspaceModel.workspace.workspace.id} onclick={() => chat.selectWorkspace(workspaceModel.workspace.workspace.id)}><CircleDot size={12} /><span class="truncate">{t("chat.draft")}</span></button>{/if}{#each workspaceModel.threads as thread}<div class="chat-thread-row group" class:active={chat.selectedThreadId === thread.id}>{#if renameThreadId === thread.id}<input class="mx-2 h-7 min-w-0 flex-1 rounded border border-ring bg-background px-1 text-xs" bind:value={renameValue} onkeydown={(event) => { if (event.key === "Enter") void commitRename(thread); if (event.key === "Escape") renameThreadId = null; }} onblur={() => void commitRename(thread)} />{:else}<button type="button" class="chat-thread-row-main" data-chat-thread-id={thread.id} title={statusLabel(thread)} onkeydown={(event) => rowKeydown(event, thread)} onclick={() => chat.selectThread(thread.id)}>{#if threadStatus(thread) === "working"}<LoaderCircle size={12} class="animate-spin" />{:else if threadStatus(thread) === "error"}<CircleAlert size={12} />{:else if threadStatus(thread) === "unread" || threadStatus(thread).startsWith("waiting")}<CircleDot size={12} />{/if}<span class="min-w-0 flex-1 truncate text-xs">{thread.title}</span><span class="thread-time">{activityTimeLabel(thread)}</span></button>{/if}<details class="relative"><summary class="chat-icon-button size-6 list-none opacity-100 @min-[500px]:opacity-0 @min-[500px]:group-hover:opacity-100" aria-label={t("chat.moreActions")}><Ellipsis size={12} /></summary><div class="chat-menu right-0 top-6"><button type="button" onclick={() => beginRename(thread)}>{t("chat.rename")}</button><button type="button" onclick={() => runRailOperation(() => chat.setThreadRead(thread, Boolean(thread.unreadAt)))}>{thread.unreadAt ? t("chat.markRead") : t("chat.markUnread")}</button><button type="button" onclick={() => void detach(thread)}>{t("chat.detach")}</button><button type="button" onclick={() => copyThreadId(thread.id)}>{t("chat.copyThreadId")}</button><button type="button" onclick={() => runRailOperation(() => chat.openWorkspaceFolder(thread.workspaceId))}>{t("chat.openFolder")}</button><button type="button" onclick={() => runRailOperation(() => chat.archiveThread(thread))}>{t("chat.archive")}</button><button type="button" class="text-destructive" onclick={() => { deleteThread = thread; }}>{t("chat.deletePermanently")}</button></div></details></div>{/each}{/each}</section>{/if}
    </div>
  {/if}

  {#if showArchive}<ChatArchiveBrowser onClose={() => { showArchive = false; }} onDelete={(thread) => { deleteThread = thread; }} />{/if}
  {#if showWorkspaceChooser}<div class="absolute inset-x-2 top-12 z-30 rounded-lg border border-border bg-popover p-2 shadow-xl"><div class="mb-2 flex items-center"><strong class="flex-1 px-1 text-xs">{t("chat.firstUse.selectWorkspaceTitle")}</strong><button type="button" class="chat-icon-button" onclick={() => { showWorkspaceChooser = false; }}><X size={13} /></button></div>{#each chat.workspaces.filter((entry) => entry.workspace.archivedAt === null) as workspace}<button type="button" class="w-full rounded p-2 text-left text-xs hover:bg-accent" onclick={() => { newChat(workspace.workspace.id); showWorkspaceChooser = false; }}>{workspace.workspace.displayName}</button>{/each}</div>{/if}
  <footer class="chat-rail-footer"><button type="button" onclick={() => settings.open("chat")}><SettingsIcon size={14} /><span>{t("chat.settings")}</span></button></footer>
</aside>

{#if deleteThread}<ConfirmDialog title={t("chat.deleteTitle")} message={t("chat.deleteMessage", deleteThread.title)} confirmLabel={t("chat.deletePermanently")} cancelLabel={t("chat.cancel")} onConfirm={confirmDelete} onCancel={() => { deleteThread = null; }} />{/if}

<style>
  .chat-rail { position: relative; display: flex; height: 100%; min-height: 0; flex-direction: column; border-right: 1px solid var(--border); background: color-mix(in srgb, var(--card) 88%, var(--cal-bg)); }
  .chat-rail-header { display: flex; min-height: 3.25rem; flex: 0 0 auto; align-items: center; gap: 0.2rem; padding-inline: 0.55rem; }
  .chat-rail-search { padding: 0.25rem 0.65rem 0.55rem; }
  .chat-rail-search label { display: flex; min-height: 2rem; align-items: center; gap: 0.5rem; border-radius: 0.5rem; padding-inline: 0.55rem; color: var(--muted-foreground); }
  .chat-rail-search label:focus-within, .chat-rail-search label:hover { background: var(--accent); color: var(--foreground); }
  .chat-rail-search input { min-width: 0; flex: 1; border: 0; background: transparent; color: var(--foreground); font-size: 0.733333rem; outline: 0; }
  .chat-rail-search input::placeholder { color: var(--muted-foreground); }
  .chat-rail-search button { display: inline-flex; width: 1.5rem; height: 1.5rem; align-items: center; justify-content: center; border-radius: 0.35rem; }
  .chat-project { margin-block: 0.2rem 0.55rem; }
  .chat-project-header { display: flex; min-height: 2rem; align-items: center; gap: 0.5rem; border-radius: 0.45rem; padding: 0.25rem 0.4rem; color: color-mix(in srgb, var(--foreground) 90%, transparent); font-size: 0.8rem; font-weight: 500; }
  .chat-project-header:hover { background: color-mix(in srgb, var(--accent) 60%, transparent); }
  .chat-rail-footer { flex: 0 0 auto; border-top: 1px solid color-mix(in srgb, var(--border) 65%, transparent); padding: 0.45rem 0.65rem; }
  .chat-rail-footer button { display: flex; min-height: 2rem; width: 100%; align-items: center; gap: 0.55rem; border-radius: 0.45rem; padding-inline: 0.55rem; color: var(--muted-foreground); font-size: 0.733333rem; text-align: left; }
  .chat-rail-footer button:hover { background: var(--accent); color: var(--foreground); }
  :global(.chat-icon-button) { display: inline-flex; width: 2rem; height: 2rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  :global(.chat-icon-button:hover) { background: var(--accent); color: var(--foreground); }
  :global(.chat-small-button) { min-height: 1.75rem; border-radius: 0.375rem; border: 1px solid var(--border); padding: 0.2rem 0.5rem; font-size: 0.666667rem; }
  :global(.chat-menu) { position: absolute; z-index: 50; display: flex; width: max-content; min-width: 10rem; flex-direction: column; border: 1px solid var(--border); border-radius: 0.375rem; background: var(--popover); padding: 0.25rem; box-shadow: 0 8px 24px rgb(0 0 0 / 0.18); }
  :global(.chat-menu button) { min-height: 1.875rem; border-radius: 0.25rem; padding: 0.25rem 0.5rem; text-align: left; font-size: 0.733333rem; }
  :global(.chat-menu button:hover) { background: var(--accent); }
  .chat-thread-row { display: flex; width: calc(100% - 0.5rem); min-height: 1.8rem; margin-inline: 0.5rem 0; align-items: center; border-radius: 0.5rem; color: var(--muted-foreground); text-align: left; }
  .chat-thread-row-main { display: flex; min-width: 0; flex: 1; align-items: center; gap: 0.45rem; padding: 0.25rem 0.4rem 0.25rem 0.65rem; text-align: left; }
  .chat-thread-row:hover { background: color-mix(in srgb, var(--accent) 65%, transparent); color: var(--foreground); }
  .chat-thread-row.active { background: var(--accent); color: var(--foreground); font-weight: 500; }
  .thread-time { flex: 0 0 auto; color: color-mix(in srgb, var(--muted-foreground) 70%, transparent); font-size: 0.6rem; font-weight: 400; }
</style>
