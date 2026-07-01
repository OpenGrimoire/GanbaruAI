<script lang="ts">
  import { getLocalization } from "$lib/i18n/translator.svelte";
  import {
    notesInsertableBlockTypes,
    type NotesInsertableBlockType,
  } from "$lib/notes/block-insertion";
  import type { Component } from "svelte";
  import Pilcrow from "@lucide/svelte/icons/pilcrow";
  import Heading1 from "@lucide/svelte/icons/heading-1";
  import Heading2 from "@lucide/svelte/icons/heading-2";
  import Heading3 from "@lucide/svelte/icons/heading-3";
  import Heading4 from "@lucide/svelte/icons/heading-4";
  import List from "@lucide/svelte/icons/list";
  import ListOrdered from "@lucide/svelte/icons/list-ordered";
  import SquareCheck from "@lucide/svelte/icons/square-check";
  import ListCollapse from "@lucide/svelte/icons/list-collapse";
  import MessageSquareWarning from "@lucide/svelte/icons/message-square-warning";
  import Quote from "@lucide/svelte/icons/quote";
  import FileText from "@lucide/svelte/icons/file-text";
  import Route from "@lucide/svelte/icons/route";
  import ListTree from "@lucide/svelte/icons/list-tree";
  import Table2 from "@lucide/svelte/icons/table-2";
  import Columns2 from "@lucide/svelte/icons/columns-2";
  import ImageIcon from "@lucide/svelte/icons/image";
  import Video from "@lucide/svelte/icons/video";
  import Music from "@lucide/svelte/icons/music";
  import FileIcon from "@lucide/svelte/icons/file";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MousePointerClick from "@lucide/svelte/icons/mouse-pointer-click";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Sigma from "@lucide/svelte/icons/sigma";
  import Minus from "@lucide/svelte/icons/minus";
  import Code from "@lucide/svelte/icons/code";

  let {
    menuClass,
    onSelect,
  }: {
    menuClass: string;
    onSelect: (type: NotesInsertableBlockType) => void;
  } = $props();

  const { t } = getLocalization();
  const commands = notesInsertableBlockTypes();

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
</script>

<div
  class={`${menuClass} z-30 max-h-[min(28rem,70vh)] w-56 overflow-auto rounded-md border border-border bg-popover py-1 text-popover-foreground shadow-lg`}
  role="menu"
  data-app-floating-surface
  tabindex="-1"
  onmousedown={(event) => event.preventDefault()}
>
  {#each commands as command}
    {@const Icon = blockIcon(command)}
    <button
      class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-[0.8rem] hover:bg-accent hover:text-accent-foreground"
      type="button"
      role="menuitem"
      onmousedown={(event) => event.preventDefault()}
      onclick={() => onSelect(command)}
    >
      <Icon class="size-4 shrink-0" />
      <span class="min-w-0 truncate">{blockLabel(command)}</span>
    </button>
  {/each}
</div>
