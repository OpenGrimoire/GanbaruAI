<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import Zap from "@lucide/svelte/icons/zap";
  import type {
    InteractionMode,
    ModelOptionDefinition,
    ModelOptionSelection,
    ModelOptionValue,
    ProviderModel,
  } from "$lib/chat/contracts";
  import { composerModelSelection, rankedModels, readComposerModelSelection } from "$lib/chat/composer-model";
  import * as chatApi from "$lib/api/chat";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    isPointerAimingAtSubmenu,
    type MenuAimPoint,
    type MenuAimRect,
    type MenuAimSide,
  } from "$lib/projects/menu-aim";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatProviderIcon from "./ChatProviderIcon.svelte";

  type KnownModelOption = Exclude<ModelOptionDefinition, { kind: "unknown" }>;
  type PickerView = "overview" | "advanced";
  type FlyoutView = "providers" | "models" | "option" | "interaction";
  interface QuickEffortChoice {
    modelId: string;
    modelName: string;
    effortKey: string;
    effortValue: string;
    effortLabel: string;
  }

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const settings = getSettingsLauncher();
  let pickerOpen = $state(false);
  let pickerWasOpen = false;
  let pickerRoot: HTMLDivElement | undefined = $state();
  let pickerPanel: HTMLDivElement | undefined = $state();
  let pickerTrigger: HTMLButtonElement | undefined = $state();
  let pickerTriggerContent: HTMLSpanElement | undefined = $state();
  let overviewPanel: HTMLDivElement | undefined = $state();
  let advancedPanel: HTMLDivElement | undefined = $state();
  let advancedToggle: HTMLButtonElement | undefined = $state();
  let advancedHeading: HTMLButtonElement | undefined = $state();
  let flyoutPanel: HTMLDivElement | undefined = $state();
  let pickerStageHeight = $state<number | null>(null);
  let modelControlWidth = $state<number | null>(null);
  let effortPressing = $state(false);
  let effortDragging = $state(false);
  let effortHandleHovered = $state(false);
  let modelSearch: HTMLInputElement | undefined = $state();
  let providerForkDialog: HTMLElement | undefined = $state();
  let confirmationReturnFocus: HTMLElement | null = null;
  let pendingProviderId = $state<string | null>(null);
  let modelQuery = $state("");
  let view = $state<PickerView>("overview");
  let flyout = $state<FlyoutView | null>(null);
  let optionViewKey = $state<string | null>(null);
  let quickAnchorModelId = $state<string | null>(null);
  let restoredKey = $state("");
  let activeFlyoutTrigger: HTMLElement | null = null;
  let hoveredFlyoutTrigger: HTMLElement | null = null;
  let flyoutCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingFlyoutTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingFlyoutTrigger: HTMLElement | null = null;
  let modelControlResetTimer: ReturnType<typeof setTimeout> | null = null;
  let effortClickResetTimer: ReturnType<typeof setTimeout> | null = null;
  let effortPointerId: number | null = null;
  let effortPointerOrigin: { x: number; y: number } | null = null;
  let effortLastDragIndex = -1;
  let suppressEffortPointerClick = false;
  const flyoutCloseDelayMs = 70;
  const flyoutGraceDelayMs = 340;
  const modelControlResizeMs = 280;
  const effortDragThresholdPx = 5;
  const effortTrackHeightRem = 1.75;
  const effortEndpointInsetRem = effortTrackHeightRem / 2;
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const healthyProviders = $derived(providers.filter((entry) => providerAvailable(entry)));
  const provider = $derived(providers.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const unconfiguredFamilies = $derived((chat.settings?.providerFamilies ?? []).filter((family) => !providers.some((entry) => entry.configuration.familyId === family.familyId)));
  const selection = $derived(readComposerModelSelection(chat.composer.modelSelection));
  const models = $derived(provider?.modelCatalog?.models.filter((model) => provider.configuration.visibleModelIds.length === 0 || provider.configuration.visibleModelIds.includes(model.id) || model.id === selection.modelId) ?? []);
  const recentIds = $derived(chat.settings?.configuration.rememberedSelections.filter((entry) => entry.providerInstanceId === provider?.configuration.instanceId && entry.modelId).map((entry) => entry.modelId as string) ?? []);
  const ranked = $derived(rankedModels(models, provider?.configuration.favoriteModelIds ?? [], recentIds, modelQuery));
  const selectedModel = $derived(models.find((model) => model.id === selection.modelId) ?? null);
  const quickAnchorModel = $derived(models.find((model) => model.id === quickAnchorModelId) ?? selectedModel);
  const knownOptions = $derived((selectedModel?.options ?? []).filter(isKnownOption));
  const effortDefinition = $derived(knownOptions.find((definition) => optionRole(definition) === "effort") ?? null);
  const speedDefinition = $derived.by(() => {
    const candidates = [selectedModel, quickAnchorModel, ...models];
    const visited = new Set<string>();
    for (const candidate of candidates) {
      if (!candidate || visited.has(candidate.id)) continue;
      visited.add(candidate.id);
      for (const definition of candidate.options) {
        if (isKnownOption(definition) && optionRole(definition) === "speed") return definition;
      }
    }
    return null;
  });
  const otherDefinitions = $derived(knownOptions.filter((definition) => optionRole(definition) === "other"));
  const optionViewDefinition = $derived(knownOptions.find((definition) => definition.key === optionViewKey) ?? (speedDefinition?.key === optionViewKey ? speedDefinition : null));
  const providerManagedOnly = $derived((provider?.modelCatalog?.models.length ?? 0) === 0);
  const capabilities = $derived(chat.interaction?.capabilities ?? provider?.lastProbe?.capabilities ?? { entries: [] });
  const supportsPlan = $derived(capabilities.entries.some((entry) => entry.capability === "native_plan" && entry.supported));
  const quickEffortChoices = $derived(buildQuickEffortChoices(quickAnchorModel, models));
  const selectedQuickEffortIndex = $derived(quickEffortChoices.findIndex((choice) => choice.modelId === selection.modelId && choice.effortValue === selectedEffortValue()));

  onDestroy(() => {
    cancelFlyoutClose();
    cancelPendingFlyoutActivation();
    cancelModelControlReset();
    cancelEffortClickReset();
  });

  $effect(() => {
    if (!pickerOpen && pickerWasOpen) queueMicrotask(() => pickerTrigger?.focus());
    pickerWasOpen = pickerOpen;
  });

  $effect(() => {
    if (!pickerOpen) return;
    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (event.target instanceof Node && !pickerRoot?.contains(event.target)) closePicker();
    };
    window.addEventListener("pointerdown", closeOnOutsidePointer, true);
    return () => window.removeEventListener("pointerdown", closeOnOutsidePointer, true);
  });

  $effect(() => {
    if (!pickerOpen) {
      pickerStageHeight = null;
      return;
    }
    const activePanel = view === "overview" ? overviewPanel : advancedPanel;
    if (!activePanel) return;
    const updateHeight = () => {
      const renderedHeight = activePanel.getBoundingClientRect().height;
      const nextHeight = view === "advanced" ? Math.max(activePanel.scrollHeight, renderedHeight) : renderedHeight;
      if (nextHeight > 0) pickerStageHeight = Math.ceil(nextHeight);
    };
    updateHeight();
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(updateHeight);
    observer.observe(activePanel);
    return () => observer.disconnect();
  });

  $effect(() => {
    const workspaceId = chat.composer.workspaceId;
    if (!workspaceId || chat.composer.providerInstanceId) return;
    const preferred = chat.settings?.configuration.workspaceProviderPreferences[workspaceId];
    if (preferred && healthyProviders.some((entry) => entry.configuration.instanceId === preferred)) chat.setComposerProvider(preferred);
  });

  $effect(() => {
    const workspaceId = chat.composer.workspaceId;
    const providerId = chat.composer.providerInstanceId;
    if (!workspaceId || !providerId || chat.composer.loading) return;
    const key = `${workspaceId}:${providerId}`;
    if (restoredKey === key || chat.composer.modelSelection || chat.composer.safetyMode || chat.composer.interactionMode) return;
    const remembered = chat.settings?.configuration.rememberedSelections.find((entry) => entry.workspaceId === workspaceId && entry.providerInstanceId === providerId);
    restoredKey = key;
    if (!remembered) return;
    chat.setComposerModel(composerModelSelection(remembered.modelId, remembered.providerManagedModel, remembered.modelOptions));
    chat.setComposerModes(remembered.safetyMode, remembered.interactionMode);
  });

  $effect(() => {
    if (!provider || chat.composer.loading || selection.modelId || models.length === 0) return;
    const recommended = models.find((model) => model.id === "default") ?? models[0];
    if (!recommended) return;
    chat.setComposerModel(composerModelSelection(recommended.id, false, defaultOptions(recommended.options)));
    quickAnchorModelId = recommended.id;
  });

  function togglePicker(): void {
    if (pickerOpen) {
      closePicker();
      return;
    }
    const anchoredChoices = buildQuickEffortChoices(
      models.find((model) => model.id === quickAnchorModelId) ?? null,
      models,
    );
    if (!anchoredChoices.some((choice) => choice.modelId === selection.modelId && choice.effortValue === selectedEffortValue())) {
      quickAnchorModelId = selectedModel?.id ?? null;
    }
    view = provider ? "overview" : "advanced";
    flyout = provider ? null : "providers";
    openPicker();
  }

  function closePicker(): void {
    pickerOpen = false;
    modelQuery = "";
    cancelEffortClickReset();
    suppressEffortPointerClick = false;
    effortHandleHovered = false;
    resetEffortPointerState();
    closeFlyout();
    void tick().then(collapseModelControl);
  }

  function numericStyleValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function compactModelControlWidth(): number {
    if (!pickerTrigger || !pickerTriggerContent) return pickerRoot?.getBoundingClientRect().width ?? 0;
    const style = getComputedStyle(pickerTrigger);
    const horizontalChrome = numericStyleValue(style.paddingLeft)
      + numericStyleValue(style.paddingRight)
      + numericStyleValue(style.borderLeftWidth)
      + numericStyleValue(style.borderRightWidth);
    const naturalWidth = pickerTriggerContent.scrollWidth + horizontalChrome;
    const maximumWidth = numericStyleValue(style.maxWidth);
    return Math.ceil(maximumWidth > 0 ? Math.min(naturalWidth, maximumWidth) : naturalWidth);
  }

  function cancelModelControlReset(): void {
    if (modelControlResetTimer === null) return;
    clearTimeout(modelControlResetTimer);
    modelControlResetTimer = null;
  }

  function openPicker(): void {
    cancelModelControlReset();
    const measuredWidth = pickerRoot?.getBoundingClientRect().width ?? 0;
    const currentWidth = measuredWidth > 0 ? measuredWidth : compactModelControlWidth();
    modelControlWidth = Math.ceil(currentWidth);
    pickerOpen = true;
    void tick().then(() => {
      const compactWidth = compactModelControlWidth();
      const panelWidth = pickerPanel?.getBoundingClientRect().width ?? compactWidth;
      pickerRoot?.getBoundingClientRect();
      modelControlWidth = Math.ceil(Math.max(compactWidth, panelWidth));
    });
  }

  function collapseModelControl(): void {
    cancelModelControlReset();
    modelControlWidth = compactModelControlWidth();
    modelControlResetTimer = setTimeout(() => {
      modelControlResetTimer = null;
      if (!pickerOpen) modelControlWidth = null;
    }, modelControlResizeMs);
  }

  function setPickerView(nextView: PickerView): void {
    if (view === nextView) return;
    view = nextView;
    closeFlyout();
    void tick().then(() => {
      const target = nextView === "advanced" ? advancedHeading : advancedToggle;
      target?.focus({ preventScroll: true });
    });
  }

  function providerAvailable(entry: (typeof providers)[number]): boolean {
    return entry.configuration.enabled && entry.lastProbe?.state === "healthy";
  }

  function probeStatus(entry: (typeof providers)[number]): string {
    if (!entry.configuration.enabled) return t("chat.composer.providerDisabled");
    if (!entry.lastProbe) return t("chat.composer.providerNotChecked");
    if (entry.lastProbe.state === "healthy") return t("chat.status.idle");
    return entry.lastProbe.detail ?? t("chat.status.providerUnavailable");
  }

  function selectProvider(entry: (typeof providers)[number]): void {
    if (!providerAvailable(entry)) {
      openProviderSettings();
      return;
    }
    if (chat.selectedThread && entry.configuration.instanceId !== chat.selectedThread.providerInstanceId) {
      confirmationReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      pendingProviderId = entry.configuration.instanceId;
      void tick().then(() => firstFocusable(providerForkDialog)?.focus());
      return;
    }
    chat.setComposerProvider(entry.configuration.instanceId);
    chat.setComposerModel(null);
    modelQuery = "";
    view = "advanced";
    flyout = "models";
  }

  function openProviderSettings(): void {
    closePicker();
    settings.open("chat", { chatSubsection: "providers" });
  }

  function chooseModel(modelId: string | null, providerManaged: boolean): void {
    const model = models.find((candidate) => candidate.id === modelId);
    chat.setComposerModel(composerModelSelection(modelId, providerManaged, model ? defaultOptions(model.options) : []));
    quickAnchorModelId = modelId;
    closePicker();
  }

  function chooseQuickEffort(choice: QuickEffortChoice): void {
    const model = models.find((candidate) => candidate.id === choice.modelId);
    if (!model) return;
    const options = choice.modelId === selection.modelId
      ? selection.options.filter((entry) => entry.key !== choice.effortKey)
      : defaultOptions(model.options).filter((entry) => entry.key !== choice.effortKey);
    options.push({ key: choice.effortKey, value: { kind: "choice", value: choice.effortValue } });
    chat.setComposerModel(composerModelSelection(model.id, false, options));
  }

  function cancelEffortClickReset(): void {
    if (effortClickResetTimer === null) return;
    clearTimeout(effortClickResetTimer);
    effortClickResetTimer = null;
  }

  function resetEffortPointerState(): void {
    effortPressing = false;
    effortDragging = false;
    effortPointerId = null;
    effortPointerOrigin = null;
    effortLastDragIndex = -1;
  }

  function updateEffortHandleHover(ladder: HTMLElement, clientX: number, clientY: number, selectedIndex = selectedQuickEffortIndex): void {
    const button = ladder.querySelectorAll<HTMLButtonElement>(".effort-options button")[selectedIndex];
    if (!button) {
      effortHandleHovered = false;
      return;
    }
    const rect = button.getBoundingClientRect();
    const radius = Math.min(rect.width, rect.height) / 2;
    effortHandleHovered = radius > 0
      && Math.hypot(clientX - (rect.left + rect.width / 2), clientY - (rect.top + rect.height / 2)) <= radius;
  }

  function chooseEffortAtPointer(ladder: HTMLElement, clientX: number): void {
    const buttons = [...ladder.querySelectorAll<HTMLButtonElement>(".effort-options button")];
    if (buttons.length === 0) return;
    let nearestIndex = 0;
    let nearestDistance = Number.POSITIVE_INFINITY;
    for (const [index, button] of buttons.entries()) {
      const rect = button.getBoundingClientRect();
      const distance = Math.abs(clientX - (rect.left + rect.width / 2));
      if (distance < nearestDistance) {
        nearestDistance = distance;
        nearestIndex = index;
      }
    }
    if (nearestIndex === effortLastDragIndex) return;
    effortLastDragIndex = nearestIndex;
    const choice = quickEffortChoices[nearestIndex];
    if (choice) chooseQuickEffort(choice);
  }

  function handleEffortPointerDown(event: PointerEvent): void {
    if (event.button !== 0 || event.isPrimary === false || !(event.currentTarget instanceof HTMLElement)) return;
    event.preventDefault();
    cancelEffortClickReset();
    effortPressing = true;
    effortDragging = false;
    effortPointerId = event.pointerId;
    effortPointerOrigin = { x: event.clientX, y: event.clientY };
    effortLastDragIndex = selectedQuickEffortIndex;
    suppressEffortPointerClick = true;
    event.currentTarget.setPointerCapture?.(event.pointerId);
    chooseEffortAtPointer(event.currentTarget, event.clientX);
  }

  function handleEffortPointerMove(event: PointerEvent): void {
    if (!(event.currentTarget instanceof HTMLElement)) return;
    if (effortPressing && event.pointerId === effortPointerId) {
      if (!effortDragging && effortPointerOrigin && Math.hypot(event.clientX - effortPointerOrigin.x, event.clientY - effortPointerOrigin.y) >= effortDragThresholdPx) {
        effortDragging = true;
      }
      chooseEffortAtPointer(event.currentTarget, event.clientX);
    }
    updateEffortHandleHover(
      event.currentTarget,
      event.clientX,
      event.clientY,
      effortPressing ? effortLastDragIndex : selectedQuickEffortIndex,
    );
  }

  function finishEffortPointer(event: PointerEvent, selectFinalChoice: boolean): void {
    if (!effortPressing || event.pointerId !== effortPointerId || !(event.currentTarget instanceof HTMLElement)) return;
    if (selectFinalChoice) chooseEffortAtPointer(event.currentTarget, event.clientX);
    if (selectFinalChoice) updateEffortHandleHover(event.currentTarget, event.clientX, event.clientY, effortLastDragIndex);
    else effortHandleHovered = false;
    if (event.currentTarget.hasPointerCapture?.(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    resetEffortPointerState();
    cancelEffortClickReset();
    effortClickResetTimer = setTimeout(() => {
      effortClickResetTimer = null;
      suppressEffortPointerClick = false;
    }, 0);
  }

  function handleEffortPointerLeave(): void {
    effortHandleHovered = false;
  }

  function handleEffortChoiceClick(choice: QuickEffortChoice, event: MouseEvent): void {
    if (event.detail > 0 && suppressEffortPointerClick) {
      event.preventDefault();
      return;
    }
    chooseQuickEffort(choice);
  }

  function chooseInteraction(value: InteractionMode): void {
    chat.setComposerModes(chat.composer.safetyMode, value);
    closePicker();
  }

  function openOption(definition: KnownModelOption, trigger?: HTMLElement): void {
    cancelFlyoutClose();
    cancelPendingFlyoutActivation();
    if (trigger) activeFlyoutTrigger = trigger;
    optionViewKey = definition.key;
    flyout = "option";
  }

  function openFlyout(next: FlyoutView, trigger?: HTMLElement): void {
    cancelFlyoutClose();
    cancelPendingFlyoutActivation();
    if (trigger) activeFlyoutTrigger = trigger;
    optionViewKey = null;
    flyout = next;
  }

  function closeFlyout(): void {
    cancelFlyoutClose();
    cancelPendingFlyoutActivation();
    optionViewKey = null;
    flyout = null;
    activeFlyoutTrigger = null;
    hoveredFlyoutTrigger = null;
  }

  function cancelFlyoutClose(): void {
    if (flyoutCloseTimer === null) return;
    clearTimeout(flyoutCloseTimer);
    flyoutCloseTimer = null;
  }

  function scheduleFlyoutClose(delayMs: number): void {
    cancelFlyoutClose();
    flyoutCloseTimer = setTimeout(() => {
      flyoutCloseTimer = null;
      closeFlyout();
    }, delayMs);
  }

  function cancelPendingFlyoutActivation(): void {
    if (pendingFlyoutTimer !== null) clearTimeout(pendingFlyoutTimer);
    pendingFlyoutTimer = null;
    pendingFlyoutTrigger = null;
  }

  function flyoutBoundaryContains(target: EventTarget | null): boolean {
    if (!(target instanceof Node)) return false;
    return Boolean(activeFlyoutTrigger?.contains(target) || flyoutPanel?.contains(target));
  }

  function menuAimRect(rect: DOMRect): MenuAimRect {
    return { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom };
  }

  function menuAimSide(source: DOMRect, destination: DOMRect): MenuAimSide {
    const horizontalGap = Math.max(destination.left - source.right, source.left - destination.right, 0);
    const verticalGap = Math.max(destination.top - source.bottom, source.top - destination.bottom, 0);
    if (horizontalGap >= verticalGap) {
      return destination.left + destination.right < source.left + source.right ? "left" : "right";
    }
    return destination.top + destination.bottom < source.top + source.bottom ? "top" : "bottom";
  }

  function pointerAimingBetween(source: HTMLElement, destination: HTMLElement, event: PointerEvent): boolean {
    const sourceRect = source.getBoundingClientRect();
    const destinationRect = destination.getBoundingClientRect();
    const origin: MenuAimPoint = {
      x: sourceRect.left + sourceRect.width / 2,
      y: sourceRect.top + sourceRect.height / 2,
    };
    return isPointerAimingAtSubmenu({
      origin,
      point: { x: event.clientX, y: event.clientY },
      submenu: menuAimRect(destinationRect),
      side: menuAimSide(sourceRect, destinationRect),
      tolerance: 10,
      topTolerance: 10,
      bottomTolerance: 20,
      minTowardDistance: 3,
    });
  }

  function activateFlyoutFromPointer(trigger: HTMLElement, event: PointerEvent, activate: () => void): void {
    hoveredFlyoutTrigger = trigger;
    const crossingTowardCurrentFlyout = Boolean(
      activeFlyoutTrigger
      && activeFlyoutTrigger !== trigger
      && flyoutPanel
      && pointerAimingBetween(activeFlyoutTrigger, flyoutPanel, event),
    );
    if (!crossingTowardCurrentFlyout) {
      activate();
      return;
    }

    cancelFlyoutClose();
    cancelPendingFlyoutActivation();
    pendingFlyoutTrigger = trigger;
    pendingFlyoutTimer = setTimeout(() => {
      pendingFlyoutTimer = null;
      pendingFlyoutTrigger = null;
      if (hoveredFlyoutTrigger === trigger) activate();
    }, flyoutGraceDelayMs);
  }

  function handleOptionPointerEnter(definition: KnownModelOption, event: PointerEvent): void {
    if (!(event.currentTarget instanceof HTMLElement)) return;
    const trigger = event.currentTarget;
    activateFlyoutFromPointer(trigger, event, () => openOption(definition, trigger));
  }

  function handleNamedFlyoutPointerEnter(next: FlyoutView, event: PointerEvent): void {
    if (!(event.currentTarget instanceof HTMLElement)) return;
    const trigger = event.currentTarget;
    activateFlyoutFromPointer(trigger, event, () => openFlyout(next, trigger));
  }

  function handleFlyoutTriggerLeave(event: PointerEvent): void {
    const leavingTrigger = event.currentTarget instanceof HTMLElement ? event.currentTarget : null;
    if (hoveredFlyoutTrigger === leavingTrigger) hoveredFlyoutTrigger = null;
    if (pendingFlyoutTrigger === leavingTrigger) cancelPendingFlyoutActivation();
    if (flyoutBoundaryContains(event.relatedTarget)) {
      cancelFlyoutClose();
      return;
    }
    const trigger = leavingTrigger ?? activeFlyoutTrigger;
    const aimingAtFlyout = Boolean(trigger && flyoutPanel && pointerAimingBetween(trigger, flyoutPanel, event));
    scheduleFlyoutClose(aimingAtFlyout ? flyoutGraceDelayMs : flyoutCloseDelayMs);
  }

  function handleFlyoutPanelLeave(event: PointerEvent): void {
    if (flyoutBoundaryContains(event.relatedTarget)) {
      cancelFlyoutClose();
      return;
    }
    const aimingAtTrigger = Boolean(flyoutPanel && activeFlyoutTrigger && pointerAimingBetween(flyoutPanel, activeFlyoutTrigger, event));
    scheduleFlyoutClose(aimingAtTrigger ? flyoutGraceDelayMs : flyoutCloseDelayMs);
  }

  function handleFlyoutPanelEnter(): void {
    hoveredFlyoutTrigger = null;
    cancelPendingFlyoutActivation();
    cancelFlyoutClose();
  }

  function handleFlyoutFocusOut(): void {
    queueMicrotask(() => {
      const activeElement = document.activeElement;
      if (flyoutBoundaryContains(activeElement)) return;
      closeFlyout();
    });
  }

  function updateOption(key: string, value: ModelOptionValue): void {
    const next = selection.options.filter((entry) => entry.key !== key);
    next.push({ key, value });
    chat.setComposerModel(composerModelSelection(selection.modelId, selection.providerManaged, next));
  }

  function removeOption(key: string): void {
    chat.setComposerModel(composerModelSelection(
      selection.modelId,
      selection.providerManaged,
      selection.options.filter((entry) => entry.key !== key),
    ));
  }

  function option(key: string): ModelOptionSelection | undefined {
    return selection.options.find((entry) => entry.key === key);
  }

  function choiceValue(key: string): string {
    const value = option(key)?.value;
    return value?.kind === "choice" ? value.value : "";
  }

  function booleanValue(key: string): boolean {
    const value = option(key)?.value;
    return value?.kind === "boolean" && value.value;
  }

  function multipleIncludes(key: string, candidate: string): boolean {
    const value = option(key)?.value;
    return value?.kind === "multiple_choice" && value.value.includes(candidate);
  }

  function toggleMultiple(key: string, value: string, checked: boolean): void {
    const current = option(key)?.value;
    const values = current?.kind === "multiple_choice" ? current.value : [];
    updateOption(key, { kind: "multiple_choice", value: checked ? [...new Set([...values, value])] : values.filter((entry) => entry !== value) });
  }

  function integerValue(key: string, fallback: number): number {
    const value = option(key)?.value;
    return value?.kind === "integer" ? value.value : fallback;
  }

  function textValue(key: string): string {
    const value = option(key)?.value;
    return value?.kind === "text" ? value.value : "";
  }

  function selectedOptionLabel(definition: KnownModelOption | null): string | null {
    if (!definition) return null;
    if (definition.kind === "choice") {
      const selected = definition.options.find((entry) => entry.value === choiceValue(definition.key));
      return humanizeOptionLabel(selected?.label ?? definition.label);
    }
    if (definition.kind === "boolean") return booleanValue(definition.key) ? t("chat.composer.enabled") : t("chat.composer.disabled");
    if (definition.kind === "integer_range") return formatNumber(localization.locale, integerValue(definition.key, definition.defaultValue ?? definition.minimum));
    if (definition.kind === "text") return textValue(definition.key) || definition.label;
    const value = option(definition.key)?.value;
    return value?.kind === "multiple_choice" ? formatNumber(localization.locale, value.value.length) : definition.label;
  }

  function selectedEffortValue(): string | null {
    if (!effortDefinition) return null;
    const value = option(effortDefinition.key)?.value;
    return value?.kind === "choice" ? value.value : null;
  }

  function isUltraSelected(): boolean {
    return selectedQuickEffortIndex >= 0 && selectedQuickEffortIndex === quickEffortChoices.length - 1;
  }

  function quickStopPosition(index: number, count: number): string {
    const progress = count <= 1 ? 0.5 : index / (count - 1);
    const percent = progress * 100;
    const endpointOffset = (1 - 2 * progress) * effortEndpointInsetRem;
    if (Math.abs(endpointOffset) < 0.0001) return `${percent}%`;
    return endpointOffset > 0
      ? `calc(${percent}% + ${endpointOffset}rem)`
      : `calc(${percent}% - ${Math.abs(endpointOffset)}rem)`;
  }

  function isFastSelected(): boolean {
    if (!speedDefinition) return false;
    if (speedDefinition.kind === "boolean") return booleanValue(speedDefinition.key);
    if (speedDefinition.kind !== "choice") return false;
    const selected = speedDefinition.options.find((entry) => entry.value === choiceValue(speedDefinition.key));
    return `${selected?.value ?? ""} ${selected?.label ?? ""}`.toLowerCase().includes("fast");
  }

  function setFastMode(enabled: boolean): void {
    if (!speedDefinition) return;
    if (speedDefinition.kind === "boolean") {
      updateOption(speedDefinition.key, { kind: "boolean", value: enabled });
      return;
    }
    if (speedDefinition.kind !== "choice") return;
    const match = speedDefinition.options.find((entry) => {
      const identity = `${entry.value} ${entry.label}`.toLowerCase();
      return enabled ? identity.includes("fast") : identity.includes("standard") || identity.includes("default");
    });
    if (match) updateOption(speedDefinition.key, { kind: "choice", value: match.value });
    else if (!enabled) removeOption(speedDefinition.key);
  }

  function humanizeOptionLabel(label: string): string {
    const normalized = label.replaceAll("_", " ").trim();
    if (provider?.configuration.familyId === "codex" && normalized.toLowerCase() === "low") return t("chat.composer.light");
    if (["xhigh", "extra high"].includes(normalized.toLowerCase())) return t("chat.composer.extraHigh");
    if (!normalized || normalized !== normalized.toLowerCase()) return normalized;
    return `${normalized[0]?.toUpperCase() ?? ""}${normalized.slice(1)}`;
  }

  function displayModelName(model: ProviderModel | null): string {
    if (!model) return provider?.configuration.label ?? t("chat.hero.chooseProvider");
    if (provider?.configuration.familyId !== "codex") return model.displayName;
    return model.displayName.replace(/^GPT-/i, "").replaceAll("-", " ");
  }

  function buildQuickEffortChoices(anchor: ProviderModel | null, availableModels: ProviderModel[]): QuickEffortChoice[] {
    if (!anchor) return [];
    const anchorEffort = anchor.options.find((definition) => isKnownOption(definition) && optionRole(definition) === "effort");
    if (!anchorEffort || anchorEffort.kind !== "choice") return [];
    const choices: QuickEffortChoice[] = [];
    const lowerModel = recommendedLowerModel(anchor, availableModels);
    if (lowerModel) {
      const lowerEffort = lowerModel.options.find((definition) => isKnownOption(definition) && optionRole(definition) === "effort");
      if (lowerEffort?.kind === "choice") {
        const light = lowerEffort.options.find((choice) => choice.value.toLowerCase() === "low");
        if (light) choices.push({ modelId: lowerModel.id, modelName: displayModelName(lowerModel), effortKey: lowerEffort.key, effortValue: light.value, effortLabel: humanizeOptionLabel(light.label) });
      }
    }
    for (const effort of anchorEffort.options) {
      if (["none", "minimal"].includes(effort.value.toLowerCase())) continue;
      choices.push({ modelId: anchor.id, modelName: displayModelName(anchor), effortKey: anchorEffort.key, effortValue: effort.value, effortLabel: humanizeOptionLabel(effort.label) });
    }
    return choices;
  }

  function recommendedLowerModel(anchor: ProviderModel, availableModels: ProviderModel[]): ProviderModel | null {
    if (provider?.configuration.familyId !== "codex") return null;
    const lowerTier = anchor.id.endsWith("-sol") ? "terra" : anchor.id.endsWith("-terra") ? "luna" : null;
    if (!lowerTier) return null;
    const lowerId = anchor.id.replace(/-(sol|terra)$/, `-${lowerTier}`);
    return availableModels.find((model) => model.id === lowerId) ?? null;
  }

  function isKnownOption(definition: ModelOptionDefinition): definition is KnownModelOption {
    return definition.kind !== "unknown";
  }

  function optionRole(definition: KnownModelOption): "effort" | "speed" | "other" {
    const identity = `${definition.key} ${definition.label}`.toLowerCase();
    if (identity.includes("effort") || identity.includes("reasoning")) return "effort";
    if (identity.includes("speed") || identity.includes("fast") || identity.includes("service tier") || identity.includes("service_tier")) return "speed";
    return "other";
  }

  function defaultOptions(definitions: ModelOptionDefinition[]): ModelOptionSelection[] {
    const options: ModelOptionSelection[] = [];
    for (const definition of definitions) {
      switch (definition.kind) {
        case "boolean": if (definition.defaultValue !== null) options.push({ key: definition.key, value: { kind: "boolean", value: definition.defaultValue } }); break;
        case "choice": if (definition.defaultValue !== null) options.push({ key: definition.key, value: { kind: "choice", value: definition.defaultValue } }); break;
        case "multiple_choice": options.push({ key: definition.key, value: { kind: "multiple_choice", value: [...definition.defaultValue] } }); break;
        case "integer_range": if (definition.defaultValue !== null) options.push({ key: definition.key, value: { kind: "integer", value: definition.defaultValue } }); break;
        case "text": if (definition.defaultValue !== null) options.push({ key: definition.key, value: { kind: "text", value: definition.defaultValue } }); break;
        case "unknown": break;
      }
    }
    return options;
  }

  function modelMetadata(contextLimit: number | null, availability: string): string[] {
    const values: string[] = [];
    if (contextLimit !== null) values.push(t("chat.composer.modelContext", formatNumber(localization.locale, contextLimit)));
    if (availability === "stale") values.push(t("chat.composer.modelStale"));
    if (availability === "unavailable") values.push(t("chat.composer.modelUnavailable"));
    if (availability === "deprecated") values.push(t("chat.composer.modelDeprecated"));
    return values;
  }

  function handlePickerKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      closePicker();
      return;
    }
    if (event.key !== "Tab" || !(event.currentTarget instanceof HTMLElement)) return;
    const focusable = focusableElements(event.currentTarget);
    const first = focusable[0];
    const last = focusable.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function focusableElements(container: HTMLElement): HTMLElement[] {
    return [...container.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex='-1'])",
    )].filter((element) => !element.hidden && !element.closest("[inert]") && element.getClientRects().length > 0);
  }

  function firstFocusable(container: HTMLElement | undefined): HTMLElement | undefined {
    return container ? focusableElements(container)[0] : undefined;
  }

  async function confirmProviderFork(): Promise<void> {
    if (!pendingProviderId) return;
    const instanceId = pendingProviderId;
    pendingProviderId = null;
    await chat.forkComposerWithProvider(instanceId);
    closePicker();
  }
