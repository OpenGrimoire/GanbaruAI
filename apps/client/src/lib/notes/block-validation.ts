import {
  NOTES_COLORS,
  NOTES_BUTTON_INSERT_POSITIONS,
  NOTES_DATABASE_VIEW_TYPES,
  NOTES_ICON_COLORS,
  NOTES_LOCAL_OBJECT_MENTION_TYPES,
  NOTES_MENTION_NOTIFICATION_KINDS,
  NOTES_MENTION_NOTIFICATION_STATUSES,
  NOTES_MENTION_NOTIFICATION_TARGET_TYPES,
  type NotesBacklink,
  type NotesBacklinkReferenceType,
  type NotesBlock,
  type NotesBlockType,
  type NotesBookmarkBlockPayload,
  type NotesButtonAction,
  type NotesButtonBlockPayload,
  type NotesButtonInsertPosition,
  type NotesCalloutBlockPayload,
  type NotesCalloutIcon,
  type NotesChildDatabaseBlockPayload,
  type NotesChildPageBlockPayload,
  type NotesCodeBlockPayload,
  type NotesComment,
  type NotesCommentAnchor,
  type NotesCommentDisplayName,
  type NotesCommentParent,
  type NotesCommentThread,
  type NotesCommentThreadStatus,
  type NotesColumnBlockPayload,
  type NotesColor,
  type NotesDateMentionReminder,
  type NotesDateMentionValue,
  type NotesCreatedDatabase,
  type NotesDatabase,
  type NotesDatabaseCalendarRowOpenMode,
  type NotesDatabaseGalleryCardSize,
  type NotesDatabaseGalleryCoverSource,
  type NotesDatabaseGalleryRowOpenMode,
  type NotesDatabaseListRowOpenMode,
  type NotesDatabaseBoardRowOpenMode,
  type NotesDatabaseDataSourceSummary,
  type NotesDatabaseTableFilterCondition,
  type NotesDatabaseTableRowOpenMode,
  type NotesDatabaseTableSortDirection,
  type NotesDatabaseTimelineRowOpenMode,
  type NotesDatabaseView,
  type NotesDatabaseViewType,
  type NotesDataSource,
  type NotesDataSourceTemplate,
  type NotesDataSourceBoardGroup,
  type NotesDataSourceBoardView,
  type NotesDataSourceCalendarView,
  type NotesDataSourceCsvExportDiagnostic,
  type NotesDataSourceCsvExportDiagnosticSeverity,
  type NotesDataSourceCsvExportResult,
  type NotesDataSourceCsvExportSaveResult,
  type NotesDataSourceCsvExportScope,
  type NotesDataSourceCsvImportColumn,
  type NotesDataSourceCsvImportDiagnostic,
  type NotesDataSourceCsvImportDiagnosticSeverity,
  type NotesDataSourceCsvImportResult,
  type NotesDataSourceCsvImportRow,
  type NotesDataSourceGalleryView,
  type NotesDataSourceListView,
  type NotesDataSourceSchema,
  type NotesDataSourceTableView,
  type NotesDataSourceTimelineView,
  type NotesEmbedBlockPayload,
  type NotesEquationBlockPayload,
  type NotesLinkPreviewBlockPayload,
  type NotesLocalUser,
  type NotesLocalObjectMentionType,
  type NotesMediaBlockPayload,
  type NotesIconColor,
  type NotesHtmlArchiveSaveResult,
  type NotesHtmlExportAsset,
  type NotesHtmlExportDiagnostic,
  type NotesHtmlExportDiagnosticSeverity,
  type NotesHtmlExportFile,
  type NotesHtmlExportResult,
  type NotesHtmlImportDiagnostic,
  type NotesHtmlImportDiagnosticSeverity,
  type NotesHtmlImportResult,
  type NotesLoadedPage,
  type NotesNotionApiImportedObject,
  type NotesNotionApiImportedUser,
  type NotesNotionApiImportDiagnostic,
  type NotesNotionApiImportDiagnosticSeverity,
  type NotesNotionApiImportResult,
  type NotesNotionExportImportDiagnostic,
  type NotesNotionExportImportDiagnosticSeverity,
  type NotesNotionExportImportResult,
  type NotesMarkdownExportDiagnostic,
  type NotesMarkdownExportDiagnosticSeverity,
  type NotesMarkdownExportResult,
  type NotesMarkdownImportDiagnostic,
  type NotesMarkdownImportDiagnosticSeverity,
  type NotesMarkdownImportResult,
  type NotesMentionNotification,
  type NotesMentionNotificationKind,
  type NotesMentionNotificationStatus,
  type NotesMentionNotificationTargetType,
  type NotesPage,
  type NotesPageAlias,
  type NotesPageBreadcrumbItem,
  type NotesPageBreadcrumbStatus,
  type NotesPageCover,
  type NotesPageHistorySettings,
  type NotesPageHistorySnapshot,
  type NotesPageIcon,
  type NotesPageTemplate,
  type NotesPaginatedBlockList,
  type NotesParent,
  type NotesPartialUser,
  type NotesRichText,
  type NotesRichTextAnnotations,
  type NotesSearchResult,
  type NotesSearchResultType,
  type NotesSidebarPageList,
  type NotesSuggestion,
  type NotesSuggestionStatus,
  type NotesUnresolvedLink,
  type NotesUnresolvedLinkSourceType,
  type NotesSyncedBlockPayload,
  type NotesTabBlockPayload,
  type NotesTableBlockPayload,
  type NotesTableRowBlockPayload,
  type NotesTableOfContentsBlockPayload,
  type NotesTemplateBlockPayload,
  type NotesTextBlockPayload,
  type NotesToggleBlockPayload,
  type NotesTodoBlockPayload,
  type NotesUnsupportedBlockPayload,
} from "./types";
import { isNotesCatalogRegisteredBlockType } from "./block-catalog";
import { externalMediaUrlIsSupported, type NotesMediaBlockType } from "./media";
import {
  isNotesPageCoverAssetPath,
  isSupportedExternalPageCoverUrl,
} from "./page-cover";
import {
  isNotesPageIconAssetPath,
  isProjectIconAssetPath,
  isSupportedExternalPageIconUrl,
} from "./page-icon";
import {
  normalizeRichTextEquationExpression,
  normalizeRichTextLinkUrl,
} from "./rich-text";

type UnknownRecord = Record<string, unknown>;

const UUID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/iu;

function isRecord(value: unknown): value is UnknownRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readRecord(value: unknown, label: string): UnknownRecord {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  return value;
}

function readString(value: unknown, label: string): string {
  if (typeof value !== "string") throw new Error(`${label} must be a string`);
  return value;
}

function readUuidString(value: unknown, label: string): string {
  const id = readString(value, label);
  if (!UUID_PATTERN.test(id)) throw new Error(`${label} must be a UUID`);
  return id;
}

function readOptionalUuidString(value: unknown, label: string): string | undefined {
  if (value === undefined) return undefined;
  return readUuidString(value, label);
}

function readNullableString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return readString(value, label);
}

function readNullableUuidString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return readUuidString(value, label);
}

function readStringArray(value: unknown, label: string): string[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((item, index) => readString(item, `${label}[${index}]`));
}

function readRecordArray(value: unknown, label: string): Record<string, unknown>[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((item, index) => readRecord(item, `${label}[${index}]`));
}

function readBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") throw new Error(`${label} must be a boolean`);
  return value;
}

function readInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw new Error(`${label} must be an integer`);
  }
  return value;
}

function containsControlCharacters(value: string): boolean {
  return [...value].some((character) => {
    const codePoint = character.codePointAt(0);
    return codePoint !== undefined && codePoint < 32 && character !== "\n" && character !== "\t";
  });
}

function readDisplayString(value: unknown, label: string): string {
  const text = readString(value, label);
  if (!text.trim()) throw new Error(`${label} must not be empty`);
  if (containsControlCharacters(text)) throw new Error(`${label} must not contain control characters`);
  return text;
}

function readMentionObjectId(value: unknown, label: string): string {
  const text = readString(value, label);
  if (!text.trim()) throw new Error(`${label} must not be empty`);
  if (text.length > 2048) throw new Error(`${label} is too long`);
  for (const character of text) {
    const codePoint = character.codePointAt(0);
    if (codePoint !== undefined && codePoint < 32) {
      throw new Error(`${label} must not contain control characters`);
    }
  }
  return text;
}

function readLocalObjectMentionType(
  value: unknown,
  label: string,
): NotesLocalObjectMentionType {
  const objectType = readString(value, label);
  if (!(NOTES_LOCAL_OBJECT_MENTION_TYPES as readonly string[]).includes(objectType)) {
    throw new Error(`${label} is unsupported`);
  }
  return objectType as NotesLocalObjectMentionType;
}

function readOptionalDisplayString(value: unknown, label: string): string | undefined {
  if (value === undefined) return undefined;
  return readDisplayString(value, label);
}

export function isNotesBlockType(value: unknown): value is NotesBlockType {
  return isNotesCatalogRegisteredBlockType(value);
}

export function isNotesColor(value: unknown): value is NotesColor {
  return typeof value === "string" && NOTES_COLORS.includes(value as NotesColor);
}

function isNotesIconColor(value: unknown): value is NotesIconColor {
  return typeof value === "string" && NOTES_ICON_COLORS.includes(value as NotesIconColor);
}

function readNotesColor(value: unknown, label: string): NotesColor {
  const color = readString(value, label);
  if (!isNotesColor(color)) throw new Error(`${label} must be a supported Notion color`);
  return color;
}

function readNotesIconColor(value: unknown, label: string): NotesIconColor {
  const color = readString(value, label);
  if (!isNotesIconColor(color)) throw new Error(`${label} must be a supported Notion icon color`);
  return color;
}

export function parseNotesParent(value: unknown): NotesParent {
  const record = readRecord(value, "parent");
  const type = readString(record.type, "parent.type");
  if (type === "workspace") {
    if (record.workspace !== true) throw new Error("parent.workspace must be true");
    return { type: "workspace", workspace: true };
  }
  if (type === "page_id") {
    return { type, page_id: readString(record.page_id, "parent.page_id") };
  }
  if (type === "block_id") {
    return { type, block_id: readString(record.block_id, "parent.block_id") };
  }
  if (type === "data_source_id") {
    return {
      type,
      data_source_id: readString(record.data_source_id, "parent.data_source_id"),
    };
  }
  throw new Error(`unsupported parent type: ${type}`);
}

export function parseNotesRichTextArray(value: unknown, label: string): NotesRichText[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((item, index) => parseNotesRichText(item, `${label}[${index}]`));
}

function parseNotesRichText(value: unknown, label: string): NotesRichText {
  const record = readRecord(value, label);
  const type = readString(record.type, `${label}.type`);
  const annotations = parseAnnotations(record.annotations, `${label}.annotations`);
  const plainText = readString(record.plain_text, `${label}.plain_text`);
  const href = readNullableString(record.href, `${label}.href`);
  if (type === "text") {
    const text = readRecord(record.text, `${label}.text`);
    return {
      type: "text",
      text: {
        content: readString(text.content, `${label}.text.content`),
        link: parseRichTextLink(text.link, `${label}.text.link`),
      },
      annotations,
      plain_text: plainText,
      href,
    };
  }
  if (type === "mention") {
    const mention = readRecord(record.mention, `${label}.mention`);
    const mentionType = readString(mention.type, `${label}.mention.type`);
    if (mentionType === "page") {
      const page = readRecord(mention.page, `${label}.mention.page`);
      return {
        type: "mention",
        mention: {
          type: "page",
          page: {
            id: readString(page.id, `${label}.mention.page.id`),
          },
        },
        annotations,
        plain_text: plainText,
        href,
      };
    }
    if (mentionType === "user") {
      const user = readRecord(mention.user, `${label}.mention.user`);
      return {
        type: "mention",
        mention: {
          type: "user",
          user: {
            object: "user",
            id: readUuidString(user.id, `${label}.mention.user.id`),
          },
        },
        annotations,
        plain_text: plainText,
        href,
      };
    }
    if (mentionType === "database") {
      const database = readRecord(mention.database, `${label}.mention.database`);
      return {
        type: "mention",
        mention: {
          type: "database",
          database: {
            id: readUuidString(database.id, `${label}.mention.database.id`),
          },
        },
        annotations,
        plain_text: plainText,
        href,
      };
    }
    if (mentionType === "ganbaru_object") {
      const object = readRecord(mention.ganbaru_object, `${label}.mention.ganbaru_object`);
      const objectType = readLocalObjectMentionType(
        object.type,
        `${label}.mention.ganbaru_object.type`,
      );
      const objectId = objectType === "music_item"
        ? readMentionObjectId(object.id, `${label}.mention.ganbaru_object.id`)
        : readUuidString(object.id, `${label}.mention.ganbaru_object.id`);
      return {
        type: "mention",
        mention: {
          type: "ganbaru_object",
          ganbaru_object: {
            type: objectType,
            id: objectId,
          },
        },
        annotations,
        plain_text: plainText,
        href,
      };
    }
    if (mentionType === "date") {
      return {
        type: "mention",
        mention: {
          type: "date",
          date: parseDateMentionValue(mention.date, `${label}.mention.date`),
        },
        annotations,
        plain_text: plainText,
        href,
      };
    }
    throw new Error(`${label}.mention.type is unsupported`);
  }
  if (type === "equation") {
    const equation = readRecord(record.equation, `${label}.equation`);
    const expression = readString(equation.expression, `${label}.equation.expression`);
    if (!normalizeRichTextEquationExpression(expression)) {
      throw new Error(
        `${label}.equation.expression must not be empty, too long, or contain control characters`,
      );
    }
    return {
      type: "equation",
      equation: { expression },
      annotations,
      plain_text: plainText,
      href,
    };
  }
  throw new Error(`${label}.type must be text, mention, or equation`);
}

