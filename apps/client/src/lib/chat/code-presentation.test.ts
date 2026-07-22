import { describe, expect, it } from "vitest";
import { chatCodeColumns, chatFilePresentation, highlightChatCode } from "./code-presentation";

describe("Chat code presentation", () => {
  it("assigns distinct presentations to common coding files", () => {
    expect(chatFilePresentation("src/app.ts")).toMatchObject({ kind: "glyph", glyph: "TS", tone: "typed" });
    expect(chatFilePresentation("src/App.svelte")).toMatchObject({ glyph: "S", tone: "markup" });
    expect(chatFilePresentation("Cargo.toml")).toMatchObject({ glyph: "R", tone: "systems" });
    expect(chatFilePresentation("assets/logo.png").kind).toBe("image");
  });

  it("preserves source text while distinguishing CSS and TypeScript tokens", () => {
    const css = '@import "tailwindcss";\n--size: 0.875rem;';
    const cssLines = highlightChatCode(css, "css");
    expect(cssLines.map((line) => line.tokens.map((token) => token.text).join("")).join("\n")).toBe(css);
    expect(cssLines[0].tokens.some((token) => token.kind === "keyword")).toBe(true);
    expect(cssLines[1].tokens.some((token) => token.kind === "variable")).toBe(true);
    expect(cssLines[1].tokens.some((token) => token.kind === "number")).toBe(true);

    const typescript = "const value: string = build(42); // result";
    const tokens = highlightChatCode(typescript, "typescript")[0].tokens;
    expect(tokens.some((token) => token.kind === "keyword" && token.text === "const")).toBe(true);
    expect(tokens.some((token) => token.kind === "type" && token.text === "string")).toBe(true);
    expect(tokens.some((token) => token.kind === "function" && token.text === "build")).toBe(true);
    expect(tokens.at(-1)).toEqual({ kind: "comment", text: "// result" });
  });

  it("recognizes common systems-language comments and keywords", () => {
    const tokens = highlightChatCode("func main() { // start", "go")[0].tokens;
    expect(tokens.some((token) => token.kind === "keyword" && token.text === "func")).toBe(true);
    expect(tokens.at(-1)).toEqual({ kind: "comment", text: "// start" });
  });

  it("bounds the editor width estimate", () => {
    expect(chatCodeColumns("a\n12345")).toBe(5);
    expect(chatCodeColumns("x".repeat(1_000), 200)).toBe(200);
  });
});
