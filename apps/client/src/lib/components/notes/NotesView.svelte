<script lang="ts">
  import { onMount } from "svelte";
  import { parseNotesLinkHash } from "$lib/notes/block-link";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getNotes } from "$lib/stores/notes.svelte";
  import NotesArchiveView from "./NotesArchiveView.svelte";
  import NotesEditor from "./NotesEditor.svelte";
  import NotesEmptyState from "./NotesEmptyState.svelte";
  import NotesSidebar from "./NotesSidebar.svelte";
  import NotesTrashView from "./NotesTrashView.svelte";

  const notes = getNotes();
  const { t } = getLocalization();

  onMount(() => {
    async function openHashTarget(): Promise<void> {
      const target = parseNotesLinkHash(window.location.hash);
      if (!target) return;
      const opened = await notes.openNotesLink(target);
      if (!opened) console.warn("notes link target was not found");
    }

    void notes.ensureLoaded()
      .then(openHashTarget)
      .catch((error) => {
        console.error("load notes failed", error);
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

  function createFirstPage(): void {
    void notes.createPage("");
  }
</script>

<div class="notes-view-root h-full min-h-0 overflow-hidden text-foreground" style="background-color: var(--cal-bg);">
  <div class="notes-view-layout flex h-full min-h-0 overflow-hidden">
    <NotesSidebar />
    {#if notes.viewMode === "archive"}
      <NotesArchiveView />
    {:else if notes.viewMode === "trash"}
      <NotesTrashView />
    {:else if notes.loaded && notes.pages.length === 0}
      <div class="min-w-0 flex-1">
        <NotesEmptyState onCreate={createFirstPage} />
      </div>
    {:else}
      <NotesEditor />
    {/if}
  </div>
</div>

<style>
  .notes-view-root {
    container: notes-view / inline-size;
  }

  @container notes-view (max-width: 520px) {
    .notes-view-layout {
      flex-direction: column;
    }
  }
</style>
