<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import Ellipsis from "@lucide/svelte/icons/ellipsis";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Menu from "@lucide/svelte/icons/menu";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import PanelRightClose from "@lucide/svelte/icons/panel-right-close";
  import Pencil from "@lucide/svelte/icons/pencil";
  import SquareArrowOutUpRight from "@lucide/svelte/icons/square-arrow-out-up-right";
  import { threadStatus } from "$lib/chat/shell-model";
  import { middleTruncate } from "$lib/chat/responsive-layout";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { openDetachedViewWindow } from "$lib/windows/detached";
  import ChatTitleEditor from "./ChatTitleEditor.svelte";

  let { onOpenRail, showRailButton = false }: { onOpenRail: () => void; showRailButton?: boolean } = $props();
  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  let editing = $state(false);
  let actionError = $state<string | null>(null);
  const thread = $derived(chat.selectedThread);
  const workspace = $derived(chat.selectedWorkspace);
  const project = $derived(workspace?.workspace.projectId ? projects.projectById(workspace.workspace.projectId) : null);
  const provider = $derived(thread ? chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === thread.providerInstanceId) ?? null : null);

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

  function statusLabel(): string {
    if (!thread) return t("chat.status.idle");
    switch (threadStatus(thread)) {
      case "waiting_answer": return t("chat.status.waitingAnswer");
      case "waiting_approval": return t("chat.status.waitingApproval");
      case "working": return t("chat.status.working");
      case "error": return t("chat.status.error");
      case "archived": return t("chat.status.archived");
      default: return t("chat.status.idle");
    }
  }

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((error: unknown) => {
      actionError = error instanceof Error ? error.message : String(error);
    });
  }
</script>

<header class="chat-conversation-header @container">
  <button type="button" class="chat-icon-button rail-open-button" class:visible={showRailButton} aria-label={t("chat.openRail")} onclick={onOpenRail}><Menu size={16} /></button>
  <div class="flex min-w-0 flex-1 items-center gap-2">
    {#if actionError}<p role="alert" class="truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
    {#if editing && thread}
      <ChatTitleEditor title={thread.title} onCommit={commitTitle} onCancel={cancelEdit} />
    {:else}
      <h1 class="truncate text-sm font-medium">{thread?.title || t("chat.header.newChat")}</h1>
      {#if thread}<button type="button" class="chat-title-edit" aria-label={t("chat.header.editTitle")} onclick={beginEdit}><Pencil size={12} /></button>{/if}
      {#if workspace}<span class="chat-header-breadcrumb" title={`${project?.name ?? t("chat.standalone")} / ${workspace.workspace.displayName}`}>{middleTruncate(project?.name ?? t("chat.standalone"), 22)} <span>/</span> {middleTruncate(workspace.workspace.displayName, 22)}</span>{/if}
    {/if}
  </div>
  {#if workspace?.bindingStatus === "available"}<button type="button" class="chat-header-action hidden @min-[560px]:inline-flex" onclick={() => run(() => chat.openWorkspaceFolder(workspace.workspace.id))}><FolderOpen size={14} /><span class="hidden @min-[760px]:inline">{t("chat.openFolder")}</span></button>{/if}
  {#if workspace?.currentBranch}<span class="chat-branch" title={workspace.currentBranch}>{workspace.currentBranch}</span>{/if}
  {#if provider}<span class="chat-provider-status"><span class="size-2 rounded-full" style={`background:${provider.configuration.accentColor ?? "var(--primary)"}`}></span><span class="truncate">{provider.configuration.label}</span><span class="hidden text-muted-foreground @min-[760px]:inline">{statusLabel()}</span></span>{/if}
  <button type="button" class="chat-icon-button" aria-label={chat.inspectorOpen ? t("chat.closeInspector") : t("chat.openInspector")} aria-pressed={chat.inspectorOpen} onclick={() => { chat.inspectorOpen = !chat.inspectorOpen; }}>{#if chat.inspectorOpen}<PanelRightClose size={16} />{:else}<PanelRight size={16} />{/if}</button>
  <details class="relative"><summary class="chat-icon-button list-none" aria-label={t("chat.moreActions")}><Ellipsis size={16} /></summary><div class="chat-menu right-0 top-8">{#if workspace?.bindingStatus === "available"}<button type="button" onclick={() => run(() => chat.openWorkspaceFolder(workspace.workspace.id))}><FolderOpen size={13} />{t("chat.openFolder")}</button>{/if}{#if thread}<button type="button" onclick={() => run(() => openDetachedViewWindow("chat"))}><SquareArrowOutUpRight size={13} />{t("chat.detach")}</button>{#if !thread.archivedAt}<button type="button" onclick={() => run(() => chat.archiveThread(thread))}><Archive size={13} />{t("chat.archive")}</button>{/if}{/if}</div></details>
</header>

<style>
  .chat-conversation-header { display: flex; min-height: 3.25rem; flex: 0 0 auto; align-items: center; gap: 0.5rem; border-bottom: 1px solid var(--border); padding-inline: 0.75rem; background: color-mix(in srgb, var(--cal-bg) 92%, transparent); }
  .rail-open-button { display: none; }
  .rail-open-button.visible { display: inline-flex; }
  .chat-title-edit { display: inline-flex; width: 1.65rem; height: 1.65rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.4rem; color: var(--muted-foreground); opacity: 0; }
  .chat-conversation-header:hover .chat-title-edit, .chat-title-edit:focus-visible { opacity: 1; }
  .chat-title-edit:hover { background: var(--accent); color: var(--foreground); }
  .chat-header-breadcrumb { display: none; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); font-size: 0.7rem; }
  .chat-header-breadcrumb::before { margin-right: 0.5rem; color: var(--border); content: "|"; }
  .chat-header-action { min-height: 2rem; align-items: center; gap: 0.4rem; border: 1px solid var(--border); border-radius: 0.55rem; padding: 0.3rem 0.55rem; color: var(--foreground); font-size: 0.733333rem; box-shadow: 0 1px 2px rgb(0 0 0 / 0.04); }
  .chat-header-action:hover { background: var(--accent); }
  .chat-branch { display: none; max-width: 8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; border: 1px solid var(--border); border-radius: 0.45rem; padding: 0.25rem 0.45rem; font-family: var(--font-mono); font-size: 0.666667rem; }
  .chat-provider-status { display: none; max-width: 12rem; align-items: center; gap: 0.4rem; font-size: 0.733333rem; }
  @container (min-width: 500px) { .chat-header-breadcrumb { display: inline; } .chat-provider-status { display: flex; } }
  @container (min-width: 680px) { .chat-branch { display: block; } }
  @media (hover: none) { .chat-title-edit { opacity: 1; } }
  :global(.chat-menu button) { display: flex; align-items: center; gap: 0.5rem; }
</style>
