<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { projectTaskArchivedBadgeClass } from "$lib/projects/project-display";
  import type { ProjectTask } from "$lib/projects/types";
  import { cn } from "$lib/utils";

  let {
    task,
    title,
    onTitleChange,
    onClose,
  }: {
    task: ProjectTask;
    title: string;
    onTitleChange: (value: string) => void;
    onClose: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<header class="flex shrink-0 items-center gap-2 border-b border-border bg-card px-4 py-3">
  <div class="min-w-0 flex-1">
    <div class="flex min-w-0 items-center gap-2">
      <input
        value={title}
        aria-label={t("projects.detail.titleLabel")}
        class="min-h-8 min-w-0 flex-1 rounded-md bg-transparent px-1 text-[1.1rem] font-semibold text-foreground outline-none focus:bg-background focus:ring-2 focus:ring-ring/30"
        oninput={(event) => onTitleChange(event.currentTarget.value)}
      />
      {#if task.archivedAt}
        <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectTaskArchivedBadgeClass(task))}>
          {t("projects.taskLifecycle.archived")}
        </span>
      {/if}
    </div>
  </div>
  <button
    type="button"
    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
    aria-label={t("projects.detail.close")}
    title={t("projects.detail.close")}
    onclick={onClose}
  >
    <X size={15} strokeWidth={1.75} />
  </button>
</header>
