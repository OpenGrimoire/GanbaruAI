export interface NotesFocusRequest {
  blockId: string | null;
  requestId: number;
}

/**
 * Return the next focus request token for a Notes block.
 */
export function nextNotesFocusRequest(
  current: NotesFocusRequest,
  blockId: string | null,
): NotesFocusRequest {
  return {
    blockId,
    requestId: current.requestId + 1,
  };
}
