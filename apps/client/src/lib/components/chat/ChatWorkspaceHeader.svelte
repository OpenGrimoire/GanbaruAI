<script lang="ts">
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import Hash from "@lucide/svelte/icons/hash";
  import Menu from "@lucide/svelte/icons/menu";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Users from "@lucide/svelte/icons/users";
  import {
    COMPACT_IDENTITY_EMOJI_SCALE,
    COMPACT_IDENTITY_ICON_SIZE,
    COMPACT_IDENTITY_ICON_STROKE_WIDTH,
  } from "$lib/icon-sizing";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { projectLifecycleBadgeClass, projectLifecycleLabel } from "$lib/projects/project-display";
  import { projectNavigatorPanelGeometry, type ProjectNavigatorPanelMode } from "$lib/projects/project-toolbar";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import WorkspaceBreadcrumbTerminalIcon from "$lib/components/WorkspaceBreadcrumbTerminalIcon.svelte";
  import ChatChannelPickerPanel from "./ChatChannelPickerPanel.svelte";
  import ChatProjectNavigator from "./ChatProjectNavigator.svelte";
  import ChatTitleEditor from "./ChatTitleEditor.svelte";

  type ChatNavigatorMode = ProjectNavigatorPanelMode | "channels";

  let {
    explorerExpanded: _explorerExpanded,
    showRailButton,
    onOpenRail,
    reserveGlobalActions,
    editingTitle = $bindable(false),
  }: {
    explorerExpanded: boolean;
    showRailButton: boolean;
    onOpenRail: () => void;
    reserveGlobalActions: boolean;
    editingTitle?: boolean;
  } = $props();

  const chat = getChat();
  const projects = getProjects();
  const viewport = getViewport();
  const { t } = getLocalization();
  const identityIconSize = COMPACT_IDENTITY_ICON_SIZE;
  const identityIconStrokeWidth = COMPACT_IDENTITY_ICON_STROKE_WIDTH;
  const identityEmojiScale = COMPACT_IDENTITY_EMOJI_SCALE;
  let navigatorOpen = $state(false);
  let navigatorMode = $state<ChatNavigatorMode>("groups");
  let navigatorAnchorElement = $state<HTMLButtonElement | null>(null);
  let groupTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectTriggerElement = $state<HTMLButtonElement | null>(null);
  let channelTriggerElement = $state<HTMLButtonElement | null>(null);
  let identityElement = $state<HTMLDivElement | null>(null);
  let headerElement = $state<HTMLDivElement | null>(null);
  let navigatorPanelElement = $state<HTMLDivElement | null>(null);
  let navigatorPanelStyle = $state("");
  let navigatorPanelMaxHeight = $state(0);
  let showInactiveProjects = $state(false);
  let actionError = $state<string | null>(null);
  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedChannel = $derived(chat.selectedChannel);
  const selectedProjectChannels = $derived(
    selectedProject ? chat.channelsForProject(selectedProject.id) : [],
  );
  const channelNavigatorPanelHeight = $derived(Math.min(
    navigatorPanelMaxHeight,
    Math.max(124, selectedProjectChannels.length * 32 + 84),
  ));
  const selectedFolder = $derived(chat.selectedWorkingFolder);

  function triggerForMode(mode: ChatNavigatorMode): HTMLButtonElement | null {
    if (mode === "groups") return groupTriggerElement;
    if (mode === "projects") return projectTriggerElement;
    return channelTriggerElement;
  }

  function refreshNavigatorGeometry(): void {
    if (!navigatorOpen || !navigatorAnchorElement) return;
    const anchor = navigatorAnchorElement.getBoundingClientRect();
    const rect = headerElement?.closest(".chat-workspace")?.getBoundingClientRect();
    const geometry = projectNavigatorPanelGeometry({
      anchorLeft: anchor.left,
      anchorBottom: anchor.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
      boundsLeft: rect?.left ?? 0,
      boundsRight: rect?.right ?? viewport.width,
      boundsTop: rect?.top ?? 0,
      boundsBottom: rect?.bottom ?? viewport.height,
    });
    navigatorPanelStyle = `left:${Math.round(geometry.left)}px;top:${Math.round(geometry.top)}px;width:${Math.round(geometry.width)}px`;
    navigatorPanelMaxHeight = geometry.height;
  }

  function openNavigator(mode: ChatNavigatorMode): void {
    if (editingTitle) return;
    navigatorMode = mode;
    navigatorAnchorElement = triggerForMode(mode);
    navigatorOpen = true;
    requestAnimationFrame(refreshNavigatorGeometry);
  }

  function toggleNavigator(mode: ChatNavigatorMode): void {
    if (navigatorOpen && navigatorMode === mode) navigatorOpen = false;
    else openNavigator(mode);
  }

  function handleWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (navigatorOpen && !identityElement?.contains(target) && !navigatorPanelElement?.contains(target)) navigatorOpen = false;
  }

  function selectProject(): void {
    navigatorOpen = false;
  }

  async function selectChannel(channelId: string): Promise<void> {
    await chat.selectChannel(channelId);
    navigatorOpen = false;
  }

  function createChannel(): void {
    navigatorOpen = false;
    window.dispatchEvent(new Event("ganbaru-ai:chat-new-channel"));
  }

  async function commitTitle(title: string): Promise<void> {
    if (!selectedChannel || selectedChannel.isDefault || !editingTitle) return;
    await chat.updateChannelDetails(selectedChannel, title, selectedChannel.topic);
    editingTitle = false;
  }

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((error: unknown) => { actionError = error instanceof Error ? error.message : String(error); });
  }

  $effect(() => {
    if (!navigatorOpen) return;
    void viewport.width;
    void viewport.height;
    requestAnimationFrame(refreshNavigatorGeometry);
  });
