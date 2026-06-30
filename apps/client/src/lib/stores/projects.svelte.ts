import { loadProjectsSnapshot } from "$lib/api/projects";
import { mergeProjectSnapshot } from "$lib/projects/project-snapshot";
import {
  loadSavedActiveProjectId,
  loadSavedProjectViewId,
  saveActiveProjectId,
  saveProjectViewId,
} from "$lib/projects/project-ui-preferences";
import { createProjectStoreActions } from "$lib/stores/project-store-actions";
import { createProjectStoreSelectors } from "$lib/stores/project-store-selectors";
import type {
  Project,
  ProjectChecklistItem,
  ProjectCustomEmoji,
  ProjectCustomField,
  ProjectCustomFieldOption,
  ProjectCustomFieldOptionValue,
  ProjectCustomFieldValue,
  ProjectGroup,
  ProjectTag,
  ProjectPriorityConfig,
  ProjectsSnapshot,
  ProjectSection,
  ProjectStatus,
  ProjectTask,
  ProjectTaskChangeEvent,
  ProjectTaskDependency,
  ProjectTaskEventLink,
  ProjectTaskTagLink,
  ProjectViewPreference,
  ProjectViewId,
} from "$lib/projects/types";

let snapshot = $state<ProjectsSnapshot>({
  groups: [],
  projects: [],
  sections: [],
  statuses: [],
  priorities: [],
  tasks: [],
  checklistItems: [],
  tags: [],
  taskTagLinks: [],
  customFields: [],
  customFieldOptions: [],
  customFieldValues: [],
  customFieldOptionValues: [],
  dependencies: [],
  eventLinks: [],
  taskChangeEvents: [],
  viewPreferences: [],
  customEmojis: [],
});
let loaded = $state(false);
let loading = $state(false);
let loadError = $state<string | null>(null);
let selectedProjectId = $state<string | null>(loadSavedActiveProjectId());
let activeView = $state<ProjectViewId>(loadSavedProjectViewId());
let loadedProjectIds = $state<string[]>([]);
let loadRequestId = 0;
let selectProjectRequestId = 0;
const selectors = createProjectStoreSelectors(() => snapshot);

function ensureSelectedProject(): void {
  if (snapshot.projects.length === 0) return;
  if (selectedProjectId && snapshot.projects.some((project) => project.id === selectedProjectId)) {
    return;
  }
  setSelectedProjectId(selectors.firstProjectId());
}

function setSelectedProjectId(projectId: string | null): void {
  if (selectedProjectId === projectId) return;
  selectedProjectId = projectId;
  saveActiveProjectId(projectId);
}

function projectDataLoaded(projectId: string | null | undefined): boolean {
  return Boolean(projectId && loadedProjectIds.includes(projectId));
}

function markProjectDataLoaded(projectId: string): void {
  if (loadedProjectIds.includes(projectId)) return;
  loadedProjectIds = [...loadedProjectIds, projectId];
}

function setActiveView(view: ProjectViewId): void {
  if (activeView === view) return;
  activeView = view;
  saveProjectViewId(view);
}

function applyLoadedSnapshot(incoming: ProjectsSnapshot, projectId: string | null | undefined): void {
  if (!projectId) {
    snapshot = {
      ...snapshot,
      groups: incoming.groups,
      projects: incoming.projects,
      viewPreferences: incoming.viewPreferences,
      customEmojis: incoming.customEmojis,
    };
    return;
  }
  snapshot = mergeProjectSnapshot(snapshot, incoming, projectId);
  if (incoming.projects.some((project) => project.id === projectId)) {
    markProjectDataLoaded(projectId);
  }
}

async function reload(projectId: string | null = selectedProjectId): Promise<void> {
  const requestId = ++loadRequestId;
  loading = true;
  loadError = null;
  try {
    const incoming = await loadProjectsSnapshot(projectId);
    if (requestId !== loadRequestId) return;
    applyLoadedSnapshot(incoming, projectId);
    loaded = true;
    const requestedProjectId = projectId;
    ensureSelectedProject();
    if (selectedProjectId && selectedProjectId !== requestedProjectId && !projectDataLoaded(selectedProjectId)) {
      const selectedSnapshot = await loadProjectsSnapshot(selectedProjectId);
      if (requestId !== loadRequestId) return;
      applyLoadedSnapshot(selectedSnapshot, selectedProjectId);
    }
  } catch (error) {
    if (requestId !== loadRequestId) return;
    loadError = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    if (requestId === loadRequestId) loading = false;
  }
}

async function ensureLoaded(): Promise<void> {
  if (loaded || loading) return;
  await reload();
}

async function ensureProjectData(projectId: string | null | undefined): Promise<void> {
  if (!projectId) return;
  if (!loaded) await ensureLoaded();
  if (projectDataLoaded(projectId)) return;
  await reload(projectId);
}

async function selectProject(projectId: string | null): Promise<void> {
  const requestId = ++selectProjectRequestId;
  if (!projectId || projectDataLoaded(projectId)) {
    setSelectedProjectId(projectId);
    return;
  }
  await ensureProjectData(projectId);
  if (requestId !== selectProjectRequestId) return;
  if (!projectDataLoaded(projectId)) return;
  setSelectedProjectId(projectId);
}

