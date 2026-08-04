use super::*;

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
        NoteDatabaseDataSourceSummaryDto::new(self.id.clone(), self.title.clone())
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

pub(in crate::notes) fn parent_from_row(
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

pub(in crate::notes) fn parse_json(value: String, label: &str) -> Result<Value, String> {
    serde_json::from_str(&value).map_err(|e| format!("parse {label}: {e}"))
}

pub(in crate::notes) fn parse_optional_json(
    value: Option<String>,
    label: &str,
) -> Result<Option<Value>, String> {
    value.map(|json| parse_json(json, label)).transpose()
}