function parseDateMentionValue(value: unknown, label: string): NotesDateMentionValue {
  const record = readRecord(value, label);
  return {
    start: readDateMentionBoundary(record.start, `${label}.start`),
    ...(record.end === undefined
      ? {}
      : { end: readNullableDateMentionBoundary(record.end, `${label}.end`) }),
    ...(record.time_zone === undefined
      ? {}
      : { time_zone: readNullableDisplayString(record.time_zone, `${label}.time_zone`) }),
    ...(record.ganbaru_reminder === undefined
      ? {}
      : {
          ganbaru_reminder: parseNullableDateMentionReminder(
            record.ganbaru_reminder,
            `${label}.ganbaru_reminder`,
          ),
        }),
  };
}

function parseNullableDateMentionReminder(
  value: unknown,
  label: string,
): NotesDateMentionReminder | null {
  if (value === null) return null;
  const record = readRecord(value, label);
  return {
    enabled: readBoolean(record.enabled, `${label}.enabled`),
  };
}

function readNullableDisplayString(value: unknown, label: string): string | null {
  if (value === null) return null;
  return readDisplayString(value, label);
}

function readDateMentionBoundary(value: unknown, label: string): string {
  const text = readDisplayString(value, label);
  if (!dateMentionBoundaryLooksIso(text)) {
    throw new Error(`${label} must be an ISO date or date-time`);
  }
  return text;
}

function readNullableDateMentionBoundary(value: unknown, label: string): string | null {
  if (value === null) return null;
  return readDateMentionBoundary(value, label);
}

function dateMentionBoundaryLooksIso(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}/u.test(value)) return false;
  const month = Number.parseInt(value.slice(5, 7), 10);
  const day = Number.parseInt(value.slice(8, 10), 10);
  if (month < 1 || month > 12 || day < 1 || day > 31) return false;
  if (value.length === 10) return true;
  return value.at(10) === "T" && value.length <= 80;
}

function parseNotesCommentParent(value: unknown, label: string): NotesCommentParent {
  const parent = parseNotesParent(value);
  if (parent.type === "page_id" || parent.type === "block_id") return parent;
  throw new Error(`${label}.type must be page_id or block_id`);
}

function parseNotesPartialUser(value: unknown, label: string): NotesPartialUser {
  const record = readRecord(value, label);
  if (record.object !== "user") throw new Error(`${label}.object must be user`);
  return {
    object: "user" as const,
    id: readString(record.id, `${label}.id`),
  };
}

export function parseNotesLocalUser(value: unknown): NotesLocalUser {
  const record = readRecord(value, "local user");
  if (record.object !== "user") throw new Error("local user.object must be user");
  return {
    object: "user",
    id: readString(record.id, "local user.id"),
    display_name: readDisplayString(record.display_name, "local user.display_name"),
    created_time: readString(record.created_time, "local user.created_time"),
    last_edited_time: readString(record.last_edited_time, "local user.last_edited_time"),
  };
}

function parseNotesCommentDisplayName(value: unknown, label: string): NotesCommentDisplayName {
  const record = readRecord(value, label);
  const type = readString(record.type, `${label}.type`);
  const resolvedName = readDisplayString(record.resolved_name, `${label}.resolved_name`);
  if (type === "user" || type === "integration" || type === "custom") {
    return { type, resolved_name: resolvedName };
  }
  throw new Error(`${label}.type must be user, integration, or custom`);
}

function parseCommentAttachments(value: unknown, label: string): Record<string, unknown>[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((attachment, index) => readRecord(attachment, `${label}[${index}]`));
}

function parseRichTextLink(value: unknown, label: string): { url: string } | null {
  if (value === null) return null;
  const record = readRecord(value, label);
  const url = readString(record.url, `${label}.url`);
  if (!normalizeRichTextLinkUrl(url)) {
    throw new Error(`${label}.url must be a valid HTTP, HTTPS, or email URL`);
  }
  return {
    url,
  };
}

function parseAnnotations(value: unknown, label: string): NotesRichTextAnnotations {
  const record = readRecord(value, label);
  return {
    bold: readBoolean(record.bold, `${label}.bold`),
    italic: readBoolean(record.italic, `${label}.italic`),
    strikethrough: readBoolean(record.strikethrough, `${label}.strikethrough`),
    underline: readBoolean(record.underline, `${label}.underline`),
    code: readBoolean(record.code, `${label}.code`),
    color: readNotesColor(record.color, `${label}.color`),
  };
}

function parseTextPayload(value: unknown, label: string): NotesTextBlockPayload {
  const record = readRecord(value, label);
  return {
    rich_text: parseNotesRichTextArray(record.rich_text, `${label}.rich_text`),
    ...(record.color === undefined ? {} : { color: readNotesColor(record.color, `${label}.color`) }),
    ...(record.is_toggleable === undefined
      ? {}
      : { is_toggleable: readBoolean(record.is_toggleable, `${label}.is_toggleable`) }),
    ...(record.ganbaru_open === undefined
      ? {}
      : { ganbaru_open: readBoolean(record.ganbaru_open, `${label}.ganbaru_open`) }),
    ...(record.icon === undefined ? {} : { icon: parseNullableNotesIcon(record.icon, `${label}.icon`) }),
  };
}

function parseEmptyObjectPayload(value: unknown, label: string): Record<string, never> {
  const record = readRecord(value, label);
  if (Object.keys(record).length > 0) throw new Error(`${label} must be an empty object`);
  return {};
}

function parseTemplatePayload(value: unknown, label: string): NotesTemplateBlockPayload {
  const record = readRecord(value, label);
  if (record.color !== undefined) throw new Error(`${label}.color is not supported`);
  if (record.children !== undefined) {
    throw new Error(`${label}.children must be stored as child blocks`);
  }
  return {
    rich_text: parseNotesRichTextArray(record.rich_text, `${label}.rich_text`),
  };
}

function isNotesButtonInsertPosition(value: unknown): value is NotesButtonInsertPosition {
  return (
    typeof value === "string"
    && NOTES_BUTTON_INSERT_POSITIONS.includes(value as NotesButtonInsertPosition)
  );
}

function parseButtonAction(value: unknown, label: string): NotesButtonAction {
  const record = readRecord(value, label);
  const type = readString(record.type, `${label}.type`);
  if (type !== "insert_blocks") throw new Error(`${label}.type must be insert_blocks`);
  const source = readString(record.source, `${label}.source`);
  if (source !== "children") throw new Error(`${label}.source must be children`);
  const position = readString(record.position, `${label}.position`);
  if (!isNotesButtonInsertPosition(position)) {
    throw new Error(`${label}.position must be a supported button insert position`);
  }
  return { type, source, position };
}

function parseButtonActions(value: unknown, label: string): NotesButtonAction[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  if (value.length < 1 || value.length > 10) {
    throw new Error(`${label} must include between 1 and 10 actions`);
  }
  return value.map((item, index) => parseButtonAction(item, `${label}[${index}]`));
}

function parseButtonPayload(value: unknown, label: string): NotesButtonBlockPayload {
  const record = readRecord(value, label);
  if (record.children !== undefined) {
    throw new Error(`${label}.children must be stored as child blocks`);
  }
  return {
    rich_text: parseNotesRichTextArray(record.rich_text, `${label}.rich_text`),
    icon: parseNullableNotesIcon(record.icon, `${label}.icon`),
    actions: parseButtonActions(record.actions, `${label}.actions`),
  };
}

function parseTodoPayload(value: unknown, label: string): NotesTodoBlockPayload {
  const record = readRecord(value, label);
  return {
    ...parseTextPayload(record, label),
    checked: readBoolean(record.checked, `${label}.checked`),
  };
}

function parseTogglePayload(value: unknown, label: string): NotesToggleBlockPayload {
  const record = readRecord(value, label);
  return {
    ...parseTextPayload(record, label),
    ...(record.ganbaru_open === undefined
      ? {}
      : { ganbaru_open: readBoolean(record.ganbaru_open, `${label}.ganbaru_open`) }),
  };
}

function parseCalloutPayload(value: unknown, label: string): NotesCalloutBlockPayload {
  const record = readRecord(value, label);
  return {
    ...parseTextPayload(record, label),
    icon: parseNullableNotesIcon(record.icon, `${label}.icon`),
  };
}

function parseNullableNotesIcon(value: unknown, label: string): NotesCalloutIcon {
  if (value === null) return null;
  return parseNotesIcon(value, label);
}

function parseNotesIcon(value: unknown, label: string): NotesPageIcon {
  const record = readRecord(value, label);
  const type = readString(record.type, `${label}.type`);
  if (type === "emoji") {
    return { type, emoji: readDisplayString(record.emoji, `${label}.emoji`) };
  }
  if (type === "custom_emoji") {
    const customEmoji = readRecord(record.custom_emoji, `${label}.custom_emoji`);
    const id = readDisplayString(customEmoji.id, `${label}.custom_emoji.id`);
    const name = readOptionalDisplayString(customEmoji.name, `${label}.custom_emoji.name`);
    const url = readOptionalDisplayString(customEmoji.url, `${label}.custom_emoji.url`);
    const assetPath = readOptionalDisplayString(
      customEmoji.ganbaru_asset_path,
      `${label}.custom_emoji.ganbaru_asset_path`,
    );
    if (assetPath !== undefined && !isProjectIconAssetPath(assetPath)) {
      throw new Error(`${label}.custom_emoji.ganbaru_asset_path must stay under a managed image asset directory`);
    }
    if (url !== undefined) {
      if (url.startsWith("ganbaru-asset:")) {
        const urlAssetPath = url.slice("ganbaru-asset:".length);
        if (!isProjectIconAssetPath(urlAssetPath)) {
          throw new Error(`${label}.custom_emoji.url must stay under a managed image asset directory`);
        }
        if (assetPath !== urlAssetPath) {
          throw new Error(`${label}.custom_emoji.url must reference the managed icon asset path`);
        }
      } else if (!isSupportedExternalPageIconUrl(url)) {
        throw new Error(`${label}.custom_emoji.url must be a supported HTTPS image URL`);
      }
    }
    return {
      type,
      custom_emoji: {
        id,
        ...(name === undefined ? {} : { name }),
        ...(url === undefined ? {} : { url }),
        ...(assetPath === undefined ? {} : { ganbaru_asset_path: assetPath }),
      },
    };
  }
  if (type === "icon") {
    const icon = readRecord(record.icon, `${label}.icon`);
    const name = readString(icon.name, `${label}.icon.name`);
    if (!name.trim()) throw new Error(`${label}.icon.name must not be empty`);
    return {
      type,
      icon: {
        name,
        ...(icon.color === undefined ? {} : { color: readNotesIconColor(icon.color, `${label}.icon.color`) }),
      },
    };
  }
  if (type === "external") {
    const external = readRecord(record.external, `${label}.external`);
    const url = readDisplayString(external.url, `${label}.external.url`);
    if (!isSupportedExternalPageIconUrl(url)) {
      throw new Error(`${label}.external.url must be a supported HTTPS image URL`);
    }
    return { type, external: { url } };
  }
  if (type === "file") {
    const file = readRecord(record.file, `${label}.file`);
    const url = readDisplayString(file.url, `${label}.file.url`);
    const expiryTime = readOptionalDisplayString(file.expiry_time, `${label}.file.expiry_time`);
    const name = readOptionalDisplayString(file.name, `${label}.file.name`);
    const assetPath = readOptionalDisplayString(file.ganbaru_asset_path, `${label}.file.ganbaru_asset_path`);
    const contentType = readOptionalDisplayString(file.content_type, `${label}.file.content_type`);
    const sha256 = readOptionalDisplayString(file.sha256, `${label}.file.sha256`);
    const byteSize = file.byte_size === undefined ? undefined : readInteger(file.byte_size, `${label}.file.byte_size`);
    if (assetPath !== undefined) {
      if (!isNotesPageIconAssetPath(assetPath)) {
        throw new Error(`${label}.file.ganbaru_asset_path must stay under a managed image asset directory`);
      }
      if (url !== `ganbaru-asset:${assetPath}`) {
        throw new Error(`${label}.file.url must reference the managed icon asset path`);
      }
      if (contentType !== "image/png" && contentType !== "image/jpeg" && contentType !== "image/webp") {
        throw new Error(`${label}.file.content_type must be a supported local image type`);
      }
      if (byteSize === undefined || byteSize <= 0) {
        throw new Error(`${label}.file.byte_size must be positive`);
      }
      if (sha256 === undefined || !/^[a-f0-9]{64}$/.test(sha256)) {
        throw new Error(`${label}.file.sha256 must be a lowercase SHA-256 hex digest`);
      }
    } else if (url.startsWith("ganbaru-asset:")) {
      throw new Error(`${label}.file.url must include managed asset metadata`);
    } else if (!isSupportedExternalPageIconUrl(url)) {
      throw new Error(`${label}.file.url must be a supported HTTPS image URL`);
    }
    return {
      type,
      file: {
        url,
        ...(expiryTime === undefined ? {} : { expiry_time: expiryTime }),
        ...(name === undefined ? {} : { name }),
        ...(contentType === undefined ? {} : { content_type: contentType as "image/png" | "image/jpeg" | "image/webp" }),
        ...(byteSize === undefined ? {} : { byte_size: byteSize }),
        ...(sha256 === undefined ? {} : { sha256 }),
        ...(assetPath === undefined ? {} : { ganbaru_asset_path: assetPath }),
      },
    };
  }
  throw new Error(`${label}.type must be a supported Notion icon type`);
}

function parseTableOfContentsPayload(
  value: unknown,
  label: string,
): NotesTableOfContentsBlockPayload {
  const record = readRecord(value, label);
  return {
    ...(record.color === undefined ? {} : { color: readNotesColor(record.color, `${label}.color`) }),
  };
}

function parseChildPagePayload(value: unknown, label: string): NotesChildPageBlockPayload {
  const record = readRecord(value, label);
  return {
    title: readString(record.title, `${label}.title`),
  };
}

function parseChildDatabasePayload(
  value: unknown,
  label: string,
): NotesChildDatabaseBlockPayload {
  const record = readRecord(value, label);
  const title = readString(record.title, `${label}.title`);
  if (containsControlCharacters(title)) {
    throw new Error(`${label}.title must not contain control characters`);
  }
  const databaseId = readOptionalUuidString(record.database_id, `${label}.database_id`);
  const dataSourceId = readOptionalUuidString(record.data_source_id, `${label}.data_source_id`);
  const viewId = readOptionalUuidString(record.view_id, `${label}.view_id`);
  return {
    title,
    ...(databaseId === undefined ? {} : { database_id: databaseId }),
    ...(dataSourceId === undefined ? {} : { data_source_id: dataSourceId }),
    ...(viewId === undefined ? {} : { view_id: viewId }),
  };
}

