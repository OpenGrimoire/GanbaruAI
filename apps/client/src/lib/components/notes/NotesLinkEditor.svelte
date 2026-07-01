<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    value,
    error,
    canRemove,
    onInput,
    onApply,
    onRemove,
    onCancel,
  }: {
    value: string;
    error: string | null;
    canRemove: boolean;
    onInput: (value: string) => void;
    onApply: () => void;
    onRemove: () => void;
    onCancel: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="mt-1 rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-sm">
  <div class="flex min-w-0 items-center gap-1.5">
    <input
      class="min-w-0 flex-1 rounded border border-input bg-background px-2 py-1 text-[0.8rem] outline-none focus-visible:ring-2 focus-visible:ring-ring"
      {value}
      aria-label={t("notes.linkUrl")}
      placeholder={t("notes.linkUrlPlaceholder")}
      oninput={(event) => onInput(event.currentTarget.value)}
      onkeydown={(event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          onApply();
        }
        if (event.key === "Escape") {
          event.preventDefault();
          onCancel();
        }
      }}
    />
    <button
      type="button"
      class="rounded bg-primary px-2 py-1 text-[0.8rem] font-medium text-primary-foreground outline-none hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring"
      onclick={onApply}
    >
      {t("notes.applyLink")}
    </button>
    {#if canRemove}
      <button
        type="button"
        class="rounded border border-border px-2 py-1 text-[0.8rem] text-muted-foreground outline-none hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        onclick={onRemove}
      >
        {t("notes.removeLink")}
      </button>
    {/if}
  </div>
  {#if error}
    <p class="mt-1 text-[0.733333rem] text-destructive">{error}</p>
  {/if}
</div>
