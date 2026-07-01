import type {
  NotesBlock,
  NotesBlockType,
  NotesBlockUpdate,
  NotesBlockWrite,
  NotesBookmarkBlockPayload,
  NotesButtonBlockPayload,
  NotesCalloutBlockPayload,
  NotesChildDatabaseBlockPayload,
  NotesChildPageBlockPayload,
  NotesCodeBlockPayload,
  NotesColumnBlockPayload,
  NotesColor,
  NotesDateMentionValue,
  NotesEmbedBlockPayload,
  NotesEquationBlockPayload,
  NotesLinkPreviewBlockPayload,
  NotesMediaBlockPayload,
  NotesRichText,
  NotesSyncedBlockPayload,
  NotesTabBlockPayload,
  NotesTableBlockPayload,
  NotesTableRowBlock,
  NotesTableRowBlockPayload,
  NotesTemplateBlockPayload,
  NotesTextBlockPayload,
  NotesToggleBlockPayload,
  NotesTodoBlockPayload,
  NotesUnsupportedBlockPayload,
} from "./types";
import { blockColor, canBlockHaveColor } from "./block-color";
import {
  applyRichTextAnnotations,
  applyRichTextLink,
  createTextRichText,
  insertDateMentionRichText,
  insertEquationRichText,
  insertPageMentionRichText,
  richTextAnnotationsForSelection,
  richTextHasVisibleFormatting,
  richTextLinkRangeForSelection,
  replacePlainTextPreservingRichText,
  richTextPlainText,
  type NotesRichTextAnnotationPatch,
  type NotesRichTextAnnotationRange,
  type NotesRichTextLinkRange,
} from "./rich-text";
import { unsupportedBlockPlainText } from "./unsupported";

export { richTextPlainText } from "./rich-text";

const DEFAULT_COLOR: NotesColor = "default";
const DEFAULT_CALLOUT_COLOR: NotesColor = "gray_background";
export const DEFAULT_CODE_LANGUAGE = "plain text";
export const DEFAULT_TABLE_WIDTH = 2;
export const DEFAULT_TABLE_ROW_COUNT = 2;

export type NotesHeadingBlockType = "heading_1" | "heading_2" | "heading_3" | "heading_4";

export interface NotesTextPayloadOptions {
  color?: NotesColor;
  isToggleable?: boolean;
  open?: boolean;
}

/** Create the minimum public Notion text rich text object Ganbaru edits today. */
export function createRichText(content: string): NotesRichText {
  return createTextRichText(content);
}

export function createTextPayload(
  content: string,
  colorOrOptions: NotesColor | NotesTextPayloadOptions = DEFAULT_COLOR,
): NotesTextBlockPayload {
  const options = typeof colorOrOptions === "string"
    ? { color: colorOrOptions }
    : colorOrOptions;
  return {
    rich_text: [createRichText(content)],
    color: options.color ?? DEFAULT_COLOR,
    ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
    ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
  };
}

export function createTodoPayload(
  content: string,
  checked = false,
  color: NotesColor = DEFAULT_COLOR,
): NotesTodoBlockPayload {
  return {
    ...createTextPayload(content, color),
    checked,
  };
}

export function createTogglePayload(
  content: string,
  open = true,
  color: NotesColor = DEFAULT_COLOR,
): NotesToggleBlockPayload {
  return {
    ...createTextPayload(content, color),
    ganbaru_open: open,
  };
}

export function createCalloutPayload(
  content: string,
  color: NotesColor = DEFAULT_CALLOUT_COLOR,
): NotesCalloutBlockPayload {
  return {
    ...createTextPayload(content, color),
    icon: { type: "icon", icon: { name: "info", color: "gray" } },
  };
}

export function createCodePayload(
  content: string,
  language = DEFAULT_CODE_LANGUAGE,
): NotesCodeBlockPayload {
  return {
    rich_text: [createRichText(content)],
    caption: [],
    language,
  };
}

export function createChildPagePayload(title: string): NotesChildPageBlockPayload {
  return { title };
}

export function createChildDatabasePayload(title: string): NotesChildDatabaseBlockPayload {
  return { title };
}

