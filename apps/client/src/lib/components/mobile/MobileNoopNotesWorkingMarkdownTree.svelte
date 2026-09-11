<script lang="ts">
  import type { NotesWorkingMarkdownTreeProps } from "$lib/components/notes/notes-working-markdown-tree-contract";

  const props: NotesWorkingMarkdownTreeProps = $props();
  $effect(() => {
    Reflect.ownKeys(props);
  });
</script>
