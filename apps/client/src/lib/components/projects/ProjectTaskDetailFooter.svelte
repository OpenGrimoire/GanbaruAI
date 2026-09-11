<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import ArchiveRestore from "@lucide/svelte/icons/archive-restore";
  import Save from "@lucide/svelte/icons/save";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectTask } from "$lib/projects/types";
  import { cn } from "$lib/utils";

  type ActionResult = void | Promise<void>;

  let {
    task,
    saving,
    dirty,
    onArchive,
    onRestore,
  }: {
    task: ProjectTask;
    saving: boolean;
    dirty: boolean;
    onArchive: (task: ProjectTask) => ActionResult;
    onRestore: (task: ProjectTask) => ActionResult;
  } = $props();

  const { t } = getLocalization();
</script>

<footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border px-3 py-2">
  {#if task.archivedAt}
    <button
      type="button"
      class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed"
      disabled={saving}
      onclick={() => { void onRestore(task); }}
    >
      <ArchiveRestore size={14} strokeWidth={1.75} />
      <span>{t("projects.detail.restore")}</span>
    </button>
  {:else}
    <button
      type="button"
      class="mr-auto flex min-h-8 items-center gap-1.5 rounded-md border border-destructive/30 bg-destructive/10 px-2 text-[0.8rem] text-destructive hover:bg-destructive/15 disabled:cursor-not-allowed"
      disabled={saving}
      onclick={() => { void onArchive(task); }}
    >
      <Archive size={14} strokeWidth={1.75} />
      <span>{t("projects.detail.archive")}</span>
    </button>
  {/if}
  <button
    type="submit"
    class={cn(
      "flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed",
      dirty || saving ? "hover:bg-primary/90" : "opacity-60",
    )}
    disabled={saving || !dirty}
  >
    <Save size={14} strokeWidth={1.75} />
    <span>{t("projects.detail.save")}</span>
  </button>
</footer>
