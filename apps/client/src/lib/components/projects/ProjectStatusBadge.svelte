<script lang="ts">
  import {
    projectStatusBadgeClass,
    projectStatusBadgeDotStyle,
    projectStatusBadgeStyle,
  } from "$lib/projects/project-display";
  import type { ProjectStatus } from "$lib/projects/types";
  import type { Theme } from "$lib/stores/themes";
  import { cn } from "$lib/utils";

  let {
    status,
    theme,
    label,
    class: className = "",
    textClass = "",
  }: {
    status: ProjectStatus | undefined;
    theme: Theme;
    label: string;
    class?: string;
    textClass?: string;
  } = $props();
</script>

<span
  class={cn(
    "inline-flex max-w-full min-w-0 items-center gap-1.5 rounded-full px-2 py-0.5 font-medium",
    projectStatusBadgeClass(status),
    className,
  )}
  style={projectStatusBadgeStyle(status, theme)}
>
  {#if status}
    <span
      class="size-2 shrink-0 rounded-full"
      style={projectStatusBadgeDotStyle(status, theme)}
      aria-hidden="true"
    ></span>
  {/if}
  <span class={cn("min-w-0 truncate", textClass)}>{label}</span>
</span>
