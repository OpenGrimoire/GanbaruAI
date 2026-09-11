<script lang="ts">
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";

  let {
    icon,
    size = 16,
    strokeWidth = 1.75,
  }: {
    icon: string;
    size?: number;
    strokeWidth?: number;
  } = $props();
</script>

<ProjectIcon name={icon} {size} {strokeWidth} />
