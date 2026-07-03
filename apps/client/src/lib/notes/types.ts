export const NOTES_BLOCK_TYPES = [
  "paragraph",
  "heading_1",
  "heading_2",
  "heading_3",
  "heading_4",
  "bulleted_list_item",
  "numbered_list_item",
  "to_do",
  "toggle",
  "callout",
  "quote",
  "child_page",
  "child_database",
  "breadcrumb",
  "table_of_contents",
  "column_list",
  "column",
  "table",
  "table_row",
  "tab",
  "image",
  "video",
  "audio",
  "file",
  "pdf",
  "bookmark",
  "link_preview",
  "synced_block",
  "template",
  "button",
  "embed",
  "equation",
  "divider",
  "code",
  "unsupported",
] as const;

export type NotesBlockType = (typeof NOTES_BLOCK_TYPES)[number];

export const NOTES_COLORS = [
  "default",
  "gray",
  "brown",
  "orange",
  "yellow",
  "green",
  "blue",
  "purple",
  "pink",
  "red",
  "gray_background",
  "brown_background",
  "orange_background",
  "yellow_background",
  "green_background",
  "blue_background",
  "purple_background",
  "pink_background",
  "red_background",
] as const;

export type NotesColor = (typeof NOTES_COLORS)[number];

export type NotesTextBlockType =
  | "paragraph"
  | "heading_1"
  | "heading_2"
  | "heading_3"
  | "heading_4"
  | "bulleted_list_item"
  | "numbered_list_item"
  | "toggle"
  | "callout"
  | "quote";

export type NotesParent =
  | { type: "workspace"; workspace: true }
  | { type: "page_id"; page_id: string }
  | { type: "block_id"; block_id: string }
  | { type: "data_source_id"; data_source_id: string };

export interface NotesRichTextLink {
  url: string;
}

export interface NotesRichTextAnnotations {
  bold: boolean;
  italic: boolean;
  strikethrough: boolean;
  underline: boolean;
  code: boolean;
  color: NotesColor;
}

