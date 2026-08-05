<script lang="ts">
  import Bot from "@lucide/svelte/icons/bot";
  import X from "@lucide/svelte/icons/x";
  import { tick } from "svelte";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { unreadMessageStartIndex } from "$lib/chat/organizational-message-model";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatMessageComposer from "./ChatMessageComposer.svelte";
  import ChatOrganizationalMessage from "./ChatOrganizationalMessage.svelte";

  let { onThreadOpened = () => undefined }: { onThreadOpened?: (trigger: HTMLElement) => void } = $props();
  const chat = getChat();
  const localization = getLocalization();
  const { t } = localization;
  let feed = $state<HTMLDivElement | null>(null);
  let teammateSetupDismissed = $state(false);
  const channel = $derived(chat.selectedChannel);
  const destination = $derived(channel ? `channel:${channel.id}` : "channel:none");
  const unreadStart = $derived(unreadMessageStartIndex(
    chat.channelMessages,
    channel?.unreadCount ?? 0,
  ));

  function sameDay(left: string, right: string): boolean {
    return new Date(left).toDateString() === new Date(right).toDateString();
  }

  function grouped(index: number): boolean {
    const current = chat.channelMessages[index];
    const previous = chat.channelMessages[index - 1];
    if (!current || !previous) return false;
    return current.author.id === previous.author.id
      && sameDay(current.createdAt, previous.createdAt)
      && Date.parse(current.createdAt) - Date.parse(previous.createdAt) < 5 * 60_000;
  }

  async function openThread(threadId: string, trigger: HTMLElement): Promise<void> {
    onThreadOpened(trigger);
    await chat.openReplyThread(threadId);
  }

  $effect(() => {
    const element = feed;
    const key = destination;
    if (!element) return;
    void tick().then(() => { element.scrollTop = chat.organizationalScrollPositions[key] ?? 0; });
  });

  $effect(() => {
    const element = feed;
    const anchorId = chat.messageAnchorId;
    if (!element || !anchorId || chat.openReplyThreadId) return;
    void tick().then(() => {
      const target = element.querySelector<HTMLElement>(`[data-message-item-id="${CSS.escape(anchorId)}"]`);
      if (!target) return;
      target.scrollIntoView({ block: "center" });
      target.focus({ preventScroll: true });
      chat.clearMessageAnchor(anchorId);
    });
  });
</script>

