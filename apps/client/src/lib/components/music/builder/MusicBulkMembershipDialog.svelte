<script lang="ts">
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { containMusicDialogFocus } from "$lib/music/music-dialog-focus";
  import type { MusicBulkEditController } from "$lib/music/music-bulk-edit-controller.svelte";
  import type { MusicPlaylistSummary } from "$lib/music/library-contracts";
  import MusicPlaylistPicker from "./MusicPlaylistPicker.svelte";

  let {
    controller,
    playlists,
    onClose,
    onSaved,
  }: {
    controller: MusicBulkEditController;
    playlists: MusicPlaylistSummary[];
    onClose: () => void;
    onSaved: () => void;
  } = $props();
  const { t } = getLocalization();

  async function save(): Promise<void> {
    if (await controller.saveMemberships()) onSaved();
  }
</script>

<div class="absolute inset-0 z-60 grid place-items-center bg-background/65 p-3 backdrop-blur-sm">
  <div use:containMusicDialogFocus={{ onEscape: onClose, escapeDisabled: controller.saving }} role="dialog" aria-modal="true" aria-labelledby="music-bulk-membership-title" tabindex="-1" class="flex h-[min(28rem,100%)] max-h-full w-full max-w-md flex-col overflow-hidden rounded-xl border border-border/70 bg-card shadow-2xl">
    <header class="flex shrink-0 items-center gap-3 border-b border-border/60 p-4">
      <div class="grid h-9 w-9 place-items-center rounded-lg bg-primary/10 text-primary"><ListPlus size={17} /></div>
      <div class="min-w-0">
        <h2 id="music-bulk-membership-title" class="text-sm font-semibold">{t("music.builder.bulkMembershipTitle")}</h2>
        <p class="mt-0.5 text-[0.68rem] text-muted-foreground">{t("music.builder.bulkMembershipDescription", controller.itemIds.length)}</p>
      </div>
    </header>
    {#if controller.loading}
      <div class="m-3 min-h-44 animate-pulse rounded-xl bg-secondary motion-reduce:animate-none"></div>
    {:else}
      <MusicPlaylistPicker
        {playlists}
        checkedIds={controller.checkedIds}
        mixedIds={controller.mixedIds}
        search={controller.search}
        onSearch={(value) => controller.search = value}
        onToggle={(playlist) => controller.toggle(playlist.id)}
      />
    {/if}
    {#if controller.error}<p class="shrink-0 px-4 pb-2 text-[0.68rem] text-destructive" role="alert">{controller.error}</p>{/if}
    <footer class="flex shrink-0 flex-wrap items-center justify-between gap-2 border-t border-border/60 p-3">
      <p class="min-w-32 flex-1 text-[0.62rem] leading-relaxed text-muted-foreground">{t("music.builder.mixedMembershipHint")}</p>
      <div class="flex shrink-0 gap-2">
        <button type="button" onclick={onClose} disabled={controller.saving} class="h-8 rounded-md bg-secondary px-3 text-xs font-medium disabled:opacity-50">{t("music.builder.cancel")}</button>
        <button type="button" onclick={() => { void save(); }} disabled={controller.loading || controller.saving} class="h-8 rounded-md bg-primary px-3 text-xs font-semibold text-primary-foreground disabled:opacity-40">{controller.saving ? t("music.builder.saving") : t("music.builder.applyChanges")}</button>
      </div>
    </footer>
  </div>
</div>