function parseColumnPayload(value: unknown, label: string): NotesColumnBlockPayload {
  const record = readRecord(value, label);
  if (record.width_ratio === undefined) return {};
  if (typeof record.width_ratio !== "number") {
    throw new Error(`${label}.width_ratio must be a number`);
  }
  if (record.width_ratio <= 0 || record.width_ratio > 1) {
    throw new Error(`${label}.width_ratio must be greater than 0 and no more than 1`);
  }
  return { width_ratio: record.width_ratio };
}

function parseTablePayload(value: unknown, label: string): NotesTableBlockPayload {
  const record = readRecord(value, label);
  const tableWidth = readInteger(record.table_width, `${label}.table_width`);
  if (tableWidth < 1 || tableWidth > 100) {
    throw new Error(`${label}.table_width must be between 1 and 100`);
  }
  return {
    table_width: tableWidth,
    has_column_header: readBoolean(record.has_column_header, `${label}.has_column_header`),
    has_row_header: readBoolean(record.has_row_header, `${label}.has_row_header`),
  };
}

function parseTableRowPayload(value: unknown, label: string): NotesTableRowBlockPayload {
  const record = readRecord(value, label);
  if (!Array.isArray(record.cells)) throw new Error(`${label}.cells must be an array`);
  if (record.cells.length < 1 || record.cells.length > 100) {
    throw new Error(`${label}.cells must include between 1 and 100 cells`);
  }
  return {
    cells: record.cells.map((cell, index) =>
      parseNotesRichTextArray(cell, `${label}.cells[${index}]`)
    ),
  };
}

function parseTabPayload(value: unknown, label: string): NotesTabBlockPayload {
  return parseEmptyObjectPayload(value, label);
}

function parseMediaPayload(
  value: unknown,
  label: string,
  blockType: NotesMediaBlockType,
): NotesMediaBlockPayload {
  const record = readRecord(value, label);
  const caption = record.caption === undefined
    ? []
    : parseNotesRichTextArray(record.caption, `${label}.caption`);
  if (blockType === "file" && record.caption === undefined) {
    throw new Error(`${label}.caption must be a rich text array`);
  }
  const name = record.name === undefined ? undefined : readString(record.name, `${label}.name`);
  if (name !== undefined && containsControlCharacters(name)) {
    throw new Error(`${label}.name must not contain control characters`);
  }
  return {
    caption,
    ...(name === undefined ? {} : { name }),
    ...parseFileObject(record, label, blockType, true),
  };
}

function parseFileObject(
  value: unknown,
  label: string,
  mediaType: NotesMediaBlockType,
  allowBlankExternalUrl = false,
): NotesPageCover {
  const record = readRecord(value, label);
  const fileType = readString(record.type, `${label}.type`);
  if (fileType === "external") {
    const external = readRecord(record.external, `${label}.external`);
    const url = readString(external.url, `${label}.external.url`);
    if ((!allowBlankExternalUrl || url.trim()) && !externalMediaUrlIsSupported(mediaType, url)) {
      throw new Error(`${label}.external.url must be a supported HTTPS ${mediaType} URL`);
    }
    return { type: "external", external: { url } };
  }
  if (fileType === "file") {
    const file = readRecord(record.file, `${label}.file`);
    const url = readString(file.url, `${label}.file.url`);
    const assetPath = readOptionalDisplayString(file.ganbaru_asset_path, `${label}.file.ganbaru_asset_path`);
    const fileName = readOptionalDisplayString(file.name, `${label}.file.name`);
    const contentType = readOptionalDisplayString(file.content_type, `${label}.file.content_type`);
    const byteSize = file.byte_size === undefined ? undefined : readInteger(file.byte_size, `${label}.file.byte_size`);
    const sha256 = readOptionalDisplayString(file.sha256, `${label}.file.sha256`);
    if (assetPath !== undefined) {
      if (!isNotesFileAssetPath(assetPath)) {
        throw new Error(`${label}.file.ganbaru_asset_path must stay under the managed Notes file directory`);
      }
      if (url !== `ganbaru-asset:${assetPath}`) {
        throw new Error(`${label}.file.url must reference the managed file asset path`);
      }
      if (contentType === undefined || !localMediaContentTypeMatchesBlock(mediaType, contentType)) {
        throw new Error(`${label}.file.content_type must match the local media block type`);
      }
      if (byteSize === undefined || byteSize <= 0) {
        throw new Error(`${label}.file.byte_size must be positive`);
      }
      if (sha256 === undefined || !/^[a-f0-9]{64}$/.test(sha256)) {
        throw new Error(`${label}.file.sha256 must be a lowercase SHA-256 hex digest`);
      }
      return {
        type: "file",
        file: {
          url,
          ...(fileName === undefined ? {} : { name: fileName }),
          content_type: contentType,
          byte_size: byteSize,
          sha256,
          ganbaru_asset_path: assetPath,
        },
      };
    }
    if (url.startsWith("ganbaru-asset:")) {
      throw new Error(`${label}.file.url must include managed asset metadata`);
    }
    const expiryTime = readString(file.expiry_time, `${label}.file.expiry_time`);
    if (!externalMediaUrlIsSupported(mediaType, url)) {
      throw new Error(`${label}.file.url must be a supported HTTPS ${mediaType} URL`);
    }
    if (containsControlCharacters(expiryTime)) {
      throw new Error(`${label}.file.expiry_time must not contain control characters`);
    }
    return { type: "file", file: { url, expiry_time: expiryTime } };
  }
  if (fileType === "file_upload") {
    const fileUpload = readRecord(record.file_upload, `${label}.file_upload`);
    const id = readString(fileUpload.id, `${label}.file_upload.id`);
    if (!UUID_PATTERN.test(id)) {
      throw new Error(`${label}.file_upload.id must be a UUID`);
    }
    return { type: "file_upload", file_upload: { id } };
  }
  throw new Error(`${label}.type must be file, external, or file_upload`);
}

function isNotesFileAssetPath(path: string): boolean {
  const remainder = path.trim().startsWith("notes/files/")
    ? path.trim().slice("notes/files/".length)
    : "";
  return Boolean(remainder)
    && !remainder.includes("/")
    && !path.includes("..")
    && !path.includes("\\");
}

function localMediaContentTypeMatchesBlock(
  mediaType: NotesMediaBlockType,
  contentType: string,
): boolean {
  if (mediaType === "image") {
    return contentType === "image/png"
      || contentType === "image/jpeg"
      || contentType === "image/webp";
  }
  if (mediaType === "video") return contentType.startsWith("video/");
  if (mediaType === "audio") return contentType.startsWith("audio/");
  if (mediaType === "pdf") return contentType === "application/pdf";
  return contentType.includes("/")
    && !contentType.includes(" ")
    && !contentType.includes(";")
    && contentType !== "image/svg+xml";
}

function parseNullablePageCover(value: unknown, label: string): NotesPageCover | null {
  if (value === null) return null;
  return parsePageCoverFileObject(value, label);
}

function parsePageCoverFileObject(value: unknown, label: string): NotesPageCover {
  const record = readRecord(value, label);
  const type = readString(record.type, `${label}.type`);
  if (type === "external") {
    const external = readRecord(record.external, `${label}.external`);
    const url = readDisplayString(external.url, `${label}.external.url`);
    if (!isSupportedExternalPageCoverUrl(url)) {
      throw new Error(`${label}.external.url must be a supported HTTPS image URL`);
    }
    return { type, external: { url } };
  }
  if (type === "file") {
    const file = readRecord(record.file, `${label}.file`);
    const url = readDisplayString(file.url, `${label}.file.url`);
    const expiryTime = readOptionalDisplayString(file.expiry_time, `${label}.file.expiry_time`);
    const name = readOptionalDisplayString(file.name, `${label}.file.name`);
    const assetPath = readOptionalDisplayString(file.ganbaru_asset_path, `${label}.file.ganbaru_asset_path`);
    const contentType = readOptionalDisplayString(file.content_type, `${label}.file.content_type`);
    const sha256 = readOptionalDisplayString(file.sha256, `${label}.file.sha256`);
    const byteSize = file.byte_size === undefined ? undefined : readInteger(file.byte_size, `${label}.file.byte_size`);
    if (assetPath !== undefined) {
      if (!isNotesPageCoverAssetPath(assetPath)) {
        throw new Error(`${label}.file.ganbaru_asset_path must stay under a managed image asset directory`);
      }
      if (url !== `ganbaru-asset:${assetPath}`) {
        throw new Error(`${label}.file.url must reference the managed cover asset path`);
      }
      if (contentType !== "image/png" && contentType !== "image/jpeg" && contentType !== "image/webp") {
        throw new Error(`${label}.file.content_type must be a supported local image type`);
      }
      if (byteSize === undefined || byteSize <= 0) {
        throw new Error(`${label}.file.byte_size must be positive`);
      }
      if (sha256 === undefined || !/^[a-f0-9]{64}$/.test(sha256)) {
        throw new Error(`${label}.file.sha256 must be a lowercase SHA-256 hex digest`);
      }
    } else if (url.startsWith("ganbaru-asset:")) {
      throw new Error(`${label}.file.url must include managed asset metadata`);
    } else {
      if (!isSupportedExternalPageCoverUrl(url)) {
        throw new Error(`${label}.file.url must be a supported HTTPS image URL`);
      }
      if (expiryTime === undefined) {
        throw new Error(`${label}.file.expiry_time must be a string`);
      }
    }
    return {
      type,
      file: {
        url,
        ...(expiryTime === undefined ? {} : { expiry_time: expiryTime }),
        ...(name === undefined ? {} : { name }),
        ...(contentType === undefined ? {} : { content_type: contentType as "image/png" | "image/jpeg" | "image/webp" }),
        ...(byteSize === undefined ? {} : { byte_size: byteSize }),
        ...(sha256 === undefined ? {} : { sha256 }),
        ...(assetPath === undefined ? {} : { ganbaru_asset_path: assetPath }),
      },
    };
  }
  if (type === "file_upload") {
    const fileUpload = readRecord(record.file_upload, `${label}.file_upload`);
    const id = readString(fileUpload.id, `${label}.file_upload.id`);
    if (!UUID_PATTERN.test(id)) {
      throw new Error(`${label}.file_upload.id must be a UUID`);
    }
    return { type, file_upload: { id } };
  }
  throw new Error(`${label}.type must be file, external, or file_upload`);
}

function parseBookmarkPayload(value: unknown, label: string): NotesBookmarkBlockPayload {
  const record = readRecord(value, label);
  return {
    caption: parseNotesRichTextArray(record.caption, `${label}.caption`),
    url: readString(record.url, `${label}.url`),
  };
}

function parseEmbedPayload(value: unknown, label: string): NotesEmbedBlockPayload {
  const record = readRecord(value, label);
  return {
    url: readString(record.url, `${label}.url`),
  };
}

function parseLinkPreviewPayload(value: unknown, label: string): NotesLinkPreviewBlockPayload {
  const record = readRecord(value, label);
  const url = readString(record.url, `${label}.url`);
  if (containsControlCharacters(url)) {
    throw new Error(`${label}.url must not contain control characters`);
  }
  return { url };
}

function parseSyncedBlockPayload(value: unknown, label: string): NotesSyncedBlockPayload {
  const record = readRecord(value, label);
  if (!("synced_from" in record)) {
    throw new Error(`${label}.synced_from is required`);
  }
  if (record.synced_from === null) return { synced_from: null };
  const syncedFrom = readRecord(record.synced_from, `${label}.synced_from`);
  const type = readString(syncedFrom.type, `${label}.synced_from.type`);
  if (type !== "block_id") {
    throw new Error(`${label}.synced_from.type must be block_id`);
  }
  const blockId = readString(syncedFrom.block_id, `${label}.synced_from.block_id`);
  if (!UUID_PATTERN.test(blockId)) {
    throw new Error(`${label}.synced_from.block_id must be a UUID`);
  }
  return {
    synced_from: {
      type,
      block_id: blockId,
    },
  };
}

function parseEquationPayload(value: unknown, label: string): NotesEquationBlockPayload {
  const record = readRecord(value, label);
  return {
    expression: readString(record.expression, `${label}.expression`),
  };
}

function parseUnsupportedPayload(value: unknown, label: string): NotesUnsupportedBlockPayload {
  const record = readRecord(value, label);
  const payload: NotesUnsupportedBlockPayload = { ...record };
  if (record.block_type !== undefined) {
    payload.block_type = readDisplayString(record.block_type, `${label}.block_type`);
  }
  if (record.source_type !== undefined) {
    payload.source_type = readDisplayString(record.source_type, `${label}.source_type`);
  }
  if (record.raw !== undefined) {
    payload.raw = readRecord(record.raw, `${label}.raw`);
  }
  if (record.warnings !== undefined) {
    if (!Array.isArray(record.warnings)) {
      throw new Error(`${label}.warnings must be an array`);
    }
    payload.warnings = record.warnings.map((warning, index) =>
      readDisplayString(warning, `${label}.warnings[${index}]`)
    );
  }
  return payload;
}

function parseCodePayload(value: unknown, label: string): NotesCodeBlockPayload {
  const record = readRecord(value, label);
  return {
    rich_text: parseNotesRichTextArray(record.rich_text, `${label}.rich_text`),
    caption: parseNotesRichTextArray(record.caption, `${label}.caption`),
    language: readString(record.language, `${label}.language`),
  };
}

