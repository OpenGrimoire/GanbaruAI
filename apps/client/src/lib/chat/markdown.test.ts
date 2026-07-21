// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { boundChatMarkdown, renderChatMarkdown, safeExternalUrl } from "./markdown";

describe("Chat Markdown", () => {
  it("renders common Markdown and preserves incomplete streaming fences as inert text", () => {
    expect(renderChatMarkdown("# Result\n\n| A | B |\n| - | - |\n| 1 | 2 |")).toContain("<table>");
    const incomplete = renderChatMarkdown("```ts\nconst value = '<unsafe>';");
    expect(incomplete).toContain("<pre><code");
    expect(incomplete).toContain("&lt;unsafe&gt;");
  });

  it("rejects raw HTML, active content, remote images, and credential-bearing links", () => {
    const html = renderChatMarkdown('<script>alert(1)</script><img src="https://tracker.example/x">\n\n[bad](javascript:alert(1)) [credentials](https://user:pass@example.com)');
    expect(html).not.toContain("<script");
    expect(html).not.toContain("<img");
    expect(html).not.toContain("javascript:");
    expect(html).not.toContain("user:pass");
    expect(html).toContain("&lt;script&gt;");
  });

  it("marks only explicit HTTP and HTTPS links for user-initiated opening", () => {
    expect(safeExternalUrl("https://example.com/path")).toBe(true);
    expect(safeExternalUrl("http://example.com/path")).toBe(true);
    expect(safeExternalUrl("file:///etc/passwd")).toBe(false);
    expect(safeExternalUrl("https://token@example.com")).toBe(false);
    expect(renderChatMarkdown("[Docs](https://example.com)")).toContain('data-chat-external-link="true"');
  });

  it("bounds adversarial line length and nesting before parsing", () => {
    const bounded = boundChatMarkdown(`${"> ".repeat(100)}deep\n${"x".repeat(40_000)}`);
    expect(bounded.match(/>/g)?.length).toBe(16);
    expect(bounded.split("\n")[1]?.length).toBe(32_768);
    expect(renderChatMarkdown("[incomplete](https://example")).toContain("[incomplete]");
  });
});
