<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import type { JsonValue } from "$lib/chat/contracts";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let selectedProviderId = $state<string | null>(null);
  let search = $state("");
  let busy = $state(false);
  let customOpen = $state(false);
  let customId = $state("");
  let customLabel = $state("");
  let customError = $state<string | null>(null);
  let operationError = $state<string | null>(null);
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedProvider = $derived(providers.find((entry) => entry.configuration.instanceId === selectedProviderId) ?? providers[0] ?? null);
  const models = $derived.by(() => {
    const query = search.trim().toLocaleLowerCase(localization.locale);
    const favorites = new Set(selectedProvider?.configuration.favoriteModelIds ?? []);
    const collator = new Intl.Collator(localization.locale);
    return [...(selectedProvider?.modelCatalog?.models ?? [])]
      .filter((model) => !query
        || model.displayName.toLocaleLowerCase(localization.locale).includes(query)
        || model.id.toLocaleLowerCase(localization.locale).includes(query))
      .sort((left, right) => Number(favorites.has(right.id)) - Number(favorites.has(left.id))
        || collator.compare(left.displayName, right.displayName));
  });
  const acceptsCustomModels = $derived(selectedProvider?.configuration.familyId === "codex" && providerConfigBoolean("allowCustomModels"));

  $effect(() => {
    if (!selectedProviderId && providers[0]) selectedProviderId = providers[0].configuration.instanceId;
  });

  async function refresh(): Promise<void> {
    if (!selectedProvider) return;
    busy = true;
    operationError = null;
    try {
      await chat.refreshModels(selectedProvider.configuration.instanceId);
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      busy = false;
    }
  }

  async function toggleVisible(modelId: string): Promise<void> {
    if (!selectedProvider) return;
    const allIds = selectedProvider.modelCatalog?.models.map((model) => model.id) ?? [];
    const configured = selectedProvider.configuration.visibleModelIds;
    const current = configured.length === 0 ? allIds : configured;
    const visible = current.includes(modelId) ? current.filter((id) => id !== modelId) : [...current, modelId];
    operationError = null;
    try {
      await chat.updateModels(selectedProvider.configuration.instanceId, visible, selectedProvider.configuration.favoriteModelIds);
    } catch (error: unknown) {
      operationError = errorMessage(error);
    }
  }

  async function toggleFavorite(modelId: string): Promise<void> {
    if (!selectedProvider) return;
    const favorites = selectedProvider.configuration.favoriteModelIds.includes(modelId)
      ? selectedProvider.configuration.favoriteModelIds.filter((id) => id !== modelId)
      : [...selectedProvider.configuration.favoriteModelIds, modelId];
    operationError = null;
    try {
      await chat.updateModels(selectedProvider.configuration.instanceId, selectedProvider.configuration.visibleModelIds, favorites);
    } catch (error: unknown) {
      operationError = errorMessage(error);
    }
  }

  function isVisible(modelId: string): boolean {
    if (!selectedProvider) return false;
    return selectedProvider.configuration.visibleModelIds.length === 0 || selectedProvider.configuration.visibleModelIds.includes(modelId);
  }

  function providerConfigBoolean(key: string): boolean {
    const value = selectedProvider?.configuration.providerConfig.value;
    return typeof value === "object" && value !== null && !Array.isArray(value) && value[key] === true;
  }

  function providerConfigRecord(): Record<string, JsonValue> {
    const value = selectedProvider?.configuration.providerConfig.value;
    return typeof value === "object" && value !== null && !Array.isArray(value) ? { ...value } : {};
  }

  function stringRecord(value: JsonValue | undefined): Record<string, string> {
    if (typeof value !== "object" || value === null || Array.isArray(value)) return {};
    return Object.fromEntries(Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === "string"));
  }

  async function saveCustomModel(): Promise<void> {
    if (!selectedProvider || !acceptsCustomModels) return;
    const id = customId.trim();
    const label = customLabel.trim();
    if (!id || id.length > 256 || [...id].some((character) => /\p{Control}/u.test(character))) {
      customError = t("settings.chat.models.customId");
      return;
    }
    const config = providerConfigRecord();
    const existingIds = Array.isArray(config.customModelIds)
      ? config.customModelIds.filter((value): value is string => typeof value === "string")
      : [];
    const existingLabels = stringRecord(config.customModelLabels);
    config.customModelIds = [...new Set([...existingIds, id])];
    config.customModelLabels = { ...existingLabels, ...(label ? { [id]: label } : {}) };
    busy = true;
    customError = null;
    try {
      await chat.saveProvider({
        ...selectedProvider.configuration,
        providerConfig: { ...selectedProvider.configuration.providerConfig, value: config },
      });
      await chat.probeProvider(selectedProvider.configuration.instanceId);
      await chat.refreshModels(selectedProvider.configuration.instanceId);
      customId = "";
      customLabel = "";
      customOpen = false;
    } catch (error: unknown) {
      customError = error instanceof Error ? error.message : String(error);
    } finally {
      busy = false;
    }
  }

  async function removeCustomModel(modelId: string): Promise<void> {
    if (!selectedProvider || !acceptsCustomModels) return;
    const config = providerConfigRecord();
    const customIds = Array.isArray(config.customModelIds)
      ? config.customModelIds.filter((value): value is string => typeof value === "string" && value !== modelId)
      : [];
    const labels = stringRecord(config.customModelLabels);
    delete labels[modelId];
    config.customModelIds = customIds;
    config.customModelLabels = labels;
    busy = true;
    operationError = null;
    try {
      await chat.saveProvider({
        ...selectedProvider.configuration,
        visibleModelIds: selectedProvider.configuration.visibleModelIds.filter((id) => id !== modelId),
        favoriteModelIds: selectedProvider.configuration.favoriteModelIds.filter((id) => id !== modelId),
        providerConfig: { ...selectedProvider.configuration.providerConfig, value: config },
      });
      await chat.probeProvider(selectedProvider.configuration.instanceId);
      await chat.refreshModels(selectedProvider.configuration.instanceId);
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      busy = false;
    }
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }
</script>

