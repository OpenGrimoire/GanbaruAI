<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import type { EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type { PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import {
    PROJECT_DEFAULT_CUSTOM_POMODORO,
    PROJECT_POMODORO_PRESET_ORDER,
    projectCustomPomodoroFromDefaults,
    type ProjectDefaultIdleSettingsSource,
    type ProjectDefaultPomodoroMode,
  } from "$lib/projects/project-default-pomodoro";
  import {
    projectCustomFieldTypeLabel,
    projectLabelColorDotStyle,
    projectLabelColorSwatchClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import { PROJECT_MAX_DURATION_MINUTES } from "$lib/projects/project-settings-duration";
  import {
    PROJECT_CUSTOM_FIELD_TYPES,
    PROJECT_LIFECYCLE_STATUSES,
  } from "$lib/projects/types";
  import type {
    Project,
    ProjectCustomField,
    ProjectCustomFieldOption,
    ProjectCustomFieldType,
    ProjectLabel,
    ProjectLifecycleStatus,
    ProjectStatus,
    ProjectStatusCategory,
  } from "$lib/projects/types";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import {
    DEFAULT_FOCUS_IDLE_PAUSE_ON_EVENT_CREATE,
    DEFAULT_FOCUS_IDLE_THRESHOLD_MINUTES,
    type FocusIdleThresholdMinutes,
  } from "$lib/stores/preferences";
  import { cn } from "$lib/utils";
  import ProjectIconPicker from "./ProjectIconPicker.svelte";
  import ProjectSettingsDefaultsSection from "./ProjectSettingsDefaultsSection.svelte";

  let {
    projectId,
    presentation = "side",
    onClose,
    onRevealInactive,
  }: {
    projectId: string;
    presentation?: "side" | "popover";
    onClose: () => void;
    onRevealInactive: () => void;
  } = $props();

  const projects = getProjects();
  const theme = getTheme();
  const { t } = getLocalization();

  const PROJECT_STATUS_CATEGORIES: ProjectStatusCategory[] = ["not_started", "active", "blocked", "done"];
  type ProjectLabelColorDraft = EventColor | "none";
  type SelectOption = { value: string; label: string };

  let projectDraftId = $state<string | null>(null);
  let projectDraftUpdatedAt = $state<string | null>(null);
  let projectGroupDraft = $state("");
  let projectNameDraft = $state("");
  let projectIconDraft = $state("folder");
  let projectStatusDraft = $state<ProjectLifecycleStatus>("active");
  let projectColorDraft = $state<EventColor | undefined>(undefined);
  let projectDefaultEventNameDraft = $state("");
  let projectDurationDraft = $state("60");
  let projectPomodoroModeDraft = $state<ProjectDefaultPomodoroMode>("preset");
  let projectPomodoroPresetDraft = $state<PomodoroPresetKey>("adaptive");
  let projectPomodoroFocusDraft = $state(PROJECT_DEFAULT_CUSTOM_POMODORO.focusDurationMinutes);
  let projectPomodoroShortBreakDraft = $state(PROJECT_DEFAULT_CUSTOM_POMODORO.shortBreakMinutes);
  let projectPomodoroLongBreakDraft = $state(PROJECT_DEFAULT_CUSTOM_POMODORO.longBreakMinutes);
  let projectPomodoroLongBreakAfterFocusDraft = $state(
    PROJECT_DEFAULT_CUSTOM_POMODORO.longBreakAfterFocusCount,
  );
  let projectIdleSettingsSourceDraft = $state<ProjectDefaultIdleSettingsSource>("global");
  let projectIdlePauseEnabledDraft = $state(DEFAULT_FOCUS_IDLE_PAUSE_ON_EVENT_CREATE);
  let projectIdleThresholdMinutesDraft = $state<FocusIdleThresholdMinutes>(
    DEFAULT_FOCUS_IDLE_THRESHOLD_MINUTES,
  );
  let projectFocusPlaylistDraft = $state("");
  let projectBreakPlaylistDraft = $state("");
  let projectWorkEnvironmentDraft = $state("");
  let projectBlockerRulesetDraft = $state("");
  let projectSettingsSaving = $state(false);
  let projectSettingsError = $state<string | null>(null);
  let statusNameDrafts = $state<Record<string, string>>({});
  let statusCategoryDrafts = $state<Record<string, ProjectStatusCategory>>({});
  let newStatusName = $state("");
  let newStatusCategory = $state<ProjectStatusCategory>("active");
  let labelNameDrafts = $state<Record<string, string>>({});
  let labelColorDrafts = $state<Record<string, ProjectLabelColorDraft>>({});
  let newLabelName = $state("");
  let newLabelColor = $state<ProjectLabelColorDraft>("none");
  let pendingDeleteLabelId = $state<string | null>(null);
  let customFieldNameDrafts = $state<Record<string, string>>({});
  let customFieldOptionNameDrafts = $state<Record<string, string>>({});
  let newCustomFieldName = $state("");
  let newCustomFieldType = $state<ProjectCustomFieldType>("text");
  let newCustomFieldOptionDrafts = $state<Record<string, string>>({});
  let pendingDeleteCustomFieldId = $state<string | null>(null);
  let pendingDeleteCustomFieldOptionId = $state<string | null>(null);
  let settingsScrollElement = $state<HTMLElement | undefined>();
  let settingsContentElement = $state<HTMLElement | undefined>();
  let settingsScrollable = $state(false);
  let settingsCanScrollUp = $state(false);
  let settingsCanScrollDown = $state(false);
  let settingsScrollStateFrame: number | null = null;

  const selectedProject = $derived(projects.projectById(projectId));
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const visibleProjectGroups = $derived.by(() => projects.visibleGroups());
  const statuses = $derived(projects.statusesForProject(selectedProjectId));
  const projectLabels = $derived(projects.labelsForProject(selectedProjectId));
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
  const customFieldTypeOptions = $derived<SelectOption[]>(
    PROJECT_CUSTOM_FIELD_TYPES.map((fieldType) => ({
      value: fieldType,
      label: projectCustomFieldTypeLabel(fieldType, t),
    })),
  );
  const pendingDeleteLabel = $derived.by(() =>
    pendingDeleteLabelId ? projectLabels.find((label) => label.id === pendingDeleteLabelId) : undefined
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
  const projectSettingsDraftReady = $derived(
    Boolean(selectedProject && projectDraftId === selectedProject.id),
  );
  const projectSettingsDirty = $derived.by(() => {
    if (!selectedProject) return false;
    return projectNameDraft !== selectedProject.name
      || projectGroupDraft !== selectedProject.groupId
      || projectIconDraft !== selectedProject.icon
      || projectStatusDraft !== selectedProject.status
      || projectColorDraft !== selectedProject.color
      || projectDefaultEventNameDraft !== (selectedProject.defaultEventName ?? "")
      || projectDurationDraft !== String(selectedProject.defaultEventDurationMinutes ?? "")
      || projectPomodoroSettingsDirty(selectedProject)
      || projectIdleSettingsDirty(selectedProject)
      || projectFocusPlaylistDraft !== (selectedProject.focusPlaylistId ?? "")
      || projectBreakPlaylistDraft !== (selectedProject.breakPlaylistId ?? "")
      || projectWorkEnvironmentDraft !== (selectedProject.workEnvironmentId ?? "")
      || projectBlockerRulesetDraft !== (selectedProject.blockerRulesetId ?? "");
  });

  $effect(() => {
    if (!selectedProject) return;
    if (
      projectDraftId !== selectedProject.id
      || (!projectSettingsDirty && projectDraftUpdatedAt !== selectedProject.updatedAt)
    ) {
      loadProjectSettingsDraft(selectedProject);
    }
  });

  function loadProjectSettingsDraft(project: Project): void {
    projectDraftId = project.id;
    projectDraftUpdatedAt = project.updatedAt;
    projectGroupDraft = project.groupId;
    projectNameDraft = project.name;
    projectIconDraft = project.icon;
    projectStatusDraft = project.status;
    projectColorDraft = project.color;
    projectDefaultEventNameDraft = project.defaultEventName ?? "";
    projectDurationDraft = String(project.defaultEventDurationMinutes ?? "");
    projectPomodoroModeDraft = project.defaultPomodoroMode;
    projectPomodoroPresetDraft = project.defaultPomodoroPresetKey ?? "adaptive";
    const customPomodoro = projectCustomPomodoroFromDefaults(project);
    projectPomodoroFocusDraft = customPomodoro.focusDurationMinutes;
    projectPomodoroShortBreakDraft = customPomodoro.shortBreakMinutes;
    projectPomodoroLongBreakDraft = customPomodoro.longBreakMinutes;
    projectPomodoroLongBreakAfterFocusDraft = customPomodoro.longBreakAfterFocusCount;
    projectIdleSettingsSourceDraft = project.defaultIdleSettingsSource;
    projectIdlePauseEnabledDraft = project.defaultIdlePauseEnabled;
    projectIdleThresholdMinutesDraft = project.defaultIdleThresholdMinutes;
    projectFocusPlaylistDraft = project.focusPlaylistId ?? "";
    projectBreakPlaylistDraft = project.breakPlaylistId ?? "";
    projectWorkEnvironmentDraft = project.workEnvironmentId ?? "";
    projectBlockerRulesetDraft = project.blockerRulesetId ?? "";
    projectSettingsError = null;
    statusNameDrafts = Object.fromEntries(statuses.map((status) => [status.id, status.name]));
    statusCategoryDrafts = Object.fromEntries(
      statuses.map((status) => [status.id, status.category]),
    );
    labelNameDrafts = Object.fromEntries(projectLabels.map((label) => [label.id, label.name]));
    const nextLabelColorDrafts: Record<string, ProjectLabelColorDraft> = {};
    for (const label of projectLabels) {
      nextLabelColorDrafts[label.id] = label.color ?? "none";
    }
    labelColorDrafts = nextLabelColorDrafts;
    customFieldNameDrafts = Object.fromEntries(projectCustomFields.map((field) => [field.id, field.name]));
    customFieldOptionNameDrafts = Object.fromEntries(
      projectCustomFields.flatMap((field) =>
        projects.customFieldOptionsForField(field.id).map((option) => [option.id, option.name]),
      ),
    );
    newStatusName = "";
    newStatusCategory = "active";
    newLabelName = "";
    newLabelColor = "none";
    pendingDeleteLabelId = null;
    newCustomFieldName = "";
    newCustomFieldType = "text";
    newCustomFieldOptionDrafts = {};
    pendingDeleteCustomFieldId = null;
    pendingDeleteCustomFieldOptionId = null;
  }

  function closeProjectSettings(): void {
    if (selectedProject) loadProjectSettingsDraft(selectedProject);
    onClose();
  }

  function normalizeProjectPositiveInteger(value: string, errorMessage: string): number {
    const trimmed = value.trim();
    const parsed = Number(trimmed);
    if (!trimmed || !Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(errorMessage);
    }
    return parsed;
  }

  function normalizeProjectDuration(value: string, errorMessage: string): number | null {
    if (!value.trim()) return null;
    const parsed = normalizeProjectPositiveInteger(value, errorMessage);
    if (parsed > PROJECT_MAX_DURATION_MINUTES) {
      throw new Error(errorMessage);
    }
    return parsed;
  }

  function normalizeOptionalIdentifier(value: string): string | null {
    const trimmed = value.trim();
    return trimmed || null;
  }

  function normalizeOptionalText(value: string): string | null {
    const trimmed = value.trim();
    return trimmed || null;
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
      projectStatusDraft = value as ProjectLifecycleStatus;
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

  function setNewCustomFieldType(value: string): void {
    if (PROJECT_CUSTOM_FIELD_TYPES.includes(value as ProjectCustomFieldType)) {
      newCustomFieldType = value as ProjectCustomFieldType;
    }
  }

  function compactChoiceClass(active: boolean): string {
    return cn(
      "min-h-7 rounded-md px-2 text-[0.733333rem] font-medium transition-colors",
      active
        ? "bg-accent text-foreground"
        : "text-muted-foreground hover:bg-accent/70 hover:text-foreground",
    );
  }

  function iconButtonClass(tone: "neutral" | "danger" = "neutral"): string {
    return cn(
      "flex h-7 w-7 shrink-0 items-center justify-center rounded-md disabled:cursor-not-allowed disabled:opacity-40",
      tone === "danger"
        ? "text-destructive hover:bg-destructive/10"
        : "text-muted-foreground hover:bg-accent hover:text-foreground",
    );
  }

  function pomodoroPresetLabel(preset: PomodoroPresetKey): string {
    if (preset === "creative") return t("projects.pomodoro.creative");
    if (preset === "balanced") return t("projects.pomodoro.balanced");
    if (preset === "deep") return t("projects.pomodoro.deep");
    if (preset === "extended") return t("projects.pomodoro.extended");
    return t("projects.pomodoro.adaptive");
  }

  function projectPomodoroSettingsDirty(project: Project): boolean {
    if (projectPomodoroModeDraft !== project.defaultPomodoroMode) return true;
    if (projectPomodoroModeDraft === "preset") {
      return projectPomodoroPresetDraft !== (project.defaultPomodoroPresetKey ?? "adaptive");
    }
    if (projectPomodoroModeDraft === "custom") {
      const customPomodoro = projectCustomPomodoroFromDefaults(project);
      return projectPomodoroFocusDraft !== customPomodoro.focusDurationMinutes
        || projectPomodoroShortBreakDraft !== customPomodoro.shortBreakMinutes
        || projectPomodoroLongBreakDraft !== customPomodoro.longBreakMinutes
        || projectPomodoroLongBreakAfterFocusDraft !== customPomodoro.longBreakAfterFocusCount;
    }
    return false;
  }

  function projectIdleSettingsDirty(project: Project): boolean {
    return projectIdleSettingsSourceDraft !== project.defaultIdleSettingsSource
      || projectIdlePauseEnabledDraft !== project.defaultIdlePauseEnabled
      || projectIdleThresholdMinutesDraft !== project.defaultIdleThresholdMinutes;
  }

  function projectPomodoroCustomDraft() {
    return {
      focusDurationMinutes: projectPomodoroFocusDraft,
      shortBreakMinutes: projectPomodoroShortBreakDraft,
      longBreakMinutes: projectPomodoroLongBreakDraft,
      longBreakAfterFocusCount: projectPomodoroLongBreakAfterFocusDraft,
    };
  }

  function customFieldAcceptsOptions(field: ProjectCustomField): boolean {
    return field.fieldType === "select" || field.fieldType === "multi_select";
  }

  function customFieldNameDraftValue(field: ProjectCustomField): string {
    return customFieldNameDrafts[field.id] ?? field.name;
  }

  function customFieldNameExists(name: string, ignoredFieldId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectCustomFields.some((field) =>
      field.id !== ignoredFieldId && field.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldDraftDirty(field: ProjectCustomField): boolean {
    return customFieldNameDraftValue(field) !== field.name;
  }

  function adjacentCustomField(field: ProjectCustomField, direction: -1 | 1): ProjectCustomField | undefined {
    const index = projectCustomFields.findIndex((entry) => entry.id === field.id);
    if (index < 0) return undefined;
    return projectCustomFields[index + direction];
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldOptionNameDraftValue(option: ProjectCustomFieldOption): string {
    return customFieldOptionNameDrafts[option.id] ?? option.name;
  }

  function customFieldOptionNameExists(fieldId: string, name: string, ignoredOptionId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projects.customFieldOptionsForField(fieldId).some((option) =>
      option.id !== ignoredOptionId && option.name.trim().toLowerCase() === normalized
    );
  }

  function customFieldOptionDraftDirty(option: ProjectCustomFieldOption): boolean {
    return customFieldOptionNameDraftValue(option) !== option.name;
  }

  function adjacentCustomFieldOption(
    option: ProjectCustomFieldOption,
    direction: -1 | 1,
  ): ProjectCustomFieldOption | undefined {
    const options = projects.customFieldOptionsForField(option.fieldId);
    const index = options.findIndex((entry) => entry.id === option.id);
    if (index < 0) return undefined;
    return options[index + direction];
  }

  function fieldForCustomFieldOption(option: ProjectCustomFieldOption | undefined): ProjectCustomField | undefined {
    return option ? projectCustomFields.find((field) => field.id === option.fieldId) : undefined;
  }

  function refreshSettingsScrollState(): void {
    settingsScrollStateFrame = null;
    const element = settingsScrollElement;
    if (!element) {
      settingsScrollable = false;
      settingsCanScrollUp = false;
      settingsCanScrollDown = false;
      return;
    }
    const maxScrollTop = element.scrollHeight - element.clientHeight;
    settingsScrollable = maxScrollTop > 1;
    settingsCanScrollUp = element.scrollTop > 1;
    settingsCanScrollDown = element.scrollTop < maxScrollTop - 1;
  }

  function requestSettingsScrollStateRefresh(): void {
    if (settingsScrollStateFrame !== null) cancelAnimationFrame(settingsScrollStateFrame);
    settingsScrollStateFrame = requestAnimationFrame(refreshSettingsScrollState);
  }

  function handleSettingsScroll(): void {
    refreshSettingsScrollState();
  }

  function labelNameDraftValue(label: ProjectLabel): string {
    return labelNameDrafts[label.id] ?? label.name;
  }

  function labelColorDraftValue(label: ProjectLabel): EventColor | undefined {
    const draft = labelColorDrafts[label.id];
    if (draft === "none") return undefined;
    return draft ?? label.color;
  }

  function newLabelColorValue(): EventColor | undefined {
    return newLabelColor === "none" ? undefined : newLabelColor;
  }

  function labelDraftDirty(label: ProjectLabel): boolean {
    return labelNameDraftValue(label) !== label.name
      || labelColorDraftValue(label) !== label.color;
  }

  function labelNameExists(name: string, ignoredLabelId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectLabels.some((label) =>
      label.id !== ignoredLabelId && label.name.trim().toLowerCase() === normalized
    );
  }

  function adjacentLabel(label: ProjectLabel, direction: -1 | 1): ProjectLabel | undefined {
    const index = projectLabels.findIndex((entry) => entry.id === label.id);
    if (index < 0) return undefined;
    return projectLabels[index + direction];
  }

  function adjacentWorkflowStatus(status: ProjectStatus, direction: -1 | 1): ProjectStatus | undefined {
    const index = statuses.findIndex((entry) => entry.id === status.id);
    if (index < 0) return undefined;
    return statuses[index + direction];
  }

  function statusDraftDirty(status: ProjectStatus): boolean {
    return (statusNameDrafts[status.id] ?? status.name) !== status.name
      || (statusCategoryDrafts[status.id] ?? status.category) !== status.category;
  }

  async function saveStatus(status: ProjectStatus): Promise<void> {
    const name = (statusNameDrafts[status.id] ?? status.name).trim();
    const category = statusCategoryDrafts[status.id] ?? status.category;
    if (!name) {
      projectSettingsError = t("projects.settings.statusNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateStatus(status, { name, category });
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitStatus(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newStatusName.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.statusNameRequired");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addStatus(selectedProjectId, name, newStatusCategory);
      newStatusName = "";
      newStatusCategory = "active";
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveLabel(label: ProjectLabel): Promise<void> {
    const name = labelNameDraftValue(label).trim();
    if (!name) {
      projectSettingsError = t("projects.settings.labelNameRequired");
      return;
    }
    if (labelNameExists(name, label.id)) {
      projectSettingsError = t("projects.settings.labelNameExists");
      return;
    }
    const color = labelColorDraftValue(label);
    projectSettingsError = null;
    try {
      await projects.updateLabel(label, { name, color });
      labelNameDrafts = { ...labelNameDrafts, [label.id]: name };
      labelColorDrafts = { ...labelColorDrafts, [label.id]: color ?? "none" };
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitLabel(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newLabelName.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.labelNameRequired");
      return;
    }
    if (labelNameExists(name)) {
      projectSettingsError = t("projects.settings.labelNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addLabel(selectedProjectId, name, newLabelColorValue());
      newLabelName = "";
      newLabelColor = "none";
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function moveProjectLabel(label: ProjectLabel, direction: -1 | 1): Promise<void> {
    projectSettingsError = null;
    try {
      await projects.moveLabel(label, direction);
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function requestDeleteLabel(label: ProjectLabel): void {
    pendingDeleteLabelId = label.id;
  }

  function cancelDeleteLabel(): void {
    pendingDeleteLabelId = null;
  }

  async function confirmDeleteLabel(): Promise<void> {
    if (!pendingDeleteLabel) return;
    const label = pendingDeleteLabel;
    pendingDeleteLabelId = null;
    projectSettingsError = null;
    try {
      await projects.removeLabel(label.id);
      const remainingNames = { ...labelNameDrafts };
      const remainingColors = { ...labelColorDrafts };
      delete remainingNames[label.id];
      delete remainingColors[label.id];
      labelNameDrafts = remainingNames;
      labelColorDrafts = remainingColors;
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.labelDeleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveCustomField(field: ProjectCustomField): Promise<void> {
    const name = customFieldNameDraftValue(field).trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.nameRequired");
      return;
    }
    if (customFieldNameExists(name, field.id)) {
      projectSettingsError = t("projects.customFields.nameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateCustomField(field, { name });
      customFieldNameDrafts = { ...customFieldNameDrafts, [field.id]: name };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitCustomField(): Promise<void> {
    if (!selectedProjectId) return;
    const name = newCustomFieldName.trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.nameRequired");
      return;
    }
    if (customFieldNameExists(name)) {
      projectSettingsError = t("projects.customFields.nameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addCustomField(selectedProjectId, name, newCustomFieldType);
      newCustomFieldName = "";
      newCustomFieldType = "text";
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
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
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.deleteFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function saveCustomFieldOption(option: ProjectCustomFieldOption): Promise<void> {
    const name = customFieldOptionNameDraftValue(option).trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldOptionNameExists(option.fieldId, name, option.id)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.updateCustomFieldOption(option, { name });
      customFieldOptionNameDrafts = { ...customFieldOptionNameDrafts, [option.id]: name };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  async function submitCustomFieldOption(field: ProjectCustomField): Promise<void> {
    const name = (newCustomFieldOptionDrafts[field.id] ?? "").trim();
    if (!name) {
      projectSettingsError = t("projects.customFields.optionNameRequired");
      return;
    }
    if (customFieldOptionNameExists(field.id, name)) {
      projectSettingsError = t("projects.customFields.optionNameExists");
      return;
    }
    projectSettingsError = null;
    try {
      await projects.addCustomFieldOption(field.id, name);
      newCustomFieldOptionDrafts = { ...newCustomFieldOptionDrafts, [field.id]: "" };
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
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

  async function moveWorkflowStatus(status: ProjectStatus, direction: -1 | 1): Promise<void> {
    await projects.moveStatus(status, direction);
  }

  async function saveProjectSettings(): Promise<void> {
    if (!selectedProject) return;
    const name = projectNameDraft.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.nameRequired");
      return;
    }
    if (!visibleProjectGroups.some((group) => group.id === projectGroupDraft)) {
      projectSettingsError = t("projects.settings.groupRequired");
      return;
    }
    projectSettingsSaving = true;
    projectSettingsError = null;
    try {
      const defaultEventDurationMinutes = normalizeProjectDuration(
        projectDurationDraft,
        t("projects.settings.invalidDuration"),
      );
      const defaultPomodoroCustom = projectPomodoroCustomDraft();
      await projects.updateProject({
        id: selectedProject.id,
        groupId: projectGroupDraft,
        name,
        icon: projectIconDraft,
        color: projectColorDraft ?? null,
        sortOrder: projectGroupDraft === selectedProject.groupId
          ? selectedProject.sortOrder
          : nextProjectSortOrderForGroup(projectGroupDraft, selectedProject.id),
        status: projectStatusDraft,
        defaultEventName: normalizeOptionalText(projectDefaultEventNameDraft),
        defaultEventDurationMinutes,
        defaultPomodoroMode: projectPomodoroModeDraft,
        defaultPomodoroPresetKey: projectPomodoroModeDraft === "preset" ? projectPomodoroPresetDraft : null,
        defaultPomodoroFocusMinutes: projectPomodoroModeDraft === "custom"
          ? defaultPomodoroCustom.focusDurationMinutes
          : null,
        defaultPomodoroShortBreakMinutes: projectPomodoroModeDraft === "custom"
          ? defaultPomodoroCustom.shortBreakMinutes
          : null,
        defaultPomodoroLongBreakMinutes: projectPomodoroModeDraft === "custom"
          ? defaultPomodoroCustom.longBreakMinutes
          : null,
        defaultPomodoroLongBreakAfterFocusCount: projectPomodoroModeDraft === "custom"
          ? defaultPomodoroCustom.longBreakAfterFocusCount
          : null,
        defaultIdleSettingsSource: projectIdleSettingsSourceDraft,
        defaultIdlePauseEnabled: projectIdlePauseEnabledDraft,
        defaultIdleThresholdMinutes: projectIdleThresholdMinutesDraft,
        focusPlaylistId: normalizeOptionalIdentifier(projectFocusPlaylistDraft),
        breakPlaylistId: normalizeOptionalIdentifier(projectBreakPlaylistDraft),
        workEnvironmentId: normalizeOptionalIdentifier(projectWorkEnvironmentDraft),
        blockerRulesetId: normalizeOptionalIdentifier(projectBlockerRulesetDraft),
      });
      if (projectStatusDraft !== "active") {
        onRevealInactive();
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      projectSettingsError = message === t("projects.settings.invalidDuration")
        ? message
        : t("projects.settings.saveFailed", message);
    } finally {
      projectSettingsSaving = false;
    }
  }

  $effect(() => {
    const scrollElement = settingsScrollElement;
    if (!scrollElement) return;
    const resizeObserver = new ResizeObserver(requestSettingsScrollStateRefresh);
    resizeObserver.observe(scrollElement);
    if (settingsContentElement) resizeObserver.observe(settingsContentElement);
    requestSettingsScrollStateRefresh();
    return () => {
      resizeObserver.disconnect();
      if (settingsScrollStateFrame !== null) {
        cancelAnimationFrame(settingsScrollStateFrame);
        settingsScrollStateFrame = null;
      }
    };
  });
</script>

{#snippet sectionHeading(label: string, count: string | null = null)}
  <div class="flex min-h-7 items-center justify-between gap-3 px-1">
    <h2 class="truncate text-[0.866667rem] font-semibold text-foreground">{label}</h2>
    {#if count}
      <span class="shrink-0 text-[0.733333rem] font-medium text-muted-foreground">{count}</span>
    {/if}
  </div>
{/snippet}

{#if selectedProject}
<aside
  class={cn(
    "project-settings-panel flex min-h-0 flex-col bg-card",
    presentation === "popover"
      ? "h-full w-full"
      : "w-[min(23rem,42vw)] min-w-64 shrink-0 border-l border-border max-[760px]:fixed max-[760px]:inset-2 max-[760px]:z-30 max-[760px]:w-auto max-[760px]:rounded-md max-[760px]:border",
  )}
>
  <header class="sticky top-0 z-10 flex shrink-0 items-center gap-2 bg-card px-3 pb-1 pt-2">
    <div class="min-w-0 flex-1">
      <div class="truncate text-[0.933333rem] font-semibold">{t("projects.settings.title")}</div>
    </div>
    <button
      type="button"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
      aria-label={t("projects.settings.discard")}
      title={t("projects.settings.discard")}
      disabled={!projectSettingsDraftReady || !projectSettingsDirty}
      onclick={() => loadProjectSettingsDraft(selectedProject)}
    >
      <RotateCcw size={14} strokeWidth={1.75} />
    </button>
    <button
      type="button"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
      aria-label={t("projects.settings.close")}
      title={t("projects.settings.close")}
      onclick={closeProjectSettings}
    >
      <X size={15} strokeWidth={1.75} />
    </button>
  </header>

  <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); void saveProjectSettings(); }}>
    {#if projectSettingsDraftReady}
    <div class="relative min-h-0 flex-1">
      <div
        bind:this={settingsScrollElement}
        data-settings-content
        class={cn(
          "project-settings-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto px-3 pb-3 pt-1",
          settingsScrollable
            && settingsCanScrollUp
            && settingsCanScrollDown
            && "project-settings-scroll-both",
          settingsScrollable
            && settingsCanScrollUp
            && !settingsCanScrollDown
            && "project-settings-scroll-top",
          settingsScrollable
            && !settingsCanScrollUp
            && settingsCanScrollDown
            && "project-settings-scroll-bottom",
        )}
        onscroll={handleSettingsScroll}
      >
        <div bind:this={settingsContentElement} class="flex flex-col gap-4">
          <section class="flex flex-col gap-2">
            <div class="h-px bg-border/70" aria-hidden="true"></div>
            <div class="flex flex-col gap-1.5">
              {@render sectionHeading(t("projects.settings.identity"))}
              <div class="flex flex-col gap-1.5">
                <div class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
                  <span class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.name")}</span>
                  <input
                    bind:value={projectNameDraft}
                    aria-label={t("projects.settings.name")}
                    class="h-7 w-44 min-w-0 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground outline-none transition-colors focus:border-ring dark:bg-transparent max-[480px]:w-full"
                  />
                </div>

                <CustomSelect
                  label={t("projects.settings.group")}
                  value={projectGroupDraft}
                  options={projectGroupOptions}
                  onChange={(value) => {
                    projectGroupDraft = value;
                  }}
                  class="w-44"
                />

                <CustomSelect
                  label={t("projects.settings.lifecycle")}
                  value={projectStatusDraft}
                  options={lifecycleOptions}
                  onChange={setLifecycleStatus}
                  class="w-44"
                />

                <div class="flex items-center justify-between gap-4 px-1 py-1 max-[480px]:flex-col max-[480px]:items-stretch max-[480px]:gap-2">
                  <div class="min-w-0 flex-1 text-[0.866667rem] text-foreground">{t("projects.settings.icon")}</div>
                  <ProjectIconPicker
                    value={projectIconDraft}
                    ariaLabel={t("projects.settings.selectIcon", projectIconDraft)}
                    allowIconColors={false}
                    class="h-7 w-44 max-[480px]:w-full"
                    onChange={(nextIcon) => {
                      projectIconDraft = nextIcon;
                    }}
                  />
                </div>
              </div>
            </div>
          </section>

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <ProjectSettingsDefaultsSection
            theme={theme.current}
            pomodoroOptions={PROJECT_POMODORO_PRESET_ORDER}
            {pomodoroPresetLabel}
            bind:projectColorDraft
            bind:projectDefaultEventNameDraft
            bind:projectDurationDraft
            bind:projectPomodoroModeDraft
            bind:projectPomodoroPresetDraft
            bind:projectPomodoroFocusDraft
            bind:projectPomodoroShortBreakDraft
            bind:projectPomodoroLongBreakDraft
            bind:projectPomodoroLongBreakAfterFocusDraft
            bind:projectIdleSettingsSourceDraft
            bind:projectIdlePauseEnabledDraft
            bind:projectIdleThresholdMinutesDraft
            bind:projectFocusPlaylistDraft
            bind:projectBreakPlaylistDraft
            bind:projectWorkEnvironmentDraft
            bind:projectBlockerRulesetDraft
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-1">
            {@render sectionHeading(t("projects.settings.labels"), String(projectLabels.length))}
            <div class="flex flex-col gap-1">
              {#each projectLabels as label (label.id)}
                {@const previousLabel = adjacentLabel(label, -1)}
                {@const nextLabel = adjacentLabel(label, 1)}
                {@const draftColor = labelColorDraftValue(label)}
                <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto_auto_auto] items-center gap-1 px-1 py-1">
                  <span
                    class={cn("h-2.5 w-2.5 shrink-0 rounded-full border", projectLabelColorSwatchClass(draftColor))}
                    style={projectLabelColorDotStyle(draftColor, theme.current)}
                  ></span>
                  <input
                    value={labelNameDraftValue(label)}
                    class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                    aria-label={t("projects.settings.labelName")}
                    oninput={(event) => {
                      labelNameDrafts = {
                        ...labelNameDrafts,
                        [label.id]: event.currentTarget.value,
                      };
                    }}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void saveLabel(label);
                      }
                    }}
                  />
                  <button
                    type="button"
                    class={compactChoiceClass(draftColor === undefined)}
                    onclick={() => {
                      labelColorDrafts = {
                        ...labelColorDrafts,
                        [label.id]: "none",
                      };
                    }}
                  >
                    {t("common.none")}
                  </button>
                  <ColorPicker
                    color={draftColor}
                    theme={theme.current}
                    title={t("projects.settings.labelColor")}
                    ariaLabel={t("projects.settings.selectLabelColor", label.name)}
                    onselect={(color) => {
                      labelColorDrafts = {
                        ...labelColorDrafts,
                        [label.id]: color ?? "none",
                      };
                    }}
                  />
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!labelDraftDirty(label)}
                    aria-label={t("projects.settings.saveLabel")}
                    title={t("projects.settings.saveLabel")}
                    onclick={() => { void saveLabel(label); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!previousLabel}
                    aria-label={t("projects.actions.moveLabelUp", label.name)}
                    title={t("projects.actions.moveLabelUp", label.name)}
                    onclick={() => { void moveProjectLabel(label, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!nextLabel}
                    aria-label={t("projects.actions.moveLabelDown", label.name)}
                    title={t("projects.actions.moveLabelDown", label.name)}
                    onclick={() => { void moveProjectLabel(label, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class={iconButtonClass("danger")}
                    aria-label={t("projects.actions.deleteLabel", label.name)}
                    title={t("projects.actions.deleteLabel", label.name)}
                    onclick={() => requestDeleteLabel(label)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>
              {:else}
                <div class="px-1 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.settings.noLabels")}
                </div>
              {/each}
            </div>

            <div class="grid min-h-8 grid-cols-[auto_minmax(0,1fr)_auto_auto_auto] items-center gap-1 px-1 py-1">
              <span
                class={cn("h-2.5 w-2.5 shrink-0 rounded-full border", projectLabelColorSwatchClass(newLabelColorValue()))}
                style={projectLabelColorDotStyle(newLabelColorValue(), theme.current)}
              ></span>
              <input
                bind:value={newLabelName}
                class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                placeholder={t("projects.settings.newLabelPlaceholder")}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void submitLabel();
                  }
                }}
              />
              <button
                type="button"
                class={compactChoiceClass(newLabelColor === "none")}
                onclick={() => {
                  newLabelColor = "none";
                }}
              >
                {t("common.none")}
              </button>
              <ColorPicker
                color={newLabelColorValue()}
                theme={theme.current}
                title={t("projects.settings.labelColor")}
                ariaLabel={t("projects.settings.selectNewLabelColor")}
                onselect={(color) => {
                  newLabelColor = color ?? "none";
                }}
              />
              <button
                type="button"
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                aria-label={t("projects.settings.addLabel")}
                title={t("projects.settings.addLabel")}
                onclick={() => { void submitLabel(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
              </button>
            </div>
          </section>

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-1">
            {@render sectionHeading(t("projects.customFields.title"), String(projectCustomFields.length))}
            <div class="flex flex-col gap-1">
              {#each projectCustomFields as field (field.id)}
                {@const previousField = adjacentCustomField(field, -1)}
                {@const nextField = adjacentCustomField(field, 1)}
                <div class="flex flex-col gap-1 px-1 py-1">
                  <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto_auto] items-center gap-1">
                    <input
                      value={customFieldNameDraftValue(field)}
                      class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                      aria-label={t("projects.customFields.fieldName")}
                      oninput={(event) => {
                        customFieldNameDrafts = {
                          ...customFieldNameDrafts,
                          [field.id]: event.currentTarget.value,
                        };
                      }}
                      onkeydown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          void saveCustomField(field);
                        }
                      }}
                    />
                    <span class="px-2 text-[0.766667rem] font-medium text-muted-foreground">
                      {projectCustomFieldTypeLabel(field.fieldType, t)}
                    </span>
                    <button
                      type="button"
                      class={iconButtonClass()}
                      disabled={!customFieldDraftDirty(field)}
                      aria-label={t("projects.customFields.saveField")}
                      title={t("projects.customFields.saveField")}
                      onclick={() => { void saveCustomField(field); }}
                    >
                      <Save size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class={iconButtonClass()}
                      disabled={!previousField}
                      aria-label={t("projects.actions.moveCustomFieldUp", field.name)}
                      title={t("projects.actions.moveCustomFieldUp", field.name)}
                      onclick={() => { void moveProjectCustomField(field, -1); }}
                    >
                      <ArrowUp size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class={iconButtonClass()}
                      disabled={!nextField}
                      aria-label={t("projects.actions.moveCustomFieldDown", field.name)}
                      title={t("projects.actions.moveCustomFieldDown", field.name)}
                      onclick={() => { void moveProjectCustomField(field, 1); }}
                    >
                      <ArrowDown size={13} strokeWidth={1.75} />
                    </button>
                    <button
                      type="button"
                      class={iconButtonClass("danger")}
                      aria-label={t("projects.actions.deleteCustomField", field.name)}
                      title={t("projects.actions.deleteCustomField", field.name)}
                      onclick={() => requestDeleteCustomField(field)}
                    >
                      <Trash2 size={13} strokeWidth={1.75} />
                    </button>
                  </div>

                  {#if customFieldAcceptsOptions(field)}
                    <div class="flex flex-col gap-1 pl-3">
                      <div class="px-2 py-0.5 text-[0.733333rem] font-medium text-muted-foreground">
                        {t("projects.customFields.options")}
                      </div>
                      {#each customFieldOptions(field) as option (option.id)}
                        {@const previousOption = adjacentCustomFieldOption(option, -1)}
                        {@const nextOption = adjacentCustomFieldOption(option, 1)}
                        <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1">
                          <input
                            value={customFieldOptionNameDraftValue(option)}
                            class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                            aria-label={t("projects.customFields.optionName")}
                            oninput={(event) => {
                              customFieldOptionNameDrafts = {
                                ...customFieldOptionNameDrafts,
                                [option.id]: event.currentTarget.value,
                              };
                            }}
                            onkeydown={(event) => {
                              if (event.key === "Enter") {
                                event.preventDefault();
                                void saveCustomFieldOption(option);
                              }
                            }}
                          />
                          <button
                            type="button"
                            class={iconButtonClass()}
                            disabled={!customFieldOptionDraftDirty(option)}
                            aria-label={t("projects.actions.saveCustomFieldOption", option.name)}
                            title={t("projects.actions.saveCustomFieldOption", option.name)}
                            onclick={() => { void saveCustomFieldOption(option); }}
                          >
                            <Save size={13} strokeWidth={1.75} />
                          </button>
                          <button
                            type="button"
                            class={iconButtonClass()}
                            disabled={!previousOption}
                            aria-label={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                            title={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                            onclick={() => { void moveProjectCustomFieldOption(option, -1); }}
                          >
                            <ArrowUp size={13} strokeWidth={1.75} />
                          </button>
                          <button
                            type="button"
                            class={iconButtonClass()}
                            disabled={!nextOption}
                            aria-label={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                            title={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                            onclick={() => { void moveProjectCustomFieldOption(option, 1); }}
                          >
                            <ArrowDown size={13} strokeWidth={1.75} />
                          </button>
                          <button
                            type="button"
                            class={iconButtonClass("danger")}
                            aria-label={t("projects.actions.deleteCustomFieldOption", option.name)}
                            title={t("projects.actions.deleteCustomFieldOption", option.name)}
                            onclick={() => requestDeleteCustomFieldOption(option)}
                          >
                            <Trash2 size={13} strokeWidth={1.75} />
                          </button>
                        </div>
                      {:else}
                        <div class="px-2 py-1.5 text-[0.8rem] text-muted-foreground">
                          {t("projects.customFields.noOptions")}
                        </div>
                      {/each}
                      <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto] items-center gap-1">
                        <input
                          value={newCustomFieldOptionDrafts[field.id] ?? ""}
                          class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                          placeholder={t("projects.customFields.newOptionPlaceholder")}
                          oninput={(event) => {
                            newCustomFieldOptionDrafts = {
                              ...newCustomFieldOptionDrafts,
                              [field.id]: event.currentTarget.value,
                            };
                          }}
                          onkeydown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void submitCustomFieldOption(field);
                            }
                          }}
                        />
                        <button
                          type="button"
                          class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                          aria-label={t("projects.customFields.addOption")}
                          title={t("projects.customFields.addOption")}
                          onclick={() => { void submitCustomFieldOption(field); }}
                        >
                          <Plus size={13} strokeWidth={1.75} />
                        </button>
                      </div>
                    </div>
                  {/if}
                </div>
              {:else}
                <div class="px-1 py-2 text-[0.8rem] text-muted-foreground">
                  {t("projects.customFields.noFields")}
                </div>
              {/each}
            </div>

            <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-1 px-1 py-1">
              <input
                bind:value={newCustomFieldName}
                class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                placeholder={t("projects.customFields.newFieldPlaceholder")}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void submitCustomField();
                  }
                }}
              />
              <CustomSelect
                value={newCustomFieldType}
                options={customFieldTypeOptions}
                onChange={setNewCustomFieldType}
                ariaLabel={t("projects.customFields.fieldType")}
                class="w-32"
              />
              <button
                type="button"
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                aria-label={t("projects.customFields.addField")}
                title={t("projects.customFields.addField")}
                onclick={() => { void submitCustomField(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
              </button>
            </div>
          </section>

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-1">
            {@render sectionHeading(t("projects.settings.workflow"), String(statuses.length))}
            <div class="flex flex-col gap-1">
              {#each statuses as status (status.id)}
                {@const previousStatus = adjacentWorkflowStatus(status, -1)}
                {@const nextStatus = adjacentWorkflowStatus(status, 1)}
                <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 px-1 py-1">
                  <input
                    value={statusNameDrafts[status.id] ?? status.name}
                    class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                    aria-label={t("projects.settings.statusName")}
                    oninput={(event) => {
                      statusNameDrafts = {
                        ...statusNameDrafts,
                        [status.id]: event.currentTarget.value,
                      };
                    }}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void saveStatus(status);
                      }
                    }}
                  />
                  <CustomSelect
                    value={statusCategoryDrafts[status.id] ?? status.category}
                    options={statusCategoryOptions}
                    onChange={(value) => setStatusCategory(status.id, value)}
                    ariaLabel={t("projects.settings.statusName")}
                    class="w-32"
                  />
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!statusDraftDirty(status)}
                    aria-label={t("projects.settings.saveStatus")}
                    title={t("projects.settings.saveStatus")}
                    onclick={() => { void saveStatus(status); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!previousStatus}
                    aria-label={t("projects.actions.moveStatusUp", status.name)}
                    title={t("projects.actions.moveStatusUp", status.name)}
                    onclick={() => { void moveWorkflowStatus(status, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class={iconButtonClass()}
                    disabled={!nextStatus}
                    aria-label={t("projects.actions.moveStatusDown", status.name)}
                    title={t("projects.actions.moveStatusDown", status.name)}
                    onclick={() => { void moveWorkflowStatus(status, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                </div>
              {/each}
            </div>

            <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-1 px-1 py-1">
              <input
                bind:value={newStatusName}
                class="h-7 min-w-0 rounded-md bg-transparent px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:bg-card"
                placeholder={t("projects.settings.newStatusPlaceholder")}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void submitStatus();
                  }
                }}
              />
              <CustomSelect
                value={newStatusCategory}
                options={statusCategoryOptions}
                onChange={setNewStatusCategory}
                ariaLabel={t("projects.settings.statusName")}
                class="w-32"
              />
              <button
                type="button"
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                aria-label={t("projects.settings.addStatus")}
                title={t("projects.settings.addStatus")}
                onclick={() => { void submitStatus(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
              </button>
            </div>
          </section>
        </div>
      </div>
      <CalendarScrollbar
        scrollContainer={settingsScrollElement}
        stickyTop={8}
        stickyBottom={8}
        wheelPassthrough
      />
    </div>

    <footer class="flex shrink-0 items-center gap-2 bg-card px-3 pb-2 pt-1">
      {#if projectSettingsError}
        <div class="min-w-0 flex-1 rounded-md border border-destructive/30 bg-destructive/10 px-2 py-1.5 text-[0.766667rem] text-destructive">
          {projectSettingsError}
        </div>
      {:else}
        <div class="min-w-0 flex-1"></div>
      {/if}
      <button
        type="submit"
        class="flex min-h-8 shrink-0 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
        disabled={projectSettingsSaving || !projectSettingsDirty}
      >
        <Save size={14} strokeWidth={1.75} />
        <span>{projectSettingsSaving ? t("common.loading") : t("projects.settings.save")}</span>
      </button>
    </footer>
    {/if}
  </form>
</aside>
{/if}

{#if pendingDeleteLabel}
  <ConfirmDialog
    title={t("projects.settings.deleteLabelTitle", pendingDeleteLabel.name)}
    message={t("projects.settings.deleteLabelMessage", pendingDeleteLabel.name)}
    confirmLabel={t("projects.settings.deleteLabelConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteLabel(); }}
    onCancel={cancelDeleteLabel}
  />
{/if}

{#if pendingDeleteCustomField}
  <ConfirmDialog
    title={t("projects.customFields.deleteFieldTitle", pendingDeleteCustomField.name)}
    message={t("projects.customFields.deleteFieldMessage", pendingDeleteCustomField.name)}
    confirmLabel={t("projects.customFields.deleteFieldConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteCustomField(); }}
    onCancel={cancelDeleteCustomField}
  />
{/if}

{#if pendingDeleteCustomFieldOption}
  <ConfirmDialog
    title={t("projects.customFields.deleteOptionTitle", pendingDeleteCustomFieldOption.name)}
    message={t(
      "projects.customFields.deleteOptionMessage",
      pendingDeleteCustomFieldOption.name,
      fieldForCustomFieldOption(pendingDeleteCustomFieldOption)?.name ?? "",
    )}
    confirmLabel={t("projects.customFields.deleteOptionConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteCustomFieldOption(); }}
    onCancel={cancelDeleteCustomFieldOption}
  />
{/if}

<style>
  .project-settings-panel {
    --cal-scrollbar-thumb: color-mix(in srgb, var(--card-foreground) 18%, var(--card));
    --cal-scrollbar-thumb-hover: color-mix(in srgb, var(--card-foreground) 36%, var(--card));
  }

  .project-settings-scroll-area {
    --project-settings-scroll-fade-size: 2rem;

    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-settings-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black var(--project-settings-scroll-fade-size), black);
    mask-image: linear-gradient(to bottom, transparent, black var(--project-settings-scroll-fade-size), black);
  }

  .project-settings-scroll-bottom {
    -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-settings-scroll-fade-size)), transparent);
    mask-image: linear-gradient(to bottom, black, black calc(100% - var(--project-settings-scroll-fade-size)), transparent);
  }

  .project-settings-scroll-both {
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-settings-scroll-fade-size),
      black calc(100% - var(--project-settings-scroll-fade-size)),
      transparent
    );
    mask-image: linear-gradient(
      to bottom,
      transparent,
      black var(--project-settings-scroll-fade-size),
      black calc(100% - var(--project-settings-scroll-fade-size)),
      transparent
    );
  }
</style>
