<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import type { EventColor } from "$lib/components/calendar/types";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { COUNT_PRESET_RHYTHMS, type PomodoroPresetKey } from "$lib/pomodoro/rhythm";
  import {
    projectCustomFieldTypeLabel,
    projectLabelColorDotStyle,
    projectLabelColorSwatchClass,
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
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
  import { cn } from "$lib/utils";
  import ProjectIcon from "./ProjectIcon.svelte";
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

  const PROJECT_ICON_OPTIONS = [
    "folder",
    "repeat",
    "apple",
    "graduation-cap",
    "book-open",
    "dumbbell",
    "bath",
    "heart",
    "sparkles",
    "clapperboard",
    "smile",
    "bed",
  ] as const;
  const PROJECT_POMODORO_OPTIONS = Object.keys(COUNT_PRESET_RHYTHMS) as PomodoroPresetKey[];
  const PROJECT_STATUS_CATEGORIES: ProjectStatusCategory[] = ["not_started", "active", "blocked", "done"];
  type ProjectLabelColorDraft = EventColor | "none";

  let projectDraftId = $state<string | null>(null);
  let projectDraftUpdatedAt = $state<string | null>(null);
  let projectGroupDraft = $state("");
  let projectNameDraft = $state("");
  let projectIconDraft = $state("folder");
  let projectStatusDraft = $state<ProjectLifecycleStatus>("active");
  let projectColorDraft = $state<EventColor | undefined>(undefined);
  let projectDurationDraft = $state("60");
  let projectPomodoroDraft = $state<PomodoroPresetKey | "none">("none");
  let projectIdleTimeoutDraft = $state("");
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

  const selectedProject = $derived(projects.projectById(projectId));
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const selectedGroup = $derived(projects.groupById(selectedProject?.groupId));
  const visibleProjectGroups = $derived.by(() => projects.visibleGroups());
  const statuses = $derived(projects.statusesForProject(selectedProjectId));
  const projectLabels = $derived(projects.labelsForProject(selectedProjectId));
  const projectCustomFields = $derived(projects.customFieldsForProject(selectedProjectId));
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
  const projectSettingsDirty = $derived.by(() => {
    if (!selectedProject) return false;
    return projectNameDraft !== selectedProject.name
      || projectGroupDraft !== selectedProject.groupId
      || projectIconDraft !== selectedProject.icon
      || projectStatusDraft !== selectedProject.status
      || projectColorDraft !== selectedProject.color
      || projectDurationDraft !== String(selectedProject.defaultEventDurationMinutes)
      || projectPomodoroDraft !== (selectedProject.defaultPomodoroPresetKey ?? "none")
      || projectIdleTimeoutDraft !== String(selectedProject.defaultIdleTimeoutMinutes ?? "")
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
    projectDurationDraft = String(project.defaultEventDurationMinutes);
    projectPomodoroDraft = project.defaultPomodoroPresetKey ?? "none";
    projectIdleTimeoutDraft = String(project.defaultIdleTimeoutMinutes ?? "");
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

  function normalizeOptionalIdentifier(value: string): string | null {
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

  function pomodoroPresetLabel(preset: PomodoroPresetKey): string {
    if (preset === "creative") return t("projects.pomodoro.creative");
    if (preset === "balanced") return t("projects.pomodoro.balanced");
    if (preset === "deep") return t("projects.pomodoro.deep");
    if (preset === "extended") return t("projects.pomodoro.extended");
    return t("projects.pomodoro.adaptive");
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
      const defaultEventDurationMinutes = normalizeProjectPositiveInteger(
        projectDurationDraft,
        t("projects.settings.invalidDuration"),
      );
      const defaultIdleTimeoutMinutes = projectIdleTimeoutDraft.trim()
        ? normalizeProjectPositiveInteger(projectIdleTimeoutDraft, t("projects.settings.invalidIdleTimeout"))
        : undefined;
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
        defaultEventDurationMinutes,
        defaultPomodoroPresetKey: projectPomodoroDraft === "none" ? null : projectPomodoroDraft,
        defaultIdleTimeoutMinutes: defaultIdleTimeoutMinutes ?? null,
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
        || message === t("projects.settings.invalidIdleTimeout")
        ? message
        : t("projects.settings.saveFailed", message);
    } finally {
      projectSettingsSaving = false;
    }
  }
</script>

{#if selectedProject}
<aside
  class={cn(
    "flex min-h-0 flex-col bg-card",
    presentation === "popover"
      ? "h-full w-full"
      : "w-[min(23rem,42vw)] min-w-64 shrink-0 border-l border-border max-[760px]:fixed max-[760px]:inset-2 max-[760px]:z-30 max-[760px]:w-auto max-[760px]:rounded-md max-[760px]:border",
  )}
>
  <header class="flex shrink-0 items-center gap-2 border-b border-border px-3 py-2">
    <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-accent text-accent-foreground">
      <ProjectIcon name={projectIconDraft} size={16} />
    </span>
    <div class="min-w-0 flex-1">
      <div class="truncate text-[0.933333rem] font-semibold">{t("projects.settings.title")}</div>
      <div class="truncate text-[0.733333rem] text-muted-foreground">{selectedGroup?.name ?? ""}</div>
    </div>
    <button
      type="button"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
      aria-label={t("projects.settings.discard")}
      title={t("projects.settings.discard")}
      disabled={!projectSettingsDirty}
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
    <div class="min-h-0 flex-1 overflow-y-auto px-3 py-3">
      <div class="grid gap-3">
        <label class="grid gap-1 text-[0.733333rem] font-medium text-muted-foreground">
          <span>{t("projects.settings.name")}</span>
          <input
            bind:value={projectNameDraft}
            class="min-h-9 rounded-md border border-border bg-background px-2 text-[0.9rem] font-medium text-foreground"
          />
        </label>

        <section class="grid gap-2 border-t border-border/70 pt-3">
          <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.identity")}</h2>
          <div class="grid gap-1">
            <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.group")}</div>
            <div class="grid gap-1">
              {#each visibleProjectGroups as group (group.id)}
                <button
                  type="button"
                  class={cn(
                    "flex min-h-8 items-center gap-2 rounded-md border px-2 text-left text-[0.8rem]",
                    projectGroupDraft === group.id
                      ? "border-primary/50 bg-primary/10 text-primary"
                      : "border-border bg-background text-foreground hover:bg-accent",
                  )}
                  onclick={() => {
                    projectGroupDraft = group.id;
                  }}
                >
                  <ProjectIcon name={group.icon} size={14} class="shrink-0" />
                  <span class="min-w-0 flex-1 truncate">{group.name}</span>
                </button>
              {/each}
            </div>
          </div>
          <div class="grid gap-1">
            <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.lifecycle")}</div>
            <div class="grid grid-cols-3 gap-1">
              {#each PROJECT_LIFECYCLE_STATUSES as status}
                <button
                  type="button"
                  class={cn(
                    "min-h-8 rounded-md border px-2 text-[0.766667rem] font-medium",
                    projectStatusDraft === status
                      ? projectLifecycleBadgeClass(status)
                      : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  onclick={() => {
                    projectStatusDraft = status;
                  }}
                >
                  {projectLifecycleLabel(status, t)}
                </button>
              {/each}
            </div>
          </div>
          <div class="grid gap-1">
            <div class="text-[0.733333rem] font-medium text-muted-foreground">{t("projects.settings.icon")}</div>
            <div class="grid grid-cols-6 gap-1">
              {#each PROJECT_ICON_OPTIONS as icon}
                <button
                  type="button"
                  class={cn(
                    "flex h-8 items-center justify-center rounded-md border",
                    projectIconDraft === icon
                      ? "border-primary/50 bg-primary/10 text-primary"
                      : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  aria-label={t("projects.settings.selectIcon", icon)}
                  onclick={() => {
                    projectIconDraft = icon;
                  }}
                >
                  <ProjectIcon name={icon} size={15} />
                </button>
              {/each}
            </div>
          </div>

          <div class="flex min-h-8 items-center justify-between gap-3 rounded-md border border-border bg-background px-2">
            <span class="text-[0.8rem]">{t("projects.settings.color")}</span>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="rounded-md border border-border px-2 py-1 text-[0.733333rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                onclick={() => {
                  projectColorDraft = undefined;
                }}
              >
                {t("common.none")}
              </button>
              <ColorPicker
                color={projectColorDraft}
                theme={theme.current}
                title={t("projects.settings.color")}
                ariaLabel={t("projects.settings.selectColor")}
                onselect={(color) => {
                  projectColorDraft = color;
                }}
              />
            </div>
          </div>
        </section>

        <ProjectSettingsDefaultsSection
          pomodoroOptions={PROJECT_POMODORO_OPTIONS}
          {pomodoroPresetLabel}
          bind:projectDurationDraft
          bind:projectPomodoroDraft
          bind:projectIdleTimeoutDraft
          bind:projectFocusPlaylistDraft
          bind:projectBreakPlaylistDraft
          bind:projectWorkEnvironmentDraft
          bind:projectBlockerRulesetDraft
        />

        <section class="grid gap-2 border-t border-border/70 pt-3">
          <div class="flex items-center justify-between gap-2">
            <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.labels")}</h2>
            <span class="text-[0.733333rem] text-muted-foreground">{projectLabels.length}</span>
          </div>
          <div class="grid gap-2">
            {#each projectLabels as label (label.id)}
              {@const previousLabel = adjacentLabel(label, -1)}
              {@const nextLabel = adjacentLabel(label, 1)}
              {@const draftColor = labelColorDraftValue(label)}
              <div class="grid gap-2 rounded-md border border-border bg-background p-2">
                <div class="flex gap-1">
                  <span
                    class={cn("mt-2.5 h-2.5 w-2.5 shrink-0 rounded-full border", projectLabelColorSwatchClass(draftColor))}
                    style={projectLabelColorDotStyle(draftColor, theme.current)}
                  ></span>
                  <input
                    value={labelNameDraftValue(label)}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
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
                    class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={!labelDraftDirty(label)}
                    onclick={() => { void saveLabel(label); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{t("projects.settings.saveLabel")}</span>
                  </button>
                </div>
                <div class="flex flex-wrap items-center gap-1">
                  <span class="mr-1 text-[0.733333rem] font-medium text-muted-foreground">
                    {t("projects.settings.labelColor")}
                  </span>
                  <button
                    type="button"
                    class={cn(
                      "rounded-md border px-2 py-1 text-[0.733333rem]",
                      draftColor === undefined
                        ? "border-primary/50 bg-primary/10 text-primary"
                        : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                    )}
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
                    class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousLabel}
                    aria-label={t("projects.actions.moveLabelUp", label.name)}
                    title={t("projects.actions.moveLabelUp", label.name)}
                    onclick={() => { void moveProjectLabel(label, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextLabel}
                    aria-label={t("projects.actions.moveLabelDown", label.name)}
                    title={t("projects.actions.moveLabelDown", label.name)}
                    onclick={() => { void moveProjectLabel(label, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-40"
                    aria-label={t("projects.actions.deleteLabel", label.name)}
                    title={t("projects.actions.deleteLabel", label.name)}
                    onclick={() => requestDeleteLabel(label)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>
              </div>
            {:else}
              <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                {t("projects.settings.noLabels")}
              </div>
            {/each}
          </div>

          <div class="grid gap-1 rounded-md border border-dashed border-border p-2">
            <div class="flex gap-1">
              <span
                class={cn("mt-2.5 h-2.5 w-2.5 shrink-0 rounded-full border", projectLabelColorSwatchClass(newLabelColorValue()))}
                style={projectLabelColorDotStyle(newLabelColorValue(), theme.current)}
              ></span>
              <input
                bind:value={newLabelName}
                class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
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
                class="flex min-h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                onclick={() => { void submitLabel(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
                <span>{t("projects.settings.addLabel")}</span>
              </button>
            </div>
            <div class="flex flex-wrap items-center gap-1">
              <span class="mr-1 text-[0.733333rem] font-medium text-muted-foreground">
                {t("projects.settings.labelColor")}
              </span>
              <button
                type="button"
                class={cn(
                  "rounded-md border px-2 py-1 text-[0.733333rem]",
                  newLabelColor === "none"
                    ? "border-primary/50 bg-primary/10 text-primary"
                    : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                )}
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
            </div>
          </div>
        </section>

        <section class="grid gap-2 border-t border-border/70 pt-3">
          <div class="flex items-center justify-between gap-2">
            <h2 class="text-[0.8rem] font-semibold">{t("projects.customFields.title")}</h2>
            <span class="text-[0.733333rem] text-muted-foreground">{projectCustomFields.length}</span>
          </div>
          <div class="grid gap-2">
            {#each projectCustomFields as field (field.id)}
              {@const previousField = adjacentCustomField(field, -1)}
              {@const nextField = adjacentCustomField(field, 1)}
              <div class="grid gap-2 rounded-md border border-border bg-background p-2">
                <div class="grid grid-cols-[minmax(0,1fr)_auto] gap-1">
                  <input
                    value={customFieldNameDraftValue(field)}
                    class="min-h-8 min-w-0 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
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
                  <button
                    type="button"
                    class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={!customFieldDraftDirty(field)}
                    onclick={() => { void saveCustomField(field); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{t("projects.customFields.saveField")}</span>
                  </button>
                </div>
                <div class="flex flex-wrap items-center gap-1">
                  <span class="rounded border border-border bg-card px-2 py-1 text-[0.733333rem] text-muted-foreground">
                    {projectCustomFieldTypeLabel(field.fieldType, t)}
                  </span>
                  <button
                    type="button"
                    class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousField}
                    aria-label={t("projects.actions.moveCustomFieldUp", field.name)}
                    title={t("projects.actions.moveCustomFieldUp", field.name)}
                    onclick={() => { void moveProjectCustomField(field, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextField}
                    aria-label={t("projects.actions.moveCustomFieldDown", field.name)}
                    title={t("projects.actions.moveCustomFieldDown", field.name)}
                    onclick={() => { void moveProjectCustomField(field, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:cursor-not-allowed disabled:opacity-40"
                    aria-label={t("projects.actions.deleteCustomField", field.name)}
                    title={t("projects.actions.deleteCustomField", field.name)}
                    onclick={() => requestDeleteCustomField(field)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>

                {#if customFieldAcceptsOptions(field)}
                  <div class="grid gap-1 border-t border-border/60 pt-2">
                    <div class="text-[0.733333rem] font-medium text-muted-foreground">
                      {t("projects.customFields.options")}
                    </div>
                    {#each customFieldOptions(field) as option (option.id)}
                      {@const previousOption = adjacentCustomFieldOption(option, -1)}
                      {@const nextOption = adjacentCustomFieldOption(option, 1)}
                      <div class="grid min-h-8 grid-cols-[minmax(0,1fr)_auto_auto_auto_auto] items-center gap-1 rounded-md border border-border bg-card px-2">
                        <input
                          value={customFieldOptionNameDraftValue(option)}
                          class="min-h-7 min-w-0 bg-transparent px-1 text-[0.8rem] text-foreground"
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
                          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={!customFieldOptionDraftDirty(option)}
                          aria-label={t("projects.actions.saveCustomFieldOption", option.name)}
                          title={t("projects.actions.saveCustomFieldOption", option.name)}
                          onclick={() => { void saveCustomFieldOption(option); }}
                        >
                          <Save size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={!previousOption}
                          aria-label={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                          title={t("projects.actions.moveCustomFieldOptionUp", option.name)}
                          onclick={() => { void moveProjectCustomFieldOption(option, -1); }}
                        >
                          <ArrowUp size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={!nextOption}
                          aria-label={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                          title={t("projects.actions.moveCustomFieldOptionDown", option.name)}
                          onclick={() => { void moveProjectCustomFieldOption(option, 1); }}
                        >
                          <ArrowDown size={13} strokeWidth={1.75} />
                        </button>
                        <button
                          type="button"
                          class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                          aria-label={t("projects.actions.deleteCustomFieldOption", option.name)}
                          title={t("projects.actions.deleteCustomFieldOption", option.name)}
                          onclick={() => requestDeleteCustomFieldOption(option)}
                        >
                          <Trash2 size={13} strokeWidth={1.75} />
                        </button>
                      </div>
                    {:else}
                      <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                        {t("projects.customFields.noOptions")}
                      </div>
                    {/each}
                    <div class="flex gap-1">
                      <input
                        value={newCustomFieldOptionDrafts[field.id] ?? ""}
                        class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
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
                        class="flex min-h-8 items-center gap-1 rounded-md border border-border bg-card px-2 text-[0.733333rem] hover:bg-accent"
                        onclick={() => { void submitCustomFieldOption(field); }}
                      >
                        <Plus size={13} strokeWidth={1.75} />
                        <span>{t("projects.customFields.addOption")}</span>
                      </button>
                    </div>
                  </div>
                {/if}
              </div>
            {:else}
              <div class="rounded-md border border-dashed border-border px-2 py-2 text-[0.8rem] text-muted-foreground">
                {t("projects.customFields.noFields")}
              </div>
            {/each}
          </div>

          <div class="grid gap-2 rounded-md border border-dashed border-border p-2">
            <div class="flex gap-1">
              <input
                bind:value={newCustomFieldName}
                class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
                placeholder={t("projects.customFields.newFieldPlaceholder")}
                onkeydown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    void submitCustomField();
                  }
                }}
              />
              <button
                type="button"
                class="flex min-h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                onclick={() => { void submitCustomField(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
                <span>{t("projects.customFields.addField")}</span>
              </button>
            </div>
            <div class="flex flex-wrap gap-1">
              {#each PROJECT_CUSTOM_FIELD_TYPES as fieldType}
                <button
                  type="button"
                  class={cn(
                    "rounded-md border px-2 py-1 text-[0.733333rem]",
                    newCustomFieldType === fieldType
                      ? "border-primary/50 bg-primary/10 text-primary"
                      : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  onclick={() => {
                    newCustomFieldType = fieldType;
                  }}
                >
                  {projectCustomFieldTypeLabel(fieldType, t)}
                </button>
              {/each}
            </div>
          </div>
        </section>

        <section class="grid gap-2 border-t border-border/70 pt-3">
          <h2 class="text-[0.8rem] font-semibold">{t("projects.settings.workflow")}</h2>
          <div class="grid gap-2">
            {#each statuses as status (status.id)}
              {@const previousStatus = adjacentWorkflowStatus(status, -1)}
              {@const nextStatus = adjacentWorkflowStatus(status, 1)}
              <div class="grid gap-1 rounded-md border border-border bg-background p-2">
                <div class="flex gap-1">
                  <input
                    value={statusNameDrafts[status.id] ?? status.name}
                    class="min-h-8 min-w-0 flex-1 rounded-md border border-border bg-card px-2 text-[0.8rem] text-foreground"
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
                  <button
                    type="button"
                    class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
                    disabled={!statusDraftDirty(status)}
                    onclick={() => { void saveStatus(status); }}
                  >
                    <Save size={13} strokeWidth={1.75} />
                    <span>{t("projects.settings.saveStatus")}</span>
                  </button>
                </div>
                <div class="flex flex-wrap gap-1">
                  {#each PROJECT_STATUS_CATEGORIES as category}
                    <button
                      type="button"
                      class={cn(
                        "rounded-md border px-2 py-1 text-[0.733333rem]",
                        (statusCategoryDrafts[status.id] ?? status.category) === category
                          ? "border-primary/50 bg-primary/10 text-primary"
                          : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                      )}
                      onclick={() => {
                        statusCategoryDrafts = {
                          ...statusCategoryDrafts,
                          [status.id]: category,
                        };
                      }}
                    >
                      {statusCategoryLabel(category)}
                    </button>
                  {/each}
                  <button
                    type="button"
                    class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!previousStatus}
                    aria-label={t("projects.actions.moveStatusUp", status.name)}
                    title={t("projects.actions.moveStatusUp", status.name)}
                    onclick={() => { void moveWorkflowStatus(status, -1); }}
                  >
                    <ArrowUp size={13} strokeWidth={1.75} />
                  </button>
                  <button
                    type="button"
                    class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                    disabled={!nextStatus}
                    aria-label={t("projects.actions.moveStatusDown", status.name)}
                    title={t("projects.actions.moveStatusDown", status.name)}
                    onclick={() => { void moveWorkflowStatus(status, 1); }}
                  >
                    <ArrowDown size={13} strokeWidth={1.75} />
                  </button>
                </div>
              </div>
            {/each}
          </div>

          <div class="grid gap-1 rounded-md border border-dashed border-border p-2">
            <input
              bind:value={newStatusName}
              class="min-h-8 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground"
              placeholder={t("projects.settings.newStatusPlaceholder")}
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  void submitStatus();
                }
              }}
            />
            <div class="flex flex-wrap gap-1">
              {#each PROJECT_STATUS_CATEGORIES as category}
                <button
                  type="button"
                  class={cn(
                    "rounded-md border px-2 py-1 text-[0.733333rem]",
                    newStatusCategory === category
                      ? "border-primary/50 bg-primary/10 text-primary"
                      : "border-border bg-card text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  onclick={() => {
                    newStatusCategory = category;
                  }}
                >
                  {statusCategoryLabel(category)}
                </button>
              {/each}
              <button
                type="button"
                class="ml-auto flex min-h-7 items-center gap-1 rounded-md bg-primary px-2 text-[0.733333rem] font-medium text-primary-foreground"
                onclick={() => { void submitStatus(); }}
              >
                <Plus size={13} strokeWidth={1.75} />
                <span>{t("projects.settings.addStatus")}</span>
              </button>
            </div>
          </div>
        </section>

        {#if projectSettingsError}
          <div class="rounded-md border border-destructive/30 bg-destructive/10 px-2 py-2 text-[0.8rem] text-destructive">
            {projectSettingsError}
          </div>
        {/if}
      </div>
    </div>

    <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-border px-3 py-2">
      <button
        type="button"
        class="flex min-h-8 items-center gap-1.5 rounded-md border border-border bg-card px-2 text-[0.8rem] hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50"
        disabled={!projectSettingsDirty}
        onclick={() => loadProjectSettingsDraft(selectedProject)}
      >
        <RotateCcw size={14} strokeWidth={1.75} />
        <span>{t("projects.settings.discard")}</span>
      </button>
      <button
        type="submit"
        class="flex min-h-8 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
        disabled={projectSettingsSaving || !projectSettingsDirty}
      >
        <Save size={14} strokeWidth={1.75} />
        <span>{projectSettingsSaving ? t("common.loading") : t("projects.settings.save")}</span>
      </button>
    </footer>
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
