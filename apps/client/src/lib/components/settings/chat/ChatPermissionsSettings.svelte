<script lang="ts">
  import FileCog from "@lucide/svelte/icons/file-cog";
  import FileText from "@lucide/svelte/icons/file-text";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Save from "@lucide/svelte/icons/save";
  import type { ProviderFileRead, ProviderInstanceId } from "$lib/chat/contracts";
  import { readChatProviderFiles, saveChatProviderFile } from "$lib/api/chat";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getChat } from "$lib/stores/chat.svelte";
  import ChatModelAvatar from "$lib/components/chat/ChatModelAvatar.svelte";
  import { cn } from "$lib/utils";

  const { t } = getLocalization();
  const chat = getChat();
  let selectedInstanceId = $state<ProviderInstanceId | null>(null);
  let selectedFileId = $state<string | null>(null);
  let files = $state<ProviderFileRead[]>([]);
  let draft = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let loadGeneration = 0;

  const providers = $derived(chat.settings?.providerInstances ?? []);
  const selectedProvider = $derived(
    providers.find((provider) => provider.configuration.instanceId === selectedInstanceId) ?? null,
  );
  const selectedFile = $derived(files.find((file) => file.fileId === selectedFileId) ?? null);
  const dirty = $derived(selectedFile !== null && draft !== selectedFile.contents);

  $effect(() => {
    if (providers.length === 0) {
      selectedInstanceId = null;
      files = [];
      return;
    }
    if (!providers.some((provider) => provider.configuration.instanceId === selectedInstanceId)) {
      selectedInstanceId = providers[0].configuration.instanceId;
    }
  });

  $effect(() => {
    const instanceId = selectedInstanceId;
    if (instanceId) void loadFiles(instanceId);
  });

  async function loadFiles(instanceId: ProviderInstanceId): Promise<void> {
    const generation = ++loadGeneration;
    loading = true;
    error = null;
    try {
      const next = await readChatProviderFiles(instanceId);
      if (generation !== loadGeneration || instanceId !== selectedInstanceId) return;
      files = next;
      const preferred = next.find((file) => file.fileId === selectedFileId) ?? next[0] ?? null;
      selectedFileId = preferred?.fileId ?? null;
      draft = preferred?.contents ?? "";
    } catch (cause: unknown) {
      if (generation !== loadGeneration) return;
      files = [];
      selectedFileId = null;
      draft = "";
      error = errorMessage(cause);
    } finally {
      if (generation === loadGeneration) loading = false;
    }
  }

  function chooseProvider(instanceId: ProviderInstanceId): void {
    if (instanceId === selectedInstanceId) return;
    if (dirty) {
      error = t("settings.chat.permissions.saveOrReload");
      return;
    }
    selectedFileId = null;
    selectedInstanceId = instanceId;
  }

  function chooseFile(file: ProviderFileRead): void {
    if (file.fileId === selectedFileId) return;
    if (dirty) {
      error = t("settings.chat.permissions.saveOrReload");
      return;
    }
    selectedFileId = file.fileId;
    draft = file.contents;
    error = null;
  }

  async function save(): Promise<void> {
    if (!selectedProvider || !selectedFile || !dirty || saving) return;
    saving = true;
    error = null;
    try {
      const saved = await saveChatProviderFile({
        instanceId: selectedProvider.configuration.instanceId,
        fileId: selectedFile.fileId,
        contents: draft,
        expectedRevision: selectedFile.revision,
      });
      files = files.map((file) => file.fileId === saved.fileId ? saved : file);
      draft = saved.contents;
    } catch (cause: unknown) {
      error = errorMessage(cause);
    } finally {
      saving = false;
    }
  }

  function reload(): void {
    if (selectedInstanceId) void loadFiles(selectedInstanceId);
  }

  function fileLabel(file: ProviderFileRead): string {
    return file.name;
  }

  function errorMessage(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }
</script>

