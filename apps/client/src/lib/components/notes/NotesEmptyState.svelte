<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    onCreate,
  }: {
    onCreate: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex h-full min-h-0 items-center justify-center px-4 py-6 text-center">
  <div class="flex max-w-sm flex-col items-center gap-3">
    <div class="text-[1.05rem] font-semibold text-foreground">{t("notes.empty.title")}</div>
    <p class="text-[0.866667rem] leading-snug text-muted-foreground">
      {t("notes.empty.body")}
    </p>
    <button
      class="rounded-md bg-primary px-3 py-2 text-[0.866667rem] font-medium text-primary-foreground hover:bg-primary/90"
      onclick={onCreate}
    >
      {t("notes.createFirstPage")}
    </button>
  </div>
</div>
