<script module lang="ts">
  import type { NotesSlashCommandKey } from "$lib/notes/slash-commands";

  let recentSlashCommandKeys: NotesSlashCommandKey[] = [];
</script>

<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import { notesBlockColorSwatchStyle } from "$lib/notes/block-color";
  import type { NotesHeadingBlockType } from "$lib/notes/block-factory";
  import type { NotesInsertableBlockType } from "$lib/notes/block-insertion";
  import {
    notesSlashCommandItems,
    recordNotesSlashCommandKey,
    sectionNotesSlashCommandItems,
    type NotesSlashAction,
    type NotesSlashCommand,
    type NotesSlashCommandItem,
    type NotesSlashCommandPanelSection,
  } from "$lib/notes/slash-commands";
  import type { NotesColor } from "$lib/notes/types";
  import type { Component } from "svelte";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import Check from "@lucide/svelte/icons/check";
  import Code from "@lucide/svelte/icons/code";
  import Columns2 from "@lucide/svelte/icons/columns-2";
  import Copy from "@lucide/svelte/icons/copy";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FileIcon from "@lucide/svelte/icons/file";
  import FileText from "@lucide/svelte/icons/file-text";
  import Heading1 from "@lucide/svelte/icons/heading-1";
  import Heading2 from "@lucide/svelte/icons/heading-2";
  import Heading3 from "@lucide/svelte/icons/heading-3";
  import Heading4 from "@lucide/svelte/icons/heading-4";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LinkIcon from "@lucide/svelte/icons/link";
  import List from "@lucide/svelte/icons/list";
  import ListCollapse from "@lucide/svelte/icons/list-collapse";
  import ListOrdered from "@lucide/svelte/icons/list-ordered";
  import ListTree from "@lucide/svelte/icons/list-tree";
  import MessageSquareWarning from "@lucide/svelte/icons/message-square-warning";
  import Minus from "@lucide/svelte/icons/minus";
  import MousePointerClick from "@lucide/svelte/icons/mouse-pointer-click";
  import Music from "@lucide/svelte/icons/music";
  import Pilcrow from "@lucide/svelte/icons/pilcrow";
  import Quote from "@lucide/svelte/icons/quote";
  import Route from "@lucide/svelte/icons/route";
  import Sigma from "@lucide/svelte/icons/sigma";
  import SquareCheck from "@lucide/svelte/icons/square-check";
  import Table2 from "@lucide/svelte/icons/table-2";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Video from "@lucide/svelte/icons/video";

  let {
    query = "",
    canSetColor = true,
    currentColor = "default",
    menuId = undefined,
    menuClass = "absolute left-[calc(var(--notes-depth)*1.25rem+1.75rem)] top-full mt-1",
    onSelect,
  }: {
    query?: string;
    canSetColor?: boolean;
    currentColor?: NotesColor;
    menuId?: string;
    menuClass?: string;
    onSelect: (command: NotesSlashCommand) => void;
  } = $props();

  const { t } = getLocalization();
  const sectionOrder = [
    "recent",
    "blocks",
    "actions",
    "colors",
  ] as const satisfies readonly NotesSlashCommandPanelSection[];
  let recentKeys = $state<readonly NotesSlashCommandKey[]>(recentSlashCommandKeys);
  const catalog = $derived(notesSlashCommandItems({ canSetColor }));
  const sections = $derived(sectionNotesSlashCommandItems(catalog, query, recentKeys));
  const hasResults = $derived(sectionOrder.some((section) => sections[section].length > 0));

  function selectCommand(item: NotesSlashCommandItem): void {
    recentSlashCommandKeys = recordNotesSlashCommandKey(recentSlashCommandKeys, item.key);
    recentKeys = recentSlashCommandKeys;
    onSelect(item.command);
  }

  function sectionLabel(section: NotesSlashCommandPanelSection): string {
    switch (section) {
      case "recent":
        return t("notes.slashRecent");
      case "blocks":
        return t("notes.slashBlocks");
      case "actions":
        return t("notes.slashActions");
      case "colors":
        return t("notes.slashColors");
    }
  }

  function commandLabel(command: NotesSlashCommand): string {
    switch (command.kind) {
      case "block":
        return blockLabel(command.blockType);
      case "toggle_heading":
        return toggleHeadingLabel(command.headingType);
      case "action":
        return actionLabel(command.action);
      case "color":
        return colorLabel(command.color);
    }
  }

  function blockLabel(type: NotesInsertableBlockType): string {
    switch (type) {
      case "paragraph":
        return t("notes.blockType.paragraph");
      case "heading_1":
        return t("notes.blockType.heading1");
      case "heading_2":
        return t("notes.blockType.heading2");
      case "heading_3":
        return t("notes.blockType.heading3");
      case "heading_4":
        return t("notes.blockType.heading4");
      case "bulleted_list_item":
        return t("notes.blockType.bullet");
      case "numbered_list_item":
        return t("notes.blockType.numbered");
      case "to_do":
        return t("notes.blockType.todo");
      case "toggle":
        return t("notes.blockType.toggle");
      case "callout":
        return t("notes.blockType.callout");
      case "quote":
        return t("notes.blockType.quote");
      case "child_page":
        return t("notes.blockType.childPage");
      case "breadcrumb":
        return t("notes.blockType.breadcrumb");
      case "table_of_contents":
        return t("notes.blockType.tableOfContents");
      case "column_list":
        return t("notes.blockType.columns");
      case "table":
        return t("notes.blockType.table");
      case "tab":
        return t("notes.blockType.tab");
      case "image":
        return t("notes.blockType.image");
      case "video":
        return t("notes.blockType.video");
      case "audio":
        return t("notes.blockType.audio");
      case "file":
        return t("notes.blockType.file");
      case "pdf":
        return t("notes.blockType.pdf");
      case "bookmark":
        return t("notes.blockType.bookmark");
      case "link_preview":
        return t("notes.blockType.linkPreview");
      case "template":
        return t("notes.blockType.template");
      case "button":
        return t("notes.blockType.button");
      case "embed":
        return t("notes.blockType.embed");
      case "equation":
        return t("notes.blockType.equation");
      case "divider":
        return t("notes.blockType.divider");
      case "code":
        return t("notes.blockType.code");
    }
  }

  function toggleHeadingLabel(type: NotesHeadingBlockType): string {
    switch (type) {
      case "heading_1":
        return t("notes.blockType.toggleHeading1");
      case "heading_2":
        return t("notes.blockType.toggleHeading2");
      case "heading_3":
        return t("notes.blockType.toggleHeading3");
      case "heading_4":
        return t("notes.blockType.toggleHeading4");
    }
  }

  function actionLabel(action: NotesSlashAction): string {
    switch (action) {
      case "copy_link":
        return t("notes.copyBlockLink");
      case "duplicate":
        return t("notes.duplicateBlock");
      case "move_up":
        return t("notes.moveBlockUp");
      case "move_down":
        return t("notes.moveBlockDown");
      case "delete":
        return t("notes.deleteBlock");
    }
  }

  function colorLabel(color: NotesColor): string {
    switch (color) {
      case "default":
        return t("notes.blockColor.default");
      case "gray":
        return t("notes.blockColor.gray");
      case "brown":
        return t("notes.blockColor.brown");
      case "orange":
        return t("notes.blockColor.orange");
      case "yellow":
        return t("notes.blockColor.yellow");
      case "green":
        return t("notes.blockColor.green");
      case "blue":
        return t("notes.blockColor.blue");
      case "purple":
        return t("notes.blockColor.purple");
      case "pink":
        return t("notes.blockColor.pink");
      case "red":
        return t("notes.blockColor.red");
      case "gray_background":
        return t("notes.blockColor.grayBackground");
      case "brown_background":
        return t("notes.blockColor.brownBackground");
      case "orange_background":
        return t("notes.blockColor.orangeBackground");
      case "yellow_background":
        return t("notes.blockColor.yellowBackground");
      case "green_background":
        return t("notes.blockColor.greenBackground");
      case "blue_background":
        return t("notes.blockColor.blueBackground");
      case "purple_background":
        return t("notes.blockColor.purpleBackground");
      case "pink_background":
        return t("notes.blockColor.pinkBackground");
      case "red_background":
        return t("notes.blockColor.redBackground");
    }
  }

  function commandIcon(command: NotesSlashCommand): Component | null {
    switch (command.kind) {
      case "block":
        return blockIcon(command.blockType);
      case "toggle_heading":
        return toggleHeadingIcon(command.headingType);
      case "action":
        return actionIcon(command.action);
      case "color":
        return null;
    }
  }

  function blockIcon(type: NotesInsertableBlockType): Component {
    switch (type) {
      case "paragraph":
        return Pilcrow;
      case "heading_1":
        return Heading1;
      case "heading_2":
        return Heading2;
      case "heading_3":
        return Heading3;
      case "heading_4":
        return Heading4;
      case "bulleted_list_item":
        return List;
      case "numbered_list_item":
        return ListOrdered;
      case "to_do":
        return SquareCheck;
      case "toggle":
        return ListCollapse;
      case "callout":
        return MessageSquareWarning;
      case "quote":
        return Quote;
      case "child_page":
        return FileText;
      case "breadcrumb":
        return Route;
      case "table_of_contents":
        return ListTree;
      case "column_list":
        return Columns2;
      case "table":
        return Table2;
      case "tab":
        return Columns2;
      case "image":
        return ImageIcon;
      case "video":
        return Video;
      case "audio":
        return Music;
      case "file":
        return FileIcon;
      case "pdf":
        return FileText;
      case "bookmark":
        return Bookmark;
      case "link_preview":
        return LinkIcon;
      case "template":
        return FileText;
      case "button":
        return MousePointerClick;
      case "embed":
        return ExternalLink;
      case "equation":
        return Sigma;
      case "divider":
        return Minus;
      case "code":
        return Code;
    }
  }

  function toggleHeadingIcon(type: NotesHeadingBlockType): Component {
    switch (type) {
      case "heading_1":
        return Heading1;
      case "heading_2":
        return Heading2;
      case "heading_3":
        return Heading3;
      case "heading_4":
        return Heading4;
    }
  }

  function actionIcon(action: NotesSlashAction): Component {
    switch (action) {
      case "copy_link":
        return LinkIcon;
      case "duplicate":
        return Copy;
      case "move_up":
        return ArrowUp;
      case "move_down":
        return ArrowDown;
      case "delete":
        return Trash2;
    }
  }
