<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import Menu from "@lucide/svelte/icons/menu";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import {
    COMPACT_IDENTITY_EMOJI_SCALE,
    COMPACT_IDENTITY_ICON_SIZE,
    COMPACT_IDENTITY_ICON_STROKE_WIDTH,
  } from "$lib/icon-sizing";
  import { chatHeaderShowsResourcePath } from "$lib/chat/shell-model";
  import type { ProjectWorkingFolderId } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    projectNavigatorPanelGeometry,
    type ProjectNavigatorPanelMode,
  } from "$lib/projects/project-toolbar";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import WorkspaceBreadcrumbTerminalIcon from "$lib/components/WorkspaceBreadcrumbTerminalIcon.svelte";
  import ProjectNavigator from "$lib/components/projects/ProjectNavigator.svelte";
  import ChatContextNavigator from "./ChatContextNavigator.svelte";
  import ChatTitleEditor from "./ChatTitleEditor.svelte";

  type NavigatorMode = ProjectNavigatorPanelMode | "folders" | "threads";

  let {
    explorerExpanded,
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
  let navigatorMode = $state<NavigatorMode>("groups");
  let navigatorAnchorElement = $state<HTMLButtonElement | null>(null);
  let groupTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectTriggerElement = $state<HTMLButtonElement | null>(null);
  let folderTriggerElement = $state<HTMLButtonElement | null>(null);
  let threadTriggerElement = $state<HTMLButtonElement | null>(null);
  let identityElement = $state<HTMLDivElement | null>(null);
  let headerElement = $state<HTMLDivElement | null>(null);
  let navigatorPanelElement = $state<HTMLDivElement | null>(null);
  let navigatorPanelStyle = $state("");
  let navigatorPanelMaxHeight = $state(0);
  let showInactiveProjects = $state(false);
  let actionError = $state<string | null>(null);

  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedFolder = $derived(chat.selectedWorkingFolder);
  const selectedThread = $derived(chat.selectedThread);
  const showResourcePath = $derived(
    chatHeaderShowsResourcePath(explorerExpanded) || editingTitle,
  );
  const chatTitle = $derived(selectedThread?.title ?? t("chat.header.newChat"));

  function triggerForMode(mode: NavigatorMode): HTMLButtonElement | null {
    if (mode === "groups") return groupTriggerElement;
    if (mode === "projects") return projectTriggerElement;
    if (mode === "folders") return folderTriggerElement ?? projectTriggerElement;
    return threadTriggerElement ?? folderTriggerElement ?? projectTriggerElement;
  }

  function navigatorBounds() {
    const rect = headerElement?.closest(".chat-workspace")?.getBoundingClientRect();
    return rect
      ? { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom }
      : { left: 0, right: viewport.width, top: 0, bottom: viewport.height };
  }

  function refreshNavigatorGeometry(): void {
    if (!navigatorOpen || !navigatorAnchorElement) return;
    const anchor = navigatorAnchorElement.getBoundingClientRect();
    const bounds = navigatorBounds();
    const geometry = projectNavigatorPanelGeometry({
      anchorLeft: anchor.left,
      anchorBottom: anchor.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
      boundsLeft: bounds.left,
      boundsRight: bounds.right,
      boundsTop: bounds.top,
      boundsBottom: bounds.bottom,
    });
    navigatorPanelStyle = [
      `left: ${Math.round(geometry.left)}px`,
      `top: ${Math.round(geometry.top)}px`,
      `width: ${Math.round(geometry.width)}px`,
    ].join("; ");
    navigatorPanelMaxHeight = geometry.height;
  }

  function openNavigator(mode: NavigatorMode): void {
    if (editingTitle) return;
    navigatorMode = mode;
    navigatorAnchorElement = triggerForMode(mode);
    navigatorOpen = true;
    refreshNavigatorGeometry();
    requestAnimationFrame(refreshNavigatorGeometry);
  }

  function toggleNavigator(mode: NavigatorMode): void {
    if (navigatorOpen && navigatorMode === mode) {
      navigatorOpen = false;
      return;
    }
    openNavigator(mode);
  }

  function handleWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (
      navigatorOpen
      && !identityElement?.contains(target)
      && !navigatorPanelElement?.contains(target)
    ) {
      navigatorOpen = false;
    }
  }

  function selectProject(): void {
    navigatorOpen = false;
    void chat.syncProjectSelection(projects.selectedProjectId);
  }

  function selectFolder(workingFolderId: ProjectWorkingFolderId): void {
    navigatorOpen = false;
    chat.selectWorkingFolder(workingFolderId);
  }

  function newChat(workingFolderId = chat.selectedWorkingFolderId): void {
    const workingFolder = workingFolderId
      ? chat.workingFolders.find((entry) => entry.workingFolder.id === workingFolderId)
      : null;
    if (
      !workingFolderId
      || workingFolder?.bindingStatus !== "available"
      || workingFolder.workingFolder.archivedAt !== null
    ) {
      openNavigator("folders");
      return;
    }
    navigatorOpen = false;
    chat.newDraft(workingFolderId);
  }

  async function addFolder(): Promise<void> {
    const projectId = projects.selectedProjectId;
    if (!projectId) return;
    actionError = null;
    try {
      const folder = await chat.addExternalWorkingFolder({
        id: `working-folder:${crypto.randomUUID()}`,
        projectId,
        displayName: "",
      }, t("projects.settings.workingFolders.pickerTitle"));
      if (folder) navigatorOpen = false;
    } catch (error: unknown) {
      actionError = error instanceof Error ? error.message : String(error);
    }
  }

  function beginRename(): void {
    if (!selectedThread) return;
    navigatorOpen = false;
    editingTitle = true;
  }

  async function commitTitle(title: string): Promise<void> {
    if (!selectedThread || !editingTitle) return;
    await chat.renameThread(selectedThread, title);
    editingTitle = false;
  }

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((error: unknown) => {
      actionError = error instanceof Error ? error.message : String(error);
    });
  }

  $effect(() => {
    if (!navigatorOpen) return;
    const width = viewport.width;
    const height = viewport.height;
    void width;
    void height;
    requestAnimationFrame(refreshNavigatorGeometry);
  });
