<script lang="ts">
  import { onMount, tick } from "svelte";
  import Archive from "@lucide/svelte/icons/archive";
  import Search from "@lucide/svelte/icons/search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import { FALLBACK_COLOR_INDEX } from "$lib/components/calendar/types";
  import {
    archiveQuickNote,
    createQuickNoteTag,
    deleteQuickNotePermanently,
    emptyQuickNotesTrash,
    getQuickNote,
    listQuickNotes,
    listQuickNoteTags,
    restoreQuickNote,
    reorderQuickNote,
    setQuickNotePinned,
    trashQuickNote,
    unarchiveQuickNote,
    updateQuickNote,
  } from "$lib/api/quick-notes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { getQuickNoteColor } from "$lib/quick-notes/colors";
  import { quickNoteViewIndexForKey, quickNoteViewShortcut } from "$lib/quick-notes/tags";
  import { applyQuickNoteGroupOrder } from "$lib/quick-notes/masonry";
  import {
    cacheQuickNotesAllWindow,
    cacheQuickNotesViewWindow,
    cacheQuickNoteTags,
    preloadQuickNotesInitialSnapshot,
    readQuickNotesInitialSnapshot,
    readQuickNotesViewWindow,
  } from "$lib/quick-notes/initial-snapshot";
  import type { QuickNote, QuickNotesCollection, QuickNoteTag } from "$lib/quick-notes/types";
  import { listenForQuickNotesChanges, publishQuickNotesChanged } from "$lib/quick-notes/window-sync";
  import { getTheme } from "$lib/stores/theme.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import QuickNoteEditorModal from "./QuickNoteEditorModal.svelte";
  import QuickNoteTagManager from "./QuickNoteTagManager.svelte";
  import QuickNotesMasonry from "./QuickNotesMasonry.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const { t } = getLocalization();
  const theme = getTheme();
  const initialSnapshot = readQuickNotesInitialSnapshot();
  let panel = $state<HTMLDivElement | null>(null);
  let searchInput = $state<HTMLInputElement | null>(null);
  let collection = $state<QuickNotesCollection>("active");
  let renderedCollection = $state<QuickNotesCollection>("active");
  let selectedTagId = $state<string | null>(null);
  let tags = $state<QuickNoteTag[]>(initialSnapshot?.tags ?? []);
  let searchText = $state("");
  let searchOpen = $state(false);
  let animateLayout = $state(false);
  let notes = $state<QuickNote[]>(initialSnapshot?.window.notes ?? []);
  let nextCursor = $state<string | null>(initialSnapshot?.window.nextCursor ?? null);
  let loading = $state(false);
  let initializing = $state(initialSnapshot === null);
  let loadingMore = $state(false);
  let loadError = $state("");
  let editorNote = $state<QuickNote | null | undefined>(undefined);
  let deleteTarget = $state<QuickNote | null>(null);
  let confirmEmptyTrash = $state(false);
  let undoMessage = $state("");
  let reorderAnnouncement = $state("");
  let undoAction = $state<(() => Promise<void>) | null>(null);
  let undoTimer: ReturnType<typeof setTimeout> | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let loadGeneration = 0;
  let reorderWorkerRunning = false;
  const pendingReorders = new Map<string, {
    id: string;
    previousId: string | null;
    nextId: string | null;
    tagId: string | null;
  }>();

  const pinnedNotes = $derived(renderedCollection === "active" ? notes.filter((note) => note.pinned) : []);
  const otherNotes = $derived(renderedCollection === "active" ? notes.filter((note) => !note.pinned) : notes);
  const creationColors = $derived(getQuickNoteColor(FALLBACK_COLOR_INDEX, theme.current));
  const reorderable = $derived(
    collection === "active"
      && renderedCollection === "active"
      && searchText.trim() === ""
      && !loading,
  );
  const emptyMessage = $derived(searchText.trim()
    ? t("quickNotes.empty.search")
    : collection === "archive"
      ? t("quickNotes.empty.archive")
      : collection === "trash"
        ? t("quickNotes.empty.trash")
        : null);

  async function load(reset = true): Promise<void> {
    const generation = ++loadGeneration;
    const requestedCollection = collection;
    const requestedTagId = requestedCollection === "active" ? selectedTagId : null;
    const requestedSearchText = searchText.trim();
    const requestedCursor = reset ? null : nextCursor;
    if (reset) {
      loading = true;
      loadError = "";
    } else {
      loadingMore = true;
    }
    try {
      const window = await listQuickNotes(
        requestedCollection,
        requestedSearchText,
        requestedTagId,
        requestedCursor,
      );
      if (generation !== loadGeneration) return;
      notes = reset
        ? window.notes
        : [...notes, ...window.notes.filter((note) => !notes.some((current) => current.id === note.id))];
      if (reset) renderedCollection = requestedCollection;
      nextCursor = window.nextCursor;
      if (reset && requestedCollection === "active" && requestedTagId === null && requestedSearchText === "") {
        cacheQuickNotesAllWindow(window);
      } else if (reset && requestedSearchText === "") {
        cacheQuickNotesViewWindow(requestedCollection, requestedTagId, window);
      }
    } catch (error: unknown) {
      if (generation !== loadGeneration) return;
      loadError = error instanceof Error ? error.message : String(error);
    } finally {
      if (generation === loadGeneration) {
        loading = false;
        loadingMore = false;
      }
    }
  }

  function restoreCachedView(nextCollection: QuickNotesCollection, nextTagId: string | null): void {
    const cached = readQuickNotesViewWindow(nextCollection, nextTagId);
    if (!cached) return;
    notes = cached.notes;
    nextCursor = cached.nextCursor;
    renderedCollection = nextCollection;
  }

  function selectCollection(next: QuickNotesCollection): void {
    if (collection === next && selectedTagId === null) return;
    animateLayout = false;
    collection = next;
    selectedTagId = null;
    nextCursor = null;
    if (searchText.trim() === "") restoreCachedView(next, null);
    void load();
  }

  function selectTag(tagId: string | null): void {
    if (collection === "active" && selectedTagId === tagId) return;
    animateLayout = false;
    collection = "active";
    selectedTagId = tagId;
    nextCursor = null;
    if (searchText.trim() === "") restoreCachedView("active", tagId);
    void load();
  }

  async function loadTags(): Promise<void> {
    tags = await listQuickNoteTags();
    cacheQuickNoteTags(tags);
    if (selectedTagId && !tags.some((tag) => tag.id === selectedTagId)) {
      selectedTagId = null;
    }
  }

  async function createTag(name: string): Promise<void> {
    const created = await createQuickNoteTag(crypto.randomUUID(), name);
    tags = [...tags, created].sort((left, right) => left.sortOrder - right.sortOrder);
    publishQuickNotesChanged();
  }

  function tagTitle(tag: QuickNoteTag, index: number): string {
    return `${tag.name} (${t("calendar.toolbar.shortcutKey", quickNoteViewShortcut(index + 1) ?? "")})`;
  }

  function updateSearch(value: string): void {
    animateLayout = false;
    searchText = value;
    if (value.trim() === "") {
      restoreCachedView(collection, collection === "active" ? selectedTagId : null);
    }
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => void load(), 200);
  }

  async function openSearch(): Promise<void> {
    searchOpen = true;
    await tick();
    searchInput?.focus();
  }

  function closeSearch(): void {
    searchOpen = false;
    if (searchText) updateSearch("");
  }

  function revisionRequest(note: QuickNote): { id: string; expectedRevision: number } {
    return { id: note.id, expectedRevision: note.revision };
  }

  function showUndo(message: string, action: () => Promise<void>): void {
    if (undoTimer) clearTimeout(undoTimer);
    undoMessage = message;
    undoAction = action;
    undoTimer = setTimeout(() => {
      undoMessage = "";
      undoAction = null;
      undoTimer = null;
    }, 5_000);
  }

  async function runMutation(action: () => Promise<unknown>): Promise<void> {
    animateLayout = true;
    loadError = "";
    try {
      await action();
      publishQuickNotesChanged();
      await load();
    } catch (error: unknown) {
      loadError = error instanceof Error ? error.message : String(error);
    }
  }

  function pinNote(note: QuickNote, pinned: boolean): void {
    void runMutation(() => setQuickNotePinned(revisionRequest(note), pinned));
  }

  async function flushPendingReorders(): Promise<void> {
    if (reorderWorkerRunning) return;
    reorderWorkerRunning = true;
    try {
      while (pendingReorders.size > 0) {
        const requests = [...pendingReorders.values()];
        pendingReorders.clear();
        for (const request of requests) await reorderQuickNote(request);
        publishQuickNotesChanged();
      }
    } catch (error: unknown) {
      pendingReorders.clear();
      reorderAnnouncement = t("quickNotes.reorderFailed");
      loadError = error instanceof Error ? error.message : String(error);
      await load();
    } finally {
      reorderWorkerRunning = false;
      if (pendingReorders.size > 0) void flushPendingReorders();
    }
  }

  function reorderNoteGroup(
    orderedIds: readonly string[],
    movedId: string,
    position: number,
    pinned: boolean,
  ): void {
    if (!reorderable) return;
    notes = applyQuickNoteGroupOrder(notes, orderedIds);
    animateLayout = true;
    reorderAnnouncement = t("quickNotes.reordered", position + 1, orderedIds.length);
    pendingReorders.set(pinned ? "pinned" : "other", {
      id: movedId,
      previousId: orderedIds[position - 1] ?? null,
      nextId: orderedIds[position + 1] ?? null,
      tagId: selectedTagId,
    });
    void flushPendingReorders();
  }

  function colorNote(note: QuickNote, color: QuickNote["color"]): void {
    void runMutation(async () => {
      const full = await getQuickNote(note.id);
      await updateQuickNote({
        id: full.id,
        expectedRevision: full.revision,
        title: full.title,
        runs: full.runs,
        color,
        tagId: full.tagId,
        pinned: full.pinned,
      });
    });
  }

  function tagNote(note: QuickNote, tagId: string | null): void {
    void runMutation(async () => {
      const full = await getQuickNote(note.id);
      await updateQuickNote({
        id: full.id,
        expectedRevision: full.revision,
        title: full.title,
        runs: full.runs,
        color: full.color,
        tagId,
        pinned: full.pinned,
      });
    });
  }

  function archiveNote(note: QuickNote): void {
    void runMutation(async () => {
      const archived = await archiveQuickNote(revisionRequest(note));
      showUndo(t("quickNotes.movedToArchive"), async () => {
        await unarchiveQuickNote(revisionRequest(archived));
        publishQuickNotesChanged();
        await load();
      });
    });
  }

  function unarchiveNote(note: QuickNote): void {
    void runMutation(() => unarchiveQuickNote(revisionRequest(note)));
  }

  function trashNote(note: QuickNote): void {
    void runMutation(async () => {
      const trashed = await trashQuickNote(revisionRequest(note));
      showUndo(t("quickNotes.movedToTrash"), async () => {
        await restoreQuickNote(revisionRequest(trashed));
        publishQuickNotesChanged();
        await load();
      });
    });
  }

  function restoreNote(note: QuickNote): void {
    void runMutation(async () => {
      const restored = await restoreQuickNote(revisionRequest(note));
      showUndo(t("quickNotes.restored"), async () => {
        await trashQuickNote(revisionRequest(restored));
        publishQuickNotesChanged();
        await load();
      });
    });
  }

  async function openNote(note: QuickNote): Promise<void> {
    loadError = "";
    try {
      editorNote = await getQuickNote(note.id);
    } catch (error: unknown) {
      loadError = error instanceof Error ? error.message : String(error);
    }
  }

  function noteSaved(saved: QuickNote): void {
    animateLayout = true;
    if (selectedTagId && saved.tagId !== selectedTagId) {
      notes = notes.filter((note) => note.id !== saved.id);
      return;
    }
    const index = notes.findIndex((note) => note.id === saved.id);
    if (index === -1) notes = [saved, ...notes];
    else notes = notes.map((note) => note.id === saved.id ? saved : note);
  }

  function closeEditorAnd(action: (note: QuickNote) => void, note: QuickNote): void {
    editorNote = undefined;
    action(note);
  }

  async function confirmDelete(): Promise<void> {
    const target = deleteTarget;
    if (!target) return;
    deleteTarget = null;
    editorNote = undefined;
    await runMutation(() => deleteQuickNotePermanently(target.id));
  }

  async function emptyTrash(): Promise<void> {
    confirmEmptyTrash = false;
    await runMutation(() => emptyQuickNotesTrash());
  }

  function handlePanelKeydown(event: KeyboardEvent): void {
    if (editorNote !== undefined || deleteTarget || confirmEmptyTrash) return;
    const target = event.target instanceof Element ? event.target : null;
    const targetDialog = target?.closest("[role='dialog']") ?? null;
    if (event.key === "Escape") {
      if (document.documentElement.dataset.quickNoteDragging === "true") return;
      if (target?.closest("[data-quick-note-tag-creator]") !== null) return;
      if (targetDialog !== null && targetDialog !== panel) return;
      event.preventDefault();
      event.stopPropagation();
      onclose();
      return;
    }
    const shortcutIndex = quickNoteViewIndexForKey(event.key);
    const shortcutBlocked = event.ctrlKey || event.metaKey || event.altKey || event.shiftKey
      || target?.closest("input, textarea, [contenteditable='true']") !== null
      || (targetDialog !== null && targetDialog !== panel);
    if (shortcutIndex !== null && !shortcutBlocked) {
      if (shortcutIndex === 0) {
        event.preventDefault();
        selectTag(null);
      } else {
        const tag = tags[shortcutIndex - 1];
        if (tag) {
          event.preventDefault();
          selectTag(tag.id);
        }
      }
      return;
    }
    if (event.key !== "Tab" || !panel) return;
    const focusable = [...panel.querySelectorAll<HTMLElement>(
      "button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex='-1'])",
    )].filter((element) => element.offsetParent !== null);
    const first = focusable[0];
    const last = focusable.at(-1);
    if (first && last && event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (first && last && !event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function observeMore(node: HTMLElement): { destroy: () => void } {
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting) && nextCursor && !loadingMore) {
        animateLayout = true;
        void load(false);
      }
    }, { root: panel, rootMargin: "160px" });
    observer.observe(node);
    return { destroy: () => observer.disconnect() };
  }

  onMount(() => {
    const returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    window.addEventListener("keydown", handlePanelKeydown, true);
    if (initialSnapshot === null) {
      void preloadQuickNotesInitialSnapshot().then((snapshot) => {
        if (loadGeneration !== 0) return;
        tags = snapshot.tags;
        notes = snapshot.window.notes;
        nextCursor = snapshot.window.nextCursor;
      }).catch((error: unknown) => {
        loadError = error instanceof Error ? error.message : String(error);
      }).finally(() => {
        initializing = false;
      });
    }
    void tick().then(() => panel?.focus());
    let stopSync: (() => void) | null = null;
    void listenForQuickNotesChanges(() => {
      animateLayout = true;
      void loadTags();
      void load();
    }).then((unlisten) => { stopSync = unlisten; });
    return () => {
      window.removeEventListener("keydown", handlePanelKeydown, true);
      stopSync?.();
      if (searchTimer) clearTimeout(searchTimer);
      if (undoTimer) clearTimeout(undoTimer);
      queueMicrotask(() => returnFocus?.focus());
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="fixed inset-0 z-40" onclick={(event) => { if (event.target === event.currentTarget) onclose(); }}></div>
<div
  bind:this={panel}
  class="fixed right-2 z-50 flex w-[min(760px,calc(100vw-1rem))] flex-col overflow-hidden rounded-xl text-foreground shadow-lg outline-none"
  style="top: calc(var(--titlebar-h) + 4px); height: min(680px, calc(100dvh - var(--titlebar-h) - 12px)); background-color: var(--cal-bg);"
  role="dialog"
  aria-modal="true"
  aria-label={t("quickNotes.title")}
  tabindex="-1"
>
  <header class="shrink-0 px-3 pb-1.5 pt-3 sm:px-4">
    <div class="flex min-w-0 items-center gap-1">
      <div class="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto" role="group" aria-label={t("quickNotes.title")}>
        <button
          type="button"
          aria-pressed={collection === "active" && selectedTagId === null}
          class={`flex h-7 shrink-0 items-center rounded-md px-2.5 text-xs text-foreground transition-colors ${collection === "active" && selectedTagId === null ? "bg-accent/60" : "hover:bg-accent/60"}`}
          title={`${t("quickNotes.collection.active")} (${t("calendar.toolbar.shortcutKey", "1")})`}
          onclick={() => selectTag(null)}
        >{t("quickNotes.collection.active")}</button>
        {#each tags as tag, index}
          <button
            type="button"
            aria-pressed={collection === "active" && selectedTagId === tag.id}
            class={`flex h-7 max-w-32 shrink-0 items-center rounded-md px-2.5 text-xs text-foreground transition-colors ${collection === "active" && selectedTagId === tag.id ? "bg-accent/60" : "hover:bg-accent/60"}`}
            title={tagTitle(tag, index)}
            onclick={() => selectTag(tag.id)}
          ><span class="truncate">{tag.name}</span></button>
        {/each}
        <QuickNoteTagManager tagCount={tags.length} oncreate={createTag} />
      </div>
      <div class="ml-auto flex shrink-0 items-center gap-1" role="group" aria-label={t("quickNotes.title")}>
        {#if searchOpen}
          <div class="relative w-24 sm:w-48">
            <Search class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" strokeWidth={1.5} />
            <input
              bind:this={searchInput}
              type="search"
              value={searchText}
              class="h-7 w-full rounded-md border border-border bg-background/65 pl-8 pr-7 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
              placeholder={t("quickNotes.search")}
              aria-label={t("quickNotes.search")}
              oninput={(event) => updateSearch(event.currentTarget.value)}
            />
            <button type="button" class="absolute right-1 top-1/2 flex size-5 -translate-y-1/2 items-center justify-center rounded hover:bg-accent" aria-label={t("quickNotes.clearSearch")} onclick={closeSearch}><X class="size-3" strokeWidth={1.5} /></button>
          </div>
        {:else}
          <button
            type="button"
            aria-label={t("quickNotes.search")}
            title={t("quickNotes.search")}
            class="flex size-7 items-center justify-center rounded-md text-foreground transition-colors hover:bg-accent/60"
            onclick={() => void openSearch()}
          ><Search class="size-3.5" strokeWidth={1.5} /></button>
        {/if}
        <button
          type="button"
          aria-pressed={collection === "archive"}
          aria-label={t("quickNotes.collection.archive")}
          title={t("quickNotes.collection.archive")}
          class={`flex size-7 items-center justify-center rounded-md text-foreground transition-colors ${collection === "archive" ? "bg-accent" : "hover:bg-accent/60"}`}
          onclick={() => selectCollection("archive")}
        ><Archive class="size-3.5" strokeWidth={1.5} /></button>
        <button
          type="button"
          aria-pressed={collection === "trash"}
          aria-label={t("quickNotes.collection.trash")}
          title={t("quickNotes.collection.trash")}
          class={`flex size-7 items-center justify-center rounded-md text-foreground transition-colors ${collection === "trash" ? "bg-accent" : "hover:bg-accent/60"}`}
          onclick={() => selectCollection("trash")}
        ><Trash2 class="size-3.5" strokeWidth={1.5} /></button>
        <button
          type="button"
          class="flex size-7 items-center justify-center rounded-md text-foreground transition-colors hover:bg-accent/60"
          aria-label={t("common.close")}
          title={t("common.close")}
          onclick={onclose}
        ><X class="size-4" strokeWidth={1.5} /></button>
      </div>
    </div>
  </header>

  <div data-quick-notes-scroll class="min-h-0 flex-1 overflow-y-auto px-3 pb-3 pt-1.5 sm:px-4" aria-busy={loading}>
    {#if renderedCollection === "active"}
      <button
        type="button"
        class="mx-auto mb-4 flex min-h-12 w-2/3 items-center rounded-xl px-4 text-left text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        style="background-color: {creationColors.bg}; color: {creationColors.text};"
        onclick={() => { editorNote = null; }}
      >{t("quickNotes.takeNote")}</button>
    {:else if renderedCollection === "trash"}
      <div class="mb-3 flex items-center gap-2 rounded-lg bg-muted/45 px-3 py-2">
        <p class="min-w-0 flex-1 text-xs text-muted-foreground">{t("quickNotes.trashRetention")}</p>
        {#if notes.length > 0}<button type="button" class="shrink-0 rounded-md px-2 py-1 text-xs text-destructive hover:bg-destructive/10" onclick={() => { confirmEmptyTrash = true; }}>{t("quickNotes.action.emptyTrash")}</button>{/if}
      </div>
    {/if}

    {#if loadError && notes.length === 0}
      <div class="flex min-h-40 flex-col items-center justify-center gap-3 text-center">
        <p class="text-sm text-muted-foreground">{t("quickNotes.loadFailed")}</p>
        <button type="button" class="rounded-md border border-border px-3 py-1.5 text-sm hover:bg-accent" onclick={() => void load()}>{t("quickNotes.retry")}</button>
      </div>
    {:else if !initializing && notes.length === 0}
      {#if emptyMessage}<div class="flex min-h-40 items-center justify-center text-center text-sm text-muted-foreground">{emptyMessage}</div>{/if}
    {:else}
      {#if pinnedNotes.length > 0}
        <h3 class="mb-2 px-1 text-[0.7rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">{t("quickNotes.pinned")}</h3>
        <QuickNotesMasonry notes={pinnedNotes} collection={renderedCollection} {animateLayout} {reorderable} theme={theme.current} {tags} onopen={(note) => void openNote(note)} onreorder={(ids, id, position) => reorderNoteGroup(ids, id, position, true)} onpin={pinNote} oncolor={colorNote} ontag={tagNote} onarchive={archiveNote} onunarchive={unarchiveNote} ontrash={trashNote} onrestore={restoreNote} ondelete={(note) => { deleteTarget = note; }} />
      {/if}
      {#if otherNotes.length > 0}
        {#if pinnedNotes.length > 0}<h3 class="mb-2 mt-5 px-1 text-[0.7rem] font-semibold uppercase tracking-[0.12em] text-muted-foreground">{t("quickNotes.others")}</h3>{/if}
        <QuickNotesMasonry notes={otherNotes} collection={renderedCollection} {animateLayout} {reorderable} theme={theme.current} {tags} onopen={(note) => void openNote(note)} onreorder={(ids, id, position) => reorderNoteGroup(ids, id, position, false)} onpin={pinNote} oncolor={colorNote} ontag={tagNote} onarchive={archiveNote} onunarchive={unarchiveNote} ontrash={trashNote} onrestore={restoreNote} ondelete={(note) => { deleteTarget = note; }} />
      {/if}
      {#if nextCursor}
        <button use:observeMore type="button" class="mt-4 w-full rounded-md py-2 text-xs text-muted-foreground hover:bg-accent" disabled={loadingMore} onclick={() => { animateLayout = true; void load(false); }}>{loadingMore ? t("common.loading") : t("quickNotes.action.loadMore")}</button>
      {/if}
    {/if}
    {#if loadError && notes.length > 0}<p class="mt-3 text-center text-xs text-destructive">{loadError}</p>{/if}
    <p class="sr-only" aria-live="polite">{reorderAnnouncement}</p>
  </div>

  {#if undoAction}
    <div class="absolute bottom-3 left-1/2 z-10 flex -translate-x-1/2 items-center gap-3 rounded-lg bg-foreground px-3 py-2 text-xs text-background shadow-lg">
      <span>{undoMessage}</span>
      <button type="button" class="font-semibold underline" onclick={() => { const action = undoAction; undoAction = null; undoMessage = ""; if (action) void action(); }}>{t("quickNotes.action.undo")}</button>
    </div>
  {/if}
</div>

{#if editorNote !== undefined}
  <QuickNoteEditorModal
    note={editorNote}
    {tags}
    defaultTagId={selectedTagId}
    theme={theme.current}
    onclose={() => { editorNote = undefined; void load(); }}
    onsaved={noteSaved}
    onarchive={(note) => closeEditorAnd(archiveNote, note)}
    onunarchive={(note) => closeEditorAnd(unarchiveNote, note)}
    ontrash={(note) => closeEditorAnd(trashNote, note)}
    onrestore={(note) => closeEditorAnd(restoreNote, note)}
    ondelete={(note) => { deleteTarget = note; }}
  />
{/if}

{#if deleteTarget}
  <ConfirmDialog
    title={t("quickNotes.deleteConfirmTitle")}
    message={t("quickNotes.deleteConfirmMessage")}
    confirmLabel={t("quickNotes.deleteConfirm")}
    cancelLabel={t("common.cancel")}
    onConfirm={() => void confirmDelete()}
    onCancel={() => { deleteTarget = null; }}
  />
{/if}

{#if confirmEmptyTrash}
  <ConfirmDialog
    title={t("quickNotes.emptyTrashConfirmTitle")}
    message={t("quickNotes.emptyTrashConfirmMessage")}
    confirmLabel={t("quickNotes.emptyTrashConfirm")}
    cancelLabel={t("common.cancel")}
    onConfirm={() => void emptyTrash()}
    onCancel={() => { confirmEmptyTrash = false; }}
  />
{/if}
