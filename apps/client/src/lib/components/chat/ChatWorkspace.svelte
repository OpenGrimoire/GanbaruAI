<script lang="ts">
  import { onMount, tick } from "svelte";
  import { quintOut } from "svelte/easing";
  import { slide } from "svelte/transition";
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
    alignPanelSizeToDevicePixel,
    chatBottomPanelResizeMaximum,
    chatLayoutDecision,
    chatInspectorResizeMaximum,
    chatRailResizeMaximum,
    fittedChatBottomPanelHeight,
    fittedChatInspectorWidth,
    fittedChatRailWidth,
    panelSizeWithCollapseSnap,
    panelWidthFromKey,
    preferredPanelWidth,
    type ChatLayoutDecision,
  } from "$lib/chat/responsive-layout";
  import type { ChatPanelPreferences } from "$lib/chat/contracts";
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
  import ChatThreadRail from "./ChatThreadRail.svelte";
  import ChatTimeline from "./ChatTimeline.svelte";
  import ChatWorkspacePanel from "./ChatWorkspacePanel.svelte";
  import { getChatBenchmarkHandle } from "./benchmark-handle.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  const projects = getProjects();
  const settings = getSettingsLauncher();
  const LEGACY_RAIL_WIDTH = 260;
  const LEGACY_INSPECTOR_WIDTH = 360;
  const DEFAULT_RAIL_WIDTH = 320;
  const DEFAULT_INSPECTOR_WIDTH = 520;
  const MIN_RAIL_WIDTH = 160;
  const MAX_RAIL_WIDTH = 520;
  const MIN_INSPECTOR_WIDTH = 240;
  const MAX_INSPECTOR_WIDTH = 960;
  const DEFAULT_BOTTOM_PANEL_HEIGHT = 190;
  const MIN_BOTTOM_PANEL_HEIGHT = 96;
  const MAX_BOTTOM_PANEL_HEIGHT = 520;
  const PANEL_TRANSITION_MS = 190;
  const INITIAL_SHELL_WIDTH = 1_200;
  const INITIAL_SHELL_HEIGHT = 700;
  const INITIAL_FONT_SCALE = 1;
  const initialRailWidth = preferredPanelWidth(
    chat.settings?.configuration.panels.railWidthPx,
    LEGACY_RAIL_WIDTH,
    DEFAULT_RAIL_WIDTH,
  );
  const initialInspectorWidth = preferredPanelWidth(
    chat.settings?.configuration.panels.inspectorWidthPx,
    LEGACY_INSPECTOR_WIDTH,
    DEFAULT_INSPECTOR_WIDTH,
  );
  let rootElement: HTMLDivElement | undefined = $state();
  let railShell: HTMLDivElement | undefined = $state();
  let inspectorShell: HTMLElement | undefined = $state();
  let commandDialog: HTMLDivElement | undefined = $state();
  let commandMenuOpen = $state(false);
  let resizingRail = $state(false);
  let resizingInspector = $state(false);
  let resizingBottomPanel = $state(false);
  let railWidth = $state(initialRailWidth);
  let inspectorWidth = $state(initialInspectorWidth);
  let bottomPanelHeight = $state(DEFAULT_BOTTOM_PANEL_HEIGHT);
  let bottomPanelOpen = $state(false);
  let bottomPanelMounted = $state(false);
  let bottomPanelSkipCloseTransition = $state(false);
  let reducedMotion = $state(false);
  let inspectorMaximized = $state(false);
  let inspectorWasOpen = false;
  let inspectorReturnFocus: HTMLElement | null = null;
  let railModalWasOpen = false;
  let railReturnFocus: HTMLElement | null = null;
  let commandMenuWasOpen = false;
  let commandReturnFocus: HTMLElement | null = null;
  let shellWidth = $state(INITIAL_SHELL_WIDTH);
  let shellHeight = $state(INITIAL_SHELL_HEIGHT);
  let shellLeft = $state(0);
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
    railWidth: initialRailWidth,
    inspectorWidth: initialInspectorWidth,
  }));
  const bottomPanelVisible = $derived(
    bottomPanelOpen && !inspectorMaximized && layout.variant !== "minimum_recovery",
  );
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
  let railResizeEndFrame: number | null = null;
  let inspectorResizeEndFrame: number | null = null;
  let bottomResizeEndFrame: number | null = null;
  let panelTransitionFrame: number | null = null;
  let panelTransitionsEnabled = $state(false);
  type ResizableOuterPanel = "rail" | "inspector" | "bottom";
  let snapTransitioning = $state<Record<ResizableOuterPanel, boolean>>({
    rail: false,
    inspector: false,
    bottom: false,
  });
  const snapTransitionTimers: Partial<Record<ResizableOuterPanel, number>> = {};
  let panelPreferencesInitialized = false;
  let railUsesPromotedDefault = false;
  let inspectorUsesPromotedDefault = false;
  let pendingPanelWidths = $state<ChatPanelPreferences | null>(null);
  let panelWidthSave: Promise<void> | null = null;

  onMount(() => {
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotionPreference = () => { reducedMotion = motionQuery.matches; };
    updateMotionPreference();
    motionQuery.addEventListener("change", updateMotionPreference);
    void Promise.all([chat.ensureLoaded(), projects.ensureLoaded()]).catch((error) => {
      loadError = error instanceof Error ? error.message : String(error);
    });
    refreshWorkspacePixelGeometry();
    railWidth = alignRailWidthToDisplay(
      preferredPanelWidth(
        chat.settings?.configuration.panels.railWidthPx,
        LEGACY_RAIL_WIDTH,
        DEFAULT_RAIL_WIDTH,
      ),
    );
    inspectorWidth = alignInspectorWidthToDisplay(
      preferredPanelWidth(
        chat.settings?.configuration.panels.inspectorWidthPx,
        LEGACY_INSPECTOR_WIDTH,
        DEFAULT_INSPECTOR_WIDTH,
      ),
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
      if (railResizeEndFrame !== null) window.cancelAnimationFrame(railResizeEndFrame);
      if (inspectorResizeEndFrame !== null) window.cancelAnimationFrame(inspectorResizeEndFrame);
      if (bottomResizeEndFrame !== null) window.cancelAnimationFrame(bottomResizeEndFrame);
      if (panelTransitionFrame !== null) window.cancelAnimationFrame(panelTransitionFrame);
      for (const timer of Object.values(snapTransitionTimers)) {
        if (timer !== undefined) window.clearTimeout(timer);
      }
      motionQuery.removeEventListener("change", updateMotionPreference);
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
    if (!chat.settings) return;
    const configuredRailWidth = chat.settings.configuration.panels.railWidthPx;
    const configuredInspectorWidth = chat.settings.configuration.panels.inspectorWidthPx;
    if (!panelPreferencesInitialized) {
      railUsesPromotedDefault = configuredRailWidth === LEGACY_RAIL_WIDTH;
      inspectorUsesPromotedDefault = configuredInspectorWidth === LEGACY_INSPECTOR_WIDTH;
      panelPreferencesInitialized = true;
    }
    if (configuredRailWidth !== LEGACY_RAIL_WIDTH) railUsesPromotedDefault = false;
    if (configuredInspectorWidth !== LEGACY_INSPECTOR_WIDTH) inspectorUsesPromotedDefault = false;
    if (!resizingRail && !pendingPanelWidths) {
      railWidth = alignRailWidthToDisplay(
        railUsesPromotedDefault ? DEFAULT_RAIL_WIDTH : configuredRailWidth,
      );
    }
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
      railWidth,
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

  function refreshWorkspacePixelGeometry(): void {
    if (!rootElement) return;
    const bounds = rootElement.getBoundingClientRect();
    shellWidth = bounds.width;
    shellHeight = bounds.height;
    shellLeft = bounds.left;
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

  function alignRailWidthToDisplay(
    value: number,
    maximum = railResizeMaximum(),
  ): number {
    return alignPanelSizeToDevicePixel({
      value,
      minimum: MIN_RAIL_WIDTH,
      maximum,
      anchor: shellLeft,
      direction: "from-start",
      devicePixelRatio: displayPixelRatio,
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

  function railWidthFromPointer(requested: number): number {
    const maximum = railResizeMaximum();
    const snapped = panelSizeWithCollapseSnap(
      requested,
      MIN_RAIL_WIDTH,
      maximum,
    );
    return snapped === 0 ? 0 : alignRailWidthToDisplay(snapped, maximum);
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

  function beginRailResize(event: PointerEvent): void {
    if (event.button !== 0) return;
    event.preventDefault();
    refreshWorkspacePixelGeometry();
    if (railResizeEndFrame !== null) window.cancelAnimationFrame(railResizeEndFrame);
    railResizeEndFrame = null;
    resizingRail = true;
    const startX = event.clientX;
    const startWidth = railWidth;
    const target = event.currentTarget as HTMLElement;
    target.focus();
    target.setPointerCapture(event.pointerId);
    let pendingWidth = startWidth;
    const move = (moveEvent: PointerEvent) => {
      const nextWidth = railWidthFromPointer(startWidth + moveEvent.clientX - startX);
      animateCollapseBoundary("rail", pendingWidth, nextWidth);
      pendingWidth = nextWidth;
      if (railResizeFrame !== null) return;
      railResizeFrame = window.requestAnimationFrame(() => {
        railWidth = pendingWidth;
        railResizeFrame = null;
      });
    };
    const end = (endEvent: PointerEvent) => {
      target.removeEventListener("pointermove", move);
      target.removeEventListener("pointerup", end);
      target.removeEventListener("pointercancel", end);
      if (railResizeFrame !== null) window.cancelAnimationFrame(railResizeFrame);
      railResizeFrame = null;
      if (endEvent.type === "pointercancel") {
        railWidth = startWidth;
      } else if (pendingWidth === 0) {
        railWidth = startWidth;
        chat.railOpen = false;
      } else {
        railWidth = pendingWidth;
        if (pendingWidth !== startWidth) persistPanelWidths();
      }
      railResizeEndFrame = window.requestAnimationFrame(() => {
        railResizeEndFrame = window.requestAnimationFrame(() => {
          resizingRail = false;
          railResizeEndFrame = null;
        });
      });
    };
    target.addEventListener("pointermove", move);
    target.addEventListener("pointerup", end);
    target.addEventListener("pointercancel", end);
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

  function resizePanelFromKey(
    event: KeyboardEvent,
    panel: "rail" | "inspector",
  ): void {
    if (event.key === "Enter") {
      event.preventDefault();
      if (panel === "rail") fitRailToAvailableSpace();
      else fitInspectorToAvailableSpace();
      return;
    }
    const next = panelWidthFromKey(panel === "rail"
      ? {
          current: railWidth,
          minimum: MIN_RAIL_WIDTH,
          maximum: railResizeMaximum(),
          defaultValue: DEFAULT_RAIL_WIDTH,
          step: 16,
          direction: "standard",
          key: event.key,
        }
      : {
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
    if (panel === "rail") railWidth = alignRailWidthToDisplay(next);
    else inspectorWidth = alignInspectorWidthToDisplay(next);
    void persistPanelWidths();
  }

  function railResizeMaximum(): number {
    return chatRailResizeMaximum({
      containerWidth: shellWidth,
      inspectorVisible: layout.inspectorPresentation === "column" && chat.inspectorOpen,
      inspectorWidth,
      fontScale,
      minimum: MIN_RAIL_WIDTH,
      maximum: MAX_RAIL_WIDTH,
    });
  }

  function inspectorResizeMaximum(): number {
    if (layout.inspectorPresentation !== "column") return MAX_INSPECTOR_WIDTH;
    return chatInspectorResizeMaximum({
      containerWidth: shellWidth,
      railVisible: layout.railPresentation === "column" && chat.railOpen,
      railWidth,
      fontScale,
      minimum: MIN_INSPECTOR_WIDTH,
      maximum: MAX_INSPECTOR_WIDTH,
    });
  }

  function fitRailToAvailableSpace(): void {
    refreshWorkspacePixelGeometry();
    const fitted = fittedChatRailWidth({
      containerWidth: shellWidth,
      inspectorVisible: layout.inspectorPresentation === "column" && chat.inspectorOpen,
      inspectorWidth,
      fontScale,
      minimum: MIN_RAIL_WIDTH,
      maximum: MAX_RAIL_WIDTH,
    });
    railWidth = alignRailWidthToDisplay(fitted);
    persistPanelWidths();
  }

  function fitInspectorToAvailableSpace(): void {
    refreshWorkspacePixelGeometry();
    const fitted = fittedChatInspectorWidth({
      containerWidth: shellWidth,
      railVisible: layout.railPresentation === "column" && chat.railOpen,
      railWidth,
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
      railWidthPx: Math.round(railWidth),
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
        railUsesPromotedDefault = false;
        inspectorUsesPromotedDefault = false;
        railWidth = alignRailWidthToDisplay(savedWidths.railWidthPx);
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

<div bind:this={rootElement} class="chat-workspace @container/chat-shell relative flex h-full min-h-0 overflow-hidden" class:resizing-panels={resizingRail || resizingInspector || resizingBottomPanel} class:panel-transitions-enabled={panelTransitionsEnabled} data-chat-workspace data-layout={layout.variant} data-rail-presentation={layout.railPresentation} data-inspector-presentation={layout.inspectorPresentation} data-active-surface={layout.activeSurface} style={`background-color:var(--cal-bg);container-type:inline-size;container-name:chat-shell;--chat-panel-transition-duration:${PANEL_TRANSITION_MS}ms;--chat-bottom-min-height:${MIN_BOTTOM_PANEL_HEIGHT}px;`}>
  <div class="sr-only" aria-live="polite" aria-atomic="true">{politeAnnouncement}</div>
  <div class="sr-only" aria-live="assertive" aria-atomic="true">{assertiveAnnouncement}</div>
  {#if layoutError}<div role="alert" class="absolute inset-x-2 top-2 z-50 rounded border border-destructive/40 bg-background p-2 text-xs text-destructive">{layoutError}</div>{/if}
  {#if layout.inspectorPresentation === "sheet" && chat.inspectorOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.closeInspector")} onclick={() => { chat.inspectorOpen = false; }}></button>
  {:else if layout.railPresentation === "sheet" && chat.railOpen}
    <button type="button" class="chat-sheet-backdrop" aria-label={t("chat.collapseRail")} onclick={() => { chat.railOpen = false; }}></button>
  {/if}
  <div bind:this={railShell} class="chat-rail-shell" class:closed={!chat.railOpen} class:maximized-hidden={inspectorMaximized} class:snap-transition={snapTransitioning.rail} role={layout.railPresentation === "sheet" && chat.railOpen ? "dialog" : undefined} aria-modal={layout.railPresentation === "sheet" && chat.railOpen ? "true" : undefined} aria-label={layout.railPresentation === "sheet" && chat.railOpen ? t("chat.title") : undefined} onkeydown={(event) => { if (layout.railPresentation === "sheet") handleSheetKeydown(event, () => { chat.railOpen = false; }); }} style={`--chat-rail-width:${railWidth}px`}>
    <ChatThreadRail onCollapse={() => { chat.railOpen = false; }} />
  </div>
  <div
    class="chat-panel-separator chat-rail-separator"
    class:hidden={!chat.railOpen || inspectorMaximized}
    class:active={resizingRail}
  >
    <input
      type="range"
      min={MIN_RAIL_WIDTH}
      max={railResizeMaximum()}
      step="any"
      value={railWidth}
      aria-label={t("chat.resizeRail")}
      onpointerdown={beginRailResize}
      onkeydown={(event) => resizePanelFromKey(event, "rail")}
      ondblclick={(event) => { event.preventDefault(); fitRailToAvailableSpace(); }}
    />
    <span class="chat-panel-separator-line" aria-hidden="true"></span>
  </div>

  <div class="workspace-content" class:maximized={inspectorMaximized}>
    <div class="workspace-top">
      <main class="main-shell relative flex min-w-0 flex-1 flex-col" class:maximized-hidden={inspectorMaximized}>
        <ChatConversationHeader
          draft={!chat.selectedThread}
          showRailButton={layout.railPresentation === "sheet" || !chat.railOpen}
          onOpenRail={() => { chat.railOpen = true; }}
          {bottomPanelOpen}
          onToggleBottomPanel={toggleBottomPanel}
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

      <div class="chat-panel-separator chat-inspector-separator" class:hidden={!chat.inspectorOpen || inspectorMaximized} class:active={resizingInspector}><input type="range" min={MIN_INSPECTOR_WIDTH} max={inspectorResizeMaximum()} step="any" value={inspectorWidth} aria-label={t("chat.resizeInspector")} onpointerdown={beginInspectorResize} onkeydown={(event) => resizePanelFromKey(event, "inspector")} ondblclick={(event) => { event.preventDefault(); fitInspectorToAvailableSpace(); }} /><span class="chat-panel-separator-line" aria-hidden="true"></span></div>
      <aside bind:this={inspectorShell} class="chat-inspector-shell" class:open={chat.inspectorOpen} class:maximized={inspectorMaximized} class:resizing={resizingInspector} class:snap-transition={snapTransitioning.inspector} data-presentation={layout.inspectorPresentation} inert={!chat.inspectorOpen} role={layout.inspectorPresentation === "sheet" && chat.inspectorOpen ? "dialog" : undefined} aria-modal={layout.inspectorPresentation === "sheet" && chat.inspectorOpen ? "true" : undefined} aria-label={t("chat.openInspector")} onkeydown={(event) => { if (layout.inspectorPresentation === "sheet") handleSheetKeydown(event, () => { chat.inspectorOpen = false; }); }} style={`--chat-inspector-width:${inspectorWidth}px`}>
        <div class="chat-inspector-content-shell">
          <ChatWorkspacePanel placement="inspector" visible={chat.inspectorOpen} onClose={() => { chat.inspectorOpen = false; }} onMaximizedChange={(value) => { inspectorMaximized = value; }} />
        </div>
      </aside>
    </div>

    {#if bottomPanelMounted}
      <div class="chat-panel-separator chat-bottom-separator" class:hidden={!bottomPanelVisible} class:active={resizingBottomPanel}><input type="range" min={MIN_BOTTOM_PANEL_HEIGHT} max={bottomPanelResizeMaximum()} step="any" value={bottomPanelHeight} aria-label={t("chat.resizeBottomPanel")} onpointerdown={beginBottomPanelResize} onkeydown={resizeBottomPanelFromKey} ondblclick={(event) => { event.preventDefault(); fitBottomPanelToAvailableSpace(); }} /><span class="chat-panel-separator-line" aria-hidden="true"></span></div>
      <div
        class="chat-bottom-transition-shell"
        class:open={bottomPanelVisible}
        class:skip-transition={bottomPanelSkipCloseTransition}
        class:snap-transition={snapTransitioning.bottom}
        inert={!bottomPanelVisible}
        style={`--chat-bottom-height:${bottomPanelHeight}px`}
        in:slide={{ duration: reducedMotion ? 0 : PANEL_TRANSITION_MS, easing: quintOut }}
      >
        <div class="chat-bottom-shell">
          <ChatWorkspacePanel placement="bottom" visible={bottomPanelVisible} onClose={() => { closeBottomPanel(); }} />
        </div>
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
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; toggleBottomPanel(); }}><PanelBottom size={14} />{bottomPanelOpen ? t("chat.closeBottomPanel") : t("chat.openBottomPanel")}</button>
        <button type="button" class="chat-command" onclick={() => { commandMenuOpen = false; chat.inspectorOpen = !chat.inspectorOpen; }}><PanelRight size={14} />{t("chat.openInspector")}</button>
        <button type="button" class="chat-command" onclick={() => { void openSettingsFromCommandMenu(); }}><Settings size={14} />{t("chat.settings")}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .chat-rail-shell { width: var(--chat-rail-width); min-width: var(--chat-rail-width); overflow: hidden; }
  .chat-workspace.panel-transitions-enabled .chat-rail-shell { transition: width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1), min-width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1), transform var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-rail-shell.closed { width: 0; min-width: 0; overflow: hidden; }
  .maximized-hidden { display: none; }
  .chat-panel-separator { position: relative; z-index: 1; background: transparent; }
  .chat-panel-separator-line { --chat-divider-highlight: color-mix(in srgb, var(--ring) 55%, var(--border)); position: absolute; pointer-events: none; background: var(--border); }
  .chat-panel-separator input { position: absolute; inset: 0; width: 100%; height: 100%; touch-action: none; appearance: none; margin: 0; cursor: inherit; opacity: 0; }
  .chat-rail-separator, .chat-inspector-separator { width: 8px; min-width: 8px; flex: 0 0 8px; margin-inline: -4px; cursor: col-resize; }
  .chat-rail-separator .chat-panel-separator-line, .chat-inspector-separator .chat-panel-separator-line { inset-block: 0; left: 50%; width: 1px; }
  .chat-rail-separator:is(:hover, .active) .chat-panel-separator-line, .chat-rail-separator input:focus-visible + .chat-panel-separator-line,
  .chat-inspector-separator:is(:hover, .active) .chat-panel-separator-line, .chat-inspector-separator input:focus-visible + .chat-panel-separator-line { background: linear-gradient(to bottom, var(--border), var(--chat-divider-highlight) 50%, var(--border)); }
  .workspace-content { display: flex; min-width: 0; min-height: 0; flex: 1; flex-direction: column; }
  .workspace-top { position: relative; display: flex; min-width: 0; min-height: 0; flex: 1; }
  .chat-inspector-shell { width: 0; min-width: 0; overflow: hidden; background: var(--cal-bg); }
  .chat-workspace.panel-transitions-enabled .chat-inspector-shell { transition: width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1), min-width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-inspector-shell.open { width: var(--chat-inspector-width); min-width: min(240px, 46cqw); }
  .chat-inspector-shell.open.resizing { min-width: 0; }
  .chat-inspector-shell.maximized { width: 100%; min-width: 0; }
  .chat-inspector-content-shell { width: var(--chat-inspector-width); height: 100%; }
  .chat-inspector-shell.maximized .chat-inspector-content-shell { width: 100cqw; }
  .chat-bottom-separator { width: 100%; height: 8px; min-height: 8px; flex: 0 0 8px; margin-block: -4px; cursor: row-resize; }
  .chat-bottom-separator .chat-panel-separator-line { inset-inline: 0; top: 50%; height: 1px; }
  .chat-bottom-separator:is(:hover, .active) .chat-panel-separator-line, .chat-bottom-separator input:focus-visible + .chat-panel-separator-line { background: linear-gradient(to right, var(--border), var(--chat-divider-highlight) 50%, var(--border)); }
  .chat-bottom-transition-shell { height: 0; min-height: 0; flex: 0 0 auto; overflow: hidden; }
  .chat-workspace.panel-transitions-enabled .chat-bottom-transition-shell { transition: height var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-bottom-transition-shell.open { height: var(--chat-bottom-height); }
  .chat-bottom-transition-shell.skip-transition { transition: none; }
  .chat-bottom-shell { height: max(var(--chat-bottom-height), var(--chat-bottom-min-height)); min-height: 0; overflow: hidden; }
  .chat-workspace.resizing-panels, .chat-workspace.resizing-panels * { user-select: none; }
  .chat-workspace.resizing-panels .chat-rail-shell, .chat-workspace.resizing-panels .chat-inspector-shell, .chat-workspace.resizing-panels .chat-bottom-transition-shell { transition: none; }
  .chat-workspace.resizing-panels .chat-rail-shell.snap-transition { transition: width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1), min-width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-workspace.resizing-panels .chat-inspector-shell.snap-transition { transition: width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1), min-width var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-bottom-transition-shell.snap-transition { transition: height var(--chat-panel-transition-duration) cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-sheet-backdrop { position: absolute; inset: 0; z-index: 30; background: rgb(0 0 0 / 0.28); }
  .main-shell { min-width: min(440px, 100cqw); background: var(--cal-bg); }
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
  .chat-workspace[data-inspector-presentation="sheet"] .chat-inspector-content-shell { width: min(620px, 94cqw); }
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell,
  .chat-workspace[data-layout="minimum_recovery"] .chat-rail-shell.closed,
  .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-shell.open { width: 100cqw; min-width: 0; box-shadow: none; }
  .chat-workspace[data-layout="minimum_recovery"] .chat-inspector-content-shell { width: 100cqw; }
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