export function createColumnPayload(widthRatio?: number): NotesColumnBlockPayload {
  if (widthRatio === undefined) return {};
  if (!Number.isFinite(widthRatio) || widthRatio <= 0) return {};
  return { width_ratio: Math.min(1, widthRatio) };
}

export function createTablePayload(
  tableWidth = DEFAULT_TABLE_WIDTH,
  hasColumnHeader = false,
  hasRowHeader = false,
): NotesTableBlockPayload {
  return {
    table_width: Math.max(1, Math.trunc(tableWidth)),
    has_column_header: hasColumnHeader,
    has_row_header: hasRowHeader,
  };
}

export function createTabPayload(): NotesTabBlockPayload {
  return {};
}

export function createTableCell(content: string): NotesRichText[] {
  return content.length > 0 ? [createRichText(content)] : [];
}

export function createTableRowPayload(
  cells: readonly (string | readonly NotesRichText[])[],
): NotesTableRowBlockPayload {
  return {
    cells: cells.map((cell) => (typeof cell === "string" ? createTableCell(cell) : [...cell])),
  };
}

export function createEmptyTableRowPayload(
  width = DEFAULT_TABLE_WIDTH,
): NotesTableRowBlockPayload {
  return createTableRowPayload(Array.from({ length: Math.max(1, Math.trunc(width)) }, () => ""));
}

export function createBookmarkPayload(
  url: string,
  caption = "",
): NotesBookmarkBlockPayload {
  const trimmedCaption = caption.trim();
  return {
    caption: trimmedCaption ? [createRichText(trimmedCaption)] : [],
    url,
  };
}

export function createLinkPreviewPayload(url: string): NotesLinkPreviewBlockPayload {
  return { url };
}

/** Create an original synced block or a duplicate reference to another block. */
export function createSyncedBlockPayload(
  syncedFromBlockId?: string | null,
): NotesSyncedBlockPayload {
  const blockId = syncedFromBlockId?.trim();
  if (!blockId) return { synced_from: null };
  return {
    synced_from: {
      type: "block_id",
      block_id: blockId,
    },
  };
}

export function createTemplatePayload(content: string): NotesTemplateBlockPayload {
  return {
    rich_text: [createRichText(content)],
  };
}

export function createButtonPayload(content: string): NotesButtonBlockPayload {
  return {
    rich_text: [createRichText(content.trim() || "Button")],
    icon: { type: "icon", icon: { name: "mouse-pointer-click", color: "gray" } },
    actions: [
      {
        type: "insert_blocks",
        source: "children",
        position: "below_button",
      },
    ],
  };
}

export function createEmbedPayload(url: string): NotesEmbedBlockPayload {
  return { url };
}

export function createEquationPayload(expression: string): NotesEquationBlockPayload {
  return { expression };
}

export function createMediaPayload(
  url = "",
  caption = "",
  name?: string,
): NotesMediaBlockPayload {
  const trimmedCaption = caption.trim();
  return {
    type: "external",
    external: { url: url.trim() },
    caption: trimmedCaption ? [createRichText(trimmedCaption)] : [],
    ...(name?.trim() ? { name: name.trim() } : {}),
  };
}

export function createUnsupportedPayload(blockType = "unsupported"): NotesUnsupportedBlockPayload {
  return { block_type: blockType.trim() || "unsupported" };
}

