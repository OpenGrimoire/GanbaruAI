<script lang="ts">
  import { tick } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Hash from "@lucide/svelte/icons/hash";
  import Search from "@lucide/svelte/icons/search";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import type { ChatChannelRead } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { Project, ProjectGroup } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { portal } from "$lib/utils/portal";
  import SettingsCheckbox from "../SettingsCheckbox.svelte";

  let {
    channels,
    selectedChannelIds,
    disabled = false,
    onSelectionChange,
  }: {
    channels: readonly ChatChannelRead[];
    selectedChannelIds: ReadonlySet<string>;
    disabled?: boolean;
    onSelectionChange: (channelIds: readonly string[], selected: boolean) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();
  const panelWidth = 300;
  const panelGap = 4;
  const viewportMargin = 8;
  const rowHeight = 32;

  let open = $state(false);
  let query = $state("");
  let channelQuery = $state("");
  let activeGroupId = $state<string | null>(null);
  let activeProjectId = $state<string | null>(null);
  let triggerElement = $state<HTMLButtonElement>();
  let mainPanelElement = $state<HTMLDivElement>();
  let projectPanelElement = $state<HTMLDivElement>();
  let channelPanelElement = $state<HTMLDivElement>();
  let searchElement = $state<HTMLInputElement>();
  let mainScrollElement = $state<HTMLElement>();
  let projectScrollElement = $state<HTMLElement>();
  let channelScrollElement = $state<HTMLElement>();
  let mainPanelStyle = $state("");
  let projectPanelStyle = $state("");
  let channelPanelStyle = $state("");

  const activeChannels = $derived(channels.filter((channel) => channel.archivedAt === null));
  const activeProjects = $derived(projects.projects.filter((project) => project.status === "active"));
  const activeGroups = $derived(projects.visibleGroups());
  const normalizedQuery = $derived(query.trim().toLocaleLowerCase());
  const normalizedChannelQuery = $derived(channelQuery.trim().toLocaleLowerCase());
  const activeGroup = $derived(activeGroups.find((group) => group.id === activeGroupId) ?? null);
  const activeProject = $derived(activeProjects.find((project) => project.id === activeProjectId) ?? null);
  const activeGroupProjects = $derived(activeGroup
    ? activeProjects.filter((project) => project.groupId === activeGroup.id && channelsForProject(project.id).length > 0)
    : []);
  const activeProjectChannels = $derived(activeProject
    ? channelsForProject(activeProject.id).filter((channel) => (
      !normalizedChannelQuery
      || channel.name.toLocaleLowerCase().includes(normalizedChannelQuery)
      || channel.topic.toLocaleLowerCase().includes(normalizedChannelQuery)
    ))
    : []);
  const searchableChannels = $derived(normalizedQuery
    ? activeChannels.filter((channel) => channelSearchText(channel).includes(normalizedQuery))
    : []);
  const visibleGroups = $derived(activeGroups.filter((group) => channelsForGroup(group.id).length > 0));

  function channelsForProject(projectId: string): ChatChannelRead[] {
    return activeChannels.filter((channel) => channel.projectId === projectId);
  }

  function channelsForGroup(groupId: string): ChatChannelRead[] {
    const projectIds = new Set(activeProjects
      .filter((project) => project.groupId === groupId)
      .map((project) => project.id));
    return activeChannels.filter((channel) => projectIds.has(channel.projectId));
  }

  function groupForProject(project: Project | undefined): ProjectGroup | undefined {
    return projects.groups.find((group) => group.id === project?.groupId);
  }

  function channelSearchText(channel: ChatChannelRead): string {
    const project = activeProjects.find((entry) => entry.id === channel.projectId);
    const group = groupForProject(project);
    return `${group?.name ?? ""} ${project?.name ?? ""} ${channel.name} ${channel.topic}`
      .toLocaleLowerCase();
  }

  function channelAncestry(channel: ChatChannelRead): string {
    const project = activeProjects.find((entry) => entry.id === channel.projectId);
    const group = groupForProject(project);
    return `${group?.name ?? t("settings.chat.teammates.unknownGroup")} / ${project?.name ?? t("settings.chat.teammates.unknownProject")}`;
  }

  function selectionState(channelIds: readonly string[]): "none" | "some" | "all" {
    const selectedCount = channelIds.filter((channelId) => selectedChannelIds.has(channelId)).length;
    if (selectedCount === 0) return "none";
    if (selectedCount === channelIds.length) return "all";
    return "some";
  }

  function panelHeight(itemCount: number, chromeHeight = 16): number {
    return Math.min(Math.max(rowHeight + chromeHeight, itemCount * rowHeight + chromeHeight), window.innerHeight - viewportMargin * 2);
  }

  function mainPositionStyle(): string {
    if (!triggerElement) return "visibility:hidden";
    const trigger = triggerElement.getBoundingClientRect();
    const itemCount = normalizedQuery ? searchableChannels.length : visibleGroups.length;
    const height = Math.min(360, panelHeight(Math.max(1, itemCount), 52));
    const availableBelow = window.innerHeight - trigger.bottom - panelGap - viewportMargin;
    const top = availableBelow >= Math.min(height, 200)
      ? trigger.bottom + panelGap
      : Math.max(viewportMargin, trigger.top - panelGap - height);
    const left = Math.min(
      Math.max(viewportMargin, trigger.left),
      window.innerWidth - viewportMargin - panelWidth,
    );
    return `left:${left}px;top:${top}px;width:${panelWidth}px;height:${height}px`;
  }

  function subpanelPositionStyle(anchor: HTMLElement, parent: HTMLElement, itemCount: number, chromeHeight = 16): string {
    const anchorRect = anchor.getBoundingClientRect();
    const parentRect = parent.getBoundingClientRect();
    const height = panelHeight(itemCount, chromeHeight);
    const top = Math.min(
      Math.max(viewportMargin, anchorRect.top - 4),
      window.innerHeight - viewportMargin - height,
    );
    const opensRight = parentRect.right + panelGap + panelWidth <= window.innerWidth - viewportMargin;
    const left = opensRight
      ? parentRect.right + panelGap
      : parentRect.left - panelGap - panelWidth;
    return `left:${Math.max(viewportMargin, left)}px;top:${top}px;width:${panelWidth}px;height:${height}px`;
  }

  async function togglePicker(): Promise<void> {
    if (disabled) return;
    if (open) {
      closePicker();
      return;
    }
    open = true;
    query = "";
    channelQuery = "";
    activeGroupId = null;
    activeProjectId = null;
    await tick();
    mainPanelStyle = mainPositionStyle();
    searchElement?.focus();
  }

  function closePicker(): void {
    open = false;
    activeGroupId = null;
    activeProjectId = null;
  }

  async function showProjects(group: ProjectGroup, anchor: HTMLElement): Promise<void> {
    activeGroupId = group.id;
    activeProjectId = null;
    await tick();
    if (mainPanelElement) {
      projectPanelStyle = subpanelPositionStyle(anchor, mainPanelElement, activeGroupProjects.length);
    }
  }

  async function showChannels(project: Project, anchor: HTMLElement): Promise<void> {
    activeProjectId = project.id;
    channelQuery = "";
    await tick();
    if (projectPanelElement) {
      channelPanelStyle = subpanelPositionStyle(
        anchor,
        projectPanelElement,
        activeProjectChannels.length,
        52,
      );
    }
  }

  function boundaryContains(target: EventTarget | null): boolean {
    if (!(target instanceof Node)) return false;
    return Boolean(
      triggerElement?.contains(target)
      || mainPanelElement?.contains(target)
      || projectPanelElement?.contains(target)
      || channelPanelElement?.contains(target),
    );
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!open || event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    closePicker();
    triggerElement?.focus();
  }

  $effect(() => {
    if (!open) return;
    function handlePointerDown(event: MouseEvent): void {
      if (!boundaryContains(event.target)) closePicker();
    }
    function handleScroll(event: Event): void {
      if (!boundaryContains(event.target)) closePicker();
    }
    function handleResize(): void {
      closePicker();
    }
    window.addEventListener("mousedown", handlePointerDown, true);
    window.addEventListener("scroll", handleScroll, true);
    window.addEventListener("resize", handleResize);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown, true);
      window.removeEventListener("scroll", handleScroll, true);
      window.removeEventListener("resize", handleResize);
    };
  });

  $effect(() => {
    if (!open) return;
    normalizedQuery;
    searchableChannels.length;
    visibleGroups.length;
    void tick().then(() => {
      if (open) mainPanelStyle = mainPositionStyle();
    });
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<button
  bind:this={triggerElement}
  type="button"
  {disabled}
  aria-haspopup="dialog"
  aria-expanded={open}
  onclick={() => { void togglePicker(); }}
  class="flex h-7 min-w-32 items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground transition-colors hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50 dark:bg-transparent"
>
  <span>{t("settings.chat.teammates.addChannel")}</span>
  <ChevronDown size={13} class={`shrink-0 text-muted-foreground transition-transform ${open ? "rotate-180" : ""}`} />
</button>

{#if open}
  <div
    bind:this={mainPanelElement}
    use:portal
    role="dialog"
    aria-label={t("settings.chat.teammates.channelPickerLabel")}
    data-channel-access-picker
    class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60"
    style={mainPanelStyle}
  >
    <div class="shrink-0 px-1.5 pb-0.5 pt-1.5">
      <label class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 px-2">
        <Search size={13} class="shrink-0 text-popover-foreground/60" />
        <input bind:this={searchElement} bind:value={query} placeholder={t("settings.chat.teammates.searchChannels")} aria-label={t("settings.chat.teammates.searchChannels")} class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground outline-none placeholder:text-popover-foreground/45" oninput={() => { activeGroupId = null; activeProjectId = null; }} />
      </label>
    </div>
    <div class="relative min-h-0 flex-1">
      <div bind:this={mainScrollElement} class="hide-scrollbar h-full min-h-0 overflow-y-auto p-1">
        {#if normalizedQuery}
          {#each searchableChannels as channel (channel.id)}
            <div class="grid min-h-8 grid-cols-[1rem_1rem_minmax(0,1fr)] items-center gap-2 rounded-md px-2 text-[0.8rem] hover:bg-accent hover:text-accent-foreground">
              <SettingsCheckbox checked={selectedChannelIds.has(channel.id)} label={t("settings.chat.teammates.channelAccessTitle", channel.name)} onChange={(checked) => onSelectionChange([channel.id], checked)} />
              <Hash size={13} />
              <span class="grid min-w-0"><strong class="truncate font-medium">{channel.name}</strong><small class="truncate text-[0.666667rem] text-muted-foreground">{channelAncestry(channel)}</small></span>
            </div>
          {:else}<p class="px-2 py-1.5 text-[0.8rem] text-muted-foreground">{t("settings.chat.teammates.noChannels")}</p>{/each}
        {:else}
          {#each visibleGroups as group (group.id)}
            {@const groupChannelIds = channelsForGroup(group.id).map((channel) => channel.id)}
            {@const state = selectionState(groupChannelIds)}
            <div class={`grid min-h-8 grid-cols-[1rem_minmax(0,1fr)] items-center gap-2 rounded-md px-2 transition-colors ${activeGroupId === group.id ? "bg-accent text-accent-foreground" : "hover:bg-accent hover:text-accent-foreground"}`}>
              <SettingsCheckbox checked={state === "all"} mixed={state === "some"} label={group.name} onChange={() => onSelectionChange(groupChannelIds, state !== "all")} />
              <button type="button" class="grid min-h-8 min-w-0 grid-cols-[1rem_minmax(0,1fr)_1rem] items-center gap-2 text-left text-[0.8rem]" onpointerenter={(event) => { void showProjects(group, event.currentTarget); }} onfocus={(event) => { void showProjects(group, event.currentTarget); }} onclick={(event) => { void showProjects(group, event.currentTarget); }}>
                <ProjectIcon name={group.icon} size={13} strokeWidth={1.6} emojiScale={0.94} class="shrink-0" />
                <span class="truncate font-medium">{group.name}</span>
                <ChevronRight size={13} class="text-popover-foreground/60" />
              </button>
            </div>
          {/each}
        {/if}
      </div>
      <CalendarScrollbar scrollContainer={mainScrollElement} stickyTop={4} stickyBottom={4} wheelPassthrough />
    </div>
  </div>

  {#if !normalizedQuery && activeGroup}
    <div bind:this={projectPanelElement} use:portal class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-md bg-popover p-1 text-popover-foreground shadow-lg ring-1 ring-border/60" style={projectPanelStyle}>
      <div bind:this={projectScrollElement} class="hide-scrollbar min-h-0 flex-1 overflow-y-auto">
        {#each activeGroupProjects as project (project.id)}
          {@const projectChannelIds = channelsForProject(project.id).map((channel) => channel.id)}
          {@const state = selectionState(projectChannelIds)}
          <div class={`grid min-h-8 grid-cols-[1rem_minmax(0,1fr)] items-center gap-2 rounded-md px-2 transition-colors ${activeProjectId === project.id ? "bg-accent text-accent-foreground" : "hover:bg-accent hover:text-accent-foreground"}`}>
            <SettingsCheckbox checked={state === "all"} mixed={state === "some"} label={project.name} onChange={() => onSelectionChange(projectChannelIds, state !== "all")} />
            <button type="button" class="grid min-h-8 min-w-0 grid-cols-[1rem_minmax(0,1fr)_1rem] items-center gap-2 text-left text-[0.8rem]" onpointerenter={(event) => { void showChannels(project, event.currentTarget); }} onfocus={(event) => { void showChannels(project, event.currentTarget); }} onclick={(event) => { void showChannels(project, event.currentTarget); }}>
              <ProjectIcon name={project.icon} size={13} strokeWidth={1.6} emojiScale={0.94} class="shrink-0" />
              <span class="truncate">{project.name}</span>
              <ChevronRight size={13} class="text-popover-foreground/60" />
            </button>
          </div>
        {/each}
      </div>
      <CalendarScrollbar scrollContainer={projectScrollElement} stickyTop={4} stickyBottom={4} wheelPassthrough />
    </div>
  {/if}

  {#if !normalizedQuery && activeProject}
    <div bind:this={channelPanelElement} use:portal class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-md bg-popover text-popover-foreground shadow-lg ring-1 ring-border/60" style={channelPanelStyle}>
      <div class="shrink-0 px-1.5 pb-0.5 pt-1.5">
        <label class="flex min-h-8 items-center gap-1.5 rounded-md border border-border/70 bg-muted/20 px-2">
          <Search size={13} class="shrink-0 text-popover-foreground/60" />
          <input bind:value={channelQuery} placeholder={t("chat.channels.search")} aria-label={t("chat.channels.search")} class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-popover-foreground outline-none placeholder:text-popover-foreground/45" />
        </label>
      </div>
      <div class="relative min-h-0 flex-1">
        <div bind:this={channelScrollElement} class="hide-scrollbar h-full min-h-0 overflow-y-auto p-1">
          {#each activeProjectChannels as channel (channel.id)}
            <div class="grid min-h-8 grid-cols-[1rem_1rem_minmax(0,1fr)] items-center gap-2 rounded-md px-2 text-[0.8rem] hover:bg-accent hover:text-accent-foreground">
              <SettingsCheckbox checked={selectedChannelIds.has(channel.id)} label={t("settings.chat.teammates.channelAccessTitle", channel.name)} onChange={(checked) => onSelectionChange([channel.id], checked)} />
              <Hash size={13} />
              <span class="truncate">{channel.name}</span>
            </div>
          {:else}<p class="px-2 py-1.5 text-[0.8rem] text-muted-foreground">{t("settings.chat.teammates.noChannels")}</p>{/each}
        </div>
        <CalendarScrollbar scrollContainer={channelScrollElement} stickyTop={4} stickyBottom={4} wheelPassthrough />
      </div>
    </div>
  {/if}
{/if}
