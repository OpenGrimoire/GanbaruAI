import type {
  MusicAssignmentOwnerKind,
  MusicContextAssignment,
} from "$lib/music/music-context-assignment";
import type { MusicPlaylistSummary } from "$lib/music/library-contracts";

/** Mobile placeholder until the native music service is implemented. */
export async function getMusicContextAssignments(
  _ownerKind: MusicAssignmentOwnerKind,
  _ownerId: string,
): Promise<MusicContextAssignment[]> {
  return [];
}

/** Mobile placeholder until the native music service is implemented. */
export async function getMusicPlaylistSummaries(
  _nowMs: number,
  _offset: number,
  _limit: number,
): Promise<MusicPlaylistSummary[]> {
  return [];
}