/** Build a Notion-shaped block create payload for the given type. */
export function createBlockWrite(
  id: string,
  type: NotesBlockType = "paragraph",
  content = "",
  color: NotesColor = DEFAULT_COLOR,
): NotesBlockWrite {
  switch (type) {
    case "paragraph":
      return { id, type, paragraph: createTextPayload(content, color) };
    case "heading_1":
      return { id, type, heading_1: createTextPayload(content, color) };
    case "heading_2":
      return { id, type, heading_2: createTextPayload(content, color) };
    case "heading_3":
      return { id, type, heading_3: createTextPayload(content, color) };
    case "heading_4":
      return { id, type, heading_4: createTextPayload(content, color) };
    case "bulleted_list_item":
      return { id, type, bulleted_list_item: createTextPayload(content, color) };
    case "numbered_list_item":
      return { id, type, numbered_list_item: createTextPayload(content, color) };
    case "to_do":
      return { id, type, to_do: createTodoPayload(content, false, color) };
    case "toggle":
      return { id, type, toggle: createTogglePayload(content, true, color) };
    case "callout":
      return {
        id,
        type,
        callout: createCalloutPayload(
          content,
          color === DEFAULT_COLOR ? DEFAULT_CALLOUT_COLOR : color,
        ),
      };
    case "quote":
      return { id, type, quote: createTextPayload(content, color) };
    case "child_page":
      return { id, type, child_page: createChildPagePayload(content.trim()) };
    case "child_database":
      return { id, type, child_database: createChildDatabasePayload(content.trim()) };
    case "breadcrumb":
      return { id, type, breadcrumb: {} };
    case "table_of_contents":
      return { id, type, table_of_contents: { color } };
    case "column_list":
      return { id, type, column_list: {} };
    case "column":
      return { id, type, column: createColumnPayload() };
    case "table":
      return { id, type, table: createTablePayload() };
    case "table_row":
      return { id, type, table_row: createEmptyTableRowPayload() };
    case "tab":
      return { id, type, tab: createTabPayload() };
    case "image":
      return { id, type, image: createMediaPayload(content.trim()) };
    case "video":
      return { id, type, video: createMediaPayload(content.trim()) };
    case "audio":
      return { id, type, audio: createMediaPayload(content.trim()) };
    case "file":
      return { id, type, file: createMediaPayload(content.trim()) };
    case "pdf":
      return { id, type, pdf: createMediaPayload(content.trim()) };
    case "bookmark":
      return { id, type, bookmark: createBookmarkPayload(content.trim()) };
    case "link_preview":
      return { id, type, link_preview: createLinkPreviewPayload(content.trim()) };
    case "synced_block":
      return { id, type, synced_block: createSyncedBlockPayload() };
    case "template":
      return { id, type, template: createTemplatePayload(content) };
    case "button":
      return { id, type, button: createButtonPayload(content) };
    case "embed":
      return { id, type, embed: createEmbedPayload(content.trim()) };
    case "equation":
      return { id, type, equation: createEquationPayload(content) };
    case "divider":
      return { id, type, divider: {} };
    case "code":
      return { id, type, code: createCodePayload(content) };
    case "unsupported":
      return { id, type, unsupported: createUnsupportedPayload() };
  }
}

export function createBlockUpdate(
  type: NotesBlockType = "paragraph",
  content = "",
  color: NotesColor = DEFAULT_COLOR,
  options: NotesTextPayloadOptions = {},
): NotesBlockUpdate {
  const write = createBlockWrite(
    "00000000-0000-4000-8000-000000000000",
    type,
    content,
    color,
  );
  const { id: _id, ...update } = write;
  if (type === "heading_1" && update.type === "heading_1") {
    return {
      ...update,
      heading_1: {
        ...update.heading_1,
        ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
        ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
      },
    };
  }
  if (type === "heading_2" && update.type === "heading_2") {
    return {
      ...update,
      heading_2: {
        ...update.heading_2,
        ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
        ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
      },
    };
  }
  if (type === "heading_3" && update.type === "heading_3") {
    return {
      ...update,
      heading_3: {
        ...update.heading_3,
        ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
        ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
      },
    };
  }
  if (type === "heading_4" && update.type === "heading_4") {
    return {
      ...update,
      heading_4: {
        ...update.heading_4,
        ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
        ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
      },
    };
  }
  return update;
}

