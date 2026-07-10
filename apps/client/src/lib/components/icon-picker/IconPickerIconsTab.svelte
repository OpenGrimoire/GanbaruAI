<script lang="ts">
  import CircleEllipsis from "@lucide/svelte/icons/circle-ellipsis";
  import LayoutGrid from "@lucide/svelte/icons/layout-grid";
  import Search from "@lucide/svelte/icons/search";
  import Shapes from "@lucide/svelte/icons/shapes";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import type { EventColor } from "$lib/components/calendar/types";
  import { EVENT_COLOR_OPTIONS } from "$lib/components/calendar/utils";
  import type {
    ProjectIconPickerGroupVirtualWindow,
    ProjectIconPickerLucideCategoryOption,
  } from "$lib/projects/project-icon-picker";
  import type {
    ProjectLucideCategory,
    ProjectLucideIconEntry,
    ProjectLucideIconNode,
  } from "$lib/projects/project-lucide-catalog.generated";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { cn } from "$lib/utils";
  import LucideNodeIcon from "$lib/components/projects/LucideNodeIcon.svelte";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";

  let {
    scrollElement = $bindable<HTMLElement | undefined>(),
    iconCategoryMenuTriggerElement = $bindable<HTMLButtonElement | undefined>(),
    iconQuery = $bindable(""),
    iconCategory,
    iconCategoryMenuOpen,
    iconCategoryOverflowActive,
    iconColor = $bindable<EventColor>(),
    iconColorPanelOpen = $bindable(false),
    skinTonePanelOpen = $bindable(false),
    askIconColorEveryTime = $bindable(true),
    allowIconColors,
    colorSelectionBorder,
    gridScrollable,
    gridCanScrollUp,
    gridCanScrollDown,
    gridColumnCount,
    lucideRecentValues,
    lucideGroupVirtual,
    lucideLoading,
    primaryLucideCategoryOptions,
    iconColorLabel,
    iconColorSwatch,
    iconColorStyle,
    lucideRecentPreviewValue,
    onScroll,
    onChooseRandom,
    onChooseRecent,
    onChooseLucideIcon,
    onSelectIconColor,
    onSelectIconCategory,
    onToggleIconCategoryMenu,
    onClearIconColorChoice,
  }: {
    scrollElement?: HTMLElement;
    iconCategoryMenuTriggerElement?: HTMLButtonElement;
    iconQuery: string;
    iconCategory: ProjectLucideCategory | "all";
    iconCategoryMenuOpen: boolean;
    iconCategoryOverflowActive: boolean;
    iconColor: EventColor;
    iconColorPanelOpen: boolean;
    skinTonePanelOpen: boolean;
    askIconColorEveryTime: boolean;
    allowIconColors: boolean;
    colorSelectionBorder: string;
    gridScrollable: boolean;
    gridCanScrollUp: boolean;
    gridCanScrollDown: boolean;
    gridColumnCount: number;
    lucideRecentValues: readonly string[];
    lucideGroupVirtual: ProjectIconPickerGroupVirtualWindow<ProjectLucideCategory, ProjectLucideIconEntry>;
    lucideLoading: boolean;
    primaryLucideCategoryOptions: readonly ProjectIconPickerLucideCategoryOption[];
    iconColorLabel: (color: EventColor) => string;
    iconColorSwatch: (color: EventColor) => string;
    iconColorStyle: (color: EventColor) => string;
    lucideRecentPreviewValue: (rawValue: string) => string;
    onScroll: () => void;
    onChooseRandom: () => void;
    onChooseRecent: (rawValue: string, target: EventTarget | null) => void;
    onChooseLucideIcon: (
      slug: string,
      label: string,
      iconNode: readonly ProjectLucideIconNode[] | null,
      target: EventTarget | null,
    ) => void;
    onSelectIconColor: (color: EventColor) => void;
    onSelectIconCategory: (category: ProjectLucideCategory | "all") => void;
    onToggleIconCategoryMenu: () => void;
    onClearIconColorChoice: () => void;
  } = $props();

  const { t } = getLocalization();
</script>

