<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { BUILD_REF, GITHUB_REPOSITORY } from "$lib/buildInfo";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { latestReleasePageUrl } from "$lib/stores/updates";

  const { t } = getLocalization();
  const releasePageUrl = latestReleasePageUrl(GITHUB_REPOSITORY);
  let error = $state<string | null>(null);

  async function openReleasePage(): Promise<void> {
    if (!releasePageUrl) return;
    error = null;
    try {
      await openUrl(releasePageUrl);
    } catch (cause: unknown) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-4">
    <h2 class="px-1 text-[0.866667rem] font-semibold text-foreground">
      {t("settings.updates.buildHeading")}
    </h2>

    <div class="flex flex-col gap-3">
      <div class="flex items-start justify-between gap-4 px-1 py-1">
        <div class="min-w-0 flex-1">
          <div class="text-[0.866667rem] text-foreground">
            {t("settings.updates.installedBuild")}
          </div>
          <div class="mt-0.5 wrap-break-word text-[0.8rem] leading-5 text-muted-foreground">
            {BUILD_REF}
          </div>
        </div>
      </div>

      <div class="flex items-start justify-between gap-4 px-1 py-1">
        <div class="min-w-0 flex-1">
          <div class="text-[0.866667rem] text-foreground">
            {t("settings.updates.releaseNotes")}
          </div>
          <div class="mt-0.5 text-[0.8rem] leading-5 text-muted-foreground">
            {t("mobile.settings.androidUpdatesDescription")}
          </div>
        </div>
        <button
          type="button"
          disabled={!releasePageUrl}
          class="inline-flex h-7 shrink-0 items-center justify-center rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground active:bg-accent disabled:pointer-events-none disabled:opacity-55 dark:bg-transparent"
          onclick={() => void openReleasePage()}
        >
          {t("mobile.settings.viewReleases")}
        </button>
      </div>
    </div>

    {#if error}
      <p class="px-1 text-[0.8rem] text-destructive" role="alert">{error}</p>
    {/if}
  </section>
</div>
