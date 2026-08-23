<script lang="ts">
  import type { ChatChannelRead } from "$lib/chat/contracts";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { Project, ProjectGroup } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import SettingsCheckbox from "../SettingsCheckbox.svelte";

  let {
    channels,
    selectedChannelIds,
    focusedChannelId = null,
    disabled = false,
    onSelectionChange,
    onFocusChange,
  }: {
    channels: readonly ChatChannelRead[];
    selectedChannelIds: ReadonlySet<string>;
    focusedChannelId?: string | null;
    disabled?: boolean;
    onSelectionChange: (channelIds: readonly string[], selected: boolean) => void;
    onFocusChange: (channelId: string) => void;
  } = $props();

  const projects = getProjects();
  const { t } = getLocalization();

  let activeGroupId = $state<string | null>(null);
  let activeProjectId = $state<string | null>(null);
  let observedFocusedChannelId = $state<string | null>(null);
  let groupScrollElement = $state<HTMLElement>();
  let projectScrollElement = $state<HTMLElement>();
  let channelScrollElement = $state<HTMLElement>();

  const activeChannels = $derived(channels.filter((channel) => channel.archivedAt === null));
  const activeProjects = $derived(projects.projects.filter((project) => project.status === "active"));
  const visibleGroups = $derived(projects.visibleGroups().filter((group) => channelsForGroup(group.id).length > 0));
  const activeGroup = $derived(visibleGroups.find((group) => group.id === activeGroupId) ?? visibleGroups[0] ?? null);
  const visibleProjects = $derived(activeGroup
    ? activeProjects.filter((project) => (
      project.groupId === activeGroup.id && channelsForProject(project.id).length > 0
    ))
    : []);
  const activeProject = $derived(visibleProjects.find((project) => project.id === activeProjectId) ?? visibleProjects[0] ?? null);
  const visibleChannels = $derived(activeProject ? channelsForProject(activeProject.id) : []);

  function channelsForProject(projectId: string): ChatChannelRead[] {
    return activeChannels.filter((channel) => channel.projectId === projectId);
  }

  function channelsForGroup(groupId: string): ChatChannelRead[] {
    const projectIds = new Set(activeProjects
      .filter((project) => project.groupId === groupId)
      .map((project) => project.id));
    return activeChannels.filter((channel) => projectIds.has(channel.projectId));
  }

  function selectionState(channelIds: readonly string[]): "none" | "some" | "all" {
    const selectedCount = channelIds.filter((channelId) => selectedChannelIds.has(channelId)).length;
    if (selectedCount === 0) return "none";
    if (selectedCount === channelIds.length) return "all";
    return "some";
  }

  function focusFirstChannel(project: Project | undefined): void {
    const firstChannel = project ? channelsForProject(project.id)[0] : undefined;
    if (firstChannel) onFocusChange(firstChannel.id);
  }

  function openGroup(group: ProjectGroup): void {
    activeGroupId = group.id;
    const firstProject = activeProjects.find((project) => (
      project.groupId === group.id && channelsForProject(project.id).length > 0
    ));
    activeProjectId = firstProject?.id ?? null;
    focusFirstChannel(firstProject);
  }

  function openProject(project: Project): void {
    activeProjectId = project.id;
    focusFirstChannel(project);
  }

  $effect(() => {
    const requestedChannelId = focusedChannelId;
    if (requestedChannelId === observedFocusedChannelId && activeGroupId && activeProjectId) return;
    observedFocusedChannelId = requestedChannelId;

    const focusedChannel = activeChannels.find((channel) => channel.id === requestedChannelId);
    const focusedProject = activeProjects.find((project) => project.id === focusedChannel?.projectId);
    if (focusedProject) {
      activeGroupId = focusedProject.groupId;
      activeProjectId = focusedProject.id;
      return;
    }

    const firstGroup = visibleGroups[0];
    const firstProject = firstGroup
      ? activeProjects.find((project) => (
        project.groupId === firstGroup.id && channelsForProject(project.id).length > 0
      ))
      : undefined;
    activeGroupId = firstGroup?.id ?? null;
    activeProjectId = firstProject?.id ?? null;
    const firstChannel = firstProject ? channelsForProject(firstProject.id)[0] : undefined;
    if (firstChannel && requestedChannelId !== firstChannel.id) onFocusChange(firstChannel.id);
  });
</script>

