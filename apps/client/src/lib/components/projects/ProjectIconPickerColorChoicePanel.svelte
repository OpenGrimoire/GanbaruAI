<script lang="ts">
  import { EVENT_COLOR_OPTIONS } from "$lib/components/calendar/utils";
  import type { EventColor } from "$lib/components/calendar/types";
  import type { ProjectLucideIconNode } from "$lib/projects/project-lucide-catalog.generated";
  import { serializeProjectIcon } from "$lib/projects/project-icons";
  import { portal } from "$lib/utils/portal";
  import LucideNodeIcon from "./LucideNodeIcon.svelte";
  import ProjectIcon from "./ProjectIcon.svelte";

  let {
    rootElement = $bindable<HTMLElement | undefined>(),
    style,
    label,
    slug,
    iconNode,
    iconColorLabel,
    iconColorStyle,
    onSelect,
  }: {
    rootElement?: HTMLElement;
    style: string;
    label: string;
    slug: string;
    iconNode: readonly ProjectLucideIconNode[] | null;
    iconColorLabel: (color: EventColor) => string;
    iconColorStyle: (color: EventColor) => string;
    onSelect: (color: EventColor) => void;
  } = $props();
</script>

<div
  bind:this={rootElement}
  use:portal
  class="fixed z-100 rounded-xl border border-border bg-popover p-2.5 text-popover-foreground shadow-xl"
  {style}
  role="dialog"
  data-app-floating-surface
  aria-label={label}
>
  <div class="grid gap-2" style="grid-template-columns: repeat(8, 1.375rem);">
    {#each EVENT_COLOR_OPTIONS as color}
      <button
        type="button"
        class="flex size-5.5 items-center justify-center rounded-md hover:bg-accent"
        aria-label={iconColorLabel(color)}
        title={iconColorLabel(color)}
        onclick={(event) => {
          event.stopPropagation();
          onSelect(color);
        }}
      >
        {#if iconNode}
          <LucideNodeIcon
            {iconNode}
            size={16}
            strokeWidth={1.75}
            style={iconColorStyle(color)}
          />
        {:else}
          <ProjectIcon
            name={serializeProjectIcon({ kind: "lucide", slug, color })}
            size={16}
            strokeWidth={1.75}
          />
        {/if}
      </button>
    {/each}
  </div>
</div>
