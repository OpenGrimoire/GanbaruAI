<script lang="ts">
  import { tick, untrack } from "svelte";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import AtSign from "@lucide/svelte/icons/at-sign";
  import Bold from "@lucide/svelte/icons/bold";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import Image from "@lucide/svelte/icons/image";
  import Italic from "@lucide/svelte/icons/italic";
  import MessageSquareShare from "@lucide/svelte/icons/message-square-share";
  import Plus from "@lucide/svelte/icons/plus";
  import Settings from "@lucide/svelte/icons/settings";
  import * as chatApi from "$lib/api/chat";
  import type { ChatParticipantRead, ChatScheduledMessageRead } from "$lib/chat/contracts";
  import {
    copyParticipantMentionSlice,
    expandEditRangeToParticipantMentions,
    insertParticipantMention,
    mentionAfterCaret,
    mentionBeforeCaret,
    mentionQueryAtCaret,
    participantMentionRichContent,
    participantMentionCandidates,
    pasteParticipantMentionSlice,
    rebaseMentionsAfterInput,
    type ParticipantMentionClipboardSlice,
  } from "$lib/chat/participant-mentions";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat, type ChatOrganizationalDraft } from "$lib/stores/chat.svelte";
  import { portal } from "$lib/utils/portal";

  type ScheduleMenuComponent = typeof import("./ChatMessageScheduleMenu.svelte").default;

  const PARTICIPANT_MENTION_CLIPBOARD_TYPE = "application/x-ganbaru-participant-mentions";

  let {
    destination,
    placeholder,
    threadComposer = false,
  }: {
    destination: string;
    placeholder: string;
    threadComposer?: boolean;
  } = $props();

  const chat = getChat();
  const localization = getLocalization();
  const { t } = localization;
  const initialDraft = untrack(() => chat.organizationalDraft(destination));
  let text = $state(initialDraft.normalizedMarkdown);
  let mentions = $state(initialDraft.participantMentions.map((mention) => ({ ...mention })));
  let attachmentIds = $state([...initialDraft.attachmentIds]);
  let resourceReferences = $state(initialDraft.resourceReferences.map((reference) => ({ ...reference })));
  let selectionStart = $state(initialDraft.selectionStart);
  let selectionEnd = $state(initialDraft.selectionEnd);
  let textarea = $state<HTMLTextAreaElement | null>(null);
  let mentionOpen = $state(false);
  let mentionStart = $state(0);
  let mentionQuery = $state("");
  let mentionIndex = $state(0);
  let mentionStyle = $state("");
  let addMenuOpen = $state(false);
  let scheduleMenuOpen = $state(false);
  let scheduledMessagesOpen = $state(false);
  let ScheduleMenu = $state<ScheduleMenuComponent | null>(null);
  let scheduleMenuLoad = $state<Promise<void> | null>(null);
  let scheduledFor = $state<string | null>(initialDraft.scheduledFor);
  let scheduledMessages = $state<ChatScheduledMessageRead[]>([]);
  let scheduledMessagesRequest = 0;
  let alsoSendToChannel = $state(false);
  let resourcePath = $state("");
  let resourceKind = $state<"file" | "folder">("file");
  let sending = $state(false);
  let error = $state<string | null>(null);
  const teammateDetails = $derived(new Map(chat.teammates.map((teammate) => [
    teammate.participant.id,
    { purpose: teammate.purpose, configurationState: teammate.configurationState },
  ])));
  const candidates = $derived(participantMentionCandidates(
    chat.selectedChannel?.memberships ?? [],
    teammateDetails,
    mentionQuery,
  ));
  const invokedAiCount = $derived(new Set(mentions
    .filter((mention) => mention.participantKind === "ai_teammate")
    .map((mention) => mention.participantId)).size);
  const validation = $derived(invokedAiCount > 1 ? t("chat.organization.oneAgentOnly") : null);
  const channelName = $derived(chat.selectedChannel?.name ?? "");
  const hasMessageContent = $derived(Boolean(text.trim() || attachmentIds.length || resourceReferences.length));
  const nextScheduledMessage = $derived(scheduledMessages.find((message) => message.state !== "failed") ?? scheduledMessages[0] ?? null);

  $effect(() => {
    const _version = chat.scheduledMessagesVersion;
    const currentDestination = destination;
    void loadScheduledMessages(currentDestination);
  });

  function persist(): void {
    const draft: ChatOrganizationalDraft = {
      normalizedMarkdown: text,
      richContent: {
        schemaVersion: 1,
        value: participantMentionRichContent(text, mentions),
      },
      attachmentIds: [...attachmentIds],
      participantMentions: mentions.map((mention) => ({ ...mention })),
      resourceReferences: resourceReferences.map((reference) => ({ ...reference })),
      selectionStart,
      selectionEnd,
      scheduledFor,
    };
    chat.setOrganizationalDraft(destination, draft);
  }

  function updateSelection(): void {
    if (!textarea) return;
    selectionStart = textarea.selectionStart;
    selectionEnd = textarea.selectionEnd;
    const query = mentionQueryAtCaret(text, selectionEnd);
    mentionOpen = query !== null;
    mentionStart = query?.start ?? selectionEnd;
    mentionQuery = query?.query ?? "";
    mentionIndex = Math.min(mentionIndex, Math.max(0, candidates.length - 1));
    refreshMentionGeometry();
    persist();
  }

  function handleInput(event: Event): void {
    const nextText = (event.currentTarget as HTMLTextAreaElement).value;
    mentions = rebaseMentionsAfterInput(text, nextText, mentions);
    text = nextText;
    updateSelection();
  }

  function handleBeforeInput(event: InputEvent): void {
    if (event.isComposing || !textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const expanded = expandEditRangeToParticipantMentions(text, mentions, start, end);
    let range = expanded;
    let inserted: string | null = null;
    if (event.inputType === "deleteContentBackward") {
      range = start === end ? mentionBeforeCaret(text, mentions, start) ?? expanded : expanded;
      if (range.start === range.end) return;
      inserted = "";
    } else if (event.inputType === "deleteContentForward") {
      range = start === end ? mentionAfterCaret(text, mentions, start) ?? expanded : expanded;
      if (range.start === range.end) return;
      inserted = "";
    } else if (event.inputType === "insertText" && event.data !== null && (range.start !== start || range.end !== end)) {
      inserted = event.data;
    } else if (event.inputType === "insertLineBreak" && (range.start !== start || range.end !== end)) {
      inserted = "\n";
    } else {
      return;
    }
    event.preventDefault();
    replaceRange(range.start, range.end, inserted);
  }

  function handleCompositionStart(): void {
    if (!textarea) return;
    const range = expandEditRangeToParticipantMentions(
      text,
      mentions,
      textarea.selectionStart,
      textarea.selectionEnd,
    );
    if (range.start !== textarea.selectionStart || range.end !== textarea.selectionEnd) {
      textarea.setSelectionRange(range.start, range.end);
    }
  }

  function handleCopy(event: ClipboardEvent): void {
    if (!textarea || !event.clipboardData || textarea.selectionStart === textarea.selectionEnd) return;
    const slice = copyParticipantMentionSlice(text, mentions, textarea.selectionStart, textarea.selectionEnd);
    event.preventDefault();
    event.clipboardData.setData("text/plain", slice.text);
    event.clipboardData.setData(PARTICIPANT_MENTION_CLIPBOARD_TYPE, JSON.stringify(slice));
  }

  function handlePaste(event: ClipboardEvent): void {
    if (!textarea || !event.clipboardData) return;
    const plainText = event.clipboardData.getData("text/plain");
    const serialized = event.clipboardData.getData(PARTICIPANT_MENTION_CLIPBOARD_TYPE);
    const parsed = parseParticipantMentionClipboardSlice(serialized, plainText);
    const slice: ParticipantMentionClipboardSlice = parsed ?? { text: plainText, mentions: [] };
    const result = pasteParticipantMentionSlice(
      text,
      mentions,
      textarea.selectionStart,
      textarea.selectionEnd,
      slice,
    );
    event.preventDefault();
    text = result.text;
    mentions = result.mentions;
    selectionStart = result.selection;
    selectionEnd = result.selection;
    persist();
    void tick().then(() => textarea?.setSelectionRange(result.selection, result.selection));
  }

  function parseParticipantMentionClipboardSlice(
    serialized: string,
    plainText: string,
  ): ParticipantMentionClipboardSlice | null {
    if (!serialized) return null;
    let value: unknown;
    try {
      value = JSON.parse(serialized);
    } catch {
      return null;
    }
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    const record = value as Record<string, unknown>;
    if (record.text !== plainText || !Array.isArray(record.mentions)) return null;
    const parsed = record.mentions.flatMap((entry) => {
      if (!entry || typeof entry !== "object" || Array.isArray(entry)) return [];
      const mention = entry as Record<string, unknown>;
      if (
        typeof mention.participantId !== "string"
        || !["local_user", "ai_teammate", "human"].includes(String(mention.participantKind))
        || (mention.handleSnapshot !== null && typeof mention.handleSnapshot !== "string")
        || typeof mention.labelSnapshot !== "string"
        || !Number.isSafeInteger(mention.startOffset)
        || !Number.isSafeInteger(mention.endOffset)
      ) return [];
      return [{
        participantId: mention.participantId,
        participantKind: mention.participantKind as "local_user" | "ai_teammate" | "human",
        handleSnapshot: mention.handleSnapshot as string | null,
        labelSnapshot: mention.labelSnapshot,
        startOffset: Number(mention.startOffset),
        endOffset: Number(mention.endOffset),
      }];
    });
    if (parsed.length !== record.mentions.length) return null;
    return { text: plainText, mentions: parsed };
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.isComposing || event.keyCode === 229) return;
    if (mentionOpen) {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        const direction = event.key === "ArrowDown" ? 1 : -1;
        mentionIndex = (mentionIndex + direction + Math.max(1, candidates.length)) % Math.max(1, candidates.length);
        return;
      }
      if ((event.key === "Enter" || event.key === "Tab") && candidates[mentionIndex]) {
        event.preventDefault();
        chooseMention(candidates[mentionIndex].participant);
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        mentionOpen = false;
        return;
      }
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void post();
    }
  }

  function chooseMention(participant: ChatParticipantRead): void {
    const result = insertParticipantMention(text, mentions, mentionStart, selectionEnd, participant);
    text = result.text;
    mentions = result.mentions;
    selectionStart = result.selection;
    selectionEnd = result.selection;
    mentionOpen = false;
    persist();
    void tick().then(() => {
      textarea?.focus();
      textarea?.setSelectionRange(result.selection, result.selection);
    });
  }

  function replaceRange(start: number, end: number, inserted: string): void {
    const next = `${text.slice(0, start)}${inserted}${text.slice(end)}`;
    mentions = rebaseMentionsAfterInput(text, next, mentions);
    text = next;
    selectionStart = start + inserted.length;
    selectionEnd = selectionStart;
    persist();
    void tick().then(() => textarea?.setSelectionRange(selectionStart, selectionEnd));
  }

  function wrapSelection(marker: "**" | "_"): void {
    if (!textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    replaceRange(start, end, `${marker}${text.slice(start, end)}${marker}`);
  }

  function refreshMentionGeometry(): void {
    if (!textarea || !mentionOpen) return;
    const bounds = textarea.getBoundingClientRect();
    const style = getComputedStyle(textarea);
    const lineHeight = Number.parseFloat(style.lineHeight) || 22;
    const charactersPerLine = Math.max(12, Math.floor((bounds.width - 24) / 8));
    const before = text.slice(0, selectionEnd);
    const visualRows = before.split("\n").reduce((rows, line) => rows + Math.max(1, Math.ceil(line.length / charactersPerLine)), 0);
    const estimatedTop = bounds.top + Math.min(bounds.height - lineHeight, visualRows * lineHeight);
    const menuHeight = Math.min(280, Math.max(80, candidates.length * 58));
    const top = estimatedTop + lineHeight + menuHeight > window.innerHeight
      ? Math.max(8, estimatedTop - menuHeight)
      : estimatedTop + lineHeight;
    const left = Math.min(Math.max(8, bounds.left + 12), Math.max(8, window.innerWidth - 328));
    mentionStyle = `position:fixed;left:${Math.round(left)}px;top:${Math.round(top)}px;width:min(20rem,calc(100vw - 1rem))`;
  }

  async function pickImages(): Promise<void> {
    const workingFolderId = chat.selectedWorkingFolderId;
    if (!workingFolderId) return;
    addMenuOpen = false;
    const available = Math.max(0, 8 - attachmentIds.length);
    if (available === 0) return;
    const imported = await chatApi.pickChatImages(
      workingFolderId,
      Array.from({ length: available }, () => crypto.randomUUID()),
      t("chat.composer.attachImages"),
    );
    attachmentIds = [...attachmentIds, ...imported.map((attachment) => attachment.id)];
    persist();
  }

  function addResourceReference(): void {
    const workingFolderId = chat.selectedWorkingFolderId;
    const relativePath = resourcePath.trim().replace(/^\.\//u, "");
    if (!workingFolderId || !relativePath) return;
    resourceReferences = [...resourceReferences, {
      workingFolderId,
      kind: resourceKind,
      relativePath,
      displayLabel: relativePath.split("/").filter(Boolean).at(-1) ?? relativePath,
    }];
    resourcePath = "";
    addMenuOpen = false;
    persist();
  }

  function clearComposerAfterDelivery(): void {
    text = "";
    mentions = [];
    attachmentIds = [];
    resourceReferences = [];
    selectionStart = 0;
    selectionEnd = 0;
    alsoSendToChannel = false;
    scheduledFor = null;
  }

  function formatScheduledInstant(value: string): string {
    const instant = new Date(value);
    if (!Number.isFinite(instant.getTime())) return value;
    return new Intl.DateTimeFormat(localization.locale, {
      weekday: "short",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(instant);
  }

  function loadScheduleMenu(): Promise<void> {
    if (ScheduleMenu) return Promise.resolve();
    scheduleMenuLoad ??= import("./ChatMessageScheduleMenu.svelte")
      .then((module) => { ScheduleMenu = module.default; })
      .catch((cause: unknown) => {
        error = cause instanceof Error ? cause.message : String(cause);
      })
      .finally(() => { scheduleMenuLoad = null; });
    return scheduleMenuLoad;
  }

  function toggleScheduleMenu(): void {
    scheduleMenuOpen = !scheduleMenuOpen;
    scheduledMessagesOpen = false;
    addMenuOpen = false;
    if (scheduleMenuOpen) void loadScheduleMenu();
  }

  function selectSchedule(nextScheduledFor: string): void {
    scheduledFor = nextScheduledFor;
    persist();
    scheduleMenuOpen = false;
    void tick().then(() => textarea?.focus());
  }

  function clearSchedule(): void {
    scheduledFor = null;
    persist();
    scheduleMenuOpen = false;
  }

  function toggleScheduledMessages(): void {
    scheduledMessagesOpen = !scheduledMessagesOpen;
    scheduleMenuOpen = false;
    addMenuOpen = false;
    if (scheduledMessagesOpen) void loadScheduleMenu();
  }

  async function loadScheduledMessages(currentDestination: string): Promise<void> {
    const request = ++scheduledMessagesRequest;
    try {
      const messages = await chat.listScheduledOrganizationalMessages(currentDestination);
      if (request !== scheduledMessagesRequest || currentDestination !== destination) return;
      scheduledMessages = messages;
      if (messages.length === 0) scheduledMessagesOpen = false;
    } catch {
      // Preserve the last successful summary during a transient read failure.
    }
  }

  function updateScheduledMessages(messages: ChatScheduledMessageRead[]): void {
    scheduledMessages = messages;
    if (messages.length === 0) scheduledMessagesOpen = false;
  }

  async function post(): Promise<void> {
    if (sending || validation || !hasMessageContent) return;
    sending = true;
    error = null;
    try {
      if (scheduledFor) {
        const { chatScheduleIsFuture } = await import("$lib/chat/message-scheduling");
        if (!chatScheduleIsFuture(scheduledFor)) {
          error = t("chat.organization.invalidScheduleTime");
          return;
        }
        const scheduled = await chat.scheduleOrganizationalMessage(destination, scheduledFor, {
          alsoSendToChannel: threadComposer && alsoSendToChannel,
        });
        scheduledMessages = [...scheduledMessages.filter((message) => message.id !== scheduled.id), scheduled]
          .sort((left, right) => left.scheduledFor.localeCompare(right.scheduledFor));
      } else {
        await chat.postOrganizationalMessage(destination, {
          alsoSendToChannel: threadComposer && alsoSendToChannel,
        });
      }
      clearComposerAfterDelivery();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      sending = false;
    }
  }

  function configureTeammate(participantId: string): void {
    mentionOpen = false;
    window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-configure-teammate", { detail: { participantId } }));
  }
</script>

<div class="composer-stack">
  {#if nextScheduledMessage}
    <div class="scheduled-summary">
      <Clock3 size={13} />
      <span>{t(
        "chat.organization.scheduledMessageSummary",
        scheduledMessages.length,
        formatScheduledInstant(nextScheduledMessage.scheduledFor),
      )}</span>
      <div class="scheduled-summary-anchor">
        <button
          type="button"
          aria-haspopup="dialog"
          aria-expanded={scheduledMessagesOpen}
          onpointerenter={() => { void loadScheduleMenu(); }}
          onfocus={() => { void loadScheduleMenu(); }}
          onclick={toggleScheduledMessages}
        >{t("chat.organization.viewScheduledMessages", scheduledMessages.length)}</button>
        {#if scheduledMessagesOpen}
          {#if ScheduleMenu}
            {@const LoadedScheduledMessagesMenu = ScheduleMenu}
            <LoadedScheduledMessagesMenu
              mode="manage"
              align="right"
              {scheduledMessages}
              onmessageschange={updateScheduledMessages}
              onclose={() => { scheduledMessagesOpen = false; }}
            />
          {:else}
            <div class="composer-menu schedule-loading align-right" role="status">{t("chat.status.working")}</div>
          {/if}
        {/if}
      </div>
    </div>
  {/if}
  <div class="organizational-composer" data-organizational-composer>
  <textarea
    bind:this={textarea}
    value={text}
    rows="3"
    maxlength="131072"
    {placeholder}
    aria-label={placeholder}
    data-chat-composer
    oninput={handleInput}
    onbeforeinput={handleBeforeInput}
    onselect={updateSelection}
    onkeyup={updateSelection}
    onclick={updateSelection}
    onkeydown={handleKeydown}
    oncopy={handleCopy}
    onpaste={handlePaste}
    oncompositionstart={handleCompositionStart}
    oncompositionend={updateSelection}
  ></textarea>

  {#if mentionOpen}
    <div use:portal class="mention-picker" style={mentionStyle} role="listbox" aria-label={t("chat.organization.mentionTeammate")}>
      {#if candidates.length === 0}
        <p>{t("chat.organization.noMentionResults")}</p>
      {:else}
        {#each candidates as candidate, index (candidate.participant.id)}
          <div class="candidate-row" class:active={index === mentionIndex} role="option" aria-selected={index === mentionIndex}>
            <button type="button" class="candidate-select" onclick={() => chooseMention(candidate.participant)}>
            <span class="participant-avatar" aria-hidden="true">{candidate.participant.displayName.slice(0, 1).toLocaleUpperCase()}</span>
            <span class="candidate-copy"><strong>{candidate.participant.displayName}</strong><small>{candidate.purpose || `@${candidate.participant.handle ?? ""}`}</small></span>
            {#if candidate.participant.kind === "ai_teammate"}<span class="agent-badge">{t("chat.organization.agent")}</span>{/if}
            </button>
            {#if candidate.configurationState && candidate.configurationState !== "healthy"}
              <span class="needs-setup">{t("chat.organization.needsSetup")}</span>
              <button type="button" class="configure" onclick={() => configureTeammate(candidate.participant.id)}><Settings size={12} />{t("chat.organization.configure")}</button>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  {/if}

  {#if attachmentIds.length > 0 || resourceReferences.length > 0}
    <div class="context-chips">
      {#each attachmentIds as id, index (id)}<button type="button" onclick={() => { attachmentIds = attachmentIds.filter((entry) => entry !== id); persist(); }}><Image size={12} />{t("chat.organization.imageNumber", index + 1)}<span>×</span></button>{/each}
      {#each resourceReferences as reference, index (`${reference.kind}:${reference.relativePath}:${index}`)}<button type="button" onclick={() => { resourceReferences = resourceReferences.filter((_, entryIndex) => entryIndex !== index); persist(); }}>{#if reference.kind === "folder"}<Folder size={12} />{:else}<File size={12} />{/if}{reference.displayLabel}<span>×</span></button>{/each}
    </div>
  {/if}

  <div class="composer-footer">
    <div class="composer-tools">
      <div class="menu-anchor">
        <button type="button" class="tool-button" aria-label={t("chat.organization.addContext")} aria-expanded={addMenuOpen} onclick={() => { addMenuOpen = !addMenuOpen; scheduleMenuOpen = false; scheduledMessagesOpen = false; }}><Plus size={16} /></button>
        {#if addMenuOpen}
          <div class="composer-menu add-menu">
            <button type="button" onclick={() => void pickImages()}><Image size={14} />{t("chat.composer.attachImages")}</button>
            <div class="resource-entry">
              <div><button type="button" class:active={resourceKind === "file"} onclick={() => { resourceKind = "file"; }}><File size={13} />{t("chat.organization.file")}</button><button type="button" class:active={resourceKind === "folder"} onclick={() => { resourceKind = "folder"; }}><Folder size={13} />{t("chat.organization.folder")}</button></div>
              <input bind:value={resourcePath} placeholder={t("chat.organization.relativePath")} onkeydown={(event) => { if (event.key === "Enter") { event.preventDefault(); addResourceReference(); } }} />
              <button type="button" disabled={!resourcePath.trim()} onclick={addResourceReference}>{t("chat.organization.add")}</button>
            </div>
          </div>
        {/if}
      </div>
      <button type="button" class="tool-button" aria-label={t("chat.organization.bold")} onclick={() => wrapSelection("**")}><Bold size={15} /></button>
      <button type="button" class="tool-button" aria-label={t("chat.organization.italic")} onclick={() => wrapSelection("_")}><Italic size={15} /></button>
      <button type="button" class="tool-button" aria-label={t("chat.organization.mentionTeammate")} onclick={() => { replaceRange(selectionStart, selectionEnd, "@"); }}><AtSign size={15} /></button>
      <div class="menu-anchor">
        <button
          type="button"
          class="tool-button"
          class:active={scheduleMenuOpen || Boolean(scheduledFor)}
          aria-label={scheduledFor ? t("chat.organization.scheduleSelected", formatScheduledInstant(scheduledFor)) : t("chat.organization.scheduleMessage")}
          aria-haspopup="dialog"
          aria-expanded={scheduleMenuOpen}
          aria-pressed={Boolean(scheduledFor)}
          title={scheduledFor ? t("chat.organization.scheduleSelected", formatScheduledInstant(scheduledFor)) : t("chat.organization.scheduleMessage")}
          onpointerenter={() => { void loadScheduleMenu(); }}
          onfocus={() => { void loadScheduleMenu(); }}
          onclick={toggleScheduleMenu}
        ><Clock3 size={15} /></button>
        {#if scheduleMenuOpen}
          {#if ScheduleMenu}
            {@const LoadedScheduleMenu = ScheduleMenu}
            <LoadedScheduleMenu
              mode="choose"
              selectedScheduledFor={scheduledFor}
              disabled={sending}
              onselect={selectSchedule}
              onclear={clearSchedule}
              onclose={() => { scheduleMenuOpen = false; }}
            />
          {:else}
            <div class="composer-menu schedule-loading" role="status">{t("chat.status.working")}</div>
          {/if}
        {/if}
      </div>
      {#if threadComposer}
        <button
          type="button"
          class="tool-button"
          class:active={alsoSendToChannel}
          aria-label={t("chat.organization.shareReplyToChannel", channelName)}
          aria-pressed={alsoSendToChannel}
          title={t("chat.organization.shareReplyToChannel", channelName)}
          onclick={() => { alsoSendToChannel = !alsoSendToChannel; }}
        ><MessageSquareShare size={15} /></button>
      {/if}
    </div>
    <button type="button" class="send-button" disabled={sending || Boolean(validation) || !hasMessageContent} onclick={() => void post()}><ArrowUp size={16} /><span class="sr-only">{scheduledFor ? t("chat.organization.scheduleMessage") : t("chat.composer.send")}</span></button>
  </div>
  {#if validation}<p class="composer-error" role="alert">{validation}</p>{/if}
  {#if error}<p class="composer-error" role="alert">{error}</p>{/if}
  </div>
</div>

<style>
  .composer-stack { width:min(100%,54rem); }
  .organizational-composer { position:relative; width:100%; border:1px solid color-mix(in srgb,var(--border) 88%,transparent); border-radius:1.3rem; background:var(--card); box-shadow:0 8px 22px -18px rgb(0 0 0 / 0.24),0 1px 4px -3px rgb(0 0 0 / 0.16); }
  .scheduled-summary { display:flex; min-height:2rem; align-items:center; gap:0.45rem; margin:0 0.35rem 0.35rem; border-radius:0.55rem; background:color-mix(in srgb,var(--accent) 58%,transparent); padding:0.25rem 0.35rem 0.25rem 0.55rem; color:var(--muted-foreground); }
  .scheduled-summary :global(svg) { flex:0 0 auto; }
  .scheduled-summary > span { min-width:0; flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:0.7rem; }
  .scheduled-summary-anchor { position:relative; flex:0 0 auto; }
  .scheduled-summary-anchor > button { min-height:1.5rem; border-radius:0.4rem; padding:0.2rem 0.45rem; color:var(--foreground); font-size:0.68rem; font-weight:500; }
  .scheduled-summary-anchor > button:hover,.scheduled-summary-anchor > button[aria-expanded="true"] { background:color-mix(in srgb,var(--background) 70%,transparent); }
  textarea { display:block; width:100%; min-height:4.15rem; max-height:15.35rem; resize:none; overflow-y:auto; border:0; background:transparent; padding:1rem 1.25rem 0.35rem; color:var(--foreground); font:inherit; font-size:var(--chat-conversation-font-size,0.933333rem); line-height:var(--chat-conversation-line-height,1.4rem); outline:none; }
  textarea::placeholder { color:color-mix(in srgb,var(--muted-foreground) 52%,transparent); }
  .composer-footer { display:flex; min-height:3rem; align-items:center; justify-content:space-between; gap:0.5rem; padding:0.3rem 0.75rem 0.65rem; }
  .composer-tools { display:flex; align-items:center; gap:0.3rem; }
  .menu-anchor { position:relative; }
  .tool-button { display:grid; width:1.9rem; height:1.9rem; place-items:center; border-radius:0.5rem; color:var(--muted-foreground); }
  .tool-button:hover { background:var(--accent); color:var(--foreground); }
  .tool-button.active { background:var(--accent); color:var(--foreground); }
  .send-button { display:grid; width:2.1rem; height:2.1rem; flex:0 0 auto; place-items:center; border-radius:999px; background:color-mix(in srgb,var(--primary) 92%,transparent); color:var(--primary-foreground); box-shadow:0 2px 7px color-mix(in srgb,var(--primary) 22%,transparent); transition:transform 120ms ease,filter 120ms ease; }
  .send-button:hover:not(:disabled) { filter:brightness(1.04); transform:scale(1.04); }
  .send-button:disabled { opacity:0.45; }
  .composer-menu { position:absolute; z-index:70; min-width:14rem; border:1px solid var(--border); border-radius:0.5rem; background:var(--popover); padding:0.3rem; box-shadow:0 10px 30px rgb(0 0 0 / 0.16); }
  .composer-menu > button { display:flex; width:100%; min-height:2rem; align-items:center; gap:0.45rem; border-radius:0.35rem; padding:0.35rem 0.5rem; text-align:left; font-size:0.75rem; }
  .composer-menu > button:hover { background:var(--accent); }
  .add-menu { bottom:calc(100% + 0.35rem); left:0; }
  .schedule-loading { bottom:calc(100% + 0.35rem); left:0; color:var(--muted-foreground); font-size:0.72rem; }
  .schedule-loading.align-right { right:0; left:auto; }
  .resource-entry { display:grid; gap:0.35rem; border-top:1px solid var(--border); padding:0.45rem 0.35rem 0.25rem; }
  .resource-entry > div { display:flex; gap:0.25rem; }
  .resource-entry button { display:flex; align-items:center; gap:0.25rem; border-radius:0.3rem; padding:0.25rem 0.4rem; font-size:0.7rem; }
  .resource-entry button.active { background:var(--accent); }
  .resource-entry input { min-width:0; border:1px solid var(--border); border-radius:0.35rem; background:var(--background); padding:0.35rem 0.45rem; font-size:0.72rem; outline:none; }
  .context-chips { display:flex; flex-wrap:wrap; gap:0.3rem; padding:0 0.75rem 0.35rem; }
  .context-chips button { display:flex; max-width:14rem; align-items:center; gap:0.25rem; border-radius:999px; background:var(--accent); padding:0.22rem 0.45rem; font-size:0.68rem; }
  .context-chips button span { color:var(--muted-foreground); }
  .mention-picker { z-index:100; max-height:min(20rem,60vh); overflow-y:auto; border:1px solid var(--border); border-radius:0.6rem; background:var(--popover); padding:0.3rem; box-shadow:0 14px 38px rgb(0 0 0 / 0.2); }
  .candidate-row { display:flex; width:100%; min-height:3rem; align-items:center; gap:0.3rem; border-radius:0.4rem; padding:0.2rem; }.candidate-row:is(:hover,.active) { background:var(--accent); }
  .candidate-select { display:flex; min-width:0; flex:1; align-items:center; gap:0.5rem; padding:0.2rem; text-align:left; }
  .mention-picker > p { padding:0.65rem; color:var(--muted-foreground); font-size:0.75rem; }
  .participant-avatar { display:grid; width:1.8rem; height:1.8rem; flex:0 0 auto; place-items:center; border-radius:0.4rem; background:color-mix(in srgb,var(--primary) 16%,var(--accent)); font-size:0.72rem; font-weight:700; }
  .candidate-copy { display:grid; min-width:0; flex:1; }
  .candidate-copy strong,.candidate-copy small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .candidate-copy strong { font-size:0.76rem; }.candidate-copy small { color:var(--muted-foreground); font-size:0.67rem; }
  .agent-badge,.needs-setup { flex:0 0 auto; border:1px solid var(--border); border-radius:999px; padding:0.1rem 0.35rem; color:var(--muted-foreground); font-size:0.6rem; }
  .needs-setup { color:var(--destructive); }.configure { display:flex; align-items:center; gap:0.2rem; font-size:0.65rem; }
  .composer-error { padding:0 0.75rem 0.5rem; color:var(--destructive); font-size:0.7rem; }
  @media (forced-colors:active) { .organizational-composer,.composer-menu,.mention-picker { border:1px solid CanvasText; } }
</style>
