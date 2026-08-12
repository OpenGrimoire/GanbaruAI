<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Link2 from "@lucide/svelte/icons/link-2";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Network from "@lucide/svelte/icons/network";
  import Youtube from "@lucide/svelte/icons/youtube";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MediaFolderSelection } from "$lib/api/music";
  import type { MusicSourcesController } from "$lib/music/music-sources-controller.svelte";
  import type { MusicYouTubeResolutionSource, MusicYouTubeSourcePreview } from "$lib/music/music-youtube-source-resolver";

  let { controller, onClose, onSaved }: { controller: MusicSourcesController; onClose: () => void; onSaved: () => void } = $props();
  const { t } = getLocalization();
  let step = $state<"choose" | "local" | "youtube">("choose");
  let youtubeKind = $state<"youtube-video" | "youtube-playlist">("youtube-video");
  let localSelection = $state<MediaFolderSelection | null>(null);
  let localRelationship = $state<"duplicate" | "nested" | "contains-existing" | "separate">("separate");
  let name = $state("");
  let link = $state("");
  let parsedSource = $state<MusicYouTubeResolutionSource | null>(null);
  let preview = $state<MusicYouTubeSourcePreview | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);

  async function chooseLocal(): Promise<void> {
    error = null;
    const result = await controller.chooseLocalFolder();
    if (!result) return;
    localSelection = result.selection;
    localRelationship = result.relationship;
    name = result.name;
    step = "local";
  }

  function chooseYouTube(kind: "youtube-video" | "youtube-playlist"): void {
    youtubeKind = kind;
    step = "youtube";
    preview = null;
    parsedSource = null;
    error = null;
  }

  async function resolveLink(): Promise<void> {
    const parsed = controller.parseYouTubeInput(link, youtubeKind);
    if (!parsed.source) { error = parsed.error; return; }
    error = null;
    parsedSource = parsed.source;
    preview = await controller.resolveYouTube(parsed.source);
    if (!preview) error = controller.resolutionError;
    else if (!name.trim()) name = preview.title;
  }

  async function saveLocal(): Promise<void> {
    if (!localSelection || localRelationship === "duplicate") return;
    saving = true;
    error = null;
    try { await controller.addLocalFolder(localSelection, name); onSaved(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }

  async function saveYouTube(): Promise<void> {
    if (!preview) return;
    saving = true;
    error = null;
    try { await controller.addYouTube(preview, name); onSaved(); }
    catch (cause) { error = cause instanceof Error ? cause.message : String(cause); }
    finally { saving = false; }
  }
</script>

<div class="absolute inset-0 z-50 grid place-items-center bg-background/60 p-2 backdrop-blur-sm">
  <button type="button" class="absolute inset-0" onclick={onClose} aria-label={t("music.builder.close")}></button>
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: controller.busy || controller.resolving }} class="source-dialog relative flex max-h-full w-[min(37rem,100%)] flex-col overflow-hidden rounded-2xl border border-border/70 bg-popover shadow-2xl" role="dialog" aria-modal="true" aria-label={t("music.builder.addSource")} tabindex="-1">
    <header class="flex min-h-11 shrink-0 items-center gap-2 border-b border-border/55 px-3">
      {#if step !== "choose"}<button type="button" onclick={() => { step = "choose"; controller.cancelResolution(); }} class="inline-flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-accent-foreground" aria-label={t("music.builder.back")}><ArrowLeft size={14} /></button>{/if}
      <h2 class="min-w-0 flex-1 truncate text-sm font-semibold">{step === "choose" ? t("music.builder.chooseSourceType") : step === "local" ? t("music.builder.localFolder") : youtubeKind === "youtube-video" ? t("music.builder.youtubeVideo") : t("music.builder.youtubePlaylist")}</h2>
      <button type="button" onclick={onClose} class="rounded-lg px-2 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground">{t("music.builder.close")}</button>
    </header>

    <div class="source-dialog-scroll min-h-0 flex-1 overflow-y-auto p-3">
      {#if step === "choose"}
        <div class="grid grid-cols-3 gap-2 max-[560px]:grid-cols-1">
          <button type="button" class="source-choice" onclick={() => { void chooseLocal(); }}><span><FolderOpen size={22} /></span><strong>{t("music.builder.localFolder")}</strong><small>{t("music.builder.localFolderDescription")}</small></button>
          <button type="button" class="source-choice" onclick={() => chooseYouTube("youtube-video")}><span><Youtube size={23} /></span><strong>{t("music.builder.youtubeVideo")}</strong><small>{t("music.builder.youtubeVideoDescription")}</small></button>
          <button type="button" class="source-choice" onclick={() => chooseYouTube("youtube-playlist")}><span><Link2 size={22} /></span><strong>{t("music.builder.youtubePlaylist")}</strong><small>{t("music.builder.youtubePlaylistDescription")}</small></button>
        </div>
      {:else if step === "local" && localSelection}
        <div class="space-y-3">
          <div class="rounded-xl bg-secondary/55 p-3"><div class="flex items-start gap-2"><FolderOpen class="mt-0.5 shrink-0 text-muted-foreground" size={17} /><div class="min-w-0"><p class="truncate text-xs font-semibold" title={localSelection.folderPath}>{localSelection.folderPath.split(/[\\/]/).filter(Boolean).at(-1)}</p><p class="mt-1 text-[0.68rem] text-muted-foreground">{t("music.builder.previewFound", localSelection.tracks.length)}</p></div></div>{#if localSelection.truncated}<p class="mt-2 rounded-lg bg-background/55 p-2 text-[0.65rem] leading-relaxed text-muted-foreground">{t("music.builder.previewTruncated")}</p>{/if}</div>
          <label class="block"><span class="mb-1 block text-[0.68rem] font-medium">{t("music.builder.sourceName")}</span><input bind:value={name} class="source-input" maxlength="200" /></label>
          <p class="rounded-xl border border-border/55 bg-card/45 p-2.5 text-[0.68rem] leading-relaxed text-muted-foreground">{t("music.builder.referenceOnlyNotice")}</p>
          {#if localRelationship === "duplicate"}<p class="text-[0.68rem] text-destructive">{t("music.builder.duplicateFolder")}</p>{:else if localRelationship !== "separate"}<p class="text-[0.68rem] text-destructive">{t("music.builder.nestedFolder")}</p>{/if}
          <button type="button" class="text-[0.68rem] font-medium text-primary hover:underline" onclick={() => { void chooseLocal(); }}>{t("music.builder.chooseDifferentFolder")}</button>
        </div>
      {:else if step === "youtube"}
        <div class="space-y-3">
          <div class="flex items-start gap-2 rounded-xl border border-border/55 bg-card/45 p-2.5 text-[0.68rem] leading-relaxed text-muted-foreground"><Network class="mt-0.5 shrink-0" size={14} /><span>{t("music.builder.networkNotice")}</span></div>
          <label class="block"><span class="mb-1 block text-[0.68rem] font-medium">{t("music.builder.youtubeLink")}</span><div class="flex gap-1.5"><input bind:value={link} oninput={() => { preview = null; error = null; }} class="source-input min-w-0 flex-1" placeholder="https://www.youtube.com/…" /><button type="button" onclick={() => { void resolveLink(); }} disabled={controller.resolving || !link.trim()} class="source-action">{#if controller.resolving}<LoaderCircle class="animate-spin motion-reduce:animate-none" size={13} />{:else}{t("music.builder.resolveLink")}{/if}</button></div></label>
          {#if controller.resolving}<div class="grid min-h-28 place-items-center rounded-xl bg-secondary/35"><div class="text-center"><LoaderCircle class="mx-auto animate-spin text-muted-foreground motion-reduce:animate-none" size={20} /><p class="mt-2 text-[0.68rem] text-muted-foreground">{t("music.builder.resolvingLink")}</p></div></div>{/if}
          {#if preview}
            <div class="rounded-xl border border-primary/25 bg-primary/5 p-3"><div class="flex items-center gap-2"><span class="grid h-10 w-10 place-items-center rounded-xl bg-secondary text-muted-foreground"><Youtube size={20} /></span><div class="min-w-0 flex-1"><strong class="block truncate text-xs">{preview.title}</strong><span class="mt-0.5 block truncate text-[0.65rem] text-muted-foreground">{preview.channel || (preview.playlistId ?? preview.videoId)}</span></div></div>{#if preview.kind === "youtube-playlist"}<p class="mt-2 text-[0.68rem] text-muted-foreground">{t("music.builder.videosFound", preview.videoIds.length)}</p>{#if preview.duplicateCount > 0}<p class="mt-1 text-[0.63rem] leading-relaxed text-muted-foreground">{t("music.builder.knownDuplicates", preview.duplicateCount)}</p>{/if}<div class="mt-1.5 flex flex-wrap gap-1">{#each preview.videoIds.slice(0, 6) as videoId}<span class="max-w-28 truncate rounded-md bg-secondary px-1.5 py-1 text-[0.58rem] text-muted-foreground">{videoId}</span>{/each}</div>{/if}</div>
            {#if preview.kind === "youtube-playlist"}<label class="block"><span class="mb-1 block text-[0.68rem] font-medium">{t("music.builder.sourceName")}</span><input bind:value={name} class="source-input" maxlength="200" /></label>{/if}
          {/if}
        </div>
      {/if}
      {#if error}<p class="mt-3 rounded-lg bg-destructive/10 px-2.5 py-2 text-[0.68rem] text-destructive" role="alert">{error}</p>{/if}
    </div>

    {#if step !== "choose"}
      <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border/55 px-3 py-2">
        <button type="button" onclick={onClose} class="source-cancel">{t("music.builder.cancel")}</button>
        {#if step === "local"}<button type="button" onclick={() => { void saveLocal(); }} disabled={!localSelection || !name.trim() || localRelationship === "duplicate" || saving} class="source-save">{t("music.builder.saveAndScan")}</button>{:else}<button type="button" onclick={() => { void saveYouTube(); }} disabled={!preview || saving} class="source-save">{t("music.builder.addToLibrary")}</button>{/if}
      </footer>
    {/if}
  </div>
</div>

<style>
  .source-dialog-scroll { scrollbar-width: thin; scrollbar-color: color-mix(in srgb, var(--foreground) 18%, transparent) transparent; }
  .source-choice { display: flex; min-height: 10rem; flex-direction: column; align-items: flex-start; gap: 0.5rem; border: 1px solid color-mix(in srgb, var(--border) 68%, transparent); border-radius: 0.9rem; background: color-mix(in srgb, var(--card) 65%, transparent); padding: 0.8rem; text-align: left; transition: border-color 130ms ease, transform 130ms ease, background-color 130ms ease; }
  .source-choice:hover { border-color: color-mix(in srgb, var(--primary) 35%, var(--border)); background: color-mix(in srgb, var(--primary) 5%, var(--card)); transform: translateY(-1px); }
  .source-choice > span { display: grid; height: 2.5rem; width: 2.5rem; place-items: center; border-radius: 0.75rem; background: var(--secondary); color: var(--muted-foreground); }
  .source-choice strong { font-size: calc(0.75rem * var(--type-scale)); }
  .source-choice small { color: var(--muted-foreground); font-size: calc(0.65rem * var(--type-scale)); line-height: 1.45; }
  .source-input { height: 2.1rem; width: 100%; border: 1px solid color-mix(in srgb, var(--border) 80%, transparent); border-radius: 0.6rem; background: color-mix(in srgb, var(--background) 65%, transparent); padding-inline: 0.65rem; color: var(--foreground); font-size: calc(0.72rem * var(--type-scale)); outline: none; }
  .source-input:focus { border-color: color-mix(in srgb, var(--ring) 65%, transparent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--ring) 18%, transparent); }
  .source-action, .source-save, .source-cancel { display: inline-flex; height: 2.1rem; align-items: center; justify-content: center; gap: 0.35rem; border-radius: 0.6rem; padding-inline: 0.8rem; font-size: calc(0.68rem * var(--type-scale)); font-weight: 600; white-space: nowrap; }
  .source-action, .source-save { background: var(--primary); color: var(--primary-foreground); }
  .source-cancel { background: var(--secondary); color: var(--secondary-foreground); }
  .source-action:disabled, .source-save:disabled { opacity: 0.45; }
  @media (prefers-reduced-motion: reduce) { .source-choice, .source-choice:hover { transition: none; transform: none; } }
</style>
