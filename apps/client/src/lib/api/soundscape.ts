import { invoke } from "@tauri-apps/api/core";
import { dbUrl } from "$lib/api/db";
import {
  parseMusicSoundscape,
  parseMusicSoundscapes,
  parseMusicSoundscapeSnapshot,
  parseMusicSoundscapeState,
  type MusicSoundscapeDefinition,
  type MusicSoundscapeSnapshot,
  type MusicSoundscapeStartRequest,
  type MusicSoundscapeState,
  type MusicSoundscapeStateWrite,
  type MusicSoundscapeWrite,
} from "$lib/music/soundscape-contracts";

export const getDeviceId = (): Promise<string> => invoke<string>("vault_device_id");
export const getMusicSoundscapes = async (deviceId: string): Promise<MusicSoundscapeDefinition[]> => parseMusicSoundscapes(await invoke<unknown>("music_library_soundscapes", { dbUrl: dbUrl(), deviceId }));
export const upsertMusicSoundscape = async (request: MusicSoundscapeWrite): Promise<MusicSoundscapeDefinition> => parseMusicSoundscape(await invoke<unknown>("music_library_upsert_soundscape", { dbUrl: dbUrl(), request }));
export const removeMusicSoundscape = (soundscapeId: string, expectedVersion: number): Promise<void> => invoke("music_library_remove_soundscape", { dbUrl: dbUrl(), soundscapeId, expectedVersion });
export const getMusicSoundscapeState = async (): Promise<MusicSoundscapeState> => parseMusicSoundscapeState(await invoke<unknown>("music_library_soundscape_state", { dbUrl: dbUrl() }));
export const updateMusicSoundscapeState = async (request: MusicSoundscapeStateWrite): Promise<MusicSoundscapeState> => parseMusicSoundscapeState(await invoke<unknown>("music_library_update_soundscape_state", { dbUrl: dbUrl(), request }));
export const startSoundscape = async (request: MusicSoundscapeStartRequest): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_start", { request }));
export const pauseSoundscape = async (): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_pause"));
export const resumeSoundscape = async (): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_resume"));
export const stopSoundscape = async (): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_stop"));
export const setSoundscapeVolume = async (volume: number): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_set_volume", { volume }));
export const recoverSoundscape = async (): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_recover"));
export const getSoundscapeSnapshot = async (): Promise<MusicSoundscapeSnapshot> => parseMusicSoundscapeSnapshot(await invoke<unknown>("soundscape_snapshot"));
