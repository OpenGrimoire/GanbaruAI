use super::*;

#[derive(Deserialize)]
pub struct NotePageCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) title: String,
    pub(in crate::notes) parent: NoteParent,
    #[serde(default)]
    pub(in crate::notes) folder_id: Option<String>,
    pub(in crate::notes) first_block_id: String,
    pub(in crate::notes) after_block_id: Option<String>,
    pub(in crate::notes) properties: Option<Value>,
}

#[derive(Deserialize)]
pub struct NoteFolderCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) project_id: String,
    pub(in crate::notes) parent_folder_id: Option<String>,
    pub(in crate::notes) name: String,
}

#[derive(Deserialize)]
pub struct NoteFolderUpdate {
    pub(in crate::notes) parent_folder_id: Option<String>,
    pub(in crate::notes) name: String,
}

#[derive(Deserialize)]
pub struct NoteChildPageFromBlockCreate {
    pub(in crate::notes) first_block_id: String,
    pub(in crate::notes) title: Option<String>,
    pub(in crate::notes) properties: Option<Value>,
}

#[derive(Deserialize)]
pub struct NoteDuplicatePage {
    pub(in crate::notes) title: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceRowPageCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) title: String,
    pub(in crate::notes) first_block_id: String,
    pub(in crate::notes) properties: Option<Value>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceCsvImportRequest {
    pub(in crate::notes) csv: String,
    pub(in crate::notes) has_header: Option<bool>,
    pub(in crate::notes) dry_run: Option<bool>,
}

#[derive(Serialize)]
pub struct NoteDataSourceCsvImportColumnDto {
    pub(in crate::notes) source_index: i64,
    pub(in crate::notes) source_name: String,
    pub(in crate::notes) property_id: Option<String>,
    pub(in crate::notes) property_name: Option<String>,
    pub(in crate::notes) property_type: Option<String>,
    pub(in crate::notes) mapped: bool,
    pub(in crate::notes) read_only: bool,
    pub(in crate::notes) warning: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct NoteDataSourceCsvImportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) row_number: Option<i64>,
    pub(in crate::notes) column_index: Option<i64>,
    pub(in crate::notes) column_name: Option<String>,
    pub(in crate::notes) property_id: Option<String>,
    pub(in crate::notes) message: String,
}

#[derive(Serialize)]
pub struct NoteDataSourceCsvImportRowDto {
    pub(in crate::notes) row_number: i64,
    pub(in crate::notes) title: String,
    pub(in crate::notes) valid: bool,
    pub(in crate::notes) mapped_cell_count: i64,
    pub(in crate::notes) error_count: i64,
}

#[derive(Serialize)]
pub struct NoteDataSourceCsvImportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) data_source_id: String,
    pub(in crate::notes) dry_run: bool,
    pub(in crate::notes) total_row_count: i64,
    pub(in crate::notes) valid_row_count: i64,
    pub(in crate::notes) skipped_row_count: i64,
    pub(in crate::notes) imported_row_count: i64,
    pub(in crate::notes) imported_page_ids: Vec<String>,
    pub(in crate::notes) columns: Vec<NoteDataSourceCsvImportColumnDto>,
    pub(in crate::notes) rows: Vec<NoteDataSourceCsvImportRowDto>,
    pub(in crate::notes) diagnostics: Vec<NoteDataSourceCsvImportDiagnosticDto>,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceCsvExportRequest {
    pub(in crate::notes) database_id: Option<String>,
    pub(in crate::notes) view_id: Option<String>,
    pub(in crate::notes) scope: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct NoteDataSourceCsvExportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) property_id: Option<String>,
    pub(in crate::notes) property_name: Option<String>,
    pub(in crate::notes) message: String,
}