<section class="channel-surface" aria-label={channel ? `#${channel.name}` : t("chat.title")}>
  <div
    bind:this={feed}
    class="channel-feed"
    role="feed"
    aria-busy={chat.channelMessagesLoading}
    onpointerdown={() => void chat.markSelectedChannelRead()}
    onscroll={() => { if (feed) chat.setOrganizationalScrollPosition(destination, feed.scrollTop); }}
  >
    <div class="feed-canvas">
      {#if channel}
        <header class="channel-introduction">
          <h1>#{channel.name}</h1>
          <p>{channel.topic || t("chat.channels.welcomeDescription")}</p>
        </header>
      {/if}
      {#if !teammateSetupDismissed && chat.teammates.length === 0}
        <section class="first-teammate" aria-label={t("chat.organization.firstTeammateTitle")}>
          <Bot size={19} />
          <div>
            <strong>{t("chat.organization.firstTeammateTitle")}</strong>
            <p>{t("chat.organization.firstTeammateDescription")}</p>
          </div>
          <button type="button" class="setup-teammate" onclick={() => window.dispatchEvent(new Event("ganbaru-ai:chat-manage-members"))}>{t("chat.organization.firstTeammateAction")}</button>
          <button type="button" class="dismiss-setup" aria-label={t("chat.organization.dismissFirstTeammate")} onclick={() => { teammateSetupDismissed = true; }}><X size={14} /></button>
        </section>
      {/if}
      {#if chat.channelMessagesError}<p class="feed-error" role="alert">{chat.channelMessagesError}</p>{/if}
      {#if chat.channelPages[0]?.previousCursor}<button type="button" class="load-older" onclick={() => void chat.loadOlderChannelMessages()}>{t("chat.organization.loadOlder")}</button>{/if}
      {#each chat.channelMessages as message, index (message.itemId)}
        {#if index === unreadStart && (channel?.unreadCount ?? 0) > 0}<div class="divider unread"><span>{t("chat.organization.unreadMessages")}</span></div>{/if}
        {#if index === 0 || !sameDay(chat.channelMessages[index - 1].createdAt, message.createdAt)}
          <div class="divider" class:first-date={index === 0}><span>{formatDateTime(localization.locale, Date.parse(message.createdAt), { dateStyle: "full" })}</span></div>
        {/if}
        <ChatOrganizationalMessage {message} grouped={grouped(index)} onOpenThread={(trigger) => { if (message.replyThread) void openThread(message.replyThread.id, trigger); }} />
      {/each}
      {#if chat.channelMessagesLoading && chat.channelMessages.length === 0}<p class="loading" role="status">{t("common.loading")}</p>{/if}
    </div>
  </div>
  {#if channel && !channel.archivedAt}
    <div class="channel-composer-dock"><ChatMessageComposer {destination} placeholder={t("chat.organization.messageChannel", channel.name)} /></div>
  {/if}
</section>

<style>
  .channel-surface { display:flex; min-height:0; flex:1; flex-direction:column; overflow:hidden; }
  .channel-feed { min-height:0; flex:1; overflow-y:auto; overscroll-behavior:contain; }
  .feed-canvas { --channel-feed-section-space:1.2rem; width:min(100%,54rem); min-height:100%; margin-inline:auto; padding:0 0.75rem 1rem; }
  .channel-introduction { max-width:54rem; padding:var(--channel-feed-section-space) 1rem 0; }
  .channel-introduction h1 { font-size:1.25rem; font-weight:700; }
  .channel-introduction p { margin-top:0.3rem; color:var(--muted-foreground); font-size:0.82rem; line-height:1.35rem; }
  .first-teammate { display:grid; grid-template-columns:auto minmax(0,1fr) auto auto; align-items:center; gap:0.65rem; margin:0 1rem 1rem; border:1px solid var(--border); border-radius:0.75rem; background:color-mix(in srgb,var(--card) 72%,transparent); padding:0.7rem 0.75rem; }
  .first-teammate > :global(svg) { color:var(--muted-foreground); }.first-teammate strong { font-size:0.78rem; }.first-teammate p { margin-top:0.12rem; color:var(--muted-foreground); font-size:0.7rem; line-height:1.05rem; }
  .setup-teammate { min-height:1.9rem; border-radius:0.45rem; background:var(--primary); padding-inline:0.65rem; color:var(--primary-foreground); font-size:0.7rem; font-weight:600; }.dismiss-setup { display:grid; width:1.8rem; height:1.8rem; place-items:center; border-radius:0.4rem; color:var(--muted-foreground); }.dismiss-setup:hover { background:var(--accent); color:var(--foreground); }
  .divider { display:flex; align-items:center; gap:0.5rem; margin:0.75rem 0; color:var(--muted-foreground); font-size:0.65rem; }
  .divider.first-date { margin-top:var(--channel-feed-section-space); margin-bottom:calc(var(--channel-feed-section-space) - var(--chat-conversation-entry-space,0.45rem)); }
  .divider::before,.divider::after { height:1px; flex:1; background:var(--border); content:""; }.divider.unread { color:var(--primary); }.divider.unread::before,.divider.unread::after { background:color-mix(in srgb,var(--primary) 55%,var(--border)); }
  .load-older { display:block; margin:0.6rem auto; border-radius:0.4rem; padding:0.3rem 0.55rem; color:var(--muted-foreground); font-size:0.7rem; }.load-older:hover { background:var(--accent); }
  .channel-composer-dock { display:flex; flex:0 0 auto; justify-content:center; padding:0.5rem 1rem 0.75rem; background:linear-gradient(to bottom,transparent,var(--cal-bg) 18%); }
  .loading,.feed-error { padding:1rem; text-align:center; color:var(--muted-foreground); font-size:0.78rem; }.feed-error { color:var(--destructive); }
  @container chat-shell (max-width:560px) { .first-teammate { grid-template-columns:auto minmax(0,1fr) auto; }.setup-teammate { grid-column:2; justify-self:start; }.dismiss-setup { grid-column:3; grid-row:1; } }
  @container chat-shell (max-width:440px) { .feed-canvas { padding-inline:0.25rem; }.channel-introduction { padding-inline:0.75rem; }.channel-composer-dock { padding-inline:0.5rem; }.first-teammate { margin-inline:0.75rem; } }
</style>
