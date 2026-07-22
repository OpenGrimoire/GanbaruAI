<script lang="ts">
  import { tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import type { InteractionMode, ModelOptionDefinition, ModelOptionSelection, ModelOptionValue, SafetyMode } from "$lib/chat/contracts";
  import { composerModelSelection, rankedModels, readComposerModelSelection } from "$lib/chat/composer-model";
  import * as chatApi from "$lib/api/chat";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";

  const { compact = false } = $props<{ compact?: boolean }>();
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let modelPickerOpen = $state(false);
  let modelPickerWasOpen = false;
  let modelTrigger: HTMLButtonElement | undefined = $state();
  let modelSearch: HTMLInputElement | undefined = $state();
  let fullAccessDialog: HTMLElement | undefined = $state();
  let providerForkDialog: HTMLElement | undefined = $state();
  let confirmationReturnFocus: HTMLElement | null = null;
  let modelQuery = $state("");
  let fullAccessDialogOpen = $state(false);
  let pendingProviderId = $state<string | null>(null);
  let fullAccessTrusted = $state(false);
  let trustKey = $state("");
  let restoredKey = $state("");
  let error = $state<string | null>(null);
  const providers = $derived((chat.settings?.providerInstances ?? []).filter((entry) => entry.configuration.enabled && entry.lastProbe?.state === "healthy"));
  const provider = $derived(providers.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const selection = $derived(readComposerModelSelection(chat.composer.modelSelection));
  const models = $derived(provider?.modelCatalog?.models.filter((model) => provider.configuration.visibleModelIds.length === 0 || provider.configuration.visibleModelIds.includes(model.id) || model.id === selection.modelId) ?? []);
  const recentIds = $derived(chat.settings?.configuration.rememberedSelections.filter((entry) => entry.providerInstanceId === provider?.configuration.instanceId && entry.modelId).map((entry) => entry.modelId as string) ?? []);
  const ranked = $derived(rankedModels(models, provider?.configuration.favoriteModelIds ?? [], recentIds, modelQuery));
  const selectedModel = $derived(models.find((model) => model.id === selection.modelId) ?? null);
  const providerManagedOnly = $derived((provider?.modelCatalog?.models.length ?? 0) === 0);
  const capabilities = $derived(chat.interaction?.capabilities ?? provider?.lastProbe?.capabilities ?? { entries: [] });
  const supportsPlan = $derived(capabilities.entries.some((entry) => entry.capability === "native_plan" && entry.supported));

  $effect(() => {
    if (modelPickerOpen && !modelPickerWasOpen) void tick().then(() => modelSearch?.focus());
    if (!modelPickerOpen && modelPickerWasOpen) queueMicrotask(() => modelTrigger?.focus());
    modelPickerWasOpen = modelPickerOpen;
  });

  $effect(() => {
    const workspaceId = chat.composer.workspaceId;
    if (!workspaceId || chat.composer.providerInstanceId) return;
    const preferred = chat.settings?.configuration.workspaceProviderPreferences[workspaceId];
    if (preferred && providers.some((entry) => entry.configuration.instanceId === preferred)) chat.setComposerProvider(preferred);
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
    const workspaceId = chat.composer.workspaceId;
    const providerId = chat.composer.providerInstanceId;
    const key = workspaceId && providerId ? `${workspaceId}:${providerId}` : "";
    if (key === trustKey) return;
    trustKey = key;
    fullAccessTrusted = false;
    if (workspaceId && providerId) {
      void chatApi.hasChatFullAccessTrust(providerId, workspaceId).then((trusted) => {
        if (trustKey === key) fullAccessTrusted = trusted;
      }).catch(() => undefined);
    }
  });

  function chooseProvider(instanceId: string): void {
    if (chat.selectedThread && instanceId && instanceId !== chat.selectedThread.providerInstanceId) {
      confirmationReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      pendingProviderId = instanceId;
      void tick().then(() => firstFocusable(providerForkDialog)?.focus());
      return;
    }
    chat.setComposerProvider(instanceId || null);
    chat.setComposerModel(null);
    modelPickerOpen = false;
  }


  async function confirmProviderFork(): Promise<void> {
    if (!pendingProviderId) return;
    const instanceId = pendingProviderId;
    closeProviderForkDialog();
    await chat.forkComposerWithProvider(instanceId);
    modelPickerOpen = false;
  }

  function chooseModel(modelId: string | null, providerManaged: boolean): void {
    const model = models.find((candidate) => candidate.id === modelId);
    chat.setComposerModel(composerModelSelection(modelId, providerManaged, model ? defaultOptions(model.options) : []));
    modelPickerOpen = false;
  }

  function chooseSafety(value: SafetyMode | ""): void {
    if (value === "full_access" && !fullAccessTrusted) {
      confirmationReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      fullAccessDialogOpen = true;
      void tick().then(() => firstFocusable(fullAccessDialog)?.focus());
      return;
    }
    chat.setComposerModes(value || null, chat.composer.interactionMode);
  }

  async function confirmFullAccess(): Promise<void> {
    if (!chat.composer.workspaceId || !chat.composer.providerInstanceId) return;
    error = null;
    try {
      await chatApi.setChatFullAccessTrust(chat.composer.providerInstanceId, chat.composer.workspaceId, true);
      fullAccessTrusted = true;
      chat.setComposerModes("full_access", chat.composer.interactionMode);
      closeFullAccessDialog();
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function chooseInteraction(value: InteractionMode | ""): void {
    chat.setComposerModes(chat.composer.safetyMode, value || null);
  }

  function handleModelPickerKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      modelPickerOpen = false;
      return;
    }
    if (event.key !== "Tab" || !(event.currentTarget instanceof HTMLElement)) return;
    const focusable = [...event.currentTarget.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex='-1'])",
    )].filter((element) => !element.hidden && element.getClientRects().length > 0);
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (!first || !last) {
      event.preventDefault();
      event.currentTarget.focus();
    } else if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function handleConfirmationKeydown(event: KeyboardEvent, close: () => void): void {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    if (event.key !== "Tab" || !(event.currentTarget instanceof HTMLElement)) return;
    const focusable = focusableElements(event.currentTarget);
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (!first || !last) {
      event.preventDefault();
      event.currentTarget.focus();
    } else if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function focusableElements(container: HTMLElement): HTMLElement[] {
    return [...container.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex='-1'])",
    )].filter((element) => !element.hidden && element.getClientRects().length > 0);
  }

  function firstFocusable(container: HTMLElement | undefined): HTMLElement | undefined {
    return container ? focusableElements(container)[0] : undefined;
  }

  function restoreConfirmationFocus(): void {
    const target = confirmationReturnFocus;
    confirmationReturnFocus = null;
    queueMicrotask(() => {
      if (target?.isConnected) target.focus();
    });
  }

  function closeFullAccessDialog(): void {
    fullAccessDialogOpen = false;
    restoreConfirmationFocus();
  }

  function closeProviderForkDialog(): void {
    pendingProviderId = null;
    restoreConfirmationFocus();
  }

  function updateOption(key: string, value: ModelOptionValue): void {
    const next = selection.options.filter((entry) => entry.key !== key);
    next.push({ key, value });
    chat.setComposerModel(composerModelSelection(selection.modelId, selection.providerManaged, next));
  }

  function option(key: string): ModelOptionSelection | undefined {
    return selection.options.find((entry) => entry.key === key);
  }

  function toggleMultiple(key: string, value: string, checked: boolean): void {
    const current = option(key)?.value;
    const values = current?.kind === "multiple_choice" ? current.value : [];
    updateOption(key, { kind: "multiple_choice", value: checked ? [...new Set([...values, value])] : values.filter((entry) => entry !== value) });
  }

  function booleanValue(key: string): boolean {
    const value = option(key)?.value;
    return value?.kind === "boolean" && value.value;
  }

  function choiceValue(key: string): string {
    const value = option(key)?.value;
    return value?.kind === "choice" ? value.value : "";
  }

  function multipleIncludes(key: string, candidate: string): boolean {
    const value = option(key)?.value;
    return value?.kind === "multiple_choice" && value.value.includes(candidate);
  }

  function integerValue(key: string, fallback: number): number {
    const value = option(key)?.value;
    return value?.kind === "integer" ? value.value : fallback;
  }

  function textValue(key: string): string {
    const value = option(key)?.value;
    return value?.kind === "text" ? value.value : "";
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
    if (contextLimit !== null) {
      values.push(t("chat.composer.modelContext", formatNumber(localization.locale, contextLimit)));
    }
    if (availability === "stale") values.push(t("chat.composer.modelStale"));
    if (availability === "unavailable") values.push(t("chat.composer.modelUnavailable"));
    if (availability === "deprecated") values.push(t("chat.composer.modelDeprecated"));
    return values;
  }