export function parseNotesPage(value: unknown): NotesPage {
  const record = readRecord(value, "page");
  if (record.object !== "page") throw new Error("page.object must be page");
  return {
    object: "page",
    id: readString(record.id, "page.id"),
    created_time: readString(record.created_time, "page.created_time"),
    last_edited_time: readString(record.last_edited_time, "page.last_edited_time"),
    parent: parseNotesParent(record.parent),
    in_trash: readBoolean(record.in_trash, "page.in_trash"),
    archived: typeof record.archived === "boolean" ? record.archived : undefined,
    icon: parseNullableNotesIcon(record.icon, "page.icon"),
    cover: parseNullablePageCover(record.cover, "page.cover"),
    properties: readRecord(record.properties, "page.properties"),
    url: readNullableString(record.url, "page.url"),
    public_url: readNullableString(record.public_url, "page.public_url"),
    source_provider: readNullableString(record.source_provider, "page.source_provider"),
    source_object_id: readNullableString(record.source_object_id, "page.source_object_id"),
    source_workspace_id: readNullableString(record.source_workspace_id, "page.source_workspace_id"),
    source_last_edited_time: readNullableString(
      record.source_last_edited_time,
      "page.source_last_edited_time",
    ),
  };
}

function parseNotesDatabaseDataSourceSummary(
  value: unknown,
  label: string,
): NotesDatabaseDataSourceSummary {
  const record = readRecord(value, label);
  return {
    id: readString(record.id, `${label}.id`),
    name: readString(record.name, `${label}.name`),
  };
}

function parseNotesDatabaseDataSources(
  value: unknown,
  label: string,
): NotesDatabaseDataSourceSummary[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((item, index) => parseNotesDatabaseDataSourceSummary(item, `${label}[${index}]`));
}

export function parseNotesDatabase(value: unknown): NotesDatabase {
  const record = readRecord(value, "database");
  if (record.object !== "database") throw new Error("database.object must be database");
  return {
    object: "database",
    id: readString(record.id, "database.id"),
    parent: parseNotesParent(record.parent),
    title: readString(record.title, "database.title"),
    title_rich_text: parseNotesRichTextArray(record.title_rich_text, "database.title_rich_text"),
    description: parseNotesRichTextArray(record.description, "database.description"),
    icon: parseNullableNotesIcon(record.icon, "database.icon"),
    cover: parseNullablePageCover(record.cover, "database.cover"),
    in_trash: readBoolean(record.in_trash, "database.in_trash"),
    is_inline: readBoolean(record.is_inline, "database.is_inline"),
    data_sources: parseNotesDatabaseDataSources(record.data_sources, "database.data_sources"),
    url: readNullableString(record.url, "database.url"),
    public_url: readNullableString(record.public_url, "database.public_url"),
    source_provider: readNullableString(record.source_provider, "database.source_provider"),
    source_object_id: readNullableString(record.source_object_id, "database.source_object_id"),
    source_workspace_id: readNullableString(
      record.source_workspace_id,
      "database.source_workspace_id",
    ),
    source_last_edited_time: readNullableString(
      record.source_last_edited_time,
      "database.source_last_edited_time",
    ),
    created_time: readString(record.created_time, "database.created_time"),
    last_edited_time: readString(record.last_edited_time, "database.last_edited_time"),
  };
}

export function parseNotesDataSource(value: unknown): NotesDataSource {
  const record = readRecord(value, "data source");
  if (record.object !== "data_source") {
    throw new Error("data source.object must be data_source");
  }
  const parent = readRecord(record.parent, "data source.parent");
  if (parent.type !== "database_id") {
    throw new Error("data source.parent.type must be database_id");
  }
  return {
    object: "data_source",
    id: readString(record.id, "data source.id"),
    parent: {
      type: "database_id",
      database_id: readString(parent.database_id, "data source.parent.database_id"),
    },
    database_parent: parseNotesParent(record.database_parent),
    title: readString(record.title, "data source.title"),
    title_rich_text: parseNotesRichTextArray(
      record.title_rich_text,
      "data source.title_rich_text",
    ),
    description: parseNotesRichTextArray(record.description, "data source.description"),
    icon: parseNullableNotesIcon(record.icon, "data source.icon"),
    properties: readRecord(record.properties, "data source.properties"),
    in_trash: readBoolean(record.in_trash, "data source.in_trash"),
    source_provider: readNullableString(record.source_provider, "data source.source_provider"),
    source_object_id: readNullableString(record.source_object_id, "data source.source_object_id"),
    source_workspace_id: readNullableString(
      record.source_workspace_id,
      "data source.source_workspace_id",
    ),
    source_last_edited_time: readNullableString(
      record.source_last_edited_time,
      "data source.source_last_edited_time",
    ),
    created_time: readString(record.created_time, "data source.created_time"),
    last_edited_time: readString(record.last_edited_time, "data source.last_edited_time"),
  };
}

function isNotesDatabaseViewType(value: unknown): value is NotesDatabaseViewType {
  return (
    typeof value === "string"
    && NOTES_DATABASE_VIEW_TYPES.includes(value as NotesDatabaseViewType)
  );
}

function readNotesDatabaseViewType(value: unknown, label: string): NotesDatabaseViewType {
  const viewType = readString(value, label);
  if (!isNotesDatabaseViewType(viewType)) {
    throw new Error(`${label} must be a supported database view type`);
  }
  return viewType;
}

function readNullableRecord(value: unknown, label: string): Record<string, unknown> | null {
  if (value === null) return null;
  return readRecord(value, label);
}

export function parseNotesDatabaseView(value: unknown): NotesDatabaseView {
  const record = readRecord(value, "database view");
  if (record.object !== "view") throw new Error("database view.object must be view");
  const parent = readRecord(record.parent, "database view.parent");
  if (parent.type !== "database_id") {
    throw new Error("database view.parent.type must be database_id");
  }
  return {
    object: "view",
    id: readString(record.id, "database view.id"),
    parent: {
      type: "database_id",
      database_id: readString(parent.database_id, "database view.parent.database_id"),
    },
    data_source_id: readString(record.data_source_id, "database view.data_source_id"),
    name: readString(record.name, "database view.name"),
    type: readNotesDatabaseViewType(record.type, "database view.type"),
    filter: readNullableRecord(record.filter, "database view.filter"),
    sorts: readRecordArray(record.sorts, "database view.sorts"),
    configuration: readNullableRecord(record.configuration, "database view.configuration"),
    url: readNullableString(record.url, "database view.url"),
    source_provider: readNullableString(record.source_provider, "database view.source_provider"),
    source_object_id: readNullableString(record.source_object_id, "database view.source_object_id"),
    source_workspace_id: readNullableString(
      record.source_workspace_id,
      "database view.source_workspace_id",
    ),
    source_last_edited_time: readNullableString(
      record.source_last_edited_time,
      "database view.source_last_edited_time",
    ),
    created_time: readString(record.created_time, "database view.created_time"),
    last_edited_time: readString(record.last_edited_time, "database view.last_edited_time"),
  };
}

export function parseNotesCreatedDatabase(value: unknown): NotesCreatedDatabase {
  const record = readRecord(value, "created database");
  const block = parseNotesBlock(record.block);
  if (block.type !== "child_database") {
    throw new Error("created database.block must be a child_database block");
  }
  return {
    database: parseNotesDatabase(record.database),
    data_source: parseNotesDataSource(record.data_source),
    view: parseNotesDatabaseView(record.view),
    block,
  };
}

export function parseNotesDataSourceSchema(value: unknown): NotesDataSourceSchema {
  const record = readRecord(value, "data source schema");
  return {
    data_source: parseNotesDataSource(record.data_source),
    view: parseNotesDatabaseView(record.view),
  };
}

function isNotesTableRowOpenMode(value: unknown): value is NotesDatabaseTableRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesBoardRowOpenMode(value: unknown): value is NotesDatabaseBoardRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesGalleryRowOpenMode(value: unknown): value is NotesDatabaseGalleryRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesGalleryCoverSource(value: unknown): value is NotesDatabaseGalleryCoverSource {
  return value === "page_cover" || value === "files_property" || value === "none";
}

function isNotesGalleryCardSize(value: unknown): value is NotesDatabaseGalleryCardSize {
  return value === "small" || value === "medium" || value === "large";
}

function isNotesListRowOpenMode(value: unknown): value is NotesDatabaseListRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesCalendarRowOpenMode(value: unknown): value is NotesDatabaseCalendarRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesTimelineRowOpenMode(value: unknown): value is NotesDatabaseTimelineRowOpenMode {
  return value === "full_page" || value === "side_panel";
}

function isNotesTableFilterCondition(value: unknown): value is NotesDatabaseTableFilterCondition {
  return (
    value === "contains"
    || value === "equals"
    || value === "is_empty"
    || value === "is_not_empty"
    || value === "checked"
    || value === "unchecked"
  );
}

function isNotesTableSortDirection(value: unknown): value is NotesDatabaseTableSortDirection {
  return value === "ascending" || value === "descending";
}

function validateNotesTableConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "table") {
    throw new Error("database table configuration.type must be table");
  }
  const table = readRecord(value.table, "database table configuration.table");
  readStringArray(table.property_order, "database table configuration.property_order");
  readStringArray(table.hidden_property_ids, "database table configuration.hidden_property_ids");
  const columnWidths = readRecord(
    table.column_widths ?? {},
    "database table configuration.column_widths",
  );
  for (const [propertyId, width] of Object.entries(columnWidths)) {
    readString(propertyId, "database table configuration.column_widths key");
    const parsedWidth = readInteger(width, `database table configuration.column_widths.${propertyId}`);
    if (parsedWidth < 96 || parsedWidth > 480) {
      throw new Error(`database table configuration.column_widths.${propertyId} is out of range`);
    }
  }
  const rowOpenMode = table.row_open_mode ?? "full_page";
  if (!isNotesTableRowOpenMode(rowOpenMode)) {
    throw new Error("database table configuration.row_open_mode must be supported");
  }
}

function validateNotesTableFilter(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "and") {
    throw new Error("database table filter.type must be and");
  }
  const filters = readRecordArray(value.filters ?? [], "database table filter.filters");
  for (const [index, filter] of filters.entries()) {
    readString(filter.property_id, `database table filter.filters[${index}].property_id`);
    if (!isNotesTableFilterCondition(filter.condition)) {
      throw new Error(`database table filter.filters[${index}].condition must be supported`);
    }
    const filterValue = filter.value;
    if (
      filterValue !== undefined
      && filterValue !== null
      && typeof filterValue !== "string"
      && typeof filterValue !== "number"
      && typeof filterValue !== "boolean"
    ) {
      throw new Error(`database table filter.filters[${index}].value must be scalar`);
    }
  }
}

function validateNotesTableSorts(value: Record<string, unknown>[]): void {
  for (const [index, sort] of value.entries()) {
    readString(sort.property_id, `database table sorts[${index}].property_id`);
    if (!isNotesTableSortDirection(sort.direction)) {
      throw new Error(`database table sorts[${index}].direction must be supported`);
    }
  }
}

function validateNotesBoardConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "board") {
    throw new Error("database board configuration.type must be board");
  }
  const board = readRecord(value.board, "database board configuration.board");
  if (board.group_property_id !== null && board.group_property_id !== undefined) {
    readString(board.group_property_id, "database board configuration.group_property_id");
  }
  readStringArray(board.group_order, "database board configuration.group_order");
  readStringArray(board.hidden_group_ids, "database board configuration.hidden_group_ids");
  readStringArray(board.visible_property_ids, "database board configuration.visible_property_ids");
  const rowOpenMode = board.row_open_mode ?? "full_page";
  if (!isNotesBoardRowOpenMode(rowOpenMode)) {
    throw new Error("database board configuration.row_open_mode must be supported");
  }
}

function validateNotesGalleryConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "gallery") {
    throw new Error("database gallery configuration.type must be gallery");
  }
  const gallery = readRecord(value.gallery, "database gallery configuration.gallery");
  if (!isNotesGalleryCoverSource(gallery.cover_source ?? "page_cover")) {
    throw new Error("database gallery configuration.cover_source must be supported");
  }
  if (gallery.cover_property_id !== null && gallery.cover_property_id !== undefined) {
    readString(gallery.cover_property_id, "database gallery configuration.cover_property_id");
  }
  readStringArray(
    gallery.visible_property_ids,
    "database gallery configuration.visible_property_ids",
  );
  if (!isNotesGalleryCardSize(gallery.card_size ?? "medium")) {
    throw new Error("database gallery configuration.card_size must be supported");
  }
  if (typeof (gallery.fit_image ?? false) !== "boolean") {
    throw new Error("database gallery configuration.fit_image must be boolean");
  }
  if (!isNotesGalleryRowOpenMode(gallery.row_open_mode ?? "full_page")) {
    throw new Error("database gallery configuration.row_open_mode must be supported");
  }
}

function validateNotesListConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "list") {
    throw new Error("database list configuration.type must be list");
  }
  const list = readRecord(value.list, "database list configuration.list");
  if (list.group_property_id !== null && list.group_property_id !== undefined) {
    readString(list.group_property_id, "database list configuration.group_property_id");
  }
  readStringArray(list.group_order, "database list configuration.group_order");
  readStringArray(list.hidden_group_ids, "database list configuration.hidden_group_ids");
  readStringArray(list.visible_property_ids, "database list configuration.visible_property_ids");
  const rowOpenMode = list.row_open_mode ?? "side_panel";
  if (!isNotesListRowOpenMode(rowOpenMode)) {
    throw new Error("database list configuration.row_open_mode must be supported");
  }
}

function validateNotesCalendarConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "calendar") {
    throw new Error("database calendar configuration.type must be calendar");
  }
  const calendar = readRecord(value.calendar, "database calendar configuration.calendar");
  if (calendar.date_property_id !== null && calendar.date_property_id !== undefined) {
    readString(calendar.date_property_id, "database calendar configuration.date_property_id");
  }
  const rangeStart = readString(calendar.range_start, "database calendar configuration.range_start");
  const rangeEnd = readString(calendar.range_end, "database calendar configuration.range_end");
  if (!dateMentionBoundaryLooksIso(rangeStart) || rangeStart.length !== 10) {
    throw new Error("database calendar configuration.range_start must be an ISO date");
  }
  if (!dateMentionBoundaryLooksIso(rangeEnd) || rangeEnd.length !== 10) {
    throw new Error("database calendar configuration.range_end must be an ISO date");
  }
  if (rangeEnd < rangeStart) {
    throw new Error("database calendar configuration.range_end must be on or after range_start");
  }
  readStringArray(
    calendar.visible_property_ids,
    "database calendar configuration.visible_property_ids",
  );
  const rowOpenMode = calendar.row_open_mode ?? "side_panel";
  if (!isNotesCalendarRowOpenMode(rowOpenMode)) {
    throw new Error("database calendar configuration.row_open_mode must be supported");
  }
}

