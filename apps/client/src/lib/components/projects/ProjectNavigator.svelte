<script lang="ts">
  import type { ProjectNavigatorPanelMode } from "$lib/projects/project-toolbar";
  import type { Project } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import ProjectPickerPanels from "./ProjectPickerPanels.svelte";

  let {
    selectedProjectId,
    selectedGroupId = null,
    showInactiveProjects,
    iconStrokeWidth = 1.6,
    onShowInactiveProjectsChange,
    onProjectSelected,
    panelMode = "groups",
    panelMaxHeight = null,
    mobileLayout = false,
    onClose = undefined,
  }: {
    selectedProjectId: string | null;
    selectedGroupId?: string | null;
    showInactiveProjects: boolean;
    iconStrokeWidth?: number;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    panelMode?: ProjectNavigatorPanelMode;
    panelMaxHeight?: number | null;
    mobileLayout?: boolean;
    onClose?: () => void;
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
  {iconStrokeWidth}
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
  {mobileLayout}
  {onClose}
/>
