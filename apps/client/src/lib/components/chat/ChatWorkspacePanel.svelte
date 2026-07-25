<script module lang="ts">
  import { ChatInspectorSessionState } from "$lib/chat/inspector-model";
  import { TerminalPanelRegistry } from "$lib/chat/terminal-model";

  const workspacePanelSessions = {
    inspector: new ChatInspectorSessionState("files"),
    bottom: new ChatInspectorSessionState("terminal"),
  };
  const terminalPanels = new TerminalPanelRegistry();
  const terminalLoadLocks = new Map<string, Promise<void>>();

  /** Serializes terminal discovery and creation within one thread and workspace. */
  async function withTerminalLoadLock<T>(scopeKey: string, operation: () => Promise<T>): Promise<T> {
    const previous = terminalLoadLocks.get(scopeKey) ?? Promise.resolve();
    let release: () => void = () => undefined;
    const current = new Promise<void>((resolve) => { release = resolve; });
    const queued = previous.then(() => current);
    terminalLoadLocks.set(scopeKey, queued);
    await previous;
    try {
      return await operation();
    } finally {
      release();
      if (terminalLoadLocks.get(scopeKey) === queued) terminalLoadLocks.delete(scopeKey);
    }
  }
</script>

<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import FileDiff from "@lucide/svelte/icons/file-diff";
  import Files from "@lucide/svelte/icons/files";
  import ListTodo from "@lucide/svelte/icons/list-todo";
  import Maximize2 from "@lucide/svelte/icons/maximize-2";
  import Minimize2 from "@lucide/svelte/icons/minimize-2";
  import Plus from "@lucide/svelte/icons/plus";
  import SquareTerminal from "@lucide/svelte/icons/square-terminal";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import type { ChatInspectorTab, ChatTerminalRead } from "$lib/chat/contracts";
  import {
    closeInspectorTab,
    inspectorSessionKey,
    moveWorkspacePanelTab,
    openInspectorTab,
    reconcileWorkspacePanelTabOrder,
    terminalWorkspacePanelTabKey,
    workspacePanelKinds,
    workspacePanelTabInsertionIndex,
    workspacePanelTabShift,
    workspacePanelTerminalId,
    type ChatInspectorThreadState,
    type ChatWorkspacePanelTabKey,
  } from "$lib/chat/inspector-model";
  import { terminalErrorMessage } from "$lib/chat/terminal-model";
  import {
    pickSelectPopoverGeometry,
    type SelectPopoverGeometry,
    type SelectPopoverRect,
  } from "$lib/components/settings/customSelectPosition";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { portal } from "$lib/utils/portal";
  import ChatChangesPanel from "./ChatChangesPanel.svelte";
  import ChatFileIcon from "./ChatFileIcon.svelte";
  import ChatFilesPanel from "./ChatFilesPanel.svelte";
  import ChatPlanPanel from "./ChatPlanPanel.svelte";
  import ChatTerminalView from "./ChatTerminalView.svelte";

  let {
    placement,
    visible = true,
    onClose,
    onMaximizedChange = () => undefined,
  }: {
    placement: "inspector" | "bottom";
    visible?: boolean;
    onClose: () => void;
    onMaximizedChange?: (maximized: boolean) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const DEFAULT_PICKER_GEOMETRY: SelectPopoverGeometry = {
    top: 0,
    left: 0,
    width: null,
    minWidth: 0,
    maxWidth: 0,
    maxHeight: 0,
    placement: "below",
  };
  let panelState = $state<ChatInspectorThreadState>(placementSession().read(null));
  let terminals: ChatTerminalRead[] = $state([]);
  let selectedTerminalId: string | null = $state(null);
  let terminalsLoading = $state(false);
  let panelPickerOpen = $state(false);
  let panelPickerReady = $state(false);
  let panelPickerTrigger: HTMLButtonElement | undefined = $state();
  let panelPicker: HTMLDivElement | undefined = $state();
  let panelPickerGeometry = $state<SelectPopoverGeometry>(DEFAULT_PICKER_GEOMETRY);
  let fadedTerminalIds: string[] = $state([]);
  let error: string | null = $state(null);
  let panelTabbar: HTMLDivElement | undefined = $state();
  let draggedTabKey: ChatWorkspacePanelTabKey | null = $state(null);
  let draggedTabOffsetX = $state(0);
  let dragTargetIndex = $state(0);
  let loadedKey: string | null = null;
  let terminalScopeKey = "";
  let tabDragListenersAttached = false;
  let tabDragGesture = $state<{
    key: ChatWorkspacePanelTabKey;
    pointerId: number;
    startClientX: number;
    order: ChatWorkspacePanelTabKey[];
    sourceIndex: number;
    sourceCenter: number;
    sourceSpan: number;
    active: boolean;
  } | null>(null);
  const threadId = $derived(chat.selectedThreadId ?? chat.draftThreadId);
  const workspaceId = $derived(chat.selectedWorkspaceId);
  const sessionKey = $derived(inspectorSessionKey(threadId, workspaceId));
  const selectedTerminal = $derived(terminals.find((terminal) => terminal.id === selectedTerminalId) ?? null);
  const panelTabs: {
    id: Exclude<ChatInspectorTab, "terminal">;
    label: "changes" | "plan" | "files";
    icon: typeof FileDiff;
  }[] = [
    { id: "changes", label: "changes", icon: FileDiff },
    { id: "plan", label: "plan", icon: ListTodo },
    { id: "files", label: "files", icon: Files },
  ];
  type WorkspacePanelRenderTab =
    | { key: ChatWorkspacePanelTabKey; type: "loading-terminal" }
    | { key: ChatWorkspacePanelTabKey; type: "terminal"; terminal: ChatTerminalRead }
    | { key: ChatWorkspacePanelTabKey; type: "panel"; panel: (typeof panelTabs)[number] };
  const orderedTabKeys = $derived(reconcileWorkspacePanelTabOrder(
    panelState.tabOrder,
    panelState.openTabs,
    terminals.map((terminal) => terminal.id),
  ));
  const orderedTabs = $derived.by<WorkspacePanelRenderTab[]>(() => {
    const tabs: WorkspacePanelRenderTab[] = [];
    for (const key of orderedTabKeys) {
      if (key === "terminal") {
        tabs.push({ key, type: "loading-terminal" });
        continue;
      }
      const terminalId = workspacePanelTerminalId(key);
      const terminal = terminalId
        ? terminals.find((candidate) => candidate.id === terminalId)
        : undefined;
      if (terminal) {
        tabs.push({ key, type: "terminal", terminal });
        continue;
      }
      const panel = panelForKey(key);
      if (panel) tabs.push({ key, type: "panel", panel });
    }
    return tabs;
  });

  function initialPanelTab(): "files" | "terminal" {
    return placement === "bottom" ? "terminal" : "files";
  }

  function placementSession(): ChatInspectorSessionState {
    return workspacePanelSessions[placement];
  }

  $effect(() => {
    const key = sessionKey;
    if (key === loadedKey) return;
    loadedKey = key;
    panelState = placementSession().read(key);
    onMaximizedChange(panelState.maximized);
  });

  $effect(() => {
    const terminalOpen = panelState.openTabs.includes("terminal");
    const thread = threadId;
    const workspace = workspaceId;
    const nextKey = `${thread ?? ""}:${workspace ?? ""}`;
    if (!terminalOpen || !thread || !workspace) {
      terminalScopeKey = "";
      terminals = [];
      selectedTerminalId = null;
      return;
    }
    if (!visible) {
      if (terminals.length === 0) terminalScopeKey = "";
      return;
    }
    if (terminalScopeKey === nextKey && (terminalsLoading || terminals.length > 0 || error)) return;
    terminalScopeKey = nextKey;
    void loadTerminals(thread, workspace, nextKey);
  });

  function update(updateValue: Partial<ChatInspectorThreadState>): void {
    const key = sessionKey;
    if (!key) return;
    panelState = placementSession().update(key, updateValue);
    if (updateValue.maximized !== undefined) onMaximizedChange(updateValue.maximized);
  }

  function openPanel(tab: ChatInspectorTab): void {
    const openTabs = openInspectorTab(panelState.openTabs, tab);
    const currentOrder = reconcileWorkspacePanelTabOrder(
      panelState.tabOrder,
      openTabs,
      terminals.map((terminal) => terminal.id),
    );
    update({
      tab,
      openTabs,
      tabOrder: currentOrder,
      ...(tab === "files" ? { fileTreeVisible: true } : {}),
    });
    panelPickerOpen = false;
  }

  function closePanel(tab: ChatInspectorTab): void {
    const currentOrder = orderedTabKeys;
    const remainingOrder = currentOrder.filter((key) => panelKind(key) !== tab);
    const next = closeInspectorTab(panelState.openTabs, tab, panelState.tab);
    if (next.tabs.length === 0) {
      onClose();
      return;
    }
    const selectedTab = panelState.tab === tab
      ? panelKind(remainingOrder[Math.min(
        Math.max(0, currentOrder.findIndex((key) => panelKind(key) === tab)),
        remainingOrder.length - 1,
      )] ?? remainingOrder[0] ?? initialPanelTab())
      : next.selectedTab ?? panelState.tab;
    update({
      openTabs: workspacePanelKinds(remainingOrder),
      tabOrder: remainingOrder,
      tab: selectedTab,
    });
  }

  function selectPanel(tab: ChatInspectorTab): void {
    update({ tab, ...(tab === "files" ? { fileTreeVisible: true } : {}) });
  }

  function panelLabel(tab: (typeof panelTabs)[number]): string {
    if (tab.id !== "files" || !panelState.filePreviewPath) return t(`chat.inspector.${tab.label}`);
    return panelState.filePreviewPath.split("/").filter(Boolean).at(-1) ?? t("chat.inspector.files");
  }

  function panelKind(key: ChatWorkspacePanelTabKey): ChatInspectorTab {
    return key === "changes" || key === "plan" || key === "files" ? key : "terminal";
  }

  function panelForKey(key: ChatWorkspacePanelTabKey): (typeof panelTabs)[number] | undefined {
    return panelTabs.find((candidate) => candidate.id === key);
  }

  async function loadTerminals(
    thread: string,
    workspace: string,
    scopeKey: string,
  ): Promise<void> {
    terminalsLoading = true;
    terminals = [];
    selectedTerminalId = null;
    error = null;
    try {
      const loaded = await withTerminalLoadLock(scopeKey, async () => {
        const available = await chatApi.listChatTerminals(thread, workspace);
        const owned = terminalPanels.claimAvailable(available, placement);
        if (owned.length > 0) return owned;
        const snapshot = await createTerminal(thread, workspace);
        return [snapshot.terminal];
      });
      if (thread !== threadId || workspace !== workspaceId) return;
      terminals = loaded;
      const remembered = terminalPanels.selected(thread, placement);
      selectTerminal(loaded.some((terminal) => terminal.id === remembered)
        ? remembered ?? null
        : loaded[0]?.id ?? null);
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      if (thread === threadId && workspace === workspaceId) terminalsLoading = false;
    }
  }

  async function createTerminal(
    thread: string,
    workspace: string,
  ): ReturnType<typeof chatApi.createChatTerminal> {
    const snapshot = await chatApi.createChatTerminal({
      terminalId: crypto.randomUUID(),
      threadId: thread,
      workspaceId: workspace,
      columns: 80,
      rows: 24,
    });
    terminalPanels.assign(snapshot.terminal.id, placement);
    return snapshot;
  }

  async function addTerminal(): Promise<void> {
    if (!threadId || !workspaceId) return;
    panelPickerOpen = false;
    const snapshot = await createTerminal(threadId, workspaceId);
    const previousOrder = orderedTabKeys;
    terminals = [...terminals, snapshot.terminal];
    update({
      tabOrder: [
        ...previousOrder.filter((key) => key !== "terminal"),
        terminalWorkspacePanelTabKey(snapshot.terminal.id),
      ],
    });
    selectTerminal(snapshot.terminal.id);
    openPanel("terminal");
  }

  function retryTerminalLoad(): void {
    const thread = threadId;
    const workspace = workspaceId;
    if (!thread || !workspace) return;
    const scopeKey = `${thread}:${workspace}`;
    terminalScopeKey = scopeKey;
    void loadTerminals(thread, workspace, scopeKey);
  }

  function openOrAddTerminal(): void {
    if (panelState.openTabs.includes("terminal")) {
      void run(addTerminal);
      return;
    }
    openPanel("terminal");
  }

  function selectTerminal(terminalId: string | null): void {
    selectedTerminalId = terminalId;
    if (threadId) terminalPanels.select(threadId, placement, terminalId);
  }

  function updateTerminal(terminal: ChatTerminalRead): void {
    terminals = terminals.map((entry) => entry.id === terminal.id ? terminal : entry);
  }

  async function closeTerminal(terminal: ChatTerminalRead): Promise<void> {
    if (!threadId || !workspaceId) return;
    let result = await chatApi.closeChatTerminal(terminal.id, threadId, workspaceId, false);
    if (result.confirmationRequired) {
      if (!window.confirm(t("chat.inspector.confirmCloseTerminal"))) return;
      result = await chatApi.closeChatTerminal(terminal.id, threadId, workspaceId, true);
    }
    if (!result.closed) return;
    terminalPanels.release(terminal.id);
    const closedKey = terminalWorkspacePanelTabKey(terminal.id);
    const currentOrder = orderedTabKeys;
    const closedIndex = currentOrder.indexOf(closedKey);
    const remainingOrder = currentOrder.filter((key) => key !== closedKey);
    const next = terminals.filter((entry) => entry.id !== terminal.id);
    if (next.length === 0) {
      closePanel("terminal");
      terminals = next;
      return;
    }
    terminals = next;
    update({ tabOrder: remainingOrder });
    if (selectedTerminalId === terminal.id) {
      const nearestKey = remainingOrder[Math.min(
        Math.max(0, closedIndex),
        remainingOrder.length - 1,
      )] ?? remainingOrder[0];
      const nearestTerminalId = nearestKey ? workspacePanelTerminalId(nearestKey) : null;
      selectTerminal(nearestTerminalId ?? next[0]?.id ?? null);
      if (nearestKey) selectPanel(panelKind(nearestKey));
    }
  }

  function trackTerminalOverflow(node: HTMLElement, terminalId: string): { destroy: () => void } {
    const update = () => {
      const overflows = node.scrollWidth > node.clientWidth;
      const included = fadedTerminalIds.includes(terminalId);
      if (overflows !== included) {
        fadedTerminalIds = overflows
          ? [...fadedTerminalIds, terminalId]
          : fadedTerminalIds.filter((id) => id !== terminalId);
      }
    };
    const observer = new ResizeObserver(update);
    observer.observe(node);
    queueMicrotask(update);
    return {
      destroy: () => {
        observer.disconnect();
        fadedTerminalIds = fadedTerminalIds.filter((id) => id !== terminalId);
      },
    };
  }

  async function run(operation: () => Promise<void>): Promise<void> {
    error = null;
    try {
      await operation();
    } catch (reason: unknown) {
      error = message(reason);
    }
  }

  function handleTabKeydown(event: KeyboardEvent): void {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    const tabbar = (event.currentTarget as HTMLElement).closest<HTMLElement>("[role='tablist']");
    const buttons = [...(tabbar?.querySelectorAll<HTMLButtonElement>("[role='tab']") ?? [])];
    const index = buttons.indexOf(event.currentTarget as HTMLButtonElement);
    if (index < 0 || buttons.length === 0) return;
    event.preventDefault();
    const next = event.key === "Home"
      ? 0
      : event.key === "End"
        ? buttons.length - 1
        : (index + (event.key === "ArrowRight" ? 1 : -1) + buttons.length) % buttons.length;
    buttons[next]?.click();
    buttons[next]?.focus();
  }

  function beginTabDrag(event: PointerEvent, key: ChatWorkspacePanelTabKey): void {
    if (event.button !== 0) return;
    if (event.target instanceof Element && event.target.closest(".tab-close")) return;
    if (!panelTabbar) return;
    const order = [...orderedTabKeys];
    const sourceIndex = order.indexOf(key);
    if (sourceIndex < 0) return;
    const slot = event.currentTarget as HTMLElement;
    const computedGap = Number.parseFloat(getComputedStyle(panelTabbar).columnGap);
    activateWorkspacePanelTab(key);
    tabDragGesture = {
      key,
      pointerId: event.pointerId,
      startClientX: event.clientX,
      order,
      sourceIndex,
      sourceCenter: slot.offsetLeft + slot.offsetWidth / 2,
      sourceSpan: slot.offsetWidth + (Number.isFinite(computedGap) ? computedGap : 0),
      active: false,
    };
    dragTargetIndex = sourceIndex;
    attachTabDragListeners();
  }

  function moveTabDrag(event: PointerEvent): void {
    const gesture = tabDragGesture;
    if (!gesture || gesture.pointerId !== event.pointerId || !panelTabbar) return;
    const offsetX = event.clientX - gesture.startClientX;
    if (!gesture.active) {
      if (Math.abs(offsetX) < 5) return;
      gesture.active = true;
      draggedTabKey = gesture.key;
    }
    event.preventDefault();
    draggedTabOffsetX = offsetX;
    const remainingCenters = [...panelTabbar.querySelectorAll<HTMLElement>("[data-panel-tab-key]")]
      .filter((element) => element.dataset.panelTabKey !== gesture.key)
      .map((element) => element.offsetLeft + element.offsetWidth / 2);
    dragTargetIndex = workspacePanelTabInsertionIndex(
      gesture.sourceCenter + offsetX,
      remainingCenters,
    );
  }

  function tabDragShift(key: ChatWorkspacePanelTabKey): number {
    const gesture = tabDragGesture;
    if (!gesture?.active) return 0;
    if (key === gesture.key) return draggedTabOffsetX;
    return workspacePanelTabShift(
      gesture.order.indexOf(key),
      gesture.sourceIndex,
      dragTargetIndex,
      gesture.sourceSpan,
    );
  }

  function finishTabDrag(event: PointerEvent, commit: boolean): void {
    const gesture = tabDragGesture;
    if (!gesture || gesture.pointerId !== event.pointerId) return;
    const nextOrder = commit && gesture.active
      ? moveWorkspacePanelTab(gesture.order, gesture.key, dragTargetIndex)
      : null;
    resetTabDrag();
    if (nextOrder) update({ tabOrder: nextOrder, openTabs: workspacePanelKinds(nextOrder) });
  }

  function attachTabDragListeners(): void {
    if (tabDragListenersAttached) return;
    tabDragListenersAttached = true;
    window.addEventListener("pointermove", handleWindowTabPointerMove, true);
    window.addEventListener("pointerup", handleWindowTabPointerUp, true);
    window.addEventListener("pointercancel", handleWindowTabPointerCancel, true);
    window.addEventListener("blur", cancelTabDrag);
  }

  function detachTabDragListeners(): void {
    if (!tabDragListenersAttached) return;
    tabDragListenersAttached = false;
    window.removeEventListener("pointermove", handleWindowTabPointerMove, true);
    window.removeEventListener("pointerup", handleWindowTabPointerUp, true);
    window.removeEventListener("pointercancel", handleWindowTabPointerCancel, true);
    window.removeEventListener("blur", cancelTabDrag);
  }

  function handleWindowTabPointerMove(event: PointerEvent): void {
    moveTabDrag(event);
  }

  function handleWindowTabPointerUp(event: PointerEvent): void {
    finishTabDrag(event, true);
  }

  function handleWindowTabPointerCancel(event: PointerEvent): void {
    finishTabDrag(event, false);
  }

  function cancelTabDrag(): void {
    resetTabDrag();
  }

  function resetTabDrag(): void {
    detachTabDragListeners();
    tabDragGesture = null;
    draggedTabKey = null;
    draggedTabOffsetX = 0;
    dragTargetIndex = 0;
  }

  onDestroy(resetTabDrag);

  function activateWorkspacePanelTab(key: ChatWorkspacePanelTabKey): void {
    const terminalId = workspacePanelTerminalId(key);
    if (terminalId) selectTerminal(terminalId);
    selectPanel(panelKind(key));
  }

  function preventMiddleButtonScroll(event: MouseEvent): void {
    if (event.button === 1) event.preventDefault();
  }

  function closeTabFromAuxClick(
    event: MouseEvent,
    key: ChatWorkspacePanelTabKey,
    terminal?: ChatTerminalRead,
  ): void {
    if (event.button !== 1) return;
    event.preventDefault();
    event.stopPropagation();
    if (terminal) void run(() => closeTerminal(terminal));
    else closePanel(panelKind(key));
  }

  function message(reason: unknown): string {
    return terminalErrorMessage(
      reason,
      t("common.viewLoadFailed", t("chat.inspector.terminal")),
    );
  }

  function rect(value: DOMRect): SelectPopoverRect {
    return {
      top: value.top,
      right: value.right,
      bottom: value.bottom,
      left: value.left,
      width: value.width,
      height: value.height,
    };
  }

  function positionPanelPicker(): void {
    if (!panelPickerTrigger) return;
    const triggerRect = panelPickerTrigger.getBoundingClientRect();
    panelPickerGeometry = pickSelectPopoverGeometry({
      triggerRect: rect(triggerRect),
      boundaryRect: {
        top: 0,
        right: window.innerWidth,
        bottom: window.innerHeight,
        left: 0,
        width: window.innerWidth,
        height: window.innerHeight,
      },
      contentHeight: panelPicker?.scrollHeight ?? 300,
      contentWidth: panelPicker?.scrollWidth ?? 272,
      horizontalAlign: triggerRect.left + 272 <= window.innerWidth - 8 ? "start" : "end",
    });
    panelPickerReady = true;
  }

  function panelPickerStyle(): string {
    if (!panelPickerReady) return "visibility:hidden;top:0;left:0;";
    const width = Math.min(272, panelPickerGeometry.maxWidth);
    return `visibility:visible;top:${panelPickerGeometry.top}px;left:${panelPickerGeometry.left}px;width:${width}px;max-height:${panelPickerGeometry.maxHeight}px;`;
  }

  async function togglePanelPicker(focusFirst = false): Promise<void> {
    if (panelPickerOpen) {
      panelPickerOpen = false;
      return;
    }
    panelPickerReady = false;
    panelPickerOpen = true;
    await tick();
    positionPanelPicker();
    if (focusFirst) panelPicker?.querySelector<HTMLButtonElement>("button")?.focus();
  }

  function handlePanelPickerTriggerKeydown(event: KeyboardEvent): void {
    if (!["ArrowDown", "ArrowUp"].includes(event.key)) return;
    event.preventDefault();
    void togglePanelPicker(true);
  }

  function handlePanelPickerKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      panelPickerOpen = false;
      queueMicrotask(() => panelPickerTrigger?.focus());
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    const buttons = [...(event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>("button")];
    if (buttons.length === 0) return;
    event.preventDefault();
    const currentIndex = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const nextIndex = event.key === "Home"
      ? 0
      : event.key === "End"
        ? buttons.length - 1
        : (Math.max(0, currentIndex) + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
    buttons[nextIndex]?.focus();
  }

  $effect(() => {
    if (!panelPickerOpen) return;
    const handleOutsidePointer = (event: MouseEvent) => {
      if (!(event.target instanceof Node)) return;
      if (panelPickerTrigger?.contains(event.target) || panelPicker?.contains(event.target)) return;
      panelPickerOpen = false;
    };
    const handleViewportChange = () => positionPanelPicker();
    window.addEventListener("mousedown", handleOutsidePointer, true);
    window.addEventListener("resize", handleViewportChange);
    window.addEventListener("scroll", handleViewportChange, true);
    return () => {
      window.removeEventListener("mousedown", handleOutsidePointer, true);
      window.removeEventListener("resize", handleViewportChange);
      window.removeEventListener("scroll", handleViewportChange, true);
    };
  });
</script>

<section class="workspace-panel" data-placement={placement} aria-label={placement === "bottom" ? t("chat.bottomPanel") : t("chat.inspector.title")}>
  <div bind:this={panelTabbar} class="panel-tabbar" class:reordering={draggedTabKey !== null} role="tablist" aria-label={placement === "bottom" ? t("chat.bottomPanel") : t("chat.inspector.title")}>
    {#each orderedTabs as item (item.key)}
      <div role="presentation" class="panel-tab-slot" class:dragging={draggedTabKey === item.key} data-panel-tab-key={item.key} style:--tab-shift-x={`${tabDragShift(item.key)}px`} onpointerdown={(event) => beginTabDrag(event, item.key)} onmousedown={preventMiddleButtonScroll} onauxclick={(event) => closeTabFromAuxClick(event, item.key, item.type === "terminal" ? item.terminal : undefined)}>
        {#if item.type === "loading-terminal"}
          <button type="button" role="tab" aria-selected={panelState.tab === "terminal"} tabindex={panelState.tab === "terminal" ? 0 : -1} class="terminal-tab loading" class:active={panelState.tab === "terminal"} onclick={() => selectPanel("terminal")} onkeydown={handleTabKeydown}>
            <SquareTerminal size={13} /><span>{t("chat.inspector.terminal")}</span>
          </button>
        {:else if item.type === "terminal"}
          <div class="terminal-tab-shell" class:active={panelState.tab === "terminal" && selectedTerminalId === item.terminal.id}>
            <button type="button" role="tab" aria-selected={panelState.tab === "terminal" && selectedTerminalId === item.terminal.id} tabindex={panelState.tab === "terminal" && selectedTerminalId === item.terminal.id ? 0 : -1} class="terminal-tab" onclick={() => { selectTerminal(item.terminal.id); selectPanel("terminal"); }} onkeydown={handleTabKeydown} title={item.terminal.name}>
              <SquareTerminal size={13} />
              <span class="terminal-label" class:faded={fadedTerminalIds.includes(item.terminal.id)} use:trackTerminalOverflow={item.terminal.id}>{item.terminal.name}</span>
            </button>
            <button type="button" class="tab-close terminal-close" aria-label={t("chat.inspector.closeTerminal")} onclick={() => void run(() => closeTerminal(item.terminal))}><X size={11} /></button>
          </div>
        {:else}
          {@const Icon = item.panel.icon}
          <div class="panel-tab-shell" class:active={panelState.tab === item.panel.id}>
            <button type="button" role="tab" aria-selected={panelState.tab === item.panel.id} tabindex={panelState.tab === item.panel.id ? 0 : -1} class="panel-tab" title={item.panel.id === "files" ? panelState.filePreviewPath ?? undefined : undefined} onclick={() => selectPanel(item.panel.id)} onkeydown={handleTabKeydown}>
              {#if item.panel.id === "files" && panelState.filePreviewPath}<ChatFileIcon path={panelState.filePreviewPath} size={13} />{:else}<Icon size={13} />{/if}<span>{panelLabel(item.panel)}</span>
            </button>
            <button type="button" class="tab-close" aria-label={t("chat.inspector.closePanel", panelLabel(item.panel))} onclick={() => closePanel(item.panel.id)}><X size={11} /></button>
          </div>
        {/if}
      </div>
    {/each}

    <button
      bind:this={panelPickerTrigger}
      type="button"
      class="chat-icon-button"
      aria-label={t("chat.inspector.addPanel")}
      aria-haspopup="menu"
      aria-expanded={panelPickerOpen}
      title={t("chat.inspector.addPanel")}
      onclick={() => void togglePanelPicker()}
      onkeydown={handlePanelPickerTriggerKeydown}
    ><Plus size={14} /></button>
    {#if panelPickerOpen}
      <div
        bind:this={panelPicker}
        use:portal
        class="panel-picker"
        role="menu"
        tabindex="-1"
        aria-label={t("chat.inspector.addPanel")}
        data-app-floating-surface
        data-placement={panelPickerGeometry.placement}
        style={panelPickerStyle()}
        onkeydown={handlePanelPickerKeydown}
      >
        <p>{t("chat.inspector.addPanel")}</p>
        <button type="button" role="menuitem" onclick={openOrAddTerminal}>
          <SquareTerminal size={15} />
          <span><strong>{t("chat.inspector.terminal")}</strong><small>{t("chat.inspector.terminalDescription")}</small></span>
        </button>
        {#each panelTabs as tab (tab.id)}
          {@const Icon = tab.icon}
          <button type="button" role="menuitem" onclick={() => openPanel(tab.id)}>
            <Icon size={15} />
            <span><strong>{t(`chat.inspector.${tab.label}`)}</strong><small>{t(`chat.inspector.${tab.label}Description`)}</small></span>
          </button>
        {/each}
      </div>
    {/if}

    <span class="flex-1"></span>
    {#if placement === "inspector"}
      <button type="button" class="chat-icon-button" aria-label={panelState.maximized ? t("chat.inspector.restore") : t("chat.inspector.maximize")} onclick={() => update({ maximized: !panelState.maximized })}>
        {#if panelState.maximized}<Minimize2 size={14} />{:else}<Maximize2 size={14} />{/if}
      </button>
    {/if}
    <button type="button" class="chat-icon-button" aria-label={placement === "bottom" ? t("chat.closeBottomPanel") : t("chat.closeInspector")} onclick={onClose}><X size={14} /></button>
  </div>

  {#if error}<p role="alert" class="border-b border-destructive/30 p-2 text-xs text-destructive">{error}</p>{/if}
  <div class="min-h-0 flex-1" role="tabpanel" aria-label={t(`chat.inspector.${panelState.tab}`)}>
    {#if panelState.tab === "terminal"}
      {#if terminalsLoading}
        <p class="grid h-full place-items-center text-xs text-muted-foreground">{t("common.loading")}</p>
      {:else if selectedTerminal}
        <div class="flex h-full min-h-0 flex-col">
          {#if !selectedTerminal.running}
            <div class="terminal-stopped-bar">
              <span>{selectedTerminal.exitCode === null ? t("chat.inspector.terminalStopped") : t("chat.inspector.terminalExited", formatNumber(localization.locale, selectedTerminal.exitCode))}</span>
            </div>
          {/if}
          {#key `${selectedTerminal.id}:${selectedTerminal.generation}`}
            <ChatTerminalView terminalRead={selectedTerminal} onState={updateTerminal} />
          {/key}
        </div>
      {:else}
        <div class="grid h-full place-items-center p-4">
          <button type="button" class="chat-secondary-button" onclick={retryTerminalLoad}>{t("common.retry")}</button>
        </div>
      {/if}
    {:else if panelState.tab === "changes"}
      <ChatChangesPanel
        scope={panelState.changeScope}
        selectedFile={panelState.selectedFile}
        fileListHeightPx={panelState.changedFileListHeightPx}
        whitespaceIgnored={panelState.whitespaceIgnored}
        diffView={panelState.diffView}
        onStateChange={(change) => update({
          ...(change.scope === undefined ? {} : { changeScope: change.scope }),
          ...(change.selectedFile === undefined ? {} : { selectedFile: change.selectedFile }),
          ...(change.fileListHeightPx === undefined ? {} : { changedFileListHeightPx: change.fileListHeightPx }),
          ...(change.whitespaceIgnored === undefined ? {} : { whitespaceIgnored: change.whitespaceIgnored }),
          ...(change.diffView === undefined ? {} : { diffView: change.diffView }),
        })}
      />
    {:else if panelState.tab === "plan"}
      <ChatPlanPanel />
    {:else}
      <ChatFilesPanel
        directoryPath={panelState.fileBrowserPath}
        selectedPath={panelState.filePreviewPath}
        treeVisible={panelState.fileTreeVisible}
        treeWidthPx={panelState.fileTreeWidthPx}
        onStateChange={(change) => update({
          ...(change.directoryPath === undefined ? {} : { fileBrowserPath: change.directoryPath }),
          ...(change.selectedPath === undefined ? {} : { filePreviewPath: change.selectedPath }),
          ...(change.treeVisible === undefined ? {} : { fileTreeVisible: change.treeVisible }),
          ...(change.treeWidthPx === undefined ? {} : { fileTreeWidthPx: change.treeWidthPx }),
        })}
      />
    {/if}
  </div>
</section>

<style>
  .workspace-panel { display: flex; height: 100%; min-height: 0; flex-direction: column; background: var(--cal-bg); }
  .panel-tabbar { position: relative; display: flex; min-height: 2.65rem; flex: 0 0 auto; align-items: center; gap: 0.2rem; padding-inline: 0.45rem; }
  .panel-tabbar.reordering { user-select: none; }
  .panel-tabbar > :global(.chat-icon-button) { align-self: center; }
  .panel-tab-slot { display: flex; min-width: 0; transform: translate3d(var(--tab-shift-x), 0, 0); align-items: stretch; transition: transform 140ms cubic-bezier(0.2, 0, 0, 1); }
  .panel-tabbar.reordering .panel-tab-slot { will-change: transform; }
  .panel-tab-slot.dragging { z-index: 1; transition: none; }
  .terminal-tab, .panel-tab { display: flex; min-width: 0; min-height: 2rem; align-items: center; gap: 0.4rem; padding: 0.3rem 0.65rem; color: inherit; font-size: 0.733333rem; }
  .terminal-tab.loading { border-radius: 0.55rem; color: var(--muted-foreground); }
  .terminal-tab-shell, .panel-tab-shell { display: flex; min-width: 0; align-items: stretch; border-radius: 0.55rem; color: var(--muted-foreground); }
  .terminal-tab-shell { max-width: min(18rem, 48cqw); }
  .terminal-tab-shell:hover, .panel-tab-shell:hover, .terminal-tab.loading:hover { background: color-mix(in srgb, var(--accent) 70%, transparent); color: var(--foreground); }
  .terminal-tab-shell.active, .panel-tab-shell.active, .terminal-tab.loading.active { background: var(--accent); color: var(--foreground); }
  .terminal-label { min-width: 0; max-width: 14rem; overflow: hidden; white-space: nowrap; }
  .terminal-label.faded { -webkit-mask-image: linear-gradient(to right, black calc(100% - 1.4rem), transparent); mask-image: linear-gradient(to right, black calc(100% - 1.4rem), transparent); }
  .panel-tab { padding-right: 0.2rem; }
  .tab-close { display: grid; width: 1.5rem; flex: 0 0 auto; place-items: center; border-radius: 0.3rem; opacity: 0; }
  .terminal-tab-shell:hover .tab-close, .panel-tab-shell:hover .tab-close, .tab-close:focus-visible { opacity: 1; }
  .tab-close:hover { background: var(--accent); }
  .terminal-close { margin-right: 0.2rem; }
  .terminal-stopped-bar { min-height: 1.8rem; flex: 0 0 auto; border-bottom: 1px solid var(--border); padding: 0.45rem 0.55rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .panel-picker { position: fixed; z-index: 80; overflow-y: auto; border: 1px solid var(--border); border-radius: 0.65rem; background: var(--popover); padding: 0.35rem; color: var(--popover-foreground); box-shadow: 0 12px 32px rgb(0 0 0 / 0.2); }
  .panel-picker > p { padding: 0.35rem 0.55rem; color: var(--muted-foreground); font-size: 0.666667rem; font-weight: 600; }
  .panel-picker button { display: flex; width: 100%; align-items: flex-start; gap: 0.65rem; border-radius: 0.45rem; padding: 0.55rem; text-align: left; }
  .panel-picker button:hover { background: var(--accent); }
  .panel-picker button > :global(svg) { margin-top: 0.1rem; flex: 0 0 auto; }
  .panel-picker span { display: grid; min-width: 0; gap: 0.1rem; }
  .panel-picker strong { font-size: 0.733333rem; font-weight: 500; }
  .panel-picker small { color: var(--muted-foreground); font-size: 0.666667rem; line-height: 1.3; }
  @container chat-shell (max-width: 520px) { .terminal-label, .panel-tab span { display: none; } .terminal-tab-shell { max-width: none; } .tab-close { width: 1.3rem; opacity: 1; } }
  @media (hover: none) { .tab-close { opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .panel-tab-slot { transition: none; } }
</style>
