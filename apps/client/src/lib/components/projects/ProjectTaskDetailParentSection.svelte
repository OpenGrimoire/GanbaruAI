<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectTask } from "$lib/projects/types";

  type ActionResult = void | Promise<void>;

  let {
    task,
    parentTask,
    parentCandidates,
    parentSearch,
    hasSubtasks,
    onParentSearchChange,
    onPromoteSubtask,
    onDemoteTask,
  }: {
    task: ProjectTask;
    parentTask: ProjectTask | undefined;
    parentCandidates: ProjectTask[];
    parentSearch: string;
    hasSubtasks: boolean;
    onParentSearchChange: (value: string) => void;
    onPromoteSubtask: (task: ProjectTask) => ActionResult;
    onDemoteTask: (task: ProjectTask, parentTask: ProjectTask) => ActionResult;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex items-center justify-between gap-2">
  <h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.parentTask")}</h2>
  {#if task.parentTaskId}
    <button
      type="button"
      class="flex min-h-7 items-center gap-1 rounded-md border border-border bg-background px-2 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
      onclick={() => { void onPromoteSubtask(task); }}
    >
      <ArrowLeft size={13} strokeWidth={1.75} />
      <span>{t("projects.detail.promoteSubtask")}</span>
    </button>
  {/if}
</div>
{#if task.parentTaskId}
  <div class="rounded-md border border-border bg-background px-2 py-1.5 text-[0.8rem]">
    {parentTask?.title ?? t("projects.detail.missingDependencyTask")}
  </div>
{:else if hasSubtasks}
  <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
    {t("projects.detail.demoteBlockedBySubtasks")}
  </div>
{:else}
  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.detail.demoteToParent")}</span>
    <input
      value={parentSearch}
      placeholder={t("projects.detail.demoteToParentPlaceholder")}
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground placeholder:text-muted-foreground"
      oninput={(event) => onParentSearchChange(event.currentTarget.value)}
    />
  </label>
  <div class="grid gap-1">
    {#each parentCandidates as candidate (candidate.id)}
      <button
        type="button"
        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left text-muted-foreground hover:bg-accent hover:text-foreground"
        onclick={() => { void onDemoteTask(task, candidate); }}
      >
        <span class="truncate text-[0.8rem] text-foreground">{candidate.title}</span>
        <ArrowRight size={13} strokeWidth={1.75} />
      </button>
    {:else}
      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("projects.detail.noParentCandidates")}
      </div>
    {/each}
  </div>
{/if}
