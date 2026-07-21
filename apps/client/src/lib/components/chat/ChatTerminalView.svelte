<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import Eraser from "@lucide/svelte/icons/eraser";
  import * as chatApi from "$lib/api/chat";
  import type { ChatTerminalRead } from "$lib/chat/contracts";
  import { parseChatTerminal, parseChatTerminalOutput } from "$lib/chat/validation";
  import { applyTerminalOutput, boundTerminalContext, terminalPasteNeedsConfirmation } from "$lib/chat/terminal-model";
  import { formatDateTime, formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import "@xterm/xterm/css/xterm.css";

  let {
    terminalRead,
    onState,
  }: {
    terminalRead: ChatTerminalRead;
    onState: (terminal: ChatTerminalRead) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let host: HTMLDivElement | undefined = $state();
  let xterm: import("@xterm/xterm").Terminal | null = null;
  let fitAddon: import("@xterm/addon-fit").FitAddon | null = null;
  let outputState = { generation: 0, lastSequence: 0 };
  let error = $state<string | null>(null);
  let lastInputSequence = 0;
  let commandOutput = "";
  let commandOutputTruncated = false;
  let outputDecoder = new TextDecoder();

  onMount(() => {
    let disposed = false;
    let resizeTimer: number | null = null;
    let resizeObserver: ResizeObserver | null = null;
    const disposers: (() => void)[] = [];
    void Promise.all([
      import("@xterm/xterm"),
      import("@xterm/addon-fit"),
      chatApi.readChatTerminalSnapshot(terminalRead.id, terminalRead.threadId, terminalRead.workspaceId),
    ]).then(async ([xtermModule, fitModule, snapshot]) => {
      if (disposed || !host) return;
      const terminal = new xtermModule.Terminal({
        allowProposedApi: false,
        convertEol: false,
        cursorBlink: true,
        scrollback: chat.settings?.configuration.behavior.terminalScrollbackLines ?? 10_000,
        fontFamily: "var(--font-mono, monospace)",
        fontSize: 12,
        theme: {
          background: "transparent",
          foreground: getComputedStyle(host).color,
        },
      });
      const fit = new fitModule.FitAddon();
      terminal.loadAddon(fit);
      terminal.open(host);
      xterm = terminal;
      fitAddon = fit;
      outputState = { generation: snapshot.terminal.generation, lastSequence: 0 };
      for (const chunk of snapshot.scrollback) applyChunk(chunk);
      terminal.onData((data) => {
        if (data.includes("\r") || data.includes("\n")) {
          lastInputSequence = outputState.lastSequence;
          commandOutput = "";
          commandOutputTruncated = false;
        }
        void chatApi.writeChatTerminal(
          terminalRead.id,
          terminalRead.threadId,
          terminalRead.workspaceId,
          data,
        ).catch((reason: unknown) => { error = message(reason); });
      });
      resizeObserver = new ResizeObserver(() => {
        if (resizeTimer !== null) window.clearTimeout(resizeTimer);
        resizeTimer = window.setTimeout(() => {
          fit.fit();
          if (terminal.cols < 20 || terminal.rows < 2) return;
          void chatApi.resizeChatTerminal(
            terminalRead.id,
            terminalRead.threadId,
            terminalRead.workspaceId,
            terminal.cols,
            terminal.rows,
          ).catch(() => undefined);
        }, 80);
      });
      resizeObserver.observe(host);
      fit.fit();
      terminal.focus();

      const unlistenOutput = await listen<unknown>("chat://terminal-output", (event) => {
        try {
          const chunk = parseChatTerminalOutput(event.payload);
          if (chunk.terminalId === terminalRead.id) applyChunk(chunk);
        } catch (reason: unknown) {
          error = message(reason);
        }
      });
      const unlistenState = await listen<unknown>("chat://terminal-state", (event) => {
        try {
          const state = parseChatTerminal(event.payload);
          if (state.id === terminalRead.id) onState(state);
        } catch (reason: unknown) {
          error = message(reason);
        }
      });
      disposers.push(unlistenOutput, unlistenState);
    }).catch((reason: unknown) => { error = message(reason); });
    return () => {
      disposed = true;
      if (resizeTimer !== null) window.clearTimeout(resizeTimer);
      resizeObserver?.disconnect();
      for (const dispose of disposers) dispose();
      xterm?.dispose();
      xterm = null;
      fitAddon = null;
    };
  });

  function applyChunk(chunk: import("$lib/chat/contracts").ChatTerminalOutputChunk): void {
    if (!xterm) return;
    const decision = applyTerminalOutput(outputState, chunk);
    if (decision.kind === "accept") {
      if (decision.state.generation !== outputState.generation) {
        commandOutput = "";
        commandOutputTruncated = false;
        outputDecoder = new TextDecoder();
      }
      outputState = decision.state;
      xterm.write(decision.bytes);
      if (!commandOutputTruncated) {
        const bounded = boundTerminalContext(
          commandOutput + outputDecoder.decode(decision.bytes, { stream: true }),
          128 * 1024,
        );
        commandOutput = bounded.text;
        commandOutputTruncated = bounded.truncated;
      }
    } else if (decision.kind === "gap") {
      void replay();
    }
  }

  async function replay(): Promise<void> {
    const snapshot = await chatApi.readChatTerminalSnapshot(
      terminalRead.id,
      terminalRead.threadId,
      terminalRead.workspaceId,
    );
    xterm?.reset();
    outputState = { generation: snapshot.terminal.generation, lastSequence: 0 };
    for (const chunk of snapshot.scrollback) applyChunk(chunk);
    onState(snapshot.terminal);
  }

  function handlePaste(event: ClipboardEvent): void {
    const text = event.clipboardData?.getData("text/plain") ?? "";
    if (!text || !terminalPasteNeedsConfirmation(
      text,
      chat.settings?.configuration.behavior.confirmMultilineTerminalPaste ?? true,
    )) return;
    event.preventDefault();
    if (window.confirm(t("chat.inspector.confirmMultilinePaste") + "\n\n" + text)) {
      xterm?.paste(text);
    }
  }

  async function attachContext(sourceKind: "selection" | "last_command_output"): Promise<void> {
    if (!xterm) return;
    const raw = sourceKind === "selection" ? xterm.getSelection() : recentOutput();
    const bounded = boundTerminalContext(raw, 128 * 1024);
    if (!bounded.text) return;
    const capturedAt = formatDateTime(localization.locale, Date.now(), {
      dateStyle: "medium",
      timeStyle: "medium",
    });
    const summary = t(
      "chat.inspector.contextSummary",
      terminalRead.name,
      formatNumber(localization.locale, bounded.lineCount),
      formatNumber(localization.locale, bounded.byteSize),
      capturedAt,
    );
    if (!window.confirm(`${t("chat.inspector.contextPreview")}\n${summary}\n\n${bounded.text}`)) return;
    const context = await chatApi.importChatTerminalContext({
      terminalId: terminalRead.id,
      threadId: terminalRead.threadId,
      workspaceId: terminalRead.workspaceId,
      attachmentId: crypto.randomUUID(),
      sourceKind,
      text: bounded.text,
      startOutputSequence: sourceKind === "last_command_output" ? lastInputSequence : null,
      endOutputSequence: outputState.lastSequence,
      truncated: bounded.truncated || (sourceKind === "last_command_output" && commandOutputTruncated),
    });
    chat.setComposerAttachments([...chat.composer.attachmentIds, context.attachmentId]);
  }

  function recentOutput(): string {
    return commandOutput.trimEnd();
  }

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<div class="flex h-full min-h-0 flex-col" data-terminal-capture>
  {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
  <div class="min-h-0 flex-1 p-1 text-foreground" bind:this={host} onpaste={handlePaste}></div>
  <div class="flex flex-wrap gap-1 border-t border-border p-1">
    <button type="button" class="chat-secondary-button" onclick={() => xterm?.clear()}><Eraser size={12} />{t("chat.inspector.clearTerminal")}</button>
    <button type="button" class="chat-secondary-button" onclick={() => { void attachContext("selection").catch((reason) => { error = message(reason); }); }}><Paperclip size={12} />{t("chat.inspector.attachSelection")}</button>
    <button type="button" class="chat-secondary-button" onclick={() => { void attachContext("last_command_output").catch((reason) => { error = message(reason); }); }}><Paperclip size={12} />{t("chat.inspector.attachLastOutput")}</button>
  </div>
</div>

<style>
  :global(.xterm) { height: 100%; padding: 0.25rem; }
  :global(.xterm-viewport) { background: transparent !important; }
</style>
