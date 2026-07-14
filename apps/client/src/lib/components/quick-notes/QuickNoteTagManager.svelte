<script lang="ts">
  import { tick } from "svelte";
  import Plus from "@lucide/svelte/icons/plus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    QUICK_NOTE_TAG_LIMIT,
    QUICK_NOTE_TAG_NAME_MAX_CHARS,
  } from "$lib/quick-notes/types";

  let {
    tagCount,
    oncreate,
  }: {
    tagCount: number;
    oncreate: (name: string) => Promise<void>;
  } = $props();

  const { t } = getLocalization();
  let creating = $state(false);
  let input = $state<HTMLInputElement | null>(null);
  let name = $state("");
  let busy = $state(false);
  let error = $state("");

  async function begin(): Promise<void> {
    if (tagCount >= QUICK_NOTE_TAG_LIMIT) return;
    creating = true;
    name = "";
    error = "";
    await tick();
    input?.focus();
  }

  async function create(): Promise<void> {
    const value = name.trim();
    if (!value || busy || tagCount >= QUICK_NOTE_TAG_LIMIT) return;
    busy = true;
    error = "";
    try {
      await oncreate(value);
      name = "";
      creating = false;
    } catch {
      error = t("quickNotes.tag.saveFailed");
    } finally {
      busy = false;
    }
  }

  function cancel(): void {
    if (busy) return;
    creating = false;
    name = "";
    error = "";
  }
</script>

<div class="shrink-0" data-quick-note-tag-creator>
  {#if creating}
    <form
      class={`flex h-7 w-30 items-center gap-1 rounded-md bg-accent/60 px-2 text-foreground ${error ? "ring-1 ring-destructive" : ""}`}
      title={error || t("quickNotes.tag.namePlaceholder")}
      onsubmit={(event) => { event.preventDefault(); void create(); }}
    >
      <Plus class="size-3.5 shrink-0 text-muted-foreground" strokeWidth={1.5} />
      <input
        bind:this={input}
        type="text"
        bind:value={name}
        maxlength={QUICK_NOTE_TAG_NAME_MAX_CHARS}
        class="min-w-0 flex-1 bg-transparent text-xs caret-primary outline-none placeholder:text-muted-foreground"
        placeholder={t("quickNotes.tag.namePlaceholder")}
        aria-label={t("quickNotes.tag.namePlaceholder")}
        aria-invalid={error ? "true" : undefined}
        disabled={busy}
        oninput={() => { error = ""; }}
        onkeydown={(event) => {
          if (event.key !== "Escape") return;
          event.preventDefault();
          event.stopPropagation();
          cancel();
        }}
        onblur={() => { if (!busy) cancel(); }}
      />
      {#if error}<span class="sr-only" role="alert">{error}</span>{/if}
    </form>
  {:else if tagCount < QUICK_NOTE_TAG_LIMIT}
  <button
    type="button"
    class="flex h-7 shrink-0 items-center gap-1 rounded-md px-2 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
    title={t("quickNotes.tag.add")}
    onclick={() => void begin()}
  >
    <Plus class="size-3.5" strokeWidth={1.5} />
    <span class="max-[360px]:hidden">{t("quickNotes.tag.add")}</span>
  </button>
  {/if}
</div>
