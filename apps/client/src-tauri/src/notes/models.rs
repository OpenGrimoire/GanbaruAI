use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum NoteParent {
    #[serde(rename = "workspace")]
    Workspace { workspace: bool },
    #[serde(rename = "page_id")]
    PageId { page_id: String },
    #[serde(rename = "block_id")]
    BlockId { block_id: String },
    #[serde(rename = "data_source_id")]
    DataSourceId { data_source_id: String },
}

#[derive(Serialize)]
pub struct NotePageDto {
    object: &'static str,
    id: String,
    created_time: String,
    last_edited_time: String,
    parent: NoteParent,
    folder_id: Option<String>,
    in_trash: bool,
    archived: bool,
    icon: Option<Value>,
    cover: Option<Value>,
    properties: Value,
    url: Option<String>,
    public_url: Option<String>,
    source_provider: Option<String>,
    source_object_id: Option<String>,
    source_workspace_id: Option<String>,
    source_last_edited_time: Option<String>,
}

impl NotePageDto {
    pub(in crate::notes) fn new(row: NotePageRow) -> Result<Self, String> {
        let in_trash = row.in_trash != 0;
        let archived = row.archived != 0;
        Ok(Self {
            object: "page",
            id: row.id,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
            parent: parent_from_row(
                &row.parent_type,
                row.parent_page_id,
                row.parent_block_id,
                row.parent_data_source_id,
            )?,
            folder_id: row.folder_id,
            in_trash,
            archived,
            icon: parse_optional_json(row.icon, "page icon")?,
            cover: parse_optional_json(row.cover, "page cover")?,
            properties: parse_json(row.properties, "page properties")?,
            url: row.url,
            public_url: row.public_url,
            source_provider: row.source_provider,
            source_object_id: row.source_object_id,
            source_workspace_id: row.source_workspace_id,
            source_last_edited_time: row.source_last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NoteFolderDto {
    object: &'static str,
    id: String,
    project_id: String,
    parent_folder_id: Option<String>,
    name: String,
    created_time: String,
    last_edited_time: String,
}

impl NoteFolderDto {
    pub(in crate::notes) fn new(row: NoteFolderRow) -> Self {
        Self {
            object: "folder",
            id: row.id,
            project_id: row.project_id,
            parent_folder_id: row.parent_folder_id,
            name: row.name,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Serialize)]
pub struct NoteBlockDto {
    object: &'static str,
    id: String,
    parent: NoteParent,
    created_time: String,
    last_edited_time: String,
    has_children: bool,
    in_trash: bool,
    archived: bool,
    #[serde(rename = "type")]
    block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    paragraph: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_1: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_2: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_3: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heading_4: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bulleted_list_item: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numbered_list_item: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_do: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    toggle: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    callout: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    child_page: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    child_database: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    breadcrumb: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    table_of_contents: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_list: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    table: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    table_row: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tab: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audio: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdf: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bookmark: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_preview: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synced_block: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    button: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embed: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    equation: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    divider: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsupported: Option<Value>,
    source_provider: Option<String>,
    source_object_id: Option<String>,
    source_last_edited_time: Option<String>,
}

impl NoteBlockDto {
    pub(in crate::notes) fn new(row: NoteBlockRow) -> Result<Self, String> {
        let payload = parse_json(row.payload, "block payload")?;
        let in_trash = row.in_trash != 0;
        let mut block = Self {
            object: "block",
            id: row.id,
            parent: parent_from_row(
                &row.parent_type,
                row.parent_page_id,
                row.parent_block_id,
                None,
            )?,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
            has_children: row.has_children != 0,
            in_trash,
            archived: in_trash,
            block_type: row.block_type.clone(),
            paragraph: None,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            heading_4: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            to_do: None,
            toggle: None,
            callout: None,
            quote: None,
            child_page: None,
            child_database: None,
            breadcrumb: None,
            table_of_contents: None,
            column_list: None,
            column: None,
            table: None,
            table_row: None,
            tab: None,
            image: None,
            video: None,
            audio: None,
            file: None,
            pdf: None,
            bookmark: None,
            link_preview: None,
            synced_block: None,
            template: None,
            button: None,
            embed: None,
            equation: None,
            divider: None,
            code: None,
            unsupported: None,
            source_provider: row.source_provider,
            source_object_id: row.source_object_id,
            source_last_edited_time: row.source_last_edited_time,
        };
        match row.block_type.as_str() {
            "paragraph" => block.paragraph = Some(payload),
            "heading_1" => block.heading_1 = Some(payload),
            "heading_2" => block.heading_2 = Some(payload),
            "heading_3" => block.heading_3 = Some(payload),
            "heading_4" => block.heading_4 = Some(payload),
            "bulleted_list_item" => block.bulleted_list_item = Some(payload),
            "numbered_list_item" => block.numbered_list_item = Some(payload),
            "to_do" => block.to_do = Some(payload),
            "toggle" => block.toggle = Some(payload),
            "callout" => block.callout = Some(payload),
            "quote" => block.quote = Some(payload),
            "child_page" => block.child_page = Some(payload),
            "child_database" => block.child_database = Some(payload),
            "breadcrumb" => block.breadcrumb = Some(payload),
            "table_of_contents" => block.table_of_contents = Some(payload),
            "column_list" => block.column_list = Some(payload),
            "column" => block.column = Some(payload),
            "table" => block.table = Some(payload),
            "table_row" => block.table_row = Some(payload),
            "tab" => block.tab = Some(payload),
            "image" => block.image = Some(payload),
            "video" => block.video = Some(payload),
            "audio" => block.audio = Some(payload),
            "file" => block.file = Some(payload),
            "pdf" => block.pdf = Some(payload),
            "bookmark" => block.bookmark = Some(payload),
            "link_preview" => block.link_preview = Some(payload),
            "synced_block" => block.synced_block = Some(payload),
            "template" => block.template = Some(payload),
            "button" => block.button = Some(payload),
            "embed" => block.embed = Some(payload),
            "equation" => block.equation = Some(payload),
            "divider" => block.divider = Some(payload),
            "code" => block.code = Some(payload),
            "unsupported" => block.unsupported = Some(payload),
            other => return Err(format!("unsupported block type in storage: {other}")),
        }
        Ok(block)
    }
}

#[derive(Serialize)]
pub struct NotePaginatedBlockList {
    object: &'static str,
    #[serde(rename = "type")]
    list_type: &'static str,
    block: Value,
    results: Vec<NoteBlockDto>,
    next_cursor: Option<String>,
    has_more: bool,
}

impl NotePaginatedBlockList {
    pub(in crate::notes) fn new(
        results: Vec<NoteBlockDto>,
        next_cursor: Option<String>,
        has_more: bool,
    ) -> Self {
        Self {
            object: "list",
            list_type: "block",
            block: Value::Object(serde_json::Map::new()),
            results,
            next_cursor,
            has_more,
        }
    }
}

#[derive(Serialize)]
pub struct NoteLoadedPage {
    page: NotePageDto,
    blocks: NotePaginatedBlockList,
}

#[derive(Serialize)]
pub struct NotePageOpenDto {
    page: NotePageDto,
    breadcrumb: Vec<NotePageBreadcrumbItemDto>,
    blocks: NotePaginatedBlockList,
    outlines: Vec<NoteBlockOutlineDto>,
}

impl NotePageOpenDto {
    pub(in crate::notes) fn new(
        page: NotePageDto,
        breadcrumb: Vec<NotePageBreadcrumbItemDto>,
        blocks: NotePaginatedBlockList,
        outlines: Vec<NoteBlockOutlineDto>,
    ) -> Self {
        Self {
            page,
            breadcrumb,
            blocks,
            outlines,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NoteBlockOutlineDto {
    id: String,
    page_id: String,
    parent: NoteParent,
    #[serde(rename = "type")]
    block_type: String,
    sort_order: f64,
    has_children: bool,
    retained_height: i64,
}

impl NoteBlockOutlineDto {
    pub(in crate::notes) fn new(
        id: String,
        page_id: String,
        parent: NoteParent,
        block_type: String,
        sort_order: f64,
        has_children: bool,
        retained_height: i64,
    ) -> Self {
        Self {
            id,
            page_id,
            parent,
            block_type,
            sort_order,
            has_children,
            retained_height,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteBlockHydrationRequest {
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) block_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct NoteBlockFrontierDto {
    blocks: Vec<NoteBlockDto>,
}

impl NoteBlockFrontierDto {
    pub(in crate::notes) fn new(blocks: Vec<NoteBlockDto>) -> Self {
        Self { blocks }
    }
}

impl NoteLoadedPage {
    pub(in crate::notes) fn new(page: NotePageDto, blocks: NotePaginatedBlockList) -> Self {
        Self { page, blocks }
    }
}

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

#[derive(Deserialize)]
pub struct NoteSidebarPagesRequest {
    #[serde(default)]
    pub(in crate::notes) expanded_page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) seed_page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) selected_page_id: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteWorkspaceShellRequest {
    pub(in crate::notes) project_id: Option<String>,
    #[serde(default)]
    pub(in crate::notes) expanded_page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) seed_page_ids: Vec<String>,
    #[serde(default)]
    pub(in crate::notes) selected_page_id: Option<String>,
    #[serde(default)]
    pub(in crate::notes) page_cursor: Option<String>,
    #[serde(default)]
    pub(in crate::notes) folder_cursor: Option<String>,
    #[serde(default)]
    pub(in crate::notes) destination_candidates: bool,
    #[serde(default)]
    pub(in crate::notes) page_query: Option<String>,
}

#[derive(Clone, Serialize, sqlx::FromRow)]
pub(in crate::notes) struct NotePageSummaryDto {
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) parent_data_source_id: Option<String>,
    pub(in crate::notes) folder_id: Option<String>,
    pub(in crate::notes) title: String,
    pub(in crate::notes) project_id: Option<String>,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) in_trash: bool,
    pub(in crate::notes) archived: bool,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}

#[derive(Default, Deserialize)]
pub struct NotePageSummaryWindowRequest {
    pub(in crate::notes) cursor: Option<String>,
    pub(in crate::notes) query: Option<String>,
    pub(in crate::notes) page_size: Option<i64>,
}

#[derive(Serialize)]
pub struct NotePageSummaryWindowDto {
    pages: Vec<NotePageSummaryDto>,
    total_count: i64,
    next_cursor: Option<String>,
}

impl NotePageSummaryWindowDto {
    pub(in crate::notes) fn new(
        pages: Vec<NotePageSummaryDto>,
        total_count: i64,
        next_cursor: Option<String>,
    ) -> Self {
        Self {
            pages,
            total_count,
            next_cursor,
        }
    }
}

#[derive(Serialize)]
pub struct NoteWorkspaceShellDto {
    pages: Vec<NotePageSummaryDto>,
    folders: Vec<NoteFolderDto>,
    page_ids_with_children: Vec<String>,
    missing_parent_page_ids: Vec<String>,
    trashed_parent_page_ids: Vec<String>,
    resolved_selected_page_id: Option<String>,
    total_page_count: i64,
    total_folder_count: i64,
    next_page_cursor: Option<String>,
    next_folder_cursor: Option<String>,
}

impl NoteWorkspaceShellDto {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::notes) fn new(
        pages: Vec<NotePageSummaryDto>,
        folders: Vec<NoteFolderDto>,
        page_ids_with_children: Vec<String>,
        missing_parent_page_ids: Vec<String>,
        trashed_parent_page_ids: Vec<String>,
        resolved_selected_page_id: Option<String>,
        total_page_count: i64,
        total_folder_count: i64,
        next_page_cursor: Option<String>,
        next_folder_cursor: Option<String>,
    ) -> Self {
        Self {
            pages,
            folders,
            page_ids_with_children,
            missing_parent_page_ids,
            trashed_parent_page_ids,
            resolved_selected_page_id,
            total_page_count,
            total_folder_count,
            next_page_cursor,
            next_folder_cursor,
        }
    }
}

#[derive(Serialize)]
pub struct NoteSidebarPageList {
    pages: Vec<NotePageSummaryDto>,
    page_ids_with_children: Vec<String>,
    missing_parent_page_ids: Vec<String>,
    trashed_parent_page_ids: Vec<String>,
}

impl NoteSidebarPageList {
    pub(in crate::notes) fn new(
        pages: Vec<NotePageSummaryDto>,
        page_ids_with_children: Vec<String>,
        missing_parent_page_ids: Vec<String>,
        trashed_parent_page_ids: Vec<String>,
    ) -> Self {
        Self {
            pages,
            page_ids_with_children,
            missing_parent_page_ids,
            trashed_parent_page_ids,
        }
    }
}

#[derive(Serialize)]
pub struct NotePageBreadcrumbItemDto {
    id: Option<String>,
    title: String,
    current: bool,
    status: &'static str,
}

impl NotePageBreadcrumbItemDto {
    pub(in crate::notes) fn active(row: NotePageRow, current: bool) -> Self {
        Self {
            id: Some(row.id),
            title: row.title,
            current,
            status: "active",
        }
    }

    pub(in crate::notes) fn unavailable(row: NotePageRow, status: &'static str) -> Self {
        Self {
            id: Some(row.id),
            title: row.title,
            current: false,
            status,
        }
    }

    pub(in crate::notes) fn missing(page_id: String) -> Self {
        Self {
            id: Some(page_id),
            title: String::new(),
            current: false,
            status: "missing",
        }
    }
}

#[derive(Serialize)]
pub struct NotePageTemplateDto {
    object: &'static str,
    id: String,
    name: String,
    source_page_id: Option<String>,
    properties: Value,
    icon: Option<Value>,
    cover: Option<Value>,
    block_count: i64,
    created_time: String,
    last_edited_time: String,
}

impl NotePageTemplateDto {
    pub(in crate::notes) fn new(row: NotePageTemplateRow) -> Result<Self, String> {
        Ok(Self {
            object: "page_template",
            id: row.id,
            name: row.name,
            source_page_id: row.source_page_id,
            properties: parse_json(row.properties, "page template properties")?,
            icon: parse_optional_json(row.icon, "page template icon")?,
            cover: parse_optional_json(row.cover, "page template cover")?,
            block_count: row.block_count,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceTemplateDto {
    object: &'static str,
    id: String,
    data_source_id: String,
    source_page_id: Option<String>,
    name: String,
    properties: Value,
    is_default: bool,
    block_count: i64,
    created_time: String,
    last_edited_time: String,
}

impl NoteDataSourceTemplateDto {
    pub(in crate::notes) fn new(row: NoteDataSourceTemplateRow) -> Result<Self, String> {
        Ok(Self {
            object: "data_source_template",
            id: row.id,
            data_source_id: row.data_source_id,
            source_page_id: row.source_page_id,
            name: row.name,
            properties: parse_json(row.properties, "data source template properties")?,
            is_default: row.is_default != 0,
            block_count: row.block_count,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NotePageHistorySnapshotDto {
    object: &'static str,
    id: String,
    page_id: String,
    title: String,
    icon: Option<Value>,
    cover: Option<Value>,
    block_count: i64,
    reason: String,
    created_by: NotePartialUserDto,
    created_time: String,
    page_last_edited_time: String,
}

impl NotePageHistorySnapshotDto {
    pub(in crate::notes) fn new(row: NotePageHistorySnapshotRow) -> Result<Self, String> {
        Ok(Self {
            object: "page_history_snapshot",
            id: row.id,
            page_id: row.page_id,
            title: row.title,
            icon: parse_optional_json(row.icon, "page history icon")?,
            cover: parse_optional_json(row.cover, "page history cover")?,
            block_count: row.block_count,
            reason: row.reason,
            created_by: NotePartialUserDto::new(row.created_by),
            created_time: row.created_time,
            page_last_edited_time: row.page_last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NotePageHistorySettingsDto {
    object: &'static str,
    retention_days: Option<i64>,
    updated_at: String,
}

impl NotePageHistorySettingsDto {
    pub(in crate::notes) fn new(row: NotePageHistorySettingsRow) -> Self {
        Self {
            object: "page_history_settings",
            retention_days: row.retention_days,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct NoteMentionNotificationDto {
    object: &'static str,
    id: String,
    source_type: String,
    source_id: String,
    page_id: String,
    page_title: String,
    block_id: Option<String>,
    comment_id: Option<String>,
    kind: String,
    target_type: String,
    target_id: Option<String>,
    trigger_at: Option<String>,
    plain_text: String,
    source_plain_text: String,
    status: String,
    delivered_at: Option<String>,
    created_time: String,
    last_edited_time: String,
}

impl NoteMentionNotificationDto {
    pub(in crate::notes) fn new(row: NoteMentionNotificationRow) -> Self {
        Self {
            object: "mention_notification",
            id: row.id,
            source_type: row.source_type,
            source_id: row.source_id,
            page_id: row.page_id,
            page_title: row.page_title,
            block_id: row.block_id,
            comment_id: row.comment_id,
            kind: row.kind,
            target_type: row.target_type,
            target_id: row.target_id,
            trigger_at: row.trigger_at,
            plain_text: row.plain_text,
            source_plain_text: row.source_plain_text,
            status: row.status,
            delivered_at: row.delivered_at,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Serialize)]
pub struct NoteBacklinkDto {
    object: &'static str,
    id: String,
    source_page: NotePageDto,
    source_block_id: String,
    source_block_type: String,
    reference_type: String,
    snippet: String,
    created_time: String,
    last_edited_time: String,
}

pub(in crate::notes) struct NoteBacklinkIndexedInput {
    pub(in crate::notes) source_page: NotePageDto,
    pub(in crate::notes) id: String,
    pub(in crate::notes) source_block_id: String,
    pub(in crate::notes) source_block_type: String,
    pub(in crate::notes) reference_type: String,
    pub(in crate::notes) snippet: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}

impl NoteBacklinkDto {
    pub(in crate::notes) fn indexed(input: NoteBacklinkIndexedInput) -> Self {
        Self {
            object: "backlink",
            id: input.id,
            source_page: input.source_page,
            source_block_id: input.source_block_id,
            source_block_type: input.source_block_type,
            reference_type: input.reference_type,
            snippet: input.snippet,
            created_time: input.created_time,
            last_edited_time: input.last_edited_time,
        }
    }
}

#[derive(Deserialize)]
pub struct NotePageAliasCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) alias: String,
}

#[derive(Serialize)]
pub struct NotePageAliasDto {
    object: &'static str,
    id: String,
    page_id: String,
    alias: String,
    normalized_alias: String,
    created_time: String,
    last_edited_time: String,
}

impl NotePageAliasDto {
    pub(in crate::notes) fn new(row: NotePageAliasRow) -> Self {
        Self {
            object: "page_alias",
            id: row.id,
            page_id: row.page_id,
            alias: row.alias,
            normalized_alias: row.normalized_alias,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Serialize)]
pub struct NoteUnresolvedLinkDto {
    object: &'static str,
    id: String,
    source_type: String,
    source_page_id: String,
    source_block_id: Option<String>,
    source_comment_id: Option<String>,
    raw_url: String,
    raw_target: String,
    normalized_target: String,
    link_text: String,
    snippet: String,
    created_time: String,
    last_edited_time: String,
}

impl NoteUnresolvedLinkDto {
    pub(in crate::notes) fn new(row: NoteUnresolvedLinkRow) -> Self {
        Self {
            object: "unresolved_link",
            id: row.id,
            source_type: row.source_type,
            source_page_id: row.source_page_id,
            source_block_id: row.source_block_id,
            source_comment_id: row.source_comment_id,
            raw_url: row.raw_url,
            raw_target: row.raw_target,
            normalized_target: row.normalized_target,
            link_text: row.link_text,
            snippet: row.snippet,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Deserialize)]
pub struct NoteUnresolvedLinkResolve {
    pub(in crate::notes) target_page_id: String,
}

#[derive(Serialize)]
pub struct NoteSearchResultDto {
    object: &'static str,
    id: String,
    #[serde(rename = "type")]
    result_type: String,
    page: NotePageSummaryDto,
    block_id: Option<String>,
    block_type: Option<String>,
    comment_id: Option<String>,
    discussion_id: Option<String>,
    comment_status: Option<String>,
    comment_author: Option<Value>,
    comment_anchor: Option<NoteCommentAnchorDto>,
    snippet: String,
    last_edited_time: String,
}

impl NoteSearchResultDto {
    pub(in crate::notes) fn page(
        page: NotePageSummaryDto,
        snippet: String,
        last_edited_time: String,
    ) -> Self {
        let id = format!("page:{}", page.id);
        Self {
            object: "search_result",
            id,
            result_type: "page".to_string(),
            page,
            block_id: None,
            block_type: None,
            comment_id: None,
            discussion_id: None,
            comment_status: None,
            comment_author: None,
            comment_anchor: None,
            snippet,
            last_edited_time,
        }
    }

    pub(in crate::notes) fn block(
        page: NotePageSummaryDto,
        block: NoteBlockRow,
        snippet: String,
    ) -> Self {
        Self {
            object: "search_result",
            id: format!("block:{}", block.id),
            result_type: "block".to_string(),
            page,
            block_id: Some(block.id),
            block_type: Some(block.block_type),
            comment_id: None,
            discussion_id: None,
            comment_status: None,
            comment_author: None,
            comment_anchor: None,
            snippet,
            last_edited_time: block.last_edited_time,
        }
    }

    pub(in crate::notes) fn comment(
        page: NotePageSummaryDto,
        comment: NoteCommentRow,
        block_id: Option<String>,
        status: String,
        anchor: Option<NoteCommentAnchorRow>,
        snippet: String,
    ) -> Result<Self, String> {
        let display_name = parse_json(comment.display_name.clone(), "comment display name")?;
        Ok(Self {
            object: "search_result",
            id: format!("comment:{}", comment.id),
            result_type: "comment".to_string(),
            page,
            block_id,
            block_type: None,
            comment_id: Some(comment.id),
            discussion_id: Some(comment.thread_id),
            comment_status: Some(status),
            comment_author: Some(display_name),
            comment_anchor: anchor.map(NoteCommentAnchorDto::new),
            snippet,
            last_edited_time: comment.last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NoteSearchWindowDto {
    results: Vec<NoteSearchResultDto>,
    next_cursor: Option<String>,
}

impl NoteSearchWindowDto {
    pub(in crate::notes) fn new(
        results: Vec<NoteSearchResultDto>,
        next_cursor: Option<String>,
    ) -> Self {
        Self {
            results,
            next_cursor,
        }
    }

    #[cfg(test)]
    pub(in crate::notes) fn into_results(self) -> Vec<NoteSearchResultDto> {
        self.results
    }
}

#[derive(Serialize)]
pub struct NotePartialUserDto {
    object: &'static str,
    id: String,
}

impl NotePartialUserDto {
    pub(in crate::notes) fn new(id: String) -> Self {
        Self { object: "user", id }
    }
}

#[derive(Serialize)]
pub struct NoteLocalUserDto {
    object: &'static str,
    id: String,
    display_name: String,
    created_time: String,
    last_edited_time: String,
}

impl NoteLocalUserDto {
    pub(in crate::notes) fn new(row: NoteLocalUserRow) -> Self {
        Self {
            object: "user",
            id: row.id,
            display_name: row.display_name,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Serialize)]
pub struct NoteCommentDto {
    object: &'static str,
    id: String,
    parent: NoteParent,
    discussion_id: String,
    created_time: String,
    last_edited_time: String,
    created_by: NotePartialUserDto,
    rich_text: Value,
    attachments: Value,
    display_name: Value,
    deleted_at: Option<String>,
}

impl NoteCommentDto {
    pub(in crate::notes) fn new(
        thread: &NoteCommentThreadRow,
        row: NoteCommentRow,
    ) -> Result<Self, String> {
        Ok(Self {
            object: "comment",
            id: row.id,
            parent: parent_from_row(
                &thread.parent_type,
                thread.parent_page_id.clone(),
                thread.parent_block_id.clone(),
                None,
            )?,
            discussion_id: row.thread_id,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
            created_by: NotePartialUserDto::new(row.created_by),
            rich_text: parse_json(row.rich_text, "comment rich_text")?,
            attachments: parse_json(row.attachments, "comment attachments")?,
            display_name: parse_json(row.display_name, "comment display name")?,
            deleted_at: row.deleted_at,
        })
    }
}

#[derive(Serialize)]
pub struct NoteCommentAnchorDto {
    object: &'static str,
    #[serde(rename = "type")]
    anchor_type: &'static str,
    block_id: String,
    start: i64,
    end: i64,
    text: String,
    prefix: String,
    suffix: String,
    created_time: String,
    last_edited_time: String,
}

impl NoteCommentAnchorDto {
    pub(in crate::notes) fn new(row: NoteCommentAnchorRow) -> Self {
        Self {
            object: "comment_anchor",
            anchor_type: "text_range",
            block_id: row.block_id,
            start: row.start_offset,
            end: row.end_offset,
            text: row.anchor_text,
            prefix: row.prefix_text,
            suffix: row.suffix_text,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        }
    }
}

#[derive(Serialize)]
pub struct NoteCommentThreadDto {
    object: &'static str,
    id: String,
    parent: NoteParent,
    page_id: String,
    block_id: Option<String>,
    status: String,
    resolved_at: Option<String>,
    resolved_by: Option<NotePartialUserDto>,
    anchor: Option<NoteCommentAnchorDto>,
    created_time: String,
    last_edited_time: String,
    unread: bool,
    comments: Vec<NoteCommentDto>,
}

impl NoteCommentThreadDto {
    pub(in crate::notes) fn new(
        row: NoteCommentThreadRow,
        comments: Vec<NoteCommentDto>,
        anchor: Option<NoteCommentAnchorDto>,
        unread: bool,
    ) -> Result<Self, String> {
        let resolved_by = row.resolved_by.clone().map(NotePartialUserDto::new);
        Ok(Self {
            object: "comment_thread",
            id: row.id,
            parent: parent_from_row(
                &row.parent_type,
                row.parent_page_id.clone(),
                row.parent_block_id.clone(),
                None,
            )?,
            page_id: row.page_id,
            block_id: row.parent_block_id,
            status: row.status,
            resolved_at: row.resolved_at,
            resolved_by,
            anchor,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
            unread,
            comments,
        })
    }
}

#[derive(Serialize)]
pub struct NoteSuggestionDto {
    object: &'static str,
    id: String,
    page_id: String,
    block_id: String,
    created_by: NotePartialUserDto,
    display_name: Value,
    status: String,
    range_start: i64,
    range_end: i64,
    original_text: String,
    proposed_text: String,
    prefix: String,
    suffix: String,
    accepted_at: Option<String>,
    accepted_by: Option<NotePartialUserDto>,
    rejected_at: Option<String>,
    rejected_by: Option<NotePartialUserDto>,
    created_time: String,
    last_edited_time: String,
}

impl NoteSuggestionDto {
    pub(in crate::notes) fn new(row: NoteSuggestionRow) -> Result<Self, String> {
        Ok(Self {
            object: "suggestion",
            id: row.id,
            page_id: row.page_id,
            block_id: row.block_id,
            created_by: NotePartialUserDto::new(row.created_by),
            display_name: parse_json(row.display_name, "suggestion display name")?,
            status: row.status,
            range_start: row.range_start,
            range_end: row.range_end,
            original_text: row.original_text,
            proposed_text: row.proposed_text,
            prefix: row.prefix_text,
            suffix: row.suffix_text,
            accepted_at: row.accepted_at,
            accepted_by: row.accepted_by.map(NotePartialUserDto::new),
            rejected_at: row.rejected_at,
            rejected_by: row.rejected_by.map(NotePartialUserDto::new),
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Deserialize)]
pub struct NoteSuggestionCreate {
    pub(in crate::notes) id: String,
    pub(in crate::notes) block_id: String,
    pub(in crate::notes) range_start: i64,
    pub(in crate::notes) range_end: i64,
    pub(in crate::notes) original_text: String,
    pub(in crate::notes) proposed_text: String,
    pub(in crate::notes) prefix: String,
    pub(in crate::notes) suffix: String,
}

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
        NoteBlockPayloadRefs::from_write(self).get(&self.block_type)
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
        NoteBlockPayloadRefs::from_update(self).get(block_type)
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

#[derive(Serialize)]
pub struct NoteCreatedDatabaseDto {
    database: NoteDatabaseDto,
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    block: NoteBlockDto,
}

impl NoteCreatedDatabaseDto {
    pub(in crate::notes) fn new(
        database: NoteDatabaseRow,
        data_source: NoteDataSourceRow,
        view: NoteDatabaseViewRow,
        block: NoteBlockRow,
    ) -> Result<Self, String> {
        let database_parent = block_parent_from_database_row(&database)?;
        Ok(Self {
            database: NoteDatabaseDto::new(database, vec![data_source.summary()])?,
            data_source: NoteDataSourceDto::new(data_source, database_parent)?,
            view: NoteDatabaseViewDto::new(view)?,
            block: NoteBlockDto::new(block)?,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceSchemaDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
}

impl NoteDataSourceSchemaDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceTableViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    rows: Vec<NotePageDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
}

impl NoteDataSourceTableViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            rows: window
                .rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceBoardGroupDto {
    id: String,
    name: String,
    color: String,
    hidden: bool,
    rows: Vec<NotePageDto>,
}

impl NoteDataSourceBoardGroupDto {
    pub(in crate::notes) fn new(
        id: String,
        name: String,
        color: String,
        hidden: bool,
        rows: Vec<NotePageRow>,
    ) -> Result<Self, String> {
        Ok(Self {
            id,
            name,
            color,
            hidden,
            rows: rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceBoardViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    groups: Vec<NoteDataSourceBoardGroupDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
    group_counts: std::collections::HashMap<String, i64>,
}

impl NoteDataSourceBoardViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        groups: Vec<NoteDataSourceBoardGroupDto>,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            groups,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
            group_counts: window.group_counts,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceGalleryViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    rows: Vec<NotePageDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
}

impl NoteDataSourceGalleryViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            rows: window
                .rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceListViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    rows: Vec<NotePageDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
    group_counts: std::collections::HashMap<String, i64>,
}

impl NoteDataSourceListViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            rows: window
                .rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
            group_counts: window.group_counts,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceCalendarViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    rows: Vec<NotePageDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
}

impl NoteDataSourceCalendarViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            rows: window
                .rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceTimelineViewDto {
    data_source: NoteDataSourceDto,
    view: NoteDatabaseViewDto,
    rows: Vec<NotePageDto>,
    total_row_count: i64,
    next_cursor: Option<String>,
    has_more: bool,
    group_counts: std::collections::HashMap<String, i64>,
}

impl NoteDataSourceTimelineViewDto {
    pub(in crate::notes) fn new(
        data_source: NoteDataSourceRow,
        database: NoteDatabaseRow,
        view: NoteDatabaseViewRow,
        window: NoteDataSourceRowWindow,
    ) -> Result<Self, String> {
        Ok(Self {
            data_source: NoteDataSourceDto::new(
                data_source,
                block_parent_from_database_row(&database)?,
            )?,
            view: NoteDatabaseViewDto::new(view)?,
            rows: window
                .rows
                .into_iter()
                .map(NotePageDto::new)
                .collect::<Result<Vec<_>, _>>()?,
            total_row_count: window.total_row_count,
            next_cursor: window.next_cursor,
            has_more: window.has_more,
            group_counts: window.group_counts,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDatabaseDataSourceSummaryDto {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub struct NoteDatabaseDto {
    object: &'static str,
    id: String,
    parent: NoteParent,
    title: String,
    title_rich_text: Value,
    description: Value,
    icon: Option<Value>,
    cover: Option<Value>,
    in_trash: bool,
    is_inline: bool,
    data_sources: Vec<NoteDatabaseDataSourceSummaryDto>,
    url: Option<String>,
    public_url: Option<String>,
    source_provider: Option<String>,
    source_object_id: Option<String>,
    source_workspace_id: Option<String>,
    source_last_edited_time: Option<String>,
    created_time: String,
    last_edited_time: String,
}

impl NoteDatabaseDto {
    pub(in crate::notes) fn new(
        row: NoteDatabaseRow,
        data_sources: Vec<NoteDatabaseDataSourceSummaryDto>,
    ) -> Result<Self, String> {
        let parent = block_parent_from_database_row(&row)?;
        Ok(Self {
            object: "database",
            id: row.id,
            parent,
            title: row.title,
            title_rich_text: parse_json(row.title_rich_text, "database title rich text")?,
            description: parse_json(row.description, "database description")?,
            icon: parse_optional_json(row.icon, "database icon")?,
            cover: parse_optional_json(row.cover, "database cover")?,
            in_trash: row.in_trash != 0,
            is_inline: row.is_inline != 0,
            data_sources,
            url: row.url,
            public_url: row.public_url,
            source_provider: row.source_provider,
            source_object_id: row.source_object_id,
            source_workspace_id: row.source_workspace_id,
            source_last_edited_time: row.source_last_edited_time,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDataSourceParentDto {
    #[serde(rename = "type")]
    parent_type: &'static str,
    database_id: String,
}

#[derive(Serialize)]
pub struct NoteDataSourceDto {
    object: &'static str,
    id: String,
    parent: NoteDataSourceParentDto,
    database_parent: NoteParent,
    title: String,
    title_rich_text: Value,
    description: Value,
    icon: Option<Value>,
    properties: Value,
    in_trash: bool,
    source_provider: Option<String>,
    source_object_id: Option<String>,
    source_workspace_id: Option<String>,
    source_last_edited_time: Option<String>,
    created_time: String,
    last_edited_time: String,
}

impl NoteDataSourceDto {
    pub(in crate::notes) fn new(
        row: NoteDataSourceRow,
        database_parent: NoteParent,
    ) -> Result<Self, String> {
        Ok(Self {
            object: "data_source",
            id: row.id,
            parent: NoteDataSourceParentDto {
                parent_type: "database_id",
                database_id: row.database_id,
            },
            database_parent,
            title: row.title,
            title_rich_text: parse_json(row.title_rich_text, "data source title rich text")?,
            description: parse_json(row.description, "data source description")?,
            icon: parse_optional_json(row.icon, "data source icon")?,
            properties: parse_json(row.properties, "data source properties")?,
            in_trash: row.in_trash != 0,
            source_provider: row.source_provider,
            source_object_id: row.source_object_id,
            source_workspace_id: row.source_workspace_id,
            source_last_edited_time: row.source_last_edited_time,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Serialize)]
pub struct NoteDatabaseViewParentDto {
    #[serde(rename = "type")]
    parent_type: &'static str,
    database_id: String,
}

#[derive(Serialize)]
pub struct NoteDatabaseViewDto {
    object: &'static str,
    id: String,
    parent: NoteDatabaseViewParentDto,
    data_source_id: String,
    name: String,
    #[serde(rename = "type")]
    view_type: String,
    filter: Option<Value>,
    sorts: Value,
    configuration: Option<Value>,
    url: Option<String>,
    source_provider: Option<String>,
    source_object_id: Option<String>,
    source_workspace_id: Option<String>,
    source_last_edited_time: Option<String>,
    created_time: String,
    last_edited_time: String,
}

impl NoteDatabaseViewDto {
    pub(in crate::notes) fn new(row: NoteDatabaseViewRow) -> Result<Self, String> {
        Ok(Self {
            object: "view",
            id: row.id,
            parent: NoteDatabaseViewParentDto {
                parent_type: "database_id",
                database_id: row.database_id,
            },
            data_source_id: row.data_source_id,
            name: row.name,
            view_type: row.view_type,
            filter: parse_optional_json(row.filter, "database view filter")?,
            sorts: parse_json(row.sorts, "database view sorts")?,
            configuration: parse_optional_json(row.configuration, "database view configuration")?,
            url: row.url,
            source_provider: row.source_provider,
            source_object_id: row.source_object_id,
            source_workspace_id: row.source_workspace_id,
            source_last_edited_time: row.source_last_edited_time,
            created_time: row.created_time,
            last_edited_time: row.last_edited_time,
        })
    }
}

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteDatabaseRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) title: String,
    pub(in crate::notes) title_rich_text: String,
    pub(in crate::notes) description: String,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) cover: Option<String>,
    pub(in crate::notes) is_inline: i64,
    pub(in crate::notes) in_trash: i64,
    pub(in crate::notes) source_provider: Option<String>,
    pub(in crate::notes) source_object_id: Option<String>,
    pub(in crate::notes) source_workspace_id: Option<String>,
    pub(in crate::notes) source_last_edited_time: Option<String>,
    pub(in crate::notes) url: Option<String>,
    pub(in crate::notes) public_url: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteDatabaseRow {
    id,
    parent_type,
    parent_page_id,
    parent_block_id,
    title,
    title_rich_text,
    description,
    icon,
    cover,
    is_inline,
    in_trash,
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    url,
    public_url,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteDataSourceRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) database_id: String,
    pub(in crate::notes) title: String,
    pub(in crate::notes) title_rich_text: String,
    pub(in crate::notes) description: String,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) properties: String,
    pub(in crate::notes) in_trash: i64,
    pub(in crate::notes) source_provider: Option<String>,
    pub(in crate::notes) source_object_id: Option<String>,
    pub(in crate::notes) source_workspace_id: Option<String>,
    pub(in crate::notes) source_last_edited_time: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteDataSourceRow {
    id,
    database_id,
    title,
    title_rich_text,
    description,
    icon,
    properties,
    in_trash,
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    created_time,
    last_edited_time,
});

impl NoteDataSourceRow {
    pub(in crate::notes) fn summary(&self) -> NoteDatabaseDataSourceSummaryDto {
        NoteDatabaseDataSourceSummaryDto {
            id: self.id.clone(),
            name: self.title.clone(),
        }
    }
}

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteDatabaseViewRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) database_id: String,
    pub(in crate::notes) data_source_id: String,
    pub(in crate::notes) name: String,
    pub(in crate::notes) view_type: String,
    pub(in crate::notes) filter: Option<String>,
    pub(in crate::notes) sorts: String,
    pub(in crate::notes) configuration: Option<String>,
    pub(in crate::notes) source_provider: Option<String>,
    pub(in crate::notes) source_object_id: Option<String>,
    pub(in crate::notes) source_workspace_id: Option<String>,
    pub(in crate::notes) source_last_edited_time: Option<String>,
    pub(in crate::notes) url: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteDatabaseViewRow {
    id,
    database_id,
    data_source_id,
    name,
    view_type,
    filter,
    sorts,
    configuration,
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    url,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NotePageRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) parent_data_source_id: Option<String>,
    pub(in crate::notes) folder_id: Option<String>,
    pub(in crate::notes) title: String,
    pub(in crate::notes) properties: String,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) cover: Option<String>,
    pub(in crate::notes) in_trash: i64,
    pub(in crate::notes) archived: i64,
    pub(in crate::notes) source_provider: Option<String>,
    pub(in crate::notes) source_object_id: Option<String>,
    pub(in crate::notes) source_workspace_id: Option<String>,
    pub(in crate::notes) source_last_edited_time: Option<String>,
    pub(in crate::notes) url: Option<String>,
    pub(in crate::notes) public_url: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NotePageRow {
    id,
    parent_type,
    parent_page_id,
    parent_block_id,
    parent_data_source_id,
    folder_id,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    source_provider,
    source_object_id,
    source_workspace_id,
    source_last_edited_time,
    url,
    public_url,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteFolderRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) project_id: String,
    pub(in crate::notes) parent_folder_id: Option<String>,
    pub(in crate::notes) name: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteFolderRow {
    id,
    project_id,
    parent_folder_id,
    name,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NotePageTemplateRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) name: String,
    pub(in crate::notes) source_page_id: Option<String>,
    pub(in crate::notes) properties: String,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) cover: Option<String>,
    pub(in crate::notes) block_count: i64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NotePageTemplateRow {
    id,
    name,
    source_page_id,
    properties,
    icon,
    cover,
    block_count,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteDataSourceTemplateRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) data_source_id: String,
    pub(in crate::notes) source_page_id: Option<String>,
    pub(in crate::notes) name: String,
    pub(in crate::notes) properties: String,
    pub(in crate::notes) is_default: i64,
    pub(in crate::notes) block_count: i64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteDataSourceTemplateRow {
    id,
    data_source_id,
    source_page_id,
    name,
    properties,
    is_default,
    block_count,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NotePageHistorySnapshotRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) parent_data_source_id: Option<String>,
    pub(in crate::notes) folder_id: Option<String>,
    pub(in crate::notes) title: String,
    pub(in crate::notes) properties: String,
    pub(in crate::notes) icon: Option<String>,
    pub(in crate::notes) cover: Option<String>,
    pub(in crate::notes) in_trash: i64,
    pub(in crate::notes) archived: i64,
    pub(in crate::notes) blocks: String,
    pub(in crate::notes) block_bundle_hash: Option<String>,
    pub(in crate::notes) block_count: i64,
    pub(in crate::notes) reason: String,
    pub(in crate::notes) created_by: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) page_created_time: String,
    pub(in crate::notes) page_last_edited_time: String,
}
impl_sqlite_from_row!(NotePageHistorySnapshotRow {
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    parent_data_source_id,
    folder_id,
    title,
    properties,
    icon,
    cover,
    in_trash,
    archived,
    blocks,
    block_bundle_hash,
    block_count,
    reason,
    created_by,
    created_time,
    page_created_time,
    page_last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteLocalUserRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) display_name: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteLocalUserRow {
    id,
    display_name,
    created_time,
    last_edited_time,
});

#[derive(Serialize)]
pub(in crate::notes) struct NotePageHistorySettingsRow {
    pub(in crate::notes) retention_days: Option<i64>,
    pub(in crate::notes) updated_at: String,
}
impl_sqlite_from_row!(NotePageHistorySettingsRow {
    retention_days,
    updated_at,
});

#[derive(Serialize)]
pub(in crate::notes) struct NoteMentionNotificationRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) source_type: String,
    pub(in crate::notes) source_id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) page_title: String,
    pub(in crate::notes) block_id: Option<String>,
    pub(in crate::notes) comment_id: Option<String>,
    pub(in crate::notes) kind: String,
    pub(in crate::notes) target_type: String,
    pub(in crate::notes) target_id: Option<String>,
    pub(in crate::notes) trigger_at: Option<String>,
    pub(in crate::notes) plain_text: String,
    pub(in crate::notes) source_plain_text: String,
    pub(in crate::notes) status: String,
    pub(in crate::notes) delivered_at: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteMentionNotificationRow {
    id,
    source_type,
    source_id,
    page_id,
    page_title,
    block_id,
    comment_id,
    kind,
    target_type,
    target_id,
    trigger_at,
    plain_text,
    source_plain_text,
    status,
    delivered_at,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NotePageAliasRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) alias: String,
    pub(in crate::notes) normalized_alias: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NotePageAliasRow {
    id,
    page_id,
    alias,
    normalized_alias,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteUnresolvedLinkRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) source_type: String,
    pub(in crate::notes) source_page_id: String,
    pub(in crate::notes) source_block_id: Option<String>,
    pub(in crate::notes) source_comment_id: Option<String>,
    pub(in crate::notes) raw_url: String,
    pub(in crate::notes) raw_target: String,
    pub(in crate::notes) normalized_target: String,
    pub(in crate::notes) link_text: String,
    pub(in crate::notes) snippet: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteUnresolvedLinkRow {
    id,
    source_type,
    source_page_id,
    source_block_id,
    source_comment_id,
    raw_url,
    raw_target,
    normalized_target,
    link_text,
    snippet,
    created_time,
    last_edited_time,
});

#[derive(Clone, Deserialize, Serialize)]
pub(in crate::notes) struct NoteBlockRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) has_children: i64,
    pub(in crate::notes) in_trash: i64,
    #[serde(rename = "type")]
    pub(in crate::notes) block_type: String,
    pub(in crate::notes) payload: String,
    pub(in crate::notes) plain_text: String,
    pub(in crate::notes) sort_order: f64,
    pub(in crate::notes) source_provider: Option<String>,
    pub(in crate::notes) source_object_id: Option<String>,
    pub(in crate::notes) source_last_edited_time: Option<String>,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteBlockRow {
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    has_children,
    in_trash,
    block_type,
    payload,
    plain_text,
    sort_order,
    source_provider,
    source_object_id,
    source_last_edited_time,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NotePageTemplateBlockRow {
    pub(in crate::notes) template_id: String,
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) has_children: i64,
    pub(in crate::notes) block_type: String,
    pub(in crate::notes) payload: String,
    pub(in crate::notes) plain_text: String,
    pub(in crate::notes) sort_order: f64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NotePageTemplateBlockRow {
    template_id,
    id,
    parent_type,
    parent_block_id,
    has_children,
    block_type,
    payload,
    plain_text,
    sort_order,
    created_time,
    last_edited_time,
});

#[derive(Clone, Serialize)]
pub(in crate::notes) struct NoteDataSourceTemplateBlockRow {
    pub(in crate::notes) template_id: String,
    pub(in crate::notes) id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) has_children: i64,
    pub(in crate::notes) block_type: String,
    pub(in crate::notes) payload: String,
    pub(in crate::notes) plain_text: String,
    pub(in crate::notes) sort_order: f64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteDataSourceTemplateBlockRow {
    template_id,
    id,
    parent_type,
    parent_block_id,
    has_children,
    block_type,
    payload,
    plain_text,
    sort_order,
    created_time,
    last_edited_time,
});

#[derive(Serialize)]
pub(in crate::notes) struct NoteCommentThreadRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) parent_type: String,
    pub(in crate::notes) parent_page_id: Option<String>,
    pub(in crate::notes) parent_block_id: Option<String>,
    pub(in crate::notes) status: String,
    pub(in crate::notes) resolved_at: Option<String>,
    pub(in crate::notes) resolved_by: Option<String>,
    pub(in crate::notes) sync_version: i64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteCommentThreadRow {
    id,
    page_id,
    parent_type,
    parent_page_id,
    parent_block_id,
    status,
    resolved_at,
    resolved_by,
    sync_version,
    created_time,
    last_edited_time,
});

#[derive(Serialize)]
pub(in crate::notes) struct NoteCommentAnchorRow {
    pub(in crate::notes) thread_id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) block_id: String,
    pub(in crate::notes) start_offset: i64,
    pub(in crate::notes) end_offset: i64,
    pub(in crate::notes) anchor_text: String,
    pub(in crate::notes) prefix_text: String,
    pub(in crate::notes) suffix_text: String,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteCommentAnchorRow {
    thread_id,
    page_id,
    block_id,
    start_offset,
    end_offset,
    anchor_text,
    prefix_text,
    suffix_text,
    created_time,
    last_edited_time,
});

#[derive(Serialize)]
pub(in crate::notes) struct NoteCommentRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) thread_id: String,
    pub(in crate::notes) rich_text: String,
    pub(in crate::notes) plain_text: String,
    pub(in crate::notes) created_by: String,
    pub(in crate::notes) display_name: String,
    pub(in crate::notes) attachments: String,
    pub(in crate::notes) deleted_at: Option<String>,
    pub(in crate::notes) sync_version: i64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteCommentRow {
    id,
    thread_id,
    rich_text,
    plain_text,
    created_by,
    display_name,
    attachments,
    deleted_at,
    sync_version,
    created_time,
    last_edited_time,
});

#[derive(Serialize)]
pub(in crate::notes) struct NoteSuggestionRow {
    pub(in crate::notes) id: String,
    pub(in crate::notes) page_id: String,
    pub(in crate::notes) block_id: String,
    pub(in crate::notes) created_by: String,
    pub(in crate::notes) display_name: String,
    pub(in crate::notes) status: String,
    pub(in crate::notes) range_start: i64,
    pub(in crate::notes) range_end: i64,
    pub(in crate::notes) original_text: String,
    pub(in crate::notes) proposed_text: String,
    pub(in crate::notes) prefix_text: String,
    pub(in crate::notes) suffix_text: String,
    pub(in crate::notes) accepted_at: Option<String>,
    pub(in crate::notes) accepted_by: Option<String>,
    pub(in crate::notes) rejected_at: Option<String>,
    pub(in crate::notes) rejected_by: Option<String>,
    pub(in crate::notes) sync_version: i64,
    pub(in crate::notes) created_time: String,
    pub(in crate::notes) last_edited_time: String,
}
impl_sqlite_from_row!(NoteSuggestionRow {
    id,
    page_id,
    block_id,
    created_by,
    display_name,
    status,
    range_start,
    range_end,
    original_text,
    proposed_text,
    prefix_text,
    suffix_text,
    accepted_at,
    accepted_by,
    rejected_at,
    rejected_by,
    sync_version,
    created_time,
    last_edited_time,
});

struct NoteBlockPayloadRefs<'a> {
    paragraph: Option<&'a Value>,
    heading_1: Option<&'a Value>,
    heading_2: Option<&'a Value>,
    heading_3: Option<&'a Value>,
    heading_4: Option<&'a Value>,
    bulleted_list_item: Option<&'a Value>,
    numbered_list_item: Option<&'a Value>,
    to_do: Option<&'a Value>,
    toggle: Option<&'a Value>,
    callout: Option<&'a Value>,
    quote: Option<&'a Value>,
    child_page: Option<&'a Value>,
    child_database: Option<&'a Value>,
    breadcrumb: Option<&'a Value>,
    table_of_contents: Option<&'a Value>,
    column_list: Option<&'a Value>,
    column: Option<&'a Value>,
    table: Option<&'a Value>,
    table_row: Option<&'a Value>,
    tab: Option<&'a Value>,
    image: Option<&'a Value>,
    video: Option<&'a Value>,
    audio: Option<&'a Value>,
    file: Option<&'a Value>,
    pdf: Option<&'a Value>,
    bookmark: Option<&'a Value>,
    link_preview: Option<&'a Value>,
    synced_block: Option<&'a Value>,
    template: Option<&'a Value>,
    button: Option<&'a Value>,
    embed: Option<&'a Value>,
    equation: Option<&'a Value>,
    divider: Option<&'a Value>,
    code: Option<&'a Value>,
    unsupported: Option<&'a Value>,
}

impl<'a> NoteBlockPayloadRefs<'a> {
    fn from_write(block: &'a NoteBlockWrite) -> Self {
        Self {
            paragraph: block.paragraph.as_ref(),
            heading_1: block.heading_1.as_ref(),
            heading_2: block.heading_2.as_ref(),
            heading_3: block.heading_3.as_ref(),
            heading_4: block.heading_4.as_ref(),
            bulleted_list_item: block.bulleted_list_item.as_ref(),
            numbered_list_item: block.numbered_list_item.as_ref(),
            to_do: block.to_do.as_ref(),
            toggle: block.toggle.as_ref(),
            callout: block.callout.as_ref(),
            quote: block.quote.as_ref(),
            child_page: block.child_page.as_ref(),
            child_database: block.child_database.as_ref(),
            breadcrumb: block.breadcrumb.as_ref(),
            table_of_contents: block.table_of_contents.as_ref(),
            column_list: block.column_list.as_ref(),
            column: block.column.as_ref(),
            table: block.table.as_ref(),
            table_row: block.table_row.as_ref(),
            tab: block.tab.as_ref(),
            image: block.image.as_ref(),
            video: block.video.as_ref(),
            audio: block.audio.as_ref(),
            file: block.file.as_ref(),
            pdf: block.pdf.as_ref(),
            bookmark: block.bookmark.as_ref(),
            link_preview: block.link_preview.as_ref(),
            synced_block: block.synced_block.as_ref(),
            template: block.template.as_ref(),
            button: block.button.as_ref(),
            embed: block.embed.as_ref(),
            equation: block.equation.as_ref(),
            divider: block.divider.as_ref(),
            code: block.code.as_ref(),
            unsupported: block.unsupported.as_ref(),
        }
    }

    fn from_update(update: &'a NoteBlockUpdate) -> Self {
        Self {
            paragraph: update.paragraph.as_ref(),
            heading_1: update.heading_1.as_ref(),
            heading_2: update.heading_2.as_ref(),
            heading_3: update.heading_3.as_ref(),
            heading_4: update.heading_4.as_ref(),
            bulleted_list_item: update.bulleted_list_item.as_ref(),
            numbered_list_item: update.numbered_list_item.as_ref(),
            to_do: update.to_do.as_ref(),
            toggle: update.toggle.as_ref(),
            callout: update.callout.as_ref(),
            quote: update.quote.as_ref(),
            child_page: update.child_page.as_ref(),
            child_database: update.child_database.as_ref(),
            breadcrumb: update.breadcrumb.as_ref(),
            table_of_contents: update.table_of_contents.as_ref(),
            column_list: update.column_list.as_ref(),
            column: update.column.as_ref(),
            table: update.table.as_ref(),
            table_row: update.table_row.as_ref(),
            tab: update.tab.as_ref(),
            image: update.image.as_ref(),
            video: update.video.as_ref(),
            audio: update.audio.as_ref(),
            file: update.file.as_ref(),
            pdf: update.pdf.as_ref(),
            bookmark: update.bookmark.as_ref(),
            link_preview: update.link_preview.as_ref(),
            synced_block: update.synced_block.as_ref(),
            template: update.template.as_ref(),
            button: update.button.as_ref(),
            embed: update.embed.as_ref(),
            equation: update.equation.as_ref(),
            divider: update.divider.as_ref(),
            code: update.code.as_ref(),
            unsupported: update.unsupported.as_ref(),
        }
    }

    fn get(&self, block_type: &str) -> Option<&'a Value> {
        match block_type {
            "paragraph" => self.paragraph,
            "heading_1" => self.heading_1,
            "heading_2" => self.heading_2,
            "heading_3" => self.heading_3,
            "heading_4" => self.heading_4,
            "bulleted_list_item" => self.bulleted_list_item,
            "numbered_list_item" => self.numbered_list_item,
            "to_do" => self.to_do,
            "toggle" => self.toggle,
            "callout" => self.callout,
            "quote" => self.quote,
            "child_page" => self.child_page,
            "child_database" => self.child_database,
            "breadcrumb" => self.breadcrumb,
            "table_of_contents" => self.table_of_contents,
            "column_list" => self.column_list,
            "column" => self.column,
            "table" => self.table,
            "table_row" => self.table_row,
            "tab" => self.tab,
            "image" => self.image,
            "video" => self.video,
            "audio" => self.audio,
            "file" => self.file,
            "pdf" => self.pdf,
            "bookmark" => self.bookmark,
            "link_preview" => self.link_preview,
            "synced_block" => self.synced_block,
            "template" => self.template,
            "button" => self.button,
            "embed" => self.embed,
            "equation" => self.equation,
            "divider" => self.divider,
            "code" => self.code,
            "unsupported" => self.unsupported,
            _ => None,
        }
    }
}

pub(in crate::notes) fn parent_columns(
    parent: &NoteParent,
) -> (&'static str, Option<&str>, Option<&str>) {
    match parent {
        NoteParent::Workspace { .. } => ("workspace", None, None),
        NoteParent::PageId { page_id } => ("page_id", Some(page_id.as_str()), None),
        NoteParent::BlockId { block_id } => ("block_id", None, Some(block_id.as_str())),
        NoteParent::DataSourceId { .. } => ("data_source_id", None, None),
    }
}

pub(in crate::notes) fn page_parent_columns(
    parent: &NoteParent,
) -> (&'static str, Option<&str>, Option<&str>, Option<&str>) {
    match parent {
        NoteParent::Workspace { .. } => ("workspace", None, None, None),
        NoteParent::PageId { page_id } => ("page_id", Some(page_id.as_str()), None, None),
        NoteParent::BlockId { block_id } => ("block_id", None, Some(block_id.as_str()), None),
        NoteParent::DataSourceId { data_source_id } => {
            ("data_source_id", None, None, Some(data_source_id.as_str()))
        }
    }
}

fn parent_from_row(
    parent_type: &str,
    parent_page_id: Option<String>,
    parent_block_id: Option<String>,
    parent_data_source_id: Option<String>,
) -> Result<NoteParent, String> {
    match parent_type {
        "workspace" => Ok(NoteParent::Workspace { workspace: true }),
        "page_id" => parent_page_id
            .map(|page_id| NoteParent::PageId { page_id })
            .ok_or_else(|| "stored page parent is missing page_id".to_string()),
        "block_id" => parent_block_id
            .map(|block_id| NoteParent::BlockId { block_id })
            .ok_or_else(|| "stored block parent is missing block_id".to_string()),
        "data_source_id" => parent_data_source_id
            .map(|data_source_id| NoteParent::DataSourceId { data_source_id })
            .ok_or_else(|| "stored page parent is missing data_source_id".to_string()),
        _ => Err(format!("unsupported parent type in storage: {parent_type}")),
    }
}

pub(in crate::notes) fn block_parent_from_database_row(
    row: &NoteDatabaseRow,
) -> Result<NoteParent, String> {
    parent_from_row(
        &row.parent_type,
        row.parent_page_id.clone(),
        row.parent_block_id.clone(),
        None,
    )
}

fn parse_json(value: String, label: &str) -> Result<Value, String> {
    serde_json::from_str(&value).map_err(|e| format!("parse {label}: {e}"))
}

fn parse_optional_json(value: Option<String>, label: &str) -> Result<Option<Value>, String> {
    value.map(|json| parse_json(json, label)).transpose()
}
