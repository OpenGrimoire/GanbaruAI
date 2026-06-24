<script lang="ts">
  import { onMount, tick } from "svelte";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import FolderX from "@lucide/svelte/icons/folder-x";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { cn } from "$lib/utils";
  import { portal } from "$lib/utils/portal";
  import ProjectIcon from "./ProjectIcon.svelte";
  import { PROJECT_TEMPLATE_IDS } from "$lib/projects/types";
  import type { Project, ProjectGroup, ProjectTemplateId } from "$lib/projects/types";

  let {
    selectedProjectId = undefined,
    disabled = false,
    onSelect,
  }: {
    selectedProjectId?: string;
    disabled?: boolean;
    onSelect: (projectId: string | undefined) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();
  const eventPanelProjectIconSize = 18;
  const eventPanelProjectEmojiScale = 0.82;
  const eventPanelSelectedProjectIconStrokeWidth = 1.5;
  const eventPanelIconStrokeWidth = 1.6;
  const projectSelectorPanelMinWidth = 202;
  const projectSelectorPanelMaxWidth = 259;
  const projectSelectorFallbackHeaderHeight = 40;
  const projectSelectorFallbackFooterHeight = 44;
  const projectSelectorVisibleGroupRows = 6;
  const projectSelectorRowHeightRem = 2;
  const projectSelectorFallbackRowHeight = 30;
  const projectSelectorListPadding = 6;
  const projectSubpanelVisibleProjectRows = 6;
  const projectSubpanelRowHeight = 32;
  const projectSubpanelListPadding = 8;
  const projectSubpanelFallbackFooterHeight = 44;
  const projectSubpanelGap = 4;
  const projectSelectorPanelRightNudge = 4;
  let open = $state(false);
  let search = $state("");
  let groupDraft = $state("");
  let projectDraftByGroup = $state<Record<string, string>>({});
  let projectTemplateDraftByGroup = $state<Record<string, ProjectTemplateId>>({});
  let createGroupOpen = $state(false);
  let createProjectGroupId = $state<string | null>(null);
  let rootEl: HTMLDivElement | undefined = $state();
  let triggerEl: HTMLButtonElement | undefined = $state();
  let dropdownHeaderElement: HTMLDivElement | undefined = $state();
  let dropdownFooterElement: HTMLDivElement | undefined = $state();
  let dropdownStyle = $state("");
  let activeGroupId = $state<string | null>(null);
  let activeGroupAnchorEl = $state<HTMLElement | null>(null);
  let projectSubpanelEl: HTMLDivElement | undefined = $state();
  let projectSubpanelBridgeEl: HTMLDivElement | undefined = $state();
  let projectSubpanelFooterElement: HTMLDivElement | undefined = $state();
  let projectSubpanelStyle = $state("");
  let projectSubpanelBridgeStyle = $state("");
  let groupScrollElement = $state<HTMLElement | undefined>();
  let groupScrollContentElement = $state<HTMLElement | undefined>();
  let projectScrollElement = $state<HTMLElement | undefined>();
  let projectScrollContentElement = $state<HTMLElement | undefined>();
  let groupScrollable = $state(false);
  let groupCanScrollUp = $state(false);
  let groupCanScrollDown = $state(false);
  let projectScrollable = $state(false);
  let projectCanScrollUp = $state(false);
  let projectCanScrollDown = $state(false);
  let groupScrollStateFrame: number | null = null;
  let projectScrollStateFrame: number | null = null;

  const selectedProject = $derived(projects.projectById(selectedProjectId));
  const selectedGroup = $derived(projects.groupById(selectedProject?.groupId));
  const normalizedSearch = $derived(search.trim().toLowerCase());
  const searchActive = $derived(normalizedSearch.length > 0);
  const groups = $derived.by(() => projects.visibleGroups());
  const visibleGroups = $derived.by(() => groups.filter(groupVisible));
  const searchResultGroups = $derived.by((): ProjectSearchResultGroup[] =>
    groups
      .map((group) => ({ group, projects: projectsInGroup(group) }))
      .filter((entry) => entry.projects.length > 0),
  );
  const activeGroup = $derived.by(() => visibleGroups.find((group) => group.id === activeGroupId));

  interface ProjectSearchResultGroup {
    group: ProjectGroup;
    projects: Project[];
  }

  onMount(() => {
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });

    const updateWhenOpen = () => {
      if (!open) return;
      updateDropdownGeometry();
      updateProjectSubpanelGeometry();
    };
    window.addEventListener("resize", updateWhenOpen);
    window.addEventListener("scroll", updateWhenOpen, true);
    return () => {
      window.removeEventListener("resize", updateWhenOpen);
      window.removeEventListener("scroll", updateWhenOpen, true);
    };
  });

  interface DropdownBounds {
    left: number;
    right: number;
    top: number;
    bottom: number;
  }

  function dropdownBounds(): DropdownBounds {
    const margin = 8;
    const panelRect = triggerEl?.closest(".event-panel-scroll")?.getBoundingClientRect();
    const viewportBounds = {
      left: margin,
      right: window.innerWidth - margin,
      top: margin,
      bottom: window.innerHeight - margin,
    };
    if (!panelRect) return viewportBounds;
    return {
      left: Math.max(panelRect.left, viewportBounds.left),
      right: Math.min(panelRect.right, viewportBounds.right),
      top: Math.max(panelRect.top, viewportBounds.top),
      bottom: Math.min(panelRect.bottom, viewportBounds.bottom),
    };
  }

  function dropdownAnchorRect(): DOMRect {
    const colorPickerEl = rootEl?.nextElementSibling;
    if (colorPickerEl instanceof HTMLElement) return colorPickerEl.getBoundingClientRect();
    return triggerEl?.getBoundingClientRect() ?? new DOMRect();
  }

  function updateDropdownGeometry(): void {
    if (!triggerEl) return;
    const anchorRect = dropdownAnchorRect();
    const bounds = dropdownBounds();
    const edge = 8;
    const gap = 4;
    const boundsWidth = Math.max(0, bounds.right - bounds.left);
    const width = Math.min(
      projectSelectorPanelMaxWidth,
      Math.max(projectSelectorPanelMinWidth, boundsWidth),
    );
    const left = Math.min(
      Math.max(anchorRect.right + projectSelectorPanelRightNudge - width, bounds.left),
      bounds.right - width,
    );
    const headerHeight = dropdownHeaderElement?.offsetHeight ?? projectSelectorFallbackHeaderHeight;
    const footerHeight = searchActive
      ? 0
      : dropdownFooterElement?.offsetHeight ?? projectSelectorFallbackFooterHeight;
    const preferredMaxHeight =
      headerHeight
      + footerHeight
      + projectSelectorListPadding
      + projectSelectorRowHeight() * projectSelectorVisibleGroupRows;
    const maxHeight = Math.min(preferredMaxHeight, Math.max(0, window.innerHeight - edge * 2));
    const measuredListHeight = groupScrollContentElement
      ? groupScrollContentElement.scrollHeight + scrollAreaVerticalPadding(
        groupScrollElement,
        projectSelectorListPadding,
      )
      : undefined;
    const listHeight = measuredListHeight ?? dropdownEstimatedListHeight();
    const naturalHeight = groupScrollContentElement
      ? headerHeight + listHeight + footerHeight
      : headerHeight + dropdownEstimatedListHeight() + footerHeight;
    const height = Math.min(maxHeight, Math.ceil(naturalHeight));
    const spaceBelow = window.innerHeight - edge - anchorRect.bottom - gap;
    const spaceAbove = anchorRect.top - edge - gap;
    const openAbove = spaceBelow < height && spaceAbove > spaceBelow;
    const preferredTop = openAbove
      ? anchorRect.top - gap - height
      : anchorRect.bottom + gap;
    const top = Math.min(
      Math.max(edge, preferredTop),
      Math.max(edge, window.innerHeight - edge - height),
    );
    dropdownStyle = [
      `left: ${Math.round(left)}px`,
      `top: ${Math.round(top)}px`,
      `width: ${Math.round(width)}px`,
      `height: ${Math.round(height)}px`,
      `max-height: ${Math.round(height)}px`,
    ].join("; ");
  }

  function dropdownEstimatedListHeight(): number {
    const visibleItemCount = searchActive
      ? searchResultGroups.reduce((count, entry) => count + entry.projects.length, 0)
      : visibleGroups.length;
    const visibleRows = Math.min(visibleItemCount, projectSelectorVisibleGroupRows);
    return projectSelectorListPadding + projectSelectorRowHeight() * visibleRows;
  }

  function projectSelectorRowHeight(): number {
    const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    return Number.isFinite(rootFontSize)
      ? rootFontSize * projectSelectorRowHeightRem
      : projectSelectorFallbackRowHeight;
  }

  function scrollAreaVerticalPadding(element: HTMLElement | undefined, fallback: number): number {
    if (!element) return fallback;
    const style = getComputedStyle(element);
    return cssPixelValue(style.paddingTop) + cssPixelValue(style.paddingBottom);
  }

  function cssPixelValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function updateProjectSubpanelGeometry(): void {
    if (!activeGroupAnchorEl) return;
    const anchorRect = activeGroupAnchorEl.getBoundingClientRect();
    const edge = 8;
    const gap = projectSubpanelGap;
    const usableWidth = Math.max(0, window.innerWidth - edge * 2);
    const width = Math.min(
      projectSelectorPanelMaxWidth,
      usableWidth,
      Math.max(projectSelectorPanelMinWidth, anchorRect.width),
    );
    const spaceRight = window.innerWidth - edge - anchorRect.right - gap;
    const spaceLeft = anchorRect.left - edge - gap;
    const openRight = spaceRight >= width || spaceRight >= spaceLeft;
    const left = openRight
      ? Math.min(anchorRect.right + gap, window.innerWidth - edge - width)
      : Math.max(edge, anchorRect.left - gap - width);
    const footerHeight = projectSubpanelFooterElement?.offsetHeight ?? projectSubpanelFallbackFooterHeight;
    const preferredMaxHeight =
      projectSubpanelListPadding
      + projectSubpanelRowHeight * projectSubpanelVisibleProjectRows
      + footerHeight;
    const maxHeight = Math.min(preferredMaxHeight, Math.max(0, window.innerHeight - edge * 2));
    const projectCount = activeGroup ? projectsInGroup(activeGroup).length : 0;
    const fallbackVisibleRows = Math.max(1, Math.min(projectSubpanelVisibleProjectRows, projectCount));
    const fallbackListHeight =
      projectSubpanelListPadding + projectSubpanelRowHeight * fallbackVisibleRows;
    const listHeight =
      projectScrollContentElement
        ? projectScrollContentElement.scrollHeight + projectSubpanelListPadding
        : fallbackListHeight;
    const naturalHeight = Math.ceil(listHeight + footerHeight);
    const heightForClamp = Math.min(maxHeight, naturalHeight);
    const top = Math.min(
      Math.max(edge, anchorRect.top),
      Math.max(edge, window.innerHeight - edge - heightForClamp),
    );
    const panelRight = left + width;
    const bridgeLeft = openRight ? anchorRect.right : panelRight;
    const bridgeRight = openRight ? left : anchorRect.left;
    const bridgeTop = Math.max(edge, anchorRect.top);
    const bridgeBottom = Math.min(window.innerHeight - edge, anchorRect.bottom);
    projectSubpanelStyle = [
      `left: ${Math.round(left)}px`,
      `top: ${Math.round(top)}px`,
      `width: ${Math.round(width)}px`,
      `height: ${Math.round(heightForClamp)}px`,
      `max-height: ${Math.round(maxHeight)}px`,
    ].join("; ");
    projectSubpanelBridgeStyle = [
      `left: ${Math.round(bridgeLeft)}px`,
      `top: ${Math.round(bridgeTop)}px`,
      `width: ${Math.max(0, Math.round(bridgeRight - bridgeLeft))}px`,
      `height: ${Math.max(0, Math.round(bridgeBottom - bridgeTop))}px`,
    ].join("; ");
  }

  function refreshGroupScrollState(): void {
    groupScrollStateFrame = null;
    const element = groupScrollElement;
    if (!element) {
      groupScrollable = false;
      groupCanScrollUp = false;
      groupCanScrollDown = false;
      return;
    }
    const maxScrollTop = element.scrollHeight - element.clientHeight;
    groupScrollable = maxScrollTop > 1;
    groupCanScrollUp = element.scrollTop > 1;
    groupCanScrollDown = element.scrollTop < maxScrollTop - 1;
  }

  function requestGroupScrollStateRefresh(): void {
    if (groupScrollStateFrame !== null) cancelAnimationFrame(groupScrollStateFrame);
    groupScrollStateFrame = requestAnimationFrame(refreshGroupScrollState);
  }

  function refreshProjectScrollState(): void {
    projectScrollStateFrame = null;
    const element = projectScrollElement;
    if (!element) {
      projectScrollable = false;
      projectCanScrollUp = false;
      projectCanScrollDown = false;
      return;
    }
    const maxScrollTop = element.scrollHeight - element.clientHeight;
    projectScrollable = maxScrollTop > 1;
    projectCanScrollUp = element.scrollTop > 1;
    projectCanScrollDown = element.scrollTop < maxScrollTop - 1;
  }

  function requestProjectScrollStateRefresh(): void {
    if (projectScrollStateFrame !== null) cancelAnimationFrame(projectScrollStateFrame);
    projectScrollStateFrame = requestAnimationFrame(refreshProjectScrollState);
  }

  function handleGroupScroll(): void {
    refreshGroupScrollState();
  }

  function handleProjectScroll(): void {
    refreshProjectScrollState();
  }

  async function openDropdown(): Promise<void> {
    updateDropdownGeometry();
    open = true;
    await tick();
    updateDropdownGeometry();
    updateProjectSubpanelGeometry();
  }

  function closeDropdown(): void {
    open = false;
    closeProjectSubpanel();
  }

  function toggleDropdown(): void {
    if (open) closeDropdown();
    else void openDropdown();
  }

  function projectsInGroup(group: ProjectGroup): Project[] {
    const groupProjects = projects.projectsForGroup(group.id).filter((project) => project.status === "active");
    if (!normalizedSearch) return groupProjects;
    return groupProjects.filter((project) =>
      project.name.toLowerCase().includes(normalizedSearch)
    );
  }

  function groupVisible(group: ProjectGroup): boolean {
    if (!normalizedSearch) return true;
    return projectsInGroup(group).length > 0;
  }

  function selectProject(project: Project): void {
    onSelect(project.id);
    closeDropdown();
    search = "";
  }

  function closeProjectSubpanel(): void {
    activeGroupId = null;
    activeGroupAnchorEl = null;
    createProjectGroupId = null;
    projectSubpanelFooterElement = undefined;
    projectSubpanelStyle = "";
    projectSubpanelBridgeStyle = "";
  }

  function showProjectSubpanel(group: ProjectGroup, target: EventTarget | null): void {
    if (activeGroupId !== group.id) {
      createProjectGroupId = null;
    }
    activeGroupId = group.id;
    activeGroupAnchorEl = target instanceof HTMLElement ? target : null;
    updateProjectSubpanelGeometry();
    void tick().then(updateProjectSubpanelGeometry);
  }

  function isProjectSubpanelBoundaryTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Node)) return false;
    return Boolean(
      activeGroupAnchorEl?.contains(target)
      || projectSubpanelBridgeEl?.contains(target)
      || projectSubpanelEl?.contains(target),
    );
  }

  function handleProjectSubpanelBoundaryLeave(event: PointerEvent): void {
    if (isProjectSubpanelBoundaryTarget(event.relatedTarget)) return;
    closeProjectSubpanel();
  }

  async function submitGroup(): Promise<void> {
    const name = groupDraft.trim();
    if (!name) return;
    await projects.addGroup(name);
    groupDraft = "";
    createGroupOpen = false;
  }

  async function submitProject(groupId: string): Promise<void> {
    const name = (projectDraftByGroup[groupId] ?? "").trim();
    if (!name) return;
    const templateId = projectTemplateDraftByGroup[groupId] ?? "blank";
    await projects.addProject(groupId, name, templateId);
    projectDraftByGroup = { ...projectDraftByGroup, [groupId]: "" };
    projectTemplateDraftByGroup = { ...projectTemplateDraftByGroup, [groupId]: "blank" };
    createProjectGroupId = null;
    onSelect(projects.selectedProjectId ?? undefined);
    closeDropdown();
  }

  function projectTemplateLabel(templateId: ProjectTemplateId): string {
    if (templateId === "software") return t("projects.templates.software");
    if (templateId === "course") return t("projects.templates.course");
    if (templateId === "routine") return t("projects.templates.routine");
    if (templateId === "reading") return t("projects.templates.reading");
    if (templateId === "chores") return t("projects.templates.chores");
    return t("projects.templates.blank");
  }

  function clearSelection(): void {
    onSelect(undefined);
    closeDropdown();
  }

  const pickerTitle = $derived.by(() => {
    if (!selectedProject) return t("calendar.eventPanel.projectPlaceholder");
    return selectedGroup ? `${selectedProject.name}, ${selectedGroup.name}` : selectedProject.name;
  });

  $effect(() => {
    if (searchActive) {
      closeProjectSubpanel();
      return;
    }
    if (!activeGroupId) return;
    if (!visibleGroups.some((group) => group.id === activeGroupId)) {
      closeProjectSubpanel();
    }
  });

  $effect(() => {
    const scrollElement = groupScrollElement;
    if (!scrollElement) return;
    const resizeObserver = new ResizeObserver(() => {
      updateDropdownGeometry();
      requestGroupScrollStateRefresh();
    });
    resizeObserver.observe(scrollElement);
    if (dropdownHeaderElement) resizeObserver.observe(dropdownHeaderElement);
    if (dropdownFooterElement) resizeObserver.observe(dropdownFooterElement);
    if (groupScrollContentElement) resizeObserver.observe(groupScrollContentElement);
    updateDropdownGeometry();
    requestGroupScrollStateRefresh();
    return () => {
      resizeObserver.disconnect();
      if (groupScrollStateFrame !== null) {
        cancelAnimationFrame(groupScrollStateFrame);
        groupScrollStateFrame = null;
      }
    };
  });

  $effect(() => {
    const scrollElement = projectScrollElement;
    if (!scrollElement) return;
    const resizeObserver = new ResizeObserver(() => {
      requestProjectScrollStateRefresh();
      updateProjectSubpanelGeometry();
    });
    resizeObserver.observe(scrollElement);
    if (projectScrollContentElement) resizeObserver.observe(projectScrollContentElement);
    if (projectSubpanelFooterElement) resizeObserver.observe(projectSubpanelFooterElement);
    requestProjectScrollStateRefresh();
    return () => {
      resizeObserver.disconnect();
      if (projectScrollStateFrame !== null) {
        cancelAnimationFrame(projectScrollStateFrame);
        projectScrollStateFrame = null;
      }
    };
  });
