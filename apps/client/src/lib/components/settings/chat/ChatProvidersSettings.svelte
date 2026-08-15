<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import Star from "@lucide/svelte/icons/star";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import type {
    JsonValue,
    ProviderFamilyMetadataRead,
    ProviderInstanceRead,
    ProviderRefreshResult,
  } from "$lib/chat/contracts";
  import { chatErrorMessage } from "$lib/chat/error-presentation";
  import { compareCompanyModels, modelCompany, modelIdAddsInformation } from "$lib/chat/model-company";
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import RedactedSensitiveText from "../RedactedSensitiveText.svelte";
  import SettingSwitch from "../SettingSwitch.svelte";
  import type { ChatProviderSetupTarget } from "../types";

  let {
    onOpenProviderSetup,
  }: {
    onOpenProviderSetup: (target: ChatProviderSetupTarget) => void;
  } = $props();

  type ProviderDisplayState = "healthy" | "attention" | "disabled" | "unconfigured";

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let selectedFamilyId = $state<string | null>(null);
  let selectedProviderId = $state<string | null>(null);
  let search = $state("");
  let busyIds = $state<string[]>([]);
  let refreshingAll = $state(false);
  let refreshResult = $state<ProviderRefreshResult | null>(null);
  let refreshedOnOpen = $state(false);
  let customOpen = $state(false);
  let customId = $state("");
  let customLabel = $state("");
  let customError = $state<string | null>(null);
  let operationError = $state<string | null>(null);
  let removeId = $state<string | null>(null);
  const families = $derived(chat.settings?.providerFamilies ?? []);
  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedFamily = $derived(
    families.find((family) => family.familyId === selectedFamilyId) ?? families[0] ?? null,
  );
  const familyProviders = $derived(
    selectedFamily
      ? providers.filter((provider) => provider.configuration.familyId === selectedFamily.familyId)
      : [],
  );
  const selectedProvider = $derived(
    familyProviders.find((entry) => entry.configuration.instanceId === selectedProviderId)
      ?? familyProviders[0]
      ?? null,
  );
  const removeProvider = $derived(
    providers.find((entry) => entry.configuration.instanceId === removeId) ?? null,
  );
  const models = $derived.by(() => {
    const query = search.trim().toLocaleLowerCase(localization.locale);
    const favorites = new Set(selectedProvider?.configuration.favoriteModelIds ?? []);
    const collator = new Intl.Collator(localization.locale);
    const familyId = selectedProvider?.configuration.familyId ?? "";
    return [...(selectedProvider?.modelCatalog?.models ?? [])]
      .filter((model) => !query
        || model.displayName.toLocaleLowerCase(localization.locale).includes(query)
        || model.id.toLocaleLowerCase(localization.locale).includes(query))
      .sort((left, right) => {
        const favoriteOrder = Number(favorites.has(right.id)) - Number(favorites.has(left.id));
        if (favoriteOrder !== 0) return favoriteOrder;
        const leftCompany = modelCompany(familyId, left);
        const rightCompany = modelCompany(familyId, right);
        return leftCompany.order - rightCompany.order
          || (leftCompany.id === rightCompany.id ? compareCompanyModels(leftCompany.id, left, right) : 0)
          || collator.compare(left.displayName, right.displayName);
      });
  });
  const acceptsCustomModels = $derived(
    selectedProvider?.configuration.familyId === "codex" && providerConfigBoolean("allowCustomModels"),
  );

  $effect(() => {
    if (families.some((family) => family.familyId === selectedFamilyId)) return;
    const healthyFamily = families.find((family) => providers.some((provider) => (
      provider.configuration.familyId === family.familyId
      && provider.configuration.enabled
      && provider.lastProbe?.state === "healthy"
    )));
    const configuredFamily = families.find((family) => providers.some(
      (provider) => provider.configuration.familyId === family.familyId,
    ));
    selectedFamilyId = healthyFamily?.familyId
      ?? configuredFamily?.familyId
      ?? families.find((family) => family.implementationStatus === "available")?.familyId
      ?? families[0]?.familyId
      ?? null;
  });

  $effect(() => {
    if (familyProviders.some((provider) => provider.configuration.instanceId === selectedProviderId)) return;
    selectedProviderId = familyProviders[0]?.configuration.instanceId ?? null;
  });

  $effect(() => {
    void chat.ensureLoaded().catch((error: unknown) => {
      operationError = errorMessage(error);
    });
  });

  $effect(() => {
    if (!chat.settings || refreshedOnOpen) return;
    refreshedOnOpen = true;
    void refreshAll();
  });

  function selectFamily(familyId: string): void {
    if (familyId === selectedFamilyId) return;
    selectedFamilyId = familyId;
    const nextProvider = providers.find((provider) => provider.configuration.familyId === familyId);
    selectedProviderId = nextProvider?.configuration.instanceId ?? null;
    resetCatalogState();
  }

  function selectProvider(instanceId: string): void {
    if (instanceId === selectedProviderId) return;
    selectedProviderId = instanceId;
    resetCatalogState();
  }

  function resetCatalogState(): void {
    search = "";
    customOpen = false;
    customError = null;
    operationError = null;
  }

  function openProviderSetup(familyId: string): void {
    onOpenProviderSetup({ mode: "create", familyId });
  }

  function setBusy(instanceId: string, busy: boolean): void {
    busyIds = busy
      ? [...new Set([...busyIds, instanceId])]
      : busyIds.filter((id) => id !== instanceId);
  }

  async function perform(instanceId: string, action: () => Promise<void>): Promise<void> {
    setBusy(instanceId, true);
    operationError = null;
    try {
      await action();
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      setBusy(instanceId, false);
    }
  }

  async function refreshAll(): Promise<void> {
    if (refreshingAll) return;
    refreshingAll = true;
    refreshResult = null;
    operationError = null;
    try {
      refreshResult = await chat.refreshAllProviders();
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      refreshingAll = false;
    }
  }

  async function refreshProvider(provider: ProviderInstanceRead): Promise<void> {
    const instanceId = provider.configuration.instanceId;
    await perform(instanceId, async () => {
      await chat.probeProvider(instanceId);
      await chat.refreshModels(instanceId);
    });
  }

  async function setEnabled(provider: ProviderInstanceRead, enabled: boolean): Promise<void> {
    const configuration = provider.configuration;
    await perform(configuration.instanceId, () => (
      chat.setProviderEnabled(configuration.instanceId, enabled)
    ));
  }

  async function confirmRemove(): Promise<void> {
    if (!removeProvider) return;
    const instanceId = removeProvider.configuration.instanceId;
    removeId = null;
    await perform(instanceId, async () => {
      const result = await chat.removeProvider(instanceId);
      if (result.credentialCleanupFailed) {
        operationError = t("settings.chat.providers.credentialCleanupFailed");
      }
    });
  }

  async function setVisible(modelId: string, shouldBeVisible: boolean): Promise<void> {
    if (!selectedProvider) return;
    const allIds = selectedProvider.modelCatalog?.models.map((model) => model.id) ?? [];
    const configured = selectedProvider.configuration.visibleModelIds;
    const current = configured.length === 0 ? allIds : configured;
    const visible = shouldBeVisible
      ? [...new Set([...current, modelId])]
      : current.filter((id) => id !== modelId);
    await perform(selectedProvider.configuration.instanceId, () => (
      chat.updateModels(
        selectedProvider.configuration.instanceId,
        visible,
        selectedProvider.configuration.favoriteModelIds,
      )
    ));
  }

  async function toggleFavorite(modelId: string): Promise<void> {
    if (!selectedProvider) return;
    const favorites = selectedProvider.configuration.favoriteModelIds.includes(modelId)
      ? selectedProvider.configuration.favoriteModelIds.filter((id) => id !== modelId)
      : [...selectedProvider.configuration.favoriteModelIds, modelId];
    await perform(selectedProvider.configuration.instanceId, () => (
      chat.updateModels(
        selectedProvider.configuration.instanceId,
        selectedProvider.configuration.visibleModelIds,
        favorites,
      )
    ));
  }

  function isVisible(modelId: string): boolean {
    if (!selectedProvider) return false;
    return selectedProvider.configuration.visibleModelIds.length === 0
      || selectedProvider.configuration.visibleModelIds.includes(modelId);
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
    return Object.fromEntries(
      Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === "string"),
    );
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
    customError = null;
    await perform(selectedProvider.configuration.instanceId, async () => {
      await chat.saveProvider({
        ...selectedProvider.configuration,
        providerConfig: { ...selectedProvider.configuration.providerConfig, value: config },
      });
      await chat.probeProvider(selectedProvider.configuration.instanceId);
      await chat.refreshModels(selectedProvider.configuration.instanceId);
      customId = "";
      customLabel = "";
      customOpen = false;
    });
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
    await perform(selectedProvider.configuration.instanceId, async () => {
      await chat.saveProvider({
        ...selectedProvider.configuration,
        visibleModelIds: selectedProvider.configuration.visibleModelIds.filter((id) => id !== modelId),
        favoriteModelIds: selectedProvider.configuration.favoriteModelIds.filter((id) => id !== modelId),
        providerConfig: { ...selectedProvider.configuration.providerConfig, value: config },
      });
      await chat.probeProvider(selectedProvider.configuration.instanceId);
      await chat.refreshModels(selectedProvider.configuration.instanceId);
    });
  }

  function providerStateLabel(provider: ProviderInstanceRead): string {
    if (!provider.configuration.enabled) return t("settings.chat.providers.disabled");
    if (provider.lastProbe?.state === "healthy") return t("settings.chat.providers.healthy");
    if (provider.lastProbe?.state === "authentication_required") {
      return t("settings.chat.providers.authenticationRequired");
    }
    if (provider.lastProbe?.state === "unsupported_version") return t("settings.chat.providers.unsupported");
    if (!provider.lastProbe) return t("settings.chat.providers.neverChecked");
    return t("settings.chat.providers.unavailable");
  }

  function providerState(provider: ProviderInstanceRead): ProviderDisplayState {
    if (!provider.configuration.enabled) return "disabled";
    return provider.lastProbe?.state === "healthy" ? "healthy" : "attention";
  }

  function providersForFamily(familyId: string): ProviderInstanceRead[] {
    return providers.filter((provider) => provider.configuration.familyId === familyId);
  }

  function familyState(family: ProviderFamilyMetadataRead): ProviderDisplayState {
    if (family.implementationStatus !== "available") return "disabled";
    const instances = providersForFamily(family.familyId);
    if (instances.length === 0) return "unconfigured";
    if (instances.some((provider) => providerState(provider) === "healthy")) return "healthy";
    if (instances.every((provider) => providerState(provider) === "disabled")) return "disabled";
    return "attention";
  }

  function familyStateLabel(family: ProviderFamilyMetadataRead): string {
    if (family.implementationStatus !== "available") return t("settings.chat.providers.notAvailable");
    const state = familyState(family);
    if (state === "unconfigured") return t("settings.chat.providers.notConfigured");
    if (state === "healthy") return t("settings.chat.providers.ready");
    if (state === "disabled") return t("settings.chat.providers.disabled");
    return t("settings.chat.providers.needsAttention");
  }

  function refreshSummary(result: ProviderRefreshResult): string {
    return t(
      "settings.chat.providers.refreshSummary",
      formatNumber(localization.locale, result.providersDiscovered),
      formatNumber(localization.locale, result.issues),
    );
  }

  function errorMessage(error: unknown): string {
    return chatErrorMessage(error, t("settings.chat.providers.operationFailed"));
  }
