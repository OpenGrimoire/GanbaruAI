const REDACTED_CHARACTER_ALPHABET = "abcdefghjkmnpqrstuvwxyz23456789";
const PRESERVED_SEPARATORS = new Set(["@", ".", "-", "_"]);

/**
 * Builds a stable fake value that preserves the shape of sensitive account text.
 *
 * The generated value never includes an original alphanumeric character at the
 * same position. Email separators remain visible so the result still reads as an
 * account identity while blurred.
 *
 * @param value - Sensitive account text used only as deterministic input.
 * @returns A same-length placeholder with the original structural separators.
 */
export function redactedSensitiveText(value: string): string {
  let state = 0x811c9dc5;
  for (const character of value) {
    state ^= character.codePointAt(0) ?? 0;
    state = Math.imul(state, 0x01000193);
  }

  return Array.from(value, (character) => {
    if (PRESERVED_SEPARATORS.has(character)) return character;

    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    const original = character.toLowerCase();
    const start = (state >>> 0) % REDACTED_CHARACTER_ALPHABET.length;
    for (let offset = 0; offset < REDACTED_CHARACTER_ALPHABET.length; offset += 1) {
      const replacement = REDACTED_CHARACTER_ALPHABET[
        (start + offset) % REDACTED_CHARACTER_ALPHABET.length
      ] ?? "x";
      if (replacement !== original) return replacement;
    }
    return "x";
  }).join("");
}
