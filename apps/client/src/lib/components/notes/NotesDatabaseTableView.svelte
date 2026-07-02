<script lang="ts">
  import { tick } from "svelte";
  import {
    applyNotesDataSourceTemplate,
    clickNotesDataSourceButton,
    createNotesDataSourceRowPage,
    createNotesDataSourceTemplateFromRow,
    deleteNotesDataSourceTemplate,
    duplicateNotesPage,
    getNotesDataSourceTableView,
    listNotesDataSourceTemplates,
    trashNotesPage,
    updateNotesDataSourceRowProperty,
    updateNotesDataSourceTableView,
  } from "$lib/api/notes";
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    notesDatabaseTableEditValuesEqual,
    notesDatabaseTableCellEditValue,
    notesDatabaseTableCellText,
    notesDatabaseTableColumnCanEdit,
    notesDatabaseTableColumns,
    notesDatabaseTableColumnWidth,
    notesDatabaseTableConfigurationFromView,
    notesDatabaseTableFiltersFromView,
    notesDatabaseTableSortsFromView,
    notesDatabaseTableUpdate,
    notesDatabaseTableVisibleColumns,
    type NotesDatabaseTableEditValue,
    type NotesDatabaseTableColumn,
  } from "$lib/notes/database-table";
  import NotesDatabaseRelationCell from "./NotesDatabaseRelationCell.svelte";
  import type {
    NotesDatabaseTableFilter,
    NotesDatabaseTableFilterCondition,
    NotesDatabaseTableRowOpenMode,
    NotesDatabaseTableSort,
    NotesDatabaseViewScope,
    NotesDataSourceTemplate,
    NotesDataSourceTableView,
    NotesPage,
  } from "$lib/notes/types";
  import Check from "@lucide/svelte/icons/check";
  import ChevronsLeftRight from "@lucide/svelte/icons/chevrons-left-right";
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Eye from "@lucide/svelte/icons/eye";
  import EyeOff from "@lucide/svelte/icons/eye-off";
  import FileText from "@lucide/svelte/icons/file-text";
  import Minus from "@lucide/svelte/icons/minus";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let {
    dataSourceId,
    databaseId = null,
    viewId = null,
    onSelectPage,
    reloadKey = 0,
  }: {
    dataSourceId: string;
    databaseId?: string | null;
    viewId?: string | null;
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

  let tableRoot: HTMLDivElement | null = $state(null);
  let table = $state<NotesDataSourceTableView | null>(null);
  let templates = $state<NotesDataSourceTemplate[]>([]);
  let loading = $state(false);
  let mutating = $state(false);
  let error = $state<string | null>(null);
  let draftTitle = $state("");
  let templateName = $state("");
  let templateSourceRowId = $state("");
  let selectedTemplateId = $state("");
  let createTemplateAsDefault = $state(false);
  let selectedPanelRowId = $state<string | null>(null);
  let lastLoadSignature = $state("");
  let pendingFocusRowId = $state<string | null>(null);

  const columns = $derived(table ? notesDatabaseTableColumns(table.data_source, table.view) : []);
  const visibleColumns = $derived(notesDatabaseTableVisibleColumns(columns));
  const filters = $derived(table ? notesDatabaseTableFiltersFromView(table.view) : []);
  const sorts = $derived(table ? notesDatabaseTableSortsFromView(table.view) : []);
  const rowOpenMode = $derived(
    table ? notesDatabaseTableConfigurationFromView(table.view).row_open_mode : "full_page",
  );
  const selectedPanelRow = $derived(
    table?.rows.find((row) => row.id === selectedPanelRowId) ?? null,
  );
  const selectedTemplate = $derived(
    templates.find((template) => template.id === selectedTemplateId) ?? null,
  );

  $effect(() => {
    const signature = `${dataSourceId}:${databaseId ?? ""}:${viewId ?? ""}:${reloadKey}`;
    if (signature === lastLoadSignature) return;
    lastLoadSignature = signature;
    void loadTable();
  });

  function viewScope(): NotesDatabaseViewScope {
    return { databaseId, viewId };
  }

  async function loadTable(): Promise<NotesDataSourceTableView | null> {
    loading = true;
    error = null;
    try {
      const [loaded, loadedTemplates] = await Promise.all([
        getNotesDataSourceTableView(dataSourceId, viewScope()),
        listNotesDataSourceTemplates(dataSourceId),
      ]);
      table = loaded;
      templates = loadedTemplates;
      if (selectedPanelRowId && !loaded.rows.some((row) => row.id === selectedPanelRowId)) {
        selectedPanelRowId = null;
      }
      if (templateSourceRowId && !loaded.rows.some((row) => row.id === templateSourceRowId)) {
        templateSourceRowId = "";
      }
      if (selectedTemplateId && !loadedTemplates.some((template) => template.id === selectedTemplateId)) {
        selectedTemplateId = "";
      }
      if (!selectedTemplateId) {
        selectedTemplateId = loadedTemplates.find((template) => template.is_default)?.id ?? "";
      }
      await focusPendingRow(loaded);
      return loaded;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
      return null;
    } finally {
      loading = false;
    }
  }

  async function focusPendingRow(loaded: NotesDataSourceTableView): Promise<void> {
    const rowId = pendingFocusRowId;
    pendingFocusRowId = null;
    if (!rowId) return;
    const rowIndex = loaded.rows.findIndex((row) => row.id === rowId);
    if (rowIndex < 0) return;
    await tick();
    focusCell(rowIndex, 0);
  }

  async function persistTable(
    nextColumns: NotesDatabaseTableColumn[],
    nextRowOpenMode: NotesDatabaseTableRowOpenMode,
    nextFilters: NotesDatabaseTableFilter[],
    nextSorts: NotesDatabaseTableSort[],
  ): Promise<void> {
    mutating = true;
    error = null;
    try {
      table = await updateNotesDataSourceTableView(
        dataSourceId,
        notesDatabaseTableUpdate(nextColumns, nextRowOpenMode, nextFilters, nextSorts),
        viewScope(),
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
      const loaded = selectedTemplateId
        ? await applyNotesDataSourceTemplate(dataSourceId, selectedTemplateId, { title })
        : await createNotesDataSourceRowPage(dataSourceId, {
            id: crypto.randomUUID(),
            first_block_id: crypto.randomUUID(),
            title,
          });
      draftTitle = "";
      pendingFocusRowId = loaded.page.id;
      await loadTable();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function createTemplateFromRow(): Promise<void> {
    const sourceRowId = templateSourceRowId || table?.rows[0]?.id || "";
    const name = templateName.trim();
    if (!sourceRowId || !name) return;
    mutating = true;
    error = null;
    try {
      const template = await createNotesDataSourceTemplateFromRow(dataSourceId, {
        id: crypto.randomUUID(),
        source_page_id: sourceRowId,
        name,
        is_default: createTemplateAsDefault,
      });
      templateName = "";
      templateSourceRowId = "";
      createTemplateAsDefault = false;
      selectedTemplateId = template.id;
      templates = await listNotesDataSourceTemplates(dataSourceId);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function deleteSelectedTemplate(): Promise<void> {
    const templateId = selectedTemplateId;
    if (!templateId) return;
    mutating = true;
    error = null;
    try {
      await deleteNotesDataSourceTemplate(dataSourceId, templateId);
      selectedTemplateId = "";
      templates = await listNotesDataSourceTemplates(dataSourceId);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function runButton(row: NotesPage, column: NotesDatabaseTableColumn): Promise<void> {
    const title = rowTitle(row);
    const confirmed = column.buttonRequiresConfirmation
      ? window.confirm(t("notes.databaseButtonConfirm", column.name, title))
      : false;
    if (column.buttonRequiresConfirmation && !confirmed) return;
    mutating = true;
    error = null;
    try {
      pendingFocusRowId = row.id;
      await clickNotesDataSourceButton(dataSourceId, row.id, {
        property_id: column.id,
        confirmed,
      });
      await loadTable();
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
      pendingFocusRowId = loaded.page.id;
      await loadTable();
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
      await loadTable();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  async function saveCell(
    row: NotesPage,
    column: NotesDatabaseTableColumn,
    value: NotesDatabaseTableEditValue,
  ): Promise<void> {
    const current = notesDatabaseTableCellEditValue(row, column);
    if (notesDatabaseTableEditValuesEqual(current, value)) return;
    mutating = true;
    error = null;
    try {
      pendingFocusRowId = row.id;
      await updateNotesDataSourceRowProperty(dataSourceId, row.id, {
        property_id: column.id,
        value,
      });
      await loadTable();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      mutating = false;
    }
  }

  function updateColumnVisibility(columnId: string, hidden: boolean): void {
    const nextColumns = columns.map((column) =>
      column.id === columnId ? { ...column, hidden } : { ...column },
    );
    void persistTable(nextColumns, rowOpenMode, filters, sorts);
  }

  function updateColumnWidth(columnId: string, delta: number): void {
    const nextColumns = columns.map((column) =>
      column.id === columnId
        ? { ...column, width: notesDatabaseTableColumnWidth(column, delta) }
        : { ...column },
    );
    void persistTable(nextColumns, rowOpenMode, filters, sorts);
  }

  function updateRowOpenMode(mode: NotesDatabaseTableRowOpenMode): void {
    void persistTable(columns.map((column) => ({ ...column })), mode, filters, sorts);
  }

  function addSort(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    const nextSorts = [
      ...sorts,
      { property_id: firstColumn.id, direction: "ascending" as const },
    ];
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, filters, nextSorts);
  }

  function updateSort(index: number, patch: Partial<NotesDatabaseTableSort>): void {
    const nextSorts = sorts.map((sort, sortIndex) =>
      sortIndex === index ? { ...sort, ...patch } : sort,
    );
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, filters, nextSorts);
  }

  function removeSort(index: number): void {
    const nextSorts = sorts.filter((_, sortIndex) => sortIndex !== index);
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, filters, nextSorts);
  }

  function addFilter(): void {
    const firstColumn = columns[0];
    if (!firstColumn) return;
    const nextFilters = [
      ...filters,
      { property_id: firstColumn.id, condition: "contains" as const, value: "" },
    ];
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, nextFilters, sorts);
  }

  function updateFilter(index: number, patch: Partial<NotesDatabaseTableFilter>): void {
    const nextFilters = filters.map((filter, filterIndex) => {
      if (filterIndex !== index) return filter;
      const next = { ...filter, ...patch };
      if (!filterConditionNeedsValue(next.condition)) next.value = null;
      return next;
    });
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, nextFilters, sorts);
  }

  function removeFilter(index: number): void {
    const nextFilters = filters.filter((_, filterIndex) => filterIndex !== index);
    void persistTable(columns.map((column) => ({ ...column })), rowOpenMode, nextFilters, sorts);
  }

  function openRow(row: NotesPage): void {
    if (rowOpenMode === "side_panel") {
      selectedPanelRowId = row.id;
      return;
    }
    onSelectPage(row.id);
  }

  function rowTitle(row: NotesPage): string {
    const titleColumn = columns.find((column) => column.type === "title");
    const title = titleColumn ? notesDatabaseTableCellText(row, titleColumn).trim() : "";
    return title || t("notes.untitled");
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

  function rowOpenModeLabel(mode: NotesDatabaseTableRowOpenMode): string {
    return mode === "side_panel"
      ? t("notes.databaseTableOpenSidePanel")
      : t("notes.databaseTableOpenFullPage");
  }

  function handleCellKeydown(
    event: KeyboardEvent,
    rowIndex: number,
    columnIndex: number,
  ): void {
    event.stopPropagation();
    if (shouldKeepInputArrow(event)) return;
    if (event.currentTarget instanceof HTMLSelectElement) return;
    const maxRowIndex = Math.max(0, (table?.rows.length ?? 1) - 1);
    const maxColumnIndex = Math.max(0, visibleColumns.length - 1);
    let nextRowIndex = rowIndex;
    let nextColumnIndex = columnIndex;
    if (event.key === "ArrowRight") nextColumnIndex = Math.min(maxColumnIndex, columnIndex + 1);
    else if (event.key === "ArrowLeft") nextColumnIndex = Math.max(0, columnIndex - 1);
    else if (event.key === "ArrowDown") nextRowIndex = Math.min(maxRowIndex, rowIndex + 1);
    else if (event.key === "ArrowUp") nextRowIndex = Math.max(0, rowIndex - 1);
    else if (event.key === "Enter") nextRowIndex = Math.min(maxRowIndex, rowIndex + 1);
    else return;
    event.preventDefault();
    focusCell(nextRowIndex, nextColumnIndex);
  }

  function shouldKeepInputArrow(event: KeyboardEvent): boolean {
    const target = event.currentTarget;
    if (!(target instanceof HTMLInputElement) || target.type === "checkbox") return false;
    const start = target.selectionStart ?? 0;
    const end = target.selectionEnd ?? start;
    if (event.key === "ArrowLeft") return start > 0 || end > 0;
    if (event.key === "ArrowRight") return start < target.value.length || end < target.value.length;
    return false;
  }

  function focusCell(rowIndex: number, columnIndex: number): void {
    const selector = `[data-table-cell="true"][data-row-index="${rowIndex}"][data-column-index="${columnIndex}"]`;
    const target = tableRoot?.querySelector(selector);
    if (target instanceof HTMLElement) target.focus();
  }
</script>

<section class="space-y-3 border-t border-border pt-3" aria-label={t("notes.databaseTableTitle")}>
  <div class="flex min-w-0 flex-wrap items-center gap-2 text-[0.8rem] text-muted-foreground">
    <span class="min-w-0 flex-1 truncate" role="status">
      {#if loading}
        {t("notes.databaseTableLoading")}
      {:else if error}
        {t("notes.databaseTableFailed", error)}
      {:else}
        {t("notes.databaseRowsCount", table?.rows.length ?? 0)}
      {/if}
    </span>
    <label class="inline-flex min-w-0 items-center gap-1">
      <span>{t("notes.databaseTableOpenMode")}</span>
      <select
        class="h-8 min-w-32 rounded-md border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
        value={rowOpenMode}
        disabled={loading || mutating || !table}
        onchange={(event) => updateRowOpenMode(event.currentTarget.value as NotesDatabaseTableRowOpenMode)}
        onkeydown={(event) => event.stopPropagation()}
      >
        <option value="full_page">{rowOpenModeLabel("full_page")}</option>
        <option value="side_panel">{rowOpenModeLabel("side_panel")}</option>
      </select>
    </label>
    <button
      type="button"
      class="inline-flex size-8 items-center justify-center rounded-md hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
      disabled={loading || mutating}
      aria-label={t("notes.databaseTableReload")}
      title={t("notes.databaseTableReload")}
      onclick={() => {
        void loadTable();
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
      <span>{selectedTemplate ? t("notes.databaseRowsAddFromTemplate", selectedTemplate.name) : t("notes.databaseRowsAdd")}</span>
    </button>
  </div>

  {#if table}
    <div class="grid gap-2 @container">
      <details class="rounded-md border border-border p-2 text-[0.8rem]">
        <summary class="cursor-pointer text-foreground">{t("notes.databaseTemplatesTitle")}</summary>
        <div class="mt-2 grid min-w-0 gap-2 @lg:grid-cols-[minmax(8rem,1fr)_minmax(8rem,1fr)_auto]">
          <label class="min-w-0 text-muted-foreground">
            <span class="mb-1 block">{t("notes.databaseTemplatesUse")}</span>
            <select
              class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
              value={selectedTemplateId}
              disabled={mutating}
              onchange={(event) => {
                selectedTemplateId = event.currentTarget.value;
              }}
              onkeydown={(event) => event.stopPropagation()}
            >
              <option value="">{t("notes.databaseTemplatesNone")}</option>
              {#each templates as template (template.id)}
                <option value={template.id}>
                  {template.is_default ? t("notes.databaseTemplatesDefaultOption", template.name) : template.name}
                </option>
              {/each}
            </select>
          </label>
          <label class="min-w-0 text-muted-foreground">
            <span class="mb-1 block">{t("notes.databaseTemplatesName")}</span>
            <input
              class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
              value={templateName}
              placeholder={t("notes.databaseTemplatesNamePlaceholder")}
              disabled={mutating}
              oninput={(event) => {
                templateName = event.currentTarget.value;
              }}
              onkeydown={(event) => event.stopPropagation()}
            />
          </label>
          <label class="min-w-0 text-muted-foreground">
            <span class="mb-1 block">{t("notes.databaseTemplatesSourceRow")}</span>
            <select
              class="h-8 w-full min-w-0 rounded-md border border-input bg-background px-2 text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
              value={templateSourceRowId}
              disabled={mutating || table.rows.length === 0}
              onchange={(event) => {
                templateSourceRowId = event.currentTarget.value;
              }}
              onkeydown={(event) => event.stopPropagation()}
            >
              <option value="">{t("notes.databaseTemplatesFirstRow")}</option>
              {#each table.rows as row (row.id)}
                <option value={row.id}>{rowTitle(row)}</option>
              {/each}
            </select>
          </label>
          <label class="flex min-w-0 items-center gap-2 text-muted-foreground">
            <input
              type="checkbox"
              checked={createTemplateAsDefault}
              disabled={mutating}
              onchange={(event) => {
                createTemplateAsDefault = event.currentTarget.checked;
              }}
              onkeydown={(event) => event.stopPropagation()}
            />
            <span>{t("notes.databaseTemplatesMakeDefault")}</span>
          </label>
          <div class="flex min-w-0 flex-wrap items-center gap-1 @lg:col-span-2">
            <button
              type="button"
              class="inline-flex h-8 items-center gap-1 rounded-md px-2 hover:bg-accent disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating || !templateName.trim() || table.rows.length === 0}
              onclick={() => {
                void createTemplateFromRow();
              }}
            >
              <Plus class="size-3.5" aria-hidden="true" />
              <span>{t("notes.databaseTemplatesCreate")}</span>
            </button>
            <button
              type="button"
              class="inline-flex h-8 items-center gap-1 rounded-md text-destructive hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
              disabled={mutating || !selectedTemplateId}
              onclick={() => {
                void deleteSelectedTemplate();
              }}
            >
              <Trash2 class="size-3.5" aria-hidden="true" />
              <span>{t("notes.databaseTemplatesDelete")}</span>
            </button>
          </div>
        </div>
      </details>

      <div class="grid gap-2 @lg:grid-cols-3">
        <details class="rounded-md border border-border p-2 text-[0.8rem]">
          <summary class="cursor-pointer text-foreground">{t("notes.databaseTableColumns")}</summary>
          <div class="mt-2 grid gap-1">
            {#each columns as column (column.id)}
              <label class="flex min-w-0 items-center gap-2 rounded-sm px-1 py-0.5 hover:bg-accent/60">
                <input
                  type="checkbox"
                  checked={!column.hidden}
                  disabled={mutating || column.type === "title"}
                  onchange={(event) => updateColumnVisibility(column.id, !event.currentTarget.checked)}
                  onkeydown={(event) => event.stopPropagation()}
                />
                <span class="min-w-0 flex-1 truncate">{column.name}</span>
                {#if column.hidden}
                  <EyeOff class="size-3.5 text-muted-foreground" aria-hidden="true" />
                {:else}
                  <Eye class="size-3.5 text-muted-foreground" aria-hidden="true" />
                {/if}
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

      <div bind:this={tableRoot} class="min-w-0 overflow-x-auto rounded-md border border-border">
        <table class="min-w-full border-collapse text-[0.866667rem]">
          <thead class="bg-muted/50 text-left text-[0.733333rem] text-muted-foreground">
            <tr>
              <th class="w-10 border-b border-border px-1 py-1 font-medium">
                <span class="sr-only">{t("notes.databaseTableRowActions")}</span>
              </th>
              {#each visibleColumns as column (column.id)}
                <th
                  class="border-b border-l border-border px-2 py-1 font-medium"
                  style={`width: ${column.width}px; min-width: ${column.width}px;`}
                >
                  <div class="flex min-w-0 items-center gap-1">
                    <span class="min-w-0 flex-1 truncate">{column.name}</span>
                    <ChevronsLeftRight class="size-3.5 shrink-0" aria-hidden="true" />
                    <button
                      type="button"
                      class="inline-flex size-6 items-center justify-center rounded-sm hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                      disabled={mutating}
                      aria-label={t("notes.databaseTableNarrowColumn", column.name)}
                      title={t("notes.databaseTableNarrowColumn", column.name)}
                      onclick={() => updateColumnWidth(column.id, -32)}
                    >
                      <Minus class="size-3" aria-hidden="true" />
                    </button>
                    <button
                      type="button"
                      class="inline-flex size-6 items-center justify-center rounded-sm hover:bg-accent disabled:pointer-events-none disabled:opacity-40"
                      disabled={mutating}
                      aria-label={t("notes.databaseTableWidenColumn", column.name)}
                      title={t("notes.databaseTableWidenColumn", column.name)}
                      onclick={() => updateColumnWidth(column.id, 32)}
                    >
                      <Plus class="size-3" aria-hidden="true" />
                    </button>
                  </div>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each table.rows as row, rowIndex (row.id)}
              {@const title = rowTitle(row)}
              <tr class="border-b border-border last:border-b-0">
                <td class="w-10 px-1 py-1 align-middle">
                  <div class="flex items-center gap-0.5">
                    <button
                      type="button"
                      class="inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
                      aria-label={t("notes.databaseRowsOpen", title)}
                      title={t("notes.databaseRowsOpen", title)}
                      onclick={() => openRow(row)}
                    >
                      <FileText class="size-3.5" aria-hidden="true" />
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
                </td>
                {#each visibleColumns as column, columnIndex (column.id)}
                  {@const editValue = notesDatabaseTableCellEditValue(row, column)}
                  <td
                    class="border-l border-border px-1 py-1 align-middle"
                    style={`width: ${column.width}px; min-width: ${column.width}px;`}
                  >
                    {#if column.type === "checkbox"}
                      <label class="flex h-8 items-center justify-center">
                        <input
                          data-table-cell="true"
                          data-row-index={rowIndex}
                          data-column-index={columnIndex}
                          type="checkbox"
                          checked={editValue === true}
                          disabled={mutating || !notesDatabaseTableColumnCanEdit(column)}
                          aria-label={column.name}
                          onchange={(event) => {
                            void saveCell(row, column, event.currentTarget.checked);
                          }}
                          onkeydown={(event) => handleCellKeydown(event, rowIndex, columnIndex)}
                        />
                      </label>
                    {:else if column.type === "select" || column.type === "status"}
                      <select
                        data-table-cell="true"
                        data-row-index={rowIndex}
                        data-column-index={columnIndex}
                        class="h-8 w-full min-w-0 rounded-sm border border-transparent bg-transparent px-1 text-foreground outline-none hover:border-input focus:border-input focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
                        value={String(editValue)}
                        disabled={mutating || !notesDatabaseTableColumnCanEdit(column)}
                        aria-label={column.name}
                        onchange={(event) => {
                          void saveCell(row, column, event.currentTarget.value || null);
                        }}
                        onkeydown={(event) => handleCellKeydown(event, rowIndex, columnIndex)}
                      >
                        <option value="">{t("notes.databaseTableEmptyCell")}</option>
                        {#each column.options as option (option.id)}
                          <option value={option.name}>{option.name}</option>
                        {/each}
                        </select>
                    {:else if column.type === "relation"}
                      <NotesDatabaseRelationCell
                        {row}
                        {column}
                        {rowIndex}
                        {columnIndex}
                        {mutating}
                        onSave={(value) => saveCell(row, column, value)}
                        onNavigate={handleCellKeydown}
                      />
                    {:else if column.type === "button"}
                      <button
                        data-table-cell="true"
                        data-row-index={rowIndex}
                        data-column-index={columnIndex}
                        type="button"
                        class="inline-flex h-8 w-full min-w-0 items-center justify-center gap-1 rounded-sm border border-input bg-background px-2 text-[0.8rem] text-foreground outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
                        disabled={mutating}
                        title={notesDatabaseTableCellText(row, column)}
                        onclick={() => {
                          void runButton(row, column);
                        }}
                        onkeydown={(event) => {
                          if (event.key === "Enter" || event.key === " ") return;
                          handleCellKeydown(event, rowIndex, columnIndex);
                        }}
                      >
                        <Check class="size-3.5 shrink-0" aria-hidden="true" />
                        <span class="min-w-0 truncate">
                          {notesDatabaseTableCellText(row, column) || column.buttonLabel || column.name}
                        </span>
                      </button>
                    {:else if notesDatabaseTableColumnCanEdit(column)}
                      <input
                        data-table-cell="true"
                        data-row-index={rowIndex}
                        data-column-index={columnIndex}
                        class="h-8 w-full min-w-0 rounded-sm border border-transparent bg-transparent px-1 text-foreground outline-none hover:border-input focus:border-input focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-60"
                        value={String(editValue)}
                        inputmode={column.type === "number" ? "decimal" : "text"}
                        aria-label={column.name}
                        disabled={mutating}
                        onblur={(event) => {
                          void saveCell(row, column, event.currentTarget.value);
                        }}
                        onkeydown={(event) => handleCellKeydown(event, rowIndex, columnIndex)}
                      />
                    {:else}
                      <button
                        data-table-cell="true"
                        data-row-index={rowIndex}
                        data-column-index={columnIndex}
                        type="button"
                        class="flex h-8 w-full min-w-0 items-center rounded-sm px-1 text-left text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        title={t("notes.databaseTableReadonly")}
                        onkeydown={(event) => handleCellKeydown(event, rowIndex, columnIndex)}
                      >
                        <span class="min-w-0 truncate">
                          {notesDatabaseTableCellText(row, column) || t("notes.databaseTableEmptyCell")}
                        </span>
                      </button>
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      {#if table.rows.length === 0 && !loading && !error}
        <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseRowsEmpty")}</p>
      {/if}

      {#if visibleColumns.length === 0}
        <p class="text-[0.8rem] text-muted-foreground">{t("notes.databaseTableNoVisibleColumns")}</p>
      {/if}

      {#if rowOpenMode === "side_panel"}
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
                    {notesDatabaseTableCellText(selectedPanelRow, column) || t("notes.databaseTableEmptyCell")}
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
      <span>{t("notes.databaseTableSaving")}</span>
    </p>
  {/if}
</section>
