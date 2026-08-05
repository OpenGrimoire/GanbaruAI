<script lang="ts">
  import MessagesSquare from "@lucide/svelte/icons/messages-square";
  import X from "@lucide/svelte/icons/x";
  import { tick } from "svelte";
  import {
    exactRunPresentationReady,
    latestRenderableAgentRun,
    replyThreadRenderEntries,
    shouldGroupReplyMessages,
  } from "$lib/chat/reply-thread-model";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { organizationalScrollFollowsEnd } from "$lib/chat/organizational-scroll";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatExecutionTimeline from "./ChatExecutionTimeline.svelte";
  import ChatMessageComposer from "./ChatMessageComposer.svelte";
  import ChatOrganizationalMessage from "./ChatOrganizationalMessage.svelte";
  import ChatRequestPanel from "./ChatRequestPanel.svelte";

  let {
    presentation = "complementary",
    reserveGlobalActions = false,
    onClose,
  }: {
    presentation?: "complementary" | "dialog" | "main";
    reserveGlobalActions?: boolean;
    onClose: () => void;
  } = $props();

  const chat = getChat();
  const localization = getLocalization();
  const { t } = localization;
  let scroller = $state<HTMLDivElement | null>(null);
  let actionError = $state<string | null>(null);
  let executionSelectionRequest = 0;
  let executionThreadScope = $state<string | null>(null);
  let followingEnd = true;
  let restoredDestination: string | null = null;
  const page = $derived(chat.replyThread);
  const assignment = $derived(page?.assignment ?? null);
  const executionRun = $derived(latestRenderableAgentRun(page?.agentRuns ?? []));
  const loadedExecutionTurnIds = $derived(new Set([
    ...chat.timelineItems.flatMap((item) => item.turnId ? [item.turnId] : []),
    ...chat.timelinePages.flatMap((timelinePage) => timelinePage.turns.map((turn) => turn.turnId)),
  ]));
  const loadedExecutionRuns = $derived((page?.agentRuns ?? []).filter((run) => (
    run.providerExecutionThreadId === chat.selectedThreadId
    && loadedExecutionTurnIds.has(run.providerExecutionTurnId)
  )));
  const executionPresentationReady = $derived(exactRunPresentationReady(
    executionRun,
    chat.selectedExecutionRunId,
    chat.selectedThreadId,
    loadedExecutionTurnIds,
  ));
  const renderEntries = $derived(replyThreadRenderEntries(
    page?.replies ?? [],
    loadedExecutionRuns,
    chat.selectedExecutionRunId ? chat.selectedThreadId : null,
  ));
  const destination = $derived(chat.openReplyThreadId
    ? `reply-thread:${chat.openReplyThreadId}`
    : "reply-thread:none");

  function sameDay(left: string, right: string): boolean {
    return new Date(left).toDateString() === new Date(right).toDateString();
  }

  function entryCreatedAt(entry: (typeof renderEntries)[number]): string {
    return entry.kind === "message" ? entry.message.createdAt : entry.run.createdAt;
  }

  function preventMiddleButtonScroll(event: MouseEvent): void {
    if (event.button === 1) event.preventDefault();
  }

  function closeFromMiddleClick(event: MouseEvent): void {
    if (event.button !== 1) return;
    event.preventDefault();
    event.stopPropagation();
    onClose();
  }

  function rememberScrollPosition(): void {
    if (!scroller) return;
    followingEnd = organizationalScrollFollowsEnd(scroller);
    chat.setOrganizationalScrollPosition(destination, scroller.scrollTop);
  }

  function scrollToBottomForUserAction(): void {
    const element = scroller;
    const key = destination;
    if (!element) return;
    followingEnd = true;
    void tick().then(() => {
      if (element !== scroller || key !== destination) return;
      element.scrollTop = element.scrollHeight;
      chat.setOrganizationalScrollPosition(key, element.scrollTop);
    });
  }

  function followNewContentIfAtBottom(): void {
    const element = scroller;
    const key = destination;
    if (!element || restoredDestination !== key || !followingEnd) return;
    void tick().then(() => {
      if (element !== scroller || key !== destination || !followingEnd) return;
      element.scrollTop = element.scrollHeight;
      chat.setOrganizationalScrollPosition(key, element.scrollTop);
    });
  }

  $effect(() => {
    const replyThreadId = chat.openReplyThreadId;
    if (executionThreadScope === replyThreadId) return;
    executionThreadScope = replyThreadId;
    executionSelectionRequest += 1;
    actionError = null;
  });

  $effect(() => {
    const agentRun = executionRun;
    if (!agentRun || chat.selectedExecutionRunId === agentRun.id) return;
    const request = ++executionSelectionRequest;
    actionError = null;
    void chat.selectAssignmentExecution(agentRun.id)
      .catch((cause: unknown) => {
        if (request === executionSelectionRequest) {
          actionError = cause instanceof Error ? cause.message : String(cause);
        }
      });
  });

  $effect(() => {
    const element = scroller;
    const key = destination;
    const presentationReady = executionPresentationReady;
    if (!element || !presentationReady) return;
    restoredDestination = null;
    void tick().then(() => {
      if (element !== scroller || key !== destination) return;
      element.scrollTop = chat.organizationalScrollPositions[key] ?? element.scrollHeight;
      followingEnd = organizationalScrollFollowsEnd(element);
      restoredDestination = key;
    });
  });

  $effect(() => {
    const element = scroller;
    const key = destination;
    if (!element || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(() => {
      if (restoredDestination !== key || !followingEnd) return;
      element.scrollTop = element.scrollHeight;
      chat.setOrganizationalScrollPosition(key, element.scrollTop);
    });
    observer.observe(element);
    return () => observer.disconnect();
  });

  $effect(() => {
    page;
    renderEntries;
    chat.timelineItems;
    chat.timelinePages;
    followNewContentIfAtBottom();
  });

  $effect(() => {
    const element = scroller;
    const anchorId = chat.messageAnchorId;
    const presentationReady = executionPresentationReady;
    if (!element || !anchorId || !chat.openReplyThreadId || !presentationReady) return;
    void tick().then(() => {
      const target = element.querySelector<HTMLElement>(`[data-message-item-id="${CSS.escape(anchorId)}"]`);
      if (!target) return;
      target.scrollIntoView({ block: "center" });
      target.focus({ preventScroll: true });
      chat.clearMessageAnchor(anchorId);
    });
  });
</script>

<section
  class="reply-thread-panel"
  role={presentation === "dialog" ? "dialog" : presentation === "complementary" ? "complementary" : undefined}
  aria-modal={presentation === "dialog" ? "true" : undefined}
  aria-label={t("chat.organization.thread")}
  data-presentation={presentation}
  data-chat-reply-thread
>
  <header class="thread-header" class:reserve-global-actions={reserveGlobalActions}>
    <div class="thread-tab-shell" role="tablist">
      <button type="button" class="thread-tab" data-thread-tab role="tab" aria-selected="true" tabindex="0" onmousedown={preventMiddleButtonScroll} onauxclick={closeFromMiddleClick}>
        <MessagesSquare size={13} /><strong>{t("chat.organization.thread")}</strong>
      </button>
      <button type="button" class="tab-close" data-thread-close aria-label={t("chat.organization.closeThread")} onmousedown={preventMiddleButtonScroll} onauxclick={closeFromMiddleClick} onclick={onClose}><X size={11} /></button>
    </div>
    <span></span>
  </header>

  {#if actionError}<p class="thread-error" role="alert">{actionError}</p>{/if}

  <div
    bind:this={scroller}
    class="thread-scroll"
    aria-busy={!executionPresentationReady || chat.replyThreadLoading || undefined}
    onscroll={rememberScrollPosition}
  >
    {#if chat.replyThreadError}<p class="thread-error" role="alert">{chat.replyThreadError}</p>{/if}
    {#if page && executionPresentationReady}
      {#if page.previousCursor}<button type="button" class="load-older" onclick={() => void chat.loadOlderReplyThreadMessages()}>{t("chat.organization.loadOlder")}</button>{/if}
      <div class="date-divider"><span>{formatDateTime(localization.locale, Date.parse(page.rootMessage.createdAt), { dateStyle: "full" })}</span></div>
      <ChatOrganizationalMessage message={page.rootMessage} showReplyStrip={false} />
      {#each renderEntries as entry, index (entry.key)}
        {@const previousEntry = renderEntries[index - 1]}
        {@const previousCreatedAt = previousEntry ? entryCreatedAt(previousEntry) : page.rootMessage.createdAt}
        {#if !sameDay(previousCreatedAt, entryCreatedAt(entry))}
          <div class="date-divider"><span>{formatDateTime(localization.locale, Date.parse(entryCreatedAt(entry)), { dateStyle: "full" })}</span></div>
        {/if}
        {#if entry.kind === "message"}
          <ChatOrganizationalMessage
            message={entry.message}
            showReplyStrip={false}
            grouped={previousEntry?.kind === "message" && shouldGroupReplyMessages(previousEntry.message, entry.message)}
          />
        {:else if chat.selectedExecutionRunId && entry.run.providerExecutionThreadId === chat.selectedThreadId}
          <ChatExecutionTimeline
            embedded
            hideUserMessages
            teammateName={assignment?.teammate.displayName ?? null}
            turnId={entry.run.providerExecutionTurnId}
          />
        {/if}
      {/each}
    {:else if chat.replyThreadLoading || page}
      <p class="thread-loading" role="status">{t("common.loading")}</p>
    {/if}
  </div>

  {#if page && !chat.selectedChannel?.archivedAt}
    {#if chat.interaction?.pendingRequest}<div class="thread-request"><ChatRequestPanel pending={chat.interaction.pendingRequest} /></div>{/if}
    <div class="thread-composer"><ChatMessageComposer {destination} threadComposer placeholder={t("chat.organization.replyInThread")} onRequestScrollToBottom={scrollToBottomForUserAction} /></div>
  {/if}
</section>

<style>
  .reply-thread-panel { display:flex; width:100%; height:100%; min-height:0; flex-direction:column; background:var(--cal-bg); }
  .thread-header { display:flex; height:var(--cal-header-row-h); min-height:var(--cal-header-row-h); flex:0 0 auto; align-items:center; gap:0.2rem; border-bottom:1px solid var(--sidebar); background:var(--cal-header-bg); padding-inline:0.45rem; }
  .thread-header.reserve-global-actions { padding-right:var(--chat-global-actions-width); }
  .thread-tab-shell { display:flex; width:9.5rem; min-width:3.75rem; height:2rem; flex:0 1 9.5rem; align-items:stretch; overflow:hidden; border-radius:0.55rem; background:var(--accent); color:var(--foreground); user-select:none; }
  .thread-tab { display:flex; min-width:0; flex:1 1 auto; align-items:center; gap:0.4rem; overflow:hidden; padding:0.3rem 0.2rem 0.3rem 0.65rem; text-align:left; user-select:none; }
  :global(html[data-focus-intent="keyboard"]) .thread-tab:focus { outline:none; }
  :global(html[data-focus-intent="keyboard"]) .thread-tab-shell:has(.thread-tab:focus) { box-shadow:inset 0 0 0 2px var(--ring); }
  .thread-tab :global(svg) { flex:0 0 auto; }.thread-tab strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:0.733333rem; font-weight:500; user-select:none; }
  .tab-close { display:grid; width:1.5rem; flex:0 0 auto; place-items:center; border-radius:0.3rem; opacity:0; }
  .thread-tab-shell:hover .tab-close, .tab-close:focus-visible { opacity:1; }
  .tab-close:hover { background:var(--accent); }
  .thread-header > span { flex:1; }
  .thread-scroll { min-height:0; flex:1; overflow-y:auto; overscroll-behavior:contain; padding-block:0.4rem; }
  .date-divider { display:flex; align-items:center; gap:0.5rem; margin:0.75rem; color:var(--muted-foreground); font-size:0.65rem; }
  .date-divider::before,.date-divider::after { height:1px; flex:1; background:var(--border); content:""; }
  .thread-request { flex:0 0 auto; border-top:1px solid var(--border); padding:0.5rem; }
  .thread-composer { display:flex; flex:0 0 auto; justify-content:center; padding:0.5rem 0.5rem 0.75rem; }
  .thread-error,.thread-loading { padding:0.6rem; color:var(--destructive); font-size:0.7rem; }.thread-loading { color:var(--muted-foreground); }
  .load-older { display:block; margin:0.3rem auto 0.6rem; border-radius:0.4rem; padding:0.3rem 0.5rem; color:var(--muted-foreground); font-size:0.68rem; }.load-older:hover { background:var(--accent); }
  @media (hover:none) { .tab-close { opacity:1; } }
</style>
