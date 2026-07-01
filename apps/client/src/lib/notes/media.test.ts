import { describe, expect, it } from "vitest";
import { createMediaPayload } from "./block-factory";
import {
  canOpenMediaUrl,
  canPreviewMedia,
  externalMediaUrlIsSupported,
  mediaDisplayName,
  mediaPlainText,
  mediaPreviewKindForUrl,
  mediaUrlIssue,
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

  it("explains invalid media URL states", () => {
    expect(mediaUrlIssue("image", "")).toBeNull();
    expect(mediaUrlIssue("image", "not a url")).toBe("invalid_url");
    expect(mediaUrlIssue("image", "http://example.com/image.png")).toBe("requires_https");
    expect(mediaUrlIssue("image", "https://example.com/image.txt")).toBe("unsupported_type");
    expect(mediaUrlIssue("file", "https://example.com/download")).toBeNull();
  });

  it("separates native previews from explicit open links", () => {
    expect(mediaPreviewKindForUrl("image", "https://example.com/image.png")).toBe("image");
    expect(mediaPreviewKindForUrl("audio", "https://example.com/audio.mp3")).toBe("audio");
    expect(mediaPreviewKindForUrl("video", "https://example.com/video.mp4")).toBe("video");
    expect(mediaPreviewKindForUrl("pdf", "https://example.com/file.pdf")).toBe("pdf");
    expect(mediaPreviewKindForUrl("file", "https://example.com/file.txt")).toBe("link");
    expect(mediaPreviewKindForUrl("video", "https://www.youtube.com/watch?v=abc123")).toBe(
      "link",
    );
    expect(mediaPreviewKindForUrl("image", "https://example.com/file.txt")).toBe("none");
  });

  it("opens non-empty HTTPS media URLs and previews renderable media only", () => {
    const image = createMediaPayload("https://example.com/image.png");
    const file = createMediaPayload("https://example.com/file.txt");
    const youtube = createMediaPayload("https://www.youtube.com/watch?v=abc123");

    expect(canOpenMediaUrl(image)).toBe(true);
    expect(canPreviewMedia("image", image)).toBe(true);
    expect(canOpenMediaUrl(file)).toBe(true);
    expect(canPreviewMedia("file", file)).toBe(false);
    expect(canOpenMediaUrl(youtube)).toBe(true);
    expect(canPreviewMedia("video", youtube)).toBe(false);
  });
});
