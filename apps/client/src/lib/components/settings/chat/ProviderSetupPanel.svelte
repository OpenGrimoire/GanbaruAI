<script lang="ts">
  import { onMount, tick } from "svelte";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Check from "@lucide/svelte/icons/check";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import * as chatApi from "$lib/api/chat";
  import type { JsonValue, ProviderInstanceConfig, ProviderSetupTestRead } from "$lib/chat/contracts";
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import {
    createProviderSetupDraft,
    PROVIDER_ACCENT_COLORS,
    providerConfigurationFromDraft,
    providerInstanceIdFromLabel,
    validateProviderSetup,
    type ProviderEnvironmentDraft,
    type ProviderSetupDraft,
    type ProviderSetupStep,
  } from "$lib/chat/provider-setup";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import type { ChatProviderSetupTarget } from "../types";
  import CustomSelect from "../CustomSelect.svelte";
  import ToggleSetting from "../ToggleSetting.svelte";

  let {
    target,
    onCancel,
    compactLayout = false,
    iconRailLayout = false,
    onScrollContainerChange = () => {},
    onScrollbarInsetsChange = () => {},
  }: {
    target: ChatProviderSetupTarget;
    onCancel: () => void;
    compactLayout?: boolean;
    iconRailLayout?: boolean;
    onScrollContainerChange?: (element: HTMLElement | undefined) => void;
    onScrollbarInsetsChange?: (insets: { top: number; bottom: number }) => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const chat = getChat();
  let draft = $state<ProviderSetupDraft>(createProviderSetupDraft());
  let scrollElement: HTMLDivElement | undefined = $state();
  let initialized = $state(false);
  let saving = $state(false);
  let testing = $state(false);
  let operationError = $state<string | null>(null);
  let testResult = $state<ProviderSetupTestRead | null>(null);
  let testSucceeded = $state(false);
  let advancedOpen = $state(false);
  let revealSecrets = $state<Record<string, boolean>>({});
  let pendingSecrets = $state<Record<string, string>>({});
  let nextRowId = 1;
  const editingId = $derived(target.mode === "edit" ? target.instanceId : null);
  const existingIds = $derived(new Set(chat.settings?.providerInstances.map((entry) => entry.configuration.instanceId) ?? []));
  const validation = $derived(validateProviderSetup(draft, existingIds, editingId));
  const stepOrder: ProviderSetupStep[] = ["provider", "identity", "connection"];
  const activeStepIndex = $derived(stepOrder.indexOf(draft.step));
  const family = $derived(chat.settings?.providerFamilies.find((entry) => entry.familyId === draft.familyId) ?? null);
  const openCodePassword = $derived(draft.environment.find((row) => row.name === "OPENCODE_SERVER_PASSWORD") ?? null);
  const openCodeModeOptions = $derived([
    { value: "local", label: t("settings.chat.setup.localMode") },
    { value: "external", label: t("settings.chat.setup.externalMode") },
  ]);
  const environmentValueTypeOptions = $derived([
    { value: "text", label: t("settings.chat.setup.textValue") },
    { value: "secret", label: t("settings.chat.setup.secretValue") },
    { value: "inherit", label: t("settings.chat.setup.inheritedValue") },
  ]);

  onMount(() => {
    onScrollContainerChange(scrollElement);
    onScrollbarInsetsChange({ top: 64, bottom: 64 });
    void initialize();
    return () => onScrollContainerChange(undefined);
  });

  async function initialize(): Promise<void> {
    await chat.ensureLoaded();
    if (target.mode === "edit") {
      const provider = chat.settings?.providerInstances.find((entry) => entry.configuration.instanceId === target.instanceId);
      if (provider) draft = draftFromConfiguration(provider.configuration);
    }
    initialized = true;
    await tick();
    scrollElement?.focus();
  }

  function selectFamily(familyId: string, executable: string): void {
    draft.familyId = familyId;
    draft.executable = executable;
    draft.providerConfig = defaultProviderConfig(familyId);
    draft.step = "identity";
  }

  function updateLabel(value: string): void {
    const previousGenerated = providerInstanceIdFromLabel(draft.label);
    draft.label = value;
    if (!draft.instanceId || draft.instanceId === previousGenerated) {
      draft.instanceId = providerInstanceIdFromLabel(value);
    }
  }

  function goBack(): void {
    if (activeStepIndex <= 0) onCancel();
    else draft.step = stepOrder[activeStepIndex - 1];
  }

  function goNext(): void {
    if (draft.step === "provider" && !draft.familyId) return;
    if (draft.step === "identity" && (validation.fields.label || validation.fields.instanceId || validation.fields.accentColor)) return;
    draft.step = stepOrder[Math.min(stepOrder.length - 1, activeStepIndex + 1)];
  }

  function addArgument(): void {
    draft.launchArguments = [...draft.launchArguments, ""];
  }

  function addEnvironment(): void {
    const row: ProviderEnvironmentDraft = {
      key: `row-${nextRowId++}`,
      name: "",
      valueType: "text",
      value: "",
      credentialReference: "",
    };
    draft.environment = [...draft.environment, row];
  }

  function setEnvironmentValueType(row: ProviderEnvironmentDraft, value: string): void {
    if (value === "text" || value === "secret" || value === "inherit") {
      row.valueType = value;
    }
  }

  function removeEnvironment(key: string): void {
    draft.environment = draft.environment.filter((row) => row.key !== key);
    delete pendingSecrets[key];
  }

  function setOpenCodeMode(mode: "local" | "external"): void {
    const current = draft.providerConfig.value;
    const value: { [key: string]: JsonValue } = typeof current === "object" && current !== null && !Array.isArray(current)
      ? { ...current, mode }
      : { mode };
    if (mode === "local") {
      delete value.serverUrl;
      delete value.endpoint;
      delete value.allowInsecureExternalHttp;
      delete value.confirmExternalWorkspaceAccess;
    } else if (typeof value.endpoint === "string" && typeof value.serverUrl !== "string") {
      value.serverUrl = value.endpoint;
      delete value.endpoint;
    }
    draft.providerConfig = { schemaVersion: 1, value };
    if (mode === "local" && openCodePassword) {
      const row = openCodePassword;
      if (row.credentialReference) {
        void removeSecret(row).catch((error: unknown) => { operationError = errorMessage(error); });
      }
      removeEnvironment(row.key);
    }
  }

  function addOpenCodePassword(): void {
    if (openCodePassword) return;
    draft.environment = [...draft.environment, {
      key: `row-${nextRowId++}`,
      name: "OPENCODE_SERVER_PASSWORD",
      valueType: "secret",
      value: "",
      credentialReference: "",
    }];
  }

  async function storeSecret(row: ProviderEnvironmentDraft): Promise<void> {
    const secret = pendingSecrets[row.key]?.trim();
    if (!secret || !row.name) return;
    const reference = `provider:${draft.instanceId}:environment:${row.name}`;
    await chatApi.replaceChatCredential(reference, secret);
    row.credentialReference = reference;
    pendingSecrets[row.key] = "";
  }

  async function removeSecret(row: ProviderEnvironmentDraft): Promise<void> {
    if (row.credentialReference) await chatApi.removeChatCredential(row.credentialReference);
    row.credentialReference = "";
  }

  async function pickExecutable(): Promise<void> {
    const selected = await chatApi.pickChatProviderExecutable(t("settings.chat.setup.chooseExecutable"));
    if (selected) draft.executable = selected;
  }

  async function pickHome(): Promise<void> {
    const selected = await chatApi.pickChatProviderHome(t("settings.chat.setup.chooseProviderHome"));
    if (selected) draft.providerHome = selected;
  }

  async function testSetup(): Promise<void> {
    operationError = null;
    testResult = null;
    testSucceeded = false;
    if (!validation.valid) return;
    testing = true;
    try {
      const configuration = providerConfigurationFromDraft(draft);
      testResult = await chat.testProvider(configuration);
      testSucceeded = testResult.probe.state === "healthy" || testResult.probe.state === "authentication_required";
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      testing = false;
    }
  }

  async function save(): Promise<void> {
    operationError = null;
    if (!validation.valid) return;
    saving = true;
    try {
      await chat.saveProvider(providerConfigurationFromDraft(draft));
      onCancel();
    } catch (error: unknown) {
      operationError = errorMessage(error);
    } finally {
      saving = false;
    }
  }

  function providerConfigValue(key: string): string {
    const value = draft.providerConfig.value;
    return typeof value === "object" && value !== null && !Array.isArray(value) && typeof value[key] === "string"
      ? value[key]
      : "";
  }

  function openCodeMode(): "local" | "external" {
    const mode = providerConfigValue("mode");
    if (mode === "external") return "external";
    if (mode === "local") return "local";
    return openCodeServerUrl() ? "external" : "local";
  }

  function openCodeServerUrl(): string {
    return providerConfigValue("serverUrl") || providerConfigValue("endpoint");
  }

  function openCodeUsesInsecureExternalHttp(): boolean {
    try {
      const url = new URL(openCodeServerUrl());
      const hostname = url.hostname.toLowerCase();
      const loopback = hostname === "localhost" || hostname === "[::1]" || /^127(?:\.\d{1,3}){3}$/.test(hostname);
      return url.protocol === "http:" && !loopback;
    } catch {
      return false;
    }
  }

  function providerConfigBoolean(key: string): boolean {
    const value = draft.providerConfig.value;
    return typeof value === "object" && value !== null && !Array.isArray(value) && value[key] === true;
  }

  function setProviderConfigValue(key: string, value: string): void {
    const current = draft.providerConfig.value;
    draft.providerConfig = {
      schemaVersion: 1,
      value: { ...(typeof current === "object" && current !== null && !Array.isArray(current) ? current : {}), [key]: value || null },
    };
  }

  function setProviderConfigBoolean(key: string, value: boolean): void {
    const current = draft.providerConfig.value;
    draft.providerConfig = {
      schemaVersion: 1,
      value: { ...(typeof current === "object" && current !== null && !Array.isArray(current) ? current : {}), [key]: value },
    };
  }

  function fieldError(field: string): string | null {
    const error = validation.fields[field];
    if (!error) return null;
    if (field === "familyId") return t("settings.chat.setup.validation.provider");
    if (field === "label") return t("settings.chat.setup.validation.label");
    if (field === "instanceId") {
      return error.includes("already")
        ? t("settings.chat.setup.validation.duplicateInstanceId")
        : t("settings.chat.setup.validation.instanceId");
    }
    if (field === "accentColor") return t("settings.chat.setup.validation.accent");
    if (field === "executable") return t("settings.chat.setup.validation.executable");
    if (field.startsWith("launchArguments")) return t("settings.chat.setup.validation.arguments");
    if (field.endsWith(".name")) return t("settings.chat.setup.validation.environmentName");
    if (field.endsWith(".value")) {
      return error.includes("secret")
        ? t("settings.chat.setup.validation.secret")
        : t("settings.chat.setup.validation.environmentValue");
    }
    if (field === "providerConfig.endpoint" || field === "providerConfig.serverUrl") return t("settings.chat.setup.validation.endpoint");
    if (field === "providerConfig.allowInsecureExternalHttp") return t("settings.chat.setup.validation.insecureExternalHttp");
    if (field === "providerConfig.confirmExternalWorkspaceAccess") return t("settings.chat.setup.validation.externalWorkspaceAccess");
    return t("settings.chat.setup.validation.providerConfig");
  }

  function draftFromConfiguration(configuration: ProviderInstanceConfig): ProviderSetupDraft {
    const environment: ProviderEnvironmentDraft[] = [];
    for (const [name, value] of Object.entries(configuration.environment)) {
      environment.push({ key: `row-${nextRowId++}`, name, valueType: value === `inherit:${name}` ? "inherit" : "text", value, credentialReference: "" });
    }
    for (const [name, reference] of Object.entries(configuration.credentialReferences)) {
      environment.push({ key: `row-${nextRowId++}`, name, valueType: "secret", value: "", credentialReference: reference });
    }
    return {
      step: "identity",
      familyId: configuration.familyId,
      label: configuration.label,
      instanceId: configuration.instanceId,
      accentColor: configuration.accentColor ?? PROVIDER_ACCENT_COLORS[0],
      executable: configuration.executable,
      providerHome: configuration.providerHome ?? "",
      launchArguments: [...configuration.launchArguments],
      environment,
      providerConfig: configuration.providerConfig,
    };
  }

  function defaultProviderConfig(familyId: string): ProviderSetupDraft["providerConfig"] {
    if (familyId === "codex") return { schemaVersion: 1, value: { refreshMcpBeforeTurn: false, allowCustomModels: false, customModelIds: [], customModelLabels: {} } };
    if (familyId === "opencode") return { schemaVersion: 1, value: { mode: "local" } };
    return { schemaVersion: 1, value: {} };
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function familyDescription(familyId: string): string {
    if (familyId === "codex") return t("settings.chat.setup.familyDescriptions.codex");
    if (familyId === "claude") return t("settings.chat.setup.familyDescriptions.claude");
    if (familyId === "cursor") return t("settings.chat.setup.familyDescriptions.cursor");
    return t("settings.chat.setup.familyDescriptions.opencode");
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="flex min-h-16 shrink-0 items-center gap-3 border-b border-border px-4">
    <button type="button" class="setup-icon-button" aria-label={t("settings.chat.setup.back")} onclick={goBack}><ArrowLeft size={16} /></button>
    <div class="min-w-0 flex-1">
      <h2 class="truncate text-[0.933333rem] font-semibold text-foreground">{target.mode === "edit" ? t("settings.chat.setup.editTitle") : t("settings.chat.setup.title")}</h2>
      <div class="mt-1 flex gap-2" aria-label={t("settings.chat.setup.stepsLabel")}>
        {#each stepOrder as step, index}
          <span class="flex items-center gap-1 text-[0.666667rem] {index === activeStepIndex ? 'text-foreground' : 'text-muted-foreground'}">
            <span class="flex size-4 items-center justify-center rounded-full border border-current">{index < activeStepIndex ? "✓" : index + 1}</span>
            {step === "provider" ? t("settings.chat.setup.providerStep") : step === "identity" ? t("settings.chat.setup.identityStep") : t("settings.chat.setup.connectionStep")}
          </span>
        {/each}
      </div>
    </div>
    <button type="button" class="setup-icon-button" aria-label={t("settings.close")} onclick={onCancel}><X size={16} /></button>
  </header>

  <div bind:this={scrollElement} tabindex="-1" class="hide-scrollbar min-h-0 flex-1 overflow-y-auto p-4 outline-none {compactLayout ? 'px-3' : iconRailLayout ? 'px-5' : 'px-8'}">
    {#if !initialized}
      <div class="py-10 text-center text-sm text-muted-foreground">{t("common.loading")}</div>
    {:else if draft.step === "provider"}
      <section class="mx-auto flex max-w-2xl flex-col gap-4">
        <div><h3 class="text-base font-semibold">{t("settings.chat.setup.chooseProvider")}</h3><p class="mt-1 text-sm text-muted-foreground">{t("settings.chat.setup.connectionDescription")}</p></div>
        <div class="grid gap-3 sm:grid-cols-2">
          {#each chat.settings?.providerFamilies ?? [] as providerFamily}
            <button type="button" class="rounded-lg border border-border bg-card p-4 text-left hover:bg-accent/50" onclick={() => selectFamily(providerFamily.familyId, providerFamily.defaultExecutableCandidates[0] ?? "")}>
              <span class="flex items-center gap-2 font-semibold text-foreground"><ChatModelAvatar familyId={providerFamily.familyId} label={providerFamily.displayName} size={24} /><span>{providerFamily.displayName}</span></span>
              <span class="mt-2 block text-[0.8rem] text-muted-foreground">{familyDescription(providerFamily.familyId)}</span>
              {#if providerFamily.implementationStatus !== "available"}<span class="mt-2 block text-[0.733333rem] text-status-tentative">{providerFamily.unavailableReason}</span>{/if}
            </button>
          {/each}
        </div>
      </section>
    {:else if draft.step === "identity"}
      <section class="mx-auto flex max-w-xl flex-col gap-4">
        <div><h3 class="text-base font-semibold">{t("settings.chat.setup.identityStep")}</h3><p class="mt-1 text-sm text-muted-foreground">{t("settings.chat.setup.identityDescription")}</p></div>
        <label class="setup-field"><span>{t("settings.chat.setup.label")}</span><input value={draft.label} oninput={(event) => updateLabel(event.currentTarget.value)} />{#if fieldError("label")}<small>{fieldError("label")}</small>{/if}</label>
        <label class="setup-field"><span>{t("settings.chat.setup.instanceId")}</span><input bind:value={draft.instanceId} disabled={target.mode === "edit"} />{#if fieldError("instanceId")}<small>{fieldError("instanceId")}</small>{/if}</label>
        {#if draft.familyId !== "cursor"}
          <label class="setup-field"><span>{t("settings.chat.setup.providerHome")}</span><div class="flex gap-2"><input class="min-w-0 flex-1" bind:value={draft.providerHome} /><button type="button" class="setup-icon-button border border-border" aria-label={t("settings.chat.setup.chooseProviderHome")} onclick={() => void pickHome()}><FolderOpen size={15} /></button></div></label>
        {/if}
        <fieldset class="setup-field"><legend>{t("settings.chat.setup.accent")}</legend><div class="flex gap-2">{#each PROVIDER_ACCENT_COLORS as color}<button type="button" aria-label={color} aria-pressed={draft.accentColor === color} class="flex size-8 items-center justify-center rounded-full border-2 {draft.accentColor === color ? 'border-foreground' : 'border-transparent'}" style={`background:${color}`} onclick={() => { draft.accentColor = color; }}>{#if draft.accentColor === color}<Check size={14} class="text-white" />{/if}</button>{/each}</div></fieldset>
      </section>
    {:else}
      <section class="mx-auto flex max-w-2xl flex-col gap-5">
        <div><h3 class="text-base font-semibold">{family?.displayName} {t("settings.chat.setup.connectionStep")}</h3><p class="mt-1 text-sm text-muted-foreground">{t("settings.chat.setup.connectionDescription")}</p></div>
        <label class="setup-field"><span>{t("settings.chat.setup.executable")}</span><div class="flex gap-2"><input class="min-w-0 flex-1" bind:value={draft.executable} /><button type="button" class="setup-icon-button border border-border" aria-label={t("settings.chat.setup.chooseExecutable")} onclick={() => void pickExecutable()}><FolderOpen size={15} /></button></div>{#if fieldError("executable")}<small>{fieldError("executable")}</small>{/if}</label>
        {#if draft.familyId === "codex"}
          <label class="setup-field"><span>{t("settings.chat.setup.codexShadowHome")}</span><input value={providerConfigValue("shadowHomePath")} oninput={(event) => setProviderConfigValue("shadowHomePath", event.currentTarget.value)} /></label>
          <ToggleSetting label={t("settings.chat.setup.allowCustomModels")} checked={providerConfigBoolean("allowCustomModels")} onChange={(value) => setProviderConfigBoolean("allowCustomModels", value)} />
        {:else if draft.familyId === "cursor"}
          <label class="setup-field"><span>{t("settings.chat.setup.endpoint")}</span><input value={providerConfigValue("endpoint")} oninput={(event) => setProviderConfigValue("endpoint", event.currentTarget.value)} />{#if fieldError("providerConfig.endpoint")}<small>{fieldError("providerConfig.endpoint")}</small>{/if}</label>
        {:else if draft.familyId === "opencode"}
          <div class="setup-field">
            <span>{t("settings.chat.setup.openCodeMode")}</span>
            <CustomSelect
              inline
              class="w-full"
              value={openCodeMode()}
              options={openCodeModeOptions}
              onChange={(value) => setOpenCodeMode(value === "external" ? "external" : "local")}
              ariaLabel={t("settings.chat.setup.openCodeMode")}
            />
          </div>
          {#if openCodeMode() === "external"}
            <label class="setup-field"><span>{t("settings.chat.setup.endpoint")}</span><input value={openCodeServerUrl()} oninput={(event) => setProviderConfigValue("serverUrl", event.currentTarget.value)} />{#if fieldError("providerConfig.serverUrl")}<small>{fieldError("providerConfig.serverUrl")}</small>{/if}</label>
            <div>
              <ToggleSetting label={t("settings.chat.setup.confirmExternalWorkspaceAccess")} checked={providerConfigBoolean("confirmExternalWorkspaceAccess")} onChange={(value) => setProviderConfigBoolean("confirmExternalWorkspaceAccess", value)} />
              {#if fieldError("providerConfig.confirmExternalWorkspaceAccess")}<small class="mt-1 block px-1 text-destructive">{fieldError("providerConfig.confirmExternalWorkspaceAccess")}</small>{/if}
            </div>
            {#if openCodeUsesInsecureExternalHttp()}
              <div class="rounded-md border border-status-tentative/50 bg-status-tentative/10 p-3 text-sm text-status-tentative"><p>{t("settings.chat.setup.insecureExternalHttpWarning")}</p><div class="mt-2"><ToggleSetting label={t("settings.chat.setup.allowInsecureExternalHttp")} checked={providerConfigBoolean("allowInsecureExternalHttp")} onChange={(value) => setProviderConfigBoolean("allowInsecureExternalHttp", value)} />{#if fieldError("providerConfig.allowInsecureExternalHttp")}<small class="mt-1 block px-1">{fieldError("providerConfig.allowInsecureExternalHttp")}</small>{/if}</div></div>
            {/if}
            {#if openCodePassword}<div class="setup-field"><span>{t("settings.chat.setup.externalPassword")}</span><div class="flex gap-1"><input class="min-w-0 flex-1" type={revealSecrets[openCodePassword.key] ? "text" : "password"} value={pendingSecrets[openCodePassword.key] ?? ""} placeholder={openCodePassword.credentialReference ? t("settings.chat.setup.stored") : t("settings.chat.setup.missing")} oninput={(event) => { pendingSecrets[openCodePassword.key] = event.currentTarget.value; }} /><button type="button" class="setup-icon-button" onclick={() => { revealSecrets[openCodePassword.key] = !revealSecrets[openCodePassword.key]; }}>{#if revealSecrets[openCodePassword.key]}<EyeOff size={13} />{:else}<Eye size={13} />{/if}</button><button type="button" class="setup-icon-button" aria-label={t("settings.chat.setup.storeSecret")} onclick={() => void storeSecret(openCodePassword)}><Check size={13} /></button><button type="button" class="setup-icon-button" aria-label={t("settings.chat.setup.removeSecret")} onclick={() => { if (openCodePassword.credentialReference) void removeSecret(openCodePassword); removeEnvironment(openCodePassword.key); }}><Trash2 size={13} /></button></div>{#if fieldError(`environment.${openCodePassword.key}.value`)}<small>{fieldError(`environment.${openCodePassword.key}.value`)}</small>{/if}</div>{:else}<button type="button" class="setup-add-button" onclick={addOpenCodePassword}><Plus size={14} />{t("settings.chat.setup.addExternalPassword")}</button>{/if}
          {/if}
        {/if}

        <div class="rounded-lg border border-border">
          <button type="button" class="flex w-full items-center gap-2 p-3 text-left text-sm font-medium hover:bg-accent/40" aria-expanded={advancedOpen} onclick={() => { advancedOpen = !advancedOpen; }}>
            <ChevronRight size={14} class="transition-transform {advancedOpen ? 'rotate-90' : ''}" />
            <span>{t("settings.chat.setup.advanced")}</span>
          </button>
          {#if advancedOpen}
          <div class="flex flex-col gap-4 border-t border-border p-3">
            <div class="setup-field"><span>{t("settings.chat.setup.launchArguments")}</span>{#each draft.launchArguments as argument, index}<div><div class="flex gap-2"><input class="min-w-0 flex-1" value={argument} oninput={(event) => { draft.launchArguments[index] = event.currentTarget.value; }} /><button type="button" class="setup-icon-button" onclick={() => { draft.launchArguments = draft.launchArguments.filter((_, candidate) => candidate !== index); }}><Trash2 size={14} /></button></div>{#if fieldError(`launchArguments.${index}`)}<small>{fieldError(`launchArguments.${index}`)}</small>{/if}</div>{/each}{#if fieldError("launchArguments")}<small>{fieldError("launchArguments")}</small>{/if}<button type="button" class="setup-add-button" onclick={addArgument}><Plus size={14} />{t("settings.chat.setup.addArgument")}</button></div>
            <div class="setup-field"><span>{t("settings.chat.setup.environment")}</span>
              {#each draft.environment.filter((row) => row.name !== "OPENCODE_SERVER_PASSWORD") as row (row.key)}
                <div class="grid gap-2 rounded-md border border-border p-2 sm:grid-cols-[1fr_8rem_1.2fr_auto]">
                  <input aria-label={t("settings.chat.setup.variableName")} bind:value={row.name} />
                  <CustomSelect
                    inline
                    class="w-full"
                    value={row.valueType}
                    options={environmentValueTypeOptions}
                    onChange={(value) => setEnvironmentValueType(row, value)}
                    ariaLabel={t("settings.chat.setup.valueType")}
                  />
                  {#if row.valueType === "text"}<input bind:value={row.value} />{:else if row.valueType === "secret"}<div class="flex gap-1"><input class="min-w-0 flex-1" type={revealSecrets[row.key] ? "text" : "password"} value={pendingSecrets[row.key] ?? ""} placeholder={row.credentialReference ? t("settings.chat.setup.stored") : t("settings.chat.setup.missing")} oninput={(event) => { pendingSecrets[row.key] = event.currentTarget.value; }} /><button type="button" class="setup-icon-button" onclick={() => { revealSecrets[row.key] = !revealSecrets[row.key]; }}>{#if revealSecrets[row.key]}<EyeOff size={13} />{:else}<Eye size={13} />{/if}</button><button type="button" class="setup-icon-button" aria-label={t("settings.chat.setup.storeSecret")} onclick={() => void storeSecret(row)}><Check size={13} /></button></div>{:else}<span class="self-center text-xs text-muted-foreground">{row.name || t("common.none")}</span>{/if}
                  <button type="button" class="setup-icon-button" onclick={() => { if (row.valueType === "secret" && row.credentialReference) void removeSecret(row); removeEnvironment(row.key); }}><Trash2 size={14} /></button>
                  {#if fieldError(`environment.${row.key}.name`) || fieldError(`environment.${row.key}.value`)}<small class="sm:col-span-4">{fieldError(`environment.${row.key}.name`) ?? fieldError(`environment.${row.key}.value`)}</small>{/if}
                </div>
              {/each}
              <button type="button" class="setup-add-button" onclick={addEnvironment}><Plus size={14} />{t("settings.chat.setup.addEnvironment")}</button>
            </div>
          </div>
          {/if}
        </div>

        {#if testResult}<div class="rounded-md border p-3 text-sm {testSucceeded ? 'border-action-confirm/50 text-action-confirm' : 'border-status-tentative/50 text-status-tentative'}"><div>{testSucceeded ? t("settings.chat.setup.testPassed", formatNumber(localization.locale, testResult.modelCatalog?.models.length ?? 0)) : t("settings.chat.setup.testFailed", testResult.probe.detail ?? testResult.probe.state)}</div><dl class="mt-2 grid gap-1 text-xs sm:grid-cols-2"><div><dt class="text-muted-foreground">{t("settings.chat.setup.executable")}</dt><dd>{draft.executable}</dd></div><div><dt class="text-muted-foreground">{t("settings.chat.providers.version")}</dt><dd>{testResult.probe.version ?? t("settings.chat.providers.neverChecked")}</dd></div><div><dt class="text-muted-foreground">{t("settings.chat.providers.account")}</dt><dd>{testResult.probe.accountLabel ?? t("settings.chat.providers.authenticationRequired")}</dd></div><div><dt class="text-muted-foreground">{t("settings.chat.models.heading")}</dt><dd>{formatNumber(localization.locale, testResult.modelCatalog?.models.length ?? 0)}</dd></div></dl>{#if testResult.probe.detail}<p class="mt-2">{testResult.probe.detail}</p>{/if}</div>{/if}
        {#if operationError}<div role="alert" class="rounded-md border border-destructive/40 p-3 text-sm text-destructive">{t("settings.chat.setup.testFailed", operationError)}</div>{/if}
      </section>
    {/if}
  </div>

  <footer class="flex min-h-16 shrink-0 items-center justify-between gap-2 border-t border-border px-4">
    <button type="button" class="setup-secondary-button" onclick={goBack}>{t("settings.chat.setup.back")}</button>
    {#if draft.step !== "connection"}
      <button type="button" class="setup-primary-button" onclick={goNext}>{t("settings.chat.setup.next")}</button>
    {:else}
      <div class="flex gap-2"><button type="button" class="setup-secondary-button" disabled={testing || !validation.valid} onclick={() => void testSetup()}>{testing ? t("settings.chat.setup.testing") : t("settings.chat.setup.test")}</button><button type="button" class="setup-primary-button" disabled={saving || !validation.valid} onclick={() => void save()}>{target.mode === "edit" ? t("settings.chat.setup.save") : t("settings.chat.setup.create")}</button></div>
    {/if}
  </footer>
</div>

<style>
  :global(.setup-field) { display: flex; flex-direction: column; gap: 0.375rem; font-size: calc(0.8rem * var(--type-scale)); font-weight: 500; }
  :global(.setup-field input), :global(.setup-field select), :global(.setup-field textarea) { min-height: 2.25rem; border-radius: 0.375rem; border: 1px solid var(--border); background: var(--background); padding: 0.375rem 0.625rem; color: var(--foreground); outline: none; }
  :global(.setup-field input:focus), :global(.setup-field select:focus) { border-color: var(--ring); box-shadow: 0 0 0 1px var(--ring); }
  :global(.setup-field small) { color: var(--destructive); font-size: calc(0.733333rem * var(--type-scale)); font-weight: 400; }
  :global(.setup-icon-button) { display: inline-flex; width: 2.25rem; height: 2.25rem; flex: 0 0 auto; align-items: center; justify-content: center; border-radius: 0.375rem; color: var(--muted-foreground); }
  :global(.setup-icon-button:hover) { background: var(--accent); color: var(--foreground); }
  :global(.setup-add-button), :global(.setup-secondary-button), :global(.setup-primary-button) { display: inline-flex; min-height: 2.25rem; align-items: center; justify-content: center; gap: 0.375rem; border-radius: 0.375rem; padding: 0.375rem 0.75rem; font-size: calc(0.8rem * var(--type-scale)); font-weight: 600; }
  :global(.setup-add-button), :global(.setup-secondary-button) { border: 1px solid var(--border); background: var(--background); }
  :global(.setup-primary-button) { background: var(--primary); color: var(--primary-foreground); }
  :global(.setup-add-button:hover), :global(.setup-secondary-button:hover) { background: var(--accent); }
  :global(.setup-primary-button:disabled), :global(.setup-secondary-button:disabled) { opacity: 0.5; }
</style>
