export type ChatSyntaxTokenKind =
  | "plain"
  | "keyword"
  | "string"
  | "number"
  | "comment"
  | "type"
  | "function"
  | "property"
  | "operator"
  | "tag"
  | "attribute"
  | "variable";

export interface ChatSyntaxToken {
  kind: ChatSyntaxTokenKind;
  text: string;
}

export interface ChatHighlightedLine {
  tokens: ChatSyntaxToken[];
}

export type ChatFileIconKind = "glyph" | "image" | "archive" | "text" | "file";
export type ChatFileIconTone =
  | "typed"
  | "script"
  | "style"
  | "markup"
  | "data"
  | "systems"
  | "docs"
  | "config"
  | "default";

export interface ChatFilePresentation {
  kind: ChatFileIconKind;
  glyph: string;
  tone: ChatFileIconTone;
}

const KEYWORDS = new Set([
  "abstract", "and", "as", "async", "await", "break", "case", "catch", "class", "const",
  "chan", "continue", "crate", "def", "default", "defer", "delete", "do", "elif", "else", "enum",
  "export", "extends", "extension", "extern", "false", "final", "finally", "fn", "for", "from", "func",
  "function", "go", "guard", "if", "impl", "import", "instanceof",
  "in", "interface", "is", "let", "loop", "match", "mod", "mut", "new", "none", "not", "null",
  "namespace", "of", "or", "override", "package", "pass", "private", "protected", "protocol", "pub",
  "public", "range", "readonly", "record", "return", "sealed", "select", "self", "static", "struct",
  "super", "switch", "synchronized", "this", "throw", "throws", "trait", "true", "try", "type",
  "typeof", "undefined", "union", "use", "using", "var", "virtual", "void", "where", "while", "with",
  "yield",
]);

const TYPE_WORDS = new Set([
  "any", "bigint", "bool", "boolean", "byte", "char", "decimal", "double", "f32", "f64", "float", "i8",
  "i16", "i32", "i64", "i128", "int", "long", "never", "number", "object", "rune", "short", "str",
  "string", "u8", "u16", "u32", "u64", "u128", "uint", "usize", "unknown",
]);

const IMAGE_EXTENSIONS = new Set(["avif", "gif", "ico", "jpeg", "jpg", "png", "svg", "webp"]);
const ARCHIVE_EXTENSIONS = new Set(["7z", "bz2", "gz", "rar", "tar", "tgz", "xz", "zip"]);
const TEXT_EXTENSIONS = new Set(["csv", "log", "pdf", "txt"]);

const GLYPHS: Readonly<Record<string, { glyph: string; tone: ChatFileIconTone }>> = {
  c: { glyph: "C", tone: "systems" },
  cc: { glyph: "C+", tone: "systems" },
  cpp: { glyph: "C+", tone: "systems" },
  cs: { glyph: "C#", tone: "typed" },
  css: { glyph: "#", tone: "style" },
  go: { glyph: "Go", tone: "typed" },
  h: { glyph: "H", tone: "systems" },
  hpp: { glyph: "H+", tone: "systems" },
  html: { glyph: "<>", tone: "markup" },
  java: { glyph: "J", tone: "systems" },
  js: { glyph: "JS", tone: "script" },
  json: { glyph: "{}", tone: "data" },
  jsx: { glyph: "JX", tone: "typed" },
  kt: { glyph: "Kt", tone: "systems" },
  lock: { glyph: "L", tone: "config" },
  md: { glyph: "M", tone: "docs" },
  mdx: { glyph: "M+", tone: "docs" },
  py: { glyph: "Py", tone: "script" },
  rb: { glyph: "Rb", tone: "script" },
  rs: { glyph: "R", tone: "systems" },
  sass: { glyph: "S", tone: "style" },
  scss: { glyph: "S#", tone: "style" },
  sh: { glyph: "$", tone: "script" },
  sql: { glyph: "DB", tone: "data" },
  svelte: { glyph: "S", tone: "markup" },
  swift: { glyph: "Sw", tone: "systems" },
  toml: { glyph: "T", tone: "config" },
  ts: { glyph: "TS", tone: "typed" },
  tsx: { glyph: "TX", tone: "typed" },
  vue: { glyph: "V", tone: "markup" },
  xml: { glyph: "<>", tone: "markup" },
  yaml: { glyph: "Y", tone: "config" },
  yml: { glyph: "Y", tone: "config" },
  zsh: { glyph: "$", tone: "script" },
};

