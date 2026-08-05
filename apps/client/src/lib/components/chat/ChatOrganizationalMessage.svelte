<script lang="ts">
  import Copy from "@lucide/svelte/icons/copy";
  import MessageCircle from "@lucide/svelte/icons/message-circle";
  import SmilePlus from "@lucide/svelte/icons/smile-plus";
  import { onDestroy } from "svelte";
  import type IconPicker from "$lib/components/icon-picker/IconPicker.svelte";
  import type { ChatMessageRead, ChatWorkAssignmentState } from "$lib/chat/contracts";
  import type { ChatMessageReaction } from "$lib/chat/organizational-message-model";
  import { chatParticipantDisplayName } from "$lib/chat/participant-display";
  import { LOCAL_CHAT_PARTICIPANT_ID } from "$lib/chat/participant-display";
  import { formatDateTime, formatList } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { writeTextToClipboard } from "$lib/utils/clipboard";
  import ChatParticipantAvatar from "./ChatParticipantAvatar.svelte";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";

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
  const chat = getChat();
  const preferences = getPreferences();
  let copied = $state(false);
  let copiedResetTimer: ReturnType<typeof setTimeout> | null = null;
  let ReactionPicker = $state<typeof IconPicker | null>(null);
  let reactionPickerValue = $state("emoji:👍");
  const authorDisplayName = $derived(chatParticipantDisplayName(
    message.author,
    preferences.profileDisplayName,
    t("chat.timeline.you"),
  ));
  const localDisplayName = $derived(preferences.profileDisplayName.trim() || t("chat.timeline.you"));
  const reactionMessageKey = $derived(`organizational:${message.itemId}`);
  const reactions = $derived(chat.messageReactionsFor(reactionMessageKey));

  async function copyMessage(): Promise<void> {
    try {
      await writeTextToClipboard(message.normalizedMarkdown);
      copied = true;
      if (copiedResetTimer) clearTimeout(copiedResetTimer);
      copiedResetTimer = setTimeout(() => {
        copied = false;
        copiedResetTimer = null;
      }, 2_000);
    } catch (error: unknown) {
      console.error("Could not copy Chat message", error);
    }
  }

  async function openReactionPicker(): Promise<void> {
    if (ReactionPicker) return;
    try {
      ReactionPicker = (await import("$lib/components/icon-picker/IconPicker.svelte")).default;
    } catch (error: unknown) {
      console.error("Could not load the Chat reaction picker", error);
    }
  }

  function toggleReaction(value: string): void {
    reactionPickerValue = value;
    chat.toggleSessionMessageReaction(reactionMessageKey, value, localDisplayName);
  }

  function localUserSelected(reaction: ChatMessageReaction): boolean {
    return reaction.participants.some(({ participantId }) => participantId === LOCAL_CHAT_PARTICIPANT_ID);
  }

  function reactionTooltip(reaction: ChatMessageReaction): string {
    const names = reaction.participants.map((participant) => participant.participantId === LOCAL_CHAT_PARTICIPANT_ID
      ? localDisplayName
      : participant.displayName);
    return t("chat.organization.reactedBy", formatList(localization.locale, names), names.length);
  }

  onDestroy(() => {
    if (copiedResetTimer) clearTimeout(copiedResetTimer);
  });

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
    <div class="message-actions">
      {#if reactions.length > 0}
        <div class="reaction-list">
          {#each reactions as reaction (reaction.value)}
            <button
              type="button"
              class="reaction-chip"
              class:selected={localUserSelected(reaction)}
              aria-pressed={localUserSelected(reaction)}
              aria-label={reactionTooltip(reaction)}
              data-app-tooltip={reactionTooltip(reaction)}
              onclick={() => toggleReaction(reaction.value)}
            >
              <ProjectIcon name={reaction.value} size={12} />
              <span>{reaction.participants.length}</span>
            </button>
          {/each}
        </div>
      {/if}
      <div class="message-action-controls">
        {#if ReactionPicker}
          <ReactionPicker
            value={reactionPickerValue}
            onChange={toggleReaction}
            ariaLabel={t("chat.organization.addReaction")}
            showIcons={false}
            showUpload={false}
            showRemove={false}
            initiallyOpen
          >
            {#snippet trigger({ open, toggle })}
              <button
                type="button"
                class="message-action-button"
                class:active={open}
                aria-label={t("chat.organization.addReaction")}
                aria-expanded={open}
                data-app-tooltip={t("chat.organization.addReaction")}
                onclick={toggle}
              ><SmilePlus size={14} /></button>
            {/snippet}
          </ReactionPicker>
        {:else}
          <button
            type="button"
            class="message-action-button"
            aria-label={t("chat.organization.addReaction")}
            data-app-tooltip={t("chat.organization.addReaction")}
            onclick={() => void openReactionPicker()}
          ><SmilePlus size={14} /></button>
        {/if}
        <button
          type="button"
          class="message-action-button"
          class:copied
          aria-label={copied ? t("chat.timeline.copied") : t("chat.timeline.copy")}
          title={copied ? t("chat.timeline.copied") : t("chat.timeline.copy")}
          onclick={() => void copyMessage()}
        ><Copy size={13} /></button>
      </div>
    </div>
  </div>
</article>

<style>
  .message-row { display:grid; grid-template-columns:2.25rem minmax(0,1fr); gap:0.65rem; padding:0.4rem 1rem; outline:none; }
  .message-row:focus-visible { border-radius:0.45rem; box-shadow:inset 0 0 0 2px var(--ring); }
  .message-row.grouped { padding-top:0.08rem; }
  .avatar-cell { min-height:1px; }
  .message-body { min-width:0; max-width:54rem; }
  header { display:flex; min-height:1.3rem; align-items:baseline; gap:0.4rem; }
  header strong { font-size:0.82rem; } header time { color:var(--muted-foreground); font-size:0.65rem; }
  .agent-badge { border:1px solid var(--border); border-radius:999px; padding:0.05rem 0.3rem; color:var(--muted-foreground); font-size:0.58rem; }
  .message-copy { white-space:pre-wrap; overflow-wrap:anywhere; color:var(--foreground); font-size:0.84rem; line-height:1.42rem; }
  .message-context { display:flex; flex-wrap:wrap; gap:0.3rem; margin-top:0.3rem; }
  .message-context span { border-radius:999px; background:var(--accent); padding:0.15rem 0.4rem; color:var(--muted-foreground); font-size:0.65rem; }
  .reaction-list { display:flex; max-width:100%; flex-wrap:wrap; gap:0.25rem; }
  .reaction-chip { display:inline-flex; min-width:2.35rem; height:1.5rem; align-items:center; justify-content:center; gap:0.3rem; border:1px solid transparent; border-radius:0.4rem; background:color-mix(in srgb,var(--accent) 76%,transparent); padding-inline:0.35rem; color:var(--muted-foreground); font-size:0.7rem; font-variant-numeric:tabular-nums; }
  .reaction-chip:hover,.reaction-chip:focus-visible { border-color:var(--border); background:var(--accent); color:var(--foreground); }
  .reaction-chip.selected { border-color:color-mix(in srgb,var(--primary) 60%,var(--border)); background:color-mix(in srgb,var(--primary) 13%,transparent); color:var(--primary); }
  .message-actions { display:flex; width:fit-content; max-width:100%; flex-wrap:wrap; align-items:center; gap:0.3rem; margin-top:0.35rem; }
  .message-action-controls { display:flex; opacity:0; transition:opacity 150ms ease; }
  .message-row:hover .message-action-controls,.message-row:focus-within .message-action-controls { opacity:1; }
  .message-action-button { display:inline-grid; width:1.75rem; height:1.75rem; place-items:center; border-radius:0.4rem; color:var(--muted-foreground); }
  .message-action-button:hover,.message-action-button:focus-visible,.message-action-button.active,.message-action-button.copied { background:var(--accent); color:var(--foreground); }
  .reply-strip { display:flex; width:min(100%,46rem); min-height:2rem; align-items:center; gap:0.4rem; margin-top:0.35rem; border-radius:0.4rem; color:color-mix(in srgb,var(--primary) 70%,var(--foreground)); font-size:0.68rem; text-align:left; }
  .reply-strip:hover,.reply-strip:focus-visible { background:var(--accent); }
  .reply-avatars { display:flex; padding-left:0.2rem; }.reply-avatars :global(.participant-avatar + .participant-avatar) { margin-left:-0.35rem; }
  .work-state { margin-left:auto; border-radius:999px; background:var(--accent); padding:0.12rem 0.38rem; color:var(--foreground); white-space:nowrap; }
  .work-state[data-state^="waiting"],.work-state[data-state="ready_for_review"] { color:var(--destructive); }
  .unread-dot { width:0.42rem; height:0.42rem; flex:0 0 auto; border-radius:999px; background:var(--primary); }
  @media (hover:none) { .message-action-controls { opacity:1; } }
</style>
