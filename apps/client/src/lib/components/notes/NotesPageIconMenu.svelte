<script lang="ts">
  import {
    pickNotesPageIconImageFile,
    saveNotesPageIconImageDataUrl,
  } from "$lib/api/notes-page-icons";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    NOTES_PAGE_EMOJI_ICON_CHOICES,
    NOTES_PAGE_ICON_COLOR_CHOICES,
    NOTES_PAGE_NATIVE_ICON_CHOICES,
    createNotesCustomEmojiPageIcon,
    createNotesEmojiPageIcon,
    createNotesExternalPageIcon,
    createNotesLocalFilePageIcon,
    createNotesNativePageIcon,
    notesPageNativeIconColor,
  } from "$lib/notes/page-icon";
  import type { NotesIconColor, NotesPageIcon as NotesPageIconValue } from "$lib/notes/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LinkIcon from "@lucide/svelte/icons/link";
  import Upload from "@lucide/svelte/icons/upload";
  import X from "@lucide/svelte/icons/x";
  import NotesPageIcon from "./NotesPageIcon.svelte";

  type NotesPageIconTab = "emoji" | "icons" | "custom" | "image";

  let {
    icon,
    onSelect,
  }: {
    icon: NotesPageIconValue | null;
    onSelect: (icon: NotesPageIconValue | null) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();
  let activeTab = $state<NotesPageIconTab>("emoji");
  let nativeColor = $state<NotesIconColor>("gray");
  let externalUrl = $state("");
  let uploading = $state(false);
  let error = $state<string | null>(null);
  let initializedFromIcon = false;

  $effect(() => {
    if (initializedFromIcon) return;
    initializedFromIcon = true;
    activeTab = icon?.type === "icon" ? "icons" : "emoji";
    nativeColor = icon?.type === "icon" ? icon.icon.color ?? "gray" : "gray";
  });

  $effect(() => {
    void projects.ensureLoaded().catch((loadError: unknown) => {
      error = loadError instanceof Error ? loadError.message : String(loadError);
    });
  });

  function tabLabel(tab: NotesPageIconTab): string {
    if (tab === "emoji") return t("notes.pageIconEmoji");
    if (tab === "icons") return t("notes.pageIconNative");
    if (tab === "custom") return t("notes.pageIconCustomEmoji");
    return t("notes.pageIconImage");
  }

  function fileToDataUrl(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error(t("notes.pageIconUploadFailed")));
      reader.onload = () => {
        if (typeof reader.result === "string") {
          resolve(reader.result);
        } else {
          reject(new Error(t("notes.pageIconUploadFailed")));
        }
      };
      reader.readAsDataURL(file);
    });
  }

  function chooseExternalIcon(): void {
    error = null;
    try {
      onSelect(createNotesExternalPageIcon(externalUrl));
    } catch (selectError) {
      error = selectError instanceof Error ? selectError.message : String(selectError);
    }
  }

  async function chooseLocalFile(): Promise<void> {
    uploading = true;
    error = null;
    try {
      const asset = await pickNotesPageIconImageFile();
      if (asset) onSelect(createNotesLocalFilePageIcon(asset));
    } catch (selectError) {
      error = selectError instanceof Error ? selectError.message : String(selectError);
    } finally {
      uploading = false;
    }
  }

  async function handlePaste(event: ClipboardEvent): Promise<void> {
    const file = event.clipboardData?.files[0];
    if (!file) return;
    event.preventDefault();
    uploading = true;
    error = null;
    try {
      const dataUrl = await fileToDataUrl(file);
      const asset = await saveNotesPageIconImageDataUrl(dataUrl, file.name);
      onSelect(createNotesLocalFilePageIcon(asset));
    } catch (selectError) {
      error = selectError instanceof Error ? selectError.message : String(selectError);
    } finally {
      uploading = false;
    }
  }
</script>

<div
  class="absolute left-0 top-10 z-30 w-72 rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
  role="menu"
  aria-label={t("notes.pageIcon")}
  data-app-floating-surface
  onpaste={(event) => { void handlePaste(event); }}
