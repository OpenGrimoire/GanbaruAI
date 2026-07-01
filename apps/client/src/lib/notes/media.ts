import { richTextPlainText } from "./block-factory";
import type { NotesBlockType, NotesMediaBlockPayload } from "./types";

export const NOTES_IMAGE_EXTENSIONS = [
  ".bmp",
  ".gif",
  ".heic",
  ".jpeg",
  ".jpg",
  ".png",
  ".svg",
  ".tif",
  ".tiff",
  ".webp",
] as const;

export const NOTES_AUDIO_EXTENSIONS = [".mp3", ".wav", ".ogg", ".oga", ".m4a"] as const;

export const NOTES_VIDEO_EXTENSIONS = [
  ".amv",
  ".asf",
  ".avi",
  ".f4v",
  ".flv",
  ".gifv",
  ".mkv",
  ".mov",
  ".mpg",
  ".mpeg",
  ".mpv",
  ".mp4",
  ".m4v",
  ".qt",
  ".wmv",
] as const;

const MEDIA_BLOCK_TYPES = ["image", "video", "audio", "file", "pdf"] as const;

export type NotesMediaBlockType = (typeof MEDIA_BLOCK_TYPES)[number];
export type NotesMediaUrlIssue = "invalid_url" | "requires_https" | "unsupported_type";
export type NotesMediaPreviewKind = "image" | "video" | "audio" | "pdf" | "link" | "none";

export function isMediaBlockType(type: NotesBlockType): type is NotesMediaBlockType {
  return MEDIA_BLOCK_TYPES.includes(type as NotesMediaBlockType);
}

export function mediaCaptionPlainText(media: NotesMediaBlockPayload): string {
  return media.caption ? richTextPlainText(media.caption) : "";
}

export function mediaSourceUrl(media: NotesMediaBlockPayload): string {
  if (media.type === "external") return media.external.url;
  if (media.type === "file") return media.file.url;
  return "";
}

export function mediaSourceId(media: NotesMediaBlockPayload): string {
  if (media.type === "file_upload") return media.file_upload.id;
  return mediaSourceUrl(media);
}

export function mediaDisplayName(media: NotesMediaBlockPayload): string {
  return mediaDisplayNameFromSource(mediaSourceId(media), media.name);
}

export function mediaDisplayNameFromSource(source: string, name?: string): string {
  if (name?.trim()) return name.trim();
  const trimmedSource = source.trim();
  if (!trimmedSource) return "";
  try {
    const parsed = new URL(trimmedSource);
    const segments = parsed.pathname.split("/").filter(Boolean);
    return decodeURIComponent(segments.at(-1) ?? parsed.hostname);
  } catch {
    return trimmedSource;
  }
}

export function mediaPlainText(media: NotesMediaBlockPayload): string {
  return [mediaCaptionPlainText(media), mediaDisplayName(media), mediaSourceId(media)]
    .map((part) => part.trim())
    .filter(Boolean)
    .join(" ");
}

export function canOpenMediaUrl(media: NotesMediaBlockPayload): boolean {
  const url = mediaSourceUrl(media).trim();
  if (!url) return false;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export function mediaUrlIssue(
  type: NotesMediaBlockType,
  url: string,
): NotesMediaUrlIssue | null {
  const trimmed = url.trim();
  if (!trimmed) return null;
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return "invalid_url";
  }
  if (parsed.protocol !== "https:") return "requires_https";
  if (type === "file") return null;
  if (type === "pdf") return hasSupportedExtension(parsed, [".pdf"]) ? null : "unsupported_type";
  if (type === "image") {
    return hasSupportedExtension(parsed, NOTES_IMAGE_EXTENSIONS) ? null : "unsupported_type";
  }
  if (type === "audio") {
    return hasSupportedExtension(parsed, NOTES_AUDIO_EXTENSIONS) ? null : "unsupported_type";
  }
  if (hasSupportedExtension(parsed, NOTES_VIDEO_EXTENSIONS) || isYouTubeVideoUrl(parsed)) return null;
  return "unsupported_type";
}

export function externalMediaUrlIsSupported(type: NotesMediaBlockType, url: string): boolean {
  return mediaUrlIssue(type, url) === null;
}

export function mediaPreviewKind(
  type: NotesMediaBlockType,
  media: NotesMediaBlockPayload,
): NotesMediaPreviewKind {
  const url = mediaSourceUrl(media).trim();
  return mediaPreviewKindForUrl(type, url);
}

export function mediaPreviewKindForUrl(
  type: NotesMediaBlockType,
  url: string,
): NotesMediaPreviewKind {
  const trimmed = url.trim();
  if (!trimmed || mediaUrlIssue(type, trimmed)) return "none";
  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    return "none";
  }
  if (type === "file") return "link";
  if (type === "video" && isYouTubeVideoUrl(parsed)) return "link";
  return type;
}

export function canPreviewMedia(type: NotesMediaBlockType, media: NotesMediaBlockPayload): boolean {
  const kind = mediaPreviewKind(type, media);
  return kind === "image" || kind === "video" || kind === "audio" || kind === "pdf";
}

function hasSupportedExtension(url: URL, extensions: readonly string[]): boolean {
  const pathname = url.pathname.toLowerCase();
  return extensions.some((extension) => pathname.endsWith(extension));
}

function isYouTubeVideoUrl(url: URL): boolean {
  const host = url.hostname.toLowerCase();
  if (host !== "www.youtube.com" && host !== "youtube.com") return false;
  if (url.pathname === "/watch") return Boolean(url.searchParams.get("v"));
  return url.pathname.startsWith("/embed/");
}
