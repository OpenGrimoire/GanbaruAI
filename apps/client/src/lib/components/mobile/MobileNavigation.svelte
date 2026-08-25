<script lang="ts">
  import BookOpen from "@lucide/svelte/icons/book-open";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import ListTodo from "@lucide/svelte/icons/list-todo";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { View } from "$lib/navigation";
  import { getZoom } from "$lib/stores/zoom.svelte";
  import { mobileNavigationShowsLabels } from "$lib/mobile-layout";
  import { cn } from "$lib/utils";

  let {
    current,
    presentation,
    onNavigate,
  }: {
    current: View;
    presentation: "bottom" | "rail";
    onNavigate: (view: View) => void;
  } = $props();

  const { t } = getLocalization();
  const zoom = getZoom();
  const showLabels = $derived(mobileNavigationShowsLabels(presentation, zoom.level));
  const destinations = [
    { view: "calendar", icon: CalendarDays },
    { view: "projects", icon: ListTodo },
    { view: "notes", icon: BookOpen },
  ] as const;
</script>

<nav
  aria-label={t("mobile.primaryNavigation")}
  class={cn(
    "mobile-primary-navigation shrink-0 border-sidebar-border bg-sidebar text-sidebar-foreground",
    presentation === "rail"
      ? "flex w-(--mobile-nav-rail-w) flex-col border-r px-2 py-3"
      : "grid h-(--mobile-nav-h) grid-cols-3 border-t px-2",
  )}
>
  {#each destinations as destination}
    {@const Icon = destination.icon}
    {@const selected = current === destination.view}
    <button
      type="button"
      aria-current={selected ? "page" : undefined}
      aria-label={t(`titleBar.tab.${destination.view}`)}
      onclick={() => onNavigate(destination.view)}
      class={cn(
        "relative flex min-h-12 min-w-12 items-center justify-center rounded-xl text-xs font-medium transition-colors",
        presentation === "rail"
          ? "mb-2 flex-col gap-1"
          : showLabels
            ? "flex-col gap-0.5"
            : "",
        selected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/70 active:bg-sidebar-accent/70",
      )}
    >
      <Icon size={21} strokeWidth={selected ? 2 : 1.7} aria-hidden="true" />
      {#if showLabels}
        <span>{t(`titleBar.tab.${destination.view}`)}</span>
      {/if}
    </button>
  {/each}
</nav>
