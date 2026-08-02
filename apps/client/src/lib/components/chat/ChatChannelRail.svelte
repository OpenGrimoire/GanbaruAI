<script lang="ts">
  import { onMount } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronsLeft from "@lucide/svelte/icons/chevrons-left";
  import ChevronsRight from "@lucide/svelte/icons/chevrons-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleDot from "@lucide/svelte/icons/circle-dot";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import Hash from "@lucide/svelte/icons/hash";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import type { ChatChannelRead, ChatMessageSearchResultRead } from "$lib/chat/contracts";
  import {
    moveChatChannelToSection,
    normalizeChatSidebarSections,
    readChatSidebarSections,
    saveChatSidebarSections,
    type ChatSidebarSection,
  } from "$lib/chat/channel-sections";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { onActiveVaultIdentityChange } from "$lib/vault/active-vault";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ChatChannelSetupDialog from "./ChatChannelSetupDialog.svelte";

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

  const chat = getChat();
  const projects = getProjects();
  const { t } = getLocalization();
  let query = $state("");
  let searchOpen = $state(false);
  let searchInput = $state<HTMLInputElement | null>(null);
  let setupChannel = $state<ChatChannelRead | null | undefined>(undefined);
  let setupSectionId = $state<string | null>(null);
  let sections = $state<ChatSidebarSection[]>([]);
  let loadedProjectId = $state<string | null>(null);
  let newSectionName = $state("");
  let creatingSection = $state(false);
  let archiveCandidate = $state<ChatChannelRead | null>(null);
  let deleteSectionCandidate = $state<ChatSidebarSection | null>(null);
  let railError = $state<string | null>(null);
  let messageSearchResults = $state<ChatMessageSearchResultRead[]>([]);
  let messageSearchLoading = $state(false);
  let messageSearchRequest = 0;
  const matchingChannels = $derived(chat.activeChannels.filter(matchesQuery));
  const assignedChannelIds = $derived(new Set(sections.flatMap((section) => section.channelIds)));
  const unsectionedChannels = $derived(matchingChannels.filter((channel) => !assignedChannelIds.has(channel.id)));

  $effect(() => {
    const projectId = projects.selectedProjectId;
    if (!projectId || projectId === loadedProjectId) return;
    const projectChannels = chat.activeChannels.filter((channel) => channel.projectId === projectId);
    if (projectChannels.length === 0 && chat.activeChannels.length > 0) return;
    loadedProjectId = projectId;
    sections = normalizeChatSidebarSections(readChatSidebarSections(projectId), projectChannels.map((channel) => channel.id));
  });

  $effect(() => {
    const normalized = query.trim();
    if (!searchOpen || normalized.length < 2) {
      messageSearchRequest += 1;
      messageSearchResults = [];
      messageSearchLoading = false;
      return;
    }
    const request = ++messageSearchRequest;
    messageSearchLoading = true;
    const timeout = window.setTimeout(() => {
      void chat.searchOrganizationalMessages(normalized)
        .then((results) => {
          if (request === messageSearchRequest) messageSearchResults = results;
        })
        .catch((cause: unknown) => {
          if (request === messageSearchRequest) {
            railError = cause instanceof Error ? cause.message : String(cause);
          }
        })
        .finally(() => {
          if (request === messageSearchRequest) messageSearchLoading = false;
        });
    }, 180);
    return () => window.clearTimeout(timeout);
  });

  $effect(() => {
    const projectId = loadedProjectId;
    const channelIds = chat.activeChannels
      .filter((channel) => channel.projectId === projectId)
      .map((channel) => channel.id);
    if (!projectId) return;
    const normalized = normalizeChatSidebarSections(sections, channelIds);
    if (JSON.stringify(normalized) !== JSON.stringify(sections)) sections = normalized;
  });

  function matchesQuery(channel: ChatChannelRead): boolean {
    const normalized = query.trim().toLocaleLowerCase();
    return !normalized
      || channel.name.toLocaleLowerCase().includes(normalized)
      || channel.topic.toLocaleLowerCase().includes(normalized);
  }

  function sectionChannels(section: ChatSidebarSection): ChatChannelRead[] {
    const byId = new Map(matchingChannels.map((channel) => [channel.id, channel]));
    return section.channelIds.flatMap((id) => byId.get(id) ?? []);
  }

  function persist(next: ChatSidebarSection[]): void {
    sections = next;
    if (loadedProjectId) saveChatSidebarSections(loadedProjectId, next);
  }

  function toggleSection(sectionId: string): void {
    persist(sections.map((section) => section.id === sectionId
      ? { ...section, collapsed: !section.collapsed }
      : section));
  }

  function createSection(): void {
    const name = newSectionName.trim();
    if (!name || [...name].length > 80) return;
    persist([...sections, { id: `section:${crypto.randomUUID()}`, name, collapsed: false, channelIds: [] }]);
    newSectionName = "";
    creatingSection = false;
  }

  function deleteSection(section: ChatSidebarSection): void {
    persist(sections.filter((entry) => entry.id !== section.id));
    deleteSectionCandidate = null;
  }

  function confirmDeleteSection(): void {
    const section = deleteSectionCandidate;
    if (section) deleteSection(section);
  }

  function renameSection(section: ChatSidebarSection): void {
    const name = window.prompt(t("chat.channels.sectionName"), section.name)?.trim();
    if (!name || [...name].length > 80 || name === section.name) return;
    persist(sections.map((entry) => entry.id === section.id ? { ...entry, name } : entry));
  }

  function moveChannel(channelId: string, sectionId: string | null): void {
    persist(moveChatChannelToSection(sections, channelId, sectionId));
  }

  function handleDrop(event: DragEvent, sectionId: string | null): void {
    event.preventDefault();
    const channelId = event.dataTransfer?.getData("application/x-ganbaru-chat-channel") ?? "";
    if (channelId) moveChannel(channelId, sectionId);
  }

  function beginDrag(event: DragEvent, channelId: string): void {
    event.dataTransfer?.setData("application/x-ganbaru-chat-channel", channelId);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  function channelStatus(channel: ChatChannelRead): { label: string; kind: "working" | "attention" | "error" | "unread" | "idle" } {
    const state = channel.attentionState;
    if (state === "waiting_for_answer") return { label: t("chat.status.waitingAnswer"), kind: "attention" };
    if (state === "waiting_for_approval" || state === "ready_for_review") {
      return { label: state === "waiting_for_approval" ? t("chat.status.waitingApproval") : t("chat.status.readyForReview"), kind: "attention" };
    }
    if (state === "queued" || state === "working") return { label: t("chat.status.working"), kind: "working" };
    if (state === "failed") return { label: t("chat.status.error"), kind: "error" };
    if (channel.unreadCount > 0) return { label: t("chat.status.unread"), kind: "unread" };
    return { label: t("chat.status.idle"), kind: "idle" };
  }

  async function confirmArchive(): Promise<void> {
    const channel = archiveCandidate;
    archiveCandidate = null;
    if (!channel) return;
    railError = null;
    try {
      await chat.archiveChannel(channel);
    } catch (cause: unknown) {
      railError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function savedChannel(channel: ChatChannelRead, sectionId: string | null): void {
    moveChannel(channel.id, sectionId);
    setupChannel = undefined;
    setupSectionId = null;
  }

  function openSearch(): void {
    searchOpen = true;
    queueMicrotask(() => searchInput?.focus());
  }

  function openCreate(sectionId: string | null = null): void {
    onExpand();
    setupChannel = null;
    setupSectionId = sectionId;
  }

  async function selectChannel(channelId: string): Promise<void> {
    railError = null;
    try {
      await chat.selectChannel(channelId);
    } catch (cause: unknown) {
      railError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function openMessageResult(result: ChatMessageSearchResultRead): Promise<void> {
    railError = null;
    try {
      await chat.openMessageSearchResult(result);
      query = "";
      searchOpen = false;
    } catch (cause: unknown) {
      railError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function searchExcerpt(result: ChatMessageSearchResultRead): string {
    return result.excerpt.replace(/<\/?mark>/g, "");
  }

  function projectName(projectId: string): string {
    return projects.projects.find((project) => project.id === projectId)?.name ?? t("chat.title");
  }

  onMount(() => {
    const createChannel = () => openCreate();
    const focusSearch = () => {
      onExpand();
      openSearch();
    };
    window.addEventListener("ganbaru-ai:chat-new-channel", createChannel);
    window.addEventListener("ganbaru-ai:chat-focus-search", focusSearch);
    const unsubscribeVault = onActiveVaultIdentityChange(() => {
      loadedProjectId = null;
      sections = [];
    });
    return () => {
      window.removeEventListener("ganbaru-ai:chat-new-channel", createChannel);
      window.removeEventListener("ganbaru-ai:chat-focus-search", focusSearch);
      unsubscribeVault();
    };
  });
</script>

{#if expanded}
  <aside class="flex h-full min-h-0 flex-col bg-sidebar/45" aria-label={t("chat.channels.explorerLabel")}>
    <div class="flex h-11 shrink-0 items-center gap-1 px-2">
      {#if searchOpen}
        <label class="flex min-w-0 flex-1 items-center gap-1 rounded-md border border-border bg-background px-2"><Search size={13} class="text-muted-foreground" /><input bind:this={searchInput} class="h-7 min-w-0 flex-1 bg-transparent text-xs outline-none" type="search" bind:value={query} placeholder={t("chat.channels.search")} /></label>
        <button type="button" class="rail-icon" aria-label={t("chat.clearSearch")} onclick={() => { query = ""; searchOpen = false; }}><span aria-hidden="true">×</span></button>
      {:else}
        <strong class="min-w-0 flex-1 truncate px-1 text-sm">{projects.selectedProject?.name ?? t("chat.title")}</strong>
        <button type="button" class="rail-icon" aria-label={t("chat.channels.search")} onclick={openSearch}><Search size={15} /></button>
        <button type="button" class="rail-icon" aria-label={t("chat.channels.createTitle")} onclick={() => openCreate()}><Plus size={16} /></button>
      {/if}
      <button type="button" class="rail-icon" aria-label={t("chat.collapseRail")} onclick={onCollapse}><ChevronsLeft size={15} /></button>
    </div>

    {#if railError}<p class="mx-2 mb-1 rounded bg-destructive/10 px-2 py-1 text-xs text-destructive" role="alert">{railError}</p>{/if}

    <div class="min-h-0 flex-1 overflow-y-auto px-1.5 pb-3">
      {#if searchOpen && query.trim().length >= 2}
        <section class="message-results" aria-label={t("chat.organization.messageSearchResults")}>
          <div class="section-heading"><span>{t("chat.organization.messages")}</span>{#if messageSearchLoading}<LoaderCircle size={12} class="animate-spin" aria-label={t("common.loading")} />{/if}</div>
          {#each messageSearchResults as result (result.messageItemId)}
            <button type="button" class="message-result" onclick={() => void openMessageResult(result)}>
              <span><strong>{result.authorDisplayName}</strong><small>{projectName(result.projectId)} · #{result.channelName}{#if result.replyThreadId} · {t("chat.organization.thread")}{/if}</small></span>
              <span>{searchExcerpt(result)}</span>
            </button>
          {/each}
          {#if !messageSearchLoading && messageSearchResults.length === 0}<p class="px-2 py-1 text-xs text-muted-foreground">{t("chat.organization.noMessageSearchResults")}</p>{/if}
        </section>
      {/if}
      <section class="channel-section" role="group" ondragover={(event) => event.preventDefault()} ondrop={(event) => handleDrop(event, null)}>
        <div class="section-heading"><span>{t("chat.channels.defaultSection")}</span><button type="button" aria-label={t("chat.channels.createTitle")} onclick={() => openCreate()}><Plus size={13} /></button></div>
        {#if chat.channelsLoading && projects.selectedProjectId}
          <div class="channel-row channel-loading" role="status" aria-label={t("common.loading")}>
            <Hash size={14} class="shrink-0 opacity-75" />
            <span class="min-w-0 flex-1 truncate">general</span>
            <LoaderCircle size={12} class="animate-spin" />
          </div>
        {:else}
          {#each unsectionedChannels as channel (channel.id)}
            {@render ChannelRow({ channel })}
          {/each}
          {#if unsectionedChannels.length === 0 && query}<p class="px-2 py-1 text-xs text-muted-foreground">{t("chat.channels.noResults")}</p>{/if}
        {/if}
      </section>

      {#each sections as section (section.id)}
        <section class="channel-section" role="group" ondragover={(event) => event.preventDefault()} ondrop={(event) => handleDrop(event, section.id)}>
          <div class="section-heading">
            <button type="button" class="min-w-0 flex-1" onclick={() => toggleSection(section.id)}>{#if section.collapsed}<ChevronRight size={13} />{:else}<ChevronDown size={13} />{/if}<span class="truncate">{section.name}</span></button>
            <button type="button" aria-label={t("chat.channels.newInSection", section.name)} onclick={() => openCreate(section.id)}><Plus size={13} /></button>
            <details class="section-menu">
              <summary aria-label={t("chat.channels.deleteSection", section.name)}><Ellipsis size={13} /></summary>
              <div><button type="button" onclick={() => renameSection(section)}>{t("chat.rename")}</button><button type="button" class="danger" onclick={() => { deleteSectionCandidate = section; }}>{t("chat.channels.deleteSectionConfirm")}</button></div>
            </details>
          </div>
          {#if !section.collapsed}
            {#each sectionChannels(section) as channel (channel.id)}{@render ChannelRow({ channel })}{/each}
          {/if}
        </section>
      {/each}

      {#if creatingSection}
        <form class="mx-1 mt-2 flex gap-1" onsubmit={(event) => { event.preventDefault(); createSection(); }}><input class="h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-xs outline-none" bind:value={newSectionName} maxlength="80" placeholder={t("chat.channels.sectionName")} /><button type="submit" class="chat-primary-button px-2">{t("chat.channels.add")}</button></form>
      {:else}
        <button type="button" class="mx-1 mt-2 flex h-8 w-[calc(100%-0.5rem)] items-center gap-2 rounded-md px-2 text-xs text-muted-foreground hover:bg-accent hover:text-foreground" onclick={() => { creatingSection = true; }}><Plus size={14} />{t("chat.channels.newSection")}</button>
      {/if}

      <section class="channel-section mt-2">
        <div class="section-heading"><span>{t("chat.channels.directMessages")}</span><button type="button" aria-label={t("chat.channels.newDirectMessage")}><Plus size={13} /></button></div>
        <p class="mx-1 rounded-md px-2 py-1.5 text-xs text-muted-foreground">{t("chat.channels.emptyDirectMessages")}</p>
      </section>
    </div>

    <div class="shrink-0 border-t border-border p-2">
      <button type="button" class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground" class:bg-accent={chat.channelArchiveOpen} onclick={() => chat.openChannelArchive()}><Archive size={15} /><span class="min-w-0 flex-1 truncate">{t("chat.channels.archive")}</span>{#if chat.archivedChannels.length > 0}<span class="rounded bg-muted px-1.5 text-[0.666667rem]">{chat.archivedChannels.length}</span>{/if}</button>
    </div>
  </aside>
{:else if showCollapsedStrip}
  <aside class="flex h-full flex-col items-center bg-sidebar/45 py-2"><button type="button" class="rail-icon" aria-label={t("chat.openRail")} onclick={onExpand}><ChevronsRight size={16} /></button><button type="button" class="rail-icon mt-2" aria-label={t("chat.channels.createTitle")} onclick={() => openCreate()}><Plus size={16} /></button></aside>
{/if}

{#snippet ChannelRow({ channel }: { channel: ChatChannelRead })}
  {@const status = channelStatus(channel)}
  <div class="channel-row-group" role="listitem" draggable="true" ondragstart={(event) => beginDrag(event, channel.id)}>
    <button type="button" class="channel-row" class:selected={chat.selectedChannelId === channel.id && !chat.channelArchiveOpen} aria-current={chat.selectedChannelId === channel.id ? "page" : undefined} onclick={() => void selectChannel(channel.id)}>
      <Hash size={14} class="shrink-0 opacity-75" />
      <span class="min-w-0 flex-1 truncate">{channel.name}</span>
      <span class:attention={status.kind === "attention"} class:error={status.kind === "error"} class:unread={status.kind === "unread"} class="status" title={status.label}>
        {#if status.kind === "working"}<LoaderCircle size={12} class="animate-spin" />{:else if status.kind === "attention" || status.kind === "error"}<CircleAlert size={12} />{:else}<CircleDot size={10} />{/if}
      </span>
    </button>
    <details class="row-menu">
      <summary aria-label={t("chat.moreActions")}><Ellipsis size={13} /></summary>
      <div>
        <button type="button" onclick={() => { setupChannel = channel; setupSectionId = sections.find((section) => section.channelIds.includes(channel.id))?.id ?? null; }}>{t("chat.channels.edit")}</button>
        <span>{t("chat.channels.moveTo")}</span>
        <button type="button" onclick={() => moveChannel(channel.id, null)}>{t("chat.channels.defaultSection")}</button>
        {#each sections as section (section.id)}<button type="button" onclick={() => moveChannel(channel.id, section.id)}>{section.name}</button>{/each}
        {#if !channel.isDefault}<button type="button" class="danger" onclick={() => { archiveCandidate = channel; }}>{t("chat.archive")}</button>{/if}
      </div>
    </details>
  </div>
{/snippet}

{#if setupChannel !== undefined}
  <ChatChannelSetupDialog channel={setupChannel} {sections} initialSectionId={setupSectionId} onSaved={savedChannel} onCancel={() => { setupChannel = undefined; setupSectionId = null; }} />
{/if}

{#if archiveCandidate}<ConfirmDialog title={t("chat.channels.archiveConfirmTitle", archiveCandidate.name)} message={t("chat.channels.archiveConfirmMessage")} confirmLabel={t("chat.archive")} cancelLabel={t("chat.cancel")} onConfirm={() => void confirmArchive()} onCancel={() => { archiveCandidate = null; }} />{/if}
{#if deleteSectionCandidate}<ConfirmDialog title={t("chat.channels.deleteSectionTitle", deleteSectionCandidate.name)} message={t("chat.channels.deleteSectionMessage")} confirmLabel={t("chat.channels.deleteSectionConfirm")} cancelLabel={t("chat.cancel")} onConfirm={confirmDeleteSection} onCancel={() => { deleteSectionCandidate = null; }} />{/if}

<svelte:window onkeydown={(event) => { if (event.key === "Escape" && searchOpen) { searchOpen = false; query = ""; } }} />

<style>
  .rail-icon { display: grid; width: 1.75rem; height: 1.75rem; flex: 0 0 auto; place-items: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  .rail-icon:hover { background: var(--accent); color: var(--foreground); }
  .channel-section { padding-top: 0.35rem; }
  .message-results { margin-block:0.3rem 0.45rem; border-bottom:1px solid var(--border); padding-bottom:0.45rem; }
  .message-result { display:grid; width:100%; gap:0.18rem; border-radius:0.4rem; padding:0.42rem 0.5rem; text-align:left; }
  .message-result:hover,.message-result:focus-visible { background:var(--accent); }
  .message-result > span:first-child { display:flex; min-width:0; align-items:baseline; gap:0.35rem; }
  .message-result strong { flex:0 0 auto; font-size:0.72rem; }
  .message-result small { min-width:0; overflow:hidden; color:var(--muted-foreground); font-size:0.6rem; text-overflow:ellipsis; white-space:nowrap; }
  .message-result > span:last-child { display:-webkit-box; overflow:hidden; color:var(--muted-foreground); font-size:0.68rem; line-height:1rem; -webkit-box-orient:vertical; -webkit-line-clamp:2; line-clamp:2; }
  .section-heading { display: flex; min-height: 1.75rem; align-items: center; gap: 0.2rem; padding-inline: 0.45rem; color: var(--muted-foreground); font-size: 0.7rem; font-weight: 600; }
  .section-heading > span:first-child { min-width: 0; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .section-heading > button { display: flex; min-width: 1.35rem; min-height: 1.35rem; align-items: center; justify-content: center; gap: 0.2rem; border-radius: 0.3rem; }
  .section-heading > button:hover { background: var(--accent); color: var(--foreground); }
  .section-menu { position:relative; }
  .section-menu summary { display:grid;min-width:1.35rem;min-height:1.35rem;list-style:none;place-items:center;border-radius:0.3rem; }
  .section-menu summary::-webkit-details-marker { display:none; }
  .section-menu summary:hover { background:var(--accent);color:var(--foreground); }
  .section-menu > div { position:absolute;right:0;z-index:75;width:9rem;border:1px solid var(--border);border-radius:0.5rem;background:var(--popover);padding:0.25rem;box-shadow:0 12px 30px rgb(0 0 0 / 0.2); }
  .section-menu button { display:flex;width:100%;min-height:1.8rem;align-items:center;border-radius:0.3rem;padding-inline:0.5rem;text-align:left;font-size:0.7rem; }
  .section-menu button:hover { background:var(--accent); }
  .section-menu button.danger { color:var(--destructive); }
  .channel-row-group { position: relative; display: flex; min-width: 0; align-items: center; }
  .channel-row { display: flex; width: 100%; min-width: 0; height: 1.85rem; align-items: center; gap: 0.4rem; border-radius: 0.35rem; padding: 0 1.8rem 0 0.55rem; color: var(--muted-foreground); font-size: 0.8rem; text-align: left; }
  .channel-row:hover, .channel-row.selected { background: var(--accent); color: var(--foreground); }
  .channel-loading { padding-right: 0.55rem; opacity: 0.72; }
  .channel-row .status { display: grid; width: 0.9rem; flex: 0 0 auto; place-items: center; opacity: 0.55; }
  .channel-row .status.attention { color: var(--status-tentative); opacity: 1; }
  .channel-row .status.error { color: var(--destructive); opacity: 1; }
  .channel-row .status.unread { color: var(--primary); opacity: 1; }
  .row-menu { position: absolute; right: 0.25rem; }
  .row-menu summary { display: none; width: 1.45rem; height: 1.45rem; list-style: none; place-items: center; border-radius: 0.3rem; color: var(--muted-foreground); }
  .channel-row-group:hover .row-menu summary, .row-menu[open] summary { display: grid; }
  .row-menu summary::-webkit-details-marker { display: none; }
  .row-menu > div { position: fixed; z-index: 80; width: 11rem; transform: translate(-9.5rem, 0.2rem); border: 1px solid var(--border); border-radius: 0.5rem; background: var(--popover); padding: 0.25rem; box-shadow: 0 12px 30px rgb(0 0 0 / 0.2); }
  .row-menu button, .row-menu span { display: flex; width: 100%; min-height: 1.8rem; align-items: center; border-radius: 0.3rem; padding-inline: 0.5rem; text-align: left; font-size: 0.7rem; }
  .row-menu button:hover { background: var(--accent); }
  .row-menu span { min-height: 1.35rem; color: var(--muted-foreground); font-size: 0.6rem; }
  .row-menu button.danger { color: var(--destructive); }
</style>
