<script lang="ts">
  const props: { projectId: string } = $props();

  $effect(() => {
    Reflect.ownKeys(props);
  });
</script>
