<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import * as chatApi from "$lib/api/chat";
  import type { ChatReviewCommentRead } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";

  const { t } = getLocalization();
  const chat = getChat();
  let comments = $state<ChatReviewCommentRead[]>([]);
  let includeResolved = $state(false);
  let loading = $state(false);
  let busyId = $state<string | null>(null);
  let error = $state<string | null>(null);
  let loadedScope = "";
  const threadId = $derived(chat.selectedThreadId);
  const visibleComments = $derived(includeResolved ? comments : comments.filter((comment) => comment.state === "open"));

  $effect(() => {
    const thread = threadId;
    const scope = `${thread ?? ""}:${includeResolved}`;
    if (!thread || scope === loadedScope) return;
    loadedScope = scope;
    void load(thread);
  });

  async function load(thread: string): Promise<void> {
    loading = true;
    error = null;
    try {
      comments = await chatApi.listChatReviewComments(thread, includeResolved);
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      loading = false;
    }
  }

  async function setResolved(comment: ChatReviewCommentRead, resolved: boolean): Promise<void> {
    if (!threadId || busyId) return;
    busyId = comment.id;
    error = null;
    try {
      const updated = await chatApi.setChatReviewCommentResolved(threadId, comment.id, resolved);
      comments = comments.map((entry) => entry.id === updated.id ? updated : entry);
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      busyId = null;
    }
  }

  async function attach(comment: ChatReviewCommentRead): Promise<void> {
    if (!threadId || busyId) return;
    busyId = comment.id;
    error = null;
    try {
      const attachment = await chatApi.attachChatReviewComment(threadId, comment.id, crypto.randomUUID());
      if (!chat.composer.attachmentIds.includes(attachment.id)) {
        chat.setComposerAttachments([...chat.composer.attachmentIds, attachment.id]);
      }
    } catch (reason: unknown) {
      error = message(reason);
    } finally {
      busyId = null;
    }
  }

  function message(reason: unknown): string {
    return reason instanceof Error ? reason.message : String(reason);
  }
</script>

<section class="review-panel" aria-label={t("chat.inspector.review")}>
  <header>
    <label><input type="checkbox" bind:checked={includeResolved} />{t("chat.review.showResolved")}</label>
    {#if loading}<span>{t("common.loading")}</span>{/if}
  </header>
  {#if error}<p role="alert" class="error">{error}</p>{/if}
  {#if !loading && visibleComments.length === 0}
    <p class="empty">{t("chat.inspector.reviewEmpty")}</p>
  {:else}
    <div class="comments">
      {#each visibleComments as comment (comment.id)}
        <article class:resolved={comment.state === "resolved"}>
          <div class="location">
            <strong title={comment.relativePath}>{comment.relativePath}</strong>
            <span>{t("chat.review.lines", comment.startLine, comment.endLine)}</span>
          </div>
          <p class="comment">{comment.commentText}</p>
          {#if comment.selectedText}<pre>{comment.selectedText}</pre>{/if}
          <footer>
            <span>{comment.contentRevision.slice(0, 12)}</span>
            <button type="button" disabled={busyId !== null} onclick={() => void attach(comment)}><Paperclip size={13} />{t("chat.review.attach")}</button>
            {#if comment.state === "open"}
              <button type="button" disabled={busyId !== null} onclick={() => void setResolved(comment, true)}><Check size={13} />{t("chat.review.resolve")}</button>
            {:else}
              <button type="button" disabled={busyId !== null} onclick={() => void setResolved(comment, false)}><RotateCcw size={13} />{t("chat.review.reopen")}</button>
            {/if}
          </footer>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .review-panel { display: flex; height: 100%; min-height: 0; flex-direction: column; overflow: hidden; }
  header { display: flex; min-height: 2.45rem; flex: 0 0 auto; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--border); padding: 0.4rem 0.6rem; color: var(--muted-foreground); font-size: 0.7rem; }
  header label { display: flex; align-items: center; gap: 0.4rem; }
  .comments { display: grid; min-height: 0; gap: 0.55rem; overflow: auto; padding: 0.6rem; }
  article { display: grid; gap: 0.45rem; align-self: start; border: 1px solid var(--border); border-radius: 0.5rem; background: var(--background); padding: 0.6rem; }
  article.resolved { opacity: 0.66; }
  .location { display: flex; min-width: 0; align-items: baseline; justify-content: space-between; gap: 0.5rem; }
  .location strong { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.733333rem; }
  .location span, footer > span { flex: 0 0 auto; color: var(--muted-foreground); font-family: monospace; font-size: 0.65rem; }
  .comment { white-space: pre-wrap; overflow-wrap: anywhere; font-size: 0.733333rem; }
  pre { max-height: 8rem; overflow: auto; border-radius: 0.35rem; background: var(--cal-bg); padding: 0.45rem; white-space: pre-wrap; font-size: 0.666667rem; }
  footer { display: flex; min-width: 0; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 0.3rem; }
  footer > span { margin-right: auto; }
  button { display: inline-flex; align-items: center; gap: 0.25rem; border-radius: 0.35rem; padding: 0.25rem 0.4rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  button:hover { background: var(--accent); color: var(--foreground); }
  button:disabled { opacity: 0.5; }
  .empty { margin: auto; padding: 1rem; text-align: center; color: var(--muted-foreground); font-size: 0.733333rem; }
  .error { border-bottom: 1px solid color-mix(in srgb, var(--destructive) 30%, var(--border)); padding: 0.5rem; color: var(--destructive); font-size: 0.7rem; }
</style>
