<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import { FALLBACK_COLOR_INDEX, type EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import {
    PROJECT_POMODORO_PRESET_ORDER,
  } from "$lib/projects/project-default-pomodoro";
  import {
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    emptyProjectSettingsProjectDraft,
    projectSettingsProjectDraftDirty,
    projectSettingsProjectDraftFromProject,
    projectSettingsProjectUpdateFromDraft,
    type ProjectSettingsProjectDraft,
    type ProjectSettingsProjectDraftError,
  } from "$lib/projects/project-settings-project-draft";
  import {
    projectSettingsPriorityColorDraftValue,
    projectSettingsPriorityDraftDirty,
    projectSettingsPriorityNameDraftValue,
    projectSettingsPrioritySaveDrafts,
    projectSettingsStatusCategoryDraftValue,
    projectSettingsStatusColorDraftValue,
    projectSettingsStatusDraftDirty,
    projectSettingsStatusNameDraftValue,
    projectSettingsStatusSaveDrafts,
    projectSettingsTagColorDraftValue,
    projectSettingsTagDraftDirty,
    projectSettingsTagNameDraftValue,
    projectSettingsTagNameExists,
    projectSettingsTagSaveDrafts,
    type ProjectSettingsPriorityDraftState,
    type ProjectSettingsPrioritySaveDraft,
    type ProjectSettingsStatusDraftState,
    type ProjectSettingsStatusSaveDraft,
    type ProjectSettingsTagDraftState,
    type ProjectSettingsTagSaveDraft,
  } from "$lib/projects/project-settings-collection-drafts";
  import { projectCustomFieldUsesOptions } from "$lib/projects/custom-fields";
  import {
    PROJECT_LIFECYCLE_STATUSES,
    PROJECT_TAG_DEFAULT_COLOR,
  } from "$lib/projects/types";
  import type {
    CustomFieldOptionSaveDraft,
    CustomFieldSaveDraft,
    NewCustomFieldDraft,
    NewCustomFieldOptionDraft,
    ProjectSettingsCustomFieldDraftError,
  } from "$lib/projects/project-settings-custom-field-drafts";
  import {
    projectSettingsCustomFieldDraftDirty,
    projectSettingsCustomFieldNameDraftValue,
    projectSettingsCustomFieldOptionCreateDraftRows,
    projectSettingsCustomFieldOptionDraftDirty,
    projectSettingsCustomFieldOptionNameDraftValue,
    projectSettingsCustomFieldOptionNamesForCreate,
    projectSettingsCustomFieldOptionSaveDrafts,
    projectSettingsCustomFieldSaveDrafts,
    projectSettingsRemoveCustomFieldCreateDraft,
    projectSettingsRemoveCustomFieldCreateDraftOption,
    projectSettingsRemoveCustomFieldOptionCreateDraft,
    projectSettingsSetCustomFieldCreateDraftName,
    projectSettingsSetCustomFieldCreateDraftOptionName,
    projectSettingsSetCustomFieldCreateDraftPendingOptionName,
    projectSettingsSetCustomFieldOptionCreateDraftName,
  } from "$lib/projects/project-settings-custom-field-drafts";
  import {
    projectSettingsDraggedEntryDropTarget,
    projectSettingsDragOverPlan,
    moveProjectSettingsEntryToIndex,
    projectSettingsDropMarkerVisible,
    projectSettingsDropPosition,
    projectSettingsStartDrag,
    type ProjectSettingsDropPosition,
  } from "$lib/projects/project-settings-reorder";
  import {
    nextProjectSettingsPaletteColor,
    nextUnusedProjectSettingsColor,
    scrollProjectSettingsRowIntoView,
  } from "$lib/projects/project-settings-ui";
  import type {
    Project,
    ProjectCustomField,
    ProjectCustomFieldOption,
    ProjectCustomFieldType,
    ProjectTag,
    ProjectLifecycleStatus,
    ProjectPriorityConfig,
    ProjectStatus,
    ProjectStatusCategory,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import ProjectSettingsCustomFieldsSection from "./ProjectSettingsCustomFieldsSection.svelte";
  import ProjectSettingsDefaultsSection from "./ProjectSettingsDefaultsSection.svelte";
  import ProjectSettingsDeleteDialogs from "./ProjectSettingsDeleteDialogs.svelte";
  import ProjectSettingsIdentitySection from "./ProjectSettingsIdentitySection.svelte";
  import { projectHasLockedSystemIdentity } from "$lib/projects/project-system-defaults";
  import ProjectSettingsPanelShell from "./ProjectSettingsPanelShell.svelte";
  import ProjectSettingsPrioritiesSection from "./ProjectSettingsPrioritiesSection.svelte";
  import ProjectSettingsStatusesSection from "./ProjectSettingsStatusesSection.svelte";
  import ProjectSettingsTagsSection from "./ProjectSettingsTagsSection.svelte";

  let {
    projectId,
    presentation = "side",
    onClose,
    onRevealInactive,
    onDirtyChange,
  }: {
    projectId: string;
    presentation?: "side" | "popover";
    onClose: () => void;
    onRevealInactive: () => void;
    onDirtyChange: (dirty: boolean) => void;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_STATUS_CATEGORIES: ProjectStatusCategory[] = ["not_started", "active", "blocked", "done"];
  const NEW_STATUS_FIRST_COLOR: EventColor = 8;
  const NEW_PRIORITY_FIRST_COLOR: EventColor = 13;
  const NEW_TAG_FIRST_COLOR = PROJECT_TAG_DEFAULT_COLOR;
  const PROJECT_STATUS_DRAG_DATA_TYPE = "application/x-ganbaru-project-status";
  const PROJECT_PRIORITY_DRAG_DATA_TYPE = "application/x-ganbaru-project-priority";
  const PROJECT_TAG_DRAG_DATA_TYPE = "application/x-ganbaru-project-tag";
  const PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE = "application/x-ganbaru-project-custom-field";
  const PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE = "application/x-ganbaru-project-custom-field-option";
  type SelectOption = { value: string; label: string };
  type StatusDropPosition = ProjectSettingsDropPosition;
  type PriorityDropPosition = ProjectSettingsDropPosition;
  type TagDropPosition = ProjectSettingsDropPosition;
  type CustomFieldDropPosition = ProjectSettingsDropPosition;
  type CustomFieldOptionDropPosition = ProjectSettingsDropPosition;
  type ProjectSettingsTargetSetter = (
    overId: string | null,
    position: ProjectSettingsDropPosition | null,
  ) => void;
  type ProjectSettingsPendingSetter = (pending: boolean) => void;

  let projectDraftId = $state<string | null>(null);
  let projectDraftUpdatedAt = $state<string | null>(null);
  let projectDraft = $state<ProjectSettingsProjectDraft>(emptyProjectSettingsProjectDraft());
  let projectSettingsSaving = $state(false);
  let projectSettingsError = $state<string | null>(null);
  let statusNameDrafts = $state<Record<string, string>>({});
  let statusCategoryDrafts = $state<Record<string, ProjectStatusCategory>>({});
  let statusColorDrafts = $state<Record<string, EventColor>>({});
  let priorityNameDrafts = $state<Record<string, string>>({});
  let priorityColorDrafts = $state<Record<string, EventColor>>({});
  let newStatusName = $state("");
  let newStatusCategory = $state<ProjectStatusCategory>("active");
  let newStatusColor = $state<EventColor>(NEW_STATUS_FIRST_COLOR);
  let newPriorityName = $state("");
  let newPriorityColor = $state<EventColor>(NEW_PRIORITY_FIRST_COLOR);
  let pendingDeleteStatusId = $state<string | null>(null);
  let pendingDeletePriorityId = $state<string | null>(null);
  let tagNameDrafts = $state<Record<string, string>>({});
  let tagColorDrafts = $state<Record<string, EventColor>>({});
  let newTagName = $state("");
  let newTagColor = $state<EventColor>(NEW_TAG_FIRST_COLOR);
  let pendingDeleteTagId = $state<string | null>(null);
  let customFieldNameDrafts = $state<Record<string, string>>({});
  let customFieldOptionNameDrafts = $state<Record<string, string>>({});
  let customFieldOptionDraftRowsByField = $state<Record<string, NewCustomFieldOptionDraft[]>>({});
  let customFieldCreateDraftRows = $state<NewCustomFieldDraft[]>([]);
  let newCustomFieldName = $state("");
  let newCustomFieldType = $state<ProjectCustomFieldType>("text");
  let newCustomFieldOptionDrafts = $state<Record<string, string>>({});
  let newCustomFieldOptionRows = $state<NewCustomFieldOptionDraft[]>([]);
  let newCustomFieldOptionName = $state("");
  let pendingDeleteCustomFieldId = $state<string | null>(null);
  let pendingDeleteCustomFieldOptionId = $state<string | null>(null);
  let settingsScrollElement = $state<HTMLElement | undefined>();
  let newStatusRowElement = $state<HTMLDivElement | undefined>();
  let newPriorityRowElement = $state<HTMLDivElement | undefined>();
  let newTagRowElement = $state<HTMLDivElement | undefined>();
  let newCustomFieldRowElement = $state<HTMLDivElement | undefined>();
  let draggedStatusId = $state<string | null>(null);
  let dragOverStatusId = $state<string | null>(null);
  let statusDropPosition = $state<StatusDropPosition | null>(null);
  let statusReorderPending = $state(false);
  let draggedPriorityId = $state<string | null>(null);
  let dragOverPriorityId = $state<string | null>(null);
  let priorityDropPosition = $state<PriorityDropPosition | null>(null);
  let priorityReorderPending = $state(false);
  let draggedTagId = $state<string | null>(null);
  let dragOverTagId = $state<string | null>(null);
  let tagDropPosition = $state<TagDropPosition | null>(null);
  let tagReorderPending = $state(false);
  let draggedCustomFieldId = $state<string | null>(null);
  let dragOverCustomFieldId = $state<string | null>(null);
  let customFieldDropPosition = $state<CustomFieldDropPosition | null>(null);
  let customFieldReorderPending = $state(false);
  let draggedCustomFieldOptionId = $state<string | null>(null);
  let dragOverCustomFieldOptionId = $state<string | null>(null);
  let customFieldOptionDropPosition = $state<CustomFieldOptionDropPosition | null>(null);
  let customFieldOptionReorderPending = $state(false);

  const selectedProject = $derived(projects.projectById(projectId));
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const selectedProjectIdentityLocked = $derived(
    selectedProject ? projectHasLockedSystemIdentity(selectedProject) : false,
  );
  const visibleProjectGroups = $derived.by(() => projects.visibleGroups());
  const statuses = $derived(projects.statusesForProject(selectedProjectId));
  const priorities = $derived(projects.prioritiesForProject(selectedProjectId));
  const projectTags = $derived(projects.tagsForProject(selectedProjectId));
  const projectCustomFields = $derived(projects.customFieldsForProject(selectedProjectId));
  const projectGroupOptions = $derived<SelectOption[]>(
    visibleProjectGroups.map((group) => ({ value: group.id, label: group.name })),
  );
  const lifecycleOptions = $derived<SelectOption[]>(
    PROJECT_LIFECYCLE_STATUSES.map((status) => ({
      value: status,
      label: projectLifecycleLabel(status, t),
    })),
  );
  const statusCategoryOptions = $derived<SelectOption[]>(
    PROJECT_STATUS_CATEGORIES.map((category) => ({
      value: category,
      label: statusCategoryLabel(category),
    })),
  );
  const pendingDeleteTag = $derived.by(() =>
    pendingDeleteTagId ? projectTags.find((tag) => tag.id === pendingDeleteTagId) : undefined
  );
  const pendingDeleteCustomField = $derived.by(() =>
    pendingDeleteCustomFieldId
      ? projectCustomFields.find((field) => field.id === pendingDeleteCustomFieldId)
      : undefined
  );
  const pendingDeleteCustomFieldOption = $derived.by(() => {
    if (!pendingDeleteCustomFieldOptionId) return undefined;
    return projectCustomFields
      .flatMap((field) => projects.customFieldOptionsForField(field.id))
      .find((entry) => entry.id === pendingDeleteCustomFieldOptionId);
  });
  const pendingDeleteStatus = $derived.by(() =>
    pendingDeleteStatusId ? statuses.find((status) => status.id === pendingDeleteStatusId) : undefined
  );
  const pendingDeletePriority = $derived.by(() =>
    pendingDeletePriorityId ? priorities.find((priority) => priority.id === pendingDeletePriorityId) : undefined
  );
  const projectSettingsDraftReady = $derived(
    Boolean(selectedProject && projectDraftId === selectedProject.id),
  );
  const projectFieldSettingsDirty = $derived.by(() => {
    if (!selectedProject) return false;
    return projectSettingsProjectDraftDirty(selectedProject, projectDraft);
  });
  const statusSettingsDirty = $derived.by(() => statuses.some(statusDraftDirty));
  const prioritySettingsDirty = $derived.by(() => priorities.some(priorityDraftDirty));
  const tagSettingsDirty = $derived.by(() => projectTags.some(tagDraftDirty));
  const customFieldSettingsDirty = $derived.by(() =>
    projectCustomFields.some(customFieldDraftDirty)
      || projectCustomFields.some((field) => customFieldOptions(field).some(customFieldOptionDraftDirty))
      || projectCustomFields.some(customFieldOptionCreateDraftDirty)
      || customFieldCreateDraftDirty()
  );
  const projectSettingsDirty = $derived(
    projectFieldSettingsDirty
      || statusSettingsDirty
      || prioritySettingsDirty
      || tagSettingsDirty
      || customFieldSettingsDirty,
  );

  $effect(() => {
    if (!selectedProject) return;
    if (
      projectDraftId !== selectedProject.id
      || (!projectSettingsDirty && projectDraftUpdatedAt !== selectedProject.updatedAt)
    ) {
      loadProjectSettingsDraft(selectedProject);
    }
  });

  $effect(() => {
    if (!selectedProject || !selectedProjectIdentityLocked) return;
    if (projectDraftId !== selectedProject.id || projectDraft.name === selectedProject.name) return;
    projectDraft.name = selectedProject.name;
    projectDraft.groupId = selectedProject.groupId;
  });

  $effect(() => {
    onDirtyChange(projectSettingsDirty);
  });

  onDestroy(() => {
    onDirtyChange(false);
  });

  function loadProjectSettingsDraft(project: Project): void {
    projectDraftId = project.id;
    projectDraftUpdatedAt = project.updatedAt;
    projectDraft = projectSettingsProjectDraftFromProject(project);
    projectSettingsError = null;
    statusNameDrafts = Object.fromEntries(statuses.map((status) => [status.id, status.name]));
    statusCategoryDrafts = Object.fromEntries(
      statuses.map((status) => [status.id, status.category]),
    );
    statusColorDrafts = Object.fromEntries(statuses.map((status) => [status.id, status.color]));
    priorityNameDrafts = Object.fromEntries(priorities.map((priority) => [priority.id, priority.name]));
    priorityColorDrafts = Object.fromEntries(priorities.map((priority) => [priority.id, priority.color]));
    tagNameDrafts = Object.fromEntries(projectTags.map((tag) => [tag.id, tag.name]));
    const nextTagColorDrafts: Record<string, EventColor> = {};
    for (const tag of projectTags) {
      nextTagColorDrafts[tag.id] = tag.color ?? FALLBACK_COLOR_INDEX;
    }
    tagColorDrafts = nextTagColorDrafts;
    customFieldNameDrafts = Object.fromEntries(projectCustomFields.map((field) => [field.id, field.name]));
    customFieldOptionNameDrafts = Object.fromEntries(
      projectCustomFields.flatMap((field) =>
        projects.customFieldOptionsForField(field.id).map((option) => [option.id, option.name]),
      ),
    );
    customFieldOptionDraftRowsByField = {};
    customFieldCreateDraftRows = [];
    newStatusName = "";
    newStatusCategory = "active";
    newStatusColor = nextUnusedStatusColor(NEW_STATUS_FIRST_COLOR);
    pendingDeleteStatusId = null;
    newPriorityName = "";
    newPriorityColor = nextUnusedPriorityColor(NEW_PRIORITY_FIRST_COLOR);
    pendingDeletePriorityId = null;
    newTagName = "";
    newTagColor = nextUnusedTagColor(NEW_TAG_FIRST_COLOR);
    pendingDeleteTagId = null;
    newCustomFieldName = "";
    newCustomFieldType = "text";
    newCustomFieldOptionDrafts = {};
    newCustomFieldOptionRows = [];
    newCustomFieldOptionName = "";
    pendingDeleteCustomFieldId = null;
    pendingDeleteCustomFieldOptionId = null;
    clearCustomFieldDrag();
    clearCustomFieldOptionDrag();
  }

  function closeProjectSettings(): void {
    onClose();
  }

  function discardProjectSettings(): void {
    if (selectedProject) loadProjectSettingsDraft(selectedProject);
  }

  function nextProjectSortOrderForGroup(groupId: string, excludeProjectId: string): number {
    return Math.max(
      0,
      ...projects.projectsForGroupIncludingInactive(groupId)
        .filter((project) => project.id !== excludeProjectId)
        .map((project) => project.sortOrder),
    ) + 1000;
  }

  function statusCategoryLabel(category: ProjectStatusCategory): string {
    if (category === "active") return t("projects.statusCategory.active");
    if (category === "blocked") return t("projects.statusCategory.blocked");
    if (category === "done") return t("projects.statusCategory.done");
    return t("projects.statusCategory.notStarted");
  }

  function setLifecycleStatus(value: string): void {
    if (PROJECT_LIFECYCLE_STATUSES.includes(value as ProjectLifecycleStatus)) {
      projectDraft.status = value as ProjectLifecycleStatus;
    }
  }

  function setStatusCategory(statusId: string, value: string): void {
    if (!PROJECT_STATUS_CATEGORIES.includes(value as ProjectStatusCategory)) return;
    statusCategoryDrafts = {
      ...statusCategoryDrafts,
      [statusId]: value as ProjectStatusCategory,
    };
  }

  function setNewStatusCategory(value: string): void {
    if (PROJECT_STATUS_CATEGORIES.includes(value as ProjectStatusCategory)) {
      newStatusCategory = value as ProjectStatusCategory;
    }
  }

  function pomodoroPresetLabel(preset: PomodoroPresetKey): string {
    if (preset === "creative") return t("projects.pomodoro.creative");
    if (preset === "balanced") return t("projects.pomodoro.balanced");
    if (preset === "deep") return t("projects.pomodoro.deep");
    if (preset === "extended") return t("projects.pomodoro.extended");
    return t("projects.pomodoro.adaptive");
  }

  function statusDraftState(): ProjectSettingsStatusDraftState {
    return {
      nameDrafts: statusNameDrafts,
      categoryDrafts: statusCategoryDrafts,
      colorDrafts: statusColorDrafts,
      fallbackColor: FALLBACK_COLOR_INDEX,
    };
  }

  function priorityDraftState(): ProjectSettingsPriorityDraftState {
    return {
      nameDrafts: priorityNameDrafts,
      colorDrafts: priorityColorDrafts,
      fallbackColor: FALLBACK_COLOR_INDEX,
    };
  }

  function tagDraftState(): ProjectSettingsTagDraftState {
    return {
      nameDrafts: tagNameDrafts,
      colorDrafts: tagColorDrafts,
      fallbackColor: FALLBACK_COLOR_INDEX,
    };
  }

  function customFieldAcceptsOptions(field: ProjectCustomField): boolean {
    return projectCustomFieldUsesOptions(field.fieldType);
  }

  function newCustomFieldAcceptsOptions(): boolean {
    return projectCustomFieldUsesOptions(newCustomFieldType);
  }

  function customFieldCreateDraftAcceptsOptions(field: NewCustomFieldDraft): boolean {
    return projectCustomFieldUsesOptions(field.fieldType);
  }

  function customFieldCreateDraftDirty(): boolean {
    return customFieldCreateDraftRows.length > 0;
  }

  function customFieldNameDraftValue(field: ProjectCustomField): string {
    return projectSettingsCustomFieldNameDraftValue(field, customFieldNameDrafts);
  }

  function customFieldNameExists(name: string, ignoredFieldId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectCustomFields.some((field) =>
      field.id !== ignoredFieldId && customFieldNameDraftValue(field).trim().toLowerCase() === normalized
    );
  }

  function customFieldCreateDraftNameExists(name: string, ignoredDraftId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    if (customFieldNameExists(name)) return true;
    return customFieldCreateDraftRows.some((field) =>
      field.id !== ignoredDraftId && field.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldDraftDirty(field: ProjectCustomField): boolean {
    return projectSettingsCustomFieldDraftDirty(field, customFieldNameDrafts);
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldOptionById(optionId: string | null | undefined): ProjectCustomFieldOption | undefined {
    if (!optionId) return undefined;
    return projects.customFieldOptions.find((option) => option.id === optionId);
  }

  function customFieldOptionNameDraftValue(option: ProjectCustomFieldOption): string {
    return projectSettingsCustomFieldOptionNameDraftValue(option, customFieldOptionNameDrafts);
  }

  function customFieldOptionNameExists(fieldId: string, name: string, ignoredOptionId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projects.customFieldOptionsForField(fieldId).some((option) =>
      option.id !== ignoredOptionId && customFieldOptionNameDraftValue(option).trim().toLowerCase() === normalized
    );
  }

  function customFieldOptionDraftDirty(option: ProjectCustomFieldOption): boolean {
    return projectSettingsCustomFieldOptionDraftDirty(option, customFieldOptionNameDrafts);
  }

  function customFieldOptionCreateDraftRows(fieldId: string): NewCustomFieldOptionDraft[] {
    return projectSettingsCustomFieldOptionCreateDraftRows(customFieldOptionDraftRowsByField, fieldId);
  }

  function customFieldOptionCreateDraftDirty(field: ProjectCustomField): boolean {
    return customFieldOptionCreateDraftRows(field.id).length > 0;
  }

  function customFieldOptionCreateDraftNameExists(
    fieldId: string,
    name: string,
    ignoredDraftId?: string,
  ): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    if (customFieldOptionNameExists(fieldId, name)) return true;
    return customFieldOptionCreateDraftRows(fieldId).some((option) =>
      option.id !== ignoredDraftId && option.name.trim().toLowerCase() === normalized
    );
  }

  function setCustomFieldOptionCreateDraftName(fieldId: string, optionId: string, name: string): void {
    customFieldOptionDraftRowsByField = projectSettingsSetCustomFieldOptionCreateDraftName(
      customFieldOptionDraftRowsByField,
      fieldId,
      optionId,
      name,
    );
  }

  function removeCustomFieldOptionCreateDraft(fieldId: string, optionId: string): void {
    customFieldOptionDraftRowsByField = projectSettingsRemoveCustomFieldOptionCreateDraft(
      customFieldOptionDraftRowsByField,
      fieldId,
      optionId,
    );
  }

  function fieldForCustomFieldOption(option: ProjectCustomFieldOption | undefined): ProjectCustomField | undefined {
    return option ? projectCustomFields.find((field) => field.id === option.fieldId) : undefined;
  }

  function newCustomFieldOptionDraftNameExists(name: string, ignoredDraftId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return newCustomFieldOptionRows.some((option) =>
      option.id !== ignoredDraftId && option.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldCreateDraftOptionNameExists(
    field: NewCustomFieldDraft,
    name: string,
    ignoredDraftId?: string,
  ): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return field.optionRows.some((option) =>
      option.id !== ignoredDraftId && option.name.trim().toLowerCase() === normalized
    );
  }

  function setCustomFieldCreateDraftName(fieldId: string, name: string): void {
    customFieldCreateDraftRows = projectSettingsSetCustomFieldCreateDraftName(
      customFieldCreateDraftRows,
      fieldId,
      name,
    );
  }

  function setCustomFieldCreateDraftOptionName(fieldId: string, optionId: string, name: string): void {
    customFieldCreateDraftRows = projectSettingsSetCustomFieldCreateDraftOptionName(
      customFieldCreateDraftRows,
      fieldId,
      optionId,
      name,
    );
  }

  function setCustomFieldCreateDraftPendingOptionName(fieldId: string, optionName: string): void {
    customFieldCreateDraftRows = projectSettingsSetCustomFieldCreateDraftPendingOptionName(
      customFieldCreateDraftRows,
      fieldId,
      optionName,
    );
  }

  function removeCustomFieldCreateDraft(fieldId: string): void {
    customFieldCreateDraftRows = projectSettingsRemoveCustomFieldCreateDraft(
      customFieldCreateDraftRows,
      fieldId,
    );
  }

  function removeCustomFieldCreateDraftOption(fieldId: string, optionId: string): void {
    customFieldCreateDraftRows = projectSettingsRemoveCustomFieldCreateDraftOption(
      customFieldCreateDraftRows,
      fieldId,
      optionId,
    );
  }

  function usedStatusColors(extraColor?: EventColor): Set<EventColor> {
    const used = new Set<EventColor>();
    for (const status of statuses) {
      used.add(statusColorDraftValue(status));
    }
    if (extraColor !== undefined) used.add(extraColor);
    return used;
  }

  function usedPriorityColors(extraColor?: EventColor): Set<EventColor> {
    const used = new Set<EventColor>();
    for (const priority of priorities) {
      used.add(priorityColorDraftValue(priority));
    }
    if (extraColor !== undefined) used.add(extraColor);
    return used;
  }

  function usedTagColors(extraColor?: EventColor): Set<EventColor> {
    const used = new Set<EventColor>();
    for (const tag of projectTags) {
      used.add(tagColorDraftValue(tag));
    }
    if (extraColor !== undefined) used.add(extraColor);
    return used;
  }

  function nextUnusedStatusColor(preferredColor: EventColor, extraColor?: EventColor): EventColor {
    return nextUnusedProjectSettingsColor({
      preferredColor,
      usedColors: usedStatusColors(extraColor),
      fallbackColor: NEW_STATUS_FIRST_COLOR,
    });
  }

  function nextUnusedPriorityColor(preferredColor: EventColor, extraColor?: EventColor): EventColor {
    return nextUnusedProjectSettingsColor({
      preferredColor,
      usedColors: usedPriorityColors(extraColor),
      fallbackColor: NEW_STATUS_FIRST_COLOR,
    });
  }

  function nextUnusedTagColor(preferredColor: EventColor, extraColor?: EventColor): EventColor {
    return nextUnusedProjectSettingsColor({
      preferredColor,
      usedColors: usedTagColors(extraColor),
      fallbackColor: NEW_STATUS_FIRST_COLOR,
    });
  }

  async function scrollToSettingsRow(rowElement: HTMLElement | undefined): Promise<void> {
    await tick();
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve());
    });
    scrollProjectSettingsRowIntoView(settingsScrollElement, rowElement);
  }

  async function scrollToNewStatusRow(): Promise<void> {
    await scrollToSettingsRow(newStatusRowElement);
  }

  async function scrollToNewPriorityRow(): Promise<void> {
    await scrollToSettingsRow(newPriorityRowElement);
  }

  async function scrollToNewTagRow(): Promise<void> {
    await scrollToSettingsRow(newTagRowElement);
  }

  async function scrollToNewCustomFieldRow(): Promise<void> {
    await scrollToSettingsRow(newCustomFieldRowElement);
  }

  function tagNameDraftValue(tag: ProjectTag): string {
    return projectSettingsTagNameDraftValue(tag, tagDraftState());
  }

  function tagColorDraftValue(tag: ProjectTag): EventColor {
    return projectSettingsTagColorDraftValue(tag, tagDraftState());
  }

  function tagDraftDirty(tag: ProjectTag): boolean {
    return projectSettingsTagDraftDirty(tag, tagDraftState());
  }

  function tagNameExists(name: string, ignoredTagId?: string): boolean {
    return projectSettingsTagNameExists({
      tags: projectTags,
      name,
      ignoredTagId,
    });
  }

  function clearStatusDrag(): void {
    draggedStatusId = null;
    dragOverStatusId = null;
    statusDropPosition = null;
  }

  function clearPriorityDrag(): void {
    draggedPriorityId = null;
    dragOverPriorityId = null;
    priorityDropPosition = null;
  }

  function clearTagDrag(): void {
    draggedTagId = null;
    dragOverTagId = null;
    tagDropPosition = null;
  }

  function clearCustomFieldDrag(): void {
    draggedCustomFieldId = null;
    dragOverCustomFieldId = null;
    customFieldDropPosition = null;
  }

  function clearCustomFieldOptionDrag(): void {
    draggedCustomFieldOptionId = null;
    dragOverCustomFieldOptionId = null;
    customFieldOptionDropPosition = null;
  }

  function dropPositionForEvent(event: DragEvent, target: HTMLElement): ProjectSettingsDropPosition {
    const bounds = target.getBoundingClientRect();
    return projectSettingsDropPosition(event.clientY, bounds.top, bounds.height);
  }

  function handleSettingsDragOver(
    event: DragEvent,
    target: HTMLElement,
    input: {
      draggedId: string | null;
      targetId: string;
      reorderPending: boolean;
      setTarget: ProjectSettingsTargetSetter;
      dropAllowed?: boolean;
    },
  ): void {
    const plan = projectSettingsDragOverPlan({
      draggedId: input.draggedId,
      targetId: input.targetId,
      reorderPending: input.reorderPending,
      position: dropPositionForEvent(event, target),
      dropAllowed: input.dropAllowed,
    });
    if (!plan) return;
    if (plan.preventDefault) {
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    }
    input.setTarget(plan.overId, plan.position);
  }

  async function moveSettingsEntryToIndex<T extends { id: string }>(
    input: {
      entryId: string;
      targetIndex: number;
      getEntries: () => readonly T[];
      moveEntry: (entry: T, direction: -1 | 1) => Promise<void>;
      setPending: ProjectSettingsPendingSetter;
      clearDrag: () => void;
      reorderFailedMessage: string;
      saveFailedMessage: (message: string) => string;
    },
  ): Promise<void> {
    input.setPending(true);
    projectSettingsError = null;
    try {
      const moved = await moveProjectSettingsEntryToIndex({
        entryId: input.entryId,
        targetIndex: input.targetIndex,
        getEntries: input.getEntries,
        moveEntry: input.moveEntry,
      });
      if (!moved) {
        projectSettingsError = input.reorderFailedMessage;
      }
    } catch (error) {
      projectSettingsError = input.saveFailedMessage(error instanceof Error ? error.message : String(error));
    } finally {
      input.setPending(false);
      input.clearDrag();
    }
  }

  async function dropSettingsEntry<T extends { id: string }>(
    event: DragEvent,
    input: {
      dataType: string;
      activeDraggedId: string | null;
      entries: readonly T[];
      targetId: string;
      position: ProjectSettingsDropPosition | null;
      clearDrag: () => void;
      moveToIndex: (entryId: string, targetIndex: number) => Promise<void>;
    },
  ): Promise<void> {
    event.preventDefault();
    const target = projectSettingsDraggedEntryDropTarget({
      event,
      dataType: input.dataType,
      activeDraggedId: input.activeDraggedId,
      entries: input.entries,
      targetId: input.targetId,
      position: input.position ?? "before",
    });
    if (!target) {
      input.clearDrag();
      return;
    }
    await input.moveToIndex(target.entryId, target.targetIndex);
  }

  function statusDropMarkerVisible(statusId: string, position: StatusDropPosition): boolean {
    return projectSettingsDropMarkerVisible({
      draggedId: draggedStatusId,
      targetId: statusId,
      overId: dragOverStatusId,
      currentPosition: statusDropPosition,
      markerPosition: position,
    });
  }

  function priorityDropMarkerVisible(priorityId: string, position: PriorityDropPosition): boolean {
    return projectSettingsDropMarkerVisible({
      draggedId: draggedPriorityId,
      targetId: priorityId,
      overId: dragOverPriorityId,
      currentPosition: priorityDropPosition,
      markerPosition: position,
    });
  }

  function tagDropMarkerVisible(tagId: string, position: TagDropPosition): boolean {
    return projectSettingsDropMarkerVisible({
      draggedId: draggedTagId,
      targetId: tagId,
      overId: dragOverTagId,
      currentPosition: tagDropPosition,
      markerPosition: position,
    });
  }

  function customFieldDropMarkerVisible(fieldId: string, position: CustomFieldDropPosition): boolean {
    return projectSettingsDropMarkerVisible({
      draggedId: draggedCustomFieldId,
      targetId: fieldId,
      overId: dragOverCustomFieldId,
      currentPosition: customFieldDropPosition,
      markerPosition: position,
    });
  }

  function customFieldOptionDropMarkerVisible(
    optionId: string,
    position: CustomFieldOptionDropPosition,
  ): boolean {
    return projectSettingsDropMarkerVisible({
      draggedId: draggedCustomFieldOptionId,
      targetId: optionId,
      overId: dragOverCustomFieldOptionId,
      currentPosition: customFieldOptionDropPosition,
      markerPosition: position,
    });
  }

  function handleStatusDragStart(event: DragEvent, status: ProjectStatus): void {
    draggedStatusId = status.id;
    dragOverStatusId = null;
    statusDropPosition = null;
    projectSettingsStartDrag(event, PROJECT_STATUS_DRAG_DATA_TYPE, status.id);
  }

  function handlePriorityDragStart(event: DragEvent, priority: ProjectPriorityConfig): void {
    draggedPriorityId = priority.id;
    dragOverPriorityId = null;
    priorityDropPosition = null;
    projectSettingsStartDrag(event, PROJECT_PRIORITY_DRAG_DATA_TYPE, priority.id);
  }

  function handleTagDragStart(event: DragEvent, tag: ProjectTag): void {
    draggedTagId = tag.id;
    dragOverTagId = null;
    tagDropPosition = null;
    projectSettingsStartDrag(event, PROJECT_TAG_DRAG_DATA_TYPE, tag.id);
  }

  function handleCustomFieldDragStart(event: DragEvent, field: ProjectCustomField): void {
    draggedCustomFieldId = field.id;
    dragOverCustomFieldId = null;
    customFieldDropPosition = null;
    projectSettingsStartDrag(event, PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE, field.id);
  }

  function handleCustomFieldOptionDragStart(event: DragEvent, option: ProjectCustomFieldOption): void {
    draggedCustomFieldOptionId = option.id;
    dragOverCustomFieldOptionId = null;
    customFieldOptionDropPosition = null;
    projectSettingsStartDrag(event, PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE, option.id);
  }

  function handleStatusDragOver(
    event: DragEvent,
    status: ProjectStatus,
    target: HTMLElement,
  ): void {
    handleSettingsDragOver(event, target, {
      draggedId: draggedStatusId,
      targetId: status.id,
      reorderPending: statusReorderPending,
      setTarget: (overId, position) => {
        dragOverStatusId = overId;
        statusDropPosition = position;
      },
    });
  }

  function handlePriorityDragOver(
    event: DragEvent,
    priority: ProjectPriorityConfig,
    target: HTMLElement,
  ): void {
    handleSettingsDragOver(event, target, {
      draggedId: draggedPriorityId,
      targetId: priority.id,
      reorderPending: priorityReorderPending,
      setTarget: (overId, position) => {
        dragOverPriorityId = overId;
        priorityDropPosition = position;
      },
    });
  }

  function handleTagDragOver(
    event: DragEvent,
    tag: ProjectTag,
    target: HTMLElement,
  ): void {
    handleSettingsDragOver(event, target, {
      draggedId: draggedTagId,
      targetId: tag.id,
      reorderPending: tagReorderPending,
      setTarget: (overId, position) => {
        dragOverTagId = overId;
        tagDropPosition = position;
      },
    });
  }

  function handleCustomFieldDragOver(
    event: DragEvent,
    field: ProjectCustomField,
    target: HTMLElement,
  ): void {
    handleSettingsDragOver(event, target, {
      draggedId: draggedCustomFieldId,
      targetId: field.id,
      reorderPending: customFieldReorderPending,
      setTarget: (overId, position) => {
        dragOverCustomFieldId = overId;
        customFieldDropPosition = position;
      },
    });
  }

  function handleCustomFieldOptionDragOver(
    event: DragEvent,
    option: ProjectCustomFieldOption,
    target: HTMLElement,
  ): void {
    const draggedOption = customFieldOptionById(draggedCustomFieldOptionId);
    handleSettingsDragOver(event, target, {
      draggedId: draggedCustomFieldOptionId,
      targetId: option.id,
      reorderPending: customFieldOptionReorderPending,
      dropAllowed: draggedOption?.fieldId === option.fieldId,
      setTarget: (overId, position) => {
        dragOverCustomFieldOptionId = overId;
        customFieldOptionDropPosition = position;
      },
    });
  }

  async function moveStatusToIndex(statusId: string, targetIndex: number): Promise<void> {
    await moveSettingsEntryToIndex({
      entryId: statusId,
      targetIndex,
      getEntries: () => statuses,
      moveEntry: projects.moveStatus,
      setPending: (pending) => { statusReorderPending = pending; },
      clearDrag: clearStatusDrag,
      reorderFailedMessage: t("projects.settings.statusReorderFailed"),
      saveFailedMessage: (message) => t("projects.settings.statusSaveFailed", message),
    });
  }

  async function movePriorityToIndex(priorityId: string, targetIndex: number): Promise<void> {
    await moveSettingsEntryToIndex({
      entryId: priorityId,
      targetIndex,
      getEntries: () => priorities,
      moveEntry: projects.movePriority,
      setPending: (pending) => { priorityReorderPending = pending; },
      clearDrag: clearPriorityDrag,
      reorderFailedMessage: t("projects.settings.priorityReorderFailed"),
      saveFailedMessage: (message) => t("projects.settings.prioritySaveFailed", message),
    });
  }

  async function moveTagToIndex(tagId: string, targetIndex: number): Promise<void> {
    await moveSettingsEntryToIndex({
      entryId: tagId,
      targetIndex,
      getEntries: () => projectTags,
      moveEntry: projects.moveTag,
      setPending: (pending) => { tagReorderPending = pending; },
      clearDrag: clearTagDrag,
      reorderFailedMessage: t("projects.settings.tagReorderFailed"),
      saveFailedMessage: (message) => t("projects.settings.tagSaveFailed", message),
    });
  }

  async function moveCustomFieldToIndex(fieldId: string, targetIndex: number): Promise<void> {
    await moveSettingsEntryToIndex({
      entryId: fieldId,
      targetIndex,
      getEntries: () => projectCustomFields,
      moveEntry: projects.moveCustomField,
      setPending: (pending) => { customFieldReorderPending = pending; },
      clearDrag: clearCustomFieldDrag,
      reorderFailedMessage: t("projects.customFields.reorderFailed"),
      saveFailedMessage: (message) => t("projects.customFields.saveFailed", message),
    });
  }

  async function moveCustomFieldOptionToIndex(optionId: string, targetIndex: number): Promise<void> {
    const initialOption = customFieldOptionById(optionId);
    if (!initialOption) return;
    const fieldId = initialOption.fieldId;
    await moveSettingsEntryToIndex({
      entryId: optionId,
      targetIndex,
      getEntries: () => projects.customFieldOptionsForField(fieldId),
      moveEntry: projects.moveCustomFieldOption,
      setPending: (pending) => { customFieldOptionReorderPending = pending; },
      clearDrag: clearCustomFieldOptionDrag,
      reorderFailedMessage: t("projects.customFields.optionReorderFailed"),
      saveFailedMessage: (message) => t("projects.customFields.optionSaveFailed", message),
    });
  }

  async function dropStatus(event: DragEvent, targetStatus: ProjectStatus): Promise<void> {
    await dropSettingsEntry(event, {
      dataType: PROJECT_STATUS_DRAG_DATA_TYPE,
      activeDraggedId: draggedStatusId,
      entries: statuses,
      targetId: targetStatus.id,
      position: statusDropPosition ?? "before",
      clearDrag: clearStatusDrag,
      moveToIndex: moveStatusToIndex,
    });
  }

  async function dropPriority(event: DragEvent, targetPriority: ProjectPriorityConfig): Promise<void> {
    await dropSettingsEntry(event, {
      dataType: PROJECT_PRIORITY_DRAG_DATA_TYPE,
      activeDraggedId: draggedPriorityId,
      entries: priorities,
      targetId: targetPriority.id,
      position: priorityDropPosition ?? "before",
      clearDrag: clearPriorityDrag,
      moveToIndex: movePriorityToIndex,
    });
  }

  async function dropTag(event: DragEvent, targetTag: ProjectTag): Promise<void> {
    await dropSettingsEntry(event, {
      dataType: PROJECT_TAG_DRAG_DATA_TYPE,
      activeDraggedId: draggedTagId,
      entries: projectTags,
      targetId: targetTag.id,
      position: tagDropPosition ?? "before",
      clearDrag: clearTagDrag,
      moveToIndex: moveTagToIndex,
    });
  }

  async function dropCustomField(event: DragEvent, targetField: ProjectCustomField): Promise<void> {
    await dropSettingsEntry(event, {
      dataType: PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE,
      activeDraggedId: draggedCustomFieldId,
      entries: projectCustomFields,
      targetId: targetField.id,
      position: customFieldDropPosition ?? "before",
      clearDrag: clearCustomFieldDrag,
      moveToIndex: moveCustomFieldToIndex,
    });
  }

  async function dropCustomFieldOption(
    event: DragEvent,
    targetOption: ProjectCustomFieldOption,
  ): Promise<void> {
    await dropSettingsEntry(event, {
      dataType: PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE,
      activeDraggedId: draggedCustomFieldOptionId,
      entries: projects.customFieldOptionsForField(targetOption.fieldId),
      targetId: targetOption.id,
      position: customFieldOptionDropPosition ?? "before",
      clearDrag: clearCustomFieldOptionDrag,
      moveToIndex: moveCustomFieldOptionToIndex,
    });
  }

  function statusDraftDirty(status: ProjectStatus): boolean {
    return projectSettingsStatusDraftDirty(status, statusDraftState());
  }

  function statusNameDraftValue(status: ProjectStatus): string {
    return projectSettingsStatusNameDraftValue(status, statusDraftState());
  }

  function statusCategoryDraftValue(status: ProjectStatus): ProjectStatusCategory {
    return projectSettingsStatusCategoryDraftValue(status, statusDraftState());
  }

  function statusColorDraftValue(status: ProjectStatus): EventColor {
    return projectSettingsStatusColorDraftValue(status, statusDraftState());
  }

  function setStatusColor(statusId: string, color: EventColor | undefined): void {
    if (color === undefined) return;
    statusColorDrafts = {
      ...statusColorDrafts,
      [statusId]: color,
    };
  }

  function setNewStatusColor(color: EventColor | undefined): void {
    if (color !== undefined) newStatusColor = color;
  }

  function statusTaskCount(status: ProjectStatus): number {
    return projects
      .tasksForProjectIncludingArchived(status.projectId)
      .filter((task) => task.statusId === status.id)
      .length;
  }

  function statusDeleteDisabled(status: ProjectStatus): boolean {
    return statuses.length <= 1 || statusTaskCount(status) > 0;
  }

  function statusDeleteTitle(status: ProjectStatus): string {
    if (statuses.length <= 1) return t("projects.settings.deleteStatusBlockedLast");
    if (statusTaskCount(status) > 0) return t("projects.settings.deleteStatusBlockedTasks");
    return t("projects.settings.deleteStatus", status.name);
  }

  function requestDeleteStatus(status: ProjectStatus): void {
    if (statusDeleteDisabled(status)) return;
    pendingDeleteStatusId = status.id;
  }

  function cancelDeleteStatus(): void {
    pendingDeleteStatusId = null;
  }

  async function confirmDeleteStatus(): Promise<void> {
    if (!pendingDeleteStatus) return;
    const status = pendingDeleteStatus;
    pendingDeleteStatusId = null;
    projectSettingsError = null;
    try {
      await projects.removeStatus(status.id);
      const remainingNames = { ...statusNameDrafts };
      const remainingCategories = { ...statusCategoryDrafts };
      delete remainingNames[status.id];
      delete remainingCategories[status.id];
      statusNameDrafts = remainingNames;
      statusCategoryDrafts = remainingCategories;
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function priorityDraftDirty(priority: ProjectPriorityConfig): boolean {
    return projectSettingsPriorityDraftDirty(priority, priorityDraftState());
  }

  function priorityNameDraftValue(priority: ProjectPriorityConfig): string {
    return projectSettingsPriorityNameDraftValue(priority, priorityDraftState());
  }

  function priorityColorDraftValue(priority: ProjectPriorityConfig): EventColor {
    return projectSettingsPriorityColorDraftValue(priority, priorityDraftState());
  }

  function setPriorityColor(priorityId: string, color: EventColor | undefined): void {
    if (color === undefined) return;
    priorityColorDrafts = {
      ...priorityColorDrafts,
      [priorityId]: color,
    };
  }

  function setNewPriorityColor(color: EventColor | undefined): void {
    if (color !== undefined) newPriorityColor = color;
  }

  function priorityTaskCount(priority: ProjectPriorityConfig): number {
    return projects
      .tasksForProjectIncludingArchived(priority.projectId)
      .filter((task) => task.priority === priority.id)
      .length;
  }

  function priorityDeleteDisabled(priority: ProjectPriorityConfig): boolean {
    return priorities.length <= 1 || priorityTaskCount(priority) > 0;
  }

  function priorityDeleteTitle(priority: ProjectPriorityConfig): string {
    if (priorities.length <= 1) return t("projects.settings.deletePriorityBlockedLast");
    if (priorityTaskCount(priority) > 0) return t("projects.settings.deletePriorityBlockedTasks");
    return t("projects.settings.deletePriority", priority.name);
  }

  function requestDeletePriority(priority: ProjectPriorityConfig): void {
    if (priorityDeleteDisabled(priority)) return;
    pendingDeletePriorityId = priority.id;
  }

  function cancelDeletePriority(): void {
    pendingDeletePriorityId = null;
  }

  async function confirmDeletePriority(): Promise<void> {
    if (!pendingDeletePriority) return;
    const priority = pendingDeletePriority;
    pendingDeletePriorityId = null;
    projectSettingsError = null;
    try {
      await projects.removePriority(priority);
      const remainingNames = { ...priorityNameDrafts };
      const remainingColors = { ...priorityColorDrafts };
      delete remainingNames[priority.id];
      delete remainingColors[priority.id];
      priorityNameDrafts = remainingNames;
      priorityColorDrafts = remainingColors;
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.priorityDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function prioritySaveDrafts(): ProjectSettingsPrioritySaveDraft[] | null {
    const result = projectSettingsPrioritySaveDrafts(priorities, priorityDraftState());
    if (result.ok) return result.drafts;
    projectSettingsError = t("projects.settings.priorityNameRequired");
    return null;
  }

  async function submitPriority(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newPriorityName.trim();
    const createdColor = newPriorityColor;
    if (!name) {
      projectSettingsError = t("projects.settings.priorityNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addPriority(selectedProjectId, name, createdColor);
      newPriorityName = "";
      newPriorityColor = nextUnusedPriorityColor(
        nextProjectSettingsPaletteColor(createdColor, NEW_STATUS_FIRST_COLOR),
        createdColor,
      );
      await scrollToNewPriorityRow();
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.prioritySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function statusSaveDrafts(): ProjectSettingsStatusSaveDraft[] | null {
    const result = projectSettingsStatusSaveDrafts(statuses, statusDraftState());
    if (result.ok) return result.drafts;
    projectSettingsError = t("projects.settings.statusNameRequired");
    return null;
  }

  async function submitStatus(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newStatusName.trim();
    const createdColor = newStatusColor;
    if (!name) {
      projectSettingsError = t("projects.settings.statusNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addStatus(selectedProjectId, name, newStatusCategory, createdColor);
      newStatusName = "";
      newStatusCategory = "active";
      newStatusColor = nextUnusedStatusColor(
        nextProjectSettingsPaletteColor(createdColor, NEW_STATUS_FIRST_COLOR),
        createdColor,
      );
      await scrollToNewStatusRow();
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function setTagColor(tagId: string, color: EventColor | undefined): void {
    if (color === undefined) return;
    tagColorDrafts = {
      ...tagColorDrafts,
      [tagId]: color,
    };
  }

  function setNewTagColor(color: EventColor | undefined): void {
    if (color !== undefined) newTagColor = color;
  }

  function tagSaveDrafts(): ProjectSettingsTagSaveDraft[] | null {
    const result = projectSettingsTagSaveDrafts(projectTags, tagDraftState());
    if (result.ok) return result.drafts;
    projectSettingsError = result.error === "name_exists"
      ? t("projects.settings.tagNameExists")
      : t("projects.settings.tagNameRequired");
    return null;
  }

  async function submitTag(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newTagName.trim();
    const createdColor = newTagColor;
    if (!name) {
      projectSettingsError = t("projects.settings.tagNameRequired");
      return;
    }
    if (tagNameExists(name)) {
      projectSettingsError = t("projects.settings.tagNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addTag(selectedProjectId, name, createdColor);
      newTagName = "";
      newTagColor = nextUnusedTagColor(
        nextProjectSettingsPaletteColor(createdColor, NEW_STATUS_FIRST_COLOR),
        createdColor,
      );
      await scrollToNewTagRow();
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.tagSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveTagByDirection(tag: ProjectTag, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveTag(tag, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.tagSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteTag(tag: ProjectTag): void {
    pendingDeleteTagId = tag.id;
  }

  function cancelDeleteTag(): void {
    pendingDeleteTagId = null;
  }

  async function confirmDeleteTag(): Promise<void> {
    if (!pendingDeleteTag) return;
    const tag = pendingDeleteTag;
    pendingDeleteTagId = null;
    projectSettingsError = null;
    try {
      await projects.removeTag(tag.id);
      const remainingNames = { ...tagNameDrafts };
      const remainingColors = { ...tagColorDrafts };
      delete remainingNames[tag.id];
      delete remainingColors[tag.id];
      tagNameDrafts = remainingNames;
      tagColorDrafts = remainingColors;
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.tagDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function customFieldSaveDrafts(): CustomFieldSaveDraft | null {
    const result = projectSettingsCustomFieldSaveDrafts({
      fields: projectCustomFields,
      fieldNameDrafts: customFieldNameDrafts,
      createDraftRows: customFieldCreateDraftRows,
    });
    if (result.ok) return result.value;
    projectSettingsError = customFieldDraftErrorMessage(result.error);
    return null;
  }

  function customFieldOptionSaveDrafts(): CustomFieldOptionSaveDraft | null {
    const result = projectSettingsCustomFieldOptionSaveDrafts({
      fields: projectCustomFields,
      fieldOptions: (fieldId) => projects.customFieldOptionsForField(fieldId),
      optionNameDrafts: customFieldOptionNameDrafts,
      optionCreateDraftRowsByField: customFieldOptionDraftRowsByField,
    });
    if (result.ok) return result.value;
    projectSettingsError = customFieldDraftErrorMessage(result.error);
    return null;
  }

  function customFieldDraftErrorMessage(error: ProjectSettingsCustomFieldDraftError): string {
    if (error === "name_required") return t("projects.customFields.nameRequired");
    if (error === "name_exists") return t("projects.customFields.nameExists");
    if (error === "option_name_exists") return t("projects.customFields.optionNameExists");
    return t("projects.customFields.optionNameRequired");
  }

  function projectDraftErrorMessage(error: ProjectSettingsProjectDraftError): string {
    if (error === "name_required") return t("projects.settings.nameRequired");
    if (error === "group_required") return t("projects.settings.groupRequired");
    return t("projects.settings.invalidDuration");
  }

  function customFieldOptionNamesForCreate(
    fieldType: ProjectCustomFieldType,
    optionRows: readonly NewCustomFieldOptionDraft[],
    pendingOptionName: string,
  ): string[] | null {
    const result = projectSettingsCustomFieldOptionNamesForCreate(
      fieldType,
      optionRows,
      pendingOptionName,
    );
    if (result.ok) return result.value;
    projectSettingsError = customFieldDraftErrorMessage(result.error);
    return null;
  }

  function setNewCustomFieldOptionDraftName(optionId: string, name: string): void {
    newCustomFieldOptionRows = newCustomFieldOptionRows.map((option) =>
      option.id === optionId ? { ...option, name } : option
    );
  }

  function removeNewCustomFieldOptionDraft(optionId: string): void {
    newCustomFieldOptionRows = newCustomFieldOptionRows.filter((option) => option.id !== optionId);
  }

  function submitNewCustomFieldOptionDraft(): void {
    if (!newCustomFieldAcceptsOptions()) return;
    const name = newCustomFieldOptionName.trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (newCustomFieldOptionDraftNameExists(name)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    newCustomFieldOptionRows = [...newCustomFieldOptionRows, { id: crypto.randomUUID(), name }];
    newCustomFieldOptionName = "";
    projectSettingsError = null;
  }

  function submitCustomFieldCreateDraftOption(field: NewCustomFieldDraft): void {
    if (!customFieldCreateDraftAcceptsOptions(field)) return;
    const name = field.optionName.trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldCreateDraftOptionNameExists(field, name)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    customFieldCreateDraftRows = customFieldCreateDraftRows.map((entry) =>
      entry.id === field.id
        ? {
            ...entry,
            optionRows: [...entry.optionRows, { id: crypto.randomUUID(), name }],
            optionName: "",
          }
        : entry
    );
    projectSettingsError = null;
  }

  function submitCustomField(): void {
    const name = newCustomFieldName.trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.nameRequired");
      return;
    }
    if (customFieldCreateDraftNameExists(name)) {
      projectSettingsError = t("projects.customFields.nameExists");
      return;
    }
    const optionNames = customFieldOptionNamesForCreate(
      newCustomFieldType,
      newCustomFieldOptionRows,
      newCustomFieldOptionName,
    );
    if (!optionNames) return;
    projectSettingsError = null;
    customFieldCreateDraftRows = [
      ...customFieldCreateDraftRows,
      {
        id: crypto.randomUUID(),
        name,
        fieldType: newCustomFieldType,
        optionRows: optionNames.map((optionName) => ({ id: crypto.randomUUID(), name: optionName })),
        optionName: "",
      },
    ];
    newCustomFieldName = "";
    newCustomFieldType = "text";
    newCustomFieldOptionRows = [];
    newCustomFieldOptionName = "";
    void scrollToNewCustomFieldRow();
  }

  async function moveProjectCustomField(field: ProjectCustomField, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveCustomField(field, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteCustomField(field: ProjectCustomField): void {
    pendingDeleteCustomFieldId = field.id;
  }

  function cancelDeleteCustomField(): void {
    pendingDeleteCustomFieldId = null;
  }

  async function confirmDeleteCustomField(): Promise<void> {
    if (!pendingDeleteCustomField) return;
    const field = pendingDeleteCustomField;
    pendingDeleteCustomFieldId = null;
    projectSettingsError = null;
    try {
      await projects.removeCustomField(field.id);
      const remainingNames = { ...customFieldNameDrafts };
      delete remainingNames[field.id];
      customFieldNameDrafts = remainingNames;
      const remainingOptionInputs = { ...newCustomFieldOptionDrafts };
      delete remainingOptionInputs[field.id];
      newCustomFieldOptionDrafts = remainingOptionInputs;
      const remainingOptionRows = { ...customFieldOptionDraftRowsByField };
      delete remainingOptionRows[field.id];
      customFieldOptionDraftRowsByField = remainingOptionRows;
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.deleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function submitCustomFieldOption(field: ProjectCustomField): void {
    const name = (newCustomFieldOptionDrafts[field.id] ?? "").trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldOptionCreateDraftNameExists(field.id, name)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    projectSettingsError = null;
    customFieldOptionDraftRowsByField = {
      ...customFieldOptionDraftRowsByField,
      [field.id]: [...customFieldOptionCreateDraftRows(field.id), { id: crypto.randomUUID(), name }],
    };
    newCustomFieldOptionDrafts = { ...newCustomFieldOptionDrafts, [field.id]: "" };
  }

  async function moveProjectCustomFieldOption(option: ProjectCustomFieldOption, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveCustomFieldOption(option, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteCustomFieldOption(option: ProjectCustomFieldOption): void {
    pendingDeleteCustomFieldOptionId = option.id;
  }

  function cancelDeleteCustomFieldOption(): void {
    pendingDeleteCustomFieldOptionId = null;
  }

  async function confirmDeleteCustomFieldOption(): Promise<void> {
    if (!pendingDeleteCustomFieldOption) return;
    const option = pendingDeleteCustomFieldOption;
    pendingDeleteCustomFieldOptionId = null;
    projectSettingsError = null;
    try {
      await projects.removeCustomFieldOption(option.id);
      const remainingNames = { ...customFieldOptionNameDrafts };
      delete remainingNames[option.id];
      customFieldOptionNameDrafts = remainingNames;
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveStatusByDirection(status: ProjectStatus, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveStatus(status, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function movePriorityByDirection(priority: ProjectPriorityConfig, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.movePriority(priority, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.prioritySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveProjectSettings(): Promise<void> {
    if (!selectedProject) return;
    const shouldUpdateProject = projectFieldSettingsDirty;
    const projectUpdateResult = shouldUpdateProject
      ? projectSettingsProjectUpdateFromDraft({
          project: selectedProject,
          draft: projectDraft,
          visibleGroupIds: new Set(visibleProjectGroups.map((group) => group.id)),
          nextSortOrderForGroup: nextProjectSortOrderForGroup,
        })
      : null;
    if (projectUpdateResult && !projectUpdateResult.ok) {
      projectSettingsError = projectDraftErrorMessage(projectUpdateResult.error);
      return;
    }
    const statusDrafts = statusSaveDrafts();
    if (!statusDrafts) return;
    const priorityDrafts = prioritySaveDrafts();
    if (!priorityDrafts) return;
    const tagDrafts = tagSaveDrafts();
    if (!tagDrafts) return;
    const customFieldDrafts = customFieldSaveDrafts();
    if (!customFieldDrafts) return;
    const customFieldOptionDrafts = customFieldOptionSaveDrafts();
    if (!customFieldOptionDrafts) return;
    const shouldRevealInactive = shouldUpdateProject && projectDraft.status !== "active";
    projectSettingsSaving = true;
    projectSettingsError = null;
    try {
      if (projectUpdateResult?.ok) {
        await projects.updateProject(projectUpdateResult.value);
      }
      for (const draft of statusDrafts) {
        await projects.updateStatus(draft.status, {
          name: draft.name,
          category: draft.category,
          color: draft.color,
        });
        statusNameDrafts = { ...statusNameDrafts, [draft.status.id]: draft.name };
        statusCategoryDrafts = { ...statusCategoryDrafts, [draft.status.id]: draft.category };
        statusColorDrafts = { ...statusColorDrafts, [draft.status.id]: draft.color };
      }
      for (const draft of priorityDrafts) {
        await projects.updatePriority(draft.priority, {
          name: draft.name,
          color: draft.color,
        });
        priorityNameDrafts = { ...priorityNameDrafts, [draft.priority.id]: draft.name };
        priorityColorDrafts = { ...priorityColorDrafts, [draft.priority.id]: draft.color };
      }
      for (const draft of tagDrafts) {
        await projects.updateTag(draft.tag, {
          name: draft.name,
          color: draft.color,
        });
        tagNameDrafts = { ...tagNameDrafts, [draft.tag.id]: draft.name };
        tagColorDrafts = { ...tagColorDrafts, [draft.tag.id]: draft.color };
      }
      for (const draft of customFieldDrafts.updates) {
        await projects.updateCustomField(draft.field, {
          name: draft.name,
        });
        customFieldNameDrafts = { ...customFieldNameDrafts, [draft.field.id]: draft.name };
      }
      for (const draft of customFieldDrafts.creates) {
        const createdField = await projects.addCustomField(selectedProject.id, draft.name, draft.fieldType);
        if (createdField) {
          customFieldNameDrafts = {
            ...customFieldNameDrafts,
            [createdField.id]: createdField.name,
          };
          for (const optionName of draft.optionNames) {
            const createdOption = await projects.addCustomFieldOption(createdField.id, optionName);
            if (createdOption) {
              customFieldOptionNameDrafts = {
                ...customFieldOptionNameDrafts,
                [createdOption.id]: createdOption.name,
              };
            }
          }
        }
        removeCustomFieldCreateDraft(draft.draftId);
      }
      for (const draft of customFieldOptionDrafts.updates) {
        await projects.updateCustomFieldOption(draft.option, {
          name: draft.name,
        });
        customFieldOptionNameDrafts = { ...customFieldOptionNameDrafts, [draft.option.id]: draft.name };
      }
      for (const draft of customFieldOptionDrafts.creates) {
        const createdOption = await projects.addCustomFieldOption(draft.field.id, draft.name);
        if (createdOption) {
          customFieldOptionNameDrafts = {
            ...customFieldOptionNameDrafts,
            [createdOption.id]: createdOption.name,
          };
        }
        removeCustomFieldOptionCreateDraft(draft.field.id, draft.draftId);
      }
      if (shouldRevealInactive) {
        onRevealInactive();
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      projectSettingsError = t("projects.settings.saveFailed", message);
    } finally {
      projectSettingsSaving = false;
    }
  }

</script>

{#if selectedProject}
<ProjectSettingsPanelShell
  {presentation}
  draftReady={projectSettingsDraftReady}
  dirty={projectSettingsDirty}
  saving={projectSettingsSaving}
  error={projectSettingsError}
  title={t("projects.settings.title")}
  discardLabel={t("projects.settings.discard")}
  closeLabel={t("projects.settings.close")}
  saveLabel={t("projects.settings.save")}
  onDiscard={discardProjectSettings}
  onClose={closeProjectSettings}
  onSave={() => { void saveProjectSettings(); }}
  bind:scrollElement={settingsScrollElement}
>
          <ProjectSettingsIdentitySection
            bind:projectNameDraft={projectDraft.name}
            bind:projectGroupDraft={projectDraft.groupId}
            bind:projectStatusDraft={projectDraft.status}
            bind:projectIconDraft={projectDraft.icon}
            {projectGroupOptions}
            {lifecycleOptions}
            identityLocked={selectedProjectIdentityLocked}
            {setLifecycleStatus}
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsDefaultsSection
            theme={theme.current}
            pomodoroOptions={PROJECT_POMODORO_PRESET_ORDER}
            {pomodoroPresetLabel}
            projectDefaultEventNamePlaceholder={selectedProjectIdentityLocked ? selectedProject.name : undefined}
            bind:projectColorDraft={projectDraft.color}
            bind:projectDefaultEventNameDraft={projectDraft.defaultEventName}
            bind:projectEventTimeModeDraft={projectDraft.defaultEventTimeMode}
            bind:projectDurationDraft={projectDraft.defaultEventDurationMinutes}
            bind:projectPomodoroModeDraft={projectDraft.defaultPomodoroMode}
            bind:projectPomodoroPresetDraft={projectDraft.defaultPomodoroPresetKey}
            bind:projectPomodoroFocusDraft={projectDraft.defaultPomodoroFocusMinutes}
            bind:projectPomodoroShortBreakDraft={projectDraft.defaultPomodoroShortBreakMinutes}
            bind:projectPomodoroLongBreakDraft={projectDraft.defaultPomodoroLongBreakMinutes}
            bind:projectPomodoroLongBreakAfterFocusDraft={projectDraft.defaultPomodoroLongBreakAfterFocusCount}
            bind:projectIdleSettingsSourceDraft={projectDraft.defaultIdleSettingsSource}
            bind:projectIdlePauseEnabledDraft={projectDraft.defaultIdlePauseEnabled}
            bind:projectIdleThresholdMinutesDraft={projectDraft.defaultIdleThresholdMinutes}
            bind:projectFocusPlaylistDraft={projectDraft.focusPlaylistId}
            bind:projectBreakPlaylistDraft={projectDraft.breakPlaylistId}
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsTagsSection
            theme={theme.current}
            tags={projectTags}
            {draggedTagId}
            {tagReorderPending}
            bind:newTagRowElement
            bind:newTagName
            bind:newTagColor
            {tagDropMarkerVisible}
            onTagDragOver={handleTagDragOver}
            onTagDrop={dropTag}
            onTagDragStart={handleTagDragStart}
            {clearTagDrag}
            {moveTagByDirection}
            {tagColorDraftValue}
            {setTagColor}
            {tagNameDraftValue}
            setTagNameDraft={(tagId, name) => {
              tagNameDrafts = {
                ...tagNameDrafts,
                [tagId]: name,
              };
            }}
            {requestDeleteTag}
            {setNewTagColor}
            {submitTag}
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsCustomFieldsSection
            {projectCustomFields}
            {customFieldOptions}
            {customFieldOptionCreateDraftRows}
            bind:customFieldNameDrafts
            bind:customFieldOptionNameDrafts
            bind:customFieldCreateDraftRows
            bind:newCustomFieldName
            bind:newCustomFieldType
            bind:newCustomFieldOptionDrafts
            bind:newCustomFieldOptionRows
            bind:newCustomFieldOptionName
            bind:newCustomFieldRowElement
            {draggedCustomFieldId}
            {draggedCustomFieldOptionId}
            {customFieldReorderPending}
            {customFieldOptionReorderPending}
            {customFieldDropMarkerVisible}
            {customFieldOptionDropMarkerVisible}
            onCustomFieldDragOver={handleCustomFieldDragOver}
            onCustomFieldDrop={dropCustomField}
            onCustomFieldDragStart={handleCustomFieldDragStart}
            {clearCustomFieldDrag}
            {moveProjectCustomField}
            {requestDeleteCustomField}
            onCustomFieldOptionDragOver={handleCustomFieldOptionDragOver}
            onCustomFieldOptionDrop={dropCustomFieldOption}
            onCustomFieldOptionDragStart={handleCustomFieldOptionDragStart}
            {clearCustomFieldOptionDrag}
            {moveProjectCustomFieldOption}
            {requestDeleteCustomFieldOption}
            {setCustomFieldOptionCreateDraftName}
            {removeCustomFieldOptionCreateDraft}
            {setCustomFieldCreateDraftName}
            {setCustomFieldCreateDraftOptionName}
            {setCustomFieldCreateDraftPendingOptionName}
            {removeCustomFieldCreateDraft}
            {removeCustomFieldCreateDraftOption}
            {submitCustomFieldOption}
            {submitCustomFieldCreateDraftOption}
            {submitCustomField}
            {setNewCustomFieldOptionDraftName}
            {removeNewCustomFieldOptionDraft}
            {submitNewCustomFieldOptionDraft}
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsStatusesSection
            theme={theme.current}
            {statuses}
            {statusCategoryOptions}
            {draggedStatusId}
            {statusReorderPending}
            bind:newStatusRowElement
            bind:newStatusName
            bind:newStatusColor
            bind:newStatusCategory
            {statusDropMarkerVisible}
            onStatusDragOver={handleStatusDragOver}
            onStatusDrop={dropStatus}
            onStatusDragStart={handleStatusDragStart}
            {clearStatusDrag}
            {moveStatusByDirection}
            {statusColorDraftValue}
            {setStatusColor}
            {statusNameDraftValue}
            setStatusNameDraft={(statusId, name) => {
              statusNameDrafts = {
                ...statusNameDrafts,
                [statusId]: name,
              };
            }}
            {statusCategoryDraftValue}
            {setStatusCategory}
            {statusDeleteDisabled}
            {statusDeleteTitle}
            {requestDeleteStatus}
            {setNewStatusColor}
            {setNewStatusCategory}
            {submitStatus}
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsPrioritiesSection
            theme={theme.current}
            {priorities}
            {draggedPriorityId}
            {priorityReorderPending}
            bind:newPriorityRowElement
            bind:newPriorityName
            bind:newPriorityColor
            {priorityDropMarkerVisible}
            onPriorityDragOver={handlePriorityDragOver}
            onPriorityDrop={dropPriority}
            onPriorityDragStart={handlePriorityDragStart}
            {clearPriorityDrag}
            {movePriorityByDirection}
            {priorityColorDraftValue}
            {setPriorityColor}
            {priorityNameDraftValue}
            setPriorityNameDraft={(priorityId, name) => {
              priorityNameDrafts = {
                ...priorityNameDrafts,
                [priorityId]: name,
              };
            }}
            {priorityDeleteDisabled}
            {priorityDeleteTitle}
            {requestDeletePriority}
            {setNewPriorityColor}
            {submitPriority}
          />
</ProjectSettingsPanelShell>
{/if}

<ProjectSettingsDeleteDialogs
  {pendingDeleteStatus}
  {pendingDeletePriority}
  {pendingDeleteTag}
  {pendingDeleteCustomField}
  {pendingDeleteCustomFieldOption}
  customFieldForOption={fieldForCustomFieldOption}
  onConfirmDeleteStatus={() => { void confirmDeleteStatus(); }}
  onCancelDeleteStatus={cancelDeleteStatus}
  onConfirmDeletePriority={() => { void confirmDeletePriority(); }}
  onCancelDeletePriority={cancelDeletePriority}
  onConfirmDeleteTag={() => { void confirmDeleteTag(); }}
  onCancelDeleteTag={cancelDeleteTag}
  onConfirmDeleteCustomField={() => { void confirmDeleteCustomField(); }}
  onCancelDeleteCustomField={cancelDeleteCustomField}
  onConfirmDeleteCustomFieldOption={() => { void confirmDeleteCustomFieldOption(); }}
  onCancelDeleteCustomFieldOption={cancelDeleteCustomFieldOption}
/>