<div class="flex shrink-0 items-center gap-2 px-3 pt-3">
  <div class="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-background px-2">
    <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
    <input
      bind:value={iconQuery}
      class="h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] outline-none placeholder:text-muted-foreground"
      placeholder={t("projects.iconPicker.filter")}
    />
  </div>
  <button
    type="button"
    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-accent hover:text-foreground"
    aria-label={t("projects.iconPicker.random")}
    onclick={onChooseRandom}
  >
    <Shuffle size={14} strokeWidth={1.75} />
  </button>
  {#if allowIconColors}
    <div class="relative shrink-0" data-icon-picker-inline-panel>
      <button
        type="button"
        class={cn(
          "flex h-8 w-8 items-center justify-center rounded-md border border-border hover:bg-accent",
          iconColorPanelOpen && "bg-accent text-foreground",
        )}
        aria-label={t("projects.iconPicker.iconColor")}
        title={iconColorLabel(iconColor)}
        onclick={(event) => {
          event.stopPropagation();
          iconColorPanelOpen = !iconColorPanelOpen;
          skinTonePanelOpen = false;
        }}
      >
        <span
          class="h-4 w-4 rounded-full border border-border"
          style={`background: ${iconColorSwatch(iconColor)};`}
        ></span>
      </button>
      {#if iconColorPanelOpen}
        <div
          class="absolute right-0 top-9 z-10 w-40 rounded-lg border border-border bg-popover px-2.5 py-2 shadow-lg"
          style={`--project-icon-color-selection-border: ${colorSelectionBorder};`}
        >
          <div class="grid justify-center gap-2" style="grid-template-columns: repeat(4, 1.375rem);">
            {#each EVENT_COLOR_OPTIONS as color}
              <button
                type="button"
                class={cn(
                  "project-icon-color-swatch relative size-5.5 rounded-full",
                  iconColor === color && "swatch-selected",
                )}
                style={`background-color: ${iconColorSwatch(color)};`}
                aria-label={iconColorLabel(color)}
                title={iconColorLabel(color)}
                onclick={(event) => {
                  event.stopPropagation();
                  onSelectIconColor(color);
                  iconColorPanelOpen = false;
                }}
              ></button>
            {/each}
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={askIconColorEveryTime}
            class="mt-1 flex h-8 w-full items-center justify-between rounded-md px-1.5 text-left text-[0.8rem] text-foreground hover:bg-accent"
            onclick={(event) => {
              event.stopPropagation();
              askIconColorEveryTime = !askIconColorEveryTime;
              if (!askIconColorEveryTime) onClearIconColorChoice();
            }}
          >
            <span>{t("projects.iconPicker.askEveryTime")}</span>
            <span
              class={cn(
                "flex h-4 w-7 shrink-0 items-center rounded-full p-0.5",
                askIconColorEveryTime ? "justify-end bg-primary" : "justify-start bg-muted-foreground/30",
              )}
            >
              <span class="h-3 w-3 rounded-full bg-background shadow-sm"></span>
            </span>
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<div
  bind:this={scrollElement}
  class={cn(
    "project-icon-picker-scroll-area min-h-0 flex-1 overflow-y-auto px-3 py-3",
    gridScrollable
      && gridCanScrollUp
      && gridCanScrollDown
      && "project-icon-picker-scroll-both",
    gridScrollable
      && gridCanScrollUp
      && !gridCanScrollDown
      && "project-icon-picker-scroll-top",
    gridScrollable
      && !gridCanScrollUp
      && gridCanScrollDown
      && "project-icon-picker-scroll-bottom",
  )}
  onscroll={onScroll}
>
  {#if lucideRecentValues.length > 0}
    <section class="mb-3">
      <div class="mb-1 flex h-5 items-center gap-2 text-[0.733333rem] text-muted-foreground">
        <span>{t("projects.iconPicker.recent")}</span>
        <span class="h-px flex-1 bg-border/70"></span>
      </div>
      <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
        {#each lucideRecentValues as recentValue}
          <button
            type="button"
            class={cn(
              "flex h-9 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
              allowIconColors ? "text-muted-foreground" : "text-foreground",
            )}
            aria-label={t("projects.iconPicker.selectRecent")}
            onclick={(event) => onChooseRecent(recentValue, event.currentTarget)}
          >
            <ProjectIcon name={lucideRecentPreviewValue(recentValue)} size={18} ignoreColor={!allowIconColors} />
          </button>
        {/each}
      </div>
    </section>
  {/if}

  {#if lucideLoading}
    <div class="py-6 text-center text-[0.8rem] text-muted-foreground">{t("common.loading")}</div>
  {:else}
    <div style={`height: ${lucideGroupVirtual.beforeHeight}px;`} aria-hidden="true"></div>
    {#each lucideGroupVirtual.groups as group (group.category)}
      <section class="mb-3">
        <div class="mb-1 flex h-5 items-center gap-2 text-[0.733333rem] text-muted-foreground">
          <span>{group.category}</span>
          <span class="h-px flex-1 bg-border/70"></span>
        </div>
        <div style={`height: ${group.beforeRowsHeight}px;`} aria-hidden="true"></div>
        <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
          {#each group.entries as entry (entry.slug)}
            <button
              type="button"
              class={cn(
                "flex h-9 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
                allowIconColors ? "text-muted-foreground" : "text-foreground",
              )}
              title={entry.label}
              onclick={(event) => onChooseLucideIcon(entry.slug, entry.label, entry.iconNode, event.currentTarget)}
            >
              <LucideNodeIcon
                iconNode={entry.iconNode}
                size={18}
                strokeWidth={1.75}
                style={allowIconColors ? iconColorStyle(iconColor) : undefined}
              />
            </button>
          {/each}
        </div>
        <div style={`height: ${group.afterRowsHeight}px;`} aria-hidden="true"></div>
      </section>
    {/each}
    <div style={`height: ${lucideGroupVirtual.afterHeight}px;`} aria-hidden="true"></div>
  {/if}
</div>

<div class="flex shrink-0 items-center gap-1 border-t border-border/70 px-3 py-2">
  <button
    type="button"
    class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", iconCategory === "all" ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
    aria-label={t("projects.iconPicker.all")}
    title={t("projects.iconPicker.all")}
    onclick={() => onSelectIconCategory("all")}
  >
    <LayoutGrid size={16} strokeWidth={1.75} />
  </button>
  {#each primaryLucideCategoryOptions as option (option.category)}
    <button
      type="button"
      class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", iconCategory === option.category ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
      aria-label={option.category}
      title={option.category}
      onclick={() => onSelectIconCategory(option.category)}
    >
      {#if option.icon}
        <LucideNodeIcon iconNode={option.icon.iconNode} size={16} strokeWidth={1.75} />
      {:else}
        <Shapes size={16} strokeWidth={1.75} />
      {/if}
    </button>
  {/each}
  <div class="h-5 w-px shrink-0 bg-border/80"></div>
  <button
    bind:this={iconCategoryMenuTriggerElement}
    type="button"
    class={cn(
      "flex h-8 w-8 shrink-0 items-center justify-center rounded-md",
      iconCategoryMenuOpen || iconCategoryOverflowActive
        ? "bg-accent text-foreground"
        : "text-muted-foreground hover:bg-accent hover:text-foreground",
    )}
    aria-label={t("projects.iconPicker.moreCategories")}
    title={t("projects.iconPicker.moreCategories")}
    onclick={onToggleIconCategoryMenu}
  >
    <CircleEllipsis size={16} strokeWidth={1.75} />
  </button>
</div>

<style>
  .project-icon-picker-scroll-area {
    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-icon-picker-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black 20px, black);
    mask-image: linear-gradient(to bottom, transparent, black 20px, black);
  }

  .project-icon-picker-scroll-bottom {
    -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - 20px), transparent);
    mask-image: linear-gradient(to bottom, black, black calc(100% - 20px), transparent);
  }

  .project-icon-picker-scroll-both {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black 20px, black calc(100% - 20px), transparent);
    mask-image: linear-gradient(to bottom, transparent, black 20px, black calc(100% - 20px), transparent);
  }

  .project-icon-color-swatch {
    overflow: hidden;
  }

  .project-icon-color-swatch.swatch-selected::after {
    content: "";
    position: absolute;
    inset: 0;
    border: 2px solid var(--project-icon-color-selection-border);
    border-radius: inherit;
    pointer-events: none;
  }
</style>
