<script lang="ts">
  import MessageCircle from "@lucide/svelte/icons/message-circle";
  import type { ChatMessageRead, ChatWorkAssignmentState } from "$lib/chat/contracts";
  import { organizationalMessageActionTarget } from "$lib/chat/message-action-target";
  import { chatParticipantDisplayName } from "$lib/chat/participant-display";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import ChatMessageActionToolbar from "./ChatMessageActionToolbar.svelte";
  import ChatMessageReactionList from "./ChatMessageReactionList.svelte";
  import ChatParticipantAvatar from "./ChatParticipantAvatar.svelte";

  let {
    message,
    grouped = false,
    showReplyStrip = true,
    onOpenThread = () => undefined,
  }: {
    message: ChatMessageRead;
    grouped?: boolean;
    showReplyStrip?: boolean;
    onOpenThread?: (trigger: HTMLElement) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const preferences = getPreferences();
  let actionToolbarVisible = $state(false);
  const authorDisplayName = $derived(chatParticipantDisplayName(
    message.author,
    preferences.profileDisplayName,
    t("chat.timeline.you"),
  ));
  const actionTarget = $derived(organizationalMessageActionTarget(message));

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
</script>

<article
  class="message-row"
  class:grouped
  data-message-item-id={message.itemId}
  tabindex="-1"
  aria-label={`${authorDisplayName}, ${formatDateTime(localization.locale, Date.parse(message.createdAt), { dateStyle: "medium", timeStyle: "short" })}`}
  onpointerenter={() => { actionToolbarVisible = true; }}
  onpointerleave={() => { actionToolbarVisible = false; }}
  onfocusin={() => { actionToolbarVisible = true; }}
  onfocusout={(event) => {
    if (!(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget)) {
      actionToolbarVisible = false;
    }
  }}
>
  <div class="avatar-cell">
    {#if !grouped}<ChatParticipantAvatar participant={message.author} size={32} />{/if}
  </div>
  <div class="message-body">
    {#if !grouped}
      <header>
        <strong>{authorDisplayName}</strong>
        {#if message.author.kind === "ai_teammate"}<span class="agent-badge">{t("chat.organization.agent")}</span>{/if}
        <time datetime={message.createdAt}>{formatDateTime(localization.locale, Date.parse(message.createdAt), { timeStyle: "short" })}</time>
      </header>
    {/if}
    <div class="message-copy">{message.normalizedMarkdown}</div>
    {#if message.resourceReferences.length > 0 || message.attachmentIds.length > 0}
      <div class="message-context">
        {#each message.resourceReferences as reference (`${reference.kind}:${reference.relativePath}`)}<span>{reference.kind === "folder" ? "▣" : "▤"} {reference.displayLabel}</span>{/each}
        {#if message.attachmentIds.length > 0}<span>{t("chat.organization.images", message.attachmentIds.length)}</span>{/if}
      </div>
    {/if}
    {#if showReplyStrip && message.replyThread}
      <button type="button" class="reply-strip" data-reply-thread-id={message.replyThread.id} onclick={(event) => onOpenThread(event.currentTarget)}>
        <span class="reply-avatars">
          {#each message.replyThread.participants.slice(0, 3) as participant (participant.id)}<ChatParticipantAvatar {participant} size={20} />{/each}
        </span>
        <MessageCircle size={13} />
        <span>{t("chat.organization.replies", message.replyThread.replyCount)}</span>
        {#if message.replyThread.workState}<span class="work-state" data-state={message.replyThread.workState}>{stateLabel(message.replyThread.workState)}</span>{/if}
        {#if message.replyThread.unread}<span class="unread-dot" aria-label={t("chat.status.unread")}></span>{/if}
      </button>
    {/if}
    <ChatMessageReactionList target={actionTarget} />
    <ChatMessageActionToolbar
      target={actionTarget}
      visible={actionToolbarVisible}
      placement={grouped ? "grouped-message" : "participant-header"}
    />
  </div>
</article>

<style>
  .message-row { display:grid; grid-template-columns:2.25rem minmax(0,1fr); gap:0.65rem; padding:var(--chat-conversation-entry-space,0.45rem) 1rem; outline:none; }
  .message-row:focus-visible { border-radius:0.45rem; box-shadow:inset 0 0 0 2px var(--ring); }
  .message-row.grouped { padding-top:0.08rem; }
  .avatar-cell { min-height:1px; }
  .message-body { --chat-message-action-anchor-bottom:1.3rem; --chat-message-reaction-margin-top:0.35rem; position:relative; min-width:0; max-width:54rem; }
  header { display:flex; min-height:1.3rem; align-items:baseline; gap:0.4rem; }
  header strong { font-size:var(--chat-organizational-font-size,0.84rem); } header time { color:var(--muted-foreground); font-size:var(--chat-organizational-time-font-size,0.65rem); }
  .agent-badge { border:1px solid var(--border); border-radius:999px; padding:0.05rem 0.3rem; color:var(--muted-foreground); font-size:0.58rem; }
  .message-copy { white-space:pre-wrap; overflow-wrap:anywhere; color:var(--foreground); font-size:var(--chat-organizational-font-size,0.84rem); line-height:var(--chat-organizational-line-height,1.42rem); }
  .message-context { display:flex; flex-wrap:wrap; gap:0.3rem; margin-top:0.3rem; }
  .message-context span { border-radius:999px; background:var(--accent); padding:0.15rem 0.4rem; color:var(--muted-foreground); font-size:0.65rem; }
  .reply-strip { display:flex; width:min(100%,46rem); min-height:2rem; align-items:center; gap:0.4rem; margin-top:0.35rem; border-radius:0.4rem; color:color-mix(in srgb,var(--primary) 70%,var(--foreground)); font-size:0.68rem; text-align:left; }
  .reply-strip:hover,.reply-strip:focus-visible { background:var(--accent); }
  .reply-avatars { display:flex; padding-left:0.2rem; }.reply-avatars :global(.participant-avatar + .participant-avatar) { margin-left:-0.35rem; }
  .work-state { margin-left:auto; border-radius:999px; background:var(--accent); padding:0.12rem 0.38rem; color:var(--foreground); white-space:nowrap; }
  .work-state[data-state^="waiting"],.work-state[data-state="ready_for_review"] { color:var(--destructive); }
  .unread-dot { width:0.42rem; height:0.42rem; flex:0 0 auto; border-radius:999px; background:var(--primary); }
</style>