<section class="flex flex-col gap-4">
  <div class="flex flex-wrap items-end justify-between gap-3">
    <div><h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.models.heading")}</h2></div>
    <div class="flex gap-2">{#if acceptsCustomModels}<button type="button" class="chat-settings-button" disabled={busy} onclick={() => { customOpen = !customOpen; }}>{t("settings.chat.models.addCustom")}</button>{/if}<button type="button" class="chat-settings-button" disabled={!selectedProvider || busy} onclick={() => void refresh()}><RefreshCw size={13} />{t("settings.chat.models.refresh")}</button></div>
  </div>
  {#if providers.length > 0}
    <div class="grid gap-2 sm:grid-cols-[minmax(10rem,14rem)_1fr]">
      <label class="setup-field"><span>{t("settings.chat.models.provider")}</span><select bind:value={selectedProviderId}>{#each providers as provider}<option value={provider.configuration.instanceId}>{provider.configuration.label}</option>{/each}</select></label>
      <label class="setup-field"><span>{t("settings.chat.models.search")}</span><div class="relative"><Search size={14} class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" /><input class="w-full pl-8" type="search" bind:value={search} /></div></label>
    </div>
  {/if}
  {#if operationError}<p role="alert" class="text-sm text-destructive">{operationError}</p>{/if}
  {#if selectedProvider?.modelCatalog?.stale}<p class="rounded-md border border-status-tentative/40 p-2 text-xs text-status-tentative">{t("settings.chat.models.catalogStale")}</p>{/if}
  {#if customOpen && acceptsCustomModels}
    <div class="grid gap-2 rounded-lg border border-border bg-card p-3 sm:grid-cols-2">
      <label class="setup-field"><span>{t("settings.chat.models.customId")}</span><input bind:value={customId} /></label>
      <label class="setup-field"><span>{t("settings.chat.models.customLabel")}</span><input bind:value={customLabel} /></label>
      {#if customError}<p role="alert" class="text-xs text-destructive sm:col-span-2">{customError}</p>{/if}
      <div class="sm:col-span-2"><button type="button" class="chat-settings-button" disabled={busy} onclick={() => void saveCustomModel()}>{t("settings.chat.models.saveCustom")}</button></div>
    </div>
  {/if}
  {#if models.length === 0}
    <p class="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">{t("settings.chat.models.empty")}</p>
  {:else}
    <div class="divide-y divide-border rounded-lg border border-border">
      {#each models as model}
        <div class="flex items-start gap-3 p-3">
          <button type="button" class="mt-0.5 text-muted-foreground hover:text-status-tentative" aria-label={t("settings.chat.models.favorite")} aria-pressed={selectedProvider?.configuration.favoriteModelIds.includes(model.id)} onclick={() => void toggleFavorite(model.id)}><Star size={16} fill={selectedProvider?.configuration.favoriteModelIds.includes(model.id) ? "currentColor" : "none"} /></button>
          <div class="min-w-0 flex-1"><div class="flex flex-wrap gap-2"><span class="font-medium text-foreground">{model.displayName}</span>{#if model.availability !== "available"}<span class="text-xs text-status-tentative">{model.availability === "stale" ? t("settings.chat.models.stale") : model.availability === "deprecated" ? t("settings.chat.models.deprecated") : t("settings.chat.models.unavailable")}</span>{/if}{#if model.custom}<span class="text-xs text-muted-foreground">{t("settings.chat.models.custom")}</span>{/if}</div><div class="mt-0.5 text-xs text-muted-foreground">{t("settings.chat.models.rawId", model.id)}{#if model.contextLimit} · {t("settings.chat.models.context", formatNumber(localization.locale, model.contextLimit))}{/if}{#if model.capabilities.length > 0} · {t("settings.chat.models.capabilities", formatNumber(localization.locale, model.capabilities.length))}{/if}</div></div>
          <div class="flex items-center gap-2"><label class="flex items-center gap-2 text-xs text-muted-foreground"><input type="checkbox" checked={isVisible(model.id)} onchange={() => void toggleVisible(model.id)} />{t("settings.chat.models.visible")}</label>{#if model.custom}<button type="button" class="chat-settings-button text-destructive" disabled={busy} onclick={() => void removeCustomModel(model.id)}>{t("settings.chat.models.removeCustom")}</button>{/if}</div>
        </div>
      {/each}
    </div>
  {/if}
</section>
