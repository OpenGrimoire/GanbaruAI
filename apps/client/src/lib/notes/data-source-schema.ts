import {
  NOTES_DATA_SOURCE_NUMBER_FORMATS,
  NOTES_DATA_SOURCE_PROPERTY_TYPES,
  NOTES_DATA_SOURCE_SELECT_COLORS,
  NOTES_DATA_SOURCE_STATUS_GROUPS,
  type NotesDataSource,
  type NotesDataSourceNumberFormat,
  type NotesDataSourcePropertyType,
  type NotesDataSourceSchemaUpdate,
  type NotesDataSourceSelectColor,
  type NotesDataSourceStatusGroup,
  type NotesDatabaseView,
} from "./types";

type UnknownRecord = Record<string, unknown>;

export interface NotesDataSourceSchemaOptionDraft {
  id: string;
  name: string;
  color: NotesDataSourceSelectColor;
  group: NotesDataSourceStatusGroup;
}

export interface NotesDataSourceSchemaPropertyDraft {
  id: string;
  name: string;
  description: string;
  type: NotesDataSourcePropertyType;
  hidden: boolean;
  numberFormat: NotesDataSourceNumberFormat;
  uniquePrefix: string;
  options: NotesDataSourceSchemaOptionDraft[];
}

const DEFAULT_OPTION_COLORS: NotesDataSourceSelectColor[] = [
  "default",
  "blue",
  "green",
  "yellow",
  "orange",
  "purple",
  "pink",
  "red",
  "gray",
  "brown",
];

const EMPTY_OBJECT_TYPES = new Set<NotesDataSourcePropertyType>([
  "title",
  "rich_text",
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
  "place",
]);

function isRecord(value: unknown): value is UnknownRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readString(value: unknown, fallback: string): string {
  return typeof value === "string" ? value : fallback;
}

function isPropertyType(value: string): value is NotesDataSourcePropertyType {
  return NOTES_DATA_SOURCE_PROPERTY_TYPES.includes(value as NotesDataSourcePropertyType);
}

function isSelectColor(value: string): value is NotesDataSourceSelectColor {
  return NOTES_DATA_SOURCE_SELECT_COLORS.includes(value as NotesDataSourceSelectColor);
}

function isNumberFormat(value: string): value is NotesDataSourceNumberFormat {
  return NOTES_DATA_SOURCE_NUMBER_FORMATS.includes(value as NotesDataSourceNumberFormat);
}

function isStatusGroup(value: string): value is NotesDataSourceStatusGroup {
  return NOTES_DATA_SOURCE_STATUS_GROUPS.includes(value as NotesDataSourceStatusGroup);
}

function readStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string");
}

function tableConfiguration(view: NotesDatabaseView): UnknownRecord {
  if (!isRecord(view.configuration)) return {};
  const table = view.configuration.table;
  return isRecord(table) ? table : {};
}

function viewPropertyOrder(view: NotesDatabaseView): string[] {
  return readStringArray(tableConfiguration(view).property_order);
}

function viewHiddenPropertyIds(view: NotesDatabaseView): Set<string> {
  return new Set(readStringArray(tableConfiguration(view).hidden_property_ids));
}

function optionDrafts(value: unknown): NotesDataSourceSchemaOptionDraft[] {
  const options = isRecord(value) && Array.isArray(value.options) ? value.options : [];
  return options.filter(isRecord).map((option, index) => {
    const color = readString(option.color, DEFAULT_OPTION_COLORS[index % DEFAULT_OPTION_COLORS.length]);
    const group = readString(option.group, "To-do");
    return {
      id: readString(option.id, crypto.randomUUID()),
      name: readString(option.name, `Option ${index + 1}`),
      color: isSelectColor(color) ? color : "default",
      group: isStatusGroup(group) ? group : "To-do",
    };
  });
}

function statusOptionDrafts(value: unknown): NotesDataSourceSchemaOptionDraft[] {
  if (!isRecord(value)) return defaultStatusOptions();
  const optionGroups = new Map<string, NotesDataSourceStatusGroup>();
  if (Array.isArray(value.groups)) {
    for (const group of value.groups.filter(isRecord)) {
      const groupName = readString(group.name, "To-do");
      const statusGroup = isStatusGroup(groupName) ? groupName : "To-do";
      for (const optionId of readStringArray(group.option_ids)) {
        optionGroups.set(optionId, statusGroup);
      }
    }
  }
  const options = optionDrafts(value).map((option) => ({
    ...option,
    group: optionGroups.get(option.id) ?? option.group,
  }));
  return options.length > 0 ? options : defaultStatusOptions();
}

function defaultStatusOptions(): NotesDataSourceSchemaOptionDraft[] {
  return [
    {
      id: crypto.randomUUID(),
      name: "Not started",
      color: "default",
      group: "To-do",
    },
    {
      id: crypto.randomUUID(),
      name: "In progress",
      color: "blue",
      group: "In progress",
    },
    {
      id: crypto.randomUUID(),
      name: "Done",
      color: "green",
      group: "Complete",
    },
  ];
}

function propertyDraftFromRecord(
  key: string,
  value: unknown,
  hiddenIds: Set<string>,
): NotesDataSourceSchemaPropertyDraft | null {
  if (!isRecord(value)) return null;
  const typeValue = readString(value.type, "rich_text");
  const type = isPropertyType(typeValue) ? typeValue : "rich_text";
  const id = type === "title" ? "title" : readString(value.id, crypto.randomUUID());
  const config = value[type];
  const numberConfig = isRecord(config) ? config : {};
  const numberFormat = readString(numberConfig.format, "number");
  const uniquePrefix = readString(isRecord(config) ? config.prefix : "", "");
  const options = type === "status"
    ? statusOptionDrafts(config)
    : type === "select" || type === "multi_select"
      ? optionDrafts(config)
      : [];
  return {
    id,
    name: readString(value.name, key),
    description: readString(value.description, ""),
    type,
    hidden: type === "title" ? false : hiddenIds.has(id),
    numberFormat: isNumberFormat(numberFormat) ? numberFormat : "number",
    uniquePrefix,
    options,
  };
}

