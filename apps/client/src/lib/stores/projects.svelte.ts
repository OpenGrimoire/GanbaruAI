import {
  loadProjectsOptionalData,
  loadProjectsWorkspace,
  refreshProjectsWorkspace,
} from "$lib/api/projects";
import {
  mergeProjectOptionalData,
  PROJECT_SCOPED_OPTIONAL_DATA_KINDS,
  projectOptionalDataKey,
  projectViewOptionalDataKinds,
} from "$lib/projects/project-optional-data";
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
  ProjectOptionalDataKind,
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
let initialLoadPromise: Promise<void> | null = null;
const optionalDataLoadedKeys = new Set<string>();
const optionalDataRequests = new Map<string, Promise<void>>();
let optionalDataVersion = $state(0);
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
  snapshot = mergeProjectSnapshot(
    snapshot,
    {
      ...incoming,
      viewPreferences: snapshot.viewPreferences,
      customEmojis: snapshot.customEmojis,
    },
    projectId,
  );
  if (incoming.projects.some((project) => project.id === projectId)) {
    markProjectDataLoaded(projectId);
  }
}

async function reload(projectId: string | null = selectedProjectId): Promise<void> {
  const requestId = ++loadRequestId;
  const initialLoad = !loaded;
  const customEmojiKey = projectOptionalDataKey("custom_emojis", null);
  const refreshCustomEmojis = optionalDataLoadedKeys.has(customEmojiKey)
    || optionalDataRequests.has(customEmojiKey);
  loading = true;
  loadError = null;
  try {
    let response = initialLoad
      ? await loadProjectsWorkspace(projectId, activeView)
      : await refreshProjectsWorkspace(projectId, activeView);
    if (requestId !== loadRequestId) return;
    if (response.activeView !== activeView) {
      response = await refreshProjectsWorkspace(response.resolvedProjectId, activeView);
      if (requestId !== loadRequestId) return;
    }
    const resolvedProjectId = response.resolvedProjectId;
    const optionalKindsToRefresh = resolvedProjectId
      ? PROJECT_SCOPED_OPTIONAL_DATA_KINDS.filter((kind) => {
          const key = projectOptionalDataKey(kind, resolvedProjectId);
          return optionalDataLoadedKeys.has(key) || optionalDataRequests.has(key);
        })
      : [];
    applyLoadedSnapshot(response.snapshot, resolvedProjectId);
    setSelectedProjectId(resolvedProjectId);
    loaded = true;
    if (resolvedProjectId && optionalKindsToRefresh.length > 0) {
      await Promise.allSettled(
        optionalKindsToRefresh
          .map((kind) => optionalDataRequests.get(projectOptionalDataKey(kind, resolvedProjectId)))
          .filter((request): request is Promise<void> => Boolean(request)),
      );
      if (requestId !== loadRequestId) return;
      for (const kind of optionalKindsToRefresh) {
        if (optionalDataLoadedKeys.delete(projectOptionalDataKey(kind, resolvedProjectId))) {
          optionalDataVersion += 1;
        }
      }
      await Promise.all(optionalKindsToRefresh.map((kind) =>
        ensureOptionalData(kind, resolvedProjectId)
      ));
    }
    if (refreshCustomEmojis) {
      const existing = optionalDataRequests.get(customEmojiKey);
      if (existing) await Promise.allSettled([existing]);
      if (requestId !== loadRequestId) return;
      if (optionalDataLoadedKeys.delete(customEmojiKey)) optionalDataVersion += 1;
      await ensureCustomEmojis();
    }
    if (snapshot.groups.some((group) => group.icon.startsWith("custom-emoji:"))
      || snapshot.projects.some((project) => project.icon.startsWith("custom-emoji:"))) {
      void ensureCustomEmojis().catch((error) => {
        console.error("load project custom emoji metadata failed", error);
      });
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
  if (loaded) return;
  if (!initialLoadPromise) {
    initialLoadPromise = reload().finally(() => {
      initialLoadPromise = null;
    });
  }
  await initialLoadPromise;
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

async function ensureOptionalData(
  kind: ProjectOptionalDataKind,
  projectId: string | null,
): Promise<void> {
  if (kind !== "custom_emojis" && !projectId) return;
  const key = projectOptionalDataKey(kind, projectId);
  if (optionalDataLoadedKeys.has(key)) return;
  const existing = optionalDataRequests.get(key);
  if (existing) return existing;
  const requestGeneration = loadRequestId;
  const request = loadProjectsOptionalData(kind, projectId)
    .then((incoming) => {
      if (requestGeneration !== loadRequestId) return;
      if (incoming.kind !== kind || incoming.projectId !== projectId) return;
      snapshot = mergeProjectOptionalData(snapshot, incoming);
      optionalDataLoadedKeys.add(key);
      optionalDataVersion += 1;
    })
    .finally(() => {
      if (optionalDataRequests.get(key) === request) optionalDataRequests.delete(key);
    });
  optionalDataRequests.set(key, request);
  return request;
}

async function ensureProjectViewData(
  projectId: string | null | undefined,
  view: ProjectViewId,
): Promise<void> {
  if (!projectId) return;
  await Promise.all(projectViewOptionalDataKinds(view).map((kind) =>
    ensureOptionalData(kind, projectId)
  ));
}

async function ensureProjectToolbarData(projectId: string | null | undefined): Promise<void> {
  if (!projectId) return;
  await Promise.all([
    ensureOptionalData("relationships", projectId),
    ensureOptionalData("custom_fields", projectId),
    ensureOptionalData("saved_views", projectId),
  ]);
}

async function ensureTaskDetailData(projectId: string | null | undefined): Promise<void> {
  if (!projectId) return;
  await Promise.all([
    ensureOptionalData("relationships", projectId),
    ensureOptionalData("custom_fields", projectId),
    ensureOptionalData("history", projectId),
    ensureOptionalData("checklist", projectId),
  ]);
}

async function ensureCustomEmojis(): Promise<void> {
  await ensureOptionalData("custom_emojis", null);
}

function projectOptionalDataLoaded(
  kind: ProjectOptionalDataKind,
  projectId: string | null,
): boolean {
  void optionalDataVersion;
  return optionalDataLoadedKeys.has(projectOptionalDataKey(kind, projectId));
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
    ensureProjectViewData,
    ensureProjectToolbarData,
    ensureTaskDetailData,
    ensureCustomEmojis,
    projectOptionalDataLoaded,
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
