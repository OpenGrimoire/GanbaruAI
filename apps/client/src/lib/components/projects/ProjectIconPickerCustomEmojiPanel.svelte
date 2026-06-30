<script lang="ts">
  import Upload from "@lucide/svelte/icons/upload";
  import X from "@lucide/svelte/icons/x";
  import type { ProjectIconAsset } from "$lib/api/project-icons";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { portal } from "$lib/utils/portal";
  import ProjectIcon from "./ProjectIcon.svelte";

  let {
    rootElement = $bindable<HTMLElement | undefined>(),
    style,
    customEmojiDraft,
    customEmojiName = $bindable(""),
    customEmojiError,
    customEmojiSaving,
    onChooseFile,
    onPaste,
    onSave,
    onClose,
  }: {
    rootElement?: HTMLElement;
    style: string;
    customEmojiDraft: ProjectIconAsset | null;
    customEmojiName: string;
    customEmojiError: string | null;
    customEmojiSaving: boolean;
    onChooseFile: () => void | Promise<void>;
    onPaste: (event: ClipboardEvent) => void | Promise<void>;
    onSave: () => void | Promise<void>;
    onClose: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<section
  bind:this={rootElement}
  use:portal
  class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-popover p-4 text-popover-foreground shadow-xl"
  {style}
  data-app-floating-surface
  onpaste={onPaste}
>
  <div class="mb-1 text-[0.933333rem] font-semibold text-foreground">{t("projects.iconPicker.addCustomEmoji")}</div>
  <div class="mb-5 text-[0.8rem] text-muted-foreground">{t("projects.iconPicker.customEmojiDescription")}</div>
  <button
    type="button"
    class="mb-4 flex min-h-16 w-full items-center justify-center gap-2 rounded-md bg-muted/50 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
    onclick={() => { void onChooseFile(); }}
  >
    {#if customEmojiDraft}
      <ProjectIcon name={`asset:${customEmojiDraft.relativePath}`} size={28} />
    {:else}
      <Upload size={17} strokeWidth={1.75} />
    {/if}
    {t("projects.iconPicker.uploadImage")}
  </button>
  <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
    {t("projects.iconPicker.emojiName")}
    <input
      bind:value={customEmojiName}
      class="h-9 rounded-md border border-border bg-background px-2 text-left text-[0.866667rem] text-foreground outline-none placeholder:text-muted-foreground focus:border-ring"
      placeholder={t("projects.iconPicker.emojiNamePlaceholder")}
    />
  </label>
  {#if customEmojiError}
    <div class="mt-2 rounded-md bg-destructive/10 px-2 py-1 text-[0.8rem] text-destructive">{customEmojiError}</div>
  {/if}
  <div class="mt-5 flex justify-between gap-2">
    <button
      type="button"
      class="h-8 rounded-md px-2 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={onClose}
    >
      {t("common.cancel")}
    </button>
    <button
      type="button"
      class="h-8 rounded-md bg-primary px-3 text-[0.866667rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
      disabled={!customEmojiDraft || !customEmojiName.trim() || customEmojiSaving}
      onclick={() => { void onSave(); }}
    >
      {t("common.save")}
    </button>
  </div>
  <button
    type="button"
    class="absolute right-2 top-2 flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
    aria-label={t("common.close")}
    onclick={onClose}
  >
    <X size={14} strokeWidth={1.75} />
  </button>
</section>
