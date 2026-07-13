import type { NotesDataSource, NotesPage } from "$lib/notes/types";
import {
  loadNotesDatabaseMentionData,
  type NotesDatabaseMentionData,
} from "./notes-block-mention-targets";

export type NotesMentionDataLoader = () => Promise<NotesDatabaseMentionData>;

/** Own mention data loading and reject stale aggregate results. */
export function createNotesMentionDataController(
  load: NotesMentionDataLoader = loadNotesDatabaseMentionData,
) {
  let dataSources = $state<NotesDataSource[]>([]);
  let rowPages = $state<NotesPage[]>([]);
  let requestId = 0;

  async function reload(): Promise<void> {
    const currentRequestId = ++requestId;
    try {
      const result = await load();
      if (currentRequestId !== requestId) return;
      dataSources = result.dataSources;
      rowPages = result.rowPages;
    } catch (error) {
      if (currentRequestId !== requestId) return;
      console.warn("notes mention data source targets failed", error);
      dataSources = [];
      rowPages = [];
    }
  }

  return {
    get dataSources() { return dataSources; },
    get rowPages() { return rowPages; },
    reload,
  };
}