export function notesDataSourceSchemaDraftFromDto(
  dataSource: NotesDataSource,
  view: NotesDatabaseView,
): NotesDataSourceSchemaPropertyDraft[] {
  const hiddenIds = viewHiddenPropertyIds(view);
  const byId = new Map<string, NotesDataSourceSchemaPropertyDraft>();
  for (const [key, value] of Object.entries(dataSource.properties)) {
    const property = propertyDraftFromRecord(key, value, hiddenIds);
    if (property) byId.set(property.id, property);
  }
  const order = viewPropertyOrder(view);
  const sorted: NotesDataSourceSchemaPropertyDraft[] = [];
  for (const propertyId of ["title", ...order.filter((id) => id !== "title")]) {
    const property = byId.get(propertyId);
    if (!property) continue;
    sorted.push(property);
    byId.delete(propertyId);
  }
  sorted.push(...Array.from(byId.values()).sort((left, right) => left.name.localeCompare(right.name)));
  return sorted;
}

export function defaultNotesDataSourcePropertyName(
  type: NotesDataSourcePropertyType,
  properties: readonly NotesDataSourceSchemaPropertyDraft[],
): string {
  const baseName = defaultNameForType(type);
  const existing = new Set(properties.map((property) => property.name.trim().toLocaleLowerCase()));
  if (!existing.has(baseName.toLocaleLowerCase())) return baseName;
  for (let index = 2; index < 1000; index += 1) {
    const candidate = `${baseName} ${index}`;
    if (!existing.has(candidate.toLocaleLowerCase())) return candidate;
  }
  return `${baseName} ${crypto.randomUUID().slice(0, 8)}`;
}

function defaultNameForType(type: NotesDataSourcePropertyType): string {
  switch (type) {
    case "title":
      return "Name";
    case "rich_text":
      return "Text";
    case "number":
      return "Number";
    case "select":
      return "Select";
    case "multi_select":
      return "Multi-select";
    case "status":
      return "Status";
    case "date":
      return "Date";
    case "checkbox":
      return "Checkbox";
    case "url":
      return "URL";
    case "email":
      return "Email";
    case "phone_number":
      return "Phone";
    case "files":
      return "Files";
    case "people":
      return "People";
    case "created_time":
      return "Created time";
    case "created_by":
      return "Created by";
    case "last_edited_time":
      return "Last edited time";
    case "last_edited_by":
      return "Last edited by";
    case "unique_id":
      return "Unique ID";
    case "place":
      return "Place";
  }
}

export function createNotesDataSourcePropertyDraft(
  type: NotesDataSourcePropertyType,
  name: string,
  id: string = crypto.randomUUID(),
): NotesDataSourceSchemaPropertyDraft {
  const propertyId = type === "title" ? "title" : id;
  return {
    id: propertyId,
    name,
    description: "",
    type,
    hidden: false,
    numberFormat: "number",
    uniquePrefix: "",
    options: type === "status" ? defaultStatusOptions() : [],
  };
}

function emptyPropertyConfig(type: NotesDataSourcePropertyType): UnknownRecord {
  if (!EMPTY_OBJECT_TYPES.has(type)) return {};
  return {};
}

function propertyConfig(property: NotesDataSourceSchemaPropertyDraft): UnknownRecord {
  switch (property.type) {
    case "number":
      return { format: property.numberFormat };
    case "select":
    case "multi_select":
      return {
        options: property.options.map(({ id, name, color }) => ({ id, name, color })),
      };
    case "status":
      return {
        options: property.options.map(({ id, name, color }) => ({ id, name, color })),
        groups: NOTES_DATA_SOURCE_STATUS_GROUPS.map((group) => ({
          id: group,
          name: group,
          color: statusGroupColor(group),
          option_ids: property.options
            .filter((option) => option.group === group)
            .map((option) => option.id),
        })),
      };
    case "unique_id":
      return { prefix: property.uniquePrefix.trim() || null };
    default:
      return emptyPropertyConfig(property.type);
  }
}

function statusGroupColor(group: NotesDataSourceStatusGroup): NotesDataSourceSelectColor {
  switch (group) {
    case "To-do":
      return "gray";
    case "In progress":
      return "blue";
    case "Complete":
      return "green";
  }
}

export function notesDataSourceSchemaUpdateFromDraft(
  properties: readonly NotesDataSourceSchemaPropertyDraft[],
): NotesDataSourceSchemaUpdate {
  const record: Record<string, unknown> = {};
  const names = new Set<string>();
  for (const property of properties) {
    const name = property.name.trim();
    if (!name) throw new Error("property name is required");
    const nameKey = name.toLocaleLowerCase();
    if (names.has(nameKey)) throw new Error("property names must be unique");
    names.add(nameKey);
    record[name] = {
      id: property.type === "title" ? "title" : property.id,
      name,
      description: property.description.trim(),
      type: property.type,
      [property.type]: propertyConfig(property),
    };
  }
  return {
    properties: record,
    property_order: properties.map((property) => property.id),
    hidden_property_ids: properties
      .filter((property) => property.hidden && property.type !== "title")
      .map((property) => property.id),
  };
}
