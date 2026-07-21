import type { ChatTerminalOutputChunk } from "./contracts";

export interface TerminalOutputState {
  generation: number;
  lastSequence: number;
}

export type TerminalOutputDecision =
  | { kind: "accept"; state: TerminalOutputState; bytes: Uint8Array }
  | { kind: "duplicate"; state: TerminalOutputState }
  | { kind: "gap"; state: TerminalOutputState; expectedSequence: number };

export function applyTerminalOutput(
  state: TerminalOutputState,
  chunk: ChatTerminalOutputChunk,
): TerminalOutputDecision {
  if (chunk.generation < state.generation) return { kind: "duplicate", state };
  const generationChanged = chunk.generation > state.generation;
  const expected = generationChanged ? 1 : state.lastSequence + 1;
  if (chunk.sequence < expected) return { kind: "duplicate", state };
  if (chunk.sequence > expected) return { kind: "gap", state, expectedSequence: expected };
  return {
    kind: "accept",
    state: { generation: chunk.generation, lastSequence: chunk.sequence },
    bytes: decodeBase64(chunk.dataBase64),
  };
}

export function terminalPasteNeedsConfirmation(text: string, confirmationEnabled: boolean): boolean {
  return confirmationEnabled && /[\r\n]/.test(text);
}

export function boundTerminalContext(text: string, byteLimit: number): {
  text: string;
  byteSize: number;
  lineCount: number;
  truncated: boolean;
} {
  const bytes = new TextEncoder().encode(text);
  if (bytes.byteLength <= byteLimit) {
    return {
      text,
      byteSize: bytes.byteLength,
      lineCount: text.length === 0 ? 0 : text.split(/\r\n|\r|\n/).length,
      truncated: false,
    };
  }
  let boundary = byteLimit;
  while (boundary > 0 && (bytes[boundary] & 0xc0) === 0x80) boundary -= 1;
  const bounded = new TextDecoder().decode(bytes.slice(0, boundary));
  return {
    text: bounded,
    byteSize: new TextEncoder().encode(bounded).byteLength,
    lineCount: bounded.length === 0 ? 0 : bounded.split(/\r\n|\r|\n/).length,
    truncated: true,
  };
}

export const selectedTerminalByThread = new Map<string, string>();

function decodeBase64(value: string): Uint8Array {
  const decoded = atob(value);
  return Uint8Array.from(decoded, (character) => character.charCodeAt(0));
}