</script>

<div bind:this={rootEl} class="relative flex items-center" data-app-shortcuts="ignore">
  <button
    bind:this={triggerEl}
    type="button"
    disabled={disabled}
    class={cn(
      "flex size-4.5 shrink-0 items-center justify-center rounded-sm text-event-panel-muted-text transition-colors hover:text-event-panel-input-text",
      disabled && "cursor-not-allowed opacity-60",
    )}
    title={pickerTitle}
    aria-label={pickerTitle}
    data-app-tooltip-focus-disabled="true"
    onclick={() => {
      if (!disabled) toggleDropdown();
    }}
  >
    {#if selectedProject}
      <ProjectIcon
        name={selectedProject.icon}
        size={eventPanelProjectIconSize}
        strokeWidth={eventPanelSelectedProjectIconStrokeWidth}
        ignoreColor
        emojiScale={eventPanelProjectEmojiScale}
      />
    {:else}
      <FolderX size={eventPanelProjectIconSize} strokeWidth={eventPanelIconStrokeWidth} />
    {/if}
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div use:portal class="fixed inset-0 z-60" onclick={closeDropdown}></div>
    <div
      use:portal
      class="project-selector-panel fixed z-61 flex flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
      style={dropdownStyle}
    >
      <div bind:this={dropdownHeaderElement} class="px-1.5 pb-0.5 pt-1.5">
        <div class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 pl-2 pr-1">
          <Search size={13} strokeWidth={eventPanelIconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
          <input
            bind:value={search}
            placeholder={t("calendar.eventPanel.searchProjects")}
            class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
          />
          {#if selectedProjectId}
            <button
              type="button"
              onclick={clearSelection}
              class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-popover-foreground/60 hover:bg-accent hover:text-accent-foreground"
              aria-label={t("calendar.eventPanel.projectPlaceholder")}
            >
              <Trash2 size={13} strokeWidth={eventPanelIconStrokeWidth} />
            </button>
          {/if}
        </div>
      </div>

      <div class="relative min-h-0 flex-1">
        <div
          bind:this={groupScrollElement}
          class={cn(
            "project-selector-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto pb-1 pt-0.5",
            groupScrollable && "pr-2",
            groupScrollable
              && groupCanScrollUp
              && groupCanScrollDown
              && "project-selector-scroll-both",
            groupScrollable
              && groupCanScrollUp
              && !groupCanScrollDown
              && "project-selector-scroll-top",
            groupScrollable
              && !groupCanScrollUp
              && groupCanScrollDown
              && "project-selector-scroll-bottom",
          )}
          onscroll={handleGroupScroll}
        >
          <div bind:this={groupScrollContentElement}>
            {#if projects.loading && !projects.loaded}
              <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
                {t("projects.loading")}
              </div>
            {:else if projects.loadError}
              <div class="px-3 py-2 text-[0.8rem] text-destructive">
                {t("projects.loadFailed", projects.loadError)}
              </div>
            {:else if searchActive && searchResultGroups.length === 0}
              <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
                {t("calendar.eventPanel.noProjectsFound")}
              </div>
            {:else if searchActive}
              <div class="grid px-1">
                {#each searchResultGroups as resultGroup (resultGroup.group.id)}
                  <div class="px-1 pb-1">
                    <div class="px-2 pb-0.5 pt-1 text-[0.7rem] font-medium text-popover-foreground/45">
                      {resultGroup.group.name}
                    </div>
                    <div class="grid">
                      {#each resultGroup.projects as project (project.id)}
                        <button
                          type="button"
                          class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
                          aria-label={t("projects.actions.selectProject", project.name, resultGroup.group.name)}
                          onclick={() => selectProject(project)}
                        >
                          <ProjectIcon name={project.icon} size={13} strokeWidth={eventPanelIconStrokeWidth} ignoreColor class="shrink-0" />
                          <span class="truncate">{project.name}</span>
                        </button>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            {:else if visibleGroups.length === 0}
              <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
                {t("calendar.eventPanel.noProjectsFound")}
              </div>
            {:else}
              <div class="grid pl-1">
                {#each visibleGroups as group (group.id)}
                  <div class="pl-1">
                    <button
                      type="button"
                      class={cn(
                        "grid min-h-8 w-full grid-cols-[1.25rem_minmax(0,1fr)_1rem] items-center gap-2 rounded-md px-2 text-left transition-colors",
                        activeGroupId === group.id
                          ? "bg-accent text-accent-foreground"
                          : "text-popover-foreground hover:bg-accent hover:text-accent-foreground",
                      )}
                      onpointerenter={(event) => showProjectSubpanel(group, event.currentTarget)}
                      onpointerleave={handleProjectSubpanelBoundaryLeave}
                      onfocus={(event) => showProjectSubpanel(group, event.currentTarget)}
                      onclick={(event) => showProjectSubpanel(group, event.currentTarget)}
                    >
                      <ProjectIcon name={group.icon} size={13} strokeWidth={eventPanelIconStrokeWidth} ignoreColor class="shrink-0" />
                      <span class="truncate text-[0.8rem] font-medium">{group.name}</span>
                      <ChevronRight size={13} strokeWidth={eventPanelIconStrokeWidth} class="justify-self-end text-popover-foreground/60" />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <CalendarScrollbar
          scrollContainer={groupScrollElement}
          stickyTop={4}
          stickyBottom={4}
          wheelPassthrough
        />
      </div>

      {#if !searchActive}
        <div bind:this={dropdownFooterElement} class="relative z-10 shrink-0 bg-popover p-1.5">
          <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
          {#if createGroupOpen}
            <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitGroup(); }}>
              <input
                bind:value={groupDraft}
                placeholder={t("projects.navigator.groupNamePlaceholder")}
                class="min-h-8 min-w-0 flex-1 rounded border border-border bg-muted/40 px-2 text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
              />
              <button
                type="submit"
                class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
              >
                {t("common.save")}
              </button>
            </form>
          {:else}
            <button
              type="button"
              class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded text-[0.8rem] text-popover-foreground hover:bg-accent hover:text-accent-foreground"
              onclick={() => { createGroupOpen = true; }}
            >
              <Plus size={13} strokeWidth={eventPanelIconStrokeWidth} />
              <span>{t("calendar.eventPanel.createGroup")}</span>
            </button>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  {#if open && !searchActive && activeGroup}
    {@const activeGroupProjects = projectsInGroup(activeGroup)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={projectSubpanelBridgeEl}
      use:portal
      aria-hidden="true"
      class="fixed z-61 bg-transparent"
      style={projectSubpanelBridgeStyle}
      onpointerleave={handleProjectSubpanelBoundaryLeave}
    ></div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      bind:this={projectSubpanelEl}
      use:portal
      class="project-selector-panel fixed z-61 flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
      style={projectSubpanelStyle}
      onpointerleave={handleProjectSubpanelBoundaryLeave}
    >
      <div class="relative min-h-0 flex-1">
        <div
          bind:this={projectScrollElement}
          class={cn(
            "project-selector-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto py-1 pl-1",
            projectScrollable && "pr-2",
            projectScrollable
              && projectCanScrollUp
              && projectCanScrollDown
              && "project-selector-scroll-both",
            projectScrollable
              && projectCanScrollUp
              && !projectCanScrollDown
              && "project-selector-scroll-top",
            projectScrollable
              && !projectCanScrollUp
              && projectCanScrollDown
              && "project-selector-scroll-bottom",
          )}
          onscroll={handleProjectScroll}
        >
          <div bind:this={projectScrollContentElement}>
            {#if activeGroupProjects.length === 0}
              <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
                {t("calendar.eventPanel.noProjectsFound")}
              </div>
            {:else}
              <div class="grid">
                {#each activeGroupProjects as project (project.id)}
                  <button
                    type="button"
                    class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
                    aria-label={t("projects.actions.selectProject", project.name, activeGroup.name)}
                    onclick={() => selectProject(project)}
                  >
                    <ProjectIcon name={project.icon} size={13} strokeWidth={eventPanelIconStrokeWidth} ignoreColor class="shrink-0" />
                    <span class="truncate">{project.name}</span>
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <CalendarScrollbar
          scrollContainer={projectScrollElement}
          stickyTop={4}
          stickyBottom={4}
          wheelPassthrough
        />
      </div>

      <div
        bind:this={projectSubpanelFooterElement}
        class="relative z-10 shrink-0 bg-popover p-1.5"
      >
        <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
        {#if createProjectGroupId === activeGroup.id}
          <form class="grid gap-1" onsubmit={(event) => { event.preventDefault(); void submitProject(activeGroup.id); }}>
            <div class="flex gap-1">
              <input
                value={projectDraftByGroup[activeGroup.id] ?? ""}
                oninput={(event) => {
                  projectDraftByGroup = {
                    ...projectDraftByGroup,
                    [activeGroup.id]: event.currentTarget.value,
                  };
                }}
                placeholder={t("projects.navigator.projectNamePlaceholder")}
                class="min-h-7 min-w-0 flex-1 rounded border border-border bg-muted/40 px-2 text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
              />
              <button
                type="submit"
                class="min-h-7 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
              >
                {t("common.save")}
              </button>
            </div>
            <div class="flex flex-wrap gap-1" aria-label={t("projects.navigator.projectTemplate")}>
              {#each PROJECT_TEMPLATE_IDS as templateId}
                <button
                  type="button"
                  class={cn(
                    "min-h-6 rounded border px-1.5 text-[0.7rem]",
                    (projectTemplateDraftByGroup[activeGroup.id] ?? "blank") === templateId
                      ? "border-primary/60 bg-primary/10 text-primary"
                      : "border-border bg-transparent text-popover-foreground/60 hover:bg-accent hover:text-accent-foreground",
                  )}
                  onclick={() => {
                    projectTemplateDraftByGroup = {
                      ...projectTemplateDraftByGroup,
                      [activeGroup.id]: templateId,
                    };
                  }}
                >
                  {projectTemplateLabel(templateId)}
                </button>
              {/each}
            </div>
          </form>
        {:else}
          <button
            type="button"
            class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
            onclick={() => {
              createProjectGroupId = activeGroup.id;
              void tick().then(updateProjectSubpanelGeometry);
            }}
          >
            <Plus size={13} strokeWidth={eventPanelIconStrokeWidth} />
            <span>{t("calendar.eventPanel.createProject")}</span>
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .project-selector-panel {
    --cal-scrollbar-thumb: color-mix(in srgb, var(--popover-foreground) 18%, var(--popover));
    --cal-scrollbar-thumb-hover: color-mix(in srgb, var(--popover-foreground) 36%, var(--popover));
  }

  .project-selector-scroll-area {
    --project-selector-scroll-fade: 28px;

    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-selector-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--project-selector-scroll-fade), black);
    mask-image: linear-gradient(to bottom, transparent, black var(--project-selector-scroll-fade), black);
  }

  .project-selector-scroll-bottom {
    -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-selector-scroll-fade)), transparent);
    mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-selector-scroll-fade)), transparent);
  }

  .project-selector-scroll-both {
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-selector-scroll-fade),
      black calc(100% - var(--project-selector-scroll-fade)),
      transparent
    );
    mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-selector-scroll-fade),
      black calc(100% - var(--project-selector-scroll-fade)),
      transparent
    );
  }
</style>