</script>

<section class="provider-settings" data-chat-settings-subsection="providers">
  <header class="page-heading">
    <h2>{t("settings.chat.providers.heading")}</h2>
    <button type="button" class="refresh-all" disabled={refreshingAll} onclick={() => void refreshAll()}>
      <RefreshCw size={13} class={refreshingAll ? "animate-spin" : undefined} />
      {refreshingAll ? t("settings.chat.providers.scanning") : t("settings.chat.providers.refreshAll")}
    </button>
  </header>

  {#if refreshResult && (refreshResult.providersDiscovered > 0 || refreshResult.issues > 0)}
    <p class="refresh-summary" class:attention={refreshResult.issues > 0}>{refreshSummary(refreshResult)}</p>
  {/if}
  {#if operationError}<p role="alert" class="operation-error">{operationError}</p>{/if}

  {#if chat.loading}
    <p class="families-empty">{t("common.loading")}</p>
  {:else if families.length === 0}
    <p class="families-empty">{t("settings.chat.providers.noIntegrations")}</p>
  {:else}
    <div class="family-tabs" role="tablist" aria-label={t("settings.chat.providers.familyTabs") }>
      {#each families as family (family.familyId)}
        {@const active = family.familyId === selectedFamily?.familyId}
        {@const state = familyState(family)}
        <button
          type="button"
          role="tab"
          aria-selected={active}
          class:active
          onclick={() => selectFamily(family.familyId)}
        >
          <ChatModelAvatar familyId={family.familyId} label={family.displayName} size={27} />
          <span>
            <strong>{family.displayName}</strong>
            <small data-state={state}><i></i>{familyStateLabel(family)}</small>
          </span>
        </button>
      {/each}
    </div>

    {#if selectedFamily}
      <div class="family-workspace" role="tabpanel">
        <section class="configurations-section">
          <h3>{t("settings.chat.providers.configurations")}</h3>

          {#if familyProviders.length === 0}
            <p class="configuration-empty">{t("settings.chat.providers.noConfigurations", selectedFamily.displayName)}</p>
          {:else}
            <div class="configuration-list">
              {#each familyProviders as provider (provider.configuration.instanceId)}
                {@const instanceId = provider.configuration.instanceId}
                {@const active = instanceId === selectedProvider?.configuration.instanceId}
                {@const busy = refreshingAll || busyIds.includes(instanceId)}
                <div class="configuration-row">
                  <div class="configuration-identity">
                    <button
                      type="button"
                      class="configuration-select"
                      aria-pressed={active}
                      onclick={() => selectProvider(instanceId)}
                    >
                      <span class="configuration-name">
                        <strong>{provider.configuration.label}</strong>
                        {#if active && familyProviders.length > 1}
                          <Check size={13} strokeWidth={2.5} class="shrink-0 text-foreground" />
                        {/if}
                      </span>
                    </button>
                    <small>
                      <span class="configuration-state" data-state={providerState(provider)}>
                        <i></i>{providerStateLabel(provider)}
                      </span>
                      {#if provider.lastProbe?.version}
                        <span>{t("settings.chat.providers.versionValue", provider.lastProbe.version)}</span>
                      {/if}
                      {#if provider.lastProbe?.accountLabel}
                        <RedactedSensitiveText
                          value={provider.lastProbe.accountLabel}
                          revealLabel={t("settings.chat.providers.revealAccount")}
                          hideLabel={t("settings.chat.providers.hideAccount")}
                        />
                      {/if}
                    </small>
                  </div>

                  <div class="configuration-actions">
                    <SettingSwitch
                      checked={provider.configuration.enabled}
                      ariaLabel={provider.configuration.enabled
                        ? t("settings.chat.providers.disable")
                        : t("settings.chat.providers.enable")}
                      disabled={busy}
                      onChange={(enabled) => void setEnabled(provider, enabled)}
                    />
                    <button
                      type="button"
                      class="icon-action"
                      aria-label={t("settings.chat.providers.configure")}
                      title={t("settings.chat.providers.configure")}
                      onclick={() => onOpenProviderSetup({ mode: "edit", instanceId })}
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      type="button"
                      class="icon-action"
                      disabled={busy}
                      aria-label={t("settings.chat.providers.refresh")}
                      title={t("settings.chat.providers.refresh")}
                      onclick={() => void refreshProvider(provider)}
                    >
                      <RefreshCw size={14} class={busy ? "animate-spin" : undefined} />
                    </button>
                    <button
                      type="button"
                      class="icon-action destructive"
                      disabled={busy}
                      aria-label={t("settings.chat.providers.remove")}
                      title={t("settings.chat.providers.remove")}
                      onclick={() => { removeId = instanceId; }}
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

          {#if selectedFamily.implementationStatus === "available"}
            <button type="button" class="add-configuration" onclick={() => openProviderSetup(selectedFamily.familyId)}>
              <Plus size={13} />
              {familyProviders.length === 0
                ? t("settings.chat.providers.setUp", selectedFamily.displayName)
                : t("settings.chat.providers.addAnother", selectedFamily.displayName)}
            </button>
          {:else}
            <p class="integration-unavailable">
              {selectedFamily.unavailableReason ?? t("settings.chat.providers.notAvailable")}
            </p>
          {/if}
        </section>

        {#if selectedProvider}
          {@const selectedBusy = refreshingAll || busyIds.includes(selectedProvider.configuration.instanceId)}
          {#if selectedProvider.lastProbe?.state === "authentication_required"}
            <p class="provider-notice">{t("settings.chat.providers.loginHelp")}</p>
          {/if}

          <section class="catalog-section">
            <header class="catalog-heading">
              <div>
                <h3>{t("settings.chat.models.heading")}</h3>
                <span>{t("settings.chat.models.modelCount", selectedProvider.modelCatalog?.models.length ?? 0)}</span>
              </div>
              {#if acceptsCustomModels}
                <button type="button" class="compact-action" disabled={selectedBusy} onclick={() => { customOpen = !customOpen; }}>
                  <Plus size={13} />
                  {t("settings.chat.models.addCustom")}
                </button>
              {/if}
            </header>

            <label class="model-search">
              <Search size={14} />
              <input
                type="search"
                bind:value={search}
                aria-label={t("settings.chat.models.search")}
                placeholder={t("settings.chat.models.search")}
              />
            </label>

            {#if selectedProvider.modelCatalog?.stale}
              <p class="catalog-warning">{t("settings.chat.models.catalogStale")}</p>
            {/if}

            {#if customOpen && acceptsCustomModels}
              <section class="custom-model-editor">
                <label>
                  <span>{t("settings.chat.models.customId")}</span>
                  <input bind:value={customId} />
                </label>
                <label>
                  <span>{t("settings.chat.models.customLabel")}</span>
                  <input bind:value={customLabel} />
                </label>
                <button type="button" class="compact-action" disabled={selectedBusy} onclick={() => void saveCustomModel()}>
                  {t("settings.chat.models.saveCustom")}
                </button>
                {#if customError}<p role="alert">{customError}</p>{/if}
              </section>
            {/if}

            {#if models.length === 0}
              <p class="models-empty">
                {search.trim() ? t("settings.chat.models.noSearchResults") : t("settings.chat.models.empty")}
              </p>
            {:else}
              <div class="model-list">
                {#each models as model (model.id)}
                  {@const company = modelCompany(selectedProvider.configuration.familyId, model)}
                  <div class="model-row">
                    <ChatModelAvatar familyId={company.iconFamilyId} label={company.name} size={25} />
                    <div class="model-identity">
                      <span class="model-name">{model.displayName}</span>
                      <div class="model-meta">
                        {#if modelIdAddsInformation(model)}<span class="model-id">{model.id}</span>{/if}
                        {#if model.contextLimit}
                          <span>{t("settings.chat.models.context", formatNumber(localization.locale, model.contextLimit))}</span>
                        {/if}
                        {#if model.availability !== "available"}
                          <span class="text-status-tentative">
                            {model.availability === "stale"
                              ? t("settings.chat.models.stale")
                              : model.availability === "deprecated"
                                ? t("settings.chat.models.deprecated")
                                : t("settings.chat.models.unavailable")}
                          </span>
                        {/if}
                        {#if model.custom}<span>{t("settings.chat.models.custom")}</span>{/if}
                      </div>
                    </div>
                    <div class="model-actions">
                      <SettingSwitch
                        checked={isVisible(model.id)}
                        ariaLabel={isVisible(model.id) ? t("settings.chat.models.hide") : t("settings.chat.models.show")}
                        disabled={selectedBusy}
                        onChange={(visible) => void setVisible(model.id, visible)}
                      />
                      <button
                        type="button"
                        class="favorite-action"
                        aria-label={t("settings.chat.models.favorite")}
                        aria-pressed={selectedProvider.configuration.favoriteModelIds.includes(model.id)}
                        onclick={() => void toggleFavorite(model.id)}
                      >
                        <Star
                          size={14}
                          fill={selectedProvider.configuration.favoriteModelIds.includes(model.id) ? "currentColor" : "none"}
                        />
                      </button>
                      {#if model.custom}
                        <button
                          type="button"
                          class="remove-custom"
                          disabled={selectedBusy}
                          onclick={() => void removeCustomModel(model.id)}
                        >
                          {t("settings.chat.models.removeCustom")}
                        </button>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </section>
        {/if}
      </div>
    {/if}
  {/if}
</section>

{#if removeProvider}
  <ConfirmDialog
    title={t("settings.chat.providers.removeTitle")}
    message={t("settings.chat.providers.removeMessage", removeProvider.configuration.label)}
    confirmLabel={t("settings.chat.providers.remove")}
    cancelLabel={t("chat.cancel")}
    onConfirm={() => void confirmRemove()}
    onCancel={() => { removeId = null; }}
  />
{/if}

<style>
  .provider-settings { display:grid; gap:0.8rem; }
  .page-heading { display:flex; min-width:0; align-items:center; justify-content:space-between; gap:0.75rem; padding-inline:0.25rem; }
  .page-heading h2 { font-size:calc(0.866667rem * var(--type-scale)); font-weight:600; }
  .refresh-all,.compact-action { display:inline-flex; height:1.75rem; align-items:center; justify-content:center; gap:0.35rem; border:1px solid var(--border); border-radius:0.375rem; background:var(--card); padding-inline:0.6rem; color:var(--foreground); font-size:calc(0.72rem * var(--type-scale)); font-weight:500; }
  .refresh-all:hover:not(:disabled),.compact-action:hover:not(:disabled) { background:var(--accent); }
  .refresh-summary,.operation-error { padding-inline:0.25rem; font-size:calc(0.7rem * var(--type-scale)); }
  .refresh-summary { color:var(--muted-foreground); }
  .refresh-summary.attention,.catalog-warning,.provider-notice,.integration-unavailable { color:var(--status-tentative); }
  .operation-error,.custom-model-editor p { color:var(--destructive); }
  .families-empty { border-block:1px solid var(--border); padding:2rem 0.25rem; color:var(--muted-foreground); font-size:calc(0.78rem * var(--type-scale)); text-align:center; }
  .family-tabs { display:flex; flex-wrap:wrap; gap:0.25rem; border:1px solid var(--border); border-radius:0.375rem; padding:0.25rem; }
  .family-tabs > button { display:flex; min-width:6.2rem; min-height:2.9rem; flex:1 1 6.2rem; align-items:center; gap:0.4rem; border-radius:0.25rem; padding:0.35rem 0.4rem; color:var(--muted-foreground); text-align:left; }
  .family-tabs > button:hover { background:color-mix(in srgb,var(--accent) 55%,transparent); color:var(--foreground); }
  .family-tabs > button.active { background:var(--accent); color:var(--accent-foreground); }
  .family-tabs > button > span { display:grid; min-width:0; gap:0.08rem; }
  .family-tabs strong { overflow:hidden; font-size:calc(0.74rem * var(--type-scale)); font-weight:600; text-overflow:ellipsis; white-space:nowrap; }
  .family-tabs small { display:flex; min-width:0; align-items:center; gap:0.3rem; overflow:hidden; font-size:calc(0.62rem * var(--type-scale)); font-weight:400; text-overflow:ellipsis; white-space:nowrap; }
  .family-tabs small i,.configuration-state i { width:0.38rem; height:0.38rem; flex:0 0 auto; border-radius:999px; background:var(--status-tentative); }
  .family-tabs small[data-state="healthy"] i,.configuration-state[data-state="healthy"] i { background:var(--action-confirm); }
  .family-tabs small[data-state="disabled"] i,.configuration-state[data-state="disabled"] i { background:var(--muted-foreground); opacity:0.7; }
  .family-tabs small[data-state="unconfigured"] i { border:1px solid currentColor; background:transparent; }
  .family-workspace { min-width:0; }
  .configurations-section { display:grid; gap:0.55rem; padding-top:0.2rem; }
  .configurations-section > h3 { padding-inline:0.25rem; font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .configuration-list { display:grid; border-top:1px solid var(--border); }
  .configuration-row { display:grid; grid-template-columns:minmax(12rem,1fr) auto; align-items:center; gap:0.5rem; border-bottom:1px solid var(--border); padding:0.55rem 0.25rem; }
  .configuration-select { display:block; min-width:0; text-align:left; }
  .configuration-identity { display:grid; min-width:0; gap:0.12rem; }
  .configuration-name { display:flex; min-width:0; align-items:center; gap:0.35rem; }
  .configuration-name > strong { overflow:hidden; color:var(--foreground); font-size:calc(0.77rem * var(--type-scale)); font-weight:600; text-overflow:ellipsis; white-space:nowrap; }
  .configuration-identity > small { display:flex; min-width:0; flex-wrap:wrap; gap:0.25rem 0.65rem; color:var(--muted-foreground); font-size:calc(0.64rem * var(--type-scale)); }
  .configuration-state { display:inline-flex; align-items:center; gap:0.3rem; }
  .configuration-actions { display:flex; flex-shrink:0; align-items:center; gap:0.3rem; }
  .icon-action,.favorite-action { display:inline-grid; width:1.75rem; height:1.75rem; place-items:center; border:1px solid var(--border); border-radius:0.375rem; background:var(--card); color:var(--muted-foreground); }
  .icon-action:hover:not(:disabled),.favorite-action:hover { background:var(--accent); color:var(--foreground); }
  .icon-action.destructive,.remove-custom { color:var(--destructive); }
  button:disabled { cursor:not-allowed; opacity:0.5; }
  .configuration-empty { border-block:1px solid var(--border); padding:0.85rem 0.25rem; color:var(--muted-foreground); font-size:calc(0.72rem * var(--type-scale)); }
  .add-configuration { display:inline-flex; width:max-content; min-height:1.8rem; align-items:center; gap:0.35rem; margin-left:0.15rem; border-radius:0.375rem; padding:0.25rem 0.45rem; color:var(--foreground); font-size:calc(0.72rem * var(--type-scale)); font-weight:500; }
  .add-configuration:hover { background:var(--accent); }
  .integration-unavailable,.provider-notice,.catalog-warning { padding:0.45rem 0.25rem; font-size:calc(0.7rem * var(--type-scale)); }
  .catalog-section { min-width:0; margin-top:1.05rem; border-top:1px solid var(--border); padding-top:0.8rem; }
  .catalog-heading { display:flex; align-items:center; justify-content:space-between; gap:0.6rem; padding:0 0.25rem 0.65rem; }
  .catalog-heading > div { display:flex; min-width:0; align-items:baseline; gap:0.5rem; }
  .catalog-heading h3 { font-size:calc(0.8rem * var(--type-scale)); font-weight:600; }
  .catalog-heading span { color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .model-search { display:grid; grid-template-columns:auto minmax(0,1fr); align-items:center; gap:0.45rem; border-block:1px solid var(--border); padding:0.45rem 0.25rem; color:var(--muted-foreground); }
  .model-search input { min-width:0; background:transparent; color:var(--foreground); font-size:calc(0.75rem * var(--type-scale)); outline:none; }
  .model-search:focus-within { border-color:var(--ring); color:var(--foreground); }
  .custom-model-editor { display:grid; grid-template-columns:minmax(0,1fr) minmax(0,1fr) auto; align-items:end; gap:0.6rem; border-bottom:1px solid var(--border); padding:0.65rem 0.25rem; }
  .custom-model-editor label { display:grid; min-width:0; gap:0.3rem; color:var(--muted-foreground); font-size:calc(0.68rem * var(--type-scale)); }
  .custom-model-editor input { min-width:0; height:2rem; border:1px solid var(--border); border-radius:0.375rem; background:var(--background); padding:0.3rem 0.55rem; color:var(--foreground); outline:none; }
  .custom-model-editor input:focus { border-color:var(--ring); }
  .custom-model-editor p { grid-column:1/-1; font-size:calc(0.68rem * var(--type-scale)); }
  .model-list { display:grid; }
  .model-row { display:grid; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:0.65rem; border-bottom:1px solid var(--border); padding:0.62rem 0.25rem; }
  .model-row:last-child { border-bottom:0; }
  .model-identity { display:grid; min-width:0; gap:0.16rem; }
  .model-name { overflow:hidden; color:var(--foreground); font-size:calc(0.8rem * var(--type-scale)); font-weight:550; line-height:1.1rem; text-overflow:ellipsis; white-space:nowrap; }
  .model-meta { display:flex; min-width:0; flex-wrap:wrap; gap:0.15rem 0.42rem; color:var(--muted-foreground); font-size:calc(0.66rem * var(--type-scale)); line-height:1rem; }
  .model-meta > span + span::before { margin-right:0.42rem; content:"·"; }
  .model-id { overflow:hidden; max-width:22rem; font-family:var(--font-mono,monospace); text-overflow:ellipsis; white-space:nowrap; }
  .model-actions { display:flex; flex-shrink:0; align-items:center; gap:0.45rem; }
  .favorite-action[aria-pressed="true"] { color:var(--status-tentative); }
  .remove-custom { min-height:1.8rem; border-radius:0.375rem; padding-inline:0.45rem; font-size:calc(0.68rem * var(--type-scale)); }
  .remove-custom:hover:not(:disabled) { background:var(--accent); }
  .models-empty { padding:0.9rem 0.25rem; color:var(--muted-foreground); font-size:calc(0.75rem * var(--type-scale)); }
  :global(.dark) .refresh-all,
  :global(.dark) .compact-action,
  :global(.dark) .icon-action,
  :global(.dark) .favorite-action { background:transparent; }
  @media (max-width:540px) {
    .configuration-row { grid-template-columns:1fr; }
    .configuration-actions { grid-column:1; grid-row:auto; justify-content:flex-end; }
    .custom-model-editor { grid-template-columns:1fr; align-items:stretch; }
    .custom-model-editor p { grid-column:auto; }
    .model-actions { gap:0.2rem; }
    .model-id { max-width:10rem; }
  }
</style>
