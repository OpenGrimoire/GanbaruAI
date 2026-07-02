<script lang="ts">
  import {
    createNotesDataSourceRowPage,
    duplicateNotesPage,
    getNotesDataSourceListView,
    trashNotesPage,
    updateNotesDataSourceListView,
  } from "$lib/api/notes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    notesDatabaseListColumns,
    notesDatabaseListConfigurationFromView,
    notesDatabaseListFiltersFromView,
    notesDatabaseListGroupableColumns,
    notesDatabaseListGroups,
    notesDatabaseListRowText,
    notesDatabaseListRowTitle,
    notesDatabaseListSortsFromView,
    notesDatabaseListUpdate,
    notesDatabaseListVisibleColumns,
  } from "$lib/notes/database-list";
  import type { NotesDatabaseTableColumn } from "$lib/notes/database-table";
  import type {
    NotesDatabaseListConfiguration,
    NotesDatabaseListRowOpenMode,
    NotesDatabaseTableFilter,
    NotesDatabaseTableFilterCondition,
    NotesDatabaseTableSort,
    NotesDataSourceListGroup,
    NotesDataSourceListView,
    NotesPage,
  } from "$lib/notes/types";
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import FileText from "@lucide/svelte/icons/file-text";
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

  let list = $state<NotesDataSourceListView | null>(null);
  let loading = $state(false);
  let mutating = $state(false);
  let error = $state<string | null>(null);
  let draftTitle = $state("");
  let selectedPanelRowId = $state<string | null>(null);
  let lastLoadSignature = $state("");

  const columns = $derived(list ? notesDatabaseListColumns(list.data_source, list.view) : []);
  const configuration = $derived(
    list ? notesDatabaseListConfigurationFromView(list.view) : defaultConfiguration(),
  );
  const groupableColumns = $derived(notesDatabaseListGroupableColumns(columns));
  const groupColumn = $derived(
    configuration.group_property_id
      ? groupableColumns.find((column) => column.id === configuration.group_property_id) ?? null
      : null,
  );
  const visibleColumns = $derived(notesDatabaseListVisibleColumns(columns, configuration));
  const filters = $derived(list ? notesDatabaseListFiltersFromView(list.view) : []);
  const sorts = $derived(list ? notesDatabaseListSortsFromView(list.view) : []);
  const groups = $derived(list ? notesDatabaseListGroups(list.rows, columns, configuration) : []);
  const visibleGroups = $derived(groups.filter((group) => !group.hidden));
  const hiddenGroups = $derived(groups.filter((group) => group.hidden));
  const selectedPanelRow = $derived(list?.rows.find((row) => row.id === selectedPanelRowId) ?? null);

  $effect(() => {
    const signature = `${dataSourceId}:${reloadKey}`;
    if (signature === lastLoadSignature) return;
    lastLoadSignature = signature;
    void loadList();
  });

  async function loadList(): Promise<NotesDataSourceListView | null> {
    loading = true;
    error = null;
    try {
      const loaded = await getNotesDataSourceListView(dataSourceId);
      list = loaded;
      if (selectedPanelRowId && !loaded.rows.some((row) => row.id === selectedPanelRowId)) {
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

  async function persistList(
    nextConfiguration: NotesDatabaseListConfiguration,
    nextVisibleColumns = visibleColumns,
    nextFilters = filters,
    nextSorts = sorts,
  ): Promise<void> {
    if (!list) return;
    mutating = true;
    error = null;
    try {
      list = await updateNotesDataSourceListView(
        dataSourceId,
        notesDatabaseListUpdate(nextConfiguration, nextVisibleColumns, nextFilters, nextSorts),
      );
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function createRow(): Promise<void> {
    const title = draftTitle.trim() || t("notes.untitled");
    mutating = true;
    error = null;
    try {
      const loaded = await createNotesDataSourceRowPage(dataSourceId, {
        id: crypto.randomUUID(),
        first_block_id: crypto.randomUUID(),
        title,
      });
      draftTitle = "";
      selectedPanelRowId = loaded.page.id;
      await loadList();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function duplicateRow(row: NotesPage): Promise<void> {
    mutating = true;
    error = null;
    try {
      const loaded = await duplicateNotesPage(row.id, {});
      selectedPanelRowId = loaded.page.id;
      await loadList();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function trashRow(row: NotesPage): Promise<void> {
    mutating = true;
    error = null;
    try {
      await trashNotesPage(row.id, true);
      if (selectedPanelRowId === row.id) selectedPanelRowId = null;
      await loadList();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  function updateGroupProperty(propertyId: string): void {
    const groupPropertyId = propertyId || null;
    const nextConfiguration: NotesDatabaseListConfiguration = {
      ...configuration,
      group_property_id: groupPropertyId,
      group_order: [],
      hidden_group_ids: [],
      visible_property_ids: configuration.visible_property_ids.filter((id) => id !== groupPropertyId),
    };
    const nextVisibleColumns = visibleColumns.filter((column) => column.id !== groupPropertyId);
    void persistList(nextConfiguration, nextVisibleColumns);
  }

  function updateRowOpenMode(mode: NotesDatabaseListRowOpenMode): void {
    void persistList({ ...configuration, row_open_mode: mode });
  }

  function updateRowProperty(columnId: string, visible: boolean): void {
    const byId = new Map(columns.map((column) => [column.id, column]));
    const nextIds = visible
      ? [...configuration.visible_property_ids, columnId]
      : configuration.visible_property_ids.filter((id) => id !== columnId);
    const nextVisibleColumns = nextIds
      .map((id) => byId.get(id))
      .filter((column): column is NotesDatabaseTableColumn =>
        column !== undefined && column.type !== "title" && column.id !== configuration.group_property_id
      );
    void persistList({ ...configuration, visible_property_ids: nextIds }, nextVisibleColumns);
  }

  function setGroupHidden(groupId: string, hidden: boolean): void {
    const hiddenGroupIds = hidden
      ? [...configuration.hidden_group_ids, groupId]
      : configuration.hidden_group_ids.filter((id) => id !== groupId);
    void persistList({ ...configuration, hidden_group_ids: uniqueStrings(hiddenGroupIds) });
  }

  function addSort(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    void persistList(configuration, visibleColumns, filters, [
      ...sorts,
      { property_id: firstColumn.id, direction: "ascending" },
    ]);
  }

  function updateSort(index: number, patch: Partial<NotesDatabaseTableSort>): void {
    const nextSorts = sorts.map((sort, sortIndex) =>
      sortIndex === index ? { ...sort, ...patch } : sort,
    );
    void persistList(configuration, visibleColumns, filters, nextSorts);
  }

  function removeSort(index: number): void {
    void persistList(
      configuration,
      visibleColumns,
      filters,
      sorts.filter((_, sortIndex) => sortIndex !== index),
    );
  }

  function addFilter(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    void persistList(configuration, visibleColumns, [
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
    void persistList(configuration, visibleColumns, nextFilters, sorts);
  }

  function removeFilter(index: number): void {
    void persistList(
      configuration,
      visibleColumns,
      filters.filter((_, filterIndex) => filterIndex !== index),
      sorts,
    );
  }

  function openRow(row: NotesPage): void {
    if (configuration.row_open_mode === "side_panel") {
      selectedPanelRowId = row.id;
      return;
    }
    onSelectPage(row.id);
  }

  function rowTitle(row: NotesPage): string {
    return notesDatabaseListRowTitle(row, columns, t("notes.untitled"));
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

  function uniqueStrings(values: string[]): string[] {
    return Array.from(new Set(values.filter(Boolean)));
  }

  function defaultConfiguration(): NotesDatabaseListConfiguration {
    return {
      group_property_id: null,
      group_order: [],
      hidden_group_ids: [],
      visible_property_ids: [],
      row_open_mode: "side_panel",
    };
  }
</script>

<section class="space-y-3 border-t border-border pt-3" aria-label={t("notes.databaseListTitle")}>
  <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.8rem] text-muted-foreground">
    <span class="min-w-0 flex-1 truncate" role="status">
      {#if loading}
        {t("notes.databaseListLoading")}
      {:else if error}
        {t("notes.databaseListFailed", error)}
      {:else}
        {t("notes.databaseListRowsCount", list?.rows.length ?? 0)}
      {/if}
    </span>
    <label class="inline-flex min-w-0 items-center gap-1">
      <span>{t("notes.databaseListGroupBy")}</span>
      <select
        class="h-8 min-w-36 rounded-md border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
        value={configuration.group_property_id ?? ""}
        disabled={loading || mutating || !list}
        onchange={(event) => updateGroupProperty(event.currentTarget.value)}
        onkeydown={(event) => event.stopPropagation()}
      >
        <option value="">{t("notes.databaseListNoGroupProperty")}</option>
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
        disabled={loading || mutating || !list}
        onchange={(event) => updateRowOpenMode(event.currentTarget.value as NotesDatabaseListRowOpenMode)}
        onkeydown={(event) => event.stopPropagation()}
      >
        <option value="side_panel">{t("notes.databaseTableOpenSidePanel")}</option>
        <option value="full_page">{t("notes.databaseTableOpenFullPage")}</option>
      </select>
    </label>
    <button
      type="button"
      class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
      disabled={loading || mutating}
      aria-label={t("notes.databaseListReload")}
      title={t("notes.databaseListReload")}
      onclick={() => {
        void loadList();
      }}
    >
      <RefreshCw class="size-3.5" aria-hidden="true" />
    </button>
  </div>

  <div class="flex min-w-0 flex-wrap items-center gap-2">
    <input
      class="h-8 min-w-40 flex-1 rounded-md border border-input bg-background px-2 text-[0.866667rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
      value={draftTitle}
      placeholder={t("notes.databaseRowsNewPlaceholder")}
      disabled={loading || mutating}
      oninput={(event) => {
        draftTitle = event.currentTarget.value;
      }}
      onkeydown={(event) => {
        event.stopPropagation();
        if (event.key === "Enter") {
          event.preventDefault();
          void createRow();
        }
      }}
    />
    <button
      type="button"
      class="inline-flex h-8 items-center gap-1 rounded-md bg-primary px-2 text-[0.8rem] text-primary-foreground disabled:pointer-events-none disabled:opacity-50"
      disabled={loading || mutating}
      onclick={() => {
        void createRow();
      }}
    >
      <Plus class="size-3.5" aria-hidden="true" />
      <span>{t("notes.databaseListAddRow")}</span>
    </button>
  </div>

  {#if list}
    <div class="grid gap-2 @container">
      <div class="grid gap-2 @lg:grid-cols-4">
        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseListRowProperties")}</summary>
          <div class="mt-2 grid gap-1">
            {#each columns.filter((column) =>
              column.type !== "title" && column.id !== configuration.group_property_id
            ) as column (column.id)}
              <label class="flex min-w-0 items-center gap-2 rounded-sm px-1 py-0.5 hover:bg-accent/60">
                <input
                  type="checkbox"
                  checked={visibleColumns.some((visibleColumn) => visibleColumn.id === column.id)}
                  disabled={mutating}
                  onchange={(event) => updateRowProperty(column.id, event.currentTarget.checked)}
                  onkeydown={(event) => event.stopPropagation()}
                />
                <span class="min-w-0 flex-1 truncate">{column.name}</span>
              </label>
            {/each}
          </div>
        </details>

        {#if groupColumn}
          <details class="rounded-md border border-border p-2 text-[0.8rem]">
            <summary class="cursor-pointer text-foreground">{t("notes.databaseListHiddenGroups")}</summary>
            <div class="mt-2 grid gap-1">
              {#each visibleGroups as group (group.id)}
                <button
                  type="button"
                  class="flex min-w-0 items-center gap-2 rounded-sm px-1 py-1 text-left hover:bg-accent/60 disabled:pointer-events-none disabled:opacity-50"
                  disabled={mutating}
                  aria-label={t("notes.databaseListHideGroup", group.name)}
                  onclick={() => setGroupHidden(group.id, true)}
                >
                  <EyeOff class="size-3.5 shrink-0" aria-hidden="true" />
                  <span class="min-w-0 flex-1 truncate">{group.name}</span>
                </button>
              {/each}
              {#each hiddenGroups as group (group.id)}
                <button
                  type="button"
                  class="flex min-w-0 items-center gap-2 rounded-sm px-1 py-1 text-left hover:bg-accent/60 disabled:pointer-events-none disabled:opacity-50"
                  disabled={mutating}
                  aria-label={t("notes.databaseListShowGroup", group.name)}
                  onclick={() => setGroupHidden(group.id, false)}
                >
                  <Eye class="size-3.5 shrink-0" aria-hidden="true" />
                  <span class="min-w-0 flex-1 truncate">{group.name}</span>
                </button>
              {/each}
            </div>
          </details>
        {/if}

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

      <div class="grid min-w-0 gap-3">
        {#each visibleGroups as group (group.id)}
          <section class="min-w-0">
            {#if groupColumn}
              <div class="mb-1 flex min-w-0 items-center gap-2 border-b border-border pb-1">
                <span class="inline-flex size-2 rounded-full bg-muted-foreground" aria-hidden="true"></span>
                <h3 class="min-w-0 flex-1 truncate text-[0.8rem] font-medium text-muted-foreground">
                  {group.name}
                </h3>
                <span class="text-[0.733333rem] text-muted-foreground">
                  {t("notes.databaseListRowsCount", group.rows.length)}
                </span>
              </div>
            {/if}

            <div class="grid min-w-0 divide-y divide-border rounded-md border border-border">
              {#each group.rows as row (row.id)}
                {@const title = rowTitle(row)}
                <article class="flex min-w-0 items-center gap-2 bg-background px-2 py-1.5">
                  <button
                    type="button"
                    class="flex min-w-0 flex-1 items-center gap-2 rounded-sm text-left outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
                    onclick={() => openRow(row)}
                  >
                    <FileText class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                    <span class="min-w-32 flex-1 truncate text-[0.866667rem] font-medium text-foreground">
                      {title}
                    </span>
                  </button>
                  {#if visibleColumns.length > 0}
                    <dl class="hidden min-w-0 flex-1 items-center justify-end gap-2 text-[0.8rem] text-muted-foreground @lg:flex">
                      {#each visibleColumns as column (column.id)}
                        {@const text = notesDatabaseListRowText(row, column)}
                        <div class="min-w-0 max-w-40 truncate">
                          <dt class="sr-only">{column.name}</dt>
                          <dd class="truncate">{text || t("notes.databaseTableEmptyCell")}</dd>
                        </div>
                      {/each}
                    </dl>
                  {/if}
                  <div class="flex shrink-0 items-center gap-1">
                    <button
                      type="button"
                      class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                      aria-label={t("notes.databaseRowsOpen", title)}
                      title={t("notes.databaseRowsOpen", title)}
                      onclick={() => openRow(row)}
                    >
                      <ExternalLink class="size-3.5" aria-hidden="true" />
                    </button>
                    <button
                      type="button"
                      class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:pointer-events-none disabled:opacity-50"
                      disabled={mutating}
                      aria-label={t("notes.databaseRowsDuplicate", title)}
                      title={t("notes.databaseRowsDuplicate", title)}
                      onclick={() => {
                        void duplicateRow(row);
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
                        void trashRow(row);
                      }}
                    >
                      <Trash2 class="size-3.5" aria-hidden="true" />
                    </button>
                  </div>
                </article>
              {/each}
              {#if group.rows.length === 0}
                <p class="px-2 py-2 text-[0.8rem] text-muted-foreground">{t("notes.databaseRowsEmpty")}</p>
              {/if}
            </div>
          </section>
        {/each}
      </div>

      {#if visibleColumns.length === 0}
        <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseListNoVisibleProperties")}</p>
      {/if}

      {#if list.rows.length === 0 && !loading && !error}
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
                    {notesDatabaseListRowText(selectedPanelRow, column) || t("notes.databaseTableEmptyCell")}
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
      <span>{t("notes.databaseListSaving")}</span>
    </p>
  {/if}
</section>