export function blockPlainText(block: NotesBlock): string {
  switch (block.type) {
    case "paragraph":
      return richTextPlainText(block.paragraph.rich_text);
    case "heading_1":
      return richTextPlainText(block.heading_1.rich_text);
    case "heading_2":
      return richTextPlainText(block.heading_2.rich_text);
    case "heading_3":
      return richTextPlainText(block.heading_3.rich_text);
    case "heading_4":
      return richTextPlainText(block.heading_4.rich_text);
    case "bulleted_list_item":
      return richTextPlainText(block.bulleted_list_item.rich_text);
    case "numbered_list_item":
      return richTextPlainText(block.numbered_list_item.rich_text);
    case "to_do":
      return richTextPlainText(block.to_do.rich_text);
    case "toggle":
      return richTextPlainText(block.toggle.rich_text);
    case "callout":
      return richTextPlainText(block.callout.rich_text);
    case "quote":
      return richTextPlainText(block.quote.rich_text);
    case "child_page":
      return block.child_page.title;
    case "child_database":
      return block.child_database.title;
    case "code":
      return richTextPlainText(block.code.rich_text);
    case "bookmark":
      return richTextPlainText(block.bookmark.caption) || block.bookmark.url;
    case "link_preview":
      return block.link_preview.url;
    case "synced_block":
      return syncedBlockPlainText(block.synced_block);
    case "template":
      return richTextPlainText(block.template.rich_text);
    case "button":
      return richTextPlainText(block.button.rich_text);
    case "tab":
      return "";
    case "embed":
      return block.embed.url;
    case "equation":
      return block.equation.expression;
    case "table_row":
      return tableRowPlainText(block.table_row);
    case "image":
      return mediaPayloadPlainText(block.image);
    case "video":
      return mediaPayloadPlainText(block.video);
    case "audio":
      return mediaPayloadPlainText(block.audio);
    case "file":
      return mediaPayloadPlainText(block.file);
    case "pdf":
      return mediaPayloadPlainText(block.pdf);
    case "breadcrumb":
    case "table_of_contents":
    case "column_list":
    case "column":
    case "table":
    case "divider":
      return "";
    case "unsupported":
      return unsupportedBlockPlainText(block.unsupported);
  }
}

export function tableCellPlainText(cell: readonly NotesRichText[]): string {
  return richTextPlainText(cell);
}

export function tableRowPlainText(row: NotesTableRowBlockPayload): string {
  return row.cells.map(tableCellPlainText).join("\t");
}

function mediaPayloadPlainText(payload: NotesMediaBlockPayload): string {
  const caption = payload.caption ? richTextPlainText(payload.caption) : "";
  const source = payload.type === "external"
    ? payload.external.url
    : payload.type === "file"
      ? payload.file.url
      : payload.file_upload.id;
  return [caption, payload.name ?? "", source]
    .map((part) => part.trim())
    .filter(Boolean)
    .join(" ");
}

function syncedBlockPlainText(payload: NotesSyncedBlockPayload): string {
  return payload.synced_from?.block_id ?? "Synced block";
}

export function isTextEditableBlock(type: NotesBlockType): boolean {
  return (
    type !== "divider"
    && type !== "child_page"
    && type !== "child_database"
    && type !== "breadcrumb"
    && type !== "table_of_contents"
    && type !== "column_list"
    && type !== "column"
    && type !== "table"
    && type !== "table_row"
    && type !== "tab"
    && type !== "image"
    && type !== "video"
    && type !== "audio"
    && type !== "file"
    && type !== "pdf"
    && type !== "bookmark"
    && type !== "link_preview"
    && type !== "synced_block"
    && type !== "embed"
    && type !== "equation"
    && type !== "unsupported"
  );
}

export function isMergeableTextBlock(type: NotesBlockType): boolean {
  return isTextEditableBlock(type);
}

export function createTextPayloadFromRichText(
  richText: readonly NotesRichText[],
  color: NotesColor = DEFAULT_COLOR,
  options: Pick<NotesTextPayloadOptions, "isToggleable" | "open"> & {
    icon?: NotesTextBlockPayload["icon"];
  } = {},
): NotesTextBlockPayload {
  return {
    rich_text: richText.length > 0 ? [...richText] : [createRichText("")],
    color,
    ...(options.isToggleable === undefined ? {} : { is_toggleable: options.isToggleable }),
    ...(options.open === undefined ? {} : { ganbaru_open: options.open }),
    ...(options.icon === undefined ? {} : { icon: options.icon }),
  };
}

function blockRichText(block: NotesBlock): NotesRichText[] {
  switch (block.type) {
    case "paragraph":
      return block.paragraph.rich_text;
    case "heading_1":
      return block.heading_1.rich_text;
    case "heading_2":
      return block.heading_2.rich_text;
    case "heading_3":
      return block.heading_3.rich_text;
    case "heading_4":
      return block.heading_4.rich_text;
    case "bulleted_list_item":
      return block.bulleted_list_item.rich_text;
    case "numbered_list_item":
      return block.numbered_list_item.rich_text;
    case "to_do":
      return block.to_do.rich_text;
    case "toggle":
      return block.toggle.rich_text;
    case "callout":
      return block.callout.rich_text;
    case "quote":
      return block.quote.rich_text;
    case "code":
      return block.code.rich_text;
    case "template":
      return block.template.rich_text;
    case "button":
      return block.button.rich_text;
    default:
      return [createRichText(blockPlainText(block))];
  }
}

