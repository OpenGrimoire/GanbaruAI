<script module lang="ts">
  const THREAD_SCROLL_OFFSETS = new Map<string, number>();
</script>

<script lang="ts">
  import { onMount, tick } from "svelte";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Copy from "@lucide/svelte/icons/copy";
  import FileText from "@lucide/svelte/icons/file-text";
  import Globe from "@lucide/svelte/icons/globe";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import Settings from "@lucide/svelte/icons/settings";
  import Terminal from "@lucide/svelte/icons/terminal";
  import Wrench from "@lucide/svelte/icons/wrench";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import { chatScrollBehavior } from "$lib/chat/responsive-layout";
  import { buildTimelineDisplayRows, projectTimelineReadModel, type TimelineActivityGroupRow, type TimelineActivityRow, type TimelineDisplayRow, type TimelineMessageRow, type TimelinePlanRow, type TimelineTurnFoldRow } from "$lib/chat/timeline-model";
  import { computeTimelineVirtualWindow, nextTimelineUnreadCount, scrollTopAfterPrepend, timelineMinimapRows, timelineScrollIntent, type TimelineScrollIntent } from "$lib/chat/timeline-virtualization";
  import { formatDateTime, formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatMarkdown from "./ChatMarkdown.svelte";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const settings = getSettingsLauncher();
  let scroller: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let viewportHeight = $state(600);
  let viewportWidth = $state(800);
  let expandedTurns = $state<string[]>([]);
  let expandedGroups = $state<string[]>([]);
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
  const pageTurns = $derived(chat.timelinePages.flatMap((page) => page.turns));
  const projection = $derived(projectTimelineReadModel(chat.timelineItems, pageTurns));
  const displayRows = $derived(buildTimelineDisplayRows(projection.rows.filter((row) => row.kind !== "plan" || !dismissedPlans.includes(row.id)), projection.turns, new Set(expandedTurns), new Set(expandedGroups)));
  const virtualWindow = $derived(computeTimelineVirtualWindow(displayRows, measuredHeights, scrollTop, viewportHeight));
  const selectedThread = $derived(chat.selectedThread);
  const selectedWorkspace = $derived(chat.selectedWorkspace);
  const selectedProvider = $derived(chat.settings?.providerInstances.find((provider) => provider.configuration.instanceId === selectedThread?.providerInstanceId) ?? null);
  const minimapRows = $derived(timelineMinimapRows(displayRows));
  const showMinimap = $derived(displayRows.length >= 80 && viewportWidth >= 900 && minimapRows.length > 0);

  onMount(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (!entry) return;
      viewportHeight = entry.contentRect.height;
      viewportWidth = entry.contentRect.width;
    });
    if (scroller) observer.observe(scroller);
    const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotion = () => { reducedMotion = motion.matches; };
    updateMotion();
    motion.addEventListener("change", updateMotion);
    return () => {
      observer.disconnect();
      motion.removeEventListener("change", updateMotion);
    };
  });

  $effect(() => {
    virtualWindow.items;
    void tick().then(measureRows);
  });

  $effect(() => {
    const count = chat.timelineItems.length;
    if (count > previousItemCount) {
      unreadEvents = nextTimelineUnreadCount(unreadEvents, count - previousItemCount, intent, false);
      if (intent === "following") void tick().then(() => scroller?.scrollTo({ top: scroller.scrollHeight, behavior: chatScrollBehavior(reducedMotion) }));
    }
    previousItemCount = count;
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
      if (after) scroller.scrollTop = scrollTopAfterPrepend(beforeScroll, beforeTop, after.offsetTop);
    } catch (error: unknown) {
      reportError(error);
    } finally {
      loadingOlder = false;
    }
  }

  function measureRows(): void {
    if (!scroller) return;
    const scrollerTop = scroller.getBoundingClientRect().top;
    const anchor = [...scroller.querySelectorAll<HTMLElement>("[data-timeline-row-id]")]
      .find((element) => element.getBoundingClientRect().bottom >= scrollerTop);
    const anchorId = anchor?.dataset.timelineRowId;
    const anchorOffset = anchor ? anchor.getBoundingClientRect().top - scrollerTop : 0;
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
      if (intent !== "following" && anchorId) {
        void tick().then(() => {
          if (!scroller) return;
          const restored = scroller.querySelector<HTMLElement>(`[data-timeline-row-id="${CSS.escape(anchorId)}"]`);
          if (restored) scroller.scrollTop += restored.getBoundingClientRect().top - scroller.getBoundingClientRect().top - anchorOffset;
        });
      }
    }
  }

  function jumpToLatest(): void {
    scroller?.scrollTo({ top: scroller.scrollHeight, behavior: chatScrollBehavior(reducedMotion) });
    intent = "following";
    unreadEvents = 0;
  }

  function toggle(list: string[], id: string): string[] {
    return list.includes(id) ? list.filter((entry) => entry !== id) : [...list, id];
  }

  function copy(value: string): void {
    void navigator.clipboard.writeText(value).catch(reportError);
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
    if (activity.id.startsWith("turn-pending:")) return t("chat.timeline.working");
    if (activity.title === "thread_reverted") return t("chat.timeline.threadRestored");
    return activity.title;
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
    if (row.kind === "activity_group") return `${row.latest.title}, ${statusLabel(row.latest.status)}`;
    if (row.kind === "turn_fold") return foldLabel(row);
    return t("chat.timeline.plan");
  }