#[derive(Clone, Serialize)]
pub struct NoteDataSourceCsvExportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) data_source_id: String,
    pub(in crate::notes) database_id: String,
    pub(in crate::notes) view_id: String,
    pub(in crate::notes) scope: String,
    pub(in crate::notes) file_name: String,
    pub(in crate::notes) csv: String,
    pub(in crate::notes) exported_row_count: i64,
    pub(in crate::notes) exported_property_count: i64,
    pub(in crate::notes) diagnostics: Vec<NoteDataSourceCsvExportDiagnosticDto>,
}

#[derive(Serialize)]
pub struct NoteDataSourceCsvExportSaveDto {
    saved: bool,
    export: Option<NoteDataSourceCsvExportDto>,
}

impl NoteDataSourceCsvExportSaveDto {
    pub(in crate::notes) fn saved(export: NoteDataSourceCsvExportDto) -> Self {
        Self {
            saved: true,
            export: Some(export),
        }
    }

    pub(in crate::notes) fn canceled() -> Self {
        Self {
            saved: false,
            export: None,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteDataSourceTemplateCreateFromRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) source_page_id: String,
    pub(in crate::notes) name: String,
    pub(in crate::notes) is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceTemplateApply {
    pub(in crate::notes) title: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceTemplateUpdate {
    pub(in crate::notes) name: Option<String>,
    pub(in crate::notes) source_page_id: Option<String>,
    pub(in crate::notes) is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceTemplateDuplicate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) name: String,
    pub(in crate::notes) is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceButtonClick {
    pub(in crate::notes) property_id: String,
    pub(in crate::notes) confirmed: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NoteDataSourceTableFilter {
    pub(in crate::notes) property_id: String,
    pub(in crate::notes) condition: String,
    pub(in crate::notes) value: Option<Value>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NoteDataSourceTableSort {
    pub(in crate::notes) property_id: String,
    pub(in crate::notes) direction: String,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceTableConfigurationUpdate {
    pub(in crate::notes) property_order: Vec<String>,
    pub(in crate::notes) hidden_property_ids: Vec<String>,
    pub(in crate::notes) column_widths: Value,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceTableViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceTableConfigurationUpdate,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NoteDataSourceViewWindowRequest {
    pub(in crate::notes) start_cursor: Option<String>,
    pub(in crate::notes) page_size: Option<i64>,
    pub(in crate::notes) range_start: Option<String>,
    pub(in crate::notes) range_end: Option<String>,
}

pub(in crate::notes) struct NoteDataSourceRowWindow {
    pub(in crate::notes) rows: Vec<NotePageRow>,
    pub(in crate::notes) total_row_count: i64,
    pub(in crate::notes) next_cursor: Option<String>,
    pub(in crate::notes) has_more: bool,
    pub(in crate::notes) group_counts: std::collections::HashMap<String, i64>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceRowPropertyUpdate {
    pub(in crate::notes) property_id: String,
    pub(in crate::notes) value: Value,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceBoardConfigurationUpdate {
    pub(in crate::notes) group_property_id: Option<String>,
    pub(in crate::notes) group_order: Vec<String>,
    pub(in crate::notes) hidden_group_ids: Vec<String>,
    pub(in crate::notes) visible_property_ids: Vec<String>,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceBoardViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceBoardConfigurationUpdate,
}

#[derive(Deserialize)]
pub struct NoteDataSourceBoardRowMove {
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) group_id: String,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceGalleryConfigurationUpdate {
    pub(in crate::notes) cover_source: String,
    pub(in crate::notes) cover_property_id: Option<String>,
    pub(in crate::notes) visible_property_ids: Vec<String>,
    pub(in crate::notes) card_size: String,
    pub(in crate::notes) fit_image: bool,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceGalleryViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceGalleryConfigurationUpdate,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceListConfigurationUpdate {
    pub(in crate::notes) group_property_id: Option<String>,
    pub(in crate::notes) group_order: Vec<String>,
    pub(in crate::notes) hidden_group_ids: Vec<String>,
    pub(in crate::notes) visible_property_ids: Vec<String>,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceListViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceListConfigurationUpdate,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceCalendarConfigurationUpdate {
    pub(in crate::notes) date_property_id: Option<String>,
    pub(in crate::notes) range_start: String,
    pub(in crate::notes) range_end: String,
    pub(in crate::notes) visible_property_ids: Vec<String>,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceCalendarViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceCalendarConfigurationUpdate,
}

#[derive(Clone, Deserialize)]
pub struct NoteDataSourceTimelineConfigurationUpdate {
    pub(in crate::notes) date_property_id: Option<String>,
    pub(in crate::notes) group_property_id: Option<String>,
    pub(in crate::notes) group_order: Vec<String>,
    pub(in crate::notes) hidden_group_ids: Vec<String>,
    pub(in crate::notes) range_start: String,
    pub(in crate::notes) range_end: String,
    pub(in crate::notes) visible_property_ids: Vec<String>,
    pub(in crate::notes) row_open_mode: String,
}

#[derive(Deserialize)]
pub struct NoteDataSourceTimelineViewUpdate {
    pub(in crate::notes) filter: Vec<NoteDataSourceTableFilter>,
    pub(in crate::notes) sorts: Vec<NoteDataSourceTableSort>,
    pub(in crate::notes) configuration: NoteDataSourceTimelineConfigurationUpdate,
}

#[derive(Deserialize)]
pub struct NoteMovePage {
    pub(in crate::notes) parent: NoteParent,
    #[serde(default)]
    pub(in crate::notes) folder_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NotePageTemplateCreateFromPage {
    pub(in crate::notes) id: String,
    pub(in crate::notes) source_page_id: String,
    pub(in crate::notes) name: String,
}

#[derive(Deserialize)]
pub struct NotePageTemplateApply {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) title: Option<String>,
}

#[derive(Deserialize)]
pub struct NotePageTemplateUpdate {
    pub(in crate::notes) name: Option<String>,
    pub(in crate::notes) source_page_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NotePageTemplateDuplicate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) name: String,
}

#[derive(Deserialize)]
pub struct NotePageHistorySettingsUpdate {
    pub(in crate::notes) retention_days: Option<i64>,
}

#[derive(Deserialize)]
pub struct NotePageHistoryCopyBlocks {
    pub(in crate::notes) after_block_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::notes) enum OptionalJsonValue {
    #[default]
    Unset,
    Null,
    Value(Value),
}

impl OptionalJsonValue {
    pub(in crate::notes) fn is_set(&self) -> bool {
        !matches!(self, Self::Unset)
    }

    pub(in crate::notes) fn value(&self) -> Option<&Value> {
        match self {
            Self::Value(value) => Some(value),
            Self::Unset | Self::Null => None,
        }
    }

    pub(in crate::notes) fn storage_value(&self) -> Option<String> {
        match self {
            Self::Value(value) => Some(value.to_string()),
            Self::Unset | Self::Null => None,
        }
    }
}

impl<'de> Deserialize<'de> for OptionalJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if value.is_null() {
            Ok(Self::Null)
        } else {
            Ok(Self::Value(value))
        }
    }
}

#[derive(Deserialize)]
pub struct NotePageUpdate {
    pub(in crate::notes) title: Option<String>,
    pub(in crate::notes) parent: Option<NoteParent>,
    pub(in crate::notes) properties: Option<Value>,
    #[serde(default)]
    pub(in crate::notes) icon: OptionalJsonValue,
    #[serde(default)]
    pub(in crate::notes) cover: OptionalJsonValue,
}

#[derive(Deserialize)]
pub struct NoteBlockWrite {
    pub(in crate::notes) id: String,
    #[serde(rename = "type")]
    pub(in crate::notes) block_type: String,
    pub(in crate::notes) paragraph: Option<Value>,
    pub(in crate::notes) heading_1: Option<Value>,
    pub(in crate::notes) heading_2: Option<Value>,
    pub(in crate::notes) heading_3: Option<Value>,
    pub(in crate::notes) heading_4: Option<Value>,
    pub(in crate::notes) bulleted_list_item: Option<Value>,
    pub(in crate::notes) numbered_list_item: Option<Value>,
    pub(in crate::notes) to_do: Option<Value>,
    pub(in crate::notes) toggle: Option<Value>,
    pub(in crate::notes) callout: Option<Value>,
    pub(in crate::notes) quote: Option<Value>,
    pub(in crate::notes) child_page: Option<Value>,
    pub(in crate::notes) child_database: Option<Value>,
    pub(in crate::notes) breadcrumb: Option<Value>,
    pub(in crate::notes) table_of_contents: Option<Value>,
    pub(in crate::notes) column_list: Option<Value>,
    pub(in crate::notes) column: Option<Value>,
    pub(in crate::notes) table: Option<Value>,
    pub(in crate::notes) table_row: Option<Value>,
    pub(in crate::notes) tab: Option<Value>,
    pub(in crate::notes) image: Option<Value>,
    pub(in crate::notes) video: Option<Value>,
    pub(in crate::notes) audio: Option<Value>,
    pub(in crate::notes) file: Option<Value>,
    pub(in crate::notes) pdf: Option<Value>,
    pub(in crate::notes) bookmark: Option<Value>,
    pub(in crate::notes) link_preview: Option<Value>,
    pub(in crate::notes) synced_block: Option<Value>,
    pub(in crate::notes) template: Option<Value>,
    pub(in crate::notes) button: Option<Value>,
    pub(in crate::notes) embed: Option<Value>,
    pub(in crate::notes) equation: Option<Value>,
    pub(in crate::notes) divider: Option<Value>,
    pub(in crate::notes) code: Option<Value>,
    pub(in crate::notes) unsupported: Option<Value>,
}

impl NoteBlockWrite {
    pub(in crate::notes) fn payload(&self) -> Option<&Value> {
        payload_refs::payload_from_write(self)
    }
}

#[derive(Deserialize)]
pub struct NoteBlockUpdate {
    #[serde(rename = "type")]
    pub(in crate::notes) block_type: Option<String>,
    pub(in crate::notes) paragraph: Option<Value>,
    pub(in crate::notes) heading_1: Option<Value>,
    pub(in crate::notes) heading_2: Option<Value>,
    pub(in crate::notes) heading_3: Option<Value>,
    pub(in crate::notes) heading_4: Option<Value>,
    pub(in crate::notes) bulleted_list_item: Option<Value>,
    pub(in crate::notes) numbered_list_item: Option<Value>,
    pub(in crate::notes) to_do: Option<Value>,
    pub(in crate::notes) toggle: Option<Value>,
    pub(in crate::notes) callout: Option<Value>,
    pub(in crate::notes) quote: Option<Value>,
    pub(in crate::notes) child_page: Option<Value>,
    pub(in crate::notes) child_database: Option<Value>,
    pub(in crate::notes) breadcrumb: Option<Value>,
    pub(in crate::notes) table_of_contents: Option<Value>,
    pub(in crate::notes) column_list: Option<Value>,
    pub(in crate::notes) column: Option<Value>,
    pub(in crate::notes) table: Option<Value>,
    pub(in crate::notes) table_row: Option<Value>,
    pub(in crate::notes) tab: Option<Value>,
    pub(in crate::notes) image: Option<Value>,
    pub(in crate::notes) video: Option<Value>,
    pub(in crate::notes) audio: Option<Value>,
    pub(in crate::notes) file: Option<Value>,
    pub(in crate::notes) pdf: Option<Value>,
    pub(in crate::notes) bookmark: Option<Value>,
    pub(in crate::notes) link_preview: Option<Value>,
    pub(in crate::notes) synced_block: Option<Value>,
    pub(in crate::notes) template: Option<Value>,
    pub(in crate::notes) button: Option<Value>,
    pub(in crate::notes) embed: Option<Value>,
    pub(in crate::notes) equation: Option<Value>,
    pub(in crate::notes) divider: Option<Value>,
    pub(in crate::notes) code: Option<Value>,
    pub(in crate::notes) unsupported: Option<Value>,
}

impl NoteBlockUpdate {
    pub(in crate::notes) fn payload_for(&self, block_type: &str) -> Option<&Value> {
        payload_refs::payload_from_update(self, block_type)
    }
}

#[derive(Deserialize)]
pub struct NoteAppendBlockChildren {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) after: Option<String>,
    pub(in crate::notes) children: Vec<NoteBlockWrite>,
}

#[derive(Deserialize)]
pub struct NoteMoveBlock {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) after: Option<String>,
    pub(in crate::notes) before: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteMoveBlocks {
    pub(in crate::notes) block_ids: Vec<String>,
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) after: Option<String>,
    pub(in crate::notes) before: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteDuplicatedBlockId {
    pub(in crate::notes) source_id: String,
    pub(in crate::notes) duplicate_id: String,
}

#[derive(Deserialize)]
pub struct NoteDuplicateBlock {
    pub(in crate::notes) duplicated_block_ids: Vec<NoteDuplicatedBlockId>,
}

#[derive(Deserialize)]
pub struct NoteDuplicateBlocks {
    pub(in crate::notes) block_ids: Vec<String>,
    pub(in crate::notes) duplicated_block_ids: Vec<NoteDuplicatedBlockId>,
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) after: Option<String>,
    pub(in crate::notes) before: Option<String>,
    pub(in crate::notes) include_trashed_sources: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteTrashBlocks {
    pub(in crate::notes) block_ids: Vec<String>,
    pub(in crate::notes) in_trash: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteCommentCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent: Option<NoteParent>,
    pub(in crate::notes) discussion_id: Option<String>,
    pub(in crate::notes) anchor: Option<NoteCommentAnchorCreate>,
    pub(in crate::notes) rich_text: Vec<Value>,
    #[serde(default)]
    pub(in crate::notes) attachments: Option<Vec<Value>>,
}

#[derive(Deserialize)]
pub struct NoteCommentAnchorCreate {
    pub(in crate::notes) start: i64,
    pub(in crate::notes) end: i64,
    pub(in crate::notes) text: String,
    pub(in crate::notes) prefix: String,
    pub(in crate::notes) suffix: String,
}

#[derive(Deserialize)]
pub struct NoteCommentUpdate {
    pub(in crate::notes) rich_text: Vec<Value>,
    #[serde(default)]
    pub(in crate::notes) attachments: Option<Vec<Value>>,
}

#[derive(Deserialize)]
pub struct NoteCommentThreadReadUpdate {
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) discussion_ids: Vec<String>,
    pub(in crate::notes) include_resolved: Option<bool>,
}

#[derive(Deserialize)]
pub struct NoteMentionNotificationDeliveryUpdate {
    pub(in crate::notes) ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct NoteLocalUserUpdate {
    pub(in crate::notes) display_name: String,
}

#[derive(Deserialize)]
pub struct NoteDatabaseCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) data_source_id: String,
    pub(in crate::notes) view_id: String,
    pub(in crate::notes) title: String,
    pub(in crate::notes) parent: Option<NoteParent>,
    pub(in crate::notes) after_block_id: Option<String>,
    pub(in crate::notes) replace_block_id: Option<String>,
    pub(in crate::notes) icon: Option<Value>,
    pub(in crate::notes) cover: Option<Value>,
}

#[derive(Deserialize)]
pub struct NoteLinkedDatabaseCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) view_id: String,
    pub(in crate::notes) source_block_id: String,
    pub(in crate::notes) title: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteDataSourceSchemaUpdate {
    pub(in crate::notes) properties: Value,
    pub(in crate::notes) property_order: Vec<String>,
    pub(in crate::notes) hidden_property_ids: Vec<String>,
}