/**
 * Resolves a compact, extension-aware presentation for common coding files.
 *
 * @param path Workspace-relative file path.
 * @returns Icon kind, short glyph, and semantic color tone.
 */
export function chatFilePresentation(path: string): ChatFilePresentation {
  const name = path.split("/").at(-1)?.toLowerCase() ?? path.toLowerCase();
  if (name === "dockerfile") return { kind: "glyph", glyph: "D", tone: "typed" };
  if (name === "makefile") return { kind: "glyph", glyph: "M", tone: "config" };
  if (name === ".gitignore" || name === ".gitattributes") return { kind: "glyph", glyph: "G", tone: "config" };
  if (name === "package.json") return { kind: "glyph", glyph: "N", tone: "data" };
  if (name === "cargo.toml") return { kind: "glyph", glyph: "R", tone: "systems" };
  const extension = name.includes(".") ? name.split(".").at(-1) ?? "" : "";
  const glyph = GLYPHS[extension];
  if (glyph) return { kind: "glyph", ...glyph };
  if (IMAGE_EXTENSIONS.has(extension)) return { kind: "image", glyph: "", tone: "markup" };
  if (ARCHIVE_EXTENSIONS.has(extension)) return { kind: "archive", glyph: "", tone: "config" };
  if (TEXT_EXTENSIONS.has(extension)) return { kind: "text", glyph: "", tone: "docs" };
  return { kind: "file", glyph: "", tone: "default" };
}

/**
 * Produces safe syntax token runs without creating HTML or loading an editor runtime.
 *
 * @param text UTF-8 file contents.
 * @param language Backend-detected language identifier.
 * @returns Highlighted lines whose text exactly reconstructs the input.
 */
export function highlightChatCode(text: string, language: string | null): ChatHighlightedLine[] {
  const normalized = language?.toLowerCase() ?? "text";
  const lines = text.split("\n");
  const result: ChatHighlightedLine[] = [];
  let blockEnd: string | null = null;

  for (const line of lines) {
    const tokens: ChatSyntaxToken[] = [];
    let index = 0;
    while (index < line.length) {
      if (blockEnd) {
        const end = line.indexOf(blockEnd, index);
        if (end < 0) {
          pushToken(tokens, "comment", line.slice(index));
          index = line.length;
          continue;
        }
        pushToken(tokens, "comment", line.slice(index, end + blockEnd.length));
        index = end + blockEnd.length;
        blockEnd = null;
        continue;
      }

      const block = blockCommentAt(line, index, normalized);
      if (block) {
        const end = line.indexOf(block.end, index + block.start.length);
        if (end < 0) {
          pushToken(tokens, "comment", line.slice(index));
          blockEnd = block.end;
          index = line.length;
        } else {
          pushToken(tokens, "comment", line.slice(index, end + block.end.length));
          index = end + block.end.length;
        }
        continue;
      }

      const comment = lineCommentAt(line, index, normalized);
      if (comment) {
        pushToken(tokens, "comment", line.slice(index));
        break;
      }

      const character = line[index];
      if (character === '"' || character === "'" || character === "`") {
        const end = stringEnd(line, index, character);
        const value = line.slice(index, end);
        const next = nextNonWhitespace(line, end);
        pushToken(tokens, next === ":" ? "property" : "string", value);
        index = end;
        continue;
      }

      const number = line.slice(index).match(/^(?:0x[\da-f]+|\d+(?:\.\d+)?(?:e[+-]?\d+)?)/i)?.[0];
      if (number) {
        pushToken(tokens, "number", number);
        index += number.length;
        continue;
      }

      const identifier = line.slice(index).match(/^(?:--[\w-]+|[$@]?[A-Za-z_][\w-]*)/)?.[0];
      if (identifier) {
        pushToken(tokens, classifyIdentifier(line, index, identifier, normalized), identifier);
        index += identifier.length;
        continue;
      }

      if (/^[{}()[\].,;:+*/%=&|!<>?~-]$/.test(character)) {
        pushToken(tokens, "operator", character);
      } else {
        pushToken(tokens, "plain", character);
      }
      index += 1;
    }
    result.push({ tokens });
  }
  return result;
}