>
  <div class="mb-2 grid grid-cols-4 gap-1">
    {#each (["emoji", "icons", "custom", "image"] as const) as tab}
      <button
        type="button"
        class={`rounded px-2 py-1 text-[0.733333rem] ${
          activeTab === tab ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground"
        }`}
        onclick={() => {
          activeTab = tab;
          error = null;
        }}
      >
        {tabLabel(tab)}
      </button>
    {/each}
  </div>

  {#if activeTab === "emoji"}
    <div class="grid grid-cols-8 gap-1">
      {#each NOTES_PAGE_EMOJI_ICON_CHOICES as choice}
        {@const selected = icon?.type === "emoji" && icon.emoji === choice}
        <button
          class={`flex size-7 items-center justify-center rounded text-[1rem] leading-none hover:bg-accent ${
            selected ? "bg-accent text-accent-foreground" : ""
          }`}
          type="button"
          aria-label={t("notes.usePageIcon", choice)}
          data-app-tooltip={t("notes.usePageIcon", choice)}
          onclick={() => {
            onSelect(createNotesEmojiPageIcon(choice));
          }}
        >
          <span aria-hidden="true">{choice}</span>
        </button>
      {/each}
    </div>
  {:else if activeTab === "icons"}
    <div class="mb-2 flex flex-wrap gap-1">
      {#each NOTES_PAGE_ICON_COLOR_CHOICES as color}
        {@const selected = nativeColor === color}
        <button
          type="button"
          class={`size-5 rounded-full border ${selected ? "border-foreground" : "border-border"}`}
          style={`background: ${notesPageNativeIconColor(color) ?? "#646470"}`}
          aria-label={t("notes.pageIconColor", color)}
          data-app-tooltip={t("notes.pageIconColor", color)}
          onclick={() => {
            nativeColor = color;
          }}
        ></button>
      {/each}
    </div>
    <div class="grid grid-cols-6 gap-1">
      {#each NOTES_PAGE_NATIVE_ICON_CHOICES as choice}
        {@const selected = icon?.type === "icon" && icon.icon.name === choice}
        <button
          class={`flex size-8 items-center justify-center rounded hover:bg-accent ${
            selected ? "bg-accent text-accent-foreground" : ""
          }`}
          type="button"
          aria-label={t("notes.usePageIcon", choice)}
          data-app-tooltip={t("notes.usePageIcon", choice)}
          onclick={() => {
            onSelect(createNotesNativePageIcon(choice, nativeColor));
          }}
        >
          <NotesPageIcon icon={createNotesNativePageIcon(choice, nativeColor)} size={18} />
        </button>
      {/each}
    </div>
  {:else if activeTab === "custom"}
    {#if projects.loading}
      <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.loading")}</div>
    {:else if projects.customEmojis.length === 0}
      <div class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.noCustomPageIcons")}</div>
    {:else}
      <div class="grid max-h-40 grid-cols-6 gap-1 overflow-auto">
        {#each projects.customEmojis as emoji (emoji.id)}
          {@const selected = icon?.type === "custom_emoji" && icon.custom_emoji.id === emoji.id}
          <button
            type="button"
            class={`flex size-8 items-center justify-center rounded hover:bg-accent ${
              selected ? "bg-accent text-accent-foreground" : ""
            }`}
            aria-label={t("notes.usePageIcon", emoji.name)}
            data-app-tooltip={emoji.name}
            onclick={() => {
              onSelect(createNotesCustomEmojiPageIcon({
                id: emoji.id,
                name: emoji.name,
                assetPath: emoji.assetPath,
              }));
            }}
          >
            <NotesPageIcon
              icon={createNotesCustomEmojiPageIcon({
                id: emoji.id,
                name: emoji.name,
                assetPath: emoji.assetPath,
              })}
              size={20}
            />
          </button>
        {/each}
      </div>
    {/if}
  {:else}
    <div class="grid gap-2">
      <button
        type="button"
        class="flex min-h-12 w-full items-center justify-center gap-2 rounded-md bg-muted/50 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
        disabled={uploading}
        onclick={() => { void chooseLocalFile(); }}
      >
        <Upload class="size-4" />
        {t("notes.uploadPageIcon")}
      </button>
      <div class="text-center text-[0.733333rem] text-muted-foreground">{t("notes.pageIconPasteHint")}</div>
      <label class="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1.5">
        <LinkIcon class="size-3.5 shrink-0 text-muted-foreground" />
        <input
          bind:value={externalUrl}
          class="min-w-0 flex-1 bg-transparent text-[0.8rem] outline-none placeholder:text-muted-foreground"
          placeholder={t("notes.pageIconExternalUrlPlaceholder")}
          aria-label={t("notes.pageIconExternalUrl")}
        />
        <button
          type="button"
          class="rounded bg-primary px-2 py-1 text-[0.733333rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
          disabled={!externalUrl.trim()}
          onclick={chooseExternalIcon}
        >
          {t("common.save")}
        </button>
      </label>
      <div class="flex items-center gap-1 text-[0.733333rem] text-muted-foreground">
        <ImageIcon class="size-3.5" />
        <span>{t("notes.pageIconImageTypes")}</span>
      </div>
    </div>
  {/if}

  {#if error}
    <div class="mt-2 rounded-md bg-destructive/10 px-2 py-1 text-[0.8rem] text-destructive">{error}</div>
  {/if}

  {#if icon}
    <button
      class="mt-2 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
      type="button"
      onclick={() => {
        onSelect(null);
      }}
    >
      <X class="size-4" />
      <span>{t("notes.removePageIcon")}</span>
    </button>
  {/if}
</div>
