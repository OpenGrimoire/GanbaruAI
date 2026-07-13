import type { Component } from "svelte";
import Palette from "@lucide/svelte/icons/palette";
import UserRound from "@lucide/svelte/icons/user-round";
import Calendar from "@lucide/svelte/icons/calendar";
import Folder from "@lucide/svelte/icons/folder";
import Book from "@lucide/svelte/icons/book";
import GlobeOff from "@lucide/svelte/icons/globe-off";
import Info from "@lucide/svelte/icons/info";
import Keyboard from "@lucide/svelte/icons/keyboard";
import Music from "@lucide/svelte/icons/music";
import Timer from "@lucide/svelte/icons/timer";
import DownloadCloud from "@lucide/svelte/icons/download-cloud";
import HardDrive from "@lucide/svelte/icons/hard-drive";
import type { SectionId } from "./types";

export interface SettingsSectionMeta {
  id: SectionId;
  labelKey: `settings.section.${SectionId}`;
  icon: Component;
}

export const SETTINGS_SECTIONS: SettingsSectionMeta[] = [
  { id: "appearance", labelKey: "settings.section.appearance", icon: Palette },
  { id: "profile", labelKey: "settings.section.profile", icon: UserRound },
  { id: "calendars", labelKey: "settings.section.calendars", icon: Calendar },
  { id: "projects", labelKey: "settings.section.projects", icon: Folder },
  { id: "notes", labelKey: "settings.section.notes", icon: Book },
  { id: "focus", labelKey: "settings.section.focus", icon: Timer },
  { id: "music", labelKey: "settings.section.music", icon: Music },
  { id: "doomscrolling", labelKey: "settings.section.doomscrolling", icon: GlobeOff },
  { id: "data", labelKey: "settings.section.data", icon: HardDrive },
  { id: "updates", labelKey: "settings.section.updates", icon: DownloadCloud },
  { id: "shortcuts", labelKey: "settings.section.shortcuts", icon: Keyboard },
  { id: "about", labelKey: "settings.section.about", icon: Info },
];
