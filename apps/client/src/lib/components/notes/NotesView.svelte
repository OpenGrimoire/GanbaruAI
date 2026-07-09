<script lang="ts">
  import { onMount } from "svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { parseNotesLinkHash } from "$lib/notes/block-link";
  import type { NotesPageOpenMode } from "$lib/notes/page-open-mode";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { getViewport } from "$lib/stores/viewport.svelte";
  import { isAppShortcutBlockedTarget } from "$lib/utils";
  import NotesArchiveView from "./NotesArchiveView.svelte";
  import NotesEditor from "./NotesEditor.svelte";
  import NotesProjectHome from "./NotesProjectHome.svelte";
  import NotesProjectSettingsPanel from "./NotesProjectSettingsPanel.svelte";
  import NotesTrashView from "./NotesTrashView.svelte";
  import NotesWorkspaceHeader from "./NotesWorkspaceHeader.svelte";

  const notes = getNotes();
  const projects = getProjects();
  const viewport = getViewport();
  const { t } = getLocalization();

  const CENTER_PEEK_FULL_PAGE_MIN_WIDTH_PX = 608;
  const CENTER_PEEK_FULL_PAGE_MIN_HEIGHT_PX = 520;

  let showInactiveProjects = $state(false);
  let initialNotesLoadPending = $state(!notes.loaded);
  let notesRootElement = $state<HTMLDivElement | null>(null);
  let projectSettingsOpen = $state(false);
  let projectSettingsDirty = $state(false);
  let projectSettingsDiscardConfirmOpen = $state(false);
  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const topBarSelectedPage = $derived(notes.pageOpenMode === "full" ? notes.loadedPage : null);
  const hasOpenPage = $derived(
    notes.viewMode === "pages"
      && notes.selectedPageId !== null
      && notes.loadedPage !== null,
  );
  const peekPromotesToFullPage = $derived(
    hasOpenPage
      && notes.pageOpenMode !== "full"
      && (
        viewport.width < CENTER_PEEK_FULL_PAGE_MIN_WIDTH_PX
        || viewport.height < CENTER_PEEK_FULL_PAGE_MIN_HEIGHT_PX
      ),
  );
  const showFullPageEditor = $derived(
    hasOpenPage && (notes.pageOpenMode === "full" || peekPromotesToFullPage),
  );
  const showPagePeek = $derived(
    hasOpenPage && notes.pageOpenMode !== "full" && !peekPromotesToFullPage,
  );
  const showCenterPeek = $derived(showPagePeek && notes.pageOpenMode === "center");
  const showSidePeek = $derived(showPagePeek && notes.pageOpenMode === "side");

  onMount(() => {
    async function openHashTarget(): Promise<void> {
      const target = parseNotesLinkHash(window.location.hash);
      if (!target) {
        return;
      }
      const opened = await notes.openNotesLink(target);
      if (!opened) console.warn("notes link target was not found");
    }

    void notes.ensureLoaded()
      .then(openHashTarget)
      .catch((error) => {
        console.error("load notes failed", error);
      })
      .finally(() => {
        initialNotesLoadPending = false;
      });
    void projects.ensureLoaded().catch((error) => {
      console.error("load projects failed", error);
    });
    const onHashChange = () => {
      void openHashTarget().catch((error) => {
        console.error("open notes block link failed", error);
      });
    };
    window.addEventListener("hashchange", onHashChange);
    return () => {
      window.removeEventListener("hashchange", onHashChange);
    };
  });

  function showProjectHome(): void {
    if (notes.viewMode === "archive") notes.closeArchive();
    if (notes.viewMode === "trash") notes.closeTrash();
    void notes.selectPage(null);
  }

  function handleProjectSelected(): void {
    projectSettingsOpen = false;
    projectSettingsDirty = false;
    projectSettingsDiscardConfirmOpen = false;
    showProjectHome();
  }

  function closeProjectSettingsImmediately(): void {
    projectSettingsDiscardConfirmOpen = false;
    projectSettingsOpen = false;
    projectSettingsDirty = false;
  }

  function requestProjectSettingsClose(): void {
    if (projectSettingsDirty) {
      projectSettingsDiscardConfirmOpen = true;
      return;
    }
    closeProjectSettingsImmediately();
  }

  function toggleProjectSettings(): void {
    if (projectSettingsOpen) {
      requestProjectSettingsClose();
      return;
    }
    projectSettingsOpen = true;
  }

  function createPage(): void {
    void notes.createPage("", { projectId: selectedProjectId });
  }

  function closePagePeek(): void {
    void notes.selectPage(null);
  }

  function showSelectedPageAs(openMode: NotesPageOpenMode): void {
    notes.showSelectedPageAs(openMode);
  }

  function handleCenterPeekBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) closePagePeek();
  }

  function notesShortcutTargetBlocked(target: EventTarget | Element | null): boolean {
    if (!(target instanceof Element)) return false;
    return (
      isAppShortcutBlockedTarget(target)
      || target.closest("[role='dialog']:not([data-notes-page-peek])") !== null
    );
  }

  function handleNotesWindowKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    if (event.key.toLowerCase() !== "n" || !hasOnlyShortcutModifier(event)) return;
    if (
      notesShortcutTargetBlocked(event.target)
      || notesShortcutTargetBlocked(document.activeElement)
      || notesRootElement?.querySelector("[role='dialog']:not([data-notes-page-peek])") !== null
    ) {
      return;
    }
    event.preventDefault();
    createPage();
  }
