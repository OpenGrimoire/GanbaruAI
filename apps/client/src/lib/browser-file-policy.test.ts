import { describe, expect, it } from "vitest";
import {
  browserFileExceedsLimit,
  inspectManagedImageFile,
  managedImageDimensionIssue,
  managedImageFileIssue,
  MANAGED_ICON_IMAGE_MAX_BYTES,
  MANAGED_IMAGE_MAX_DIMENSION_PIXELS,
  MANAGED_IMAGE_MAX_TOTAL_PIXELS,
  NOTES_DATABASE_CSV_MAX_BYTES,
  NOTES_PAGE_COVER_IMAGE_MAX_BYTES,
  normalizeManagedImageDataUrl,
  parseManagedImageMetadata,
  type BrowserImageFile,
} from "./browser-file-policy";

function writeU16Be(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = value >>> 8;
  bytes[offset + 1] = value;
}

function writeU16Le(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = value;
  bytes[offset + 1] = value >>> 8;
}

function writeU24Le(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = value;
  bytes[offset + 1] = value >>> 8;
  bytes[offset + 2] = value >>> 16;
}

function writeU32Be(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = value >>> 24;
  bytes[offset + 1] = value >>> 16;
  bytes[offset + 2] = value >>> 8;
  bytes[offset + 3] = value;
}

function writeU32Le(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = value;
  bytes[offset + 1] = value >>> 8;
  bytes[offset + 2] = value >>> 16;
  bytes[offset + 3] = value >>> 24;
}

function png(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(33);
  bytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  writeU32Be(bytes, 8, 13);
  bytes.set([0x49, 0x48, 0x44, 0x52], 12);
  writeU32Be(bytes, 16, width);
  writeU32Be(bytes, 20, height);
  bytes.set([8, 6, 0, 0, 0], 24);
  return bytes;
}

function jpeg(width: number, height: number, sofMarker = 0xc0): Uint8Array {
  const bytes = new Uint8Array(27);
  bytes.set([0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00, 0xff, sofMarker, 0x00, 0x11, 8]);
  writeU16Be(bytes, 13, height);
  writeU16Be(bytes, 15, width);
  return bytes;
}

function webpVp8x(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(30);
  bytes.set([0x52, 0x49, 0x46, 0x46]);
  writeU32Le(bytes, 4, 22);
  bytes.set([0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x58], 8);
  writeU32Le(bytes, 16, 10);
  writeU24Le(bytes, 24, width - 1);
  writeU24Le(bytes, 27, height - 1);
  return bytes;
}

function webpVp8l(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(25);
  bytes.set([0x52, 0x49, 0x46, 0x46]);
  writeU32Le(bytes, 4, 17);
  bytes.set([0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x4c], 8);
  writeU32Le(bytes, 16, 5);
  const widthMinusOne = width - 1;
  const heightMinusOne = height - 1;
  bytes[20] = 0x2f;
  bytes[21] = widthMinusOne;
  bytes[22] = ((widthMinusOne >>> 8) & 0x3f) | ((heightMinusOne & 0x03) << 6);
  bytes[23] = heightMinusOne >>> 2;
  bytes[24] = (heightMinusOne >>> 10) & 0x0f;
  return bytes;
}

function webpVp8(width: number, height: number): Uint8Array {
  const bytes = new Uint8Array(30);
  bytes.set([0x52, 0x49, 0x46, 0x46]);
  writeU32Le(bytes, 4, 22);
  bytes.set([0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x20], 8);
  writeU32Le(bytes, 16, 10);
  bytes.set([0, 0, 0, 0x9d, 0x01, 0x2a], 20);
  writeU16Le(bytes, 26, width);
  writeU16Le(bytes, 28, height);
  return bytes;
}

function browserImageFile(
  bytes: Uint8Array,
  type: string,
  reportedSize = bytes.byteLength,
): BrowserImageFile {
  const blobBytes = new Uint8Array(bytes.byteLength);
  blobBytes.set(bytes);
  const blob = new Blob([blobBytes.buffer], { type });
  return {
    size: reportedSize,
    type,
    slice: (start, end) => blob.slice(start, end),
  };
}

