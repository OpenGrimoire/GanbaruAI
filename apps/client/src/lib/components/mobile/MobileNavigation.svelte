<script lang="ts">
  import BookOpen from "@lucide/svelte/icons/book-open";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import ListTodo from "@lucide/svelte/icons/list-todo";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { View } from "$lib/navigation";
  import { cn } from "$lib/utils";

  let {
    current,
    presentation,
    onNavigate,
  }: {
    current: View;
    presentation: "top" | "rail";
    onNavigate: (view: View) => void;
  } = $props();

  const { t } = getLocalization();
  const destinations = [
    { view: "calendar", icon: CalendarDays },
    { view: "projects", icon: ListTodo },
    { view: "notes", icon: BookOpen },
    { view: "chat", icon: MessageSquare },
  ] as const;
</script>

<nav
  aria-label={t("mobile.primaryNavigation")}
  class={cn(
    "mobile-primary-navigation border-sidebar-border bg-sidebar text-sidebar-foreground",
    presentation === "rail"
      ? "flex w-(--mobile-nav-rail-w) shrink-0 flex-col border-r px-2 py-3"
      : "col-span-4 grid min-w-0 self-stretch grid-cols-4",
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
        "relative flex min-h-12 items-center justify-center rounded-xl text-xs font-medium transition-colors",
        presentation === "rail"
          ? "mb-2 flex-col gap-1"
          : "min-w-0",
        selected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/70 active:bg-sidebar-accent/70",
      )}
    >
      <Icon size={21} strokeWidth={selected ? 2 : 1.7} aria-hidden="true" />
      {#if presentation === "rail"}
        <span>{t(`titleBar.tab.${destination.view}`)}</span>
      {/if}
    </button>
  {/each}
</nav>
