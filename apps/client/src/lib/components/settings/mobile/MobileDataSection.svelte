<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    formatDataFolderError,
    getActiveVaultInfo,
    type DataFolderInfo,
  } from "$lib/vault/state";

  const { t } = getLocalization();
  let activeDataFolder = $state<DataFolderInfo | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function loadDataFolderState(): Promise<void> {
    loading = true;
    error = null;
    try {
      activeDataFolder = await getActiveVaultInfo();
    } catch (cause: unknown) {
      error = formatDataFolderError(cause, "startup", t);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadDataFolderState();
  });
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-4">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">
      {t("mobile.settings.storageHeading")}
    </h2>

    <div class="flex items-start justify-between gap-4 px-1 py-1">
      <div class="min-w-0 flex-1">
        <div class="text-[0.866667rem] text-foreground">
          {t("settings.data.currentFolder")}
        </div>
        <div class="mt-0.5 wrap-break-word text-[0.8rem] leading-5 text-muted-foreground">
          {#if loading}
            {t("settings.data.loadingFolder")}
          {:else}
            {activeDataFolder?.displayName ?? t("settings.data.noActiveFolder")}
          {/if}
        </div>
      </div>
      <span class="shrink-0 text-[0.8rem] font-medium text-muted-foreground">
        {t("mobile.settings.privateStorage")}
      </span>
    </div>

    <div class="px-1 text-[0.8rem] leading-5 text-muted-foreground">
      {t("mobile.settings.privateStorageDescription")}
    </div>

    {#if error}
      <div class="flex items-center justify-between gap-3 px-1" role="alert">
        <span class="min-w-0 text-[0.8rem] text-destructive">{error}</span>
        <button
          type="button"
          class="inline-flex h-7 shrink-0 items-center justify-center rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground active:bg-accent dark:bg-transparent"
          onclick={() => void loadDataFolderState()}
        >
          {t("common.retry")}
        </button>
      </div>
    {/if}
  </section>
</div>
