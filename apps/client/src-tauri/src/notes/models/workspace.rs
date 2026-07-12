use super::*;

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
