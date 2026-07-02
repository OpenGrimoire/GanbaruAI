import { describe, expect, it } from "vitest";
import { createMediaPayload } from "./block-factory";
import {
  canOpenMediaUrl,
  canPreviewMedia,
  createManagedMediaPayload,
  externalMediaUrlIsSupported,
  mediaDisplayName,
  mediaManagedAssetMetadata,
  mediaPlainText,
  mediaPreviewKindForUrl,
  mediaUrlIssue,
} from "./media";

const localPngSha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const localPngPath = `notes/files/${localPngSha}.png`;

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

  it("creates and reads managed local media metadata without treating it as an external URL", () => {
    const payload = createManagedMediaPayload(
      {
        relativePath: localPngPath,
        originalName: "local.png",
        contentType: "image/png",
        byteSize: 42,
        sha256: localPngSha,
        kind: "image",
      },
      "Local cover",
    );

    expect(payload).toEqual({
      type: "file",
      file: {
        url: `ganbaru-asset:${localPngPath}`,
        name: "local.png",
        content_type: "image/png",
        byte_size: 42,
        sha256: localPngSha,
        ganbaru_asset_path: localPngPath,
      },
      caption: expect.arrayContaining([expect.objectContaining({ plain_text: "Local cover" })]),
      name: "local.png",
    });
    expect(mediaManagedAssetMetadata(payload)).toEqual({
      relativePath: localPngPath,
      originalName: "local.png",
      contentType: "image/png",
      byteSize: 42,
      sha256: localPngSha,
      kind: "image",
    });
    expect(mediaPreviewKindForUrl("image", `ganbaru-asset:${localPngPath}`)).toBe("image");
    expect(mediaUrlIssue("image", `ganbaru-asset:${localPngPath}`)).toBe("requires_https");
    expect(canOpenMediaUrl(payload)).toBe(false);
    expect(canPreviewMedia("image", payload)).toBe(true);
  });
});
