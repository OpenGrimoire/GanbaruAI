import { describe, expect, it } from "vitest";
import { createMediaPayload } from "./block-factory";
import {
  canOpenMediaUrl,
  canPreviewMedia,
  externalMediaUrlIsSupported,
  mediaDisplayName,
  mediaPlainText,
} from "./media";

describe("notes media helpers", () => {
  it("derives display text from captions, names, and source URLs", () => {
    const payload = createMediaPayload(
      "https://example.com/assets/brief.pdf",
      "Reference",
      "brief.pdf",
    );

    expect(mediaDisplayName(payload)).toBe("brief.pdf");
    expect(mediaPlainText(payload)).toBe("Reference brief.pdf https://example.com/assets/brief.pdf");
  });

  it("allows only supported HTTPS media URLs for previews", () => {
    expect(externalMediaUrlIsSupported("image", "https://example.com/image.png")).toBe(true);
    expect(externalMediaUrlIsSupported("image", "https://example.com/image.txt")).toBe(false);
    expect(externalMediaUrlIsSupported("audio", "https://example.com/audio.mp3")).toBe(true);
    expect(externalMediaUrlIsSupported("pdf", "https://example.com/file.pdf")).toBe(true);
    expect(externalMediaUrlIsSupported("video", "https://www.youtube.com/watch?v=abc123")).toBe(
      true,
    );
    expect(externalMediaUrlIsSupported("video", "http://example.com/video.mp4")).toBe(false);
  });

  it("opens non-empty HTTPS media URLs and previews renderable media only", () => {
    const image = createMediaPayload("https://example.com/image.png");
    const file = createMediaPayload("https://example.com/file.txt");

    expect(canOpenMediaUrl(image)).toBe(true);
    expect(canPreviewMedia("image", image)).toBe(true);
    expect(canOpenMediaUrl(file)).toBe(true);
    expect(canPreviewMedia("file", file)).toBe(false);
  });
});
