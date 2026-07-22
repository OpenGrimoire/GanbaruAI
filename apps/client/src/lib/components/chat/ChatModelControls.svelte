<script lang="ts">
  import { tick } from "svelte";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import type { InteractionMode, ModelOptionDefinition, ModelOptionSelection, ModelOptionValue, SafetyMode } from "$lib/chat/contracts";
  import { composerModelSelection, rankedModels, readComposerModelSelection } from "$lib/chat/composer-model";
  import * as chatApi from "$lib/api/chat";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import { getSettingsLauncher } from "$lib/stores/settingsLauncher.svelte";
  import ChatControlMenu, { type ChatControlOption } from "./ChatControlMenu.svelte";
  import ChatProviderIcon from "./ChatProviderIcon.svelte";

  const { compact = false } = $props<{ compact?: boolean }>();
  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  const settings = getSettingsLauncher();
  let modelPickerOpen = $state(false);
  let modelPickerWasOpen = false;
  let modelPickerRoot: HTMLDivElement | undefined = $state();
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
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const healthyProviders = $derived(providers.filter((entry) => entry.configuration.enabled && entry.lastProbe?.state === "healthy"));
  const provider = $derived(providers.find((entry) => entry.configuration.instanceId === chat.composer.providerInstanceId) ?? null);
  const unconfiguredFamilies = $derived((chat.settings?.providerFamilies ?? []).filter((family) => !providers.some((entry) => entry.configuration.familyId === family.familyId)));
  const selection = $derived(readComposerModelSelection(chat.composer.modelSelection));
  const models = $derived(provider?.modelCatalog?.models.filter((model) => provider.configuration.visibleModelIds.length === 0 || provider.configuration.visibleModelIds.includes(model.id) || model.id === selection.modelId) ?? []);
  const recentIds = $derived(chat.settings?.configuration.rememberedSelections.filter((entry) => entry.providerInstanceId === provider?.configuration.instanceId && entry.modelId).map((entry) => entry.modelId as string) ?? []);
  const ranked = $derived(rankedModels(models, provider?.configuration.favoriteModelIds ?? [], recentIds, modelQuery));
  const selectedModel = $derived(models.find((model) => model.id === selection.modelId) ?? null);
  const providerManagedOnly = $derived((provider?.modelCatalog?.models.length ?? 0) === 0);
  const capabilities = $derived(chat.interaction?.capabilities ?? provider?.lastProbe?.capabilities ?? { entries: [] });
  const supportsPlan = $derived(capabilities.entries.some((entry) => entry.capability === "native_plan" && entry.supported));
  const safetyOptions = $derived<ChatControlOption[]>([
    { value: "", label: t("chat.hero.chooseSafety"), icon: "shield" },
    { value: "supervised", label: t("chat.hero.supervised"), description: t("chat.composer.supervisedDescription"), icon: "shield-check" },
    { value: "auto_accept_edits", label: t("chat.hero.autoAccept"), description: t("chat.composer.autoAcceptDescription"), icon: "file-pen" },
    { value: "full_access", label: t("chat.hero.fullAccess"), description: t("chat.composer.fullAccessShortDescription"), icon: "shield-off" },
  ]);
  const interactionOptions = $derived<ChatControlOption[]>([
    { value: "", label: t("chat.hero.chooseInteraction"), icon: "bot" },
    { value: "build", label: t("chat.hero.build"), description: t("chat.composer.buildDescription"), icon: "bot" },
    { value: "plan", label: t("chat.hero.plan"), description: supportsPlan ? t("chat.composer.planDescription") : t("chat.composer.planUnavailable"), icon: "pencil-ruler", disabled: !supportsPlan },
  ]);

  $effect(() => {
    if (modelPickerOpen && !modelPickerWasOpen) void tick().then(() => modelSearch?.focus());
    if (!modelPickerOpen && modelPickerWasOpen) queueMicrotask(() => modelTrigger?.focus());
    modelPickerWasOpen = modelPickerOpen;
  });

  $effect(() => {
    if (!modelPickerOpen) return;
    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (event.target instanceof Node && !modelPickerRoot?.contains(event.target)) {
        modelPickerOpen = false;
      }
    };
    window.addEventListener("pointerdown", closeOnOutsidePointer, true);
    return () => window.removeEventListener("pointerdown", closeOnOutsidePointer, true);
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
    modelQuery = "";
  }

  function providerAvailable(entry: (typeof providers)[number]): boolean {
    return entry.configuration.enabled && entry.lastProbe?.state === "healthy";
  }

  function selectProvider(entry: (typeof providers)[number]): void {
    if (!providerAvailable(entry)) {
      openProviderSettings();
      return;
    }
    chooseProvider(entry.configuration.instanceId);
  }

  function openProviderSettings(): void {
    modelPickerOpen = false;
    settings.open("chat", { chatSubsection: "providers" });
  }

  function probeStatus(entry: (typeof providers)[number]): string {
    if (!entry.configuration.enabled) return t("chat.composer.providerDisabled");
    if (!entry.lastProbe) return t("chat.composer.providerNotChecked");
    if (entry.lastProbe.state === "healthy") return t("chat.status.idle");
    return entry.lastProbe.detail ?? t("chat.status.providerUnavailable");
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

<div class="chat-model-toolbar" data-compact={compact}>
  <div bind:this={modelPickerRoot} class="relative shrink-0">
    <button bind:this={modelTrigger} type="button" class="picker-trigger" data-chat-model-trigger aria-expanded={modelPickerOpen} onclick={() => { modelPickerOpen = !modelPickerOpen; }}>
      <ChatProviderIcon familyId={provider?.configuration.familyId ?? ""} label={provider?.configuration.label ?? t("chat.hero.provider")} accentColor={provider?.configuration.accentColor} />
      <span class="truncate">{selection.providerManaged ? provider?.configuration.label ?? t("chat.composer.providerManagedModel") : selectedModel?.displayName ?? provider?.configuration.label ?? t("chat.hero.chooseProvider")}</span>
      <ChevronDown size={13} />
    </button>
    {#if modelPickerOpen}
      <div class="model-picker" role="dialog" aria-label={t("chat.hero.model")} tabindex="-1" onkeydown={handleModelPickerKeydown}>
        <aside class="provider-pane" role="group" aria-label={t("chat.hero.provider")}>
          <p class="picker-heading">{t("chat.hero.provider")}</p>
          {#each providers as entry}
            <button type="button" class="provider-row" class:selected={entry.configuration.instanceId === provider?.configuration.instanceId} class:unavailable={!providerAvailable(entry)} onclick={() => selectProvider(entry)} title={probeStatus(entry)}>
              <ChatProviderIcon familyId={entry.configuration.familyId} label={entry.configuration.label} accentColor={entry.configuration.accentColor} />
              <span><strong>{entry.configuration.label}</strong><small>{probeStatus(entry)}</small></span>
              {#if entry.configuration.instanceId === provider?.configuration.instanceId}<Check size={13} />{:else if !providerAvailable(entry)}<CircleAlert size={13} />{/if}
            </button>
          {/each}
          {#each unconfiguredFamilies as family}
            <button type="button" class="provider-row unavailable" onclick={openProviderSettings} title={t("chat.composer.configureProvider")}>
              <ChatProviderIcon familyId={family.familyId} label={family.displayName} />
              <span><strong>{family.displayName}</strong><small>{t("chat.composer.notConfigured")}</small></span>
              <Plus size={13} />
            </button>
          {/each}
        </aside>
        <section class="model-pane">
          {#if provider}
            <div class="model-pane-heading"><div><strong>{provider.configuration.label}</strong><small>{t("chat.hero.model")}</small></div>{#if !providerAvailable(provider)}<button type="button" onclick={openProviderSettings}>{t("chat.composer.configureProvider")}</button>{/if}</div>
            <label class="model-search"><Search size={14} /><input bind:this={modelSearch} bind:value={modelQuery} placeholder={t("chat.composer.modelSearch")} /></label>
            <div class="model-list">
              {#if !providerAvailable(provider)}<div class="provider-warning"><CircleAlert size={15} /><span>{probeStatus(provider)}</span></div>{/if}
              {#if providerManagedOnly || selection.providerManaged}<button type="button" class="model-row" disabled={!providerAvailable(provider)} onclick={() => chooseModel(null, true)}><span><strong>{t("chat.composer.providerManagedModel")}</strong><small>{provider.configuration.label}</small></span>{#if selection.providerManaged}<Check size={14} />{/if}</button>{/if}
              {#each ranked as model}<button type="button" class="model-row" disabled={!providerAvailable(provider) || model.availability === "unavailable" || model.availability === "deprecated"} aria-disabled={!providerAvailable(provider) || model.availability === "unavailable" || model.availability === "deprecated"} title={model.availability === "available" ? undefined : modelMetadata(model.contextLimit, model.availability).join(" · ")} onclick={() => chooseModel(model.id, false)}><span><strong>{model.displayName}</strong><small>{[model.id, ...modelMetadata(model.contextLimit, model.availability)].join(" · ")}</small></span>{#if provider.configuration.favoriteModelIds.includes(model.id)}<Star size={12} />{/if}{#if selection.modelId === model.id}<Check size={14} />{/if}</button>{/each}
              {#if ranked.length === 0 && !providerManagedOnly}<p class="empty-models">{t("chat.composer.noModels")}</p>{/if}
            </div>
          {:else}
            <div class="empty-provider"><ChatProviderIcon familyId="" label={t("chat.hero.provider")} size={18} /><strong>{t("chat.hero.chooseProvider")}</strong><span>{t("chat.composer.chooseProviderHint")}</span></div>
          {/if}
        </section>
      </div>
    {/if}
  </div>
  {#if selectedModel?.options.length}<div class="chat-traits">{#each selectedModel.options as definition}{#if definition.kind !== "unknown"}<div class="trait-control" title={definition.description ?? undefined}>{#if definition.kind === "boolean"}<button type="button" class:active={booleanValue(definition.key)} aria-pressed={booleanValue(definition.key)} onclick={() => updateOption(definition.key, { kind: "boolean", value: !booleanValue(definition.key) })}>{definition.label}</button>{:else if definition.kind === "choice"}<ChatControlMenu value={choiceValue(definition.key)} options={[{ value: "", label: definition.label, icon: "brain" }, ...definition.options.map((choice) => ({ value: choice.value, label: choice.label, description: choice.description ?? undefined, icon: "brain" as const }))]} ariaLabel={definition.label} onChange={(value) => updateOption(definition.key, { kind: "choice", value })} compact />{:else if definition.kind === "multiple_choice"}<span class="trait-options">{#each definition.options as choice}<label><input type="checkbox" checked={multipleIncludes(definition.key, choice.value)} onchange={(event) => toggleMultiple(definition.key, choice.value, event.currentTarget.checked)} />{choice.label}</label>{/each}</span>{:else if definition.kind === "integer_range"}<label><span>{definition.label}</span><input type="range" min={definition.minimum} max={definition.maximum} step={definition.step} value={integerValue(definition.key, definition.defaultValue ?? definition.minimum)} oninput={(event) => updateOption(definition.key, { kind: "integer", value: event.currentTarget.valueAsNumber })} /></label>{:else if definition.kind === "text"}<label><span>{definition.label}</span><input type="text" value={textValue(definition.key)} oninput={(event) => updateOption(definition.key, { kind: "text", value: event.currentTarget.value })} /></label>{/if}</div>{/if}{/each}</div>{/if}
  <span class="toolbar-divider" aria-hidden="true"></span>
  <ChatControlMenu value={chat.composer.safetyMode ?? ""} options={safetyOptions} ariaLabel={t("chat.hero.safety")} dataField="safety" onChange={(value) => chooseSafety(value as SafetyMode | "")} {compact} />
  <span class="toolbar-divider" aria-hidden="true"></span>
  <ChatControlMenu value={chat.composer.interactionMode ?? ""} options={interactionOptions} ariaLabel={t("chat.hero.interaction")} dataField="interaction" onChange={(value) => chooseInteraction(value as InteractionMode | "")} {compact} />
</div>

{#if fullAccessDialogOpen}<div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closeFullAccessDialog}></button><div bind:this={fullAccessDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="full-access-title" tabindex="-1" onkeydown={(event) => handleConfirmationKeydown(event, closeFullAccessDialog)}><h2 id="full-access-title" class="font-semibold">{t("chat.composer.fullAccessTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.fullAccessDescription", provider?.configuration.label ?? "", chat.selectedWorkspace?.workspace.displayName ?? "")}</p>{#if error}<p role="alert" class="mt-2 text-sm text-destructive">{error}</p>{/if}<div class="mt-4 flex justify-end gap-2"><button type="button" class="chat-secondary-button" onclick={closeFullAccessDialog}>{t("chat.cancel")}</button><button type="button" class="chat-primary-button" onclick={() => void confirmFullAccess()}>{t("chat.composer.confirmFullAccess")}</button></div></div></div>{/if}

{#if pendingProviderId}<div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4"><button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={closeProviderForkDialog}></button><div bind:this={providerForkDialog} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="provider-fork-title" tabindex="-1" onkeydown={(event) => handleConfirmationKeydown(event, closeProviderForkDialog)}><h2 id="provider-fork-title" class="font-semibold">{t("chat.composer.changeProviderTitle")}</h2><p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.changeProviderDescription")}</p><div class="mt-4 flex justify-end gap-2"><button type="button" class="chat-secondary-button" onclick={closeProviderForkDialog}>{t("chat.cancel")}</button><button type="button" class="chat-primary-button" onclick={() => void confirmProviderFork()}>{t("chat.composer.startProviderFork")}</button></div></div></div>{/if}

<style>
  .chat-model-toolbar { display: flex; min-width: 0; max-width: 100%; align-items: center; gap: 0.25rem; color: var(--muted-foreground); }
  .picker-trigger { display: inline-flex; min-width: 0; height: 1.9rem; max-width: 13rem; align-items: center; gap: 0.35rem; border-radius: 0.55rem; padding: 0.25rem 0.45rem; color: var(--muted-foreground); font-size: 0.733333rem; }
  .picker-trigger:hover, .picker-trigger[aria-expanded="true"] { background: var(--accent); color: var(--foreground); }
  .picker-trigger:focus-visible { outline: 2px solid var(--ring); outline-offset: 1px; }
  .picker-trigger > span:nth-child(2) { min-width: 0; flex: 1; }
  .toolbar-divider { width: 1px; height: 1rem; flex: 0 0 auto; background: var(--border); }
  .model-picker { position: absolute; left: 0; bottom: calc(100% + 0.5rem); z-index: 35; display: grid; width: min(34rem, 86vw); height: min(25rem, 62vh); grid-template-columns: minmax(9.5rem, 0.78fr) minmax(15rem, 1.5fr); overflow: hidden; border: 1px solid var(--border); border-radius: 0.8rem; background: var(--popover); box-shadow: 0 18px 48px rgb(0 0 0 / 0.22); }
  .picker-heading { padding: 0.3rem 0.45rem 0.2rem; color: var(--muted-foreground); font-size: 0.666667rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; }
  .provider-pane { min-width: 0; overflow-y: auto; border-right: 1px solid var(--border); padding: 0.4rem; background: color-mix(in srgb, var(--muted) 28%, var(--popover)); }
  .provider-row { display: grid; width: 100%; min-width: 0; grid-template-columns: 1.25rem minmax(0, 1fr) 0.9rem; align-items: center; gap: 0.4rem; border-radius: 0.45rem; padding: 0.45rem; color: var(--muted-foreground); text-align: left; }
  .provider-row:hover, .provider-row.selected { background: var(--accent); color: var(--foreground); }
  .provider-row.unavailable { opacity: 0.72; }
  .provider-row > span:nth-child(2) { min-width: 0; }
  .provider-row strong, .provider-row small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .provider-row strong { color: currentColor; font-size: 0.733333rem; font-weight: 500; }
  .provider-row small { margin-top: 0.05rem; color: var(--muted-foreground); font-size: 0.6rem; }
  .model-pane { display: flex; min-width: 0; min-height: 0; flex-direction: column; }
  .model-pane-heading { display: flex; min-height: 3rem; align-items: center; gap: 0.5rem; border-bottom: 1px solid var(--border); padding: 0.45rem 0.65rem; }
  .model-pane-heading > div { min-width: 0; flex: 1; }
  .model-pane-heading strong, .model-pane-heading small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .model-pane-heading strong { font-size: 0.8rem; font-weight: 500; }
  .model-pane-heading small { color: var(--muted-foreground); font-size: 0.633333rem; }
  .model-pane-heading button { border-radius: 0.4rem; background: var(--accent); padding: 0.3rem 0.45rem; font-size: 0.666667rem; }
  .model-search { display: flex; align-items: center; gap: 0.45rem; margin: 0.5rem 0.55rem 0.2rem; border: 1px solid var(--border); border-radius: 0.5rem; padding: 0.4rem 0.5rem; color: var(--muted-foreground); }
  .model-search:focus-within { border-color: var(--ring); color: var(--foreground); }
  .model-search input { min-width: 0; flex: 1; background: transparent; color: var(--foreground); font-size: 0.733333rem; outline: none; }
  .model-list { min-height: 0; flex: 1; overflow-y: auto; padding: 0.3rem 0.45rem 0.45rem; }
  .model-row { display: flex; width: 100%; align-items: center; gap: 0.45rem; border-radius: 0.375rem; padding: 0.45rem; text-align: left; }
  .model-row:hover { background: var(--accent); }
  .model-row:disabled { opacity: 0.55; }
  .model-row span { min-width: 0; flex: 1; }
  .model-row strong, .model-row small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .model-row strong { font-size: 0.766667rem; font-weight: 500; }
  .model-row small { color: var(--muted-foreground); font-size: 0.666667rem; }
  .provider-warning { display: flex; align-items: flex-start; gap: 0.45rem; margin: 0.2rem 0.1rem 0.4rem; border-radius: 0.45rem; background: color-mix(in srgb, var(--status-tentative) 10%, transparent); padding: 0.5rem; color: var(--status-tentative); font-size: 0.666667rem; line-height: 1rem; }
  .provider-warning :global(svg) { flex: 0 0 auto; margin-top: 0.05rem; }
  .empty-models { padding: 0.75rem; text-align: center; color: var(--muted-foreground); font-size: 0.7rem; }
  .empty-provider { display: grid; min-height: 100%; place-content: center; justify-items: center; gap: 0.4rem; padding: 2rem; color: var(--muted-foreground); text-align: center; }
  .empty-provider strong { color: var(--foreground); font-size: 0.8rem; font-weight: 500; }
  .empty-provider span { max-width: 15rem; font-size: 0.666667rem; line-height: 1rem; }
  .chat-traits { display: flex; min-width: 0; align-items: center; gap: 0.35rem; }
  .trait-control, .trait-control > label { display: flex; min-width: 0; align-items: center; gap: 0.3rem; white-space: nowrap; color: var(--muted-foreground); font-size: 0.666667rem; }
  .trait-control > button { height: 1.9rem; border-radius: 0.5rem; padding-inline: 0.45rem; }
  .trait-control > button:hover, .trait-control > button.active { background: var(--accent); color: var(--foreground); }
  .chat-traits input[type="text"] { max-width: 8rem; border: 0; border-radius: 0.3rem; background: var(--muted); padding: 0.2rem 0.3rem; color: var(--foreground); }
  .trait-options { display: flex; flex-wrap: wrap; gap: 0.35rem; }
  .trait-options label { display: inline-flex; align-items: center; gap: 0.2rem; }
  @container chat-composer (max-width: 640px) { .chat-traits { display: none; } .picker-trigger { max-width: 11rem; } }
  @container chat-composer (max-width: 460px) { .toolbar-divider { display: none; } .picker-trigger { max-width: 9rem; } .model-picker { width: min(22rem, calc(100vw - 1rem)); grid-template-columns: 1fr; } .provider-pane { max-height: 8.5rem; border-right: 0; border-bottom: 1px solid var(--border); } .provider-pane .picker-heading { display: none; } .model-pane { min-height: 12rem; } }
</style>
