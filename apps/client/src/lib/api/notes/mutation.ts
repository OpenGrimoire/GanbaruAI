import { invoke } from "@tauri-apps/api/core";
import {
  applyNotesProjectHistoryMutationDeadline,
  notifyNotesProjectHistoryMutation,
} from "$lib/notes/project-history-scheduler";

/** Invoke a Notes mutation and apply its project-history checkpoint envelope. */
export async function invokeNotesMutation(
  command: string,
  args: Record<string, unknown>,
  forceCheckpoint = false,
): Promise<unknown> {
  const result = await invoke<unknown>(command, args);
  if (typeof result === "object" && result !== null && !Array.isArray(result)) {
    const record = result as Record<string, unknown>;
    if (Object.hasOwn(record, "value") && Object.hasOwn(record, "nextHistoryCheckpointAt")) {
      const deadline = record.nextHistoryCheckpointAt;
      if (deadline !== null && typeof deadline !== "string") {
        throw new Error(`${command} returned an invalid Notes history deadline`);
      }
      applyNotesProjectHistoryMutationDeadline(deadline as string | null);
      return record.value;
    }
  }
  notifyNotesProjectHistoryMutation(forceCheckpoint);
  return result;
}