<section class="flex min-h-0 flex-col gap-4">
  <div class="px-1">
    <h2 class="text-[0.866667rem] font-semibold text-foreground">{t("settings.chat.permissions.heading")}</h2>
    <p class="mt-1 text-[0.8rem] text-muted-foreground">{t("settings.chat.permissions.description")}</p>
  </div>

  {#if providers.length === 0}
    <p class="rounded-lg border border-dashed border-border p-4 text-sm text-muted-foreground">
      {t("settings.chat.permissions.empty")}
    </p>
  {:else}
    <div class="flex flex-wrap gap-2" role="tablist" aria-label={t("settings.chat.permissions.providerLabel")}>
      {#each providers as provider (provider.configuration.instanceId)}
        {@const active = provider.configuration.instanceId === selectedInstanceId}
        <button
          type="button"
          role="tab"
          aria-selected={active}
          class={cn(
            "flex min-h-9 items-center gap-2 rounded-md border px-3 text-[0.8rem] font-medium transition-colors",
            active ? "border-foreground/20 bg-accent text-foreground" : "border-border bg-background text-muted-foreground hover:bg-accent/50 hover:text-foreground",
          )}
          onclick={() => chooseProvider(provider.configuration.instanceId)}
        >
          <ChatModelAvatar familyId={provider.configuration.familyId} label={provider.configuration.label} size={22} />
          <span>{provider.configuration.label}</span>
        </button>
      {/each}
    </div>

    {#if error}
      <p role="alert" class="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-[0.733333rem] text-destructive">{error}</p>
    {/if}

    {#if loading}
      <div class="rounded-lg border border-border p-8 text-center text-sm text-muted-foreground">{t("common.loading")}</div>
    {:else if selectedProvider && files.length > 0}
      <div class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-card/30">
        <div class="flex flex-wrap items-center justify-between gap-2 border-b border-border px-3 py-2.5">
          <div class="flex flex-wrap gap-1" role="tablist" aria-label={t("settings.chat.permissions.fileLabel")}>
            {#each files as file (file.fileId)}
              {@const Icon = file.kind === "configuration" ? FileCog : FileText}
              <button
                type="button"
                role="tab"
                aria-selected={file.fileId === selectedFileId}
                class={cn(
                  "flex min-h-8 items-center gap-1.5 rounded-md px-2.5 text-[0.766667rem] font-medium text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                  file.fileId === selectedFileId && "bg-accent text-foreground",
                )}
                onclick={() => chooseFile(file)}
              >
                <Icon size={14} strokeWidth={1.75} />
                {fileLabel(file)}
              </button>
            {/each}
          </div>
          <div class="flex gap-2">
            <button type="button" class="chat-settings-button" disabled={loading || saving} onclick={reload}>
              <RefreshCw size={13} />
              {dirty ? t("settings.chat.permissions.discardAndReload") : t("settings.chat.permissions.reload")}
            </button>
            <button type="button" class="setup-primary-button gap-1.5" disabled={!dirty || saving} onclick={() => void save()}>
              <Save size={13} />
              {saving ? t("settings.chat.permissions.saving") : t("settings.chat.permissions.save")}
            </button>
          </div>
        </div>

        {#if selectedFile}
          <div class="border-b border-border bg-muted/20 px-3 py-2">
            <p class="break-all font-mono text-[0.7rem] text-muted-foreground">{selectedFile.path}</p>
            <p class="mt-1 text-[0.666667rem] text-muted-foreground">
              {selectedFile.exists
                ? t("settings.chat.permissions.existingFile", selectedFile.format.toUpperCase())
                : t("settings.chat.permissions.newFile", selectedFile.format.toUpperCase())}
            </p>
          </div>
          <textarea
            class="h-[min(26rem,48vh)] min-h-48 w-full resize-y bg-background/70 p-3 font-mono text-[0.766667rem] leading-5 text-foreground outline-none placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-ring"
            value={draft}
            aria-label={fileLabel(selectedFile)}
            placeholder={selectedFile.kind === "instructions"
              ? t("settings.chat.permissions.instructionsPlaceholder")
              : t("settings.chat.permissions.configurationPlaceholder")}
            spellcheck="false"
            autocapitalize="off"
            oninput={(event) => { draft = event.currentTarget.value; error = null; }}
          ></textarea>
          <div class="flex flex-wrap items-center justify-between gap-2 border-t border-border px-3 py-2 text-[0.666667rem] text-muted-foreground">
            <span>{t("settings.chat.permissions.fileLimit")}</span>
            {#if dirty}<span class="font-medium text-status-tentative">{t("settings.chat.permissions.unsaved")}</span>{/if}
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</section>
