<script lang="ts">
  import { tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    mediaCaptionPlainText,
    mediaDisplayNameFromSource,
    mediaDisplayName,
    mediaPlainText,
    mediaPreviewKindForUrl,
    mediaSourceUrl,
    mediaUrlIssue,
    type NotesMediaUrlIssue,
    type NotesMediaBlockType,
  } from "$lib/notes/media";
  import {
    planNotesKeyboardAction,
    type NotesKeyboardAction,
  } from "$lib/notes/block-keyboard";
  import { notesUndoShortcutAction } from "$lib/notes/undo-history";
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
    onUndo,
    onRedo,
    onMediaChange,
  }: {
    block: NotesMediaBlock;
    previousBlockType: NotesBlockType | null;
    isOnlyBlock: boolean;
    focusBlockId: string | null;
    focusRequestId: number;
    onKeyboardAction: (blockId: string, action: NotesKeyboardAction) => void;
    onUndo: () => Promise<void> | void;
    onRedo: () => Promise<void> | void;
    onMediaChange: (blockId: string, url: string, caption: string, name?: string) => void;
  } = $props();

  const { t } = getLocalization();
  let urlInput: HTMLInputElement | null = $state(null);
  let openError = $state<string | null>(null);
  let urlDraft = $state("");
  let captionDraft = $state("");
  let nameDraft = $state("");
  let draftSourceKey = $state("");
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
  const displayName = $derived(
    mediaDisplayNameFromSource(urlDraft, mediaType === "file" ? nameDraft : undefined)
      || mediaDisplayName(media)
      || t(`notes.blockType.${mediaType}`),
  );
  const draftIssue = $derived(mediaUrlIssue(mediaType, urlDraft));
  const canOpen = $derived(!draftIssue && canOpenDraftUrl(urlDraft));
  const previewKind = $derived(mediaPreviewKindForUrl(mediaType, urlDraft));
  const currentText = $derived(
    [captionDraft, displayName, urlDraft || mediaPlainText(media)]
      .map((part) => part.trim())
      .filter(Boolean)
      .join(" "),
  );
  const MediaIcon = $derived(mediaIcon());

  $effect(() => {
    const sourceKey = `${block.id}\u0000${url}\u0000${caption}\u0000${name}`;
    if (draftSourceKey === sourceKey) return;
    urlDraft = url;
    captionDraft = caption;
    nameDraft = name;
    draftSourceKey = sourceKey;
  });

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

  function canOpenDraftUrl(source: string): boolean {
    const trimmed = source.trim();
    if (!trimmed) return false;
    try {
      return new URL(trimmed).protocol === "https:";
    } catch {
      return false;
    }
  }

  function validationMessage(issue: NotesMediaUrlIssue | null): string | null {
    if (!issue) return null;
    if (issue === "invalid_url") return t("notes.mediaUrlInvalid");
    if (issue === "requires_https") return t("notes.mediaUrlRequiresHttps");
    return t("notes.mediaUrlUnsupported", t(`notes.blockType.${mediaType}`));
  }

  function previewMessage(): string | null {
    if (draftIssue || !urlDraft.trim()) return null;
    if (previewKind === "link") return t("notes.mediaExternalPreviewOnly");
    return null;
  }

  function commitDrafts(): void {
    if (draftIssue) return;
    openError = null;
    onMediaChange(
      block.id,
      urlDraft,
      captionDraft,
      mediaType === "file" ? nameDraft : undefined,
    );
  }

  function handleKeydown(event: KeyboardEvent): void {
    const undoAction = notesUndoShortcutAction(event);
    if (undoAction) {
      event.preventDefault();
      void Promise.resolve(undoAction === "undo" ? onUndo() : onRedo());
      return;
    }
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
    commitDrafts();
    onKeyboardAction(block.id, action);
  }

  async function openMedia(): Promise<void> {
    if (!canOpen) return;
    try {
      openError = null;
      await openUrl(urlDraft.trim());
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
          value={nameDraft}
          placeholder={t("notes.mediaNamePlaceholder")}
          oninput={(event) => {
            nameDraft = event.currentTarget.value;
            openError = null;
          }}
          onblur={commitDrafts}
          onkeydown={handleKeydown}
        />
      {/if}
      <input
        bind:this={urlInput}
        class="min-h-7 w-full min-w-0 bg-transparent text-[0.866667rem] font-medium outline-none placeholder:text-muted-foreground"
        type="url"
        inputmode="url"
        aria-label={t("notes.mediaUrl")}
        value={urlDraft}
        placeholder={t("notes.mediaUrlPlaceholder")}
        oninput={(event) => {
          urlDraft = event.currentTarget.value;
          openError = null;
        }}
        onblur={commitDrafts}
        onkeydown={handleKeydown}
      />
      <input
        class="min-h-7 w-full min-w-0 bg-transparent text-[0.8rem] text-muted-foreground outline-none placeholder:text-muted-foreground"
        type="text"
        aria-label={t("notes.mediaCaption")}
        value={captionDraft}
        placeholder={t("notes.mediaCaptionPlaceholder")}
        oninput={(event) => {
          captionDraft = event.currentTarget.value;
          openError = null;
        }}
        onblur={commitDrafts}
        onkeydown={handleKeydown}
      />
      {#if validationMessage(draftIssue)}
        <p class="text-[0.733333rem] text-destructive">
          {validationMessage(draftIssue)}
        </p>
      {:else if previewMessage()}
        <p class="text-[0.733333rem] text-muted-foreground">
          {previewMessage()}
        </p>
      {/if}
      {#if openError}
        <p class="text-[0.733333rem] text-destructive">
          {t("notes.openMediaFailed", openError)}
        </p>
      {/if}
    </div>
    <button
      type="button"
      class="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
      aria-label={t("notes.openMedia", urlDraft)}
      disabled={!canOpen}
      onclick={() => {
        void openMedia();
      }}
    >
      <ExternalLink class="size-4" />
    </button>
  </div>

  {#if previewKind !== "none" && previewKind !== "link"}
    {#if previewKind === "image"}
      <img
        class="notes-media-preview notes-media-preview-image"
        src={urlDraft}
        alt={captionDraft || displayName}
        loading="lazy"
      />
    {:else if previewKind === "video"}
      <video class="notes-media-preview" src={urlDraft} controls>
        <track kind="captions" />
        {displayName}
      </video>
    {:else if previewKind === "audio"}
      <audio class="w-full" src={urlDraft} controls>
        {displayName}
      </audio>
    {:else if previewKind === "pdf"}
      <object
        class="notes-media-preview notes-media-preview-pdf"
        data={urlDraft}
        type="application/pdf"
        aria-label={t("notes.mediaPreview")}
      >
        <span class="block p-3 text-[0.8rem] text-muted-foreground">{displayName}</span>
      </object>
    {/if}
  {:else}
    <div class="notes-media-placeholder">
      <span class="min-w-0 truncate">{displayName || t(`notes.blockType.${mediaType}`)}</span>
    </div>
  {/if}
</section>

<style>
  .notes-media-preview {
    width: 100%;
    max-height: 20rem;
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 0.375rem;
  }

  .notes-media-preview-image {
    object-fit: contain;
  }

  .notes-media-preview-pdf {
    height: min(20rem, 55vh);
  }

  .notes-media-placeholder {
    display: flex;
    min-width: 0;
    align-items: center;
    border-radius: 0.375rem;
    background: hsl(var(--muted) / 0.4);
    padding: 0.5rem 0.75rem;
    color: hsl(var(--muted-foreground));
    font-size: 0.866667rem;
  }
</style>
