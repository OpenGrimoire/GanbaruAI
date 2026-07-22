<script module lang="ts">
  let restoreComposerFocus = false;
  let restoreSelectionStart = 0;
  let restoreSelectionEnd = 0;
</script>

<script lang="ts">
  import { onMount, tick } from "svelte";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import AtSign from "@lucide/svelte/icons/at-sign";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import Square from "@lucide/svelte/icons/square";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import type { ChatPromptCatalogEntry, ChatWorkspacePathRead, ProviderCapabilities } from "$lib/chat/contracts";
  import {
    autosizeComposerHeight,
    composerActionState,
    composerTokenTrigger,
    contextMeter,
    filterPromptCatalog,
    replaceComposerToken,
    shouldSendComposerKey,
    validateComposerSelections,
    validateImageFiles,
    validateModelOptions,
  } from "$lib/chat/composer-model";
  import { formatDateTime, formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatModelControls from "./ChatModelControls.svelte";
  import ChatRequestPanel from "./ChatRequestPanel.svelte";

  const { hero = false } = $props<{ hero?: boolean }>();
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let textarea: HTMLTextAreaElement | undefined = $state();
  let textareaFocused = false;
  let fileInput: HTMLInputElement | undefined = $state();
  let menuEntries = $state<(ChatWorkspacePathRead | ChatPromptCatalogEntry)[]>([]);
  let menuKind = $state<"mention" | "skill" | "command" | null>(null);
  let menuIndex = $state(0);
  let menuLoading = $state(false);
  let includeIgnored = $state(false);
  let menuCursor = $state<string | null>(null);
  let menuRequest = 0;
  let operationError = $state<string | null>(null);
  let sending = $state(false);
  let importing = $state(false);
  let previewAttachmentId = $state<string | null>(null);
  let previewUrl = $state<string | null>(null);
  let previewDialog: HTMLDivElement | undefined = $state();
  let previewReturnFocus: HTMLElement | null = null;
  let thumbnailUrls = $state<Record<string, string>>({});
  let forceStopAvailable = $state(false);
  let stopTimer: ReturnType<typeof setTimeout> | null = null;
  const provider = $derived(chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const capabilities = $derived<ProviderCapabilities>(chat.interaction?.capabilities ?? provider?.lastProbe?.capabilities ?? { entries: [] });
  const pending = $derived(chat.interaction?.pendingRequest ?? null);
  const hasDraft = $derived(Boolean(chat.composer.text.trim()) || chat.composer.attachmentIds.length > 0 || chat.composer.mentions.length > 0);
  const action = $derived(composerActionState(chat.interaction?.sessionState ?? "stopped", capabilities, hasDraft, pending !== null));
  const latestUsage = $derived(chat.interaction?.usage ?? chat.timelinePages.flatMap((page) => page.turns).at(-1)?.usage ?? null);
  const activeTurn = $derived(chat.timelinePages
    .flatMap((page) => page.turns)
    .find((turn) => turn.turnId === chat.interaction?.activeTurnId) ?? null);
  const meter = $derived(contextMeter(latestUsage?.contextTokens ?? null, latestUsage?.contextLimit ?? null));
  const hasProviderStatus = $derived(Boolean(
    chat.interaction?.accountStatus
    || chat.interaction?.rateLimitStatus
    || latestUsage?.cost?.providerReported,
  ));
  const previewAttachment = $derived(chat.composerAttachments.find((attachment) => attachment.id === previewAttachmentId) ?? null);

  onMount(() => {
    const stop = () => void stopTurn();
    const continuePlan = () => preparePlanTurn("plan");
    const implementPlan = () => preparePlanTurn("build");
    window.addEventListener("ganbaru-ai:chat-stop-requested", stop);
    window.addEventListener("ganbaru-ai:chat-continue-plan", continuePlan);
    window.addEventListener("ganbaru-ai:chat-implement-plan", implementPlan);
    if (restoreComposerFocus) {
      void tick().then(() => {
        textarea?.focus();
        textarea?.setSelectionRange(restoreSelectionStart, restoreSelectionEnd);
      });
    }
    return () => {
      restoreComposerFocus = textareaFocused;
      if (textarea) {
        restoreSelectionStart = textarea.selectionStart;
        restoreSelectionEnd = textarea.selectionEnd;
      }
      window.removeEventListener("ganbaru-ai:chat-stop-requested", stop);
      window.removeEventListener("ganbaru-ai:chat-continue-plan", continuePlan);
      window.removeEventListener("ganbaru-ai:chat-implement-plan", implementPlan);
      if (stopTimer !== null) clearTimeout(stopTimer);
      void chat.flushComposer().catch(() => undefined);
    };
  });

  function preparePlanTurn(mode: "plan" | "build"): void {
    const action = mode === "plan"
      ? t("chat.inspector.continuePlanningPrompt")
      : t("chat.inspector.implementPlanPrompt");
    const existing = chat.composer.text.trim();
    chat.setComposerText(existing ? `${existing}\n\n${action}` : action);
    chat.setComposerModes(chat.composer.safetyMode, mode);
    queueMicrotask(() => textarea?.focus());
  }

  $effect(() => {
    chat.composer.text;
    void tick().then(autosize);
  });

  $effect(() => {
    for (const attachment of chat.composerAttachments) {
      if (thumbnailUrls[attachment.id]) continue;
      void chatApi.chatAttachmentDataUrl(attachment.id).then((url) => {
        thumbnailUrls = { ...thumbnailUrls, [attachment.id]: url };
      }).catch(() => undefined);
    }
  });

  $effect(() => {
    if (chat.interaction?.sessionState !== "stopping") forceStopAvailable = false;
  });

  function autosize(): void {
    if (!textarea) return;
    textarea.style.height = "0px";
    const lineHeight = Number.parseFloat(getComputedStyle(textarea).lineHeight) || 20;
    textarea.style.height = `${autosizeComposerHeight(textarea.scrollHeight, lineHeight)}px`;
    textarea.style.overflowY = textarea.scrollHeight > textarea.clientHeight ? "auto" : "hidden";
  }

  function handleInput(event: Event): void {
    const target = event.currentTarget as HTMLTextAreaElement;
    rememberSelection(target);
    chat.setComposerText(target.value);
    void updateMenu(target.value, target.selectionStart);
  }

  function rememberSelection(target: HTMLTextAreaElement): void {
    restoreSelectionStart = target.selectionStart;
    restoreSelectionEnd = target.selectionEnd;
  }

  async function updateMenu(text: string, cursor: number): Promise<void> {
    const trigger = composerTokenTrigger(text, cursor);
    if (!trigger || !chat.composer.workspaceId || !chat.composer.providerInstanceId) {
      closeMenu();
      return;
    }
    const request = ++menuRequest;
    menuKind = trigger.kind;
    menuLoading = true;
    menuIndex = 0;
    try {
      if (trigger.kind === "mention") {
        const page = await chatApi.searchChatWorkspacePaths(chat.composer.workspaceId, trigger.query, includeIgnored);
        if (request !== menuRequest) return;
        menuEntries = page.entries;
        menuCursor = page.nextCursor;
      } else {
        const catalog = await chatApi.listChatPromptCatalog(chat.composer.providerInstanceId);
        if (request !== menuRequest) return;
        menuEntries = filterPromptCatalog(catalog, trigger.kind, trigger.query);
        menuCursor = null;
      }
    } catch (cause: unknown) {
      if (request === menuRequest) operationError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (request === menuRequest) menuLoading = false;
    }
  }

  async function loadMoreMentions(): Promise<void> {
    if (!menuCursor || !chat.composer.workspaceId || !textarea) return;
    const trigger = composerTokenTrigger(textarea.value, textarea.selectionStart);
    if (!trigger) return;
    const page = await chatApi.searchChatWorkspacePaths(chat.composer.workspaceId, trigger.query, includeIgnored, menuCursor);
    menuEntries = [...menuEntries, ...page.entries];
    menuCursor = page.nextCursor;
  }

  function chooseMenuEntry(index: number): void {
    const entry = menuEntries[index];
    if (!entry || !textarea) return;
    const trigger = composerTokenTrigger(chat.composer.text, textarea.selectionStart);
    if (!trigger) return;
    if (menuKind === "mention" && "relativePath" in entry) {
      if (!chat.composer.mentions.some((mention) => mention.relativePath === entry.relativePath)) {
        chat.setComposerMentions([...chat.composer.mentions, { relativePath: entry.relativePath, kind: entry.kind, ignored: entry.ignored }]);
      }
      chat.setComposerText(`${chat.composer.text.slice(0, trigger.start)}${chat.composer.text.slice(trigger.end)}`);
    } else if ("value" in entry) {
      chat.setComposerText(replaceComposerToken(chat.composer.text, trigger, entry.value));
    }
    closeMenu();
    void tick().then(() => textarea?.focus());
  }

  function closeMenu(): void {
    menuRequest += 1;
    menuKind = null;
    menuEntries = [];
    menuCursor = null;
    menuLoading = false;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (menuKind) {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        const direction = event.key === "ArrowDown" ? 1 : -1;
        menuIndex = (menuIndex + direction + Math.max(1, menuEntries.length)) % Math.max(1, menuEntries.length);
        return;
      }
      if (event.key === "Enter" && menuEntries.length > 0) {
        event.preventDefault();
        chooseMenuEntry(menuIndex);
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        closeMenu();
        return;
      }
    }
    const sendKey = chat.settings?.configuration.behavior.sendKey ?? "enter";
    if (shouldSendComposerKey(event, sendKey)) {
      event.preventDefault();
      void performComposerAction();
    }
  }

  async function performComposerAction(): Promise<void> {
    if (sending || action.primary === "stopping" || action.primary === "resolve_request") return;
    if (action.primary === "stop") {
      if (action.followup === "steer") await run(() => chat.steerComposer());
      else if (action.followup === "queue") await run(() => chat.queueComposer());
      return;
    }
    if (action.followup === "retain") return;
    await send();
  }

  async function performPrimaryAction(): Promise<void> {
    if (action.primary === "stop") await stopTurn();
    else await performComposerAction();
  }

  async function send(): Promise<void> {
    const workspaceId = chat.composer.workspaceId;
    const providerId = chat.composer.providerInstanceId;
    const trusted = workspaceId && providerId ? await chatApi.hasChatFullAccessTrust(providerId, workspaceId) : false;
    const model = chat.composer.modelSelection?.value;
    const modelId = typeof model === "object" && model !== null && !Array.isArray(model) && typeof model.modelId === "string" ? model.modelId : null;
    const providerManagedModel = typeof model === "object" && model !== null && !Array.isArray(model) && model.providerManaged === true;
    const errors = validateComposerSelections({
      workspaceId,
      providerInstanceId: providerId,
      modelId,
      providerManagedModel,
      safetyMode: chat.composer.safetyMode,
      interactionMode: chat.composer.interactionMode,
      fullAccessTrusted: chat.composer.safetyMode !== "full_access" || trusted,
    }, capabilities);
    const modelOptionErrors = provider && modelId
      ? validateModelOptions(provider.modelCatalog?.models.find((entry) => entry.id === modelId)?.options ?? [], readComposerOptions())
      : [];
    if (errors.length > 0 || modelOptionErrors.length > 0 || !hasDraft) {
      const field = errors[0]?.field;
      operationError = field ? selectionError(field) : modelOptionErrors.length > 0
        ? t("chat.composer.invalidTrait")
        : t("chat.composer.messageRequired");
      const targetField = field === "trust" ? "safety" : field;
      const target = targetField
        ? document.querySelector<HTMLElement>(`[data-chat-field="${targetField}"]`)
        : textarea;
      target?.focus();
      return;
    }
    await run(() => chat.sendComposer());
  }

  function readComposerOptions(): import("$lib/chat/contracts").ModelOptionSelection[] {
    const value = chat.composer.modelSelection?.value;
    if (typeof value !== "object" || value === null || Array.isArray(value) || !Array.isArray(value.options)) return [];
    return value.options as unknown as import("$lib/chat/contracts").ModelOptionSelection[];
  }

  function selectionError(field: import("$lib/chat/composer-model").ComposerSelectionError["field"]): string {
    switch (field) {
      case "workspace": return t("chat.composer.chooseWorkspace");
      case "provider": return t("chat.composer.chooseProvider");
      case "model": return t("chat.composer.chooseModel");
      case "safety": return t("chat.composer.chooseSafety");
      case "interaction": return t("chat.composer.chooseInteraction");
      case "trust": return t("chat.composer.confirmTrust");
    }
  }

  async function stopTurn(): Promise<void> {
    operationError = null;
    try {
      await chat.stop(false);
      if (stopTimer !== null) clearTimeout(stopTimer);
      stopTimer = setTimeout(() => {
        forceStopAvailable = chat.interaction?.sessionState === "stopping";
      }, 10_000);
    } catch (cause: unknown) {
      operationError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function run(operation: () => Promise<void>): Promise<void> {
    sending = true;
    operationError = null;
    try {
      await operation();
    } catch (cause: unknown) {
      operationError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      sending = false;
    }
  }

  async function importFiles(files: File[]): Promise<void> {
    const validation = validateImageFiles(
      files,
      chat.composerAttachments.length,
      chat.composerAttachments.reduce((total, attachment) => total + attachment.byteSize, 0),
    );
    if (validation) { operationError = validation; return; }
    importing = true;
    operationError = null;
    try { await chat.importComposerImages(files); }
    catch (cause: unknown) { operationError = cause instanceof Error ? cause.message : String(cause); }
    finally { importing = false; }
  }

  function handlePaste(event: ClipboardEvent): void {
    const files = [...(event.clipboardData?.files ?? [])].filter((file) => file.type.startsWith("image/"));
    if (files.length > 0) { event.preventDefault(); void importFiles(files); }
  }

  function handleDrop(event: DragEvent): void {
    event.preventDefault();
    const files = [...(event.dataTransfer?.files ?? [])].filter((file) => file.type.startsWith("image/"));
    if (files.length > 0) void importFiles(files);
  }

  async function openPreview(attachmentId: string): Promise<void> {
    previewReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    previewAttachmentId = attachmentId;
    previewUrl = thumbnailUrls[attachmentId] ?? await chatApi.chatAttachmentDataUrl(attachmentId);
    await tick();
    previewDialog?.querySelector<HTMLElement>("button")?.focus();
  }

  function closePreview(): void {
    previewAttachmentId = null;
    previewUrl = null;
    const target = previewReturnFocus;
    queueMicrotask(() => target?.isConnected && target.focus());
  }

  function handlePreviewKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closePreview();
      return;
    }
    if (event.key === "Tab") {
      event.preventDefault();
      previewDialog?.querySelector<HTMLElement>("button")?.focus();
    }
  }

  function contextLabel(): string | null {
    if (!meter) return null;
    const used = formatNumber(localization.locale, meter.usedTokens);
    return meter.maximumTokens === null
      ? t("chat.composer.contextUnknown", used)
      : t("chat.composer.contextUsed", used, formatNumber(localization.locale, meter.maximumTokens));
  }

  function costLabel(): string | null {
    const cost = latestUsage?.cost;
    if (!cost?.providerReported) return null;
    const amount = formatNumber(localization.locale, cost.amount, { maximumFractionDigits: 6 });
    return t("chat.composer.cost", amount, cost.currency);
  }

  function resetLabel(): string | null {
    const resetsAt = chat.interaction?.rateLimitStatus?.resetsAt;
    if (!resetsAt) return null;
    const timestamp = Date.parse(resetsAt);
    return Number.isNaN(timestamp)
      ? null
      : t("chat.composer.resetsAt", formatDateTime(localization.locale, timestamp, { dateStyle: "medium", timeStyle: "short" }));
  }
</script>

<section class:hero class="chat-composer" data-chat-composer-container role="group" ondragover={(event) => event.preventDefault()} ondrop={handleDrop}>
  {#if chat.interaction?.queuedFollowup}<div class="queued-row"><div><strong>{t("chat.composer.queued")}</strong><p>{chat.interaction.queuedFollowup.text}</p></div><button type="button" onclick={() => void chat.editQueuedFollowup()}>{t("chat.composer.editQueued")}</button><button type="button" onclick={() => void chat.cancelQueuedFollowup()}>{t("chat.composer.cancelQueued")}</button></div>{/if}
  {#if chat.sendError}<div role="alert" class="recovery-row"><strong>{t("chat.composer.launchFailed")}</strong><span>{chat.sendError}</span><button type="button" onclick={() => void run(() => chat.retryFailedSend())}>{t("chat.timeline.retry")}</button><button type="button" onclick={() => void chat.editFailedSend()}>{t("chat.composer.editDraft")}</button><button type="button" onclick={() => void chat.changeProviderAfterFailure()}>{t("chat.composer.changeProvider")}</button></div>{/if}
  <ChatModelControls compact={!hero} />
  {#if activeTurn}<p class="active-turn-modes">{t("chat.composer.activeTurnModes", activeTurn.modes.safetyMode === "supervised" ? t("chat.hero.supervised") : activeTurn.modes.safetyMode === "auto_accept_edits" ? t("chat.hero.autoAccept") : t("chat.hero.fullAccess"), activeTurn.modes.interactionMode === "plan" ? t("chat.hero.plan") : t("chat.hero.build"))}</p>{/if}
  {#if chat.composerAttachments.length > 0}<div class="attachment-grid">{#each chat.composerAttachments as attachment}<article><button type="button" class="attachment-preview" aria-label={t("chat.composer.previewAttachment", attachment.originalDisplayName)} onclick={() => void openPreview(attachment.id)}>{#if thumbnailUrls[attachment.id]}<img src={thumbnailUrls[attachment.id]} alt={attachment.originalDisplayName} />{:else}<LoaderCircle size={16} class="animate-spin" />{/if}</button><span title={attachment.originalDisplayName}>{attachment.originalDisplayName}</span><button type="button" aria-label={t("chat.composer.removeAttachment", attachment.originalDisplayName)} onclick={() => chat.removeComposerAttachment(attachment.id)}><X size={12} /></button></article>{/each}</div>{/if}
  {#if chat.composer.mentions.length > 0}<div class="mention-chips">{#each chat.composer.mentions as mention}<span title={mention.relativePath}><AtSign size={11} />{mention.relativePath}{#if mention.ignored}<small>{t("chat.composer.ignored")}</small>{/if}<button type="button" aria-label={t("chat.composer.removeAttachment", mention.relativePath)} onclick={() => chat.setComposerMentions(chat.composer.mentions.filter((entry) => entry.relativePath !== mention.relativePath))}><X size={10} /></button></span>{/each}</div>{/if}
  <div class="editor-shell">
    <textarea bind:this={textarea} data-chat-composer value={chat.composer.text} placeholder={action.primary === "stop" ? t("chat.composer.placeholderWorking") : t("chat.composer.placeholder")} disabled={chat.selectedThread?.archivedAt !== null && chat.selectedThread !== null} aria-label={t("chat.composer.placeholder")} onfocus={() => { textareaFocused = true; restoreComposerFocus = true; }} onblur={(event) => { const target = event.currentTarget; queueMicrotask(() => { if (target.isConnected) { textareaFocused = false; restoreComposerFocus = false; } }); }} onselect={(event) => rememberSelection(event.currentTarget)} onkeyup={(event) => rememberSelection(event.currentTarget)} oninput={handleInput} onkeydown={handleKeydown} onpaste={handlePaste}></textarea>
    {#if menuKind}<div class="composer-menu" role="listbox" aria-label={menuKind === "mention" ? t("chat.composer.mentionFiles") : menuKind === "skill" ? "$ skills" : "/ commands"}>{#if menuKind === "mention"}<label><input type="checkbox" bind:checked={includeIgnored} onchange={() => textarea && void updateMenu(textarea.value, textarea.selectionStart)} />{t("chat.composer.showIgnored")}</label>{/if}{#if menuLoading}<p><LoaderCircle size={13} class="animate-spin" />{t("common.loading")}</p>{:else if menuEntries.length === 0}<p>{t("chat.composer.noMatches")}</p>{:else}{#each menuEntries as entry, index}<button type="button" class:selected={index === menuIndex} role="option" aria-selected={index === menuIndex} onclick={() => chooseMenuEntry(index)}>{#if "relativePath" in entry}<strong>{entry.displayName}</strong><small>{entry.relativePath}{#if entry.ignored} · {t("chat.composer.ignored")}{/if}</small>{:else}<strong>{entry.value} · {entry.label}</strong>{#if entry.description}<small>{entry.description}</small>{/if}{#if entry.stale}<small>{t("chat.composer.staleEntry")}</small>{/if}{/if}</button>{/each}{#if menuCursor}<button type="button" onclick={() => void loadMoreMentions()}>{t("chat.composer.loadMore")}</button>{/if}{/if}</div>{/if}
    <div class="composer-toolbar"><div class="toolbar-left"><input bind:this={fileInput} class="sr-only" type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple onchange={(event) => void importFiles([...(event.currentTarget.files ?? [])])} /><button type="button" aria-label={t("chat.composer.attachImages")} title={t("chat.composer.imageLimit")} disabled={importing} onclick={() => fileInput?.click()}>{#if importing}<LoaderCircle size={14} class="animate-spin" />{:else}<Paperclip size={14} />{/if}</button><button type="button" aria-label={t("chat.composer.attachImages")} onclick={() => void run(() => chat.pickComposerImages(t("chat.composer.imagePickerTitle")))}><ImagePlus size={14} /></button><button type="button" aria-label={t("chat.composer.mentionFiles")} onclick={() => { const start = textarea?.selectionStart ?? chat.composer.text.length; chat.setComposerText(`${chat.composer.text.slice(0, start)}@${chat.composer.text.slice(start)}`); void tick().then(() => { if (textarea) { textarea.focus(); textarea.setSelectionRange(start + 1, start + 1); void updateMenu(textarea.value, start + 1); } }); }}><AtSign size={14} /></button>{#if contextLabel()}<span class:warning={meter?.warning} title={chat.interaction?.automaticCompactionReported ? t("chat.composer.automaticCompactionReported") : meter?.warning ? t("chat.composer.contextCompaction") : undefined}>{#if meter?.ratio !== null}<meter min="0" max="1" value={meter?.ratio ?? 0} aria-label={contextLabel() ?? undefined}></meter>{/if}{contextLabel()}</span>{/if}{#if hasProviderStatus}<details class="provider-status"><summary>{t("chat.composer.providerStatus")}</summary><div>{#if chat.interaction?.accountStatus}<strong>{t("chat.composer.account")}</strong>{#if chat.interaction.accountStatus.accountLabel}<p>{chat.interaction.accountStatus.accountLabel}</p>{/if}{#if chat.interaction.accountStatus.planLabel}<p>{chat.interaction.accountStatus.planLabel}</p>{/if}{/if}{#if chat.interaction?.rateLimitStatus}<strong>{t("chat.composer.rateLimit")}</strong><p>{chat.interaction.rateLimitStatus.limited ? t("chat.composer.rateLimited") : t("chat.composer.rateAvailable")}</p>{#if chat.interaction.rateLimitStatus.detail}<p>{chat.interaction.rateLimitStatus.detail}</p>{/if}{#if resetLabel()}<p>{resetLabel()}</p>{/if}{/if}{#if costLabel()}<strong>{costLabel()}</strong>{/if}</div></details>{/if}</div><div class="toolbar-right">{#if action.followup === "retain"}<small>{t("chat.composer.retained")}</small>{/if}{#if action.followup === "steer"}<button type="button" onclick={() => void run(() => chat.steerComposer())}><ArrowUp size={14} />{t("chat.composer.steer")}</button>{:else if action.followup === "queue"}<button type="button" onclick={() => void run(() => chat.queueComposer())}><ArrowUp size={14} />{t("chat.composer.queue")}</button>{/if}{#if forceStopAvailable}<button type="button" class="force-stop" title={t("chat.composer.forceStopDescription")} onclick={() => void run(() => chat.stop(true))}>{t("chat.composer.forceStop")}</button>{/if}{#if action.primary !== "resolve_request"}<button type="button" class="primary-action" disabled={sending || action.primary === "stopping" || (action.primary === "send" && !action.sendEnabled)} aria-label={action.primary === "stop" ? t("chat.composer.stop") : t("chat.composer.send")} onclick={() => void performPrimaryAction()}>{#if sending}<LoaderCircle size={14} class="animate-spin" />{:else if action.primary === "stop"}<Square size={13} />{t("chat.composer.stop")}{:else if action.primary === "stopping"}<LoaderCircle size={14} class="animate-spin" />{t("chat.composer.stopping")}{:else}<ArrowUp size={14} />{t("chat.composer.send")}{/if}</button>{/if}</div></div>
  </div>
  {#if pending}<ChatRequestPanel {pending} />{/if}
  {#if operationError || chat.composer.error}<p role="alert" class="composer-error">{operationError ?? chat.composer.error}</p>{/if}
</section>

{#if previewAttachment && previewUrl}<div class="fixed inset-0 z-60 grid place-items-center bg-black/50 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closePreview}></button><div bind:this={previewDialog} class="relative flex max-h-full max-w-full flex-col rounded-lg border border-border bg-background p-3 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.composer.previewAttachment", previewAttachment.originalDisplayName)} tabindex="-1" onkeydown={handlePreviewKeydown}><header class="mb-2 flex items-center gap-2"><strong class="min-w-0 flex-1 truncate text-sm">{previewAttachment.originalDisplayName}</strong><span class="text-xs text-muted-foreground">{formatNumber(localization.locale, previewAttachment.byteSize)} B</span><button type="button" aria-label={t("chat.cancel")} onclick={closePreview}><X size={14} /></button></header><img class="min-h-0 max-h-[75vh] max-w-[85vw] object-contain" src={previewUrl} alt={previewAttachment.originalDisplayName} /></div></div>{/if}

<style>
  .chat-composer { container-type: inline-size; container-name: chat-composer; position: relative; display: grid; gap: 0.5rem; width: min(100% - 1rem, 52rem); margin: 0 auto 0.5rem; border: 1px solid var(--border); border-radius: 0.75rem; background: var(--card); padding: 0.65rem; box-shadow: 0 8px 24px rgb(0 0 0 / 0.08); }
  .chat-composer.hero { width: min(100%, 52rem); margin-top: 1.25rem; margin-bottom: 0; text-align: left; }
  .editor-shell { position: relative; border: 1px solid var(--border); border-radius: 0.55rem; background: var(--background); }
  .active-turn-modes { color: var(--muted-foreground); font-size: 0.666667rem; }
  textarea[data-chat-composer] { display: block; width: 100%; min-height: 44px; resize: none; background: transparent; padding: 0.65rem 0.7rem 0.25rem; color: var(--foreground); font-size: 0.866667rem; line-height: 1.35rem; outline: none; }
  .composer-toolbar { display: flex; min-height: 2.25rem; align-items: center; justify-content: space-between; gap: 0.4rem; padding: 0.25rem 0.35rem; }
  .toolbar-left, .toolbar-right { display: flex; min-width: 0; align-items: center; gap: 0.25rem; }
  .toolbar-left button, .toolbar-right button { display: inline-flex; min-height: 1.9rem; align-items: center; justify-content: center; gap: 0.3rem; border-radius: 0.35rem; padding: 0.25rem 0.4rem; font-size: 0.7rem; }
  .toolbar-left button:hover, .toolbar-right button:hover { background: var(--accent); }
  .toolbar-left span { overflow: hidden; max-width: 12rem; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); font-size: 0.666667rem; }
  .toolbar-left span.warning { color: var(--status-tentative); }
  .toolbar-left meter { width: 2.5rem; height: 0.35rem; }
  .provider-status { position: relative; }
  .provider-status summary { cursor: pointer; color: var(--muted-foreground); font-size: 0.666667rem; }
  .provider-status > div { position: absolute; left: 0; bottom: calc(100% + 0.4rem); z-index: 30; width: min(20rem, 80vw); border: 1px solid var(--border); border-radius: 0.4rem; background: var(--popover); padding: 0.55rem; box-shadow: 0 8px 24px rgb(0 0 0 / 0.2); }
  .provider-status strong, .provider-status p { display: block; margin: 0.1rem 0; white-space: normal; font-size: 0.666667rem; }
  .toolbar-right small { max-width: 18rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .primary-action { background: var(--primary); color: var(--primary-foreground); }
  .primary-action:disabled { opacity: 0.5; }
  .force-stop { color: var(--destructive); }
  .attachment-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr)); gap: 0.4rem; }
  .attachment-grid article { position: relative; display: grid; grid-template-columns: 2.5rem minmax(0, 1fr) auto; align-items: center; gap: 0.35rem; border: 1px solid var(--border); border-radius: 0.45rem; padding: 0.3rem; }
  .attachment-grid article > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.666667rem; }
  .attachment-preview { display: grid; width: 2.5rem; height: 2.5rem; place-items: center; overflow: hidden; border-radius: 0.3rem; background: var(--muted); }
  .attachment-preview img { width: 100%; height: 100%; object-fit: cover; }
  .mention-chips { display: flex; flex-wrap: wrap; gap: 0.3rem; }
  .mention-chips > span { display: inline-flex; max-width: 100%; align-items: center; gap: 0.2rem; border: 1px solid var(--border); border-radius: 999px; padding: 0.2rem 0.4rem; font-size: 0.666667rem; }
  .mention-chips small { color: var(--status-tentative); }
  .composer-menu { position: absolute; inset-inline: 0; bottom: calc(100% + 0.35rem); z-index: 25; max-height: min(20rem, 55vh); overflow: auto; border: 1px solid var(--border); border-radius: 0.5rem; background: var(--popover); padding: 0.35rem; box-shadow: 0 12px 30px rgb(0 0 0 / 0.22); }
  .composer-menu > label, .composer-menu > p { display: flex; align-items: center; gap: 0.35rem; padding: 0.35rem; color: var(--muted-foreground); font-size: 0.7rem; }
  .composer-menu > button { display: block; width: 100%; border-radius: 0.35rem; padding: 0.4rem; text-align: left; }
  .composer-menu > button:hover, .composer-menu > button.selected { background: var(--accent); }
  .composer-menu strong, .composer-menu small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .composer-menu strong { font-size: 0.733333rem; }
  .composer-menu small { color: var(--muted-foreground); font-size: 0.666667rem; }
  .queued-row, .recovery-row { display: flex; flex-wrap: wrap; align-items: center; gap: 0.4rem; border: 1px solid var(--border); border-radius: 0.45rem; padding: 0.45rem; font-size: 0.7rem; }
  .queued-row div { min-width: 0; flex: 1; }
  .queued-row p { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); }
  .queued-row button, .recovery-row button { border-radius: 0.3rem; padding: 0.25rem 0.4rem; }
  .recovery-row { border-color: color-mix(in oklab, var(--destructive) 45%, var(--border)); color: var(--destructive); }
  .composer-error { color: var(--destructive); font-size: 0.733333rem; }
  @container chat-composer (max-width: 500px) { .composer-toolbar { align-items: flex-end; } .toolbar-left, .toolbar-right { flex-wrap: wrap; } .toolbar-left span { max-width: 8rem; } .toolbar-right small { max-width: 10rem; } }
  @container chat-composer (max-width: 300px) { .chat-composer { padding: 0.4rem; } .composer-toolbar { align-items: stretch; flex-direction: column; } .toolbar-right { justify-content: flex-end; } .attachment-grid { grid-template-columns: 1fr; } }
  @media (prefers-reduced-motion: reduce) { .chat-composer { scroll-behavior: auto; } }
</style>
