<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Command from "@lucide/svelte/icons/command";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Search from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import { nextThreadIndex } from "$lib/chat/shell-model";
  import { inspectorFocusAction } from "$lib/chat/inspector-model";
  import {
    chatLayoutDecision,
    panelWidthFromKey,
    type ChatLayoutDecision,
  } from "$lib/chat/responsive-layout";
  import { parseChatChangeNotification } from "$lib/chat/validation";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import * as chatApi from "$lib/api/chat";
  import ChatConversationHeader from "./ChatConversationHeader.svelte";
  import ChatComposer from "./ChatComposer.svelte";
  import ChatFirstUse from "./ChatFirstUse.svelte";
  import ChatInspector from "./ChatInspector.svelte";
  import ChatThreadRail from "./ChatThreadRail.svelte";
  import ChatTimeline from "./ChatTimeline.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  let rootElement: HTMLDivElement | undefined = $state();
  let railShell: HTMLDivElement | undefined = $state();
  let inspectorShell: HTMLElement | undefined = $state();
  let commandDialog: HTMLDivElement | undefined = $state();
  let commandMenuOpen = $state(false);
  let resizingRail = $state(false);
  let resizingInspector = $state(false);
  let railWidth = $state(260);
  let inspectorWidth = $state(360);
  let inspectorMaximized = $state(false);
  let inspectorWasOpen = false;
  let inspectorReturnFocus: HTMLElement | null = null;
  let railModalWasOpen = false;
  let railReturnFocus: HTMLElement | null = null;
  let commandMenuWasOpen = false;
  let commandReturnFocus: HTMLElement | null = null;
  let shellWidth = $state(1_200);
  let shellHeight = $state(700);
  let fontScale = $state(1);
  let layout = $state<ChatLayoutDecision>(chatLayoutDecision({
    containerWidth: shellWidth,
    containerHeight: shellHeight,
    fontScale,
    railOpen: true,
    inspectorOpen: false,
    railWidth,
    inspectorWidth,
  }));
  let loadError = $state<string | null>(null);
  let layoutError = $state<string | null>(null);
  let politeAnnouncement = $state("");
  let assertiveAnnouncement = $state("");
  let announcedThreadId: string | null = null;
  let announcedTurnState: string | null = null;
  let announcedRequestId: string | null = null;

  onMount(() => {
    void Promise.all([chat.ensureLoaded(), projects.ensureLoaded()]).catch((error) => {
      loadError = error instanceof Error ? error.message : String(error);
    });
    railWidth = chat.settings?.configuration.panels.railWidthPx ?? 260;
    inspectorWidth = chat.settings?.configuration.panels.inspectorWidthPx ?? 360;
    const unlisten = listen<unknown>("chat://change", (event) => {
      try {
        const change = parseChatChangeNotification(event.payload);
        void chat.handleNativeChange(change.threadId).catch(() => undefined);
      } catch (error: unknown) {
        console.error("Invalid Chat change notification", error);
      }
    });
    const observer = new ResizeObserver(([entry]) => {
      if (!entry) return;
      shellWidth = entry.contentRect.width;
      shellHeight = entry.contentRect.height;
      const rootSize = rootElement ? Number.parseFloat(getComputedStyle(rootElement).fontSize) : 15;
      fontScale = Number.isFinite(rootSize) ? Math.max(1, rootSize / 15) : 1;
    });
    if (rootElement) observer.observe(rootElement);
    const revertMessage = (event: Event) => {
      if (!(event instanceof CustomEvent) || !isRevertMessageDetail(event.detail)) return;
      void restoreMessageCheckpoint(event.detail.threadId, event.detail.checkpointId);
    };
    window.addEventListener("ganbaru-ai:chat-revert-message", revertMessage);
    return () => {
      observer.disconnect();
      window.removeEventListener("ganbaru-ai:chat-revert-message", revertMessage);
      void unlisten.then((dispose) => dispose());
    };
  });

  $effect(() => {
    if (!resizingRail && chat.settings) railWidth = chat.settings.configuration.panels.railWidthPx;
    if (!resizingInspector && chat.settings) inspectorWidth = chat.settings.configuration.panels.inspectorWidthPx;
  });

  $effect(() => {
    const thread = chat.selectedThread;
    if (thread?.id !== announcedThreadId) {
      announcedThreadId = thread?.id ?? null;
      announcedTurnState = thread?.latestTurnState ?? null;
      politeAnnouncement = "";
      return;
    }
    const state = thread?.latestTurnState ?? null;
    if (state === announcedTurnState) return;
    announcedTurnState = state;
    if (state === "active" || state === "dispatching") politeAnnouncement = t("chat.accessibility.providerWorking");
    else if (state === "completed") politeAnnouncement = t("chat.accessibility.responseCompleted");
    else if (state === "interrupted") politeAnnouncement = t("chat.accessibility.turnInterrupted");
    else if (state === "failed") politeAnnouncement = t("chat.accessibility.turnFailed");
  });

  $effect(() => {
    const pending = chat.interaction?.pendingRequest ?? null;
    if (!pending) {
      announcedRequestId = null;
      assertiveAnnouncement = "";
      return;
    }
    if (pending.id === announcedRequestId) return;
    announcedRequestId = pending.id;
    assertiveAnnouncement = pending.requestKind === "approval"
      ? t("chat.accessibility.approvalRequired")
      : t("chat.accessibility.answerRequired");
  });

  $effect(() => {
    const next = chatLayoutDecision({
      containerWidth: shellWidth,
      containerHeight: shellHeight,
      fontScale,
      railOpen: chat.railOpen,
      inspectorOpen: chat.inspectorOpen,
      railWidth,
      inspectorWidth,
      previousVariant: layout.variant,
    });
    if (!sameLayout(layout, next)) layout = next;
  });

  $effect(() => {
    const open = chat.inspectorOpen;
    const action = inspectorFocusAction(inspectorWasOpen, open);
    if (action === "enter") {
      inspectorReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      queueMicrotask(() => document.querySelector<HTMLElement>("[data-chat-inspector] [role='tab'][aria-selected='true']")?.focus());
    } else if (action === "restore") {
      const target = inspectorReturnFocus;
      queueMicrotask(() => target?.isConnected && target.focus());
      inspectorMaximized = false;
    }
    inspectorWasOpen = open;
  });

  $effect(() => {
    const open = layout.railPresentation === "sheet" && chat.railOpen;
    if (open && !railModalWasOpen) {
      railReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      queueMicrotask(() => firstFocusable(railShell)?.focus());
    } else if (!open && railModalWasOpen) {
      const target = railReturnFocus;
      queueMicrotask(() => target?.isConnected && target.focus());
    }
    railModalWasOpen = open;
  });

  $effect(() => {
    if (commandMenuOpen && !commandMenuWasOpen) {
      commandReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      queueMicrotask(() => firstFocusable(commandDialog)?.focus());
    } else if (!commandMenuOpen && commandMenuWasOpen) {
      const target = commandReturnFocus;
      queueMicrotask(() => target?.isConnected && target.focus());
    }
    commandMenuWasOpen = commandMenuOpen;
  });

  function isEditingTarget(target: EventTarget | null): boolean {
    return target instanceof HTMLElement && (
      target.matches("input, textarea, select, [contenteditable='true']")
      || Boolean(target.closest("[role='dialog'], [data-terminal-capture]"))
    );
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (commandMenuOpen && event.key === "Escape") {
      event.preventDefault();
      commandMenuOpen = false;
      return;
    }
    if (isEditingTarget(event.target)) return;
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "n") {
      event.preventDefault();
      if (chat.selectedWorkspaceId) chat.newDraft(chat.selectedWorkspaceId);
      else chat.railOpen = true;
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "f") {
      event.preventDefault();
      chat.railOpen = true;
      window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search"));
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "l") {
      event.preventDefault();
      document.querySelector<HTMLElement>("[data-chat-composer]")?.focus();
      return;
    }
    if (event.altKey && !event.ctrlKey && !event.metaKey && ["ArrowDown", "ArrowUp"].includes(event.key)) {
      event.preventDefault();
      const index = chat.activeThreads.findIndex((thread) => thread.id === chat.selectedThreadId);
      const next = nextThreadIndex(index, chat.activeThreads.length, event.key === "ArrowDown" ? "next" : "previous");
      if (next >= 0) chat.selectThread(chat.activeThreads[next].id);
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key.toLowerCase() === "b") {
      event.preventDefault();
      chat.railOpen = !chat.railOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event, { shift: true }) && event.key.toLowerCase() === "j") {
      event.preventDefault();
      chat.inspectorOpen = !chat.inspectorOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event, { shift: true }) && event.key.toLowerCase() === "p") {
      event.preventDefault();
      commandMenuOpen = !commandMenuOpen;
      return;
    }
    if (hasOnlyShortcutModifier(event) && event.key === ".") {
      event.preventDefault();
      window.dispatchEvent(new Event("ganbaru-ai:chat-stop-requested"));
    }
  }

  function handleSheetKeydown(event: KeyboardEvent, close: () => void): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    trapFocus(event);
  }

  function trapFocus(event: KeyboardEvent): void {
    if (event.key !== "Tab") return;
    const container = event.currentTarget;
    if (!(container instanceof HTMLElement)) return;
    const focusable = focusableElements(container);
    if (focusable.length === 0) {
      event.preventDefault();
      container.focus();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function focusableElements(container: HTMLElement): HTMLElement[] {
    return [...container.querySelectorAll<HTMLElement>("button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex='-1'])")]
      .filter((element) => !element.hidden && element.getClientRects().length > 0);
  }

  function firstFocusable(container: HTMLElement | undefined): HTMLElement | undefined {
    return container ? focusableElements(container)[0] : undefined;
  }

  function sameLayout(left: ChatLayoutDecision, right: ChatLayoutDecision): boolean {
    return left.variant === right.variant
      && left.railPresentation === right.railPresentation
      && left.inspectorPresentation === right.inspectorPresentation
      && left.activeSurface === right.activeSurface;
  }

  function beginRailResize(event: PointerEvent): void {
    resizingRail = true;
    const startX = event.clientX;
    const startWidth = railWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => { railWidth = Math.max(160, Math.min(520, startWidth + moveEvent.clientX - startX)); };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      resizingRail = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
  }

  function beginInspectorResize(event: PointerEvent): void {
    resizingInspector = true;
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => { inspectorWidth = Math.max(240, Math.min(960, startWidth + startX - moveEvent.clientX)); };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      resizingInspector = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
  }

  function resizePanelFromKey(
    event: KeyboardEvent,
    panel: "rail" | "inspector",
  ): void {
    const next = panelWidthFromKey(panel === "rail"
      ? {
          current: railWidth,
          minimum: 160,
          maximum: 520,
          defaultValue: 260,
          step: 16,
          direction: "standard",
          key: event.key,
        }
      : {
          current: inspectorWidth,
          minimum: 240,
          maximum: 960,
          defaultValue: 360,
          step: 16,
          direction: "reversed",
          key: event.key,
        });
    if (next === null) return;
    event.preventDefault();
    if (panel === "rail") railWidth = next;
    else inspectorWidth = next;
    void persistPanelWidths();
  }

  async function persistPanelWidths(): Promise<void> {
    if (!chat.settings) return;
    layoutError = null;
    try {
      const { updateChatPanels } = await import("$lib/api/chat");
      await updateChatPanels({ railWidthPx: Math.round(railWidth), inspectorWidthPx: Math.round(inspectorWidth) });
      await chat.refreshSettings();
    } catch (error: unknown) {
      layoutError = error instanceof Error ? error.message : String(error);
    }
  }

  function isRevertMessageDetail(value: unknown): value is { threadId: string; checkpointId: string } {
    if (typeof value !== "object" || value === null) return false;
    const record = value as Record<string, unknown>;
    return typeof record.threadId === "string" && typeof record.checkpointId === "string";
  }

  async function openSettingsFromCommandMenu(): Promise<void> {
    commandMenuOpen = false;
    await tick();
    if (commandReturnFocus?.isConnected) commandReturnFocus.focus();
    settings.open("chat");
  }

  async function restoreMessageCheckpoint(threadId: string, checkpointId: string): Promise<void> {
    const thread = chat.selectedThread;
    if (!thread || thread.id !== threadId) return;
    layoutError = null;
    try {
      const preview = await chatApi.previewChatCheckpointRestore(threadId, checkpointId);
      const affected = preview.files.map((file) => file.relativePath).join("\n");
      const confirmed = window.confirm(
        [t("chat.timeline.revert"), affected, ...preview.warnings].filter(Boolean).join("\n\n"),
      );
      if (!confirmed) return;
      await chatApi.executeChatCheckpointRestore({
        command: {
          clientCommandId: crypto.randomUUID(),
          expectedThreadRevision: thread.revision,
        },
        threadId,
        previewId: preview.previewId,
        confirmed: true,
      });
      await chat.handleNativeChange(threadId);
    } catch (error: unknown) {
      layoutError = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div bind:this={rootElement} class="chat-workspace @container/chat-shell relative flex h-full min-h-0 overflow-hidden" data-layout={layout.variant} data-rail-presentation={layout.railPresentation} data-inspector-presentation={layout.inspectorPresentation} data-active-surface={layout.activeSurface} style="background-color:var(--cal-bg);container-type:inline-size;container-name:chat-shell;">
  <div class="sr-only" aria-live="polite" aria-atomic="true">{politeAnnouncement}</div>
  <div class="sr-only" aria-live="assertive" aria-atomic="true">{assertiveAnnouncement}</div>
  {#if layoutError}<div role="alert" class="absolute inset-x-2 top-2 z-50 rounded border border-destructive/40 bg-background p-2 text-xs text-destructive">{layoutError}</div>{/if}
  {#if layout.inspectorPresentation === "sheet" && chat.inspectorOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.closeInspector")} onclick={() => { chat.inspectorOpen = false; }}></button>
  {:else if layout.railPresentation === "sheet" && chat.railOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.collapseRail")} onclick={() => { chat.railOpen = false; }}></button>
  {/if}
  <div bind:this={railShell} class="chat-rail-shell" class:closed={!chat.railOpen} class:maximized-hidden={inspectorMaximized} role={layout.railPresentation === "sheet" && chat.railOpen ? "dialog" : undefined} aria-modal={layout.railPresentation === "sheet" && chat.railOpen ? "true" : undefined} aria-label={layout.railPresentation === "sheet" && chat.railOpen ? t("chat.title") : undefined} tabindex={layout.railPresentation === "sheet" && chat.railOpen ? -1 : undefined} onkeydown={(event) => { if (layout.railPresentation === "sheet") handleSheetKeydown(event, () => { chat.railOpen = false; }); }} style={`--chat-rail-width:${railWidth}px`}>
    <ChatThreadRail onCollapse={() => { chat.railOpen = false; }} />
  </div>
  <button
    type="button"
    class="chat-rail-separator"
    class:hidden={!chat.railOpen || inspectorMaximized}
    role="separator"
    aria-orientation="vertical"
    aria-valuemin="160"
    aria-valuemax="520"
    aria-valuenow={Math.round(railWidth)}
    aria-label={t("chat.resizeRail")}
    onpointerdown={beginRailResize}
    onkeydown={(event) => resizePanelFromKey(event, "rail")}
  ></button>

  <main class="main-shell relative flex min-w-0 flex-1 flex-col bg-background/35" class:maximized-hidden={inspectorMaximized}>
    <ChatConversationHeader showRailButton={layout.railPresentation === "sheet" || !chat.railOpen} onOpenRail={() => { chat.railOpen = true; }} />
    {#if loadError}
      <div role="alert" class="m-auto max-w-md p-4 text-center text-sm text-destructive">{loadError}<div><button type="button" class="chat-secondary-button mt-3" onclick={() => { loadError = null; void chat.reload().catch((error) => { loadError = error instanceof Error ? error.message : String(error); }); }}>{t("common.retry")}</button></div></div>
    {:else if chat.loading}
      <div class="m-auto text-sm text-muted-foreground">{t("common.loading")}</div>
    {:else if chat.selectedThread}
      <ChatTimeline />
      {#if !chat.selectedThread.archivedAt}<ChatComposer />{/if}
    {:else}
      <ChatFirstUse />
    {/if}
  </main>

  <button type="button" class="chat-inspector-separator" class:hidden={!chat.inspectorOpen || inspectorMaximized} role="separator" aria-orientation="vertical" aria-valuemin="240" aria-valuemax="960" aria-valuenow={Math.round(inspectorWidth)} aria-label={t("chat.resizeInspector")} onpointerdown={beginInspectorResize} onkeydown={(event) => resizePanelFromKey(event, "inspector")}></button>
  <aside bind:this={inspectorShell} class="chat-inspector-shell" class:open={chat.inspectorOpen} class:maximized={inspectorMaximized} data-presentation={layout.inspectorPresentation} role={layout.inspectorPresentation === "sheet" ? "dialog" : undefined} aria-modal={layout.inspectorPresentation === "sheet" ? "true" : undefined} aria-label={t("chat.openInspector")} tabindex={layout.inspectorPresentation === "sheet" ? -1 : undefined} onkeydown={(event) => { if (layout.inspectorPresentation === "sheet") handleSheetKeydown(event, () => { chat.inspectorOpen = false; }); }} style={`--chat-inspector-width:${inspectorWidth}px`}>
    <ChatInspector onClose={() => { chat.inspectorOpen = false; }} onMaximizedChange={(value) => { inspectorMaximized = value; }} />
  </aside>

  {#if commandMenuOpen}
    <div class="absolute inset-0 z-50 flex items-start justify-center bg-black/30 p-3 pt-[10vh]">
      <button type="button" class="absolute inset-0" aria-label={t("chat.commandMenu.close")} onclick={() => { commandMenuOpen = false; }}></button>
      <div bind:this={commandDialog} class="relative w-full max-w-md rounded-lg border border-border bg-popover p-2 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.commandMenu.title")} tabindex="-1" onkeydown={(event) => trapFocus(event)}>
        <div class="flex items-center gap-2 border-b border-border px-2 py-2 text-xs text-muted-foreground"><Command size={14} />{t("chat.commandMenu.title")}</div>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; if (chat.selectedWorkspaceId) chat.newDraft(chat.selectedWorkspaceId); }}><MessageSquarePlus size={14} />{t("chat.newChat")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.railOpen = true; window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search")); }}><Search size={14} />{t("chat.search")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.inspectorOpen = !chat.inspectorOpen; }}><PanelRight size={14} />{t("chat.openInspector")}</button>
        <button type="button" class="chat-command" onclick={() => { void openSettingsFromCommandMenu(); }}><Settings size={14} />{t("chat.settings")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-rail-shell { width: var(--chat-rail-width); min-width: var(--chat-rail-width); transition: width 140ms ease, min-width 140ms ease, transform 140ms ease; }
  .chat-rail-shell.closed { width: 0; min-width: 0; overflow: hidden; }
  .maximized-hidden { display: none; }
  .chat-rail-separator { width: 4px; flex: 0 0 4px; cursor: col-resize; background: transparent; }
  .chat-rail-separator:hover, .chat-rail-separator:focus-visible { background: var(--ring); }
  .chat-inspector-separator { width: 4px; flex: 0 0 4px; cursor: col-resize; background: transparent; }
  .chat-inspector-separator:hover, .chat-inspector-separator:focus-visible { background: var(--ring); }
  .chat-inspector-shell { width: 0; min-width: 0; overflow: hidden; border-left: 0 solid var(--border); background: var(--background); transition: width 140ms ease, min-width 140ms ease; }
  .chat-inspector-shell.open { width: min(var(--chat-inspector-width), 34cqw); min-width: min(240px, 34cqw); border-left-width: 1px; }
  .chat-inspector-shell.maximized { width: 100%; min-width: 0; border-left-width: 0; }
  .chat-sheet-backdrop { position: absolute; inset: 0; z-index: 30; background: rgb(0 0 0 / 0.28); }
  .chat-command { display: flex; width: 100%; min-height: 2.25rem; align-items: center; gap: 0.5rem; border-radius: 0.375rem; padding: 0.375rem 0.5rem; font-size: 0.8rem; }
  .chat-command:hover { background: var(--accent); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell {
    position: absolute; inset-block: 0; left: 0; z-index: 40; width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); box-shadow: 8px 0 28px rgb(0 0 0 / 0.22);
  }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell.closed { width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); transform: translateX(-105%); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-separator { display: none; }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-separator { display: none; }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell { position: absolute; inset-block: 0; right: 0; z-index: 45; box-shadow: -8px 0 28px rgb(0 0 0 / 0.22); }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell.open { width: min(380px, 92cqw); min-width: min(280px, 92cqw); }
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell,
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell.closed,
  .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-shell.open { width: 100cqw; min-width: 0; box-shadow: none; }
  .chat-workspace[data-layout="minimum_recovery"][data-active-surface="rail"] .main-shell,
  .chat-workspace[data-layout="minimum_recovery"][data-active-surface="inspector"] .main-shell { display: none; }
  @media (prefers-reduced-motion: reduce) {
    :global(.chat-workspace *), :global(.chat-workspace *::before), :global(.chat-workspace *::after) {
      scroll-behavior: auto !important;
      transition: none !important;
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
    }
  }
</style>
