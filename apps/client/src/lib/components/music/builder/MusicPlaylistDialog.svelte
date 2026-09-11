<script lang="ts">
  import Copy from "@lucide/svelte/icons/copy";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { onMount } from "svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import IconPicker from "$lib/components/icon-picker/IconPicker.svelte";
  import MusicPlaylistIcon from "./MusicPlaylistIcon.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicPlaylistController, MusicPlaylistDraft } from "$lib/music/music-playlist-controller.svelte";
  import type { MusicIntendedUse, MusicRepeatMode } from "$lib/music/library-contracts";
  import { isSystemMusicPlaylistId, systemMusicPlaylistName } from "$lib/music/music-system-playlists";

  let {
    controller,
    mode,
    onClose,
    onSaved,
    onDeleted = () => undefined,
    playlists = [],
    activeInPlayer = false,
  }: {
    controller: MusicPlaylistController;
    mode: "create" | "edit" | "duplicate" | "delete";
    onClose: () => void;
    onSaved: (playlistId: string) => void;
    onDeleted?: (replacementPlaylistId: string | null) => void;
    playlists?: import("$lib/music/library-contracts").MusicPlaylistSummary[];
    activeInPlayer?: boolean;
  } = $props();

  const { t } = getLocalization();
  let name = $state("");
  let icon = $state("lucide:list-music");
  let shuffleEnabled = $state(true);
  let repeatMode = $state<MusicRepeatMode>("all");
  let intendedUses = $state<MusicIntendedUse[]>([]);
  let replacementPlaylistId = $state("");
  const useOptions: MusicIntendedUse[] = ["general", "focus", "reading", "relaxation", "energizing"];
  const protectedIdentity = $derived(Boolean(controller.detail && isSystemMusicPlaylistId(controller.detail.id)));

  onMount(() => {
    const detail = controller.detail;
    if (detail) {
      const displayName = systemMusicPlaylistName(detail.id, detail.name, t);
      name = mode === "duplicate" ? t("music.builder.playlistCopyName", displayName) : displayName;
      icon = detail.icon;
      shuffleEnabled = detail.shuffleEnabled;
      repeatMode = detail.repeatMode;
      intendedUses = [...detail.intendedUses];
      if (mode === "delete" && !controller.deleteImpact) void controller.inspectDelete();
    }
  });

  function title(): string {
    if (mode === "create") return t("music.builder.createPlaylistTitle");
    if (mode === "edit") return t("music.builder.editPlaylistTitle");
    if (mode === "duplicate") return t("music.builder.duplicatePlaylistTitle");
    return t("music.builder.deletePlaylistTitle");
  }

  function toggleUse(use: MusicIntendedUse): void {
    intendedUses = intendedUses.includes(use)
      ? intendedUses.filter((entry) => entry !== use)
      : [...intendedUses, use];
  }

  function useLabel(use: MusicIntendedUse): string {
    return t(`music.builder.intendedUse.${use}`);
  }

  async function save(): Promise<void> {
    if (mode === "delete") {
      const replacement = replacementPlaylistId || null;
      if (await controller.remove(replacement)) onDeleted(replacement);
      return;
    }
    const draft: MusicPlaylistDraft = { name, icon, shuffleEnabled, repeatMode, intendedUses };
    if (mode === "create") {
      const playlistId = await controller.create(draft);
      if (playlistId) onSaved(playlistId);
      return;
    }
    if (mode === "duplicate") {
      const playlistId = await controller.duplicate(name);
      if (playlistId) onSaved(playlistId);
      return;
    }
    if (await controller.update(draft) && controller.detail) onSaved(controller.detail.id);
  }
</script>