</script>

<svelte:window onpointerdown={handleWindowPointerDown} />

<div
  bind:this={headerElement}
  class={cn(
    "chat-workspace-header flex min-w-0 items-center gap-1 overflow-x-auto pl-3",
  )}
  style={`height: var(--cal-header-row-h); background-color: var(--cal-header-bg); border-bottom: 1px solid var(--sidebar); padding-right: var(--chat-header-action-inset, ${reserveGlobalActions ? "6.5rem" : "0.75rem"});`}
  onscroll={refreshNavigatorGeometry}
  data-chat-workspace-header
>
  {#if showRailButton}
    <button type="button" class="chat-toolbar-icon-button" aria-label={t("chat.openRail")} onclick={onOpenRail}>
      <Menu size={14} strokeWidth={1.75} />
    </button>
  {/if}
  <div bind:this={identityElement} class="relative min-w-36 shrink-0 min-[760px]:max-w-2xl">
    <div class="flex h-7 min-w-0 max-w-full items-center gap-0.5 text-sm">
      {#if selectedProject && selectedGroup}
        <button
          bind:this={groupTriggerElement}
          type="button"
          class={cn("chat-context-segment", navigatorOpen && navigatorMode === "groups" && "bg-accent text-accent-foreground")}
          aria-label={t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "groups"}
          data-chat-group-trigger
          onpointerenter={() => openNavigator("groups")}
          onclick={() => toggleNavigator("groups")}
        >
          <ProjectIcon name={selectedGroup.icon} size={identityIconSize} strokeWidth={identityIconStrokeWidth} emojiScale={identityEmojiScale} class="shrink-0" />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedGroup.name}</span>
        </button>
        <span class="chat-context-divider">/</span>
        <button
          bind:this={projectTriggerElement}
          type="button"
          class={cn("chat-context-segment", navigatorOpen && navigatorMode === "projects" && "bg-accent text-accent-foreground")}
          aria-label={t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "projects"}
          data-chat-project-trigger
          onpointerenter={() => openNavigator("projects")}
          onclick={() => toggleNavigator("projects")}
        >
          <ProjectIcon name={selectedProject.icon} size={identityIconSize} strokeWidth={identityIconStrokeWidth} emojiScale={identityEmojiScale} class="shrink-0" />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedProject.name}</span>
          {#if selectedProject.status !== "active"}
            <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>{projectLifecycleLabel(selectedProject.status, t)}</span>
          {/if}
          {#if !showResourcePath || !selectedFolder}
            <WorkspaceBreadcrumbTerminalIcon kind="chevron" context="chat" class="shrink-0 text-muted-foreground" />
          {/if}
        </button>
        {#if showResourcePath && selectedFolder}
          <span class="chat-context-divider">/</span>
          <button
            bind:this={folderTriggerElement}
            type="button"
            class={cn("chat-context-segment", navigatorOpen && navigatorMode === "folders" && "bg-accent text-accent-foreground")}
            aria-label={t("chat.header.workingFolder")}
            aria-expanded={navigatorOpen && navigatorMode === "folders"}
            data-chat-folder-trigger
            onpointerenter={() => openNavigator("folders")}
            onclick={() => toggleNavigator("folders")}
          >
            <Folder size={identityIconSize} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
            <span class="min-w-0 truncate font-semibold text-foreground">{selectedFolder.workingFolder.displayName}</span>
          </button>
          <span class="chat-context-divider">/</span>
          {#if editingTitle && selectedThread}
            <div class="min-w-36 max-w-64 px-1">
              <ChatTitleEditor title={selectedThread.title} onCommit={commitTitle} onCancel={() => { editingTitle = false; }} />
            </div>
          {:else}
            <button
              bind:this={threadTriggerElement}
              type="button"
              class={cn("chat-context-segment", navigatorOpen && navigatorMode === "threads" && "bg-accent text-accent-foreground")}
              aria-label={chatTitle}
              aria-expanded={navigatorOpen && navigatorMode === "threads"}
              data-chat-thread-trigger
              onpointerenter={() => openNavigator("threads")}
              onclick={() => toggleNavigator("threads")}
            >
              <MessageSquare size={identityIconSize} strokeWidth={identityIconStrokeWidth} class="shrink-0" />
              <span class="min-w-0 truncate font-semibold text-foreground">{chatTitle}</span>
              <WorkspaceBreadcrumbTerminalIcon kind="chevron" context="chat" class="shrink-0 text-muted-foreground" />
            </button>
          {/if}
        {/if}
        <button type="button" class="chat-inline-new-button" aria-label={t("chat.newChat")} data-chat-new-button onclick={() => newChat()}>
          <WorkspaceBreadcrumbTerminalIcon kind="plus" />
        </button>
      {:else}
        <div class="chat-context-segment">
          <MessageSquare size={identityIconSize} strokeWidth={identityIconStrokeWidth} />
          <span class="font-semibold">{t("chat.title")}</span>
        </div>
      {/if}
    </div>
    {#if navigatorOpen}
      <div bind:this={navigatorPanelElement} class="fixed z-80" style={navigatorPanelStyle} role="dialog" tabindex="-1" aria-label={navigatorMode === "groups" || navigatorMode === "projects" ? t("projects.navigator.pickerLabel") : t("chat.header.contextNavigator")}>
        {#if navigatorMode === "groups" || navigatorMode === "projects"}
          <ProjectNavigator
            selectedProjectId={projects.selectedProjectId}
            selectedGroupId={selectedGroup?.id ?? null}
            iconStrokeWidth={identityIconStrokeWidth}
            {showInactiveProjects}
            panelMode={navigatorMode}
            panelMaxHeight={navigatorPanelMaxHeight}
            onShowInactiveProjectsChange={(value) => { showInactiveProjects = value; }}
            onProjectSelected={selectProject}
          />
        {:else}
          <ChatContextNavigator
            mode={navigatorMode}
            projectId={projects.selectedProjectId}
            workingFolderId={chat.selectedWorkingFolderId}
            panelMaxHeight={navigatorPanelMaxHeight}
            onFolderSelected={selectFolder}
            onThreadSelected={(thread) => { navigatorOpen = false; chat.selectThread(thread.id); }}
            onNewChat={newChat}
            onAddFolder={() => { void addFolder(); }}
          />
        {/if}
      </div>
    {/if}
  </div>
  <div class="flex-1"></div>
  {#if actionError}<p role="alert" class="max-w-40 truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
  <div class="flex shrink-0 items-center gap-1">
    {#if selectedFolder?.bindingStatus === "available"}
      <button type="button" class="chat-header-action" title={t("chat.openFolder")} onclick={() => run(() => chat.openWorkingFolder(selectedFolder.workingFolder.id))}>
        <FolderOpen size={14} strokeWidth={1.75} />
        <span class="hidden @min-[760px]:inline">{t("chat.header.open")}</span>
      </button>
    {/if}
    {#if selectedFolder?.currentBranch}
      <span class="chat-branch" title={t("chat.header.branch", selectedFolder.currentBranch)}><GitBranch size={13} strokeWidth={1.75} /><span>{selectedFolder.currentBranch}</span></span>
    {/if}
  </div>
</div>

<style>
  .chat-context-segment { display: flex; height: 1.75rem; min-width: 0; align-items: center; gap: 0.375rem; border-radius: 0.375rem; padding-inline: 0.375rem; text-align: left; }
  button.chat-context-segment:hover { background: var(--accent); color: var(--accent-foreground); }
  .chat-context-divider { flex: 0 0 auto; padding-inline: 0.125rem; font-weight: 600; color: var(--muted-foreground); }
  .chat-inline-new-button { display: flex; height: 1.75rem; width: 1.25rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  .chat-inline-new-button:hover { background: var(--accent); color: var(--accent-foreground); }
  .chat-toolbar-icon-button { display: flex; height: 1.75rem; width: 1.75rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); transition: color 120ms ease, background-color 120ms ease; }
  .chat-toolbar-icon-button:hover { background: var(--accent); color: var(--foreground); }
  .chat-header-action { display: inline-flex; min-height: 1.75rem; align-items: center; gap: 0.375rem; border-radius: 0.375rem; padding-inline: 0.5rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .chat-header-action:hover { background: var(--accent); color: var(--foreground); }
  .chat-branch { display: none; min-width: 0; max-width: 9rem; align-items: center; gap: 0.3rem; border-radius: 0.375rem; padding: 0.25rem 0.4rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-branch span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  @container chat-shell (min-width: 760px) { .chat-branch { display: inline-flex; } }
  @media (prefers-reduced-motion: reduce) { .chat-toolbar-icon-button { transition: none; } }
</style>
