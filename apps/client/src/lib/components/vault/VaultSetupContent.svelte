<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import type { Snippet } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import VaultLanguageDropdown from "./VaultLanguageDropdown.svelte";

  let {
    intro,
    developmentWarning = null,
    location,
    actions,
    notice,
    error = null,
  }: {
    intro: string;
    developmentWarning?: string | null;
    location: string;
    actions: Snippet;
    notice?: Snippet;
    error?: string | null;
  } = $props();

  const { t } = getLocalization();
</script>

<section class="h-full overflow-y-auto px-4 min-[560px]:px-8 min-[760px]:px-10">
  <div class="setup-content-grid mx-auto grid min-h-full w-full max-w-2xl py-5 min-[760px]:py-8">
    <div class="flex items-end pb-4">
      <VaultLanguageDropdown />
    </div>

    <div class="flex flex-col gap-7">
      <div class="space-y-2">
        <h1 class="max-w-xl text-2xl font-semibold leading-tight text-foreground min-[560px]:text-3xl">
          {t("vaultSetup.title")}
        </h1>
        <p class="text-sm leading-6 text-muted-foreground">
          {intro}
          {#if developmentWarning}
            <br />
            <strong class="font-semibold text-warning">{developmentWarning}</strong>
          {/if}
        </p>
      </div>

      <div class="grid gap-5">
        <div class="grid gap-2 border-y border-border py-4 min-[560px]:grid-cols-[auto_1fr] min-[560px]:items-start">
          <div class="flex items-center gap-2 text-sm font-medium text-foreground">
            <Folder size={16} strokeWidth={1.8} aria-hidden="true" />
            {t("vaultSetup.defaultLocation")}
          </div>
          <p class="break-all text-sm leading-5 text-muted-foreground min-[560px]:text-right">
            {location}
          </p>
        </div>

        {#if notice}
          {@render notice()}
        {/if}

        {@render actions()}
      </div>
    </div>

    <div class="pt-4">
      {#if error}
        <p
          role="alert"
          class="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
        >
          {error}
        </p>
      {/if}
    </div>
  </div>
</section>

<style>
  .setup-content-grid {
    grid-template-rows: minmax(4.5rem, 1fr) auto minmax(4.5rem, 1fr);
  }
</style>
