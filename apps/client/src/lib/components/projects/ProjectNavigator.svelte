<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import MoreHorizontal from "@lucide/svelte/icons/more-horizontal";
  import Plus from "@lucide/svelte/icons/plus";
  import Save from "@lucide/svelte/icons/save";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import type { EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    PROJECT_TEMPLATE_IDS,
  } from "$lib/projects/types";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import type {
    Project,
    ProjectGroup,
    ProjectTemplateId,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "./ProjectIcon.svelte";
  import ProjectIconPicker from "./ProjectIconPicker.svelte";

  type ProjectNavigatorPresentation = "sidebar" | "panel";

  let {
    selectedProjectId,
    showInactiveProjects,
    onShowInactiveProjectsChange,
    onProjectSelected,
    presentation = "sidebar",
  }: {
    selectedProjectId: string | null;
    showInactiveProjects: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    presentation?: ProjectNavigatorPresentation;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();
  const { t } = getLocalization();

  let projectSearch = $state("");
  let groupDraft = $state("");
  let createGroupOpen = $state(false);
  let createProjectGroupId = $state<string | null>(null);
  let projectDraftByGroup = $state<Record<string, string>>({});
  let projectTemplateDraftByGroup = $state<Record<string, ProjectTemplateId>>({});
  let editingGroupId = $state<string | null>(null);
  let groupEditorName = $state("");
  let groupEditorIcon = $state("folder");
  let groupEditorColor = $state<EventColor | undefined>(undefined);
  let groupEditorSaving = $state(false);
  let groupEditorError = $state<string | null>(null);
  let pendingDeleteGroupId = $state<string | null>(null);
  let groupEditorDeleting = $state(false);

  const visibleProjectGroups = $derived.by(() => projects.visibleGroups());
  const normalizedProjectSearch = $derived(projectSearch.trim().toLowerCase());
  const pendingDeleteGroup = $derived.by(() =>
    pendingDeleteGroupId ? projects.groupById(pendingDeleteGroupId) : undefined
  );

  function filteredProjectsForGroup(groupId: string) {
    const groupProjects = showInactiveProjects
      ? projects.projectsForGroupIncludingInactive(groupId)
      : projects.projectsForGroup(groupId);
    if (!normalizedProjectSearch) return groupProjects;
    return groupProjects.filter((project) =>
      project.name.toLowerCase().includes(normalizedProjectSearch)
    );
  }

  function groupVisible(groupId: string, groupName: string): boolean {
    return filteredProjectsForGroup(groupId).length > 0
      || groupName.toLowerCase().includes(normalizedProjectSearch);
  }

  function adjacentNavigatorGroup(group: ProjectGroup, direction: -1 | 1): ProjectGroup | undefined {
    const index = visibleProjectGroups.findIndex((entry) => entry.id === group.id);
    if (index < 0) return undefined;
    return visibleProjectGroups[index + direction];
  }

  function adjacentProjectInGroup(project: Project, direction: -1 | 1): Project | undefined {
    const ordered = filteredProjectsForGroup(project.groupId);
    const index = ordered.findIndex((entry) => entry.id === project.id);
    if (index < 0) return undefined;
    return ordered[index + direction];
  }

  function projectTemplateLabel(templateId: ProjectTemplateId): string {
    if (templateId === "software") return t("projects.templates.software");
    if (templateId === "course") return t("projects.templates.course");
    if (templateId === "routine") return t("projects.templates.routine");
    if (templateId === "reading") return t("projects.templates.reading");
    if (templateId === "chores") return t("projects.templates.chores");
    return t("projects.templates.blank");
  }

  async function submitGroup(): Promise<void> {
    await projects.addGroup(groupDraft);
    groupDraft = "";
    createGroupOpen = false;
  }

  function loadGroupEditorDraft(group: ProjectGroup): void {
    editingGroupId = group.id;
    groupEditorName = group.name;
    groupEditorIcon = group.icon;
    groupEditorColor = group.color;
    groupEditorError = null;
  }

  function toggleGroupEditor(group: ProjectGroup): void {
    if (editingGroupId === group.id) {
      editingGroupId = null;
      groupEditorError = null;
      return;
    }
    loadGroupEditorDraft(group);
  }

  function groupEditorDirty(group: ProjectGroup): boolean {
    return groupEditorName !== group.name
      || groupEditorIcon !== group.icon
      || groupEditorColor !== group.color;
  }

  async function saveGroupEditor(group: ProjectGroup): Promise<void> {
    const name = groupEditorName.trim();
    if (!name) {
      groupEditorError = t("projects.navigator.groupNameRequired");
      return;
    }
    groupEditorSaving = true;
    groupEditorError = null;
    try {
      await projects.updateGroup(group, {
        name,
        icon: groupEditorIcon,
        color: groupEditorColor,
      });
      editingGroupId = null;
    } catch (error) {
      groupEditorError = t(
        "projects.navigator.groupSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      groupEditorSaving = false;
    }
  }

  async function moveGroupInNavigator(group: ProjectGroup, direction: -1 | 1): Promise<void> {
    groupEditorError = null;
    try {
      await projects.moveGroup(group, direction);
    } catch (error) {
      groupEditorError = t(
        "projects.navigator.groupSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteGroup(group: ProjectGroup): void {
    pendingDeleteGroupId = group.id;
  }

  function cancelDeleteGroup(): void {
    pendingDeleteGroupId = null;
  }

  async function confirmDeleteGroup(): Promise<void> {
    if (!pendingDeleteGroup || groupEditorDeleting) return;
    const group = pendingDeleteGroup;
    pendingDeleteGroupId = null;
    groupEditorDeleting = true;
    groupEditorError = null;
    try {
      await projects.removeGroup(group);
      if (editingGroupId === group.id) editingGroupId = null;
      if (createProjectGroupId === group.id) createProjectGroupId = null;
      onProjectSelected();
    } catch (error) {
      groupEditorError = t(
        "projects.navigator.groupDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      groupEditorDeleting = false;
    }
  }

  async function submitProject(groupId: string): Promise<void> {
    const name = projectDraftByGroup[groupId] ?? "";
    const templateId = projectTemplateDraftByGroup[groupId] ?? "blank";
    await projects.addProject(groupId, name, templateId);
    projectDraftByGroup = { ...projectDraftByGroup, [groupId]: "" };
    projectTemplateDraftByGroup = { ...projectTemplateDraftByGroup, [groupId]: "blank" };
    createProjectGroupId = null;
  }

  async function moveProjectInNavigator(project: Project, direction: -1 | 1): Promise<void> {
    await projects.moveProject(project, direction, showInactiveProjects);
  }
</script>

  <aside
    class={cn(
      "flex min-h-0 flex-col bg-card/95",
      presentation === "sidebar"
        ? "w-[min(17rem,42vw)] min-w-40 shrink-0 border-r border-border"
        : "h-full w-full overflow-hidden rounded-md border border-border bg-popover text-popover-foreground",
    )}
  >
    <div class="flex shrink-0 items-center gap-2 border-b border-border px-2.5 py-2">
      <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
      <input
        bind:value={projectSearch}
        placeholder={t("projects.navigator.searchPlaceholder")}
        class="min-h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] placeholder:text-muted-foreground"
      />
      <button
        type="button"
        class={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
          showInactiveProjects ? "text-foreground" : "text-muted-foreground",
        )}
        aria-label={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
        title={showInactiveProjects ? t("projects.navigator.hideInactive") : t("projects.navigator.showInactive")}
        onclick={() => {
          onShowInactiveProjectsChange(!showInactiveProjects);
        }}
      >
        {#if showInactiveProjects}
          <EyeOff size={14} strokeWidth={1.75} />
        {:else}
          <Eye size={14} strokeWidth={1.75} />
        {/if}
      </button>
    </div>
    <div class="min-h-0 flex-1 overflow-y-auto py-1">
      {#if projects.loading && !projects.loaded}
        <div class="px-3 py-2 text-[0.8rem] text-muted-foreground">{t("projects.loading")}</div>
      {:else if projects.loadError}
        <div class="px-3 py-2 text-[0.8rem] text-destructive">
          {t("projects.loadFailed", projects.loadError)}
        </div>
      {:else if projects.groups.length === 0}
        <div class="px-3 py-2 text-[0.8rem] text-muted-foreground">{t("projects.navigator.empty")}</div>
      {:else}
        {#each visibleProjectGroups.filter((group) => groupVisible(group.id, group.name)) as group (group.id)}
          {@const groupProjects = filteredProjectsForGroup(group.id)}
          {@const expanded = normalizedProjectSearch.length > 0 || !group.collapsed}
          {@const previousGroup = adjacentNavigatorGroup(group, -1)}
          {@const nextGroup = adjacentNavigatorGroup(group, 1)}
          <div class="px-1">
            <div class="flex items-center gap-1">
              <button
                type="button"
                class="flex min-h-8 min-w-0 flex-1 items-center gap-1.5 rounded px-1.5 text-left text-[0.8rem] font-medium hover:bg-accent"
                aria-label={expanded ? t("projects.actions.collapseGroup") : t("projects.actions.expandGroup")}
                onclick={() => { void projects.setGroupCollapsed(group.id, !group.collapsed); }}
              >
                {#if expanded}
                  <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                {:else}
                  <ChevronRight size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
                {/if}
                <ProjectIcon name={group.icon} size={14} class="shrink-0" />
                <span class="truncate">{group.name}</span>
              </button>
              <button
                type="button"
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("projects.navigator.createProject")}
                onclick={() => {
                  createProjectGroupId = createProjectGroupId === group.id ? null : group.id;
                }}
              >
                <Plus size={14} strokeWidth={1.75} />
              </button>
              <button
                type="button"
                class={cn(
                  "flex h-8 w-8 shrink-0 items-center justify-center rounded hover:bg-accent hover:text-foreground",
                  editingGroupId === group.id ? "text-foreground" : "text-muted-foreground",
                )}
                aria-label={t("projects.navigator.editGroup")}
                title={t("projects.navigator.editGroup")}
                onclick={() => toggleGroupEditor(group)}
              >
                <MoreHorizontal size={14} strokeWidth={1.75} />
              </button>
            </div>
            {#if editingGroupId === group.id}
              <div class="grid gap-2 px-5 py-1">
                <input
                  bind:value={groupEditorName}
                  class="min-h-8 min-w-0 rounded border border-border bg-background px-2 text-[0.8rem]"
                  aria-label={t("projects.navigator.groupName")}
                />
                <ProjectIconPicker
                  value={groupEditorIcon}
                  ariaLabel={t("projects.navigator.selectGroupIcon", groupEditorIcon)}
                  class="w-full"
                  onChange={(nextIcon) => {
                    groupEditorIcon = nextIcon;
                  }}
                />
                <div class="flex min-h-8 items-center justify-between gap-2 rounded border border-border bg-background px-2">
                  <span class="text-[0.733333rem] text-muted-foreground">{t("projects.navigator.groupColor")}</span>
                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      class="rounded border border-border px-1.5 py-0.5 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                      onclick={() => {
                        groupEditorColor = undefined;
                      }}
                    >
                      {t("common.none")}
                    </button>
                    <ColorPicker
                      color={groupEditorColor}
                      theme={theme.current}
                      title={t("projects.navigator.groupColor")}
                      ariaLabel={t("projects.navigator.selectGroupColor")}
                      onselect={(color) => {
                        groupEditorColor = color;
                      }}
                    />
                  </div>
                </div>
                <div class="flex items-center gap-1">
                  <button
                    type="button"
                    class="flex h-8 w-8 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousGroup}
                    aria-label={t("projects.actions.moveGroupUp", group.name)}
                    title={t("projects.actions.moveGroupUp", group.name)}
                    onclick={() => { void moveGroupInNavigator(group, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-8 w-8 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextGroup}
                    aria-label={t("projects.actions.moveGroupDown", group.name)}
                    title={t("projects.actions.moveGroupDown", group.name)}
                    onclick={() => { void moveGroupInNavigator(group, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="ml-auto flex min-h-8 items-center gap-1 rounded border border-border bg-background px-2 text-[0.733333rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={groupEditorSaving || !groupEditorDirty(group)}
                    onclick={() => { void saveGroupEditor(group); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{groupEditorSaving ? t("common.loading") : t("common.save")}</span>
                  </button>
                  <button
                    type="button"
                    class="flex min-h-8 items-center gap-1 rounded border border-destructive/35 bg-background px-2 text-[0.733333rem] text-destructive hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={groupEditorSaving || groupEditorDeleting}
                    aria-label={t("projects.navigator.deleteGroup", group.name)}
                    title={t("projects.navigator.deleteGroup", group.name)}
                    onclick={() => requestDeleteGroup(group)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                    <span>{groupEditorDeleting ? t("common.loading") : t("common.delete")}</span>
                  </button>
                </div>
                {#if groupEditorError}
                  <div class="rounded border border-destructive/40 bg-destructive/10 px-2 py-1 text-[0.733333rem] text-destructive">
                    {groupEditorError}
                  </div>
                {/if}
              </div>
            {/if}
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
                    class="min-h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-[0.8rem]"
                  />
                  <button type="submit" class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
                    {t("common.save")}
                  </button>
                </div>
                <div class="flex flex-wrap gap-1" aria-label={t("projects.navigator.projectTemplate")}>
                  {#each PROJECT_TEMPLATE_IDS as templateId}
                    <button
                      type="button"
                      class={cn(
                        "min-h-7 rounded border px-2 text-[0.733333rem]",
                        (projectTemplateDraftByGroup[group.id] ?? "blank") === templateId
                          ? "border-primary/60 bg-primary/10 text-primary"
                          : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
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
            {#if expanded}
              <div class="pb-1 pl-5">
                {#each groupProjects as project (project.id)}
                  {@const previousProject = adjacentProjectInGroup(project, -1)}
                  {@const nextProject = adjacentProjectInGroup(project, 1)}
                  <div class="flex items-center gap-1">
                    <button
                      type="button"
                      class={cn(
                        "flex min-h-8 min-w-0 flex-1 items-center gap-2 rounded px-2 text-left text-[0.833333rem] hover:bg-accent",
                        selectedProjectId === project.id ? "bg-accent text-accent-foreground" : "text-foreground",
                        project.status !== "active" && selectedProjectId !== project.id && "text-muted-foreground",
                      )}
                      aria-label={t("projects.actions.selectProject", project.name, group.name)}
                      onclick={() => {
                        void projects.selectProject(project.id).catch((error) => {
                          console.error("select project failed", error);
                        });
                        onProjectSelected();
                      }}
                    >
                      <ProjectIcon name={project.icon} size={14} class="shrink-0" />
                      <span class="min-w-0 flex-1 truncate">{project.name}</span>
                      {#if project.status !== "active"}
                        <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(project.status))}>
                          {projectLifecycleLabel(project.status, t)}
                        </span>
                      {/if}
                      {#if project.color !== undefined}
                        <span class="h-2 w-2 shrink-0 rounded-full bg-primary/70"></span>
                      {/if}
                    </button>
                    {#if selectedProjectId === project.id}
                      <button
                        type="button"
                        class="flex h-7 w-7 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!previousProject}
                        aria-label={previousProject ? t("projects.actions.moveProjectUp", project.name) : t("projects.actions.noPreviousProject")}
                        title={previousProject ? t("projects.actions.moveProjectUp", project.name) : t("projects.actions.noPreviousProject")}
                        onclick={() => { void moveProjectInNavigator(project, -1); }}
                      >
                        <ArrowUp size={13} strokeWidth={1.75} />
                      </button>
                      <button
                        type="button"
                        class="flex h-7 w-7 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                        disabled={!nextProject}
                        aria-label={nextProject ? t("projects.actions.moveProjectDown", project.name) : t("projects.actions.noNextProject")}
                        title={nextProject ? t("projects.actions.moveProjectDown", project.name) : t("projects.actions.noNextProject")}
                        onclick={() => { void moveProjectInNavigator(project, 1); }}
                      >
                        <ArrowDown size={13} strokeWidth={1.75} />
                      </button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
    <div class="shrink-0 border-t border-border p-2">
      {#if createGroupOpen}
        <form class="flex gap-1" onsubmit={(event) => { event.preventDefault(); void submitGroup(); }}>
          <input
            bind:value={groupDraft}
            placeholder={t("projects.navigator.groupNamePlaceholder")}
            class="min-h-8 min-w-0 flex-1 rounded border border-border bg-background px-2 text-[0.8rem]"
          />
          <button type="submit" class="min-h-8 rounded bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground">
            {t("common.save")}
          </button>
        </form>
      {:else}
        <button
          type="button"
          class="flex min-h-8 w-full items-center justify-center gap-1.5 rounded text-[0.8rem] text-foreground hover:bg-accent"
          onclick={() => { createGroupOpen = true; }}
        >
          <Plus size={14} strokeWidth={1.75} />
          <span>{t("projects.navigator.createGroup")}</span>
        </button>
      {/if}
    </div>
  </aside>

  {#if pendingDeleteGroup}
    <ConfirmDialog
      title={t("projects.navigator.deleteGroupTitle", pendingDeleteGroup.name)}
      message={t(
        "projects.navigator.deleteGroupMessage",
        pendingDeleteGroup.name,
        projects.projectsForGroupIncludingInactive(pendingDeleteGroup.id).length,
      )}
      confirmLabel={t("projects.navigator.deleteGroupConfirm")}
      cancelLabel={t("common.cancelShortcut")}
      onConfirm={() => { void confirmDeleteGroup(); }}
      onCancel={cancelDeleteGroup}
    />
  {/if}
