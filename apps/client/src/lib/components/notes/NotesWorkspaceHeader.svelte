<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Plus from "@lucide/svelte/icons/plus";
  import Settings2 from "@lucide/svelte/icons/settings-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import { NOTES_PAGE_CHROME_EMOJI_SCALE } from "$lib/notes/page-icon";
  import { notesPageTitle } from "$lib/notes/page-title";
  import type { NotesPage } from "$lib/notes/types";
  import {
    projectLifecycleBadgeClass,
    projectLifecycleLabel,
  } from "$lib/projects/project-display";
  import {
    projectNavigatorPanelGeometry,
    type ProjectNavigatorPanelMode,
  } from "$lib/projects/project-toolbar";
  import type { Project, ProjectGroup } from "$lib/projects/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { cn } from "$lib/utils";
  import ProjectIcon from "$lib/components/projects/ProjectIcon.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import NotesPagePickerPanel from "./NotesPagePickerPanel.svelte";
  import NotesProjectNavigator from "./NotesProjectNavigator.svelte";

  type NotesNavigatorMode = ProjectNavigatorPanelMode | "notes";

  let {
    selectedProject,
    selectedGroup,
    selectedProjectId,
    selectedPage,
    showInactiveProjects,
    onShowInactiveProjectsChange,
    onProjectSelected,
    onShowHome,
    projectSettingsOpen,
    onToggleProjectSettings,
  }: {
    selectedProject: Project | undefined;
    selectedGroup: ProjectGroup | undefined;
    selectedProjectId: string | null;
    selectedPage: NotesPage | null;
    showInactiveProjects: boolean;
    onShowInactiveProjectsChange: (value: boolean) => void;
    onProjectSelected: () => void;
    onShowHome: () => void;
    projectSettingsOpen: boolean;
    onToggleProjectSettings: () => void;
  } = $props();

  const notes = getNotes();
  const viewport = getViewport();
  const { t } = getLocalization();
  const projectIdentityIconStrokeWidth = 1.5;
  const projectIdentityEmojiScale = 0.94;
  const newPageShortcut = $derived(formatShortcut("Mod + N"));
  const newPageTitle = $derived(`${t("notes.newPage")} (${newPageShortcut})`);

  let navigatorOpen = $state(false);
  let navigatorMode = $state<NotesNavigatorMode>("groups");
  let notesHeaderElement = $state<HTMLDivElement | null>(null);
  let notesIdentityElement = $state<HTMLDivElement | null>(null);
  let navigatorAnchorElement = $state<HTMLButtonElement | null>(null);
  let groupTriggerElement = $state<HTMLButtonElement | null>(null);
  let projectTriggerElement = $state<HTMLButtonElement | null>(null);
  let noteTriggerElement = $state<HTMLButtonElement | null>(null);
  let navigatorPanelElement = $state<HTMLDivElement | null>(null);
  let navigatorPanelStyle = $state("");
  let navigatorPanelMaxHeight = $state(0);

  interface NavigatorBounds {
    left: number;
    right: number;
    top: number;
    bottom: number;
  }

  const selectedPageTitle = $derived.by(() => {
    if (notes.viewMode === "archive") return t("notes.archive");
    if (notes.viewMode === "trash") return t("notes.trash");
    if (!selectedPage) return null;
    const draftTitle = notes.pageTitleDraftForPage(selectedPage.id);
    if (draftTitle !== null) {
      const trimmedDraftTitle = draftTitle.trim();
      return trimmedDraftTitle.length > 0 ? trimmedDraftTitle : t("notes.untitled");
    }
    return notesPageTitle(selectedPage, t("notes.untitled"));
  });
  const selectedPageId = $derived(selectedPage?.id ?? null);

  function toolbarIconButtonClass(active = false, open = false, primary = false): string {
    return cn(
      "flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-colors",
      primary
        ? "bg-primary text-primary-foreground hover:bg-primary/90"
        : "hover:bg-accent",
      !primary && (active ? "text-foreground" : "text-muted-foreground"),
      !primary && open && "bg-accent text-accent-foreground",
    );
  }

  function inlineNewPageButtonClass(): string {
    return cn(
      "flex h-7 w-5 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors",
      "hover:bg-accent hover:text-accent-foreground",
    );
  }

  function navigatorBounds(): NavigatorBounds {
    const boundsElement = notesHeaderElement?.closest(".notes-view-root");
    const rect = boundsElement?.getBoundingClientRect();
    if (rect) {
      return {
        left: rect.left,
        right: rect.right,
        top: rect.top,
        bottom: rect.bottom,
      };
    }

    return {
      left: 0,
      right: viewport.width,
      top: 0,
      bottom: viewport.height,
    };
  }

  function refreshNavigatorPanelGeometry(): void {
    if (!navigatorOpen || !navigatorAnchorElement) return;
    const rect = navigatorAnchorElement.getBoundingClientRect();
    const bounds = navigatorBounds();
    const geometry = projectNavigatorPanelGeometry({
      anchorLeft: rect.left,
      anchorBottom: rect.bottom,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height,
      boundsLeft: bounds.left,
      boundsRight: bounds.right,
      boundsTop: bounds.top,
      boundsBottom: bounds.bottom,
    });
    navigatorPanelStyle = [
      `left: ${Math.round(geometry.left)}px`,
      `top: ${Math.round(geometry.top)}px`,
      `width: ${Math.round(geometry.width)}px`,
    ].join("; ");
    navigatorPanelMaxHeight = geometry.height;
  }

  function triggerForMode(mode: NotesNavigatorMode): HTMLButtonElement | null {
    if (mode === "groups") return groupTriggerElement;
    if (mode === "projects") return projectTriggerElement;
    return noteTriggerElement ?? projectTriggerElement;
  }

  function openNavigator(mode: NotesNavigatorMode): void {
    navigatorMode = mode;
    navigatorAnchorElement = triggerForMode(mode);
    navigatorOpen = true;
    refreshNavigatorPanelGeometry();
    requestAnimationFrame(refreshNavigatorPanelGeometry);
  }

  function toggleNavigator(mode: NotesNavigatorMode): void {
    if (navigatorOpen && navigatorMode === mode) {
      navigatorOpen = false;
      return;
    }
    openNavigator(mode);
  }

  function handleProjectTriggerClick(): void {
    if (selectedPageTitle) {
      navigatorOpen = false;
      onShowHome();
      return;
    }
    toggleNavigator("projects");
  }

  function handleWindowPointerDown(event: PointerEvent): void {
    const target = event.target;
    if (!(target instanceof Node)) return;
    if (
      navigatorOpen
      && !groupTriggerElement?.contains(target)
      && !projectTriggerElement?.contains(target)
      && !noteTriggerElement?.contains(target)
      && !navigatorPanelElement?.contains(target)
    ) {
      navigatorOpen = false;
    }
  }

  function createPage(): void {
    navigatorOpen = false;
    void notes.createPage("", { projectId: selectedProjectId });
  }

  $effect(() => {
    if (!navigatorOpen) return;
    const viewportWidth = viewport.width;
    const viewportHeight = viewport.height;
    void viewportWidth;
    void viewportHeight;
    requestAnimationFrame(refreshNavigatorPanelGeometry);
  });
