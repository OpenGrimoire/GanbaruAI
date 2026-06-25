<script lang="ts">
  import { tick } from "svelte";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    isPointerAimingAtSubmenu,
    type MenuAimPoint,
    type MenuAimRect,
    type MenuAimSide,
  } from "$lib/projects/menu-aim";
  import {
    PROJECT_TEMPLATE_IDS,
    type Project,
    type ProjectGroup,
    type ProjectTemplateId,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { cn } from "$lib/utils";
  import type { ProjectNavigatorPanelMode } from "$lib/projects/project-toolbar";
  import ProjectIcon from "./ProjectIcon.svelte";

  type MaybePromise<T> = T | Promise<T>;

  interface PanelBounds {
    left: number;
    right: number;
    top: number;
    bottom: number;
  }

  interface ProjectSearchResultGroup {
    group: ProjectGroup;
    projects: Project[];
  }

  let {
    selectedProjectId = null,
    selectedGroupId = null,
    mode = "groups",
    panelMaxHeight = null,
    panelHeight = $bindable(0),
    mainVisibleRows = 6,
    subpanelVisibleRows = 6,
    pickerBounds = null,
    boundsSelector = null,
    zIndexClass = "z-61",
    showInactiveProjects = false,
    showInactiveToggle = false,
    showClearProject = false,
    showLifecycleBadges = false,
    closeOnProjectCreate = false,
    projectSearch = $bindable(""),
    onShowInactiveProjectsChange = undefined,
    onProjectSelected,
    onProjectCreated = undefined,
    onClearProject = undefined,
  }: {
    selectedProjectId?: string | null;
    selectedGroupId?: string | null;
    mode?: ProjectNavigatorPanelMode;
    panelMaxHeight?: number | null;
    panelHeight?: number;
    mainVisibleRows?: number | null;
    subpanelVisibleRows?: number | null;
    pickerBounds?: PanelBounds | null;
    boundsSelector?: string | null;
    zIndexClass?: string;
    showInactiveProjects?: boolean;
    showInactiveToggle?: boolean;
    showClearProject?: boolean;
    showLifecycleBadges?: boolean;
    closeOnProjectCreate?: boolean;
    projectSearch?: string;
    onShowInactiveProjectsChange?: (value: boolean) => void;
    onProjectSelected: (project: Project) => MaybePromise<void>;
    onProjectCreated?: (projectId: string | null) => MaybePromise<void>;
    onClearProject?: () => MaybePromise<void>;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();
  const iconStrokeWidth = 1.6;
  const iconSize = 13;
  const emojiScale = 0.94;
  const panelFallbackHeaderHeight = 40;
  const panelFallbackFooterHeight = 44;
  const panelListPadding = 6;
  const panelRowHeightRem = 2;
  const panelFallbackRowHeight = 30;
  const subpanelListPadding = 8;
  const subpanelFallbackFooterHeight = 44;
  const subpanelRowHeight = 32;
  const subpanelGap = 4;

  let groupDraft = $state("");
  let createGroupOpen = $state(false);
  let createProjectGroupId = $state<string | null>(null);
  let projectDraftByGroup = $state<Record<string, string>>({});
  let projectTemplateDraftByGroup = $state<Record<string, ProjectTemplateId>>({});
  let activeGroupId = $state<string | null>(null);
  let activeGroupAnchorElement = $state<HTMLElement | null>(null);
  let panelRootElement = $state<HTMLDivElement | null>(null);
  let panelHeaderElement = $state<HTMLDivElement | undefined>();
  let panelFooterElement = $state<HTMLDivElement | undefined>();
  let panelStyle = $state("");
  let groupScrollElement = $state<HTMLElement | undefined>();
  let groupScrollContentElement = $state<HTMLElement | undefined>();
  let groupScrollable = $state(false);
  let groupCanScrollUp = $state(false);
  let groupCanScrollDown = $state(false);
  let groupScrollStateFrame: number | null = null;
  let projectSubpanelElement = $state<HTMLDivElement | undefined>();
  let projectSubpanelBridgeElement = $state<HTMLDivElement | undefined>();
  let projectSubpanelFooterElement = $state<HTMLDivElement | undefined>();
  let projectSubpanelStyle = $state("");
  let projectSubpanelBridgeStyle = $state("");
  let projectScrollElement = $state<HTMLElement | undefined>();
  let projectScrollContentElement = $state<HTMLElement | undefined>();
  let projectScrollable = $state(false);
  let projectCanScrollUp = $state(false);
  let projectCanScrollDown = $state(false);
  let projectScrollStateFrame: number | null = null;

  const selectedProject = $derived(projects.projectById(selectedProjectId));
  const selectedGroup = $derived(projects.groupById(selectedProject?.groupId ?? selectedGroupId));
  const normalizedSearch = $derived(projectSearch.trim().toLowerCase());
  const searchActive = $derived(normalizedSearch.length > 0);
  const groups = $derived.by(() => projects.visibleGroups());
  const visibleGroups = $derived.by(() => groups.filter(groupVisible));
  const searchResultGroups = $derived.by((): ProjectSearchResultGroup[] =>
    groups
      .map((group) => ({ group, projects: projectsInGroup(group) }))
      .filter((entry) => entry.projects.length > 0),
  );
  const activeGroup = $derived.by(() => visibleGroups.find((group) => group.id === activeGroupId));
  const directProjectGroup = $derived.by(() => selectedGroup);
  const directProjects = $derived.by(() => directProjectGroup ? projectsInGroup(directProjectGroup) : []);

  function projectsInGroup(group: ProjectGroup): Project[] {
    const groupProjects = showInactiveProjects
      ? projects.projectsForGroupIncludingInactive(group.id)
      : projects.projectsForGroup(group.id);
    if (!normalizedSearch) return groupProjects;
    return groupProjects.filter((project) =>
      project.name.toLowerCase().includes(normalizedSearch)
    );
  }

  function groupVisible(group: ProjectGroup): boolean {
    if (!normalizedSearch) return true;
    return projectsInGroup(group).length > 0;
  }

  function projectTemplateLabel(templateId: ProjectTemplateId): string {
    if (templateId === "software") return t("projects.templates.software");
    if (templateId === "course") return t("projects.templates.course");
    if (templateId === "routine") return t("projects.templates.routine");
    if (templateId === "reading") return t("projects.templates.reading");
    if (templateId === "chores") return t("projects.templates.chores");
    return t("projects.templates.blank");
  }

  function rowHeight(): number {
    const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    return Number.isFinite(rootFontSize)
      ? rootFontSize * panelRowHeightRem
      : panelFallbackRowHeight;
  }

  function cssPixelValue(value: string): number {
    const parsed = Number.parseFloat(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  function scrollAreaVerticalPadding(element: HTMLElement | undefined, fallback: number): number {
    if (!element) return fallback;
    const style = getComputedStyle(element);
    return cssPixelValue(style.paddingTop) + cssPixelValue(style.paddingBottom);
  }

  function boundedNumber(value: number | null): number | null {
    if (value === null || !Number.isFinite(value)) return null;
    return Math.max(0, value);
  }

  function mainListEstimatedHeight(): number {
    const itemCount = searchActive
      ? searchResultGroups.reduce((count, entry) => count + entry.projects.length, 0)
      : mode === "projects" && directProjectGroup
        ? projectsInGroup(directProjectGroup).length
        : visibleGroups.length;
    const visibleRows = mainVisibleRows === null
      ? itemCount
      : Math.min(itemCount, mainVisibleRows);
    return panelListPadding + rowHeight() * visibleRows;
  }

  function mainPanelCap(headerHeight: number, footerHeight: number): number {
    const availableMaxHeight = boundedNumber(panelMaxHeight);
    const rowCapHeight = mainVisibleRows === null
      ? null
      : headerHeight + footerHeight + panelListPadding + rowHeight() * mainVisibleRows;

    if (availableMaxHeight === null && rowCapHeight === null) return Number.POSITIVE_INFINITY;
    if (availableMaxHeight === null) return rowCapHeight ?? Number.POSITIVE_INFINITY;
    if (rowCapHeight === null) return availableMaxHeight;
    return Math.min(availableMaxHeight, rowCapHeight);
  }

  function updatePanelStyle(): void {
    const headerHeight = panelHeaderElement?.offsetHeight ?? panelFallbackHeaderHeight;
    const footerHeight = searchActive
      ? 0
      : panelFooterElement?.offsetHeight ?? panelFallbackFooterHeight;
    const measuredListHeight = groupScrollContentElement
      ? groupScrollContentElement.scrollHeight + scrollAreaVerticalPadding(
        groupScrollElement,
        panelListPadding,
      )
      : undefined;
    const listHeight = measuredListHeight ?? mainListEstimatedHeight();
    const naturalHeight = Math.ceil(headerHeight + listHeight + footerHeight);
    const maxHeight = mainPanelCap(headerHeight, footerHeight);
    const height = Math.min(maxHeight, naturalHeight);

    panelHeight = Number.isFinite(height) ? Math.round(height) : naturalHeight;
    panelStyle = [
      `height: ${panelHeight}px`,
      `max-height: ${panelHeight}px`,
    ].join("; ");
  }

  function currentPanelBounds(): PanelBounds {
    if (pickerBounds) return pickerBounds;
    const margin = 8;
    const viewportBounds = {
      left: margin,
      right: window.innerWidth - margin,
      top: margin,
      bottom: window.innerHeight - margin,
    };
    const boundsElement = boundsSelector
      ? panelRootElement?.closest(boundsSelector)
      : null;
    const rect = boundsElement?.getBoundingClientRect();
    if (!rect) return viewportBounds;
    return {
      left: Math.max(rect.left, viewportBounds.left),
      right: Math.min(rect.right, viewportBounds.right),
      top: Math.max(rect.top, viewportBounds.top),
      bottom: Math.min(rect.bottom, viewportBounds.bottom),
    };
  }

  function rectToMenuAimRect(rect: DOMRect): MenuAimRect {
    return {
      left: rect.left,
      right: rect.right,
      top: rect.top,
      bottom: rect.bottom,
    };
  }

  function pointerEventPoint(event: PointerEvent): MenuAimPoint {
    return { x: event.clientX, y: event.clientY };
  }

  function projectSubpanelSide(anchorRect: DOMRect, subpanelRect: DOMRect): MenuAimSide {
    if (subpanelRect.right <= anchorRect.left) return "left";
    if (subpanelRect.left >= anchorRect.right) return "right";
    return subpanelRect.left < anchorRect.left ? "left" : "right";
  }

  function projectSubpanelAimOrigin(anchorRect: DOMRect): MenuAimPoint {
    return {
      x: anchorRect.left + anchorRect.width / 2,
      y: anchorRect.top + anchorRect.height / 2,
    };
  }

  function pointerAimingAtProjectSubpanel(point: MenuAimPoint): boolean {
    if (!activeGroupAnchorElement || !projectSubpanelElement) return false;
    const anchorRect = activeGroupAnchorElement.getBoundingClientRect();
    const subpanelRect = projectSubpanelElement.getBoundingClientRect();
    const side = projectSubpanelSide(anchorRect, subpanelRect);
    return isPointerAimingAtSubmenu({
      origin: projectSubpanelAimOrigin(anchorRect),
      point,
      submenu: rectToMenuAimRect(subpanelRect),
      side,
      tolerance: 12,
      topTolerance: 8,
      bottomTolerance: 32,
      minTowardDistance: 3,
    });
  }

  function updateProjectSubpanelGeometry(): void {
    if (!activeGroupAnchorElement || !panelRootElement) return;
    const anchorRect = activeGroupAnchorElement.getBoundingClientRect();
    const panelRect = panelRootElement.getBoundingClientRect();
    const bounds = currentPanelBounds();
    const gap = subpanelGap;
    const minLeft = bounds.left;
    const maxRight = Math.max(minLeft, bounds.right);
    const minTop = bounds.top;
    const maxBottom = Math.max(minTop, bounds.bottom);
    const usableWidth = Math.max(0, maxRight - minLeft);
    const width = Math.min(
      Math.max(panelRect.width, 0),
      usableWidth,
    );
    const rightOrigin = Math.min(Math.max(anchorRect.right, panelRect.left), panelRect.right);
    const leftOrigin = Math.min(Math.max(anchorRect.left, panelRect.left), panelRect.right);
    const spaceRight = maxRight - rightOrigin - gap;
    const spaceLeft = leftOrigin - minLeft - gap;
    const openRight = spaceRight >= width || spaceRight >= spaceLeft;
    const left = openRight
      ? Math.min(rightOrigin + gap, maxRight - width)
      : Math.max(minLeft, leftOrigin - gap - width);
    const footerHeight = projectSubpanelFooterElement?.offsetHeight ?? subpanelFallbackFooterHeight;
    const projectCount = activeGroup ? projectsInGroup(activeGroup).length : 0;
    const fallbackVisibleRows = subpanelVisibleRows === null
      ? projectCount
      : Math.max(1, Math.min(subpanelVisibleRows, projectCount));
    const fallbackListHeight = subpanelListPadding + subpanelRowHeight * fallbackVisibleRows;
    const measuredListHeight = projectScrollContentElement
      ? projectScrollContentElement.scrollHeight + scrollAreaVerticalPadding(
        projectScrollElement,
        subpanelListPadding,
      )
      : undefined;
    const listHeight = measuredListHeight ?? fallbackListHeight;
    const naturalHeight = Math.ceil(listHeight + footerHeight);
    const rowCapHeight = subpanelVisibleRows === null
      ? null
      : subpanelListPadding + subpanelRowHeight * subpanelVisibleRows + footerHeight;
    const boundsMaxHeight = Math.max(0, maxBottom - minTop);
    const maxHeight = rowCapHeight === null
      ? boundsMaxHeight
      : Math.min(rowCapHeight, boundsMaxHeight);
    const height = Math.min(maxHeight, naturalHeight);
    const top = Math.min(
      Math.max(minTop, anchorRect.top),
      Math.max(minTop, maxBottom - height),
    );
    const panelRight = left + width;
    const bridgeLeft = openRight ? rightOrigin : panelRight;
    const bridgeRight = openRight ? left : leftOrigin;
    const bridgeTop = Math.max(minTop, Math.min(anchorRect.top, top));
    const bridgeBottom = Math.min(maxBottom, Math.max(anchorRect.bottom, top + height));

    projectSubpanelStyle = [
      `left: ${Math.round(left)}px`,
      `top: ${Math.round(top)}px`,
      `width: ${Math.round(width)}px`,
      `height: ${Math.round(height)}px`,
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

  function closeProjectSubpanel(): void {
    activeGroupId = null;
    activeGroupAnchorElement = null;
    createProjectGroupId = null;
    projectSubpanelFooterElement = undefined;
    projectSubpanelStyle = "";
    projectSubpanelBridgeStyle = "";
  }

  function showProjectSubpanel(
    group: ProjectGroup,
    target: EventTarget | null,
  ): void {
    if (activeGroupId !== group.id) {
      createProjectGroupId = null;
    }
    activeGroupId = group.id;
    activeGroupAnchorElement = target instanceof HTMLElement ? target : null;
    updateProjectSubpanelGeometry();
    void tick().then(updateProjectSubpanelGeometry);
  }

  function handleGroupPointerEnter(group: ProjectGroup, event: PointerEvent): void {
    const point = pointerEventPoint(event);
    if (activeGroupId && activeGroupId !== group.id && pointerAimingAtProjectSubpanel(point)) {
      return;
    }
    showProjectSubpanel(group, event.currentTarget);
  }

  function handleGroupPointerMove(group: ProjectGroup, event: PointerEvent): void {
    const point = pointerEventPoint(event);
    if (activeGroupId === group.id) {
      return;
    }
    if (activeGroupId && pointerAimingAtProjectSubpanel(point)) {
      return;
    }
    showProjectSubpanel(group, event.currentTarget);
  }

  function isProjectSubpanelBoundaryTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Node)) return false;
    return Boolean(
      activeGroupAnchorElement?.contains(target)
      || projectSubpanelBridgeElement?.contains(target)
      || projectSubpanelElement?.contains(target),
    );
  }

  function handleProjectSubpanelBoundaryLeave(event: PointerEvent): void {
    if (isProjectSubpanelBoundaryTarget(event.relatedTarget)) return;
    if (pointerAimingAtProjectSubpanel(pointerEventPoint(event))) return;
    closeProjectSubpanel();
  }

  async function selectProject(project: Project): Promise<void> {
    await onProjectSelected(project);
    projectSearch = "";
  }

  async function clearProject(): Promise<void> {
    await onClearProject?.();
    projectSearch = "";
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
    await onProjectCreated?.(projects.selectedProjectId);
    if (closeOnProjectCreate) {
      projectSearch = "";
    }
  }

  $effect(() => {
    if (searchActive || mode === "projects") {
      closeProjectSubpanel();
      return;
    }
    if (!activeGroupId) return;
    if (!visibleGroups.some((group) => group.id === activeGroupId)) {
      closeProjectSubpanel();
    }
  });

  $effect(() => {
    const maxHeight = panelMaxHeight;
    const activeSearch = searchActive;
    const panelMode = mode;
    const groupCount = visibleGroups.length;
    const resultCount = searchResultGroups.reduce((count, entry) => count + entry.projects.length, 0);
    const directProjectCount = directProjectGroup ? projectsInGroup(directProjectGroup).length : 0;
    const creatingGroup = createGroupOpen;
    const creatingProject = createProjectGroupId;
    void maxHeight;
    void activeSearch;
    void panelMode;
    void groupCount;
    void resultCount;
    void directProjectCount;
    void creatingGroup;
    void creatingProject;
    requestAnimationFrame(() => {
      updatePanelStyle();
      requestGroupScrollStateRefresh();
      updateProjectSubpanelGeometry();
    });
  });

  $effect(() => {
    const scrollElement = groupScrollElement;
    if (!scrollElement) return;
    const resizeObserver = new ResizeObserver(() => {
      updatePanelStyle();
      requestGroupScrollStateRefresh();
      updateProjectSubpanelGeometry();
    });
    resizeObserver.observe(scrollElement);
    if (panelHeaderElement) resizeObserver.observe(panelHeaderElement);
    if (panelFooterElement) resizeObserver.observe(panelFooterElement);
    if (groupScrollContentElement) resizeObserver.observe(groupScrollContentElement);
    updatePanelStyle();
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

<div
  bind:this={panelRootElement}
  class="project-picker-panel flex min-h-0 w-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
  style={panelStyle}
>
  <div bind:this={panelHeaderElement} class="px-1.5 pb-0.5 pt-1.5">
    <div class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 pl-2 pr-1">
      <Search size={13} strokeWidth={iconStrokeWidth} class="shrink-0 text-popover-foreground/60" />
      <input
        bind:value={projectSearch}
        placeholder={t("calendar.eventPanel.searchProjects")}
        class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
      />
      {#if showInactiveToggle}
        <button
          type="button"
          class={cn(
            "flex h-6 w-6 shrink-0 items-center justify-center rounded text-popover-foreground/60 hover:bg-accent hover:text-accent-foreground",
            showInactiveProjects && "bg-accent text-accent-foreground",
          )}
          aria-label={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
          title={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
          onclick={() => {
            onShowInactiveProjectsChange?.(!showInactiveProjects);
          }}
        >
          {#if showInactiveProjects}
            <EyeOff size={13} strokeWidth={iconStrokeWidth} />
          {:else}
            <Eye size={13} strokeWidth={iconStrokeWidth} />
          {/if}
        </button>
      {/if}
      {#if showClearProject && selectedProjectId}
        <button
          type="button"
          onclick={() => { void clearProject(); }}
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-popover-foreground/60 hover:bg-accent hover:text-accent-foreground"
          aria-label={t("calendar.eventPanel.projectPlaceholder")}
        >
          <Trash2 size={13} strokeWidth={iconStrokeWidth} />
        </button>
      {/if}
    </div>
  </div>

  <div class="relative min-h-0 flex-1">
    <div
      bind:this={groupScrollElement}
      class={cn(
        "project-picker-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto pb-1 pt-0.5",
        groupScrollable && "pr-2",
        groupScrollable && groupCanScrollUp && groupCanScrollDown && "project-picker-scroll-both",
        groupScrollable && groupCanScrollUp && !groupCanScrollDown && "project-picker-scroll-top",
        groupScrollable && !groupCanScrollUp && groupCanScrollDown && "project-picker-scroll-bottom",
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
                      class={cn(
                        "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] transition-colors hover:bg-accent hover:text-accent-foreground",
                        project.status === "active" ? "text-popover-foreground" : "text-popover-foreground/60",
                      )}
                      aria-label={t("projects.actions.selectProject", project.name, resultGroup.group.name)}
                      onclick={() => { void selectProject(project); }}
                    >
                      <ProjectIcon name={project.icon} size={iconSize} strokeWidth={iconStrokeWidth} ignoreColor emojiScale={emojiScale} class="shrink-0" />
                      <span class="min-w-0 flex-1 truncate">{project.name}</span>
                      {#if showLifecycleBadges && project.status !== "active"}
                        <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(project.status))}>
                          {projectLifecycleLabel(project.status, t)}
                        </span>
                      {/if}
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {:else if mode === "groups"}
          {#if visibleGroups.length === 0}
            <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
              {t("calendar.eventPanel.noProjectsFound")}
            </div>
          {:else}
            <div class="grid pl-1">
              {#each visibleGroups as group (group.id)}
                <div class="pl-1 pr-1">
                  <button
                    type="button"
                    class={cn(
                      "grid min-h-8 w-full grid-cols-[1.25rem_minmax(0,1fr)_1rem] items-center gap-2 rounded-md px-2 text-left transition-colors",
                      activeGroupId === group.id
                        ? "bg-accent text-accent-foreground"
                        : "text-popover-foreground hover:bg-accent hover:text-accent-foreground",
                    )}
                    onpointerenter={(event) => handleGroupPointerEnter(group, event)}
                    onpointermove={(event) => handleGroupPointerMove(group, event)}
                    onpointerleave={handleProjectSubpanelBoundaryLeave}
                    onfocus={(event) => showProjectSubpanel(group, event.currentTarget)}
                    onclick={(event) => showProjectSubpanel(group, event.currentTarget)}
                  >
                    <ProjectIcon name={group.icon} size={iconSize} strokeWidth={iconStrokeWidth} ignoreColor emojiScale={emojiScale} class="shrink-0" />
                    <span class="truncate text-[0.8rem] font-medium">{group.name}</span>
                    <ChevronRight size={13} strokeWidth={iconStrokeWidth} class="justify-self-end text-popover-foreground/60" />
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        {:else if !directProjectGroup}
          <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
            {t("projects.navigator.empty")}
          </div>
        {:else if directProjects.length === 0}
          <div class="px-3 py-2 text-[0.8rem] text-popover-foreground/60">
            {t("calendar.eventPanel.noProjectsFound")}
          </div>
        {:else}
          <div class="grid px-1">
            {#each directProjects as project (project.id)}
              <div class="px-1">
                <button
                  type="button"
                  class={cn(
                    "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] transition-colors hover:bg-accent hover:text-accent-foreground",
                    project.status === "active" ? "text-popover-foreground" : "text-popover-foreground/60",
                  )}
                  aria-label={t("projects.actions.selectProject", project.name, directProjectGroup.name)}
                  onclick={() => { void selectProject(project); }}
                >
                  <ProjectIcon name={project.icon} size={iconSize} strokeWidth={iconStrokeWidth} ignoreColor emojiScale={emojiScale} class="shrink-0" />
                  <span class="min-w-0 flex-1 truncate">{project.name}</span>
                  {#if showLifecycleBadges && project.status !== "active"}
                    <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(project.status))}>
                      {projectLifecycleLabel(project.status, t)}
                    </span>
                  {/if}
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
    <div bind:this={panelFooterElement} class="relative z-10 shrink-0 bg-popover p-1.5">
      <div class="pointer-events-none absolute left-1.5 right-1.5 top-0 border-t border-border/70"></div>
      {#if mode === "groups"}
        {#if createGroupOpen}
          <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitGroup(); }}>
            <input
              bind:value={groupDraft}
              placeholder={t("projects.navigator.groupNamePlaceholder")}
              class="min-h-8 min-w-0 flex-1 rounded border border-border bg-muted/40 px-2 text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
            />
            <button type="submit" class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
              {t("common.save")}
            </button>
          </form>
        {:else}
          <button
            type="button"
            class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded-md text-[0.8rem] text-popover-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
            onclick={() => { createGroupOpen = true; }}
          >
            <Plus size={13} strokeWidth={iconStrokeWidth} />
            <span>{t("calendar.eventPanel.createGroup")}</span>
          </button>
        {/if}
      {:else if directProjectGroup}
        {#if createProjectGroupId === directProjectGroup.id}
          <form class="grid gap-1" onsubmit={(event) => { event.preventDefault(); void submitProject(directProjectGroup.id); }}>
            <div class="flex gap-1">
              <input
                value={projectDraftByGroup[directProjectGroup.id] ?? ""}
                oninput={(event) => {
                  projectDraftByGroup = {
                    ...projectDraftByGroup,
                    [directProjectGroup.id]: event.currentTarget.value,
                  };
                }}
                placeholder={t("projects.navigator.projectNamePlaceholder")}
                class="min-h-7 min-w-0 flex-1 rounded border border-border bg-muted/40 px-2 text-[0.8rem] text-popover-foreground placeholder:text-popover-foreground/45"
              />
              <button type="submit" class="min-h-7 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
                {t("common.save")}
              </button>
            </div>
            <div class="flex flex-wrap gap-1" aria-label={t("projects.navigator.projectTemplate")}>
              {#each PROJECT_TEMPLATE_IDS as templateId}
                <button
                  type="button"
                  class={cn(
                    "min-h-6 rounded border px-1.5 text-[0.7rem]",
                    (projectTemplateDraftByGroup[directProjectGroup.id] ?? "blank") === templateId
                      ? "border-primary/60 bg-primary/10 text-primary"
                      : "border-border bg-transparent text-popover-foreground/60 hover:bg-accent hover:text-accent-foreground",
                  )}
                  onclick={() => {
                    projectTemplateDraftByGroup = {
                      ...projectTemplateDraftByGroup,
                      [directProjectGroup.id]: templateId,
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
              createProjectGroupId = directProjectGroup.id;
            }}
          >
            <Plus size={13} strokeWidth={iconStrokeWidth} />
            <span>{t("calendar.eventPanel.createProject")}</span>
          </button>
        {/if}
      {/if}
    </div>
  {/if}
</div>

{#if mode === "groups" && !searchActive && activeGroup && activeGroupAnchorElement}
  {@const activeGroupProjects = projectsInGroup(activeGroup)}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={projectSubpanelBridgeElement}
    aria-hidden="true"
    class={cn("fixed bg-transparent", zIndexClass)}
    style={projectSubpanelBridgeStyle}
    onpointerleave={handleProjectSubpanelBoundaryLeave}
  ></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    bind:this={projectSubpanelElement}
    class={cn("project-picker-panel fixed flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60", zIndexClass)}
    style={projectSubpanelStyle}
    onpointerleave={handleProjectSubpanelBoundaryLeave}
  >
    <div class="relative min-h-0 flex-1">
      <div
        bind:this={projectScrollElement}
        class={cn(
          "project-picker-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto py-1",
          projectScrollable ? "px-1 pr-2" : "px-1",
          projectScrollable && projectCanScrollUp && projectCanScrollDown && "project-picker-scroll-both",
          projectScrollable && projectCanScrollUp && !projectCanScrollDown && "project-picker-scroll-top",
          projectScrollable && !projectCanScrollUp && projectCanScrollDown && "project-picker-scroll-bottom",
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
                  class={cn(
                    "flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem] transition-colors hover:bg-accent hover:text-accent-foreground",
                    project.status === "active" ? "text-popover-foreground" : "text-popover-foreground/60",
                  )}
                  aria-label={t("projects.actions.selectProject", project.name, activeGroup.name)}
                  onclick={() => { void selectProject(project); }}
                >
                  <ProjectIcon name={project.icon} size={iconSize} strokeWidth={iconStrokeWidth} ignoreColor emojiScale={emojiScale} class="shrink-0" />
                  <span class="min-w-0 flex-1 truncate">{project.name}</span>
                  {#if showLifecycleBadges && project.status !== "active"}
                    <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(project.status))}>
                      {projectLifecycleLabel(project.status, t)}
                    </span>
                  {/if}
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

    <div bind:this={projectSubpanelFooterElement} class="relative z-10 shrink-0 bg-popover p-1.5">
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
            <button type="submit" class="min-h-7 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
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
          <Plus size={13} strokeWidth={iconStrokeWidth} />
          <span>{t("calendar.eventPanel.createProject")}</span>
        </button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .project-picker-panel {
    --cal-scrollbar-thumb: color-mix(in srgb, var(--popover-foreground) 18%, var(--popover));
    --cal-scrollbar-thumb-hover: color-mix(in srgb, var(--popover-foreground) 36%, var(--popover));
  }

  .project-picker-scroll-area {
    --project-picker-scroll-fade: 28px;

    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-picker-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--project-picker-scroll-fade), black);
    mask-image: linear-gradient(to bottom, transparent, black var(--project-picker-scroll-fade), black);
  }

  .project-picker-scroll-bottom {
    -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-picker-scroll-fade)), transparent);
    mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-picker-scroll-fade)), transparent);
  }

  .project-picker-scroll-both {
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-picker-scroll-fade),
      black calc(100% - var(--project-picker-scroll-fade)),
      transparent
    );
    mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-picker-scroll-fade),
      black calc(100% - var(--project-picker-scroll-fade)),
      transparent
    );
  }
</style>
