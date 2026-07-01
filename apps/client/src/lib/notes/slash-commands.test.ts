import { describe, expect, it } from "vitest";
import {
  filterNotesSlashCommandItems,
  notesSlashCommandKey,
  notesSlashCommandItems,
  recordNotesSlashCommandKey,
  sectionNotesSlashCommandItems,
} from "./slash-commands";

describe("notes slash commands", () => {
  it("builds block action and color commands for color-capable blocks", () => {
    const commands = notesSlashCommandItems({ canSetColor: true });

    expect(commands).toContainEqual(
      expect.objectContaining({
        section: "blocks",
        command: { kind: "block", blockType: "paragraph" },
      }),
    );
    expect(commands).toContainEqual(
      expect.objectContaining({
        section: "actions",
        command: { kind: "action", action: "duplicate" },
      }),
    );
    expect(commands).toContainEqual(
      expect.objectContaining({
        section: "colors",
        command: { kind: "color", color: "red_background" },
      }),
    );
  });

  it("omits color commands for blocks that cannot store block color", () => {
    const commands = notesSlashCommandItems({ canSetColor: false });

    expect(commands.some((command) => command.command.kind === "color")).toBe(false);
    expect(commands.some((command) => command.command.kind === "action")).toBe(true);
    expect(commands.some((command) => command.command.kind === "block")).toBe(true);
  });

  it("builds toggle heading commands in the block section", () => {
    const commands = notesSlashCommandItems({ canSetColor: false });

    expect(commands).toContainEqual(
      expect.objectContaining({
        section: "blocks",
        command: { kind: "toggle_heading", headingType: "heading_2" },
      }),
    );
    expect(commands).toContainEqual(
      expect.objectContaining({
        section: "blocks",
        command: { kind: "toggle_heading", headingType: "heading_4" },
      }),
    );
    expect(notesSlashCommandKey({ kind: "toggle_heading", headingType: "heading_2" })).toBe(
      "toggle-heading:heading_2",
    );
  });

  it("filters by command aliases", () => {
    const commands = notesSlashCommandItems({ canSetColor: true });

    expect(filterNotesSlashCommandItems(commands, "dup")).toContainEqual(
      expect.objectContaining({ command: { kind: "action", action: "duplicate" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "red bg")).toContainEqual(
      expect.objectContaining({ command: { kind: "color", color: "red_background" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "toc")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "table_of_contents" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "preview")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "link_preview" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "template button")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "template" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "insert blocks")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "button" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "tabs")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "tab" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "toggle h2")).toContainEqual(
      expect.objectContaining({ command: { kind: "toggle_heading", headingType: "heading_2" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "h4")).toContainEqual(
      expect.objectContaining({ command: { kind: "block", blockType: "heading_4" } }),
    );
    expect(filterNotesSlashCommandItems(commands, "toggle h4")).toContainEqual(
      expect.objectContaining({ command: { kind: "toggle_heading", headingType: "heading_4" } }),
    );
  });

  it("keeps recent commands first without duplicating them in their normal section", () => {
    const commands = notesSlashCommandItems({ canSetColor: true });
    const recentKeys = recordNotesSlashCommandKey([], "action:duplicate");
    const sections = sectionNotesSlashCommandItems(commands, "", recentKeys);

    expect(sections.recent).toEqual([
      expect.objectContaining({ command: { kind: "action", action: "duplicate" } }),
    ]);
    expect(sections.actions).not.toContainEqual(
      expect.objectContaining({ command: { kind: "action", action: "duplicate" } }),
    );
  });

  it("does not show recent commands while filtering", () => {
    const commands = notesSlashCommandItems({ canSetColor: true });
    const recentKeys = recordNotesSlashCommandKey([], "action:duplicate");
    const sections = sectionNotesSlashCommandItems(commands, "dup", recentKeys);

    expect(sections.recent).toEqual([]);
    expect(sections.actions).toContainEqual(
      expect.objectContaining({ command: { kind: "action", action: "duplicate" } }),
    );
  });
});