function validateNotesTimelineConfiguration(value: Record<string, unknown> | null): void {
  if (value === null) return;
  if (value.type !== undefined && value.type !== "timeline") {
    throw new Error("database timeline configuration.type must be timeline");
  }
  const timeline = readRecord(value.timeline, "database timeline configuration.timeline");
  if (timeline.date_property_id !== null && timeline.date_property_id !== undefined) {
    readString(timeline.date_property_id, "database timeline configuration.date_property_id");
  }
  if (timeline.group_property_id !== null && timeline.group_property_id !== undefined) {
    readString(timeline.group_property_id, "database timeline configuration.group_property_id");
  }
  readStringArray(timeline.group_order, "database timeline configuration.group_order");
  readStringArray(timeline.hidden_group_ids, "database timeline configuration.hidden_group_ids");
  const rangeStart = readString(timeline.range_start, "database timeline configuration.range_start");
  const rangeEnd = readString(timeline.range_end, "database timeline configuration.range_end");
  if (!dateMentionBoundaryLooksIso(rangeStart) || rangeStart.length !== 10) {
    throw new Error("database timeline configuration.range_start must be an ISO date");
  }
  if (!dateMentionBoundaryLooksIso(rangeEnd) || rangeEnd.length !== 10) {
    throw new Error("database timeline configuration.range_end must be an ISO date");
  }
  if (rangeEnd < rangeStart) {
    throw new Error("database timeline configuration.range_end must be on or after range_start");
  }
  readStringArray(
    timeline.visible_property_ids,
    "database timeline configuration.visible_property_ids",
  );
  const rowOpenMode = timeline.row_open_mode ?? "side_panel";
  if (!isNotesTimelineRowOpenMode(rowOpenMode)) {
    throw new Error("database timeline configuration.row_open_mode must be supported");
  }
}

function parseNotesDataSourceBoardGroup(
  value: unknown,
  label: string,
): NotesDataSourceBoardGroup {
  const record = readRecord(value, label);
  const rows = Array.isArray(record.rows)
    ? record.rows.map(parseNotesPage)
    : null;
  if (rows === null) throw new Error(`${label}.rows must be an array`);
  return {
    id: readString(record.id, `${label}.id`),
    name: readString(record.name, `${label}.name`),
    color: readString(record.color, `${label}.color`),
    hidden: readBoolean(record.hidden, `${label}.hidden`),
    rows,
  };
}

export function parseNotesDataSourceTableView(value: unknown): NotesDataSourceTableView {
  const record = readRecord(value, "data source table view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "table") {
    throw new Error("data source table view.view.type must be table");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source table view ids must match");
  }
  validateNotesTableConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) throw new Error("data source table view.rows must be an array");
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
  };
}

export function parseNotesDataSourceBoardView(value: unknown): NotesDataSourceBoardView {
  const record = readRecord(value, "data source board view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "board") {
    throw new Error("data source board view.view.type must be board");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source board view ids must match");
  }
  validateNotesBoardConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.groups)) {
    throw new Error("data source board view.groups must be an array");
  }
  return {
    data_source: dataSource,
    view,
    groups: record.groups.map((group, index) =>
      parseNotesDataSourceBoardGroup(group, `data source board view.groups[${index}]`)
    ),
  };
}

export function parseNotesDataSourceGalleryView(value: unknown): NotesDataSourceGalleryView {
  const record = readRecord(value, "data source gallery view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "gallery") {
    throw new Error("data source gallery view.view.type must be gallery");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source gallery view ids must match");
  }
  validateNotesGalleryConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) {
    throw new Error("data source gallery view.rows must be an array");
  }
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
  };
}

export function parseNotesDataSourceListView(value: unknown): NotesDataSourceListView {
  const record = readRecord(value, "data source list view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "list") {
    throw new Error("data source list view.view.type must be list");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source list view ids must match");
  }
  validateNotesListConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) {
    throw new Error("data source list view.rows must be an array");
  }
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
  };
}

export function parseNotesDataSourceCalendarView(value: unknown): NotesDataSourceCalendarView {
  const record = readRecord(value, "data source calendar view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "calendar") {
    throw new Error("data source calendar view.view.type must be calendar");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source calendar view ids must match");
  }
  validateNotesCalendarConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) {
    throw new Error("data source calendar view.rows must be an array");
  }
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
  };
}

export function parseNotesDataSourceTimelineView(value: unknown): NotesDataSourceTimelineView {
  const record = readRecord(value, "data source timeline view");
  const dataSource = parseNotesDataSource(record.data_source);
  const view = parseNotesDatabaseView(record.view);
  if (view.type !== "timeline") {
    throw new Error("data source timeline view.view.type must be timeline");
  }
  if (view.data_source_id !== dataSource.id) {
    throw new Error("data source timeline view ids must match");
  }
  validateNotesTimelineConfiguration(view.configuration);
  validateNotesTableFilter(view.filter);
  validateNotesTableSorts(view.sorts);
  if (!Array.isArray(record.rows)) {
    throw new Error("data source timeline view.rows must be an array");
  }
  return {
    data_source: dataSource,
    view,
    rows: record.rows.map(parseNotesPage),
  };
}

export function parseNotesBlock(value: unknown): NotesBlock {
  const record = readRecord(value, "block");
  if (record.object !== "block") throw new Error("block.object must be block");
  const type = readString(record.type, "block.type");
  if (!isNotesBlockType(type)) throw new Error(`unsupported block type: ${type}`);
  const base = {
    object: "block" as const,
    id: readString(record.id, "block.id"),
    parent: parseNotesParent(record.parent),
    created_time: readString(record.created_time, "block.created_time"),
    last_edited_time: readString(record.last_edited_time, "block.last_edited_time"),
    has_children: readBoolean(record.has_children, "block.has_children"),
    in_trash: readBoolean(record.in_trash, "block.in_trash"),
    archived: typeof record.archived === "boolean" ? record.archived : undefined,
    source_provider: readNullableString(record.source_provider, "block.source_provider"),
    source_object_id: readNullableString(record.source_object_id, "block.source_object_id"),
    source_last_edited_time: readNullableString(
      record.source_last_edited_time,
      "block.source_last_edited_time",
    ),
  };
  switch (type) {
    case "paragraph":
      return { ...base, type, paragraph: parseTextPayload(record.paragraph, "block.paragraph") };
    case "heading_1":
      return { ...base, type, heading_1: parseTextPayload(record.heading_1, "block.heading_1") };
    case "heading_2":
      return { ...base, type, heading_2: parseTextPayload(record.heading_2, "block.heading_2") };
    case "heading_3":
      return { ...base, type, heading_3: parseTextPayload(record.heading_3, "block.heading_3") };
    case "heading_4":
      return { ...base, type, heading_4: parseTextPayload(record.heading_4, "block.heading_4") };
    case "bulleted_list_item":
      return {
        ...base,
        type,
        bulleted_list_item: parseTextPayload(
          record.bulleted_list_item,
          "block.bulleted_list_item",
        ),
      };
    case "numbered_list_item":
      return {
        ...base,
        type,
        numbered_list_item: parseTextPayload(
          record.numbered_list_item,
          "block.numbered_list_item",
        ),
      };
    case "to_do":
      return { ...base, type, to_do: parseTodoPayload(record.to_do, "block.to_do") };
    case "toggle":
      return { ...base, type, toggle: parseTogglePayload(record.toggle, "block.toggle") };
    case "callout":
      return { ...base, type, callout: parseCalloutPayload(record.callout, "block.callout") };
    case "quote":
      return { ...base, type, quote: parseTextPayload(record.quote, "block.quote") };
    case "child_page":
      return {
        ...base,
        type,
        child_page: parseChildPagePayload(record.child_page, "block.child_page"),
      };
    case "child_database":
      return {
        ...base,
        type,
        child_database: parseChildDatabasePayload(
          record.child_database,
          "block.child_database",
        ),
      };
    case "breadcrumb":
      return { ...base, type, breadcrumb: readRecord(record.breadcrumb, "block.breadcrumb") };
    case "table_of_contents":
      return {
        ...base,
        type,
        table_of_contents: parseTableOfContentsPayload(
          record.table_of_contents,
          "block.table_of_contents",
        ),
      };
    case "column_list":
      return { ...base, type, column_list: readRecord(record.column_list, "block.column_list") };
    case "column":
      return { ...base, type, column: parseColumnPayload(record.column, "block.column") };
    case "table":
      return { ...base, type, table: parseTablePayload(record.table, "block.table") };
    case "table_row":
      return {
        ...base,
        type,
        table_row: parseTableRowPayload(record.table_row, "block.table_row"),
      };
    case "tab":
      return { ...base, type, tab: parseTabPayload(record.tab, "block.tab") };
    case "image":
      return { ...base, type, image: parseMediaPayload(record.image, "block.image", type) };
    case "video":
      return { ...base, type, video: parseMediaPayload(record.video, "block.video", type) };
    case "audio":
      return { ...base, type, audio: parseMediaPayload(record.audio, "block.audio", type) };
    case "file":
      return { ...base, type, file: parseMediaPayload(record.file, "block.file", type) };
    case "pdf":
      return { ...base, type, pdf: parseMediaPayload(record.pdf, "block.pdf", type) };
    case "bookmark":
      return { ...base, type, bookmark: parseBookmarkPayload(record.bookmark, "block.bookmark") };
    case "link_preview":
      return {
        ...base,
        type,
        link_preview: parseLinkPreviewPayload(record.link_preview, "block.link_preview"),
      };
    case "synced_block":
      return {
        ...base,
        type,
        synced_block: parseSyncedBlockPayload(record.synced_block, "block.synced_block"),
      };
    case "template":
      return { ...base, type, template: parseTemplatePayload(record.template, "block.template") };
    case "button":
      return { ...base, type, button: parseButtonPayload(record.button, "block.button") };
    case "embed":
      return { ...base, type, embed: parseEmbedPayload(record.embed, "block.embed") };
    case "equation":
      return { ...base, type, equation: parseEquationPayload(record.equation, "block.equation") };
    case "divider":
      return { ...base, type, divider: readRecord(record.divider, "block.divider") };
    case "code":
      return { ...base, type, code: parseCodePayload(record.code, "block.code") };
    case "unsupported":
      return {
        ...base,
        type,
        unsupported: parseUnsupportedPayload(record.unsupported, "block.unsupported"),
      };
  }
}

export function parseNotesPaginatedBlockList(value: unknown): NotesPaginatedBlockList {
  const record = readRecord(value, "block list");
  if (record.object !== "list") throw new Error("block list.object must be list");
  if (record.type !== "block") throw new Error("block list.type must be block");
  if (!Array.isArray(record.results)) throw new Error("block list.results must be an array");
  return {
    object: "list",
    type: "block",
    block: readRecord(record.block, "block list.block"),
    results: record.results.map(parseNotesBlock),
    next_cursor: readNullableString(record.next_cursor, "block list.next_cursor"),
    has_more: readBoolean(record.has_more, "block list.has_more"),
  };
}

export function parseNotesLoadedPage(value: unknown): NotesLoadedPage {
  const record = readRecord(value, "loaded page");
  return {
    page: parseNotesPage(record.page),
    blocks: parseNotesPaginatedBlockList(record.blocks),
  };
}

export function parseNotesMarkdownImportResult(value: unknown): NotesMarkdownImportResult {
  const record = readRecord(value, "markdown import result");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("markdown import result.diagnostics must be an array");
  }
  const importedBlockCount = readInteger(
    record.imported_block_count,
    "markdown import result.imported_block_count",
  );
  if (importedBlockCount < 0) {
    throw new Error("markdown import result.imported_block_count must not be negative");
  }
  return {
    page: parseNotesLoadedPage(record.page),
    diagnostics: record.diagnostics.map(parseNotesMarkdownImportDiagnostic),
    imported_block_count: importedBlockCount,
  };
}

function parseNotesMarkdownImportDiagnostic(
  value: unknown,
  index: number,
): NotesMarkdownImportDiagnostic {
  const record = readRecord(value, `markdown import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `markdown import result.diagnostics[${index}].severity`,
  );
  if (!isMarkdownImportDiagnosticSeverity(severity)) {
    throw new Error(`markdown import result.diagnostics[${index}].severity is unsupported`);
  }
  const line = record.line === null
    ? null
    : readInteger(record.line, `markdown import result.diagnostics[${index}].line`);
  if (line !== null && line < 1) {
    throw new Error(`markdown import result.diagnostics[${index}].line must be positive`);
  }
  return {
    code: readString(record.code, `markdown import result.diagnostics[${index}].code`),
    severity,
    line,
    message: readString(record.message, `markdown import result.diagnostics[${index}].message`),
  };
}

function isMarkdownImportDiagnosticSeverity(
  value: string,
): value is NotesMarkdownImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesHtmlImportResult(value: unknown): NotesHtmlImportResult {
  const record = readRecord(value, "HTML import result");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("HTML import result.diagnostics must be an array");
  }
  const importedBlockCount = readInteger(
    record.imported_block_count,
    "HTML import result.imported_block_count",
  );
  if (importedBlockCount < 0) {
    throw new Error("HTML import result.imported_block_count must not be negative");
  }
  return {
    page: parseNotesLoadedPage(record.page),
    diagnostics: record.diagnostics.map(parseNotesHtmlImportDiagnostic),
    imported_block_count: importedBlockCount,
  };
}

function parseNotesHtmlImportDiagnostic(
  value: unknown,
  index: number,
): NotesHtmlImportDiagnostic {
  const record = readRecord(value, `HTML import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `HTML import result.diagnostics[${index}].severity`,
  );
  if (!isHtmlImportDiagnosticSeverity(severity)) {
    throw new Error(`HTML import result.diagnostics[${index}].severity is unsupported`);
  }
  const line = record.line === null
    ? null
    : readInteger(record.line, `HTML import result.diagnostics[${index}].line`);
  if (line !== null && line < 1) {
    throw new Error(`HTML import result.diagnostics[${index}].line must be positive`);
  }
  return {
    code: readString(record.code, `HTML import result.diagnostics[${index}].code`),
    severity,
    line,
    message: readString(record.message, `HTML import result.diagnostics[${index}].message`),
  };
}

