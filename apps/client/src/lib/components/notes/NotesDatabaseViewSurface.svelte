<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    NOTES_DATABASE_VIEW_KINDS,
    type NotesDatabaseViewKind,
  } from "$lib/notes/database-view-kind";
  import NotesDatabaseBoardView from "./NotesDatabaseBoardView.svelte";
  import NotesDatabaseCalendarView from "./NotesDatabaseCalendarView.svelte";
  import NotesDatabaseGalleryView from "./NotesDatabaseGalleryView.svelte";
  import NotesDatabaseListView from "./NotesDatabaseListView.svelte";
  import NotesDatabaseTableView from "./NotesDatabaseTableView.svelte";
  import NotesDatabaseTimelineView from "./NotesDatabaseTimelineView.svelte";

  let {
    activeView,
    dataSourceId,
    databaseId,
    viewId,
    reloadKeys,
    onSelectPage,
    onActiveViewChange,
  }: {
    activeView: NotesDatabaseViewKind;
    dataSourceId: string;
    databaseId: string | null;
    viewId: string | null;
    reloadKeys: Record<NotesDatabaseViewKind, number>;
    onSelectPage: (pageId: string) => void;
    onActiveViewChange: (view: NotesDatabaseViewKind) => void;
  } = $props();

  const { t } = getLocalization();

  function viewLabel(view: NotesDatabaseViewKind): string {
    if (view === "table") return t("notes.databaseViewTable");
    if (view === "board") return t("notes.databaseViewBoard");
    if (view === "gallery") return t("notes.databaseViewGallery");
    if (view === "list") return t("notes.databaseViewList");
    if (view === "calendar") return t("notes.databaseViewCalendar");
    return t("notes.databaseViewTimeline");
  }

  function viewTabClass(view: NotesDatabaseViewKind): string {
    const base = "inline-flex h-8 items-center rounded-md px-2 text-[0.8rem]";
    return activeView === view
      ? `${base} bg-primary text-primary-foreground`
      : `${base} text-muted-foreground hover:bg-accent hover:text-foreground`;
  }
</script>

<div class="flex min-w-0 flex-wrap items-center gap-1 border-t border-border pt-3">
  {#each NOTES_DATABASE_VIEW_KINDS as view}
    <button
      type="button"
      class={viewTabClass(view)}
      aria-pressed={activeView === view}
      onclick={() => {
        onActiveViewChange(view);
      }}
    >
      {viewLabel(view)}
    </button>
  {/each}
</div>

{#if activeView === "table"}
  <NotesDatabaseTableView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.table}
  />
{:else if activeView === "board"}
  <NotesDatabaseBoardView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.board}
  />
{:else if activeView === "gallery"}
  <NotesDatabaseGalleryView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.gallery}
  />
{:else if activeView === "list"}
  <NotesDatabaseListView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.list}
  />
{:else if activeView === "calendar"}
  <NotesDatabaseCalendarView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.calendar}
  />
{:else}
  <NotesDatabaseTimelineView
    {dataSourceId}
    {databaseId}
    {viewId}
    {onSelectPage}
    reloadKey={reloadKeys.timeline}
  />
{/if}