<div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm">
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: controller.saving }} role="dialog" aria-modal="true" aria-labelledby="music-playlist-dialog-title" class="flex max-h-full w-full max-w-md flex-col overflow-hidden rounded-xl border border-border/70 bg-card shadow-2xl" tabindex="-1">
    <header class="flex shrink-0 items-center gap-3 border-b border-border/60 p-4">
      <div class="grid h-9 w-9 place-items-center rounded-lg bg-secondary text-muted-foreground">
        {#if mode === "delete"}<Trash2 size={17} />{:else if mode === "duplicate"}<Copy size={17} />{:else}<ListMusic size={17} />{/if}
      </div>
      <div class="min-w-0"><h2 id="music-playlist-dialog-title" class="text-sm font-semibold">{title()}</h2><p class="mt-0.5 text-[0.68rem] text-muted-foreground">{t("music.builder.playlistDialogDescription")}</p></div>
    </header>

    <div class="min-h-0 overflow-y-auto p-4">
      {#if mode === "delete"}
        {#if controller.deleteImpact}
          <p class="text-xs leading-relaxed">{t("music.builder.deletePlaylistWarning", controller.detail?.name ?? "")}</p>
          <div class="mt-3 grid grid-cols-2 gap-2 text-[0.68rem] sm:grid-cols-3">
            <div class="rounded-lg bg-secondary/65 p-2"><strong class="block text-sm">{controller.deleteImpact.membershipCount}</strong><span class="text-muted-foreground">{t("music.builder.memberships")}</span></div>
            <div class="rounded-lg bg-secondary/65 p-2"><strong class="block text-sm">{controller.deleteImpact.projectFocusAssignmentCount + controller.deleteImpact.projectBreakAssignmentCount}</strong><span class="text-muted-foreground">{t("music.builder.projectAssignments")}</span></div>
            <div class="rounded-lg bg-secondary/65 p-2"><strong class="block text-sm">{controller.deleteImpact.calendarAssignmentCount}</strong><span class="text-muted-foreground">{t("music.builder.eventAssignments")}</span></div>
            <div class="rounded-lg bg-secondary/65 p-2"><strong class="block text-sm">{controller.deleteImpact.contextAssignmentCount}</strong><span class="text-muted-foreground">{t("music.builder.contextAssignments")}</span></div>
            <div class="rounded-lg bg-secondary/65 p-2"><strong class="block text-sm">0</strong><span class="text-muted-foreground">{t("music.builder.mediaFilesDeleted")}</span></div>
          </div>
          <p class="mt-3 rounded-lg bg-warning/10 p-2.5 text-[0.68rem] leading-relaxed text-warning">{t("music.builder.deleteAssignmentFallback")}</p>
          {#if activeInPlayer}<p class="mt-2 rounded-lg border border-primary/25 bg-primary/8 p-2.5 text-[0.68rem] leading-relaxed">{t("music.builder.activePlaylistDeleteBehavior")}</p>{/if}
          {#if controller.deleteImpact.assignments.length > 0}
            <div class="mt-3 max-h-32 space-y-1 overflow-y-auto rounded-lg border border-border/60 bg-background p-1.5" data-music-scrollable="true">
              {#each controller.deleteImpact.assignments as assignment (`${assignment.kind}:${assignment.id}`)}
                <div class="flex min-w-0 items-center gap-2 rounded-md px-2 py-1.5 text-[0.67rem]"><span class="shrink-0 rounded bg-secondary px-1.5 py-0.5 text-[0.58rem] text-muted-foreground">{t(`music.builder.assignmentKind.${assignment.kind}`)}</span><span class="min-w-0 flex-1 truncate" title={assignment.label}>{assignment.label || assignment.id}</span></div>
              {/each}
            </div>
          {/if}
          {#if controller.deleteImpact.projectFocusAssignmentCount + controller.deleteImpact.projectBreakAssignmentCount + controller.deleteImpact.calendarAssignmentCount + controller.deleteImpact.contextAssignmentCount > 0}
            <label class="mt-3 block text-[0.7rem] font-medium" for="music-delete-replacement">{t("music.builder.replacementPlaylist")}</label>
            <select id="music-delete-replacement" bind:value={replacementPlaylistId} class="mt-1.5 h-9 w-full rounded-md border border-border/70 bg-background px-2 text-xs">
              <option value="">{t("music.builder.safeNoPlaylist")}</option>
              {#each playlists.filter((entry) => entry.id !== controller.detail?.id) as playlist}<option value={playlist.id}>{systemMusicPlaylistName(playlist.id, playlist.name, t)}</option>{/each}
            </select>
          {/if}
        {:else}
          <div class="h-28 animate-pulse rounded-lg bg-secondary motion-reduce:animate-none"></div>
        {/if}
      {:else}
        <label class="block text-[0.7rem] font-medium" for="music-playlist-name">{t("music.builder.playlistName")}</label>
        <div class="mt-1.5 flex items-center gap-2">
          {#if mode !== "duplicate" && !protectedIdentity}
            <IconPicker value={icon} onChange={(value) => icon = value} ariaLabel={t("music.builder.selectPlaylistIcon")} showUpload={false}>
              {#snippet trigger({ open, toggle, panelId })}
                <button
                  type="button"
                  class={`grid h-9 w-9 shrink-0 place-items-center rounded-lg bg-secondary text-foreground transition-colors hover:bg-accent ${open ? "bg-accent" : ""}`}
                  aria-label={t("music.builder.selectPlaylistIcon")}
                  aria-haspopup="dialog"
                  aria-expanded={open}
                  aria-controls={panelId}
                  title={t("music.builder.selectPlaylistIcon")}
                  onclick={toggle}
                >
                  <MusicPlaylistIcon {icon} size={18} strokeWidth={1.6} />
                </button>
              {/snippet}
            </IconPicker>
          {/if}
          <input data-dialog-autofocus id="music-playlist-name" bind:value={name} disabled={protectedIdentity && mode === "edit"} class="h-9 min-w-0 flex-1 rounded-md border border-border/70 bg-background px-3 text-xs outline-none focus:border-primary disabled:opacity-60" />
        </div>
        {#if mode !== "duplicate"}
          <fieldset class="mt-3"><legend class="text-[0.7rem] font-medium">{t("music.builder.intendedUses")}</legend><div class="mt-1.5 flex flex-wrap gap-1.5">{#each useOptions as use}<button type="button" aria-pressed={intendedUses.includes(use)} onclick={() => toggleUse(use)} title={t(`music.builder.intendedUseDescription.${use}`)} class="rounded-full border border-border/70 px-2.5 py-1 text-[0.68rem] aria-pressed:border-primary aria-pressed:bg-primary/10">{useLabel(use)}</button>{/each}</div>{#if intendedUses.length > 0}<div class="mt-2 space-y-1 rounded-lg bg-secondary/45 p-2">{#each intendedUses as use}<p class="text-[0.62rem] leading-relaxed text-muted-foreground"><strong class="text-foreground">{useLabel(use)}:</strong> {t(`music.builder.intendedUseDescription.${use}`)}</p>{/each}</div>{/if}</fieldset>
          <div class="mt-3 grid grid-cols-2 gap-3">
            <label class="flex items-center gap-2 rounded-lg bg-secondary/55 p-2 text-[0.68rem]"><input type="checkbox" bind:checked={shuffleEnabled} class="accent-primary" />{t("music.builder.shuffleDefault")}</label>
            <label class="text-[0.68rem]"><span class="block font-medium">{t("music.builder.repeatDefault")}</span><select bind:value={repeatMode} class="mt-1 h-8 w-full rounded-md border border-border/70 bg-background px-2"><option value="off">{t("music.builder.repeatOff")}</option><option value="all">{t("music.builder.repeatAll")}</option><option value="one">{t("music.builder.repeatOne")}</option></select></label>
          </div>
        {/if}
      {/if}
      {#if controller.error}<p class="mt-3 text-[0.68rem] text-destructive" role="alert">{controller.error}</p>{/if}
    </div>

    <footer class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-border/60 p-3">
      <button type="button" onclick={onClose} disabled={controller.saving} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium disabled:opacity-50">{t("music.builder.cancel")}</button>
      <button type="button" onclick={() => { void save(); }} disabled={controller.saving || (mode !== "delete" && !name.trim()) || (mode === "delete" && !controller.deleteImpact)} class={mode === "delete" ? "h-8 rounded-md bg-destructive px-3 text-xs font-semibold text-destructive-foreground disabled:opacity-40" : "h-8 rounded-md bg-primary px-3 text-xs font-semibold text-primary-foreground disabled:opacity-40"}>{mode === "delete" ? t("music.builder.deletePlaylist") : t("music.builder.savePlaylist")}</button>
    </footer>
  </div>
</div>
