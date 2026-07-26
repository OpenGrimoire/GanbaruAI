<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import Menu from "@lucide/svelte/icons/menu";
  import PanelBottom from "@lucide/svelte/icons/panel-bottom";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Pencil from "@lucide/svelte/icons/pencil";
  import SquareArrowOutUpRight from "@lucide/svelte/icons/square-arrow-out-up-right";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getNavigation } from "$lib/stores/navigation.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { openDetachedViewWindow } from "$lib/windows/detached";
  import ChatTitleEditor from "./ChatTitleEditor.svelte";

  let {
    onOpenRail,
    onToggleBottomPanel,
    showRailButton = false,
    bottomPanelOpen = false,
    draft = false,
  }: {
    onOpenRail: () => void;
    onToggleBottomPanel: () => void;
    showRailButton?: boolean;
    bottomPanelOpen?: boolean;
    draft?: boolean;
  } = $props();
  const { t } = getLocalization();
  const chat = getChat();
  const navigation = getNavigation();
  const projects = getProjects();
  let editing = $state(false);
  let actionError = $state<string | null>(null);
  const thread = $derived(chat.selectedThread);
  const workspace = $derived(chat.selectedWorkingFolder);
  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const availableWorkingFolders = $derived(chat.workingFolders.filter((entry) => (
    entry.workingFolder.projectId === projects.selectedProjectId
      && entry.workingFolder.archivedAt === null
  )));

  function beginEdit(): void {
    if (!thread) return;
    editing = true;
  }

  async function commitTitle(title: string): Promise<void> {
    if (!thread || !editing) return;
    await chat.renameThread(thread, title);
    editing = false;
  }

  function cancelEdit(): void {
    editing = false;
  }

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((error: unknown) => {
      actionError = error instanceof Error ? error.message : String(error);
    });
  }
</script>

