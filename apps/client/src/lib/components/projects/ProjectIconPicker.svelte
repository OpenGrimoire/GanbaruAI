<script lang="ts">
  import { tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import CircleEllipsis from "@lucide/svelte/icons/circle-ellipsis";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LayoutGrid from "@lucide/svelte/icons/layout-grid";
  import Leaf from "@lucide/svelte/icons/leaf";
  import Package from "@lucide/svelte/icons/package";
  import Plane from "@lucide/svelte/icons/plane";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Shapes from "@lucide/svelte/icons/shapes";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Smile from "@lucide/svelte/icons/smile";
  import Trophy from "@lucide/svelte/icons/trophy";
  import Utensils from "@lucide/svelte/icons/utensils";
  import Upload from "@lucide/svelte/icons/upload";
  import UserRound from "@lucide/svelte/icons/user-round";
  import X from "@lucide/svelte/icons/x";
  import {
    deleteProjectIconAssetsIfUnreferenced,
    downloadProjectIconImageUrl,
    pickProjectIconImageFile,
    saveProjectIconImageDataUrl,
    type ProjectIconAsset,
  } from "$lib/api/project-icons";
  import {
    FALLBACK_COLOR_INDEX,
    type EventColor,
  } from "$lib/components/calendar/types";
  import { EVENT_COLOR_OPTIONS, getEventColor } from "$lib/components/calendar/utils";
  import { contrastRatio } from "$lib/components/ui/colorMath";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    PROJECT_EMOJI_CATEGORIES,
    PROJECT_EMOJI_ENTRIES,
    type ProjectEmojiCategoryId,
  } from "$lib/projects/project-emoji-catalog";
  import {
    applyProjectEmojiSkinTone,
    cleanupProjectIconRecentValues,
    filterProjectEmojiEntries,
    filterProjectLucideIcons,
    prependProjectIconRecentValue,
    projectEmojiSkinToneFromEmoji,
    projectIconPickerPanelPlacement,
    stripProjectEmojiSkinTone,
    type ProjectEmojiSkinTone,
    type ProjectIconPickerRect,
  } from "$lib/projects/project-icon-picker";
  import {
    parseProjectIcon,
    projectIconColorToEventColor,
    projectIconDisplayLabel,
    serializeProjectIcon,
    type ProjectIconValue,
  } from "$lib/projects/project-icons";
  import type {
    ProjectLucideCategory,
    ProjectLucideIconEntry,
    ProjectLucideIconNode,
  } from "$lib/projects/project-lucide-catalog.generated";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getTheme } from "$lib/stores/theme.svelte";
  import { resolveAppTokens } from "$lib/stores/themes";
  import { cn } from "$lib/utils";
  import { portal } from "$lib/utils/portal";
  import {
    ensureConfigLoaded,
    getConfigKey,
    setConfigKey,
  } from "$lib/vault/config";
  import LucideNodeIcon from "./LucideNodeIcon.svelte";
  import ProjectIcon from "./ProjectIcon.svelte";

  type ProjectIconPickerTab = "emoji" | "icons" | "upload";
  type IconColorChoicePlacement = { left: number; top: number };
  type LucideIconGroup = {
    category: ProjectLucideCategory;
    entries: readonly ProjectLucideIconEntry[];
  };
  type LucideVirtualGroup = LucideIconGroup & {
    beforeRowsHeight: number;
    afterRowsHeight: number;
  };
  type LucideGroupVirtualWindow = {
    groups: readonly LucideVirtualGroup[];
    beforeHeight: number;
    afterHeight: number;
  };
  type IconColorChoice = {
    slug: string;
    label: string;
    iconNode: readonly ProjectLucideIconNode[] | null;
    anchor: HTMLElement;
    placement: IconColorChoicePlacement;
  };
  type VisibleEmojiCategory = {
    id: ProjectEmojiCategoryId;
    label: string;
  };

  let {
    value,
    onChange,
    ariaLabel,
    allowIconColors = true,
    class: className = "",
  }: {
    value: string;
    onChange: (value: string) => void;
    ariaLabel: string;
    allowIconColors?: boolean;
    class?: string;
  } = $props();

  const { t } = getLocalization();
  const projects = getProjects();
  const theme = getTheme();
  const recentConfigKey = "projects.iconPicker.recent";
  const panelWidth = 360;
  const panelPreferredHeight = 440;
  const uploadPanelPreferredHeight = 228;
  const uploadDraftPanelPreferredHeight = 276;
  const uploadWarningHeight = 72;
  const customPanelWidth = 330;
  const iconColorChoicePanelWidth = 260;
  const iconColorChoicePanelHeight = 144;
  const iconCategoryMenuWidth = 240;
  const iconCategoryMenuMaxHeight = 280;
  const gridCellSize = 36;
  const gridRowHeight = 36;
  const gridGroupHeaderHeight = 24;
  const gridGroupGapHeight = 12;

  const skinToneOptions: readonly { value: ProjectEmojiSkinTone }[] = [
    { value: "default" },
    { value: "light" },
    { value: "medium-light" },
    { value: "medium" },
    { value: "medium-dark" },
    { value: "dark" },
  ];
  const lucideCategoryIconSlugs: Partial<Record<ProjectLucideCategory, string>> = {
    Accessibility: "accessibility",
    "Accounts and access": "user-round",
    Animals: "paw-print",
    Arrows: "arrow-up-right",
    Buildings: "building-2",
    Charts: "chart-no-axes-column-increasing",
    Communication: "message-circle",
    Connectivity: "wifi",
    Cursors: "mouse-pointer-2",
    Design: "paintbrush",
    "Coding and development": "code-xml",
    Devices: "monitor",
    Emoji: "laugh",
    "File icons": "file",
    Finance: "badge-dollar-sign",
    "Food and beverage": "utensils",
    Gaming: "gamepad-2",
    Home: "house",
    Layout: "layout-grid",
    Mail: "mail",
    Mathematics: "sigma",
    Medical: "cross",
    Multimedia: "video",
    Nature: "leaf",
    "Navigation and places": "plane",
    Notification: "bell",
    People: "users",
    Photography: "camera",
    Science: "flask-conical",
    Seasons: "snowflake",
    Security: "shield",
    Shapes: "shapes",
    Shopping: "shopping-cart",
    Social: "share-2",
    Sports: "dumbbell",
    Sustainability: "recycle",
    "Text formatting": "type",
    "Time and calendar": "calendar",
    Tools: "wrench",
    Transportation: "car",
    Travel: "luggage",
    Weather: "cloud-sun",
  };
  const primaryLucideCategories: readonly ProjectLucideCategory[] = [
    "File icons",
    "Tools",
    "Coding and development",
    "Design",
    "Charts",
    "Communication",
    "Time and calendar",
    "Navigation and places",
  ];
  const primaryLucideCategorySet = new Set<ProjectLucideCategory>(primaryLucideCategories);
  const visibleEmojiCategoryIds = new Set<ProjectEmojiCategoryId>([
    "smileys",
    "people",
    "nature",
    "food",
    "activity",
    "travel",
    "objects",
    "symbols",
  ]);

  let open = $state(false);
  let activeTab = $state<ProjectIconPickerTab>("icons");
  let triggerElement = $state<HTMLButtonElement | undefined>();
  let panelElement = $state<HTMLElement | undefined>();
  let customPanelElement = $state<HTMLElement | undefined>();
  let customEmojiTriggerElement = $state<HTMLButtonElement | undefined>();
  let iconColorChoicePanelElement = $state<HTMLElement | undefined>();
  let iconCategoryMenuElement = $state<HTMLElement | undefined>();
  let iconCategoryMenuTriggerElement = $state<HTMLButtonElement | undefined>();
  let gridScrollElement = $state<HTMLElement | undefined>();
  let panelPlacement = $state({ left: 0, top: 0, width: panelWidth, height: panelPreferredHeight });
  let customPanelPlacement = $state({ left: 0, top: 0, maxHeight: 360 });
  let iconCategoryMenuPlacement = $state({ left: 0, top: 0, maxHeight: iconCategoryMenuMaxHeight });
  let emojiQuery = $state("");
  let iconQuery = $state("");
  let uploadUrl = $state("");
  let uploadDraft = $state<ProjectIconAsset | null>(null);
  let uploadError = $state<string | null>(null);
  let uploading = $state(false);
  let iconColor = $state<EventColor>(FALLBACK_COLOR_INDEX);
  let emojiSkinTone = $state<ProjectEmojiSkinTone>("default");
  let emojiCategory = $state<ProjectEmojiCategoryId | "all">("all");
  let iconCategory = $state<ProjectLucideCategory | "all">("all");
  let lucideIcons = $state<readonly ProjectLucideIconEntry[]>([]);
  let lucideCategories = $state<readonly ProjectLucideCategory[]>([]);
  let lucideLoaded = $state(false);
  let lucideLoading = $state(false);
  let customEmojiPanelOpen = $state(false);
  let customEmojiName = $state("");
  let customEmojiDraft = $state<ProjectIconAsset | null>(null);
  let customEmojiError = $state<string | null>(null);
  let customEmojiSaving = $state(false);
  let skinTonePanelOpen = $state(false);
  let iconColorPanelOpen = $state(false);
  let iconCategoryMenuOpen = $state(false);
  let askIconColorEveryTime = $state(true);
  let iconColorChoice = $state<IconColorChoice | null>(null);
  let recentValues = $state<string[]>([]);
  let gridScrollTop = $state(0);
  let gridScrollable = $state(false);
  let gridCanScrollUp = $state(false);
  let gridCanScrollDown = $state(false);
  let gridViewportHeight = $state(260);
  let gridColumnCount = $state(8);
  let gridScrollStateFrame: number | null = null;

  const parsedValue = $derived(parseProjectIcon(value));
  const customEmojiIds = $derived(new Set(projects.customEmojis.map((emoji) => emoji.id)));
  const pickerLabel = $derived.by(() => {
    if (parsedValue.kind === "emoji") {
      const baseEmoji = stripProjectEmojiSkinTone(parsedValue.emoji);
      return PROJECT_EMOJI_ENTRIES.find((entry) => entry.emoji === baseEmoji)?.name
        ?? t("projects.iconPicker.emoji");
    }
    if (parsedValue.kind === "custom-emoji") {
      return projects.customEmojis.find((emoji) => emoji.id === parsedValue.id)?.name ?? projectIconDisplayLabel(parsedValue);
    }
    return projectIconDisplayLabel(parsedValue);
  });
  const emojiRecentValues = $derived(
    recentValues.filter((rawValue) => {
      const icon = parseProjectIcon(rawValue);
      return icon.kind === "emoji" || icon.kind === "custom-emoji";
    }),
  );
  const lucideRecentValues = $derived(
    recentValues.filter((rawValue) => parseProjectIcon(rawValue).kind === "lucide"),
  );
  const visibleCustomEmojis = $derived.by(() => {
    const query = emojiQuery.trim().toLowerCase();
    if (!query) return projects.customEmojis;
    return projects.customEmojis.filter((emoji) => emoji.name.toLowerCase().includes(query));
  });
  const visibleEmojiCategories = $derived.by((): readonly VisibleEmojiCategory[] =>
    PROJECT_EMOJI_CATEGORIES
      .filter((category) => visibleEmojiCategoryIds.has(category.id))
      .map((category) => ({
        id: category.id,
        label: category.id === "symbols" ? t("projects.iconPicker.symbolsAndFlags") : category.label,
      }))
  );
  const emojiGroups = $derived.by(() =>
    visibleEmojiCategories
      .filter((category) => emojiCategory === "all" || emojiCategory === category.id)
      .map((category) => ({
        category,
        entries: filterProjectEmojiEntries(PROJECT_EMOJI_ENTRIES, emojiQuery, category.id),
      }))
      .filter((group) => group.entries.length > 0)
  );
  const filteredEmojiEntries = $derived(
    filterProjectEmojiEntries(PROJECT_EMOJI_ENTRIES, emojiQuery, emojiCategory),
  );
  const filteredLucideEntries = $derived(
    filterProjectLucideIcons(lucideIcons, iconQuery, iconCategory),
  );
  const lucideGroups = $derived.by(() => {
    const groups = new Map<ProjectLucideCategory, ProjectLucideIconEntry[]>();
    for (const category of lucideCategories) {
      groups.set(category, []);
    }
    for (const entry of filteredLucideEntries) {
      const group = groups.get(entry.category);
      if (group) group.push(entry);
    }
    return lucideCategories
      .map((category) => ({
        category,
        entries: groups.get(category) ?? [],
      }))
      .filter((group) => group.entries.length > 0);
  });
  const lucideCategoryOptions = $derived(
    lucideCategories.map((category) => ({
      category,
      icon: lucideCategoryIcon(category),
    })),
  );
  const primaryLucideCategoryOptions = $derived.by(() =>
    primaryLucideCategories
      .map((category) => lucideCategoryOptions.find((option) => option.category === category))
      .filter((option): option is { category: ProjectLucideCategory; icon: ProjectLucideIconEntry | undefined } => option !== undefined),
  );
  const iconCategoryOverflowActive = $derived(
    iconCategory !== "all" && !primaryLucideCategorySet.has(iconCategory),
  );
  const lucideRecentHeight = $derived.by(() => {
    if (lucideRecentValues.length === 0) return 0;
    return gridGroupHeaderHeight
      + (Math.ceil(lucideRecentValues.length / Math.max(1, gridColumnCount)) * gridRowHeight)
      + gridGroupGapHeight;
  });
  const lucideGroupVirtual = $derived(virtualizeLucideGroups(
    lucideGroups,
    gridColumnCount,
    gridViewportHeight,
    Math.max(0, gridScrollTop - lucideRecentHeight),
  ));
  const panelStyle = $derived.by(() => {
    const baseStyle = `left: ${panelPlacement.left}px; top: ${panelPlacement.top}px; width: ${panelPlacement.width}px;`;
    if (activeTab === "upload") return `${baseStyle} max-height: ${panelPlacement.height}px;`;
    return `${baseStyle} height: ${panelPlacement.height}px;`;
  });
  const uploadBodyStyle = $derived(`max-height: ${Math.max(0, panelPlacement.height - 48)}px;`);
  const appTokens = $derived(resolveAppTokens(theme.current));
  const colorSelectionBorder = $derived(
    contrastRatio(appTokens["--popover"], "#000000") >= contrastRatio(appTokens["--popover"], "#ffffff")
      ? "#000000"
      : "#ffffff",
  );
  const customPanelStyle = $derived(
    `left: ${customPanelPlacement.left}px; top: ${customPanelPlacement.top}px; width: ${customPanelWidth}px; max-height: ${customPanelPlacement.maxHeight}px;`,
  );
  const iconCategoryMenuStyle = $derived(
    `left: ${iconCategoryMenuPlacement.left}px; top: ${iconCategoryMenuPlacement.top}px; width: ${iconCategoryMenuWidth}px; max-height: ${iconCategoryMenuPlacement.maxHeight}px;`,
  );

  function readRecentValues(value: unknown): string[] {
    if (!Array.isArray(value)) return [];
    return value.filter((entry): entry is string => typeof entry === "string");
  }

  function tabLabel(tab: ProjectIconPickerTab): string {
    if (tab === "icons") return t("projects.iconPicker.icons");
    if (tab === "upload") return t("projects.iconPicker.upload");
    return t("projects.iconPicker.emoji");
  }

  function iconColorLabel(color: EventColor): string {
    return t("calendar.color.selectEventColor", color + 1);
  }

  function iconColorSwatch(color: EventColor): string {
    return getEventColor(color, theme.current).bg;
  }

  function iconColorStyle(color: EventColor): string {
    return `color: ${iconColorSwatch(color)};`;
  }

  function lucideRecentPreviewValue(rawValue: string): string {
    const icon = parseProjectIcon(rawValue);
    return icon.kind === "lucide"
      ? serializeProjectIcon({ ...icon, color: allowIconColors ? iconColor : "default" })
      : rawValue;
  }

  function iconColorChoiceStyle(choice: IconColorChoice): string {
    return `left: ${choice.placement.left}px; top: ${choice.placement.top}px; width: ${iconColorChoicePanelWidth}px;`;
  }

  function skinToneLabel(skinTone: ProjectEmojiSkinTone): string {
    if (skinTone === "light") return t("projects.iconPicker.skinToneLight");
    if (skinTone === "medium-light") return t("projects.iconPicker.skinToneMediumLight");
    if (skinTone === "medium") return t("projects.iconPicker.skinToneMedium");
    if (skinTone === "medium-dark") return t("projects.iconPicker.skinToneMediumDark");
    if (skinTone === "dark") return t("projects.iconPicker.skinToneDark");
    return t("projects.iconPicker.skinToneDefault");
  }

  function skinTonePreview(skinTone: ProjectEmojiSkinTone): string {
    return applyProjectEmojiSkinTone("✋", skinTone);
  }

  function displayEmoji(emoji: string): string {
    return applyProjectEmojiSkinTone(emoji, emojiSkinTone);
  }

  function resetGridScroll(): void {
    gridScrollTop = 0;
    if (gridScrollElement) {
      gridScrollElement.scrollTop = 0;
      refreshGridScrollState();
    } else {
      requestGridScrollStateRefresh();
    }
  }

  function closeInlinePanels(): void {
    skinTonePanelOpen = false;
    iconColorPanelOpen = false;
    iconCategoryMenuOpen = false;
    iconColorChoice = null;
  }

  function handlePanelPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Element)) return;
    if (target.closest("[data-icon-picker-inline-panel]")) return;
    closeInlinePanels();
  }

  function lucideCategoryIcon(category: ProjectLucideCategory): ProjectLucideIconEntry | undefined {
    const preferredSlug = lucideCategoryIconSlugs[category];
    return lucideIcons.find((entry) => entry.slug === preferredSlug)
      ?? lucideIcons.find((entry) => entry.category === category);
  }

  async function loadRecentValues(): Promise<void> {
    await ensureConfigLoaded();
    recentValues = cleanupProjectIconRecentValues(
      readRecentValues(getConfigKey<unknown>(recentConfigKey, [])),
      customEmojiIds,
    );
  }

  function saveRecentValues(values: string[]): void {
    recentValues = cleanupProjectIconRecentValues(values, customEmojiIds);
    setConfigKey(recentConfigKey, recentValues);
  }

  function updateGridMetrics(): void {
    const element = gridScrollElement;
    if (!element) return;
    gridViewportHeight = element.clientHeight;
    gridColumnCount = Math.max(1, Math.floor(element.clientWidth / gridCellSize));
    requestGridScrollStateRefresh();
  }

  function virtualizeLucideGroups(
    groups: readonly LucideIconGroup[],
    columnCount: number,
    viewportHeight: number,
    scrollTop: number,
  ): LucideGroupVirtualWindow {
    const safeColumnCount = Math.max(1, Math.floor(columnCount));
    const viewportStart = Math.max(0, scrollTop - gridRowHeight * 2);
    const viewportEnd = Math.max(0, scrollTop) + Math.max(0, viewportHeight) + gridRowHeight * 2;
    const visibleGroups: LucideVirtualGroup[] = [];
    let offset = 0;
    let beforeHeight = 0;
    let afterHeight = 0;

    for (const group of groups) {
      const rowCount = Math.ceil(group.entries.length / safeColumnCount);
      const rowsHeight = rowCount * gridRowHeight;
      const groupHeight = gridGroupHeaderHeight + rowsHeight + gridGroupGapHeight;
      const groupStart = offset;
      const groupEnd = groupStart + groupHeight;
      offset = groupEnd;

      if (groupEnd < viewportStart) {
        beforeHeight += groupHeight;
        continue;
      }

      if (groupStart > viewportEnd) {
        afterHeight += groupHeight;
        continue;
      }

      const localStart = Math.max(0, viewportStart - groupStart - gridGroupHeaderHeight);
      const localEnd = Math.min(rowsHeight, viewportEnd - groupStart - gridGroupHeaderHeight);
      const startRow = Math.max(0, Math.floor(localStart / gridRowHeight));
      const endRow = Math.min(
        rowCount,
        Math.max(startRow + 1, Math.ceil(Math.max(0, localEnd) / gridRowHeight)),
      );
      const startIndex = Math.min(group.entries.length, startRow * safeColumnCount);
      const endIndex = Math.min(group.entries.length, endRow * safeColumnCount);
      visibleGroups.push({
        category: group.category,
        entries: group.entries.slice(startIndex, endIndex),
        beforeRowsHeight: startRow * gridRowHeight,
        afterRowsHeight: Math.max(0, (rowCount - endRow) * gridRowHeight),
      });
    }

    return {
      groups: visibleGroups,
      beforeHeight,
      afterHeight,
    };
  }

  function refreshGridScrollState(): void {
    gridScrollStateFrame = null;
    const element = gridScrollElement;
    if (!element) {
      gridScrollable = false;
      gridCanScrollUp = false;
      gridCanScrollDown = false;
      return;
    }
    const maxScrollTop = element.scrollHeight - element.clientHeight;
    gridScrollable = maxScrollTop > 1;
    gridCanScrollUp = element.scrollTop > 1;
    gridCanScrollDown = element.scrollTop < maxScrollTop - 1;
  }

  function requestGridScrollStateRefresh(): void {
    if (gridScrollStateFrame !== null) cancelAnimationFrame(gridScrollStateFrame);
    gridScrollStateFrame = requestAnimationFrame(refreshGridScrollState);
  }

  function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }

  function preferredPanelHeight(): number {
    if (activeTab !== "upload") return panelPreferredHeight;
    const baseHeight = uploadDraft ? uploadDraftPanelPreferredHeight : uploadPanelPreferredHeight;
    return uploadError ? baseHeight + uploadWarningHeight : baseHeight;
  }

  function toPickerRect(rect: DOMRect): ProjectIconPickerRect {
    return {
      top: rect.top,
      right: rect.right,
      bottom: rect.bottom,
      left: rect.left,
      width: rect.width,
      height: rect.height,
    };
  }

  function viewportBoundaryRect(): ProjectIconPickerRect {
    return {
      top: 0,
      right: window.innerWidth,
      bottom: window.innerHeight,
      left: 0,
      width: window.innerWidth,
      height: window.innerHeight,
    };
  }

  function panelBoundaryRect(trigger: HTMLElement): ProjectIconPickerRect {
    const viewport = viewportBoundaryRect();
    const settingsContent = trigger.closest<HTMLElement>("[data-settings-content]");
    if (!settingsContent) return viewport;
    const rect = settingsContent.getBoundingClientRect();
    const top = Math.max(viewport.top, rect.top);
    const bottom = Math.min(viewport.bottom, rect.bottom);
    return {
      top,
      right: viewport.right,
      bottom,
      left: viewport.left,
      width: viewport.width,
      height: Math.max(0, bottom - top),
    };
  }

  async function refreshPanelPlacement(): Promise<void> {
    await tick();
    placePanel();
  }

  function placePanel(): void {
    const trigger = triggerElement;
    if (!trigger) return;
    panelPlacement = projectIconPickerPanelPlacement({
      triggerRect: toPickerRect(trigger.getBoundingClientRect()),
      boundaryRect: panelBoundaryRect(trigger),
      preferredWidth: panelWidth,
      preferredHeight: preferredPanelHeight(),
    });
  }

  function placeCustomPanel(): void {
    const trigger = customEmojiTriggerElement;
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const margin = 8;
    const gap = 4;
    const availableAbove = Math.max(0, rect.top - margin - gap);
    const availableBelow = Math.max(0, viewportHeight - rect.bottom - margin - gap);
    const preferredHeight = 360;
    const minimumUsefulHeight = 180;
    const maxViewportHeight = Math.max(1, viewportHeight - margin * 2);
    const openAbove = availableAbove >= Math.min(preferredHeight, minimumUsefulHeight)
      || availableAbove >= availableBelow;
    const height = Math.min(
      preferredHeight,
      maxViewportHeight,
      Math.max(minimumUsefulHeight, openAbove ? availableAbove : availableBelow),
    );
    customPanelPlacement = {
      left: clamp(
        rect.right - customPanelWidth,
        margin,
        Math.max(margin, viewportWidth - customPanelWidth - margin),
      ),
      top: openAbove
        ? Math.max(margin, rect.top - height - gap)
        : clamp(rect.bottom + gap, margin, Math.max(margin, viewportHeight - height - margin)),
      maxHeight: height,
    };
  }

  function placeIconCategoryMenu(): void {
    const trigger = iconCategoryMenuTriggerElement;
    if (!trigger) return;
    const rect = trigger.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const margin = 8;
    const gap = 4;
    const maxHeight = Math.min(iconCategoryMenuMaxHeight, viewportHeight - margin * 2);
    const below = viewportHeight - rect.bottom - margin;
    const above = rect.top - margin;
    iconCategoryMenuPlacement = {
      left: clamp(
        rect.right - iconCategoryMenuWidth,
        margin,
        Math.max(margin, viewportWidth - iconCategoryMenuWidth - margin),
      ),
      top: below >= Math.min(maxHeight, 180) || below >= above
        ? rect.bottom + gap
        : Math.max(margin, rect.top - maxHeight - gap),
      maxHeight,
    };
  }

  function toggleIconCategoryMenu(): void {
    iconCategoryMenuOpen = !iconCategoryMenuOpen;
    skinTonePanelOpen = false;
    iconColorPanelOpen = false;
    iconColorChoice = null;
    if (iconCategoryMenuOpen) {
      void tick().then(placeIconCategoryMenu);
    }
  }

  function selectIconCategory(category: ProjectLucideCategory | "all"): void {
    iconCategory = category;
    iconCategoryMenuOpen = false;
    resetGridScroll();
  }

  function placeIconColorChoice(anchor: HTMLElement): IconColorChoicePlacement {
    const rect = anchor.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const margin = 8;
    const gap = 4;
    const below = viewportHeight - rect.bottom - margin;
    const above = rect.top - margin;
    return {
      left: clamp(
        rect.left + rect.width / 2 - iconColorChoicePanelWidth / 2,
        margin,
        Math.max(margin, viewportWidth - iconColorChoicePanelWidth - margin),
      ),
      top: below >= iconColorChoicePanelHeight || below >= above
        ? Math.min(rect.bottom + gap, viewportHeight - iconColorChoicePanelHeight - margin)
        : Math.max(margin, rect.top - iconColorChoicePanelHeight - gap),
    };
  }

  function updateIconColorChoicePlacement(): void {
    const choice = iconColorChoice;
    if (!choice) return;
    if (!document.contains(choice.anchor)) {
      iconColorChoice = null;
      return;
    }
    iconColorChoice = {
      ...choice,
      placement: placeIconColorChoice(choice.anchor),
    };
  }

  async function loadLucideCatalog(): Promise<void> {
    if (lucideLoaded || lucideLoading) return;
    lucideLoading = true;
    try {
      const catalog = await import("$lib/projects/project-lucide-catalog.generated");
      lucideCategories = catalog.PROJECT_LUCIDE_CATEGORIES;
      lucideIcons = catalog.PROJECT_LUCIDE_ICONS;
      lucideLoaded = true;
    } finally {
      lucideLoading = false;
    }
  }

  async function openPicker(): Promise<void> {
    placePanel();
    open = true;
    if (parsedValue.kind === "lucide") {
      iconColor = projectIconColorToEventColor(parsedValue.color) ?? FALLBACK_COLOR_INDEX;
    }
    if (parsedValue.kind === "emoji") emojiSkinTone = projectEmojiSkinToneFromEmoji(parsedValue.emoji);
    await loadRecentValues();
    if (activeTab === "icons") await loadLucideCatalog();
    await tick();
    placePanel();
    updateGridMetrics();
  }

  function closePicker(): void {
    open = false;
    customEmojiPanelOpen = false;
    skinTonePanelOpen = false;
    iconColorPanelOpen = false;
    iconCategoryMenuOpen = false;
    iconColorChoice = null;
    uploadError = null;
    customEmojiError = null;
  }

  async function setTab(tab: ProjectIconPickerTab): Promise<void> {
    activeTab = tab;
    skinTonePanelOpen = false;
    iconColorPanelOpen = false;
    iconCategoryMenuOpen = false;
    customEmojiPanelOpen = false;
    iconColorChoice = null;
    resetGridScroll();
    placePanel();
    if (tab === "icons") await loadLucideCatalog();
    await tick();
    placePanel();
    updateGridMetrics();
  }

  function chooseIcon(icon: ProjectIconValue, closeAfterSelect = true): void {
    onChange(serializeProjectIcon(icon));
    const nextRecent = prependProjectIconRecentValue(recentValues, icon, customEmojiIds);
    saveRecentValues(nextRecent);
    if (closeAfterSelect) closePicker();
  }

  function openIconColorChoice(
    slug: string,
    label: string,
    iconNode: readonly ProjectLucideIconNode[] | null,
    target: EventTarget | null,
  ): void {
    if (!allowIconColors) {
      chooseIcon({ kind: "lucide", slug, color: "default" });
      return;
    }
    const anchor = target instanceof HTMLElement ? target : null;
    if (!anchor) {
      chooseIcon({ kind: "lucide", slug, color: iconColor });
      return;
    }
    skinTonePanelOpen = false;
    iconColorPanelOpen = false;
    customEmojiPanelOpen = false;
    iconColorChoice = {
      slug,
      label,
      iconNode,
      anchor,
      placement: placeIconColorChoice(anchor),
    };
  }

  function chooseLucideIcon(
    slug: string,
    label: string,
    iconNode: readonly ProjectLucideIconNode[] | null,
    target: EventTarget | null,
  ): void {
    if (!allowIconColors) {
      chooseIcon({ kind: "lucide", slug, color: "default" });
      return;
    }
    if (askIconColorEveryTime) {
      openIconColorChoice(slug, label, iconNode, target);
      return;
    }
    chooseIcon({ kind: "lucide", slug, color: iconColor });
  }

  function chooseRecent(rawValue: string, target: EventTarget | null = null): void {
    const icon = parseProjectIcon(rawValue);
    if (icon.kind === "lucide") {
      const entry = lucideIcons.find((candidate) => candidate.slug === icon.slug);
      chooseLucideIcon(icon.slug, entry?.label ?? projectIconDisplayLabel(icon), entry?.iconNode ?? null, target);
      return;
    }
    chooseIcon(icon);
  }

  function chooseRandomEmoji(): void {
    if (filteredEmojiEntries.length === 0) return;
    const entry = filteredEmojiEntries[Math.floor(Math.random() * filteredEmojiEntries.length)];
    chooseIcon({ kind: "emoji", emoji: displayEmoji(entry.emoji) });
  }

  function chooseRandomIcon(): void {
    if (filteredLucideEntries.length === 0) return;
    const entry = filteredLucideEntries[Math.floor(Math.random() * filteredLucideEntries.length)];
    chooseIcon({ kind: "lucide", slug: entry.slug, color: allowIconColors ? iconColor : "default" });
  }

  function handleGridScroll(): void {
    gridScrollTop = gridScrollElement?.scrollTop ?? 0;
    updateGridMetrics();
    updateIconColorChoicePlacement();
    refreshGridScrollState();
  }

  function fileToDataUrl(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onerror = () => reject(new Error(t("projects.iconPicker.uploadFailed")));
      reader.onload = () => {
        if (typeof reader.result === "string") {
          resolve(reader.result);
        } else {
          reject(new Error(t("projects.iconPicker.uploadFailed")));
        }
      };
      reader.readAsDataURL(file);
    });
  }

  async function savePastedFile(file: File): Promise<ProjectIconAsset> {
    const dataUrl = await fileToDataUrl(file);
    return saveProjectIconImageDataUrl(dataUrl);
  }

  async function chooseUploadFile(): Promise<void> {
    uploading = true;
    uploadError = null;
    try {
      uploadDraft = await pickProjectIconImageFile();
    } catch (error) {
      uploadError = error instanceof Error ? error.message : String(error);
    } finally {
      uploading = false;
      await refreshPanelPlacement();
    }
  }

  async function downloadUploadUrl(): Promise<void> {
    const url = uploadUrl.trim();
    if (!url) return;
    uploading = true;
    uploadError = null;
    try {
      uploadDraft = await downloadProjectIconImageUrl(url);
    } catch (error) {
      uploadError = error instanceof Error ? error.message : String(error);
    } finally {
      uploading = false;
      await refreshPanelPlacement();
    }
  }

  async function handleUploadPaste(event: ClipboardEvent): Promise<void> {
    const file = event.clipboardData?.files[0];
    if (!file) return;
    event.preventDefault();
    uploading = true;
    uploadError = null;
    try {
      uploadDraft = await savePastedFile(file);
    } catch (error) {
      uploadError = error instanceof Error ? error.message : String(error);
    } finally {
      uploading = false;
      await refreshPanelPlacement();
    }
  }

  async function selectUploadDraft(): Promise<void> {
    if (!uploadDraft) return;
    chooseIcon({ kind: "asset", relativePath: uploadDraft.relativePath });
    uploadDraft = null;
    uploadUrl = "";
  }

  async function chooseCustomEmojiFile(): Promise<void> {
    customEmojiError = null;
    try {
      customEmojiDraft = await pickProjectIconImageFile();
    } catch (error) {
      customEmojiError = error instanceof Error ? error.message : String(error);
    }
  }

  async function handleCustomEmojiPaste(event: ClipboardEvent): Promise<void> {
    const file = event.clipboardData?.files[0];
    if (!file) return;
    event.preventDefault();
    customEmojiError = null;
    try {
      customEmojiDraft = await savePastedFile(file);
    } catch (error) {
      customEmojiError = error instanceof Error ? error.message : String(error);
    }
  }

  async function saveCustomEmoji(): Promise<void> {
    if (!customEmojiDraft) return;
    const name = customEmojiName.trim();
    if (!name) {
      customEmojiError = t("projects.iconPicker.nameRequired");
      return;
    }
    customEmojiSaving = true;
    customEmojiError = null;
    try {
      const customEmoji = await projects.addCustomEmoji(name, customEmojiDraft.relativePath);
      if (customEmoji) chooseIcon({ kind: "custom-emoji", id: customEmoji.id });
      customEmojiName = "";
      customEmojiDraft = null;
      customEmojiPanelOpen = false;
    } catch (error) {
      customEmojiError = error instanceof Error ? error.message : String(error);
    } finally {
      customEmojiSaving = false;
    }
  }

  function removeIcon(): void {
    onChange("none");
    closePicker();
  }

  function selectIconColor(color: EventColor): void {
    if (!allowIconColors) return;
    iconColor = color;
    if (parsedValue.kind === "lucide") {
      onChange(serializeProjectIcon({ ...parsedValue, color }));
    }
  }

  function selectColorChoice(color: EventColor): void {
    const choice = iconColorChoice;
    if (!choice) return;
    chooseIcon({ kind: "lucide", slug: choice.slug, color });
  }

  async function discardUploadDraft(): Promise<void> {
    const draft = uploadDraft;
    uploadDraft = null;
    await tick();
    placePanel();
    if (draft) await deleteProjectIconAssetsIfUnreferenced([draft.relativePath]);
  }

  $effect(() => {
    if (!open) return;
    const resize = () => {
      placePanel();
      placeCustomPanel();
      placeIconCategoryMenu();
      updateIconColorChoicePlacement();
      updateGridMetrics();
    };
    const outside = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (triggerElement?.contains(target)) return;
      if (panelElement?.contains(target)) return;
      if (customPanelElement?.contains(target)) return;
      if (iconColorChoicePanelElement?.contains(target)) return;
      if (iconCategoryMenuElement?.contains(target)) return;
      closePicker();
    };
    const keydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") closePicker();
    };
    window.addEventListener("resize", resize);
    window.addEventListener("scroll", resize, true);
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", keydown);
    return () => {
      window.removeEventListener("resize", resize);
      window.removeEventListener("scroll", resize, true);
      document.removeEventListener("pointerdown", outside, true);
      document.removeEventListener("keydown", keydown);
      if (gridScrollStateFrame !== null) {
        cancelAnimationFrame(gridScrollStateFrame);
        gridScrollStateFrame = null;
      }
    };
  });

  $effect(() => {
    if (!open) return;
    const element = gridScrollElement;
    if (!element) return;
    const observer = new ResizeObserver(updateGridMetrics);
    observer.observe(element);
    updateGridMetrics();
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!open) return;
    activeTab;
    emojiQuery;
    iconQuery;
    emojiCategory;
    iconCategory;
    emojiRecentValues.length;
    visibleCustomEmojis.length;
    emojiGroups.length;
    filteredLucideEntries.length;
    lucideGroups.length;
    lucideRecentHeight;
    lucideGroupVirtual.beforeHeight;
    lucideGroupVirtual.afterHeight;
    gridColumnCount;
    void tick().then(() => {
      updateGridMetrics();
      requestGridScrollStateRefresh();
    });
  });

  $effect(() => {
    if (!customEmojiPanelOpen) return;
    void tick().then(placeCustomPanel);
  });