export function blockEditableRichText(block: NotesBlock): NotesRichText[] {
  return blockRichText(block);
}

export function blockHasVisibleRichTextFormatting(block: NotesBlock): boolean {
  return richTextHasVisibleFormatting(blockRichText(block));
}

export function isHeadingBlockType(type: NotesBlockType): type is NotesHeadingBlockType {
  return type === "heading_1"
    || type === "heading_2"
    || type === "heading_3"
    || type === "heading_4";
}

function headingPayload(block: NotesBlock): NotesTextBlockPayload | null {
  if (block.type === "heading_1") return block.heading_1;
  if (block.type === "heading_2") return block.heading_2;
  if (block.type === "heading_3") return block.heading_3;
  if (block.type === "heading_4") return block.heading_4;
  return null;
}

export function headingIsToggleable(block: NotesBlock): boolean {
  return headingPayload(block)?.is_toggleable === true;
}

export function headingToggleOpen(block: NotesBlock): boolean {
  const payload = headingPayload(block);
  return payload?.is_toggleable !== true || payload.ganbaru_open !== false;
}

export function canBlockHaveChildren(blockOrType: NotesBlock | NotesBlockType): boolean {
  if (typeof blockOrType !== "string") {
    if (isHeadingBlockType(blockOrType.type)) return headingIsToggleable(blockOrType);
    if (blockOrType.type === "synced_block") {
      return blockOrType.synced_block.synced_from === null;
    }
    return canBlockHaveChildren(blockOrType.type);
  }
  const type = blockOrType;
  return [
    "paragraph",
    "bulleted_list_item",
    "numbered_list_item",
    "to_do",
    "toggle",
    "callout",
    "quote",
    "child_database",
    "column_list",
    "column",
    "table",
    "tab",
    "template",
    "button",
  ].includes(type);
}