function isHtmlImportDiagnosticSeverity(
  value: string,
): value is NotesHtmlImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesNotionApiImportResult(value: unknown): NotesNotionApiImportResult {
  const record = readRecord(value, "Notion API import result");
  const object = readString(record.object, "Notion API import result.object");
  if (object !== "notes_notion_api_import") {
    throw new Error("Notion API import result.object is unsupported");
  }
  if (!Array.isArray(record.imported_pages)) {
    throw new Error("Notion API import result.imported_pages must be an array");
  }
  if (!Array.isArray(record.imported_data_sources)) {
    throw new Error("Notion API import result.imported_data_sources must be an array");
  }
  if (!Array.isArray(record.imported_users)) {
    throw new Error("Notion API import result.imported_users must be an array");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("Notion API import result.diagnostics must be an array");
  }
  return {
    object,
    imported_pages: record.imported_pages.map(parseNotesLoadedPage),
    imported_data_sources: record.imported_data_sources.map(parseNotesNotionApiImportedObject),
    imported_users: record.imported_users.map(parseNotesNotionApiImportedUser),
    diagnostics: record.diagnostics.map(parseNotesNotionApiImportDiagnostic),
    request_count: readNonNegativeInteger(record.request_count, "Notion API import result.request_count"),
    retry_count: readNonNegativeInteger(record.retry_count, "Notion API import result.retry_count"),
    rate_limit_count: readNonNegativeInteger(
      record.rate_limit_count,
      "Notion API import result.rate_limit_count",
    ),
    imported_page_count: readNonNegativeInteger(
      record.imported_page_count,
      "Notion API import result.imported_page_count",
    ),
    imported_block_count: readNonNegativeInteger(
      record.imported_block_count,
      "Notion API import result.imported_block_count",
    ),
    imported_data_source_count: readNonNegativeInteger(
      record.imported_data_source_count,
      "Notion API import result.imported_data_source_count",
    ),
    imported_comment_count: readNonNegativeInteger(
      record.imported_comment_count,
      "Notion API import result.imported_comment_count",
    ),
    imported_user_count: readNonNegativeInteger(
      record.imported_user_count,
      "Notion API import result.imported_user_count",
    ),
    imported_file_count: readNonNegativeInteger(
      record.imported_file_count,
      "Notion API import result.imported_file_count",
    ),
    unsupported_block_count: readNonNegativeInteger(
      record.unsupported_block_count,
      "Notion API import result.unsupported_block_count",
    ),
  };
}

function parseNotesNotionApiImportDiagnostic(
  value: unknown,
  index: number,
): NotesNotionApiImportDiagnostic {
  const record = readRecord(value, `Notion API import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `Notion API import result.diagnostics[${index}].severity`,
  );
  if (!isNotionApiImportDiagnosticSeverity(severity)) {
    throw new Error(`Notion API import result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `Notion API import result.diagnostics[${index}].code`),
    severity,
    source_object_id: readNullableString(
      record.source_object_id,
      `Notion API import result.diagnostics[${index}].source_object_id`,
    ),
    message: readString(record.message, `Notion API import result.diagnostics[${index}].message`),
  };
}

function parseNotesNotionApiImportedObject(
  value: unknown,
  index: number,
): NotesNotionApiImportedObject {
  const record = readRecord(value, `Notion API import result.imported_data_sources[${index}]`);
  return {
    object_type: readString(record.object_type, `Notion API import result.imported_data_sources[${index}].object_type`),
    source_object_id: readString(
      record.source_object_id,
      `Notion API import result.imported_data_sources[${index}].source_object_id`,
    ),
    local_id: readUuidString(record.local_id, `Notion API import result.imported_data_sources[${index}].local_id`),
    title: readString(record.title, `Notion API import result.imported_data_sources[${index}].title`),
  };
}

function parseNotesNotionApiImportedUser(value: unknown, index: number): NotesNotionApiImportedUser {
  const record = readRecord(value, `Notion API import result.imported_users[${index}]`);
  return {
    source_user_id: readString(record.source_user_id, `Notion API import result.imported_users[${index}].source_user_id`),
    name: readString(record.name, `Notion API import result.imported_users[${index}].name`),
    user_type: readString(record.user_type, `Notion API import result.imported_users[${index}].user_type`),
  };
}

function isNotionApiImportDiagnosticSeverity(
  value: string,
): value is NotesNotionApiImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesNotionExportImportResult(
  value: unknown,
): NotesNotionExportImportResult {
  const record = readRecord(value, "Notion export import result");
  const object = readString(record.object, "Notion export import result.object");
  if (object !== "notes_notion_export_import") {
    throw new Error("Notion export import result.object must be notes_notion_export_import");
  }
  if (!Array.isArray(record.imported_pages)) {
    throw new Error("Notion export import result.imported_pages must be an array");
  }
  if (!Array.isArray(record.imported_data_sources)) {
    throw new Error("Notion export import result.imported_data_sources must be an array");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("Notion export import result.diagnostics must be an array");
  }
  return {
    object,
    imported_pages: record.imported_pages.map(parseNotesLoadedPage),
    imported_data_sources: record.imported_data_sources.map(parseNotesNotionApiImportedObject),
    diagnostics: record.diagnostics.map(parseNotesNotionExportImportDiagnostic),
    imported_page_count: readNonNegativeInteger(
      record.imported_page_count,
      "Notion export import result.imported_page_count",
    ),
    imported_block_count: readNonNegativeInteger(
      record.imported_block_count,
      "Notion export import result.imported_block_count",
    ),
    imported_data_source_count: readNonNegativeInteger(
      record.imported_data_source_count,
      "Notion export import result.imported_data_source_count",
    ),
    imported_file_count: readNonNegativeInteger(
      record.imported_file_count,
      "Notion export import result.imported_file_count",
    ),
    skipped_file_count: readNonNegativeInteger(
      record.skipped_file_count,
      "Notion export import result.skipped_file_count",
    ),
    unsupported_block_count: readNonNegativeInteger(
      record.unsupported_block_count,
      "Notion export import result.unsupported_block_count",
    ),
  };
}

function parseNotesNotionExportImportDiagnostic(
  value: unknown,
  index: number,
): NotesNotionExportImportDiagnostic {
  const record = readRecord(value, `Notion export import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `Notion export import result.diagnostics[${index}].severity`,
  );
  if (!isNotionExportImportDiagnosticSeverity(severity)) {
    throw new Error(`Notion export import result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `Notion export import result.diagnostics[${index}].code`),
    severity,
    source_path: readNullableString(
      record.source_path,
      `Notion export import result.diagnostics[${index}].source_path`,
    ),
    message: readString(record.message, `Notion export import result.diagnostics[${index}].message`),
  };
}

function isNotionExportImportDiagnosticSeverity(
  value: string,
): value is NotesNotionExportImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesMarkdownExportResult(value: unknown): NotesMarkdownExportResult {
  const record = readRecord(value, "markdown export result");
  if (record.object !== "notes_markdown_export") {
    throw new Error("markdown export result.object must be notes_markdown_export");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("markdown export result.diagnostics must be an array");
  }
  const exportedBlockCount = readInteger(
    record.exported_block_count,
    "markdown export result.exported_block_count",
  );
  if (exportedBlockCount < 0) {
    throw new Error("markdown export result.exported_block_count must not be negative");
  }
  const exportedCommentCount = readInteger(
    record.exported_comment_count,
    "markdown export result.exported_comment_count",
  );
  if (exportedCommentCount < 0) {
    throw new Error("markdown export result.exported_comment_count must not be negative");
  }
  return {
    object: "notes_markdown_export",
    page_id: readUuidString(record.page_id, "markdown export result.page_id"),
    markdown: readString(record.markdown, "markdown export result.markdown"),
    diagnostics: record.diagnostics.map(parseNotesMarkdownExportDiagnostic),
    exported_block_count: exportedBlockCount,
    exported_comment_count: exportedCommentCount,
  };
}

function parseNotesMarkdownExportDiagnostic(
  value: unknown,
  index: number,
): NotesMarkdownExportDiagnostic {
  const record = readRecord(value, `markdown export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `markdown export result.diagnostics[${index}].severity`,
  );
  if (!isMarkdownExportDiagnosticSeverity(severity)) {
    throw new Error(`markdown export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `markdown export result.diagnostics[${index}].code`),
    severity,
    block_id: readNullableUuidString(
      record.block_id,
      `markdown export result.diagnostics[${index}].block_id`,
    ),
    comment_id: readNullableUuidString(
      record.comment_id,
      `markdown export result.diagnostics[${index}].comment_id`,
    ),
    message: readString(record.message, `markdown export result.diagnostics[${index}].message`),
  };
}

function isMarkdownExportDiagnosticSeverity(
  value: string,
): value is NotesMarkdownExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesHtmlExportResult(value: unknown): NotesHtmlExportResult {
  const record = readRecord(value, "HTML export result");
  if (record.object !== "notes_html_archive_export") {
    throw new Error("HTML export result.object must be notes_html_archive_export");
  }
  if (!Array.isArray(record.files)) throw new Error("HTML export result.files must be an array");
  if (!Array.isArray(record.assets)) throw new Error("HTML export result.assets must be an array");
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("HTML export result.diagnostics must be an array");
  }
  return {
    object: "notes_html_archive_export",
    root_page_id: readUuidString(record.root_page_id, "HTML export result.root_page_id"),
    files: record.files.map(parseNotesHtmlExportFile),
    assets: record.assets.map(parseNotesHtmlExportAsset),
    diagnostics: record.diagnostics.map(parseNotesHtmlExportDiagnostic),
    manifest_json: readString(record.manifest_json, "HTML export result.manifest_json"),
    exported_page_count: readNonNegativeInteger(
      record.exported_page_count,
      "HTML export result.exported_page_count",
    ),
    exported_block_count: readNonNegativeInteger(
      record.exported_block_count,
      "HTML export result.exported_block_count",
    ),
    exported_asset_count: readNonNegativeInteger(
      record.exported_asset_count,
      "HTML export result.exported_asset_count",
    ),
    exported_comment_count: readNonNegativeInteger(
      record.exported_comment_count,
      "HTML export result.exported_comment_count",
    ),
    exported_database_view_count: readNonNegativeInteger(
      record.exported_database_view_count,
      "HTML export result.exported_database_view_count",
    ),
  };
}

export function parseNotesHtmlArchiveSaveResult(value: unknown): NotesHtmlArchiveSaveResult {
  const record = readRecord(value, "HTML archive save result");
  if (record.object !== "notes_html_archive_save") {
    throw new Error("HTML archive save result.object must be notes_html_archive_save");
  }
  return {
    object: "notes_html_archive_save",
    saved: readBoolean(record.saved, "HTML archive save result.saved"),
    export: record.export === null
      ? null
      : parseNotesHtmlExportResult(record.export),
  };
}

function parseNotesHtmlExportFile(value: unknown, index: number): NotesHtmlExportFile {
  const record = readRecord(value, `HTML export result.files[${index}]`);
  return {
    path: readString(record.path, `HTML export result.files[${index}].path`),
    content_type: readString(
      record.content_type,
      `HTML export result.files[${index}].content_type`,
    ),
    contents: readString(record.contents, `HTML export result.files[${index}].contents`),
    byte_size: readNonNegativeInteger(
      record.byte_size,
      `HTML export result.files[${index}].byte_size`,
    ),
  };
}

function parseNotesHtmlExportAsset(value: unknown, index: number): NotesHtmlExportAsset {
  const record = readRecord(value, `HTML export result.assets[${index}]`);
  return {
    id: readString(record.id, `HTML export result.assets[${index}].id`),
    archive_path: readString(
      record.archive_path,
      `HTML export result.assets[${index}].archive_path`,
    ),
    source_path: readString(
      record.source_path,
      `HTML export result.assets[${index}].source_path`,
    ),
    content_type: readString(
      record.content_type,
      `HTML export result.assets[${index}].content_type`,
    ),
    byte_size: readNonNegativeInteger(
      record.byte_size,
      `HTML export result.assets[${index}].byte_size`,
    ),
    sha256: readString(record.sha256, `HTML export result.assets[${index}].sha256`),
    storage_state: readString(
      record.storage_state,
      `HTML export result.assets[${index}].storage_state`,
    ),
    exported: readBoolean(record.exported, `HTML export result.assets[${index}].exported`),
  };
}

function parseNotesHtmlExportDiagnostic(
  value: unknown,
  index: number,
): NotesHtmlExportDiagnostic {
  const record = readRecord(value, `HTML export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `HTML export result.diagnostics[${index}].severity`,
  );
  if (!isHtmlExportDiagnosticSeverity(severity)) {
    throw new Error(`HTML export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `HTML export result.diagnostics[${index}].code`),
    severity,
    page_id: readNullableUuidString(
      record.page_id,
      `HTML export result.diagnostics[${index}].page_id`,
    ),
    block_id: readNullableUuidString(
      record.block_id,
      `HTML export result.diagnostics[${index}].block_id`,
    ),
    asset_id: readNullableString(
      record.asset_id,
      `HTML export result.diagnostics[${index}].asset_id`,
    ),
    comment_id: readNullableUuidString(
      record.comment_id,
      `HTML export result.diagnostics[${index}].comment_id`,
    ),
    message: readString(record.message, `HTML export result.diagnostics[${index}].message`),
  };
}

