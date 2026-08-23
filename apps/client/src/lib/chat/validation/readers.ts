import type { JsonValue, TurnModeSnapshot, UtcTimestamp, VersionedJson } from "../contracts";
import { INTERACTION_MODES, SAFETY_MODES } from "../contracts";

const MAX_IDENTIFIER_BYTES = 1_024;
const MAX_JSON_DEPTH = 32;
const MAX_JSON_NODES = 10_000;

export type UnknownRecord = Record<string, unknown>;

export function readRecord(value: unknown, label: string): UnknownRecord {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as UnknownRecord;
}

export function readString(value: unknown, label: string): string {
  if (typeof value !== "string") throw new Error(`${label} must be a string`);
  return value;
}

export function readIdentifier(value: unknown, label: string): string {
  const identifier = readString(value, label);
  if (identifier.length === 0) throw new Error(`${label} is required`);
  if (new TextEncoder().encode(identifier).byteLength > MAX_IDENTIFIER_BYTES) {
    throw new Error(`${label} exceeds the ${MAX_IDENTIFIER_BYTES} byte limit`);
  }
  for (const character of identifier) {
    const codePoint = character.codePointAt(0);
    if (codePoint !== undefined && (codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f))) {
      throw new Error(`${label} contains a control character`);
    }
  }
  return identifier;
}

export function readBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") throw new Error(`${label} must be a boolean`);
  return value;
}

export function readSafeInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    throw new Error(`${label} must be a safe integer`);
  }
  return value;
}

export function readFiniteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${label} must be a finite number`);
  }
  return value;
}

export function readNonNegativeSafeInteger(value: unknown, label: string): number {
  const integer = readSafeInteger(value, label);
  if (integer < 0) throw new Error(`${label} must not be negative`);
  return integer;
}

export function readNullable<T>(value: unknown, label: string, read: (value: unknown, label: string) => T): T | null {
  return value === null ? null : read(value, label);
}

export function readArray<T>(value: unknown, label: string, read: (value: unknown, label: string) => T): T[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((entry, index) => read(entry, `${label}[${index}]`));
}

export function readStringArray(value: unknown, label: string): string[] {
  return readArray(value, label, readString);
}

export function readStringRecord(value: unknown, label: string): Record<string, string> {
  const record = readRecord(value, label);
  return Object.fromEntries(
    Object.entries(record).map(([key, entry]) => [key, readString(entry, `${label}.${key}`)]),
  );
}

export function readEnum<const T extends readonly string[]>(value: unknown, values: T, label: string): T[number] {
  const candidate = readString(value, label);
  if (!(values as readonly string[]).includes(candidate)) {
    throw new Error(`${label} has an unsupported value`);
  }
  return candidate as T[number];
}

export function readUtcTimestamp(value: unknown, label: string): UtcTimestamp {
  const timestamp = readString(value, label);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|[+-]00:00)$/.exec(timestamp);
  if (!match) {
    throw new Error(`${label} must be an RFC 3339 UTC timestamp`);
  }
  const [, yearText, monthText, dayText, hourText, minuteText, secondText] = match;
  const [year, month, day, hour, minute, second] = [
    yearText,
    monthText,
    dayText,
    hourText,
    minuteText,
    secondText,
  ].map(Number);
  const parsed = new Date(0);
  parsed.setUTCFullYear(year, month - 1, day);
  parsed.setUTCHours(hour, minute, second, 0);
  if (
    parsed.getUTCFullYear() !== year
    || parsed.getUTCMonth() !== month - 1
    || parsed.getUTCDate() !== day
    || parsed.getUTCHours() !== hour
    || parsed.getUTCMinutes() !== minute
    || parsed.getUTCSeconds() !== second
  ) {
    throw new Error(`${label} must be an RFC 3339 UTC timestamp`);
  }
  return timestamp;
}

export function readJsonValue(value: unknown, label: string): JsonValue {
  let nodes = 0;

  const visit = (entry: unknown, path: string, depth: number): JsonValue => {
    nodes += 1;
    if (nodes > MAX_JSON_NODES) throw new Error(`${label} exceeds the JSON node limit`);
    if (depth > MAX_JSON_DEPTH) throw new Error(`${label} exceeds the JSON depth limit`);
    if (entry === null || typeof entry === "string" || typeof entry === "boolean") return entry;
    if (typeof entry === "number" && Number.isFinite(entry)) return entry;
    if (Array.isArray(entry)) return entry.map((item, index) => visit(item, `${path}[${index}]`, depth + 1));
    if (typeof entry === "object") {
      const result: { [key: string]: JsonValue } = {};
      for (const [key, item] of Object.entries(entry)) result[key] = visit(item, `${path}.${key}`, depth + 1);
      return result;
    }
    throw new Error(`${path} is not valid JSON`);
  };

  return visit(value, label, 0);
}

export function readVersionedJson(value: unknown, label: string): VersionedJson {
  const record = readRecord(value, label);
  return {
    schemaVersion: readNonNegativeSafeInteger(record.schemaVersion, `${label}.schemaVersion`),
    value: readJsonValue(record.value, `${label}.value`),
  };
}

export function readTurnModeSnapshot(value: unknown, label: string): TurnModeSnapshot {
  const record = readRecord(value, label);
  return {
    safetyMode: readEnum(record.safetyMode, SAFETY_MODES, `${label}.safetyMode`),
    interactionMode: readEnum(record.interactionMode, INTERACTION_MODES, `${label}.interactionMode`),
  };
}