/** Convert a loaded block to a Notion-shaped update payload while replacing its rich text. */
export function blockWithRichText(
  block: NotesBlock,
  richText: readonly NotesRichText[],
): NotesBlockUpdate {
  switch (block.type) {
    case "paragraph":
      return {
        type: block.type,
        paragraph: createTextPayloadFromRichText(richText, blockColor(block), {
          icon: block.paragraph.icon,
        }),
      };
    case "heading_1":
      return {
        type: block.type,
        heading_1: createTextPayloadFromRichText(richText, blockColor(block), {
          isToggleable: block.heading_1.is_toggleable,
          open: block.heading_1.ganbaru_open,
        }),
      };
    case "heading_2":
      return {
        type: block.type,
        heading_2: createTextPayloadFromRichText(richText, blockColor(block), {
          isToggleable: block.heading_2.is_toggleable,
          open: block.heading_2.ganbaru_open,
        }),
      };
    case "heading_3":
      return {
        type: block.type,
        heading_3: createTextPayloadFromRichText(richText, blockColor(block), {
          isToggleable: block.heading_3.is_toggleable,
          open: block.heading_3.ganbaru_open,
        }),
      };
    case "heading_4":
      return {
        type: block.type,
        heading_4: createTextPayloadFromRichText(richText, blockColor(block), {
          isToggleable: block.heading_4.is_toggleable,
          open: block.heading_4.ganbaru_open,
        }),
      };
    case "bulleted_list_item":
      return {
        type: block.type,
        bulleted_list_item: createTextPayloadFromRichText(richText, blockColor(block)),
      };
    case "numbered_list_item":
      return {
        type: block.type,
        numbered_list_item: createTextPayloadFromRichText(richText, blockColor(block)),
      };
    case "to_do":
      return {
        type: block.type,
        to_do: {
          ...createTextPayloadFromRichText(richText, blockColor(block)),
          checked: block.to_do.checked,
        },
      };
    case "toggle":
      return {
        type: block.type,
        toggle: {
          ...createTextPayloadFromRichText(richText, blockColor(block)),
          ganbaru_open: block.toggle.ganbaru_open ?? true,
        },
      };
    case "callout":
      return {
        type: block.type,
        callout: {
          ...createTextPayloadFromRichText(richText, blockColor(block)),
          icon: block.callout.icon,
        },
      };
    case "quote":
      return { type: block.type, quote: createTextPayloadFromRichText(richText, blockColor(block)) };
    case "child_page":
      return { type: block.type, child_page: block.child_page };
    case "child_database":
      return { type: block.type, child_database: block.child_database };
    case "breadcrumb":
      return { type: block.type, breadcrumb: block.breadcrumb };
    case "table_of_contents":
      return { type: block.type, table_of_contents: block.table_of_contents };
    case "column_list":
      return { type: block.type, column_list: block.column_list };
    case "column":
      return { type: block.type, column: block.column };
    case "table":
      return { type: block.type, table: block.table };
    case "table_row":
      return { type: block.type, table_row: block.table_row };
    case "tab":
      return { type: block.type, tab: block.tab };
    case "image":
      return { type: block.type, image: block.image };
    case "video":
      return { type: block.type, video: block.video };
    case "audio":
      return { type: block.type, audio: block.audio };
    case "file":
      return { type: block.type, file: block.file };
    case "pdf":
      return { type: block.type, pdf: block.pdf };
    case "bookmark":
      return { type: block.type, bookmark: block.bookmark };
    case "link_preview":
      return { type: block.type, link_preview: block.link_preview };
    case "synced_block":
      return { type: block.type, synced_block: block.synced_block };
    case "template":
      return {
        type: block.type,
        template: {
          rich_text: richText.length > 0 ? [...richText] : [createRichText("")],
        },
      };
    case "button":
      return {
        type: block.type,
        button: {
          ...block.button,
          rich_text: richText.length > 0 ? [...richText] : [createRichText("")],
        },
      };
    case "embed":
      return { type: block.type, embed: block.embed };
    case "equation":
      return { type: block.type, equation: block.equation };
    case "code":
      return {
        type: block.type,
        code: {
          ...block.code,
          rich_text: [...richText],
        },
      };
    case "divider":
      return { type: block.type, divider: {} };
    case "unsupported":
      return { type: block.type, unsupported: block.unsupported };
  }
}

/** Convert a loaded block to a Notion-shaped update payload while replacing text. */
export function blockWithText(block: NotesBlock, text: string): NotesBlockUpdate {
  return blockWithRichText(block, replacePlainTextPreservingRichText(blockRichText(block), text));
}

export function blockWithPageMention(
  block: NotesBlock,
  start: number,
  end: number,
  pageId: string,
  title: string,
  href: string | null,
): NotesBlockUpdate {
  return blockWithRichText(
    block,
    insertPageMentionRichText(blockRichText(block), start, end, pageId, title, href),
  );
}

export function blockWithDateMention(
  block: NotesBlock,
  start: number,
  end: number,
  date: NotesDateMentionValue,
  title: string,
): NotesBlockUpdate {
  return blockWithRichText(
    block,
    insertDateMentionRichText(blockRichText(block), start, end, date, title),
  );
}

export function blockWithInlineEquation(
  block: NotesBlock,
  start: number,
  end: number,
  expression: string,
): NotesBlockUpdate {
  return blockWithRichText(
    block,
    insertEquationRichText(blockRichText(block), start, end, expression),
  );
}

export function blockTextLinkRangeForSelection(
  block: NotesBlock,
  selectionStart: number,
  selectionEnd: number,
): NotesRichTextLinkRange {
  return richTextLinkRangeForSelection(blockRichText(block), selectionStart, selectionEnd);
}

export function blockTextAnnotationsForSelection(
  block: NotesBlock,
  selectionStart: number,
  selectionEnd: number,
): NotesRichTextAnnotationRange {
  return richTextAnnotationsForSelection(blockRichText(block), selectionStart, selectionEnd);
}

export function blockWithTextLink(
  block: NotesBlock,
  start: number,
  end: number,
  url: string | null,
): NotesBlockUpdate {
  return blockWithRichText(
    block,
    applyRichTextLink(blockRichText(block), start, end, url),
  );
}

