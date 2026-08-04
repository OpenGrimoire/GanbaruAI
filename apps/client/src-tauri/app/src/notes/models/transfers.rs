use super::*;

#[derive(Deserialize)]
pub struct NoteMarkdownImportRequest {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) markdown: String,
    pub(in crate::notes) title: Option<String>,
    pub(in crate::notes) source_name: Option<String>,
    pub(in crate::notes) after_block_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteMarkdownImportDiagnosticDto {
    code: String,
    severity: String,
    line: Option<i64>,
    message: String,
}

impl NoteMarkdownImportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        line: Option<i64>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            line,
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct NoteMarkdownImportDto {
    page: NoteLoadedPage,
    diagnostics: Vec<NoteMarkdownImportDiagnosticDto>,
    imported_block_count: i64,
}

impl NoteMarkdownImportDto {
    pub(in crate::notes) fn new(
        page: NoteLoadedPage,
        diagnostics: Vec<NoteMarkdownImportDiagnosticDto>,
        imported_block_count: i64,
    ) -> Self {
        Self {
            page,
            diagnostics,
            imported_block_count,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteHtmlImportRequest {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) html: String,
    pub(in crate::notes) title: Option<String>,
    pub(in crate::notes) source_name: Option<String>,
    pub(in crate::notes) after_block_id: Option<String>,
    pub(in crate::notes) keep_external_file_references: Option<bool>,
    pub(in crate::notes) project_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteHtmlImportDiagnosticDto {
    code: String,
    severity: String,
    line: Option<i64>,
    message: String,
}

impl NoteHtmlImportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        line: Option<i64>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            line,
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct NoteHtmlImportDto {
    page: NoteLoadedPage,
    diagnostics: Vec<NoteHtmlImportDiagnosticDto>,
    imported_block_count: i64,
}

impl NoteHtmlImportDto {
    pub(in crate::notes) fn new(
        page: NoteLoadedPage,
        diagnostics: Vec<NoteHtmlImportDiagnosticDto>,
        imported_block_count: i64,
    ) -> Self {
        Self {
            page,
            diagnostics,
            imported_block_count,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteNotionApiImportRequest {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) integration_token: String,
    pub(in crate::notes) source_workspace_id: Option<String>,
    #[serde(default)]
    pub(in crate::notes) page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) data_source_ids: Vec<String>,
    pub(in crate::notes) include_comments: Option<bool>,
    pub(in crate::notes) include_users: Option<bool>,
    pub(in crate::notes) keep_external_file_references: Option<bool>,
    pub(in crate::notes) page_size: Option<i64>,
    pub(in crate::notes) project_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteNotionApiImportDiagnosticDto {
    code: String,
    severity: String,
    source_object_id: Option<String>,
    message: String,
}

impl NoteNotionApiImportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        source_object_id: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            source_object_id: source_object_id.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteNotionApiImportedObjectDto {
    object_type: String,
    source_object_id: String,
    local_id: String,
    title: String,
}

impl NoteNotionApiImportedObjectDto {
    pub(in crate::notes) fn new(
        object_type: impl Into<String>,
        source_object_id: impl Into<String>,
        local_id: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            object_type: object_type.into(),
            source_object_id: source_object_id.into(),
            local_id: local_id.into(),
            title: title.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteNotionApiImportedUserDto {
    source_user_id: String,
    name: String,
    user_type: String,
}

impl NoteNotionApiImportedUserDto {
    pub(in crate::notes) fn new(
        source_user_id: impl Into<String>,
        name: impl Into<String>,
        user_type: impl Into<String>,
    ) -> Self {
        Self {
            source_user_id: source_user_id.into(),
            name: name.into(),
            user_type: user_type.into(),
        }
    }
}

#[derive(Serialize)]
pub struct NoteNotionApiImportDto {
    object: &'static str,
    imported_pages: Vec<NoteLoadedPage>,
    imported_data_sources: Vec<NoteNotionApiImportedObjectDto>,
    imported_users: Vec<NoteNotionApiImportedUserDto>,
    diagnostics: Vec<NoteNotionApiImportDiagnosticDto>,
    request_count: i64,
    retry_count: i64,
    rate_limit_count: i64,
    imported_page_count: i64,
    imported_block_count: i64,
    imported_data_source_count: i64,
    imported_comment_count: i64,
    imported_user_count: i64,
    imported_file_count: i64,
    unsupported_block_count: i64,
}

pub(in crate::notes) struct NoteNotionApiImportSummary {
    pub(in crate::notes) imported_pages: Vec<NoteLoadedPage>,
    pub(in crate::notes) imported_data_sources: Vec<NoteNotionApiImportedObjectDto>,
    pub(in crate::notes) imported_users: Vec<NoteNotionApiImportedUserDto>,
    pub(in crate::notes) diagnostics: Vec<NoteNotionApiImportDiagnosticDto>,
    pub(in crate::notes) request_count: i64,
    pub(in crate::notes) retry_count: i64,
    pub(in crate::notes) rate_limit_count: i64,
    pub(in crate::notes) imported_block_count: i64,
    pub(in crate::notes) imported_comment_count: i64,
    pub(in crate::notes) imported_file_count: i64,
    pub(in crate::notes) unsupported_block_count: i64,
}

impl NoteNotionApiImportDto {
    pub(in crate::notes) fn new(summary: NoteNotionApiImportSummary) -> Self {
        let imported_page_count = summary.imported_pages.len() as i64;
        let imported_data_source_count = summary.imported_data_sources.len() as i64;
        let imported_user_count = summary.imported_users.len() as i64;
        Self {
            object: "notes_notion_api_import",
            imported_pages: summary.imported_pages,
            imported_data_sources: summary.imported_data_sources,
            imported_users: summary.imported_users,
            diagnostics: summary.diagnostics,
            request_count: summary.request_count,
            retry_count: summary.retry_count,
            rate_limit_count: summary.rate_limit_count,
            imported_page_count,
            imported_block_count: summary.imported_block_count,
            imported_data_source_count,
            imported_comment_count: summary.imported_comment_count,
            imported_user_count,
            imported_file_count: summary.imported_file_count,
            unsupported_block_count: summary.unsupported_block_count,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteNotionExportImportRequest {
    pub(in crate::notes) parent: NoteParent,
    pub(in crate::notes) export_root_path: String,
    pub(in crate::notes) source_workspace_id: Option<String>,
    pub(in crate::notes) keep_external_file_references: Option<bool>,
    pub(in crate::notes) copy_local_file_references: Option<bool>,
    pub(in crate::notes) import_markdown: Option<bool>,
    pub(in crate::notes) import_html: Option<bool>,
    pub(in crate::notes) import_csv: Option<bool>,
    pub(in crate::notes) project_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoteNotionExportImportDiagnosticDto {
    code: String,
    severity: String,
    source_path: Option<String>,
    message: String,
}

impl NoteNotionExportImportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        source_path: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            source_path: source_path.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct NoteNotionExportImportDto {
    object: &'static str,
    imported_pages: Vec<NoteLoadedPage>,
    imported_data_sources: Vec<NoteNotionApiImportedObjectDto>,
    diagnostics: Vec<NoteNotionExportImportDiagnosticDto>,
    imported_page_count: i64,
    imported_block_count: i64,
    imported_data_source_count: i64,
    imported_file_count: i64,
    skipped_file_count: i64,
    unsupported_block_count: i64,
}

pub(in crate::notes) struct NoteNotionExportImportSummary {
    pub(in crate::notes) imported_pages: Vec<NoteLoadedPage>,
    pub(in crate::notes) imported_data_sources: Vec<NoteNotionApiImportedObjectDto>,
    pub(in crate::notes) diagnostics: Vec<NoteNotionExportImportDiagnosticDto>,
    pub(in crate::notes) imported_block_count: i64,
    pub(in crate::notes) imported_file_count: i64,
    pub(in crate::notes) skipped_file_count: i64,
    pub(in crate::notes) unsupported_block_count: i64,
}

impl NoteNotionExportImportDto {
    pub(in crate::notes) fn new(summary: NoteNotionExportImportSummary) -> Self {
        let imported_page_count = summary.imported_pages.len() as i64;
        let imported_data_source_count = summary.imported_data_sources.len() as i64;
        Self {
            object: "notes_notion_export_import",
            imported_pages: summary.imported_pages,
            imported_data_sources: summary.imported_data_sources,
            diagnostics: summary.diagnostics,
            imported_page_count,
            imported_block_count: summary.imported_block_count,
            imported_data_source_count,
            imported_file_count: summary.imported_file_count,
            skipped_file_count: summary.skipped_file_count,
            unsupported_block_count: summary.unsupported_block_count,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteMarkdownExportRequest {
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) include_page_title: Option<bool>,
    pub(in crate::notes) include_comments: Option<bool>,
    pub(in crate::notes) include_resolved_comments: Option<bool>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteMarkdownExportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) block_id: Option<String>,
    pub(in crate::notes) comment_id: Option<String>,
    pub(in crate::notes) message: String,
}

impl NoteMarkdownExportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        block_id: Option<impl Into<String>>,
        comment_id: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            block_id: block_id.map(Into::into),
            comment_id: comment_id.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
pub struct NoteMarkdownExportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) markdown: String,
    pub(in crate::notes) diagnostics: Vec<NoteMarkdownExportDiagnosticDto>,
    pub(in crate::notes) exported_block_count: i64,
    pub(in crate::notes) exported_comment_count: i64,
}

impl NoteMarkdownExportDto {
    pub(in crate::notes) fn new(
        page_id: String,
        markdown: String,
        diagnostics: Vec<NoteMarkdownExportDiagnosticDto>,
        exported_block_count: i64,
        exported_comment_count: i64,
    ) -> Self {
        Self {
            object: "notes_markdown_export",
            page_id,
            markdown,
            diagnostics,
            exported_block_count,
            exported_comment_count,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct NoteHtmlExportRequest {
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) include_page_tree: Option<bool>,
    pub(in crate::notes) include_comments: Option<bool>,
    pub(in crate::notes) include_resolved_comments: Option<bool>,
    pub(in crate::notes) include_assets: Option<bool>,
    pub(in crate::notes) include_database_views: Option<bool>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteHtmlExportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) page_id: Option<String>,
    pub(in crate::notes) block_id: Option<String>,
    pub(in crate::notes) asset_id: Option<String>,
    pub(in crate::notes) comment_id: Option<String>,
    pub(in crate::notes) message: String,
}

impl NoteHtmlExportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        page_id: Option<impl Into<String>>,
        block_id: Option<impl Into<String>>,
        asset_id: Option<impl Into<String>>,
        comment_id: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            page_id: page_id.map(Into::into),
            block_id: block_id.map(Into::into),
            asset_id: asset_id.map(Into::into),
            comment_id: comment_id.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteHtmlExportFileDto {
    pub(in crate::notes) path: String,
    pub(in crate::notes) content_type: String,
    pub(in crate::notes) contents: String,
    pub(in crate::notes) byte_size: i64,
}

impl NoteHtmlExportFileDto {
    pub(in crate::notes) fn new(
        path: impl Into<String>,
        content_type: impl Into<String>,
        contents: impl Into<String>,
    ) -> Self {
        let contents = contents.into();
        Self {
            path: path.into(),
            content_type: content_type.into(),
            byte_size: contents.len() as i64,
            contents,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteHtmlExportAssetDto {
    pub(in crate::notes) id: String,
    pub(in crate::notes) archive_path: String,
    pub(in crate::notes) source_path: String,
    pub(in crate::notes) content_type: String,
    pub(in crate::notes) byte_size: i64,
    pub(in crate::notes) sha256: String,
    pub(in crate::notes) storage_state: String,
    pub(in crate::notes) exported: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct NoteHtmlExportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) root_page_id: String,
    pub(in crate::notes) files: Vec<NoteHtmlExportFileDto>,
    pub(in crate::notes) assets: Vec<NoteHtmlExportAssetDto>,
    pub(in crate::notes) diagnostics: Vec<NoteHtmlExportDiagnosticDto>,
    pub(in crate::notes) manifest_json: String,
    pub(in crate::notes) exported_page_count: i64,
    pub(in crate::notes) exported_block_count: i64,
    pub(in crate::notes) exported_asset_count: i64,
    pub(in crate::notes) exported_comment_count: i64,
    pub(in crate::notes) exported_database_view_count: i64,
}

#[derive(Serialize)]
pub struct NoteHtmlArchiveSaveDto {
    object: &'static str,
    saved: bool,
    export: Option<NoteHtmlExportDto>,
}

impl NoteHtmlArchiveSaveDto {
    pub(in crate::notes) fn canceled() -> Self {
        Self {
            object: "notes_html_archive_save",
            saved: false,
            export: None,
        }
    }

    pub(in crate::notes) fn saved(export: NoteHtmlExportDto) -> Self {
        Self {
            object: "notes_html_archive_save",
            saved: true,
            export: Some(export),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NoteJsonGraphExportRequest {
    pub(in crate::notes) include_indexes: Option<bool>,
    pub(in crate::notes) include_history: Option<bool>,
    pub(in crate::notes) include_templates: Option<bool>,
    pub(in crate::notes) include_local_state: Option<bool>,
    pub(in crate::notes) pretty: Option<bool>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteJsonGraphExportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) table_name: Option<String>,
    pub(in crate::notes) row_id: Option<String>,
    pub(in crate::notes) message: String,
}

impl NoteJsonGraphExportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        table_name: Option<impl Into<String>>,
        row_id: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            table_name: table_name.map(Into::into),
            row_id: row_id.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NoteJsonGraphExportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) export_version: i64,
    pub(in crate::notes) schema_version: String,
    pub(in crate::notes) generated_at: String,
    pub(in crate::notes) file_name: String,
    pub(in crate::notes) content_type: &'static str,
    pub(in crate::notes) json: String,
    pub(in crate::notes) byte_size: i64,
    pub(in crate::notes) counts: Value,
    pub(in crate::notes) diagnostics: Vec<NoteJsonGraphExportDiagnosticDto>,
    pub(in crate::notes) exported_page_count: i64,
    pub(in crate::notes) exported_block_count: i64,
    pub(in crate::notes) exported_comment_count: i64,
    pub(in crate::notes) exported_data_source_count: i64,
    pub(in crate::notes) exported_file_count: i64,
    pub(in crate::notes) exported_index_record_count: i64,
    pub(in crate::notes) exported_property_schema_count: i64,
    pub(in crate::notes) exported_table_count: i64,
    pub(in crate::notes) exported_record_count: i64,
    pub(in crate::notes) warning_count: i64,
}

#[derive(Serialize)]
pub struct NoteJsonGraphExportSaveDto {
    object: &'static str,
    saved: bool,
    export: Option<NoteJsonGraphExportDto>,
}

impl NoteJsonGraphExportSaveDto {
    pub(in crate::notes) fn canceled() -> Self {
        Self {
            object: "notes_json_graph_export_save",
            saved: false,
            export: None,
        }
    }

    pub(in crate::notes) fn saved(export: NoteJsonGraphExportDto) -> Self {
        Self {
            object: "notes_json_graph_export_save",
            saved: true,
            export: Some(export),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NoteAgentBridgeExportRequest {
    #[serde(default)]
    pub(in crate::notes) page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) project_ids: Vec<String>,
    pub(in crate::notes) include_descendants: Option<bool>,
    pub(in crate::notes) include_backlinks: Option<bool>,
    pub(in crate::notes) include_database_views: Option<bool>,
    pub(in crate::notes) include_task_context: Option<bool>,
    pub(in crate::notes) include_page_comments: Option<bool>,
    pub(in crate::notes) include_resolved_comments: Option<bool>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NoteAgentBridgeExportDiagnosticDto {
    pub(in crate::notes) code: String,
    pub(in crate::notes) severity: String,
    pub(in crate::notes) source_type: Option<String>,
    pub(in crate::notes) source_id: Option<String>,
    pub(in crate::notes) message: String,
}

impl NoteAgentBridgeExportDiagnosticDto {
    pub(in crate::notes) fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        source_type: Option<impl Into<String>>,
        source_id: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            source_type: source_type.map(Into::into),
            source_id: source_id.map(Into::into),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NoteAgentBridgeExportDto {
    pub(in crate::notes) object: &'static str,
    pub(in crate::notes) export_version: i64,
    pub(in crate::notes) schema_version: &'static str,
    pub(in crate::notes) file_name: String,
    pub(in crate::notes) content_type: &'static str,
    pub(in crate::notes) markdown: String,
    pub(in crate::notes) byte_size: i64,
    pub(in crate::notes) diagnostics: Vec<NoteAgentBridgeExportDiagnosticDto>,
    pub(in crate::notes) exported_page_count: i64,
    pub(in crate::notes) exported_project_count: i64,
    pub(in crate::notes) exported_task_count: i64,
    pub(in crate::notes) exported_database_view_count: i64,
    pub(in crate::notes) exported_backlink_count: i64,
    pub(in crate::notes) warning_count: i64,
}

#[derive(Serialize)]
pub struct NoteAgentBridgeExportSaveDto {
    object: &'static str,
    saved: bool,
    export: Option<NoteAgentBridgeExportDto>,
}

impl NoteAgentBridgeExportSaveDto {
    pub(in crate::notes) fn canceled() -> Self {
        Self {
            object: "notes_agent_bridge_export_save",
            saved: false,
            export: None,
        }
    }

    pub(in crate::notes) fn saved(export: NoteAgentBridgeExportDto) -> Self {
        Self {
            object: "notes_agent_bridge_export_save",
            saved: true,
            export: Some(export),
        }
    }
}
