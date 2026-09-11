import { invoke } from "@tauri-apps/api/core";
import {
  inspectManagedImageFile,
  MANAGED_ICON_IMAGE_MAX_BYTES,
  MANAGED_ICON_IMAGE_MAX_MEGABYTES,
  MANAGED_IMAGE_FILE_ACCEPT,
  normalizeManagedImageDataUrl,
  type ManagedImageFileIssue,
} from "$lib/browser-file-policy";
import type { ProfileImageAsset } from "$lib/api/profile-image";

function imageIssueMessage(issue: ManagedImageFileIssue): string {
  switch (issue) {
    case "empty":
      return "The selected image is empty.";
    case "unsupported-type":
      return "Use a PNG, JPG, or WebP image.";
    case "too-large":
      return `The selected image exceeds the ${MANAGED_ICON_IMAGE_MAX_MEGABYTES} MB limit.`;
    case "dimensions-too-large":
      return "The selected image dimensions are too large.";
    case "too-many-pixels":
      return "The selected image contains too many pixels.";
    case "invalid-image":
      return "The selected file is not a valid PNG, JPG, or WebP image.";
  }
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") {
        resolve(reader.result);
        return;
      }
      reject(new Error("The selected image could not be read."));
    }, { once: true });
    reader.addEventListener("error", () => {
      reject(new Error("The selected image could not be read."));
    }, { once: true });
    reader.readAsDataURL(file);
  });
}

function chooseBrowserImage(title: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = MANAGED_IMAGE_FILE_ACCEPT;
    input.setAttribute("aria-label", title);
    input.style.position = "fixed";
    input.style.left = "-10000px";
    document.body.append(input);

    let settled = false;
    const finish = (file: File | null): void => {
      if (settled) return;
      settled = true;
      window.removeEventListener("focus", handleWindowFocus);
      input.remove();
      resolve(file);
    };
    const handleWindowFocus = (): void => {
      window.setTimeout(() => finish(input.files?.item(0) ?? null), 250);
    };

    input.addEventListener("change", () => finish(input.files?.item(0) ?? null), { once: true });
    input.addEventListener("cancel", () => finish(null), { once: true });
    window.addEventListener("focus", handleWindowFocus, { once: true });
    input.click();
  });
}

/** Open Android's document UI and copy a validated image into managed profile assets. */
export async function pickProfileImageFile(title: string): Promise<ProfileImageAsset | null> {
  const file = await chooseBrowserImage(title);
  if (!file) return null;

  const inspection = await inspectManagedImageFile(file, MANAGED_ICON_IMAGE_MAX_BYTES);
  if (!inspection.ok) throw new Error(imageIssueMessage(inspection.issue));

  const rawDataUrl = await readFileAsDataUrl(file);
  const dataUrl = normalizeManagedImageDataUrl(rawDataUrl, inspection.metadata.mimeType);
  if (!dataUrl) throw new Error("The selected image could not be encoded safely.");
  return invoke<ProfileImageAsset>("profile_image_save_data_url", { dataUrl });
}
