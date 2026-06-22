<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { projectTaskHistoryEventLabel } from "$lib/projects/project-display";
  import type { ProjectTaskChangeEvent } from "$lib/projects/types";

  let {
    events,
  }: {
    events: ProjectTaskChangeEvent[];
  } = $props();

  const { t } = getLocalization();
</script>

<section class="task-detail-section">
  <div class="flex items-center justify-between gap-2">
    <h2 class="task-detail-section-title">{t("projects.detail.history")}</h2>
    <span class="text-[0.733333rem] text-muted-foreground">{events.length}</span>
  </div>
  <div class="grid gap-1">
    {#each events as event (event.id)}
      <div class="rounded-md border border-border bg-background px-2 py-1.5">
        <div class="truncate text-[0.8rem]">{projectTaskHistoryEventLabel(event, t)}</div>
        {#if event.reason}
          <div class="truncate text-[0.733333rem] text-muted-foreground">
            {t("projects.history.reason", event.reason)}
          </div>
        {/if}
        <div class="truncate text-[0.733333rem] text-muted-foreground">{event.occurredAt}</div>
      </div>
    {:else}
      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
        {t("projects.detail.noHistory")}
      </div>
    {/each}
  </div>
</section>
