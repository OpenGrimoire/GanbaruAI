<script lang="ts">
  import Edit3 from "@lucide/svelte/icons/square-pen";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import * as chatApi from "$lib/api/chat";
  import type { ChatTerminalRead } from "$lib/chat/contracts";
  import { selectedTerminalByThread } from "$lib/chat/terminal-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatTerminalView from "./ChatTerminalView.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  let terminals = $state<ChatTerminalRead[]>([]);
  let selectedId = $state<string | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let scopeKey = "";
  const threadId = $derived(chat.selectedThreadId);
  const workspaceId = $derived(chat.selectedWorkspaceId);
  const selected = $derived(terminals.find((terminal) => terminal.id === selectedId) ?? null);

  $effect(() => {
    const thread = threadId;
    const workspace = workspaceId;
    const nextKey = `${thread ?? ""}:${workspace ?? ""}`;
    if (!thread || !workspace || scopeKey === nextKey) return;
    scopeKey = nextKey;
    void load(thread, workspace);
  });

  async function load(thread: string, workspace: string): Promise<void> {
    loading = true;
    error = null;
    try {
      terminals = await chatApi.listChatTerminals(thread, workspace);
      const remembered = selectedTerminalByThread.get(thread);
      select(terminals.some((terminal) => terminal.id === remembered) ? remembered ?? null : terminals[0]?.id ?? null);
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      loading = false;
    }
  }

  async function create(): Promise<void> {
    if (!threadId || !workspaceId) return;
    const number = terminals.length + 1;
    const snapshot = await chatApi.createChatTerminal({
      terminalId: crypto.randomUUID(),
      threadId,
      workspaceId,
      name: `${t("chat.inspector.terminal")} ${number}`,
      columns: 80,
      rows: 24,
    });
    terminals = [...terminals, snapshot.terminal];
    select(snapshot.terminal.id);
  }

  function select(terminalId: string | null): void {
    selectedId = terminalId;
    if (threadId && terminalId) selectedTerminalByThread.set(threadId, terminalId);
  }

  function updateTerminal(terminal: ChatTerminalRead): void {
    terminals = terminals.map((entry) => entry.id === terminal.id ? terminal : entry);
  }

  async function rename(): Promise<void> {
    if (!selected || !threadId || !workspaceId) return;
    const name = window.prompt(t("chat.inspector.terminalName"), selected.name)?.trim();
    if (!name || name === selected.name) return;
    updateTerminal(await chatApi.renameChatTerminal(selected.id, threadId, workspaceId, name));
  }

  async function restart(): Promise<void> {
    if (!selected || !threadId || !workspaceId) return;
    updateTerminal((await chatApi.restartChatTerminal(selected.id, threadId, workspaceId)).terminal);
  }

  async function close(): Promise<void> {
    if (!selected || !threadId || !workspaceId) return;
    let result = await chatApi.closeChatTerminal(selected.id, threadId, workspaceId, false);
    if (result.confirmationRequired) {
      if (!window.confirm(t("chat.inspector.confirmCloseTerminal"))) return;
      result = await chatApi.closeChatTerminal(selected.id, threadId, workspaceId, true);
    }
    if (!result.closed) return;
    const next = terminals.filter((terminal) => terminal.id !== selected.id);
    terminals = next;
    select(next[0]?.id ?? null);
  }

  async function run(operation: () => Promise<void>): Promise<void> {
    error = null;
    try {
      await operation();
    } catch (reason: unknown) {
      error = message(reason);
    }
  }

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="flex min-h-10 items-center gap-1 overflow-x-auto border-b border-border p-1">
    {#each terminals as terminal (terminal.id)}
      <button type="button" class="terminal-tab" class:active={selectedId === terminal.id} onclick={() => select(terminal.id)} title={terminal.name}>
        <span class:running={terminal.running} class="status-dot"></span>
        <span class="max-w-28 truncate">{terminal.name}</span>
      </button>
    {/each}
    <button type="button" class="chat-icon-button shrink-0" aria-label={t("chat.inspector.newTerminal")} onclick={() => void run(create)}><Plus size={13} /></button>
    {#if selected}
      <span class="ml-auto flex shrink-0 gap-1">
        <button type="button" class="chat-icon-button" aria-label={t("chat.inspector.renameTerminal")} onclick={() => void run(rename)}><Edit3 size={13} /></button>
        <button type="button" class="chat-icon-button" aria-label={t("chat.inspector.restartTerminal")} onclick={() => void run(restart)}><RefreshCw size={13} /></button>
        <button type="button" class="chat-icon-button" aria-label={t("chat.inspector.closeTerminal")} onclick={() => void run(close)}><Trash2 size={13} /></button>
      </span>
    {/if}
  </div>
  {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
  {#if loading}
    <p class="m-auto text-xs text-muted-foreground">{t("common.loading")}</p>
  {:else if selected}
    <div class="flex min-h-0 flex-1 flex-col">
      {#if !selected.running}<p class="border-b border-border px-2 py-1 text-[0.666667rem] text-muted-foreground">{selected.exitCode === null ? t("chat.inspector.terminalStopped") : t("chat.inspector.terminalExited", selected.exitCode)}</p>{/if}
      {#key `${selected.id}:${selected.generation}`}
        <ChatTerminalView terminalRead={selected} onState={updateTerminal} />
      {/key}
    </div>
  {:else}
    <div class="m-auto p-4 text-center text-xs text-muted-foreground">
      <p>{t("chat.inspector.terminalEmpty")}</p>
      <button type="button" class="chat-secondary-button mt-3" onclick={() => void run(create)}><Plus size={13} />{t("chat.inspector.newTerminal")}</button>
    </div>
  {/if}
</div>

<style>
  .terminal-tab { display: flex; min-height: 1.75rem; align-items: center; gap: 0.3rem; border-radius: 0.25rem; padding: 0.2rem 0.4rem; font-size: 0.666667rem; }
  .terminal-tab:hover, .terminal-tab.active { background: var(--accent); }
  .status-dot { width: 0.4rem; height: 0.4rem; flex: none; border-radius: 9999px; background: var(--muted-foreground); }
  .status-dot.running { background: var(--success, #16a34a); }
</style>
