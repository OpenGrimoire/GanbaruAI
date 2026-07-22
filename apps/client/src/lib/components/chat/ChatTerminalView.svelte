<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import * as chatApi from "$lib/api/chat";
  import type { ChatTerminalRead } from "$lib/chat/contracts";
  import { parseChatTerminal, parseChatTerminalOutput } from "$lib/chat/validation";
  import { applyTerminalOutput, terminalPasteNeedsConfirmation } from "$lib/chat/terminal-model";
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

  const { t } = getLocalization();
  const chat = getChat();
  let host: HTMLDivElement | undefined = $state();
  let xterm: import("@xterm/xterm").Terminal | null = null;
  let fitAddon: import("@xterm/addon-fit").FitAddon | null = null;
  let outputState = { generation: 0, lastSequence: 0 };
  let error = $state<string | null>(null);
  const TERMINAL_FONT_FAMILY = '"SF Mono", "SFMono-Regular", "JetBrains Mono", "Cascadia Code", Consolas, "Liberation Mono", Menlo, monospace';
  const TERMINAL_FONT_SIZE = 12;

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
      const hostStyles = getComputedStyle(host);
      const terminalBackground = hostStyles.getPropertyValue("--cal-bg").trim() || hostStyles.backgroundColor;
      const terminalForeground = hostStyles.color;
      const terminal = new xtermModule.Terminal({
        allowProposedApi: false,
        convertEol: false,
        cursorBlink: true,
        scrollback: chat.settings?.configuration.behavior.terminalScrollbackLines ?? 10_000,
        fontFamily: TERMINAL_FONT_FAMILY,
        fontSize: TERMINAL_FONT_SIZE,
        fontWeight: "400",
        fontWeightBold: "600",
        letterSpacing: 0,
        lineHeight: 1.1,
        theme: {
          background: terminalBackground,
          foreground: terminalForeground,
          cursor: terminalForeground,
          selectionBackground: hostStyles.getPropertyValue("--selection-background").trim(),
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
      outputState = decision.state;
      xterm.write(decision.bytes);
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

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<div class="terminal-view" data-terminal-capture>
  {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
  <div class="terminal-host" bind:this={host} onpaste={handlePaste}></div>
</div>

<style>
  .terminal-view { container-type: inline-size; display: flex; height: 100%; min-height: 0; flex-direction: column; background: var(--cal-bg); }
  .terminal-host { min-height: 0; flex: 1; overflow: hidden; color: var(--foreground); background: var(--cal-bg); }
  :global(.xterm) { height: 100%; padding: 0.45rem 0.55rem; }
  :global(.xterm-viewport), :global(.xterm-screen) { background: var(--cal-bg) !important; }
  :global(.xterm .xterm-scrollable-element > .scrollbar.vertical) { width: 6px !important; }
  :global(.xterm .xterm-scrollable-element > .scrollbar > .slider) { border-radius: 3px; }
</style>
