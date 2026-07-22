<script module lang="ts">
  import { ChatInspectorSessionState } from "$lib/chat/inspector-model";

  const bottomPanelSession = new ChatInspectorSessionState();
  const initializedBottomKeys = new Set<string>();
</script>

<script lang="ts">
  import FileDiff from "@lucide/svelte/icons/file-diff";
  import Files from "@lucide/svelte/icons/files";
  import ListTodo from "@lucide/svelte/icons/list-todo";
  import Terminal from "@lucide/svelte/icons/terminal";
  import X from "@lucide/svelte/icons/x";
  import type { ChatInspectorTab } from "$lib/chat/contracts";
  import {
    inspectorSessionKey,
    type ChatInspectorThreadState,
  } from "$lib/chat/inspector-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatChangesPanel from "./ChatChangesPanel.svelte";
  import ChatFilesPanel from "./ChatFilesPanel.svelte";
  import ChatPlanPanel from "./ChatPlanPanel.svelte";
  import ChatTerminalPanel from "./ChatTerminalPanel.svelte";

  let { onClose }: { onClose: () => void } = $props();

  const { t } = getLocalization();
  const chat = getChat();
  let state = $state<ChatInspectorThreadState>({ ...bottomPanelSession.read(null), tab: "terminal" });
  let loadedKey: string | null = null;
  const baseSessionKey = $derived(inspectorSessionKey(chat.selectedThreadId, chat.selectedWorkspaceId));
  const sessionKey = $derived(baseSessionKey ? `bottom:${baseSessionKey}` : null);
  const tabs: { id: ChatInspectorTab; label: "changes" | "plan" | "files" | "terminal"; icon: typeof FileDiff }[] = [
    { id: "terminal", label: "terminal", icon: Terminal },
    { id: "changes", label: "changes", icon: FileDiff },
    { id: "plan", label: "plan", icon: ListTodo },
    { id: "files", label: "files", icon: Files },
  ];

  $effect(() => {
    const key = sessionKey;
    if (key === loadedKey) return;
    loadedKey = key;
    const stored = bottomPanelSession.read(key);
    if (key && !initializedBottomKeys.has(key)) {
      initializedBottomKeys.add(key);
      state = bottomPanelSession.update(key, { tab: "terminal" });
    } else {
      state = stored;
    }
  });

  function update(updateValue: Partial<ChatInspectorThreadState>): void {
    const key = sessionKey;
    if (!key) return;
    state = bottomPanelSession.update(key, updateValue);
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

<section class="bottom-panel" aria-label={t("chat.bottomPanel")}>
  <div class="bottom-tabbar" role="tablist" aria-label={t("chat.bottomPanel")}>
    {#each tabs as tab, index (tab.id)}
      {@const Icon = tab.icon}
      <button
        type="button"
        role="tab"
        aria-selected={state.tab === tab.id}
        tabindex={state.tab === tab.id ? 0 : -1}
        class="bottom-tab"
        class:active={state.tab === tab.id}
        onclick={() => update({ tab: tab.id })}
        onkeydown={(event) => handleTabKeydown(event, index)}
      ><Icon size={13} /><span>{t(`chat.inspector.${tab.label}`)}</span></button>
    {/each}
    <span class="flex-1"></span>
    <button type="button" class="chat-icon-button" aria-label={t("chat.closeBottomPanel")} onclick={onClose}><X size={14} /></button>
  </div>
  <div class="min-h-0 flex-1" role="tabpanel" aria-label={t(`chat.inspector.${state.tab}`)}>
    {#if state.tab === "terminal"}
      <ChatTerminalPanel />
    {:else if state.tab === "changes"}
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
    {:else}
      <ChatFilesPanel
        directoryPath={state.fileBrowserPath}
        selectedPath={state.filePreviewPath}
        treeVisible={state.fileTreeVisible}
        onStateChange={(change) => update({
          ...(change.directoryPath === undefined ? {} : { fileBrowserPath: change.directoryPath }),
          ...(change.selectedPath === undefined ? {} : { filePreviewPath: change.selectedPath }),
          ...(change.treeVisible === undefined ? {} : { fileTreeVisible: change.treeVisible }),
        })}
      />
    {/if}
  </div>
</section>

<style>
  .bottom-panel { display: flex; height: 100%; min-height: 0; flex-direction: column; background: var(--cal-bg); }
  .bottom-tabbar { display: flex; min-height: 2.3rem; flex: 0 0 auto; align-items: stretch; gap: 0.1rem; border-bottom: 1px solid var(--border); padding-inline: 0.35rem; }
  .bottom-tabbar > :global(.chat-icon-button) { align-self: center; }
  .bottom-tab { display: flex; min-width: 0; align-items: center; gap: 0.35rem; border-bottom: 2px solid transparent; padding: 0.3rem 0.55rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .bottom-tab:hover { background: var(--accent); color: var(--foreground); }
  .bottom-tab.active { border-bottom-color: var(--primary); color: var(--foreground); }
  @container chat-shell (max-width: 420px) { .bottom-tab span { display: none; } }
</style>
