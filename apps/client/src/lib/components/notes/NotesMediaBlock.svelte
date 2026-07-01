<script lang="ts">
  import { tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    canOpenMediaUrl,
    canPreviewMedia,
    mediaCaptionPlainText,
    mediaDisplayName,
    mediaPlainText,
    mediaSourceUrl,
    type NotesMediaBlockType,
  } from "$lib/notes/media";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import type {
    NotesAudioBlock,
    NotesBlockType,
    NotesFileBlock,
    NotesImageBlock,
    NotesPdfBlock,
    NotesVideoBlock,
  } from "$lib/notes/types";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FileIcon from "@lucide/svelte/icons/file";
  import FileText from "@lucide/svelte/icons/file-text";
  import ImageIcon from "@lucide/svelte/icons/image";
  import Music from "@lucide/svelte/icons/music";
  import Video from "@lucide/svelte/icons/video";

  type NotesMediaBlock =
    | NotesImageBlock
    | NotesVideoBlock
    | NotesAudioBlock
    | NotesFileBlock
    | NotesPdfBlock;

  let {
    block,
    previousBlockType,
    isOnlyBlock,
    focusBlockId,
    focusRequestId,
    onKeyboardAction,
    onMediaChange,
  }: {
    block: NotesMediaBlock;
    previousBlockType: NotesBlockType | null;
    isOnlyBlock: boolean;
    focusBlockId: string | null;
    focusRequestId: number;
    onKeyboardAction: (blockId: string, action: NotesKeyboardAction) => void;
    onMediaChange: (blockId: string, url: string, caption: string, name?: string) => void;
  } = $props();

  const { t } = getLocalization();
  let urlInput: HTMLInputElement | null = $state(null);
  let openError = $state<string | null>(null);
  const mediaType = $derived(block.type as NotesMediaBlockType);
  const media = $derived(
    block.type === "image"
      ? block.image
      : block.type === "video"
        ? block.video
        : block.type === "audio"
          ? block.audio
          : block.type === "pdf"
            ? block.pdf
            : block.file,
  );
  const url = $derived(mediaSourceUrl(media));
  const caption = $derived(mediaCaptionPlainText(media));
  const name = $derived(media.name ?? "");
  const displayName = $derived(mediaDisplayName(media) || t(`notes.blockType.${mediaType}`));
  const canOpen = $derived(canOpenMediaUrl(media));
  const canPreview = $derived(canPreviewMedia(mediaType, media));
  const currentText = $derived(mediaPlainText(media));
  const MediaIcon = $derived(mediaIcon());

  $effect(() => {
    const _focusRequestId = focusRequestId;
    if (focusBlockId !== block.id) return;
    void tick().then(() => {
      urlInput?.focus();
    });
  });

  function mediaIcon() {
    if (mediaType === "image") return ImageIcon;
    if (mediaType === "video") return Video;
    if (mediaType === "audio") return Music;
    if (mediaType === "pdf") return FileText;
    return FileIcon;
  }

  function handleInput(nextUrl: string, nextCaption: string, nextName?: string): void {
    openError = null;
    onMediaChange(block.id, nextUrl, nextCaption, nextName);
  }

  function handleKeydown(event: KeyboardEvent): void {
    const target = event.currentTarget;
    const input = target instanceof HTMLInputElement ? target : null;
    const action = planNotesKeyboardAction({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      text: currentText || (input?.value ?? ""),
      selectionStart: input?.selectionStart ?? 0,
      selectionEnd: input?.selectionEnd ?? 0,
      blockType: block.type,
      previousBlockType,
      isOnlyBlock,
    });
    if (
      action.type === "none"
      || action.type === "insert_newline"
      || action.type === "open_slash_menu"
    ) {
      return;
    }
    if (action.preventDefault) event.preventDefault();
    onKeyboardAction(block.id, action);
  }

  async function openMedia(): Promise<void> {
    if (!canOpen) return;
    try {
      openError = null;
      await openUrl(url.trim());
    } catch (error) {
      openError = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<section
  class="my-1 flex min-w-0 flex-col gap-2 rounded-md border border-border bg-background/70 p-2"
  aria-label={t(`notes.blockType.${mediaType}`)}
>
  <div class="flex min-w-0 items-start gap-2">
    <div
      class="mt-1 flex size-7 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
      aria-hidden="true"
    >
      <MediaIcon class="size-4" />
    </div>
    <div class="flex min-w-0 flex-1 flex-col gap-1">
      {#if mediaType === "file"}
        <input
          class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
          type="text"
          aria-label={t("notes.mediaName")}
          value={name}
          placeholder={t("notes.mediaNamePlaceholder")}
          oninput={(event) => {
            handleInput(url, caption, event.currentTarget.value);
          }}
          onkeydown={handleKeydown}
        />
      {/if}
      <input
        bind:this={urlInput}
        class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
        type="url"
        inputmode="url"
        aria-label={t("notes.mediaUrl")}
        value={url}
        placeholder={t("notes.mediaUrlPlaceholder")}
        oninput={(event) => {
          handleInput(event.currentTarget.value, caption, name);
        }}
        onkeydown={handleKeydown}
      />
      <input
        class="min-h-7 w-full min-w-0 bg-transparent text-[0.8rem] text-muted-foreground outline-none placeholder:text-muted-foreground"
        type="text"
        aria-label={t("notes.mediaCaption")}
        value={caption}
        placeholder={t("notes.mediaCaptionPlaceholder")}
        oninput={(event) => {
          handleInput(url, event.currentTarget.value, name);
        }}
        onkeydown={handleKeydown}
      />
      {#if openError}
        <p class="text-[0.733333rem] text-destructive">
          {t("notes.openMediaFailed", openError)}
        </p>
      {/if}
    </div>
    <button
      type="button"
      class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
      aria-label={t("notes.openMedia", url)}
      disabled={!canOpen}
      onclick={() => {
        void openMedia();
      }}
    >
      <ExternalLink class="size-4" />
    </button>
  </div>

  {#if canPreview}
    {#if mediaType === "image"}
      <img
        class="max-h-80 w-full rounded-md border border-border object-contain"
        src={url}
        alt={caption || displayName}
        loading="lazy"
      />
    {:else if mediaType === "video"}
      <video class="max-h-80 w-full rounded-md border border-border" src={url} controls>
        <track kind="captions" />
        {displayName}
      </video>
    {:else if mediaType === "audio"}
      <audio class="w-full" src={url} controls>
        {displayName}
      </audio>
    {:else if mediaType === "pdf"}
      <object
        class="h-80 w-full rounded-md border border-border"
        data={url}
        type="application/pdf"
        aria-label={t("notes.mediaPreview")}
      >
        <span class="block p-3 text-[0.8rem] text-muted-foreground">{displayName}</span>
      </object>
    {/if}
  {:else}
    <div class="min-w-0 truncate rounded-md bg-muted/40 px-3 py-2 text-[0.866667rem] text-muted-foreground">
      {displayName || t(`notes.blockType.${mediaType}`)}
    </div>
  {/if}
</section>
