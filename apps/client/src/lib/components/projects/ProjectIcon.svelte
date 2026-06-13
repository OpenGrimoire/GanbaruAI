<script lang="ts">
  import type { Component } from "svelte";
  import Apple from "@lucide/svelte/icons/apple";
  import Bath from "@lucide/svelte/icons/bath";
  import Bed from "@lucide/svelte/icons/bed";
  import BookOpen from "@lucide/svelte/icons/book-open";
  import Clapperboard from "@lucide/svelte/icons/clapperboard";
  import Dumbbell from "@lucide/svelte/icons/dumbbell";
  import Folder from "@lucide/svelte/icons/folder";
  import GraduationCap from "@lucide/svelte/icons/graduation-cap";
  import Heart from "@lucide/svelte/icons/heart";
  import Repeat from "@lucide/svelte/icons/repeat";
  import Smile from "@lucide/svelte/icons/smile";
  import Sparkles from "@lucide/svelte/icons/sparkles";

  let {
    name,
    size = 14,
    strokeWidth = 1.75,
    class: className = "",
  }: {
    name?: string;
    size?: number;
    strokeWidth?: number;
    class?: string;
  } = $props();

  const icons: Record<string, Component> = {
    apple: Apple,
    bath: Bath,
    bed: Bed,
    "book-open": BookOpen,
    clapperboard: Clapperboard,
    dumbbell: Dumbbell,
    folder: Folder,
    "graduation-cap": GraduationCap,
    heart: Heart,
    repeat: Repeat,
    smile: Smile,
    sparkles: Sparkles,
  };

  const Icon = $derived(icons[name ?? ""] ?? Folder);
</script>

<Icon {size} {strokeWidth} class={className} />