export function blockWithTextAnnotations(
  block: NotesBlock,
  start: number,
  end: number,
  patch: NotesRichTextAnnotationPatch,
): NotesBlockUpdate {
  return blockWithRichText(
    block,
    applyRichTextAnnotations(blockRichText(block), start, end, patch),
  );
}

export function blockWithBookmark(
  block: NotesBlock,
  url: string,
  caption: string,
): NotesBlockUpdate {
  if (block.type !== "bookmark") return blockWithText(block, blockPlainText(block));
  return {
    type: "bookmark",
    bookmark: createBookmarkPayload(url, caption),
  };
}

export function blockWithEmbedUrl(block: NotesBlock, url: string): NotesBlockUpdate {
  if (block.type !== "embed") return blockWithText(block, blockPlainText(block));
  return {
    type: "embed",
    embed: createEmbedPayload(url),
  };
}

export function blockWithLinkPreviewUrl(block: NotesBlock, url: string): NotesBlockUpdate {
  if (block.type !== "link_preview") return blockWithText(block, blockPlainText(block));
  return {
    type: "link_preview",
    link_preview: createLinkPreviewPayload(url),
  };
}

export function blockWithEquationExpression(
  block: NotesBlock,
  expression: string,
): NotesBlockUpdate {
  if (block.type !== "equation") return blockWithText(block, blockPlainText(block));
  return {
    type: "equation",
    equation: createEquationPayload(expression),
  };
}

export function blockWithMedia(
  block: NotesBlock,
  url: string,
  caption: string,
  name?: string,
): NotesBlockUpdate {
  if (block.type === "image") return { type: "image", image: createMediaPayload(url, caption, name) };
  if (block.type === "video") return { type: "video", video: createMediaPayload(url, caption, name) };
  if (block.type === "audio") return { type: "audio", audio: createMediaPayload(url, caption, name) };
  if (block.type === "file") return { type: "file", file: createMediaPayload(url, caption, name) };
  if (block.type === "pdf") return { type: "pdf", pdf: createMediaPayload(url, caption, name) };
  return blockWithText(block, blockPlainText(block));
}

export function blockWithTableCell(
  block: NotesTableRowBlock,
  columnIndex: number,
  text: string,
): NotesBlockUpdate {
  const width = Math.max(block.table_row.cells.length, columnIndex + 1);
  const cells = Array.from({ length: width }, (_, index) => [
    ...(block.table_row.cells[index] ?? []),
  ]);
  cells[columnIndex] = createTableCell(text);
  return {
    type: "table_row",
    table_row: { cells },
  };
}

export function blockWithTodoChecked(block: NotesBlock, checked: boolean): NotesBlockUpdate {
  if (block.type !== "to_do") return blockWithText(block, blockPlainText(block));
  return {
    type: "to_do",
    to_do: createTodoPayload(blockPlainText(block), checked, blockColor(block)),
  };
}

export function blockWithCodeLanguage(block: NotesBlock, language: string): NotesBlockUpdate {
  const content = blockPlainText(block);
  return {
    type: "code",
    code: createCodePayload(content, language.trim() || DEFAULT_CODE_LANGUAGE),
  };
}

export function blockWithToggleOpen(block: NotesBlock, open: boolean): NotesBlockUpdate {
  if (block.type !== "toggle") return blockWithText(block, blockPlainText(block));
  return {
    type: "toggle",
    toggle: {
      ...block.toggle,
      ganbaru_open: open,
    },
  };
}

export function blockWithHeadingToggleable(
  block: NotesBlock,
  headingType: NotesHeadingBlockType,
  isToggleable = true,
): NotesBlockUpdate {
  const text = blockPlainText(block);
  const color = canBlockHaveColor(block.type) ? blockColor(block) : DEFAULT_COLOR;
  return createBlockUpdate(headingType, text, color, {
    isToggleable,
    open: isToggleable ? true : undefined,
  });
}

