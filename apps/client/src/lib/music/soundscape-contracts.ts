export type MusicSoundscapeSourceKind = "generated-noise" | "local-loop" | "bundled-loop";
export type MusicGeneratedNoiseKind = "white" | "pink" | "brown";
export type MusicSoundscapeAvailability = "available" | "missing" | "unsupported";
export type MusicSoundscapeStatus = "idle" | "playing" | "paused" | "error";

export interface MusicSoundscapeDefinition {
  id: string;
  sourceKind: MusicSoundscapeSourceKind;
  generatedKind: MusicGeneratedNoiseKind | null;
  bundledIdentity: string | null;
  name: string;
  availability: MusicSoundscapeAvailability;
  localPath: string | null;
  createdAt: number;
  updatedAt: number;
  version: number;
}

export interface MusicSoundscapeWrite {
  id: string;
  sourceKind: MusicSoundscapeSourceKind;
  generatedKind: MusicGeneratedNoiseKind | null;
  bundledIdentity: string | null;
  name: string;
  deviceId: string;
  localPath: string | null;
  expectedVersion: number | null;
  updatedAt: number;
}

export interface MusicSoundscapeState {
  activeSoundscapeId: string | null;
  desiredPlaying: boolean;
  volume: number;
  updatedAt: number;
  version: number;
}

export interface MusicSoundscapeStateWrite {
  activeSoundscapeId: string | null;
  desiredPlaying: boolean;
  volume: number;
  expectedVersion: number;
  updatedAt: number;
}

export interface MusicSoundscapeSnapshot {
  status: MusicSoundscapeStatus;
  sourceId: string | null;
  volume: number;
  errorCode: string | null;
}

export interface MusicSoundscapeStartRequest {
  sourceId: string;
  generatedKind: MusicGeneratedNoiseKind | null;
  localPath: string | null;
  volume: number;
}

const sourceKinds = new Set<MusicSoundscapeSourceKind>(["generated-noise", "local-loop", "bundled-loop"]);
const generatedKinds = new Set<MusicGeneratedNoiseKind>(["white", "pink", "brown"]);
const availability = new Set<MusicSoundscapeAvailability>(["available", "missing", "unsupported"]);
const statuses = new Set<MusicSoundscapeStatus>(["idle", "playing", "paused", "error"]);

function record(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new Error(`${label} must be an object`);
  return value as Record<string, unknown>;
}
function text(value: unknown, label: string): string { if (typeof value !== "string") throw new Error(`${label} must be a string`); return value; }
function optionalText(value: unknown, label: string): string | null { return value === null ? null : text(value, label); }
function number(value: unknown, label: string): number { if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${label} must be finite`); return value; }
function integer(value: unknown, label: string): number { const result = number(value, label); if (!Number.isSafeInteger(result)) throw new Error(`${label} must be an integer`); return result; }
function boolean(value: unknown, label: string): boolean { if (typeof value !== "boolean") throw new Error(`${label} must be boolean`); return value; }
function enumeration<T extends string>(value: unknown, values: ReadonlySet<T>, label: string): T { if (typeof value !== "string" || !values.has(value as T)) throw new Error(`${label} is unsupported`); return value as T; }

export function parseMusicSoundscape(value: unknown, label = "soundscape"): MusicSoundscapeDefinition {
  const row = record(value, label);
  return {
    id: text(row.id, `${label}.id`),
    sourceKind: enumeration(row.sourceKind, sourceKinds, `${label}.sourceKind`),
    generatedKind: row.generatedKind === null ? null : enumeration(row.generatedKind, generatedKinds, `${label}.generatedKind`),
    bundledIdentity: optionalText(row.bundledIdentity, `${label}.bundledIdentity`),
    name: text(row.name, `${label}.name`),
    availability: enumeration(row.availability, availability, `${label}.availability`),
    localPath: optionalText(row.localPath, `${label}.localPath`),
    createdAt: integer(row.createdAt, `${label}.createdAt`),
    updatedAt: integer(row.updatedAt, `${label}.updatedAt`),
    version: integer(row.version, `${label}.version`),
  };
}
export function parseMusicSoundscapes(value: unknown): MusicSoundscapeDefinition[] {
  if (!Array.isArray(value)) throw new Error("soundscapes must be an array");
  return value.map((entry, index) => parseMusicSoundscape(entry, `soundscapes[${index}]`));
}
export function parseMusicSoundscapeState(value: unknown): MusicSoundscapeState {
  const row = record(value, "soundscape state");
  return { activeSoundscapeId: optionalText(row.activeSoundscapeId, "soundscape state.activeSoundscapeId"), desiredPlaying: boolean(row.desiredPlaying, "soundscape state.desiredPlaying"), volume: number(row.volume, "soundscape state.volume"), updatedAt: integer(row.updatedAt, "soundscape state.updatedAt"), version: integer(row.version, "soundscape state.version") };
}
export function parseMusicSoundscapeSnapshot(value: unknown): MusicSoundscapeSnapshot {
  const row = record(value, "soundscape snapshot");
  return { status: enumeration(row.status, statuses, "soundscape snapshot.status"), sourceId: optionalText(row.sourceId, "soundscape snapshot.sourceId"), volume: number(row.volume, "soundscape snapshot.volume"), errorCode: optionalText(row.errorCode, "soundscape snapshot.errorCode") };
}