<div class="access-browser" aria-label={t("settings.chat.teammates.channelPickerLabel")}>
  <section class="browser-column" aria-labelledby="access-groups-heading">
    <h5 id="access-groups-heading">{t("settings.chat.teammates.groups")}</h5>
    <div class="browser-scroll-frame">
      <div bind:this={groupScrollElement} class="browser-list hide-scrollbar">
        {#each visibleGroups as group (group.id)}
          {@const groupChannelIds = channelsForGroup(group.id).map((channel) => channel.id)}
          {@const state = selectionState(groupChannelIds)}
          <div class:active={activeGroup?.id === group.id} class="browser-row">
            <SettingsCheckbox
              checked={state === "all"}
              mixed={state === "some"}
              label={group.name}
              {disabled}
              onChange={() => onSelectionChange(groupChannelIds, state !== "all")}
            />
            <button type="button" disabled={disabled} onclick={() => openGroup(group)}>{group.name}</button>
          </div>
        {/each}
      </div>
      <CalendarScrollbar scrollContainer={groupScrollElement} stickyTop={2} stickyBottom={2} wheelPassthrough />
    </div>
  </section>

  <section class="browser-column" aria-labelledby="access-projects-heading">
    <h5 id="access-projects-heading">{t("settings.chat.teammates.projects")}</h5>
    <div class="browser-scroll-frame">
      <div bind:this={projectScrollElement} class="browser-list hide-scrollbar">
        {#each visibleProjects as project (project.id)}
          {@const projectChannelIds = channelsForProject(project.id).map((channel) => channel.id)}
          {@const state = selectionState(projectChannelIds)}
          <div class:active={activeProject?.id === project.id} class="browser-row">
            <SettingsCheckbox
              checked={state === "all"}
              mixed={state === "some"}
              label={project.name}
              {disabled}
              onChange={() => onSelectionChange(projectChannelIds, state !== "all")}
            />
            <button type="button" disabled={disabled} onclick={() => openProject(project)}>{project.name}</button>
          </div>
        {/each}
      </div>
      <CalendarScrollbar scrollContainer={projectScrollElement} stickyTop={2} stickyBottom={2} wheelPassthrough />
    </div>
  </section>

  <section class="browser-column" aria-labelledby="access-channels-heading">
    <h5 id="access-channels-heading">{t("settings.chat.teammates.channels")}</h5>
    <div class="browser-scroll-frame">
      <div bind:this={channelScrollElement} class="browser-list hide-scrollbar">
        {#each visibleChannels as channel (channel.id)}
          <div class:active={focusedChannelId === channel.id} class="browser-row">
            <SettingsCheckbox
              checked={selectedChannelIds.has(channel.id)}
              label={t("settings.chat.teammates.channelAccessTitle", channel.name)}
              {disabled}
              onChange={(checked) => onSelectionChange([channel.id], checked)}
            />
            <button type="button" disabled={disabled} onclick={() => onFocusChange(channel.id)}>{channel.name}</button>
          </div>
        {:else}
          <p>{t("settings.chat.teammates.noChannels")}</p>
        {/each}
      </div>
      <CalendarScrollbar scrollContainer={channelScrollElement} stickyTop={2} stickyBottom={2} wheelPassthrough />
    </div>
  </section>
</div>

<style>
  .access-browser {
    display:grid;
    min-width:0;
    height:100%;
    min-height:0;
    grid-template-columns:repeat(3,minmax(0,1fr));
  }

  .browser-column {
    display:grid;
    min-width:0;
    min-height:0;
    grid-template-rows:auto minmax(0,1fr);
    padding:0.45rem 0.5rem 0.35rem;
  }

  .browser-column:first-child {
    padding-left:0;
  }

  .browser-column + .browser-column {
    border-left:1px solid color-mix(in srgb,var(--border) 55%,transparent);
  }

  h5 {
    padding:0 0.35rem 0.35rem;
    color:var(--muted-foreground);
    font-size:calc(0.616667rem * var(--type-scale));
    font-weight:650;
  }

  .browser-column:first-child h5,
  .browser-column:first-child .browser-row {
    padding-left:0;
  }

  .browser-scroll-frame {
    position:relative;
    min-height:0;
  }

  .browser-list {
    height:100%;
    min-height:0;
    overflow-y:auto;
    overscroll-behavior:contain;
  }

  .browser-row {
    display:grid;
    min-width:0;
    min-height:1.9rem;
    grid-template-columns:auto minmax(0,1fr);
    align-items:center;
    gap:0.42rem;
    border-radius:0.35rem;
    padding:0.2rem 0.35rem;
  }

  .browser-row:hover,
  .browser-row.active {
    background:var(--accent);
    color:var(--accent-foreground);
  }

  .browser-row > button {
    min-width:0;
    height:100%;
    overflow:hidden;
    color:inherit;
    font-size:calc(0.7rem * var(--type-scale));
    font-weight:500;
    text-align:left;
    text-overflow:ellipsis;
    white-space:nowrap;
  }

  .browser-row > button:disabled {
    cursor:not-allowed;
  }

  .browser-list > p {
    padding:0.3rem 0.35rem;
    color:var(--muted-foreground);
    font-size:calc(0.66rem * var(--type-scale));
  }

  @media (pointer:coarse) {
    .browser-row { min-height:2.75rem; }
  }
</style>