</script>

<svelte:window onpointerdown={handleWindowPointerDown} />

<div
  bind:this={notesHeaderElement}
  class="flex shrink-0 items-center gap-1 overflow-x-auto px-3"
  style="height: var(--cal-header-row-h); background-color: var(--cal-header-bg); border-bottom: 1px solid var(--sidebar);"
  onscroll={refreshNavigatorPanelGeometry}
>
  <div bind:this={notesIdentityElement} class="relative min-w-36 shrink-0 min-[760px]:max-w-xl">
    <div class="flex h-7 min-w-0 max-w-full items-center gap-0.5 text-sm">
      {#if selectedProject && selectedGroup}
        <button
          bind:this={groupTriggerElement}
          type="button"
          class={cn(
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md px-1.5 text-left hover:bg-accent hover:text-accent-foreground",
            navigatorOpen && navigatorMode === "groups" && "bg-accent text-accent-foreground",
          )}
          aria-label={t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "groups"}
          onpointerenter={() => openNavigator("groups")}
          onclick={() => toggleNavigator("groups")}
        >
          <ProjectIcon
            name={selectedGroup.icon}
            size={14}
            strokeWidth={projectIdentityIconStrokeWidth}
            emojiScale={projectIdentityEmojiScale}
            class="shrink-0"
          />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedGroup.name}</span>
        </button>
        <span class="shrink-0 px-0.5 font-semibold text-muted-foreground">/</span>
        <button
          bind:this={projectTriggerElement}
          type="button"
          class={cn(
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md pl-1.5 text-left hover:bg-accent hover:text-accent-foreground",
            selectedPageTitle ? "pr-1.5" : "pr-0.5",
            navigatorOpen && navigatorMode === "projects" && "bg-accent text-accent-foreground",
          )}
          aria-label={selectedPageTitle ? t("notes.showProjectHome") : t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "projects"}
          onpointerenter={() => openNavigator("projects")}
          onclick={handleProjectTriggerClick}
        >
          <ProjectIcon
            name={selectedProject.icon}
            size={14}
            strokeWidth={projectIdentityIconStrokeWidth}
            emojiScale={projectIdentityEmojiScale}
            class="shrink-0"
          />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedProject.name}</span>
          {#if !selectedPageTitle}
            <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
          {/if}
          {#if selectedProject.status !== "active"}
            <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>
              {projectLifecycleLabel(selectedProject.status, t)}
            </span>
          {/if}
        </button>
        {#if !selectedPageTitle}
          <button
            type="button"
            class={inlineNewPageButtonClass()}
            aria-label={t("notes.newPage")}
            aria-keyshortcuts="Control+N Meta+N"
            title={newPageTitle}
            onclick={createPage}
          >
            <Plus size={14} strokeWidth={1.75} />
          </button>
        {/if}
        {#if selectedPageTitle}
          <span class="shrink-0 px-0.5 font-semibold text-muted-foreground">/</span>
          <button
            bind:this={noteTriggerElement}
            type="button"
            class={cn(
              "flex h-7 min-w-0 items-center gap-1.5 rounded-md pl-1.5 pr-0.5 text-left hover:bg-accent hover:text-accent-foreground",
              navigatorOpen && navigatorMode === "notes" && "bg-accent text-accent-foreground",
            )}
            aria-label={t("notes.openNoteNavigator")}
            aria-expanded={navigatorOpen && navigatorMode === "notes"}
            onpointerenter={() => openNavigator("notes")}
            onclick={() => toggleNavigator("notes")}
          >
            <NotesPageIcon
              icon={selectedPage?.icon ?? null}
              size={14}
              strokeWidth={1.75}
              emojiScale={NOTES_PAGE_CHROME_EMOJI_SCALE}
              class="shrink-0"
            />
            <span class="min-w-0 truncate font-semibold text-foreground">{selectedPageTitle}</span>
            <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
          </button>
          <button
            type="button"
            class={inlineNewPageButtonClass()}
            aria-label={t("notes.newPage")}
            aria-keyshortcuts="Control+N Meta+N"
            title={newPageTitle}
            onclick={createPage}
          >
            <Plus size={14} strokeWidth={1.75} />
          </button>
        {/if}
      {:else}
        <button
          bind:this={noteTriggerElement}
          type="button"
          class={cn(
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md pl-1.5 pr-0.5 text-left hover:bg-accent hover:text-accent-foreground",
            navigatorOpen && navigatorMode === "notes" && "bg-accent text-accent-foreground",
          )}
          aria-label={t("notes.openNoteNavigator")}
          aria-expanded={navigatorOpen && navigatorMode === "notes"}
          onclick={() => toggleNavigator("notes")}
        >
          <NotesPageIcon
            icon={selectedPage?.icon ?? null}
            size={14}
            strokeWidth={1.75}
            emojiScale={NOTES_PAGE_CHROME_EMOJI_SCALE}
            class="shrink-0"
          />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedPageTitle ?? t("notes.title")}</span>
          <ChevronDown size={14} strokeWidth={1.75} class="shrink-0 text-muted-foreground" />
        </button>
        <button
          type="button"
          class={inlineNewPageButtonClass()}
          aria-label={t("notes.newPage")}
          aria-keyshortcuts="Control+N Meta+N"
          title={newPageTitle}
          onclick={createPage}
        >
          <Plus size={14} strokeWidth={1.75} />
        </button>
      {/if}
    </div>
    {#if navigatorOpen}
      <div
        bind:this={navigatorPanelElement}
        class="fixed z-80"
        style={navigatorPanelStyle}
        role="dialog"
        tabindex="-1"
        aria-label={navigatorMode === "notes" ? t("notes.noteNavigatorLabel") : t("projects.navigator.pickerLabel")}
      >
        {#if navigatorMode === "notes"}
          <NotesPagePickerPanel
            selectedPageId={notes.selectedPageId}
            projectId={selectedProjectId}
            panelMaxHeight={navigatorPanelMaxHeight}
            focusSearchRequestId={navigatorOpen ? 1 : 0}
            onPageSelected={() => {
              navigatorOpen = false;
            }}
          />
        {:else}
          <NotesProjectNavigator
            {selectedProjectId}
            selectedGroupId={selectedGroup?.id ?? null}
            {showInactiveProjects}
            panelMode={navigatorMode}
            panelMaxHeight={navigatorPanelMaxHeight}
            onShowInactiveProjectsChange={onShowInactiveProjectsChange}
            onProjectSelected={() => {
              navigatorOpen = false;
              onProjectSelected();
            }}
            onPageSelected={() => {
              navigatorOpen = false;
            }}
          />
        {/if}
      </div>
    {/if}
  </div>
  <div class="flex-1"></div>
  <div class="flex shrink-0 items-center gap-1">
    {#if selectedProject}
      <button
        type="button"
        data-notes-toolbar-trigger="settings"
        class={toolbarIconButtonClass(false, projectSettingsOpen)}
        aria-label={t("notes.projectSettingsTitle", selectedProject.name)}
        title={t("notes.projectSettingsTitle", selectedProject.name)}
        aria-expanded={projectSettingsOpen}
        onclick={onToggleProjectSettings}
      >
        <Settings2 size={14} strokeWidth={1.75} />
      </button>
    {/if}
  </div>
</div>