</script>

<div class:compact class="chat-model-controls">
  <label><span>{t("chat.hero.provider")}</span><select data-chat-field="provider" value={chat.composer.providerInstanceId ?? ""} onchange={(event) => chooseProvider(event.currentTarget.value)}><option value="">{t("chat.hero.chooseProvider")}</option>{#each providers as entry}<option value={entry.configuration.instanceId}>{entry.configuration.label}</option>{/each}</select></label>
  <div class="relative">
    <span class="control-label">{t("chat.hero.model")}</span>
    <button bind:this={modelTrigger} type="button" class="picker-trigger" data-chat-field="model" disabled={!provider} aria-expanded={modelPickerOpen} onclick={() => { modelPickerOpen = !modelPickerOpen; }}>{selection.providerManaged ? t("chat.composer.providerManagedModel") : selectedModel?.displayName ?? t("chat.hero.chooseModel")}<ChevronDown size={13} /></button>
    {#if modelPickerOpen}<div class="model-picker" role="dialog" aria-label={t("chat.hero.model")} tabindex="-1" onkeydown={handleModelPickerKeydown}><label class="model-search"><Search size={13} /><input bind:this={modelSearch} bind:value={modelQuery} placeholder={t("chat.composer.modelSearch")} /></label>{#if providerManagedOnly || selection.providerManaged}<button type="button" class="model-row" onclick={() => chooseModel(null, true)}><span><strong>{t("chat.composer.providerManagedModel")}</strong></span>{#if selection.providerManaged}<Check size={14} />{/if}</button>{/if}{#each ranked as model}<button type="button" class="model-row" disabled={model.availability === "unavailable" || model.availability === "deprecated"} aria-disabled={model.availability === "unavailable" || model.availability === "deprecated"} title={model.availability === "available" ? undefined : modelMetadata(model.contextLimit, model.availability).join(" · ")} onclick={() => chooseModel(model.id, false)}><span><strong>{model.displayName}</strong><small>{[model.id, ...modelMetadata(model.contextLimit, model.availability)].join(" · ")}</small></span>{#if provider?.configuration.favoriteModelIds.includes(model.id)}<Star size={12} />{/if}{#if selection.modelId === model.id}<Check size={14} />{/if}</button>{/each}</div>{/if}
  </div>
  <div class="secondary-controls"><label><span>{t("chat.hero.safety")}</span><select data-chat-field="safety" value={chat.composer.safetyMode ?? ""} onchange={(event) => chooseSafety(event.currentTarget.value as SafetyMode | "")}><option value="">{t("chat.hero.chooseSafety")}</option><option value="supervised">{t("chat.hero.supervised")}</option><option value="auto_accept_edits">{t("chat.hero.autoAccept")}</option><option value="full_access">{t("chat.hero.fullAccess")}</option></select></label><label><span>{t("chat.hero.interaction")}</span><select data-chat-field="interaction" title={!supportsPlan ? t("chat.composer.planUnavailable") : undefined} value={chat.composer.interactionMode ?? ""} onchange={(event) => chooseInteraction(event.currentTarget.value as InteractionMode | "")}><option value="">{t("chat.hero.chooseInteraction")}</option><option value="build">{t("chat.hero.build")}</option><option value="plan" disabled={!supportsPlan}>{t("chat.hero.plan")}</option></select></label></div>
  <details class="compact-controls"><summary>{t("chat.composer.controls")}</summary><div><label><span>{t("chat.hero.safety")}</span><select value={chat.composer.safetyMode ?? ""} onchange={(event) => chooseSafety(event.currentTarget.value as SafetyMode | "")}><option value="">{t("chat.hero.chooseSafety")}</option><option value="supervised">{t("chat.hero.supervised")}</option><option value="auto_accept_edits">{t("chat.hero.autoAccept")}</option><option value="full_access">{t("chat.hero.fullAccess")}</option></select></label><label><span>{t("chat.hero.interaction")}</span><select title={!supportsPlan ? t("chat.composer.planUnavailable") : undefined} value={chat.composer.interactionMode ?? ""} onchange={(event) => chooseInteraction(event.currentTarget.value as InteractionMode | "")}><option value="">{t("chat.hero.chooseInteraction")}</option><option value="build">{t("chat.hero.build")}</option><option value="plan" disabled={!supportsPlan}>{t("chat.hero.plan")}</option></select></label></div></details>
</div>

{#if selectedModel?.options.length}<div class="chat-traits">{#each selectedModel.options as definition}{#if definition.kind !== "unknown"}<label title={definition.description ?? undefined}><span>{definition.label}</span>{#if definition.kind === "boolean"}<input type="checkbox" checked={booleanValue(definition.key)} onchange={(event) => updateOption(definition.key, { kind: "boolean", value: event.currentTarget.checked })} />{:else if definition.kind === "choice"}<select value={choiceValue(definition.key)} onchange={(event) => updateOption(definition.key, { kind: "choice", value: event.currentTarget.value })}>{#each definition.options as choice}<option value={choice.value}>{choice.label}</option>{/each}</select>{:else if definition.kind === "multiple_choice"}<span class="trait-options">{#each definition.options as choice}<label><input type="checkbox" checked={multipleIncludes(definition.key, choice.value)} onchange={(event) => toggleMultiple(definition.key, choice.value, event.currentTarget.checked)} />{choice.label}</label>{/each}</span>{:else if definition.kind === "integer_range"}<input type="range" min={definition.minimum} max={definition.maximum} step={definition.step} value={integerValue(definition.key, definition.defaultValue ?? definition.minimum)} oninput={(event) => updateOption(definition.key, { kind: "integer", value: event.currentTarget.valueAsNumber })} />{:else if definition.kind === "text"}<input type="text" value={textValue(definition.key)} oninput={(event) => updateOption(definition.key, { kind: "text", value: event.currentTarget.value })} />{/if}</label>{/if}{/each}</div>{/if}

{#if fullAccessDialogOpen}<div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closeFullAccessDialog}></button><div bind:this={fullAccessDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="full-access-title" tabindex="-1" onkeydown={(event) => handleConfirmationKeydown(event, closeFullAccessDialog)}><h2 id="full-access-title" class="font-semibold">{t("chat.composer.fullAccessTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.fullAccessDescription", provider?.configuration.label ?? "", chat.selectedWorkspace?.workspace.displayName ?? "")}</p>{#if error}<p role="alert" class="mt-2 text-sm text-destructive">{error}</p>{/if}<div class="mt-4 flex justify-end gap-2"><button type="button" class="chat-secondary-button" onclick={closeFullAccessDialog}>{t("chat.cancel")}</button><button type="button" class="chat-primary-button" onclick={() => void confirmFullAccess()}>{t("chat.composer.confirmFullAccess")}</button></div></div></div>{/if}

{#if pendingProviderId}<div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closeProviderForkDialog}></button><div bind:this={providerForkDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="provider-fork-title" tabindex="-1" onkeydown={(event) => handleConfirmationKeydown(event, closeProviderForkDialog)}><h2 id="provider-fork-title" class="font-semibold">{t("chat.composer.changeProviderTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.changeProviderDescription")}</p><div class="mt-4 flex justify-end gap-2"><button type="button" class="chat-secondary-button" onclick={closeProviderForkDialog}>{t("chat.cancel")}</button><button type="button" class="chat-primary-button" onclick={() => void confirmProviderFork()}>{t("chat.composer.startProviderFork")}</button></div></div></div>{/if}

<style>
  .chat-model-controls { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 0.5rem; }
  .secondary-controls { display: contents; }
  .compact-controls { display: none; grid-column: 1 / -1; }
  .compact-controls summary { cursor: pointer; color: var(--muted-foreground); font-size: 0.733333rem; }
  .compact-controls > div { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 0.5rem; margin-top: 0.4rem; }
  .chat-model-controls label, .control-label { display: flex; flex-direction: column; gap: 0.25rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-model-controls select, .picker-trigger { display: flex; min-height: 2rem; width: 100%; min-width: 0; align-items: center; justify-content: space-between; gap: 0.3rem; border: 1px solid var(--border); border-radius: 0.375rem; background: var(--background); padding: 0.3rem 0.45rem; color: var(--foreground); font-size: 0.733333rem; }
  .model-picker { position: absolute; right: 0; bottom: calc(100% + 0.35rem); z-index: 30; display: flex; width: min(28rem, 86vw); max-height: min(24rem, 60vh); flex-direction: column; overflow: auto; border: 1px solid var(--border); border-radius: 0.5rem; background: var(--popover); padding: 0.35rem; box-shadow: 0 12px 32px rgb(0 0 0 / 0.24); }
  .model-search { position: sticky; top: 0; z-index: 1; display: flex !important; flex-direction: row !important; align-items: center; gap: 0.4rem !important; background: var(--popover); padding: 0.35rem; }
  .model-search input { min-width: 0; flex: 1; border: 1px solid var(--border); border-radius: 0.375rem; padding: 0.35rem; color: var(--foreground); }
  .model-row { display: flex; width: 100%; align-items: center; gap: 0.45rem; border-radius: 0.375rem; padding: 0.45rem; text-align: left; }
  .model-row:hover { background: var(--accent); }
  .model-row:disabled { opacity: 0.55; }
  .model-row span { min-width: 0; flex: 1; }
  .model-row strong, .model-row small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .model-row strong { font-size: 0.8rem; }
  .model-row small { color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-traits { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .chat-traits > label { display: flex; align-items: center; gap: 0.35rem; color: var(--muted-foreground); font-size: 0.666667rem; }
  .chat-traits select, .chat-traits input[type="text"] { border: 1px solid var(--border); border-radius: 0.3rem; background: var(--background); padding: 0.25rem; color: var(--foreground); }
  .trait-options { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .trait-options label { display: inline-flex; align-items: center; gap: 0.2rem; }
  @container chat-composer (max-width: 640px) { .chat-model-controls { grid-template-columns: repeat(2, minmax(0, 1fr)); } .secondary-controls { display: none; } .compact-controls { display: block; } }
  @container chat-composer (max-width: 360px) { .chat-model-controls, .compact-controls > div { grid-template-columns: 1fr; } }
</style>
