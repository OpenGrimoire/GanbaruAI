<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import {
    deleteProjectIconAssetsIfUnreferenced,
    downloadProjectIconImageUrl,
    pickProjectIconImageFile,
    projectIconAssetUrl,
    saveProjectIconImageDataUrl,
    type ProjectIconAsset,
  } from "$lib/api/project-icons";
  import type {
    IconPickerAsset,
    IconPickerTriggerContext,
    IconPickerUploadAdapter,
  } from "$lib/components/icon-picker/types";
  import {
    FALLBACK_COLOR_INDEX,
    type EventColor,
  } from "$lib/components/calendar/types";
  import { getEventColor } from "$lib/components/calendar/utils";
  import { contrastRatio } from "$lib/components/ui/colorMath";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    PROJECT_EMOJI_ENTRIES,
    type ProjectEmojiCategoryId,
  } from "$lib/projects/project-emoji-catalog";
  import {
    cleanupProjectIconRecentValues,
    filterProjectEmojiEntries,
    filterProjectLucideIcons,
    projectIconEmojiRecentValues,
    projectIconLucideRecentValues,
    prependProjectIconRecentValue,
    projectEmojiSkinToneFromEmoji,
    projectIconPickerAnchoredPanelPlacement,
    projectIconPickerGroupVirtualWindow,
    projectIconPickerIsPrimaryLucideCategory,
    projectIconPickerLucideCategoryOptions,
    projectIconPickerLucideGroups,
    projectIconPickerLucideRecentPreviewValue,
    projectIconPickerMenuPlacement,
    projectIconPickerPanelPlacement,
    projectIconPickerPointPlacement,
    projectIconPickerPrimaryLucideCategoryOptions,
    projectIconPickerRandomEmojiIcon,
    projectIconPickerRandomLucideIcon,
    projectIconPickerVisibleEmojiCategories,
    projectIconPickerVisibleCustomEmojis,
    readProjectIconRecentValues,
    stripProjectEmojiSkinTone,
    type ProjectEmojiSkinTone,
    type ProjectIconPickerAnchoredPanelPlacement,
    type ProjectIconPickerPointPlacement,
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
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import IconPickerCategoryMenu from "./IconPickerCategoryMenu.svelte";
  import IconPickerColorChoicePanel from "./IconPickerColorChoicePanel.svelte";
  import IconPickerCustomEmojiPanel from "./IconPickerCustomEmojiPanel.svelte";
  import IconPickerEmojiTab from "./IconPickerEmojiTab.svelte";
  import IconPickerIconsTab from "./IconPickerIconsTab.svelte";
  import IconPickerUploadPanel from "./IconPickerUploadPanel.svelte";

  type ProjectIconPickerTab = "emoji" | "icons" | "upload";
  type IconColorChoice = {
    slug: string;
    label: string;
    iconNode: readonly ProjectLucideIconNode[] | null;
    anchor: HTMLElement;
    placement: ProjectIconPickerPointPlacement;
  };

  let {
    value,
    onChange,
    ariaLabel,
    allowIconColors = true,
    trigger,
    uploadAdapter,
    class: className = "",
  }: {
    value: string;
    onChange: (value: string) => void;
    ariaLabel: string;
    allowIconColors?: boolean;
    trigger?: Snippet<[IconPickerTriggerContext]>;
    uploadAdapter?: IconPickerUploadAdapter;
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

  let open = $state(false);
  let activeTab = $state<ProjectIconPickerTab>("icons");
  let triggerElement = $state<HTMLElement | undefined>();
  let panelElement = $state<HTMLElement | undefined>();
  let customPanelElement = $state<HTMLElement | undefined>();
  let customEmojiTriggerElement = $state<HTMLButtonElement | undefined>();
  let iconColorChoicePanelElement = $state<HTMLElement | undefined>();
  let iconCategoryMenuElement = $state<HTMLElement | undefined>();
  let iconCategoryMenuTriggerElement = $state<HTMLButtonElement | undefined>();
  let gridScrollElement = $state<HTMLElement | undefined>();
  let panelPlacement = $state({ left: 0, top: 0, width: panelWidth, height: panelPreferredHeight });
  let customPanelPlacement = $state<ProjectIconPickerAnchoredPanelPlacement>({ left: 0, top: 0, maxHeight: 360 });
  let iconCategoryMenuPlacement = $state<ProjectIconPickerAnchoredPanelPlacement>({
    left: 0,
    top: 0,
    maxHeight: iconCategoryMenuMaxHeight,
  });
  let emojiQuery = $state("");
  let iconQuery = $state("");
  let uploadUrl = $state("");
  let uploadDraft = $state<IconPickerAsset | null>(null);
  let uploadPreviewUrl = $state<string | null>(null);
  let uploadPreviewRequestId = 0;
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
    projectIconEmojiRecentValues(recentValues),
  );
  const lucideRecentValues = $derived(projectIconLucideRecentValues(recentValues));
  const visibleCustomEmojis = $derived(projectIconPickerVisibleCustomEmojis(projects.customEmojis, emojiQuery));
  const visibleEmojiCategories = $derived(
    projectIconPickerVisibleEmojiCategories(t("projects.iconPicker.symbolsAndFlags")),
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
  const lucideGroups = $derived(projectIconPickerLucideGroups(filteredLucideEntries, lucideCategories));
  const lucideCategoryOptions = $derived(
    projectIconPickerLucideCategoryOptions(lucideCategories, lucideIcons),
  );
  const primaryLucideCategoryOptions = $derived(
    projectIconPickerPrimaryLucideCategoryOptions(lucideCategoryOptions),
  );
  const iconCategoryOverflowActive = $derived(
    iconCategory !== "all" && !projectIconPickerIsPrimaryLucideCategory(iconCategory),
  );
  const lucideRecentHeight = $derived.by(() => {
    if (lucideRecentValues.length === 0) return 0;
    return gridGroupHeaderHeight
      + (Math.ceil(lucideRecentValues.length / Math.max(1, gridColumnCount)) * gridRowHeight)
      + gridGroupGapHeight;
  });
  const lucideGroupVirtual = $derived(projectIconPickerGroupVirtualWindow({
    groups: lucideGroups,
    columnCount: gridColumnCount,
    viewportHeight: gridViewportHeight,
    scrollTop: Math.max(0, gridScrollTop - lucideRecentHeight),
    rowHeight: gridRowHeight,
    groupHeaderHeight: gridGroupHeaderHeight,
    groupGapHeight: gridGroupGapHeight,
  }));
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
    return projectIconPickerLucideRecentPreviewValue({ rawValue, allowIconColors, iconColor });
  }

  function iconColorChoiceStyle(choice: IconColorChoice): string {
    return `left: ${choice.placement.left}px; top: ${choice.placement.top}px; width: ${iconColorChoicePanelWidth}px;`;
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

  async function loadRecentValues(): Promise<void> {
    await ensureConfigLoaded();
    recentValues = cleanupProjectIconRecentValues(
      readProjectIconRecentValues(getConfigKey<unknown>(recentConfigKey, [])),
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
    customPanelPlacement = projectIconPickerAnchoredPanelPlacement({
      anchorRect: toPickerRect(trigger.getBoundingClientRect()),
      viewportRect: viewportBoundaryRect(),
      panelWidth: customPanelWidth,
      preferredHeight: 360,
    });
  }

  function placeIconCategoryMenu(): void {
    const trigger = iconCategoryMenuTriggerElement;
    if (!trigger) return;
    iconCategoryMenuPlacement = projectIconPickerMenuPlacement({
      anchorRect: toPickerRect(trigger.getBoundingClientRect()),
      viewportRect: viewportBoundaryRect(),
      menuWidth: iconCategoryMenuWidth,
      menuMaxHeight: iconCategoryMenuMaxHeight,
    });
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

  function placeIconColorChoice(anchor: HTMLElement): ProjectIconPickerPointPlacement {
    return projectIconPickerPointPlacement({
      anchorRect: toPickerRect(anchor.getBoundingClientRect()),
      viewportRect: viewportBoundaryRect(),
      panelWidth: iconColorChoicePanelWidth,
      panelHeight: iconColorChoicePanelHeight,
    });
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
    const icon = projectIconPickerRandomEmojiIcon(filteredEmojiEntries, emojiSkinTone);
    if (icon) chooseIcon(icon);
  }

  function chooseRandomIcon(): void {
    const icon = projectIconPickerRandomLucideIcon(filteredLucideEntries, { allowIconColors, iconColor });
    if (icon) chooseIcon(icon);
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

  async function saveProjectIconPastedFile(file: File): Promise<ProjectIconAsset> {
    const dataUrl = await fileToDataUrl(file);
    return saveProjectIconImageDataUrl(dataUrl);
  }

  async function saveUploadPastedFile(file: File): Promise<IconPickerAsset> {
    const dataUrl = await fileToDataUrl(file);
    return uploadAdapter
      ? uploadAdapter.saveImageDataUrl(dataUrl, file.name)
      : saveProjectIconImageDataUrl(dataUrl);
  }

  async function chooseUploadFile(): Promise<void> {
    uploading = true;
    uploadError = null;
    try {
      const asset = uploadAdapter
        ? await uploadAdapter.pickImageFile()
        : await pickProjectIconImageFile();
      if (asset && uploadAdapter?.selectPickedAssetImmediately) {
        await uploadAdapter.selectAsset(asset);
        closePicker();
      } else {
        uploadDraft = asset;
      }
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
      if (uploadAdapter?.selectExternalUrl) {
        await uploadAdapter.selectExternalUrl(url);
        uploadUrl = "";
        closePicker();
      } else {
        uploadDraft = uploadAdapter?.downloadImageUrl
          ? await uploadAdapter.downloadImageUrl(url)
          : await downloadProjectIconImageUrl(url);
      }
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
      const asset = await saveUploadPastedFile(file);
      if (uploadAdapter?.selectPickedAssetImmediately) {
        await uploadAdapter.selectAsset(asset);
        closePicker();
      } else {
        uploadDraft = asset;
      }
    } catch (error) {
      uploadError = error instanceof Error ? error.message : String(error);
    } finally {
      uploading = false;
      await refreshPanelPlacement();
    }
  }

  async function selectUploadDraft(): Promise<void> {
    if (!uploadDraft) return;
    if (uploadAdapter) {
      await uploadAdapter.selectAsset(uploadDraft);
      closePicker();
    } else {
      chooseIcon({ kind: "asset", relativePath: uploadDraft.relativePath });
    }
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
      customEmojiDraft = await saveProjectIconPastedFile(file);
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
    if (!draft) return;
    if (uploadAdapter?.deleteAssetsIfUnreferenced) {
      await uploadAdapter.deleteAssetsIfUnreferenced([draft.relativePath]);
    } else if (!uploadAdapter) {
      await deleteProjectIconAssetsIfUnreferenced([draft.relativePath]);
    }
  }

  function togglePicker(): void {
    if (open) closePicker();
    else void openPicker();
  }

  $effect(() => {
    const draft = uploadDraft;
    const requestId = ++uploadPreviewRequestId;
    uploadPreviewUrl = null;
    if (!draft) return;
    const request = uploadAdapter
      ? uploadAdapter.assetUrl(draft)
      : projectIconAssetUrl(draft.relativePath);
    void request
      .then((url) => {
        if (requestId === uploadPreviewRequestId) uploadPreviewUrl = url;
      })
      .catch(() => {
        if (requestId === uploadPreviewRequestId) uploadPreviewUrl = null;
      });
  });

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

{#if trigger}
  <span bind:this={triggerElement} class="inline-flex">
    {@render trigger({ open, toggle: togglePicker })}
  </span>
{:else}
  <button
    bind:this={triggerElement}
    type="button"
    class={cn(
      "flex h-8 min-w-0 items-center justify-between gap-2 rounded-md border border-border bg-card px-2.5 text-left text-[0.8rem] font-medium text-foreground hover:bg-accent/60",
      className,
    )}
    aria-label={ariaLabel}
    onclick={togglePicker}
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
{/if}

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
      <IconPickerEmojiTab
        bind:scrollElement={gridScrollElement}
        bind:customEmojiTriggerElement={customEmojiTriggerElement}
        bind:emojiQuery
        bind:emojiCategory
        bind:emojiSkinTone
        bind:skinTonePanelOpen
        bind:iconColorPanelOpen
        bind:customEmojiPanelOpen
        {gridScrollable}
        {gridCanScrollUp}
        {gridCanScrollDown}
        {gridColumnCount}
        {emojiRecentValues}
        {visibleCustomEmojis}
        {emojiGroups}
        {visibleEmojiCategories}
        onScroll={handleGridScroll}
        onChooseRandom={chooseRandomEmoji}
        onChooseRecent={chooseRecent}
        onChooseIcon={chooseIcon}
        onResetGridScroll={resetGridScroll}
      />
    {:else if activeTab === "icons"}
      <IconPickerIconsTab
        bind:scrollElement={gridScrollElement}
        bind:iconCategoryMenuTriggerElement={iconCategoryMenuTriggerElement}
        bind:iconQuery
        bind:iconColor
        bind:iconColorPanelOpen
        bind:skinTonePanelOpen
        bind:askIconColorEveryTime
        {iconCategory}
        {iconCategoryMenuOpen}
        {iconCategoryOverflowActive}
        {allowIconColors}
        {colorSelectionBorder}
        {gridScrollable}
        {gridCanScrollUp}
        {gridCanScrollDown}
        {gridColumnCount}
        {lucideRecentValues}
        {lucideGroupVirtual}
        {lucideLoading}
        {primaryLucideCategoryOptions}
        {iconColorLabel}
        {iconColorSwatch}
        {iconColorStyle}
        {lucideRecentPreviewValue}
        onScroll={handleGridScroll}
        onChooseRandom={chooseRandomIcon}
        onChooseRecent={chooseRecent}
        onChooseLucideIcon={chooseLucideIcon}
        onSelectIconColor={selectIconColor}
        onSelectIconCategory={selectIconCategory}
        onToggleIconCategoryMenu={toggleIconCategoryMenu}
        onClearIconColorChoice={() => {
          iconColorChoice = null;
        }}
      />
    {:else}
      <IconPickerUploadPanel
        {uploadDraft}
        {uploadPreviewUrl}
        {uploadError}
        {uploading}
        {uploadBodyStyle}
        bind:uploadUrl
        onChooseFile={chooseUploadFile}
        onDiscardDraft={discardUploadDraft}
        onSelectDraft={selectUploadDraft}
        onDownloadUrl={downloadUploadUrl}
      />
    {/if}
  </div>

  {#if iconCategoryMenuOpen}
    <IconPickerCategoryMenu
      bind:rootElement={iconCategoryMenuElement}
      style={iconCategoryMenuStyle}
      options={lucideCategoryOptions}
      selectedCategory={iconCategory}
      ariaLabel={t("projects.iconPicker.moreCategories")}
      onSelect={selectIconCategory}
    />
  {/if}

  {#if iconColorChoice && allowIconColors}
    <IconPickerColorChoicePanel
      bind:rootElement={iconColorChoicePanelElement}
      style={iconColorChoiceStyle(iconColorChoice)}
      label={iconColorChoice.label}
      slug={iconColorChoice.slug}
      iconNode={iconColorChoice.iconNode}
      {iconColorLabel}
      {iconColorStyle}
      onSelect={selectColorChoice}
    />
  {/if}

  {#if customEmojiPanelOpen}
    <IconPickerCustomEmojiPanel
      bind:rootElement={customPanelElement}
      style={customPanelStyle}
      {customEmojiDraft}
      bind:customEmojiName
      {customEmojiError}
      {customEmojiSaving}
      onChooseFile={chooseCustomEmojiFile}
      onPaste={handleCustomEmojiPaste}
      onSave={saveCustomEmoji}
      onClose={() => {
        customEmojiPanelOpen = false;
      }}
    />
  {/if}
{/if}
