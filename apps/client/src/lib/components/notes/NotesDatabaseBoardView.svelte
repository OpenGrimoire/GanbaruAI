<script lang="ts">
  import {
    createNotesDataSourceRowPage,
    duplicateNotesPage,
    getNotesDataSourceBoardView,
    moveNotesDataSourceBoardRow,
    trashNotesPage,
    updateNotesDataSourceBoardView,
  } from "$lib/api/notes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    notesDatabaseBoardCanMoveCards,
    notesDatabaseBoardCardText,
    notesDatabaseBoardCardTitle,
    notesDatabaseBoardColumns,
    notesDatabaseBoardConfigurationFromView,
    notesDatabaseBoardFiltersFromView,
    notesDatabaseBoardGroupableColumns,
    notesDatabaseBoardSortsFromView,
    notesDatabaseBoardUpdate,
    notesDatabaseBoardVisibleColumns,
  } from "$lib/notes/database-board";
  import type { NotesDatabaseTableColumn } from "$lib/notes/database-table";
  import type {
    NotesDatabaseBoardConfiguration,
    NotesDatabaseBoardRowOpenMode,
    NotesDatabaseTableFilter,
    NotesDatabaseTableFilterCondition,
    NotesDatabaseTableSort,
    NotesDataSourceBoardGroup,
    NotesDataSourceBoardView,
    NotesPage,
  } from "$lib/notes/types";
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import FileText from "@lucide/svelte/icons/file-text";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    dataSourceId,
    onSelectPage,
    reloadKey = 0,
  }: {
    dataSourceId: string;
    onSelectPage: (pageId: string) => void;
    reloadKey?: number;
  } = $props();

  const { t } = getLocalization();
  const FILTER_CONDITIONS: NotesDatabaseTableFilterCondition[] = [
    "contains",
    "equals",
    "is_empty",
    "is_not_empty",
    "checked",
    "unchecked",
  ];

  let board = $state<NotesDataSourceBoardView | null>(null);
  let loading = $state(false);
  let mutating = $state(false);
  let error = $state<string | null>(null);
  let draftTitleByGroup = $state<Record<string, string>>({});
  let selectedPanelRowId = $state<string | null>(null);
  let draggingRowId = $state<string | null>(null);
  let lastLoadSignature = $state("");

  const columns = $derived(board ? notesDatabaseBoardColumns(board.data_source, board.view) : []);
  const configuration = $derived(
    board ? notesDatabaseBoardConfigurationFromView(board.view) : defaultConfiguration(),
  );
  const groupableColumns = $derived(notesDatabaseBoardGroupableColumns(columns));
  const groupColumn = $derived(
    configuration.group_property_id
      ? groupableColumns.find((column) => column.id === configuration.group_property_id) ?? null
      : null,
  );
  const visibleColumns = $derived(notesDatabaseBoardVisibleColumns(columns, configuration));
  const filters = $derived(board ? notesDatabaseBoardFiltersFromView(board.view) : []);
  const sorts = $derived(board ? notesDatabaseBoardSortsFromView(board.view) : []);
  const visibleGroups = $derived(board ? board.groups.filter((group) => !group.hidden) : []);
  const hiddenGroups = $derived(board ? board.groups.filter((group) => group.hidden) : []);
  const cardCount = $derived(board?.groups.reduce((count, group) => count + group.rows.length, 0) ?? 0);
  const selectedPanelRow = $derived(findBoardRow(selectedPanelRowId));
  const canMoveCards = $derived(notesDatabaseBoardCanMoveCards(groupColumn));

  $effect(() => {
    const signature = `${dataSourceId}:${reloadKey}`;
    if (signature === lastLoadSignature) return;
    lastLoadSignature = signature;
    void loadBoard();
  });

  async function loadBoard(): Promise<NotesDataSourceBoardView | null> {
    loading = true;
    error = null;
    try {
      const loaded = await getNotesDataSourceBoardView(dataSourceId);
      board = loaded;
      if (selectedPanelRowId && !hasBoardRow(selectedPanelRowId, loaded.groups)) {
        selectedPanelRowId = null;
      }
      return loaded;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
      return null;
    } finally {
      loading = false;
    }
  }

  async function persistBoard(
    nextConfiguration: NotesDatabaseBoardConfiguration,
    nextVisibleColumns = visibleColumns,
    nextFilters = filters,
    nextSorts = sorts,
  ): Promise<void> {
    if (!board) return;
    mutating = true;
    error = null;
    try {
      board = await updateNotesDataSourceBoardView(
        dataSourceId,
        notesDatabaseBoardUpdate(
          nextConfiguration,
          board.groups,
          nextVisibleColumns,
          nextFilters,
          nextSorts,
        ),
      );
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function createCard(group: NotesDataSourceBoardGroup): Promise<void> {
    const title = (draftTitleByGroup[group.id] ?? "").trim() || t("notes.untitled");
    mutating = true;
    error = null;
    try {
      const loaded = await createNotesDataSourceRowPage(dataSourceId, {
        id: crypto.randomUUID(),
        first_block_id: crypto.randomUUID(),
        title,
      });
      draftTitleByGroup = { ...draftTitleByGroup, [group.id]: "" };
      if (canMoveCards) {
        await moveNotesDataSourceBoardRow(dataSourceId, {
          page_id: loaded.page.id,
          group_id: group.id,
        });
      }
      selectedPanelRowId = loaded.page.id;
      await loadBoard();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function moveCardToGroup(group: NotesDataSourceBoardGroup): Promise<void> {
    const pageId = draggingRowId;
    draggingRowId = null;
    if (!pageId || !canMoveCards) return;
    mutating = true;
    error = null;
    try {
      board = await moveNotesDataSourceBoardRow(dataSourceId, {
        page_id: pageId,
        group_id: group.id,
      });
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function duplicateCard(row: NotesPage): Promise<void> {
    mutating = true;
    error = null;
    try {
      const loaded = await duplicateNotesPage(row.id, {});
      selectedPanelRowId = loaded.page.id;
      await loadBoard();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function trashCard(row: NotesPage): Promise<void> {
    mutating = true;
    error = null;
    try {
      await trashNotesPage(row.id, true);
      if (selectedPanelRowId === row.id) selectedPanelRowId = null;
      await loadBoard();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  function updateGroupProperty(propertyId: string): void {
    const nextConfiguration = {
      ...configuration,
      group_property_id: propertyId || null,
      group_order: [],
      hidden_group_ids: [],
    };
    const nextVisibleColumns = notesDatabaseBoardVisibleColumns(columns, nextConfiguration);
    void persistBoard(nextConfiguration, nextVisibleColumns);
  }

  function updateRowOpenMode(mode: NotesDatabaseBoardRowOpenMode): void {
    void persistBoard({ ...configuration, row_open_mode: mode });
  }

  function updateCardProperty(columnId: string, visible: boolean): void {
    const byId = new Map(columns.map((column) => [column.id, column]));
    const nextIds = visible
      ? [...configuration.visible_property_ids, columnId]
      : configuration.visible_property_ids.filter((id) => id !== columnId);
    const nextVisibleColumns = nextIds
      .map((id) => byId.get(id))
      .filter((column): column is NotesDatabaseTableColumn =>
        column !== undefined && column.type !== "title" && column.id !== configuration.group_property_id
      );
    void persistBoard({ ...configuration, visible_property_ids: nextIds }, nextVisibleColumns);
  }

  function updateGroupHidden(group: NotesDataSourceBoardGroup, hidden: boolean): void {
    const hiddenIds = hidden
      ? [...configuration.hidden_group_ids, group.id]
      : configuration.hidden_group_ids.filter((id) => id !== group.id);
    void persistBoard({ ...configuration, hidden_group_ids: Array.from(new Set(hiddenIds)) });
  }

  function addSort(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    void persistBoard(configuration, visibleColumns, filters, [
      ...sorts,
      { property_id: firstColumn.id, direction: "ascending" },
    ]);
  }

  function updateSort(index: number, patch: Partial<NotesDatabaseTableSort>): void {
    const nextSorts = sorts.map((sort, sortIndex) =>
      sortIndex === index ? { ...sort, ...patch } : sort,
    );
    void persistBoard(configuration, visibleColumns, filters, nextSorts);
  }

  function removeSort(index: number): void {
    void persistBoard(
      configuration,
      visibleColumns,
      filters,
      sorts.filter((_, sortIndex) => sortIndex !== index),
    );
  }

  function addFilter(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    void persistBoard(configuration, visibleColumns, [
      ...filters,
      { property_id: firstColumn.id, condition: "contains", value: "" },
    ], sorts);
  }

  function updateFilter(index: number, patch: Partial<NotesDatabaseTableFilter>): void {
    const nextFilters = filters.map((filter, filterIndex) => {
      if (filterIndex !== index) return filter;
      const next = { ...filter, ...patch };
      if (!filterConditionNeedsValue(next.condition)) next.value = null;
      return next;
    });
    void persistBoard(configuration, visibleColumns, nextFilters, sorts);
  }

  function removeFilter(index: number): void {
    void persistBoard(
      configuration,
      visibleColumns,
      filters.filter((_, filterIndex) => filterIndex !== index),
      sorts,
    );
  }

  function openCard(row: NotesPage): void {
    if (configuration.row_open_mode === "side_panel") {
      selectedPanelRowId = row.id;
      return;
    }
    onSelectPage(row.id);
  }

  function rowTitle(row: NotesPage): string {
    return notesDatabaseBoardCardTitle(row, columns, t("notes.untitled"));
  }

  function findBoardRow(rowId: string | null): NotesPage | null {
    if (!rowId || !board) return null;
    for (const group of board.groups) {
      const row = group.rows.find((item) => item.id === rowId);
      if (row) return row;
    }
    return null;
  }

  function hasBoardRow(rowId: string, groups: NotesDataSourceBoardGroup[]): boolean {
    return groups.some((group) => group.rows.some((row) => row.id === rowId));
  }

  function filterConditionNeedsValue(condition: NotesDatabaseTableFilterCondition): boolean {
    return condition === "contains" || condition === "equals";
  }

  function filterConditionLabel(condition: NotesDatabaseTableFilterCondition): string {
    switch (condition) {
      case "contains":
        return t("notes.databaseTableFilterCondition.contains");
      case "equals":
        return t("notes.databaseTableFilterCondition.equals");
      case "is_empty":
        return t("notes.databaseTableFilterCondition.isEmpty");
      case "is_not_empty":
        return t("notes.databaseTableFilterCondition.isNotEmpty");
      case "checked":
        return t("notes.databaseTableFilterCondition.checked");
      case "unchecked":
        return t("notes.databaseTableFilterCondition.unchecked");
    }
  }

  function defaultConfiguration(): NotesDatabaseBoardConfiguration {
    return {
      group_property_id: null,
      group_order: [],
      hidden_group_ids: [],
      visible_property_ids: [],
      row_open_mode: "full_page",
    };
  }
</script>

<section class="space-y-3 border-t border-border pt-3" aria-label={t("notes.databaseBoardTitle")}>
  <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.8rem] text-muted-foreground">
    <span class="min-w-0 flex-1 truncate" role="status">
      {#if loading}
        {t("notes.databaseBoardLoading")}
      {:else if error}
        {t("notes.databaseBoardFailed", error)}
      {:else}
        {t("notes.databaseBoardCardsCount", cardCount)}
      {/if}
    </span>
    <label class="inline-flex min-w-0 items-center gap-1">
      <span>{t("notes.databaseBoardGroupBy")}</span>
      <select
        class="h-8 min-w-32 rounded-md border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
        value={configuration.group_property_id ?? ""}
        disabled={loading || mutating || !board}
        onchange={(event) => updateGroupProperty(event.currentTarget.value)}
        onkeydown={(event) => event.stopPropagation()}
      >
        <option value="">{t("notes.databaseBoardNoGroupProperty")}</option>
        {#each groupableColumns as column (column.id)}
          <option value={column.id}>{column.name}</option>
        {/each}
      </select>
    </label>
    <label class="inline-flex min-w-0 items-center gap-1">
      <span>{t("notes.databaseTableOpenMode")}</span>
      <select
        class="h-8 min-w-32 rounded-md border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
        value={configuration.row_open_mode}
        disabled={loading || mutating || !board}
        onchange={(event) => updateRowOpenMode(event.currentTarget.value as NotesDatabaseBoardRowOpenMode)}
        onkeydown={(event) => event.stopPropagation()}
      >
        <option value="full_page">{t("notes.databaseTableOpenFullPage")}</option>
        <option value="side_panel">{t("notes.databaseTableOpenSidePanel")}</option>
      </select>
    </label>
    <button
      type="button"
      class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
      disabled={loading || mutating}
      aria-label={t("notes.databaseBoardReload")}
      title={t("notes.databaseBoardReload")}
      onclick={() => {
        void loadBoard();
      }}
    >
      <RefreshCw class="size-3.5" aria-hidden="true" />
    </button>
  </div>

  {#if board}
    <div class="grid gap-2 @container">
      <div class="grid gap-2 @lg:grid-cols-3">
        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseBoardCardProperties")}</summary>
          <div class="mt-2 grid gap-1">
            {#each columns.filter((column) => column.type !== "title" && column.id !== configuration.group_property_id) as column (column.id)}
              <label class="flex min-w-0 items-center gap-2 rounded-sm px-1 py-0.5 hover:bg-accent/60">
                <input
                  type="checkbox"
                  checked={visibleColumns.some((visibleColumn) => visibleColumn.id === column.id)}
                  disabled={mutating}
                  onchange={(event) => updateCardProperty(column.id, event.currentTarget.checked)}
                  onkeydown={(event) => event.stopPropagation()}
                />
                <span class="min-w-0 flex-1 truncate">{column.name}</span>
              </label>
            {/each}
          </div>
        </details>

        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseTableSorts")}</summary>
          <div class="mt-2 grid gap-2">
            {#each sorts as sort, index}
              <div class="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] gap-1">
                <select
                  class="h-8 min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={sort.property_id}
                  disabled={mutating}
                  aria-label={t("notes.databaseTableSortProperty")}
                  onchange={(event) => updateSort(index, { property_id: event.currentTarget.value })}
                  onkeydown={(event) => event.stopPropagation()}
                >
                  {#each columns as column (column.id)}
                    <option value={column.id}>{column.name}</option>
                  {/each}
                </select>
                <select
                  class="h-8 min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={sort.direction}
                  disabled={mutating}
                  aria-label={t("notes.databaseTableSortDirection")}
                  onchange={(event) =>
                    updateSort(index, {
                      direction: event.currentTarget.value === "descending" ? "descending" : "ascending",
                    })}
                  onkeydown={(event) => event.stopPropagation()}
                >
                  <option value="ascending">{t("notes.databaseTableSortAscending")}</option>
                  <option value="descending">{t("notes.databaseTableSortDescending")}</option>
                </select>
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
                  disabled={mutating}
                  aria-label={t("notes.databaseTableRemoveSort")}
                  title={t("notes.databaseTableRemoveSort")}
                  onclick={() => removeSort(index)}
                >
                  <Trash2 class="size-3.5" aria-hidden="true" />
                </button>
              </div>
            {/each}
            <button
              type="button"
              class="inline-flex h-8 items-center gap-1 rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating || columns.length === 0}
              onclick={addSort}
            >
              <Plus class="size-3.5" aria-hidden="true" />
              <span>{t("notes.databaseTableAddSort")}</span>
            </button>
          </div>
        </details>

        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseTableFilters")}</summary>
          <div class="mt-2 grid gap-2">
            {#each filters as filter, index}
              <div class="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1fr)_minmax(0,1fr)_auto] gap-1">
                <select
                  class="h-8 min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={filter.property_id}
                  disabled={mutating}
                  aria-label={t("notes.databaseTableFilterProperty")}
                  onchange={(event) => updateFilter(index, { property_id: event.currentTarget.value })}
                  onkeydown={(event) => event.stopPropagation()}
                >
                  {#each columns as column (column.id)}
                    <option value={column.id}>{column.name}</option>
                  {/each}
                </select>
                <select
                  class="h-8 min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={filter.condition}
                  disabled={mutating}
                  aria-label={t("notes.databaseTableFilterConditionLabel")}
                  onchange={(event) =>
                    updateFilter(index, {
                      condition: event.currentTarget.value as NotesDatabaseTableFilterCondition,
                    })}
                  onkeydown={(event) => event.stopPropagation()}
                >
                  {#each FILTER_CONDITIONS as condition}
                    <option value={condition}>{filterConditionLabel(condition)}</option>
                  {/each}
                </select>
                <input
                  class="h-8 min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
                  value={String(filter.value ?? "")}
                  disabled={mutating || !filterConditionNeedsValue(filter.condition)}
                  aria-label={t("notes.databaseTableFilterValue")}
                  onblur={(event) => updateFilter(index, { value: event.currentTarget.value })}
                  onkeydown={(event) => event.stopPropagation()}
                />
                <button
                  type="button"
                  class="inline-flex size-8 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
                  disabled={mutating}
                  aria-label={t("notes.databaseTableRemoveFilter")}
                  title={t("notes.databaseTableRemoveFilter")}
                  onclick={() => removeFilter(index)}
                >
                  <Trash2 class="size-3.5" aria-hidden="true" />
                </button>
              </div>
            {/each}
            <button
              type="button"
              class="inline-flex h-8 items-center gap-1 rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating || columns.length === 0}
              onclick={addFilter}
            >
              <Plus class="size-3.5" aria-hidden="true" />
              <span>{t("notes.databaseTableAddFilter")}</span>
            </button>
          </div>
        </details>
      </div>

      {#if hiddenGroups.length > 0}
        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseBoardHiddenGroups")}</summary>
          <div class="mt-2 flex min-w-0 flex-wrap gap-1">
            {#each hiddenGroups as group (group.id)}
              <button
                type="button"
                class="inline-flex h-8 items-center gap-1 rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
                disabled={mutating}
                aria-label={t("notes.databaseBoardShowGroup", group.name)}
                onclick={() => updateGroupHidden(group, false)}
              >
                <Eye class="size-3.5" aria-hidden="true" />
                <span class="max-w-32 truncate">{group.name}</span>
              </button>
            {/each}
          </div>
        </details>
      {/if}

      <div class="min-w-0 overflow-x-auto pb-1">
        <div class="grid min-w-max auto-cols-72 grid-flow-col gap-3">
          {#each visibleGroups as group (group.id)}
            <section
              class="flex max-h-136 min-h-72 min-w-0 flex-col rounded-md border border-border bg-muted/20"
              aria-label={group.name}
              ondragover={(event) => {
                if (!canMoveCards) return;
                event.preventDefault();
              }}
              ondrop={(event) => {
                event.preventDefault();
                void moveCardToGroup(group);
              }}
            >
              <div class="flex min-w-0 items-center gap-2 border-b border-border px-2 py-2">
                <span class="min-w-0 flex-1 truncate text-[0.866667rem] font-medium text-foreground">
                  {group.name}
                </span>
                <span class="shrink-0 text-[0.733333rem] text-muted-foreground">
                  {group.rows.length}
                </span>
                <button
                  type="button"
                  class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
                  disabled={mutating}
                  aria-label={t("notes.databaseBoardHideGroup", group.name)}
                  title={t("notes.databaseBoardHideGroup", group.name)}
                  onclick={() => updateGroupHidden(group, true)}
                >
                  <EyeOff class="size-3.5" aria-hidden="true" />
                </button>
              </div>
              <div class="min-h-0 flex-1 space-y-2 overflow-y-auto p-2">
                {#if !canMoveCards && group.rows.length > 0}
                  <p class="text-[0.733333rem] text-muted-foreground">
                    {t("notes.databaseBoardDropDisabled")}
                  </p>
                {/if}
                {#each group.rows as row (row.id)}
                  {@const title = rowTitle(row)}
                  <article
                    class="rounded-md border border-border bg-background p-2 shadow-sm"
                    draggable={canMoveCards && !mutating}
                    ondragstart={() => {
                      draggingRowId = row.id;
                    }}
                    ondragend={() => {
                      draggingRowId = null;
                    }}
                  >
                    <div class="flex min-w-0 items-start gap-1">
                      <GripVertical class="mt-1 size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
                      <button
                        type="button"
                        class="min-w-0 flex-1 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        onclick={() => openCard(row)}
                      >
                        <span class="block truncate text-[0.866667rem] font-medium text-foreground">
                          {title}
                        </span>
                      </button>
                      <button
                        type="button"
                        class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                        aria-label={t("notes.databaseRowsOpen", title)}
                        title={t("notes.databaseRowsOpen", title)}
                        onclick={() => openCard(row)}
                      >
                        <FileText class="size-3.5" aria-hidden="true" />
                      </button>
                    </div>
                    {#if visibleColumns.length > 0}
                      <dl class="mt-2 grid gap-1">
                        {#each visibleColumns as column (column.id)}
                          {@const text = notesDatabaseBoardCardText(row, column)}
                          <div class="min-w-0">
                            <dt class="truncate text-[0.666667rem] text-muted-foreground">{column.name}</dt>
                            <dd class="truncate text-[0.8rem] text-foreground">
                              {text || t("notes.databaseTableEmptyCell")}
                            </dd>
                          </div>
                        {/each}
                      </dl>
                    {/if}
                    <div class="mt-2 flex justify-end gap-1">
                      <button
                        type="button"
                        class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
                        disabled={mutating}
                        aria-label={t("notes.databaseRowsDuplicate", title)}
                        title={t("notes.databaseRowsDuplicate", title)}
                        onclick={() => {
                          void duplicateCard(row);
                        }}
                      >
                        <Copy class="size-3.5" aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        class="inline-flex size-7 items-center justify-center rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
                        disabled={mutating}
                        aria-label={t("notes.databaseRowsTrash", title)}
                        title={t("notes.databaseRowsTrash", title)}
                        onclick={() => {
                          void trashCard(row);
                        }}
                      >
                        <Trash2 class="size-3.5" aria-hidden="true" />
                      </button>
                    </div>
                  </article>
                {/each}
              </div>
              <div class="border-t border-border p-2">
                <input
                  class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
                  value={draftTitleByGroup[group.id] ?? ""}
                  placeholder={t("notes.databaseRowsNewPlaceholder")}
                  disabled={loading || mutating}
                  oninput={(event) => {
                    draftTitleByGroup = {
                      ...draftTitleByGroup,
                      [group.id]: event.currentTarget.value,
                    };
                  }}
                  onkeydown={(event) => {
                    event.stopPropagation();
                    if (event.key === "Enter") {
                      event.preventDefault();
                      void createCard(group);
                    }
                  }}
                />
                <button
                  type="button"
                  class="mt-1 inline-flex h-8 w-full items-center justify-center gap-1 rounded-md px-2 text-[0.8rem] hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
                  disabled={loading || mutating}
                  onclick={() => {
                    void createCard(group);
                  }}
                >
                  <Plus class="size-3.5" aria-hidden="true" />
                  <span>{t("notes.databaseBoardAddCard")}</span>
                </button>
              </div>
            </section>
          {/each}
        </div>
      </div>

      {#if visibleColumns.length === 0}
        <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseBoardNoVisibleProperties")}</p>
      {/if}

      {#if cardCount === 0 && !loading && !error}
        <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseRowsEmpty")}</p>
      {/if}

      {#if configuration.row_open_mode === "side_panel"}
        <aside class="rounded-md border border-border p-3" aria-label={t("notes.databaseTableSidePanelTitle")}>
          {#if selectedPanelRow}
            {@const title = rowTitle(selectedPanelRow)}
            <div class="flex min-w-0 items-start gap-2">
              <div class="min-w-0 flex-1">
                <h3 class="truncate text-[0.933333rem] font-medium text-foreground">{title}</h3>
                <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseTableSidePanelSubtitle")}</p>
              </div>
              <button
                type="button"
                class="inline-flex h-8 items-center gap-1 rounded-md px-2 text-[0.8rem] hover:bg-accent"
                onclick={() => onSelectPage(selectedPanelRow.id)}
              >
                <ExternalLink class="size-3.5" aria-hidden="true" />
                <span>{t("notes.databaseTableOpenFullPage")}</span>
              </button>
            </div>
            <dl class="mt-3 grid gap-2 @lg:grid-cols-2">
              {#each columns as column (column.id)}
                <div class="min-w-0 rounded-sm bg-muted/40 p-2">
                  <dt class="truncate text-[0.733333rem] text-muted-foreground">{column.name}</dt>
                  <dd class="mt-0.5 min-w-0 truncate text-[0.866667rem] text-foreground">
                    {notesDatabaseBoardCardText(selectedPanelRow, column) || t("notes.databaseTableEmptyCell")}
                  </dd>
                </div>
              {/each}
            </dl>
          {:else}
            <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseTableSidePanelEmpty")}</p>
          {/if}
        </aside>
      {/if}
    </div>
  {/if}

  {#if mutating}
    <p class="flex items-center gap-1 text-[0.8rem] text-muted-foreground">
      <Check class="size-3.5" aria-hidden="true" />
      <span>{t("notes.databaseBoardSaving")}</span>
    </p>
  {/if}
</section>