</script>

<div
  id={menuId}
  class={`${menuClass} z-30 max-h-[min(28rem,70vh)] w-64 overflow-auto rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg`}
  role="menu"
  aria-label={t("notes.slashMenu")}
  aria-live="polite"
  data-app-floating-surface
  tabindex="-1"
  onmousedown={(event) => event.preventDefault()}
>
  {#if hasResults}
    {#each sectionOrder as section}
      {@const items = sections[section]}
      {#if items.length > 0}
        <div class="px-2.5 pb-1 pt-2 text-[0.7rem] font-medium text-muted-foreground">
          {sectionLabel(section)}
        </div>
        {#each items as item (item.key)}
          {@const Icon = commandIcon(item.command)}
          <button
            class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] outline-none hover:bg-accent hover:text-accent-foreground focus-visible:bg-accent focus-visible:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring"
            type="button"
            role={item.command.kind === "color" ? "menuitemradio" : "menuitem"}
            aria-checked={item.command.kind === "color"
              ? currentColor === item.command.color
              : undefined}
            onmousedown={(event) => event.preventDefault()}
            onclick={() => selectCommand(item)}
          >
            {#if item.command.kind === "color"}
              <span
                class="notes-color-swatch"
                style={notesBlockColorSwatchStyle(item.command.color)}
                aria-hidden="true"
              >
                A
              </span>
            {:else if Icon}
              <Icon class="size-4 shrink-0" aria-hidden="true" />
            {/if}
            <span class="min-w-0 flex-1 truncate">{commandLabel(item.command)}</span>
            {#if item.command.kind === "color" && currentColor === item.command.color}
              <Check class="size-3.5 shrink-0" aria-hidden="true" />
            {/if}
          </button>
        {/each}
      {/if}
    {/each}
  {:else}
    <div class="px-2.5 py-2 text-[0.8rem] text-muted-foreground" role="status">
      {t("notes.slashNoResults")}
    </div>
  {/if}
</div>

<style>
  .notes-color-swatch {
    display: inline-flex;
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--notes-color-swatch-border);
    border-radius: 0.25rem;
    background: var(--notes-color-swatch-bg);
    color: var(--notes-color-swatch-fg);
    font-size: 0.65rem;
    font-weight: 600;
    line-height: 1;
  }
</style>
