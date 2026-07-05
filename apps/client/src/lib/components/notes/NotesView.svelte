<script lang="ts">
  import { onMount } from "svelte";
  import { hasOnlyShortcutModifier } from "$lib/keyboard-shortcuts";
  import { parseNotesLinkHash } from "$lib/notes/block-link";
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

  function notesShortcutTargetBlocked(target: EventTarget | Element | null): boolean {
    if (!(target instanceof Element)) return false;
    return isAppShortcutBlockedTarget(target) || target.closest("[role='dialog']") !== null;
  }

  function handleNotesWindowKeydown(event: KeyboardEvent): void {
    if (event.defaultPrevented) return;
    if (event.key.toLowerCase() !== "n" || !hasOnlyShortcutModifier(event)) return;
    if (
      notesShortcutTargetBlocked(event.target)
      || notesShortcutTargetBlocked(document.activeElement)
      || notesRootElement?.querySelector("[role='dialog']") !== null
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
  <div class="notes-view-layout min-h-0 flex-1 overflow-hidden">
    {#if notes.viewMode === "archive"}
      <NotesArchiveView />
    {:else if notes.viewMode === "trash"}
      <NotesTrashView />
    {:else if initialNotesLoadPending || (!notes.loaded && notes.loading)}
      <div class="min-w-0 flex-1" aria-busy="true"></div>
    {:else if notes.selectedPageId && notes.loadedPage}
      <NotesEditor projectId={selectedProjectId} />
    {:else}
      <NotesProjectHome projectId={selectedProjectId} />
    {/if}
  </div>
</div>

<style>
  .notes-view-root {
    container: notes-view / inline-size;
  }
</style>