</script>

<svelte:window onkeydown={handleNotesWindowKeydown} />

<div
  bind:this={notesRootElement}
  class="notes-view-root flex h-full min-h-0 flex-col overflow-hidden text-foreground"
  style="background-color: var(--cal-bg);"
>
  <NotesWorkspaceHeader
    {selectedProject}
    {selectedGroup}
    {selectedProjectId}
    selectedPage={topBarSelectedPage}
    {showInactiveProjects}
    onShowInactiveProjectsChange={(value) => {
      showInactiveProjects = value;
    }}
    onProjectSelected={handleProjectSelected}
    onShowHome={showProjectHome}
    {projectSettingsOpen}
    onToggleProjectSettings={toggleProjectSettings}
  />
  {#if projectSettingsOpen && selectedProjectId}
    <NotesProjectSettingsPanel
      projectId={selectedProjectId}
      popoverBoundaryElement={notesRootElement}
      onRequestClose={requestProjectSettingsClose}
      onDirtyChange={(dirty) => {
        projectSettingsDirty = dirty;
      }}
    />
  {/if}
  {#if projectSettingsDiscardConfirmOpen}
    <ConfirmDialog
      title={t("calendar.view.discardUnsavedTitle")}
      message={t("calendar.view.changesLost")}
      confirmLabel={t("calendar.view.discard")}
      cancelLabel={t("common.cancelShortcut")}
      onConfirm={closeProjectSettingsImmediately}
      onCancel={() => {
        projectSettingsDiscardConfirmOpen = false;
      }}
    />
  {/if}
  <div class="notes-view-layout relative flex min-h-0 flex-1 overflow-hidden">
    <div class={showSidePeek ? "min-w-0 basis-1/2 overflow-hidden" : "min-w-0 flex-1 overflow-hidden"}>
      {#if notes.viewMode === "archive"}
        <NotesArchiveView />
      {:else if notes.viewMode === "trash"}
        <NotesTrashView />
      {:else if initialNotesLoadPending || (!notes.loaded && notes.loading)}
        <div class="min-w-0 flex-1" aria-busy="true"></div>
      {:else if showFullPageEditor}
        <NotesEditor
          projectId={selectedProjectId}
          openMode="full"
          onClose={closePagePeek}
          onOpenModeChange={showSelectedPageAs}
        />
      {:else}
        <NotesProjectHome projectId={selectedProjectId} />
      {/if}
    </div>

    {#if showSidePeek}
      <div
        class="min-w-0 basis-1/2 overflow-hidden border-l border-border"
        role="dialog"
        aria-modal="false"
        data-notes-page-peek
      >
        <NotesEditor
          projectId={selectedProjectId}
          openMode="side"
          onClose={closePagePeek}
          onOpenModeChange={showSelectedPageAs}
        />
      </div>
    {/if}

    {#if showCenterPeek}
      <div
        class="fixed inset-x-0 bottom-0 z-50 flex items-center justify-center bg-black/45 px-3 py-4 sm:px-6 sm:py-8"
        style="top: calc(var(--titlebar-h) + var(--cal-header-row-h));"
        role="presentation"
        onclick={handleCenterPeekBackdropClick}
      >
        <div
          class="notes-center-peek-panel flex min-w-0 overflow-hidden rounded-lg border border-border"
          style="background-color: var(--cal-bg);"
          role="dialog"
          aria-modal="true"
          data-notes-page-peek
        >
          <NotesEditor
            projectId={selectedProjectId}
            openMode="center"
            onClose={closePagePeek}
            onOpenModeChange={showSelectedPageAs}
          />
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .notes-view-root {
    container: notes-view / inline-size;
  }

  .notes-center-peek-panel {
    width: clamp(560px, calc(100vw - 214px), 960px);
    height: min(667px, calc(100dvh - 214px));
  }
</style>
