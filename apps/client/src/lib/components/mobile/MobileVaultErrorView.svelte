<script lang="ts">
  import HardDrive from "@lucide/svelte/icons/hard-drive";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let { message, onRetry }: { message: string; onRetry: () => void } = $props();
  const { t } = getLocalization();
</script>

<main
  class="mobile-viewport-height flex w-screen items-center justify-center bg-background pb-(--safe-area-bottom) pt-(--safe-area-top) text-foreground"
  style="padding-left: calc(var(--safe-area-left) + 1.5rem); padding-right: calc(var(--safe-area-right) + 1.5rem);"
>
  <section class="flex max-w-md flex-col items-center gap-4 text-center" role="alert">
    <div class="flex h-14 w-14 items-center justify-center rounded-2xl bg-muted">
      <HardDrive size={26} aria-hidden="true" />
    </div>
    <h1 class="text-xl font-semibold">{t("mobile.startupFailed")}</h1>
    <p class="text-sm leading-6 text-muted-foreground">{message}</p>
    <button
      type="button"
      onclick={onRetry}
      class="mt-2 flex min-h-12 items-center justify-center gap-2 rounded-xl bg-primary px-5 text-sm font-medium text-primary-foreground"
    >
      <RotateCcw size={18} aria-hidden="true" />
      <span>{t("common.retry")}</span>
    </button>
  </section>
</main>