</script>

<div class="relative min-h-0 flex-1">
  {#if selectedThread?.archivedAt}<div class="chat-timeline-banner"><span>{t("chat.firstUse.archivedDescription")}</span><button type="button" onclick={() => void chat.restoreThread(selectedThread).catch(reportError)}><RotateCcw size={13} />{t("chat.restore")}</button></div>{/if}
  {#if selectedWorkspace && selectedWorkspace.bindingStatus !== "available"}<div class="chat-timeline-banner text-status-tentative"><CircleAlert size={14} /><span>{t("chat.timeline.workspaceMissing")}</span><button type="button" onclick={() => void chat.rebindWorkspace(selectedWorkspace.workspace.id, t("chat.firstUse.chooseWorkspaceFolder")).catch(reportError)}>{t("chat.timeline.rebind")}</button></div>{/if}
  {#if selectedProvider && (!selectedProvider.configuration.enabled || selectedProvider.lastProbe?.state !== "healthy")}<div class="chat-timeline-banner text-status-tentative"><CircleAlert size={14} /><span>{selectedProvider.lastProbe?.detail ?? t("chat.status.providerUnavailable")}</span><button type="button" onclick={() => void chat.probeProvider(selectedProvider.configuration.instanceId).catch(reportError)}>{t("chat.timeline.retry")}</button><button type="button" onclick={() => settings.open("chat", { chatSubsection: "providers" })}><Settings size={13} />{t("chat.timeline.openSettings")}</button></div>{/if}
  {#if selectedThread?.state === "error"}<div class="chat-timeline-banner text-destructive"><CircleAlert size={14} /><span>{t("chat.timeline.threadError")}</span><button type="button" onclick={() => chat.newDraft(selectedThread.workspaceId)}><MessageSquare size={13} />{t("chat.timeline.startNewThread")}</button></div>{/if}
  {#if operationError || chat.timelineError}<div role="alert" class="chat-timeline-banner text-destructive"><span>{operationError ?? chat.timelineError}</span>{#if selectedThread}<button type="button" onclick={() => { operationError = null; chat.selectThread(selectedThread.id); }}>{t("chat.timeline.retry")}</button>{/if}</div>{/if}
  <div bind:this={scroller} class="h-full overflow-y-auto" role="feed" aria-busy={chat.timelineLoading || undefined} aria-label={t("chat.title")} onscroll={handleScroll}>
    <div class="mx-auto w-full max-w-3xl px-4 py-6" style={`padding-top:${virtualWindow.paddingTop + 24}px;padding-bottom:${virtualWindow.paddingBottom + 96}px`}>
      {#if loadingOlder}<div class="mb-3 flex justify-center text-xs text-muted-foreground"><LoaderCircle size={14} class="animate-spin" />{t("chat.timeline.loadingOlder")}</div>{/if}
      {#if chat.timelineLoading && displayRows.length === 0}<div class="py-12 text-center text-sm text-muted-foreground">{t("common.loading")}</div>{/if}
      {#each virtualWindow.items as virtual (virtual.row.id)}
        {@const row = virtual.row}
        <div data-timeline-row-id={row.id} class="mb-4" role="article" aria-label={rowAriaLabel(row)} aria-posinset={virtual.index + 1} aria-setsize={displayRows.length} tabindex="-1">
          {#if row.kind === "message"}
            {@const message = row as TimelineMessageRow}
            <article class={message.role === "user" ? "chat-user-message" : "chat-assistant-message"}>
              <div class:chat-message-collapsed={message.role === "user" && message.markdown.length > 1200 && !expandedMessages.includes(message.id)}>
                {#if message.role === "assistant"}<ChatMarkdown markdown={message.markdown} onError={reportError} />{:else}<p class="wrap-break-word whitespace-pre-wrap">{message.markdown}</p>{/if}
              </div>
              {#if message.role === "user" && message.userContext}
                <div class="chat-user-context">
                  {#each message.userContext.attachments as attachment}<button type="button" title={attachment.status ?? t("chat.timeline.attachment")} onclick={() => copy(attachment.displayName)}><FileText size={12} /><span>{attachment.displayName}</span>{#if attachment.byteSize !== null}<small>{formatNumber(localization.locale, attachment.byteSize)} B</small>{/if}</button>{/each}
                  {#each message.userContext.mentions as mention}<button type="button" title={t("chat.timeline.mention")} onclick={() => copy(mention.relativePath)}><span>@</span><span>{mention.relativePath}</span></button>{/each}
                  {#each message.userContext.terminalContext as context}<button type="button" title={t("chat.timeline.terminalContext")} onclick={() => copy(context)}><Terminal size={12} /><span>{context}</span></button>{/each}
                </div>
              {/if}
              <div class="mt-2 flex flex-wrap items-center gap-2 text-[0.666667rem] text-muted-foreground">
                {#if message.role === "user" && message.markdown.length > 1200}<button type="button" onclick={() => { expandedMessages = toggle(expandedMessages, message.id); }}>{expandedMessages.includes(message.id) ? t("chat.timeline.showLess") : t("chat.timeline.showMore")}</button>{/if}
                {#if message.role === "user"}<span title={t("chat.timeline.timestamp")}>{timestampLabel(message.createdAt)}</span>{/if}
                {#if message.role === "user" && message.userContext?.preCheckpointId}<button type="button" class="inline-flex items-center gap-1" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-revert-message", { detail: { threadId: chat.selectedThreadId, checkpointId: message.userContext?.preCheckpointId, turnId: message.turnId } }))}><RotateCcw size={11} />{t("chat.timeline.revert")}</button>{/if}
                <button type="button" class="ml-auto inline-flex items-center gap-1" onclick={() => copy(message.markdown)}><Copy size={11} />{t("chat.timeline.copy")}</button>
                {#if message.metadata}<span>{durationLabel(message.metadata.durationMs)}</span>{#if message.metadata.modelId}<span>{message.metadata.modelId}</span>{/if}{#if tokenUsageLabel(message)}<span>{tokenUsageLabel(message)}</span>{/if}{#if message.metadata.changedFiles.length > 0}<span>{t("chat.timeline.changedFiles", formatNumber(localization.locale, message.metadata.changedFiles.length))}</span>{/if}{/if}
              </div>
            </article>
          {:else if row.kind === "activity"}
            {@const activity = row as TimelineActivityRow}<details class="chat-activity" class:failed={activity.status === "failed"}><summary>{#if activity.activityKind === "command_execution" || activity.activityKind === "command_output"}<Terminal size={13} />{:else if activity.activityKind === "file_change" || activity.activityKind === "file_change_output"}<FileText size={13} />{:else if activity.activityKind === "web_search"}<Globe size={13} />{:else if activity.activityKind === "image_view"}<ImageIcon size={13} />{:else if activity.id.startsWith("turn-pending:")}<LoaderCircle size={13} class="animate-spin" />{:else}<Wrench size={13} />{/if}<span class="min-w-0 flex-1 truncate">{activityTitle(activity)}</span><span>{statusLabel(activity.status)}</span></summary>{#if activityDetail(activity)}<pre>{activityDetail(activity)}</pre>{/if}</details>
          {:else if row.kind === "activity_group"}
            {@const group = row as TimelineActivityGroupRow}<button type="button" class="chat-activity-group" onclick={() => { expandedGroups = toggle(expandedGroups, group.id); }}>{#if group.expanded}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}<span class="min-w-0 flex-1 truncate">{group.latest.title}</span><span>{t("chat.timeline.earlierSteps", formatNumber(localization.locale, group.earlierRows.length))}</span></button>{#if group.expanded}{#each [...group.earlierRows, group.latest] as activity}<details class="chat-activity ml-4" class:failed={activity.status === "failed"}><summary><Wrench size={13} /><span class="min-w-0 flex-1 truncate">{activity.title}</span><span>{statusLabel(activity.status)}</span></summary>{#if activity.detail}<pre>{activity.detail}</pre>{/if}</details>{/each}{/if}
          {:else if row.kind === "turn_fold"}
            {@const fold = row as TimelineTurnFoldRow}<button type="button" class="chat-turn-fold" onclick={() => { expandedTurns = toggle(expandedTurns, fold.turnId); }}>{#if fold.expanded}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}{foldLabel(fold)}</button>
          {:else if row.kind === "plan"}
            {@const plan = row as TimelinePlanRow}<article class="chat-plan-card"><h3>{t("chat.timeline.plan")}</h3><ChatMarkdown markdown={plan.markdown} onError={reportError} />{#if plan.steps.length > 0}<ol>{#each plan.steps as step}<li>{step.text} <span>{statusLabel(step.status)}</span></li>{/each}</ol>{/if}<div class="mt-3 flex flex-wrap gap-2"><button type="button" onclick={() => copy(plan.markdown)}>{t("chat.timeline.copy")}</button><button type="button" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-continue-plan", { detail: { planId: plan.id } }))}>{t("chat.timeline.continuePlanning")}</button><button type="button" onclick={() => window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-implement-plan", { detail: { planId: plan.id } }))}>{t("chat.timeline.implementPlan")}</button><button type="button" onclick={() => { dismissedPlans = [...dismissedPlans, plan.id]; }}>{t("chat.timeline.dismiss")}</button></div></article>
          {/if}
        </div>
      {/each}
    </div>
  </div>
  {#if showMinimap}<nav class="chat-timeline-minimap" aria-label={t("chat.timeline.minimap")}>{#each minimapRows as row}<button type="button" class:user={row.kind === "message" && row.role === "user"} class:assistant={row.kind === "message" && row.role === "assistant"} class:error={row.kind === "activity"} class:current={row.sequence >= (virtualWindow.items[0]?.row.sequence ?? Number.MAX_SAFE_INTEGER) && row.sequence <= (virtualWindow.items.at(-1)?.row.sequence ?? Number.MIN_SAFE_INTEGER)} title={minimapLabel(row)} aria-label={t("chat.timeline.minimapRow", minimapLabel(row))} onclick={() => scrollToMinimapRow(row.id)}></button>{/each}</nav>{/if}
  {#if intent !== "following"}<button type="button" class="chat-jump-latest" onclick={jumpToLatest}><ArrowDown size={13} />{t("chat.timeline.jumpLatest")}{#if unreadEvents > 0}<span>{unreadEvents}</span>{/if}</button>{/if}
</div>

<style>
  .chat-timeline-banner { display: flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.5rem; border-bottom: 1px solid var(--border); background: var(--background); padding: 0.4rem 0.75rem; font-size: 0.733333rem; }
  .chat-timeline-banner button { display: inline-flex; align-items: center; gap: 0.25rem; border-radius: 0.25rem; border: 1px solid var(--border); padding: 0.2rem 0.45rem; }
  .chat-user-message { margin-left: auto; max-width: min(86%, 42rem); border: 1px solid var(--border); border-radius: 0.75rem; background: var(--card); padding: 0.75rem 0.9rem; font-size: 0.866667rem; }
  .chat-assistant-message { color: var(--foreground); font-size: 0.866667rem; }
  .chat-user-context { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: 0.55rem; }
  .chat-user-context button { display: inline-flex; max-width: 100%; align-items: center; gap: 0.3rem; border: 1px solid var(--border); border-radius: 999px; padding: 0.18rem 0.45rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-user-context button span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chat-user-context small { font-size: inherit; opacity: 0.8; }
  .chat-message-collapsed { position: relative; max-height: 14rem; overflow: hidden; }
  .chat-message-collapsed::after { position: absolute; inset: auto 0 0; height: 3rem; background: linear-gradient(transparent, var(--card)); content: ""; pointer-events: none; }
  .chat-activity { margin-block: 0.25rem; border-radius: 0.375rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .chat-activity summary { display: flex; cursor: pointer; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0.35rem 0.5rem; }
  .chat-activity.failed { color: var(--destructive); }
  .chat-activity pre { max-height: 18rem; overflow: auto; border-left: 2px solid var(--border); padding: 0.5rem; white-space: pre-wrap; }
  .chat-activity-group, .chat-turn-fold { display: flex; width: 100%; align-items: center; gap: 0.4rem; border-radius: 0.375rem; padding: 0.4rem 0.5rem; color: var(--muted-foreground); font-size: 0.733333rem; text-align: left; }
  .chat-activity-group:hover, .chat-turn-fold:hover { background: var(--accent); color: var(--foreground); }
  .chat-plan-card { border: 1px solid var(--border); border-radius: 0.75rem; background: var(--card); padding: 1rem; }
  .chat-plan-card h3 { margin-bottom: 0.6rem; font-weight: 650; }
  .chat-plan-card button { border-radius: 0.375rem; border: 1px solid var(--border); padding: 0.3rem 0.55rem; font-size: 0.733333rem; }
  .chat-timeline-minimap { position: absolute; right: 0.55rem; top: 3rem; bottom: 4rem; display: flex; width: 0.7rem; flex-direction: column; justify-content: space-evenly; gap: 1px; border-radius: 999px; background: color-mix(in srgb, var(--popover) 88%, transparent); padding: 0.2rem; box-shadow: 0 2px 10px rgb(0 0 0 / 0.12); }
  .chat-timeline-minimap button { min-height: 2px; flex: 1 1 2px; border-radius: 999px; background: var(--muted-foreground); opacity: 0.45; }
  .chat-timeline-minimap button.user { background: var(--primary); opacity: 0.8; }
  .chat-timeline-minimap button.assistant { background: var(--foreground); opacity: 0.65; }
  .chat-timeline-minimap button.error { background: var(--destructive); opacity: 0.9; }
  .chat-timeline-minimap button.current { outline: 1px solid var(--ring); opacity: 1; }
  .chat-timeline-minimap button:focus-visible { width: 0.8rem; outline: 2px solid var(--ring); }
  .chat-jump-latest { position: absolute; bottom: 1rem; left: 50%; display: inline-flex; min-height: 2.25rem; transform: translateX(-50%); align-items: center; gap: 0.4rem; border: 1px solid var(--border); border-radius: 999px; background: var(--popover); padding: 0.35rem 0.75rem; box-shadow: 0 6px 20px rgb(0 0 0 / 0.16); font-size: 0.733333rem; }
</style>
