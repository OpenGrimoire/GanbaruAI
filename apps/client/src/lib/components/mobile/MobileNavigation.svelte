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
    "mobile-primary-navigation",
    presentation === "rail"
      ? "flex w-(--mobile-nav-rail-w) shrink-0 flex-col border-r border-sidebar-border bg-sidebar px-2 py-3 text-sidebar-foreground"
      : "col-span-4 grid min-w-0 self-stretch grid-cols-4 px-1 text-foreground",
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
        "group relative flex items-center justify-center text-xs font-medium transition-colors",
        presentation === "rail"
          ? "mb-2 min-h-12 flex-col gap-1 rounded-xl"
          : "min-h-0 min-w-0",
        presentation === "rail"
          ? selected
            ? "bg-sidebar-accent text-sidebar-accent-foreground"
            : "text-sidebar-foreground/70 active:bg-sidebar-accent/70"
          : selected
            ? "text-foreground"
            : "text-muted-foreground",
      )}
    >
      <span class={cn(
        "grid place-items-center transition-colors",
        presentation === "top" && "h-8 w-8 rounded-full group-active:bg-accent/70",
      )}>
        <Icon size={presentation === "top" ? 19 : 21} strokeWidth={selected ? 2 : 1.7} aria-hidden="true" />
      </span>
      {#if presentation === "top" && selected}
        <span data-mobile-navigation-indicator class="absolute bottom-0 h-0.5 w-5 rounded-full bg-foreground/75" aria-hidden="true"></span>
      {/if}
      {#if presentation === "rail"}
        <span>{t(`titleBar.tab.${destination.view}`)}</span>
      {/if}
    </button>
  {/each}
</nav>
