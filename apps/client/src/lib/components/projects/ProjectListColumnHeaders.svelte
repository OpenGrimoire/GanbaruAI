<script lang="ts">
  import CirclePlus from "@lucide/svelte/icons/circle-plus";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type {
    ProjectTaskListColumn,
  } from "$lib/projects/types";
  import type { ProjectTaskListResizableColumn } from "$lib/projects/project-list-view";
  import { cn } from "$lib/utils";

  type HeaderMode = "section" | "group";

  let {
    mode,
    gridTemplate,
    gridMinWidth,
    taskListColumns,
    taskListColumnLabel,
    onResizePointerDown,
    onResizeDoubleClick,
    onResizeKeydown,
  }: {
    mode: HeaderMode;
    gridTemplate: string;
    gridMinWidth: string;
    taskListColumns: ProjectTaskListColumn[];
    taskListColumnLabel: (column: ProjectTaskListColumn) => string;
    onResizePointerDown: (event: PointerEvent, column: ProjectTaskListResizableColumn) => void;
    onResizeDoubleClick: (event: MouseEvent, column: ProjectTaskListResizableColumn) => void;
    onResizeKeydown: (event: KeyboardEvent, column: ProjectTaskListResizableColumn) => void;
  } = $props();

  const { t } = getLocalization();
</script>

{#snippet columnHeaderCell(label: string, column: ProjectTaskListResizableColumn)}
  <div class="project-list-header-cell relative flex min-h-11 min-w-0 items-center self-stretch rounded-md px-2 py-1 hover:bg-accent/20">
    <span class="relative z-10 truncate">{label}</span>
    <button
      type="button"
      class="project-list-column-resize-hit"
      aria-label={t("projects.columns.resizeColumn", label)}
      data-app-tooltip-disabled="true"
      onpointerdown={(event) => onResizePointerDown(event, column)}
      ondblclick={(event) => onResizeDoubleClick(event, column)}
      onkeydown={(event) => onResizeKeydown(event, column)}
    ></button>
  </div>
{/snippet}

<div
  class={cn(
    "project-list-divider grid min-h-11 items-center px-1 text-[0.866667rem] font-semibold text-foreground",
    mode === "section" ? "group/column-header" : "group/list-column-header",
  )}
  style={`grid-template-columns: ${gridTemplate}; min-width: ${gridMinWidth};`}
>
  <div></div>
  <div></div>
  {@render columnHeaderCell(t("projects.list.name"), "name")}
  {#each taskListColumns as column (column)}
    {@render columnHeaderCell(taskListColumnLabel(column), column)}
  {/each}
  <div
    class="flex min-h-11 min-w-0 items-center justify-center self-stretch rounded-md text-muted-foreground"
    aria-hidden="true"
  >
    <CirclePlus size={15} strokeWidth={1.75} />
  </div>
</div>

<style>
  .project-list-header-cell::before {
    position: absolute;
    inset: 0;
    z-index: 1;
    border: 1px solid transparent;
    border-radius: 0.375rem;
    content: "";
    pointer-events: none;
  }

  .project-list-header-cell:hover::before {
    border-color: color-mix(in srgb, var(--foreground) 25%, transparent);
  }

  .project-list-column-resize-hit {
    position: absolute;
    top: 0.375rem;
    right: -0.375rem;
    bottom: 0.375rem;
    z-index: 20;
    width: 0.75rem;
    border: 0;
    background: transparent;
    cursor: col-resize;
    padding: 0;
  }

  .project-list-column-resize-hit::after {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 0.25rem;
    transform: translateX(-50%);
    border-radius: 9999px;
    background: var(--primary);
    content: "";
    opacity: 0;
  }

  .project-list-column-resize-hit:hover::after,
  .project-list-column-resize-hit:focus-visible::after,
  .project-list-column-resize-hit:active::after {
    opacity: 1;
  }
</style>
