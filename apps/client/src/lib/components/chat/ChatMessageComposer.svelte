<script lang="ts">
  import { onMount, tick, untrack } from "svelte";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import AtSign from "@lucide/svelte/icons/at-sign";
  import Bold from "@lucide/svelte/icons/bold";
  import Clock3 from "@lucide/svelte/icons/clock-3";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import Image from "@lucide/svelte/icons/image";
  import Italic from "@lucide/svelte/icons/italic";
  import MessageSquareShare from "@lucide/svelte/icons/message-square-share";
  import Plus from "@lucide/svelte/icons/plus";
  import Settings from "@lucide/svelte/icons/settings";
  import Square from "@lucide/svelte/icons/square";
  import * as chatApi from "$lib/api/chat";
  import type {
    ChatParticipantRead,
    ChatScheduledMessageRead,
    ChatWorkAssignmentState,
  } from "$lib/chat/contracts";
  import {
    composerTextareaLayout,
    measureTextareaContentHeight,
    revealTextareaComposerCaret,
  } from "$lib/chat/composer-scroll";
  import {
    copyParticipantMentionSlice,
    expandEditRangeToParticipantMentions,
    insertParticipantMention,
    mentionAfterCaret,
    mentionBeforeCaret,
    mentionQueryAtCaret,
    participantMentionRichContent,
    participantMentionCandidates,
    participantMentionTextSegments,
    pasteParticipantMentionSlice,
    rebaseMentionsAfterInput,
    type ParticipantMentionClipboardSlice,
  } from "$lib/chat/participant-mentions";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import { getChat, type ChatOrganizationalDraft } from "$lib/stores/chat.svelte";
  import { getPreferences } from "$lib/stores/preferences.svelte";
  import { portal } from "$lib/utils/portal";
  import ChatParticipantAvatar from "./ChatParticipantAvatar.svelte";

  type ScheduleMenuComponent = typeof import("./ChatMessageScheduleMenu.svelte").default;

  const PARTICIPANT_MENTION_CLIPBOARD_TYPE = "application/x-ganbaru-participant-mentions";
  const MENTION_PICKER_WIDTH_PX = 320;
  const MENTION_PICKER_VIEWPORT_INSET_PX = 8;
  const MENTION_PICKER_GAP_PX = 6;
  const STOPPABLE_ASSIGNMENT_STATES = new Set<ChatWorkAssignmentState>([
    "queued",
    "working",
    "waiting_for_answer",
    "waiting_for_approval",
  ]);

  let {
    destination,
    placeholder,
    threadComposer = false,
    onRequestScrollToBottom = () => undefined,
  }: {
    destination: string;
    placeholder: string;
    threadComposer?: boolean;
    onRequestScrollToBottom?: () => void;
  } = $props();

  const chat = getChat();
  const preferences = getPreferences();
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
  let mentionPicker = $state<HTMLDivElement | null>(null);
  let textareaScrollLeft = $state(0);
  let textareaScrollTop = $state(0);
  let addMenuOpen = $state(false);
  let addMenuAnchor = $state<HTMLDivElement | null>(null);
  let scheduleMenuOpen = $state(false);
  let scheduleMenuAnchor = $state<HTMLDivElement | null>(null);
  let scheduledMessagesOpen = $state(false);
  let scheduledMessagesAnchor = $state<HTMLDivElement | null>(null);
  let ScheduleMenu = $state<ScheduleMenuComponent | null>(null);
  let scheduleMenuLoad = $state<Promise<void> | null>(null);
  let scheduledFor = $state<string | null>(initialDraft.scheduledFor);
  let scheduledMessages = $state<ChatScheduledMessageRead[]>([]);
  let scheduledMessagesRequest = 0;
  let alsoSendToChannel = $state(false);
  let resourcePath = $state("");
  let resourceKind = $state<"file" | "folder">("file");
  let sending = $state(false);
  let stopping = $state(false);
  let retryingAssignment = $state(false);
  let error = $state<string | null>(null);
  const teammateDetails = $derived(new Map(chat.teammates.map((teammate) => [
    teammate.participant.id,
    { role: teammate.role, configurationState: teammate.configurationState },
  ])));
  const candidates = $derived(participantMentionCandidates(
    chat.selectedChannel?.memberships ?? [],
    teammateDetails,
    mentionQuery,
  ));
  const mentionSegments = $derived(participantMentionTextSegments(text, mentions));
  const invokedAiCount = $derived(new Set(mentions
    .filter((mention) => mention.participantKind === "ai_teammate")
    .map((mention) => mention.participantId)).size);
  const validation = $derived(invokedAiCount > 1 ? t("chat.organization.oneAgentOnly") : null);
  const channelName = $derived(chat.selectedChannel?.name ?? "");
  const hasMessageContent = $derived(Boolean(text.trim() || attachmentIds.length || resourceReferences.length));
  const activeAssignment = $derived.by(() => {
    const assignment = threadComposer ? chat.replyThread?.assignment : null;
    return assignment && STOPPABLE_ASSIGNMENT_STATES.has(assignment.state) ? assignment : null;
  });
  const failedAssignment = $derived.by(() => {
    const assignment = threadComposer ? chat.replyThread?.assignment : null;
    return assignment?.state === "failed" ? assignment : null;
  });
  const stopShortcut = formatShortcut("Mod + .");
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

  function resizeTextarea(): void {
    if (!textarea) return;
    const lineHeight = Number.parseFloat(getComputedStyle(textarea).lineHeight);
    if (!Number.isFinite(lineHeight) || lineHeight <= 0) return;
    const contentHeight = measureTextareaContentHeight(textarea);
    const layout = composerTextareaLayout(contentHeight, lineHeight);
    textarea.style.height = `${layout.height}px`;
    textarea.style.overflowY = layout.overflowing ? "auto" : "hidden";
  }

  function syncTextareaScroll(): void {
    if (!textarea) return;
    textareaScrollLeft = textarea.scrollLeft;
    textareaScrollTop = textarea.scrollTop;
    refreshMentionGeometry();
  }

  onMount(() => {
    if (!textarea) return;
    if (typeof ResizeObserver === "undefined") {
      resizeTextarea();
      return;
    }
    let measuredWidth = -1;
    const observer = new ResizeObserver(([entry]) => {
      if (!entry || entry.contentRect.width === measuredWidth) return;
      measuredWidth = entry.contentRect.width;
      resizeTextarea();
    });
    observer.observe(textarea);
    return () => observer.disconnect();
  });

  onMount(() => {
    const stop = () => { void stopAssignment(); };
    window.addEventListener("ganbaru-ai:chat-stop-requested", stop);
    return () => window.removeEventListener("ganbaru-ai:chat-stop-requested", stop);
  });

  $effect(() => {
    text;
    void tick().then(() => {
      resizeTextarea();
      refreshMentionGeometry();
    });
  });

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
    void tick().then(() => {
      if (!textarea) return;
      resizeTextarea();
      textarea.setSelectionRange(result.selection, result.selection);
      revealTextareaComposerCaret(textarea, result.selection);
    });
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
        || typeof mention.labelSnapshot !== "string"
        || !Number.isSafeInteger(mention.startOffset)
        || !Number.isSafeInteger(mention.endOffset)
      ) return [];
      return [{
        participantId: mention.participantId,
        participantKind: mention.participantKind as "local_user" | "ai_teammate" | "human",
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
      if (textarea) revealTextareaComposerCaret(textarea, result.selection);
    });
  }

  function insertMentionTrigger(): void {
    const start = selectionStart;
    replaceRange(start, selectionEnd, "@");
    mentionStart = start;
    mentionQuery = "";
    mentionIndex = 0;
    mentionOpen = true;
    void tick().then(() => {
      textarea?.focus();
      refreshMentionGeometry();
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
    const style = getComputedStyle(textarea);
    const lineHeight = Number.parseFloat(style.lineHeight) || 22;
    const caret = textareaCaretViewportPoint(textarea, selectionEnd);
    const menuHeight = mentionPicker?.getBoundingClientRect().height
      ?? Math.min(280, Math.max(80, candidates.length * 58));
    const topAbove = caret.top - menuHeight - MENTION_PICKER_GAP_PX;
    const topBelow = caret.top + lineHeight + MENTION_PICKER_GAP_PX;
    const spaceAbove = caret.top - MENTION_PICKER_GAP_PX - MENTION_PICKER_VIEWPORT_INSET_PX;
    const spaceBelow = window.innerHeight - topBelow - MENTION_PICKER_VIEWPORT_INSET_PX;
    const top = spaceAbove >= menuHeight
      ? topAbove
      : spaceBelow >= menuHeight
        ? topBelow
        : spaceAbove >= spaceBelow
          ? Math.max(MENTION_PICKER_VIEWPORT_INSET_PX, topAbove)
          : Math.min(
            topBelow,
            Math.max(
              MENTION_PICKER_VIEWPORT_INSET_PX,
              window.innerHeight - menuHeight - MENTION_PICKER_VIEWPORT_INSET_PX,
            ),
          );
    const left = Math.min(
      Math.max(MENTION_PICKER_VIEWPORT_INSET_PX, caret.left),
      Math.max(
        MENTION_PICKER_VIEWPORT_INSET_PX,
        window.innerWidth - MENTION_PICKER_WIDTH_PX - MENTION_PICKER_VIEWPORT_INSET_PX,
      ),
    );
    mentionStyle = `position:fixed;left:${Math.round(left)}px;top:${Math.round(top)}px;width:min(20rem,calc(100vw - 1rem))`;
  }

  function textareaCaretViewportPoint(
    element: HTMLTextAreaElement,
    caret: number,
  ): { left: number; top: number } {
    const style = getComputedStyle(element);
    const mirror = document.createElement("div");
    const marker = document.createElement("span");
    Object.assign(mirror.style, {
      position: "fixed",
      left: "-10000px",
      top: "0",
      boxSizing: style.boxSizing,
      width: `${element.clientWidth}px`,
      borderTop: style.borderTop,
      borderRight: style.borderRight,
      borderBottom: style.borderBottom,
      borderLeft: style.borderLeft,
      padding: style.padding,
      font: style.font,
      letterSpacing: style.letterSpacing,
      lineHeight: style.lineHeight,
      textAlign: style.textAlign,
      textIndent: style.textIndent,
      textTransform: style.textTransform,
      whiteSpace: "pre-wrap",
      overflowWrap: "break-word",
      visibility: "hidden",
    });
    mirror.textContent = text.slice(0, caret);
    marker.textContent = "\u200b";
    mirror.append(marker);
    document.body.append(mirror);
    const mirrorBounds = mirror.getBoundingClientRect();
    const markerBounds = marker.getBoundingClientRect();
    const elementBounds = element.getBoundingClientRect();
    const point = {
      left: elementBounds.left + markerBounds.left - mirrorBounds.left - element.scrollLeft,
      top: elementBounds.top + markerBounds.top - mirrorBounds.top - element.scrollTop,
    };
    mirror.remove();
    return point;
  }

  function candidateDisplayName(participant: ChatParticipantRead): string {
    if (participant.kind !== "local_user") return participant.displayName;
    return preferences.profileDisplayName || t("chat.timeline.you");
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
    persist();
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

  $effect(() => {
    if (!addMenuOpen && !scheduleMenuOpen && !scheduledMessagesOpen && !mentionOpen) return;
    const closeMenusFromOutside = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (addMenuOpen && !addMenuAnchor?.contains(target)) addMenuOpen = false;
      if (scheduleMenuOpen && !scheduleMenuAnchor?.contains(target)) scheduleMenuOpen = false;
      if (scheduledMessagesOpen && !scheduledMessagesAnchor?.contains(target)) scheduledMessagesOpen = false;
      if (mentionOpen && !textarea?.contains(target) && !mentionPicker?.contains(target)) mentionOpen = false;
    };
    window.addEventListener("pointerdown", closeMenusFromOutside, true);
    return () => window.removeEventListener("pointerdown", closeMenusFromOutside, true);
  });

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
        onRequestScrollToBottom();
      }
      clearComposerAfterDelivery();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      sending = false;
    }
  }

  async function stopAssignment(): Promise<void> {
    const assignment = activeAssignment;
    if (!assignment || stopping) return;
    stopping = true;
    error = null;
    try {
      await chat.cancelAssignment(assignment.id);
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      stopping = false;
    }
  }

  async function retryFailedAssignment(): Promise<void> {
    const assignment = failedAssignment;
    if (!assignment || retryingAssignment) return;
    retryingAssignment = true;
    error = null;
    try {
      await chat.retryAssignment(assignment.id);
      onRequestScrollToBottom();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      retryingAssignment = false;
    }
  }

  function performPrimaryAction(): void {
    if (activeAssignment) void stopAssignment();
    else void post();
  }

  function configureTeammate(participantId: string): void {
    mentionOpen = false;
    window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-configure-teammate", { detail: { participantId } }));
  }
