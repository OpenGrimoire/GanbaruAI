<script lang="ts">
  import type { Theme } from "$lib/stores/themes";

  let { theme }: { theme: Theme } = $props();
</script>

<div data-theme-editor-stub>{theme.displayName}</div>
