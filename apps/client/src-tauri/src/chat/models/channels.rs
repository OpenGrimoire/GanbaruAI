use super::{
    ChatChannelId, ChatThreadShellRead, ChatTimelineItemRead, ChatTimelineTurnRead, ModelId,
    ModelOptionSelection, ProjectWorkingFolderId, ProviderInstanceId, UtcTimestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelTargetRead {
    pub working_folder_id: Option<ProjectWorkingFolderId>,
    pub provider_instance_id: Option<ProviderInstanceId>,
    pub provider_managed_model: bool,
    pub model_id: Option<ModelId>,
    pub model_options: Vec<ModelOptionSelection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelRead {
    pub id: ChatChannelId,
    pub project_id: String,
    pub name: String,
    pub topic: String,
    pub is_default: bool,
    pub target: ChatChannelTargetRead,
    pub current_thread: Option<ChatThreadShellRead>,
    pub session_count: u64,
    pub message_count: u64,
    pub latest_preview: Option<String>,
    pub last_activity_at: UtcTimestamp,
    pub unread_at: Option<UtcTimestamp>,
    pub revision: u64,
    pub archived_at: Option<UtcTimestamp>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelSessionRead {
    pub channel_id: ChatChannelId,
    pub ordinal: u64,
    pub is_current: bool,
    pub created_at: UtcTimestamp,
    pub thread: ChatThreadShellRead,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatChannelTimelinePageRead {
    pub channel_id: ChatChannelId,
    pub sessions: Vec<ChatChannelSessionRead>,
    pub items: Vec<ChatTimelineItemRead>,
    pub turns: Vec<ChatTimelineTurnRead>,
    pub previous_cursor: Option<String>,
    pub revision: u64,
}
