<script lang="ts">
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Circle from "@lucide/svelte/icons/circle";
  import Plus from "@lucide/svelte/icons/plus";
  import X from "@lucide/svelte/icons/x";
  import { tick } from "svelte";
  import { formatDateTime } from "$lib/i18n/formatters";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { activateModalFocus } from "$lib/modal-focus";
  import { getMobileBackStack } from "$lib/stores/mobile-back-stack.svelte";
  import { projectPriorityDisplayLabel } from "$lib/projects/project-display";
  import type {
    ProjectPriorityConfig,
    ProjectStatus,
    ProjectTask,
  } from "$lib/projects/types";

  let {
    tasks,
    statuses,
    priorities,
    onOpenTask,
    onCreateTask,
    onNeedMore,
  }: {
    tasks: ProjectTask[];
    statuses: ProjectStatus[];
    priorities: ProjectPriorityConfig[];
    onOpenTask: (task: ProjectTask) => void;
    onCreateTask: (title: string) => Promise<ProjectTask | undefined>;
    onNeedMore: () => void;
  } = $props();

  const localization = getLocalization();
  const { t } = localization;
  const mobileBackStack = getMobileBackStack();
  const locale = $derived(localization.locale);
  let createOpen = $state(false);
  let createDialog = $state<HTMLDivElement | null>(null);
  let createInput = $state<HTMLInputElement | null>(null);
  let createTitle = $state("");
  let createPending = $state(false);
  let createError = $state<string | null>(null);

  $effect(() => {
    if (!createOpen) return;
    return mobileBackStack.activate({
      handle: closeCreate,
    });
  });

  $effect(() => {
    if (!createOpen || !createDialog) return;
    return activateModalFocus(createDialog, createInput);
  });

  function openCreate(): void {
    createError = null;
    createOpen = true;
  }

  function closeCreate(): void {
    if (createPending) return;
    createOpen = false;
    createTitle = "";
    createError = null;
  }

  async function submitCreate(): Promise<void> {
    const title = createTitle.trim();
    if (!title || createPending) return;
    createPending = true;
    createError = null;
    try {
      const created = await onCreateTask(title);
      createOpen = false;
      createTitle = "";
      await tick();
      if (created) onOpenTask(created);
    } catch (error) {
      createError = t(
        "projects.tasks.createFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      createPending = false;
    }
  }

  function statusName(task: ProjectTask): string {
    return statuses.find((status) => status.id === task.statusId)?.name
      ?? t("projects.list.status");
  }

  function dueDateLabel(task: ProjectTask): string | null {
    if (!task.dueDate) return null;
    const date = new Date(`${task.dueDate}T12:00:00`);
    if (Number.isNaN(date.getTime())) return task.dueDate;
    return formatDateTime(locale, date, { month: "short", day: "numeric" });
  }
</script>

<div class="relative h-full">
  <div class="h-full overflow-y-auto pb-20">
  {#if tasks.length === 0}
    <div class="flex min-h-48 items-center justify-center p-6 text-sm text-muted-foreground">
      {t("projects.list.noTasks")}
    </div>
  {:else}
    <ul class="divide-y divide-border" aria-label={t("projects.tabs.list")}>
      {#each tasks as task (task.id)}
        {@const dueDate = dueDateLabel(task)}
        <li>
          <button
            type="button"
            onclick={() => onOpenTask(task)}
            class="flex min-h-16 w-full items-center gap-3 px-4 py-2 text-left active:bg-accent"
          >
            <Circle
              size={18}
              strokeWidth={1.8}
              class={task.completedAt ? "fill-primary text-primary" : "text-muted-foreground"}
              aria-hidden="true"
            />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm font-medium">{task.title}</span>
              <span class="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-muted-foreground">
                <span>{statusName(task)}</span>
                <span>{projectPriorityDisplayLabel(task.priority, priorities, t)}</span>
                {#if dueDate}<span>{dueDate}</span>{/if}
              </span>
            </span>
            <ChevronRight size={19} class="shrink-0 text-muted-foreground" aria-hidden="true" />
          </button>
        </li>
      {/each}
    </ul>
    <button
      type="button"
      onclick={onNeedMore}
      class="min-h-12 w-full px-4 text-sm font-medium text-muted-foreground active:bg-accent"
    >
      {t("common.loadMore")}
    </button>
  {/if}

  </div>
  <button
    type="button"
    aria-label={t("projects.header.addTask")}
    onclick={openCreate}
    class="absolute bottom-4 right-4 z-50 flex min-h-14 min-w-14 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-lg active:opacity-85"
  >
    <Plus size={25} strokeWidth={2} aria-hidden="true" />
  </button>
</div>

{#if createOpen}
  <div class="fixed inset-0 z-90 flex items-end" role="presentation">
    <button
      type="button"
      class="absolute inset-0 h-full w-full bg-black/45"
      aria-label={t("common.cancel")}
      onclick={closeCreate}
    ></button>
    <div
      bind:this={createDialog}
      role="dialog"
      aria-modal="true"
      aria-label={t("projects.header.addTask")}
      class="relative z-10 w-full rounded-t-3xl border border-b-0 border-border bg-card px-4 pb-[calc(var(--safe-area-bottom)+var(--keyboard-inset)+1rem)] pt-2 shadow-2xl"
      style="padding-left: calc(var(--safe-area-left) + 1rem); padding-right: calc(var(--safe-area-right) + 1rem);"
      tabindex="-1"
      onkeydown={(event) => {
        if (event.key !== "Escape") return;
        event.preventDefault();
        event.stopPropagation();
        closeCreate();
      }}
    >
      <div class="mx-auto mb-2 h-1 w-10 rounded-full bg-muted-foreground/35" aria-hidden="true"></div>
      <div class="flex min-h-12 items-center gap-2">
        <h2 class="min-w-0 flex-1 text-base font-semibold">{t("projects.header.addTask")}</h2>
        <button
          type="button"
          class="flex min-h-12 min-w-12 items-center justify-center rounded-xl active:bg-accent"
          aria-label={t("common.cancel")}
          onclick={closeCreate}
        >
          <X size={21} aria-hidden="true" />
        </button>
      </div>
      <form class="flex flex-col gap-3 pb-1" onsubmit={(event) => {
        event.preventDefault();
        void submitCreate();
      }}>
        <input
          bind:this={createInput}
          bind:value={createTitle}
          disabled={createPending}
          class="min-h-12 rounded-xl border border-border bg-background px-3 text-base outline-none focus:border-primary"
          placeholder={t("projects.header.quickAddPlaceholder")}
          aria-label={t("projects.header.addTask")}
        />
        {#if createError}
          <p class="text-sm text-destructive" role="alert">{createError}</p>
        {/if}
        <button
          type="submit"
          disabled={!createTitle.trim() || createPending}
          class="min-h-12 rounded-xl bg-primary px-4 text-sm font-medium text-primary-foreground disabled:opacity-45"
        >
          {t("projects.header.addTask")}
        </button>
      </form>
    </div>
  </div>
{/if}
