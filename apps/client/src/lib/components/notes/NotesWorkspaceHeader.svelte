<script lang="ts">
  import Folder from "@lucide/svelte/icons/folder";
  import Settings2 from "@lucide/svelte/icons/settings-2";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    COMPACT_IDENTITY_EMOJI_SCALE,
    COMPACT_IDENTITY_ICON_SIZE,
    COMPACT_IDENTITY_ICON_STROKE_WIDTH,
  } from "$lib/icon-sizing";
  import { formatShortcut } from "$lib/keyboard-shortcuts";
  import { NOTES_PAGE_CHROME_EMOJI_SCALE } from "$lib/notes/page-icon";
  import {
    notesHierarchyChildren,
    notesHierarchyNodeParent,
    notesHierarchyPath,
    type NotesHierarchyNode,
    type NotesHierarchyParent,
  } from "$lib/notes/hierarchy-navigation";
  import { notesFoldersForProject } from "$lib/notes/navigation-tree";
  import { notesPageTitle } from "$lib/notes/page-title";
  import { notesPagesForProject } from "$lib/notes/project-membership";
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
  import WorkspaceBreadcrumbTerminalIcon from "$lib/components/WorkspaceBreadcrumbTerminalIcon.svelte";
  import NotesHierarchyPickerPanel from "./NotesHierarchyPickerPanel.svelte";
  import NotesPageIcon from "./NotesPageIcon.svelte";
  import NotesProjectNavigator from "./NotesProjectNavigator.svelte";

  type NotesNavigatorMode = ProjectNavigatorPanelMode | "notes";

  let {
    selectedProject,
    selectedGroup,
    selectedProjectId,
    selectedPage,
    explorerCollapsed,
    creationFolderId,
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
    explorerCollapsed: boolean;
    creationFolderId: string | null;
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
  const identityIconSize = COMPACT_IDENTITY_ICON_SIZE;
  const identityIconStrokeWidth = COMPACT_IDENTITY_ICON_STROKE_WIDTH;
  const projectIdentityEmojiScale = COMPACT_IDENTITY_EMOJI_SCALE;
  const newPageShortcut = $derived(formatShortcut("Mod + N"));
  const newPageTitle = $derived(`${t("notes.newPage")} (${newPageShortcut})`);

  let navigatorOpen = $state(false);
  let navigatorMode = $state<NotesNavigatorMode>("groups");
  let notesNavigatorParent = $state<NotesHierarchyParent>({ kind: "root" });
  let notesNavigatorSourceKey = $state("root");
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
  const selectedProjectPages = $derived.by(() => notesPagesForProject(
    [...new Map(
      [...notes.allPages, ...notes.linkResolutionPages].map((page) => [page.id, page]),
    ).values()],
    selectedProjectId,
  ));
  const selectedProjectFolders = $derived.by(() => notesFoldersForProject(
    notes.folders,
    selectedProjectId,
  ));
  const selectedPagePath = $derived(notesHierarchyPath(
    selectedPageId,
    selectedProjectPages,
    selectedProjectFolders,
    notes.sidebarPageIdsWithChildren,
  ));
  const showSelectedPagePath = $derived(explorerCollapsed && selectedPagePath.length > 0);
  const notesNavigatorItemCount = $derived(notesHierarchyChildren(
    selectedProjectPages,
    selectedProjectFolders,
    notesNavigatorParent,
    t("notes.untitled"),
    notes.sidebarPageIdsWithChildren,
  ).length);
  const notesNavigatorPanelHeight = $derived(Math.min(
    navigatorPanelMaxHeight,
    Math.max(124, notesNavigatorItemCount * 32 + 92),
  ));

  function toolbarIconButtonClass(active = false, open = false, primary = false): string {
    return cn(
      "flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-colors",
      primary
        ? "bg-primary text-primary-foreground hover:bg-primary/90"
        : "hover:bg-accent",
      !primary && "text-foreground",
      !primary && (active || open) && "bg-accent",
    );
  }

  function inlineNewPageButtonClass(): string {
    return cn(
      "flex h-7 w-5 shrink-0 items-center justify-center rounded-md text-foreground transition-colors",
      "hover:bg-accent",
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

  function openNavigator(
    mode: NotesNavigatorMode,
    anchor: HTMLButtonElement | null = triggerForMode(mode),
  ): void {
    navigatorMode = mode;
    navigatorAnchorElement = anchor;
    navigatorOpen = true;
    refreshNavigatorPanelGeometry();
    requestAnimationFrame(refreshNavigatorPanelGeometry);
  }

  function openHierarchyNavigator(
    node: NotesHierarchyNode,
    anchor: EventTarget | null,
  ): void {
    notesNavigatorParent = notesHierarchyNodeParent(node);
    notesNavigatorSourceKey = node.key;
    openNavigator("notes", anchor instanceof HTMLButtonElement ? anchor : null);
  }

  function toggleHierarchyNavigator(
    node: NotesHierarchyNode,
    anchor: EventTarget | null,
  ): void {
    if (navigatorOpen && navigatorMode === "notes" && notesNavigatorSourceKey === node.key) {
      navigatorOpen = false;
      return;
    }
    openHierarchyNavigator(node, anchor);
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
      && !notesIdentityElement?.contains(target)
      && !navigatorPanelElement?.contains(target)
    ) {
      navigatorOpen = false;
    }
  }

  function createPage(): void {
    navigatorOpen = false;
    void notes.createPage("", {
      projectId: selectedProjectId,
      folderId: creationFolderId,
      openMode: "full",
    });
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
  data-notes-workspace-header
>
  <div bind:this={notesIdentityElement} class="relative min-w-36 shrink-0 min-[760px]:max-w-xl">
    <div class="flex h-7 min-w-0 max-w-full items-center gap-0.5 text-sm">
      {#if selectedProject && selectedGroup}
        <button
          bind:this={groupTriggerElement}
          type="button"
          class={cn(
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md px-1.5 text-left hover:bg-accent",
            navigatorOpen && navigatorMode === "groups" && "bg-accent",
          )}
          aria-label={t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "groups"}
          onpointerenter={() => openNavigator("groups")}
          onclick={() => toggleNavigator("groups")}
        >
          <ProjectIcon
            name={selectedGroup.icon}
            size={identityIconSize}
            strokeWidth={identityIconStrokeWidth}
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
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md pl-1.5 text-left hover:bg-accent",
            selectedPageTitle ? "pr-1.5" : "pr-0.5",
            navigatorOpen && navigatorMode === "projects" && "bg-accent",
          )}
          aria-label={selectedPageTitle ? t("notes.showProjectHome") : t("projects.navigator.open")}
          aria-expanded={navigatorOpen && navigatorMode === "projects"}
          onpointerenter={() => openNavigator("projects")}
          onclick={handleProjectTriggerClick}
        >
          <ProjectIcon
            name={selectedProject.icon}
            size={identityIconSize}
            strokeWidth={identityIconStrokeWidth}
            emojiScale={projectIdentityEmojiScale}
            class="shrink-0"
          />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedProject.name}</span>
          {#if selectedProject.status !== "active"}
            <span class={cn("shrink-0 rounded border px-1.5 py-0.5 text-[0.666667rem]", projectLifecycleBadgeClass(selectedProject.status))}>
              {projectLifecycleLabel(selectedProject.status, t)}
            </span>
          {/if}
          {#if !showSelectedPagePath}
            <WorkspaceBreadcrumbTerminalIcon kind="chevron" context="notes" class="shrink-0 text-muted-foreground" />
          {/if}
        </button>
        {#if !showSelectedPagePath}
          <button
            type="button"
            class={inlineNewPageButtonClass()}
            aria-label={t("notes.newPage")}
            aria-keyshortcuts="Control+N Meta+N"
            title={newPageTitle}
            onclick={createPage}
          >
            <WorkspaceBreadcrumbTerminalIcon kind="plus" />
          </button>
        {/if}
        {#if showSelectedPagePath}
          {#each selectedPagePath as node, nodeIndex (node.key)}
            {@const pathTitle = node.kind === "folder"
              ? node.folder.name
              : node.page.id === selectedPageId && selectedPageTitle
                ? selectedPageTitle
                : notesPageTitle(node.page, t("notes.untitled"))}
            <span class="shrink-0 px-0.5 font-semibold text-muted-foreground">/</span>
            <button
              type="button"
              class={cn(
                "flex h-7 min-w-0 items-center gap-1.5 rounded-md px-1.5 text-left hover:bg-accent",
                navigatorOpen
                  && navigatorMode === "notes"
                  && notesNavigatorSourceKey === node.key
                  && "bg-accent",
              )}
              aria-label={pathTitle}
              aria-expanded={navigatorOpen && navigatorMode === "notes" && notesNavigatorSourceKey === node.key}
              onpointerenter={(event) => openHierarchyNavigator(node, event.currentTarget)}
              onclick={(event) => toggleHierarchyNavigator(node, event.currentTarget)}
            >
              {#if node.kind === "folder"}
                <Folder
                  size={identityIconSize}
                  strokeWidth={identityIconStrokeWidth}
                  class="shrink-0"
                />
              {:else}
                <NotesPageIcon
                  icon={node.page.icon}
                  size={identityIconSize}
                  strokeWidth={identityIconStrokeWidth}
                  emojiScale={NOTES_PAGE_CHROME_EMOJI_SCALE}
                  class="shrink-0"
                />
              {/if}
              <span class="min-w-0 truncate font-semibold text-foreground">{pathTitle}</span>
              {#if nodeIndex === selectedPagePath.length - 1}
                <WorkspaceBreadcrumbTerminalIcon kind="chevron" context="notes" class="shrink-0 text-muted-foreground" />
              {/if}
            </button>
          {/each}
          <button
            type="button"
            class={inlineNewPageButtonClass()}
            aria-label={t("notes.newPage")}
            aria-keyshortcuts="Control+N Meta+N"
            title={newPageTitle}
            onclick={createPage}
          >
            <WorkspaceBreadcrumbTerminalIcon kind="plus" />
          </button>
        {/if}
      {:else}
        <button
          bind:this={noteTriggerElement}
          type="button"
          class={cn(
            "flex h-7 min-w-0 items-center gap-1.5 rounded-md pl-1.5 pr-0.5 text-left hover:bg-accent",
            navigatorOpen && navigatorMode === "notes" && "bg-accent",
          )}
          aria-label={t("notes.openNoteNavigator")}
          aria-expanded={navigatorOpen && navigatorMode === "notes"}
          onclick={() => toggleNavigator("notes")}
        >
          <NotesPageIcon
            icon={selectedPage?.icon ?? null}
            size={identityIconSize}
            strokeWidth={identityIconStrokeWidth}
            emojiScale={NOTES_PAGE_CHROME_EMOJI_SCALE}
            class="shrink-0"
          />
          <span class="min-w-0 truncate font-semibold text-foreground">{selectedPageTitle ?? t("notes.title")}</span>
          <WorkspaceBreadcrumbTerminalIcon kind="chevron" class="shrink-0 text-muted-foreground" />
        </button>
        <button
          type="button"
          class={inlineNewPageButtonClass()}
          aria-label={t("notes.newPage")}
          aria-keyshortcuts="Control+N Meta+N"
          title={newPageTitle}
          onclick={createPage}
        >
          <WorkspaceBreadcrumbTerminalIcon kind="plus" />
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
          <NotesHierarchyPickerPanel
            projectId={selectedProjectId}
            parent={notesNavigatorParent}
            frameStyle={`width: 100%; height: ${notesNavigatorPanelHeight}px; max-height: ${notesNavigatorPanelHeight}px;`}
            className="relative"
            zIndexClass=""
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
