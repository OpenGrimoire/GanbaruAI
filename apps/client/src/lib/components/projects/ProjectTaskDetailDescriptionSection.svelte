<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    value,
    onChange,
  }: {
    value: string;
    onChange: (value: string) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
  <span>{t("projects.detail.description")}</span>
  <textarea
    {value}
    rows="5"
    class="min-h-28 resize-none rounded-md border border-border bg-card px-3 py-2 text-[0.833333rem] text-foreground outline-none focus:ring-2 focus:ring-ring/30"
    oninput={(event) => onChange(event.currentTarget.value)}
  ></textarea>
</label>
