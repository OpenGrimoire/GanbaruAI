<script lang="ts">
  import { onMount, tick } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import CircleQuestionMark from "@lucide/svelte/icons/circle-question-mark";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import { getEventColor } from "$lib/components/calendar/utils";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { cn } from "$lib/utils";
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
  const theme = getTheme();
  const { t } = getLocalization();
  let open = $state(false);
  let search = $state("");
  let groupDraft = $state("");
  let projectDraftByGroup = $state<Record<string, string>>({});
  let projectTemplateDraftByGroup = $state<Record<string, ProjectTemplateId>>({});
  let createGroupOpen = $state(false);
  let createProjectGroupId = $state<string | null>(null);
  let triggerEl: HTMLButtonElement | undefined = $state();
  let dropdownStyle = $state("");

  const selectedProject = $derived(projects.projectById(selectedProjectId));
  const selectedGroup = $derived(projects.groupById(selectedProject?.groupId));
  const pickerColor = $derived(getEventColor(selectedProject?.color, theme.current));
  const normalizedSearch = $derived(search.trim().toLowerCase());
  const groups = $derived.by(() => projects.visibleGroups());

  onMount(() => {
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });

    const updateWhenOpen = () => {
      if (open) updateDropdownGeometry();
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

  function updateDropdownGeometry(): void {
    if (!triggerEl) return;
    const triggerRect = triggerEl.getBoundingClientRect();
    const bounds = dropdownBounds();
    const gap = 4;
    const maxHeight = 304;
    const minWidth = 224;
    const maxWidth = 288;
    const boundsWidth = Math.max(0, bounds.right - bounds.left);
    const width = Math.min(maxWidth, Math.max(minWidth, boundsWidth));
    const left = Math.min(
      Math.max(triggerRect.right - width, bounds.left),
      bounds.right - width,
    );
    const spaceBelow = bounds.bottom - triggerRect.bottom - gap;
    const spaceAbove = triggerRect.top - bounds.top - gap;
    const openAbove = spaceBelow < 144 && spaceAbove > spaceBelow;
    const availableHeight = Math.max(0, openAbove ? spaceAbove : spaceBelow);
    const height = Math.min(maxHeight, availableHeight);
    const top = openAbove
      ? Math.max(bounds.top, triggerRect.top - gap - height)
      : triggerRect.bottom + gap;
    dropdownStyle = [
      `left: ${Math.round(left)}px`,
      `top: ${Math.round(top)}px`,
      `width: ${Math.round(width)}px`,
      `max-height: ${Math.round(height)}px`,
    ].join("; ");
  }

  async function openDropdown(): Promise<void> {
    updateDropdownGeometry();
    open = true;
    await tick();
    updateDropdownGeometry();
  }

  function closeDropdown(): void {
    open = false;
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
      || group.name.toLowerCase().includes(normalizedSearch)
    );
  }

  function groupVisible(group: ProjectGroup): boolean {
    return projectsInGroup(group).length > 0 || group.name.toLowerCase().includes(normalizedSearch);
  }

  function groupExpanded(group: ProjectGroup): boolean {
    return normalizedSearch.length > 0 || !group.collapsed;
  }

  function selectProject(project: Project): void {
    onSelect(project.id);
    closeDropdown();
    search = "";
  }

  async function toggleGroup(group: ProjectGroup): Promise<void> {
    await projects.setGroupCollapsed(group.id, !group.collapsed);
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

  const pickerStyle = $derived(`background-color: ${pickerColor.bg}; color: ${pickerColor.text};`);
</script>

<div class="relative flex items-center" data-app-shortcuts="ignore">
  <button
    bind:this={triggerEl}
    type="button"
    disabled={disabled}
    class={cn(
      "flex size-4.5 shrink-0 items-center justify-center rounded-sm border border-event-panel-divider/70 transition-opacity hover:opacity-90",
      disabled && "cursor-not-allowed opacity-60",
    )}
    style={pickerStyle}
    title={pickerTitle}
    aria-label={pickerTitle}
    data-app-tooltip-focus-disabled="true"
    onclick={() => {
      if (!disabled) toggleDropdown();
    }}
  >
    {#if selectedProject}
      <ProjectIcon name={selectedProject.icon} size={13} strokeWidth={2} />
    {:else}
      <CircleQuestionMark size={13} strokeWidth={2} />
    {/if}
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-60" onclick={closeDropdown}></div>
    <div
      class="fixed z-61 flex flex-col overflow-hidden rounded-md border border-event-panel-divider bg-event-panel-bg shadow-xl"
      style={dropdownStyle}
    >
      <div class="flex items-center gap-2 border-b border-event-panel-divider/70 px-2 py-1.5">
        <Search size={13} strokeWidth={1.75} class="shrink-0 text-event-panel-muted-text" />
        <input
          bind:value={search}
          placeholder={t("calendar.eventPanel.searchProjects")}
          class="min-w-0 flex-1 bg-transparent text-[0.8rem] text-event-panel-input-text placeholder:text-event-panel-placeholder"
        />
        {#if selectedProjectId}
          <button
            type="button"
            onclick={clearSelection}
            class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-event-panel-muted-text hover:bg-event-panel-contrast hover:text-event-panel-input-text"
            aria-label={t("calendar.eventPanel.projectPlaceholder")}
          >
            <X size={13} strokeWidth={1.75} />
          </button>
        {/if}
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1">
        {#if projects.loading && !projects.loaded}
          <div class="px-3 py-2 text-[0.8rem] text-event-panel-muted-text">
            {t("projects.loading")}
          </div>
        {:else if projects.loadError}
          <div class="px-3 py-2 text-[0.8rem] text-destructive">
            {t("projects.loadFailed", projects.loadError)}
          </div>
        {:else if groups.filter(groupVisible).length === 0}
          <div class="px-3 py-2 text-[0.8rem] text-event-panel-muted-text">
            {t("calendar.eventPanel.noProjectsFound")}
          </div>
        {:else}
          {#each groups.filter(groupVisible) as group (group.id)}
            {@const groupProjects = projectsInGroup(group)}
            <div class="px-1">
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  class="flex min-h-7 min-w-0 flex-1 items-center gap-1.5 rounded px-1.5 text-left text-[0.8rem] font-medium text-event-panel-text hover:bg-event-panel-contrast"
                  onclick={() => { void toggleGroup(group); }}
                >
                  {#if groupExpanded(group)}
                    <ChevronDown size={13} strokeWidth={1.75} class="shrink-0" />
                  {:else}
                    <ChevronRight size={13} strokeWidth={1.75} class="shrink-0" />
                  {/if}
                  <ProjectIcon name={group.icon} size={13} class="shrink-0" />
                  <span class="truncate">{group.name}</span>
                </button>
                <button
                  type="button"
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded text-event-panel-muted-text hover:bg-event-panel-contrast hover:text-event-panel-input-text"
                  aria-label={t("calendar.eventPanel.createProject")}
                  onclick={() => {
                    createProjectGroupId = createProjectGroupId === group.id ? null : group.id;
                  }}
                >
                  <Plus size={13} strokeWidth={1.75} />
                </button>
              </div>
              {#if createProjectGroupId === group.id}
                <form class="grid gap-1 px-5 py-1" onsubmit={(event) => { event.preventDefault(); void submitProject(group.id); }}>
                  <div class="flex gap-1">
                    <input
                      value={projectDraftByGroup[group.id] ?? ""}
                      oninput={(event) => {
                        projectDraftByGroup = {
                          ...projectDraftByGroup,
                          [group.id]: event.currentTarget.value,
                        };
                      }}
                      placeholder={t("projects.navigator.projectNamePlaceholder")}
                      class="min-h-7 min-w-0 flex-1 rounded border border-event-panel-divider bg-event-panel-contrast px-2 text-[0.8rem] text-event-panel-input-text placeholder:text-event-panel-placeholder"
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
                          (projectTemplateDraftByGroup[group.id] ?? "blank") === templateId
                            ? "border-primary/60 bg-primary/10 text-primary"
                            : "border-event-panel-divider bg-event-panel-contrast text-event-panel-muted-text hover:bg-event-panel-surface hover:text-event-panel-input-text",
                        )}
                        onclick={() => {
                          projectTemplateDraftByGroup = {
                            ...projectTemplateDraftByGroup,
                            [group.id]: templateId,
                          };
                        }}
                      >
                        {projectTemplateLabel(templateId)}
                      </button>
                    {/each}
                  </div>
                </form>
              {/if}
              {#if groupExpanded(group)}
                <div class="pb-1 pl-5">
                  {#each groupProjects as project (project.id)}
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-7 w-full items-center gap-2 rounded px-2 text-left text-[0.8rem] hover:bg-event-panel-contrast",
                        selectedProjectId === project.id
                          ? "bg-event-panel-contrast text-event-panel-input-text"
                          : "text-event-panel-text",
                      )}
                      aria-label={t("projects.actions.selectProject", project.name, group.name)}
                      onclick={() => selectProject(project)}
                    >
                      <ProjectIcon name={project.icon} size={13} class="shrink-0" />
                      <span class="truncate">{project.name}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>

      <div class="border-t border-event-panel-divider/70 p-1.5">
        {#if createGroupOpen}
          <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitGroup(); }}>
            <input
              bind:value={groupDraft}
              placeholder={t("projects.navigator.groupNamePlaceholder")}
              class="min-h-8 min-w-0 flex-1 rounded border border-event-panel-divider bg-event-panel-contrast px-2 text-[0.8rem] text-event-panel-input-text placeholder:text-event-panel-placeholder"
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
            class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded text-[0.8rem] text-event-panel-text hover:bg-event-panel-contrast"
            onclick={() => { createGroupOpen = true; }}
          >
            <Plus size={13} strokeWidth={1.75} />
            <span>{t("calendar.eventPanel.createGroup")}</span>
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
