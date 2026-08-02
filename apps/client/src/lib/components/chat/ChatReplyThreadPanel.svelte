<script lang="ts">
  import CircleStop from "@lucide/svelte/icons/circle-stop";
  import MessagesSquare from "@lucide/svelte/icons/messages-square";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import X from "@lucide/svelte/icons/x";
  import { tick } from "svelte";
  import type { ChatWorkAssignmentState } from "$lib/chat/contracts";
  import {
    latestRenderableAgentRun,
    repliesWithoutRenderedRunProjection,
  } from "$lib/chat/reply-thread-model";
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
  const { t } = getLocalization();
  let scroller = $state<HTMLDivElement | null>(null);
  let actionError = $state<string | null>(null);
  let executionLoading = $state(false);
  let executionSelectionRequest = 0;
  let executionThreadScope = $state<string | null>(null);
  const page = $derived(chat.replyThread);
  const assignment = $derived(page?.assignment ?? null);
  const executionRun = $derived(latestRenderableAgentRun(page?.agentRuns ?? []));
  const visibleReplies = $derived(repliesWithoutRenderedRunProjection(
    page?.replies ?? [],
    executionRun?.id ?? null,
  ));
  const destination = $derived(chat.openReplyThreadId
    ? `reply-thread:${chat.openReplyThreadId}`
    : "reply-thread:none");
  const canCancel = $derived(assignment && [
    "queued", "working", "waiting_for_answer", "waiting_for_approval",
  ].includes(assignment.state));

  function stateLabel(state: ChatWorkAssignmentState): string {
    const labels: Record<ChatWorkAssignmentState, string> = {
      queued: t("chat.organization.queued"),
      working: t("chat.status.working"),
      waiting_for_answer: t("chat.status.waitingAnswer"),
      waiting_for_approval: t("chat.status.waitingApproval"),
      ready_for_review: t("chat.status.readyForReview"),
      completed: t("chat.organization.completed"),
      failed: t("chat.organization.failed"),
      cancelled: t("chat.organization.cancelled"),
    };
    return labels[state];
  }

  function run(action: () => Promise<unknown>): void {
    actionError = null;
    void action().catch((cause: unknown) => {
      actionError = cause instanceof Error ? cause.message : String(cause);
    });
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

  $effect(() => {
    const replyThreadId = chat.openReplyThreadId;
    if (executionThreadScope === replyThreadId) return;
    executionThreadScope = replyThreadId;
    executionSelectionRequest += 1;
    executionLoading = false;
    actionError = null;
  });

  $effect(() => {
    const agentRun = executionRun;
    if (!agentRun || chat.selectedExecutionRunId === agentRun.id) return;
    const request = ++executionSelectionRequest;
    executionLoading = true;
    actionError = null;
    void chat.selectAssignmentExecution(agentRun.id)
      .catch((cause: unknown) => {
        if (request === executionSelectionRequest) {
          actionError = cause instanceof Error ? cause.message : String(cause);
        }
      })
      .finally(() => {
        if (request === executionSelectionRequest) executionLoading = false;
      });
  });

  $effect(() => {
    const element = scroller;
    const key = destination;
    if (!element) return;
    void tick().then(() => { element.scrollTop = chat.organizationalScrollPositions[key] ?? 0; });
  });

  $effect(() => {
    const element = scroller;
    const anchorId = chat.messageAnchorId;
    if (!element || !anchorId || !chat.openReplyThreadId) return;
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

  {#if assignment && (assignment.state === "failed" || canCancel)}
    <div class="work-banner" data-state={assignment.state}>
      <span><strong>{assignment.teammate.displayName}</strong> · {stateLabel(assignment.state)}</span>
      <div>
        {#if assignment.state === "failed"}<button type="button" onclick={() => run(() => chat.retryAssignment(assignment.id))}><RotateCcw size={13} />{t("chat.organization.retry")}</button>{/if}
        {#if canCancel}<button type="button" onclick={() => run(() => chat.cancelAssignment(assignment.id))}><CircleStop size={13} />{t("chat.organization.cancelWork")}</button>{/if}
      </div>
    </div>
  {/if}
  {#if actionError}<p class="thread-error" role="alert">{actionError}</p>{/if}

  <div
    bind:this={scroller}
    class="thread-scroll"
    onscroll={() => { if (scroller) chat.setOrganizationalScrollPosition(destination, scroller.scrollTop); }}
  >
    {#if chat.replyThreadError}<p class="thread-error" role="alert">{chat.replyThreadError}</p>{/if}
    {#if page}
      {#if page.previousCursor}<button type="button" class="load-older" onclick={() => void chat.loadOlderReplyThreadMessages()}>{t("chat.organization.loadOlder")}</button>{/if}
      <div class="root-message"><ChatOrganizationalMessage message={page.rootMessage} showReplyStrip={false} /></div>
      <div class="reply-divider"><span>{t("chat.organization.replies", page.thread.replyCount)}</span></div>
      {#each visibleReplies as reply, index (reply.itemId)}
        <ChatOrganizationalMessage message={reply} showReplyStrip={false} grouped={index > 0 && visibleReplies[index - 1].author.id === reply.author.id} />
      {/each}
      {#if executionLoading && chat.selectedExecutionRunId !== executionRun?.id}
        <p class="thread-loading" role="status">{t("common.loading")}</p>
      {:else if executionRun && chat.selectedExecutionRunId === executionRun.id}
        <ChatExecutionTimeline
          embedded
          hideUserMessages
          teammateName={assignment?.teammate.displayName ?? null}
          effort={executionRun.effort}
        />
      {/if}
    {:else if chat.replyThreadLoading}
      <p class="thread-loading" role="status">{t("common.loading")}</p>
    {/if}
  </div>

  {#if page && !chat.selectedChannel?.archivedAt}
    {#if chat.interaction?.pendingRequest}<div class="thread-request"><ChatRequestPanel pending={chat.interaction.pendingRequest} /></div>{/if}
    <div class="thread-composer"><ChatMessageComposer {destination} threadComposer placeholder={t("chat.organization.replyInThread")} /></div>
  {/if}
</section>

<style>
  .reply-thread-panel { display:flex; width:100%; height:100%; min-height:0; flex-direction:column; background:var(--cal-bg); }
  .thread-header { display:flex; height:var(--cal-header-row-h); min-height:var(--cal-header-row-h); flex:0 0 auto; align-items:center; gap:0.2rem; border-bottom:1px solid var(--sidebar); background:var(--cal-header-bg); padding-inline:0.45rem; }
  .thread-header.reserve-global-actions { padding-right:var(--chat-global-actions-width); }
  .thread-tab-shell { display:flex; width:9.5rem; min-width:3.75rem; height:2rem; flex:0 1 9.5rem; align-items:stretch; overflow:hidden; border-radius:0.55rem; background:var(--accent); color:var(--foreground); user-select:none; }
  .thread-tab { display:flex; min-width:0; flex:1 1 auto; align-items:center; gap:0.4rem; overflow:hidden; padding:0.3rem 0.2rem 0.3rem 0.65rem; text-align:left; user-select:none; }
  .thread-tab :global(svg) { flex:0 0 auto; }.thread-tab strong { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:0.733333rem; font-weight:500; user-select:none; }
  .tab-close { display:grid; width:1.5rem; flex:0 0 auto; place-items:center; border-radius:0.3rem; opacity:0; }
  .thread-tab-shell:hover .tab-close, .tab-close:focus-visible { opacity:1; }
  .tab-close:hover { background:var(--accent); }
  .thread-header > span { flex:1; }
  .work-banner { display:flex; flex:0 0 auto; flex-wrap:wrap; align-items:center; justify-content:space-between; gap:0.4rem; border-bottom:1px solid var(--border); background:color-mix(in srgb,var(--accent) 62%,transparent); padding:0.45rem 0.7rem; font-size:0.7rem; }
  .work-banner > div { display:flex; flex-wrap:wrap; gap:0.25rem; }.work-banner button { display:flex; align-items:center; gap:0.25rem; border-radius:0.35rem; padding:0.25rem 0.4rem; }.work-banner button:hover { background:var(--accent); }
  .thread-scroll { min-height:0; flex:1; overflow-y:auto; overscroll-behavior:contain; padding-block:0.4rem; }
  .root-message { padding-block:0.35rem; background:color-mix(in srgb,var(--accent) 28%,transparent); }
  .reply-divider { display:flex; align-items:center; gap:0.45rem; margin:0.7rem; color:var(--muted-foreground); font-size:0.65rem; }.reply-divider::before,.reply-divider::after { height:1px; flex:1; background:var(--border); content:""; }
  .thread-request { flex:0 0 auto; border-top:1px solid var(--border); padding:0.5rem; }
  .thread-composer { display:flex; flex:0 0 auto; justify-content:center; border-top:1px solid color-mix(in srgb,var(--border) 55%,transparent); padding:0.5rem; }
  .thread-error,.thread-loading { padding:0.6rem; color:var(--destructive); font-size:0.7rem; }.thread-loading { color:var(--muted-foreground); }
  .load-older { display:block; margin:0.3rem auto 0.6rem; border-radius:0.4rem; padding:0.3rem 0.5rem; color:var(--muted-foreground); font-size:0.68rem; }.load-older:hover { background:var(--accent); }
  @media (hover:none) { .tab-close { opacity:1; } }
</style>