/**
 * Estimates the editor width from the longest source line.
 *
 * @param text UTF-8 file contents.
 * @param maximum Maximum measured columns.
 * @returns Bounded monospace column count.
 */
export function chatCodeColumns(text: string, maximum = 500): number {
  let longest = 0;
  for (const line of text.split("\n")) longest = Math.max(longest, Array.from(line).length);
  return Math.min(Math.max(1, maximum), Math.max(1, longest));
}

function pushToken(tokens: ChatSyntaxToken[], kind: ChatSyntaxTokenKind, text: string): void {
  if (!text) return;
  const previous = tokens.at(-1);
  if (previous?.kind === kind) previous.text += text;
  else tokens.push({ kind, text });
}

function blockCommentAt(
  line: string,
  index: number,
  language: string,
): { start: string; end: string } | null {
  if (["html", "svelte", "vue", "markdown"].includes(language) && line.startsWith("<!--", index)) {
    return { start: "<!--", end: "-->" };
  }
  if (!["python", "shell", "yaml", "toml", "markdown", "text"].includes(language) && line.startsWith("/*", index)) {
    return { start: "/*", end: "*/" };
  }
  return null;
}

function lineCommentAt(line: string, index: number, language: string): boolean {
  if (["python", "ruby", "shell", "yaml", "toml"].includes(language)) return line[index] === "#";
  if (language === "sql") return line.startsWith("--", index);
  if ([
    "c", "cpp", "csharp", "go", "java", "javascript", "kotlin", "rust", "svelte", "swift",
    "typescript", "vue",
  ].includes(language)) return line.startsWith("//", index);
  return false;
}

function stringEnd(line: string, start: number, quote: string): number {
  let escaped = false;
  for (let index = start + 1; index < line.length; index++) {
    const character = line[index];
    if (escaped) escaped = false;
    else if (character === "\\") escaped = true;
    else if (character === quote) return index + 1;
  }
  return line.length;
}

function nextNonWhitespace(line: string, start: number): string {
  for (let index = start; index < line.length; index++) {
    if (!/\s/.test(line[index])) return line[index];
  }
  return "";
}

function classifyIdentifier(
  line: string,
  start: number,
  identifier: string,
  language: string,
): ChatSyntaxTokenKind {
  const normalized = identifier.replace(/^[$@]/, "").toLowerCase();
  const next = nextNonWhitespace(line, start + identifier.length);
  const previous = start > 0 ? line[start - 1] : "";
  if (identifier.startsWith("--") || identifier.startsWith("$")) return "variable";
  if (KEYWORDS.has(normalized) || identifier.startsWith("@")) return "keyword";
  if (TYPE_WORDS.has(normalized) || /^[A-Z]/.test(identifier)) return "type";
  if (["html", "svelte", "vue"].includes(language) && (previous === "<" || line.slice(0, start).endsWith("</"))) return "tag";
  if (["html", "svelte", "vue"].includes(language) && next === "=") return "attribute";
  if (next === "(") return "function";
  if (next === ":" && ["css", "json", "yaml", "toml"].includes(language)) return "property";
  return "plain";
}
