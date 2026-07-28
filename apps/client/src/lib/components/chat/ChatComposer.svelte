<script module lang="ts">
  type ActiveProviderCommand = {
    boundaryKey: string;
    entry: import("$lib/chat/contracts").ChatPromptCatalogEntry;
  };

  let restoreComposerFocus = false;
  let restoreSelectionStart = 0;
  let restoreSelectionEnd = 0;
  let restoreProviderCommand: ActiveProviderCommand | null = null;
</script>

<script lang="ts">
  import { onMount, tick } from "svelte";
  import Activity from "@lucide/svelte/icons/activity";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import AtSign from "@lucide/svelte/icons/at-sign";
  import Bold from "@lucide/svelte/icons/bold";
  import Box from "@lucide/svelte/icons/box";
  import CircleGauge from "@lucide/svelte/icons/circle-gauge";
  import CommandIcon from "@lucide/svelte/icons/command";
  import Hammer from "@lucide/svelte/icons/hammer";
  import ImagePlus from "@lucide/svelte/icons/image-plus";
  import Italic from "@lucide/svelte/icons/italic";
  import ListChecks from "@lucide/svelte/icons/list-checks";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import ScanSearch from "@lucide/svelte/icons/scan-search";
  import Server from "@lucide/svelte/icons/server";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import Sparkles from "@lucide/svelte/icons/sparkles";
  import Square from "@lucide/svelte/icons/square";
  import Target from "@lucide/svelte/icons/target";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import type { ChatPromptCatalogEntry, ChatThreadId, McpStatusRead, ProjectWorkingFolderId, ProjectWorkingFolderPathRead, ProviderCapabilities, ProviderInstanceId, ProviderSessionState, SafetyMode } from "$lib/chat/contracts";
  import { ChatComposerEditor, type ChatComposerEditorChange } from "$lib/chat/composer-editor";
  import { parseChatComposerDocument, type ChatComposerMark, type ChatComposerSelection } from "$lib/chat/composer-rich-text";
  import {
    composerActionState,
    composerRateLimitWindows,
    composerModeCommand,
    composerTokenTrigger,
    contextMeter,
    filterPromptCatalog,
    interactionModeForPrompt,
    shouldSendComposerKey,
    validateComposerSelections,
    validateImageFiles,
    validateModelOptions,
  } from "$lib/chat/composer-model";
  import { formatDateTime, formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import ChatAccessControl from "./ChatAccessControl.svelte";
  import ChatModelControls from "./ChatModelControls.svelte";
  import ChatRequestPanel from "./ChatRequestPanel.svelte";

  const { hero = false } = $props<{ hero?: boolean }>();
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const projects = getProjects();
  let editorRoot: HTMLDivElement | undefined = $state();
  let editorController: ChatComposerEditor | undefined;
  let editorFocused = false;
  let boldActive = $state(false);
  let italicActive = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();
  let attachmentMenu: HTMLDetailsElement | undefined = $state();
  let menuEntries = $state<(ProjectWorkingFolderPathRead | ChatPromptCatalogEntry)[]>([]);
  let menuRoot: HTMLDivElement | undefined = $state();
  let menuKind = $state<"mention" | "skill" | "command" | null>(null);
  let menuIndex = $state(0);
  let menuLoading = $state(false);
  let includeIgnored = $state(false);
  let menuCursor = $state<string | null>(null);
  let menuRequest = 0;
  let promptCatalogCache: { key: string; entries: ChatPromptCatalogEntry[] } | null = null;
  let promptCatalogPending: { key: string; promise: Promise<ChatPromptCatalogEntry[]> } | null = null;
  let operationError = $state<string | null>(null);
  let composerPanel = $state<"status" | "mcp" | null>(null);
  let composerPanelBoundary = $state<string | null>(null);
  let mcpStatus = $state<McpStatusRead | null>(null);
  let panelLoading = $state(false);
  let panelError = $state<string | null>(null);
  let activeProviderCommand = $state<ActiveProviderCommand | null>(null);
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
  const visibleComposerMode = $derived(activeProviderCommand
    ? { kind: "provider" as const, label: promptEntryLabel(activeProviderCommand.entry) }
    : chat.composer.interactionMode === "plan"
      ? { kind: "plan" as const, label: t("chat.composer.planCommand") }
      : null);
  const action = $derived(composerActionState(chat.interaction?.sessionState ?? "stopped", capabilities, hasDraft, pending !== null));
  const latestUsage = $derived(chat.interaction?.usage ?? chat.timelinePages.flatMap((page) => page.turns).at(-1)?.usage ?? null);
  const meter = $derived(contextMeter(latestUsage?.contextTokens ?? null, latestUsage?.contextLimit ?? null));
  const rateLimitWindows = $derived(composerRateLimitWindows(chat.interaction?.rateLimitStatus ?? null));
  const previewAttachment = $derived(chat.composerAttachments.find((attachment) => attachment.id === previewAttachmentId) ?? null);
  const projectArchived = $derived(projects.selectedProject?.status === "archived");
  const workingFolderUnavailable = $derived(
    chat.selectedWorkingFolder === null
      || chat.selectedWorkingFolder.workingFolder.archivedAt !== null
      || chat.selectedWorkingFolder.bindingStatus !== "available",
  );
  const sendingBlocked = $derived(projectArchived || workingFolderUnavailable);
  const composerDisabled = $derived(sendingBlocked || (chat.selectedThread?.archivedAt !== null && chat.selectedThread !== null));

  onMount(() => {
    const stop = () => void stopTurn();
    const continuePlan = () => preparePlanTurn("plan");
    const implementPlan = () => preparePlanTurn("build");
    window.addEventListener("ganbaru-ai:chat-stop-requested", stop);
    window.addEventListener("ganbaru-ai:chat-continue-plan", continuePlan);
    window.addEventListener("ganbaru-ai:chat-implement-plan", implementPlan);
    if (editorRoot) {
      editorController = new ChatComposerEditor(
        editorRoot,
        parseChatComposerDocument(chat.composer.richContent, chat.composer.text),
        {
          onChange: handleEditorChange,
          onSelectionChange: handleEditorSelectionChange,
        },
      );
    }
    const selectionChanged = () => {
      if (document.activeElement === editorRoot) editorController?.handleSelectionChange();
    };
    document.addEventListener("selectionchange", selectionChanged);
    if (restoreComposerFocus) {
      void tick().then(() => {
        editorController?.focus({ start: restoreSelectionStart, end: restoreSelectionEnd });
      });
    }
    const boundaryKey = providerCommandBoundaryKey();
    if (restoreProviderCommand?.boundaryKey === boundaryKey) {
      activeProviderCommand = restoreProviderCommand;
    }
    return () => {
      restoreComposerFocus = editorFocused;
      restoreProviderCommand = activeProviderCommand;
      document.removeEventListener("selectionchange", selectionChanged);
      editorController = undefined;
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
    const existing = editorController?.plainText().trim() ?? chat.composer.text.trim();
    if (editorController) editorController.appendPlainText(existing ? `\n\n${action}` : action);
    else chat.setComposerText(existing ? `${existing}\n\n${action}` : action);
    chat.setComposerModes(chat.composer.safetyMode, mode);
    queueMicrotask(() => editorController?.focus());
  }

  $effect(() => {
    const text = chat.composer.text;
    const richContent = chat.composer.richContent;
    editorController?.setDocument(parseChatComposerDocument(richContent, text));
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

  $effect(() => {
    const key = promptCatalogKey();
    if (promptCatalogCache && promptCatalogCache.key !== key) promptCatalogCache = null;
    const workingFolderId = chat.composer.workingFolderId;
    const providerInstanceId = chat.composer.providerInstanceId;
    const threadId = chat.selectedThread?.id ?? null;
    if (!workingFolderId || !providerInstanceId) return;
    void requestPromptCatalog(key, workingFolderId, providerInstanceId, threadId).catch(() => undefined);
  });

  $effect(() => {
    const boundaryKey = providerCommandBoundaryKey();
    if (activeProviderCommand && activeProviderCommand.boundaryKey !== boundaryKey) {
      activeProviderCommand = null;
      restoreProviderCommand = null;
    }
  });

  $effect(() => {
    const boundaryKey = providerCommandBoundaryKey();
    if (composerPanel && composerPanelBoundary !== boundaryKey) closeComposerPanel();
  });

  $effect(() => {
    if (!menuKind || !menuRoot) return;
    const selectedIndex = menuIndex;
    const entryCount = menuEntries.length;
    void tick().then(() => {
      if (!menuRoot || entryCount === 0) return;
      const selected = menuRoot.querySelector<HTMLElement>(`[data-menu-index="${selectedIndex}"]`);
      if (!selected) return;
      const top = selected.offsetTop;
      const bottom = top + selected.offsetHeight;
      if (top < menuRoot.scrollTop) menuRoot.scrollTop = top;
      else if (bottom > menuRoot.scrollTop + menuRoot.clientHeight) {
        menuRoot.scrollTop = Math.max(0, bottom - menuRoot.clientHeight);
      }
    });
  });

  function handleEditorChange(change: ChatComposerEditorChange): void {
    chat.setComposerRichContent(change.markdown, change.richContent);
    void updateMenu(change.plainText, editorController?.selection().start ?? change.plainText.length);
  }

  function handleEditorSelectionChange(selection: ChatComposerSelection, activeMarks: ChatComposerMark[]): void {
    restoreSelectionStart = selection.start;
    restoreSelectionEnd = selection.end;
    boldActive = activeMarks.includes("bold");
    italicActive = activeMarks.includes("italic");
  }

  function handleInput(): void {
    editorController?.handleInput();
  }

  function handleBeforeInput(event: InputEvent): void {
    editorController?.handleBeforeInput(event);
  }

  async function updateMenu(text: string, cursor: number): Promise<void> {
    const trigger = composerTokenTrigger(text, cursor);
    const workingFolderId = chat.composer.workingFolderId;
    const providerInstanceId = chat.composer.providerInstanceId;
    if (!trigger
      || (trigger.kind === "mention" && !workingFolderId)
      || (trigger.kind === "skill" && (!workingFolderId || !providerInstanceId))) {
      closeMenu();
      return;
    }
    const request = ++menuRequest;
    if (composerPanel) closeComposerPanel();
    menuKind = trigger.kind;
    menuLoading = trigger.kind === "mention";
    menuIndex = 0;
    menuEntries = [];
    try {
      if (trigger.kind === "mention") {
        if (!workingFolderId) return;
        const page = await chatApi.searchChatWorkingFolderPaths(workingFolderId, trigger.query, includeIgnored);
        if (request !== menuRequest) return;
        menuEntries = page.entries;
        menuCursor = page.nextCursor;
      } else {
        const key = promptCatalogKey();
        let providerCatalog = promptCatalogCache?.key === key ? promptCatalogCache.entries : [];
        menuEntries = filteredPromptCatalog(providerCatalog, trigger.kind, trigger.query);
        menuLoading = trigger.kind === "skill"
          && providerCatalog.length === 0
          && Boolean(workingFolderId && providerInstanceId);
        if (promptCatalogCache?.key !== key && workingFolderId && providerInstanceId) {
          providerCatalog = await requestPromptCatalog(
            key,
            workingFolderId,
            providerInstanceId,
            chat.selectedThread?.id ?? null,
          );
        }
        if (request !== menuRequest) return;
        menuEntries = filteredPromptCatalog(providerCatalog, trigger.kind, trigger.query);
        menuCursor = null;
      }
    } catch (cause: unknown) {
      if (request === menuRequest) operationError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (request === menuRequest) menuLoading = false;
    }
  }

  async function loadMoreMentions(): Promise<void> {
    if (!menuCursor || !chat.composer.workingFolderId || !editorController) return;
    const selection = editorController.selection();
    const trigger = composerTokenTrigger(editorController.plainText(), selection.start);
    if (!trigger) return;
    const page = await chatApi.searchChatWorkingFolderPaths(chat.composer.workingFolderId, trigger.query, includeIgnored, menuCursor);
    menuEntries = [...menuEntries, ...page.entries];
    menuCursor = page.nextCursor;
  }

  async function chooseMenuEntry(index: number): Promise<void> {
    const entry = menuEntries[index];
    if (!entry || !editorController) return;
    const trigger = composerTokenTrigger(editorController.plainText(), editorController.selection().start);
    if (!trigger) return;
    let cursor: number;
    if (menuKind === "mention" && "relativePath" in entry) {
      if (!chat.composer.mentions.some((mention) => mention.relativePath === entry.relativePath)) {
        chat.setComposerMentions([...chat.composer.mentions, { relativePath: entry.relativePath, kind: entry.kind, ignored: entry.ignored }]);
      }
      cursor = trigger.start;
      editorController.replaceRange(trigger.start, trigger.end, "");
    } else if ("value" in entry) {
      if (entry.kind === "command" && entry.source === "app") {
        runAppCommand(entry.value, trigger);
        return;
      }
      if (entry.kind === "command") {
        editorController.replaceRange(trigger.start, trigger.end, "");
        closeMenu();
        editorController.focus({ start: trigger.start, end: trigger.start });
        const directAction = providerDirectAction(entry);
        if (directAction) {
          await runProviderDirectAction(directAction);
        } else if (commandNeedsInput(entry)) {
          beginProviderCommand(entry);
        } else {
          await send(entry);
        }
        return;
      }
      cursor = trigger.start + entry.value.length + 1;
      editorController.replaceRange(trigger.start, trigger.end, `${entry.value} `);
    } else return;
    closeMenu();
    editorController.focus({ start: cursor, end: cursor });
  }

  function runAppCommand(value: string, trigger: import("$lib/chat/composer-model").ComposerTokenTrigger): void {
    if (!editorController) return;
    editorController.replaceRange(trigger.start, trigger.end, "");
    closeMenu();
    operationError = null;
    switch (value) {
      case "/plan":
        clearProviderCommand();
        chat.setComposerModes(chat.composer.safetyMode, "plan");
        break;
      case "/build":
        clearProviderCommand();
        chat.setComposerModes(chat.composer.safetyMode, "build");
        break;
      case "/model":
        queueMicrotask(() => document.querySelector<HTMLButtonElement>("[data-chat-model-trigger]")?.click());
        break;
      case "/permissions":
        queueMicrotask(() => document.querySelector<HTMLButtonElement>('[data-chat-field="safety"]')?.click());
        break;
      case "/status":
        openComposerPanel("status");
        break;
      case "/changes":
        window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-open-changes"));
        break;
    }
    queueMicrotask(() => editorController?.focus());
  }

  function providerDirectAction(entry: ChatPromptCatalogEntry): "compact" | "mcp" | null {
    if (entry.source !== "provider") return null;
    const family = provider?.configuration.familyId;
    if (entry.value.toLowerCase() === "/compact" && (family === "codex" || family === "claude")) {
      return "compact";
    }
    if (entry.value.toLowerCase() === "/mcp" && family === "codex") return "mcp";
    return null;
  }

  async function runProviderDirectAction(action: "compact" | "mcp"): Promise<void> {
    const threadId = chat.selectedThread?.id;
    if (action === "mcp") {
      const workingFolderId = chat.composer.workingFolderId;
      const providerInstanceId = chat.composer.providerInstanceId;
      if (!workingFolderId) {
        operationError = t("chat.composer.workingFolderUnavailable");
        return;
      }
      if (!providerInstanceId) {
        operationError = t("chat.composer.chooseProvider");
        return;
      }
      clearProviderCommand();
      await openMcpPanel(workingFolderId, providerInstanceId, threadId ?? null);
      return;
    }
    if (!threadId) {
      operationError = t("chat.composer.commandNeedsActiveTask");
      return;
    }
    clearProviderCommand();
    const completed = await run(async () => {
      await chatApi.compactChatContext(threadId);
      await chat.refreshInteraction(threadId);
    });
    if (completed) queueMicrotask(() => editorController?.focus());
  }

  function openComposerPanel(panel: "status" | "mcp"): void {
    closeMenu();
    composerPanel = panel;
    composerPanelBoundary = providerCommandBoundaryKey();
    panelError = null;
  }

  async function openMcpPanel(
    workingFolderId: ProjectWorkingFolderId,
    providerInstanceId: ProviderInstanceId,
    threadId: ChatThreadId | null,
  ): Promise<void> {
    openComposerPanel("mcp");
    panelLoading = true;
    mcpStatus = null;
    try {
      mcpStatus = await chatApi.readChatMcpStatus(workingFolderId, providerInstanceId, threadId);
    } catch (cause: unknown) {
      panelError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      panelLoading = false;
    }
  }

  function closeComposerPanel(): void {
    composerPanel = null;
    composerPanelBoundary = null;
    panelLoading = false;
    panelError = null;
    queueMicrotask(() => editorController?.focus());
  }

  function insertMentionTrigger(): void {
    attachmentMenu?.removeAttribute("open");
    if (!editorController) return;
    const selection = editorController.selection();
    editorController.replaceRange(selection.start, selection.end, "@");
    const cursor = selection.start + 1;
    editorController.focus({ start: cursor, end: cursor });
    void updateMenu(editorController.plainText(), cursor);
  }

  function closeMenu(): void {
    menuRequest += 1;
    menuKind = null;
    menuEntries = [];
    menuCursor = null;
    menuLoading = false;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (editorController?.isComposing || event.isComposing || event.keyCode === 229) return;
    if (menuKind) {
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        const direction = event.key === "ArrowDown" ? 1 : -1;
        menuIndex = (menuIndex + direction + Math.max(1, menuEntries.length)) % Math.max(1, menuEntries.length);
        return;
      }
      if (event.key === "Enter" && menuEntries.length > 0) {
        event.preventDefault();
        void chooseMenuEntry(menuIndex);
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        closeMenu();
        return;
      }
    }
    if (editorController?.handleKeydown(event)) return;
    const sendKey = chat.settings?.configuration.behavior.sendKey ?? "enter";
    if (shouldSendComposerKey(event, sendKey)) {
      event.preventDefault();
      void performComposerAction();
    }
  }

  async function performComposerAction(): Promise<void> {
    if (projectArchived) {
      operationError = t("chat.composer.archivedProject");
      return;
    }
    if (workingFolderUnavailable) {
      operationError = t("chat.composer.workingFolderUnavailable");
      return;
    }
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

  async function send(selectedCommand: ChatPromptCatalogEntry | null = activeProviderCommand?.entry ?? null): Promise<void> {
    if (projectArchived) {
      operationError = t("chat.composer.archivedProject");
      return;
    }
    if (workingFolderUnavailable) {
      operationError = t("chat.composer.workingFolderUnavailable");
      return;
    }
    if (!selectedCommand && (executeTypedAppCommand() || executeTypedProviderDirectAction())) return;
    if (selectedCommand && (chat.composer.attachmentIds.length > 0 || chat.composer.mentions.length > 0)) {
      operationError = t("chat.composer.commandContextUnsupported");
      editorRoot?.focus();
      return;
    }
    const workingFolderId = chat.composer.workingFolderId;
    const providerId = chat.composer.providerInstanceId;
    const trusted = workingFolderId && providerId ? await chatApi.hasChatFullAccessTrust(providerId, workingFolderId) : false;
    const model = chat.composer.modelSelection?.value;
    const modelId = typeof model === "object" && model !== null && !Array.isArray(model) && typeof model.modelId === "string" ? model.modelId : null;
    const providerManagedModel = typeof model === "object" && model !== null && !Array.isArray(model) && model.providerManaged === true;
    const modeCommand = selectedCommand ? null : composerModeCommand(chat.composer.text);
    if (modeCommand && editorController) {
      const prefixLength = chat.composer.text.length - modeCommand.prompt.length;
      editorController.replaceRange(0, prefixLength, "");
    }
    const interactionMode = selectedCommand
      ? chat.composer.interactionMode
      : modeCommand?.mode ?? interactionModeForPrompt(chat.composer.text, chat.composer.interactionMode);
    if (interactionMode !== chat.composer.interactionMode) {
      chat.setComposerModes(chat.composer.safetyMode, interactionMode);
    }
    const errors = validateComposerSelections({
      workingFolderId,
      providerInstanceId: providerId,
      modelId,
      providerManagedModel,
      safetyMode: chat.composer.safetyMode,
      interactionMode,
      fullAccessTrusted: !matchesBroadPermissionMode(chat.composer.safetyMode) || trusted,
    }, capabilities);
    const modelOptionErrors = provider && modelId
      ? validateModelOptions(provider.modelCatalog?.models.find((entry) => entry.id === modelId)?.options ?? [], readComposerOptions())
      : [];
    const commandPrompt = selectedCommand
      ? `${selectedCommand.value}${chat.composer.text.trim() ? ` ${chat.composer.text.trim()}` : ""}`
      : null;
    if (errors.length > 0 || modelOptionErrors.length > 0 || (!selectedCommand && !hasDraft) || (selectedCommand && commandNeedsInput(selectedCommand) && !chat.composer.text.trim())) {
      const field = errors[0]?.field;
      operationError = field ? selectionError(field) : modelOptionErrors.length > 0
        ? t("chat.composer.invalidTrait")
        : t("chat.composer.messageRequired");
      const targetField = field === "trust" ? "safety" : field;
      const target = targetField === "provider" || targetField === "model" || targetField === "interaction"
        ? document.querySelector<HTMLElement>("[data-chat-model-trigger]")
        : targetField
          ? document.querySelector<HTMLElement>(`[data-chat-field="${targetField}"]`)
          : editorRoot;
      target?.focus();
      return;
    }
    const sent = await run(() => chat.sendComposer(commandPrompt
      ? { promptOverride: commandPrompt, omitComposerContext: true }
      : undefined));
    if (sent && !chat.sendError && activeProviderCommand?.entry.value === selectedCommand?.value) {
      clearProviderCommand();
    }
  }

  async function retryFailedComposerSend(): Promise<void> {
    const retried = await run(() => chat.retryFailedSend());
    if (retried && !chat.sendError) clearProviderCommand();
  }

  function executeTypedAppCommand(): boolean {
    const value = chat.composer.text.trim().toLowerCase();
    if (!["/plan", "/build", "/model", "/permissions", "/status", "/changes"].includes(value)) {
      return false;
    }
    const trigger = { kind: "command" as const, query: value.slice(1), start: 0, end: chat.composer.text.length };
    runAppCommand(value, trigger);
    return true;
  }

  function executeTypedProviderDirectAction(): boolean {
    const family = provider?.configuration.familyId;
    const value = chat.composer.text.trim().toLowerCase();
    const action = value === "/compact" && (family === "codex" || family === "claude")
      ? "compact"
      : value === "/mcp" && family === "codex"
        ? "mcp"
        : null;
    if (!action) return false;
    const trigger = { kind: "command" as const, query: value.slice(1), start: 0, end: chat.composer.text.length };
    editorController?.replaceRange(trigger.start, trigger.end, "");
    closeMenu();
    void runProviderDirectAction(action);
    return true;
  }

  function promptCatalogKey(): string {
    return [
      chat.composer.workingFolderId ?? "",
      chat.composer.providerInstanceId ?? "",
      chat.selectedThread?.id ?? "",
      chat.interaction?.sessionState ?? "stopped",
    ].join(":");
  }

  function requestPromptCatalog(
    key: string,
    workingFolderId: ProjectWorkingFolderId,
    providerInstanceId: ProviderInstanceId,
    threadId: ChatThreadId | null,
  ): Promise<ChatPromptCatalogEntry[]> {
    if (promptCatalogCache?.key === key) return Promise.resolve(promptCatalogCache.entries);
    if (promptCatalogPending?.key === key) return promptCatalogPending.promise;
    const promise = chatApi.listChatPromptCatalog(workingFolderId, providerInstanceId, threadId)
      .then((entries) => {
        if (promptCatalogKey() === key) promptCatalogCache = { key, entries };
        return entries;
      })
      .finally(() => {
        if (promptCatalogPending?.promise === promise) promptCatalogPending = null;
      });
    promptCatalogPending = { key, promise };
    return promise;
  }

  function filteredPromptCatalog(
    providerCatalog: ChatPromptCatalogEntry[],
    kind: "skill" | "command",
    query: string,
  ): ChatPromptCatalogEntry[] {
    const appCatalog = appCommandCatalog();
    const appValues = new Set(appCatalog.map((entry) => entry.value.toLowerCase()));
    return filterPromptCatalog([
      ...appCatalog,
      ...providerCatalog.filter((entry) => !appValues.has(entry.value.toLowerCase())),
    ], kind, query);
  }

  function appCommandCatalog(): ChatPromptCatalogEntry[] {
    return [
      appCommand("/plan", t("chat.composer.planCommand"), t("chat.composer.planCommandDescription")),
      appCommand("/build", t("chat.composer.buildCommand"), t("chat.composer.buildCommandDescription")),
      appCommand("/model", t("chat.composer.modelCommand"), t("chat.composer.modelCommandDescription")),
      appCommand("/permissions", t("chat.composer.permissionsCommand"), t("chat.composer.permissionsCommandDescription")),
      appCommand("/status", t("chat.composer.statusCommand"), t("chat.composer.statusCommandDescription")),
      appCommand("/changes", t("chat.composer.changesCommand"), t("chat.composer.changesCommandDescription")),
    ];
  }

  function appCommand(value: string, label: string, description: string): ChatPromptCatalogEntry {
    return {
      value,
      label,
      description,
      argumentHint: null,
      kind: "command",
      source: "app",
      stale: false,
    };
  }

  function beginProviderCommand(entry: ChatPromptCatalogEntry): void {
    activeProviderCommand = { boundaryKey: providerCommandBoundaryKey(), entry };
    restoreProviderCommand = activeProviderCommand;
    operationError = null;
    if (chat.composer.interactionMode !== "build") {
      chat.setComposerModes(chat.composer.safetyMode, "build");
    }
    queueMicrotask(() => editorController?.focus());
  }

  function clearProviderCommand(): void {
    activeProviderCommand = null;
    restoreProviderCommand = null;
  }

  function clearVisibleComposerMode(): void {
    if (activeProviderCommand) clearProviderCommand();
    else if (chat.composer.interactionMode === "plan") {
      chat.setComposerModes(chat.composer.safetyMode, "build");
    }
    queueMicrotask(() => editorController?.focus());
  }

  function commandNeedsInput(entry: ChatPromptCatalogEntry): boolean {
    const command = entry.value.toLowerCase();
    const family = provider?.configuration.familyId;
    if (entry.source === "provider" && family === "codex" && ["/compact", "/review", "/mcp"].includes(command)) {
      return false;
    }
    if (entry.source === "provider" && family === "claude" && ["/compact", "/clear", "/context", "/usage"].includes(command)) {
      return false;
    }
    return (entry.source === "provider" && family === "codex" && command === "/goal")
      || entry.argumentHint !== null;
  }

  function providerCommandBoundaryKey(): string {
    return [
      chat.composer.workingFolderId ?? "",
      chat.composer.providerInstanceId ?? "",
      chat.composer.threadId ?? "",
    ].join(":");
  }

  function providerCommandPlaceholder(): string {
    if (!activeProviderCommand) return t("chat.composer.placeholder");
    return activeProviderCommand.entry.value.toLowerCase() === "/goal"
      ? t("chat.composer.goalInputPlaceholder")
      : t("chat.composer.commandInputPlaceholder", promptEntryLabel(activeProviderCommand.entry));
  }

  function promptEntryLabel(entry: ChatPromptCatalogEntry): string {
    if (entry.kind !== "command") return entry.label;
    const commandName = entry.value.replace(/^\//, "");
    if (entry.label.toLowerCase() !== commandName.toLowerCase()) return entry.label;
    return `${entry.label.charAt(0).toUpperCase()}${entry.label.slice(1)}`;
  }

  function promptEntryIcon(entry: ChatPromptCatalogEntry): typeof Activity {
    if (entry.kind === "skill") return Sparkles;
    switch (entry.value.toLowerCase()) {
      case "/review":
      case "/changes":
        return ScanSearch;
      case "/compact":
      case "/context":
        return CircleGauge;
      case "/goal":
        return Target;
      case "/mcp":
        return Server;
      case "/plan":
        return ListChecks;
      case "/build":
        return Hammer;
      case "/model":
        return Box;
      case "/permissions":
        return ShieldCheck;
      case "/status":
      case "/usage":
        return Activity;
      case "/clear":
        return RotateCcw;
      default:
        return CommandIcon;
    }
  }

  function selectedModelLabel(): string {
    return provider?.modelCatalog?.models.find((entry) => entry.id === readSelectedModelId())?.displayName
      ?? t("chat.composer.statusProviderManaged");
  }

  function accountStatusLabel(): string {
    const account = chat.interaction?.accountStatus;
    if (account?.accountLabel && account.planLabel) return `${account.accountLabel} · ${account.planLabel}`;
    return account?.accountLabel ?? account?.planLabel ?? t("chat.composer.statusAccountUnavailable");
  }

  function contextRemainingLabel(): string {
    if (!meter || meter.maximumTokens === null || meter.ratio === null) {
      return contextLabel() ?? t("chat.composer.contextUnavailable");
    }
    return t(
      "chat.composer.contextRemaining",
      contextPercentage(1 - meter.ratio),
      formatNumber(localization.locale, meter.usedTokens),
      formatNumber(localization.locale, meter.maximumTokens),
    );
  }

  function resetTimeLabel(seconds: number | null): string | null {
    if (seconds === null) return null;
    return formatDateTime(localization.locale, seconds * 1_000, {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }

  function readableProviderStatus(value: string | null): string {
    if (!value) return t("chat.composer.mcpConfigured");
    const words = value.replaceAll(/[_-]+/g, " ").trim();
    return words ? `${words.charAt(0).toUpperCase()}${words.slice(1)}` : t("chat.composer.mcpConfigured");
  }

  function providerSessionLabel(state: ProviderSessionState): string {
    switch (state) {
      case "stopped": return t("chat.composer.statusSessionStopped");
      case "starting": return t("chat.composer.statusSessionStarting");
      case "ready": return t("chat.composer.statusSessionReady");
      case "active": return t("chat.composer.statusSessionActive");
      case "waiting_for_approval": return t("chat.composer.statusSessionApproval");
      case "waiting_for_user_input": return t("chat.composer.statusSessionInput");
      case "stopping": return t("chat.composer.statusSessionStopping");
      case "failed": return t("chat.composer.statusSessionFailed");
    }
  }

  function readSelectedModelId(): string | null {
    const value = chat.composer.modelSelection?.value;
    return typeof value === "object" && value !== null && !Array.isArray(value) && typeof value.modelId === "string"
      ? value.modelId
      : null;
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

  async function run(operation: () => Promise<void>): Promise<boolean> {
    sending = true;
    operationError = null;
    try {
      await operation();
      return true;
    } catch (cause: unknown) {
      operationError = cause instanceof Error ? cause.message : String(cause);
      return false;
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
    if (files.length > 0) { event.preventDefault(); void importFiles(files); return; }
    const text = event.clipboardData?.getData("text/plain");
    if (text === undefined) return;
    event.preventDefault();
    editorController?.insertPlainText(text);
  }

  function handleDrop(event: DragEvent): void {
    event.preventDefault();
    const files = [...(event.dataTransfer?.files ?? [])].filter((file) => file.type.startsWith("image/"));
    if (files.length > 0) { void importFiles(files); return; }
    const text = event.dataTransfer?.getData("text/plain");
    if (text) editorController?.insertPlainText(text);
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

  function contextPercentage(value: number): string {
    return formatNumber(localization.locale, value * 100, { maximumFractionDigits: 0 });
  }

  function matchesBroadPermissionMode(mode: SafetyMode | null): boolean {
    return mode === "full_access" || mode === "custom";
  }

</script>

<section class:hero class="chat-composer" data-chat-composer-container role="group" ondragover={(event) => event.preventDefault()} ondrop={handleDrop}>
  {#if chat.interaction?.queuedFollowup}<div class="queued-row"><div><strong>{t("chat.composer.queued")}</strong><p>{chat.interaction.queuedFollowup.text}</p></div><button type="button" onclick={() => void chat.editQueuedFollowup()}>{t("chat.composer.editQueued")}</button><button type="button" onclick={() => void chat.cancelQueuedFollowup()}>{t("chat.composer.cancelQueued")}</button></div>{/if}
  {#if chat.sendError}<div role="alert" class="recovery-row"><strong>{t("chat.composer.launchFailed")}</strong><span>{chat.sendError}</span><button type="button" onclick={() => void retryFailedComposerSend()}>{t("chat.timeline.retry")}</button><button type="button" onclick={() => void chat.editFailedSend()}>{t("chat.composer.editDraft")}</button><button type="button" onclick={() => void chat.changeProviderAfterFailure()}>{t("chat.composer.changeProvider")}</button></div>{/if}
  {#if projectArchived}<div role="status" class="recovery-row"><strong>{t("chat.firstUse.archivedProjectTitle")}</strong><span>{t("chat.composer.archivedProject")}</span></div>{/if}
  {#if workingFolderUnavailable}<div role="status" class="recovery-row"><strong>{t("chat.firstUse.missingBindingTitle")}</strong><span>{t("chat.composer.workingFolderUnavailable")}</span></div>{/if}
  {#if composerPanel}
    <div class="composer-info-panel" role="region" aria-label={composerPanel === "mcp" ? t("chat.composer.mcpPanelTitle") : t("chat.composer.statusPanelTitle")}>
      <header>
        <strong>{composerPanel === "mcp" ? t("chat.composer.mcpPanelTitle") : t("chat.composer.statusPanelTitle")}</strong>
        <button type="button" onclick={closeComposerPanel}>{t("chat.composer.closePanel")}</button>
      </header>
      {#if composerPanel === "status"}
        <dl class="status-panel-grid">
          <div><dt>{t("chat.composer.statusProvider")}</dt><dd>{provider?.configuration.label ?? t("chat.composer.statusUnknown")}</dd></div>
          <div><dt>{t("chat.composer.statusSession")}</dt><dd><code title={chat.interaction?.sessionId ?? undefined}>{chat.interaction?.sessionId ?? t("chat.composer.statusNoSession")}</code><span class="status-state">{providerSessionLabel(chat.interaction?.sessionState ?? "stopped")}</span></dd></div>
          <div><dt>{t("chat.composer.statusModel")}</dt><dd>{selectedModelLabel()}</dd></div>
          <div><dt>{t("chat.composer.statusAccount")}</dt><dd>{accountStatusLabel()}</dd></div>
          <div class="status-meter-row">
            <dt>{t("chat.composer.statusContext")}</dt>
            <dd>
              <span>{contextRemainingLabel()}</span>
              {#if meter && meter.ratio !== null}<span class="panel-meter" style={`--panel-progress:${Math.max(0, 1 - meter.ratio)}`}><span></span></span>{/if}
            </dd>
          </div>
          {#each rateLimitWindows as window (window.id)}
            <div class="status-meter-row">
              <dt>{window.label}</dt>
              <dd>
                <span>{t("chat.composer.rateLimitRemaining", formatNumber(localization.locale, 100 - window.usedPercent, { maximumFractionDigits: 0 }))}{#if resetTimeLabel(window.resetsAtSeconds)} <small>{t("chat.composer.rateLimitResets", resetTimeLabel(window.resetsAtSeconds) ?? "")}</small>{/if}</span>
                <span class="panel-meter" style={`--panel-progress:${Math.max(0, 1 - window.usedPercent / 100)}`}><span></span></span>
              </dd>
            </div>
          {/each}
        </dl>
      {:else if panelLoading}
        <p class="panel-state"><LoaderCircle size={14} class="animate-spin" />{t("common.loading")}</p>
      {:else if panelError}
        <p class="panel-error" role="alert">{panelError}</p>
      {:else if !mcpStatus || mcpStatus.servers.length === 0}
        <p class="panel-state">{t("chat.composer.mcpEmpty")}</p>
      {:else}
        <div class="mcp-status-table" aria-label={t("chat.composer.mcpPanelTitle")}>
          {#each mcpStatus.servers as server (server.name)}
            <div>
              <strong title={server.name}>{server.name}</strong>
              <span>{t("chat.composer.mcpAuthStatus", readableProviderStatus(server.authStatus))}</span>
              <span class:enabled={server.enabled}>{server.enabled ? t("chat.composer.enabled") : t("chat.composer.disabled")}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
  {#if chat.composerAttachments.length > 0}<div class="attachment-grid">{#each chat.composerAttachments as attachment}<article><button type="button" class="attachment-preview" aria-label={t("chat.composer.previewAttachment", attachment.originalDisplayName)} onclick={() => void openPreview(attachment.id)}>{#if thumbnailUrls[attachment.id]}<img src={thumbnailUrls[attachment.id]} alt={attachment.originalDisplayName} />{:else}<LoaderCircle size={16} class="animate-spin" />{/if}</button><span title={attachment.originalDisplayName}>{attachment.originalDisplayName}</span><button type="button" aria-label={t("chat.composer.removeAttachment", attachment.originalDisplayName)} onclick={() => chat.removeComposerAttachment(attachment.id)}><X size={12} /></button></article>{/each}</div>{/if}
  {#if chat.composer.mentions.length > 0}<div class="mention-chips">{#each chat.composer.mentions as mention}<span title={mention.relativePath}><AtSign size={11} />{mention.relativePath}{#if mention.ignored}<small>{t("chat.composer.ignored")}</small>{/if}<button type="button" aria-label={t("chat.composer.removeAttachment", mention.relativePath)} onclick={() => chat.setComposerMentions(chat.composer.mentions.filter((entry) => entry.relativePath !== mention.relativePath))}><X size={10} /></button></span>{/each}</div>{/if}
  <div class="editor-shell">
    <div
      bind:this={editorRoot}
      class="composer-editor"
      data-chat-composer
      contenteditable={composerDisabled ? "false" : "true"}
      role="textbox"
      aria-multiline="true"
      aria-disabled={composerDisabled}
      aria-label={providerCommandPlaceholder()}
      aria-controls={menuKind ? "chat-composer-menu" : undefined}
      aria-activedescendant={menuKind && menuEntries.length > 0 ? `chat-composer-option-${menuIndex}` : undefined}
      data-placeholder={action.primary === "stop"
        ? t("chat.composer.placeholderWorking")
        : activeProviderCommand
          ? providerCommandPlaceholder()
          : t("chat.composer.placeholder")}
      spellcheck="true"
      tabindex="0"
      onfocus={() => { editorFocused = true; restoreComposerFocus = true; editorController?.handleSelectionChange(); }}
      onblur={(event) => { const target = event.currentTarget; queueMicrotask(() => { if (target.isConnected) { editorFocused = false; restoreComposerFocus = false; } }); }}
      oninput={handleInput}
      onbeforeinput={handleBeforeInput}
      onkeydown={handleKeydown}
      onpaste={handlePaste}
      oncompositionstart={() => editorController?.handleCompositionStart()}
      oncompositionend={() => editorController?.handleCompositionEnd()}
    ></div>
    {#if menuKind}<div bind:this={menuRoot} id="chat-composer-menu" class="composer-menu" role="listbox" aria-label={menuKind === "mention" ? t("chat.composer.mentionFiles") : menuKind === "skill" ? t("chat.composer.skillsMenu") : t("chat.composer.commandsMenu")}>
      {#if menuKind === "mention"}<label><input type="checkbox" bind:checked={includeIgnored} onchange={() => { if (editorController) void updateMenu(editorController.plainText(), editorController.selection().start); }} />{t("chat.composer.showIgnored")}</label>{/if}
      {#if menuLoading}<p><LoaderCircle size={13} class="animate-spin" />{t("common.loading")}</p>
      {:else if menuEntries.length === 0}<p>{t("chat.composer.noMatches")}</p>
      {:else if menuKind === "mention"}
        {#each menuEntries as entry, index}
          {#if "relativePath" in entry}<button id={`chat-composer-option-${index}`} data-menu-index={index} type="button" class:selected={index === menuIndex} role="option" aria-selected={index === menuIndex} onpointerdown={(event) => event.preventDefault()} onclick={() => void chooseMenuEntry(index)}><strong>{entry.displayName}</strong><small>{entry.relativePath}{#if entry.ignored} · {t("chat.composer.ignored")}{/if}</small></button>{/if}
        {/each}
        {#if menuCursor}<button type="button" onclick={() => void loadMoreMentions()}>{t("chat.composer.loadMore")}</button>{/if}
      {:else}
        {#each menuEntries as entry, index}
          {#if "value" in entry}
            {@const EntryIcon = promptEntryIcon(entry)}
            <button id={`chat-composer-option-${index}`} data-menu-index={index} type="button" class="prompt-option" class:selected={index === menuIndex} role="option" aria-selected={index === menuIndex} onpointerdown={(event) => event.preventDefault()} onclick={() => void chooseMenuEntry(index)}>
              <EntryIcon size={15} strokeWidth={1.8} />
              <span class="prompt-option-copy">
                <strong>{promptEntryLabel(entry)}</strong>
                {#if entry.description}<small>{entry.description}</small>{/if}
                {#if entry.stale}<small class="stale-command">{t("chat.composer.staleEntry")}</small>{/if}
              </span>
            </button>
          {/if}
        {/each}
      {/if}
    </div>{/if}
    <div class="composer-toolbar">
      <div class="toolbar-left">
        <input bind:this={fileInput} class="sr-only" type="file" accept="image/png,image/jpeg,image/gif,image/webp" multiple onchange={(event) => void importFiles([...(event.currentTarget.files ?? [])])} />
        <details bind:this={attachmentMenu} class="attachment-menu">
          <summary aria-label={t("chat.composer.attachImages")}><Plus size={15} /></summary>
          <div>
            <button type="button" title={t("chat.composer.imageLimit")} disabled={importing} onclick={() => { attachmentMenu?.removeAttribute("open"); fileInput?.click(); }}>{#if importing}<LoaderCircle size={14} class="animate-spin" />{:else}<Paperclip size={14} />{/if}{t("chat.composer.attachImages")}</button>
            <button type="button" onclick={() => { attachmentMenu?.removeAttribute("open"); void run(() => chat.pickComposerImages(t("chat.composer.imagePickerTitle"))); }}><ImagePlus size={14} />{t("chat.composer.imagePickerTitle")}</button>
            <button type="button" onclick={insertMentionTrigger}><AtSign size={14} />{t("chat.composer.mentionFiles")}</button>
          </div>
        </details>
        <ChatAccessControl />
        {#if visibleComposerMode}
          <span class="mode-divider" aria-hidden="true"></span>
          <button type="button" class="composer-mode" aria-label={t("chat.composer.exitCommandMode", visibleComposerMode.label)} onclick={clearVisibleComposerMode}>
            {#if visibleComposerMode.kind === "provider"}<Target size={14} />{:else}<ListChecks size={14} />{/if}
            <span>{visibleComposerMode.label}</span>
          </button>
        {/if}
        <button type="button" class="format-action" class:active={boldActive} aria-label={t("chat.composer.bold")} aria-pressed={boldActive} title={`${t("chat.composer.bold")} (Ctrl+B)`} disabled={composerDisabled} onpointerdown={(event) => event.preventDefault()} onclick={() => editorController?.toggleMark("bold")}><Bold size={14} /></button>
        <button type="button" class="format-action" class:active={italicActive} aria-label={t("chat.composer.italic")} aria-pressed={italicActive} title={`${t("chat.composer.italic")} (Ctrl+I)`} disabled={composerDisabled} onpointerdown={(event) => event.preventDefault()} onclick={() => editorController?.toggleMark("italic")}><Italic size={14} /></button>
      </div>
      <div class="toolbar-right">
        {#if action.followup === "retain"}<small>{t("chat.composer.retained")}</small>{/if}
        {#if action.followup === "steer"}<button type="button" class="round-action" title={t("chat.composer.steer")} aria-label={t("chat.composer.steer")} onclick={() => void run(() => chat.steerComposer())}><ArrowUp size={14} /></button>{:else if action.followup === "queue"}<button type="button" class="round-action" title={t("chat.composer.queue")} aria-label={t("chat.composer.queue")} onclick={() => void run(() => chat.queueComposer())}><ArrowUp size={14} /></button>{/if}
        <span class="context-ring" class:warning={meter?.warning} style={`--context-progress:${Math.max(0, Math.min(1, meter?.ratio ?? 0))}`}>
          <svg viewBox="0 0 20 20" role="img" aria-label={contextLabel() ?? t("chat.composer.contextUnavailable")}><circle class="context-track" cx="10" cy="10" r="7"></circle><circle class="context-fill" cx="10" cy="10" r="7"></circle></svg>
          <span class="context-tooltip" role="tooltip"><strong>{t("chat.composer.contextWindow")}</strong>{#if meter?.ratio !== null}<span>{t("chat.composer.contextPercentage", contextPercentage(meter?.ratio ?? 0), contextPercentage(1 - (meter?.ratio ?? 0)))}</span>{/if}<span>{contextLabel() ?? t("chat.composer.contextUnavailable")}</span>{#if chat.interaction?.automaticCompactionReported || meter?.warning}<small>{t("chat.composer.contextCompaction")}</small>{/if}</span>
        </span>
        <ChatModelControls />
        {#if forceStopAvailable}<button type="button" class="force-stop" title={t("chat.composer.forceStopDescription")} onclick={() => void run(() => chat.stop(true))}>{t("chat.composer.forceStop")}</button>{/if}
        {#if action.primary !== "resolve_request"}<button type="button" class="primary-action" disabled={sendingBlocked || sending || action.primary === "stopping" || (action.primary === "send" && !action.sendEnabled)} aria-label={action.primary === "stop" ? t("chat.composer.stop") : t("chat.composer.send")} title={action.primary === "stop" ? t("chat.composer.stop") : t("chat.composer.send")} onclick={() => void performPrimaryAction()}>{#if sending}<LoaderCircle size={15} class="animate-spin" />{:else if action.primary === "stop"}<Square size={13} />{:else if action.primary === "stopping"}<LoaderCircle size={15} class="animate-spin" />{:else}<ArrowUp size={16} />{/if}</button>{/if}
      </div>
    </div>
  </div>
  {#if pending}<div class="request-panel-shell"><ChatRequestPanel {pending} /></div>{/if}
  {#if operationError || chat.composer.error}<p role="alert" class="composer-error">{operationError ?? chat.composer.error}</p>{/if}
</section>

{#if previewAttachment && previewUrl}<div class="fixed inset-0 z-60 grid place-items-center bg-black/50 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closePreview}></button><div bind:this={previewDialog} class="relative flex max-h-full max-w-full flex-col rounded-lg border border-border bg-background p-3 shadow-2xl" role="dialog" aria-modal="true" aria-label={t("chat.composer.previewAttachment", previewAttachment.originalDisplayName)} tabindex="-1" onkeydown={handlePreviewKeydown}><header class="mb-2 flex items-center gap-2"><strong class="min-w-0 flex-1 truncate text-sm">{previewAttachment.originalDisplayName}</strong><span class="text-xs text-muted-foreground">{formatNumber(localization.locale, previewAttachment.byteSize)} B</span><button type="button" aria-label={t("chat.cancel")} onclick={closePreview}><X size={14} /></button></header><img class="min-h-0 max-h-[75vh] max-w-[85vw] object-contain" src={previewUrl} alt={previewAttachment.originalDisplayName} /></div></div>{/if}

<style>
  .chat-composer { container-type: inline-size; container-name: chat-composer; position: relative; display: grid; width: min(100%, 54rem); margin: 0 auto; overflow: visible; border: 1px solid color-mix(in srgb, var(--border) 88%, transparent); border-radius: 1.3rem; background: var(--card); box-shadow: 0 8px 22px -18px rgb(0 0 0 / 0.24), 0 1px 4px -3px rgb(0 0 0 / 0.16); }
  .chat-composer.hero { width: min(100%, 54rem); text-align: left; }
  .chat-composer > :not(.editor-shell) { margin-inline: 0.75rem; }
  .editor-shell { position: relative; }
  .composer-editor { position: relative; display: block; width: 100%; min-height: 4.15rem; max-height: 15.35rem; overflow-y: auto; background: transparent; padding: 1rem 1.25rem 0.35rem; color: var(--foreground); caret-color: var(--foreground); font-size: var(--chat-conversation-font-size, 0.933333rem); line-height: var(--chat-conversation-line-height, 1.4rem); outline: none; overflow-wrap: anywhere; white-space: pre-wrap; }
  .composer-editor:global([data-empty="true"])::before { position: absolute; color: color-mix(in srgb, var(--muted-foreground) 52%, transparent); content: attr(data-placeholder); pointer-events: none; }
  .composer-editor :global([data-chat-composer-line]) { display: block; min-height: var(--chat-conversation-line-height, 1.4rem); line-height: inherit; }
  .composer-toolbar { display: grid; min-height: 3rem; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 0.5rem; padding: 0.3rem 0.75rem 0.65rem; }
  .toolbar-left, .toolbar-right { display: flex; min-width: 0; align-items: center; gap: 0.3rem; }
  .toolbar-left { overflow: hidden; }
  .toolbar-left:has(.attachment-menu[open]) { overflow: visible; }
  .toolbar-right { flex: 0 1 auto; justify-content: flex-end; }
  .attachment-menu { position: relative; flex: 0 0 auto; }
  .attachment-menu summary, .round-action, .format-action { display: inline-flex; width: 1.9rem; height: 1.9rem; cursor: pointer; list-style: none; align-items: center; justify-content: center; border-radius: 0.5rem; color: var(--muted-foreground); }
  .attachment-menu summary::-webkit-details-marker { display: none; }
  .attachment-menu summary:hover, .round-action:hover, .format-action:hover, .format-action.active { background: var(--accent); color: var(--foreground); }
  .format-action:disabled { cursor: default; opacity: 0.45; }
  .mode-divider { width: 1px; height: 1.25rem; flex: 0 0 auto; background: color-mix(in srgb, var(--border) 78%, transparent); margin-inline: 0.1rem; }
  .composer-mode { display: inline-flex; min-width: 0; height: 1.9rem; flex: 0 1 auto; align-items: center; gap: 0.35rem; border-radius: 0.45rem; padding-inline: 0.4rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .composer-mode:hover { background: var(--accent); color: var(--foreground); }
  .composer-mode :global(svg) { flex: 0 0 auto; }
  .composer-mode span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .attachment-menu > div { position: absolute; left: 0; bottom: calc(100% + 0.5rem); z-index: 35; display: grid; min-width: 13rem; gap: 0.15rem; border: 1px solid var(--border); border-radius: 0.7rem; background: var(--popover); padding: 0.35rem; box-shadow: 0 14px 36px rgb(0 0 0 / 0.2); }
  .attachment-menu > div button { display: flex; min-height: 2rem; align-items: center; gap: 0.55rem; border-radius: 0.4rem; padding: 0.35rem 0.5rem; color: var(--foreground); font-size: 0.733333rem; text-align: left; }
  .attachment-menu > div button:hover { background: var(--accent); }
  .context-ring { position: relative; display: grid; width: 1.45rem; height: 1.45rem; flex: 0 0 auto; place-items: center; border-radius: 999px; color: var(--muted-foreground); outline: none; }
  .context-ring.warning { color: var(--status-tentative); }
  .context-ring svg { width: 1.2rem; height: 1.2rem; transform: rotate(-90deg); overflow: visible; }
  .context-ring circle { fill: none; stroke-width: 3; }
  .context-track { stroke: color-mix(in srgb, var(--muted-foreground) 22%, transparent); }
  .context-fill { stroke: currentColor; stroke-linecap: round; stroke-dasharray: 43.9823; stroke-dashoffset: calc(43.9823 * (1 - var(--context-progress))); transition: stroke-dashoffset 180ms ease; }
  .context-tooltip { position: absolute; right: 50%; bottom: calc(100% + 0.55rem); z-index: 55; display: none; width: max-content; max-width: min(18rem, 80vw); transform: translateX(50%); border: 1px solid var(--border); border-radius: 0.75rem; background: var(--popover); padding: 0.55rem 0.75rem; color: var(--popover-foreground); box-shadow: 0 12px 30px rgb(0 0 0 / 0.18); text-align: center; }
  .context-ring:hover .context-tooltip { display: grid; gap: 0.15rem; }
  .context-tooltip strong { color: var(--muted-foreground); font-size: 0.7rem; font-weight: 400; }
  .context-tooltip span { font-size: 0.733333rem; }
  .context-tooltip small { max-width: 15rem; color: var(--muted-foreground); font-size: 0.633333rem; line-height: 0.9rem; }
  .toolbar-right small { max-width: 18rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .primary-action { display: inline-flex; width: 2.1rem; height: 2.1rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 999px; background: color-mix(in srgb, var(--primary) 92%, transparent); color: var(--primary-foreground); box-shadow: 0 2px 7px color-mix(in srgb, var(--primary) 22%, transparent); transition: transform 120ms ease, filter 120ms ease; }
  .primary-action:hover:not(:disabled) { filter: brightness(1.04); transform: scale(1.04); }
  .primary-action:disabled { opacity: 0.5; }
  .force-stop { color: var(--destructive); }
  .attachment-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr)); gap: 0.4rem; margin-top: 0.65rem; }
  .attachment-grid article { position: relative; display: grid; grid-template-columns: 2.5rem minmax(0, 1fr) auto; align-items: center; gap: 0.35rem; border: 1px solid var(--border); border-radius: 0.45rem; padding: 0.3rem; }
  .attachment-grid article > span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.666667rem; }
  .attachment-preview { display: grid; width: 2.5rem; height: 2.5rem; place-items: center; overflow: hidden; border-radius: 0.3rem; background: var(--muted); }
  .attachment-preview img { width: 100%; height: 100%; object-fit: cover; }
  .mention-chips { display: flex; flex-wrap: wrap; gap: 0.3rem; margin-top: 0.65rem; }
  .mention-chips > span { display: inline-flex; max-width: 100%; align-items: center; gap: 0.2rem; border: 1px solid var(--border); border-radius: 999px; padding: 0.2rem 0.4rem; font-size: 0.666667rem; }
  .mention-chips small { color: var(--status-tentative); }
  .composer-menu { position: absolute; inset-inline: 0; bottom: calc(100% + 0.35rem); z-index: 25; max-height: min(20rem, 55vh); overflow: auto; overscroll-behavior: contain; scrollbar-gutter: stable; border: 1px solid var(--border); border-radius: 0.5rem; background: var(--popover); padding: 0.35rem; }
  .composer-menu > label, .composer-menu > p { display: flex; align-items: center; gap: 0.35rem; padding: 0.35rem; color: var(--muted-foreground); font-size: 0.7rem; }
  .composer-menu > button { display: block; width: 100%; border-radius: 0.35rem; padding: 0.45rem 0.5rem; text-align: left; }
  .composer-menu > button:hover, .composer-menu > button.selected { background: var(--accent); }
  .composer-menu strong, .composer-menu small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .composer-menu strong { font-size: 0.733333rem; }
  .composer-menu small { color: var(--muted-foreground); font-size: 0.666667rem; }
  .composer-menu > .prompt-option { display: flex; min-height: 2.35rem; align-items: center; gap: 0.65rem; padding: 0.45rem 0.6rem; }
  .prompt-option > :global(svg) { flex: 0 0 auto; color: var(--muted-foreground); }
  .prompt-option-copy { display: flex; min-width: 0; flex: 1; align-items: baseline; gap: 0.45rem; }
  .prompt-option-copy strong, .prompt-option-copy small { display: block; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .prompt-option-copy strong { flex: 0 0 auto; color: var(--foreground); font-size: 0.8rem; font-weight: 500; }
  .prompt-option-copy small { flex: 1; color: var(--muted-foreground); font-size: 0.733333rem; }
  .prompt-option-copy .stale-command { flex: 0 1 auto; color: var(--status-tentative); }
  .composer-info-panel { position: absolute; inset-inline: 0; bottom: calc(100% + 0.4rem); z-index: 30; max-height: min(22rem, calc(100dvh - 7rem)); overflow: auto; overscroll-behavior: contain; margin-inline: 0 !important; border: 1px solid var(--border); border-radius: 0.85rem; background: var(--popover); color: var(--popover-foreground); }
  .composer-info-panel > header { position: sticky; top: 0; z-index: 1; display: flex; min-height: 2.6rem; align-items: center; justify-content: space-between; gap: 1rem; border-bottom: 1px solid color-mix(in srgb, var(--border) 72%, transparent); background: var(--popover); padding: 0.55rem 0.9rem; }
  .composer-info-panel > header strong { font-size: 0.866667rem; font-weight: 600; }
  .composer-info-panel > header button { border-radius: 0.4rem; padding: 0.2rem 0.35rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .composer-info-panel > header button:hover { background: var(--accent); color: var(--foreground); }
  .status-panel-grid { display: grid; padding: 0.55rem 0.9rem 0.75rem; font-size: 0.733333rem; }
  .status-panel-grid > div { display: grid; min-width: 0; grid-template-columns: 6.5rem minmax(0, 1fr); align-items: baseline; gap: 0.75rem; padding-block: 0.28rem; }
  .status-panel-grid dt { color: var(--muted-foreground); }
  .status-panel-grid dd { display: flex; min-width: 0; align-items: center; gap: 0.55rem; margin: 0; overflow: hidden; }
  .status-panel-grid code { overflow: hidden; color: inherit; font: inherit; text-overflow: ellipsis; white-space: nowrap; }
  .status-state { flex: 0 0 auto; border-radius: 999px; background: var(--muted); padding: 0.08rem 0.38rem; color: var(--muted-foreground); font-size: 0.633333rem; }
  .status-panel-grid > .status-meter-row { align-items: start; }
  .status-meter-row dd { display: grid; gap: 0.28rem; }
  .status-meter-row dd > span:first-child { display: flex; min-width: 0; justify-content: space-between; gap: 0.75rem; }
  .status-meter-row small { color: var(--muted-foreground); font-size: 0.666667rem; }
  .panel-meter { display: block; width: 100%; height: 0.38rem; overflow: hidden; border-radius: 999px; background: color-mix(in srgb, var(--muted-foreground) 18%, transparent); }
  .panel-meter > span { display: block; width: calc(var(--panel-progress) * 100%); height: 100%; border-radius: inherit; background: color-mix(in srgb, var(--foreground) 72%, var(--muted-foreground)); }
  .mcp-status-table { display: grid; padding-block: 0.4rem; font-size: 0.733333rem; }
  .mcp-status-table > div { display: grid; min-width: 0; grid-template-columns: minmax(8rem, 1fr) minmax(8rem, 1fr) auto; align-items: center; gap: 1rem; padding: 0.36rem 0.9rem; }
  .mcp-status-table > div:hover { background: color-mix(in srgb, var(--accent) 52%, transparent); }
  .mcp-status-table strong { overflow: hidden; font-family: var(--font-mono, monospace); font-weight: 400; text-overflow: ellipsis; white-space: nowrap; }
  .mcp-status-table span { color: var(--muted-foreground); }
  .mcp-status-table span:last-child { justify-self: end; color: var(--destructive); }
  .mcp-status-table span.enabled { color: var(--foreground); }
  .panel-state, .panel-error { display: flex; min-height: 4.5rem; align-items: center; gap: 0.5rem; padding: 0.75rem 0.9rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .panel-error { color: var(--destructive); }
  .queued-row, .recovery-row { display: flex; flex-wrap: wrap; align-items: center; gap: 0.4rem; margin-top: 0.65rem; border: 1px solid var(--border); border-radius: 0.55rem; padding: 0.45rem; font-size: 0.7rem; }
  .queued-row div { min-width: 0; flex: 1; }
  .queued-row p { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); }
  .queued-row button, .recovery-row button { border-radius: 0.3rem; padding: 0.25rem 0.4rem; }
  .recovery-row { border-color: color-mix(in oklab, var(--destructive) 45%, var(--border)); color: var(--destructive); }
  .composer-error { margin-bottom: 0.65rem; color: var(--destructive); font-size: 0.733333rem; }
  .request-panel-shell { margin-bottom: 0.75rem; }
  @container chat-composer (max-width: 640px) { .toolbar-right small { max-width: 8rem; } }
  @container chat-composer (max-width: 460px) { .format-action { display: none; } }
  @container chat-composer (max-width: 390px) { .toolbar-left { gap: 0.1rem; } .context-ring { display: none; } .prompt-option-copy small { display: none; } .status-panel-grid > div { grid-template-columns: 5rem minmax(0, 1fr); } .mcp-status-table > div { grid-template-columns: minmax(0, 1fr) auto; gap: 0.65rem; } .mcp-status-table span:nth-child(2) { grid-column: 1 / -1; grid-row: 2; } }
  @container chat-composer (max-width: 300px) { .composer-editor { padding-inline: 0.8rem; } .composer-toolbar { padding-inline: 0.4rem; } .attachment-grid { grid-template-columns: 1fr; } }
  @media (prefers-reduced-motion: reduce) { .chat-composer { scroll-behavior: auto; } }
</style>
