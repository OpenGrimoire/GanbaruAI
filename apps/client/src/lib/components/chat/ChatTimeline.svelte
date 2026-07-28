<script module lang="ts">
  const THREAD_SCROLL_OFFSETS = new Map<string, number>();
</script>

<script lang="ts">
  import { onDestroy, onMount, tick } from "svelte";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Copy from "@lucide/svelte/icons/copy";
  import FileText from "@lucide/svelte/icons/file-text";
  import Globe from "@lucide/svelte/icons/globe";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import ListChecks from "@lucide/svelte/icons/list-checks";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Settings from "@lucide/svelte/icons/settings";
  import Terminal from "@lucide/svelte/icons/terminal";
  import Wrench from "@lucide/svelte/icons/wrench";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import { chatScrollBehavior } from "$lib/chat/responsive-layout";
  import { chatModelParticipant, type ChatModelParticipant } from "$lib/chat/participant-identity";
  import { buildTimelineDisplayRows, includeOptimisticTimelineMessage, projectTimelineReadModel, timelineActivityShowsLiveStatus, timelineActivitySupportsDisclosure, timelineModelGroupStartIds, type TimelineActivityGroupRow, type TimelineActivityRow, type TimelineDisplayRow, type TimelineMessageRow, type TimelinePlanRow, type TimelineTurnFoldRow } from "$lib/chat/timeline-model";
  import { computeTimelineVirtualWindow, nextTimelineUnreadCount, scrollTopForPreservedAnchor, timelineMinimapRows, timelineScrollbarThumbGeometry, timelineScrollIntent, type TimelineScrollbarThumbGeometry, type TimelineScrollIntent } from "$lib/chat/timeline-virtualization";
  import { formatDateTime, formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import { writeTextToClipboard } from "$lib/utils/clipboard";
  import ChatMarkdown from "./ChatMarkdown.svelte";
  import ChatActivityDetail from "./ChatActivityDetail.svelte";
  import ChatChangedFilesSummary from "./ChatChangedFilesSummary.svelte";
  import ChatModelAvatar from "./ChatModelAvatar.svelte";
  import ProfileAvatar from "$lib/components/profile/ProfileAvatar.svelte";

  const { bottomInsetPx = 0 } = $props<{ bottomInsetPx?: number }>();
  const COMPOSER_READING_GAP_PX = 8;
  const TIMELINE_EDGE_PADDING_PX = 16;
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const preferences = getPreferences();
  const settings = getSettingsLauncher();
  let scroller: HTMLDivElement | undefined = $state();
  let timelineContent: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewportHeight = $state(600);
  let viewportWidth = $state(800);
  let expandedTurns = $state<string[]>([]);
  let expandedGroups = $state<string[]>([]);
  let expandedActivities = $state<string[]>([]);
  let expandedMessages = $state<string[]>([]);
  let dismissedPlans = $state<string[]>([]);
  let measuredHeights = $state<Map<string, number>>(new Map());
  let intent = $state<TimelineScrollIntent>("following");
  let unreadEvents = $state(0);
  let operationError = $state<string | null>(null);
  let loadingOlder = $state(false);
  let previousItemCount = $state(0);
  let restoredThreadId = $state<string | null>(null);
  let reducedMotion = $state(false);
  let copiedMessageId = $state<string | null>(null);
  let copiedMessageTimer: ReturnType<typeof setTimeout> | null = null;
  let disclosureMeasureFrame: number | null = null;
  let pendingDisclosureAnchor: { rowId: string; viewportOffset: number } | null = null;
  let scrollbarGeometry = $state<TimelineScrollbarThumbGeometry | null>(null);
  let scrollbarTrackTop = $state(0);
  let scrollbarTrackHeight = $state(0);
  let scrollbarDrag = $state<{
    pointerId: number;
    startClientY: number;
    startScrollTop: number;
    scrollPerPixel: number;
  } | null>(null);
  const pageTurns = $derived(chat.timelinePages.flatMap((page) => page.turns));
  const projection = $derived(projectTimelineReadModel(chat.timelineItems, pageTurns));
  const selectedThread = $derived(chat.selectedThread);
  const optimisticMessage = $derived(chat.pendingUserMessage?.threadId === (selectedThread?.id ?? chat.draftThreadId)
    ? chat.pendingUserMessage.row
    : null);
  const timelineRows = $derived(includeOptimisticTimelineMessage(projection.rows, optimisticMessage));
  const displayRows = $derived(buildTimelineDisplayRows(timelineRows.filter((row) => row.kind !== "plan" || !dismissedPlans.includes(row.id)), projection.turns, new Set(expandedTurns), new Set(expandedGroups)));
  const turnsById = $derived(new Map(projection.turns.map((turn) => [turn.id, turn])));
  const modelGroupStartIds = $derived(timelineModelGroupStartIds(displayRows));
  const virtualWindow = $derived(computeTimelineVirtualWindow(displayRows, measuredHeights, scrollTop, viewportHeight));
  const selectedWorkingFolder = $derived(chat.selectedWorkingFolder);
  const selectedProvider = $derived(chat.settings?.providerInstances.find((provider) => provider.configuration.instanceId === selectedThread?.providerInstanceId) ?? null);
  const minimapRows = $derived(timelineMinimapRows(displayRows));
  const showMinimap = $derived(displayRows.length >= 80 && viewportWidth >= 900 && minimapRows.length > 0);
  const timelineRevision = $derived(`${chat.timelinePages.at(-1)?.threadRevision ?? 0}:${chat.pendingUserMessage?.row.id ?? ""}`);
  const bottomPadding = $derived(
    virtualWindow.paddingBottom
      + (bottomInsetPx > 0 ? bottomInsetPx + COMPOSER_READING_GAP_PX : TIMELINE_EDGE_PADDING_PX),
  );

  onMount(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (!entry) return;
      viewportHeight = entry.contentRect.height;
      viewportWidth = entry.contentRect.width;
      updateTimelineScrollbar();
    });
    if (scroller) observer.observe(scroller);
    const contentObserver = new ResizeObserver(updateTimelineScrollbar);
    if (timelineContent) contentObserver.observe(timelineContent);
    updateTimelineScrollbar();
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotion = () => { reducedMotion = motion.matches; };
    updateMotion();
    motion.addEventListener("change", updateMotion);
    scroller?.addEventListener("click", handleTimelineClick);
    scroller?.addEventListener("transitionend", handleTimelineTransitionEnd);
    return () => {
      observer.disconnect();
      contentObserver.disconnect();
      motion.removeEventListener("change", updateMotion);
      scroller?.removeEventListener("click", handleTimelineClick);
      scroller?.removeEventListener("transitionend", handleTimelineTransitionEnd);
    };
  });

  onDestroy(() => {
    if (copiedMessageTimer) clearTimeout(copiedMessageTimer);
    if (disclosureMeasureFrame !== null) cancelAnimationFrame(disclosureMeasureFrame);
  });

  $effect(() => {
    virtualWindow.items;
    void tick().then(measureRows);
  });

  $effect(() => {
    const count = chat.timelineItems.length;
    if (count > previousItemCount) {
      unreadEvents = nextTimelineUnreadCount(unreadEvents, count - previousItemCount, intent, false);
    }
    previousItemCount = count;
  });

  $effect(() => {
    timelineRevision;
    if (intent !== "following") return;
    void tick().then(pinToLatest);
  });

  $effect(() => {
    bottomInsetPx;
    if (intent !== "following") return;
    void tick().then(pinToLatest);
  });

  $effect(() => {
    const threadId = selectedThread?.id;
    if (!threadId || chat.timelineLoading || restoredThreadId === threadId) return;
    restoredThreadId = threadId;
    void tick().then(() => {
      if (!scroller || selectedThread?.id !== threadId) return;
      scroller.scrollTop = THREAD_SCROLL_OFFSETS.get(threadId) ?? scroller.scrollHeight;
      scrollTop = scroller.scrollTop;
      intent = timelineScrollIntent(scroller.scrollHeight - scroller.clientHeight - scroller.scrollTop, "anchored");
    });
  });

  function handleScroll(): void {
    if (!scroller) return;
    scrollTop = scroller.scrollTop;
    updateTimelineScrollbar();
    if (selectedThread) THREAD_SCROLL_OFFSETS.set(selectedThread.id, scrollTop);
    intent = timelineScrollIntent(scroller.scrollHeight - scroller.clientHeight - scroller.scrollTop, intent);
    if (intent === "following") unreadEvents = 0;
    if (scroller.scrollTop < 240) void loadOlder();
  }

  async function loadOlder(): Promise<void> {
    if (!scroller || loadingOlder || !chat.timelinePages[0]?.previousCursor) return;
    loadingOlder = true;
    const anchor = scroller.querySelector<HTMLElement>("[data-timeline-row-id]");
    const anchorId = anchor?.dataset.timelineRowId;
    const beforeTop = anchor?.offsetTop ?? 0;
    const beforeScroll = scroller.scrollTop;
    try {
      await chat.loadOlderTimeline(virtualWindow.items[0]?.row.sequence ?? null);
      await tick();
      const after = anchorId ? scroller.querySelector<HTMLElement>(`[data-timeline-row-id="${CSS.escape(anchorId)}"]`) : null;
      if (after) scroller.scrollTop = scrollTopForPreservedAnchor(beforeScroll, beforeTop, after.offsetTop);
    } catch (error: unknown) {
      reportError(error);
    } finally {
      loadingOlder = false;
    }
  }

  function measureRows(): void {
    if (!scroller) return;
    const scrollerTop = scroller.getBoundingClientRect().top;
    const disclosureAnchor = pendingDisclosureAnchor;
    pendingDisclosureAnchor = null;
    const anchor = disclosureAnchor
      ? scroller.querySelector<HTMLElement>(`[data-timeline-row-id="${CSS.escape(disclosureAnchor.rowId)}"]`)
      : [...scroller.querySelectorAll<HTMLElement>("[data-timeline-row-id]")]
        .find((element) => element.getBoundingClientRect().bottom >= scrollerTop);
    const anchorId = anchor?.dataset.timelineRowId;
    const anchorOffset = disclosureAnchor?.viewportOffset
      ?? (anchor ? anchor.getBoundingClientRect().top - scrollerTop : 0);
    const next = new Map(measuredHeights);
    let changed = false;
    for (const element of scroller.querySelectorAll<HTMLElement>("[data-timeline-row-id]")) {
      const id = element.dataset.timelineRowId;
      if (!id) continue;
      const height = element.getBoundingClientRect().height;
      if (height > 0 && next.get(id) !== height) { next.set(id, height); changed = true; }
    }
    if (changed) {
      measuredHeights = next;
      if (intent === "following") {
        void tick().then(pinToLatest);
      } else if (anchorId) {
        void tick().then(() => {
          if (!scroller) return;
          const restored = scroller.querySelector<HTMLElement>(`[data-timeline-row-id="${CSS.escape(anchorId)}"]`);
          if (!restored) return;
          const restoredOffset = restored.getBoundingClientRect().top - scroller.getBoundingClientRect().top;
          scroller.scrollTop = scrollTopForPreservedAnchor(scroller.scrollTop, anchorOffset, restoredOffset);
          scrollTop = scroller.scrollTop;
          if (selectedThread) THREAD_SCROLL_OFFSETS.set(selectedThread.id, scrollTop);
        });
      }
    }
  }

  function handleTimelineClick(event: MouseEvent): void {
    if (!scroller || !(event.target instanceof Element)) return;
    const disclosure = event.target.closest<HTMLElement>("[data-timeline-disclosure-expanded]");
    if (!disclosure || !scroller.contains(disclosure)) return;
    const expanding = disclosure.dataset.timelineDisclosureExpanded === "false";
    if (!expanding) return;
    const row = disclosure.closest<HTMLElement>("[data-timeline-row-id]");
    const rowId = row?.dataset.timelineRowId;
    if (!row || !rowId) return;
    pendingDisclosureAnchor = {
      rowId,
      viewportOffset: row.getBoundingClientRect().top - scroller.getBoundingClientRect().top,
    };
    intent = "anchored";
    if (disclosureMeasureFrame !== null) cancelAnimationFrame(disclosureMeasureFrame);
    disclosureMeasureFrame = requestAnimationFrame(() => {
      disclosureMeasureFrame = null;
      measureRows();
    });
  }

  function handleTimelineTransitionEnd(event: TransitionEvent): void {
    if (!(event.target instanceof HTMLElement)) return;
    if (event.propertyName !== "grid-template-rows" && event.propertyName !== "max-height") return;
    if (!event.target.matches(".chat-disclosure-region, .chat-message-expandable, .file-change-region")) return;
    measureRows();
  }

  function updateTimelineScrollbar(): void {
    if (!scroller) return;
    scrollbarTrackTop = scroller.offsetTop;
    scrollbarTrackHeight = scroller.clientHeight;
    scrollbarGeometry = timelineScrollbarThumbGeometry(scroller.scrollHeight, scroller.clientHeight, scroller.scrollTop);
  }

  function handleScrollbarTrackPointerDown(event: PointerEvent): void {
    if (!scroller || !scrollbarGeometry || event.button !== 0 || event.target !== event.currentTarget) return;
    event.preventDefault();
    const track = event.currentTarget;
    if (!(track instanceof HTMLElement)) return;
    const pointerOffset = event.clientY - track.getBoundingClientRect().top;
    if (pointerOffset < scrollbarGeometry.offset) scroller.scrollTop -= scroller.clientHeight;
    else if (pointerOffset > scrollbarGeometry.offset + scrollbarGeometry.size) scroller.scrollTop += scroller.clientHeight;
    updateTimelineScrollbar();
  }

  function handleScrollbarThumbPointerDown(event: PointerEvent): void {
    if (!scroller || !scrollbarGeometry || event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    const thumb = event.currentTarget;
    if (!(thumb instanceof HTMLElement)) return;
    const thumbTravel = scroller.clientHeight - scrollbarGeometry.size;
    const scrollRange = scroller.scrollHeight - scroller.clientHeight;
    scrollbarDrag = {
      pointerId: event.pointerId,
      startClientY: event.clientY,
      startScrollTop: scroller.scrollTop,
      scrollPerPixel: thumbTravel > 0 ? scrollRange / thumbTravel : 0,
    };
    thumb.setPointerCapture(event.pointerId);
  }

  function handleScrollbarThumbPointerMove(event: PointerEvent): void {
    if (!scroller || !scrollbarDrag || event.pointerId !== scrollbarDrag.pointerId) return;
    event.preventDefault();
    scroller.scrollTop = scrollbarDrag.startScrollTop
      + (event.clientY - scrollbarDrag.startClientY) * scrollbarDrag.scrollPerPixel;
    updateTimelineScrollbar();
  }

  function finishScrollbarDrag(event: PointerEvent): void {
    if (!scrollbarDrag || event.pointerId !== scrollbarDrag.pointerId) return;
    const thumb = event.currentTarget;
    if (thumb instanceof HTMLElement && thumb.hasPointerCapture(event.pointerId)) thumb.releasePointerCapture(event.pointerId);
    scrollbarDrag = null;
  }

  function handleScrollbarWheel(event: WheelEvent): void {
    if (!scroller) return;
    event.preventDefault();
    scroller.scrollTop += event.deltaY;
    updateTimelineScrollbar();
  }

  function measureExpandableHeight(node: HTMLElement): { destroy(): void } {
    const content = node.firstElementChild;
    const update = () => {
      const height = content instanceof HTMLElement ? content.scrollHeight : node.scrollHeight;
      node.style.setProperty("--chat-expanded-height", `${Math.ceil(height)}px`);
    };
    update();
    const observer = new ResizeObserver(update);
    if (content) observer.observe(content);
    else observer.observe(node);
    return { destroy: () => observer.disconnect() };
  }

  function pinToLatest(): void {
    if (!scroller || intent !== "following") return;
    scroller.scrollTop = scroller.scrollHeight;
    scrollTop = scroller.scrollTop;
  }

  function jumpToLatest(): void {
    scroller?.scrollTo({ top: scroller.scrollHeight, behavior: chatScrollBehavior(reducedMotion) });
    intent = "following";
    unreadEvents = 0;
  }

  function toggle(list: string[], id: string): string[] {
    return list.includes(id) ? list.filter((entry) => entry !== id) : [...list, id];
  }

  function copy(value: string, messageId: string | null = null): void {
    void writeTextToClipboard(value).then(() => {
      if (!messageId) return;
      copiedMessageId = messageId;
      if (copiedMessageTimer) clearTimeout(copiedMessageTimer);
      copiedMessageTimer = setTimeout(() => {
        copiedMessageId = null;
        copiedMessageTimer = null;
      }, 2_000);
    }).catch(reportError);
  }

  function reportError(error: unknown): void {
    operationError = error instanceof Error ? error.message : String(error);
  }

  function durationLabel(milliseconds: number | null): string {
    if (milliseconds === null) return t("chat.timeline.durationUnknown");
    const seconds = Math.max(1, Math.round(milliseconds / 1000));
    return t("chat.timeline.seconds", formatNumber(localization.locale, seconds));
  }

  function foldLabel(row: TimelineTurnFoldRow): string {
    if (row.state === "interrupted") return t("chat.timeline.stoppedAfter", durationLabel(row.durationMs));
    if (row.state === "failed") return t("chat.timeline.failedAfter", durationLabel(row.durationMs));
    return t("chat.timeline.workedFor", durationLabel(row.durationMs));
  }

  function timestampLabel(value: string): string {
    return formatDateTime(localization.locale, new Date(value), { dateStyle: "medium", timeStyle: "short" });
  }

  function statusLabel(status: TimelineActivityRow["status"]): string {
    switch (status) {
      case "pending": return t("chat.timeline.activityStatus.pending");
      case "active": return t("chat.timeline.activityStatus.active");
      case "waiting": return t("chat.timeline.activityStatus.waiting");
      case "completed": return t("chat.timeline.activityStatus.completed");
      case "interrupted": return t("chat.timeline.activityStatus.interrupted");
      case "failed": return t("chat.timeline.activityStatus.failed");
      case "unknown": return t("chat.timeline.activityStatus.unknown");
    }
  }

  function tokenUsageLabel(message: TimelineMessageRow): string | null {
    const usage = message.metadata?.usage;
    if (!usage || usage.inputTokens === null || usage.outputTokens === null) return null;
    return t("chat.timeline.tokenUsage", formatNumber(localization.locale, usage.inputTokens), formatNumber(localization.locale, usage.outputTokens));
  }

  function scrollToMinimapRow(rowId: string): void {
    if (!scroller) return;
    const index = displayRows.findIndex((row) => row.id === rowId);
    if (index < 0) return;
    const denominator = Math.max(1, displayRows.length - 1);
    scroller.scrollTo({ top: (index / denominator) * Math.max(0, scroller.scrollHeight - scroller.clientHeight), behavior: chatScrollBehavior(reducedMotion) });
    intent = index === displayRows.length - 1 ? "following" : "anchored";
  }

  function minimapLabel(row: TimelineMessageRow | TimelineActivityRow): string {
    if (row.kind === "activity") return t("chat.timeline.errorActivity");
    return row.role === "user" ? t("chat.timeline.userMessage") : t("chat.timeline.assistantMessage");
  }

  function activityTitle(activity: TimelineActivityRow): string {
    if (activityIsThinking(activity)) return t("chat.timeline.thinking");
    if (activity.title === "thread_reverted") return t("chat.timeline.threadRestored");
    const active = activity.status === "pending" || activity.status === "active" || activity.status === "waiting";
    if (activity.activityKind === "command_execution" || activity.activityKind === "command_output") {
      const command = activity.title.trim().replace(/^(?:run|running|ran)\s+/i, "");
      if (!command || command === "command execution" || command === "command output") {
        return active ? t("chat.timeline.runningCommands") : t("chat.timeline.ranCommands");
      }
      return active
        ? t("chat.timeline.runningCommand", command)
        : t("chat.timeline.ranCommand", command);
    }
    if (activity.activityKind === "file_change" || activity.activityKind === "file_change_output") {
      return active ? t("chat.timeline.editingFile") : t("chat.timeline.editedFile");
    }
    if (activity.activityKind === "web_search") {
      return active ? t("chat.timeline.searchingWeb") : t("chat.timeline.searchedWeb");
    }
    if (activity.activityKind === "image_view" || isImageTool(activity.title)) {
      return active ? t("chat.timeline.viewingImage") : t("chat.timeline.viewedImage");
    }
    if (isFileReadTool(activity.title)) {
      return active ? t("chat.timeline.readingFiles") : t("chat.timeline.readFiles");
    }
    if (activity.activityKind === "mcp_tool_call" || activity.activityKind === "dynamic_tool_call") {
      return active
        ? t("chat.timeline.usingTool", activity.title)
        : t("chat.timeline.usedTool", activity.title);
    }
    if (activity.activityKind === "collaboration_task") {
      return active ? t("chat.timeline.delegatingWork") : t("chat.timeline.delegatedWork");
    }
    if (activity.activityKind === "context_compaction") {
      return active ? t("chat.timeline.compactingContext") : t("chat.timeline.compactedContext");
    }
    return activity.title;
  }

  function activityIsThinking(activity: TimelineActivityRow): boolean {
    return activity.id.startsWith("turn-pending:")
      || activity.activityKind === "reasoning"
      || activity.activityKind === "reasoning_text"
      || activity.activityKind === "reasoning_summary";
  }

  function activityIsInProgress(activity: TimelineActivityRow): boolean {
    const turnState = activity.turnId ? turnsById.get(activity.turnId)?.state : null;
    return timelineActivityShowsLiveStatus(activity, turnState);
  }

  function normalizedToolName(title: string): string {
    return title.trim().toLowerCase().replaceAll(/[^a-z0-9]+/g, "_");
  }

  function isFileReadTool(title: string): boolean {
    const tool = normalizedToolName(title);
    return ["read", "read_file", "read_files", "open_file", "read_mcp_resource"].some((name) => tool === name || tool.endsWith(`_${name}`));
  }

  function isImageTool(title: string): boolean {
    const tool = normalizedToolName(title);
    return ["view_image", "image_view"].some((name) => tool === name || tool.endsWith(`_${name}`));
  }

  function activityDetail(activity: TimelineActivityRow): string | null {
    if (activity.title !== "thread_reverted") return activity.detail;
    const value = activity.metadata?.value;
    if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
    const count = "revertedTurnCount" in value && typeof value.revertedTurnCount === "number"
      ? value.revertedTurnCount
      : 0;
    const action = "providerHistoryAction" in value && value.providerHistoryAction === "rolled_back"
      ? t("chat.timeline.providerHistoryRolledBack")
      : t("chat.timeline.providerHistoryForkRequired");
    return t("chat.timeline.threadRestoredDetail", formatNumber(localization.locale, count), action);
  }

  function rowAriaLabel(row: TimelineDisplayRow): string {
    if (row.kind === "message") {
      const role = row.role === "user" ? t("chat.timeline.userMessage") : t("chat.timeline.assistantMessage");
      return `${role}, ${timestampLabel(row.createdAt)}`;
    }
    if (row.kind === "activity") return `${activityTitle(row)}, ${statusLabel(row.status)}`;
    if (row.kind === "activity_group") return `${activityTitle(row.latest)}, ${statusLabel(row.latest.status)}`;
    if (row.kind === "turn_fold") return foldLabel(row);
    return t("chat.timeline.plan");
  }

  function isModelOwnedRow(row: TimelineDisplayRow): boolean {
    return row.turnId !== null && !(row.kind === "message" && row.role === "user");
  }

  function participantForRow(row: TimelineDisplayRow): ChatModelParticipant {
    const turn = row.turnId ? turnsById.get(row.turnId) : null;
    const modelId = row.kind === "message" && row.metadata?.modelId
      ? row.metadata.modelId
      : turn?.effectiveModelId ?? turn?.modelId ?? selectedThread?.modelId ?? null;
    const familyId = selectedThread?.providerFamilyId
      ?? selectedProvider?.configuration.familyId
      ?? "opencode";
    return chatModelParticipant(familyId, modelId, selectedProvider?.modelCatalog ?? null);
  }
</script>

{#snippet activityIcon(activity: TimelineActivityRow)}
  {#if activity.activityKind === "command_execution" || activity.activityKind === "command_output"}
    <Terminal size={15} />
  {:else if activity.activityKind === "file_change" || activity.activityKind === "file_change_output" || isFileReadTool(activity.title)}
    <FileText size={15} />
  {:else if activity.activityKind === "web_search"}
    <Globe size={15} />
  {:else if activity.activityKind === "image_view" || isImageTool(activity.title)}
    <ImageIcon size={15} />
  {:else}
    <Wrench size={15} />
  {/if}
{/snippet}

{#snippet activityHistoryRow(activity: TimelineActivityRow)}
  {#if timelineActivitySupportsDisclosure(activity)}
    {@const activityExpanded = expandedActivities.includes(activity.id)}
    <div class="chat-process-step disclosure" class:active={activityIsInProgress(activity)} class:failed={activity.status === "failed"}>
      <button type="button" class="chat-process-step-trigger" data-timeline-disclosure-expanded={activityExpanded} aria-expanded={activityExpanded} onclick={() => { expandedActivities = toggle(expandedActivities, activity.id); }}>
        {@render activityIcon(activity)}
        <span>{activityTitle(activity)}</span>
        <ChevronRight class={activityExpanded ? "chat-step-chevron expanded" : "chat-step-chevron"} size={14} />
      </button>
      <div class="chat-disclosure-region" class:expanded={activityExpanded} aria-hidden={!activityExpanded} inert={!activityExpanded}>
        <div class="chat-disclosure-inner">
          <div class="chat-step-detail">
            <ChatActivityDetail {activity} detail={activityDetail(activity)} />
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="chat-process-step" class:active={activityIsInProgress(activity)} class:failed={activity.status === "failed"} class:thinking={activityIsThinking(activity)}>
      {#if !activityIsThinking(activity)}{@render activityIcon(activity)}{/if}
      <span>{activityTitle(activity)}</span>
    </div>
  {/if}
{/snippet}

{#snippet modelRowContent(row: TimelineDisplayRow)}
  {#if row.kind === "message"}
    {@const message = row as TimelineMessageRow}
    <article class="chat-assistant-message">
      <ChatMarkdown markdown={message.markdown} onError={reportError} />
      {#if message.turnId && message.metadata?.changedFiles.length}
        <ChatChangedFilesSummary turnId={message.turnId} files={message.metadata.changedFiles} />
      {/if}
      {#if message.metadata}
        <div class="chat-message-meta mt-2 flex flex-wrap items-center gap-2 text-muted-foreground">
          {#if message.state === "complete"}
          <button type="button" class="inline-flex items-center gap-1" onclick={() => copy(message.markdown, message.id)}><Copy size={11} />{copiedMessageId === message.id ? t("chat.timeline.copied") : t("chat.timeline.copy")}</button>
          {/if}
          <span>{durationLabel(message.metadata.durationMs)}</span>{#if message.metadata.changedFiles.length > 0}<span>{t("chat.timeline.changedFiles", formatNumber(localization.locale, message.metadata.changedFiles.length))}</span>{/if}{#if tokenUsageLabel(message)}<span>{tokenUsageLabel(message)}</span>{/if}
        </div>
      {/if}
    </article>
  {:else if row.kind === "activity"}
    {@const activity = row as TimelineActivityRow}
    {@render activityHistoryRow(activity)}
  {:else if row.kind === "activity_group"}
    {@const group = row as TimelineActivityGroupRow}
    <button type="button" class="chat-process-toggle" class:active={activityIsInProgress(group.latest)} data-timeline-disclosure-expanded={group.expanded} aria-expanded={group.expanded} onclick={() => { expandedGroups = toggle(expandedGroups, group.id); }}>
      {#if group.expanded}<ChevronDown size={15} />{:else}<ChevronRight size={15} />{/if}
      <span>{activityTitle(group.latest)}</span>
      <small>{t("chat.timeline.earlierSteps", formatNumber(localization.locale, group.earlierRows.length))}</small>
    </button>
    <div class="chat-disclosure-region" class:expanded={group.expanded} aria-hidden={!group.expanded} inert={!group.expanded}>
      <div class="chat-disclosure-inner"><div class="chat-process-history">{#each [...group.earlierRows, group.latest] as activity}{@render activityHistoryRow(activity)}{/each}</div></div>
    </div>
  {:else if row.kind === "turn_fold"}
    {@const fold = row as TimelineTurnFoldRow}
    {#if fold.hiddenRows.length > 0}
      <button type="button" class="chat-process-toggle" class:failed={fold.state === "failed"} data-timeline-disclosure-expanded={fold.expanded} aria-expanded={fold.expanded} onclick={() => { expandedTurns = toggle(expandedTurns, fold.turnId); }}>
        {#if fold.expanded}<ChevronDown size={15} />{:else}<ChevronRight size={15} />{/if}
        <span>{foldLabel(fold)}</span>
      </button>
    {:else}
      <div class="chat-process-toggle chat-process-summary" class:failed={fold.state === "failed"}>
        <span>{foldLabel(fold)}</span>
      </div>
    {/if}
    <div class="chat-disclosure-region" class:expanded={fold.expanded} aria-hidden={!fold.expanded} inert={!fold.expanded}>
      <div class="chat-disclosure-inner"><div class="chat-process-history">{#each fold.hiddenRows as hiddenRow}{@render modelRowContent(hiddenRow)}{/each}</div></div>
    </div>
  {:else if row.kind === "plan"}
    {@const plan = row as TimelinePlanRow}
    <article class="chat-plan">
      <h3><ListChecks size={16} />{t("chat.timeline.plan")}</h3>
      <ChatMarkdown markdown={plan.markdown} onError={reportError} />
      {#if plan.steps.length > 0}<ol>{#each plan.steps as step}<li><span>{step.text}</span><small>{statusLabel(step.status)}</small></li>{/each}</ol>{/if}
      <div class="chat-plan-actions"><button type="button" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-continue-plan", { detail: { planId: plan.id } }))}>{t("chat.timeline.continuePlanning")}</button><button type="button" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-implement-plan", { detail: { planId: plan.id } }))}>{t("chat.timeline.implementPlan")}</button><button type="button" onclick={() => { dismissedPlans = [...dismissedPlans, plan.id]; }}>{t("chat.timeline.dismiss")}</button></div>
    </article>
  {/if}
{/snippet}

<div class="relative min-h-0 flex-1">
  {#if selectedThread?.archivedAt}<div class="chat-timeline-banner"><span>{t("chat.firstUse.archivedDescription")}</span><button type="button" onclick={() => void chat.restoreThread(selectedThread).catch(reportError)}><RotateCcw size={13} />{t("chat.restore")}</button></div>{/if}
  {#if selectedWorkingFolder?.workingFolder.archivedAt}<div class="chat-timeline-banner text-status-tentative"><CircleAlert size={14} /><span>{t("chat.timeline.workingFolderArchived")}</span><button type="button" onclick={() => void chat.restoreWorkingFolder(selectedWorkingFolder.workingFolder.id).catch(reportError)}>{t("chat.restore")}</button></div>{:else if selectedWorkingFolder && selectedWorkingFolder.bindingStatus !== "available"}<div class="chat-timeline-banner text-status-tentative"><CircleAlert size={14} /><span>{t("chat.timeline.workspaceMissing")}</span>{#if selectedWorkingFolder.workingFolder.kind === "managed"}<button type="button" onclick={() => void chat.recreateManagedWorkingFolder(selectedWorkingFolder.workingFolder.id).catch(reportError)}>{t("chat.firstUse.recreateFolder")}</button>{:else}<button type="button" onclick={() => void chat.rebindWorkingFolder(selectedWorkingFolder.workingFolder.id, t("chat.firstUse.chooseWorkingFolder")).catch(reportError)}>{t("chat.timeline.rebind")}</button>{/if}</div>{/if}
  {#if selectedProvider && (!selectedProvider.configuration.enabled || selectedProvider.lastProbe?.state !== "healthy")}<div class="chat-timeline-banner text-status-tentative"><CircleAlert size={14} /><span>{selectedProvider.lastProbe?.detail ?? t("chat.status.providerUnavailable")}</span><button type="button" onclick={() => void chat.probeProvider(selectedProvider.configuration.instanceId).catch(reportError)}>{t("chat.timeline.retry")}</button><button type="button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={13} />{t("chat.timeline.openSettings")}</button></div>{/if}
  {#if selectedThread?.state === "error"}<div class="chat-timeline-banner text-destructive"><CircleAlert size={14} /><span>{t("chat.timeline.threadError")}</span><button type="button" onclick={() => chat.newDraft(selectedThread.workingFolderId)}><MessageSquare size={13} />{t("chat.timeline.startNewThread")}</button></div>{/if}
  {#if operationError || chat.timelineError}<div role="alert" class="chat-timeline-banner text-destructive"><span>{operationError ?? chat.timelineError}</span>{#if selectedThread}<button type="button" onclick={() => { operationError = null; chat.selectThread(selectedThread.id); }}>{t("chat.timeline.retry")}</button>{/if}</div>{/if}
  <div bind:this={scroller} class="chat-timeline-scroller h-full overflow-y-scroll" role="feed" aria-busy={chat.timelineLoading || undefined} aria-label={t("chat.title")} onscroll={handleScroll}>
    <div bind:this={timelineContent} class="chat-timeline-content mx-auto flex min-h-full flex-col justify-end py-4" style={`padding-top:${virtualWindow.paddingTop + TIMELINE_EDGE_PADDING_PX}px;padding-bottom:${bottomPadding}px`}>
      {#if loadingOlder}<div class="mb-3 flex justify-center text-xs text-muted-foreground"><LoaderCircle size={14} class="animate-spin" />{t("chat.timeline.loadingOlder")}</div>{/if}
      {#if chat.timelineLoading && displayRows.length === 0}<div class="py-12 text-center text-sm text-muted-foreground">{t("common.loading")}</div>{/if}
      {#each virtualWindow.items as virtual (virtual.row.id)}
        {@const row = virtual.row}
        <div data-timeline-row-id={row.id} class="chat-timeline-row" class:optimistic={row.id === optimisticMessage?.id} class:participant-start={row.kind === "message" && row.role === "user" || modelGroupStartIds.has(row.id)} role="article" aria-label={rowAriaLabel(row)} aria-posinset={virtual.index + 1} aria-setsize={displayRows.length} tabindex="-1">
          {#if row.kind === "message"}
            {@const message = row as TimelineMessageRow}
            {@const messageExpanded = expandedMessages.includes(message.id)}
            {#if message.role === "user"}
              {@const userDisplayName = preferences.profileDisplayName || t("chat.timeline.you")}
              <div class="chat-participant-row">
                <ProfileAvatar displayName={userDisplayName} imagePath={preferences.profileImagePath} size={36} />
                <div class="chat-participant-content">
                  <div class="chat-participant-header"><strong>{userDisplayName}</strong><span title={t("chat.timeline.timestamp")}>{timestampLabel(message.createdAt)}</span></div>
                  <article class="chat-user-message">
                    <div use:measureExpandableHeight class="chat-message-expandable" class:collapsed={message.markdown.length > 1200 && !messageExpanded} class:expanded={messageExpanded}><div><p class="wrap-break-word whitespace-pre-wrap">{message.markdown}</p></div></div>
                    {#if message.userContext}<div class="chat-user-context">{#each message.userContext.attachments as attachment}<button type="button" title={attachment.status ?? t("chat.timeline.attachment")} onclick={() => copy(attachment.displayName)}><FileText size={12} /><span>{attachment.displayName}</span>{#if attachment.byteSize !== null}<small>{formatNumber(localization.locale, attachment.byteSize)} B</small>{/if}</button>{/each}{#each message.userContext.mentions as mention}<button type="button" title={t("chat.timeline.mention")} onclick={() => copy(mention.relativePath)}><span>@</span><span>{mention.relativePath}</span></button>{/each}{#each message.userContext.terminalContext as context}<button type="button" title={t("chat.timeline.terminalContext")} onclick={() => copy(context)}><Terminal size={12} /><span>{context}</span></button>{/each}</div>{/if}
                    {#if message.markdown.length > 1200 || message.userContext?.preCheckpointId}<div class="chat-message-meta mt-2 flex flex-wrap items-center gap-2 text-muted-foreground">{#if message.markdown.length > 1200}<button type="button" data-timeline-disclosure-expanded={messageExpanded} aria-expanded={messageExpanded} onclick={() => { expandedMessages = toggle(expandedMessages, message.id); }}>{messageExpanded ? t("chat.timeline.showLess") : t("chat.timeline.showMore")}</button>{/if}{#if message.userContext?.preCheckpointId}<button type="button" class="inline-flex items-center gap-1" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-revert-message", { detail: { threadId: chat.selectedThreadId, checkpointId: message.userContext?.preCheckpointId, turnId: message.turnId } }))}><RotateCcw size={11} />{t("chat.timeline.revert")}</button>{/if}</div>{/if}
                  </article>
                </div>
              </div>
            {:else if modelGroupStartIds.has(row.id)}
              {@const participant = participantForRow(row)}
              <div class="chat-participant-row">
                <ChatModelAvatar familyId={participant.company.iconFamilyId} label={participant.company.name} size={36} />
                <div class="chat-participant-content"><div class="chat-participant-header"><strong>{participant.displayName}</strong><span>{timestampLabel(row.createdAt)}</span></div>{@render modelRowContent(row)}</div>
              </div>
            {:else}
              <div class="chat-participant-followup">{@render modelRowContent(row)}</div>
            {/if}
          {:else if isModelOwnedRow(row)}
            {#if modelGroupStartIds.has(row.id)}
              {@const participant = participantForRow(row)}
              <div class="chat-participant-row">
                <ChatModelAvatar familyId={participant.company.iconFamilyId} label={participant.company.name} size={36} />
                <div class="chat-participant-content"><div class="chat-participant-header"><strong>{participant.displayName}</strong><span>{timestampLabel(row.createdAt)}</span></div>{@render modelRowContent(row)}</div>
              </div>
            {:else}
              <div class="chat-participant-followup">{@render modelRowContent(row)}</div>
            {/if}
          {:else}
            {@render modelRowContent(row)}
          {/if}
        </div>
      {/each}
    </div>
  </div>
  <div
    class="chat-timeline-scrollbar"
    class:visible={scrollbarGeometry !== null}
    style={`top:${scrollbarTrackTop}px;height:${scrollbarTrackHeight}px`}
    aria-hidden="true"
    onpointerdown={handleScrollbarTrackPointerDown}
    onwheel={handleScrollbarWheel}
  >
    {#if scrollbarGeometry}
      <div
        class="chat-timeline-scrollbar-thumb"
        class:dragging={scrollbarDrag !== null}
        role="presentation"
        style={`height:${scrollbarGeometry.size}px;transform:translateY(${scrollbarGeometry.offset}px)`}
        onpointerdown={handleScrollbarThumbPointerDown}
        onpointermove={handleScrollbarThumbPointerMove}
        onpointerup={finishScrollbarDrag}
        onpointercancel={finishScrollbarDrag}
        onlostpointercapture={finishScrollbarDrag}
      ></div>
    {/if}
  </div>
  {#if showMinimap}<nav class="chat-timeline-minimap" aria-label={t("chat.timeline.minimap")}>{#each minimapRows as row}<button type="button" class:user={row.kind === "message" && row.role === "user"} class:assistant={row.kind === "message" && row.role === "assistant"} class:error={row.kind === "activity"} class:current={row.sequence >= (virtualWindow.items[0]?.row.sequence ?? Number.MAX_SAFE_INTEGER) && row.sequence <= (virtualWindow.items.at(-1)?.row.sequence ?? Number.MIN_SAFE_INTEGER)} title={minimapLabel(row)} aria-label={t("chat.timeline.minimapRow", minimapLabel(row))} onclick={() => scrollToMinimapRow(row.id)}></button>{/each}</nav>{/if}
  {#if intent !== "following"}<button type="button" class="chat-jump-latest" onclick={jumpToLatest}><ArrowDown size={13} />{t("chat.timeline.jumpLatest")}{#if unreadEvents > 0}<span>{unreadEvents}</span>{/if}</button>{/if}
</div>

<style>
  .chat-timeline-content { width: calc(100% - 1.5rem); max-width: 54rem; }
  .chat-timeline-scroller { overflow-anchor: none; scrollbar-color: transparent transparent; scrollbar-gutter: stable; }
  .chat-timeline-scroller::-webkit-scrollbar-thumb, .chat-timeline-scroller::-webkit-scrollbar-thumb:hover, .chat-timeline-scroller::-webkit-scrollbar-thumb:active { background-color: transparent; }
  .chat-timeline-scrollbar { pointer-events: none; position: absolute; right: 0; z-index: 30; width: 8px; contain: strict; opacity: 0; touch-action: none; }
  .chat-timeline-scrollbar.visible { pointer-events: auto; opacity: 1; }
  .chat-timeline-scrollbar-thumb { box-sizing: border-box; width: 8px; contain: strict; border: 2px solid transparent; border-radius: 9999px; background-color: var(--app-scrollbar-thumb); background-clip: content-box; cursor: default; will-change: transform; }
  .chat-timeline-scrollbar-thumb:hover, .chat-timeline-scrollbar-thumb.dragging { background-color: var(--app-scrollbar-thumb-hover); }
  .chat-timeline-row { --chat-participant-gap: 0.75rem; margin-bottom: 0.25rem; border-radius: 0.4rem; padding-block: 0.2rem; }
  .chat-timeline-row.optimistic { animation: chat-message-in 140ms ease-out; }
  .chat-timeline-row.participant-start { margin-top: 1.1rem; }
  .chat-timeline-row:first-child { margin-top: 0; }
  .chat-participant-row { display: grid; min-width: 0; grid-template-columns: 36px minmax(0, 1fr); align-items: start; gap: var(--chat-participant-gap); }
  .chat-participant-content { min-width: 0; }
  .chat-participant-followup { min-width: 0; margin-left: calc(36px + var(--chat-participant-gap)); }
  .chat-participant-header { display: flex; min-height: 1.25rem; min-width: 0; align-items: baseline; gap: 0.45rem; margin-bottom: 0.12rem; line-height: 1.25rem; }
  .chat-participant-header strong { min-width: 0; overflow: hidden; color: var(--foreground); font-size: var(--chat-conversation-font-size, 0.933333rem); font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
  .chat-participant-header span { flex: 0 0 auto; color: var(--muted-foreground); font-size: 0.7rem; font-weight: 400; }
  .chat-timeline-banner { display: flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.5rem; border-bottom: 1px solid var(--border); background: var(--background); padding: 0.4rem 0.75rem; font-size: 0.733333rem; }
  .chat-timeline-banner button { display: inline-flex; align-items: center; gap: 0.25rem; border-radius: 0.25rem; border: 1px solid var(--border); padding: 0.2rem 0.45rem; }
  .chat-user-message, .chat-assistant-message { width: 100%; min-width: 0; color: var(--foreground); font-size: var(--chat-conversation-font-size, 0.933333rem); line-height: var(--chat-conversation-line-height, 1.4rem); }
  .chat-message-meta { font-size: 0.7rem; opacity: 0; transition: opacity 150ms ease; }
  .chat-user-message:hover .chat-message-meta, .chat-user-message:focus-within .chat-message-meta, .chat-assistant-message:hover .chat-message-meta, .chat-assistant-message:focus-within .chat-message-meta { opacity: 1; }
  .chat-user-context { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: 0.55rem; }
  .chat-user-context button { display: inline-flex; max-width: 100%; align-items: center; gap: 0.3rem; border: 1px solid var(--border); border-radius: 999px; padding: 0.18rem 0.45rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-user-context button span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chat-user-context small { font-size: inherit; opacity: 0.8; }
  .chat-message-expandable { --chat-expanded-height: none; position: relative; max-height: var(--chat-expanded-height); overflow: hidden; transition: max-height 420ms cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-message-expandable.collapsed { max-height: 14rem; }
  .chat-message-expandable::after { position: absolute; inset: auto 0 0; height: 3rem; background: linear-gradient(transparent, var(--cal-bg)); content: ""; opacity: 0; pointer-events: none; transition: opacity 220ms ease; }
  .chat-message-expandable.collapsed::after { opacity: 1; }
  .chat-process-toggle { display: flex; width: 100%; min-height: var(--chat-conversation-line-height, 1.4rem); align-items: center; gap: 0.4rem; color: var(--muted-foreground); font-size: var(--chat-conversation-font-size, 0.933333rem); line-height: var(--chat-conversation-line-height, 1.4rem); text-align: left; }
  .chat-process-toggle > span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chat-process-toggle > small { margin-left: auto; font-size: 0.7rem; }
  .chat-process-toggle :global(svg) { flex: 0 0 auto; transition: color 120ms ease; }
  .chat-process-toggle:hover { color: var(--foreground); }
  .chat-process-toggle:focus-visible { border-radius: 0.25rem; outline: 2px solid var(--ring); outline-offset: 2px; }
  .chat-process-toggle.failed, .chat-process-step.failed { color: var(--destructive); }
  .chat-process-summary { padding-left: 1.4rem; }
  .chat-process-history { display: grid; min-width: 0; gap: 0.1rem; margin-block: 0.35rem 0.55rem; color: var(--muted-foreground); }
  .chat-process-step { display: grid; width: 100%; min-width: 0; grid-template-columns: 1rem minmax(0, 1fr) 1rem; align-items: start; gap: 0.45rem; color: var(--muted-foreground); padding-block: 0.15rem; font-size: var(--chat-conversation-font-size, 0.933333rem); line-height: var(--chat-conversation-line-height, 1.4rem); }
  .chat-process-step.disclosure { display: block; }
  .chat-process-step.thinking { grid-template-columns: minmax(0, 1fr); }
  .chat-process-step > span, .chat-process-step-trigger > span { width: fit-content; min-width: 0; max-width: 100%; justify-self: start; overflow-wrap: anywhere; }
  .chat-process-step-trigger { display: grid; width: 100%; min-width: 0; cursor: pointer; grid-template-columns: 1rem minmax(0, 1fr) 1rem; align-items: start; gap: 0.45rem; text-align: left; }
  .chat-process-step.active, .chat-process-toggle.active { color: color-mix(in srgb, var(--muted-foreground) 78%, var(--foreground)); }
  .chat-process-step.active > span, .chat-process-step.active .chat-process-step-trigger > span, .chat-process-toggle.active > span {
    animation: chat-process-shimmer 6s ease-in-out infinite;
    background: linear-gradient(100deg, var(--muted-foreground) 0%, var(--muted-foreground) 42%, var(--foreground) 50%, var(--muted-foreground) 58%, var(--muted-foreground) 100%);
    background-repeat: no-repeat;
    background-size: 320% 100%;
    background-clip: text;
    color: transparent;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .chat-process-step-trigger:focus-visible { border-radius: 0.25rem; outline: 2px solid var(--ring); outline-offset: 2px; }
  :global(.chat-step-chevron.expanded) { transform: rotate(90deg); }
  :global(.chat-step-chevron) { transition: transform 120ms ease; }
  .chat-disclosure-region { display: grid; grid-template-rows: 0fr; opacity: 0; transition: grid-template-rows 420ms cubic-bezier(0.22, 1, 0.36, 1), opacity 180ms ease; }
  .chat-disclosure-region.expanded { grid-template-rows: 1fr; opacity: 1; transition: grid-template-rows 420ms cubic-bezier(0.22, 1, 0.36, 1), opacity 240ms ease 55ms; }
  .chat-disclosure-inner { min-height: 0; overflow: hidden; transform: translateY(-0.3rem); transition: transform 360ms cubic-bezier(0.22, 1, 0.36, 1); }
  .chat-disclosure-region.expanded > .chat-disclosure-inner { transform: translateY(0); }
  .chat-step-detail { box-sizing: border-box; width: calc(100% - 1.45rem); min-width: 0; max-width: calc(100% - 1.45rem); margin-top: 0.35rem; margin-left: 1.45rem; }
  .chat-plan { color: var(--foreground); font-size: var(--chat-conversation-font-size, 0.933333rem); line-height: var(--chat-conversation-line-height, 1.4rem); }
  .chat-plan h3 { display: flex; align-items: center; gap: 0.4rem; margin-bottom: 0.35rem; font-weight: 650; }
  .chat-plan ol { display: grid; gap: 0.2rem; margin-top: 0.55rem; }
  .chat-plan li { display: flex; align-items: baseline; gap: 0.5rem; }
  .chat-plan li small { margin-left: auto; color: var(--muted-foreground); font-size: 0.7rem; }
  .chat-plan-actions { display: flex; flex-wrap: wrap; gap: 0.65rem; margin-top: 0.55rem; color: var(--muted-foreground); font-size: 0.7rem; opacity: 0; transition: opacity 150ms ease; }
  .chat-plan:hover .chat-plan-actions, .chat-plan:focus-within .chat-plan-actions { opacity: 1; }
  .chat-timeline-minimap { position: absolute; right: 0.55rem; top: 3rem; bottom: 4rem; display: flex; width: 0.7rem; flex-direction: column; justify-content: space-evenly; gap: 1px; border-radius: 999px; background: color-mix(in srgb, var(--popover) 88%, transparent); padding: 0.2rem; box-shadow: 0 2px 10px rgb(0 0 0 / 0.12); }
  .chat-timeline-minimap button { min-height: 2px; flex: 1 1 2px; border-radius: 999px; background: var(--muted-foreground); opacity: 0.45; }
  .chat-timeline-minimap button.user { background: var(--primary); opacity: 0.8; }
  .chat-timeline-minimap button.assistant { background: var(--foreground); opacity: 0.65; }
  .chat-timeline-minimap button.error { background: var(--destructive); opacity: 0.9; }
  .chat-timeline-minimap button.current { outline: 1px solid var(--ring); opacity: 1; }
  .chat-timeline-minimap button:focus-visible { width: 0.8rem; outline: 2px solid var(--ring); }
  .chat-jump-latest { position: absolute; bottom: 1rem; left: 50%; display: inline-flex; min-height: 2.25rem; transform: translateX(-50%); align-items: center; gap: 0.4rem; border: 1px solid var(--border); border-radius: 999px; background: var(--popover); padding: 0.35rem 0.75rem; box-shadow: 0 6px 20px rgb(0 0 0 / 0.16); font-size: 0.733333rem; }
  @keyframes chat-message-in { from { opacity: 0; transform: translateY(0.2rem); } to { opacity: 1; transform: translateY(0); } }
  @keyframes chat-process-shimmer { 0%, 8% { background-position: 100% 0; } 65%, 100% { background-position: 0% 0; } }
  @media (hover: none) { .chat-message-meta, .chat-plan-actions { opacity: 1; } }
  @media (prefers-reduced-motion: reduce) { .chat-timeline-row.optimistic, .chat-process-step.active > span, .chat-process-step.active .chat-process-step-trigger > span, .chat-process-toggle.active > span { animation: none; background: none; color: inherit; -webkit-text-fill-color: currentColor; } .chat-process-toggle :global(svg), :global(.chat-step-chevron), .chat-disclosure-region, .chat-disclosure-inner, .chat-message-expandable, .chat-message-expandable::after { transition: none; } }
  @media (forced-colors: active) { .chat-process-step.active > span, .chat-process-step.active .chat-process-step-trigger > span, .chat-process-toggle.active > span { animation: none; background: none; color: inherit; -webkit-text-fill-color: currentColor; } }
  @container chat-shell (max-width: 420px) {
    .chat-timeline-row { --chat-participant-gap: 0.5rem; }
    .chat-participant-header { flex-wrap: wrap; column-gap: 0.35rem; }
  }
</style>
