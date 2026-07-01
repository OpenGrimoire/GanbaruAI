<script lang="ts">
  import {
    pickNotesPageCoverImageFile,
    saveNotesPageCoverImageDataUrl,
  } from "$lib/api/notes-page-covers";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    createNotesExternalPageCover,
    createNotesLocalFilePageCover,
    notesPageCoverPresetBackground,
    notesPageCoverUrl,
    NOTES_PAGE_COVER_PRESETS,
    type NotesPageCoverPreset,
  } from "$lib/notes/page-cover";
  import type { NotesPageCover } from "$lib/notes/types";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LinkIcon from "@lucide/svelte/icons/link";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Upload from "@lucide/svelte/icons/upload";

  type CoverTab = "presets" | "upload" | "url";

  let {
    cover,
    onSelect,
  }: {
    cover: NotesPageCover | null;
    onSelect: (cover: NotesPageCover | null) => void;
  } = $props();

  const { t } = getLocalization();
  let activeTab = $state<CoverTab>("presets");
  let urlDraft = $state("");
  let error = $state<string | null>(null);
  let uploading = $state(false);
  let lastCoverUrl = "";

  $effect(() => {
    const nextUrl = notesPageCoverUrl(cover) ?? "";
    if (nextUrl === lastCoverUrl) return;
    lastCoverUrl = nextUrl;
    urlDraft = nextUrl;
    error = null;
  });

  function tabLabel(tab: CoverTab): string {
    if (tab === "presets") return t("notes.pageCoverGenerated");
    if (tab === "upload") return t("notes.pageCoverLocal");
    return t("notes.pageCoverExternal");
  }

  function presetLabel(preset: NotesPageCoverPreset): string {
    if (preset.id === "calm-lines") return t("notes.pageCoverPresetCalmLines");
    if (preset.id === "focus-dawn") return t("notes.pageCoverPresetFocusDawn");
    if (preset.id === "deep-work") return t("notes.pageCoverPresetDeepWork");
    return t("notes.pageCoverPresetGreenhouse");
  }

  function fileToDataUrl(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error(t("notes.pageCoverUploadFailed")));
      reader.onload = () => {
        if (typeof reader.result === "string") {
          resolve(reader.result);
        } else {
          reject(new Error(t("notes.pageCoverUploadFailed")));
        }
      };
      reader.readAsDataURL(file);
    });
  }

  function generatedCoverDataUrl(preset: NotesPageCoverPreset): string {
    const canvas = document.createElement("canvas");
    canvas.width = 1600;
    canvas.height = 480;
    const context = canvas.getContext("2d");
    if (!context) throw new Error(t("notes.pageCoverUploadFailed"));
    const [start, middle, end] = preset.colors;
    const gradient = context.createLinearGradient(0, 0, canvas.width, canvas.height);
    gradient.addColorStop(0, start);
    gradient.addColorStop(0.52, middle);
    gradient.addColorStop(1, end);
    context.fillStyle = gradient;
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.globalAlpha = 0.16;
    context.strokeStyle = "#ffffff";
    context.lineWidth = 3;
    for (let index = -2; index < 10; index += 1) {
      context.beginPath();
      context.moveTo(index * 180, canvas.height + 20);
      context.bezierCurveTo(
        index * 180 + 120,
        260,
        index * 180 + 260,
        240,
        index * 180 + 420,
        -20,
      );
      context.stroke();
    }
    context.globalAlpha = 0.18;
    context.fillStyle = "#ffffff";
    context.beginPath();
    context.arc(1320, 120, 160, 0, Math.PI * 2);
    context.fill();
    context.globalAlpha = 1;
    return canvas.toDataURL("image/png");
  }

  function saveExternalCover(): void {
    error = null;
    try {
      onSelect(createNotesExternalPageCover(urlDraft));
    } catch {
      error = t("notes.pageCoverUrlInvalid");
    }
  }

  async function choosePreset(preset: NotesPageCoverPreset): Promise<void> {
    uploading = true;
    error = null;
    try {
      const asset = await saveNotesPageCoverImageDataUrl(generatedCoverDataUrl(preset), `${preset.id}.png`);
      onSelect(createNotesLocalFilePageCover(asset));
    } catch (selectError) {
      error = selectError instanceof Error ? selectError.message : String(selectError);
    } finally {
      uploading = false;
    }
  }

  async function chooseLocalFile(): Promise<void> {
    uploading = true;
    error = null;
    try {
      const asset = await pickNotesPageCoverImageFile();
      if (asset) onSelect(createNotesLocalFilePageCover(asset));
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
      const asset = await saveNotesPageCoverImageDataUrl(dataUrl, file.name);
      onSelect(createNotesLocalFilePageCover(asset));
    } catch (selectError) {
      error = selectError instanceof Error ? selectError.message : String(selectError);
    } finally {
      uploading = false;
    }
  }
