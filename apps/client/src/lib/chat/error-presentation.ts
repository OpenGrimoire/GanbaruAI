const FALLBACK_CHAT_ERROR = "Chat operation failed";

/** Extracts a useful bounded message from browser, Tauri, and provider failures. */
export function chatErrorMessage(error: unknown, fallback = FALLBACK_CHAT_ERROR): string {
  if (error instanceof Error && error.message.trim()) return error.message.trim();
  if (typeof error === "string" && error.trim()) return error.trim();
  const record = objectRecord(error);
  if (!record) return fallback;
  const direct = stringValue(record.message) ?? stringValue(record.error) ?? stringValue(record.detail);
  if (direct) return direct;
  const nested = objectRecord(record.error);
  return stringValue(nested?.message) ?? stringValue(nested?.detail) ?? fallback;
}

function objectRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function stringValue(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}
