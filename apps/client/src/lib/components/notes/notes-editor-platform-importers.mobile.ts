function unsupportedMobileExport<T>(): Promise<T> {
  return Promise.reject(new Error("Notes file export is unavailable in the mobile composition"));
}

export function importNotesHtmlExportDialog(): Promise<
  typeof import("./NotesHtmlExportDialog.svelte")
> {
  return unsupportedMobileExport();
}

export function importNotesAgentBridgeExportDialog(): Promise<
  typeof import("./NotesAgentBridgeExportDialog.svelte")
> {
  return unsupportedMobileExport();
}

export function importNotesDatabaseCsvExportPanel(): Promise<
  typeof import("./NotesDatabaseCsvExportPanel.svelte")
> {
  return unsupportedMobileExport();
}
