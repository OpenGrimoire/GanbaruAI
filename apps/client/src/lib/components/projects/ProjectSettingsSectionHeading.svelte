<script lang="ts">
  let {
    label,
    count = null,
  }: {
    label: string;
    count?: string | null;
  } = $props();
</script>

<div class="flex min-h-7 items-center justify-between gap-3 px-1">
  <h2 class="truncate text-[0.866667rem] font-semibold text-foreground">{label}</h2>
  {#if count}
    <span class="shrink-0 text-[0.733333rem] font-medium text-muted-foreground">{count}</span>
  {/if}
</div>
