import type { MusicMembershipSkipRange } from "$lib/music/library-contracts";

export function parseMusicTimecode(value: string): number | null {
  const input = value.trim();
  if (!input) return null;
  const parts = input.split(":");
  if (parts.length > 3 || parts.some((part) => !/^\d+(?:\.\d{1,3})?$/.test(part))) return null;
  const seconds = parts.reduce((total, part) => total * 60 + Number(part), 0);
  if (!Number.isFinite(seconds) || seconds < 0) return null;
  return Math.round(seconds * 1_000);
}

export function formatMusicTimecode(valueMs: number | null): string {
  if (valueMs === null) return "";
  const totalSeconds = Math.max(0, Math.round(valueMs / 1_000));
  const seconds = totalSeconds % 60;
  const totalMinutes = Math.floor(totalSeconds / 60);
  const minutes = totalMinutes % 60;
  const hours = Math.floor(totalMinutes / 60);
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`
    : `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function validateMusicSkipRanges(ranges: readonly MusicMembershipSkipRange[]): string | null {
  let previousEnd = -1;
  for (const range of ranges) {
    if (range.startMs < 0 || range.endMs <= range.startMs) return "invalid-range";
    if (range.startMs < previousEnd) return "overlapping-range";
    previousEnd = range.endMs;
  }
  return null;
}
