<script lang="ts">
  import FileText from "@lucide/svelte/icons/file-text";
  import Files from "@lucide/svelte/icons/files";
  import Maximize2 from "@lucide/svelte/icons/maximize-2";
  import Minimize2 from "@lucide/svelte/icons/minimize-2";
  import Terminal from "@lucide/svelte/icons/terminal";
  import X from "@lucide/svelte/icons/x";
  import type { ChatInspectorTab } from "$lib/chat/contracts";
  import { chatInspectorSession, type ChatInspectorThreadState } from "$lib/chat/inspector-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatChangesPanel from "./ChatChangesPanel.svelte";
  import ChatFilesPanel from "./ChatFilesPanel.svelte";
  import ChatPlanPanel from "./ChatPlanPanel.svelte";
  import ChatTerminalPanel from "./ChatTerminalPanel.svelte";

  let {
    onClose,
    onMaximizedChange,
  }: {
    onClose: () => void;
    onMaximizedChange: (maximized: boolean) => void;
  } = $props();

  const { t } = getLocalization();
  const chat = getChat();
  let state = $state<ChatInspectorThreadState>(chatInspectorSession.read(null));
  let loadedThreadId: string | null = null;
  const tabs: { id: ChatInspectorTab; label: "changes" | "plan" | "files" | "terminal"; icon: typeof FileText }[] = [
    { id: "changes", label: "changes", icon: FileText },
    { id: "plan", label: "plan", icon: FileText },
    { id: "files", label: "files", icon: Files },
    { id: "terminal", label: "terminal", icon: Terminal },
  ];

  $effect(() => {
    const threadId = chat.selectedThreadId;
    if (threadId === loadedThreadId) return;
    loadedThreadId = threadId;
    state = chatInspectorSession.read(threadId);
    onMaximizedChange(state.maximized);
  });

  function update(updateValue: Partial<ChatInspectorThreadState>): void {
    if (!chat.selectedThreadId) return;
    state = chatInspectorSession.update(chat.selectedThreadId, updateValue);
    if (updateValue.maximized !== undefined) onMaximizedChange(updateValue.maximized);
  }

  function handleTabKeydown(event: KeyboardEvent, index: number): void {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const next = event.key === "Home"
      ? 0
      : event.key === "End"
        ? tabs.length - 1
        : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
    const tab = tabs[next];
    if (!tab) return;
    update({ tab: tab.id });
    const buttons = (event.currentTarget as HTMLElement).parentElement?.querySelectorAll<HTMLElement>("[role='tab']");
    queueMicrotask(() => buttons?.[next]?.focus());
  }
</script>

<div class="flex h-full min-h-0 flex-col" data-chat-inspector>
  <header class="flex min-h-11 items-center gap-1 border-b border-border px-2">
    <strong class="min-w-0 flex-1 truncate text-xs">{t("chat.inspector.title")}</strong>
    <button type="button" class="chat-icon-button" aria-label={state.maximized ? t("chat.inspector.restore") : t("chat.inspector.maximize")} onclick={() => update({ maximized: !state.maximized })}>
      {#if state.maximized}<Minimize2 size={14} />{:else}<Maximize2 size={14} />{/if}
    </button>
    <button type="button" class="chat-icon-button" aria-label={t("chat.closeInspector")} onclick={onClose}><X size={14} /></button>
  </header>
  <div class="grid grid-cols-4 border-b border-border" role="tablist" aria-label={t("chat.inspector.title")}>
    {#each tabs as tab, index (tab.id)}
      {@const Icon = tab.icon}
      <button
        type="button"
        role="tab"
        data-inspector-tab={tab.id}
        aria-selected={state.tab === tab.id}
        tabindex={state.tab === tab.id ? 0 : -1}
        class="inspector-tab"
        class:active={state.tab === tab.id}
        onclick={() => update({ tab: tab.id })}
        onkeydown={(event) => handleTabKeydown(event, index)}
      ><Icon size={13} /><span>{t(`chat.inspector.${tab.label}`)}</span></button>
    {/each}
  </div>
  <div class="min-h-0 flex-1" role="tabpanel" aria-label={t(`chat.inspector.${state.tab}`)}>
    {#if state.tab === "changes"}
      <ChatChangesPanel
        scope={state.changeScope}
        selectedFile={state.selectedFile}
        whitespaceIgnored={state.whitespaceIgnored}
        diffView={state.diffView}
        onStateChange={(change) => update({
          ...(change.scope === undefined ? {} : { changeScope: change.scope }),
          ...(change.selectedFile === undefined ? {} : { selectedFile: change.selectedFile }),
          ...(change.whitespaceIgnored === undefined ? {} : { whitespaceIgnored: change.whitespaceIgnored }),
          ...(change.diffView === undefined ? {} : { diffView: change.diffView }),
        })}
      />
    {:else if state.tab === "plan"}
      <ChatPlanPanel />
    {:else if state.tab === "files"}
      <ChatFilesPanel
        directoryPath={state.fileBrowserPath}
        selectedPath={state.filePreviewPath}
        onStateChange={(change) => update({
          ...(change.directoryPath === undefined ? {} : { fileBrowserPath: change.directoryPath }),
          ...(change.selectedPath === undefined ? {} : { filePreviewPath: change.selectedPath }),
        })}
      />
    {:else}
      <ChatTerminalPanel />
    {/if}
  </div>
</div>

<style>
  .inspector-tab { display: flex; min-width: 0; min-height: 2.4rem; align-items: center; justify-content: center; gap: 0.3rem; border-bottom: 2px solid transparent; padding: 0.35rem 0.2rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .inspector-tab:hover { background: var(--accent); color: var(--accent-foreground); }
  .inspector-tab.active { border-bottom-color: var(--primary); color: var(--foreground); }
  @container chat-shell (max-width: 359px) { .inspector-tab span { display: none; } }
</style>
