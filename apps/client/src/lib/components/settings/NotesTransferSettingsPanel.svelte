<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import type {
    NotesHtmlImportRequest,
    NotesJsonGraphExportRequest,
    NotesNotionApiImportRequest,
    NotesNotionExportImportRequest,
  } from "$lib/notes/types";
  import { getNotes } from "$lib/stores/notes.svelte";
  import NotesHtmlImportDialog from "$lib/components/notes/NotesHtmlImportDialog.svelte";
  import NotesJsonGraphExportDialog from "$lib/components/notes/NotesJsonGraphExportDialog.svelte";
  import NotesNotionApiImportDialog from "$lib/components/notes/NotesNotionApiImportDialog.svelte";
  import NotesNotionExportImportDialog from "$lib/components/notes/NotesNotionExportImportDialog.svelte";
  import type { NotesTransferOperation } from "./types";

  let {
    operation,
    onCancel,
    compactLayout = false,
    iconRailLayout = false,
    onScrollContainerChange = () => {},
    onScrollbarInsetsChange = () => {},
  }: {
    operation: NotesTransferOperation;
    onCancel: () => void;
    compactLayout?: boolean;
    iconRailLayout?: boolean;
    onScrollContainerChange?: (scrollContainer: HTMLElement | undefined) => void;
    onScrollbarInsetsChange?: (insets: { top: number; bottom: number }) => void;
  } = $props();

  const notes = getNotes();
  const { t } = getLocalization();
  let panelRootEl: HTMLElement | undefined = $state();
  let panelScrollEl: HTMLElement | undefined = $state();
  const contentPaddingX = $derived(compactLayout ? "0.75rem" : iconRailLayout ? "1.25rem" : "2rem");
  const contentPaddingY = $derived(compactLayout ? "1rem" : iconRailLayout ? "1.25rem" : "2rem");

  $effect(() => {
    onScrollContainerChange(panelScrollEl);
    reportScrollbarInsets();

    const contentEl = panelRootEl?.closest<HTMLElement>("[data-settings-content]");
    if (!panelRootEl || !panelScrollEl || !contentEl) {
      return () => {
        onScrollContainerChange(undefined);
        onScrollbarInsetsChange({ top: 0, bottom: 0 });
      };
    }

    const observer = new ResizeObserver(reportScrollbarInsets);
    observer.observe(panelRootEl);
    observer.observe(panelScrollEl);
    observer.observe(contentEl);
    window.addEventListener("resize", reportScrollbarInsets);

    return () => {
      observer.disconnect();
      window.removeEventListener("resize", reportScrollbarInsets);
      onScrollContainerChange(undefined);
      onScrollbarInsetsChange({ top: 0, bottom: 0 });
    };
  });

  function reportScrollbarInsets(): void {
    const contentEl = panelRootEl?.closest<HTMLElement>("[data-settings-content]");
    if (!panelScrollEl || !contentEl) {
      onScrollbarInsetsChange({ top: 0, bottom: 0 });
      return;
    }
    const contentRect = contentEl.getBoundingClientRect();
    const scrollRect = panelScrollEl.getBoundingClientRect();
    onScrollbarInsetsChange({
      top: Math.max(0, scrollRect.top - contentRect.top),
      bottom: Math.max(0, contentRect.bottom - scrollRect.bottom),
    });
  }

  function operationDescription(): string {
    if (operation === "html-import") return t("settings.notesTransfers.htmlImportDescription");
    if (operation === "notion-api-import") return t("settings.notesTransfers.notionApiImportDescription");
    if (operation === "notion-export-import") return t("settings.notesTransfers.notionExportImportDescription");
    return t("settings.notesTransfers.jsonGraphExportDescription");
  }

  function importHtmlPage(input: {
    html: string;
    title: string | null;
    sourceName: string | null;
    keepExternalFileReferences: boolean;
    projectId: string | null;
  }) {
    const request: Omit<NotesHtmlImportRequest, "parent"> = {
      html: input.html,
      title: input.title,
      source_name: input.sourceName,
      keep_external_file_references: input.keepExternalFileReferences,
      project_id: input.projectId,
    };
    return notes.importHtmlPage(request);
  }

  function importNotionApi(input: {
    integrationToken: string;
    sourceWorkspaceId: string | null;
    pageIds: string[];
    dataSourceIds: string[];
    includeComments: boolean;
    includeUsers: boolean;
    keepExternalFileReferences: boolean;
    projectId: string | null;
  }) {
    const request: Omit<NotesNotionApiImportRequest, "parent"> = {
      integration_token: input.integrationToken,
      source_workspace_id: input.sourceWorkspaceId,
      page_ids: input.pageIds,
      data_source_ids: input.dataSourceIds,
      include_comments: input.includeComments,
      include_users: input.includeUsers,
      keep_external_file_references: input.keepExternalFileReferences,
      project_id: input.projectId,
    };
    return notes.importNotionApi(request);
  }

  function importNotionExportFolder(input: {
    exportRootPath: string;
    sourceWorkspaceId: string | null;
    keepExternalFileReferences: boolean;
    copyLocalFileReferences: boolean;
    importMarkdown: boolean;
    importHtml: boolean;
    importCsv: boolean;
    projectId: string | null;
  }) {
    const request: Omit<NotesNotionExportImportRequest, "parent"> = {
      export_root_path: input.exportRootPath,
      source_workspace_id: input.sourceWorkspaceId,
      keep_external_file_references: input.keepExternalFileReferences,
      copy_local_file_references: input.copyLocalFileReferences,
      import_markdown: input.importMarkdown,
      import_html: input.importHtml,
      import_csv: input.importCsv,
      project_id: input.projectId,
    };
    return notes.importNotionExportFolder(request);
  }

  function exportJsonGraph(input: {
    includeIndexes: boolean;
    includeHistory: boolean;
    includeTemplates: boolean;
    includeLocalState: boolean;
    pretty: boolean;
  }) {
    const request: NotesJsonGraphExportRequest = {
      include_indexes: input.includeIndexes,
      include_history: input.includeHistory,
      include_templates: input.includeTemplates,
      include_local_state: input.includeLocalState,
      pretty: input.pretty,
    };
    return notes.exportJsonGraph(request);
  }
</script>

<div bind:this={panelRootEl} class="flex h-full min-h-0 flex-col">
  <main
    bind:this={panelScrollEl}
    class="hide-scrollbar min-h-0 flex-1 overflow-y-auto"
    style="padding: {contentPaddingY} {contentPaddingX};"
  >
    {#if operation === "html-import"}
      <NotesHtmlImportDialog
        embedded
        description={operationDescription()}
        onImport={importHtmlPage}
        onCancel={onCancel}
      />
    {:else if operation === "notion-api-import"}
      <NotesNotionApiImportDialog
        embedded
        description={operationDescription()}
        onImport={importNotionApi}
        onCancel={onCancel}
      />
    {:else if operation === "notion-export-import"}
      <NotesNotionExportImportDialog
        embedded
        description={operationDescription()}
        onImport={importNotionExportFolder}
        onCancel={onCancel}
      />
    {:else}
      <NotesJsonGraphExportDialog
        embedded
        description={operationDescription()}
        onExport={exportJsonGraph}
        onCancel={onCancel}
      />
    {/if}
  </main>
</div>