</script>

<div bind:this={pickerRoot} class="model-control" class:measured={modelControlWidth !== null} style:width={modelControlWidth === null ? undefined : `${modelControlWidth}px`}>
  <button bind:this={pickerTrigger} type="button" class="model-trigger" data-chat-model-trigger aria-expanded={pickerOpen} onclick={togglePicker}>
    <span bind:this={pickerTriggerContent} class="model-trigger-content">
      <span class="fast-indicator" class:active={isFastSelected()} aria-hidden="true"><Zap size={13} fill="currentColor" /></span>
      <span class="model-name">{selection.providerManaged ? provider?.configuration.label ?? t("chat.composer.providerManagedModel") : displayModelName(selectedModel)}</span>
      {#if selectedOptionLabel(effortDefinition)}<span class="effort-name" class:ultra={isUltraSelected()}>{selectedOptionLabel(effortDefinition)}</span>{/if}
      <ChevronDown size={13} class="model-chevron" />
    </span>
  </button>

  {#if pickerOpen}
    <div bind:this={pickerPanel} class="model-popover" role="dialog" aria-label={t("chat.hero.model")} tabindex="-1" onkeydown={handlePickerKeydown}>
      <div class="picker-stage" style:height={pickerStageHeight === null ? undefined : `${pickerStageHeight}px`}>
        <div bind:this={overviewPanel} class="picker-view overview-view" class:active={view === "overview"} inert={view !== "overview"} aria-hidden={view !== "overview"}>
        {#if quickEffortChoices.length > 0}
          <div class="effort-ladder" class:fast={isFastSelected()} class:ultra={isUltraSelected()} class:holding={effortPressing} class:handle-hovered={effortHandleHovered} style={`--effort-track-height:${effortTrackHeightRem}rem`} role="group" aria-label={t("chat.composer.quickModelEffort")} onpointerdown={handleEffortPointerDown} onpointermove={handleEffortPointerMove} onpointerleave={handleEffortPointerLeave} onpointerup={(event) => finishEffortPointer(event, true)} onpointercancel={(event) => finishEffortPointer(event, false)} onlostpointercapture={(event) => finishEffortPointer(event, false)}>
            <span class="effort-fill" style={`width:${selectedQuickEffortIndex < 0 ? "0" : isUltraSelected() ? "100%" : quickStopPosition(selectedQuickEffortIndex, quickEffortChoices.length)}`} aria-hidden="true">
              <span class="effort-particles calm"></span>
              <span class="effort-particles rapid"></span>
            </span>
            <span class="effort-options">
              {#each quickEffortChoices as choice, index}
                <button type="button" style={`left:${quickStopPosition(index, quickEffortChoices.length)}`} aria-label={`${choice.modelName} ${choice.effortLabel}`} aria-pressed={index === selectedQuickEffortIndex} data-app-tooltip-disabled="true" data-app-tooltip-focus-disabled="true" onclick={(event) => handleEffortChoiceClick(choice, event)}><i></i></button>
              {/each}
              {#if selectedQuickEffortIndex >= 0}<span class="effort-knob" style={`left:${quickStopPosition(selectedQuickEffortIndex, quickEffortChoices.length)}`} aria-hidden="true"></span>{/if}
            </span>
          </div>
        {/if}
        <div class="effort-footer" class:holding={effortDragging}>
          <div class="quick-actions" inert={effortDragging} aria-hidden={effortDragging}>
            <button bind:this={advancedToggle} type="button" class="advanced-toggle" onclick={() => setPickerView("advanced")}><span>{t("chat.composer.advanced")}</span><ChevronRight size={15} /></button>
            {#if speedDefinition}<button type="button" class="fast-button" class:active={isFastSelected()} class:ultra={isUltraSelected()} title={isFastSelected() ? t("chat.composer.fastEnabled") : t("chat.composer.enableFast")} aria-label={isFastSelected() ? t("chat.composer.fastEnabled") : t("chat.composer.enableFast")} aria-pressed={isFastSelected()} onclick={() => setFastMode(!isFastSelected())}><Zap size={16} /></button>{/if}
          </div>
          <div class="effort-guidance" aria-hidden="true"><span>{t("chat.composer.faster")}</span><span>{t("chat.composer.smarter")}</span></div>
        </div>
        </div>
        <div bind:this={advancedPanel} class="picker-view advanced-view" class:active={view === "advanced"} inert={view !== "advanced"} aria-hidden={view !== "advanced"} onfocusout={handleFlyoutFocusOut}>
        <button bind:this={advancedHeading} type="button" class="advanced-heading" onclick={() => setPickerView("overview")}><span>{t("chat.composer.advanced")}</span><ChevronDown size={15} /></button>
        <div class="advanced-list">
          <button type="button" onpointerenter={(event) => handleNamedFlyoutPointerEnter("models", event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openFlyout("models", event.currentTarget)} onclick={(event) => { openFlyout("models", event.currentTarget); void tick().then(() => modelSearch?.focus()); }}><span>{t("chat.hero.model")}</span><small>{selection.providerManaged ? t("chat.composer.providerManagedModel") : displayModelName(selectedModel)}</small><ChevronRight size={15} /></button>
          {#if effortDefinition}<button type="button" onpointerenter={(event) => handleOptionPointerEnter(effortDefinition, event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openOption(effortDefinition, event.currentTarget)} onclick={(event) => openOption(effortDefinition, event.currentTarget)}><span>{t("chat.composer.effort")}</span><small>{selectedOptionLabel(effortDefinition)}</small><ChevronRight size={15} /></button>{/if}
          {#if speedDefinition}<button type="button" onpointerenter={(event) => handleOptionPointerEnter(speedDefinition, event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openOption(speedDefinition, event.currentTarget)} onclick={(event) => openOption(speedDefinition, event.currentTarget)}><span>{t("chat.composer.speed")}</span><small>{isFastSelected() ? t("chat.composer.fast") : t("chat.composer.standard")}</small><ChevronRight size={15} /></button>{/if}
          {#each otherDefinitions as definition}<button type="button" onpointerenter={(event) => handleOptionPointerEnter(definition, event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openOption(definition, event.currentTarget)} onclick={(event) => openOption(definition, event.currentTarget)}><span>{definition.label}</span><small>{selectedOptionLabel(definition)}</small><ChevronRight size={15} /></button>{/each}
          {#if providers.length > 1 || unconfiguredFamilies.length > 0}<button type="button" onpointerenter={(event) => handleNamedFlyoutPointerEnter("providers", event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openFlyout("providers", event.currentTarget)} onclick={(event) => openFlyout("providers", event.currentTarget)}><span>{t("chat.hero.provider")}</span><small>{provider?.configuration.label ?? t("chat.hero.chooseProvider")}</small><ChevronRight size={15} /></button>{/if}
          <button type="button" onpointerenter={(event) => handleNamedFlyoutPointerEnter("interaction", event)} onpointerleave={handleFlyoutTriggerLeave} onfocus={(event) => openFlyout("interaction", event.currentTarget)} onclick={(event) => openFlyout("interaction", event.currentTarget)}><span>{t("chat.hero.interaction")}</span><small>{chat.composer.interactionMode === "plan" ? t("chat.hero.plan") : t("chat.hero.build")}</small><ChevronRight size={15} /></button>
        </div>
        </div>
      </div>

        {#if view === "advanced" && flyout}
          <div bind:this={flyoutPanel} class="model-flyout" role="group" aria-label={flyout === "models" ? t("chat.hero.model") : flyout === "providers" ? t("chat.hero.provider") : flyout === "interaction" ? t("chat.hero.interaction") : optionViewDefinition?.label} onpointerenter={handleFlyoutPanelEnter} onpointerleave={handleFlyoutPanelLeave} onfocusout={handleFlyoutFocusOut}>
            {#if flyout === "providers"}
              <p class="flyout-heading">{t("chat.hero.provider")}</p>
              <div class="selection-list">
                {#each providers as entry}<button type="button" class:unavailable={!providerAvailable(entry)} onclick={() => selectProvider(entry)} title={probeStatus(entry)}><ChatProviderIcon familyId={entry.configuration.familyId} label={entry.configuration.label} accentColor={entry.configuration.accentColor} /><span><strong>{entry.configuration.label}</strong><small>{probeStatus(entry)}</small></span>{#if entry.configuration.instanceId === provider?.configuration.instanceId}<Check size={14} />{:else if !providerAvailable(entry)}<CircleAlert size={14} />{/if}</button>{/each}
                {#each unconfiguredFamilies as family}<button type="button" class="unavailable" onclick={openProviderSettings}><ChatProviderIcon familyId={family.familyId} label={family.displayName} /><span><strong>{family.displayName}</strong><small>{t("chat.composer.notConfigured")}</small></span><Plus size={14} /></button>{/each}
              </div>
            {:else if flyout === "models"}
              <p class="flyout-heading">{t("chat.hero.model")}</p>
              <label class="model-search"><Search size={14} /><input bind:this={modelSearch} bind:value={modelQuery} placeholder={t("chat.composer.modelSearch")} /></label>
              <div class="selection-list model-list">
                {#if providerManagedOnly || selection.providerManaged}<button type="button" disabled={!provider || !providerAvailable(provider)} onclick={() => chooseModel(null, true)}><span><strong>{t("chat.composer.providerManagedModel")}</strong><small>{provider?.configuration.label}</small></span>{#if selection.providerManaged}<Check size={14} />{/if}</button>{/if}
                {#each ranked as model}<button type="button" disabled={!provider || !providerAvailable(provider) || model.availability === "unavailable" || model.availability === "deprecated"} title={model.availability === "available" ? undefined : modelMetadata(model.contextLimit, model.availability).join(" · ")} onclick={() => chooseModel(model.id, false)}><span><strong>{model.displayName}</strong><small>{[model.id, ...modelMetadata(model.contextLimit, model.availability)].join(" · ")}</small></span>{#if provider?.configuration.favoriteModelIds.includes(model.id)}<Star size={12} />{/if}{#if selection.modelId === model.id}<Check size={14} />{/if}</button>{/each}
                {#if ranked.length === 0 && !providerManagedOnly}<p>{t("chat.composer.noModels")}</p>{/if}
              </div>
            {:else if flyout === "interaction"}
              <p class="flyout-heading">{t("chat.hero.interaction")}</p>
              <div class="selection-list"><button type="button" onclick={() => chooseInteraction("build")}><span><strong>{t("chat.hero.build")}</strong><small>{t("chat.composer.buildDescription")}</small></span>{#if chat.composer.interactionMode === "build"}<Check size={14} />{/if}</button><button type="button" disabled={!supportsPlan} onclick={() => chooseInteraction("plan")}><span><strong>{t("chat.hero.plan")}</strong><small>{supportsPlan ? t("chat.composer.planDescription") : t("chat.composer.planUnavailable")}</small></span>{#if chat.composer.interactionMode === "plan"}<Check size={14} />{/if}</button></div>
            {:else if optionViewDefinition}
              <p class="flyout-heading">{optionRole(optionViewDefinition) === "effort" ? t("chat.composer.effort") : optionRole(optionViewDefinition) === "speed" ? t("chat.composer.speed") : optionViewDefinition.label}</p>
              <div class="selection-list option-list">
                {#if optionViewDefinition.kind === "choice"}
                  {#each optionViewDefinition.options as choice}<button type="button" onclick={() => { updateOption(optionViewDefinition.key, { kind: "choice", value: choice.value }); closePicker(); }}><span><strong>{humanizeOptionLabel(choice.label)}</strong>{#if choice.description}<small>{choice.description}</small>{/if}</span>{#if choiceValue(optionViewDefinition.key) === choice.value}<Check size={14} />{/if}</button>{/each}
                {:else if optionViewDefinition.kind === "boolean"}
                  <button type="button" onclick={() => { updateOption(optionViewDefinition.key, { kind: "boolean", value: !booleanValue(optionViewDefinition.key) }); closePicker(); }}><span><strong>{optionViewDefinition.label}</strong>{#if optionViewDefinition.description}<small>{optionViewDefinition.description}</small>{/if}</span>{#if booleanValue(optionViewDefinition.key)}<Check size={14} />{/if}</button>
                {:else if optionViewDefinition.kind === "multiple_choice"}
                  {#each optionViewDefinition.options as choice}<label><input type="checkbox" checked={multipleIncludes(optionViewDefinition.key, choice.value)} onchange={(event) => toggleMultiple(optionViewDefinition.key, choice.value, event.currentTarget.checked)} /><span><strong>{choice.label}</strong>{#if choice.description}<small>{choice.description}</small>{/if}</span></label>{/each}
                {:else if optionViewDefinition.kind === "integer_range"}
                  <label class="range-option"><span>{formatNumber(localization.locale, integerValue(optionViewDefinition.key, optionViewDefinition.defaultValue ?? optionViewDefinition.minimum))}</span><input type="range" min={optionViewDefinition.minimum} max={optionViewDefinition.maximum} step={optionViewDefinition.step} value={integerValue(optionViewDefinition.key, optionViewDefinition.defaultValue ?? optionViewDefinition.minimum)} oninput={(event) => updateOption(optionViewDefinition.key, { kind: "integer", value: event.currentTarget.valueAsNumber })} /></label>
                {:else if optionViewDefinition.kind === "text"}
                  <label class="text-option"><span>{optionViewDefinition.label}</span><input type="text" value={textValue(optionViewDefinition.key)} oninput={(event) => updateOption(optionViewDefinition.key, { kind: "text", value: event.currentTarget.value })} /></label>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
    </div>
  {/if}
</div>

{#if pendingProviderId}
  <div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={() => { pendingProviderId = null; }}></button><div bind:this={providerForkDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="provider-fork-title" tabindex="-1"><h2 id="provider-fork-title" class="font-semibold">{t("chat.composer.changeProviderTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.changeProviderDescription")}</p><div class="mt-4 flex justify-end gap-2"><button type="button" class="chat-secondary-button" onclick={() => { pendingProviderId = null; }}>{t("chat.cancel")}</button><button type="button" class="chat-primary-button" onclick={() => void confirmProviderFork()}>{t("chat.composer.startProviderFork")}</button></div></div></div>
{/if}

<style>
  .model-control { position: relative; min-width: 0; flex: 0 1 auto; user-select: none; transition: width 280ms cubic-bezier(0.22, 0.75, 0.18, 1); will-change: width; }
  .model-trigger { display: flex; width: auto; height: 2rem; max-width: 17rem; align-items: center; justify-content: center; border-radius: 999px; background: transparent; padding: 0.3rem 0.55rem; color: var(--foreground); font-size: 0.766667rem; }
  .model-control.measured .model-trigger { width: 100%; max-width: none; }
  .model-trigger-content { display: inline-flex; min-width: 0; max-width: 100%; align-items: center; gap: 0.3rem; }
  .model-trigger:hover, .model-trigger[aria-expanded="true"] { background: color-mix(in srgb, var(--accent) 52%, transparent); }
  .model-trigger:focus-visible { outline: 1px solid color-mix(in srgb, var(--ring) 50%, transparent); outline-offset: 1px; }
  .model-trigger :global(svg) { flex: 0 0 auto; }
  .model-trigger :global(.model-chevron) { margin-left: 0.2rem; color: var(--muted-foreground); }
  .fast-indicator { display: grid; width: 0; flex: 0 0 auto; place-items: center; overflow: hidden; opacity: 0; transform: scale(0.72); transition: width 180ms cubic-bezier(0.2, 0.8, 0.2, 1), opacity 140ms ease, transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .fast-indicator.active { width: 0.9rem; opacity: 1; transform: scale(1); }
  .model-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .effort-name { flex: 0 0 auto; color: var(--primary); transition: color 260ms ease; }
  .effort-name.ultra { color: #7c3aed; }
  .model-popover { position: absolute; right: 0; bottom: calc(100% + 0.45rem); z-index: 45; width: min(18.5rem, calc(100vw - 1rem)); overflow: visible; border: 1px solid var(--border); border-radius: 0.8rem; background: var(--popover); padding: 0.65rem 0.6rem 0.5rem; color: var(--popover-foreground); box-shadow: 0 14px 38px rgb(0 0 0 / 0.16); }
  .picker-stage { position: relative; overflow: clip; transition: height 320ms cubic-bezier(0.22, 0.75, 0.18, 1); }
  .picker-view { width: 100%; opacity: 0; pointer-events: none; transition: opacity 190ms ease, transform 300ms cubic-bezier(0.22, 0.75, 0.18, 1); will-change: opacity, transform; }
  .picker-view:not(.active) { position: absolute; inset: 0 0 auto; }
  .overview-view { padding-top: 0.45rem; transform: translateY(-0.7rem) scale(0.99); }
  .advanced-view { transform: translateY(0.8rem) scale(0.99); }
  .picker-view.active { position: relative; opacity: 1; pointer-events: auto; transform: translateY(0) scale(1); transition-delay: 55ms, 0ms; }
  .effort-ladder { position: relative; height: var(--effort-track-height); overflow: visible; border: 1px solid color-mix(in srgb, var(--border) 88%, transparent); border-radius: 999px; background: #e4e5e9; cursor: default; touch-action: none; user-select: none; transition: border-color 260ms ease, background-color 260ms ease; }
  .effort-fill { position: absolute; inset-block: 0; left: 0; overflow: hidden; border-radius: inherit; background: #0c78d0; transition: width 240ms cubic-bezier(0.22, 0.75, 0.18, 1), background-color 280ms ease; }
  .effort-fill::before { position: absolute; content: ""; inset: 0; opacity: 0; background-image: linear-gradient(105deg, #6ea8ff 0%, #7457f5 48%, #a83cf2 100%); background-size: 180% 100%; animation: ultra-color-flow 3.6s ease-in-out infinite alternate; pointer-events: none; transition: opacity 300ms ease; }
  .effort-particles { position: absolute; inset: 0; opacity: 0; background-image: radial-gradient(circle at 23% 22%, rgb(255 255 255 / 0.78) 0 0.7px, transparent 1.45px), radial-gradient(circle at 68% 72%, rgb(255 255 255 / 0.58) 0 1px, transparent 1.85px), radial-gradient(circle at 37% 44%, rgb(255 255 255 / 0.88) 0 0.6px, transparent 1.3px), radial-gradient(circle at 79% 31%, rgb(255 255 255 / 0.64) 0 1.2px, transparent 2px), radial-gradient(circle at 14% 66%, rgb(255 255 255 / 0.7) 0 0.8px, transparent 1.55px), radial-gradient(circle at 53% 19%, rgb(255 255 255 / 0.54) 0 0.65px, transparent 1.35px), radial-gradient(circle at 86% 58%, rgb(255 255 255 / 0.82) 0 0.9px, transparent 1.7px), radial-gradient(circle at 42% 81%, rgb(255 255 255 / 0.62) 0 0.75px, transparent 1.45px); background-repeat: repeat-x; background-size: 83px 100%, 107px 100%, 131px 100%, 157px 100%, 191px 100%, 223px 100%, 269px 100%, 311px 100%; pointer-events: none; transition: opacity 240ms ease; }
  .effort-particles.calm { animation: galaxy-drift 7s linear infinite; }
  .effort-particles.rapid { animation: galaxy-stream 700ms linear infinite; }
  .effort-options { position: absolute; inset: 0; }
  .effort-options button { position: absolute; top: 50%; display: grid; width: 2.15rem; height: 2.15rem; place-items: center; transform: translate(-50%, -50%); border-radius: 999px; cursor: inherit; }
  .effort-options button:focus-visible { outline: 1px solid color-mix(in srgb, var(--ring) 52%, transparent); outline-offset: -2px; }
  .effort-options button i { width: 0.3rem; min-width: 0.3rem; max-width: 0.3rem; height: 0.3rem; min-height: 0.3rem; max-height: 0.3rem; flex: 0 0 0.3rem; border-radius: 999px; background: color-mix(in srgb, var(--muted-foreground) 58%, #e4e5e9); opacity: 1; transform: scale(1); transition: opacity 180ms ease 70ms, background-color 240ms ease, transform 180ms ease 70ms; }
  .effort-options button:hover i { transform: scale(1.4); transition-delay: 0ms; }
  .effort-options button[aria-pressed="true"] i, .effort-options button:has(~ button[aria-pressed="true"]) i { background: color-mix(in srgb, white 48%, #0c78d0); }
  .effort-knob { position: absolute; top: 50%; width: 2.15rem; height: 2.15rem; transform: translate(-50%, -50%) scale(1); border: 1px solid color-mix(in srgb, var(--foreground) 10%, transparent); border-radius: 999px; background: #ffffff; box-shadow: 0 1px 4px rgb(0 0 0 / 0.14); pointer-events: none; transition: left 240ms cubic-bezier(0.22, 0.75, 0.18, 1), transform 170ms cubic-bezier(0.2, 0.8, 0.2, 1), border-color 260ms ease, background-color 260ms ease, box-shadow 260ms ease; }
  .effort-options button[aria-pressed="true"]:focus-visible ~ .effort-knob, .effort-ladder:is(.holding, .handle-hovered) .effort-knob { transform: translate(-50%, -50%) scale(1.065); box-shadow: 0 2px 7px rgb(0 0 0 / 0.18); }
  .effort-ladder.ultra { border-color: color-mix(in srgb, #8b5cf6 30%, transparent); background: #ddd6fe; }
  .effort-ladder.ultra .effort-fill::before, .effort-ladder.ultra:not(.fast) .effort-particles.calm, .effort-ladder.fast .effort-particles.rapid { opacity: 1; }
  .effort-ladder.fast .effort-options button:has(~ button[aria-pressed="true"]) i, .effort-ladder.ultra .effort-options button:has(~ button[aria-pressed="true"]) i { opacity: 0; transform: scale(0.65); transition-delay: 0ms; }
  :global(.dark) .effort-name.ultra { color: #b794ff; }
  :global(.dark) .effort-ladder { border-color: rgb(255 255 255 / 0.09); background: #45464a; }
  :global(.dark) .effort-fill { background: #1681dc; }
  :global(.dark) .effort-options button i { background: color-mix(in srgb, white 32%, #45464a); }
  :global(.dark) .effort-options button[aria-pressed="true"] i, :global(.dark) .effort-options button:has(~ button[aria-pressed="true"]) i { background: color-mix(in srgb, white 46%, #1681dc); }
  :global(.dark) .effort-knob { border-color: rgb(255 255 255 / 0.1); background: #f4f4f6; box-shadow: 0 1px 5px rgb(0 0 0 / 0.28); }
  :global(.dark) .effort-ladder.ultra { border-color: rgb(139 92 246 / 0.4); background: #4c3b78; }
  :global(.dark) .effort-fill::before { background-image: linear-gradient(105deg, #476cf4 0%, #7144e8 48%, #9f35d7 100%); }
  .effort-footer { display: grid; margin-top: 0.4rem; }
  .quick-actions, .effort-guidance { grid-area: 1 / 1; min-height: 2rem; }
  .quick-actions { display: flex; align-items: center; justify-content: space-between; gap: 0.25rem; opacity: 1; transform: translateY(0); transition: opacity 150ms ease, transform 190ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .effort-guidance { display: flex; align-items: center; justify-content: space-between; padding-inline: 0.15rem; color: var(--muted-foreground); opacity: 0; pointer-events: none; transform: translateY(0.28rem); transition: opacity 170ms ease, transform 210ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .effort-footer.holding .quick-actions { opacity: 0; pointer-events: none; transform: translateY(0.22rem); }
  .effort-footer.holding .effort-guidance { opacity: 1; transform: translateY(0); }
  .advanced-toggle, .advanced-heading { display: flex; min-height: 2rem; align-items: center; gap: 0.35rem; border-radius: 0.5rem; padding: 0.35rem 0.5rem; color: var(--muted-foreground); text-align: left; }
  .advanced-toggle { width: auto; min-width: 0; flex: 0 0 auto; gap: 0.2rem; padding: 0.3rem 0.2rem; }
  .advanced-toggle:hover, .advanced-heading:hover { background: color-mix(in srgb, var(--accent) 65%, transparent); color: var(--foreground); }
  .advanced-heading span { flex: 1; }
  .advanced-heading { width: 100%; border-bottom: 1px solid var(--border); border-radius: 0.55rem 0.55rem 0 0; }
  .fast-button { display: flex; width: 2rem; height: 2rem; flex: 0 0 auto; align-items: center; justify-content: flex-end; border-radius: 0.55rem; color: var(--muted-foreground); transition: color 240ms ease, background-color 240ms ease, transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .fast-button :global(svg) { fill: transparent; transform: scale(0.94); transition: fill 200ms ease, transform 220ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .fast-button.active :global(svg) { fill: currentColor; transform: scale(1); }
  .fast-button:hover { background: color-mix(in srgb, var(--accent) 65%, transparent); color: var(--foreground); }
  .fast-button.active { background: rgb(12 120 208 / 0.1); color: #0879d8; }
  .fast-button.active.ultra { background: rgb(124 58 237 / 0.1); color: #7c3aed; }
  .fast-button:active { transform: scale(0.92); }
  :global(.dark) .fast-button.active { background: rgb(22 129 220 / 0.16); color: #3b9aeb; }
  :global(.dark) .fast-button.active.ultra { background: rgb(167 139 250 / 0.15); color: #b794ff; }
  .advanced-list { padding-top: 0.3rem; }
  .advanced-list button { display: grid; width: 100%; grid-template-columns: minmax(0, 1fr) minmax(0, auto) 1rem; align-items: center; gap: 0.5rem; border-radius: 0.55rem; padding: 0.5rem 0.6rem; text-align: left; }
  .advanced-list button:hover, .advanced-list button:focus-visible { background: color-mix(in srgb, var(--accent) 70%, transparent); outline: none; }
  .advanced-list small { overflow: hidden; max-width: 9rem; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); }
  .model-flyout { position: absolute; right: calc(100% + 0.4rem); bottom: 0; width: min(22rem, calc(100vw - 1rem)); max-height: min(28rem, 72vh); overflow: hidden auto; border: 1px solid var(--border); border-radius: 0.9rem; background: var(--popover); padding: 0.45rem; box-shadow: 0 18px 48px rgb(0 0 0 / 0.18); }
  .flyout-heading { padding: 0.35rem 0.5rem 0.45rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .selection-list { padding-top: 0.3rem; }
  .selection-list > button { display: grid; width: 100%; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 0.45rem; border-radius: 0.55rem; padding: 0.5rem 0.55rem; text-align: left; }
  .selection-list > button:has(> :global(svg:first-child)) { grid-template-columns: 1.3rem minmax(0, 1fr) auto; }
  .selection-list > button:hover, .selection-list > button:focus-visible { background: var(--accent); outline: none; }
  .selection-list > button:disabled, .selection-list > button.unavailable { opacity: 0.55; }
  .selection-list > button > span { min-width: 0; }
  .selection-list strong, .selection-list small { display: block; }
  .selection-list strong { font-size: 0.766667rem; font-weight: 500; }
  .selection-list small { overflow: hidden; margin-top: 0.08rem; text-overflow: ellipsis; white-space: nowrap; color: var(--muted-foreground); font-size: 0.666667rem; }
  .selection-list > p { padding: 0.8rem; color: var(--muted-foreground); text-align: center; font-size: 0.7rem; }
  .model-search { display: flex; align-items: center; gap: 0.4rem; margin: 0.45rem 0.15rem 0.15rem; border-radius: 0.55rem; background: var(--muted); padding: 0.4rem 0.5rem; color: var(--muted-foreground); }
  .model-search:focus-within { color: var(--foreground); }
  .model-search input { min-width: 0; flex: 1; user-select: text; background: transparent; color: var(--foreground); font-size: 0.733333rem; outline: none; }
  .model-list { max-height: min(19rem, 52vh); overflow-y: auto; }
  .option-list > label { display: flex; align-items: center; gap: 0.55rem; border-radius: 0.5rem; padding: 0.5rem; }
  .option-list > label:hover { background: var(--accent); }
  .option-list > label > span { min-width: 0; flex: 1; }
  .range-option, .text-option { flex-direction: column; align-items: stretch !important; }
  .range-option input, .text-option input { width: 100%; }
  .text-option input { user-select: text; border-radius: 0.45rem; background: var(--muted); padding: 0.4rem 0.5rem; color: var(--foreground); outline: none; }
  @keyframes ultra-color-flow { from { background-position: 0 0; } to { background-position: 100% 0; } }
  @keyframes galaxy-drift { from { background-position: 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0; } to { background-position: -83px 0, -107px 0, -131px 0, -157px 0, -191px 0, -223px 0, -269px 0, -311px 0; } }
  @keyframes galaxy-stream { from { background-position: 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0; } to { background-position: -83px 0, -107px 0, -131px 0, -157px 0, -191px 0, -223px 0, -269px 0, -311px 0; } }
  @container chat-composer (max-width: 680px) { .model-flyout { right: 0; bottom: calc(100% + 0.4rem); } }
  @container chat-composer (max-width: 460px) { .model-trigger { max-width: 11rem; } .effort-name { display: none; } }
  @container chat-composer (max-width: 330px) { .model-trigger { max-width: 7.5rem; padding-inline: 0.45rem; } }
  @media (prefers-reduced-motion: reduce) { .model-control, .picker-stage, .picker-view, .quick-actions, .effort-guidance { transition-duration: 0.01ms; } .effort-fill::before, .effort-particles { animation: none; background-position: 50% 0; } }
</style>
