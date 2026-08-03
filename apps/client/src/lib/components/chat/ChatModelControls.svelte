<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Plus from "@lucide/svelte/icons/plus";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import Zap from "@lucide/svelte/icons/zap";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import type {
    ModelOptionSelection,
    ModelOptionValue,
    ProviderInstanceRead,
    ProviderModel,
  } from "$lib/chat/contracts";
  import {
    composerModelSelection,
    defaultModelOptions,
    providerAvailable,
    readComposerModelSelection,
  } from "$lib/chat/composer-model";
  import { modelCompany } from "$lib/chat/model-company";
  import {
    buildFavoriteModelEntries,
    buildModelCompanySections,
    buildQuickEffortChoices,
    isKnownModelOption,
    modelEffortStopPosition,
    modelOptionRole,
    visibleProviderModels,
    type KnownModelOption,
    type QuickEffortChoice,
  } from "$lib/chat/model-picker-model";
  import * as chatApi from "$lib/api/chat";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { trapTabFocus } from "$lib/chat/focus-navigation";
  import { isPointerAimingAtSubmenu, type MenuAimPoint, type MenuAimSide } from "$lib/projects/menu-aim";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatProviderIcon from "./ChatProviderIcon.svelte";
  import ChatProviderForkDialog from "./ChatProviderForkDialog.svelte";

  interface PendingProviderModelSelection {
    providerInstanceId: string;
    modelId: string | null;
    providerManaged: boolean;
  }
  interface FlyoutPosition {
    left: number;
    top: number;
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
  let modelToggle: HTMLButtonElement | undefined = $state();
  let flyoutPanel: HTMLDivElement | undefined = $state();
  let modelControlWidth = $state<number | null>(null);
  let effortPressing = $state(false);
  let effortDragging = $state(false);
  let effortHandleHovered = $state(false);
  let modelSearch: HTMLInputElement | undefined = $state();
  let modelListElement: HTMLDivElement | undefined = $state();
  let modelListContentElement: HTMLDivElement | undefined = $state();
  let modelListScrollable = $state(false);
  let modelListCanScrollUp = $state(false);
  let modelListCanScrollDown = $state(false);
  let modelListScrollFrame: number | null = null;
  let providerForkDialog: HTMLElement | undefined = $state();
  let pendingProviderModel = $state<PendingProviderModelSelection | null>(null);
  let modelQuery = $state("");
  let modelPickerError = $state<string | null>(null);
  let collapsedModelSections = $state<Set<string>>(new Set());
  let flyout = $state<"models" | null>(null);
  let flyoutPosition = $state<FlyoutPosition | null>(null);
  let quickAnchorModelId = $state<string | null>(null);
  let activeFlyoutTrigger: HTMLElement | null = null;
  let modelControlResetTimer: ReturnType<typeof setTimeout> | null = null;
  let modelFlyoutCloseTimer: ReturnType<typeof setTimeout> | null = null;
  let modelFlyoutAimOrigin: MenuAimPoint | null = null;
  let modelFlyoutSide: Extract<MenuAimSide, "left" | "right"> = "left";
  let effortClickResetTimer: ReturnType<typeof setTimeout> | null = null;
  let effortPointerId: number | null = null;
  let effortPointerOrigin: { x: number; y: number } | null = null;
  let effortLastDragIndex = -1;
  let suppressEffortPointerClick = false;
  const flyoutGapPx = 6;
  const flyoutViewportInsetPx = 8;
  const modelFlyoutCloseDelayMs = 240;
  const modelFlyoutExitDelayMs = 120;
  const modelControlResizeMs = 280;
  const effortDragThresholdPx = 5;
  const effortTrackHeightRem = 1.75;
  const effortEndpointInsetRem = effortTrackHeightRem / 2;
  const dummyEffortStops = [0, 1, 2, 3, 4, 5] as const;
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const provider = $derived(providers.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const unconfiguredFamilies = $derived((chat.settings?.providerFamilies ?? []).filter((family) => !providers.some((entry) => entry.configuration.familyId === family.familyId)));
  const selection = $derived(readComposerModelSelection(chat.composer.modelSelection));
  const catalogSelection = $derived({
    providerInstanceId: provider?.configuration.instanceId ?? null,
    modelId: selection.modelId,
    providerManaged: selection.providerManaged,
  });
  const models = $derived(visibleProviderModels(provider, catalogSelection));
  const favoriteModelEntries = $derived.by(() => buildFavoriteModelEntries(
    providers,
    chat.settings?.configuration.rememberedSelections ?? [],
    catalogSelection,
    modelQuery,
  ));
  const modelCompanySections = $derived.by(() => buildModelCompanySections(
    providers,
    unconfiguredFamilies,
    catalogSelection,
    modelQuery,
    t("chat.composer.providerManagedModel"),
    localization.locale,
  ));
  const selectedModel = $derived(models.find((model) => model.id === selection.modelId) ?? null);
  const quickAnchorModel = $derived(models.find((model) => model.id === quickAnchorModelId) ?? selectedModel);
  const knownOptions = $derived((selectedModel?.options ?? []).filter(isKnownModelOption));
  const effortDefinition = $derived(knownOptions.find((definition) => modelOptionRole(definition) === "effort") ?? null);
  const speedDefinition = $derived.by(() => {
    const candidates = [selectedModel, quickAnchorModel, ...models];
    const visited = new Set<string>();
    for (const candidate of candidates) {
      if (!candidate || visited.has(candidate.id)) continue;
      visited.add(candidate.id);
      for (const definition of candidate.options) {
        if (isKnownModelOption(definition) && modelOptionRole(definition) === "speed") return definition;
      }
    }
    return null;
  });
  const quickEffortChoices = $derived(buildQuickEffortChoices(
    quickAnchorModel,
    models,
    provider?.configuration.familyId ?? null,
    displayModelName,
    humanizeOptionLabel,
  ));
  const selectedQuickEffortIndex = $derived(quickEffortChoices.findIndex((choice) => choice.modelId === selection.modelId && choice.effortValue === selectedEffortValue()));

  onDestroy(() => {
    cancelModelControlReset();
    cancelModelFlyoutClose();
    cancelEffortClickReset();
  });

  $effect(() => {
    if (!pickerOpen && pickerWasOpen) queueMicrotask(() => pickerTrigger?.focus());
    pickerWasOpen = pickerOpen;
  });

  $effect(() => {
    if (!flyout || !flyoutPanel) return;
    const reposition = () => positionFlyout();
    reposition();
    window.addEventListener("resize", reposition);
    if (typeof ResizeObserver === "undefined") {
      return () => window.removeEventListener("resize", reposition);
    }
    const observer = new ResizeObserver(reposition);
    observer.observe(flyoutPanel);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", reposition);
    };
  });

  $effect(() => {
    const element = modelListElement;
    if (!element) return;
    const resizeObserver = new ResizeObserver(requestModelListScrollStateRefresh);
    resizeObserver.observe(element);
    if (modelListContentElement) resizeObserver.observe(modelListContentElement);
    requestModelListScrollStateRefresh();
    return () => {
      resizeObserver.disconnect();
      if (modelListScrollFrame !== null) {
        cancelAnimationFrame(modelListScrollFrame);
        modelListScrollFrame = null;
      }
    };
  });

  $effect(() => {
    if (!pickerOpen) return;
    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (!(event.target instanceof Node)) return;
      if (pickerRoot?.contains(event.target) || providerForkDialog?.contains(event.target)) return;
      closePicker();
    };
    window.addEventListener("pointerdown", closeOnOutsidePointer, true);
    return () => window.removeEventListener("pointerdown", closeOnOutsidePointer, true);
  });

  function togglePicker(): void {
    if (pickerOpen) {
      closePicker();
      return;
    }
    const anchoredChoices = buildQuickEffortChoices(
      models.find((model) => model.id === quickAnchorModelId) ?? null,
      models,
      provider?.configuration.familyId ?? null,
      displayModelName,
      humanizeOptionLabel,
    );
    if (!anchoredChoices.some((choice) => choice.modelId === selection.modelId && choice.effortValue === selectedEffortValue())) {
      quickAnchorModelId = selectedModel?.id ?? null;
    }
    flyout = provider ? null : "models";
    openPicker();
  }

  function closePicker(): void {
    pickerOpen = false;
    modelQuery = "";
    modelPickerError = null;
    collapsedModelSections = new Set();
    cancelEffortClickReset();
    suppressEffortPointerClick = false;
    effortHandleHovered = false;
    resetEffortPointerState();
    closeFlyout();
    void tick().then(collapseModelControl);
  }

  function modelSectionCollapsed(sectionId: string): boolean {
    return collapsedModelSections.has(sectionId);
  }

  function toggleModelSection(sectionId: string): void {
    const next = new Set(collapsedModelSections);
    if (next.has(sectionId)) next.delete(sectionId);
    else next.add(sectionId);
    collapsedModelSections = next;
  }

  function refreshModelListScrollState(): void {
    modelListScrollFrame = null;
    const element = modelListElement;
    if (!element) {
      modelListScrollable = false;
      modelListCanScrollUp = false;
      modelListCanScrollDown = false;
      return;
    }
    const maxScrollTop = element.scrollHeight - element.clientHeight;
    modelListScrollable = maxScrollTop > 1;
    modelListCanScrollUp = element.scrollTop > 1;
    modelListCanScrollDown = element.scrollTop < maxScrollTop - 1;
  }

  function requestModelListScrollStateRefresh(): void {
    if (modelListScrollFrame !== null) cancelAnimationFrame(modelListScrollFrame);
    modelListScrollFrame = requestAnimationFrame(refreshModelListScrollState);
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
      if (flyout === "models" && modelToggle) {
        activeFlyoutTrigger = modelToggle;
        positionFlyout();
        modelSearch?.focus();
      }
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

  function probeStatus(entry: (typeof providers)[number]): string {
    if (!entry.configuration.enabled) return t("chat.composer.providerDisabled");
    if (!entry.lastProbe) return t("chat.composer.providerNotChecked");
    if (entry.lastProbe.state === "healthy") return t("chat.status.idle");
    return entry.lastProbe.detail ?? t("chat.status.providerUnavailable");
  }

  async function toggleModelFavorite(entry: ProviderInstanceRead, modelId: string): Promise<void> {
    const favorites = entry.configuration.favoriteModelIds.includes(modelId)
      ? entry.configuration.favoriteModelIds.filter((id) => id !== modelId)
      : [...entry.configuration.favoriteModelIds, modelId];
    modelPickerError = null;
    try {
      await chat.updateModels(entry.configuration.instanceId, entry.configuration.visibleModelIds, favorites);
    } catch (error: unknown) {
      modelPickerError = error instanceof Error ? error.message : String(error);
    }
  }

  function openProviderSettings(): void {
    closePicker();
    settings.open("chat", { chatSubsection: "providers" });
  }

  function chooseModel(entry: (typeof providers)[number], modelId: string | null, providerManaged: boolean): void {
    const target = { providerInstanceId: entry.configuration.instanceId, modelId, providerManaged };
    closeFlyout();
    if (chat.selectedThread && entry.configuration.instanceId !== chat.selectedThread.providerInstanceId) {
      pendingProviderModel = target;
      return;
    }
    applyProviderModelSelection(entry, modelId, providerManaged);
  }

  function applyProviderModelSelection(
    entry: (typeof providers)[number],
    modelId: string | null,
    providerManaged: boolean,
  ): void {
    const model = visibleProviderModels(entry, catalogSelection).find((candidate) => candidate.id === modelId);
    if (entry.configuration.instanceId !== provider?.configuration.instanceId) {
      chat.setComposerProvider(entry.configuration.instanceId);
    }
    chat.setComposerModel(composerModelSelection(modelId, providerManaged, model ? defaultModelOptions(model.options) : []));
    quickAnchorModelId = modelId;
  }

  function chooseQuickEffort(choice: QuickEffortChoice): void {
    const model = models.find((candidate) => candidate.id === choice.modelId);
    if (!model) return;
    const options = choice.modelId === selection.modelId
      ? selection.options.filter((entry) => entry.key !== choice.effortKey)
      : defaultModelOptions(model.options).filter((entry) => entry.key !== choice.effortKey);
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

  function openModelFlyout(trigger?: HTMLElement): void {
    cancelModelFlyoutClose();
    if (trigger) activeFlyoutTrigger = trigger;
    if (flyout !== "models") {
      modelQuery = "";
      modelPickerError = null;
    }
    flyout = "models";
    scheduleFlyoutPosition();
  }

  function closeFlyout(): void {
    cancelModelFlyoutClose();
    flyout = null;
    flyoutPosition = null;
    activeFlyoutTrigger = null;
  }

  function pointerPoint(event: PointerEvent): MenuAimPoint {
    return { x: event.clientX, y: event.clientY };
  }

  function cancelModelFlyoutClose(): void {
    if (modelFlyoutCloseTimer !== null) {
      clearTimeout(modelFlyoutCloseTimer);
      modelFlyoutCloseTimer = null;
    }
    modelFlyoutAimOrigin = null;
    if (typeof window !== "undefined") {
      window.removeEventListener("pointermove", handleModelFlyoutAim, true);
    }
  }

  function armModelFlyoutClose(delayMs = modelFlyoutCloseDelayMs): void {
    if (modelFlyoutCloseTimer !== null) clearTimeout(modelFlyoutCloseTimer);
    modelFlyoutCloseTimer = setTimeout(() => {
      modelFlyoutCloseTimer = null;
      closeFlyout();
    }, delayMs);
  }

  function scheduleModelFlyoutClose(event: PointerEvent): void {
    const relatedTarget = event.relatedTarget;
    if (
      relatedTarget instanceof Node
      && (modelToggle?.contains(relatedTarget) || flyoutPanel?.contains(relatedTarget))
    ) {
      cancelModelFlyoutClose();
      return;
    }
    if (!flyoutPanel) {
      closeFlyout();
      return;
    }
    modelFlyoutAimOrigin = pointerPoint(event);
    window.removeEventListener("pointermove", handleModelFlyoutAim, true);
    window.addEventListener("pointermove", handleModelFlyoutAim, true);
    armModelFlyoutClose();
  }

  function handleModelFlyoutAim(event: PointerEvent): void {
    const target = event.target;
    if (
      target instanceof Node
      && (modelToggle?.contains(target) || flyoutPanel?.contains(target))
    ) {
      cancelModelFlyoutClose();
      return;
    }
    const origin = modelFlyoutAimOrigin;
    const panel = flyoutPanel;
    if (!origin || !panel) {
      closeFlyout();
      return;
    }
    const rect = panel.getBoundingClientRect();
    const aiming = isPointerAimingAtSubmenu({
      origin,
      point: pointerPoint(event),
      submenu: { left: rect.left, right: rect.right, top: rect.top, bottom: rect.bottom },
      side: modelFlyoutSide,
      minTowardDistance: 0,
    });
    if (!aiming) {
      closeFlyout();
      return;
    }
    armModelFlyoutClose();
  }

  function handleModelFlyoutPointerLeave(event: PointerEvent): void {
    const relatedTarget = event.relatedTarget;
    if (relatedTarget instanceof Node && modelToggle?.contains(relatedTarget)) {
      cancelModelFlyoutClose();
      return;
    }
    cancelModelFlyoutClose();
    armModelFlyoutClose(modelFlyoutExitDelayMs);
  }

  function handleModelFlyoutFocusOut(): void {
    queueMicrotask(() => {
      const focused = document.activeElement;
      if (
        focused instanceof Node
        && (modelToggle?.contains(focused) || flyoutPanel?.contains(focused))
      ) {
        return;
      }
      closeFlyout();
    });
  }

  function scheduleFlyoutPosition(): void {
    void tick().then(positionFlyout);
  }

  function positionFlyout(): void {
    if (!activeFlyoutTrigger || !flyoutPanel || !pickerPanel) return;
    const anchor = activeFlyoutTrigger.getBoundingClientRect();
    const parent = pickerPanel.getBoundingClientRect();
    const panel = flyoutPanel.getBoundingClientRect();
    const rightPosition = anchor.right + flyoutGapPx;
    const leftPosition = anchor.left - panel.width - flyoutGapPx;
    const rightFits = rightPosition + panel.width <= window.innerWidth - flyoutViewportInsetPx;
    const leftFits = leftPosition >= flyoutViewportInsetPx;
    const maximumLeft = Math.max(flyoutViewportInsetPx, window.innerWidth - panel.width - flyoutViewportInsetPx);
    const viewportLeft = leftFits
      ? leftPosition
      : rightFits
        ? rightPosition
        : Math.min(Math.max(flyoutViewportInsetPx, leftPosition), maximumLeft);
    modelFlyoutSide = leftFits || (!rightFits && viewportLeft < anchor.left) ? "left" : "right";
    const maximumTop = Math.max(flyoutViewportInsetPx, window.innerHeight - panel.height - flyoutViewportInsetPx);
    const viewportTop = Math.min(Math.max(flyoutViewportInsetPx, anchor.top - 4), maximumTop);
    flyoutPosition = {
      left: Math.max(flyoutViewportInsetPx, viewportLeft) - parent.left,
      top: viewportTop - parent.top,
    };
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

  function modelMetadata(contextLimit: number | null, availability: string): string[] {
    const values: string[] = [];
    if (contextLimit !== null) values.push(t("chat.composer.modelContext", formatNumber(localization.locale, contextLimit)));
    if (availability === "stale") values.push(t("chat.composer.modelStale"));
    if (availability === "unavailable") values.push(t("chat.composer.modelUnavailable"));
    return values;
  }

  function handlePickerKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      closePicker();
      return;
    }
    trapTabFocus(event);
  }

  async function confirmProviderFork(): Promise<void> {
    if (!pendingProviderModel) return;
    const target = pendingProviderModel;
    pendingProviderModel = null;
    await chat.forkComposerWithProvider(target.providerInstanceId);
    const entry = providers.find((candidate) => candidate.configuration.instanceId === target.providerInstanceId);
    if (entry) applyProviderModelSelection(entry, target.modelId, target.providerManaged);
  }
</script>

<div bind:this={pickerRoot} class="model-control" class:measured={modelControlWidth !== null} style:width={modelControlWidth === null ? undefined : `${modelControlWidth}px`}>
  <button bind:this={pickerTrigger} type="button" class="model-trigger" data-chat-model-trigger aria-expanded={pickerOpen} disabled={chat.composer.loading} onclick={togglePicker}>
    <span bind:this={pickerTriggerContent} class="model-trigger-content">
      <span class="fast-indicator" class:active={isFastSelected()} aria-hidden="true">{#if chat.providerDiscoveryLoading && !provider}<LoaderCircle size={13} class="animate-spin" />{:else}<Zap size={13} fill="currentColor" />{/if}</span>
      <span class="model-name">{chat.providerDiscoveryLoading && !provider ? t("chat.composer.detectingProviders") : selection.providerManaged ? provider?.configuration.label ?? t("chat.composer.providerManagedModel") : displayModelName(selectedModel)}</span>
      {#if selectedOptionLabel(effortDefinition)}<span class="effort-name" class:ultra={isUltraSelected()}>{selectedOptionLabel(effortDefinition)}</span>{/if}
      <ChevronDown size={13} class="model-chevron" />
    </span>
  </button>

  {#if pickerOpen}
    <div bind:this={pickerPanel} class="model-popover" role="dialog" aria-label={t("chat.hero.model")} tabindex="-1" onkeydown={handlePickerKeydown}>
      <div class="effort-footer" class:holding={effortDragging}>
        <div class="quick-actions" inert={effortDragging} aria-hidden={effortDragging}>
          <button bind:this={modelToggle} type="button" class="model-picker-toggle" aria-expanded={flyout === "models"} onpointerenter={(event) => openModelFlyout(event.currentTarget)} onpointerleave={scheduleModelFlyoutClose} onfocus={(event) => openModelFlyout(event.currentTarget)} onfocusout={handleModelFlyoutFocusOut} onclick={(event) => openModelFlyout(event.currentTarget)}><span>{t("chat.hero.model")}</span><ChevronRight size={15} strokeWidth={1} /></button>
          {#if speedDefinition}<button type="button" class="fast-button" class:active={isFastSelected()} class:ultra={isUltraSelected()} title={isFastSelected() ? t("chat.composer.fastEnabled") : t("chat.composer.enableFast")} aria-label={isFastSelected() ? t("chat.composer.fastEnabled") : t("chat.composer.enableFast")} aria-pressed={isFastSelected()} onclick={() => setFastMode(!isFastSelected())}><Zap size={16} strokeWidth={1} /></button>
          {:else if !provider}<button type="button" class="fast-button dummy" title={t("chat.composer.providerRequiredForModelOptions")} aria-label={t("chat.composer.providerRequiredForModelOptions")} disabled><Zap size={16} strokeWidth={1} /></button>{/if}
        </div>
        <div class="effort-guidance" aria-hidden="true"><span>{t("chat.composer.faster")}</span><span>{t("chat.composer.smarter")}</span></div>
      </div>

        {#if quickEffortChoices.length > 0}
          <div class="effort-ladder" class:fast={isFastSelected()} class:ultra={isUltraSelected()} class:holding={effortPressing} class:handle-hovered={effortHandleHovered} style={`--effort-track-height:${effortTrackHeightRem}rem`} role="group" aria-label={t("chat.composer.quickModelEffort")} onpointerdown={handleEffortPointerDown} onpointermove={handleEffortPointerMove} onpointerleave={handleEffortPointerLeave} onpointerup={(event) => finishEffortPointer(event, true)} onpointercancel={(event) => finishEffortPointer(event, false)} onlostpointercapture={(event) => finishEffortPointer(event, false)}>
            <span class="effort-fill" style={`width:${selectedQuickEffortIndex < 0 ? "0" : isUltraSelected() ? "100%" : modelEffortStopPosition(selectedQuickEffortIndex, quickEffortChoices.length, effortEndpointInsetRem)}`} aria-hidden="true">
              <span class="effort-particles calm"></span>
              <span class="effort-particles rapid"></span>
            </span>
            <span class="effort-options">
              {#each quickEffortChoices as choice, index}
                <button type="button" style={`left:${modelEffortStopPosition(index, quickEffortChoices.length, effortEndpointInsetRem)}`} aria-label={`${choice.modelName} ${choice.effortLabel}`} aria-pressed={index === selectedQuickEffortIndex} data-app-tooltip-disabled="true" data-app-tooltip-focus-disabled="true" onclick={(event) => handleEffortChoiceClick(choice, event)}><i></i></button>
              {/each}
              {#if selectedQuickEffortIndex >= 0}<span class="effort-knob" style={`left:${modelEffortStopPosition(selectedQuickEffortIndex, quickEffortChoices.length, effortEndpointInsetRem)}`} aria-hidden="true"></span>{/if}
            </span>
          </div>
        {:else if !provider}
          <div class="effort-ladder dummy" style={`--effort-track-height:${effortTrackHeightRem}rem`} role="group" aria-label={t("chat.composer.quickModelEffort")} aria-disabled="true">
            <span class="effort-options">
              {#each dummyEffortStops as stop}
                <button type="button" style={`left:${modelEffortStopPosition(stop, dummyEffortStops.length, effortEndpointInsetRem)}`} aria-label={t("chat.composer.providerRequiredForModelOptions")} disabled tabindex="-1"><i></i></button>
              {/each}
              <span class="effort-knob" style="left:50%" aria-hidden="true"></span>
            </span>
          </div>
        {/if}

        {#if flyout === "models"}
          <div bind:this={flyoutPanel} class="model-flyout model-picker-flyout" class:positioned={flyoutPosition !== null} style:left={flyoutPosition === null ? undefined : `${flyoutPosition.left}px`} style:top={flyoutPosition === null ? undefined : `${flyoutPosition.top}px`} role="group" aria-label={t("chat.hero.model")} onpointerenter={cancelModelFlyoutClose} onpointerleave={handleModelFlyoutPointerLeave} onfocusout={handleModelFlyoutFocusOut}>
              <div class="model-picker-shell">
                <div class="model-picker-main">
                  <div class="model-list-frame">
                  <div
                    bind:this={modelListElement}
                    class="selection-list model-list hide-scrollbar"
                    class:model-list-scroll-both={modelListScrollable && modelListCanScrollUp && modelListCanScrollDown}
                    class:model-list-scroll-top={modelListScrollable && modelListCanScrollUp && !modelListCanScrollDown}
                    class:model-list-scroll-bottom={modelListScrollable && !modelListCanScrollUp && modelListCanScrollDown}
                    onscroll={refreshModelListScrollState}
                  >
                    <div bind:this={modelListContentElement} class="model-list-content">
                    <div class="model-search-row">
                      <label class="model-search"><Search size={16} /><input bind:this={modelSearch} bind:value={modelQuery} placeholder={t("chat.composer.modelSearch")} /></label>
                    </div>
                    <section class="model-company-section favorite-company-section" aria-labelledby="favorite-models-heading">
                      <button id="favorite-models-heading" type="button" class="model-company-heading" aria-expanded={!modelSectionCollapsed("favorites")} aria-controls="favorite-models-content" onclick={() => toggleModelSection("favorites")}><Star size={14} fill="currentColor" /><span>{t("chat.composer.favorites")}</span><ChevronDown size={13} class={modelSectionCollapsed("favorites") ? "collapsed" : undefined} /></button>
                      <div id="favorite-models-content" class="model-company-content" class:collapsed={modelSectionCollapsed("favorites")} inert={modelSectionCollapsed("favorites")} aria-hidden={modelSectionCollapsed("favorites")}>
                        <div class="model-company-content-inner">
                          {#each favoriteModelEntries as favorite (`favorite:${favorite.provider.configuration.instanceId}:${favorite.model.id}`)}
                            {@const metadata = modelMetadata(favorite.model.contextLimit, favorite.model.availability)}
                            {@const company = modelCompany(favorite.provider.configuration.familyId, favorite.model)}
                            <div class="model-row">
                              <button type="button" class="model-choice" disabled={!providerAvailable(favorite.provider) || favorite.model.availability === "unavailable"} title={favorite.model.availability === "available" ? undefined : metadata.join(" · ")} onclick={() => chooseModel(favorite.provider, favorite.model.id, false)}>
                                <span class="favorite-model-label"><ChatProviderIcon familyId={company.iconFamilyId} label={company.name} monochrome size={14} /><strong>{favorite.model.displayName}</strong></span>
                                {#if favorite.provider.configuration.instanceId === provider?.configuration.instanceId && selection.modelId === favorite.model.id}<Check size={15} />{/if}
                              </button>
                              <button type="button" class="model-favorite active" aria-label={`${t("chat.composer.favorite")}: ${favorite.model.displayName}`} aria-pressed="true" onclick={() => void toggleModelFavorite(favorite.provider, favorite.model.id)}><Star size={16} fill="currentColor" /></button>
                            </div>
                          {/each}
                          {#if favoriteModelEntries.length === 0}<p>{modelQuery ? t("chat.composer.noFavoriteMatches") : t("chat.composer.noFavoriteModels")}</p>{/if}
                        </div>
                      </div>
                    </section>

                    {#each modelCompanySections as section (section.company.id)}
                      {@const sectionCollapsed = modelSectionCollapsed(section.company.id)}
                      <section class="model-company-section" data-model-company={section.company.id} aria-labelledby={`model-company-${section.company.id}`}>
                        <button id={`model-company-${section.company.id}`} type="button" class="model-company-heading" aria-expanded={!sectionCollapsed} aria-controls={`model-company-${section.company.id}-content`} onclick={() => toggleModelSection(section.company.id)}><ChatProviderIcon familyId={section.company.iconFamilyId} label={section.company.name} monochrome size={14} /><span>{section.company.name}</span><ChevronDown size={13} class={sectionCollapsed ? "collapsed" : undefined} /></button>
                        <div id={`model-company-${section.company.id}-content`} class="model-company-content" class:collapsed={sectionCollapsed} inert={sectionCollapsed} aria-hidden={sectionCollapsed}>
                          <div class="model-company-content-inner">
                            {#each section.managedProviders as managedProvider (managedProvider.configuration.instanceId)}
                              <button type="button" disabled={!providerAvailable(managedProvider)} title={probeStatus(managedProvider)} onclick={() => chooseModel(managedProvider, null, true)}>
                                <span><strong>{t("chat.composer.providerManagedModel")}</strong></span>
                                {#if managedProvider.configuration.instanceId === provider?.configuration.instanceId && selection.providerManaged}<Check size={14} />{/if}
                              </button>
                            {/each}
                            {#each section.models as entry (`${entry.provider.configuration.instanceId}:${entry.model.id}`)}
                              {@const metadata = modelMetadata(entry.model.contextLimit, entry.model.availability)}
                              <div class="model-row">
                                <button type="button" class="model-choice" disabled={!providerAvailable(entry.provider) || entry.model.availability === "unavailable"} title={entry.model.availability === "available" ? undefined : metadata.join(" · ")} onclick={() => chooseModel(entry.provider, entry.model.id, false)}>
                                  <span><strong>{entry.model.displayName}</strong></span>
                                  {#if entry.provider.configuration.instanceId === provider?.configuration.instanceId && selection.modelId === entry.model.id}<Check size={15} />{/if}
                                </button>
                                <button type="button" class="model-favorite" class:active={entry.provider.configuration.favoriteModelIds.includes(entry.model.id)} aria-label={`${t("chat.composer.favorite")}: ${entry.model.displayName}`} aria-pressed={entry.provider.configuration.favoriteModelIds.includes(entry.model.id)} onclick={() => void toggleModelFavorite(entry.provider, entry.model.id)}><Star size={16} fill={entry.provider.configuration.favoriteModelIds.includes(entry.model.id) ? "currentColor" : "none"} /></button>
                              </div>
                            {/each}
                            {#each section.setupFamilies as family (family.familyId)}
                              <button type="button" class="company-setup" onclick={openProviderSettings}><span><strong>{t("chat.composer.configureProvider")} {section.company.name}</strong></span><Plus size={14} /></button>
                            {/each}
                          </div>
                        </div>
                      </section>
                    {/each}
                    {#if modelCompanySections.length === 0}<p>{t("chat.composer.noModels")}</p>{/if}
                    {#if modelPickerError}<p class="model-picker-error" role="alert">{modelPickerError}</p>{/if}
                    </div>
                  </div>
                  <CalendarScrollbar scrollContainer={modelListElement} stickyTop={4} stickyBottom={4} wheelPassthrough />
                  </div>
                </div>
              </div>
          </div>
        {/if}
    </div>
  {/if}
</div>

{#if pendingProviderModel}
  <ChatProviderForkDialog
    bind:element={providerForkDialog}
    onCancel={() => { pendingProviderModel = null; }}
    onConfirm={() => void confirmProviderFork()}
  />
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
  .model-popover { position: absolute; right: 0; bottom: calc(100% + 0.65rem); z-index: 45; width: min(18.5rem, calc(100vw - 1rem)); overflow: visible; border: 1px solid var(--border); border-radius: 0.8rem; background: var(--popover); padding: 0.8rem 0.8rem 1.2rem; color: var(--popover-foreground); font-size: 0.875rem; box-shadow: 0 2px 6px rgb(0 0 0 / 0.06); }
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
  .effort-knob { position: absolute; top: 50%; width: 2.15rem; height: 2.15rem; transform: translate(-50%, -50%) scale(1); border: 1px solid color-mix(in srgb, var(--foreground) 16%, transparent); border-radius: 999px; background: #ffffff; pointer-events: none; transition: left 240ms cubic-bezier(0.22, 0.75, 0.18, 1), transform 170ms cubic-bezier(0.2, 0.8, 0.2, 1), border-color 260ms ease, background-color 260ms ease; }
  .effort-options button[aria-pressed="true"]:focus-visible ~ .effort-knob, .effort-ladder:is(.holding, .handle-hovered) .effort-knob { transform: translate(-50%, -50%) scale(1.065); }
  .effort-ladder.ultra { border-color: color-mix(in srgb, #8b5cf6 30%, transparent); background: #ddd6fe; }
  .effort-ladder.ultra .effort-fill::before, .effort-ladder.ultra:not(.fast) .effort-particles.calm, .effort-ladder.fast .effort-particles.rapid { opacity: 1; }
  .effort-ladder.fast .effort-options button:has(~ button[aria-pressed="true"]) i, .effort-ladder.ultra .effort-options button:has(~ button[aria-pressed="true"]) i { opacity: 0; transform: scale(0.65); transition-delay: 0ms; }
  .effort-ladder.dummy { cursor: not-allowed; opacity: 0.48; }
  .effort-ladder.dummy .effort-options button:disabled { cursor: not-allowed; }
  .effort-ladder.dummy .effort-options button:hover i { transform: scale(1); }
  .effort-ladder.dummy .effort-knob { opacity: 0.72; }
  :global(.dark) .effort-name.ultra { color: #b794ff; }
  :global(.dark) .effort-ladder { border-color: rgb(255 255 255 / 0.09); background: #45464a; }
  :global(.dark) .effort-fill { background: #1681dc; }
  :global(.dark) .effort-options button i { background: color-mix(in srgb, white 32%, #45464a); }
  :global(.dark) .effort-options button[aria-pressed="true"] i, :global(.dark) .effort-options button:has(~ button[aria-pressed="true"]) i { background: color-mix(in srgb, white 46%, #1681dc); }
  :global(.dark) .effort-knob { border-color: rgb(255 255 255 / 0.18); background: #f4f4f6; }
  :global(.dark) .effort-ladder.ultra { border-color: rgb(139 92 246 / 0.4); background: #4c3b78; }
  :global(.dark) .effort-fill::before { background-image: linear-gradient(105deg, #476cf4 0%, #7144e8 48%, #9f35d7 100%); }
  .effort-footer { display: grid; margin-bottom: 0.5rem; }
  .quick-actions, .effort-guidance { grid-area: 1 / 1; min-height: 2rem; }
  .quick-actions { display: flex; align-items: center; justify-content: space-between; gap: 0.25rem; opacity: 1; transition: opacity 180ms ease; }
  .effort-guidance { display: flex; align-items: center; justify-content: space-between; padding: 0 0.5rem 0 0.35rem; color: var(--muted-foreground); opacity: 0; pointer-events: none; transition: opacity 180ms ease; }
  .effort-footer.holding .quick-actions { opacity: 0; pointer-events: none; }
  .effort-footer.holding .effort-guidance { opacity: 1; }
  .model-picker-toggle { display: flex; min-width: 0; min-height: 2rem; flex: 1 1 auto; align-items: center; gap: 0.2rem; border-radius: 0.5rem; margin-left: -0.2rem; padding: 0.3rem 0.35rem 0.3rem 0.55rem; color: var(--muted-foreground); text-align: left; }
  .model-picker-toggle:hover, .model-picker-toggle[aria-expanded="true"] { background: color-mix(in srgb, var(--accent) 65%, transparent); color: var(--foreground); }
  .model-picker-toggle:focus-visible { outline: 1px solid color-mix(in srgb, var(--ring) 52%, transparent); outline-offset: -1px; }
  .fast-button { display: flex; width: 2rem; height: 2rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.55rem; color: var(--muted-foreground); transition: color 240ms ease, background-color 240ms ease, transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .fast-button :global(svg) { fill: transparent; transform: scale(0.94); transition: fill 200ms ease, transform 220ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .fast-button.active :global(svg) { fill: currentColor; transform: scale(1); }
  .fast-button:hover { background: color-mix(in srgb, var(--accent) 65%, transparent); color: var(--foreground); }
  .fast-button.active { background: rgb(12 120 208 / 0.1); color: #0879d8; }
  .fast-button.active.ultra { background: rgb(124 58 237 / 0.1); color: #7c3aed; }
  .fast-button:active { transform: scale(0.92); }
  .fast-button.dummy:disabled { cursor: not-allowed; opacity: 0.48; }
  :global(.dark) .fast-button.active { background: rgb(22 129 220 / 0.16); color: #3b9aeb; }
  :global(.dark) .fast-button.active.ultra { background: rgb(167 139 250 / 0.15); color: #b794ff; }
  .model-flyout { position: absolute; z-index: 46; width: min(16.5rem, calc(100vw - 1rem)); max-height: min(28rem, 72vh); overflow: hidden auto; border: 1px solid var(--border); border-radius: 0.8rem; background: var(--popover); padding: 0.35rem; font-size: 0.875rem; box-shadow: 0 2px 6px rgb(0 0 0 / 0.07); }
  .model-flyout:not(.positioned) { visibility: hidden; }
  .model-flyout.model-picker-flyout { width: min(16.5rem, calc(100vw - 1rem)); overflow: hidden; padding: 0; }
  .model-picker-shell { width: 100%; height: min(24rem, 72vh); min-height: min(18rem, 72vh); }
  .model-picker-main { display: flex; width: 100%; height: 100%; min-width: 0; min-height: 0; flex-direction: column; overflow: hidden; padding: 0.5rem 0.55rem; }
  .selection-list { padding-top: 0.3rem; }
  .model-company-content-inner > button { display: grid; width: 100%; grid-template-columns: minmax(0, 1fr) auto auto; align-items: center; gap: 0.45rem; border-radius: 0.55rem; padding: 0.5rem 0.55rem; text-align: left; }
  .model-company-content-inner > button:hover, .model-company-content-inner > button:focus-visible { background: var(--accent); outline: none; }
  .model-company-content-inner > button:disabled { opacity: 0.55; }
  .model-company-content-inner > button > span { min-width: 0; }
  .selection-list strong { display: block; }
  .selection-list strong { font-size: 0.875rem; font-weight: 500; }
  .model-list-content > p, .model-company-content-inner > p { padding: 0.38rem 0.5rem; color: var(--muted-foreground); font-size: 0.75rem; }
  .model-search-row { display: flex; flex: 0 0 auto; align-items: center; gap: 0.35rem; border-bottom: 1px solid var(--border); margin: 0 0.15rem 0.2rem; transition: border-color 150ms ease; }
  .model-search-row:focus-within { border-color: var(--primary); }
  .model-search { display: flex; min-width: 0; flex: 1; align-items: center; gap: 0.45rem; padding: 0.5rem 0.15rem; color: var(--muted-foreground); }
  .model-search:focus-within { color: var(--foreground); }
  .model-search input { min-width: 0; flex: 1; user-select: text; background: transparent; color: var(--foreground); font-size: 0.875rem; outline: none; }
  .model-list-frame { --cal-scrollbar-thumb: color-mix(in srgb, var(--popover-foreground) 18%, var(--popover)); --cal-scrollbar-thumb-hover: var(--cal-scrollbar-thumb); position: relative; min-height: 0; flex: 1; margin-right: -0.55rem; }
  .model-list { --model-list-scroll-fade-size: 2rem; height: 100%; min-height: 0; overflow-y: auto; padding: 0 0.55rem 0 0; transition: -webkit-mask-image 120ms ease, mask-image 120ms ease; }
  .model-list-scroll-top { -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--model-list-scroll-fade-size), black); mask-image: linear-gradient(to bottom, transparent, black var(--model-list-scroll-fade-size), black); }
  .model-list-scroll-bottom { -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - var(--model-list-scroll-fade-size)), transparent); mask-image: linear-gradient(to bottom, black, black calc(100% - var(--model-list-scroll-fade-size)), transparent); }
  .model-list-scroll-both { -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--model-list-scroll-fade-size), black calc(100% - var(--model-list-scroll-fade-size)), transparent); mask-image: linear-gradient(to bottom, transparent, black var(--model-list-scroll-fade-size), black calc(100% - var(--model-list-scroll-fade-size)), transparent); }
  .model-company-section { padding: 0.38rem 0; }
  .model-company-section + .model-company-section { border-top: 1px solid color-mix(in srgb, var(--border) 72%, transparent); }
  .model-company-heading { display: grid; width: 100%; min-width: 0; min-height: 2rem; grid-template-columns: 0.875rem minmax(0, 1fr) 1rem; align-items: center; gap: 0.35rem; border-radius: 0.55rem; padding: 0.38rem 0.5rem; color: var(--muted-foreground); font-size: 0.6875rem; font-weight: 550; text-align: left; }
  .model-company-heading:hover { background: transparent; }
  .model-company-heading:focus-visible { outline: 1px solid color-mix(in srgb, var(--ring) 55%, transparent); outline-offset: -2px; }
  .model-company-heading :global(svg:last-child) { color: var(--muted-foreground); transition: transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1); }
  .model-company-heading :global(svg:last-child.collapsed) { transform: rotate(-90deg); }
  .model-company-heading span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .model-company-content { display: grid; grid-template-rows: 1fr; opacity: 1; transition: grid-template-rows 190ms cubic-bezier(0.2, 0.8, 0.2, 1), opacity 140ms ease; }
  .model-company-content.collapsed { grid-template-rows: 0fr; opacity: 0; }
  .model-company-content-inner { min-height: 0; overflow: hidden; }
  .company-setup { color: var(--muted-foreground); }
  .model-row { display: grid; grid-template-columns: minmax(0, 1fr) 2rem; align-items: center; border-radius: 0.55rem; }
  .model-row:hover, .model-row:has(:focus-visible) { background: var(--accent); }
  .model-choice { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 0.4rem; padding: 0.38rem 0.5rem; text-align: left; }
  .model-choice:focus-visible, .model-favorite:focus-visible { outline: 1px solid color-mix(in srgb, var(--ring) 55%, transparent); outline-offset: -2px; }
  .model-choice:disabled { opacity: 0.55; }
  .model-choice > span { min-width: 0; }
  .model-choice > .favorite-model-label { display: flex; align-items: center; gap: 0.45rem; }
  .favorite-model-label :global(svg) { color: var(--muted-foreground); }
  .model-favorite { display: grid; width: 2rem; height: 2rem; place-items: center; border-radius: 0.45rem; color: var(--muted-foreground); }
  .model-favorite:hover, .model-favorite:focus-visible, .model-favorite.active { color: var(--foreground); }
  .model-picker-error { color: var(--destructive) !important; }
  @keyframes ultra-color-flow { from { background-position: 0 0; } to { background-position: 100% 0; } }
  @keyframes galaxy-drift { from { background-position: 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0; } to { background-position: -83px 0, -107px 0, -131px 0, -157px 0, -191px 0, -223px 0, -269px 0, -311px 0; } }
  @keyframes galaxy-stream { from { background-position: 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0, 0 0; } to { background-position: -83px 0, -107px 0, -131px 0, -157px 0, -191px 0, -223px 0, -269px 0, -311px 0; } }
  @container chat-composer (max-width: 460px) { .model-trigger { max-width: 11rem; } .effort-name { display: none; } }
  @container chat-composer (max-width: 330px) { .model-trigger { max-width: 7.5rem; padding-inline: 0.45rem; } }
  @media (prefers-reduced-motion: reduce) { .model-control, .quick-actions, .effort-guidance, .model-company-content, .model-list { transition-duration: 0.01ms; } .effort-fill::before, .effort-particles { animation: none; background-position: 50% 0; } }
</style>