function isHtmlExportDiagnosticSeverity(
  value: string,
): value is NotesHtmlExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesDataSourceCsvImportResult(
  value: unknown,
): NotesDataSourceCsvImportResult {
  const record = readRecord(value, "CSV import result");
  if (record.object !== "notes_data_source_csv_import") {
    throw new Error("CSV import result.object must be notes_data_source_csv_import");
  }
  if (!Array.isArray(record.imported_page_ids)) {
    throw new Error("CSV import result.imported_page_ids must be an array");
  }
  if (!Array.isArray(record.columns)) {
    throw new Error("CSV import result.columns must be an array");
  }
  if (!Array.isArray(record.rows)) {
    throw new Error("CSV import result.rows must be an array");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("CSV import result.diagnostics must be an array");
  }
  return {
    object: "notes_data_source_csv_import",
    data_source_id: readUuidString(record.data_source_id, "CSV import result.data_source_id"),
    dry_run: readBoolean(record.dry_run, "CSV import result.dry_run"),
    total_row_count: readNonNegativeInteger(
      record.total_row_count,
      "CSV import result.total_row_count",
    ),
    valid_row_count: readNonNegativeInteger(
      record.valid_row_count,
      "CSV import result.valid_row_count",
    ),
    skipped_row_count: readNonNegativeInteger(
      record.skipped_row_count,
      "CSV import result.skipped_row_count",
    ),
    imported_row_count: readNonNegativeInteger(
      record.imported_row_count,
      "CSV import result.imported_row_count",
    ),
    imported_page_ids: record.imported_page_ids.map((pageId, index) =>
      readUuidString(pageId, `CSV import result.imported_page_ids[${index}]`)
    ),
    columns: record.columns.map(parseNotesDataSourceCsvImportColumn),
    rows: record.rows.map(parseNotesDataSourceCsvImportRow),
    diagnostics: record.diagnostics.map(parseNotesDataSourceCsvImportDiagnostic),
  };
}

function parseNotesDataSourceCsvImportColumn(
  value: unknown,
  index: number,
): NotesDataSourceCsvImportColumn {
  const record = readRecord(value, `CSV import result.columns[${index}]`);
  const sourceIndex = readInteger(
    record.source_index,
    `CSV import result.columns[${index}].source_index`,
  );
  if (sourceIndex < 1) {
    throw new Error(`CSV import result.columns[${index}].source_index must be positive`);
  }
  return {
    source_index: sourceIndex,
    source_name: readString(record.source_name, `CSV import result.columns[${index}].source_name`),
    property_id: readNullableString(
      record.property_id,
      `CSV import result.columns[${index}].property_id`,
    ),
    property_name: readNullableString(
      record.property_name,
      `CSV import result.columns[${index}].property_name`,
    ),
    property_type: readNullableString(
      record.property_type,
      `CSV import result.columns[${index}].property_type`,
    ),
    mapped: readBoolean(record.mapped, `CSV import result.columns[${index}].mapped`),
    read_only: readBoolean(record.read_only, `CSV import result.columns[${index}].read_only`),
    warning: readNullableString(record.warning, `CSV import result.columns[${index}].warning`),
  };
}

function parseNotesDataSourceCsvImportRow(
  value: unknown,
  index: number,
): NotesDataSourceCsvImportRow {
  const record = readRecord(value, `CSV import result.rows[${index}]`);
  const rowNumber = readInteger(record.row_number, `CSV import result.rows[${index}].row_number`);
  if (rowNumber < 1) {
    throw new Error(`CSV import result.rows[${index}].row_number must be positive`);
  }
  return {
    row_number: rowNumber,
    title: readString(record.title, `CSV import result.rows[${index}].title`),
    valid: readBoolean(record.valid, `CSV import result.rows[${index}].valid`),
    mapped_cell_count: readNonNegativeInteger(
      record.mapped_cell_count,
      `CSV import result.rows[${index}].mapped_cell_count`,
    ),
    error_count: readNonNegativeInteger(
      record.error_count,
      `CSV import result.rows[${index}].error_count`,
    ),
  };
}

