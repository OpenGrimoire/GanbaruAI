/**
 * Moves one playlist to a bounded position without mutating the source array.
 *
 * @param playlists Current ordered playlists.
 * @param playlistId Playlist to move.
 * @param targetIndex Requested zero-based destination.
 * @returns The reordered array, or the original values when the id is absent.
 */
export function moveMusicPlaylistOrder<T extends { id: string }>(
  playlists: readonly T[],
  playlistId: string,
  targetIndex: number,
): T[] {
  const sourceIndex = playlists.findIndex((playlist) => playlist.id === playlistId);
  if (sourceIndex < 0) return [...playlists];
  const boundedTarget = Math.max(0, Math.min(Math.trunc(targetIndex), playlists.length - 1));
  if (sourceIndex === boundedTarget) return [...playlists];
  const reordered = [...playlists];
  const [moved] = reordered.splice(sourceIndex, 1);
  if (!moved) return [...playlists];
  reordered.splice(boundedTarget, 0, moved);
  return reordered;
}