describe("managed image metadata", () => {
  it("parses PNG and progressive JPEG dimensions", () => {
    expect(parseManagedImageMetadata(png(4032, 3024))).toEqual({
      ok: true,
      metadata: { kind: "png", mimeType: "image/png", width: 4032, height: 3024 },
    });
    expect(parseManagedImageMetadata(jpeg(4096, 2160, 0xc2))).toEqual({
      ok: true,
      metadata: { kind: "jpeg", mimeType: "image/jpeg", width: 4096, height: 2160 },
    });
  });

  it("parses VP8, VP8L, and VP8X WebP dimensions", () => {
    for (const bytes of [webpVp8(1920, 1080), webpVp8l(4000, 3000), webpVp8x(8192, 1200)]) {
      const result = parseManagedImageMetadata(bytes);
      expect(result.ok).toBe(true);
      if (result.ok) expect(result.metadata.kind).toBe("webp");
    }
    expect(parseManagedImageMetadata(webpVp8l(4000, 3000))).toMatchObject({
      metadata: { width: 4000, height: 3000 },
    });
  });

  it("rejects unsupported signatures and malformed or zero dimensions", () => {
    expect(parseManagedImageMetadata(new TextEncoder().encode("GIF89a"))).toEqual({
      ok: false,
      issue: "unsupported-signature",
    });
    expect(parseManagedImageMetadata(png(0, 100))).toEqual({ ok: false, issue: "malformed-header" });
    expect(parseManagedImageMetadata(jpeg(100, 0))).toEqual({ ok: false, issue: "malformed-header" });
    expect(parseManagedImageMetadata(webpVp8(0, 100))).toEqual({ ok: false, issue: "malformed-header" });
  });

  it("rejects truncated chunks, invalid JPEG segment lengths, and unknown WebP chunks", () => {
    expect(parseManagedImageMetadata(png(10, 10).slice(0, 23))).toEqual({
      ok: false,
      issue: "malformed-header",
    });
    expect(parseManagedImageMetadata(new Uint8Array([0xff, 0xd8, 0xff, 0xe0, 0xff, 0xff]))).toEqual({
      ok: false,
      issue: "malformed-header",
    });
    const unknownWebp = webpVp8x(10, 10);
    unknownWebp.set([0x4a, 0x55, 0x4e, 0x4b], 12);
    expect(parseManagedImageMetadata(unknownWebp)).toEqual({ ok: false, issue: "malformed-header" });
  });

  it("accepts normal 12 MP geometry and enforces side and total-pixel limits", () => {
    expect(managedImageDimensionIssue({ width: 4032, height: 3024 })).toBeNull();
    expect(managedImageDimensionIssue({
      width: MANAGED_IMAGE_MAX_DIMENSION_PIXELS + 1,
      height: 1,
    })).toBe("dimensions-too-large");
    expect(managedImageDimensionIssue({
      width: 4000,
      height: Math.floor(MANAGED_IMAGE_MAX_TOTAL_PIXELS / 4000) + 1,
    })).toBe("too-many-pixels");
  });
});

describe("browser file policy", () => {
  it("accepts supported images through each inclusive byte limit", async () => {
    await expect(managedImageFileIssue(
      browserImageFile(png(512, 512), "image/png", MANAGED_ICON_IMAGE_MAX_BYTES),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBeNull();
    await expect(managedImageFileIssue(
      browserImageFile(jpeg(4032, 3024), " IMAGE/JPEG ", NOTES_PAGE_COVER_IMAGE_MAX_BYTES),
      NOTES_PAGE_COVER_IMAGE_MAX_BYTES,
    )).resolves.toBeNull();
    await expect(managedImageFileIssue(
      browserImageFile(webpVp8l(512, 512), ""),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBeNull();
  });

  it("exposes a canonical MIME for blank browser metadata and normalizes the data URL", async () => {
    const inspection = await inspectManagedImageFile(
      browserImageFile(webpVp8l(512, 512), ""),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    );
    expect(inspection).toMatchObject({
      ok: true,
      metadata: { mimeType: "image/webp" },
    });
    if (!inspection.ok) throw new Error("expected a valid managed image");
    expect(normalizeManagedImageDataUrl(
      "data:;base64,AQID",
      inspection.metadata.mimeType,
    )).toBe("data:image/webp;base64,AQID");
    expect(normalizeManagedImageDataUrl(
      "data:application/octet-stream;base64,AQID",
      inspection.metadata.mimeType,
    )).toBe("data:image/webp;base64,AQID");
  });

  it("does not normalize malformed or non-base64 data URLs", () => {
    expect(normalizeManagedImageDataUrl("not-a-data-url", "image/png")).toBeNull();
    expect(normalizeManagedImageDataUrl("data:image/png,raw", "image/png")).toBeNull();
  });

  it("rejects empty, unsupported, and oversized images before header reads", async () => {
    await expect(managedImageFileIssue(
      browserImageFile(png(1, 1), "image/png", 0),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("empty");
    await expect(managedImageFileIssue(
      browserImageFile(png(1, 1), "image/gif"),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("unsupported-type");
    await expect(managedImageFileIssue(
      browserImageFile(png(1, 1), "image/png", MANAGED_ICON_IMAGE_MAX_BYTES + 1),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("too-large");
  });

  it("rejects MIME and signature mismatch before FileReader allocation", async () => {
    await expect(managedImageFileIssue(
      browserImageFile(png(512, 512), "image/jpeg"),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("invalid-image");
    await expect(managedImageFileIssue(
      browserImageFile(new TextEncoder().encode("not an image"), "image/png"),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("invalid-image");
  });

  it("reports dimension and pixel limits independently", async () => {
    await expect(managedImageFileIssue(
      browserImageFile(png(MANAGED_IMAGE_MAX_DIMENSION_PIXELS + 1, 1), "image/png"),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("dimensions-too-large");
    await expect(managedImageFileIssue(
      browserImageFile(png(5000, 4000), "image/png"),
      MANAGED_ICON_IMAGE_MAX_BYTES,
    )).resolves.toBe("too-many-pixels");
  });

  it("treats the CSV byte limit as inclusive", () => {
    expect(browserFileExceedsLimit(
      { size: NOTES_DATABASE_CSV_MAX_BYTES },
      NOTES_DATABASE_CSV_MAX_BYTES,
    )).toBe(false);
    expect(browserFileExceedsLimit(
      { size: NOTES_DATABASE_CSV_MAX_BYTES + 1 },
      NOTES_DATABASE_CSV_MAX_BYTES,
    )).toBe(true);
  });
});