function parseNotesDataSourceCsvImportDiagnostic(
  value: unknown,
  index: number,
): NotesDataSourceCsvImportDiagnostic {
  const record = readRecord(value, `CSV import result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `CSV import result.diagnostics[${index}].severity`,
  );
  if (!isDataSourceCsvImportDiagnosticSeverity(severity)) {
    throw new Error(`CSV import result.diagnostics[${index}].severity is unsupported`);
  }
  const rowNumber = record.row_number === null
    ? null
    : readInteger(record.row_number, `CSV import result.diagnostics[${index}].row_number`);
  const columnIndex = record.column_index === null
    ? null
    : readInteger(record.column_index, `CSV import result.diagnostics[${index}].column_index`);
  if (rowNumber !== null && rowNumber < 1) {
    throw new Error(`CSV import result.diagnostics[${index}].row_number must be positive`);
  }
  if (columnIndex !== null && columnIndex < 1) {
    throw new Error(`CSV import result.diagnostics[${index}].column_index must be positive`);
  }
  return {
    code: readString(record.code, `CSV import result.diagnostics[${index}].code`),
    severity,
    row_number: rowNumber,
    column_index: columnIndex,
    column_name: readNullableString(
      record.column_name,
      `CSV import result.diagnostics[${index}].column_name`,
    ),
    property_id: readNullableString(
      record.property_id,
      `CSV import result.diagnostics[${index}].property_id`,
    ),
    message: readString(record.message, `CSV import result.diagnostics[${index}].message`),
  };
}

function isDataSourceCsvImportDiagnosticSeverity(
  value: string,
): value is NotesDataSourceCsvImportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

export function parseNotesDataSourceCsvExportResult(
  value: unknown,
): NotesDataSourceCsvExportResult {
  const record = readRecord(value, "CSV export result");
  if (record.object !== "notes_data_source_csv_export") {
    throw new Error("CSV export result.object must be notes_data_source_csv_export");
  }
  if (!Array.isArray(record.diagnostics)) {
    throw new Error("CSV export result.diagnostics must be an array");
  }
  const scope = readString(record.scope, "CSV export result.scope");
  if (!isDataSourceCsvExportScope(scope)) {
    throw new Error("CSV export result.scope is unsupported");
  }
  return {
    object: "notes_data_source_csv_export",
    data_source_id: readUuidString(record.data_source_id, "CSV export result.data_source_id"),
    database_id: readUuidString(record.database_id, "CSV export result.database_id"),
    view_id: readUuidString(record.view_id, "CSV export result.view_id"),
    scope,
    file_name: readString(record.file_name, "CSV export result.file_name"),
    csv: readString(record.csv, "CSV export result.csv"),
    exported_row_count: readNonNegativeInteger(
      record.exported_row_count,
      "CSV export result.exported_row_count",
    ),
    exported_property_count: readNonNegativeInteger(
      record.exported_property_count,
      "CSV export result.exported_property_count",
    ),
    diagnostics: record.diagnostics.map(parseNotesDataSourceCsvExportDiagnostic),
  };
}

export function parseNotesDataSourceCsvExportSaveResult(
  value: unknown,
): NotesDataSourceCsvExportSaveResult {
  const record = readRecord(value, "CSV export save result");
  const saved = readBoolean(record.saved, "CSV export save result.saved");
  const exportResult = record.export === null
    ? null
    : parseNotesDataSourceCsvExportResult(record.export);
  if (saved && exportResult === null) {
    throw new Error("CSV export save result.export is required when saved is true");
  }
  return {
    saved,
    export: exportResult,
  };
}

function parseNotesDataSourceCsvExportDiagnostic(
  value: unknown,
  index: number,
): NotesDataSourceCsvExportDiagnostic {
  const record = readRecord(value, `CSV export result.diagnostics[${index}]`);
  const severity = readString(
    record.severity,
    `CSV export result.diagnostics[${index}].severity`,
  );
  if (!isDataSourceCsvExportDiagnosticSeverity(severity)) {
    throw new Error(`CSV export result.diagnostics[${index}].severity is unsupported`);
  }
  return {
    code: readString(record.code, `CSV export result.diagnostics[${index}].code`),
    severity,
    property_id: readNullableString(
      record.property_id,
      `CSV export result.diagnostics[${index}].property_id`,
    ),
    property_name: readNullableString(
      record.property_name,
      `CSV export result.diagnostics[${index}].property_name`,
    ),
    message: readString(record.message, `CSV export result.diagnostics[${index}].message`),
  };
}

function isDataSourceCsvExportScope(value: string): value is NotesDataSourceCsvExportScope {
  return value === "view" || value === "all";
}

function isDataSourceCsvExportDiagnosticSeverity(
  value: string,
): value is NotesDataSourceCsvExportDiagnosticSeverity {
  return value === "info" || value === "warning" || value === "error";
}

function readNonNegativeInteger(value: unknown, label: string): number {
  const integer = readInteger(value, label);
  if (integer < 0) throw new Error(`${label} must not be negative`);
  return integer;
}

export function parseNotesSidebarPageList(value: unknown): NotesSidebarPageList {
  const record = readRecord(value, "sidebar page list");
  if (!Array.isArray(record.pages)) throw new Error("sidebar page list.pages must be an array");
  return {
    pages: record.pages.map(parseNotesPage),
    page_ids_with_children: readStringArray(
      record.page_ids_with_children,
      "sidebar page list.page_ids_with_children",
    ),
    missing_parent_page_ids: readStringArray(
      record.missing_parent_page_ids,
      "sidebar page list.missing_parent_page_ids",
    ),
    trashed_parent_page_ids: readStringArray(
      record.trashed_parent_page_ids,
      "sidebar page list.trashed_parent_page_ids",
    ),
  };
}

export function parseNotesPageTemplate(value: unknown): NotesPageTemplate {
  const record = readRecord(value, "page template");
  if (record.object !== "page_template") {
    throw new Error("page template.object must be page_template");
  }
  const blockCount = readInteger(record.block_count, "page template.block_count");
  if (blockCount < 0) throw new Error("page template.block_count must not be negative");
  return {
    object: "page_template",
    id: readString(record.id, "page template.id"),
    name: readDisplayString(record.name, "page template.name"),
    source_page_id: readNullableString(record.source_page_id, "page template.source_page_id"),
    properties: readRecord(record.properties, "page template.properties"),
    icon: parseNullableNotesIcon(record.icon, "page template.icon"),
    cover: parseNullablePageCover(record.cover, "page template.cover"),
    block_count: blockCount,
    created_time: readString(record.created_time, "page template.created_time"),
    last_edited_time: readString(record.last_edited_time, "page template.last_edited_time"),
  };
}

export function parseNotesDataSourceTemplate(value: unknown): NotesDataSourceTemplate {
  const record = readRecord(value, "data source template");
  if (record.object !== "data_source_template") {
    throw new Error("data source template.object must be data_source_template");
  }
  const blockCount = readInteger(record.block_count, "data source template.block_count");
  if (blockCount < 0) {
    throw new Error("data source template.block_count must not be negative");
  }
  return {
    object: "data_source_template",
    id: readString(record.id, "data source template.id"),
    data_source_id: readString(record.data_source_id, "data source template.data_source_id"),
    source_page_id: readNullableString(
      record.source_page_id,
      "data source template.source_page_id",
    ),
    name: readDisplayString(record.name, "data source template.name"),
    properties: readRecord(record.properties, "data source template.properties"),
    is_default: readBoolean(record.is_default, "data source template.is_default"),
    block_count: blockCount,
    created_time: readString(record.created_time, "data source template.created_time"),
    last_edited_time: readString(
      record.last_edited_time,
      "data source template.last_edited_time",
    ),
  };
}

export function parseNotesPageHistorySnapshot(value: unknown): NotesPageHistorySnapshot {
  const record = readRecord(value, "page history snapshot");
  if (record.object !== "page_history_snapshot") {
    throw new Error("page history snapshot.object must be page_history_snapshot");
  }
  const blockCount = readInteger(record.block_count, "page history snapshot.block_count");
  if (blockCount < 0) throw new Error("page history snapshot.block_count must not be negative");
  return {
    object: "page_history_snapshot",
    id: readString(record.id, "page history snapshot.id"),
    page_id: readString(record.page_id, "page history snapshot.page_id"),
    title: readString(record.title, "page history snapshot.title"),
    icon: parseNullableNotesIcon(record.icon, "page history snapshot.icon"),
    cover: parseNullablePageCover(record.cover, "page history snapshot.cover"),
    block_count: blockCount,
    reason: readDisplayString(record.reason, "page history snapshot.reason"),
    created_by: parseNotesPartialUser(
      record.created_by,
      "page history snapshot.created_by",
    ),
    created_time: readString(record.created_time, "page history snapshot.created_time"),
    page_last_edited_time: readString(
      record.page_last_edited_time,
      "page history snapshot.page_last_edited_time",
    ),
  };
}

export function parseNotesPageHistorySettings(value: unknown): NotesPageHistorySettings {
  const record = readRecord(value, "page history settings");
  if (record.object !== "page_history_settings") {
    throw new Error("page history settings.object must be page_history_settings");
  }
  const retentionDays = record.retention_days === null
    ? null
    : readInteger(record.retention_days, "page history settings.retention_days");
  if (retentionDays !== null && (retentionDays < 1 || retentionDays > 3650)) {
    throw new Error("page history settings.retention_days must be between 1 and 3650");
  }
  return {
    object: "page_history_settings",
    retention_days: retentionDays,
    updated_at: readString(record.updated_at, "page history settings.updated_at"),
  };
}

function parseNotesMentionNotificationKind(value: unknown): NotesMentionNotificationKind {
  const kind = readString(value, "mention notification.kind");
  if ((NOTES_MENTION_NOTIFICATION_KINDS as readonly string[]).includes(kind)) {
    return kind as NotesMentionNotificationKind;
  }
  throw new Error("mention notification.kind is unsupported");
}

function parseNotesMentionNotificationTargetType(
  value: unknown,
): NotesMentionNotificationTargetType {
  const targetType = readString(value, "mention notification.target_type");
  if ((NOTES_MENTION_NOTIFICATION_TARGET_TYPES as readonly string[]).includes(targetType)) {
    return targetType as NotesMentionNotificationTargetType;
  }
  throw new Error("mention notification.target_type is unsupported");
}

function parseNotesMentionNotificationStatus(value: unknown): NotesMentionNotificationStatus {
  const status = readString(value, "mention notification.status");
  if ((NOTES_MENTION_NOTIFICATION_STATUSES as readonly string[]).includes(status)) {
    return status as NotesMentionNotificationStatus;
  }
  throw new Error("mention notification.status is unsupported");
}

export function parseNotesMentionNotification(value: unknown): NotesMentionNotification {
  const record = readRecord(value, "mention notification");
  if (record.object !== "mention_notification") {
    throw new Error("mention notification.object must be mention_notification");
  }
  const sourceType = readString(record.source_type, "mention notification.source_type");
  if (sourceType !== "block" && sourceType !== "comment") {
    throw new Error("mention notification.source_type is unsupported");
  }
  return {
    object: "mention_notification",
    id: readString(record.id, "mention notification.id"),
    source_type: sourceType,
    source_id: readString(record.source_id, "mention notification.source_id"),
    page_id: readString(record.page_id, "mention notification.page_id"),
    page_title: readString(record.page_title, "mention notification.page_title"),
    block_id: readNullableString(record.block_id, "mention notification.block_id"),
    comment_id: readNullableString(record.comment_id, "mention notification.comment_id"),
    kind: parseNotesMentionNotificationKind(record.kind),
    target_type: parseNotesMentionNotificationTargetType(record.target_type),
    target_id: readNullableString(record.target_id, "mention notification.target_id"),
    trigger_at: readNullableString(record.trigger_at, "mention notification.trigger_at"),
    plain_text: readString(record.plain_text, "mention notification.plain_text"),
    source_plain_text: readString(
      record.source_plain_text,
      "mention notification.source_plain_text",
    ),
    status: parseNotesMentionNotificationStatus(record.status),
    delivered_at: readNullableString(record.delivered_at, "mention notification.delivered_at"),
    created_time: readString(record.created_time, "mention notification.created_time"),
    last_edited_time: readString(
      record.last_edited_time,
      "mention notification.last_edited_time",
    ),
  };
}

function parseBacklinkReferenceType(value: unknown): NotesBacklinkReferenceType {
  const referenceType = readString(value, "backlink.reference_type");
  if (
    referenceType === "child_page"
    || referenceType === "page_mention"
    || referenceType === "link"
    || referenceType === "comment_mention"
    || referenceType === "comment_link"
  ) {
    return referenceType;
  }
  if (referenceType === "database_relation") return referenceType;
  throw new Error(
    "backlink.reference_type must be child_page, page_mention, link, database_relation, comment_mention, or comment_link",
  );
}

function parsePageBreadcrumbStatus(value: unknown): NotesPageBreadcrumbStatus {
  const status = readString(value, "page breadcrumb.status");
  if (
    status === "workspace"
    || status === "active"
    || status === "archived"
    || status === "trashed"
    || status === "missing"
  ) {
    return status;
  }
  throw new Error("page breadcrumb.status must be workspace, active, archived, trashed, or missing");
}

function parseSearchResultType(value: unknown): NotesSearchResultType {
  const resultType = readString(value, "search_result.type");
  if (resultType === "page" || resultType === "block" || resultType === "comment") {
    return resultType;
  }
  throw new Error("search_result.type must be page, block, or comment");
}

function parseUnresolvedLinkSourceType(value: unknown): NotesUnresolvedLinkSourceType {
  const sourceType = readString(value, "unresolved_link.source_type");
  if (sourceType === "block" || sourceType === "comment") return sourceType;
  throw new Error("unresolved_link.source_type must be block or comment");
}

export function parseNotesPageBreadcrumbItem(value: unknown): NotesPageBreadcrumbItem {
  const record = readRecord(value, "page breadcrumb");
  const id = record.id === null ? null : readString(record.id, "page breadcrumb.id");
  return {
    id,
    title: readString(record.title, "page breadcrumb.title"),
    current: readBoolean(record.current, "page breadcrumb.current"),
    status: parsePageBreadcrumbStatus(record.status),
  };
}

export function parseNotesBacklink(value: unknown): NotesBacklink {
  const record = readRecord(value, "backlink");
  if (record.object !== "backlink") throw new Error("backlink.object must be backlink");
  const referenceType = parseBacklinkReferenceType(record.reference_type);
  const sourceBlockType = readString(record.source_block_type, "backlink.source_block_type");
  if (
    !isNotesBlockType(sourceBlockType)
    && !(referenceType === "database_relation" && sourceBlockType === "database_relation")
    && !(
      (referenceType === "comment_mention" || referenceType === "comment_link")
      && sourceBlockType === "comment"
    )
  ) {
    throw new Error("backlink.source_block_type must be a supported block type");
  }
  return {
    object: "backlink",
    id: readString(record.id, "backlink.id"),
    source_page: parseNotesPage(record.source_page),
    source_block_id: readString(record.source_block_id, "backlink.source_block_id"),
    source_block_type: sourceBlockType,
    reference_type: referenceType,
    snippet: readString(record.snippet, "backlink.snippet"),
    created_time: readString(record.created_time, "backlink.created_time"),
    last_edited_time: readString(record.last_edited_time, "backlink.last_edited_time"),
  };
}

export function parseNotesPageAlias(value: unknown): NotesPageAlias {
  const record = readRecord(value, "page alias");
  if (record.object !== "page_alias") throw new Error("page alias.object must be page_alias");
  return {
    object: "page_alias",
    id: readString(record.id, "page alias.id"),
    page_id: readString(record.page_id, "page alias.page_id"),
    alias: readDisplayString(record.alias, "page alias.alias"),
    normalized_alias: readDisplayString(record.normalized_alias, "page alias.normalized_alias"),
    created_time: readString(record.created_time, "page alias.created_time"),
    last_edited_time: readString(record.last_edited_time, "page alias.last_edited_time"),
  };
}

export function parseNotesUnresolvedLink(value: unknown): NotesUnresolvedLink {
  const record = readRecord(value, "unresolved_link");
  if (record.object !== "unresolved_link") {
    throw new Error("unresolved_link.object must be unresolved_link");
  }
  return {
    object: "unresolved_link",
    id: readString(record.id, "unresolved_link.id"),
    source_type: parseUnresolvedLinkSourceType(record.source_type),
    source_page_id: readString(record.source_page_id, "unresolved_link.source_page_id"),
    source_block_id: readNullableString(record.source_block_id, "unresolved_link.source_block_id"),
    source_comment_id: readNullableString(
      record.source_comment_id,
      "unresolved_link.source_comment_id",
    ),
    raw_url: readString(record.raw_url, "unresolved_link.raw_url"),
    raw_target: readDisplayString(record.raw_target, "unresolved_link.raw_target"),
    normalized_target: readDisplayString(
      record.normalized_target,
      "unresolved_link.normalized_target",
    ),
    link_text: readString(record.link_text, "unresolved_link.link_text"),
    snippet: readString(record.snippet, "unresolved_link.snippet"),
    created_time: readString(record.created_time, "unresolved_link.created_time"),
    last_edited_time: readString(record.last_edited_time, "unresolved_link.last_edited_time"),
  };
}

function parseCommentThreadStatus(value: unknown): NotesCommentThreadStatus {
  const status = readString(value, "comment thread.status");
  if (status === "open" || status === "resolved") return status;
  throw new Error("comment thread.status must be open or resolved");
}

function parseSuggestionStatus(value: unknown): NotesSuggestionStatus {
  const status = readString(value, "suggestion.status");
  if (status === "open" || status === "accepted" || status === "rejected") return status;
  throw new Error("suggestion.status must be open, accepted, or rejected");
}

function parseNotesCommentAnchor(value: unknown): NotesCommentAnchor {
  const record = readRecord(value, "comment anchor");
  if (record.object !== "comment_anchor") {
    throw new Error("comment anchor.object must be comment_anchor");
  }
  if (record.type !== "text_range") {
    throw new Error("comment anchor.type must be text_range");
  }
  const start = readInteger(record.start, "comment anchor.start");
  const end = readInteger(record.end, "comment anchor.end");
  if (start < 0 || end <= start) {
    throw new Error("comment anchor range must be non-empty");
  }
  const text = readString(record.text, "comment anchor.text");
  if (!text.trim()) {
    throw new Error("comment anchor.text must not be empty");
  }
  return {
    object: "comment_anchor",
    type: "text_range",
    block_id: readString(record.block_id, "comment anchor.block_id"),
    start,
    end,
    text,
    prefix: readString(record.prefix, "comment anchor.prefix"),
    suffix: readString(record.suffix, "comment anchor.suffix"),
    created_time: readString(record.created_time, "comment anchor.created_time"),
    last_edited_time: readString(record.last_edited_time, "comment anchor.last_edited_time"),
  };
}

export function parseNotesSearchResult(value: unknown): NotesSearchResult {
  const record = readRecord(value, "search_result");
  if (record.object !== "search_result") {
    throw new Error("search_result.object must be search_result");
  }
  const resultType = parseSearchResultType(record.type);
  const blockType = record.block_type === null
    ? null
    : readString(record.block_type, "search_result.block_type");
  if (blockType !== null && !isNotesBlockType(blockType)) {
    throw new Error("search_result.block_type must be a supported block type");
  }
  return {
    object: "search_result",
    id: readString(record.id, "search_result.id"),
    type: resultType,
    page: parseNotesPage(record.page),
    block_id: readNullableString(record.block_id, "search_result.block_id"),
    block_type: blockType,
    comment_id: readNullableString(record.comment_id, "search_result.comment_id"),
    discussion_id: readNullableString(record.discussion_id, "search_result.discussion_id"),
    comment_status: record.comment_status === null
      ? null
      : parseCommentThreadStatus(record.comment_status),
    comment_author: record.comment_author === null
      ? null
      : parseNotesCommentDisplayName(record.comment_author, "search_result.comment_author"),
    comment_anchor: record.comment_anchor === null
      ? null
      : parseNotesCommentAnchor(record.comment_anchor),
    snippet: readString(record.snippet, "search_result.snippet"),
    last_edited_time: readString(record.last_edited_time, "search_result.last_edited_time"),
  };
}

export function parseNotesComment(value: unknown): NotesComment {
  const record = readRecord(value, "comment");
  if (record.object !== "comment") throw new Error("comment.object must be comment");
  return {
    object: "comment",
    id: readString(record.id, "comment.id"),
    parent: parseNotesCommentParent(record.parent, "comment.parent"),
    discussion_id: readString(record.discussion_id, "comment.discussion_id"),
    created_time: readString(record.created_time, "comment.created_time"),
    last_edited_time: readString(record.last_edited_time, "comment.last_edited_time"),
    created_by: parseNotesPartialUser(record.created_by, "comment.created_by"),
    rich_text: parseNotesRichTextArray(record.rich_text, "comment.rich_text"),
    attachments: parseCommentAttachments(record.attachments, "comment.attachments"),
    display_name: parseNotesCommentDisplayName(record.display_name, "comment.display_name"),
    deleted_at: readNullableString(record.deleted_at, "comment.deleted_at"),
  };
}

export function parseNotesCommentThread(value: unknown): NotesCommentThread {
  const record = readRecord(value, "comment thread");
  if (record.object !== "comment_thread") {
    throw new Error("comment thread.object must be comment_thread");
  }
  if (!Array.isArray(record.comments)) {
    throw new Error("comment thread.comments must be an array");
  }
  return {
    object: "comment_thread",
    id: readString(record.id, "comment thread.id"),
    parent: parseNotesCommentParent(record.parent, "comment thread.parent"),
    page_id: readString(record.page_id, "comment thread.page_id"),
    block_id: readNullableString(record.block_id, "comment thread.block_id"),
    status: parseCommentThreadStatus(record.status),
    resolved_at: readNullableString(record.resolved_at, "comment thread.resolved_at"),
    resolved_by: record.resolved_by === null
      ? null
      : parseNotesPartialUser(record.resolved_by, "comment thread.resolved_by"),
    anchor: record.anchor === null
      ? null
      : parseNotesCommentAnchor(record.anchor),
    created_time: readString(record.created_time, "comment thread.created_time"),
    last_edited_time: readString(record.last_edited_time, "comment thread.last_edited_time"),
    unread: readBoolean(record.unread, "comment thread.unread"),
    comments: record.comments.map(parseNotesComment),
  };
}

export function parseNotesSuggestion(value: unknown): NotesSuggestion {
  const record = readRecord(value, "suggestion");
  if (record.object !== "suggestion") throw new Error("suggestion.object must be suggestion");
  const rangeStart = readInteger(record.range_start, "suggestion.range_start");
  const rangeEnd = readInteger(record.range_end, "suggestion.range_end");
  if (rangeStart < 0 || rangeEnd <= rangeStart) {
    throw new Error("suggestion range must be non-empty");
  }
  const originalText = readString(record.original_text, "suggestion.original_text");
  if (!originalText.trim()) throw new Error("suggestion.original_text must not be empty");
  return {
    object: "suggestion",
    id: readString(record.id, "suggestion.id"),
    page_id: readString(record.page_id, "suggestion.page_id"),
    block_id: readString(record.block_id, "suggestion.block_id"),
    created_by: parseNotesPartialUser(record.created_by, "suggestion.created_by"),
    display_name: parseNotesCommentDisplayName(record.display_name, "suggestion.display_name"),
    status: parseSuggestionStatus(record.status),
    range_start: rangeStart,
    range_end: rangeEnd,
    original_text: originalText,
    proposed_text: readString(record.proposed_text, "suggestion.proposed_text"),
    prefix: readString(record.prefix, "suggestion.prefix"),
    suffix: readString(record.suffix, "suggestion.suffix"),
    accepted_at: readNullableString(record.accepted_at, "suggestion.accepted_at"),
    accepted_by: record.accepted_by === null
      ? null
      : parseNotesPartialUser(record.accepted_by, "suggestion.accepted_by"),
    rejected_at: readNullableString(record.rejected_at, "suggestion.rejected_at"),
    rejected_by: record.rejected_by === null
      ? null
      : parseNotesPartialUser(record.rejected_by, "suggestion.rejected_by"),
    created_time: readString(record.created_time, "suggestion.created_time"),
    last_edited_time: readString(record.last_edited_time, "suggestion.last_edited_time"),
  };
}
