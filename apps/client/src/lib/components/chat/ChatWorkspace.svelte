<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Command from "@lucide/svelte/icons/command";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import PanelBottom from "@lucide/svelte/icons/panel-bottom";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Search from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import { filterThreadTitles, nextThreadIndex } from "$lib/chat/shell-model";
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
  import ChatBottomPanel from "./ChatBottomPanel.svelte";
  import ChatFirstUse from "./ChatFirstUse.svelte";
  import ChatInspector from "./ChatInspector.svelte";
  import ChatThreadRail from "./ChatThreadRail.svelte";
  import ChatTimeline from "./ChatTimeline.svelte";
  import { getChatBenchmarkHandle } from "./benchmark-handle.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  const DEFAULT_RAIL_WIDTH = 260;
  const DEFAULT_INSPECTOR_WIDTH = 520;
  const MIN_INSPECTOR_WIDTH = 420;
  const DEFAULT_BOTTOM_PANEL_HEIGHT = 260;
  const MIN_BOTTOM_PANEL_HEIGHT = 120;
  const INITIAL_SHELL_WIDTH = 1_200;
  const INITIAL_SHELL_HEIGHT = 700;
  const INITIAL_FONT_SCALE = 1;
  let rootElement: HTMLDivElement | undefined = $state();
  let railShell: HTMLDivElement | undefined = $state();
  let inspectorShell: HTMLElement | undefined = $state();
  let commandDialog: HTMLDivElement | undefined = $state();
  let commandMenuOpen = $state(false);
  let resizingRail = $state(false);
  let resizingInspector = $state(false);
  let resizingBottomPanel = $state(false);
  let railWidth = $state(DEFAULT_RAIL_WIDTH);
  let inspectorWidth = $state(DEFAULT_INSPECTOR_WIDTH);
  let bottomPanelHeight = $state(DEFAULT_BOTTOM_PANEL_HEIGHT);
  let bottomPanelOpen = $state(false);
  let bottomInitializedThreadIds = $state<string[]>([]);
  let inspectorMaximized = $state(false);
  let inspectorWasOpen = false;
  let inspectorReturnFocus: HTMLElement | null = null;
  let railModalWasOpen = false;
  let railReturnFocus: HTMLElement | null = null;
  let commandMenuWasOpen = false;
  let commandReturnFocus: HTMLElement | null = null;
  let shellWidth = $state(INITIAL_SHELL_WIDTH);
  let shellHeight = $state(INITIAL_SHELL_HEIGHT);
  let fontScale = $state(INITIAL_FONT_SCALE);
  let layout = $state<ChatLayoutDecision>(chatLayoutDecision({
    containerWidth: INITIAL_SHELL_WIDTH,
    containerHeight: INITIAL_SHELL_HEIGHT,
    fontScale: INITIAL_FONT_SCALE,
    railOpen: true,
    inspectorOpen: false,
    railWidth: DEFAULT_RAIL_WIDTH,
    inspectorWidth: DEFAULT_INSPECTOR_WIDTH,
  }));
  let loadError = $state<string | null>(null);
  let layoutError = $state<string | null>(null);
  let politeAnnouncement = $state("");
  let assertiveAnnouncement = $state("");
  let announcedThreadId: string | null = null;
  let announcedTurnState: string | null = null;
  let announcedRequestId: string | null = null;
  let railResizeFrame: number | null = null;
  let inspectorResizeFrame: number | null = null;
  let bottomResizeFrame: number | null = null;

  onMount(() => {
    void Promise.all([chat.ensureLoaded(), projects.ensureLoaded()]).catch((error) => {
      loadError = error instanceof Error ? error.message : String(error);
    });
    railWidth = chat.settings?.configuration.panels.railWidthPx ?? DEFAULT_RAIL_WIDTH;
    inspectorWidth = Math.max(
      MIN_INSPECTOR_WIDTH,
      chat.settings?.configuration.panels.inspectorWidthPx ?? DEFAULT_INSPECTOR_WIDTH,
    );
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
    const unregisterBenchmark = getChatBenchmarkHandle().register({
      threadIds: () => chat.activeThreads.map((thread) => thread.id),
      waitUntilUsable: () => waitForBenchmarkState(() => !chat.loading && chat.activeThreads.length > 0),
      switchThread: async (threadId) => {
        chat.selectThread(threadId);
        await waitForBenchmarkState(() => (
          chat.selectedThreadId === threadId
          && !chat.timelineLoading
          && chat.timelineItems.length > 0
        ));
        await nextAnimationFrame();
      },
      localSearch: (query) => filterThreadTitles(
        [...chat.activeThreads, ...chat.archivedThreads],
        query,
      ).length,
      streamFrames: (frameCount) => measureBenchmarkStreamFrames(frameCount),
    });
    return () => {
      unregisterBenchmark();
      observer.disconnect();
      if (railResizeFrame !== null) window.cancelAnimationFrame(railResizeFrame);
      if (inspectorResizeFrame !== null) window.cancelAnimationFrame(inspectorResizeFrame);
      if (bottomResizeFrame !== null) window.cancelAnimationFrame(bottomResizeFrame);
      window.removeEventListener("ganbaru-ai:chat-revert-message", revertMessage);
      void unlisten.then((dispose) => dispose());
    };
  });

  function nextAnimationFrame(): Promise<number> {
    return new Promise((resolve) => requestAnimationFrame(resolve));
  }

  async function waitForBenchmarkState(
    predicate: () => boolean,
    timeoutMs = 10_000,
  ): Promise<void> {
    const deadline = performance.now() + timeoutMs;
    while (!predicate()) {
      if (performance.now() >= deadline) throw new Error("Chat benchmark state timed out");
      await nextAnimationFrame();
    }
  }

  async function measureBenchmarkStreamFrames(frameCount: number): Promise<number[]> {
    const targetIndex = chat.timelineItems.findLastIndex((item) => item.kind === "message");
    const original = chat.timelineItems[targetIndex];
    if (!original || targetIndex < 0) throw new Error("Chat benchmark requires a loaded message");
    const originalValue = original.data.value;
    if (!originalValue || typeof originalValue !== "object" || Array.isArray(originalValue)) {
      throw new Error("Chat benchmark message payload is invalid");
    }
    const markdown = typeof originalValue.markdown === "string" ? originalValue.markdown : "";
    const samples: number[] = [];
    let previous = await nextAnimationFrame();
    try {
      for (let index = 0; index < frameCount; index++) {
        chat.timelineItems = chat.timelineItems.map((item, itemIndex) => itemIndex === targetIndex
          ? {
              ...item,
              data: {
                ...item.data,
                value: { ...originalValue, markdown: `${markdown}\nstream-${index}` },
              },
            }
          : item);
        await tick();
        const painted = await nextAnimationFrame();
        samples.push(painted - previous);
        previous = painted;
      }
    } finally {
      chat.timelineItems = chat.timelineItems.map((item, itemIndex) => itemIndex === targetIndex
        ? { ...item, data: { ...item.data, value: originalValue } }
        : item);
      await tick();
    }
    return samples;
  }

  $effect(() => {
    if (!resizingRail && chat.settings) railWidth = chat.settings.configuration.panels.railWidthPx;
    if (!resizingInspector && chat.settings) {
      inspectorWidth = Math.max(MIN_INSPECTOR_WIDTH, chat.settings.configuration.panels.inspectorWidthPx);
    }
  });

  $effect(() => {
    const threadId = chat.selectedThreadId;
    if (!threadId || bottomInitializedThreadIds.includes(threadId)) return;
    bottomInitializedThreadIds = [...bottomInitializedThreadIds, threadId];
    bottomPanelOpen = true;
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
    event.preventDefault();
    resizingRail = true;
    const startX = event.clientX;
    const startWidth = railWidth;
    const target = event.currentTarget as HTMLElement;
    target.focus();
    target.setPointerCapture(event.pointerId);
    let pendingWidth = startWidth;
    const move = (moveEvent: PointerEvent) => {
      pendingWidth = Math.max(160, Math.min(520, startWidth + moveEvent.clientX - startX));
      if (railResizeFrame !== null) return;
      railResizeFrame = window.requestAnimationFrame(() => {
        railWidth = pendingWidth;
        railResizeFrame = null;
      });
    };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (railResizeFrame !== null) window.cancelAnimationFrame(railResizeFrame);
      railResizeFrame = null;
      railWidth = pendingWidth;
      resizingRail = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function beginInspectorResize(event: PointerEvent): void {
    event.preventDefault();
    resizingInspector = true;
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.focus();
    target.setPointerCapture(event.pointerId);
    let pendingWidth = startWidth;
    const move = (moveEvent: PointerEvent) => {
      pendingWidth = Math.max(MIN_INSPECTOR_WIDTH, Math.min(960, startWidth + startX - moveEvent.clientX));
      if (inspectorResizeFrame !== null) return;
      inspectorResizeFrame = window.requestAnimationFrame(() => {
        inspectorWidth = pendingWidth;
        inspectorResizeFrame = null;
      });
    };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (inspectorResizeFrame !== null) window.cancelAnimationFrame(inspectorResizeFrame);
      inspectorResizeFrame = null;
      inspectorWidth = pendingWidth;
      resizingInspector = false;
      void persistPanelWidths();
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function beginBottomPanelResize(event: PointerEvent): void {
    event.preventDefault();
    resizingBottomPanel = true;
    const startY = event.clientY;
    const startHeight = bottomPanelHeight;
    const target = event.currentTarget as HTMLElement;
    let pendingHeight = startHeight;
    target.focus();
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => {
      const maximum = Math.max(MIN_BOTTOM_PANEL_HEIGHT, shellHeight * 0.7);
      pendingHeight = Math.max(
        MIN_BOTTOM_PANEL_HEIGHT,
        Math.min(maximum, startHeight + startY - moveEvent.clientY),
      );
      if (bottomResizeFrame !== null) return;
      bottomResizeFrame = window.requestAnimationFrame(() => {
        bottomPanelHeight = pendingHeight;
        bottomResizeFrame = null;
      });
    };
    const end = () => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (bottomResizeFrame !== null) window.cancelAnimationFrame(bottomResizeFrame);
      bottomResizeFrame = null;
      bottomPanelHeight = pendingHeight;
      resizingBottomPanel = false;
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
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
          minimum: MIN_INSPECTOR_WIDTH,
          maximum: 960,
          defaultValue: DEFAULT_INSPECTOR_WIDTH,
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

  function resizeBottomPanelFromKey(event: KeyboardEvent): void {
    const maximum = Math.max(MIN_BOTTOM_PANEL_HEIGHT, Math.round(shellHeight * 0.7));
    let next: number;
    switch (event.key) {
      case "ArrowUp": next = bottomPanelHeight + 16; break;
      case "ArrowDown": next = bottomPanelHeight - 16; break;
      case "Home": next = MIN_BOTTOM_PANEL_HEIGHT; break;
      case "End": next = maximum; break;
      case "Enter": next = DEFAULT_BOTTOM_PANEL_HEIGHT; break;
      default: return;
    }
    event.preventDefault();
    bottomPanelHeight = Math.max(MIN_BOTTOM_PANEL_HEIGHT, Math.min(maximum, next));
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

<div bind:this={rootElement} class="chat-workspace @container/chat-shell relative flex h-full min-h-0 overflow-hidden" class:resizing-panels={resizingRail || resizingInspector || resizingBottomPanel} data-chat-workspace data-layout={layout.variant} data-rail-presentation={layout.railPresentation} data-inspector-presentation={layout.inspectorPresentation} data-active-surface={layout.activeSurface} style="background-color:var(--cal-bg);container-type:inline-size;container-name:chat-shell;">
  <div class="sr-only" aria-live="polite" aria-atomic="true">{politeAnnouncement}</div>
  <div class="sr-only" aria-live="assertive" aria-atomic="true">{assertiveAnnouncement}</div>
  {#if layoutError}<div role="alert" class="absolute inset-x-2 top-2 z-50 rounded border border-destructive/40 bg-background p-2 text-xs text-destructive">{layoutError}</div>{/if}
  {#if layout.inspectorPresentation === "sheet" && chat.inspectorOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.closeInspector")} onclick={() => { chat.inspectorOpen = false; }}></button>
  {:else if layout.railPresentation === "sheet" && chat.railOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.collapseRail")} onclick={() => { chat.railOpen = false; }}></button>
  {/if}
  <div bind:this={railShell} class="chat-rail-shell" class:closed={!chat.railOpen} class:maximized-hidden={inspectorMaximized} role={layout.railPresentation === "sheet" && chat.railOpen ? "dialog" : undefined} aria-modal={layout.railPresentation === "sheet" && chat.railOpen ? "true" : undefined} aria-label={layout.railPresentation === "sheet" && chat.railOpen ? t("chat.title") : undefined} onkeydown={(event) => { if (layout.railPresentation === "sheet") handleSheetKeydown(event, () => { chat.railOpen = false; }); }} style={`--chat-rail-width:${railWidth}px`}>
    <ChatThreadRail onCollapse={() => { chat.railOpen = false; }} />
  </div>
  <input
    type="range"
    class="chat-rail-separator"
    class:hidden={!chat.railOpen || inspectorMaximized}
    min="160"
    max="520"
    value={Math.round(railWidth)}
    aria-label={t("chat.resizeRail")}
    onpointerdown={beginRailResize}
    onkeydown={(event) => resizePanelFromKey(event, "rail")}
  />

  <div class="workspace-content" class:maximized={inspectorMaximized}>
    <div class="workspace-top">
      <main class="main-shell relative flex min-w-0 flex-1 flex-col" class:maximized-hidden={inspectorMaximized}>
        <ChatConversationHeader
          showRailButton={layout.railPresentation === "sheet" || !chat.railOpen}
          onOpenRail={() => { chat.railOpen = true; }}
          {bottomPanelOpen}
          onToggleBottomPanel={() => { bottomPanelOpen = !bottomPanelOpen; }}
        />
        {#if loadError}
          <div role="alert" class="m-auto max-w-md p-4 text-center text-sm text-destructive">{loadError}<div><button type="button" class="chat-secondary-button mt-3" onclick={() => { loadError = null; void chat.reload().catch((error) => { loadError = error instanceof Error ? error.message : String(error); }); }}>{t("common.retry")}</button></div></div>
        {:else if chat.loading}
          <div class="m-auto text-sm text-muted-foreground">{t("common.loading")}</div>
        {:else if chat.selectedThread}
          <div class="chat-conversation-shell">
            <ChatTimeline />
            {#if !chat.selectedThread.archivedAt}
              <div class="chat-composer-dock">
                <div class="chat-composer-backdrop" aria-hidden="true"></div>
                <ChatComposer />
              </div>
            {/if}
          </div>
        {:else}
          <ChatFirstUse />
        {/if}
      </main>

      <input type="range" class="chat-inspector-separator" class:hidden={!chat.inspectorOpen || inspectorMaximized} min={MIN_INSPECTOR_WIDTH} max="960" value={Math.round(inspectorWidth)} aria-label={t("chat.resizeInspector")} onpointerdown={beginInspectorResize} onkeydown={(event) => resizePanelFromKey(event, "inspector")} />
      <aside bind:this={inspectorShell} class="chat-inspector-shell" class:open={chat.inspectorOpen} class:maximized={inspectorMaximized} data-presentation={layout.inspectorPresentation} role={layout.inspectorPresentation === "sheet" ? "dialog" : undefined} aria-modal={layout.inspectorPresentation === "sheet" ? "true" : undefined} aria-label={t("chat.openInspector")} onkeydown={(event) => { if (layout.inspectorPresentation === "sheet") handleSheetKeydown(event, () => { chat.inspectorOpen = false; }); }} style={`--chat-inspector-width:${inspectorWidth}px`}>
        <ChatInspector onClose={() => { chat.inspectorOpen = false; }} onMaximizedChange={(value) => { inspectorMaximized = value; }} />
      </aside>
    </div>

    {#if bottomPanelOpen && !inspectorMaximized && layout.variant !== "minimum_recovery"}
      <input type="range" class="chat-bottom-separator" min={MIN_BOTTOM_PANEL_HEIGHT} max={Math.max(MIN_BOTTOM_PANEL_HEIGHT, Math.round(shellHeight * 0.7))} value={Math.round(bottomPanelHeight)} aria-label={t("chat.resizeBottomPanel")} onpointerdown={beginBottomPanelResize} onkeydown={resizeBottomPanelFromKey} />
      <div class="chat-bottom-shell" style={`--chat-bottom-height:${bottomPanelHeight}px`}>
        <ChatBottomPanel onClose={() => { bottomPanelOpen = false; }} />
      </div>
    {/if}
  </div>

  {#if commandMenuOpen}
    <div class="absolute inset-0 z-50 flex items-start justify-center bg-black/30 p-3 pt-[10vh]">
      <button type="button" class="absolute inset-0" aria-label={t("chat.commandMenu.close")} onclick={() => { commandMenuOpen = false; }}></button>
      <div bind:this={commandDialog} class="relative w-full max-w-md rounded-lg border border-border bg-popover p-2 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.commandMenu.title")} tabindex="-1" onkeydown={(event) => trapFocus(event)}>
        <div class="flex items-center gap-2 border-b border-border px-2 py-2 text-xs text-muted-foreground"><Command size={14} />{t("chat.commandMenu.title")}</div>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; if (chat.selectedWorkspaceId) chat.newDraft(chat.selectedWorkspaceId); }}><MessageSquarePlus size={14} />{t("chat.newChat")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.railOpen = true; window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search")); }}><Search size={14} />{t("chat.search")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; bottomPanelOpen = !bottomPanelOpen; }}><PanelBottom size={14} />{bottomPanelOpen ? t("chat.closeBottomPanel") : t("chat.openBottomPanel")}</button>
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
  .chat-rail-separator { width: 4px; min-width: 0; flex: 0 0 4px; appearance: none; border: 0; border-radius: 0; padding: 0; cursor: col-resize; background: transparent; }
  .chat-rail-separator:hover, .chat-rail-separator:focus-visible { background: var(--ring); }
  .chat-inspector-separator { width: 4px; min-width: 0; flex: 0 0 4px; appearance: none; border: 0; border-radius: 0; padding: 0; cursor: col-resize; background: transparent; }
  .chat-inspector-separator:hover, .chat-inspector-separator:focus-visible { background: var(--ring); }
  .workspace-content { display: flex; min-width: 0; min-height: 0; flex: 1; flex-direction: column; }
  .workspace-top { position: relative; display: flex; min-width: 0; min-height: 0; flex: 1; }
  .chat-inspector-shell { width: 0; min-width: 0; overflow: hidden; border-left: 0 solid var(--border); background: var(--cal-bg); transition: width 140ms ease, min-width 140ms ease; }
  .chat-inspector-shell.open { width: min(var(--chat-inspector-width), 46cqw); min-width: min(420px, 46cqw); border-left-width: 1px; }
  .chat-inspector-shell.maximized { width: 100%; min-width: 0; border-left-width: 0; }
  .chat-bottom-separator { width: 100%; height: 4px; min-height: 4px; flex: 0 0 4px; appearance: none; border: 0; border-radius: 0; padding: 0; cursor: row-resize; background: transparent; }
  .chat-bottom-separator:hover, .chat-bottom-separator:focus-visible { background: var(--ring); }
  .chat-bottom-shell { height: min(var(--chat-bottom-height), 45%); min-height: min(120px, 45%); flex: 0 0 min(var(--chat-bottom-height), 45%); overflow: hidden; border-top: 1px solid var(--border); }
  .chat-workspace.resizing-panels, .chat-workspace.resizing-panels * { user-select: none; }
  .chat-workspace.resizing-panels .chat-rail-shell, .chat-workspace.resizing-panels .chat-inspector-shell { transition: none; }
  .chat-sheet-backdrop { position: absolute; inset: 0; z-index: 30; background: rgb(0 0 0 / 0.28); }
  .main-shell { background: var(--cal-bg); }
  .chat-conversation-shell { position: relative; display: flex; min-height: 0; flex: 1; flex-direction: column; overflow: hidden; }
  .chat-composer-dock { pointer-events: none; position: absolute; inset-inline: 0; bottom: 0; z-index: 20; padding: 0.5rem 0.75rem 0.75rem; }
  .chat-composer-backdrop { position: absolute; inset: -1.5rem 0 -2rem; background: linear-gradient(to bottom, transparent, color-mix(in srgb, var(--cal-bg) 72%, transparent) 35%, var(--cal-bg) 74%); backdrop-filter: blur(10px); -webkit-mask-image: linear-gradient(to bottom, transparent, black 35%); mask-image: linear-gradient(to bottom, transparent, black 35%); }
  .chat-composer-dock :global(.chat-composer) { pointer-events: auto; }
  .chat-command { display: flex; width: 100%; min-height: 2.25rem; align-items: center; gap: 0.5rem; border-radius: 0.375rem; padding: 0.375rem 0.5rem; font-size: 0.8rem; }
  .chat-command:hover { background: var(--accent); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell {
    position: absolute; inset-block: 0; left: 0; z-index: 40; width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); box-shadow: 8px 0 28px rgb(0 0 0 / 0.22);
  }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell.closed { width: min(var(--chat-rail-width), 88cqw); min-width: min(var(--chat-rail-width), 88cqw); transform: translateX(-105%); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-separator { display: none; }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-separator { display: none; }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell { position: absolute; inset-block: 0; right: 0; z-index: 45; box-shadow: -8px 0 28px rgb(0 0 0 / 0.22); }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell.open { width: min(620px, 94cqw); min-width: min(320px, 94cqw); }
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell,
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell.closed,
  .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-shell.open { width: 100cqw; min-width: 0; box-shadow: none; }
  .chat-workspace[data-layout="minimum_recovery"][data-active-surface="rail"] .main-shell,
  .chat-workspace[data-layout="minimum_recovery"][data-active-surface="inspector"] .main-shell { display: none; }
  :global(.chat-workspace svg) { stroke-width: 1.5; }
  @media (prefers-reduced-motion: reduce) {
    :global(.chat-workspace *), :global(.chat-workspace *::before), :global(.chat-workspace *::after) {
      scroll-behavior: auto !important;
      transition: none !important;
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
    }
  }
</style>