function updateSnapshot(updater: (current: ProjectsSnapshot) => ProjectsSnapshot): void {
  snapshot = updater(snapshot);
}

const actions = createProjectStoreActions({
  selectors,
  readSnapshot: () => snapshot,
  updateSnapshot,
  readSelectedProjectId: () => selectedProjectId,
  setSelectedProjectId,
  reload,
  ensureProjectData,
});

export function getProjects() {
  const {
    projectById,
    groupById,
    projectsForGroup,
    projectsForGroupIncludingInactive,
    visibleGroups,
    sectionsForProject,
    sectionsForProjectIncludingInactive,
    statusesForProject,
    prioritiesForProject,
    tasksForProject,
    tasksForProjectIncludingArchived,
    topLevelTasksForSection,
    topLevelTasksForStatus,
    subtasksForTask,
    subtasksForTaskIncludingArchived,
    defaultSection,
    defaultStatus,
    statusById,
    taskById,
    eventLinksForTask,
    eventLinksForEvent,
    taskChangeEventsForTask,
    recentTaskChangeEventsForProject,
    checklistItemsForTask,
    tagsForProject,
    tagById,
    taskTagLinksForTask,
    tagsForTask,
    unlinkedTagsForTask,
    customFieldsForProject,
    customFieldById,
    customFieldOptionsForField,
    customFieldValueForTask,
    customFieldOptionValuesForTask,
    dependenciesBlockingTask,
    dependenciesBlockedByTask,
    savedTaskViewsForProject,
  } = selectors;

  return {
    get snapshot(): ProjectsSnapshot {
      return snapshot;
    },
    get groups(): ProjectGroup[] {
      return snapshot.groups;
    },
    get projects(): Project[] {
      return snapshot.projects;
    },
    get sections(): ProjectSection[] {
      return snapshot.sections;
    },
    get statuses(): ProjectStatus[] {
      return snapshot.statuses;
    },
    get priorities(): ProjectPriorityConfig[] {
      return snapshot.priorities;
    },
    get tasks(): ProjectTask[] {
      return snapshot.tasks;
    },
    get checklistItems(): ProjectChecklistItem[] {
      return snapshot.checklistItems;
    },
    get tags(): ProjectTag[] {
      return snapshot.tags;
    },
    get taskTagLinks(): ProjectTaskTagLink[] {
      return snapshot.taskTagLinks;
    },
    get customFields(): ProjectCustomField[] {
      return snapshot.customFields;
    },
    get customFieldOptions(): ProjectCustomFieldOption[] {
      return snapshot.customFieldOptions;
    },
    get customFieldValues(): ProjectCustomFieldValue[] {
      return snapshot.customFieldValues;
    },
    get customFieldOptionValues(): ProjectCustomFieldOptionValue[] {
      return snapshot.customFieldOptionValues;
    },
    get dependencies(): ProjectTaskDependency[] {
      return snapshot.dependencies;
    },
    get eventLinks(): ProjectTaskEventLink[] {
      return snapshot.eventLinks;
    },
    get taskChangeEvents(): ProjectTaskChangeEvent[] {
      return snapshot.taskChangeEvents;
    },
    get viewPreferences(): ProjectViewPreference[] {
      return snapshot.viewPreferences;
    },
    get customEmojis(): ProjectCustomEmoji[] {
      return snapshot.customEmojis;
    },
    get selectedProjectId(): string | null {
      ensureSelectedProject();
      return selectedProjectId;
    },
    set selectedProjectId(projectId: string | null) {
      setSelectedProjectId(projectId);
    },
    get selectedProject(): Project | undefined {
      ensureSelectedProject();
      return selectors.projectById(selectedProjectId);
    },
    get selectedGroup(): ProjectGroup | undefined {
      const selectedProject = selectors.projectById(selectedProjectId);
      return selectors.groupById(selectedProject?.groupId);
    },
    get activeView(): ProjectViewId {
      return activeView;
    },
    set activeView(view: ProjectViewId) {
      setActiveView(view);
    },
    get loaded(): boolean {
      return loaded;
    },
    get loading(): boolean {
      return loading;
    },
    get loadError(): string | null {
      return loadError;
    },
    load: reload,
    ensureLoaded,
    ensureProjectData,
    projectDataLoaded,
    selectProject,
    projectById,
    groupById,
    projectsForGroup,
    projectsForGroupIncludingInactive,
    visibleGroups,
    sectionsForProject,
    sectionsForProjectIncludingInactive,
    statusesForProject,
    prioritiesForProject,
    tasksForProject,
    tasksForProjectIncludingArchived,
    topLevelTasksForSection,
    topLevelTasksForStatus,
    subtasksForTask,
    subtasksForTaskIncludingArchived,
    defaultSection,
    defaultStatus,
    statusById,
    taskById,
    eventLinksForTask,
    eventLinksForEvent,
    taskChangeEventsForTask,
    recentTaskChangeEventsForProject,
    checklistItemsForTask,
    tagsForProject,
    tagById,
    taskTagLinksForTask,
    tagsForTask,
    unlinkedTagsForTask,
    customFieldsForProject,
    customFieldById,
    customFieldOptionsForField,
    customFieldValueForTask,
    customFieldOptionValuesForTask,
    dependenciesBlockingTask,
    dependenciesBlockedByTask,
    savedTaskViewsForProject,
    ...actions,
  };
}
