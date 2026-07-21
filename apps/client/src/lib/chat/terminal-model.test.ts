import { describe, expect, it } from "vitest";
import { applyTerminalOutput, boundTerminalContext, terminalPasteNeedsConfirmation } from "./terminal-model";

function chunk(generation: number, sequence: number, text: string) {
  return {
    terminalId: "terminal",
    generation,
    sequence,
    dataBase64: btoa(text),
    replay: false,
  };
}

describe("Chat terminal model", () => {
  it("accepts contiguous output and detects duplicates and replay gaps", () => {
    const initial = { generation: 1, lastSequence: 0 };
    const accepted = applyTerminalOutput(initial, chunk(1, 1, "hello"));
    expect(accepted.kind).toBe("accept");
    if (accepted.kind !== "accept") throw new Error("expected accepted output");
    expect(new TextDecoder().decode(accepted.bytes)).toBe("hello");
    expect(applyTerminalOutput(accepted.state, chunk(1, 1, "hello")).kind).toBe("duplicate");
    expect(applyTerminalOutput(accepted.state, chunk(1, 3, "later")).kind).toBe("gap");
    expect(applyTerminalOutput(accepted.state, chunk(2, 1, "restart")).kind).toBe("accept");
  });

  it("requires confirmation only for multiline paste when enabled", () => {
    expect(terminalPasteNeedsConfirmation("pnpm test", true)).toBe(false);
    expect(terminalPasteNeedsConfirmation("pnpm test\npnpm check", true)).toBe(true);
    expect(terminalPasteNeedsConfirmation("pnpm test\npnpm check", false)).toBe(false);
  });

  it("bounds UTF-8 context without splitting a code point", () => {
    const result = boundTerminalContext("abécd", 4);
    expect(result.text).toBe("abé");
    expect(result.byteSize).toBe(4);
    expect(result.truncated).toBe(true);
    expect(boundTerminalContext("abécd", 3).text).toBe("ab");
  });
});
