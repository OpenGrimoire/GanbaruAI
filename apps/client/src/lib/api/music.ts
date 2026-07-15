import { invoke } from "@tauri-apps/api/core";
import { dbUrl } from "$lib/api/db";
import type { PlaybackStatus } from "$lib/music/playback";
import type { MusicSourceKind } from "$lib/music/sources";

export interface PlaybackStateRow {
  sourceIdentity: string;
  sourceKind: MusicSourceKind;
  positionMs: number;
  durationMs: number | null;
  status: PlaybackStatus;
  updatedAt: number;
}

export interface MediaFolderTrack {
  path: string;
  title: string;
  artworkPath: string | null;
}

export interface MediaFolderSelection {
  folderPath: string;
  tracks: MediaFolderTrack[];
  truncated: boolean;
}

export async function getPlaybackState(sourceIdentity: string): Promise<PlaybackStateRow | null> {
  return invoke("music_get_playback_state", {
    dbUrl: dbUrl(),
    sourceIdentity,
  });
}

export async function pickMediaFolder(): Promise<MediaFolderSelection | null> {
  return invoke("music_pick_media_folder");
}

export async function pickMediaFile(): Promise<string | null> {
  return invoke("music_pick_media_file");
}

export async function registerMediaFile(path: string, generation: number): Promise<string> {
  return invoke("music_register_media_file", { path, generation });
}

export async function registerEmbeddedArtwork(
  path: string,
  generation: number,
): Promise<string | null> {
  return invoke("music_register_embedded_artwork", { path, generation });
}

/** Retains only the hosted media URLs referenced by the current playback lifecycle. */
export async function retainHostedMedia(mediaUrls: string[], generation: number): Promise<void> {
  await invoke("music_retain_hosted_media", { mediaUrls, generation });
}

/** Removes registrations abandoned by one stale playback lifecycle. */
export async function unregisterHostedMedia(
  mediaUrls: string[],
  generation: number,
): Promise<void> {
  await invoke("music_unregister_hosted_media", { mediaUrls, generation });
}

export async function revealLocalFile(path: string): Promise<void> {
  await invoke("music_reveal_local_file", { path });
}

export async function getYouTubeHostUrl(): Promise<string> {
  return invoke("music_youtube_host_url");
}

export async function savePlaybackState(state: PlaybackStateRow): Promise<void> {
  await invoke("music_save_playback_state", {
    dbUrl: dbUrl(),
    state,
  });
}