export function blockWithHeadingToggleOpen(
  block: NotesBlock,
  open: boolean,
): NotesBlockUpdate {
  if (block.type === "heading_1" && block.heading_1.is_toggleable === true) {
    return {
      type: "heading_1",
      heading_1: {
        ...block.heading_1,
        ganbaru_open: open,
      },
    };
  }
  if (block.type === "heading_2" && block.heading_2.is_toggleable === true) {
    return {
      type: "heading_2",
      heading_2: {
        ...block.heading_2,
        ganbaru_open: open,
      },
    };
  }
  if (block.type === "heading_3" && block.heading_3.is_toggleable === true) {
    return {
      type: "heading_3",
      heading_3: {
        ...block.heading_3,
        ganbaru_open: open,
      },
    };
  }
  if (block.type === "heading_4" && block.heading_4.is_toggleable === true) {
    return {
      type: "heading_4",
      heading_4: {
        ...block.heading_4,
        ganbaru_open: open,
      },
    };
  }
  return blockWithText(block, blockPlainText(block));
}

export function blockConvertedToType(block: NotesBlock, type: NotesBlockType): NotesBlockUpdate {
  const text = type === "divider" ? "" : blockPlainText(block);
  const color = canBlockHaveColor(type) ? blockColor(block) : DEFAULT_COLOR;
  return createBlockUpdate(type, text, color);
}

export function applyBlockUpdate(block: NotesBlock, update: NotesBlockUpdate): NotesBlock {
  const base = {
    object: "block" as const,
    id: block.id,
    parent: block.parent,
    created_time: block.created_time,
    last_edited_time: new Date().toISOString(),
    has_children: block.has_children,
    in_trash: block.in_trash,
    archived: block.archived,
    source_provider: block.source_provider,
    source_object_id: block.source_object_id,
    source_last_edited_time: block.source_last_edited_time,
  };
  switch (update.type) {
    case "paragraph":
      return { ...base, type: update.type, paragraph: update.paragraph };
    case "heading_1":
      return { ...base, type: update.type, heading_1: update.heading_1 };
    case "heading_2":
      return { ...base, type: update.type, heading_2: update.heading_2 };
    case "heading_3":
      return { ...base, type: update.type, heading_3: update.heading_3 };
    case "heading_4":
      return { ...base, type: update.type, heading_4: update.heading_4 };
    case "bulleted_list_item":
      return { ...base, type: update.type, bulleted_list_item: update.bulleted_list_item };
    case "numbered_list_item":
      return { ...base, type: update.type, numbered_list_item: update.numbered_list_item };
    case "to_do":
      return { ...base, type: update.type, to_do: update.to_do };
    case "toggle":
      return { ...base, type: update.type, toggle: update.toggle };
    case "callout":
      return { ...base, type: update.type, callout: update.callout };
    case "quote":
      return { ...base, type: update.type, quote: update.quote };
    case "child_page":
      return { ...base, type: update.type, child_page: update.child_page };
    case "child_database":
      return { ...base, type: update.type, child_database: update.child_database };
    case "breadcrumb":
      return { ...base, type: update.type, breadcrumb: update.breadcrumb };
    case "table_of_contents":
      return { ...base, type: update.type, table_of_contents: update.table_of_contents };
    case "column_list":
      return { ...base, type: update.type, column_list: update.column_list };
    case "column":
      return { ...base, type: update.type, column: update.column };
    case "table":
      return { ...base, type: update.type, table: update.table };
    case "table_row":
      return { ...base, type: update.type, table_row: update.table_row };
    case "image":
      return { ...base, type: update.type, image: update.image };
    case "video":
      return { ...base, type: update.type, video: update.video };
    case "audio":
      return { ...base, type: update.type, audio: update.audio };
    case "file":
      return { ...base, type: update.type, file: update.file };
    case "pdf":
      return { ...base, type: update.type, pdf: update.pdf };
    case "bookmark":
      return { ...base, type: update.type, bookmark: update.bookmark };
    case "link_preview":
      return { ...base, type: update.type, link_preview: update.link_preview };
    case "synced_block":
      return { ...base, type: update.type, synced_block: update.synced_block };
    case "template":
      return { ...base, type: update.type, template: update.template };
    case "button":
      return { ...base, type: update.type, button: update.button };
    case "tab":
      return { ...base, type: update.type, tab: update.tab };
    case "embed":
      return { ...base, type: update.type, embed: update.embed };
    case "equation":
      return { ...base, type: update.type, equation: update.equation };
    case "divider":
      return { ...base, type: update.type, divider: update.divider };
    case "code":
      return { ...base, type: update.type, code: update.code };
    case "unsupported":
      return { ...base, type: update.type, unsupported: update.unsupported };
  }
}
