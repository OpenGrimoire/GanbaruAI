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

<header class="flex min-h-11 shrink-0 items-center gap-2 border-b border-border bg-background/50 px-2 @container">
  <button type="button" class="chat-icon-button rail-open-button" class:visible={showRailButton} aria-label={t("chat.openRail")} onclick={onOpenRail}><Menu size={16} /></button>
  <div class="min-w-0 flex-1">
    {#if actionError}<p role="alert" class="truncate text-[0.666667rem] text-destructive">{actionError}</p>{/if}
    {#if editing && thread}
      <ChatTitleEditor title={thread.title} onCommit={commitTitle} onCancel={cancelEdit} />
    {:else}
      <div class="flex min-w-0 items-center gap-1"><h1 class="truncate text-sm font-semibold">{thread?.title || t("chat.header.newChat")}</h1>{#if thread}<button type="button" class="chat-icon-button size-6" aria-label={t("chat.header.editTitle")} onclick={beginEdit}><Pencil size={12} /></button>{/if}</div>
      <div class="hidden min-w-0 items-center gap-1 text-[0.666667rem] text-muted-foreground @min-[420px]:flex"><span class="truncate" title={project?.name ?? t("chat.standalone")}>{middleTruncate(project?.name ?? t("chat.standalone"), 32)}</span>{#if workspace}<span>/</span><span class="truncate" title={workspace.workspace.displayName}>{middleTruncate(workspace.workspace.displayName, 32)}</span>{/if}</div>
    {/if}
  </div>
  {#if provider}<span class="hidden max-w-32 items-center gap-1.5 truncate text-xs @min-[560px]:flex"><span class="size-2 rounded-full" style={`background:${provider.configuration.accentColor ?? "var(--primary)"}`}></span>{provider.configuration.label}</span>{/if}
  {#if workspace?.currentBranch}<span class="hidden max-w-32 truncate rounded bg-muted px-2 py-1 font-mono text-[0.666667rem] @min-[680px]:block" title={workspace.currentBranch}>{workspace.currentBranch}</span>{/if}
  <span class="max-w-32 truncate text-[0.666667rem] text-muted-foreground">{statusLabel()}</span>
  <button type="button" class="chat-icon-button" aria-label={chat.inspectorOpen ? t("chat.closeInspector") : t("chat.openInspector")} aria-pressed={chat.inspectorOpen} onclick={() => { chat.inspectorOpen = !chat.inspectorOpen; }}>{#if chat.inspectorOpen}<PanelRightClose size={16} />{:else}<PanelRight size={16} />{/if}</button>
  <details class="relative"><summary class="chat-icon-button list-none" aria-label={t("chat.moreActions")}><Ellipsis size={16} /></summary><div class="chat-menu right-0 top-8">{#if workspace?.bindingStatus === "available"}<button type="button" onclick={() => run(() => chat.openWorkspaceFolder(workspace.workspace.id))}><FolderOpen size={13} />{t("chat.openFolder")}</button>{/if}{#if thread}<button type="button" onclick={() => run(() => openDetachedViewWindow("chat"))}><SquareArrowOutUpRight size={13} />{t("chat.detach")}</button>{#if !thread.archivedAt}<button type="button" onclick={() => run(() => chat.archiveThread(thread))}><Archive size={13} />{t("chat.archive")}</button>{/if}{/if}</div></details>
</header>

<style>
  .rail-open-button { display: none; }
  .rail-open-button.visible { display: inline-flex; }
  :global(.chat-menu button) { display: flex; align-items: center; gap: 0.5rem; }
</style>
