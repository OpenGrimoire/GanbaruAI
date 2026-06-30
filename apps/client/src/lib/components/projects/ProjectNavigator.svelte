<script lang="ts">
  import type { ProjectNavigatorPanelMode } from "$lib/projects/project-toolbar";
  import type { Project } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import ProjectPickerPanels from "./ProjectPickerPanels.svelte";

  let {
    selectedProjectId,
    selectedGroupId = null,
    showInactiveProjects,
    onShowInactiveProjectsChange,
    onProjectSelected,
    panelMode = "groups",
    panelMaxHeight = null,
  }: {
    selectedProjectId: string | null;
    selectedGroupId?: string | null;
    showInactiveProjects: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    panelMode?: ProjectNavigatorPanelMode;
    panelMaxHeight?: number | null;
  } = $props();

  const projects = getProjects();

  let projectSearch = $state("");

  async function selectProjectInNavigator(project: Project): Promise<void> {
    await projects.selectProject(project.id);
    onProjectSelected();
  }
</script>

<ProjectPickerPanels
  {selectedProjectId}
  {selectedGroupId}
  mode={panelMode}
  panelMaxHeight={panelMaxHeight}
  mainVisibleRows={null}
  subpanelVisibleRows={null}
  boundsSelector=".projects-view-root"
  zIndexClass="z-81"
  showInactiveProjects={showInactiveProjects}
  showInactiveToggle
  showLifecycleBadges
  bind:projectSearch
  onShowInactiveProjectsChange={onShowInactiveProjectsChange}
  onProjectSelected={selectProjectInNavigator}
/>
