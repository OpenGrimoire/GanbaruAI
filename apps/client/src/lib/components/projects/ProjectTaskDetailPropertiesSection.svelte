<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { projectTaskTypeLabel } from "$lib/projects/project-display";
  import { PROJECT_TASK_TYPES } from "$lib/projects/types";
  import type {
    ProjectPriority,
    ProjectPriorityConfig,
    ProjectSection,
    ProjectStatus,
    ProjectTaskType,
  } from "$lib/projects/types";
  import type { Theme } from "$lib/stores/themes";
  import { cn } from "$lib/utils";
  import PriorityFlagIcon from "./PriorityFlagIcon.svelte";
  import ProjectStatusBadge from "./ProjectStatusBadge.svelte";

  let {
    statuses,
    sections,
    priorities,
    theme,
    statusId,
    sectionId,
    priority,
    taskType,
    milestone,
    onStatusChange,
    onSectionChange,
    onPriorityChange,
    onTaskTypeChange,
    onMilestoneChange,
  }: {
    statuses: ProjectStatus[];
    sections: ProjectSection[];
    priorities: ProjectPriorityConfig[];
    theme: Theme;
    statusId: string;
    sectionId: string;
    priority: ProjectPriority;
    taskType: ProjectTaskType;
    milestone: boolean;
    onStatusChange: (statusId: string) => void;
    onSectionChange: (sectionId: string) => void;
    onPriorityChange: (priority: ProjectPriority) => void;
    onTaskTypeChange: (taskType: ProjectTaskType) => void;
    onMilestoneChange: (milestone: boolean) => void;
  } = $props();

  const { t } = getLocalization();
</script>

<h2 class="text-[0.8rem] font-semibold tracking-normal">{t("projects.detail.properties")}</h2>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.status")}</div>
  <div class="flex flex-wrap gap-1">
    {#each statuses as status (status.id)}
      <button
        type="button"
        class={cn(
          "rounded-full outline-none transition-shadow hover:ring-1 hover:ring-ring/50 focus:ring-1 focus:ring-ring",
          statusId === status.id && "ring-1 ring-ring",
        )}
        onclick={() => onStatusChange(status.id)}
      >
        <ProjectStatusBadge
          {status}
          {theme}
          label={status.name}
          class="text-[0.766667rem]"
        />
      </button>
    {/each}
  </div>
</div>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.section")}</div>
  <div class="flex flex-wrap gap-1">
    {#each sections as section (section.id)}
      <button
        type="button"
        class={cn(
          "rounded-md border px-2 py-1 text-[0.766667rem]",
          sectionId === section.id
            ? "border-primary/50 bg-primary/10 text-primary"
            : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
        )}
        onclick={() => onSectionChange(section.id)}
      >
        {section.name}
      </button>
    {/each}
  </div>
</div>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.priority")}</div>
  <div class="flex flex-wrap gap-1">
    {#each priorities as priorityConfig (priorityConfig.id)}
      <button
        type="button"
        class={cn(
          "flex min-h-7 items-center gap-1.5 rounded-md px-2 text-[0.766667rem] text-foreground hover:bg-accent",
          priority === priorityConfig.id && "bg-accent",
        )}
        onclick={() => onPriorityChange(priorityConfig.id)}
      >
        <PriorityFlagIcon color={priorityConfig.color} {theme} size={13} class="shrink-0" />
        <span>{priorityConfig.name}</span>
      </button>
    {/each}
  </div>
</div>

<div class="grid gap-1">
  <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.detail.type")}</div>
  <div class="flex flex-wrap gap-1">
    {#each PROJECT_TASK_TYPES as option}
      <button
        type="button"
        class={cn(
          "rounded-md border px-2 py-1 text-[0.766667rem]",
          taskType === option
            ? "border-primary/50 bg-primary/10 text-primary"
            : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
        )}
        onclick={() => onTaskTypeChange(option)}
      >
        {projectTaskTypeLabel(option, t)}
      </button>
    {/each}
  </div>
</div>

<label class="flex min-h-8 items-center gap-2 rounded-md border border-border bg-background px-2 text-[0.8rem]">
  <input
    type="checkbox"
    checked={milestone}
    class="h-4 w-4 accent-primary"
    onchange={(event) => onMilestoneChange(event.currentTarget.checked)}
  />
  <span>{t("projects.detail.milestone")}</span>
</label>
