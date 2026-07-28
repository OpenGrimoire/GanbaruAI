export interface ClipboardWriter {
  writeText(value: string): Promise<void>;
}

export interface ClipboardFallback {
  copy(value: string): boolean;
}

/**
 * Writes text through the modern Clipboard API and falls back to a synchronous
 * document copy while the original user gesture is still active.
 */
export async function writeTextToClipboard(
  value: string,
  writer: ClipboardWriter | null = browserClipboardWriter(),
  fallback: ClipboardFallback | null = browserClipboardFallback(),
): Promise<void> {
  let primaryError: unknown = null;
  if (writer) {
    try {
      await writer.writeText(value);
      return;
    } catch (error: unknown) {
      primaryError = error;
    }
  }
  if (fallback?.copy(value)) return;
  if (primaryError instanceof Error) throw primaryError;
  throw new Error("Clipboard access is unavailable");
}

function browserClipboardWriter(): ClipboardWriter | null {
  if (typeof navigator === "undefined" || typeof navigator.clipboard?.writeText !== "function") {
    return null;
  }
  return {
    writeText: (value) => navigator.clipboard.writeText(value),
  };
}

function browserClipboardFallback(): ClipboardFallback | null {
  if (typeof document === "undefined" || typeof document.execCommand !== "function") return null;
  return {
    copy(value): boolean {
      const textarea = document.createElement("textarea");
      textarea.value = value;
      textarea.setAttribute("readonly", "");
      textarea.style.position = "fixed";
      textarea.style.inset = "0 auto auto -9999px";
      textarea.style.opacity = "0";
      document.body.append(textarea);
      textarea.select();
      textarea.setSelectionRange(0, value.length);
      try {
        return document.execCommand("copy");
      } finally {
        textarea.remove();
      }
    },
  };
}