</script>

<div class="composer-stack">
  {#if failedAssignment}
    <div class="assignment-error-summary" role="status">
      <CircleAlert size={13} />
      <span>{t("chat.organization.workStopped")}</span>
      <button type="button" disabled={retryingAssignment} onclick={() => void retryFailedAssignment()}>{t("chat.organization.retry")}</button>
    </div>
  {/if}
  {#if nextScheduledMessage}
    <div class="scheduled-summary">
      <Clock3 size={13} />
      <span>{t(
        "chat.organization.scheduledMessageSummary",
        scheduledMessages.length,
        formatScheduledInstant(nextScheduledMessage.scheduledFor),
      )}</span>
      <div bind:this={scheduledMessagesAnchor} class="scheduled-summary-anchor">
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
  <div class="textarea-frame">
    <div class="textarea-surface">
      {#if text}
        <div
          class="textarea-highlights"
          style={`transform:translate(${-textareaScrollLeft}px,${-textareaScrollTop}px)`}
          aria-hidden="true"
        >{#each mentionSegments as segment, index (`${segment.kind}:${index}`)}{#if segment.kind === "mention"}<mark>{segment.text}</mark>{:else}{segment.text}{/if}{/each}{#if text.endsWith("\n")}<br />{/if}</div>
      {/if}
      <textarea
        bind:this={textarea}
        class:has-highlights={Boolean(text)}
        value={text}
        rows="2"
        maxlength="131072"
        {placeholder}
        aria-label={placeholder}
        aria-autocomplete="list"
        aria-haspopup="listbox"
        aria-controls={mentionOpen ? "chat-participant-mention-picker" : undefined}
        aria-activedescendant={mentionOpen && candidates[mentionIndex] ? `chat-participant-mention-${candidates[mentionIndex].participant.id}` : undefined}
        data-chat-composer
        oninput={handleInput}
        onbeforeinput={handleBeforeInput}
        onselect={updateSelection}
        onkeyup={updateSelection}
        onclick={updateSelection}
        onkeydown={handleKeydown}
        onscroll={syncTextareaScroll}
        oncopy={handleCopy}
        onpaste={handlePaste}
        oncompositionstart={handleCompositionStart}
        oncompositionend={updateSelection}
      ></textarea>
    </div>
  </div>

  {#if mentionOpen}
    <div bind:this={mentionPicker} use:portal id="chat-participant-mention-picker" class="mention-picker" style={mentionStyle} role="listbox" aria-label={t("chat.organization.mentionTeammate")}>
      {#if candidates.length === 0}
        <p>{t("chat.organization.noMentionResults")}</p>
      {:else}
        {#each candidates as candidate, index (candidate.participant.id)}
          <div class="candidate-row" class:active={index === mentionIndex}>
            <button id={`chat-participant-mention-${candidate.participant.id}`} type="button" class="candidate-select" role="option" aria-selected={index === mentionIndex} tabindex="-1" onpointerenter={() => { mentionIndex = index; }} onpointerdown={(event) => event.preventDefault()} onclick={() => chooseMention(candidate.participant)}>
            <ChatParticipantAvatar participant={candidate.participant} size={31} />
            <span class="candidate-copy">
              <span class="candidate-identity"><strong>{candidateDisplayName(candidate.participant)}</strong></span>
              {#if candidate.role}<small>{candidate.role}</small>
              {:else if candidate.participant.kind !== "ai_teammate"}<small>{t("chat.organization.yourProfile")}</small>{/if}
            </span>
            </button>
            {#if candidate.configurationState && candidate.configurationState !== "healthy"}
              <button type="button" class="configure" onpointerdown={(event) => event.preventDefault()} onclick={() => configureTeammate(candidate.participant.id)}><Settings size={12} />{t("chat.organization.configure")}</button>
            {:else}
              <span class="candidate-kind"><i></i>{t("chat.organization.availableParticipant")}</span>
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
      <div bind:this={addMenuAnchor} class="menu-anchor">
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
      <button type="button" class="tool-button" aria-label={t("chat.organization.mentionTeammate")} onpointerdown={(event) => event.preventDefault()} onclick={insertMentionTrigger}><AtSign size={15} /></button>
      <div bind:this={scheduleMenuAnchor} class="menu-anchor">
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
    <button
      type="button"
      class="send-button"
      data-action={activeAssignment ? "stop" : "send"}
      disabled={sending || Boolean(validation) || (!activeAssignment && !hasMessageContent)}
      aria-label={activeAssignment ? t("chat.organization.stopWorkShortcut", stopShortcut) : scheduledFor ? t("chat.organization.scheduleMessage") : t("chat.composer.send")}
      aria-busy={stopping || undefined}
      title={activeAssignment ? t("chat.organization.stopWorkShortcut", stopShortcut) : undefined}
      onclick={performPrimaryAction}
    >{#if activeAssignment}<Square size={13} />{:else}<ArrowUp size={16} />{/if}</button>
  </div>
  {#if validation}<p class="composer-error" role="alert">{validation}</p>{/if}
  {#if error}<p class="composer-error" role="alert">{error}</p>{/if}
  </div>
</div>

<style>
  .composer-stack { width:min(100%,var(--chat-conversation-max-width,60rem)); }
  .assignment-error-summary { display:flex; min-height:2rem; align-items:center; gap:0.45rem; margin:0 0.35rem 0.35rem; border-radius:0.55rem; background:color-mix(in srgb,var(--destructive) 9%,transparent); padding:0.25rem 0.35rem 0.25rem 0.55rem; color:var(--destructive); }
  .assignment-error-summary :global(svg) { flex:0 0 auto; }
  .assignment-error-summary > span { min-width:0; flex:1; font-size: calc(0.7rem * var(--type-scale)); }
  .assignment-error-summary > button { min-height:1.5rem; border-radius:0.4rem; padding:0.2rem 0.45rem; font-size: calc(0.68rem * var(--type-scale)); font-weight:500; }
  .assignment-error-summary > button:hover:not(:disabled) { background:color-mix(in srgb,var(--destructive) 10%,transparent); }
  .organizational-composer { position:relative; width:100%; border:1px solid color-mix(in srgb,var(--border) 88%,transparent); border-radius:1.3rem; background:var(--card); box-shadow:0 8px 22px -18px rgb(0 0 0 / 0.24),0 1px 4px -3px rgb(0 0 0 / 0.16); }
  .scheduled-summary { display:flex; min-height:2rem; align-items:center; gap:0.45rem; margin:0 0.35rem 0.35rem; border-radius:0.55rem; background:color-mix(in srgb,var(--accent) 58%,transparent); padding:0.25rem 0.35rem 0.25rem 0.55rem; color:var(--muted-foreground); }
  .scheduled-summary :global(svg) { flex:0 0 auto; }
  .scheduled-summary > span { min-width:0; flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size: calc(0.7rem * var(--type-scale)); }
  .scheduled-summary-anchor { position:relative; flex:0 0 auto; }
  .scheduled-summary-anchor > button { min-height:1.5rem; border-radius:0.4rem; padding:0.2rem 0.45rem; color:var(--foreground); font-size: calc(0.68rem * var(--type-scale)); font-weight:500; }
  .scheduled-summary-anchor > button:hover,.scheduled-summary-anchor > button[aria-expanded="true"] { background:color-mix(in srgb,var(--background) 70%,transparent); }
  .textarea-frame { padding:1rem 1.25rem 0; }
  .textarea-surface { position:relative; overflow:hidden; }
  textarea,.textarea-highlights { box-sizing:border-box; width:100%; min-height:2lh; border:0; padding:0; font:inherit; font-size:var(--chat-conversation-font-size,calc(0.875rem * var(--type-scale))); line-height:var(--chat-conversation-line-height,calc(1.3125rem * var(--type-scale))); overflow-wrap:break-word; white-space:pre-wrap; }
  textarea { position:relative; display:block; max-height:6lh; resize:none; overflow-y:hidden; background:transparent; color:var(--foreground); outline:none; }
  textarea.has-highlights { color:transparent; caret-color:var(--foreground); -webkit-text-fill-color:transparent; }
  .textarea-highlights { pointer-events:none; position:absolute; inset:0; color:var(--foreground); transform-origin:top left; }
  .textarea-highlights mark { border-radius:0.2rem; background:color-mix(in srgb,var(--primary) 15%,transparent); color:color-mix(in srgb,var(--primary) 76%,var(--foreground)); }
  textarea::placeholder { color:color-mix(in srgb,var(--muted-foreground) 52%,transparent); }
  .composer-footer { display:flex; min-height:2.6rem; align-items:center; justify-content:space-between; gap:0.5rem; padding:0 0.75rem 0.5rem; }
  .composer-tools { display:flex; align-items:center; gap:0.3rem; }
  .menu-anchor { position:relative; }
  .tool-button { display:grid; width:1.9rem; height:1.9rem; place-items:center; border-radius:0.5rem; color:var(--muted-foreground); }
  .tool-button:hover { background:var(--accent); color:var(--foreground); }
  .tool-button.active { background:var(--accent); color:var(--foreground); }
  .send-button { display:grid; width:2rem; height:2rem; flex:0 0 auto; place-items:center; border-radius:999px; background:color-mix(in srgb,var(--primary) 92%,transparent); color:var(--primary-foreground); box-shadow:0 2px 7px color-mix(in srgb,var(--primary) 22%,transparent); transition:transform 120ms ease,filter 120ms ease; }
  .send-button:hover:not(:disabled) { filter:brightness(1.04); transform:scale(1.04); }
  .send-button:disabled { opacity:0.45; }
  .composer-menu { position:absolute; z-index:70; min-width:14rem; border:1px solid var(--border); border-radius:0.5rem; background:var(--popover); padding:0.3rem; box-shadow:0 10px 30px rgb(0 0 0 / 0.16); }
  .composer-menu > button { display:flex; width:100%; min-height:2rem; align-items:center; gap:0.45rem; border-radius:0.35rem; padding:0.35rem 0.5rem; text-align:left; font-size: calc(0.75rem * var(--type-scale)); }
  .composer-menu > button:hover { background:var(--accent); }
  .add-menu { bottom:calc(100% + 0.35rem); left:0; }
  .schedule-loading { bottom:calc(100% + 0.35rem); left:0; color:var(--muted-foreground); font-size: calc(0.72rem * var(--type-scale)); }
  .schedule-loading.align-right { right:0; left:auto; }
  .resource-entry { display:grid; gap:0.35rem; border-top:1px solid var(--border); padding:0.45rem 0.35rem 0.25rem; }
  .resource-entry > div { display:flex; gap:0.25rem; }
  .resource-entry button { display:flex; align-items:center; gap:0.25rem; border-radius:0.3rem; padding:0.25rem 0.4rem; font-size: calc(0.7rem * var(--type-scale)); }
  .resource-entry button.active { background:var(--accent); }
  .resource-entry input { min-width:0; border:1px solid var(--border); border-radius:0.35rem; background:var(--background); padding:0.35rem 0.45rem; font-size: calc(0.72rem * var(--type-scale)); outline:none; }
  .context-chips { display:flex; flex-wrap:wrap; gap:0.3rem; padding:0 0.75rem 0.35rem; }
  .context-chips button { display:flex; max-width:14rem; align-items:center; gap:0.25rem; border-radius:999px; background:var(--accent); padding:0.22rem 0.45rem; font-size: calc(0.68rem * var(--type-scale)); }
  .context-chips button span { color:var(--muted-foreground); }
  .mention-picker { z-index:100; max-height:min(20rem,60vh); overflow-y:auto; border:1px solid color-mix(in srgb,var(--border) 92%,var(--foreground)); border-radius:0.7rem; background:var(--popover); padding:0.35rem; }
  .mention-picker :global(svg.lucide) { stroke-width:var(--icon-stroke-width); }
  .candidate-row { display:flex; width:100%; min-height:3.35rem; align-items:flex-start; gap:0.35rem; border-radius:0.45rem; padding:0.3rem 0.4rem; }.candidate-row:is(:hover,.active) { background:var(--accent); }
  .candidate-select { display:flex; min-width:0; flex:1; align-items:center; gap:0.6rem; text-align:left; }
  .mention-picker > p { padding:0.65rem; color:var(--muted-foreground); font-size: calc(0.75rem * var(--type-scale)); }
  .candidate-copy { display:grid; min-width:0; flex:1; }
  .candidate-copy strong,.candidate-copy small { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .candidate-copy strong { font-size: calc(0.76rem * var(--type-scale)); }.candidate-copy small { color:var(--muted-foreground); font-size: calc(0.67rem * var(--type-scale)); }
  .candidate-identity { display:flex; min-width:0; align-items:baseline; gap:0.4rem; }
  .candidate-identity strong { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .candidate-kind { display:flex; flex:0 0 auto; align-items:center; gap:0.3rem; margin-top:0.18rem; color:var(--muted-foreground); font-size:calc(0.6rem * var(--type-scale)); }
  .candidate-kind i { width:0.38rem; height:0.38rem; border-radius:999px; background:var(--action-confirm); }
  .configure { display:flex; flex:0 0 auto; align-items:center; gap:0.25rem; margin-top:0.02rem; border-radius:0.35rem; padding:0.25rem 0.35rem; color:var(--destructive); font-size:calc(0.65rem * var(--type-scale)); }
  .configure:hover,.configure:focus-visible { background:color-mix(in srgb,var(--destructive) 9%,transparent); }
  .composer-error { padding:0 0.75rem 0.5rem; color:var(--destructive); font-size: calc(0.7rem * var(--type-scale)); }
  @media (forced-colors:active) { .organizational-composer,.composer-menu,.mention-picker { border:1px solid CanvasText; } }
</style>
