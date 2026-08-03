<script lang="ts">
  import { onMount, tick } from "svelte";
  import { firstFocusable, trapTabFocus } from "$lib/chat/focus-navigation";
  import { getLocalization } from "$lib/i18n/translator.svelte";

  let {
    element = $bindable(),
    onCancel,
    onConfirm,
  }: {
    element?: HTMLElement;
    onCancel: () => void;
    onConfirm: () => void;
  } = $props();

  const { t } = getLocalization();

  onMount(() => {
    const returnFocus = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    void tick().then(() => firstFocusable(element)?.focus());
    return () => queueMicrotask(() => returnFocus?.isConnected && returnFocus.focus());
  });

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      onCancel();
      return;
    }
    trapTabFocus(event, element);
  }
</script>

<div class="fixed inset-0 z-60 grid place-items-center bg-black/40 p-4">
  <button type="button" class="absolute inset-0" aria-label={t("chat.cancel")} onclick={onCancel}></button>
  <div bind:this={element} class="relative w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="provider-fork-title" tabindex="-1" onkeydown={handleKeydown}>
    <h2 id="provider-fork-title" class="font-semibold">{t("chat.composer.changeProviderTitle")}</h2>
    <p class="mt-2 text-sm text-muted-foreground">{t("chat.composer.changeProviderDescription")}</p>
    <div class="mt-4 flex justify-end gap-2">
      <button type="button" class="chat-secondary-button" onclick={onCancel}>{t("chat.cancel")}</button>
      <button type="button" class="chat-primary-button" onclick={onConfirm}>{t("chat.composer.startProviderFork")}</button>
    </div>
  </div>
</div>