</script>

<button
  bind:this={triggerElement}
  type="button"
  class={cn(
    "flex h-8 min-w-0 items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground hover:bg-accent/60",
    className,
  )}
  aria-label={ariaLabel}
  onclick={() => {
    if (open) closePicker();
    else void openPicker();
  }}
>
  <span class="flex min-w-0 items-center gap-2">
    <ProjectIcon name={value} size={15} strokeWidth={1.8} ignoreColor={!allowIconColors} class="shrink-0" />
    <span class="truncate">{pickerLabel}</span>
  </span>
  <ChevronDown
    size={13}
    strokeWidth={2}
    class={cn("shrink-0 text-muted-foreground transition-transform", open && "rotate-180")}
  />
</button>

{#if open}
  <div
    bind:this={panelElement}
    use:portal
    class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-xl"
    style={panelStyle}
    role="dialog"
    data-app-floating-surface
    aria-label={ariaLabel}
    tabindex="-1"
    onpaste={activeTab === "upload" ? handleUploadPaste : undefined}
    onpointerdown={handlePanelPointerDown}
  >
    <div class="flex h-12 shrink-0 items-center justify-between border-b border-border/70 px-3">
      <div class="flex min-w-0 items-center gap-3">
        {#each (["icons", "emoji", "upload"] as const) as tab}
          <button
            type="button"
            class={cn(
              "h-10 border-b-2 px-0.5 text-[0.866667rem] transition-colors",
              activeTab === tab
                ? "border-foreground text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground",
            )}
            onclick={() => void setTab(tab)}
          >
            {tabLabel(tab)}
          </button>
        {/each}
      </div>
      <button
        type="button"
        class="h-9 px-1 text-[0.866667rem] text-muted-foreground hover:text-foreground"
        onclick={removeIcon}
      >
        {t("projects.iconPicker.remove")}
      </button>
    </div>

    {#if activeTab === "emoji"}
      <div class="flex shrink-0 items-center gap-2 px-3 pt-3">
        <div class="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-background px-2">
          <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
          <input
            bind:value={emojiQuery}
            class="h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] outline-none placeholder:text-muted-foreground"
            placeholder={t("projects.iconPicker.filter")}
          />
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.iconPicker.random")}
          onclick={chooseRandomEmoji}
        >
          <Shuffle size={14} strokeWidth={1.75} />
        </button>
        <div class="relative shrink-0" data-icon-picker-inline-panel>
          <button
            type="button"
            class={cn(
              "flex h-8 w-8 items-center justify-center rounded-md border border-border text-[1rem] text-muted-foreground hover:bg-accent hover:text-foreground",
              skinTonePanelOpen && "bg-accent text-foreground",
            )}
            aria-label={t("projects.iconPicker.skinTone")}
            title={skinToneLabel(emojiSkinTone)}
            onclick={(event) => {
              event.stopPropagation();
              skinTonePanelOpen = !skinTonePanelOpen;
              iconColorPanelOpen = false;
            }}
          >
            {skinTonePreview(emojiSkinTone)}
          </button>
          {#if skinTonePanelOpen}
            <div
              class="absolute right-0 top-9 z-10 grid gap-1 rounded-lg border border-border bg-popover p-2 shadow-lg"
              style="grid-template-columns: repeat(3, 2rem); width: 7rem;"
            >
              {#each skinToneOptions as tone}
                <button
                  type="button"
                  class={cn(
                    "flex h-8 w-8 items-center justify-center rounded-md text-[1rem]",
                    emojiSkinTone === tone.value ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  aria-label={skinToneLabel(tone.value)}
                  title={skinToneLabel(tone.value)}
                  onclick={(event) => {
                    event.stopPropagation();
                    emojiSkinTone = tone.value;
                    skinTonePanelOpen = false;
                  }}
                >
                  {skinTonePreview(tone.value)}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <div
        bind:this={gridScrollElement}
        class={cn(
          "project-icon-picker-scroll-area min-h-0 flex-1 overflow-y-auto px-3 py-3",
          gridScrollable
            && gridCanScrollUp
            && gridCanScrollDown
            && "project-icon-picker-scroll-both",
          gridScrollable
            && gridCanScrollUp
            && !gridCanScrollDown
            && "project-icon-picker-scroll-top",
          gridScrollable
            && !gridCanScrollUp
            && gridCanScrollDown
            && "project-icon-picker-scroll-bottom",
        )}
        onscroll={handleGridScroll}
      >
        {#if emojiRecentValues.length > 0}
          <section class="mb-3">
            <div class="mb-1 flex items-center gap-2 text-[0.733333rem] text-muted-foreground">
              <span>{t("projects.iconPicker.recent")}</span>
              <span class="h-px flex-1 bg-border/70"></span>
            </div>
            <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
              {#each emojiRecentValues as recentValue}
                <button
                  type="button"
                  class="flex h-9 items-center justify-center rounded-md hover:bg-accent"
                  aria-label={t("projects.iconPicker.selectRecent")}
                  onclick={() => chooseRecent(recentValue)}
                >
                  <ProjectIcon name={recentValue} size={18} />
                </button>
              {/each}
            </div>
          </section>
        {/if}

        {#if visibleCustomEmojis.length > 0}
          <section class="mb-3">
            <div class="mb-1 flex items-center gap-2 text-[0.733333rem] text-muted-foreground">
              <span>{t("projects.iconPicker.custom")}</span>
              <span class="h-px flex-1 bg-border/70"></span>
            </div>
            <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
              {#each visibleCustomEmojis as emoji}
                <button
                  type="button"
                  class="flex h-9 items-center justify-center rounded-md hover:bg-accent"
                  title={emoji.name}
                  onclick={() => chooseIcon({ kind: "custom-emoji", id: emoji.id })}
                >
                  <ProjectIcon name={`custom-emoji:${emoji.id}`} size={20} />
                </button>
              {/each}
            </div>
          </section>
        {/if}

        {#each emojiGroups as group (group.category.id)}
          <section class="mb-3">
            <div class="mb-1 flex items-center gap-2 text-[0.733333rem] text-muted-foreground">
              <span>{group.category.label}</span>
              <span class="h-px flex-1 bg-border/70"></span>
            </div>
            <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
              {#each group.entries as entry (entry.emoji)}
                <button
                  type="button"
                  class="flex h-9 items-center justify-center rounded-md text-[1.2rem] hover:bg-accent"
                  title={entry.name}
                  onclick={() => chooseIcon({ kind: "emoji", emoji: displayEmoji(entry.emoji) })}
                >
                  {displayEmoji(entry.emoji)}
                </button>
              {/each}
            </div>
          </section>
        {/each}
      </div>

      <div class="flex shrink-0 items-center gap-1 overflow-x-auto border-t border-border/70 px-3 py-2">
        <button
          type="button"
          class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", emojiCategory === "all" ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
          aria-label={t("projects.iconPicker.all")}
          title={t("projects.iconPicker.all")}
          onclick={() => {
            emojiCategory = "all";
            resetGridScroll();
          }}
        >
          <LayoutGrid size={16} strokeWidth={1.75} />
        </button>
        {#each visibleEmojiCategories as category}
          <button
            type="button"
            class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", emojiCategory === category.id ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
            aria-label={category.label}
            title={category.label}
            onclick={() => {
              emojiCategory = category.id;
              resetGridScroll();
            }}
          >
            {#if category.id === "smileys"}
              <Smile size={16} strokeWidth={1.75} />
            {:else if category.id === "people"}
              <UserRound size={16} strokeWidth={1.75} />
            {:else if category.id === "nature"}
              <Leaf size={16} strokeWidth={1.75} />
            {:else if category.id === "food"}
              <Utensils size={16} strokeWidth={1.75} />
            {:else if category.id === "activity"}
              <Trophy size={16} strokeWidth={1.75} />
            {:else if category.id === "travel"}
              <Plane size={16} strokeWidth={1.75} />
            {:else if category.id === "objects"}
              <Package size={16} strokeWidth={1.75} />
            {:else if category.id === "symbols"}
              <Shapes size={16} strokeWidth={1.75} />
            {/if}
          </button>
        {/each}
        <button
          bind:this={customEmojiTriggerElement}
          type="button"
          class={cn(
            "ml-auto flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground",
            customEmojiPanelOpen && "bg-accent text-foreground",
          )}
          aria-label={t("projects.iconPicker.addCustomEmoji")}
          title={t("projects.iconPicker.addCustomEmoji")}
          onclick={() => {
            customEmojiPanelOpen = !customEmojiPanelOpen;
            skinTonePanelOpen = false;
            iconColorPanelOpen = false;
          }}
        >
          <Plus size={16} strokeWidth={1.75} />
        </button>
      </div>
    {:else if activeTab === "icons"}
      <div class="flex shrink-0 items-center gap-2 px-3 pt-3">
        <div class="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-border bg-background px-2">
          <Search size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
          <input
            bind:value={iconQuery}
            class="h-8 min-w-0 flex-1 bg-transparent text-[0.866667rem] outline-none placeholder:text-muted-foreground"
            placeholder={t("projects.iconPicker.filter")}
          />
        </div>
        <button
          type="button"
          class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-border text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t("projects.iconPicker.random")}
          onclick={chooseRandomIcon}
        >
          <Shuffle size={14} strokeWidth={1.75} />
        </button>
        {#if allowIconColors}
          <div class="relative shrink-0" data-icon-picker-inline-panel>
            <button
              type="button"
              class={cn(
                "flex h-8 w-8 items-center justify-center rounded-md border border-border hover:bg-accent",
                iconColorPanelOpen && "bg-accent text-foreground",
              )}
              aria-label={t("projects.iconPicker.iconColor")}
              title={iconColorLabel(iconColor)}
              onclick={(event) => {
                event.stopPropagation();
                iconColorPanelOpen = !iconColorPanelOpen;
                skinTonePanelOpen = false;
              }}
            >
              <span
                class="h-4 w-4 rounded-full border border-border"
                style={`background: ${iconColorSwatch(iconColor)};`}
              ></span>
            </button>
            {#if iconColorPanelOpen}
              <div
                class="absolute right-0 top-9 z-10 w-40 rounded-lg border border-border bg-popover px-2.5 py-2 shadow-lg"
                style={`--project-icon-color-selection-border: ${colorSelectionBorder};`}
              >
                <div class="grid justify-center gap-2" style="grid-template-columns: repeat(4, 1.375rem);">
                  {#each EVENT_COLOR_OPTIONS as color}
                    <button
                      type="button"
                      class={cn(
                        "project-icon-color-swatch relative size-5.5 rounded-full",
                        iconColor === color && "swatch-selected",
                      )}
                      style={`background-color: ${iconColorSwatch(color)};`}
                      aria-label={iconColorLabel(color)}
                      title={iconColorLabel(color)}
                      onclick={(event) => {
                        event.stopPropagation();
                        selectIconColor(color);
                        iconColorPanelOpen = false;
                      }}
                    ></button>
                  {/each}
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={askIconColorEveryTime}
                  class="mt-1 flex h-8 w-full items-center justify-between rounded-md px-1.5 text-left text-[0.8rem] text-foreground hover:bg-accent"
                  onclick={(event) => {
                    event.stopPropagation();
                    askIconColorEveryTime = !askIconColorEveryTime;
                    if (!askIconColorEveryTime) iconColorChoice = null;
                  }}
                >
                  <span>{t("projects.iconPicker.askEveryTime")}</span>
                  <span
                    class={cn(
                      "flex h-4 w-7 shrink-0 items-center rounded-full p-0.5",
                      askIconColorEveryTime ? "justify-end bg-primary" : "justify-start bg-muted-foreground/30",
                    )}
                  >
                    <span class="h-3 w-3 rounded-full bg-background shadow-sm"></span>
                  </span>
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <div
        bind:this={gridScrollElement}
        class={cn(
          "project-icon-picker-scroll-area min-h-0 flex-1 overflow-y-auto px-3 py-3",
          gridScrollable
            && gridCanScrollUp
            && gridCanScrollDown
            && "project-icon-picker-scroll-both",
          gridScrollable
            && gridCanScrollUp
            && !gridCanScrollDown
            && "project-icon-picker-scroll-top",
          gridScrollable
            && !gridCanScrollUp
            && gridCanScrollDown
            && "project-icon-picker-scroll-bottom",
        )}
        onscroll={handleGridScroll}
      >
        {#if lucideRecentValues.length > 0}
          <section class="mb-3">
            <div class="mb-1 flex h-5 items-center gap-2 text-[0.733333rem] text-muted-foreground">
              <span>{t("projects.iconPicker.recent")}</span>
              <span class="h-px flex-1 bg-border/70"></span>
            </div>
            <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
              {#each lucideRecentValues as recentValue}
                <button
                  type="button"
                  class={cn(
                    "flex h-9 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
                    allowIconColors ? "text-muted-foreground" : "text-foreground",
                  )}
                  aria-label={t("projects.iconPicker.selectRecent")}
                  onclick={(event) => chooseRecent(recentValue, event.currentTarget)}
                >
                  <ProjectIcon name={lucideRecentPreviewValue(recentValue)} size={18} ignoreColor={!allowIconColors} />
                </button>
              {/each}
            </div>
          </section>
        {/if}

        {#if lucideLoading}
          <div class="py-6 text-center text-[0.8rem] text-muted-foreground">{t("common.loading")}</div>
        {:else}
          <div style={`height: ${lucideGroupVirtual.beforeHeight}px;`} aria-hidden="true"></div>
          {#each lucideGroupVirtual.groups as group (group.category)}
            <section class="mb-3">
              <div class="mb-1 flex h-5 items-center gap-2 text-[0.733333rem] text-muted-foreground">
                <span>{group.category}</span>
                <span class="h-px flex-1 bg-border/70"></span>
              </div>
              <div style={`height: ${group.beforeRowsHeight}px;`} aria-hidden="true"></div>
              <div class="grid gap-0" style={`grid-template-columns: repeat(${gridColumnCount}, minmax(0, 1fr));`}>
                {#each group.entries as entry (entry.slug)}
                  <button
                    type="button"
                    class={cn(
                      "flex h-9 items-center justify-center rounded-md hover:bg-accent hover:text-foreground",
                      allowIconColors ? "text-muted-foreground" : "text-foreground",
                    )}
                    title={entry.label}
                    onclick={(event) => chooseLucideIcon(entry.slug, entry.label, entry.iconNode, event.currentTarget)}
                  >
                    <LucideNodeIcon
                      iconNode={entry.iconNode}
                      size={18}
                      strokeWidth={1.75}
                      style={allowIconColors ? iconColorStyle(iconColor) : undefined}
                    />
                  </button>
                {/each}
              </div>
              <div style={`height: ${group.afterRowsHeight}px;`} aria-hidden="true"></div>
            </section>
            {/each}
          <div style={`height: ${lucideGroupVirtual.afterHeight}px;`} aria-hidden="true"></div>
        {/if}
      </div>

      <div class="flex shrink-0 items-center gap-1 border-t border-border/70 px-3 py-2">
        <button
          type="button"
          class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", iconCategory === "all" ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
          aria-label={t("projects.iconPicker.all")}
          title={t("projects.iconPicker.all")}
          onclick={() => selectIconCategory("all")}
        >
          <LayoutGrid size={16} strokeWidth={1.75} />
        </button>
        {#each primaryLucideCategoryOptions as option (option.category)}
          <button
            type="button"
            class={cn("flex h-8 w-8 shrink-0 items-center justify-center rounded-md", iconCategory === option.category ? "bg-accent text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground")}
            aria-label={option.category}
            title={option.category}
            onclick={() => selectIconCategory(option.category)}
          >
            {#if option.icon}
              <LucideNodeIcon iconNode={option.icon.iconNode} size={16} strokeWidth={1.75} />
            {:else}
              <Shapes size={16} strokeWidth={1.75} />
            {/if}
          </button>
        {/each}
        <div class="h-5 w-px shrink-0 bg-border/80"></div>
        <button
          bind:this={iconCategoryMenuTriggerElement}
          type="button"
          class={cn(
            "flex h-8 w-8 shrink-0 items-center justify-center rounded-md",
            iconCategoryMenuOpen || iconCategoryOverflowActive
              ? "bg-accent text-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-foreground",
          )}
          aria-label={t("projects.iconPicker.moreCategories")}
          title={t("projects.iconPicker.moreCategories")}
          onclick={toggleIconCategoryMenu}
        >
          <CircleEllipsis size={16} strokeWidth={1.75} />
        </button>
      </div>
    {:else}
      <div class="min-h-0 space-y-3 overflow-y-auto p-3" style={uploadBodyStyle}>
        {#if uploadDraft}
          <div class="grid gap-3">
            <div class="flex h-36 items-center justify-center rounded-lg bg-muted/45">
              <ProjectIcon name={`asset:${uploadDraft.relativePath}`} size={112} class="shadow-sm" />
            </div>
            <div class="flex items-center justify-between gap-2">
              <button
                type="button"
                class="h-8 rounded-md px-2 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
                onclick={() => void discardUploadDraft()}
              >
                {t("common.cancel")}
              </button>
              <button
                type="button"
                class="h-8 rounded-md bg-primary px-3 text-[0.866667rem] font-medium text-primary-foreground"
                onclick={() => void selectUploadDraft()}
              >
                {t("common.save")}
              </button>
            </div>
          </div>
        {:else}
          <button
            type="button"
            class="flex min-h-16 w-full items-center justify-center gap-2 rounded-md bg-muted/50 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
            disabled={uploading}
            onclick={chooseUploadFile}
          >
            <ImageIcon size={17} strokeWidth={1.75} />
            {t("projects.iconPicker.uploadImage")}
          </button>
          <div class="text-center text-[0.733333rem] text-muted-foreground">{t("projects.iconPicker.pasteHint")}</div>
          <div class="flex gap-2">
            <input
              bind:value={uploadUrl}
              class="h-8 min-w-0 flex-1 rounded-md border border-border bg-background px-2 text-[0.8rem] outline-none placeholder:text-muted-foreground focus:border-ring"
              placeholder={t("projects.iconPicker.imageUrl")}
            />
            <button
              type="button"
              class="h-8 rounded-md bg-primary px-2 text-[0.8rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
              disabled={uploading || !uploadUrl.trim()}
              onclick={downloadUploadUrl}
            >
              {t("projects.iconPicker.fetch")}
            </button>
          </div>
        {/if}
        {#if uploadError}
          <div class="rounded-md bg-destructive/10 px-2 py-1 text-[0.8rem] text-destructive">{uploadError}</div>
        {/if}
      </div>
    {/if}
  </div>

  {#if iconCategoryMenuOpen}
    <div
      bind:this={iconCategoryMenuElement}
      use:portal
      class="fixed z-100 min-h-0 overflow-y-auto rounded-xl border border-border bg-popover p-1.5 text-popover-foreground shadow-xl"
      style={iconCategoryMenuStyle}
      role="dialog"
      data-app-floating-surface
      aria-label={t("projects.iconPicker.moreCategories")}
    >
      {#each lucideCategoryOptions as option (option.category)}
        <button
          type="button"
          class={cn(
            "flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-[0.8rem]",
            iconCategory === option.category
              ? "bg-accent/70 text-foreground"
              : "text-foreground hover:bg-accent/40",
          )}
          aria-checked={iconCategory === option.category}
          role="menuitemradio"
          onclick={() => selectIconCategory(option.category)}
        >
          <span class="flex h-5 w-5 shrink-0 items-center justify-center">
            {#if option.icon}
              <LucideNodeIcon iconNode={option.icon.iconNode} size={15} strokeWidth={1.75} class="block" />
            {:else}
              <Shapes size={15} strokeWidth={1.75} class="block" />
            {/if}
          </span>
          <span class="min-w-0 flex-1 truncate">{option.category}</span>
          <span class="flex h-4 w-4 shrink-0 items-center justify-center">
          {#if iconCategory === option.category}
            <Check size={14} strokeWidth={1.75} />
          {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}

  {#if iconColorChoice && allowIconColors}
    <div
      bind:this={iconColorChoicePanelElement}
      use:portal
      class="fixed z-100 rounded-xl border border-border bg-popover p-2.5 text-popover-foreground shadow-xl"
      style={iconColorChoiceStyle(iconColorChoice)}
      role="dialog"
      data-app-floating-surface
      aria-label={iconColorChoice.label}
    >
      <div class="grid gap-2" style="grid-template-columns: repeat(8, 1.375rem);">
        {#each EVENT_COLOR_OPTIONS as color}
          <button
            type="button"
            class="flex size-5.5 items-center justify-center rounded-md hover:bg-accent"
            aria-label={iconColorLabel(color)}
            title={iconColorLabel(color)}
            onclick={(event) => {
              event.stopPropagation();
              selectColorChoice(color);
            }}
          >
            {#if iconColorChoice.iconNode}
              <LucideNodeIcon
                iconNode={iconColorChoice.iconNode}
                size={16}
                strokeWidth={1.75}
                style={iconColorStyle(color)}
              />
            {:else}
              <ProjectIcon
                name={serializeProjectIcon({ kind: "lucide", slug: iconColorChoice.slug, color })}
                size={16}
                strokeWidth={1.75}
              />
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  {#if customEmojiPanelOpen}
    <section
      bind:this={customPanelElement}
      use:portal
      class="fixed z-90 flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-popover p-4 text-popover-foreground shadow-xl"
      style={customPanelStyle}
      data-app-floating-surface
      onpaste={handleCustomEmojiPaste}
    >
      <div class="mb-1 text-[0.933333rem] font-semibold text-foreground">{t("projects.iconPicker.addCustomEmoji")}</div>
      <div class="mb-5 text-[0.8rem] text-muted-foreground">{t("projects.iconPicker.customEmojiDescription")}</div>
      <button
        type="button"
        class="mb-4 flex min-h-16 w-full items-center justify-center gap-2 rounded-md bg-muted/50 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
        onclick={chooseCustomEmojiFile}
      >
        {#if customEmojiDraft}
          <ProjectIcon name={`asset:${customEmojiDraft.relativePath}`} size={28} />
        {:else}
          <Upload size={17} strokeWidth={1.75} />
        {/if}
        {t("projects.iconPicker.uploadImage")}
      </button>
      <label class="grid gap-1 text-[0.733333rem] text-muted-foreground">
        {t("projects.iconPicker.emojiName")}
        <input
          bind:value={customEmojiName}
          class="h-9 rounded-md border border-border bg-background px-2 text-left text-[0.866667rem] text-foreground outline-none placeholder:text-muted-foreground focus:border-ring"
          placeholder={t("projects.iconPicker.emojiNamePlaceholder")}
        />
      </label>
      {#if customEmojiError}
        <div class="mt-2 rounded-md bg-destructive/10 px-2 py-1 text-[0.8rem] text-destructive">{customEmojiError}</div>
      {/if}
      <div class="mt-5 flex justify-between gap-2">
        <button
          type="button"
          class="h-8 rounded-md px-2 text-[0.866667rem] text-muted-foreground hover:bg-accent hover:text-foreground"
          onclick={() => {
            customEmojiPanelOpen = false;
          }}
        >
          {t("common.cancel")}
        </button>
        <button
          type="button"
          class="h-8 rounded-md bg-primary px-3 text-[0.866667rem] font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
          disabled={!customEmojiDraft || !customEmojiName.trim() || customEmojiSaving}
          onclick={() => void saveCustomEmoji()}
        >
          {t("common.save")}
        </button>
      </div>
      <button
        type="button"
        class="absolute right-2 top-2 flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
        aria-label={t("common.close")}
        onclick={() => {
          customEmojiPanelOpen = false;
        }}
      >
        <X size={14} strokeWidth={1.75} />
      </button>
    </section>
  {/if}
{/if}

<style>
  .project-icon-picker-scroll-area {
    transition: -webkit-mask-image 120ms ease, mask-image 120ms ease;
  }

  .project-icon-picker-scroll-top {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black 20px, black);
    mask-image: linear-gradient(to bottom, transparent, black 20px, black);
  }

  .project-icon-picker-scroll-bottom {
    -webkit-mask-image: linear-gradient(to bottom, black, black calc(100% - 20px), transparent);
    mask-image: linear-gradient(to bottom, black, black calc(100% - 20px), transparent);
  }

  .project-icon-picker-scroll-both {
    -webkit-mask-image: linear-gradient(to bottom, transparent, black 20px, black calc(100% - 20px), transparent);
    mask-image: linear-gradient(to bottom, transparent, black 20px, black calc(100% - 20px), transparent);
  }

  .project-icon-color-swatch {
    overflow: hidden;
  }

  .project-icon-color-swatch.swatch-selected::after {
    content: "";
    position: absolute;
    inset: 0;
    border: 2px solid var(--project-icon-color-selection-border);
    border-radius: inherit;
    pointer-events: none;
  }
</style>
