<script lang="ts">
  import CircleAlert from "@lucide/svelte/icons/circle-alert";

  let {
    heading,
    description,
    detail,
  }: {
    heading: string;
    description: string;
    detail?: string;
  } = $props();
</script>

<div class="flex flex-col gap-4">
  <section class="rounded-xl border border-border bg-card p-4">
    <div class="flex items-start gap-3">
      <CircleAlert class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div class="min-w-0">
        <h2 class="text-sm font-semibold text-foreground">{heading}</h2>
        <p class="mt-1 text-sm leading-6 text-muted-foreground">{description}</p>
        {#if detail}
          <p class="mt-3 text-xs leading-5 text-muted-foreground">{detail}</p>
        {/if}
      </div>
    </div>
  </section>
</div>
