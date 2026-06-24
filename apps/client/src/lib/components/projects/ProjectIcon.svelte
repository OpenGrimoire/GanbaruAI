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
  import { projectIconAssetUrl } from "$lib/api/project-icons";
  import { getEventColor } from "$lib/components/calendar/utils";
  import {
    parseProjectIcon,
    projectIconColorToEventColor,
  } from "$lib/projects/project-icons";
  import type { ProjectLucideIconNode } from "$lib/projects/project-lucide-catalog.generated";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import LucideNodeIcon from "./LucideNodeIcon.svelte";

  let {
    name,
    size = 14,
    strokeWidth = 1.75,
    ignoreColor = false,
    emojiScale = 1,
    class: className = "",
  }: {
    name?: string;
    size?: number;
    strokeWidth?: number;
    ignoreColor?: boolean;
    emojiScale?: number;
    class?: string;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();

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

  let assetUrl = $state<string | null>(null);
  let lucideNode = $state<readonly ProjectLucideIconNode[] | null>(null);
  let assetRequestId = 0;
  let lucideRequestId = 0;

  const parsedIcon = $derived(parseProjectIcon(name));
  const customEmoji = $derived(
    parsedIcon.kind === "custom-emoji"
      ? projects.customEmojis.find((emoji) => emoji.id === parsedIcon.id)
      : undefined,
  );
  const relativeAssetPath = $derived(
    parsedIcon.kind === "asset"
      ? parsedIcon.relativePath
      : parsedIcon.kind === "custom-emoji"
        ? customEmoji?.assetPath
        : undefined,
  );
  const Icon = $derived(
    parsedIcon.kind === "lucide"
      ? icons[parsedIcon.slug] ?? null
      : null,
  );
  const eventIconColor = $derived(
    parsedIcon.kind === "lucide"
      ? projectIconColorToEventColor(parsedIcon.color)
      : undefined,
  );
  const iconStyle = $derived(
    !ignoreColor && parsedIcon.kind === "lucide" && eventIconColor !== undefined
      ? `color: ${getEventColor(eventIconColor, theme.current).bg};`
      : undefined,
  );
  const emojiFontSize = $derived(Math.max(1, size * emojiScale));

  $effect(() => {
    const currentPath = relativeAssetPath;
    const requestId = ++assetRequestId;
    assetUrl = null;
    if (!currentPath) return;
    void projectIconAssetUrl(currentPath)
      .then((url) => {
        if (requestId === assetRequestId) assetUrl = url;
      })
      .catch(() => {
        if (requestId === assetRequestId) assetUrl = null;
      });
  });

  $effect(() => {
    const currentIcon = parsedIcon;
    const requestId = ++lucideRequestId;
    lucideNode = null;
    if (currentIcon.kind !== "lucide" || icons[currentIcon.slug]) return;
    void import("$lib/projects/project-lucide-catalog.generated")
      .then((catalog) => {
        if (requestId !== lucideRequestId) return;
        lucideNode = catalog.PROJECT_LUCIDE_ICONS.find((icon) => icon.slug === currentIcon.slug)?.iconNode ?? null;
      })
      .catch(() => {
        if (requestId === lucideRequestId) lucideNode = null;
      });
  });
</script>

{#if parsedIcon.kind === "none"}
  <span class={className} style={`display: inline-block; width: ${size}px; height: ${size}px;`} aria-hidden="true"></span>
{:else if parsedIcon.kind === "emoji"}
  <span
    class={className}
    style={`display: inline-flex; width: ${size}px; height: ${size}px; align-items: center; justify-content: center; overflow: hidden; font-size: ${emojiFontSize}px; line-height: 1;`}
    aria-hidden="true"
  >
    {parsedIcon.emoji}
  </span>
{:else if relativeAssetPath && assetUrl}
  <img
    src={assetUrl}
    alt=""
    class={className}
    style={`width: ${size}px; height: ${size}px; object-fit: cover; border-radius: 0.25rem;`}
  />
{:else if Icon}
  <Icon {size} {strokeWidth} class={className} style={iconStyle} />
{:else if lucideNode}
  <LucideNodeIcon iconNode={lucideNode} {size} {strokeWidth} class={className} style={iconStyle} />
{:else}
  <Folder {size} {strokeWidth} class={className} style={iconStyle} />
{/if}