</script>

<div
  class="absolute right-0 top-9 z-30 max-h-[min(26rem,calc(100vh-1rem))] w-[min(18rem,calc(100vw-1rem))] overflow-auto rounded-md border border-border bg-popover p-2 text-popover-foreground shadow-lg"
  aria-label={t("notes.pageCover")}
  data-app-floating-surface
  onpaste={(event) => { void handlePaste(event); }}
>
  <div class="mb-2 grid grid-cols-3 gap-1">
    {#each (["presets", "upload", "url"] as const) as tab}
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

  {#if activeTab === "presets"}
    <div class="grid gap-2">
      {#each NOTES_PAGE_COVER_PRESETS as preset}
        <button
          type="button"
          class="group flex h-16 w-full items-end overflow-hidden rounded-md border border-border p-2 text-left shadow-sm hover:border-foreground/30 disabled:cursor-not-allowed disabled:opacity-50"
          style={`background: ${notesPageCoverPresetBackground(preset)}`}
          disabled={uploading}
          aria-label={t("notes.usePageCover", presetLabel(preset))}
          data-app-tooltip={t("notes.usePageCover", presetLabel(preset))}
          onclick={() => { void choosePreset(preset); }}
        >
          <span class="rounded bg-background/85 px-2 py-1 text-[0.733333rem] font-medium text-foreground shadow-sm">
            {presetLabel(preset)}
          </span>
        </button>
      {/each}
    </div>
  {:else if activeTab === "upload"}
    <div class="grid gap-2">
      <button
        type="button"
        class="flex min-h-14 w-full items-center justify-center gap-2 rounded-md bg-muted/50 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
        disabled={uploading}
        onclick={() => { void chooseLocalFile(); }}
      >
        <Upload class="size-4" />
        {t("notes.uploadPageCover")}
      </button>
      <div class="text-center text-[0.733333rem] text-muted-foreground">{t("notes.pageCoverPasteHint")}</div>
      <div class="flex items-center justify-center gap-1 text-[0.733333rem] text-muted-foreground">
        <ImageIcon class="size-3.5" />
        <span>{t("notes.pageCoverImageTypes")}</span>
      </div>
    </div>
  {:else}
    <form
      class="grid gap-2"
      onsubmit={(event) => {
        event.preventDefault();
        saveExternalCover();
      }}
    >
      <label class="block text-[0.733333rem] font-medium text-muted-foreground" for="notes-cover-url">
        {t("notes.pageCoverUrl")}
      </label>
      <div class="flex items-center gap-2 rounded-md border border-border bg-background px-2 py-1.5">
        <LinkIcon class="size-3.5 shrink-0 text-muted-foreground" />
        <input
          id="notes-cover-url"
          class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-foreground outline-none placeholder:text-muted-foreground"
          type="url"
          bind:value={urlDraft}
          placeholder={t("notes.pageCoverUrlPlaceholder")}
        />
      </div>
      <button
        class="flex items-center justify-center gap-1.5 rounded bg-primary px-2 py-1.5 text-[0.8rem] text-primary-foreground hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
        type="submit"
        disabled={!urlDraft.trim()}
      >
        <Save class="size-3.5" />
        <span>{t("notes.savePageCover")}</span>
      </button>
    </form>
  {/if}

  {#if error}
    <div class="mt-2 rounded-md bg-destructive/10 px-2 py-1 text-[0.8rem] text-destructive">{error}</div>
  {/if}

  {#if cover}
    <button
      class="mt-2 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-[0.8rem] text-muted-foreground hover:bg-accent hover:text-foreground"
      type="button"
      onclick={() => {
        onSelect(null);
      }}
    >
      <Trash2 class="size-3.5" />
      <span>{t("notes.removePageCover")}</span>
    </button>
  {/if}
</div>
