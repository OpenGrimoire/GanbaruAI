<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { ProjectTask, ProjectTaskDependency } from "$lib/projects/types";

  type ActionResult = void | Promise<void>;

  let {
    task,
    blockedByDependencies,
    blocksDependencies,
    dependencyCandidates,
    dependencySearch,
    taskById,
    onDependencySearchChange,
    onAddBlockingDependency,
    onRemoveDependency,
  }: {
    task: ProjectTask;
    blockedByDependencies: ProjectTaskDependency[];
    blocksDependencies: ProjectTaskDependency[];
    dependencyCandidates: ProjectTask[];
    dependencySearch: string;
    taskById: (taskId: string) => ProjectTask | undefined;
    onDependencySearchChange: (value: string) => void;
    onAddBlockingDependency: (blockingTask: ProjectTask, blockedTask: ProjectTask) => ActionResult;
    onRemoveDependency: (dependencyId: string) => ActionResult;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex items-center justify-between gap-2">
  <h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.dependencies")}</h2>
  <span class="text-[0.733333rem] text-muted-foreground">
    {blockedByDependencies.length + blocksDependencies.length}
  </span>
</div>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blockedBy")}</div>
  {#each blockedByDependencies as dependency (dependency.id)}
    {@const blockingTask = taskById(dependency.blockingTaskId)}
    <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
      <span class="truncate text-[0.8rem]">
        {blockingTask?.title ?? t("projects.detail.missingDependencyTask")}
      </span>
      <button
        type="button"
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
        aria-label={t("projects.actions.deleteDependency")}
        title={t("projects.actions.deleteDependency")}
        onclick={() => { void onRemoveDependency(dependency.id); }}
      >
        <Trash2 size={13} strokeWidth={1.75} />
      </button>
    </div>
  {:else}
    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
      {t("projects.detail.noBlockedBy")}
    </div>
  {/each}
</div>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.blocks")}</div>
  {#each blocksDependencies as dependency (dependency.id)}
    {@const blockedTask = taskById(dependency.blockedTaskId)}
    <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2">
      <span class="truncate text-[0.8rem]">
        {blockedTask?.title ?? t("projects.detail.missingDependencyTask")}
      </span>
      <button
        type="button"
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
        aria-label={t("projects.actions.deleteDependency")}
        title={t("projects.actions.deleteDependency")}
        onclick={() => { void onRemoveDependency(dependency.id); }}
      >
        <Trash2 size={13} strokeWidth={1.75} />
      </button>
    </div>
  {:else}
    <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
      {t("projects.detail.noBlocks")}
    </div>
  {/each}
</div>

<div class="grid gap-1">
  <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
    <span>{t("projects.detail.addBlockedBy")}</span>
    <input
      value={dependencySearch}
      placeholder={t("projects.detail.addBlockedByPlaceholder")}
      class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
      oninput={(event) => onDependencySearchChange(event.currentTarget.value)}
    />
  </label>
  <div class="grid gap-1">
    {#each dependencyCandidates as candidate (candidate.id)}
      <button
        type="button"
        class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-md border border-border bg-background px-2 text-left hover:bg-accent"
        onclick={() => { void onAddBlockingDependency(candidate, task); }}
      >
        <span class="truncate text-[0.8rem]">{candidate.title}</span>
        <Plus size={13} strokeWidth={1.75} class="text-muted-foreground" />
      </button>
    {:else}
      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("projects.detail.noDependencyCandidates")}
      </div>
    {/each}
  </div>
</div>
