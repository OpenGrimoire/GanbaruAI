import { describe, expect, it, vi } from "vitest";
import { writeTextToClipboard, type ClipboardFallback, type ClipboardWriter } from "./clipboard";

describe("writeTextToClipboard", () => {
  it("uses the modern clipboard writer when it succeeds", async () => {
    const writeText = vi.fn<ClipboardWriter["writeText"]>().mockResolvedValue(undefined);
    const copy = vi.fn<ClipboardFallback["copy"]>().mockReturnValue(true);

    await writeTextToClipboard("final response", { writeText }, { copy });

    expect(writeText).toHaveBeenCalledWith("final response");
    expect(copy).not.toHaveBeenCalled();
  });

  it("falls back when the modern clipboard writer rejects", async () => {
    const writeText = vi.fn<ClipboardWriter["writeText"]>().mockRejectedValue(new Error("denied"));
    const copy = vi.fn<ClipboardFallback["copy"]>().mockReturnValue(true);

    await writeTextToClipboard("final response", { writeText }, { copy });

    expect(copy).toHaveBeenCalledWith("final response");
  });

  it("reports the writer failure when no fallback succeeds", async () => {
    const writeText = vi.fn<ClipboardWriter["writeText"]>().mockRejectedValue(new Error("denied"));
    const copy = vi.fn<ClipboardFallback["copy"]>().mockReturnValue(false);

    await expect(writeTextToClipboard("final response", { writeText }, { copy }))
      .rejects.toThrow("denied");
  });
});
