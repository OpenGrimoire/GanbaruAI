<script lang="ts">
  type SoundscapeFilter = "all" | "generated" | "local";

  let {
    filter = "all",
    compact = false,
    addRequest = 0,
    onPlaybackStart = () => undefined,
  }: {
    filter?: SoundscapeFilter;
    compact?: boolean;
    addRequest?: number;
    onPlaybackStart?: () => void;
  } = $props();
</script>