export interface NotesTextRichText {
  type: "text";
  text: {
    content: string;
    link: NotesRichTextLink | null;
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export interface NotesPageMentionRichText {
  type: "mention";
  mention: {
    type: "page";
    page: {
      id: string;
    };
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export interface NotesUserMentionRichText {
  type: "mention";
  mention: {
    type: "user";
    user: {
      object: "user";
      id: string;
    };
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export interface NotesDatabaseMentionRichText {
  type: "mention";
  mention: {
    type: "database";
    database: {
      id: string;
    };
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export const NOTES_LOCAL_OBJECT_MENTION_TYPES = [
  "project",
  "project_task",
  "calendar_event",
  "pomodoro_run",
  "music_item",
] as const;

export type NotesLocalObjectMentionType =
  (typeof NOTES_LOCAL_OBJECT_MENTION_TYPES)[number];

export interface NotesLocalObjectMentionRichText {
  type: "mention";
  mention: {
    type: "ganbaru_object";
    ganbaru_object: {
      type: NotesLocalObjectMentionType;
      id: string;
    };
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export interface NotesDateMentionReminder {
  enabled: boolean;
}

export interface NotesDateMentionValue {
  start: string;
  end?: string | null;
  time_zone?: string | null;
  ganbaru_reminder?: NotesDateMentionReminder | null;
}

export interface NotesDateMentionRichText {
  type: "mention";
  mention: {
    type: "date";
    date: NotesDateMentionValue;
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export interface NotesEquationRichText {
  type: "equation";
  equation: {
    expression: string;
  };
  annotations: NotesRichTextAnnotations;
  plain_text: string;
  href: string | null;
}

export type NotesRichText =
  | NotesTextRichText
  | NotesPageMentionRichText
  | NotesUserMentionRichText
  | NotesDatabaseMentionRichText
  | NotesLocalObjectMentionRichText
  | NotesDateMentionRichText
  | NotesEquationRichText;

export interface NotesTextBlockPayload {
  rich_text: NotesRichText[];
  color?: NotesColor;
  is_toggleable?: boolean;
  ganbaru_open?: boolean;
  icon?: NotesIcon | null;
}

export interface NotesTodoBlockPayload extends NotesTextBlockPayload {
  checked: boolean;
}

export interface NotesToggleBlockPayload extends NotesTextBlockPayload {
  ganbaru_open?: boolean;
}

export const NOTES_ICON_COLORS = [
  "gray",
  "lightgray",
  "brown",
  "yellow",
  "orange",
  "green",
  "blue",
  "purple",
  "pink",
  "red",
] as const;

export type NotesIconColor = (typeof NOTES_ICON_COLORS)[number];

export interface NotesCustomEmojiIcon {
  id: string;
  name?: string;
  url?: string;
  ganbaru_asset_path?: string;
}

export interface NotesIconFile {
  url: string;
  expiry_time?: string;
  name?: string;
  content_type?: "image/png" | "image/jpeg" | "image/webp";
  byte_size?: number;
  sha256?: string;
  ganbaru_asset_path?: string;
}

export type NotesIcon =
  | { type: "emoji"; emoji: string }
  | { type: "custom_emoji"; custom_emoji: NotesCustomEmojiIcon }
  | { type: "icon"; icon: { name: string; color?: NotesIconColor } }
  | { type: "external"; external: { url: string } }
  | { type: "file"; file: NotesIconFile };

export type NotesCalloutIcon = NotesIcon | null;

export type NotesPageIcon = NotesIcon;

export interface NotesCalloutBlockPayload extends NotesTextBlockPayload {
  icon: NotesCalloutIcon;
}

export interface NotesCodeBlockPayload {
  rich_text: NotesRichText[];
  caption: NotesRichText[];
  language: string;
}

export interface NotesChildPageBlockPayload {
  title: string;
}
export interface NotesChildDatabaseBlockPayload {
  title: string;
  database_id?: string;
  data_source_id?: string;
  view_id?: string;
}
export type NotesDividerBlockPayload = Record<string, unknown>;
export type NotesBreadcrumbBlockPayload = Record<string, unknown>;
export interface NotesTableOfContentsBlockPayload {
  color?: NotesColor;
}
export type NotesColumnListBlockPayload = Record<string, unknown>;
export interface NotesColumnBlockPayload {
  width_ratio?: number;
}
export interface NotesTableBlockPayload {
  table_width: number;
  has_column_header: boolean;
  has_row_header: boolean;
}
export type NotesTableCell = NotesRichText[];
export interface NotesTableRowBlockPayload {
  cells: NotesTableCell[];
}
export type NotesTabBlockPayload = Record<string, unknown>;
export type NotesFileObject =
  | { type: "external"; external: { url: string } }
  | {
      type: "file";
      file: {
        url: string;
        expiry_time?: string;
        name?: string;
        content_type?: string;
        byte_size?: number;
        sha256?: string;
        ganbaru_asset_path?: string;
      };
    }
  | { type: "file_upload"; file_upload: { id: string } };
export type NotesPageCover = NotesFileObject;
export type NotesMediaBlockPayload = NotesFileObject & {
  caption?: NotesRichText[];
  name?: string;
};
export interface NotesBookmarkBlockPayload {
  caption: NotesRichText[];
  url: string;
}
export interface NotesLinkPreviewBlockPayload {
  url: string;
}
export interface NotesSyncedBlockReference {
  type: "block_id";
  block_id: string;
}
export interface NotesSyncedBlockPayload {
  synced_from: NotesSyncedBlockReference | null;
}
export interface NotesTemplateBlockPayload {
  rich_text: NotesRichText[];
}
export const NOTES_BUTTON_INSERT_POSITIONS = [
  "below_button",
  "above_button",
  "top_of_page",
  "bottom_of_page",
] as const;
export type NotesButtonInsertPosition = (typeof NOTES_BUTTON_INSERT_POSITIONS)[number];
export interface NotesButtonInsertBlocksAction {
  type: "insert_blocks";
  source: "children";
  position: NotesButtonInsertPosition;
}
export type NotesButtonAction = NotesButtonInsertBlocksAction;
export interface NotesButtonBlockPayload {
  rich_text: NotesRichText[];
  icon: NotesIcon | null;
  actions: NotesButtonAction[];
}
export interface NotesEmbedBlockPayload {
  url: string;
}
export interface NotesEquationBlockPayload {
  expression: string;
}
export type NotesUnsupportedBlockPayload = Record<string, unknown> & {
  block_type?: string;
  source_type?: string;
  raw?: Record<string, unknown>;
  warnings?: string[];
};

export interface NotesPage {
  object: "page";
  id: string;
  created_time: string;
  last_edited_time: string;
  parent: NotesParent;
  in_trash: boolean;
  archived?: boolean;
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  properties: Record<string, unknown>;
  url: string | null;
  public_url: string | null;
  source_provider: string | null;
  source_object_id: string | null;
  source_workspace_id: string | null;
  source_last_edited_time: string | null;
}

export interface NotesSidebarPagesRequest {
  expanded_page_ids: string[];
  seed_page_ids: string[];
  selected_page_id: string | null;
}

export interface NotesSidebarPageList {
  pages: NotesPage[];
  page_ids_with_children: string[];
  missing_parent_page_ids: string[];
  trashed_parent_page_ids: string[];
}

export interface NotesPageTemplate {
  object: "page_template";
  id: string;
  name: string;
  source_page_id: string | null;
  properties: Record<string, unknown>;
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  block_count: number;
  created_time: string;
  last_edited_time: string;
}

export interface NotesDataSourceTemplate {
  object: "data_source_template";
  id: string;
  data_source_id: string;
  source_page_id: string | null;
  name: string;
  properties: Record<string, unknown>;
  is_default: boolean;
  block_count: number;
  created_time: string;
  last_edited_time: string;
}

export interface NotesPageHistorySnapshot {
  object: "page_history_snapshot";
  id: string;
  page_id: string;
  title: string;
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  block_count: number;
  reason: string;
  created_by: NotesPartialUser;
  created_time: string;
  page_last_edited_time: string;
}

export interface NotesPageHistorySettings {
  object: "page_history_settings";
  retention_days: number | null;
  updated_at: string;
}

interface NotesBlockBase<Type extends NotesBlockType> {
  object: "block";
  id: string;
  parent: NotesParent;
  created_time: string;
  last_edited_time: string;
  has_children: boolean;
  in_trash: boolean;
  archived?: boolean;
  type: Type;
  source_provider: string | null;
  source_object_id: string | null;
  source_last_edited_time: string | null;
}

export type NotesParagraphBlock = NotesBlockBase<"paragraph"> & {
  paragraph: NotesTextBlockPayload;
};
export type NotesHeading1Block = NotesBlockBase<"heading_1"> & {
  heading_1: NotesTextBlockPayload;
};
export type NotesHeading2Block = NotesBlockBase<"heading_2"> & {
  heading_2: NotesTextBlockPayload;
};
export type NotesHeading3Block = NotesBlockBase<"heading_3"> & {
  heading_3: NotesTextBlockPayload;
};
export type NotesHeading4Block = NotesBlockBase<"heading_4"> & {
  heading_4: NotesTextBlockPayload;
};
export type NotesBulletedListItemBlock = NotesBlockBase<"bulleted_list_item"> & {
  bulleted_list_item: NotesTextBlockPayload;
};
export type NotesNumberedListItemBlock = NotesBlockBase<"numbered_list_item"> & {
  numbered_list_item: NotesTextBlockPayload;
};
export type NotesTodoBlock = NotesBlockBase<"to_do"> & {
  to_do: NotesTodoBlockPayload;
};
export type NotesToggleBlock = NotesBlockBase<"toggle"> & {
  toggle: NotesToggleBlockPayload;
};
export type NotesCalloutBlock = NotesBlockBase<"callout"> & {
  callout: NotesCalloutBlockPayload;
};
export type NotesQuoteBlock = NotesBlockBase<"quote"> & {
  quote: NotesTextBlockPayload;
};
export type NotesChildPageBlock = NotesBlockBase<"child_page"> & {
  child_page: NotesChildPageBlockPayload;
};
export type NotesChildDatabaseBlock = NotesBlockBase<"child_database"> & {
  child_database: NotesChildDatabaseBlockPayload;
};
export type NotesBreadcrumbBlock = NotesBlockBase<"breadcrumb"> & {
  breadcrumb: NotesBreadcrumbBlockPayload;
};
export type NotesTableOfContentsBlock = NotesBlockBase<"table_of_contents"> & {
  table_of_contents: NotesTableOfContentsBlockPayload;
};
export type NotesColumnListBlock = NotesBlockBase<"column_list"> & {
  column_list: NotesColumnListBlockPayload;
};
export type NotesColumnBlock = NotesBlockBase<"column"> & {
  column: NotesColumnBlockPayload;
};
export type NotesTableBlock = NotesBlockBase<"table"> & {
  table: NotesTableBlockPayload;
};
export type NotesTableRowBlock = NotesBlockBase<"table_row"> & {
  table_row: NotesTableRowBlockPayload;
};
export type NotesTabBlock = NotesBlockBase<"tab"> & {
  tab: NotesTabBlockPayload;
};
export type NotesImageBlock = NotesBlockBase<"image"> & {
  image: NotesMediaBlockPayload;
};
export type NotesVideoBlock = NotesBlockBase<"video"> & {
  video: NotesMediaBlockPayload;
};
export type NotesAudioBlock = NotesBlockBase<"audio"> & {
  audio: NotesMediaBlockPayload;
};
export type NotesFileBlock = NotesBlockBase<"file"> & {
  file: NotesMediaBlockPayload;
};
export type NotesPdfBlock = NotesBlockBase<"pdf"> & {
  pdf: NotesMediaBlockPayload;
};
export type NotesBookmarkBlock = NotesBlockBase<"bookmark"> & {
  bookmark: NotesBookmarkBlockPayload;
};
export type NotesLinkPreviewBlock = NotesBlockBase<"link_preview"> & {
  link_preview: NotesLinkPreviewBlockPayload;
};
export type NotesSyncedBlock = NotesBlockBase<"synced_block"> & {
  synced_block: NotesSyncedBlockPayload;
};
export type NotesTemplateBlock = NotesBlockBase<"template"> & {
  template: NotesTemplateBlockPayload;
};
export type NotesButtonBlock = NotesBlockBase<"button"> & {
  button: NotesButtonBlockPayload;
};
export type NotesEmbedBlock = NotesBlockBase<"embed"> & {
  embed: NotesEmbedBlockPayload;
};
export type NotesEquationBlock = NotesBlockBase<"equation"> & {
  equation: NotesEquationBlockPayload;
};
export type NotesDividerBlock = NotesBlockBase<"divider"> & {
  divider: NotesDividerBlockPayload;
};
export type NotesCodeBlock = NotesBlockBase<"code"> & {
  code: NotesCodeBlockPayload;
};
export type NotesUnsupportedBlock = NotesBlockBase<"unsupported"> & {
  unsupported: NotesUnsupportedBlockPayload;
};

export type NotesBlock =
  | NotesParagraphBlock
  | NotesHeading1Block
  | NotesHeading2Block
  | NotesHeading3Block
  | NotesHeading4Block
  | NotesBulletedListItemBlock
  | NotesNumberedListItemBlock
  | NotesTodoBlock
  | NotesToggleBlock
  | NotesCalloutBlock
  | NotesQuoteBlock
  | NotesChildPageBlock
  | NotesChildDatabaseBlock
  | NotesBreadcrumbBlock
  | NotesTableOfContentsBlock
  | NotesColumnListBlock
  | NotesColumnBlock
  | NotesTableBlock
  | NotesTableRowBlock
  | NotesTabBlock
  | NotesImageBlock
  | NotesVideoBlock
  | NotesAudioBlock
  | NotesFileBlock
  | NotesPdfBlock
  | NotesBookmarkBlock
  | NotesLinkPreviewBlock
  | NotesSyncedBlock
  | NotesTemplateBlock
  | NotesButtonBlock
  | NotesEmbedBlock
  | NotesEquationBlock
  | NotesDividerBlock
  | NotesCodeBlock
  | NotesUnsupportedBlock;

export type NotesBlockWrite =
  | { id: string; type: "paragraph"; paragraph: NotesTextBlockPayload }
  | { id: string; type: "heading_1"; heading_1: NotesTextBlockPayload }
  | { id: string; type: "heading_2"; heading_2: NotesTextBlockPayload }
  | { id: string; type: "heading_3"; heading_3: NotesTextBlockPayload }
  | { id: string; type: "heading_4"; heading_4: NotesTextBlockPayload }
  | { id: string; type: "bulleted_list_item"; bulleted_list_item: NotesTextBlockPayload }
  | { id: string; type: "numbered_list_item"; numbered_list_item: NotesTextBlockPayload }
  | { id: string; type: "to_do"; to_do: NotesTodoBlockPayload }
  | { id: string; type: "toggle"; toggle: NotesToggleBlockPayload }
  | { id: string; type: "callout"; callout: NotesCalloutBlockPayload }
  | { id: string; type: "quote"; quote: NotesTextBlockPayload }
  | { id: string; type: "child_page"; child_page: NotesChildPageBlockPayload }
  | { id: string; type: "child_database"; child_database: NotesChildDatabaseBlockPayload }
  | { id: string; type: "breadcrumb"; breadcrumb: NotesBreadcrumbBlockPayload }
  | {
      id: string;
      type: "table_of_contents";
      table_of_contents: NotesTableOfContentsBlockPayload;
    }
  | { id: string; type: "column_list"; column_list: NotesColumnListBlockPayload }
  | { id: string; type: "column"; column: NotesColumnBlockPayload }
  | { id: string; type: "table"; table: NotesTableBlockPayload }
  | { id: string; type: "table_row"; table_row: NotesTableRowBlockPayload }
  | { id: string; type: "tab"; tab: NotesTabBlockPayload }
  | { id: string; type: "image"; image: NotesMediaBlockPayload }
  | { id: string; type: "video"; video: NotesMediaBlockPayload }
  | { id: string; type: "audio"; audio: NotesMediaBlockPayload }
  | { id: string; type: "file"; file: NotesMediaBlockPayload }
  | { id: string; type: "pdf"; pdf: NotesMediaBlockPayload }
  | { id: string; type: "bookmark"; bookmark: NotesBookmarkBlockPayload }
  | { id: string; type: "link_preview"; link_preview: NotesLinkPreviewBlockPayload }
  | { id: string; type: "synced_block"; synced_block: NotesSyncedBlockPayload }
  | { id: string; type: "template"; template: NotesTemplateBlockPayload }
  | { id: string; type: "button"; button: NotesButtonBlockPayload }
  | { id: string; type: "embed"; embed: NotesEmbedBlockPayload }
  | { id: string; type: "equation"; equation: NotesEquationBlockPayload }
  | { id: string; type: "divider"; divider: NotesDividerBlockPayload }
  | { id: string; type: "code"; code: NotesCodeBlockPayload }
  | { id: string; type: "unsupported"; unsupported: NotesUnsupportedBlockPayload };

export type NotesBlockUpdate = NotesBlockWrite extends infer Block
  ? Block extends NotesBlockWrite
    ? Omit<Block, "id">
    : never
  : never;

export interface NotesPageCreate {
  id: string;
  title: string;
  parent: NotesParent;
  first_block_id: string;
  after_block_id?: string | null;
}

export interface NotesChildPageFromBlockCreate {
  first_block_id: string;
  title?: string | null;
}

export interface NotesDuplicatePageRequest {
  title?: string | null;
}

export interface NotesMovePageRequest {
  parent: NotesParent;
}

export interface NotesPageTemplateCreateFromPageRequest {
  id: string;
  source_page_id: string;
  name: string;
}

export interface NotesPageTemplateApplyRequest {
  parent: NotesParent;
  title?: string | null;
}

export interface NotesPageTemplateUpdateRequest {
  name?: string;
  source_page_id?: string;
}

export interface NotesPageTemplateDuplicateRequest {
  id: string;
  name: string;
}

export interface NotesPageHistorySettingsUpdate {
  retention_days: number | null;
}

export interface NotesLocalUserUpdate {
  display_name: string;
}

export const NOTES_MENTION_NOTIFICATION_KINDS = [
  "reminder",
  "user_mention",
  "task_mention",
] as const;

export type NotesMentionNotificationKind =
  (typeof NOTES_MENTION_NOTIFICATION_KINDS)[number];

export const NOTES_MENTION_NOTIFICATION_TARGET_TYPES = [
  "date",
  "user",
  "project_task",
] as const;

export type NotesMentionNotificationTargetType =
  (typeof NOTES_MENTION_NOTIFICATION_TARGET_TYPES)[number];

export const NOTES_MENTION_NOTIFICATION_STATUSES = [
  "pending",
  "delivered",
  "dismissed",
] as const;

export type NotesMentionNotificationStatus =
  (typeof NOTES_MENTION_NOTIFICATION_STATUSES)[number];

export type NotesMentionNotificationSourceType = "block" | "comment";

export interface NotesMentionNotification {
  object: "mention_notification";
  id: string;
  source_type: NotesMentionNotificationSourceType;
  source_id: string;
  page_id: string;
  page_title: string;
  block_id: string | null;
  comment_id: string | null;
  kind: NotesMentionNotificationKind;
  target_type: NotesMentionNotificationTargetType;
  target_id: string | null;
  trigger_at: string | null;
  plain_text: string;
  source_plain_text: string;
  status: NotesMentionNotificationStatus;
  delivered_at: string | null;
  created_time: string;
  last_edited_time: string;
}

export interface NotesMentionNotificationDeliveryUpdate {
  ids: string[];
}

export interface NotesPageHistoryCopyBlocksRequest {
  after_block_id?: string | null;
}

export interface NotesPageUpdate {
  title?: string;
  parent?: NotesParent;
  properties?: Record<string, unknown>;
  icon?: NotesPageIcon | null;
  cover?: NotesPageCover | null;
}

export interface NotesDatabaseDataSourceSummary {
  id: string;
  name: string;
}

export interface NotesDatabase {
  object: "database";
  id: string;
  parent: NotesParent;
  title: string;
  title_rich_text: NotesRichText[];
  description: NotesRichText[];
  icon: NotesPageIcon | null;
  cover: NotesPageCover | null;
  in_trash: boolean;
  is_inline: boolean;
  data_sources: NotesDatabaseDataSourceSummary[];
  url: string | null;
  public_url: string | null;
  source_provider: string | null;
  source_object_id: string | null;
  source_workspace_id: string | null;
  source_last_edited_time: string | null;
  created_time: string;
  last_edited_time: string;
}

export interface NotesDataSourceParent {
  type: "database_id";
  database_id: string;
}

export interface NotesDataSource {
  object: "data_source";
  id: string;
  parent: NotesDataSourceParent;
  database_parent: NotesParent;
  title: string;
  title_rich_text: NotesRichText[];
  description: NotesRichText[];
  icon: NotesPageIcon | null;
  properties: Record<string, unknown>;
  in_trash: boolean;
  source_provider: string | null;
  source_object_id: string | null;
  source_workspace_id: string | null;
  source_last_edited_time: string | null;
  created_time: string;
  last_edited_time: string;
}

export const NOTES_DATA_SOURCE_PROPERTY_TYPES = [
  "title",
  "rich_text",
  "number",
  "select",
  "multi_select",
  "status",
  "date",
  "checkbox",
  "url",
  "email",
  "phone_number",
  "files",
  "people",
  "created_time",
  "created_by",
  "last_edited_time",
  "last_edited_by",
  "unique_id",
  "place",
  "relation",
  "rollup",
  "formula",
  "button",
] as const;

export type NotesDataSourcePropertyType = (typeof NOTES_DATA_SOURCE_PROPERTY_TYPES)[number];

export const NOTES_DATA_SOURCE_ROLLUP_FUNCTIONS = [
  "average",
  "checked",
  "count",
  "count_values",
  "date_range",
  "earliest_date",
  "empty",
  "latest_date",
  "max",
  "median",
  "min",
  "not_empty",
  "percent_checked",
  "percent_empty",
  "percent_not_empty",
  "percent_unchecked",
  "range",
  "show_original",
  "show_unique",
  "sum",
  "unchecked",
  "unique",
] as const;

export type NotesDataSourceRollupFunction = (typeof NOTES_DATA_SOURCE_ROLLUP_FUNCTIONS)[number];

export const NOTES_DATA_SOURCE_SELECT_COLORS = [
  "default",
  "gray",
  "brown",
  "orange",
  "yellow",
  "green",
  "blue",
  "purple",
  "pink",
  "red",
] as const;

export type NotesDataSourceSelectColor = (typeof NOTES_DATA_SOURCE_SELECT_COLORS)[number];

export const NOTES_DATA_SOURCE_NUMBER_FORMATS = [
  "number",
  "number_with_commas",
  "percent",
  "dollar",
  "euro",
  "pound",
  "yen",
  "yuan",
  "won",
  "ruble",
  "rupee",
  "franc",
  "real",
  "lira",
  "krona",
  "ringgit",
] as const;

export type NotesDataSourceNumberFormat = (typeof NOTES_DATA_SOURCE_NUMBER_FORMATS)[number];

export const NOTES_DATA_SOURCE_STATUS_GROUPS = [
  "To-do",
  "In progress",
  "Complete",
] as const;

export type NotesDataSourceStatusGroup = (typeof NOTES_DATA_SOURCE_STATUS_GROUPS)[number];

export interface NotesDataSourceSchema {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
}

export interface NotesDatabaseViewScope {
  databaseId?: string | null;
  viewId?: string | null;
}

export interface NotesDataSourceSchemaUpdate {
  properties: Record<string, unknown>;
  property_order: string[];
  hidden_property_ids: string[];
}

export interface NotesDataSourceRowPageCreateRequest {
  id: string;
  title: string;
  first_block_id: string;
  properties?: Record<string, unknown> | null;
}

export interface NotesDataSourceTemplateCreateFromRowRequest {
  id: string;
  source_page_id: string;
  name: string;
  is_default?: boolean | null;
}

export interface NotesDataSourceTemplateApplyRequest {
  title?: string | null;
}

export interface NotesDataSourceTemplateUpdateRequest {
  name?: string;
  source_page_id?: string;
  is_default?: boolean;
}

export interface NotesDataSourceTemplateDuplicateRequest {
  id: string;
  name: string;
  is_default?: boolean | null;
}

export interface NotesDataSourceButtonClickRequest {
  property_id: string;
  confirmed?: boolean | null;
}

export type NotesDatabaseTableRowOpenMode = "full_page" | "side_panel";

export type NotesDatabaseTableFilterCondition =
  | "contains"
  | "equals"
  | "is_empty"
  | "is_not_empty"
  | "checked"
  | "unchecked";

export type NotesDatabaseTableSortDirection = "ascending" | "descending";

export interface NotesDatabaseTableFilter {
  property_id: string;
  condition: NotesDatabaseTableFilterCondition;
  value?: string | number | boolean | null;
}

export interface NotesDatabaseTableSort {
  property_id: string;
  direction: NotesDatabaseTableSortDirection;
}

export interface NotesDatabaseTableConfiguration {
  property_order: string[];
  hidden_property_ids: string[];
  column_widths: Record<string, number>;
  row_open_mode: NotesDatabaseTableRowOpenMode;
}

export interface NotesDataSourceTableViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseTableConfiguration;
}

export interface NotesDataSourceRowPropertyUpdate {
  property_id: string;
  value: unknown;
}

export interface NotesDataSourceTableView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
}

export type NotesDatabaseBoardRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseBoardConfiguration {
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseBoardRowOpenMode;
}

export interface NotesDataSourceBoardViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseBoardConfiguration;
}

export interface NotesDataSourceBoardRowMove {
  page_id: string;
  group_id: string;
}

export interface NotesDataSourceBoardGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceBoardView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  groups: NotesDataSourceBoardGroup[];
}

export type NotesDatabaseGalleryRowOpenMode = "full_page" | "side_panel";
export type NotesDatabaseGalleryCoverSource = "page_cover" | "files_property" | "none";
export type NotesDatabaseGalleryCardSize = "small" | "medium" | "large";

export interface NotesDatabaseGalleryConfiguration {
  cover_source: NotesDatabaseGalleryCoverSource;
  cover_property_id: string | null;
  visible_property_ids: string[];
  card_size: NotesDatabaseGalleryCardSize;
  fit_image: boolean;
  row_open_mode: NotesDatabaseGalleryRowOpenMode;
}

export interface NotesDataSourceGalleryViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseGalleryConfiguration;
}

export interface NotesDataSourceGalleryView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
}

export type NotesDatabaseListRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseListConfiguration {
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseListRowOpenMode;
}

export interface NotesDataSourceListViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseListConfiguration;
}

export interface NotesDataSourceListGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceListView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
}

export type NotesDatabaseCalendarRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseCalendarConfiguration {
  date_property_id: string | null;
  range_start: string;
  range_end: string;
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseCalendarRowOpenMode;
}

export interface NotesDataSourceCalendarViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseCalendarConfiguration;
}

export interface NotesDataSourceCalendarDay {
  date: string;
  in_month: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceCalendarView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
}

export type NotesDatabaseTimelineRowOpenMode = "full_page" | "side_panel";

export interface NotesDatabaseTimelineConfiguration {
  date_property_id: string | null;
  group_property_id: string | null;
  group_order: string[];
  hidden_group_ids: string[];
  range_start: string;
  range_end: string;
  visible_property_ids: string[];
  row_open_mode: NotesDatabaseTimelineRowOpenMode;
}

export interface NotesDataSourceTimelineViewUpdate {
  filter: NotesDatabaseTableFilter[];
  sorts: NotesDatabaseTableSort[];
  configuration: NotesDatabaseTimelineConfiguration;
}

export interface NotesDataSourceTimelineGroup {
  id: string;
  name: string;
  color: string;
  hidden: boolean;
  rows: NotesPage[];
}

export interface NotesDataSourceTimelineView {
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  rows: NotesPage[];
}

export const NOTES_DATABASE_VIEW_TYPES = [
  "table",
  "board",
  "list",
  "calendar",
  "timeline",
  "gallery",
  "form",
  "chart",
  "map",
  "dashboard",
] as const;

export type NotesDatabaseViewType = (typeof NOTES_DATABASE_VIEW_TYPES)[number];

export interface NotesDatabaseViewParent {
  type: "database_id";
  database_id: string;
}

export interface NotesDatabaseView {
  object: "view";
  id: string;
  parent: NotesDatabaseViewParent;
  data_source_id: string;
  name: string;
  type: NotesDatabaseViewType;
  filter: Record<string, unknown> | null;
  sorts: Record<string, unknown>[];
  configuration: Record<string, unknown> | null;
  url: string | null;
  source_provider: string | null;
  source_object_id: string | null;
  source_workspace_id: string | null;
  source_last_edited_time: string | null;
  created_time: string;
  last_edited_time: string;
}

export interface NotesDatabaseCreateRequest {
  id: string;
  data_source_id: string;
  view_id: string;
  title: string;
  parent?: NotesParent | null;
  after_block_id?: string | null;
  replace_block_id?: string | null;
  icon?: NotesPageIcon | null;
  cover?: NotesPageCover | null;
}

export interface NotesLinkedDatabaseCreateRequest {
  id: string;
  view_id: string;
  source_block_id: string;
  title?: string | null;
}

export interface NotesCreatedDatabase {
  database: NotesDatabase;
  data_source: NotesDataSource;
  view: NotesDatabaseView;
  block: NotesChildDatabaseBlock;
}

export interface NotesAppendBlockChildrenRequest {
  parent: NotesParent;
  after: string | null;
  children: NotesBlockWrite[];
}

export interface NotesMoveBlockRequest {
  parent: NotesParent;
  after: string | null;
  before?: string | null;
}

export interface NotesMoveBlocksRequest {
  block_ids: string[];
  parent: NotesParent;
  after: string | null;
  before?: string | null;
}

export interface NotesDuplicateBlockIdPair {
  source_id: string;
  duplicate_id: string;
}

export interface NotesDuplicateBlockRequest {
  duplicated_block_ids: NotesDuplicateBlockIdPair[];
}

export interface NotesDuplicateBlocksRequest {
  block_ids: string[];
  duplicated_block_ids: NotesDuplicateBlockIdPair[];
  parent: NotesParent;
  after: string | null;
  before?: string | null;
  include_trashed_sources?: boolean;
}

export interface NotesTrashBlocksRequest {
  block_ids: string[];
  in_trash?: boolean;
}

export interface NotesPaginatedBlockList {
  object: "list";
  type: "block";
  block: Record<string, unknown>;
  results: NotesBlock[];
  next_cursor: string | null;
  has_more: boolean;
}

export interface NotesLoadedPage {
  page: NotesPage;
  blocks: NotesPaginatedBlockList;
}

export interface NotesMarkdownImportRequest {
  parent: NotesParent;
  markdown: string;
  title?: string | null;
  source_name?: string | null;
  after_block_id?: string | null;
}

export type NotesMarkdownImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesMarkdownImportDiagnostic {
  code: string;
  severity: NotesMarkdownImportDiagnosticSeverity;
  line: number | null;
  message: string;
}

export interface NotesMarkdownImportResult {
  page: NotesLoadedPage;
  diagnostics: NotesMarkdownImportDiagnostic[];
  imported_block_count: number;
}

export interface NotesHtmlImportRequest {
  parent: NotesParent;
  html: string;
  title?: string | null;
  source_name?: string | null;
  after_block_id?: string | null;
  keep_external_file_references?: boolean | null;
}

export type NotesHtmlImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesHtmlImportDiagnostic {
  code: string;
  severity: NotesHtmlImportDiagnosticSeverity;
  line: number | null;
  message: string;
}

export interface NotesHtmlImportResult {
  page: NotesLoadedPage;
  diagnostics: NotesHtmlImportDiagnostic[];
  imported_block_count: number;
}

export interface NotesNotionApiImportRequest {
  parent: NotesParent;
  integration_token: string;
  source_workspace_id?: string | null;
  page_ids?: string[];
  data_source_ids?: string[];
  include_comments?: boolean | null;
  include_users?: boolean | null;
  keep_external_file_references?: boolean | null;
  page_size?: number | null;
}

export type NotesNotionApiImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesNotionApiImportDiagnostic {
  code: string;
  severity: NotesNotionApiImportDiagnosticSeverity;
  source_object_id: string | null;
  message: string;
}

export interface NotesNotionApiImportedObject {
  object_type: string;
  source_object_id: string;
  local_id: string;
  title: string;
}

export interface NotesNotionApiImportedUser {
  source_user_id: string;
  name: string;
  user_type: string;
}

export interface NotesNotionApiImportResult {
  object: "notes_notion_api_import";
  imported_pages: NotesLoadedPage[];
  imported_data_sources: NotesNotionApiImportedObject[];
  imported_users: NotesNotionApiImportedUser[];
  diagnostics: NotesNotionApiImportDiagnostic[];
  request_count: number;
  retry_count: number;
  rate_limit_count: number;
  imported_page_count: number;
  imported_block_count: number;
  imported_data_source_count: number;
  imported_comment_count: number;
  imported_user_count: number;
  imported_file_count: number;
  unsupported_block_count: number;
}

export interface NotesNotionExportImportRequest {
  parent: NotesParent;
  export_root_path: string;
  source_workspace_id?: string | null;
  keep_external_file_references?: boolean | null;
  copy_local_file_references?: boolean | null;
  import_markdown?: boolean | null;
  import_html?: boolean | null;
  import_csv?: boolean | null;
}

export type NotesNotionExportImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesNotionExportImportDiagnostic {
  code: string;
  severity: NotesNotionExportImportDiagnosticSeverity;
  source_path: string | null;
  message: string;
}

export interface NotesNotionExportImportResult {
  object: "notes_notion_export_import";
  imported_pages: NotesLoadedPage[];
  imported_data_sources: NotesNotionApiImportedObject[];
  diagnostics: NotesNotionExportImportDiagnostic[];
  imported_page_count: number;
  imported_block_count: number;
  imported_data_source_count: number;
  imported_file_count: number;
  skipped_file_count: number;
  unsupported_block_count: number;
}

export interface NotesMarkdownExportRequest {
  page_id: string;
  include_page_title?: boolean | null;
  include_comments?: boolean | null;
  include_resolved_comments?: boolean | null;
}

export type NotesMarkdownExportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesMarkdownExportDiagnostic {
  code: string;
  severity: NotesMarkdownExportDiagnosticSeverity;
  block_id: string | null;
  comment_id: string | null;
  message: string;
}

export interface NotesMarkdownExportResult {
  object: "notes_markdown_export";
  page_id: string;
  markdown: string;
  diagnostics: NotesMarkdownExportDiagnostic[];
  exported_block_count: number;
  exported_comment_count: number;
}

export interface NotesHtmlExportRequest {
  page_id: string;
  include_page_tree?: boolean | null;
  include_comments?: boolean | null;
  include_resolved_comments?: boolean | null;
  include_assets?: boolean | null;
  include_database_views?: boolean | null;
}

export type NotesHtmlExportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesHtmlExportDiagnostic {
  code: string;
  severity: NotesHtmlExportDiagnosticSeverity;
  page_id: string | null;
  block_id: string | null;
  asset_id: string | null;
  comment_id: string | null;
  message: string;
}

export interface NotesHtmlExportFile {
  path: string;
  content_type: string;
  contents: string;
  byte_size: number;
}

export interface NotesHtmlExportAsset {
  id: string;
  archive_path: string;
  source_path: string;
  content_type: string;
  byte_size: number;
  sha256: string;
  storage_state: string;
  exported: boolean;
}

export interface NotesHtmlExportResult {
  object: "notes_html_archive_export";
  root_page_id: string;
  files: NotesHtmlExportFile[];
  assets: NotesHtmlExportAsset[];
  diagnostics: NotesHtmlExportDiagnostic[];
  manifest_json: string;
  exported_page_count: number;
  exported_block_count: number;
  exported_asset_count: number;
  exported_comment_count: number;
  exported_database_view_count: number;
}

export interface NotesHtmlArchiveSaveResult {
  object: "notes_html_archive_save";
  saved: boolean;
  export: NotesHtmlExportResult | null;
}

export interface NotesDataSourceCsvImportRequest {
  csv: string;
  has_header?: boolean | null;
  dry_run?: boolean | null;
}

export interface NotesDataSourceCsvImportColumn {
  source_index: number;
  source_name: string;
  property_id: string | null;
  property_name: string | null;
  property_type: NotesDataSourcePropertyType | string | null;
  mapped: boolean;
  read_only: boolean;
  warning: string | null;
}

export type NotesDataSourceCsvImportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesDataSourceCsvImportDiagnostic {
  code: string;
  severity: NotesDataSourceCsvImportDiagnosticSeverity;
  row_number: number | null;
  column_index: number | null;
  column_name: string | null;
  property_id: string | null;
  message: string;
}

export interface NotesDataSourceCsvImportRow {
  row_number: number;
  title: string;
  valid: boolean;
  mapped_cell_count: number;
  error_count: number;
}

export interface NotesDataSourceCsvImportResult {
  object: "notes_data_source_csv_import";
  data_source_id: string;
  dry_run: boolean;
  total_row_count: number;
  valid_row_count: number;
  skipped_row_count: number;
  imported_row_count: number;
  imported_page_ids: string[];
  columns: NotesDataSourceCsvImportColumn[];
  rows: NotesDataSourceCsvImportRow[];
  diagnostics: NotesDataSourceCsvImportDiagnostic[];
}

export type NotesDataSourceCsvExportScope = "view" | "all";

export interface NotesDataSourceCsvExportRequest {
  database_id?: string | null;
  view_id?: string | null;
  scope?: NotesDataSourceCsvExportScope | null;
}

export type NotesDataSourceCsvExportDiagnosticSeverity = "info" | "warning" | "error";

export interface NotesDataSourceCsvExportDiagnostic {
  code: string;
  severity: NotesDataSourceCsvExportDiagnosticSeverity;
  property_id: string | null;
  property_name: string | null;
  message: string;
}

export interface NotesDataSourceCsvExportResult {
  object: "notes_data_source_csv_export";
  data_source_id: string;
  database_id: string;
  view_id: string;
  scope: NotesDataSourceCsvExportScope;
  file_name: string;
  csv: string;
  exported_row_count: number;
  exported_property_count: number;
  diagnostics: NotesDataSourceCsvExportDiagnostic[];
}

export interface NotesDataSourceCsvExportSaveResult {
  saved: boolean;
  export: NotesDataSourceCsvExportResult | null;
}

export type NotesBacklinkReferenceType =
  | "child_page"
  | "page_mention"
  | "link"
  | "database_relation"
  | "comment_mention"
  | "comment_link";

export interface NotesBacklink {
  object: "backlink";
  id: string;
  source_page: NotesPage;
  source_block_id: string;
  source_block_type: NotesBlockType | "database_relation" | "comment";
  reference_type: NotesBacklinkReferenceType;
  snippet: string;
  created_time: string;
  last_edited_time: string;
}

export interface NotesPageAlias {
  object: "page_alias";
  id: string;
  page_id: string;
  alias: string;
  normalized_alias: string;
  created_time: string;
  last_edited_time: string;
}

export interface NotesPageAliasCreate {
  id: string;
  alias: string;
}

export type NotesUnresolvedLinkSourceType = "block" | "comment";

export interface NotesUnresolvedLink {
  object: "unresolved_link";
  id: string;
  source_type: NotesUnresolvedLinkSourceType;
  source_page_id: string;
  source_block_id: string | null;
  source_comment_id: string | null;
  raw_url: string;
  raw_target: string;
  normalized_target: string;
  link_text: string;
  snippet: string;
  created_time: string;
  last_edited_time: string;
}

export interface NotesUnresolvedLinkResolve {
  target_page_id: string;
}

export type NotesSearchResultType = "page" | "block" | "comment";

export interface NotesSearchResult {
  object: "search_result";
  id: string;
  type: NotesSearchResultType;
  page: NotesPage;
  block_id: string | null;
  block_type: NotesBlockType | null;
  comment_id: string | null;
  discussion_id: string | null;
  comment_status: NotesCommentThreadStatus | null;
  comment_author: NotesCommentDisplayName | null;
  comment_anchor: NotesCommentAnchor | null;
  snippet: string;
  last_edited_time: string;
}

export type NotesCommentParent =
  | { type: "page_id"; page_id: string }
  | { type: "block_id"; block_id: string };

export type NotesCommentThreadStatus = "open" | "resolved";

export interface NotesPartialUser {
  object: "user";
  id: string;
}

export interface NotesLocalUser {
  object: "user";
  id: string;
  display_name: string;
  created_time: string;
  last_edited_time: string;
}

export type NotesCommentDisplayName =
  | { type: "user"; resolved_name: string }
  | { type: "integration"; resolved_name: string }
  | { type: "custom"; resolved_name: string };

export interface NotesComment {
  object: "comment";
  id: string;
  parent: NotesCommentParent;
  discussion_id: string;
  created_time: string;
  last_edited_time: string;
  created_by: NotesPartialUser;
  rich_text: NotesRichText[];
  attachments: Record<string, unknown>[];
  display_name: NotesCommentDisplayName;
  deleted_at: string | null;
}

export interface NotesCommentAnchor {
  object: "comment_anchor";
  type: "text_range";
  block_id: string;
  start: number;
  end: number;
  text: string;
  prefix: string;
  suffix: string;
  created_time: string;
  last_edited_time: string;
}

export interface NotesCommentThread {
  object: "comment_thread";
  id: string;
  parent: NotesCommentParent;
  page_id: string;
  block_id: string | null;
  status: NotesCommentThreadStatus;
  resolved_at: string | null;
  resolved_by: NotesPartialUser | null;
  anchor: NotesCommentAnchor | null;
  created_time: string;
  last_edited_time: string;
  unread: boolean;
  comments: NotesComment[];
}

export interface NotesCommentAnchorCreate {
  start: number;
  end: number;
  text: string;
  prefix: string;
  suffix: string;
}

export interface NotesCommentCreate {
  id: string;
  parent?: NotesCommentParent;
  discussion_id?: string;
  anchor?: NotesCommentAnchorCreate;
  rich_text: NotesRichText[];
  attachments?: Record<string, unknown>[];
}

export interface NotesCommentUpdate {
  rich_text: NotesRichText[];
  attachments?: Record<string, unknown>[];
}

export interface NotesCommentThreadReadUpdate {
  page_id: string;
  discussion_ids: string[];
  include_resolved?: boolean;
}

export type NotesSuggestionStatus = "open" | "accepted" | "rejected";

export interface NotesSuggestion {
  object: "suggestion";
  id: string;
  page_id: string;
  block_id: string;
  created_by: NotesPartialUser;
  display_name: NotesCommentDisplayName;
  status: NotesSuggestionStatus;
  range_start: number;
  range_end: number;
  original_text: string;
  proposed_text: string;
  prefix: string;
  suffix: string;
  accepted_at: string | null;
  accepted_by: NotesPartialUser | null;
  rejected_at: string | null;
  rejected_by: NotesPartialUser | null;
  created_time: string;
  last_edited_time: string;
}

export interface NotesSuggestionCreate {
  id: string;
  block_id: string;
  range_start: number;
  range_end: number;
  original_text: string;
  proposed_text: string;
  prefix: string;
  suffix: string;
}

export interface NotesBlockTreeItem {
  block: NotesBlock;
  depth: number;
  parentId: string;
  previousSiblingId: string | null;
  previousVisibleId: string | null;
}

export interface NotesColumnBlockItems {
  column: NotesColumnBlock;
  items: NotesBlockTreeItem[];
}

export interface NotesTabBlockItems {
  label: NotesParagraphBlock;
  items: NotesBlockTreeItem[];
}

export type NotesPageBreadcrumbStatus = "workspace" | "active" | "archived" | "trashed" | "missing";

export interface NotesPageBreadcrumbItem {
  id: string | null;
  title: string;
  current: boolean;
  status: NotesPageBreadcrumbStatus;
}

export interface NotesTableOfContentsItem {
  blockId: string;
  title: string;
  level: 1 | 2 | 3 | 4;
}