</script>

<svelte:window onpointerdown={handleWindowPointerDown} />

<div bind:this={headerElement} class="chat-workspace-header flex min-w-0 items-center gap-1 overflow-x-auto pl-3" style={`height:var(--cal-header-row-h);background-color:var(--cal-header-bg);border-bottom:1px solid var(--sidebar);padding-right:var(--chat-header-action-inset, ${reserveGlobalActions ? "6.5rem" : "0.75rem"})`} onscroll={refreshNavigatorGeometry} data-chat-workspace-header>
  {#if showRailButton}<button type="button" class="chat-toolbar-icon-button" aria-label={t("chat.openRail")} onclick={onOpenRail}><Menu size={14} /></button>{/if}
  <div bind:this={identityElement} class="relative min-w-36 shrink-0 min-[760px]:max-w-3xl">
    <div class="flex h-7 min-w-0 max-w-full items-center gap-0.5 text-sm">
      {#if selectedProject && selectedGroup}
        <button bind:this={groupTriggerElement} type="button" class={cn("chat-context-segment", navigatorOpen && navigatorMode === "groups" && "bg-accent")} aria-expanded={navigatorOpen && navigatorMode === "groups"} data-chat-group-trigger onpointerenter={() => openNavigator("groups")} onclick={() => toggleNavigator("groups")}><ProjectIcon name={selectedGroup.icon} size={identityIconSize} strokeWidth={identityIconStrokeWidth} emojiScale={identityEmojiScale} /><span class="truncate font-semibold">{selectedGroup.name}</span></button>
        <span class="chat-context-divider">/</span>
        <button bind:this={projectTriggerElement} type="button" class={cn("chat-context-segment", navigatorOpen && navigatorMode === "projects" && "bg-accent")} aria-expanded={navigatorOpen && navigatorMode === "projects"} data-chat-project-trigger onpointerenter={() => openNavigator("projects")} onclick={() => toggleNavigator("projects")}><ProjectIcon name={selectedProject.icon} size={identityIconSize} strokeWidth={identityIconStrokeWidth} emojiScale={identityEmojiScale} /><span class="truncate font-semibold">{selectedProject.name}</span>{#if selectedProject.status !== "active"}<span class={cn("rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>{projectLifecycleLabel(selectedProject.status, t)}</span>{/if}</button>
        {#if selectedChannel}
          <span class="chat-context-divider">/</span>
          {#if editingTitle && !selectedChannel.isDefault}
            <div class="min-w-36 max-w-64 px-1"><ChatTitleEditor title={selectedChannel.name} onCommit={commitTitle} onCancel={() => { editingTitle = false; }} /></div>
          {:else}
            <button bind:this={channelTriggerElement} type="button" class={cn("chat-context-segment", navigatorOpen && navigatorMode === "channels" && "bg-accent")} aria-label={t("chat.channels.navigatorLabel")} aria-expanded={navigatorOpen && navigatorMode === "channels"} data-chat-channel-trigger onpointerenter={() => openNavigator("channels")} onclick={() => toggleNavigator("channels")}><Hash size={identityIconSize} strokeWidth={identityIconStrokeWidth} class="shrink-0" /><span class="truncate font-semibold">{selectedChannel.name}</span><WorkspaceBreadcrumbTerminalIcon kind="chevron" class="shrink-0 text-muted-foreground" /></button>
          {/if}
          {#if selectedChannel.topic}<span class="hidden max-w-80 truncate px-1 text-xs text-muted-foreground @min-[760px]:inline">{selectedChannel.topic}</span>{/if}
        {/if}
        <button type="button" class="chat-inline-new-button" aria-label={t("chat.channels.createTitle")} data-chat-new-channel-button onclick={() => window.dispatchEvent(new Event("ganbaru-ai:chat-new-channel"))}><WorkspaceBreadcrumbTerminalIcon kind="plus" /></button>
      {:else}
        <div class="chat-context-segment"><MessageSquare size={identityIconSize} /><span class="font-semibold">{t("chat.title")}</span></div>
      {/if}
    </div>
    {#if navigatorOpen}
      <div bind:this={navigatorPanelElement} class="fixed z-80" style={navigatorPanelStyle} role="dialog" tabindex="-1" aria-label={navigatorMode === "channels" ? t("chat.channels.navigatorLabel") : t("projects.navigator.pickerLabel")}>
        {#if navigatorMode === "channels"}
          <ChatChannelPickerPanel
            channels={selectedProjectChannels}
            selectedChannelId={selectedChannel?.id ?? null}
            frameStyle={`width:100%;height:${channelNavigatorPanelHeight}px;max-height:${channelNavigatorPanelHeight}px`}
            iconStrokeWidth={identityIconStrokeWidth}
            onChannelSelected={(channel) => selectChannel(channel.id)}
            onCreateChannel={createChannel}
          />
        {:else}
          <ChatProjectNavigator
            selectedProjectId={projects.selectedProjectId}
            selectedGroupId={selectedGroup?.id ?? null}
            iconStrokeWidth={identityIconStrokeWidth}
            {showInactiveProjects}
            panelMode={navigatorMode}
            panelMaxHeight={navigatorPanelMaxHeight}
            onShowInactiveProjectsChange={(value) => { showInactiveProjects = value; }}
            onProjectSelected={selectProject}
            onChannelSelected={() => { navigatorOpen = false; }}
            onCreateChannel={() => { createChannel(); }}
          />
        {/if}
      </div>
    {/if}
  </div>
  <div class="flex-1"></div>
  {#if actionError}<p role="alert" class="max-w-40 truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
  <div class="flex shrink-0 items-center gap-1">
    {#if selectedFolder?.bindingStatus === "available"}<button type="button" class="chat-toolbar-icon-button" title={t("chat.openFolder")} aria-label={t("chat.openFolder")} onclick={() => run(() => chat.openWorkingFolder(selectedFolder.workingFolder.id))}><FolderOpen size={14} /></button>{/if}
    {#if selectedChannel}
      <button type="button" class="chat-toolbar-icon-button" title={t("chat.organization.manageMembers")} aria-label={t("chat.organization.manageMembers")} onclick={() => window.dispatchEvent(new Event("ganbaru-ai:chat-manage-members"))}><Users size={14} /></button>
    {/if}
    {#if selectedFolder?.currentBranch}<span class="chat-branch" title={t("chat.header.branch", selectedFolder.currentBranch)}><GitBranch size={13} /><span>{selectedFolder.currentBranch}</span></span>{/if}
  </div>
</div>

<style>
  .chat-context-segment { display:flex;height:1.75rem;min-width:0;align-items:center;gap:0.375rem;border-radius:0.375rem;padding-inline:0.375rem;text-align:left; }
  button.chat-context-segment:hover { background:var(--accent);color:var(--accent-foreground); }
  .chat-context-divider { flex:0 0 auto;padding-inline:0.125rem;font-weight:600;color:var(--muted-foreground); }
  .chat-inline-new-button,.chat-toolbar-icon-button { display:flex;height:1.75rem;width:1.75rem;flex:0 0 auto;align-items:center;justify-content:center;border-radius:0.375rem;color:var(--foreground); }
  .chat-inline-new-button:hover,.chat-toolbar-icon-button:hover { background:var(--accent); }
  .chat-branch { display:none;min-width:0;max-width:9rem;align-items:center;gap:0.3rem;border-radius:0.375rem;padding:0.25rem 0.4rem;color:var(--muted-foreground);font-size: calc(0.666667rem * var(--type-scale)); }
  .chat-branch span { overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  @container chat-shell (min-width:760px) { .chat-branch { display:inline-flex; } }
</style>
