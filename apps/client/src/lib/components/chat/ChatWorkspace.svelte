<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import { quintOut } from "svelte/easing";
  import { slide } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Command from "@lucide/svelte/icons/command";
  import Hash from "@lucide/svelte/icons/hash";
  import MessageSquarePlus from "@lucide/svelte/icons/message-square-plus";
  import PanelBottom from "@lucide/svelte/icons/panel-bottom";
  import PanelRight from "@lucide/svelte/icons/panel-right";
  import Search from "@lucide/svelte/icons/search";
  import Settings from "@lucide/svelte/icons/settings";
  import { chatHeaderActionInset, nextThreadIndex } from "$lib/chat/shell-model";
  import { loadChatCodeEditorRuntime } from "$lib/chat/code-editor-loader";
  import { inspectorFocusAction } from "$lib/chat/inspector-model";
  import {
    alignPanelSizeToDevicePixel,
    chatBottomPanelResizeMaximum,
    chatLayoutDecision,
    chatInspectorResizeMaximum,
    fittedChatBottomPanelHeight,
    fittedChatInspectorWidth,
    panelSizeWithCollapseSnap,
    panelWidthFromKey,
    type ChatLayoutDecision,
  } from "$lib/chat/responsive-layout";
  import type { ChatChannelRead, ChatPanelPreferences } from "$lib/chat/contracts";
  import { parseChatChangeNotification } from "$lib/chat/validation";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import * as chatApi from "$lib/api/chat";
  import ChatWorkspaceHeader from "./ChatWorkspaceHeader.svelte";
  import ChatComposer from "./ChatComposer.svelte";
  import ChatFirstUse from "./ChatFirstUse.svelte";
  import ChatHeaderActions from "./ChatHeaderActions.svelte";
  import ChatChannelRail from "./ChatChannelRail.svelte";
  import ChatTimeline from "./ChatTimeline.svelte";
  import ChatWorkspaceObserver from "./ChatWorkspaceObserver.svelte";
  import ChatWorkspacePanel from "./ChatWorkspacePanel.svelte";
  import { getChatBenchmarkHandle } from "./benchmark-handle.svelte";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  const LEGACY_INSPECTOR_WIDTH = 360;
  const DEFAULT_INSPECTOR_WIDTH = 520;
  const MIN_INSPECTOR_WIDTH = 240;
  const MAX_INSPECTOR_WIDTH = 960;
  const DEFAULT_BOTTOM_PANEL_HEIGHT = 190;
  const MIN_BOTTOM_PANEL_HEIGHT = 96;
  const MAX_BOTTOM_PANEL_HEIGHT = 520;
  const PANEL_TRANSITION_MS = 440;
  const HEADER_ACTION_EDGE_GAP_PX = 12;
  const INITIAL_SHELL_WIDTH = 1_200;
  const INITIAL_SHELL_HEIGHT = 700;
  const INITIAL_FONT_SCALE = 1;
  const initialInspectorWidth = chat.settings?.configuration.panels.inspectorWidthPx
    ?? DEFAULT_INSPECTOR_WIDTH;
  let rootElement: HTMLDivElement | undefined = $state();
  let primaryHeaderElement: HTMLDivElement | undefined = $state();
  let globalActionsElement: HTMLDivElement | undefined = $state();
  let railShell: HTMLDivElement | undefined = $state();
  let inspectorShell: HTMLElement | undefined = $state();
  let composerDockElement: HTMLDivElement | undefined = $state();
  let composerDockHeight = $state(0);
  let commandDialog: HTMLDivElement | undefined = $state();
  let commandMenuOpen = $state(false);
  let resizingInspector = $state(false);
  let resizingBottomPanel = $state(false);
  let inspectorWidth = $state(initialInspectorWidth);
  let bottomPanelHeight = $state(DEFAULT_BOTTOM_PANEL_HEIGHT);
  let bottomPanelOpen = $state(false);
  let bottomPanelMounted = $state(false);
  let bottomPanelSkipCloseTransition = $state(false);
  let reducedMotion = $state(false);
  let headerEditingTitle = $state(false);
  let inspectorWasOpen = false;
  let inspectorReturnFocus: HTMLElement | null = null;
  let railModalWasOpen = false;
  let railReturnFocus: HTMLElement | null = null;
  let commandMenuWasOpen = false;
  let commandReturnFocus: HTMLElement | null = null;
  let shellWidth = $state(INITIAL_SHELL_WIDTH);
  let shellHeight = $state(INITIAL_SHELL_HEIGHT);
  let shellRight = $state(INITIAL_SHELL_WIDTH);
  let shellBottom = $state(INITIAL_SHELL_HEIGHT);
  let displayPixelRatio = $state(1);
  let fontScale = $state(INITIAL_FONT_SCALE);
  let layout = $state<ChatLayoutDecision>(chatLayoutDecision({
    containerWidth: INITIAL_SHELL_WIDTH,
    containerHeight: INITIAL_SHELL_HEIGHT,
    fontScale: INITIAL_FONT_SCALE,
    railOpen: chat.railOpen,
    inspectorOpen: chat.inspectorOpen,
    inspectorWidth: initialInspectorWidth,
  }));
  const bottomPanelVisible = $derived(
    bottomPanelOpen && layout.variant !== "minimum_recovery",
  );
  let loadError = $state<string | null>(null);
  let archiveQuery = $state("");
  let archiveRestoringId = $state<string | null>(null);
  let archiveError = $state<string | null>(null);
  const filteredArchivedChannels = $derived(chat.archivedChannels.filter((channel) => {
    const normalized = archiveQuery.trim().toLocaleLowerCase();
    return !normalized
      || channel.name.toLocaleLowerCase().includes(normalized)
      || channel.topic.toLocaleLowerCase().includes(normalized);
  }));
  let initialLoadingVisible = $state(false);
  let layoutError = $state<string | null>(null);
  let politeAnnouncement = $state("");
  let assertiveAnnouncement = $state("");
  let announcedThreadId: string | null = null;
  let announcedTurnState: string | null = null;
  let announcedRequestId: string | null = null;
  let inspectorResizeFrame: number | null = null;
  let bottomResizeFrame: number | null = null;
  let inspectorResizeEndFrame: number | null = null;
  let bottomResizeEndFrame: number | null = null;
  let panelTransitionFrame: number | null = null;
  let initialLoadingTimer: number | null = null;
  let panelTransitionsEnabled = $state(false);
  type ResizableOuterPanel = "inspector" | "bottom";
  let snapTransitioning = $state<Record<ResizableOuterPanel, boolean>>({
    inspector: false,
    bottom: false,
  });
  const snapTransitionTimers: Partial<Record<ResizableOuterPanel, number>> = {};
  let panelPreferencesInitialized = false;
  let inspectorUsesPromotedDefault = false;
  let pendingPanelWidths = $state<ChatPanelPreferences | null>(null);
  let panelWidthSave: Promise<void> | null = null;

  onMount(() => {
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotionPreference = () => { reducedMotion = motionQuery.matches; };
    updateMotionPreference();
    motionQuery.addEventListener("change", updateMotionPreference);
    void Promise.all([chat.ensureLoaded(), projects.ensureLoaded()])
      .then(() => {
        requestAnimationFrame(() => {
          void loadChatCodeEditorRuntime().catch(() => undefined);
        });
      })
      .catch((error) => {
        loadError = error instanceof Error ? error.message : String(error);
      });
    refreshWorkspacePixelGeometry();
    inspectorWidth = alignInspectorWidthToDisplay(
      chat.settings?.configuration.panels.inspectorWidthPx ?? DEFAULT_INSPECTOR_WIDTH,
    );
    if (chat.settings) enablePanelTransitionsAfterLayout();
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
      refreshWorkspacePixelGeometry();
      refreshHeaderActionInset();
      const rootSize = rootElement ? Number.parseFloat(getComputedStyle(rootElement).fontSize) : 15;
      fontScale = Number.isFinite(rootSize) ? Math.max(1, rootSize / 15) : 1;
    });
    if (rootElement) observer.observe(rootElement);
    const headerGeometryObserver = new ResizeObserver(refreshHeaderActionInset);
    if (primaryHeaderElement) headerGeometryObserver.observe(primaryHeaderElement);
    if (globalActionsElement) headerGeometryObserver.observe(globalActionsElement);
    refreshHeaderActionInset();
    const revertMessage = (event: Event) => {
      if (!(event instanceof CustomEvent) || !isRevertMessageDetail(event.detail)) return;
      void restoreMessageCheckpoint(event.detail.threadId, event.detail.checkpointId);
    };
    const openWorkspaceTool = () => {
      chat.inspectorOpen = true;
    };
    window.addEventListener("ganbaru-ai:chat-revert-message", revertMessage);
    window.addEventListener("ganbaru-ai:chat-open-changes", openWorkspaceTool);
    window.addEventListener("ganbaru-ai:chat-open-review", openWorkspaceTool);
    window.addEventListener("ganbaru-ai:chat-open-file", openWorkspaceTool);
    const unregisterBenchmark = getChatBenchmarkHandle().register({
      channelIds: () => chat.activeChannels.map((channel) => channel.id),
      waitUntilUsable: () => waitForBenchmarkState(() => !chat.loading && chat.activeChannels.length > 0),
      switchChannel: async (channelId) => {
        await chat.selectChannel(channelId);
        await waitForBenchmarkState(() => (
          chat.selectedChannelId === channelId
          && !chat.timelineLoading
          && chat.timelineItems.length > 0
        ));
        await nextAnimationFrame();
      },
      localSearch: (query) => {
        const normalized = query.trim().toLocaleLowerCase();
        return [...chat.activeChannels, ...chat.archivedChannels].filter((channel) => (
          channel.name.toLocaleLowerCase().includes(normalized)
          || channel.topic.toLocaleLowerCase().includes(normalized)
        )).length;
      },
      streamFrames: (frameCount) => measureBenchmarkStreamFrames(frameCount),
    });
    return () => {
      unregisterBenchmark();
      observer.disconnect();
      headerGeometryObserver.disconnect();
      if (inspectorResizeFrame !== null) window.cancelAnimationFrame(inspectorResizeFrame);
      if (bottomResizeFrame !== null) window.cancelAnimationFrame(bottomResizeFrame);
      if (inspectorResizeEndFrame !== null) window.cancelAnimationFrame(inspectorResizeEndFrame);
      if (bottomResizeEndFrame !== null) window.cancelAnimationFrame(bottomResizeEndFrame);
      if (panelTransitionFrame !== null) window.cancelAnimationFrame(panelTransitionFrame);
      if (initialLoadingTimer !== null) window.clearTimeout(initialLoadingTimer);
      for (const timer of Object.values(snapTransitionTimers)) {
        if (timer !== undefined) window.clearTimeout(timer);
      }
      motionQuery.removeEventListener("change", updateMotionPreference);
      window.removeEventListener("ganbaru-ai:chat-revert-message", revertMessage);
      window.removeEventListener("ganbaru-ai:chat-open-changes", openWorkspaceTool);
      window.removeEventListener("ganbaru-ai:chat-open-review", openWorkspaceTool);
      window.removeEventListener("ganbaru-ai:chat-open-file", openWorkspaceTool);
      void unlisten.then((dispose) => dispose());
    };
  });

  $effect(() => {
    const projectId = projects.selectedProjectId;
    if (chat.loading) return;
    untrack(() => {
      void chat.syncProjectSelection(projectId).catch((error: unknown) => {
        if (projects.selectedProjectId === projectId) {
          loadError = error instanceof Error ? error.message : String(error);
        }
      });
    });
  });

  $effect(() => {
    if (!chat.loading || chat.settings) {
      initialLoadingVisible = false;
      if (initialLoadingTimer !== null) window.clearTimeout(initialLoadingTimer);
      initialLoadingTimer = null;
      return;
    }
    if (initialLoadingTimer !== null) return;
    initialLoadingTimer = window.setTimeout(() => {
      initialLoadingTimer = null;
      if (chat.loading && !chat.settings) initialLoadingVisible = true;
    }, 140);
  });

  function nextAnimationFrame(): Promise<number> {
    return new Promise((resolve) => requestAnimationFrame(resolve));
  }

  async function restoreArchivedChannel(channel: ChatChannelRead): Promise<void> {
    archiveRestoringId = channel.id;
    archiveError = null;
    try {
      await chat.restoreChannel(channel);
    } catch (cause: unknown) {
      archiveError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      archiveRestoringId = null;
    }
  }

  function refreshHeaderActionInset(): void {
    if (!primaryHeaderElement || !globalActionsElement) return;
    const bottomPanelAction = globalActionsElement.querySelector<HTMLElement>(
      "[data-chat-bottom-panel-action]",
    );
    const inspectorAction = globalActionsElement.querySelector<HTMLElement>(
      "[data-chat-inspector-action]",
    );
    const headerBounds = primaryHeaderElement.getBoundingClientRect();
    const actionBounds = bottomPanelAction?.getBoundingClientRect()
      ?? globalActionsElement.getBoundingClientRect();
    const inspectorActionBounds = inspectorAction?.getBoundingClientRect();
    const actionGap = inspectorActionBounds
      ? Math.max(0, inspectorActionBounds.left - actionBounds.right)
      : HEADER_ACTION_EDGE_GAP_PX;
    const inset = chatHeaderActionInset(
      headerBounds.right,
      actionBounds.left,
      actionGap,
    );
    primaryHeaderElement.style.setProperty("--chat-header-action-inset", `${inset}px`);
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
    if (!chat.settings) return;
    const configuredInspectorWidth = chat.settings.configuration.panels.inspectorWidthPx;
    if (!panelPreferencesInitialized) {
      inspectorUsesPromotedDefault = configuredInspectorWidth === LEGACY_INSPECTOR_WIDTH;
      panelPreferencesInitialized = true;
    }
    if (configuredInspectorWidth !== LEGACY_INSPECTOR_WIDTH) inspectorUsesPromotedDefault = false;
    if (!resizingInspector && !pendingPanelWidths) {
      inspectorWidth = alignInspectorWidthToDisplay(
        inspectorUsesPromotedDefault ? DEFAULT_INSPECTOR_WIDTH : configuredInspectorWidth,
      );
    }
    enablePanelTransitionsAfterLayout();
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
      inspectorWidth,
      previousVariant: layout.variant,
    });
    if (!sameLayout(layout, next)) layout = next;
  });

  $effect(() => {
    const maximum = bottomPanelResizeMaximum();
    if (!resizingBottomPanel) {
      bottomPanelHeight = alignBottomPanelHeightToDisplay(
        bottomPanelHeight,
        maximum,
      );
    }
  });

  $effect(() => {
    const dock = composerDockElement;
    if (!dock) {
      composerDockHeight = 0;
      return;
    }
    const updateHeight = () => {
      const next = Math.ceil(dock.getBoundingClientRect().height);
      if (next !== composerDockHeight) composerDockHeight = next;
    };
    updateHeight();
    const observer = new ResizeObserver(updateHeight);
    observer.observe(dock);
    return () => observer.disconnect();
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
      chat.railOpen = true;
      window.dispatchEvent(new Event("ganbaru-ai:chat-new-channel"));
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
      const index = chat.activeChannels.findIndex((channel) => channel.id === chat.selectedChannelId);
      const next = nextThreadIndex(index, chat.activeChannels.length, event.key === "ArrowDown" ? "next" : "previous");
      const channel = chat.activeChannels[next];
      if (channel) void chat.selectChannel(channel.id);
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

  function refreshWorkspacePixelGeometry(): void {
    if (!rootElement) return;
    const bounds = rootElement.getBoundingClientRect();
    shellWidth = bounds.width;
    shellHeight = bounds.height;
    shellRight = bounds.right;
    shellBottom = bounds.bottom;
    displayPixelRatio = Number.isFinite(window.devicePixelRatio) && window.devicePixelRatio > 0
      ? window.devicePixelRatio
      : 1;
  }

  function enablePanelTransitionsAfterLayout(): void {
    if (panelTransitionsEnabled || panelTransitionFrame !== null) return;
    panelTransitionFrame = window.requestAnimationFrame(() => {
      panelTransitionFrame = window.requestAnimationFrame(() => {
        panelTransitionsEnabled = true;
        panelTransitionFrame = null;
      });
    });
  }

  function alignInspectorWidthToDisplay(
    value: number,
    maximum = inspectorResizeMaximum(),
  ): number {
    return alignPanelSizeToDevicePixel({
      value,
      minimum: MIN_INSPECTOR_WIDTH,
      maximum,
      anchor: shellRight,
      direction: "from-end",
      devicePixelRatio: displayPixelRatio,
    });
  }

  function alignBottomPanelHeightToDisplay(
    value: number,
    maximum = bottomPanelResizeMaximum(),
  ): number {
    return alignPanelSizeToDevicePixel({
      value,
      minimum: MIN_BOTTOM_PANEL_HEIGHT,
      maximum,
      anchor: shellBottom,
      direction: "from-end",
      devicePixelRatio: displayPixelRatio,
    });
  }

  function inspectorWidthFromPointer(requested: number): number {
    const maximum = inspectorResizeMaximum();
    const snapped = panelSizeWithCollapseSnap(
      requested,
      MIN_INSPECTOR_WIDTH,
      maximum,
    );
    return snapped === 0 ? 0 : alignInspectorWidthToDisplay(snapped, maximum);
  }

  function bottomPanelHeightFromPointer(requested: number): number {
    const maximum = bottomPanelResizeMaximum();
    const snapped = panelSizeWithCollapseSnap(
      requested,
      MIN_BOTTOM_PANEL_HEIGHT,
      maximum,
    );
    return snapped === 0 ? 0 : alignBottomPanelHeightToDisplay(snapped, maximum);
  }

  function animateCollapseBoundary(
    panel: ResizableOuterPanel,
    currentSize: number,
    nextSize: number,
  ): void {
    if ((currentSize === 0) === (nextSize === 0)) return;
    const activeTimer = snapTransitionTimers[panel];
    if (activeTimer !== undefined) window.clearTimeout(activeTimer);
    snapTransitioning[panel] = true;
    snapTransitionTimers[panel] = window.setTimeout(() => {
      snapTransitioning[panel] = false;
      delete snapTransitionTimers[panel];
    }, PANEL_TRANSITION_MS);
  }

  function beginInspectorResize(event: PointerEvent): void {
    if (event.button !== 0) return;
    event.preventDefault();
    refreshWorkspacePixelGeometry();
    if (inspectorResizeEndFrame !== null) window.cancelAnimationFrame(inspectorResizeEndFrame);
    inspectorResizeEndFrame = null;
    resizingInspector = true;
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.focus();
    target.setPointerCapture(event.pointerId);
    let pendingWidth = startWidth;
    const move = (moveEvent: PointerEvent) => {
      const nextWidth = inspectorWidthFromPointer(startWidth + startX - moveEvent.clientX);
      animateCollapseBoundary("inspector", pendingWidth, nextWidth);
      pendingWidth = nextWidth;
      if (inspectorResizeFrame !== null) return;
      inspectorResizeFrame = window.requestAnimationFrame(() => {
        inspectorWidth = pendingWidth;
        inspectorResizeFrame = null;
      });
    };
    const end = (endEvent: PointerEvent) => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (inspectorResizeFrame !== null) window.cancelAnimationFrame(inspectorResizeFrame);
      inspectorResizeFrame = null;
      if (endEvent.type === "pointercancel") {
        inspectorWidth = startWidth;
      } else if (pendingWidth === 0) {
        inspectorWidth = startWidth;
        chat.inspectorOpen = false;
      } else {
        inspectorWidth = pendingWidth;
        if (pendingWidth !== startWidth) persistPanelWidths();
      }
      inspectorResizeEndFrame = window.requestAnimationFrame(() => {
        inspectorResizeEndFrame = window.requestAnimationFrame(() => {
          resizingInspector = false;
          inspectorResizeEndFrame = null;
        });
      });
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function beginBottomPanelResize(event: PointerEvent): void {
    if (event.button !== 0) return;
    event.preventDefault();
    refreshWorkspacePixelGeometry();
    if (bottomResizeEndFrame !== null) window.cancelAnimationFrame(bottomResizeEndFrame);
    bottomResizeEndFrame = null;
    resizingBottomPanel = true;
    const startY = event.clientY;
    const startHeight = bottomPanelHeight;
    const target = event.currentTarget as HTMLElement;
    let pendingHeight = startHeight;
    target.focus();
    target.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => {
      const nextHeight = bottomPanelHeightFromPointer(startHeight + startY - moveEvent.clientY);
      animateCollapseBoundary("bottom", pendingHeight, nextHeight);
      pendingHeight = nextHeight;
      if (bottomResizeFrame !== null) return;
      bottomResizeFrame = window.requestAnimationFrame(() => {
        bottomPanelHeight = pendingHeight;
        bottomResizeFrame = null;
      });
    };
    const end = (endEvent: PointerEvent) => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (bottomResizeFrame !== null) window.cancelAnimationFrame(bottomResizeFrame);
      bottomResizeFrame = null;
      if (endEvent.type === "pointercancel") {
        bottomPanelHeight = startHeight;
      } else if (pendingHeight === 0) {
        bottomPanelHeight = startHeight;
        closeBottomPanel(true);
      } else {
        bottomPanelHeight = pendingHeight;
      }
      bottomResizeEndFrame = window.requestAnimationFrame(() => {
        bottomResizeEndFrame = window.requestAnimationFrame(() => {
          resizingBottomPanel = false;
          bottomResizeEndFrame = null;
        });
      });
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
  }

  function resizeInspectorFromKey(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      fitInspectorToAvailableSpace();
      return;
    }
    const next = panelWidthFromKey({
      current: inspectorWidth,
      minimum: MIN_INSPECTOR_WIDTH,
      maximum: inspectorResizeMaximum(),
      defaultValue: DEFAULT_INSPECTOR_WIDTH,
      step: 16,
      direction: "reversed",
      key: event.key,
    });
    if (next === null) return;
    event.preventDefault();
    refreshWorkspacePixelGeometry();
    inspectorWidth = alignInspectorWidthToDisplay(next);
    void persistPanelWidths();
  }

  function inspectorResizeMaximum(): number {
    if (layout.inspectorPresentation !== "column") return MAX_INSPECTOR_WIDTH;
    return chatInspectorResizeMaximum({
      containerWidth: shellWidth,
      railVisible: layout.railPresentation === "column" && chat.railOpen,
      fontScale,
      minimum: MIN_INSPECTOR_WIDTH,
      maximum: MAX_INSPECTOR_WIDTH,
    });
  }

  function fitInspectorToAvailableSpace(): void {
    refreshWorkspacePixelGeometry();
    const fitted = fittedChatInspectorWidth({
      containerWidth: shellWidth,
      railVisible: layout.railPresentation === "column" && chat.railOpen,
      fontScale,
      minimum: MIN_INSPECTOR_WIDTH,
      maximum: MAX_INSPECTOR_WIDTH,
    });
    inspectorWidth = alignInspectorWidthToDisplay(fitted);
    persistPanelWidths();
  }

  function fitBottomPanelToAvailableSpace(): void {
    refreshWorkspacePixelGeometry();
    const fitted = fittedChatBottomPanelHeight({
      containerHeight: shellHeight,
      fontScale,
      minimum: MIN_BOTTOM_PANEL_HEIGHT,
      maximum: MAX_BOTTOM_PANEL_HEIGHT,
    });
    bottomPanelHeight = alignBottomPanelHeightToDisplay(fitted);
  }

  function toggleBottomPanel(): void {
    bottomPanelSkipCloseTransition = false;
    if (bottomPanelOpen) {
      bottomPanelOpen = false;
      return;
    }
    bottomPanelMounted = true;
    bottomPanelOpen = true;
  }

  function closeBottomPanel(skipTransition = false): void {
    bottomPanelSkipCloseTransition = skipTransition;
    bottomPanelOpen = false;
  }

  function resizeBottomPanelFromKey(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      fitBottomPanelToAvailableSpace();
      return;
    }
    const maximum = bottomPanelResizeMaximum();
    let next: number;
    switch (event.key) {
      case "ArrowUp": next = bottomPanelHeight + 16; break;
      case "ArrowDown": next = bottomPanelHeight - 16; break;
      case "Home": next = MIN_BOTTOM_PANEL_HEIGHT; break;
      case "End": next = maximum; break;
      default: return;
    }
    event.preventDefault();
    refreshWorkspacePixelGeometry();
    bottomPanelHeight = alignBottomPanelHeightToDisplay(next, maximum);
  }

  function bottomPanelResizeMaximum(): number {
    return chatBottomPanelResizeMaximum({
      containerHeight: shellHeight,
      fontScale,
      minimum: MIN_BOTTOM_PANEL_HEIGHT,
      maximum: MAX_BOTTOM_PANEL_HEIGHT,
    });
  }

  function persistPanelWidths(): void {
    if (!chat.settings) return;
    pendingPanelWidths = {
      inspectorWidthPx: Math.round(inspectorWidth),
    };
    panelWidthSave ??= savePendingPanelWidths();
  }

  async function savePendingPanelWidths(): Promise<void> {
    layoutError = null;
    try {
      const { updateChatPanels } = await import("$lib/api/chat");
      while (pendingPanelWidths) {
        const savedWidths = pendingPanelWidths;
        await updateChatPanels(savedWidths);
        await chat.refreshSettings();
        if (pendingPanelWidths !== savedWidths) continue;
        inspectorUsesPromotedDefault = false;
        inspectorWidth = alignInspectorWidthToDisplay(savedWidths.inspectorWidthPx);
        pendingPanelWidths = null;
      }
    } catch (error: unknown) {
      pendingPanelWidths = null;
      layoutError = error instanceof Error ? error.message : String(error);
    } finally {
      panelWidthSave = null;
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

<div bind:this={rootElement} class="chat-workspace @container/chat-shell relative grid h-full min-h-0 overflow-hidden" class:resizing-panels={resizingInspector || resizingBottomPanel} class:panel-transitions-enabled={panelTransitionsEnabled} data-chat-workspace data-layout={layout.variant} data-rail-presentation={layout.railPresentation} data-inspector-presentation={layout.inspectorPresentation} data-active-surface={layout.activeSurface} data-rail-open={chat.railOpen} style={`background-color:var(--cal-bg);container-type:inline-size;container-name:chat-shell;--chat-panel-transition-duration:${PANEL_TRANSITION_MS}ms;--chat-bottom-min-height:${MIN_BOTTOM_PANEL_HEIGHT}px;--chat-inspector-width:${inspectorWidth}px;`}>
  <ChatWorkspaceObserver />
  <div class="sr-only" aria-live="polite" aria-atomic="true">{politeAnnouncement}</div>
  <div class="sr-only" aria-live="assertive" aria-atomic="true">{assertiveAnnouncement}</div>
  {#if layoutError}<div role="alert" class="absolute inset-x-2 top-2 z-50 rounded border border-destructive/40 bg-background p-2 text-xs text-destructive">{layoutError}</div>{/if}
  {#if layout.inspectorPresentation === "sheet" && chat.inspectorOpen}
    <button type="button" class="chat-sheet-backdrop chat-inspector-backdrop" aria-label={t("chat.closeInspector")} onclick={() => { chat.inspectorOpen = false; }}></button>
  {:else if layout.railPresentation === "sheet" && chat.railOpen}
    <button type="button" class="chat-sheet-backdrop chat-rail-backdrop" aria-label={t("chat.collapseRail")} onclick={() => { chat.railOpen = false; }}></button>
  {/if}
  <div bind:this={primaryHeaderElement} class="chat-primary-header">
    <ChatWorkspaceHeader
      bind:editingTitle={headerEditingTitle}
      explorerExpanded={layout.railPresentation === "column" && chat.railOpen}
      showRailButton={layout.railPresentation === "sheet" && !chat.railOpen}
      reserveGlobalActions={!chat.inspectorOpen}
      onOpenRail={() => { chat.railOpen = true; }}
    />
  </div>
  <div bind:this={railShell} class="chat-rail-shell" class:closed={!chat.railOpen} role={layout.railPresentation === "sheet" && chat.railOpen ? "dialog" : undefined} aria-modal={layout.railPresentation === "sheet" && chat.railOpen ? "true" : undefined} aria-label={layout.railPresentation === "sheet" && chat.railOpen ? t("chat.title") : undefined} onkeydown={(event) => { if (layout.railPresentation === "sheet") handleSheetKeydown(event, () => { chat.railOpen = false; }); }}>
    <ChatChannelRail expanded={chat.railOpen} showCollapsedStrip={layout.railPresentation === "column"} onExpand={() => { chat.railOpen = true; }} onCollapse={() => { chat.railOpen = false; }} />
  </div>
  <main class="main-shell relative flex min-w-0 flex-col">
        {#if loadError}
          <div role="alert" class="m-auto max-w-md p-4 text-center text-sm text-destructive">{loadError}<div><button type="button" class="chat-secondary-button mt-3" onclick={() => { loadError = null; void chat.reload().catch((error) => { loadError = error instanceof Error ? error.message : String(error); }); }}>{t("common.retry")}</button></div></div>
        {:else if chat.channelArchiveOpen}
          <section class="flex min-w-0 flex-1 flex-col overflow-hidden">
            <div class="flex shrink-0 items-center gap-2 border-b border-border px-4 py-3 sm:px-6">
              <button type="button" class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground" aria-label={t("chat.channels.backToChannel")} onclick={() => chat.closeChannelArchive()}><ArrowLeft size={16} /></button>
              <Archive size={16} class="text-muted-foreground" />
              <h2 class="min-w-0 flex-1 truncate text-[1.05rem] font-semibold">{t("chat.channels.archive")}</h2>
            </div>
            <div class="shrink-0 px-4 py-3 sm:px-6">
              <label class="flex max-w-xl items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5"><Search size={16} class="text-muted-foreground" /><input class="min-w-0 flex-1 bg-transparent text-[0.866667rem] outline-none" type="search" bind:value={archiveQuery} placeholder={t("chat.channels.searchArchive")} aria-label={t("chat.channels.searchArchive")} /></label>
            </div>
            <div class="min-h-0 flex-1 overflow-auto px-4 pb-5 sm:px-6">
              {#if archiveError}<p class="py-2 text-sm text-destructive" role="alert">{archiveError}</p>{/if}
              {#if chat.archivedChannelsError}<p class="py-2 text-sm text-destructive" role="alert">{chat.archivedChannelsError}</p>{/if}
              {#if chat.archivedChannelsLoading}
                <p class="py-2 text-sm text-muted-foreground" role="status">{t("common.loading")}</p>
              {:else if filteredArchivedChannels.length === 0}
                <p class="py-2 text-sm text-muted-foreground">{archiveQuery.trim() ? t("chat.channels.noArchiveResults") : t("chat.channels.emptyArchive")}</p>
              {:else}
                <div class="flex max-w-3xl flex-col gap-1">
                  {#each filteredArchivedChannels as channel (channel.id)}
                    <div class="flex min-w-0 flex-wrap items-start gap-3 rounded-md px-2 py-2 hover:bg-accent/70">
                      <Hash size={16} class="mt-0.5 shrink-0 text-muted-foreground" />
                      <div class="min-w-32 flex-1"><div class="truncate text-sm font-medium">{channel.name}</div><div class="mt-0.5 truncate text-xs text-muted-foreground">{channel.topic || t("chat.channels.noTopic")} · {formatDateTime(localization.locale, Date.parse(channel.archivedAt ?? channel.updatedAt), { dateStyle: "medium", timeStyle: "short" })}</div></div>
                      <button type="button" class="flex shrink-0 items-center gap-1.5 rounded-md border border-border bg-background px-2 py-1.5 text-xs hover:bg-accent disabled:opacity-60" disabled={archiveRestoringId === channel.id} onclick={() => void restoreArchivedChannel(channel)}><ArchiveRestore size={15} />{t("chat.restore")}</button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          </section>
        {:else if chat.selectedChannel}
          <div class="chat-conversation-shell">
            <ChatTimeline bottomInsetPx={composerDockHeight} />
            {#if chat.timelineItems.length === 0 && !chat.timelineLoading}
              <div class="pointer-events-none absolute inset-x-0 top-[18%] z-1 mx-auto flex max-w-lg flex-col items-center px-6 text-center">
                <span class="grid size-11 place-items-center rounded-xl bg-accent text-muted-foreground"><Hash size={21} /></span>
                <h2 class="mt-3 text-base font-semibold">{t("chat.channels.welcomeTitle", chat.selectedChannel.name)}</h2>
                <p class="mt-1 text-sm leading-relaxed text-muted-foreground">{chat.selectedChannel.topic || t("chat.channels.welcomeDescription")}</p>
              </div>
            {/if}
            {#if !chat.selectedChannel.archivedAt}
              <div bind:this={composerDockElement} class="chat-composer-dock">
                <div class="chat-composer-backdrop" aria-hidden="true"></div>
                <ChatComposer />
              </div>
            {/if}
          </div>
        {:else if !chat.loading && !chat.channelsLoading}
          <ChatFirstUse />
        {:else if initialLoadingVisible || chat.channelsLoading}
          <div class="m-auto text-sm text-muted-foreground" role="status">{t("common.loading")}</div>
        {/if}
  </main>

  <div class="chat-panel-separator chat-inspector-separator" class:hidden={!chat.inspectorOpen || layout.inspectorPresentation !== "column"} class:active={resizingInspector}><input type="range" min={MIN_INSPECTOR_WIDTH} max={inspectorResizeMaximum()} step="any" value={inspectorWidth} aria-label={t("chat.resizeInspector")} onpointerdown={beginInspectorResize} onkeydown={resizeInspectorFromKey} ondblclick={(event) => { event.preventDefault(); fitInspectorToAvailableSpace(); }} /><span class="chat-panel-separator-line" aria-hidden="true"></span></div>
  <aside bind:this={inspectorShell} class="chat-inspector-shell" class:open={chat.inspectorOpen} class:resizing={resizingInspector} class:snap-transition={snapTransitioning.inspector} data-presentation={layout.inspectorPresentation} inert={!chat.inspectorOpen} role={layout.inspectorPresentation === "sheet" && chat.inspectorOpen ? "dialog" : undefined} aria-modal={layout.inspectorPresentation === "sheet" && chat.inspectorOpen ? "true" : undefined} aria-label={t("chat.openInspector")} onkeydown={(event) => { if (layout.inspectorPresentation === "sheet") handleSheetKeydown(event, () => { chat.inspectorOpen = false; }); }}>
    <div class="chat-inspector-content-shell">
      <ChatWorkspacePanel placement="inspector" visible={chat.inspectorOpen} onClose={() => { chat.inspectorOpen = false; }} />
    </div>
  </aside>

  <div bind:this={globalActionsElement} class="chat-global-actions">
    <ChatHeaderActions {bottomPanelOpen} onToggleBottomPanel={toggleBottomPanel} onRename={() => { headerEditingTitle = true; }} />
  </div>

  {#if bottomPanelMounted}
    <div class="chat-panel-separator chat-bottom-separator" class:hidden={!bottomPanelVisible} class:active={resizingBottomPanel}><input type="range" min={MIN_BOTTOM_PANEL_HEIGHT} max={bottomPanelResizeMaximum()} step="any" value={bottomPanelHeight} aria-label={t("chat.resizeBottomPanel")} onpointerdown={beginBottomPanelResize} onkeydown={resizeBottomPanelFromKey} ondblclick={(event) => { event.preventDefault(); fitBottomPanelToAvailableSpace(); }} /><span class="chat-panel-separator-line" aria-hidden="true"></span></div>
    <div class="chat-bottom-transition-shell" class:open={bottomPanelVisible} class:skip-transition={bottomPanelSkipCloseTransition} class:snap-transition={snapTransitioning.bottom} inert={!bottomPanelVisible} style={`--chat-bottom-height:${bottomPanelHeight}px`} in:slide={{ duration: reducedMotion ? 0 : PANEL_TRANSITION_MS, easing: quintOut }}>
      <div class="chat-bottom-shell"><ChatWorkspacePanel placement="bottom" visible={bottomPanelVisible} onClose={() => { closeBottomPanel(); }} /></div>
    </div>
  {/if}

  {#if commandMenuOpen}
    <div class="absolute inset-0 z-50 flex items-start justify-center bg-black/30 p-3 pt-[10vh]">
      <button type="button" class="absolute inset-0" aria-label={t("chat.commandMenu.close")} onclick={() => { commandMenuOpen = false; }}></button>
      <div bind:this={commandDialog} class="relative w-full max-w-md rounded-lg border border-border bg-popover p-2 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.commandMenu.title")} tabindex="-1" onkeydown={(event) => trapFocus(event)}>
        <div class="flex items-center gap-2 border-b border-border px-2 py-2 text-xs text-muted-foreground"><Command size={14} />{t("chat.commandMenu.title")}</div>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.railOpen = true; window.dispatchEvent(new Event("ganbaru-ai:chat-new-channel")); }}><MessageSquarePlus size={14} />{t("chat.channels.createTitle")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.railOpen = true; window.dispatchEvent(new Event("ganbaru-ai:chat-focus-search")); }}><Search size={14} />{t("chat.search")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; toggleBottomPanel(); }}><PanelBottom size={14} />{bottomPanelOpen ? t("chat.closeBottomPanel") : t("chat.openBottomPanel")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.inspectorOpen = !chat.inspectorOpen; }}><PanelRight size={14} />{t("chat.openInspector")}</button>
        <button type="button" class="chat-command" onclick={() => { void openSettingsFromCommandMenu(); }}><Settings size={14} />{t("chat.settings")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-workspace {
    --chat-conversation-font-size: 0.933333rem;
    --chat-conversation-line-height: 1.4rem;
    --chat-rail-column-width: 2.75rem;
    --chat-inspector-column-width: 0px;
    --chat-global-actions-width: 6.5rem;
    grid-template-columns: var(--chat-rail-column-width) minmax(0, 1fr) 0 var(--chat-inspector-column-width);
    grid-template-rows: var(--cal-header-row-h) minmax(0, 1fr) auto auto;
  }
  .chat-workspace[data-rail-presentation="column"][data-rail-open="true"] { --chat-rail-column-width: 16rem; }
  .chat-workspace[data-rail-presentation="sheet"] { --chat-rail-column-width: 0px; }
  .chat-workspace[data-inspector-presentation="column"] { --chat-inspector-column-width: var(--chat-inspector-width); }
  .chat-primary-header { grid-column: 1 / 3; grid-row: 1; min-width: 0; }
  .chat-global-actions { position: absolute; top: 0; right: 0; z-index: 5; display: flex; width: max-content; min-width: var(--chat-global-actions-width); height: var(--cal-header-row-h); align-items: center; justify-content: flex-end; border-bottom: 1px solid var(--sidebar); background: var(--cal-header-bg); padding-right: 0.75rem; }
  .chat-rail-shell { grid-column: 1; grid-row: 2; min-width: 0; overflow: hidden; }
  .chat-workspace[data-rail-presentation="column"][data-rail-open="true"] .chat-rail-shell { grid-row: 2 / 5; }
  .main-shell { grid-column: 2; grid-row: 2; min-width: min(440px, 100cqw); min-height: 0; background: var(--cal-bg); }
  .chat-inspector-separator { grid-column: 3; grid-row: 1 / 3; }
  .chat-inspector-shell { grid-column: 4; grid-row: 1 / 3; min-width: 0; overflow: hidden; background: var(--cal-bg); }
  .chat-bottom-separator { grid-column: 1 / 5; grid-row: 3; }
  .chat-bottom-transition-shell { grid-column: 1 / 5; grid-row: 4; }
  .chat-workspace[data-rail-presentation="column"][data-rail-open="true"] .chat-bottom-separator,
  .chat-workspace[data-rail-presentation="column"][data-rail-open="true"] .chat-bottom-transition-shell { grid-column-start: 2; }
  .chat-workspace.panel-transitions-enabled { transition: grid-template-columns var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-workspace.panel-transitions-enabled .chat-rail-shell { transition: transform var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-panel-separator { position: relative; z-index: 1; background: transparent; }
  .chat-panel-separator-line { --chat-divider-highlight: color-mix(in srgb, var(--ring) 55%, var(--border)); position: absolute; pointer-events: none; background: var(--border); }
  .chat-panel-separator input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; appearance: none; touch-action: none; cursor: inherit; opacity: 0; }
  .chat-inspector-separator { width: 8px; min-width: 8px; margin-inline: -4px; cursor: col-resize; }
  .chat-inspector-separator::before { position: absolute; inset: 0 0 auto; height: var(--cal-header-row-h); border-bottom: 1px solid var(--sidebar); background: var(--cal-header-bg); content: ""; pointer-events: none; }
  .chat-inspector-separator .chat-panel-separator-line { inset-block: 0; left: 50%; width: 1px; }
  .chat-inspector-separator:is(:hover, .active) .chat-panel-separator-line, .chat-inspector-separator input:focus-visible + .chat-panel-separator-line { background: linear-gradient(to bottom, var(--border), var(--chat-divider-highlight) 50%, var(--border)); }
  .chat-inspector-shell.open.resizing { min-width: 0; }
  .chat-inspector-content-shell { width: var(--chat-inspector-width); height: 100%; }
  .chat-bottom-separator { width: 100%; height: 8px; min-height: 8px; margin-block: -4px; cursor: row-resize; }
  .chat-bottom-separator .chat-panel-separator-line { inset-inline: 0; top: 50%; height: 1px; }
  .chat-bottom-separator:is(:hover, .active) .chat-panel-separator-line, .chat-bottom-separator input:focus-visible + .chat-panel-separator-line { background: linear-gradient(to right, var(--border), var(--chat-divider-highlight) 50%, var(--border)); }
  .chat-bottom-transition-shell { height: 0; min-height: 0; overflow: hidden; }
  .chat-workspace.panel-transitions-enabled .chat-bottom-transition-shell { transition: height var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-bottom-transition-shell.open { height: var(--chat-bottom-height); }
  .chat-bottom-transition-shell.skip-transition { transition: none; }
  .chat-bottom-shell { height: max(var(--chat-bottom-height), var(--chat-bottom-min-height)); min-height: 0; overflow: hidden; }
  .chat-workspace.resizing-panels, .chat-workspace.resizing-panels * { user-select: none; }
  .chat-workspace.resizing-panels, .chat-workspace.resizing-panels .chat-bottom-transition-shell { transition: none; }
  .chat-bottom-transition-shell.snap-transition { transition: height var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-sheet-backdrop { position: absolute; inset: 0; z-index: 30; background: rgb(0 0 0 / 0.28); }
  .chat-rail-backdrop { top: var(--cal-header-row-h); }
  .chat-conversation-shell { --chat-scrollbar-gutter: 8px; position: relative; display: flex; min-height: 0; flex: 1; flex-direction: column; overflow: hidden; }
  .chat-composer-dock { pointer-events: none; position: absolute; inset-inline: 0; bottom: 0; z-index: 20; padding: 0.5rem 0.75rem 0.75rem; }
  .chat-composer-backdrop { position: absolute; inset: -1.5rem var(--chat-scrollbar-gutter) -2rem 0; background: linear-gradient(to bottom, transparent, color-mix(in srgb, var(--cal-bg) 72%, transparent) 35%, var(--cal-bg) 74%); -webkit-mask-image: linear-gradient(to bottom, transparent, black 35%); mask-image: linear-gradient(to bottom, transparent, black 35%); }
  .chat-composer-dock :global(.chat-composer) { pointer-events: auto; }
  .chat-command { display: flex; width: 100%; min-height: 2.25rem; align-items: center; gap: 0.5rem; border-radius: 0.375rem; padding: 0.375rem 0.5rem; font-size: 0.8rem; }
  .chat-command:hover { background: var(--accent); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell { position: absolute; top: var(--cal-header-row-h); bottom: 0; left: 0; z-index: 40; width: min(16rem, 88cqw); min-width: min(16rem, 88cqw); box-shadow: 8px 0 28px rgb(0 0 0 / 0.22); }
  .chat-workspace[data-rail-presentation="sheet"] .chat-rail-shell.closed { transform: translateX(-105%); }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell { position: absolute; inset-block: 0; right: 0; z-index: 45; width: 0; box-shadow: -8px 0 28px rgb(0 0 0 / 0.22); }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-shell.open { width: min(620px, 94cqw); min-width: min(320px, 94cqw); }
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-content-shell { width: min(620px, 94cqw); }
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell, .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell.closed, .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-shell.open { width: 100cqw; min-width: 0; box-shadow: none; }
  .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-content-shell { width: 100cqw; }
  .chat-workspace[data-layout="minimum_recovery"][data-active-surface="rail"] .main-shell, .chat-workspace[data-layout="minimum_recovery"][data-active-surface="inspector"] .main-shell { display: none; }
  @media (prefers-reduced-motion: reduce) {
    :global(.chat-workspace *), :global(.chat-workspace *::before), :global(.chat-workspace *::after) {
      scroll-behavior: auto !important;
      transition: none !important;
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
    }
  }
</style>
