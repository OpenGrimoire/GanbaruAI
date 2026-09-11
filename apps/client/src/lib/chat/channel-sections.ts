import { requireActiveVaultIdentity } from "$lib/vault/active-vault";

export interface ChatSidebarSection {
  id: string;
  name: string;
  collapsed: boolean;
  channelIds: string[];
}

const STORAGE_PREFIX = "ganbaru.chat.channel-sections.v1";
const LAST_CHANNEL_STORAGE_PREFIX = "ganbaru.chat.last-channel.v1";
const MAX_SECTIONS = 64;
const MAX_CHANNELS_PER_SECTION = 1_000;

/** Read and validate the custom channel-section layout for one project. */
export function readChatSidebarSections(projectId: string): ChatSidebarSection[] {
  if (typeof localStorage === "undefined") return [];
  const key = storageKey(STORAGE_PREFIX, projectId);
  if (!key) return [];
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    if (!Array.isArray(parsed)) return [];
    return parsed.slice(0, MAX_SECTIONS).flatMap((entry) => parseSection(entry));
  } catch {
    return [];
  }
}

/** Persist a bounded custom channel-section layout for one project. */
export function saveChatSidebarSections(projectId: string, sections: readonly ChatSidebarSection[]): void {
  if (typeof localStorage === "undefined") return;
  const key = storageKey(STORAGE_PREFIX, projectId);
  if (!key) return;
  localStorage.setItem(key, JSON.stringify(sections.slice(0, MAX_SECTIONS)));
}

/** Read the last channel selected for one project on this device. */
export function readLastChatChannelId(projectId: string): string | null {
  if (typeof localStorage === "undefined") return null;
  const key = storageKey(LAST_CHANNEL_STORAGE_PREFIX, projectId);
  if (!key) return null;
  const value = localStorage.getItem(key);
  return value && value.length <= 1_024 ? value : null;
}

/** Remember the last channel selected for one project on this device. */
export function saveLastChatChannelId(projectId: string, channelId: string): void {
  if (typeof localStorage === "undefined" || !channelId || channelId.length > 1_024) return;
  const key = storageKey(LAST_CHANNEL_STORAGE_PREFIX, projectId);
  if (!key) return;
  localStorage.setItem(key, channelId);
}

/** Return section layout without stale or duplicate channel assignments. */
export function normalizeChatSidebarSections(
  sections: readonly ChatSidebarSection[],
  channelIds: readonly string[],
): ChatSidebarSection[] {
  const available = new Set(channelIds);
  const assigned = new Set<string>();
  return sections.map((section) => ({
    ...section,
    channelIds: section.channelIds.filter((channelId) => {
      if (!available.has(channelId) || assigned.has(channelId)) return false;
      assigned.add(channelId);
      return true;
    }),
  }));
}

/** Move a channel into a custom section, or to the fixed Channels section with null. */
export function moveChatChannelToSection(
  sections: readonly ChatSidebarSection[],
  channelId: string,
  sectionId: string | null,
): ChatSidebarSection[] {
  return sections.map((section) => ({
    ...section,
    channelIds: [
      ...section.channelIds.filter((id) => id !== channelId),
      ...(section.id === sectionId ? [channelId] : []),
    ],
  }));
}

function parseSection(value: unknown): ChatSidebarSection[] {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return [];
  const record = value as Record<string, unknown>;
  if (typeof record.id !== "string" || typeof record.name !== "string" || typeof record.collapsed !== "boolean") return [];
  const name = record.name.trim();
  if (!record.id || record.id.length > 1_024 || !name || [...name].length > 80 || !Array.isArray(record.channelIds)) return [];
  return [{
    id: record.id,
    name,
    collapsed: record.collapsed,
    channelIds: record.channelIds
      .filter((entry): entry is string => typeof entry === "string" && entry.length > 0 && entry.length <= 1_024)
      .slice(0, MAX_CHANNELS_PER_SECTION),
  }];
}

function storageKey(prefix: string, projectId: string): string | null {
  try {
    return `${prefix}:${requireActiveVaultIdentity()}:${projectId}`;
  } catch {
    return null;
  }
}
