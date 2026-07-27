<script lang="ts">
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FileDiff from "@lucide/svelte/icons/file-diff";
  import type { ChangedFileSummary, ChatTurnId } from "$lib/chat/contracts";
  import { formatNumber } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import ChatFileIcon from "./ChatFileIcon.svelte";

  let { turnId, files }: { turnId: ChatTurnId; files: ChangedFileSummary[] } = $props();
  const localization = getLocalization();
  const { t } = localization;
  const additions = $derived(files.reduce((total, file) => total + (file.additions ?? 0), 0));
  const deletions = $derived(files.reduce((total, file) => total + (file.deletions ?? 0), 0));

  function openChanges(relativePath: string | null): void {
    window.dispatchEvent(new CustomEvent("ganbaru-ai:chat-open-changes", {
      detail: { turnId, relativePath },
    }));
  }
</script>

<section class="changed-files-card">
  <button type="button" class="changed-files-header" onclick={() => openChanges(files[0]?.relativePath ?? null)}>
    <FileDiff size={14} />
    <strong>{t("chat.timeline.changedFiles", formatNumber(localization.locale, files.length))}</strong>
    {#if additions > 0}<span class="additions">+{formatNumber(localization.locale, additions)}</span>{/if}
    {#if deletions > 0}<span class="deletions">−{formatNumber(localization.locale, deletions)}</span>{/if}
    <span class="view-diff">{t("chat.timeline.viewDiff")}<ChevronRight size={13} /></span>
  </button>
  <div class="changed-files-list">
    {#each files as file (file.relativePath)}
      <button type="button" onclick={() => openChanges(file.relativePath)}>
        <ChatFileIcon path={file.relativePath} size={13} />
        <span title={file.relativePath}>{file.relativePath}</span>
        {#if file.additions}<small class="additions">+{formatNumber(localization.locale, file.additions)}</small>{/if}
        {#if file.deletions}<small class="deletions">−{formatNumber(localization.locale, file.deletions)}</small>{/if}
      </button>
    {/each}
  </div>
</section>

<style>
  .changed-files-card { overflow: hidden; margin-top: 0.9rem; border: 1px solid var(--border); border-radius: 0.8rem; background: color-mix(in srgb, var(--muted) 24%, var(--background)); }
  .changed-files-header { display: flex; width: 100%; min-width: 0; align-items: center; gap: 0.45rem; padding: 0.6rem 0.7rem; text-align: left; }
  .changed-files-header:hover { background: color-mix(in srgb, var(--accent) 45%, transparent); }
  .changed-files-header strong { font-size: 0.78rem; font-weight: 600; }
  .additions { color: var(--action-confirm); }
  .deletions { color: var(--destructive); }
  .view-diff { display: inline-flex; margin-left: auto; align-items: center; gap: 0.15rem; color: var(--muted-foreground); font-size: 0.7rem; }
  .changed-files-list { display: grid; max-height: 12rem; overflow: auto; border-top: 1px solid var(--border); padding: 0.3rem; }
  .changed-files-list button { display: flex; min-width: 0; align-items: center; gap: 0.4rem; border-radius: 0.45rem; padding: 0.35rem 0.45rem; text-align: left; }
  .changed-files-list button:hover { background: var(--accent); }
  .changed-files-list button > span { min-width: 0; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: "SF Mono", "SFMono-Regular", Consolas, monospace; font-size: 0.72rem; }
  .changed-files-list small { font-family: "SF Mono", "SFMono-Regular", Consolas, monospace; font-size: 0.66rem; }
</style>