<header class="chat-conversation-header @container" class:draft>
  <button type="button" class="chat-icon-button rail-open-button" class:visible={showRailButton} aria-label={t("chat.openRail")} onclick={onOpenRail}><Menu size={16} /></button>
  <nav class="project-context" aria-label={t("chat.header.projectContext")}>
    {#if selectedGroup}<button type="button" class="context-segment context-group" onclick={() => navigation.navigate("projects")}>{selectedGroup.name}</button><span>/</span>{/if}
    <button type="button" class="context-segment" onclick={() => navigation.navigate("projects")}>{selectedProject?.name ?? t("chat.header.chooseProject")}</button>
    <span>/</span>
    <details class="working-folder-picker">
      <summary class="context-segment" aria-label={t("chat.header.workingFolder")}>
        <span>{workspace?.workingFolder.displayName ?? t("chat.header.chooseWorkingFolder")}</span>
        {#if draft && availableWorkingFolders.length > 1}<ChevronDown size={12} />{/if}
      </summary>
      {#if draft && availableWorkingFolders.length > 1}
        <div class="chat-menu left-0 top-7">
          {#each availableWorkingFolders as entry (entry.workingFolder.id)}
            <button type="button" onclick={() => chat.selectWorkingFolder(entry.workingFolder.id)}>
              <span>{entry.workingFolder.displayName}</span>
              <small>{entry.workingFolder.kind === "managed" ? t("chat.header.managedFolder") : t("chat.header.externalFolder")}</small>
            </button>
          {/each}
        </div>
      {/if}
    </details>
  </nav>
  <div class="title-region" class:empty={draft}>
    {#if actionError}<p role="alert" class="truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
    {#if !draft}
      {#if editing && thread}
        <ChatTitleEditor title={thread.title} onCommit={commitTitle} onCancel={cancelEdit} />
      {:else}
        <h1 class="truncate text-sm font-medium">{thread?.title || t("chat.header.newChat")}</h1>
        {#if thread}<button type="button" class="chat-title-edit" aria-label={t("chat.header.editTitle")} onclick={beginEdit}><Pencil size={12} /></button>{/if}
      {/if}
    {/if}
  </div>
  <div class="header-actions">
    {#if !draft && workspace?.bindingStatus === "available"}<button type="button" class="chat-header-action" title={t("chat.openFolder")} onclick={() => run(() => chat.openWorkingFolder(workspace.workingFolder.id))}><FolderOpen size={14} /><span class="hidden @min-[620px]:inline">{t("chat.header.open")}</span></button>{/if}
    {#if !draft && workspace?.currentBranch}<span class="chat-branch" title={t("chat.header.branch", workspace.currentBranch)}><GitBranch size={13} /><span>{workspace.currentBranch}</span></span>{/if}
    <button type="button" class="chat-icon-button" class:active={bottomPanelOpen} aria-label={bottomPanelOpen ? t("chat.closeBottomPanel") : t("chat.openBottomPanel")} aria-pressed={bottomPanelOpen} onclick={onToggleBottomPanel}><PanelBottom size={16} /></button>
    <button type="button" class="chat-icon-button" class:active={chat.inspectorOpen} aria-label={chat.inspectorOpen ? t("chat.closeInspector") : t("chat.openInspector")} aria-pressed={chat.inspectorOpen} onclick={() => { chat.inspectorOpen = !chat.inspectorOpen; }}><PanelRight size={16} /></button>
    <details class="relative"><summary class="chat-icon-button list-none" aria-label={t("chat.moreActions")}><Ellipsis size={16} /></summary><div class="chat-menu right-0 top-8">{#if workspace?.bindingStatus === "available"}<button type="button" onclick={() => run(() => chat.openWorkingFolder(workspace.workingFolder.id))}><FolderOpen size={13} />{t("chat.openFolder")}</button>{/if}{#if thread}<button type="button" onclick={() => run(() => openDetachedViewWindow("chat"))}><SquareArrowOutUpRight size={13} />{t("chat.detach")}</button>{#if !thread.archivedAt}<button type="button" onclick={() => run(() => chat.archiveThread(thread))}><Archive size={13} />{t("chat.archive")}</button>{/if}{/if}</div></details>
  </div>
</header>

<style>
  .chat-conversation-header { display: flex; min-height: 3.05rem; flex: 0 0 auto; align-items: center; gap: 0.5rem; border-bottom: 1px solid var(--border); padding-inline: 0.85rem 0.6rem; background: var(--cal-bg); }
  .chat-conversation-header.draft { position: absolute; inset-inline: 0; top: 0; z-index: 25; border-bottom: 0; background: transparent; pointer-events: none; }
  .chat-conversation-header.draft button, .chat-conversation-header.draft details { pointer-events: auto; }
  .rail-open-button { display: none; }
  .rail-open-button.visible { display: inline-flex; }
  .project-context { display: flex; min-width: 0; max-width: min(48%, 34rem); align-items: center; gap: 0.3rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .context-segment { display: inline-flex; min-width: 0; max-width: 11rem; align-items: center; gap: 0.25rem; border-radius: 0.35rem; padding: 0.2rem 0.3rem; color: var(--foreground); }
  .context-segment > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .context-segment:hover { background: var(--accent); }
  .working-folder-picker { position: relative; min-width: 0; }
  .working-folder-picker summary { list-style: none; }
  .working-folder-picker summary::-webkit-details-marker { display: none; }
  .working-folder-picker .chat-menu button { display: flex; min-width: 13rem; flex-direction: column; align-items: flex-start; gap: 0.1rem; }
  .working-folder-picker small { color: var(--muted-foreground); font-size: 0.633333rem; }
  .title-region { display: flex; min-width: 0; flex: 1; align-items: center; gap: 0.3rem; }
  .title-region.empty { min-height: 1px; }
  .header-actions { display: flex; min-width: 0; flex: 0 0 auto; align-items: center; gap: 0.35rem; }
  .chat-title-edit { display: inline-flex; width: 1.65rem; height: 1.65rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.4rem; color: var(--muted-foreground); opacity: 0; }
  .chat-conversation-header:hover .chat-title-edit, .chat-title-edit:focus-visible { opacity: 1; }
  .chat-title-edit:hover { background: var(--accent); color: var(--foreground); }
  .chat-header-action { display: inline-flex; min-height: 1.9rem; align-items: center; gap: 0.4rem; border: 1px solid var(--border); border-radius: 0.55rem; padding: 0.25rem 0.55rem; color: var(--foreground); font-size: 0.733333rem; box-shadow: 0 1px 2px rgb(0 0 0 / 0.04); }
  .chat-header-action:hover { background: var(--accent); }
  .chat-branch { display: none; min-width: 0; max-width: 9rem; align-items: center; gap: 0.3rem; border: 1px solid var(--border); border-radius: 0.5rem; padding: 0.3rem 0.45rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-branch span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  :global(.chat-icon-button.active) { background: var(--accent); color: var(--foreground); }
  @container (min-width: 620px) { .chat-branch { display: inline-flex; } }
  @container (max-width: 560px) { .context-group, .context-group + span { display: none; } .project-context { max-width: 58%; } .title-region:not(.empty) { display: none; } }
  @media (hover: none) { .chat-title-edit { opacity: 1; } }
  :global(.chat-menu button) { display: flex; align-items: center; gap: 0.5rem; }
</style>
