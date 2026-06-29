<script lang="ts">
  import { tick } from "svelte";
  import Binary from "@lucide/svelte/icons/binary";
  import CalendarDays from "@lucide/svelte/icons/calendar-days";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import CircleCheck from "@lucide/svelte/icons/circle-check";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import Link2 from "@lucide/svelte/icons/link-2";
  import List from "@lucide/svelte/icons/list";
  import ListChecks from "@lucide/svelte/icons/list-checks";
  import Mail from "@lucide/svelte/icons/mail";
  import Paperclip from "@lucide/svelte/icons/paperclip";
  import Phone from "@lucide/svelte/icons/phone";
  import Plus from "@lucide/svelte/icons/plus";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import SquareCheckBig from "@lucide/svelte/icons/square-check-big";
  import TextAlignStart from "@lucide/svelte/icons/text-align-start";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import UserRound from "@lucide/svelte/icons/user-round";
  import X from "@lucide/svelte/icons/x";
  import CalendarScrollbar from "$lib/components/calendar/CalendarScrollbar.svelte";
  import ColorPicker from "$lib/components/calendar/ColorPicker.svelte";
  import { EVENT_COLOR_OPTIONS } from "$lib/components/calendar/utils";
  import CustomSelect from "$lib/components/settings/CustomSelect.svelte";
  import {
    pickSelectPopoverGeometry,
    type SelectPopoverGeometry,
    type SelectPopoverRect,
  } from "$lib/components/settings/customSelectPosition";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { FALLBACK_COLOR_INDEX, type EventColor } from "$lib/components/calendar/types";
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
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    PROJECT_MAX_DURATION_MINUTES,
    type ProjectDefaultEventTimeMode,
  } from "$lib/projects/project-settings-duration";
  import { projectCustomFieldUsesOptions } from "$lib/projects/custom-fields";
  import {
    PROJECT_CUSTOM_FIELD_TYPES,
    PROJECT_LIFECYCLE_STATUSES,
    PROJECT_TAG_DEFAULT_COLOR,
  } from "$lib/projects/types";
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
  import {
    DEFAULT_FOCUS_IDLE_PAUSE_ON_EVENT_CREATE,
    DEFAULT_FOCUS_IDLE_THRESHOLD_MINUTES,
    type FocusIdleThresholdMinutes,
  } from "$lib/stores/preferences";
  import { cn } from "$lib/utils";
  import { portal } from "$lib/utils/portal";
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
  const NEW_STATUS_FIRST_COLOR: EventColor = 8;
  const NEW_PRIORITY_FIRST_COLOR: EventColor = 13;
  const NEW_TAG_FIRST_COLOR = PROJECT_TAG_DEFAULT_COLOR;
  const PROJECT_STATUS_DRAG_DATA_TYPE = "application/x-ganbaru-project-status";
  const PROJECT_PRIORITY_DRAG_DATA_TYPE = "application/x-ganbaru-project-priority";
  const PROJECT_TAG_DRAG_DATA_TYPE = "application/x-ganbaru-project-tag";
  const PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE = "application/x-ganbaru-project-custom-field";
  const PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE = "application/x-ganbaru-project-custom-field-option";
  const CUSTOM_FIELD_OPTION_CONNECTOR_SPINE_OFFSET_REM = 1.125;
  const CUSTOM_FIELD_OPTION_CONNECTOR_START_Y_REM = -0.625;
  const CUSTOM_FIELD_OPTION_CONNECTOR_FIRST_CENTER_Y_REM = 1;
  const CUSTOM_FIELD_OPTION_CONNECTOR_ROW_STEP_REM = 2.5;
  const CUSTOM_FIELD_OPTION_CONNECTOR_RADIUS_REM = 0.45;
  const CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM = 0.15;
  const CUSTOM_FIELD_TYPE_PICKER_ESTIMATED_HEIGHT = 224;
  const DEFAULT_CUSTOM_FIELD_TYPE_PICKER_GEOMETRY: SelectPopoverGeometry = {
    top: 0,
    left: 0,
    width: null,
    minWidth: 0,
    maxWidth: 0,
    maxHeight: 0,
    placement: "below",
  };
  type SelectOption = { value: string; label: string };
  type StatusDropPosition = "before" | "after";
  type PriorityDropPosition = "before" | "after";
  type TagDropPosition = "before" | "after";
  type CustomFieldDropPosition = "before" | "after";
  type CustomFieldOptionDropPosition = "before" | "after";
  type NewCustomFieldOptionDraft = { id: string; name: string };
  type StatusSaveDraft = {
    status: ProjectStatus;
    name: string;
    category: ProjectStatusCategory;
    color: EventColor;
  };
  type PrioritySaveDraft = {
    priority: ProjectPriorityConfig;
    name: string;
    color: EventColor;
  };
  type CustomFieldSaveDraft = {
    field: ProjectCustomField;
    name: string;
  };
  type CustomFieldOptionUpdateDraft = {
    option: ProjectCustomFieldOption;
    name: string;
  };
  type CustomFieldOptionCreateDraft = {
    field: ProjectCustomField;
    name: string;
    draftId: string | null;
  };
  type CustomFieldOptionSaveDraft = {
    updates: CustomFieldOptionUpdateDraft[];
    creates: CustomFieldOptionCreateDraft[];
  };
  type TagSaveDraft = {
    tag: ProjectTag;
    name: string;
    color: EventColor;
  };

  let projectDraftId = $state<string | null>(null);
  let projectDraftUpdatedAt = $state<string | null>(null);
  let projectGroupDraft = $state("");
  let projectNameDraft = $state("");
  let projectIconDraft = $state("folder");
  let projectStatusDraft = $state<ProjectLifecycleStatus>("active");
  let projectColorDraft = $state<EventColor | undefined>(undefined);
  let projectDefaultEventNameDraft = $state("");
  let projectEventTimeModeDraft = $state<ProjectDefaultEventTimeMode>("timed");
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
  let newCustomFieldName = $state("");
  let newCustomFieldType = $state<ProjectCustomFieldType>("text");
  let newCustomFieldOptionDrafts = $state<Record<string, string>>({});
  let newCustomFieldOptionRows = $state<NewCustomFieldOptionDraft[]>([]);
  let newCustomFieldOptionName = $state("");
  let pendingDeleteCustomFieldId = $state<string | null>(null);
  let pendingDeleteCustomFieldOptionId = $state<string | null>(null);
  let settingsScrollElement = $state<HTMLElement | undefined>();
  let settingsContentElement = $state<HTMLElement | undefined>();
  let customFieldTypePickerOpen = $state(false);
  let customFieldTypeTriggerElement = $state<HTMLButtonElement | undefined>();
  let customFieldTypePanelElement = $state<HTMLDivElement | undefined>();
  let customFieldTypePickerGeometry = $state<SelectPopoverGeometry>(
    DEFAULT_CUSTOM_FIELD_TYPE_PICKER_GEOMETRY,
  );
  let customFieldTypePickerReady = $state(false);
  let newStatusRowElement = $state<HTMLDivElement | undefined>();
  let newPriorityRowElement = $state<HTMLDivElement | undefined>();
  let newTagRowElement = $state<HTMLDivElement | undefined>();
  let newCustomFieldRowElement = $state<HTMLDivElement | undefined>();
  let settingsScrollable = $state(false);
  let settingsCanScrollUp = $state(false);
  let settingsCanScrollDown = $state(false);
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
  let settingsScrollStateFrame: number | null = null;

  const selectedProject = $derived(projects.projectById(projectId));
  const selectedProjectId = $derived(selectedProject?.id ?? null);
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
    return projectNameDraft !== selectedProject.name
      || projectGroupDraft !== selectedProject.groupId
      || projectIconDraft !== selectedProject.icon
      || projectStatusDraft !== selectedProject.status
      || projectColorDraft !== selectedProject.color
      || projectDefaultEventNameDraft !== (selectedProject.defaultEventName ?? "")
      || projectEventTimeModeDraft !== selectedProject.defaultEventTimeMode
      || projectDurationDraft !== String(selectedProject.defaultEventDurationMinutes ?? "")
      || projectPomodoroSettingsDirty(selectedProject)
      || projectIdleSettingsDirty(selectedProject)
      || projectFocusPlaylistDraft !== (selectedProject.focusPlaylistId ?? "")
      || projectBreakPlaylistDraft !== (selectedProject.breakPlaylistId ?? "");
  });
  const statusSettingsDirty = $derived.by(() => statuses.some(statusDraftDirty));
  const prioritySettingsDirty = $derived.by(() => priorities.some(priorityDraftDirty));
  const tagSettingsDirty = $derived.by(() => projectTags.some(tagDraftDirty));
  const customFieldSettingsDirty = $derived.by(() =>
    projectCustomFields.some(customFieldDraftDirty)
      || projectCustomFields.some((field) => customFieldOptions(field).some(customFieldOptionDraftDirty))
      || projectCustomFields.some(customFieldOptionCreateDraftDirty)
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

  function loadProjectSettingsDraft(project: Project): void {
    projectDraftId = project.id;
    projectDraftUpdatedAt = project.updatedAt;
    projectGroupDraft = project.groupId;
    projectNameDraft = project.name;
    projectIconDraft = project.icon;
    projectStatusDraft = project.status;
    projectColorDraft = project.color;
    projectDefaultEventNameDraft = project.defaultEventName ?? "";
    projectEventTimeModeDraft = project.defaultEventTimeMode;
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
    projectFocusPlaylistDraft = "";
    projectBreakPlaylistDraft = "";
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

  function selectNewCustomFieldType(value: ProjectCustomFieldType): void {
    setNewCustomFieldType(value);
    customFieldTypePickerOpen = false;
  }

  function toSelectPopoverRect(rect: DOMRect): SelectPopoverRect {
    return {
      top: rect.top,
      right: rect.right,
      bottom: rect.bottom,
      left: rect.left,
      width: rect.width,
      height: rect.height,
    };
  }

  function customFieldTypePickerBoundaryRect(): SelectPopoverRect {
    const viewportRect: SelectPopoverRect = {
      top: 0,
      right: window.innerWidth,
      bottom: window.innerHeight,
      left: 0,
      width: window.innerWidth,
      height: window.innerHeight,
    };
    if (!customFieldTypeTriggerElement) return viewportRect;
    const boundaryEl =
      customFieldTypeTriggerElement.closest<HTMLElement>("[data-settings-content]")
      ?? customFieldTypeTriggerElement.closest<HTMLElement>("[data-settings-modal-panel]");
    if (!boundaryEl) return viewportRect;
    const boundary = boundaryEl.getBoundingClientRect();
    const top = Math.max(viewportRect.top, boundary.top);
    const right = Math.min(viewportRect.right, boundary.right);
    const bottom = Math.min(viewportRect.bottom, boundary.bottom);
    const left = Math.max(viewportRect.left, boundary.left);
    return {
      top,
      right,
      bottom,
      left,
      width: Math.max(0, right - left),
      height: Math.max(0, bottom - top),
    };
  }

  function computeCustomFieldTypePickerPosition(): void {
    if (!customFieldTypeTriggerElement) return;
    customFieldTypePickerGeometry = pickSelectPopoverGeometry({
      triggerRect: toSelectPopoverRect(customFieldTypeTriggerElement.getBoundingClientRect()),
      boundaryRect: customFieldTypePickerBoundaryRect(),
      contentHeight: customFieldTypePanelElement?.scrollHeight ?? CUSTOM_FIELD_TYPE_PICKER_ESTIMATED_HEIGHT,
      contentWidth: customFieldTypePanelElement?.scrollWidth,
      horizontalAlign: "end",
    });
    customFieldTypePickerReady = true;
  }

  function customFieldTypePickerStyle(): string {
    if (!customFieldTypePickerReady) {
      return "top: 0px; left: 0px; min-width: max-content; max-width: max-content; max-height: none; visibility: hidden;";
    }
    const width = customFieldTypePickerGeometry.width === null
      ? ""
      : ` width: ${customFieldTypePickerGeometry.width}px;`;
    return `top: ${customFieldTypePickerGeometry.top}px; left: ${customFieldTypePickerGeometry.left}px;${width} min-width: ${customFieldTypePickerGeometry.minWidth}px; max-width: ${customFieldTypePickerGeometry.maxWidth}px; max-height: ${customFieldTypePickerGeometry.maxHeight}px; visibility: visible;`;
  }

  async function toggleCustomFieldTypePicker(): Promise<void> {
    if (customFieldTypePickerOpen) {
      customFieldTypePickerOpen = false;
      return;
    }
    customFieldTypePickerReady = false;
    customFieldTypePickerOpen = true;
    await tick();
    computeCustomFieldTypePickerPosition();
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

  function svgNumber(value: number): string {
    return Number(value.toFixed(3)).toString();
  }

  function customFieldOptionConnectorCenterY(index: number): number {
    return CUSTOM_FIELD_OPTION_CONNECTOR_FIRST_CENTER_Y_REM
      + index * CUSTOM_FIELD_OPTION_CONNECTOR_ROW_STEP_REM;
  }

  function customFieldOptionConnectorBottom(optionCount: number): number {
    return customFieldOptionConnectorCenterY(optionCount - 1)
      + CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM;
  }

  function customFieldOptionConnectorTop(): number {
    return CUSTOM_FIELD_OPTION_CONNECTOR_START_Y_REM
      - CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM;
  }

  function customFieldOptionConnectorHeight(optionCount: number): number {
    return customFieldOptionConnectorBottom(optionCount) - customFieldOptionConnectorTop();
  }

  function customFieldOptionConnectorStyle(optionCount: number): string {
    const width = CUSTOM_FIELD_OPTION_CONNECTOR_SPINE_OFFSET_REM
      + CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM * 2;
    const left = -(CUSTOM_FIELD_OPTION_CONNECTOR_SPINE_OFFSET_REM + CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM);
    return [
      `left: ${svgNumber(left)}rem`,
      `top: ${svgNumber(customFieldOptionConnectorTop())}rem`,
      `width: ${svgNumber(width)}rem`,
      `height: ${svgNumber(customFieldOptionConnectorHeight(optionCount))}rem`,
    ].join("; ");
  }

  function customFieldOptionConnectorViewBox(optionCount: number): string {
    const left = -CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM;
    const top = customFieldOptionConnectorTop();
    const width = CUSTOM_FIELD_OPTION_CONNECTOR_SPINE_OFFSET_REM
      + CUSTOM_FIELD_OPTION_CONNECTOR_PADDING_REM * 2;
    const height = customFieldOptionConnectorHeight(optionCount);
    return [
      svgNumber(left),
      svgNumber(top),
      svgNumber(width),
      svgNumber(height),
    ].join(" ");
  }

  function customFieldOptionConnectorPath(optionCount: number): string {
    if (optionCount <= 0) return "";
    const branchRadius = CUSTOM_FIELD_OPTION_CONNECTOR_RADIUS_REM;
    const spineEndY = customFieldOptionConnectorCenterY(optionCount - 1) - branchRadius;
    const parts = [
      `M 0 ${svgNumber(CUSTOM_FIELD_OPTION_CONNECTOR_START_Y_REM)}`,
      `L 0 ${svgNumber(spineEndY)}`,
    ];
    for (let index = 0; index < optionCount; index += 1) {
      const centerY = customFieldOptionConnectorCenterY(index);
      const curveStartY = centerY - branchRadius;
      const curveMidX = branchRadius * 0.32;
      const curveMidY = centerY - branchRadius * 0.22;
      parts.push(
        `M 0 ${svgNumber(curveStartY)}`,
        `C 0 ${svgNumber(curveMidY)} ${svgNumber(curveMidX)} ${svgNumber(centerY)} ${svgNumber(branchRadius)} ${svgNumber(centerY)}`,
        `L ${svgNumber(CUSTOM_FIELD_OPTION_CONNECTOR_SPINE_OFFSET_REM)} ${svgNumber(centerY)}`,
      );
    }
    return parts.join(" ");
  }

  function customFieldAcceptsOptions(field: ProjectCustomField): boolean {
    return projectCustomFieldUsesOptions(field.fieldType);
  }

  function newCustomFieldAcceptsOptions(): boolean {
    return projectCustomFieldUsesOptions(newCustomFieldType);
  }

  function customFieldNameDraftValue(field: ProjectCustomField): string {
    return customFieldNameDrafts[field.id] ?? field.name;
  }

  function customFieldNameExists(name: string, ignoredFieldId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectCustomFields.some((field) =>
      field.id !== ignoredFieldId && customFieldNameDraftValue(field).trim().toLowerCase() === normalized
    );
  }

  function customFieldDraftDirty(field: ProjectCustomField): boolean {
    return customFieldNameDraftValue(field) !== field.name;
  }

  function customFieldOptions(field: ProjectCustomField): ProjectCustomFieldOption[] {
    return projects.customFieldOptionsForField(field.id);
  }

  function customFieldOptionById(optionId: string | null | undefined): ProjectCustomFieldOption | undefined {
    if (!optionId) return undefined;
    return projects.customFieldOptions.find((option) => option.id === optionId);
  }

  function customFieldOptionNameDraftValue(option: ProjectCustomFieldOption): string {
    return customFieldOptionNameDrafts[option.id] ?? option.name;
  }

  function customFieldOptionNameExists(fieldId: string, name: string, ignoredOptionId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projects.customFieldOptionsForField(fieldId).some((option) =>
      option.id !== ignoredOptionId && customFieldOptionNameDraftValue(option).trim().toLowerCase() === normalized
    );
  }

  function customFieldOptionDraftDirty(option: ProjectCustomFieldOption): boolean {
    return customFieldOptionNameDraftValue(option) !== option.name;
  }

  function customFieldOptionCreateDraftRows(fieldId: string): NewCustomFieldOptionDraft[] {
    return customFieldOptionDraftRowsByField[fieldId] ?? [];
  }

  function customFieldOptionCreateDraftDirty(field: ProjectCustomField): boolean {
    return customFieldOptionCreateDraftRows(field.id).length > 0
      || Boolean((newCustomFieldOptionDrafts[field.id] ?? "").trim());
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
    customFieldOptionDraftRowsByField = {
      ...customFieldOptionDraftRowsByField,
      [fieldId]: customFieldOptionCreateDraftRows(fieldId).map((option) =>
        option.id === optionId ? { ...option, name } : option
      ),
    };
  }

  function removeCustomFieldOptionCreateDraft(fieldId: string, optionId: string): void {
    const remainingRows = customFieldOptionCreateDraftRows(fieldId).filter((option) => option.id !== optionId);
    const nextRowsByField = { ...customFieldOptionDraftRowsByField };
    if (remainingRows.length > 0) {
      nextRowsByField[fieldId] = remainingRows;
    } else {
      delete nextRowsByField[fieldId];
    }
    customFieldOptionDraftRowsByField = nextRowsByField;
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

  function randomStatusColor(): EventColor {
    const index = Math.floor(Math.random() * EVENT_COLOR_OPTIONS.length);
    return EVENT_COLOR_OPTIONS[index] ?? NEW_STATUS_FIRST_COLOR;
  }

  function nextPaletteColor(color: EventColor): EventColor {
    const index = EVENT_COLOR_OPTIONS.indexOf(color);
    if (index < 0) return NEW_STATUS_FIRST_COLOR;
    return EVENT_COLOR_OPTIONS[(index + 1) % EVENT_COLOR_OPTIONS.length] ?? NEW_STATUS_FIRST_COLOR;
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
    const used = usedStatusColors(extraColor);
    if (used.size >= EVENT_COLOR_OPTIONS.length) return randomStatusColor();
    const preferredIndex = Math.max(0, EVENT_COLOR_OPTIONS.indexOf(preferredColor));
    for (let offset = 0; offset < EVENT_COLOR_OPTIONS.length; offset += 1) {
      const color = EVENT_COLOR_OPTIONS[(preferredIndex + offset) % EVENT_COLOR_OPTIONS.length];
      if (color !== undefined && !used.has(color)) return color;
    }
    return randomStatusColor();
  }

  function nextUnusedPriorityColor(preferredColor: EventColor, extraColor?: EventColor): EventColor {
    const used = usedPriorityColors(extraColor);
    if (used.size >= EVENT_COLOR_OPTIONS.length) return randomStatusColor();
    const preferredIndex = Math.max(0, EVENT_COLOR_OPTIONS.indexOf(preferredColor));
    for (let offset = 0; offset < EVENT_COLOR_OPTIONS.length; offset += 1) {
      const color = EVENT_COLOR_OPTIONS[(preferredIndex + offset) % EVENT_COLOR_OPTIONS.length];
      if (color !== undefined && !used.has(color)) return color;
    }
    return randomStatusColor();
  }

  function nextUnusedTagColor(preferredColor: EventColor, extraColor?: EventColor): EventColor {
    const used = usedTagColors(extraColor);
    if (used.size >= EVENT_COLOR_OPTIONS.length) return randomStatusColor();
    const preferredIndex = Math.max(0, EVENT_COLOR_OPTIONS.indexOf(preferredColor));
    for (let offset = 0; offset < EVENT_COLOR_OPTIONS.length; offset += 1) {
      const color = EVENT_COLOR_OPTIONS[(preferredIndex + offset) % EVENT_COLOR_OPTIONS.length];
      if (color !== undefined && !used.has(color)) return color;
    }
    return randomStatusColor();
  }

  function cssLengthToPixels(value: string, context: HTMLElement, fallback: number): number {
    const trimmed = value.trim();
    const parsed = Number.parseFloat(trimmed);
    if (!Number.isFinite(parsed)) return fallback;
    if (trimmed.endsWith("rem")) {
      const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
      return parsed * (Number.isFinite(rootFontSize) ? rootFontSize : 16);
    }
    if (trimmed.endsWith("em")) {
      const contextFontSize = Number.parseFloat(getComputedStyle(context).fontSize);
      return parsed * (Number.isFinite(contextFontSize) ? contextFontSize : 16);
    }
    return parsed;
  }

  function settingsScrollFadeInset(scrollElement: HTMLElement): number {
    return cssLengthToPixels(
      getComputedStyle(scrollElement).getPropertyValue("--project-settings-scroll-fade-size"),
      scrollElement,
      32,
    );
  }

  async function scrollToSettingsRow(rowElement: HTMLElement | undefined): Promise<void> {
    await tick();
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve());
    });
    const scrollElement = settingsScrollElement;
    if (!scrollElement || !rowElement) return;
    const scrollRect = scrollElement.getBoundingClientRect();
    const rowRect = rowElement.getBoundingClientRect();
    const maxScrollTop = Math.max(0, scrollElement.scrollHeight - scrollElement.clientHeight);
    const fadeInset = settingsScrollFadeInset(scrollElement);
    const breathingRoom = 4;
    const safeTop = scrollRect.top + (scrollElement.scrollTop > 1 ? fadeInset : 0) + breathingRoom;
    const safeBottom = scrollRect.bottom
      - (scrollElement.scrollTop < maxScrollTop - 1 ? fadeInset : 0)
      - breathingRoom;
    if (rowRect.top >= safeTop && rowRect.bottom <= safeBottom) return;
    const scrollDelta = rowRect.top < safeTop
      ? rowRect.top - safeTop
      : rowRect.bottom - safeBottom;
    const nextScrollTop = Math.min(
      Math.max(0, scrollElement.scrollTop + scrollDelta),
      maxScrollTop,
    );
    if (Math.abs(nextScrollTop - scrollElement.scrollTop) <= 1) return;
    scrollElement.scrollTo({ top: nextScrollTop, behavior: "auto" });
    requestSettingsScrollStateRefresh();
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

  function handleSettingsScroll(): void {
    refreshSettingsScrollState();
  }

  function tagNameDraftValue(tag: ProjectTag): string {
    return tagNameDrafts[tag.id] ?? tag.name;
  }

  function tagColorDraftValue(tag: ProjectTag): EventColor {
    return tagColorDrafts[tag.id] ?? tag.color ?? FALLBACK_COLOR_INDEX;
  }

  function tagDraftDirty(tag: ProjectTag): boolean {
    return tagNameDraftValue(tag) !== tag.name
      || tagColorDraftValue(tag) !== (tag.color ?? FALLBACK_COLOR_INDEX);
  }

  function tagNameExists(name: string, ignoredTagId?: string): boolean {
    const normalized = name.trim().toLowerCase();
    if (!normalized) return false;
    return projectTags.some((tag) =>
      tag.id !== ignoredTagId && tag.name.trim().toLowerCase() === normalized
    );
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

  function statusDropPositionForEvent(event: DragEvent, target: HTMLElement): StatusDropPosition {
    const bounds = target.getBoundingClientRect();
    return event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
  }

  function priorityDropPositionForEvent(event: DragEvent, target: HTMLElement): PriorityDropPosition {
    const bounds = target.getBoundingClientRect();
    return event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
  }

  function tagDropPositionForEvent(event: DragEvent, target: HTMLElement): TagDropPosition {
    const bounds = target.getBoundingClientRect();
    return event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
  }

  function customFieldDropPositionForEvent(event: DragEvent, target: HTMLElement): CustomFieldDropPosition {
    const bounds = target.getBoundingClientRect();
    return event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
  }

  function customFieldOptionDropPositionForEvent(
    event: DragEvent,
    target: HTMLElement,
  ): CustomFieldOptionDropPosition {
    const bounds = target.getBoundingClientRect();
    return event.clientY < bounds.top + bounds.height / 2 ? "before" : "after";
  }

  function statusDropMarkerVisible(statusId: string, position: StatusDropPosition): boolean {
    return draggedStatusId !== null
      && draggedStatusId !== statusId
      && dragOverStatusId === statusId
      && statusDropPosition === position;
  }

  function priorityDropMarkerVisible(priorityId: string, position: PriorityDropPosition): boolean {
    return draggedPriorityId !== null
      && draggedPriorityId !== priorityId
      && dragOverPriorityId === priorityId
      && priorityDropPosition === position;
  }

  function tagDropMarkerVisible(tagId: string, position: TagDropPosition): boolean {
    return draggedTagId !== null
      && draggedTagId !== tagId
      && dragOverTagId === tagId
      && tagDropPosition === position;
  }

  function customFieldDropMarkerVisible(fieldId: string, position: CustomFieldDropPosition): boolean {
    return draggedCustomFieldId !== null
      && draggedCustomFieldId !== fieldId
      && dragOverCustomFieldId === fieldId
      && customFieldDropPosition === position;
  }

  function customFieldOptionDropMarkerVisible(
    optionId: string,
    position: CustomFieldOptionDropPosition,
  ): boolean {
    return draggedCustomFieldOptionId !== null
      && draggedCustomFieldOptionId !== optionId
      && dragOverCustomFieldOptionId === optionId
      && customFieldOptionDropPosition === position;
  }

  function handleStatusDragStart(event: DragEvent, status: ProjectStatus): void {
    draggedStatusId = status.id;
    dragOverStatusId = null;
    statusDropPosition = null;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(PROJECT_STATUS_DRAG_DATA_TYPE, status.id);
    event.dataTransfer.setData("text/plain", status.id);
  }

  function handlePriorityDragStart(event: DragEvent, priority: ProjectPriorityConfig): void {
    draggedPriorityId = priority.id;
    dragOverPriorityId = null;
    priorityDropPosition = null;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(PROJECT_PRIORITY_DRAG_DATA_TYPE, priority.id);
    event.dataTransfer.setData("text/plain", priority.id);
  }

  function handleTagDragStart(event: DragEvent, tag: ProjectTag): void {
    draggedTagId = tag.id;
    dragOverTagId = null;
    tagDropPosition = null;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(PROJECT_TAG_DRAG_DATA_TYPE, tag.id);
    event.dataTransfer.setData("text/plain", tag.id);
  }

  function handleCustomFieldDragStart(event: DragEvent, field: ProjectCustomField): void {
    draggedCustomFieldId = field.id;
    dragOverCustomFieldId = null;
    customFieldDropPosition = null;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE, field.id);
    event.dataTransfer.setData("text/plain", field.id);
  }

  function handleCustomFieldOptionDragStart(event: DragEvent, option: ProjectCustomFieldOption): void {
    draggedCustomFieldOptionId = option.id;
    dragOverCustomFieldOptionId = null;
    customFieldOptionDropPosition = null;
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE, option.id);
    event.dataTransfer.setData("text/plain", option.id);
  }

  function handleStatusDragOver(
    event: DragEvent,
    status: ProjectStatus,
    target: HTMLElement,
  ): void {
    if (!draggedStatusId || statusReorderPending) return;
    if (draggedStatusId === status.id) {
      dragOverStatusId = null;
      statusDropPosition = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dragOverStatusId = status.id;
    statusDropPosition = statusDropPositionForEvent(event, target);
  }

  function handlePriorityDragOver(
    event: DragEvent,
    priority: ProjectPriorityConfig,
    target: HTMLElement,
  ): void {
    if (!draggedPriorityId || priorityReorderPending) return;
    if (draggedPriorityId === priority.id) {
      dragOverPriorityId = null;
      priorityDropPosition = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dragOverPriorityId = priority.id;
    priorityDropPosition = priorityDropPositionForEvent(event, target);
  }

  function handleTagDragOver(
    event: DragEvent,
    tag: ProjectTag,
    target: HTMLElement,
  ): void {
    if (!draggedTagId || tagReorderPending) return;
    if (draggedTagId === tag.id) {
      dragOverTagId = null;
      tagDropPosition = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dragOverTagId = tag.id;
    tagDropPosition = tagDropPositionForEvent(event, target);
  }

  function handleCustomFieldDragOver(
    event: DragEvent,
    field: ProjectCustomField,
    target: HTMLElement,
  ): void {
    if (!draggedCustomFieldId || customFieldReorderPending) return;
    if (draggedCustomFieldId === field.id) {
      dragOverCustomFieldId = null;
      customFieldDropPosition = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dragOverCustomFieldId = field.id;
    customFieldDropPosition = customFieldDropPositionForEvent(event, target);
  }

  function handleCustomFieldOptionDragOver(
    event: DragEvent,
    option: ProjectCustomFieldOption,
    target: HTMLElement,
  ): void {
    if (!draggedCustomFieldOptionId || customFieldOptionReorderPending) return;
    const draggedOption = customFieldOptionById(draggedCustomFieldOptionId);
    if (!draggedOption || draggedOption.fieldId !== option.fieldId) return;
    if (draggedCustomFieldOptionId === option.id) {
      dragOverCustomFieldOptionId = null;
      customFieldOptionDropPosition = null;
      return;
    }
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
    dragOverCustomFieldOptionId = option.id;
    customFieldOptionDropPosition = customFieldOptionDropPositionForEvent(event, target);
  }

  async function moveStatusToIndex(statusId: string, targetIndex: number): Promise<void> {
    statusReorderPending = true;
    projectSettingsError = null;
    try {
      let currentIndex = statuses.findIndex((entry) => entry.id === statusId);
      let remainingMoves = statuses.length;
      while (currentIndex >= 0 && currentIndex !== targetIndex && remainingMoves > 0) {
        const direction: -1 | 1 = currentIndex < targetIndex ? 1 : -1;
        const status = statuses[currentIndex];
        if (!status) break;
        await projects.moveStatus(status, direction);
        currentIndex = statuses.findIndex((entry) => entry.id === statusId);
        remainingMoves -= 1;
      }
      if (currentIndex !== targetIndex) {
        projectSettingsError = t("projects.settings.statusReorderFailed");
      }
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.statusSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      statusReorderPending = false;
      clearStatusDrag();
    }
  }

  async function movePriorityToIndex(priorityId: string, targetIndex: number): Promise<void> {
    priorityReorderPending = true;
    projectSettingsError = null;
    try {
      let currentIndex = priorities.findIndex((entry) => entry.id === priorityId);
      let remainingMoves = priorities.length;
      while (currentIndex >= 0 && currentIndex !== targetIndex && remainingMoves > 0) {
        const direction: -1 | 1 = currentIndex < targetIndex ? 1 : -1;
        const priority = priorities[currentIndex];
        if (!priority) break;
        await projects.movePriority(priority, direction);
        currentIndex = priorities.findIndex((entry) => entry.id === priorityId);
        remainingMoves -= 1;
      }
      if (currentIndex !== targetIndex) {
        projectSettingsError = t("projects.settings.priorityReorderFailed");
      }
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.prioritySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      priorityReorderPending = false;
      clearPriorityDrag();
    }
  }

  async function moveTagToIndex(tagId: string, targetIndex: number): Promise<void> {
    tagReorderPending = true;
    projectSettingsError = null;
    try {
      let currentIndex = projectTags.findIndex((entry) => entry.id === tagId);
      let remainingMoves = projectTags.length;
      while (currentIndex >= 0 && currentIndex !== targetIndex && remainingMoves > 0) {
        const direction: -1 | 1 = currentIndex < targetIndex ? 1 : -1;
        const tag = projectTags[currentIndex];
        if (!tag) break;
        await projects.moveTag(tag, direction);
        currentIndex = projectTags.findIndex((entry) => entry.id === tagId);
        remainingMoves -= 1;
      }
      if (currentIndex !== targetIndex) {
        projectSettingsError = t("projects.settings.tagReorderFailed");
      }
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.tagSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      tagReorderPending = false;
      clearTagDrag();
    }
  }

  async function moveCustomFieldToIndex(fieldId: string, targetIndex: number): Promise<void> {
    customFieldReorderPending = true;
    projectSettingsError = null;
    try {
      let currentIndex = projectCustomFields.findIndex((entry) => entry.id === fieldId);
      let remainingMoves = projectCustomFields.length;
      while (currentIndex >= 0 && currentIndex !== targetIndex && remainingMoves > 0) {
        const direction: -1 | 1 = currentIndex < targetIndex ? 1 : -1;
        const field = projectCustomFields[currentIndex];
        if (!field) break;
        await projects.moveCustomField(field, direction);
        currentIndex = projectCustomFields.findIndex((entry) => entry.id === fieldId);
        remainingMoves -= 1;
      }
      if (currentIndex !== targetIndex) {
        projectSettingsError = t("projects.customFields.reorderFailed");
      }
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.saveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      customFieldReorderPending = false;
      clearCustomFieldDrag();
    }
  }

  async function moveCustomFieldOptionToIndex(optionId: string, targetIndex: number): Promise<void> {
    const initialOption = customFieldOptionById(optionId);
    if (!initialOption) return;
    const fieldId = initialOption.fieldId;
    customFieldOptionReorderPending = true;
    projectSettingsError = null;
    try {
      let options = projects.customFieldOptionsForField(fieldId);
      let currentIndex = options.findIndex((entry) => entry.id === optionId);
      let remainingMoves = options.length;
      while (currentIndex >= 0 && currentIndex !== targetIndex && remainingMoves > 0) {
        const direction: -1 | 1 = currentIndex < targetIndex ? 1 : -1;
        const option = options[currentIndex];
        if (!option) break;
        await projects.moveCustomFieldOption(option, direction);
        options = projects.customFieldOptionsForField(fieldId);
        currentIndex = options.findIndex((entry) => entry.id === optionId);
        remainingMoves -= 1;
      }
      if (currentIndex !== targetIndex) {
        projectSettingsError = t("projects.customFields.optionReorderFailed");
      }
    } catch (error) {
      projectSettingsError = t(
        "projects.customFields.optionSaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    } finally {
      customFieldOptionReorderPending = false;
      clearCustomFieldOptionDrag();
    }
  }

  async function dropStatus(event: DragEvent, targetStatus: ProjectStatus): Promise<void> {
    event.preventDefault();
    const statusId = draggedStatusId
      ?? event.dataTransfer?.getData(PROJECT_STATUS_DRAG_DATA_TYPE)
      ?? event.dataTransfer?.getData("text/plain")
      ?? null;
    if (!statusId || statusId === targetStatus.id) {
      clearStatusDrag();
      return;
    }

    const sourceIndex = statuses.findIndex((entry) => entry.id === statusId);
    let targetIndex = statuses.findIndex((entry) => entry.id === targetStatus.id);
    if (sourceIndex < 0 || targetIndex < 0) {
      clearStatusDrag();
      return;
    }
    if ((statusDropPosition ?? "before") === "after") targetIndex += 1;
    if (sourceIndex < targetIndex) targetIndex -= 1;
    const boundedTargetIndex = Math.max(0, Math.min(statuses.length - 1, targetIndex));
    if (boundedTargetIndex === sourceIndex) {
      clearStatusDrag();
      return;
    }
    await moveStatusToIndex(statusId, boundedTargetIndex);
  }

  async function dropPriority(event: DragEvent, targetPriority: ProjectPriorityConfig): Promise<void> {
    event.preventDefault();
    const priorityId = draggedPriorityId
      ?? event.dataTransfer?.getData(PROJECT_PRIORITY_DRAG_DATA_TYPE)
      ?? event.dataTransfer?.getData("text/plain")
      ?? null;
    if (!priorityId || priorityId === targetPriority.id) {
      clearPriorityDrag();
      return;
    }

    const sourceIndex = priorities.findIndex((entry) => entry.id === priorityId);
    let targetIndex = priorities.findIndex((entry) => entry.id === targetPriority.id);
    if (sourceIndex < 0 || targetIndex < 0) {
      clearPriorityDrag();
      return;
    }
    if ((priorityDropPosition ?? "before") === "after") targetIndex += 1;
    if (sourceIndex < targetIndex) targetIndex -= 1;
    const boundedTargetIndex = Math.max(0, Math.min(priorities.length - 1, targetIndex));
    if (boundedTargetIndex === sourceIndex) {
      clearPriorityDrag();
      return;
    }
    await movePriorityToIndex(priorityId, boundedTargetIndex);
  }

  async function dropTag(event: DragEvent, targetTag: ProjectTag): Promise<void> {
    event.preventDefault();
    const tagId = draggedTagId
      ?? event.dataTransfer?.getData(PROJECT_TAG_DRAG_DATA_TYPE)
      ?? event.dataTransfer?.getData("text/plain")
      ?? null;
    if (!tagId || tagId === targetTag.id) {
      clearTagDrag();
      return;
    }

    const sourceIndex = projectTags.findIndex((entry) => entry.id === tagId);
    let targetIndex = projectTags.findIndex((entry) => entry.id === targetTag.id);
    if (sourceIndex < 0 || targetIndex < 0) {
      clearTagDrag();
      return;
    }
    if ((tagDropPosition ?? "before") === "after") targetIndex += 1;
    if (sourceIndex < targetIndex) targetIndex -= 1;
    const boundedTargetIndex = Math.max(0, Math.min(projectTags.length - 1, targetIndex));
    if (boundedTargetIndex === sourceIndex) {
      clearTagDrag();
      return;
    }
    await moveTagToIndex(tagId, boundedTargetIndex);
  }

  async function dropCustomField(event: DragEvent, targetField: ProjectCustomField): Promise<void> {
    event.preventDefault();
    const fieldId = draggedCustomFieldId
      ?? event.dataTransfer?.getData(PROJECT_CUSTOM_FIELD_DRAG_DATA_TYPE)
      ?? event.dataTransfer?.getData("text/plain")
      ?? null;
    if (!fieldId || fieldId === targetField.id) {
      clearCustomFieldDrag();
      return;
    }

    const sourceIndex = projectCustomFields.findIndex((entry) => entry.id === fieldId);
    let targetIndex = projectCustomFields.findIndex((entry) => entry.id === targetField.id);
    if (sourceIndex < 0 || targetIndex < 0) {
      clearCustomFieldDrag();
      return;
    }
    if ((customFieldDropPosition ?? "before") === "after") targetIndex += 1;
    if (sourceIndex < targetIndex) targetIndex -= 1;
    const boundedTargetIndex = Math.max(0, Math.min(projectCustomFields.length - 1, targetIndex));
    if (boundedTargetIndex === sourceIndex) {
      clearCustomFieldDrag();
      return;
    }
    await moveCustomFieldToIndex(fieldId, boundedTargetIndex);
  }

  async function dropCustomFieldOption(
    event: DragEvent,
    targetOption: ProjectCustomFieldOption,
  ): Promise<void> {
    event.preventDefault();
    const optionId = draggedCustomFieldOptionId
      ?? event.dataTransfer?.getData(PROJECT_CUSTOM_FIELD_OPTION_DRAG_DATA_TYPE)
      ?? event.dataTransfer?.getData("text/plain")
      ?? null;
    if (!optionId || optionId === targetOption.id) {
      clearCustomFieldOptionDrag();
      return;
    }
    const sourceOption = customFieldOptionById(optionId);
    if (!sourceOption || sourceOption.fieldId !== targetOption.fieldId) {
      clearCustomFieldOptionDrag();
      return;
    }

    const options = projects.customFieldOptionsForField(targetOption.fieldId);
    const sourceIndex = options.findIndex((entry) => entry.id === optionId);
    let targetIndex = options.findIndex((entry) => entry.id === targetOption.id);
    if (sourceIndex < 0 || targetIndex < 0) {
      clearCustomFieldOptionDrag();
      return;
    }
    if ((customFieldOptionDropPosition ?? "before") === "after") targetIndex += 1;
    if (sourceIndex < targetIndex) targetIndex -= 1;
    const boundedTargetIndex = Math.max(0, Math.min(options.length - 1, targetIndex));
    if (boundedTargetIndex === sourceIndex) {
      clearCustomFieldOptionDrag();
      return;
    }
    await moveCustomFieldOptionToIndex(optionId, boundedTargetIndex);
  }

  function statusDraftDirty(status: ProjectStatus): boolean {
    return (statusNameDrafts[status.id] ?? status.name) !== status.name
      || (statusCategoryDrafts[status.id] ?? status.category) !== status.category
      || statusColorDraftValue(status) !== status.color;
  }

  function statusColorDraftValue(status: ProjectStatus): EventColor {
    return statusColorDrafts[status.id] ?? status.color ?? FALLBACK_COLOR_INDEX;
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
    return (priorityNameDrafts[priority.id] ?? priority.name) !== priority.name
      || priorityColorDraftValue(priority) !== priority.color;
  }

  function priorityColorDraftValue(priority: ProjectPriorityConfig): EventColor {
    return priorityColorDrafts[priority.id] ?? priority.color ?? FALLBACK_COLOR_INDEX;
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

  function prioritySaveDrafts(): PrioritySaveDraft[] | null {
    const drafts: PrioritySaveDraft[] = [];
    for (const priority of priorities) {
      if (!priorityDraftDirty(priority)) continue;
      const name = (priorityNameDrafts[priority.id] ?? priority.name).trim();
      if (!name) {
        projectSettingsError = t("projects.settings.priorityNameRequired");
        return null;
      }
      drafts.push({
        priority,
        name,
        color: priorityColorDraftValue(priority),
      });
    }
    return drafts;
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
      newPriorityColor = nextUnusedPriorityColor(nextPaletteColor(createdColor), createdColor);
      await scrollToNewPriorityRow();
    } catch (error) {
      projectSettingsError = t(
        "projects.settings.prioritySaveFailed",
        error instanceof Error ? error.message : String(error),
      );
    }
  }

  function statusSaveDrafts(): StatusSaveDraft[] | null {
    const drafts: StatusSaveDraft[] = [];
    for (const status of statuses) {
      if (!statusDraftDirty(status)) continue;
      const name = (statusNameDrafts[status.id] ?? status.name).trim();
      if (!name) {
        projectSettingsError = t("projects.settings.statusNameRequired");
        return null;
      }
      drafts.push({
        status,
        name,
        category: statusCategoryDrafts[status.id] ?? status.category,
        color: statusColorDraftValue(status),
      });
    }
    return drafts;
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
      newStatusColor = nextUnusedStatusColor(nextPaletteColor(createdColor), createdColor);
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

  function tagSaveDrafts(): TagSaveDraft[] | null {
    const drafts: TagSaveDraft[] = [];
    const seenNames = new Set<string>();
    for (const tag of projectTags) {
      const name = tagNameDraftValue(tag).trim();
      if (!name) {
        projectSettingsError = t("projects.settings.tagNameRequired");
        return null;
      }
      const normalizedName = name.toLowerCase();
      if (seenNames.has(normalizedName)) {
        projectSettingsError = t("projects.settings.tagNameExists");
        return null;
      }
      seenNames.add(normalizedName);
      if (!tagDraftDirty(tag)) continue;
      drafts.push({
        tag,
        name,
        color: tagColorDraftValue(tag),
      });
    }
    return drafts;
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
      newTagColor = nextUnusedTagColor(nextPaletteColor(createdColor), createdColor);
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

  function customFieldSaveDrafts(): CustomFieldSaveDraft[] | null {
    const drafts: CustomFieldSaveDraft[] = [];
    const seenNames = new Set<string>();
    for (const field of projectCustomFields) {
      const name = customFieldNameDraftValue(field).trim();
      if (!name) {
        projectSettingsError = t("projects.customFields.nameRequired");
        return null;
      }
      const normalizedName = name.toLowerCase();
      if (seenNames.has(normalizedName)) {
        projectSettingsError = t("projects.customFields.nameExists");
        return null;
      }
      seenNames.add(normalizedName);
      if (!customFieldDraftDirty(field)) continue;
      drafts.push({ field, name });
    }
    return drafts;
  }

  function customFieldOptionSaveDrafts(): CustomFieldOptionSaveDraft | null {
    const updates: CustomFieldOptionUpdateDraft[] = [];
    const creates: CustomFieldOptionCreateDraft[] = [];
    for (const field of projectCustomFields) {
      if (!customFieldAcceptsOptions(field)) continue;
      const seenNames = new Set<string>();
      for (const option of customFieldOptions(field)) {
        const name = customFieldOptionNameDraftValue(option).trim();
        if (!name) {
          projectSettingsError = t("projects.customFields.optionNameRequired");
          return null;
        }
        const normalizedName = name.toLowerCase();
        if (seenNames.has(normalizedName)) {
          projectSettingsError = t("projects.customFields.optionNameExists");
          return null;
        }
        seenNames.add(normalizedName);
        if (!customFieldOptionDraftDirty(option)) continue;
        updates.push({ option, name });
      }
      for (const option of customFieldOptionCreateDraftRows(field.id)) {
        const name = option.name.trim();
        if (!name) {
          projectSettingsError = t("projects.customFields.optionNameRequired");
          return null;
        }
        const normalizedName = name.toLowerCase();
        if (seenNames.has(normalizedName)) {
          projectSettingsError = t("projects.customFields.optionNameExists");
          return null;
        }
        seenNames.add(normalizedName);
        creates.push({ field, name, draftId: option.id });
      }
      const pendingName = (newCustomFieldOptionDrafts[field.id] ?? "").trim();
      if (pendingName) {
        const normalizedName = pendingName.toLowerCase();
        if (seenNames.has(normalizedName)) {
          projectSettingsError = t("projects.customFields.optionNameExists");
          return null;
        }
        seenNames.add(normalizedName);
        creates.push({ field, name: pendingName, draftId: null });
      }
    }
    return { updates, creates };
  }

  function newCustomFieldOptionNamesForCreate(): string[] | null {
    if (!newCustomFieldAcceptsOptions()) return [];
    const names: string[] = [];
    const seenNames = new Set<string>();
    for (const option of newCustomFieldOptionRows) {
      const name = option.name.trim();
      if (!name) {
        projectSettingsError = t("projects.customFields.optionNameRequired");
        return null;
      }
      const normalizedName = name.toLowerCase();
      if (seenNames.has(normalizedName)) {
        projectSettingsError = t("projects.customFields.optionNameExists");
        return null;
      }
      seenNames.add(normalizedName);
      names.push(name);
    }
    const pendingName = newCustomFieldOptionName.trim();
    if (pendingName) {
      const normalizedName = pendingName.toLowerCase();
      if (seenNames.has(normalizedName)) {
        projectSettingsError = t("projects.customFields.optionNameExists");
        return null;
      }
      names.push(pendingName);
    }
    return names;
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
    const optionNames = newCustomFieldOptionNamesForCreate();
    if (!optionNames) return;
    projectSettingsError = null;
    try {
      const field = await projects.addCustomField(selectedProjectId, name, newCustomFieldType);
      if (field) {
        for (const optionName of optionNames) {
          await projects.addCustomFieldOption(field.id, optionName);
        }
      }
      newCustomFieldName = "";
      newCustomFieldType = "text";
      newCustomFieldOptionRows = [];
      newCustomFieldOptionName = "";
      await scrollToNewCustomFieldRow();
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
    const name = projectNameDraft.trim();
    if (!name) {
      projectSettingsError = t("projects.settings.nameRequired");
      return;
    }
    if (!visibleProjectGroups.some((group) => group.id === projectGroupDraft)) {
      projectSettingsError = t("projects.settings.groupRequired");
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
    const shouldUpdateProject = projectFieldSettingsDirty;
    const shouldRevealInactive = shouldUpdateProject && projectStatusDraft !== "active";
    projectSettingsSaving = true;
    projectSettingsError = null;
    try {
      if (shouldUpdateProject) {
        const defaultEventDurationMinutes = projectEventTimeModeDraft === "all_day"
          ? null
          : normalizeProjectDuration(
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
          defaultEventTimeMode: projectEventTimeModeDraft,
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
          workEnvironmentId: selectedProject.workEnvironmentId ?? null,
          blockerRulesetId: selectedProject.blockerRulesetId ?? null,
        });
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
      for (const draft of customFieldDrafts) {
        await projects.updateCustomField(draft.field, {
          name: draft.name,
        });
        customFieldNameDrafts = { ...customFieldNameDrafts, [draft.field.id]: draft.name };
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
        if (draft.draftId) {
          removeCustomFieldOptionCreateDraft(draft.field.id, draft.draftId);
        } else {
          newCustomFieldOptionDrafts = {
            ...newCustomFieldOptionDrafts,
            [draft.field.id]: "",
          };
        }
      }
      if (shouldRevealInactive) {
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

  $effect(() => {
    if (!customFieldTypePickerOpen) return;
    function handleClickOutside(event: MouseEvent): void {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (customFieldTypeTriggerElement?.contains(target)) return;
      if (customFieldTypePanelElement?.contains(target)) return;
      customFieldTypePickerOpen = false;
    }
    function handleScroll(event: Event): void {
      if (event.target instanceof Node && customFieldTypePanelElement?.contains(event.target)) return;
      customFieldTypePickerOpen = false;
    }
    function handleResize(): void {
      computeCustomFieldTypePickerPosition();
    }
    function handleKeydown(event: KeyboardEvent): void {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      customFieldTypePickerOpen = false;
    }
    window.addEventListener("mousedown", handleClickOutside, true);
    window.addEventListener("scroll", handleScroll, true);
    window.addEventListener("resize", handleResize);
    window.addEventListener("keydown", handleKeydown, true);
    return () => {
      window.removeEventListener("mousedown", handleClickOutside, true);
      window.removeEventListener("scroll", handleScroll, true);
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("keydown", handleKeydown, true);
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

{#snippet newRowDragHandle(showIcon: boolean)}
  {#if showIcon}
    <div
      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground opacity-40"
      aria-hidden="true"
    >
      <GripVertical size={13} strokeWidth={1.75} />
    </div>
  {:else}
    <div class="h-7 w-7 shrink-0" aria-hidden="true"></div>
  {/if}
{/snippet}

{#snippet customFieldTypeIcon(fieldType: ProjectCustomFieldType)}
  {#if fieldType === "text"}
    <TextAlignStart size={14} strokeWidth={1.8} />
  {:else if fieldType === "number"}
    <Binary size={15.5} strokeWidth={1.8} />
  {:else if fieldType === "select"}
    <List size={14} strokeWidth={1.8} />
  {:else if fieldType === "multi_select"}
    <ListChecks size={14} strokeWidth={1.8} />
  {:else if fieldType === "status"}
    <CircleCheck size={14} strokeWidth={1.8} />
  {:else if fieldType === "date"}
    <CalendarDays size={14} strokeWidth={1.8} />
  {:else if fieldType === "person"}
    <UserRound size={14} strokeWidth={1.8} />
  {:else if fieldType === "files"}
    <Paperclip size={14} strokeWidth={1.8} />
  {:else if fieldType === "checkbox"}
    <SquareCheckBig size={14} strokeWidth={1.8} />
  {:else if fieldType === "url"}
    <Link2 size={14} strokeWidth={1.8} />
  {:else if fieldType === "phone"}
    <Phone size={14} strokeWidth={1.8} />
  {:else}
    <Mail size={14} strokeWidth={1.8} />
  {/if}
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

  <form class="flex min-h-0 flex-1 flex-col" onsubmit={(event) => { event.preventDefault(); }}>
    {#if projectSettingsDraftReady}
    <div class="relative min-h-0 flex-1">
      <div
        bind:this={settingsScrollElement}
        data-settings-content
        class={cn(
          "project-settings-scroll-area hide-scrollbar h-full min-h-0 overflow-y-auto px-3 pb-8 pt-1",
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
            bind:projectEventTimeModeDraft
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
          />

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-0.5">
            {@render sectionHeading(t("projects.settings.tags"))}
            <div class="flex flex-col gap-2">
              {#each projectTags as tag (tag.id)}
                <div
                  class={cn(
                    "relative grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-1.5 px-1 py-0.5",
                    draggedTagId === tag.id && "opacity-50",
                  )}
                  role="group"
                  aria-label={tag.name}
                  ondragover={(event) => handleTagDragOver(event, tag, event.currentTarget)}
                  ondrop={(event) => { void dropTag(event, tag); }}
                >
                  {#if tagDropMarkerVisible(tag.id, "before")}
                    <div class="pointer-events-none absolute left-1 right-1 top-0 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  {#if tagDropMarkerVisible(tag.id, "after")}
                    <div class="pointer-events-none absolute bottom-0 left-1 right-1 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                    draggable={projectTags.length > 1 && !tagReorderPending}
                    disabled={projectTags.length <= 1 || tagReorderPending}
                    aria-label={t("projects.actions.dragTag", tag.name)}
                    ondragstart={(event) => handleTagDragStart(event, tag)}
                    ondragend={clearTagDrag}
                    onkeydown={(event) => {
                      if (event.key === "ArrowUp") {
                        event.preventDefault();
                        void moveTagByDirection(tag, -1);
                      }
                      if (event.key === "ArrowDown") {
                        event.preventDefault();
                        void moveTagByDirection(tag, 1);
                      }
                    }}
                  >
                    <GripVertical size={13} strokeWidth={1.75} />
                  </button>
                  <ColorPicker
                    color={tagColorDraftValue(tag)}
                    theme={theme.current}
                    title={t("projects.settings.tagColor")}
                    ariaLabel={t("projects.settings.selectTagColor", tag.name)}
                    class="h-7 w-7 justify-center self-center"
                    buttonClass="size-6 rounded-md"
                    onselect={(color) => setTagColor(tag.id, color)}
                  />
                  <input
                    value={tagNameDraftValue(tag)}
                    class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                    aria-label={t("projects.settings.tagName")}
                    oninput={(event) => {
                      tagNameDrafts = {
                        ...tagNameDrafts,
                        [tag.id]: event.currentTarget.value,
                      };
                    }}
                  />
                  <button
                    type="button"
                    class={iconButtonClass("danger")}
                    aria-label={t("projects.actions.deleteTag", tag.name)}
                    title={t("projects.actions.deleteTag", tag.name)}
                    onclick={() => requestDeleteTag(tag)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>
              {/each}

              <div
                bind:this={newTagRowElement}
                class="grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-1.5 px-1 py-0.5"
              >
                {@render newRowDragHandle(projectTags.length === 0)}
                <ColorPicker
                  color={newTagColor}
                  theme={theme.current}
                  title={t("projects.settings.tagColor")}
                  ariaLabel={t("projects.settings.selectNewTagColor")}
                  class="h-7 w-7 justify-center self-center"
                  buttonClass="size-6 rounded-md"
                  onselect={setNewTagColor}
                />
                <input
                  bind:value={newTagName}
                  class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                  placeholder={t("projects.settings.newTagPlaceholder")}
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitTag();
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                  aria-label={t("projects.settings.addTag")}
                  title={t("projects.settings.addTag")}
                  onclick={() => { void submitTag(); }}
                >
                  <Plus size={13} strokeWidth={1.75} />
                </button>
              </div>
            </div>
          </section>

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-0.5">
            {@render sectionHeading(t("projects.customFields.title"))}
            <div class="flex flex-col gap-2">
              {#each projectCustomFields as field (field.id)}
                {@const fieldOptions = customFieldOptions(field)}
                {@const fieldOptionDraftRows = customFieldOptionCreateDraftRows(field.id)}
                {@const displayedFieldOptionCount = fieldOptions.length + fieldOptionDraftRows.length}
                <div class="custom-field-config flex flex-col gap-2">
                  <div
                    class={cn(
                      "relative grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-1.5 px-1 py-0.5",
                      draggedCustomFieldId === field.id && "opacity-50",
                    )}
                    role="group"
                    aria-label={field.name}
                    ondragover={(event) => handleCustomFieldDragOver(event, field, event.currentTarget)}
                    ondrop={(event) => { void dropCustomField(event, field); }}
                  >
                    {#if customFieldDropMarkerVisible(field.id, "before")}
                      <div class="pointer-events-none absolute left-1 right-1 top-0 h-0.5 rounded-full bg-primary"></div>
                    {/if}
                    {#if customFieldDropMarkerVisible(field.id, "after")}
                      <div class="pointer-events-none absolute bottom-0 left-1 right-1 h-0.5 rounded-full bg-primary"></div>
                    {/if}
                    <button
                      type="button"
                      class="flex h-7 w-7 shrink-0 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                      draggable={projectCustomFields.length > 1 && !customFieldReorderPending}
                      disabled={projectCustomFields.length <= 1 || customFieldReorderPending}
                      aria-label={t("projects.actions.dragCustomField", field.name)}
                      ondragstart={(event) => handleCustomFieldDragStart(event, field)}
                      ondragend={clearCustomFieldDrag}
                      onkeydown={(event) => {
                        if (event.key === "ArrowUp") {
                          event.preventDefault();
                          void moveProjectCustomField(field, -1);
                        }
                        if (event.key === "ArrowDown") {
                          event.preventDefault();
                          void moveProjectCustomField(field, 1);
                        }
                      }}
                    >
                      <GripVertical size={13} strokeWidth={1.75} />
                    </button>
                    <input
                      value={customFieldNameDraftValue(field)}
                      class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                      aria-label={t("projects.customFields.fieldName")}
                      oninput={(event) => {
                        customFieldNameDrafts = {
                          ...customFieldNameDrafts,
                          [field.id]: event.currentTarget.value,
                        };
                      }}
                    />
                    <div
                      class="flex h-7 w-32 shrink-0 cursor-not-allowed items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-[0.8rem] font-medium text-foreground transition-colors dark:bg-transparent"
                      aria-label={t("projects.customFields.fieldType")}
                      data-app-tooltip={t("projects.customFields.typeLockedTooltip")}
                    >
                      <span class="min-w-0 flex-1 truncate">{projectCustomFieldTypeLabel(field.fieldType, t)}</span>
                      <ChevronDown size={13} strokeWidth={2} class="shrink-0 text-muted-foreground" />
                    </div>
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
                    <div class="flex flex-col gap-2 pl-9 pr-1">
                      {#if displayedFieldOptionCount > 0}
                        <div class="custom-field-option-branch flex flex-col gap-2">
                          <svg
                            class="custom-field-option-connector"
                            aria-hidden="true"
                            viewBox={customFieldOptionConnectorViewBox(displayedFieldOptionCount)}
                            style={customFieldOptionConnectorStyle(displayedFieldOptionCount)}
                          >
                            <path d={customFieldOptionConnectorPath(displayedFieldOptionCount)} />
                          </svg>
                          {#each fieldOptions as option (option.id)}
                            <div
                              class={cn(
                                "relative grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 py-0.5",
                                draggedCustomFieldOptionId === option.id && "opacity-50",
                              )}
                              role="group"
                              aria-label={option.name}
                              ondragover={(event) => handleCustomFieldOptionDragOver(event, option, event.currentTarget)}
                              ondrop={(event) => { void dropCustomFieldOption(event, option); }}
                            >
                              {#if customFieldOptionDropMarkerVisible(option.id, "before")}
                                <div class="pointer-events-none absolute left-0 right-0 top-0 h-0.5 rounded-full bg-primary"></div>
                              {/if}
                              {#if customFieldOptionDropMarkerVisible(option.id, "after")}
                                <div class="pointer-events-none absolute bottom-0 left-0 right-0 h-0.5 rounded-full bg-primary"></div>
                              {/if}
                              <button
                                type="button"
                                class="flex h-7 w-7 shrink-0 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                                draggable={fieldOptions.length > 1 && !customFieldOptionReorderPending}
                                disabled={fieldOptions.length <= 1 || customFieldOptionReorderPending}
                                aria-label={t("projects.actions.dragCustomFieldOption", option.name)}
                                ondragstart={(event) => handleCustomFieldOptionDragStart(event, option)}
                                ondragend={clearCustomFieldOptionDrag}
                                onkeydown={(event) => {
                                  if (event.key === "ArrowUp") {
                                    event.preventDefault();
                                    void moveProjectCustomFieldOption(option, -1);
                                  }
                                  if (event.key === "ArrowDown") {
                                    event.preventDefault();
                                    void moveProjectCustomFieldOption(option, 1);
                                  }
                                }}
                              >
                                <GripVertical size={13} strokeWidth={1.75} />
                              </button>
                              <input
                                value={customFieldOptionNameDraftValue(option)}
                                class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                                aria-label={t("projects.customFields.optionName")}
                                oninput={(event) => {
                                  customFieldOptionNameDrafts = {
                                    ...customFieldOptionNameDrafts,
                                    [option.id]: event.currentTarget.value,
                                  };
                                }}
                              />
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
                          {/each}
                          {#each fieldOptionDraftRows as option (option.id)}
                            <div
                              class="relative grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 py-0.5"
                              role="group"
                              aria-label={option.name || t("projects.customFields.optionName")}
                            >
                              <div
                                class="flex h-7 w-7 shrink-0 cursor-not-allowed items-center justify-center rounded-md text-muted-foreground opacity-40"
                                aria-hidden="true"
                              >
                                <GripVertical size={13} strokeWidth={1.75} />
                              </div>
                              <input
                                value={option.name}
                                class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                                aria-label={t("projects.customFields.optionName")}
                                oninput={(event) => {
                                  setCustomFieldOptionCreateDraftName(field.id, option.id, event.currentTarget.value);
                                }}
                              />
                              <button
                                type="button"
                                class={iconButtonClass("danger")}
                                aria-label={t("projects.actions.deleteCustomFieldOption", option.name)}
                                title={t("projects.actions.deleteCustomFieldOption", option.name)}
                                onclick={() => removeCustomFieldOptionCreateDraft(field.id, option.id)}
                              >
                                <Trash2 size={13} strokeWidth={1.75} />
                              </button>
                            </div>
                          {/each}
                        </div>
                      {/if}
                      <div class="grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 py-0.5">
                        {@render newRowDragHandle(displayedFieldOptionCount === 0)}
                        <input
                          value={newCustomFieldOptionDrafts[field.id] ?? ""}
                          class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
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
              {/each}

              <div class="flex flex-col gap-2">
                <div
                  bind:this={newCustomFieldRowElement}
                  class="grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-1.5 px-1 py-0.5"
                >
                  {@render newRowDragHandle(projectCustomFields.length === 0)}
                  <input
                    bind:value={newCustomFieldName}
                    class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                    placeholder={t("projects.customFields.newFieldPlaceholder")}
                    onkeydown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void submitCustomField();
                      }
                    }}
                  />
                  <div class="relative min-w-0 w-32">
                    <button
                      bind:this={customFieldTypeTriggerElement}
                      type="button"
                      class="flex h-7 w-full max-w-full items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground transition-colors hover:bg-accent dark:bg-transparent"
                      aria-haspopup="listbox"
                      aria-expanded={customFieldTypePickerOpen}
                      aria-label={t("projects.customFields.fieldType")}
                      onclick={() => { void toggleCustomFieldTypePicker(); }}
                    >
                      <span class="min-w-0 flex-1 truncate">{projectCustomFieldTypeLabel(newCustomFieldType, t)}</span>
                      <ChevronDown
                        size={13}
                        strokeWidth={2}
                        class={cn("shrink-0 text-muted-foreground transition-transform", customFieldTypePickerOpen && "rotate-180")}
                      />
                    </button>
                    {#if customFieldTypePickerOpen}
                      <div
                        bind:this={customFieldTypePanelElement}
                        use:portal
                        role="listbox"
                        data-app-floating-surface
                        class="fixed z-80 overflow-x-hidden overflow-y-auto rounded-lg border border-border bg-popover p-1.5 shadow-lg"
                        style={customFieldTypePickerStyle()}
                      >
                        <div class="grid min-w-64 grid-cols-2 gap-1">
                          {#each PROJECT_CUSTOM_FIELD_TYPES as fieldType}
                            {@const selected = fieldType === newCustomFieldType}
                            <button
                              type="button"
                              role="option"
                              aria-selected={selected}
                              class={cn(
                                "flex min-h-9 min-w-0 cursor-pointer items-center gap-2 rounded-md px-2 text-left text-[0.8rem] transition-colors",
                                selected
                                  ? "bg-accent/70 font-semibold text-foreground"
                                  : "text-foreground hover:bg-accent/45",
                              )}
                              onclick={() => selectNewCustomFieldType(fieldType)}
                            >
                              <span class="flex h-5 w-5 shrink-0 items-center justify-center text-muted-foreground">
                                {@render customFieldTypeIcon(fieldType)}
                              </span>
                              <span class="min-w-0 truncate">{projectCustomFieldTypeLabel(fieldType, t)}</span>
                            </button>
                          {/each}
                        </div>
                      </div>
                    {/if}
                  </div>
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

              {#if newCustomFieldAcceptsOptions()}
                <div class="flex flex-col gap-2 pl-9 pr-1">
                  {#each newCustomFieldOptionRows as option (option.id)}
                    <div class="grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 py-0.5">
                      <div
                        class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-muted-foreground opacity-40"
                        aria-hidden="true"
                      >
                        <GripVertical size={13} strokeWidth={1.75} />
                      </div>
                      <input
                        value={option.name}
                        class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                        aria-label={t("projects.customFields.optionName")}
                        oninput={(event) => setNewCustomFieldOptionDraftName(option.id, event.currentTarget.value)}
                      />
                      <button
                        type="button"
                        class={iconButtonClass("danger")}
                        aria-label={t("projects.actions.deleteCustomFieldOption", option.name)}
                        title={t("projects.actions.deleteCustomFieldOption", option.name)}
                        onclick={() => removeNewCustomFieldOptionDraft(option.id)}
                      >
                        <Trash2 size={13} strokeWidth={1.75} />
                      </button>
                    </div>
                  {/each}
                  <div class="grid min-h-7 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 py-0.5">
                    {@render newRowDragHandle(newCustomFieldOptionRows.length === 0)}
                    <input
                      bind:value={newCustomFieldOptionName}
                      class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                      placeholder={t("projects.customFields.newOptionPlaceholder")}
                      onkeydown={(event) => {
                        if (event.key === "Enter") {
                          event.preventDefault();
                          submitNewCustomFieldOptionDraft();
                        }
                      }}
                    />
                    <button
                      type="button"
                      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                      aria-label={t("projects.customFields.addOption")}
                      title={t("projects.customFields.addOption")}
                      onclick={submitNewCustomFieldOptionDraft}
                    >
                      <Plus size={13} strokeWidth={1.75} />
                    </button>
                  </div>
                </div>
              {/if}
              </div>
            </div>
          </section>

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-0.5">
            {@render sectionHeading(t("projects.settings.taskStatuses"))}
            <div class="flex flex-col gap-0.5">
              {#each statuses as status (status.id)}
                {@const deleteStatusTitle = statusDeleteTitle(status)}
                <div
                  class={cn(
                    "relative grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-1 px-1 py-0.5",
                    draggedStatusId === status.id && "opacity-50",
                  )}
                  role="group"
                  aria-label={status.name}
                  ondragover={(event) => handleStatusDragOver(event, status, event.currentTarget)}
                  ondrop={(event) => { void dropStatus(event, status); }}
                >
                  {#if statusDropMarkerVisible(status.id, "before")}
                    <div class="pointer-events-none absolute left-1 right-1 top-0 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  {#if statusDropMarkerVisible(status.id, "after")}
                    <div class="pointer-events-none absolute bottom-0 left-1 right-1 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                    draggable={statuses.length > 1 && !statusReorderPending}
                    disabled={statuses.length <= 1 || statusReorderPending}
                    aria-label={t("projects.actions.dragStatus", status.name)}
                    ondragstart={(event) => handleStatusDragStart(event, status)}
                    ondragend={clearStatusDrag}
                    onkeydown={(event) => {
                      if (event.key === "ArrowUp") {
                        event.preventDefault();
                        void moveStatusByDirection(status, -1);
                      }
                      if (event.key === "ArrowDown") {
                        event.preventDefault();
                        void moveStatusByDirection(status, 1);
                      }
                    }}
                  >
                    <GripVertical size={13} strokeWidth={1.75} />
                  </button>
                  <ColorPicker
                    color={statusColorDraftValue(status)}
                    theme={theme.current}
                    title={t("projects.settings.statusColor")}
                    ariaLabel={t("projects.settings.selectStatusColor", status.name)}
                    class="h-7 w-7 justify-center self-center"
                    buttonClass="size-6 rounded-md"
                    onselect={(color) => setStatusColor(status.id, color)}
                  />
                  <input
                    value={statusNameDrafts[status.id] ?? status.name}
                    class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                    aria-label={t("projects.settings.statusName")}
                    oninput={(event) => {
                      statusNameDrafts = {
                        ...statusNameDrafts,
                        [status.id]: event.currentTarget.value,
                      };
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
                    class={iconButtonClass("danger")}
                    disabled={statusDeleteDisabled(status)}
                    aria-label={deleteStatusTitle}
                    title={deleteStatusTitle}
                    onclick={() => requestDeleteStatus(status)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>
              {/each}
            </div>

            <div
              bind:this={newStatusRowElement}
              class="grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto_auto] items-center gap-1 px-1 py-0.5"
            >
              {@render newRowDragHandle(statuses.length === 0)}
              <ColorPicker
                color={newStatusColor}
                theme={theme.current}
                title={t("projects.settings.statusColor")}
                ariaLabel={t("projects.settings.selectNewStatusColor")}
                class="h-7 w-7 justify-center self-center"
                buttonClass="size-6 rounded-md"
                onselect={setNewStatusColor}
              />
              <input
                bind:value={newStatusName}
                class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
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

          <div class="h-px bg-border/70" aria-hidden="true"></div>

          <section class="flex flex-col gap-0.5">
            {@render sectionHeading(t("projects.settings.taskPriorities"))}
            <div class="flex flex-col gap-2">
              {#each priorities as priority (priority.id)}
                {@const deletePriorityTitle = priorityDeleteTitle(priority)}
                <div
                  class={cn(
                    "relative grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-1.5 px-1 py-0.5",
                    draggedPriorityId === priority.id && "opacity-50",
                  )}
                  role="group"
                  aria-label={priority.name}
                  ondragover={(event) => handlePriorityDragOver(event, priority, event.currentTarget)}
                  ondrop={(event) => { void dropPriority(event, priority); }}
                >
                  {#if priorityDropMarkerVisible(priority.id, "before")}
                    <div class="pointer-events-none absolute left-1 right-1 top-0 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  {#if priorityDropMarkerVisible(priority.id, "after")}
                    <div class="pointer-events-none absolute bottom-0 left-1 right-1 h-0.5 rounded-full bg-primary"></div>
                  {/if}
                  <button
                    type="button"
                    class="flex h-7 w-7 shrink-0 cursor-grab items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
                    draggable={priorities.length > 1 && !priorityReorderPending}
                    disabled={priorities.length <= 1 || priorityReorderPending}
                    aria-label={t("projects.actions.dragPriority", priority.name)}
                    ondragstart={(event) => handlePriorityDragStart(event, priority)}
                    ondragend={clearPriorityDrag}
                    onkeydown={(event) => {
                      if (event.key === "ArrowUp") {
                        event.preventDefault();
                        void movePriorityByDirection(priority, -1);
                      }
                      if (event.key === "ArrowDown") {
                        event.preventDefault();
                        void movePriorityByDirection(priority, 1);
                      }
                    }}
                  >
                    <GripVertical size={13} strokeWidth={1.75} />
                  </button>
                  <ColorPicker
                    color={priorityColorDraftValue(priority)}
                    theme={theme.current}
                    title={t("projects.settings.priorityColor")}
                    ariaLabel={t("projects.settings.selectPriorityColor", priority.name)}
                    class="h-7 w-7 justify-center self-center"
                    buttonClass="size-6 rounded-md"
                    onselect={(color) => setPriorityColor(priority.id, color)}
                  />
                  <input
                    value={priorityNameDrafts[priority.id] ?? priority.name}
                    class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                    aria-label={t("projects.settings.priorityName")}
                    oninput={(event) => {
                      priorityNameDrafts = {
                        ...priorityNameDrafts,
                        [priority.id]: event.currentTarget.value,
                      };
                    }}
                  />
                  <button
                    type="button"
                    class={iconButtonClass("danger")}
                    disabled={priorityDeleteDisabled(priority)}
                    aria-label={deletePriorityTitle}
                    title={deletePriorityTitle}
                    onclick={() => requestDeletePriority(priority)}
                  >
                    <Trash2 size={13} strokeWidth={1.75} />
                  </button>
                </div>
              {/each}

              <div
                bind:this={newPriorityRowElement}
                class="grid min-h-7 grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-1.5 px-1 py-0.5"
              >
                {@render newRowDragHandle(priorities.length === 0)}
                <ColorPicker
                  color={newPriorityColor}
                  theme={theme.current}
                  title={t("projects.settings.priorityColor")}
                  ariaLabel={t("projects.settings.selectNewPriorityColor")}
                  class="h-7 w-7 justify-center self-center"
                  buttonClass="size-6 rounded-md"
                  onselect={setNewPriorityColor}
                />
                <input
                  bind:value={newPriorityName}
                  class="h-7 min-w-0 rounded-md border border-border bg-background px-2 text-[0.8rem] text-foreground outline-none transition-colors focus:border-ring placeholder:text-muted-foreground"
                  placeholder={t("projects.settings.newPriorityPlaceholder")}
                  onkeydown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void submitPriority();
                    }
                  }}
                />
                <button
                  type="button"
                  class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
                  aria-label={t("projects.settings.addPriority")}
                  title={t("projects.settings.addPriority")}
                  onclick={() => { void submitPriority(); }}
                >
                  <Plus size={13} strokeWidth={1.75} />
                </button>
              </div>
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
        type="button"
        class="flex min-h-8 shrink-0 items-center gap-1.5 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
        disabled={projectSettingsSaving || !projectSettingsDirty}
        onclick={() => { void saveProjectSettings(); }}
      >
        <Save size={14} strokeWidth={1.75} />
        <span>{projectSettingsSaving ? t("common.loading") : t("projects.settings.save")}</span>
      </button>
    </footer>
    {/if}
  </form>
</aside>
{/if}

{#if pendingDeleteStatus}
  <ConfirmDialog
    title={t("projects.settings.deleteStatusTitle", pendingDeleteStatus.name)}
    message={t("projects.settings.deleteStatusMessage", pendingDeleteStatus.name)}
    confirmLabel={t("projects.settings.deleteStatusConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteStatus(); }}
    onCancel={cancelDeleteStatus}
  />
{/if}

{#if pendingDeletePriority}
  <ConfirmDialog
    title={t("projects.settings.deletePriorityTitle", pendingDeletePriority.name)}
    message={t("projects.settings.deletePriorityMessage", pendingDeletePriority.name)}
    confirmLabel={t("projects.settings.deletePriorityConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeletePriority(); }}
    onCancel={cancelDeletePriority}
  />
{/if}

{#if pendingDeleteTag}
  <ConfirmDialog
    title={t("projects.settings.deleteTagTitle", pendingDeleteTag.name)}
    message={t("projects.settings.deleteTagMessage", pendingDeleteTag.name)}
    confirmLabel={t("projects.settings.deleteTagConfirm")}
    cancelLabel={t("common.cancelShortcut")}
    onConfirm={() => { void confirmDeleteTag(); }}
    onCancel={cancelDeleteTag}
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

  .custom-field-config {
    --custom-field-option-branch-color: color-mix(
      in srgb,
      var(--muted-foreground) 54%,
      var(--card)
    );
  }

  .custom-field-option-branch {
    position: relative;
  }

  .custom-field-option-connector {
    position: absolute;
    pointer-events: none;
    overflow: visible;
  }

  .custom-field-option-connector path {
    fill: none;
    stroke: var(--custom-field-option-branch-color);
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-width: 0.125;
  }
</style>
