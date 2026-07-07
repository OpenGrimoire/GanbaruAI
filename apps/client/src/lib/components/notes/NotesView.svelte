<script lang="ts">
  import { onMount } from "svelte";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { parseNotesLinkHash } from "$lib/notes/block-link";
  import type { NotesPageOpenMode } from "$lib/notes/page-open-mode";
  import { getNotes } from "$lib/stores/notes.svelte";
  import { getProjects } from "$lib/stores/projects.svelte";
  import { isAppShortcutBlockedTarget } from "$lib/utils";
  import NotesArchiveView from "./NotesArchiveView.svelte";
  import NotesEditor from "./NotesEditor.svelte";
  import NotesProjectHome from "./NotesProjectHome.svelte";
  import NotesTrashView from "./NotesTrashView.svelte";
  import NotesWorkspaceHeader from "./NotesWorkspaceHeader.svelte";

  const notes = getNotes();
  const projects = getProjects();

  let showInactiveProjects = $state(false);
  let initialNotesLoadPending = $state(!notes.loaded);
  let notesRootElement = $state<HTMLDivElement | null>(null);
  const selectedProject = $derived(projects.selectedProject);
  const selectedGroup = $derived(projects.selectedGroup);
  const selectedProjectId = $derived(selectedProject?.id ?? null);
  const showFullPageEditor = $derived(
    notes.viewMode === "pages"
      && notes.selectedPageId !== null
      && notes.loadedPage !== null
      && notes.pageOpenMode === "full",
  );
  const showPagePeek = $derived(
    notes.viewMode === "pages"
      && notes.selectedPageId !== null
      && notes.loadedPage !== null
      && notes.pageOpenMode !== "full",
  );

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
    showProjectHome();
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
    selectedPage={notes.loadedPage}
    {showInactiveProjects}
    onShowInactiveProjectsChange={(value) => {
      showInactiveProjects = value;
    }}
    onProjectSelected={handleProjectSelected}
    onShowHome={showProjectHome}
  />
  <div class="notes-view-layout relative min-h-0 flex-1 overflow-hidden">
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

    {#if showPagePeek && notes.pageOpenMode === "center"}
      <div
        class="absolute inset-0 z-50 flex items-center justify-center bg-foreground/40 px-3 py-4 sm:px-6 sm:py-8"
        role="presentation"
        onclick={handleCenterPeekBackdropClick}
      >
        <div
          class="notes-center-peek-panel flex min-w-0 overflow-hidden rounded-lg border border-border bg-background shadow-2xl"
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
    {:else if showPagePeek && notes.pageOpenMode === "side"}
      <div
        class="absolute inset-y-0 right-0 z-40 flex w-[min(46rem,calc(100vw-2rem))] min-w-0 overflow-hidden border-l border-border bg-background shadow-2xl"
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
  </div>
</div>

<style>
  .notes-view-root {
    container: notes-view / inline-size;
  }

  .notes-center-peek-panel {
    width: min(960px, calc(100vw - 480px));
    height: min(667px, calc(100dvh - 214px));
  }

  @media (max-width: 760px), (max-height: 520px) {
    .notes-center-peek-panel {
      width: calc(100vw - 24px);
      height: calc(100dvh - 24px);
    }
  }
</style>
